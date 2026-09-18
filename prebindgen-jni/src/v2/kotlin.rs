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
//! A handle crosses the native boundary as a `Long`. The public function is
//! where it becomes a class: an address handed out is wrapped, and one taken
//! back is taken out of its wrapper, which forgets it — so the wrapper's
//! `free()` afterwards releases nothing, and a second use raises the binding
//! failure the native side reports for a null address.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use kotlin_codegen::{
    write_files, KtClass, KtCode, KtCtorParam, KtDecl, KtFile, KtFun, KtParam, KtType, KtVis,
    WriteKotlinError,
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
                if !conditions.is_empty() {
                    // Backticks, because KDoc reads `[name]` as a reference to
                    // a declaration: an unquoted `#[cfg(unix)]` would be an
                    // unresolved link on every conditional function, and code
                    // is what it is anyway.
                    let spelled: Vec<String> = conditions
                        .iter()
                        .map(|condition| format!("`{condition}`"))
                        .collect();
                    // `kdoc` replaces. Nothing carries a source item's `///`
                    // into Kotlin yet, so there is nothing to replace; whoever
                    // adds that has to join the two rather than call this
                    // second.
                    public = public.kdoc(format!(
                        "Present only where {} holds in the source crate.\n\nKotlin cannot \
                         state a condition, so this function is declared either way; calling \
                         it against a library built without that condition raises \
                         UnsatisfiedLinkError.",
                        spelled.join(" and ")
                    ));
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
                // holds zero, and zero is what the native side refuses.
                let declaration = KtClass::class_(class)
                    .vis(KtVis::Public)
                    .ctor_param(KtCtorParam::new("ptr", KtType::cls("Long")))
                    .member(KtDecl::Raw {
                        name: "ptr".to_string(),
                        code: KtCode::new().line("private var ptr: Long = ptr"),
                    })
                    .member(KtDecl::Raw {
                        name: "take".to_string(),
                        code: KtCode::new().lines(
                            "internal fun take(): Long {\n    val taken = ptr\n    ptr = 0L\n    \
                             return taken\n}",
                        ),
                    })
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
        })
        .collect();
    let call = format!("{harness}.{native}({})", args.join(", "));
    match ret {
        KotlinType::Value(_) => call,
        KotlinType::Handle(class) => {
            let class = match class.rsplit_once('.') {
                Some((declared_in, short)) if declared_in == package => short,
                _ => class.as_str(),
            };
            format!("{class}({call})")
        }
    }
}

/// A data class property: `val name: Type`.
fn property(name: &str, ty: &str) -> KtCtorParam {
    KtCtorParam::new(name, KtType::cls(ty)).val()
}
