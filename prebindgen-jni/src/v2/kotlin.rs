//! The Kotlin side of a v2 generation.
//!
//! The engine renders Rust only; what Kotlin a binding needs is the JNI
//! adapter's to say, and it says it here from the payloads its declarations
//! carried through planning — a `data class` per emitted record, one `external
//! fun` per emitted function on the harness object, and a public function
//! calling each. Rendering goes through `kotlin-codegen`, as v1's does, so the
//! output is validated Kotlin and lands in the same generator-owned tree.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use kotlin_codegen::{
    write_files, KtClass, KtCode, KtCtorParam, KtDecl, KtFile, KtFun, KtParam, KtType, KtVis,
    WriteKotlinError,
};
use prebindgen_registry_v2::Generation;

use super::target::JniPayload;
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
            }) => {
                let signature = |mut function: KtFun| {
                    for (name, ty) in params {
                        function = function.param(KtParam::new(name, KtType::cls(ty)));
                    }
                    function.returns(KtType::cls(ret))
                };
                // The native method, on the harness.
                natives.push(
                    signature(KtFun::new(native))
                        .annotation("JvmSynthetic")
                        .external()
                        .into(),
                );
                // The function a caller uses, delegating to it. The harness is
                // named in full only from another package.
                let harness = match *package == harness_package {
                    true => harness.as_str(),
                    false => harness_fqn.as_str(),
                };
                let args: Vec<&str> = params.iter().map(|(name, _)| name.as_str()).collect();
                let call = format!("{harness}.{native}({})", args.join(", "));
                file(package, &mut files).decls.push(
                    signature(KtFun::new(method))
                        .vis(KtVis::Public)
                        .expr_body(KtCode::new().line(call))
                        .into(),
                );
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

/// A data class property: `val name: Type`.
fn property(name: &str, ty: &str) -> KtCtorParam {
    KtCtorParam::new(name, KtType::cls(ty)).val()
}
