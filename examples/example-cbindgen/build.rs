//! Build script generating C bindings for `example-flat` using prebindgen + cbindgen.
//!
//! This is a language-specific binding crate. It reads the `#[prebindgen]` items
//! captured by `example-flat`, runs them through the `prebindgen_c::CbindgenBuilder`
//! adapter to produce a Rust file of `extern "C"` wrappers, then runs cbindgen on
//! that file to produce a C header.
//!
//! Both generated artifacts are also published into this crate's tree so they can
//! be committed and inspected, under a file name naming **every input that
//! changes the output** — the target architecture and the enabled features,
//! because `#[prebindgen]` `cfg` handling makes the generated code differ by both
//! (see `Foo` / `InsideFoo` in `example-flat`):
//!   - `generated/example_flat_<arch>[_<feature>…].rs` — the Rust FFI layer (`include!`d by `lib.rs`)
//!   - `include/example_flat_<arch>[_<feature>…].h`    — the C header
//!
//! One name per variant is what keeps them independent: a `--all-features` build
//! writes `…_internal_unstable.rs` and leaves the default-feature file alone,
//! instead of overwriting it with a variant that then gets committed by accident.
//!
//! Build for several targets (e.g. via the bundled `CMakeLists.txt`, or
//! `cargo build -p example-cbindgen --target x86_64-... ` then `--target aarch64-...`)
//! to produce both and see that they differ.

use std::path::{Path, PathBuf};

use prebindgen_c::{
    callback, data_type, decls, enum_type, error_type, fun,
    pipeline::{fresh_output_root, Pipeline},
    ptr_type, tagged_union,
};
use syn::parse_quote as pq;

/// The target architecture this build targets (`x86_64`, `aarch64`, …), used to
/// name the per-target generated artifacts. `CARGO_CFG_TARGET_ARCH` is set by
/// cargo for the *target* being built, so a `--target` cross-build names its own
/// file.
fn target_arch() -> String {
    std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string())
}

/// The features of `example-flat` this build enables, in declaration order, as a
/// file-name suffix (`""`, `"_unstable"`, `"_internal_unstable"`, …).
///
/// The features gate `#[prebindgen]` items, so they change the generated output
/// exactly the way the target arch does — and so they belong in the name for the
/// same reason. Without this, `cargo test --all-features` (which CI runs) silently
/// rewrites the default-feature artifact with the all-features one.
fn feature_suffix() -> String {
    let mut suffix = String::new();
    for feature in ["internal", "unstable"] {
        if std::env::var(format!("CARGO_FEATURE_{}", feature.to_uppercase())).is_ok() {
            suffix.push('_');
            suffix.push_str(feature);
        }
    }
    suffix
}

/// The committed artifact's stem: every input that changes the output, and
/// nothing else.
fn variant() -> String {
    format!("example_flat_{}{}", target_arch(), feature_suffix())
}

fn main() {
    // prebindgen part: build the Rust file of extern "C" wrappers.
    let (bindings_file, header_dest) = generate_ffi_bindings();
    // cbindgen part: build the C header from that Rust file.
    generate_c_headers(&bindings_file, &header_dest);
}

/// Generate the Rust FFI bindings from `example-flat`'s prebindgen output via the
/// `prebindgen_c::CbindgenBuilder` adapter, and publish the result to `generated/example_flat.rs`.
fn generate_ffi_bindings() -> (PathBuf, PathBuf) {
    let unstable = std::env::var("CARGO_FEATURE_UNSTABLE").is_ok();

    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name, so no per-item `.name(...)` is needed.
    let mut cbindgen = prebindgen_c::Cbindgen::builder()
        .source(example_flat::PREBINDGEN_OUT_DIR)
        .source_module(pq!(example_flat))
        // Single universal freer for the `char*` data the layer hands out
        // (the `String` returns). Opaque handles keep their typed `*_drop`.
        .free_memory_function("example_free")
        // `Calculator` -> `calculator_t` / `calculator_drop`; the base defaults to
        // the snake_case of the Rust short name (no `mangle_rust_type` override).
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        // Keep the Rust function names verbatim as the exported C symbols.
        .mangle_function(|n| n.to_string());

    // Everything this binding declares, as one flat list — which is what the C
    // header is. `calculator_apply(calculator_t *, …)` is a free function that
    // takes a handle, not a method on one, so nothing here groups functions
    // under the types they operate on. Each declaration carries its own
    // options, so moving a line cannot re-target a modifier.
    //
    // `Calculator` is a Box-owned handle (`calculator_t` / `calculator_drop`).
    // Its constructors and `Result`-returning operations route a fallible input
    // through the error out-param, so they need no `.abort_on_conversion_error()`; its borrow-only
    // accessors and predicates have no `Result` channel, so `.abort_on_conversion_error()` is what
    // lets the wrapper abort on a null handle. `calculator_merge` and
    // `calculator_absorb` are the two shapes the alias preflight covers — two
    // consumed handles of one type, and one consumed beside one borrowed — and
    // both report a rejection through `char **e` rather than aborting.
    //
    // `Error` is opaque (a boxed std error) and reaches C as a `char *` message
    // through `error_get_message`, which is consumed only as that accessor and
    // so is ignored rather than exported.
    //
    // `Shape` is a data-carrying enum: a `#[repr(C)]` tag plus union, whose
    // `Labeled` arm owns a `char *` that the generated `shape_drop` frees.
    // `Note` is the same one level up (#158 part 2): its payload wires come from
    // the resolved converter destination, which admits a nested `data_struct`
    // by value (`Caption`, owning a `char *` the union's drop reaches through)
    // and a converted leaf (`Millis` -> `u64`). The tag a C caller supplies is
    // validated before the Rust enum exists — abortively where there is no
    // `char **e`, through it where there is.
    //
    // `InsideFoo` and `Foo` are the multi-target cfg demonstration: the enum's
    // discriminants and the struct's field set vary by `target_arch` (and by
    // feature), and exactly one of each survives the target's cfg filtering, so
    // the generated `inside_foo_t` / `foo_t` differ per target. `inside_foo_value`
    // panics for a different reason than the borrow-takers: a C enum is an `int`
    // at the ABI, so an out-of-range discriminant is rejected on the way in, and
    // with no `char **e` to report it the wrapper aborts.
    //
    // The four callback signatures each name themselves, since the base derived
    // from the argument type would be `f64`, `___payload` and the like. They
    // cover the shapes a closure argument can take: a plain scalar, an
    // `Option<f64>` (no spare bit pattern, so a `bool` beside the value), a
    // `Vec<f64>` (a malloc'd `(double *, size_t)` the C side frees, so a NULL
    // `call` must convert nothing at all), and an `Option<Grade>` over an enum
    // whose discriminants skip zero — which is what says the absent slot is left
    // unwritten rather than filled with a fabricated zero.
    let mut api = decls!()
        .ptr_type(ptr_type!(Calculator))
        .error_type(error_type!(Error, error_get_message))
        .ignore_fun(pq!(error_get_message))
        .enum_type(enum_type!(Operation))
        .tagged_union(tagged_union!(Shape))
        .data_type(data_type!(Drawing))
        .data_type(data_type!(Caption))
        .convert(
            prebindgen_registry::convert!(Millis)
                .input(prebindgen_registry::fun!(millis_from_raw))
                .output(prebindgen_registry::fun!(millis_to_raw)),
        )
        .ignore_fun(pq!(millis_from_raw))
        .ignore_fun(pq!(millis_to_raw))
        .tagged_union(tagged_union!(Note))
        .enum_type(enum_type!(InsideFoo))
        .data_type(data_type!(Foo))
        .enum_type(enum_type!(Grade))
        .callback(callback!(impl Fn(f64) + Send + Sync + 'static).base_name("value"))
        .callback(callback!(impl Fn(Option<f64>) + Send + Sync + 'static).base_name("maybe_value"))
        .callback(callback!(impl Fn(Vec<f64>) + Send + Sync + 'static).base_name("history_batch"))
        .callback(
            callback!(impl Fn(Option<Grade>) + Send + Sync + 'static).base_name("maybe_grade"),
        )
        // Constructors and `Result`-returning operations: a fallible input
        // routes through the error out-param, so none needs `.abort_on_conversion_error()`.
        .fun(fun!(calculator_new))
        .fun(fun!(calculator_new_from_str))
        .fun(fun!(calculator_apply))
        .fun(fun!(calculator_merge))
        .fun(fun!(calculator_absorb))
        .fun(fun!(foo_new))
        .fun(fun!(foo_get_id))
        .fun(fun!(inside_foo_default))
        .fun(fun!(shape_new_empty))
        .fun(fun!(shape_new_circle))
        .fun(fun!(shape_new_rect))
        .fun(fun!(shape_try_area))
        .fun(fun!(note_new_silent))
        .fun(fun!(note_new_after))
        .fun(fun!(note_new_flagged))
        // Borrow-only accessors, predicates and the callback drivers: fallible
        // inputs with no `Result` channel, so `.abort_on_conversion_error()` lets the wrapper abort.
        .fun(fun!(inside_foo_value).abort_on_conversion_error())
        .fun(fun!(shape_area).abort_on_conversion_error())
        .fun(fun!(shape_get_label).abort_on_conversion_error())
        .fun(fun!(shape_new_labeled).abort_on_conversion_error())
        .fun(fun!(drawing_new).abort_on_conversion_error())
        .fun(fun!(drawing_get_shape).abort_on_conversion_error())
        .fun(fun!(note_value).abort_on_conversion_error())
        .fun(fun!(note_emphatic).abort_on_conversion_error())
        .fun(fun!(note_new_titled).abort_on_conversion_error())
        .fun(fun!(note_new_sketched).abort_on_conversion_error())
        .fun(fun!(caption_new).abort_on_conversion_error())
        .fun(fun!(calculator_new_clone).abort_on_conversion_error())
        .fun(fun!(calculator_get_value).abort_on_conversion_error())
        .fun(fun!(calculator_get_count).abort_on_conversion_error())
        .fun(fun!(calculator_is).abort_on_conversion_error())
        .fun(fun!(calculator_to_string).abort_on_conversion_error())
        .fun(fun!(calculator_get_history).abort_on_conversion_error())
        .fun(fun!(calculator_for_each).abort_on_conversion_error())
        .fun(fun!(calculator_last_or_none).abort_on_conversion_error())
        .fun(fun!(calculator_grade_or_none).abort_on_conversion_error())
        .fun(fun!(calculator_history_batch).abort_on_conversion_error());

    if unstable {
        // `calculator_reset` mirrors an `#[unstable]` slice of the API; only
        // present in the captured source when the feature is enabled. Its `&mut`
        // borrow is fallible (null-checked) with no `Result`, so `.abort_on_conversion_error()`.
        api = api.fun(fun!(calculator_reset).abort_on_conversion_error());
    }

    cbindgen = cbindgen.declare(api);

    // Reads example-flat's `#[prebindgen]` output straight from its directory.
    // Always written to OUT_DIR under a stable name too, so the commented-out
    // `include!(OUT_DIR ...)` alternative in `lib.rs` works for any target.
    let binding = cbindgen.build().expect("build prebindgen items");
    let out_file = binding
        .write_rust("example_flat.rs")
        .unwrap_or_else(|error| panic!("write generated bindings: {error}"));

    // Publish the generated Rust under its per-variant name. V1 owns the
    // committed source-tree artifacts and keeps writing them; any other engine
    // writes into a root of its own, emptied first, so it can never overwrite
    // v1's files and cannot leave its own previous run's behind. Write only on
    // change so cargo doesn't rebuild-loop on the `include!`d file.
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let (in_tree, header_dest) = match binding.pipeline() {
        Pipeline::V1 => (
            crate_dir
                .join("generated")
                .join(format!("{}.rs", variant())),
            crate_dir.join("include").join(format!("{}.h", variant())),
        ),
        other => {
            let root = fresh_output_root(&crate_dir, other).expect("make the output root");
            (
                root.join(format!("{}.rs", variant())),
                root.join(format!("{}.h", variant())),
            )
        }
    };
    write_if_changed(&in_tree, &std::fs::read_to_string(&out_file).unwrap());

    // The emitted-surface manifest, for an engine that produces one. Empty
    // under v1, whose answer is "everything declared, or the build failed".
    for path in binding
        .write_manifest(in_tree.parent().expect("the published file has a parent"))
        .expect("write_manifest failed")
    {
        println!("cargo:warning=Wrote {}", path.display());
    }

    // `lib.rs` includes this path rather than matching on `cfg`: the build script
    // already knows which variant it wrote, and a `cfg` matrix in the source would
    // have to be re-taught every arch × feature combination.
    println!(
        "cargo:rustc-env=EXAMPLE_FLAT_BINDINGS={}",
        in_tree.display()
    );
    println!("cargo:warning=Generated bindings at: {}", in_tree.display());
    (in_tree, header_dest)
}

/// Generate the C header from the prebindgen-generated Rust file via cbindgen, and
/// publish it to `include/example_flat_<arch>[_<feature>…].h` (per-variant, like
/// the Rust file).
fn generate_c_headers(bindings_file: &Path, header_dest: &Path) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let header_path = PathBuf::from(&out_dir).join("example_flat.h");

    match cbindgen::Builder::new()
        .with_config(config)
        .with_crate(&crate_dir)
        .with_src(bindings_file) // the prebindgen-generated Rust file
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(&header_path);
            // The engine that produced the Rust layer decides where its header
            // goes too, so the two can never come from different runs. Under v1
            // that is the in-tree, committed `include/` dir, per-variant.
            write_if_changed(header_dest, &std::fs::read_to_string(&header_path).unwrap());
            println!(
                "cargo:warning=Generated C header at: {}",
                header_dest.display()
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to generate C header: {e:?}");
        }
    }
}

/// Overwrite `path` only when `contents` differs from what is already there
/// (a no-op otherwise), so re-running the build introduces no spurious changes.
fn write_if_changed(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::read_to_string(path).ok().as_deref() != Some(contents) {
        std::fs::write(path, contents).expect("write generated artifact");
    }
}
