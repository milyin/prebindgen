//! Build script generating C bindings for `perftest-flat` using prebindgen + cbindgen.
//!
//! It reads the `#[prebindgen]` items captured by `perftest-flat`, runs them through
//! the `prebindgen_c::CbindgenBuilder` adapter to produce a Rust file of `extern "C"`
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

use prebindgen_c::pipeline::{fresh_output_root, Pipeline};
use syn::parse_quote as pq;

fn main() {
    let (bindings_file, header_dest) = generate_ffi_bindings();
    generate_c_headers(&bindings_file, &header_dest);
}

/// Generate the Rust FFI bindings from `perftest-flat`'s prebindgen output via the
/// `prebindgen_c::CbindgenBuilder` adapter, and publish the result to `generated/perftest_<arch>.rs`.
fn generate_ffi_bindings() -> (PathBuf, PathBuf) {
    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name (e.g. `Payload` -> `payload_t`, `String` -> `string_t`).
    let mut cbindgen = prebindgen_c::Cbindgen::builder()
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
        .source_module(pq!(perftest_flat))
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        .mangle_function(|n| n.to_string())
        // The universal raw-memory freer for the malloc'd `(payload_t *, size_t)`
        // array returned by `storage_get_vec` (Vec<Payload>). The C side releases
        // each element's `label` with `payload_drop`, then frees the block with
        // this `perftest_free`. (Library-namespaced, like `sqlite3_free` /
        // `curl_free` — it MUST run inside this cdylib so the block is released on
        // the same heap it was allocated on, which a plain `free()` from the
        // consumer cannot guarantee across module/CRT boundaries.)
        .free_memory_function("perftest_free");

    // `String` as an opaque handle: C holds it as `string_t *` (= `Box<String>`),
    // built by `string_new`, read via `string_len`, freed by `string_drop`. This is
    // what lets the FFI-safe `Payload` carry a heap string by opaque pointer.
    cbindgen = cbindgen.opaque_ptr(pq!(String));

    // `Storage` as an opaque handle (`storage_t *` = `Box<Storage>`, `storage_drop`):
    // it owns the payload that the functions read/write.
    cbindgen = cbindgen.opaque_ptr(pq!(Storage));

    // `PayloadHandler` as an opaque handle (`payload_handler_t *`, `payload_handler_drop`):
    // a prepared callback built once via `payload_handler_new` and fired by
    // `storage_callback` — the registered-subscriber pattern.
    cbindgen = cbindgen.opaque_ptr(pq!(PayloadHandler));

    // `PayloadVecHandler` as an opaque handle: a prepared WHOLE-BATCH callback fired by
    // `storage_callback_vec` (its closure receives the slice by reference).
    cbindgen = cbindgen.opaque_ptr(pq!(PayloadVecHandler));

    // The zero-copy, `#[repr(C)]` value struct. Emits a visible-field `payload_t`
    // mirror (`Option<Box<String>>` -> `string_t *label`) + a `Transmute` and a
    // compile-time size/align assert proving the reinterpret sound. Owned-ness is
    // INFERRED from the fields: `Payload` has an opaque-pointer `label`, so a by-value
    // consume (`storage_put_by_take`) reads the value out through a `*mut payload_t` and
    // nulls the moved-out `label` in place, making the caller's later free a no-op (no
    // `.owned()` modifier, no `Default` requirement — the `label` is nullable).
    //
    // `.assume_c_field_validity()` acknowledges the one gap the mirror policy
    // cannot close: `Payload::flag` is a `bool`, and the whole-struct
    // reinterpret gives no hook to normalise a byte outside `{0, 1}` that a C
    // caller wrote (#170 instance 3). This benchmark's C side writes the field
    // through `_Bool`, so it is in-domain by construction; a real binding should
    // prefer a `data_struct` field, which normalises per field.
    cbindgen = cbindgen
        .repr_c_struct(pq!(Payload))
        .assume_c_field_validity();

    // The `&Payload` callback signature -> a `closure_payload_t` closure struct
    // whose `call` takes a `const payload_t *` (zero-copy borrow). `.base_name`
    // gives it a clean name (the `&Payload` base would mangle to `___payload`).
    cbindgen = cbindgen
        .callback(pq!(impl Fn(&Payload) + Send + Sync + 'static))
        .base_name("payload");

    // The whole-batch `&[Payload]` callback signature -> a `closure_payload_vec_t` whose
    // `call` takes a `const payload_t *` + `size_t` — the slice delivered **by reference**
    // (zero-copy, no per-element materialization).
    cbindgen = cbindgen
        .callback(pq!(impl Fn(&[Payload]) + Send + Sync + 'static))
        .base_name("payload_vec");

    // Functions. `storage_new` returns a fresh handle (no fallible input). The others
    // take null-checked borrows / by-value consumes with no `Result`, so they `.panic()`
    // on a null pointer. The five `storage_put_*`/`storage_get_into_*` demonstrate the
    // distinct C parameter semantics (by-value consume, `const *` read, `*` read/write,
    // out-param-into-init, out-param-into-uninit).
    cbindgen = cbindgen.function(pq!(storage_new));
    cbindgen = cbindgen.function(pq!(storage_get)).panic();
    cbindgen = cbindgen.function(pq!(storage_put_by_take)).panic();
    cbindgen = cbindgen.function(pq!(storage_put_by_read)).panic();
    cbindgen = cbindgen
        .function(pq!(storage_put_by_read_and_update))
        .panic();
    cbindgen = cbindgen.function(pq!(storage_get_into_init)).panic();
    cbindgen = cbindgen.function(pq!(storage_get_into_uninit)).panic();
    cbindgen = cbindgen.function(pq!(payload_handler_new));
    cbindgen = cbindgen.function(pq!(storage_callback)).panic();
    cbindgen = cbindgen.function(pq!(string_new)).panic();
    cbindgen = cbindgen.function(pq!(string_len)).panic();

    // Array (slice / Vec) API. `storage_put_slice` takes `&[Payload]` — a
    // `repr_c_struct` slice — which lowers to `(const payload_t *, size_t)`
    // reinterpreted zero-copy (the slice analogue of the `&Payload` borrow).
    // `storage_get_vec` returns `Vec<Payload>` → a malloc'd `(payload_t *, size_t)`
    // array the C side frees per-element. Both have only null-checked borrow inputs
    // and no `Result`, so `.panic()`.
    cbindgen = cbindgen.function(pq!(storage_put_slice)).panic();
    cbindgen = cbindgen.function(pq!(storage_get_vec)).panic();

    // Whole-batch callback: prepare once, fire with the slice by reference.
    cbindgen = cbindgen.function(pq!(payload_vec_handler_new));
    cbindgen = cbindgen.function(pq!(storage_callback_vec)).panic();

    // Reads perftest-flat's `#[prebindgen]` output straight from its directory.
    let binding = cbindgen.build().expect("build prebindgen items");
    let out_file = binding
        .write_rust("perftest.rs")
        .unwrap_or_else(|error| panic!("write generated bindings: {error}"));

    // V1 owns the committed source-tree artifacts and keeps writing them. Any
    // other engine writes into a root of its own, emptied first, so it can
    // never overwrite v1's files and cannot leave its own previous run's
    // behind.
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let (published, header_dest) = match binding.pipeline() {
        Pipeline::V1 => (
            crate_dir.join("generated").join("perftest.rs"),
            crate_dir.join("include").join("perftest.h"),
        ),
        other => {
            let root = fresh_output_root(&crate_dir, other).expect("make the output root");
            (root.join("perftest.rs"), root.join("perftest.h"))
        }
    };
    write_if_changed(&published, &std::fs::read_to_string(&out_file).unwrap());

    // The emitted-surface manifest, for an engine that produces one. Empty
    // under v1, whose answer is "everything declared, or the build failed".
    for path in binding
        .write_manifest(published.parent().expect("the published file has a parent"))
        .expect("write_manifest failed")
    {
        println!("cargo:warning=Wrote {}", path.display());
    }

    // `lib.rs` includes the path this build script chose rather than a literal
    // one, so the engine that ran decides which file is compiled.
    println!(
        "cargo:rustc-env=PERFTEST_C_BINDINGS={}",
        published.display()
    );
    println!(
        "cargo:warning=Generated bindings at: {}",
        published.display()
    );
    (published, header_dest)
}

/// Generate the C header from the prebindgen-generated Rust file via cbindgen,
/// and publish it to `header_dest` — `include/perftest.h` under v1, and the
/// engine's own output root otherwise.
fn generate_c_headers(bindings_file: &Path, header_dest: &Path) {
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
            // The engine that produced the Rust layer decides where its header
            // goes too, so the two can never come from different runs.
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

/// Overwrite `path` only when `contents` differs from what is already there.
fn write_if_changed(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::read_to_string(path).ok().as_deref() != Some(contents) {
        std::fs::write(path, contents).expect("write generated artifact");
    }
}
