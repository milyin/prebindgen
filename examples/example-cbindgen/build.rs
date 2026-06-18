//! Build script generating C bindings for `example-flat` using prebindgen + cbindgen.
//!
//! This is a language-specific binding crate. It reads the `#[prebindgen]` items
//! captured by `example-flat`, runs them through the `prebindgen::lang::Cbindgen`
//! adapter to produce a Rust file of `extern "C"` wrappers, then runs cbindgen on
//! that file to produce a C header.
//!
//! Both generated artifacts are also published into this crate's tree so they can
//! be committed and inspected:
//!   - `generated/example_flat.rs` — the Rust FFI layer (also `include!`d by `lib.rs`)
//!   - `include/example_flat.h`    — the C header

use std::path::{Path, PathBuf};

use syn::parse_quote as pq;

fn main() {
    // prebindgen part: build the Rust file of extern "C" wrappers.
    let bindings_file = generate_ffi_bindings();
    // cbindgen part: build the C header from that Rust file.
    generate_c_headers(&bindings_file);
}

/// Generate the Rust FFI bindings from `example-flat`'s prebindgen output via the
/// `lang::Cbindgen` adapter, and publish the result to `generated/example_flat.rs`.
fn generate_ffi_bindings() -> PathBuf {
    // Reader over the data emitted by example-flat's `#[prebindgen]` macro.
    let source = prebindgen::Source::new(example_flat::PREBINDGEN_OUT_DIR);
    let unstable = std::env::var("CARGO_FEATURE_UNSTABLE").is_ok();

    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name, so no per-item `.name(...)` is needed.
    let mut cbindgen = prebindgen::lang::Cbindgen::new()
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

    // The opaque `Calculator` handle (Box-owned; `calculator_drop` generated).
    cbindgen = cbindgen.opaque_ptr(pq!(Calculator));

    // `Error` is opaque (a boxed std error), marshalled to C as a `char*` message
    // via `error_get_message(&e)`; each fallible wrapper gains a `char **e` out-param.
    // `error_get_message` is consumed only as that message fn, so it is not also
    // declared as an exported `.function` — mark it ignored to silence the
    // "skipping undeclared" notice.
    cbindgen = cbindgen
        .opaque_error(pq!(Error), pq!(error_get_message))
        .ignore_function(pq!(error_get_message));

    // The primitive-repr `Operation` enum -> a C enum.
    cbindgen = cbindgen.enum_type(pq!(Operation));

    // The borrowed-handle callback signature -> a `closure_calculator_t` closure
    // struct. `.base_name` pins the name (otherwise the `&` reference base carries
    // an internal `__` prefix into the mangled struct name).
    cbindgen = cbindgen
        .callback(pq!(impl Fn(&Calculator) + Send + Sync + 'static))
        .base_name("calculator");

    // Constructors and `Result`-returning operations: their fallible inputs route
    // through the error out-param, so no `.panic()`.
    for function in [
        pq!(calculator_new),
        pq!(calculator_new_from_str),
        pq!(calculator_apply),
    ] {
        cbindgen = cbindgen.function(function);
    }
    if unstable {
        // `calculator_reset` mirrors an `#[unstable]` slice of the API; only present
        // in the captured source when the feature is enabled. Its `&mut` borrow is
        // fallible (null-checked) with no `Result`, so `.panic()`.
        cbindgen = cbindgen.function(pq!(calculator_reset)).panic();
    }

    // Borrow-only accessors / predicates / the callback driver: they have fallible
    // (null-checked) borrow inputs but no `Result` channel, so `.panic()` lets the
    // wrapper abort on a null handle.
    for function in [
        pq!(calculator_new_clone),
        pq!(calculator_get_value),
        pq!(calculator_get_count),
        pq!(calculator_is),
        pq!(calculator_to_string),
        pq!(calculator_get_history),
        pq!(calculator_inspect),
    ] {
        cbindgen = cbindgen.function(function).panic();
    }

    let mut registry =
        prebindgen::core::Registry::from_items(source.items_all()).expect("scan prebindgen items");
    let out_file = registry
        .write_rust(&cbindgen, "example_flat.rs")
        .expect("write generated bindings");

    // Publish the generated Rust into the crate tree (committed artifact). Write
    // only on change so cargo doesn't rebuild-loop on the `include!`d file.
    let in_tree = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("generated")
        .join("example_flat.rs");
    write_if_changed(&in_tree, &std::fs::read_to_string(&out_file).unwrap());

    println!("cargo:warning=Generated bindings at: {}", in_tree.display());
    in_tree
}

/// Generate the C header from the prebindgen-generated Rust file via cbindgen, and
/// publish it to `include/example_flat.h`.
fn generate_c_headers(bindings_file: &Path) {
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
            // Publish the header to the in-tree, committed `include/` dir.
            let stable = PathBuf::from(&crate_dir)
                .join("include")
                .join("example_flat.h");
            write_if_changed(&stable, &std::fs::read_to_string(&header_path).unwrap());
            println!(
                "cargo:warning=Generated C header at: {}",
                stable.display()
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
