//! Build script generating C bindings for `perftest-flat` using prebindgen + cbindgen.
//!
//! It reads the `#[prebindgen]` items captured by `perftest-flat`, runs them through
//! the `prebindgen::lang::Cbindgen` adapter to produce a Rust file of `extern "C"`
//! wrappers, then runs cbindgen on that file to produce a C header.
//!
//! The headline feature exercised here is **`.repr_c_struct(Payload)`**: `Payload`
//! is `#[repr(C)]` and FFI-safe, so it crosses the C ABI by **direct reinterpret**
//! (zero-copy) — `payload_get`/`payload_put`/`payload_callback` move/borrow the
//! struct's raw memory, with no per-field marshalling. Its `String` rides along as
//! an **opaque pointer**: `String` is declared `.opaque_ptr` (⇒ `string_t *`), and
//! the `Option<Box<String>>` field renders as `string_t *label`.
//!
//! Both generated artifacts are also published into this crate's tree (committed so
//! they can be inspected; regenerated on every build):
//!   - `generated/perftest.rs` — the Rust FFI layer (`include!`d by `lib.rs`)
//!   - `include/perftest.h`    — the C header (`#include`d by `c/perftest.c`)

use std::path::{Path, PathBuf};

use syn::parse_quote as pq;

fn main() {
    let bindings_file = generate_ffi_bindings();
    generate_c_headers(&bindings_file);
}

/// Generate the Rust FFI bindings from `perftest-flat`'s prebindgen output via the
/// `lang::Cbindgen` adapter, and publish the result to `generated/perftest_<arch>.rs`.
fn generate_ffi_bindings() -> PathBuf {
    // Reader over the data emitted by perftest-flat's `#[prebindgen]` macro.
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name (e.g. `Payload` -> `payload_t`, `String` -> `string_t`).
    let mut cbindgen = prebindgen::lang::Cbindgen::new()
        .source_module(pq!(perftest_flat))
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        .mangle_function(|n| n.to_string());

    // `String` as an opaque handle: C holds it as `string_t *` (= `Box<String>`),
    // built by `string_new`, read via `string_len`, freed by `string_drop`. This is
    // what lets the FFI-safe `Payload` carry a heap string by opaque pointer.
    cbindgen = cbindgen.opaque_ptr(pq!(String));

    // The zero-copy, `#[repr(C)]` value struct. Emits a visible-field `payload_t`
    // mirror (`Option<Box<String>>` -> `string_t *label`) + a `Transmute` and a
    // compile-time size/align assert proving the reinterpret sound.
    cbindgen = cbindgen.repr_c_struct(pq!(Payload));

    // The `&Payload` callback signature -> a `closure_payload_t` closure struct
    // whose `call` takes a `const payload_t *` (zero-copy borrow). `.base_name`
    // gives it a clean name (the `&Payload` base would mangle to `___payload`).
    cbindgen = cbindgen
        .callback(pq!(impl Fn(&Payload) + Send + Sync + 'static))
        .base_name("payload");

    // Functions. `payload_put`/`string_len` take null-checked borrows with no
    // `Result`, so they `.panic()` on a null pointer; `string_new` decodes a
    // null-checked `&str`.
    cbindgen = cbindgen.function(pq!(payload_get));
    cbindgen = cbindgen.function(pq!(payload_put)).panic();
    cbindgen = cbindgen.function(pq!(payload_callback));
    cbindgen = cbindgen.function(pq!(string_new)).panic();
    cbindgen = cbindgen.function(pq!(string_len)).panic();

    let mut registry =
        prebindgen::core::Registry::from_items(source.items_all()).expect("scan prebindgen items");
    let out_file = registry
        .write_rust(&cbindgen, "perftest.rs")
        .expect("write generated bindings");

    // Publish the generated Rust into the crate tree (committed artifact `lib.rs`
    // `include!`s; regenerated on every build).
    let in_tree = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("generated")
        .join("perftest.rs");
    write_if_changed(&in_tree, &std::fs::read_to_string(&out_file).unwrap());

    println!("cargo:warning=Generated bindings at: {}", in_tree.display());
    in_tree
}

/// Generate the C header from the prebindgen-generated Rust file via cbindgen, and
/// publish it to `include/perftest_<arch>.h` (per-target, like the Rust file).
fn generate_c_headers(bindings_file: &Path) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let header_path = PathBuf::from(&out_dir).join("perftest.h");

    match cbindgen::Builder::new()
        .with_config(config)
        .with_crate(&crate_dir)
        .with_src(bindings_file)
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(&header_path);
            let stable = PathBuf::from(&crate_dir).join("include").join("perftest.h");
            write_if_changed(&stable, &std::fs::read_to_string(&header_path).unwrap());
            println!("cargo:warning=Generated C header at: {}", stable.display());
        }
        Err(e) => {
            println!("cargo:warning=Failed to generate C header: {e:?}");
        }
    }
}

/// Overwrite `path` only when `contents` differs from what is already there.
fn write_if_changed(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::read_to_string(path).ok().as_deref() != Some(contents) {
        std::fs::write(path, contents).expect("write generated artifact");
    }
}
