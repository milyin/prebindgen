//! Emits JNI bindings for `src/myflat.rs` so that **rustc** gets to judge them.
//!
//! ## Why this crate exists (#269)
//!
//! The adapter unit tests call `write_rust` in ~100 places and then assert on
//! the result as *text*. An emission can be well-formed, contain every
//! substring a test looks for, and still not compile — #268 was exactly that,
//! and 41 of 41 tests stayed green over it. The two Kotlin examples do compile
//! their generated Rust, but only for the shapes they declare; the shapes only
//! the unit tests reach were compiled by nothing.
//!
//! So this crate compiles emitted Rust and nothing else. There is no JVM side,
//! no runtime assertion, no Kotlin: "does this emit valid Rust" is a question
//! it can answer on its own, which is what separates it from `covertest-kotlin`.
//!
//! ## How it differs from the other examples
//!
//! Every other example crate pairs a `#[prebindgen]`-annotated source crate
//! with a consumer that reads the captured records. This one has no source
//! crate: it parses `src/myflat.rs` and hands the items to
//! `Flat::builder().items(..)` — the same entry point the unit-test fixtures
//! use, which is the point, since the shape space being covered is theirs.
//! One file is therefore both the compiled definitions and the model input,
//! and adding a spelling means editing `src/myflat.rs` alone.

use prebindgen_jni::{
    matching, package,
    pipeline::{fresh_output_root, Pipeline},
    ptr_class, JniGen,
};
use prebindgen_registry::{expand_return, fields, fun};

/// The crate name stamped on every item, and so the qualifier the generated
/// code calls through (`myflat::z_keyexpr_as_str(..)`). `src/lib.rs` mounts
/// `src/myflat.rs` under this name to match. Kept identical to the unit-test
/// fixtures' `myflat_loc()`.
const SOURCE_CRATE: &str = "myflat";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let myflat = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("myflat.rs");
    println!("cargo:rerun-if-changed={}", myflat.display());

    let loc = prebindgen::SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    let source = std::fs::read_to_string(&myflat).expect("read src/myflat.rs");
    let items = syn::parse_file(&source)
        .expect("src/myflat.rs parses")
        .items
        .into_iter()
        // The model's item kinds. `src/myflat.rs` is a real Rust file, so it
        // also carries the `use` its own signatures need — which is not
        // something a flat surface declares.
        .filter(|item| {
            matches!(
                item,
                syn::Item::Fn(_) | syn::Item::Struct(_) | syn::Item::Enum(_) | syn::Item::Const(_)
            )
        })
        .map(|item| (item, loc.clone()));

    let jni = JniGen::builder()
        // `items` rather than `source`: there is no captured JSONL, because
        // there is no `#[prebindgen]` source crate. This is the entry point the
        // builder documents for "synthetic items in a test".
        .items(items)
        // The two halves of what the row differential does with this binding's
        // decompositions: the ones it does not compare, each named with a stable
        // reason code, and how many it does. Every decomposition is in exactly one,
        // so a build fails if either moves — a decomposition leaving the comparison,
        // or leaving the population. Each entry is a part binding #701 step 3 owes.
        .expect_parity_skips(["the callback argument `ZSample`: value-form-field-with-parts"])
        .expect_parity_compared(0)
        .set_package_prefix("io.prebindgen.emitcheck")
        .package(
            package!()
                .class(ptr_class!(ZSample))
                .class(ptr_class!(ZKeyExpr))
                // The one output position: everything below is reached through
                // `ZSample` crossing into this callback.
                .fun(fun!(z_sample_sub)),
        )
        // The child boundary every value-form handle field must still cross
        // through.
        .expand(expand_return!(ZKeyExpr).field(fun!(z_keyexpr_as_str)))
        // `.fields(fields!(..))` derives the boundary FROM the struct, so every
        // field of `ZSampleStruct` becomes an emitted access — which is the
        // code under test.
        .expand(expand_return!(ZSample).fields(fields!(z_sample_to_struct)))
        // The value form is read through `fields!`, never emitted as a class of
        // its own — acknowledge it so the build log carries no standing
        // "skipping undeclared" warning to hide a real one behind.
        .ignore(matching(|name| name == "ZSampleStruct"));

    let generation = jni.build().unwrap_or_else(|err| {
        // A resolve failure names its own item; surface it as the build error
        // rather than letting a stale generated file compile.
        panic!("emitcheck: resolving the binding failed: {err}");
    });

    // Committed next to the source (not `OUT_DIR`) for the same two reasons the
    // other examples do it: `examples/regen-check.sh` can diff it, and a rustc
    // error lands on a stable reviewable path instead of a build hash.
    //
    // V1 owns that committed file. Any other engine writes into a root of its
    // own, emptied first, so it can never overwrite v1's file and cannot leave
    // its own previous run's behind.
    let dest = match generation.pipeline() {
        Pipeline::V1 => std::path::Path::new(&manifest_dir)
            .join("src")
            .join("generated_bindings.rs"),
        other => fresh_output_root(&manifest_dir, other)
            .expect("make the output root")
            .join("generated_bindings.rs"),
    };
    let written = generation
        .write_rust(&dest)
        .unwrap_or_else(|error| panic!("write_rust failed: {error}"));
    // `lib.rs` includes the path this build script chose rather than a literal
    // one, so the engine that generated the file is the one whose file is
    // compiled.
    println!("cargo:rustc-env=EMITCHECK_BINDINGS={}", written.display());
    println!("cargo:warning=emitcheck: wrote {}", written.display());

    // The emitted-surface manifest, for an engine that produces one. Empty
    // under v1, whose answer is "everything declared, or the build failed".
    for path in generation
        .write_manifest(dest.parent().expect("the written file has a parent"))
        .expect("write_manifest failed")
    {
        println!("cargo:warning=emitcheck: wrote {}", path.display());
    }
}
