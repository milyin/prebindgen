//! Runs the v2 engine over `src/source.rs` through both real frontends, so
//! **rustc** gets to judge what it emitted.
//!
//! This is `docs/v2`'s worked example, executed: the same two declarations, the
//! same two targets, the same expected wrappers. `prebindgen-c` and
//! `prebindgen-jni` take the declarations exactly as a consumer's build script
//! writes them, hand them to `prebindgen-registry-v2`, and write what it
//! planned. What lands in `OUT_DIR` is compiled by `src/lib.rs`, and the tests
//! there check it against the specification pages themselves.
//!
//! Like `emitcheck`, this crate has no captured source crate: it parses one
//! file and hands its items to the frontends' `.items(..)`, which is the entry
//! point the unit-test fixtures use.

use std::path::{Path, PathBuf};

use prebindgen_c::pipeline::Pipeline;

/// The crate name stamped on every item, and so the qualifier the generated
/// code calls through (`source::stamp_sum(..)`). `src/lib.rs` mounts
/// `src/source.rs` under this name to match — as `emitcheck` does with
/// `myflat`.
const SOURCE_CRATE: &str = "source";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let source = PathBuf::from(&manifest_dir).join("src").join("source.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));

    // The C binding: `Stamp` as a by-value aggregate, the functions as entry
    // points. Both keep the source name, because nothing renamed them.
    //
    // `Pair` has positional fields, `Marker` none at all, and `Reading` a scalar
    // neither adapter carries: all three are declared so that a target refusing
    // them is visible in the report rather than only in a comment.
    let c = prebindgen_c::Cbindgen::builder()
        .items(items(&source))
        .source_module(syn::parse_quote!(source))
        // The specification keeps the source name; the frontend's default is
        // the C-style `snake_case`, and a real binding usually adds `_t`.
        .mangle_rust_type(|name| name.to_string())
        .declare(
            prebindgen_c::decls!()
                .data_type(prebindgen_c::data_type!(Stamp))
                .data_type(prebindgen_c::data_type!(Pair))
                .data_type(prebindgen_c::data_type!(Marker))
                .fun(prebindgen_c::fun!(stamp_sum))
                .fun(prebindgen_c::fun!(stamp_delta))
                .fun(prebindgen_c::fun!(stamp_show))
                .fun(prebindgen_c::fun!(marker_value)),
        )
        .build_with(Pipeline::V2)
        .expect("the C binding plans");

    // The JNI binding: `Stamp` as an `example.Stamp` object whose properties are
    // read, the functions as `example.stampSum` and friends over the harness.
    let jni = prebindgen_jni::JniGen::builder()
        .items(items(&source))
        .set_package_prefix("example")
        .package(
            prebindgen_jni::package!()
                .class(prebindgen_jni::data_class!(Stamp))
                .class(prebindgen_jni::data_class!(Reading))
                .class(prebindgen_jni::data_class!(Marker))
                .fun(prebindgen_registry::fun!(stamp_sum))
                .fun(prebindgen_registry::fun!(stamp_delta))
                .fun(prebindgen_registry::fun!(stamp_show))
                .fun(prebindgen_registry::fun!(marker_value)),
        )
        .build_with(Pipeline::V2)
        .expect("the JNI binding plans");

    let c_path = c.write_rust(out_dir.join("c.rs")).expect("write c.rs");
    let jni_path = jni
        .write_rust(out_dir.join("kotlin.rs"))
        .expect("write kotlin.rs");
    let kotlin_root = out_dir.join("kotlin");
    let written = jni.write_kotlin(&kotlin_root).expect("write the Kotlin");
    let kotlin_path = match written.as_slice() {
        [one] => one.clone(),
        other => panic!("one package, one Kotlin file; written: {other:?}"),
    };

    // The report is published beside the code, as it is for a real binding.
    c.write_manifest(&out_dir).expect("write the C report");
    jni.write_manifest(&out_dir).expect("write the JNI report");

    println!("cargo:rustc-env=V2CHECK_C={}", c_path.display());
    println!("cargo:rustc-env=V2CHECK_JNI={}", jni_path.display());
    println!("cargo:rustc-env=V2CHECK_KOTLIN={}", kotlin_path.display());
}

/// The fixture's items, as a captured source crate would hand them over.
fn items(source: &Path) -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    let location = prebindgen::SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    let text = std::fs::read_to_string(source).expect("read src/source.rs");
    syn::parse_file(&text)
        .expect("src/source.rs parses")
        .items
        .into_iter()
        .map(|item| (item, location.clone()))
        .collect()
}
