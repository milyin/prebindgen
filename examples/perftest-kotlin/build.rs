//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter. It produces:
//!   * `src/generated_bindings.rs` — the Rust-side JNI wrappers (included by
//!     `src/lib.rs`), and
//!   * `kotlin/generated/**` — the matching typed Kotlin classes.
//!
//! The example mirrors the native Rust (`perftest-flat/examples/perftest.rs`) and
//! C (`perftest-c/c/perftest.c`) micro-benchmarks: `put`/`get`/`callback`, each
//! moving a **whole** `Payload` across the boundary. A `Payload` returned by
//! `storage_get` or delivered to a `storage_callback` crosses as a Kotlin
//! `data class` (`.data_class`) — its fields cross as decoupled primitive leaves
//! and are reassembled on the Kotlin side via a generated `fromParts(...)` factory
//! (no Java object is built on the Rust side).
//!
//! `Payload.label` is `Option<Box<String>>` (an opaque-pointer string field);
//! JniGen maps `Box<String>` → Kotlin `String` and `Option<Box<String>>` →
//! `String?` automatically.

use prebindgen::core::Registry;
use prebindgen::lang::JniGen;
use syn::parse_quote as pq;

fn main() {
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new()
        .source_module(pq!(perftest_flat))
        .package_prefix("io.prebindgen.perftest")
        // Trigger native-library loading from the generated `JNINative` static
        // init (the single choke point through which every JNI call routes).
        .jni_native_init("io.prebindgen.perftest.NativeLibrary.ensureLoaded()")
        // `Payload` as a Kotlin `data class` with a `fromParts` companion factory:
        // returning/accepting it crosses its fields as decoupled leaves and the
        // object is (re)assembled on the Kotlin side — no Java object is built on
        // the Rust side.
        .data_class(pq!(Payload))
        // `Storage` as an opaque Kotlin handle class (`NativeHandle`, closeable);
        // the functions read/write the payload it owns.
        .ptr_class(pq!(Storage))
        // `PayloadHandler` as an opaque Kotlin handle class: a prepared callback built
        // once via `payloadHandlerNew` and fired by `storageCallback` — the
        // registered-subscriber pattern (the JNI trampoline is built once, not per call).
        .ptr_class(pq!(PayloadHandler))
        .package("storage")
        // Only the value/ref-input put forms map to JNI: `storage_put_by_take`
        // (by-value `Payload`) and `storage_put_by_read` (`&Payload`). The
        // `&mut Payload` / `&mut MaybeUninit<Payload>` out-param forms
        // (`storage_put_by_read_and_update`, `storage_get_into_*`) are C-only — a
        // Kotlin `data class` is an immutable value with no out-param/uninit
        // semantics — so they are left undeclared here.
        //
        // `payload_handler_new(impl Fn(&Payload)) -> PayloadHandler` prepares the callback
        // ONCE (decodes the JVM callback into the reusable native trampoline); then
        // `storage_callback(s, &PayloadHandler)` fires it. The callback arg still crosses
        // as decoupled leaves reassembled into a whole `PayloadCallback.run(Payload)` — only
        // WHERE the trampoline is built changes (once, in `payloadHandlerNew`).
        .fun(pq!(storage_new))
        .fun(pq!(storage_get))
        .fun(pq!(storage_put_by_take))
        .fun(pq!(storage_put_by_read))
        .fun(pq!(payload_handler_new))
        .fun(pq!(storage_callback))
        // Array (slice / Vec) API. `storage_put_slice(&[Payload])` takes a
        // `List<Payload>` (decoded element-by-element into an owned `Vec`, then
        // borrowed as a slice); `storage_get_vec() -> Vec<Payload>` returns a
        // `List<Payload>`. Each element is a `data class`, so it crosses via the
        // per-element struct object path.
        .fun(pq!(storage_put_slice))
        .fun(pq!(storage_get_vec));

    let mut registry = Registry::from_items(source.items_all()).expect("scan prebindgen items");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
    let rust_dest = std::path::Path::new(&crate_dir)
        .join("src")
        .join("generated_bindings.rs");
    let rust_path = registry
        .write_rust(&jni, &rust_dest)
        .expect("write_rust failed");
    println!(
        "cargo:warning=Generated bindings at: {}",
        rust_path.display()
    );

    // Kotlin classes → kotlin/generated/** (picked up by the Gradle source set).
    let kotlin_root = std::path::Path::new(&crate_dir)
        .join("kotlin")
        .join("generated");
    if let Err(err) = std::fs::remove_dir_all(&kotlin_root) {
        if err.kind() != std::io::ErrorKind::NotFound {
            panic!("cleanup kotlin/generated failed: {err}");
        }
    }
    for path in jni
        .write_kotlin(&registry, &kotlin_root)
        .expect("write_kotlin failed")
    {
        println!("cargo:warning=Wrote {}", path.display());
    }
}
