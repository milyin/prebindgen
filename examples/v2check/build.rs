//! Runs the v2 engine over the `v2check-source` crate through both real
//! frontends, so
//! **rustc** gets to judge what it emitted.
//!
//! This is `docs/v2`'s worked example, executed: the same two declarations, the
//! same two targets, the same expected wrappers. `prebindgen-c` and
//! `prebindgen-jni` take the declarations exactly as a consumer's build script
//! writes them, hand them to `prebindgen-registry-v2`, and write what it
//! planned. What lands in `OUT_DIR` is compiled by `src/lib.rs`, and the tests
//! there check it against the specification pages themselves.
//!
//! Like `emitcheck`, this crate runs no proc-macro capture: it parses the
//! source crate's file and hands its items to the frontends' `.items(..)`,
//! which is the entry point the unit-test fixtures use. The crate it parses
//! is a crate it also links, so the generated code is compiled across the
//! boundary every real binding has.

use std::path::{Path, PathBuf};

use prebindgen_c::pipeline::Pipeline;

/// The crate name stamped on every item, and so the qualifier the generated
/// code calls through (`source::stamp_sum(..)`). `Cargo.toml` renames the
/// `v2check-source` dependency to this, so a generated path resolves to the
/// crate the items were parsed from.
const SOURCE_CRATE: &str = "source";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let source = PathBuf::from(&manifest_dir)
        .join("../v2check-source/src/lib.rs")
        .canonicalize()
        .expect("the source crate is a sibling of this one");
    println!("cargo:rerun-if-changed={}", source.display());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));

    // The C binding: `Stamp` as a by-value aggregate, `Ledger` as an opaque
    // pointer freed by `ledger_drop`, the functions as entry points. All keep
    // the source name, because nothing renamed them.
    //
    // `Pair` has positional fields, `Marker` none at all, and `Reading` a scalar
    // neither adapter carries: all three are declared so that a target refusing
    // them is visible in the report rather than only in a comment.
    let c = prebindgen_c::Cbindgen::builder()
        .items(items(&source))
        .source_module(syn::parse_quote!(source))
        // The specification keeps the source name; the frontend's default is
        // the C-style `snake_case`, and a real binding usually adds `_t`. The
        // destructor keeps the default's case, since `Ledger_drop` is nobody's
        // C.
        .mangle_rust_type(|name| name.to_string())
        .mangle_destructor(|base| format!("{}_drop", base.to_lowercase()))
        .declare(
            prebindgen_c::decls!()
                .data_type(prebindgen_c::data_type!(Stamp))
                .data_type(prebindgen_c::data_type!(Pair))
                .data_type(prebindgen_c::data_type!(Marker))
                .fun(prebindgen_c::fun!(stamp_sum))
                .fun(prebindgen_c::fun!(stamp_delta))
                .fun(prebindgen_c::fun!(stamp_show))
                .fun(prebindgen_c::fun!(marker_value))
                .enum_type(prebindgen_c::enum_type!(Operation))
                .fun(prebindgen_c::fun!(operation_flip))
                .enum_type(prebindgen_c::enum_type!(Adjust))
                .fun(prebindgen_c::fun!(adjust_invert))
                .enum_type(prebindgen_c::enum_type!(Gear))
                .fun(prebindgen_c::fun!(gear_rank))
                .enum_type(prebindgen_c::enum_type!(Sweep))
                .enum_type(prebindgen_c::enum_type!(Detent))
                .data_type(prebindgen_c::data_type!(Sample))
                .fun(prebindgen_c::fun!(sample_total))
                .fun(prebindgen_c::fun!(stamp_ratio))
                .ptr_type(prebindgen_c::ptr_type!(Ledger))
                .fun(prebindgen_c::fun!(ledger_open))
                .fun(prebindgen_c::fun!(ledger_close)),
        )
        .build_with(Pipeline::V2)
        .expect("the C binding plans");

    // The JNI binding: `Stamp` as an `example.Stamp` object whose properties are
    // read, `Ledger` as an `example.Ledger` wrapping the address a `Long`
    // carries, the functions as `example.stampSum` and friends over the harness.
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
                .fun(prebindgen_registry::fun!(marker_value))
                .class(prebindgen_jni::enum_class!(Operation))
                .fun(prebindgen_registry::fun!(operation_flip))
                .class(prebindgen_jni::enum_class!(Adjust))
                .fun(prebindgen_registry::fun!(adjust_invert))
                .class(prebindgen_jni::enum_class!(Sweep))
                .class(prebindgen_jni::enum_class!(Detent))
                .class(prebindgen_jni::data_class!(Sample))
                .fun(prebindgen_registry::fun!(sample_total))
                .fun(prebindgen_registry::fun!(stamp_ratio))
                .class(prebindgen_jni::ptr_class!(Ledger))
                .fun(prebindgen_registry::fun!(ledger_open))
                .fun(prebindgen_registry::fun!(ledger_close)),
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

    // What neither target could generate, as cargo warnings, and as a file
    // the tests read back: a build script owns what it makes of a skip, and
    // this one publishes it.
    c.warn_skipped();
    jni.warn_skipped();
    write_skipped(
        &out_dir.join("c-skipped.txt"),
        c.skipped()
            .iter()
            .map(|(declaration, skip)| format!("{declaration}\t{}", skip.capability))
            .collect(),
    );
    write_skipped(
        &out_dir.join("jni-skipped.txt"),
        jni.skipped()
            .iter()
            .map(|(declaration, skip)| format!("{declaration}\t{}", skip.capability))
            .collect(),
    );

    // `Sample`'s field condition reaches the generated Rust, so this crate
    // compiles a `#[cfg]` over a name rustc would otherwise report as one
    // nobody declared.
    println!("cargo::rustc-check-cfg=cfg(v2check_conditional_field)");
    println!("cargo::rustc-check-cfg=cfg(v2check_conditional_fn)");

    println!("cargo:rustc-env=V2CHECK_C={}", c_path.display());
    println!("cargo:rustc-env=V2CHECK_JNI={}", jni_path.display());
    println!("cargo:rustc-env=V2CHECK_KOTLIN={}", kotlin_path.display());
}

/// One `<declaration>\t<capability>` line per skip, in the order the engine
/// left them out.
fn write_skipped(path: &Path, lines: Vec<String>) {
    std::fs::write(path, lines.join("\n")).expect("write the skip list");
}

/// The fixture's items, as a captured source crate would hand them over.
fn items(source: &Path) -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    let location = prebindgen::SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    let text = std::fs::read_to_string(source).expect("read the source crate");
    syn::parse_file(&text)
        .expect("the source crate parses")
        .items
        .into_iter()
        .map(|item| (item, location.clone()))
        .collect()
}
