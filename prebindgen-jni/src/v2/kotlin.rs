//! The Kotlin side of a v2 generation.
//!
//! The engine renders Rust only; what Kotlin a binding needs is the JNI
//! adapter's to say, and it says it here from the payloads its declarations
//! carried through planning — a `data class` per emitted struct, a handle
//! class per emitted opaque type, one `external fun` per emitted function on
//! the harness object, and a public function calling each. Rendering goes
//! through `kotlin-codegen`, as v1's does, so the output is validated Kotlin
//! and lands in the same generator-owned tree.
//!
//! A handle crosses the wrapper boundary as a `Long`. The public function is
//! where it becomes a class: an address handed out is wrapped, and one taken
//! back is taken out of its wrapper, which forgets it — so the wrapper's
//! `free()` afterwards releases nothing, and a second use raises the binding
//! failure the Rust side reports for a null address.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use kotlin_codegen::{
    write_files, KtClass, KtCode, KtCompanion, KtCtorParam, KtDecl, KtEnumEntry, KtFile, KtFun,
    KtParam, KtProperty, KtType, KtVis, WriteKotlinError,
};
use prebindgen_registry_v2::Generation;

use super::target::{JniPayload, KotlinType};
use crate::jni::Declarations;

/// Write one file per package under `kotlin_root`, and return the paths.
pub(super) fn write(
    decls: &Declarations,
    generation: &Generation<JniPayload>,
    kotlin_root: &Path,
) -> Result<Vec<PathBuf>, WriteKotlinError> {
    let mut files: BTreeMap<String, KtFile> = BTreeMap::new();
    fn file<'f>(package: &str, files: &'f mut BTreeMap<String, KtFile>) -> &'f mut KtFile {
        files
            .entry(package.to_string())
            .or_insert_with(|| KtFile::new(package))
    }
    // Every native method lands on one harness object in the base package,
    // whichever package its Kotlin function is declared in: the `Java_…`
    // symbol names the harness, so the two have to agree.
    let harness = decls.jni_native_class_name();
    let harness_package = decls.package.clone();
    let harness_fqn = match harness_package.is_empty() {
        true => harness.clone(),
        false => format!("{harness_package}.{harness}"),
    };
    let mut natives: Vec<KtDecl> = Vec::new();

    for surface in generation.surfaces() {
        match &surface.payload {
            Some(JniPayload::Class {
                package,
                class,
                properties,
            }) => {
                let mut properties = properties.iter();
                let (name, ty) = properties
                    .next()
                    .expect("a class with no properties is refused before it is emitted");
                let mut declaration = KtClass::data(class, property(name, ty)).vis(KtVis::Public);
                for (name, ty) in properties {
                    declaration = declaration.ctor_param(property(name, ty));
                }
                file(package, &mut files).decls.push(declaration.into());
            }
            // A Kotlin `enum class` of the same values, each carrying the
            // number Rust assigns it, and a `fromInt` looking one up: that
            // number is what crosses, in both directions.
            Some(JniPayload::EnumClass {
                package,
                class,
                values,
                conditions,
            }) => {
                let mut declaration = KtClass::enum_(class)
                    .vis(KtVis::Public)
                    .ctor_param(KtCtorParam::new("value", KtType::int()).val());
                for (name, number) in values {
                    declaration =
                        declaration.entry(KtEnumEntry::with_args(name, number.to_string()));
                }
                // `@JvmStatic` so the companion's `fromInt` is a real static
                // method on the class, which is how a caller outside Kotlin
                // reaches it.
                declaration = declaration.companion(
                    KtCompanion::new().vis(KtVis::Public).member(
                        KtFun::new("fromInt")
                            .vis(KtVis::Public)
                            .annotation("JvmStatic")
                            .param(KtParam::new("value", KtType::int()))
                            .returns(KtType::cls(class))
                            .expr_body(KtCode::new().line("entries.first { it.value == value }")),
                    ),
                );
                if let Some(kdoc) = conditions_kdoc(conditions) {
                    declaration = declaration.kdoc(kdoc);
                }
                file(package, &mut files).decls.push(declaration.into());
            }
            Some(JniPayload::Method {
                package,
                method,
                native,
                params,
                ret,
                conditions,
            }) => {
                natives.push(native_method(native, params, ret));
                // The function a caller uses, delegating to it. The harness is
                // named in full only from another package.
                let harness = match *package == harness_package {
                    true => harness.as_str(),
                    false => harness_fqn.as_str(),
                };
                let mut public = KtFun::new(method).vis(KtVis::Public);
                for (name, ty) in params {
                    public = public.param(KtParam::new(name, KtType::cls(ty.public())));
                }
                public = public
                    .returns(KtType::cls(ret.public()))
                    .expr_body(KtCode::new().line(call(harness, native, params, ret, package)));
                // Kotlin has no conditional compilation, so a function whose
                // source was written under a condition is declared here
                // whatever that condition says, and the symbol behind it is
                // there only where the condition held. Saying so is all this
                // writer can do about it.
                if let Some(kdoc) = conditions_kdoc(conditions) {
                    // `kdoc` replaces. Nothing carries a source item's `///`
                    // into Kotlin yet, so there is nothing to replace; whoever
                    // adds that has to join the two rather than call this
                    // second.
                    public = public.kdoc(kdoc);
                }
                file(package, &mut files).decls.push(public.into());
            }
            Some(JniPayload::Handle {
                package,
                class,
                release,
            }) => {
                let JniPayload::Method {
                    native,
                    params,
                    ret,
                    ..
                } = &**release
                else {
                    unreachable!("a handle's release is a native method");
                };
                natives.push(native_method(native, params, ret));
                let harness = match *package == harness_package {
                    true => harness.as_str(),
                    false => harness_fqn.as_str(),
                };
                // The address is private, and leaves the class exactly once:
                // through `take()`, which the public functions call for a
                // handle they consume and `free()` calls for one they do not.
                // What is taken is forgotten, so a freed or consumed handle
                // holds zero, and zero is what the Rust side refuses.
                //
                // `getAndSet` is what makes that true between threads as well:
                // every use of a handle on this path consumes it, so two
                // racing consumers are one exchange — exactly one gets the
                // address and the other gets zero. A lock is what v1 needs
                // instead, because v1 *borrows*: it holds the address across
                // the call into Rust, and a borrowed handle is not this path.
                //
                // Minting one from an arbitrary `Long` is not a thing a caller
                // may do — `Ledger(0xdeadbeef).free()` would reach
                // `Box::from_raw` on it — so the constructor is `internal`:
                // the generated functions share a module with the class, and
                // nothing outside it does. It stops Kotlin, not Java, since an
                // internal constructor is a public JVM one; v1 closes that too,
                // with a private constructor and a companion factory, and doing
                // the same here waits for a caller that needs it.
                let declaration = KtClass::class_(class)
                    .vis(KtVis::Public)
                    .ctor_vis(KtVis::Internal)
                    .ctor_param(KtCtorParam::new("ptr", KtType::cls("Long")))
                    .member(
                        KtProperty::val("ptr")
                            .ty(KtType::cls("java.util.concurrent.atomic.AtomicLong"))
                            .initializer("AtomicLong(ptr)")
                            .vis(KtVis::Private),
                    )
                    .member(
                        KtFun::new("take")
                            .vis(KtVis::Internal)
                            .returns(KtType::cls("Long"))
                            .expr_body(KtCode::new().line("ptr.getAndSet(0L)")),
                    )
                    .member(
                        KtFun::new("free")
                            .vis(KtVis::Public)
                            .returns(KtType::cls("Unit"))
                            .expr_body(KtCode::new().line(format!("{harness}.{native}(take())"))),
                    );
                file(package, &mut files).decls.push(declaration.into());
            }
            _ => {}
        }
    }

    if !natives.is_empty() {
        let mut object = KtClass::object_(&harness).vis(KtVis::Internal);
        // Every native call routes through this object, so its `<clinit>` is
        // where a consumer loads the library — the same hook v1 offers.
        if let Some(init) = &decls.jni_native_init {
            object = object.member(KtDecl::Raw {
                name: "init".to_string(),
                code: KtCode::new().blk("init", |code| code.lines(init)),
            });
        }
        for native in natives {
            object = object.member(native);
        }
        file(&harness_package, &mut files).decls.push(object.into());
    }

    let files: Vec<KtFile> = files.into_values().collect();
    if files.is_empty() {
        // Nothing to write: the root still exists, so a Gradle source set
        // pointed at it resolves to an empty set rather than a missing
        // directory.
        std::fs::create_dir_all(kotlin_root)?;
        return Ok(Vec::new());
    }
    write_files(&files, kotlin_root)
}

/// The native method on the harness: every parameter and the result as the
/// JVM passes them, a handle as its `Long`.
fn native_method(native: &str, params: &[(String, KotlinType)], ret: &KotlinType) -> KtDecl {
    let mut function = KtFun::new(native);
    for (name, ty) in params {
        function = function.param(KtParam::new(name, KtType::cls(ty.native())));
    }
    function
        .returns(KtType::cls(ret.native()))
        .annotation("JvmSynthetic")
        .external()
        .into()
}

/// The call a public function makes: a handle argument is taken out of its
/// class, and a handle result is wrapped in one — named in full only from
/// another package.
fn call(
    harness: &str,
    native: &str,
    params: &[(String, KotlinType)],
    ret: &KotlinType,
    package: &str,
) -> String {
    let args: Vec<String> = params
        .iter()
        .map(|(name, ty)| match ty {
            KotlinType::Value(_) => name.clone(),
            KotlinType::Handle(_) => format!("{name}.take()"),
            // The `enum class` carries its own number, which is what the
            // native method takes.
            KotlinType::Enum(_) => format!("{name}.value"),
        })
        .collect();
    let call = format!("{harness}.{native}({})", args.join(", "));
    let wrap = |class: &String, call: &str, open: &str| {
        let class = match class.rsplit_once('.') {
            Some((declared_in, short)) if declared_in == package => short,
            _ => class.as_str(),
        };
        format!("{class}{open}({call})")
    };
    match ret {
        KotlinType::Value(_) => call,
        KotlinType::Handle(class) => wrap(class, &call, ""),
        // `fromInt` is the companion the enum class carries, and it is what
        // turns the number back into a value of the enum.
        KotlinType::Enum(class) => wrap(class, &call, ".fromInt"),
    }
}

/// What a declaration whose source item was written under `#[cfg]` says for
/// itself, or `None` for the ordinary unconditional one.
///
/// Kotlin has no conditional compilation, so the declaration exists whatever
/// the condition says and the symbol behind it is there only where the
/// condition held. Saying so is all this writer can do about it.
fn conditions_kdoc(conditions: &[String]) -> Option<String> {
    if conditions.is_empty() {
        return None;
    }
    // Backticks, because KDoc reads `[name]` as a reference to a declaration:
    // an unquoted `#[cfg(unix)]` would be an unresolved link on every
    // conditional declaration, and code is what it is anyway.
    let spelled: Vec<String> = conditions
        .iter()
        .map(|condition| format!("`{condition}`"))
        .collect();
    Some(format!(
        "Present only where {} holds in the source crate.\n\nKotlin cannot state a \
         condition, so this is declared either way; using it against a library built \
         without that condition raises UnsatisfiedLinkError.",
        spelled.join(" and ")
    ))
}

/// A data class property: `val name: Type`.
fn property(name: &str, ty: &str) -> KtCtorParam {
    KtCtorParam::new(name, KtType::cls(ty)).val()
}
