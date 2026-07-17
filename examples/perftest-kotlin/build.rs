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

use prebindgen::{core::Registry, data_class, fun, lang::JniGen, package, ptr_class};

fn main() {
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new()
        .set_package_prefix("io.prebindgen.perftest")
        // Trigger native-library loading from the generated `JNINative` static
        // init (the single choke point through which every JNI call routes).
        .set_jni_native_init("io.prebindgen.perftest.NativeLibrary.ensureLoaded()")
        // Base-package types.
        .package(
            package!()
                // `Payload` as a Kotlin `data class` with a `fromParts` companion factory:
                // returning/accepting it crosses its fields as decoupled leaves and the
                // object is (re)assembled on the Kotlin side — no Java object is built on
                // the Rust side.
                .class(data_class!(Payload))
                // `Storage` as an opaque Kotlin handle class (`NativeHandle`, closeable);
                // the functions read/write the payload it owns.
                .class(ptr_class!(Storage))
                // `PayloadHandler` as an opaque Kotlin handle class: a prepared callback built
                // once via `payloadHandlerNew` and fired by `storageCallback` — the
                // registered-subscriber pattern (the JNI trampoline is built once, not per call).
                .class(ptr_class!(PayloadHandler))
                // `PayloadVecHandler`: a prepared WHOLE-BATCH callback fired by
                // `storageCallbackVec` — its `PayloadListCallback.run(List<Payload>)` receives
                // the entire batch in one upcall. Because `Payload` is a `data_class!`, the
                // `List` is assembled by a **fold**: the trampoline allocates the list and
                // folds each element's raw leaves through the hoisted folder (Kotlin does
                // `fromParts` + `add`) — no per-element Java object is built on the Rust side.
                .class(ptr_class!(PayloadVecHandler))
                // Twin minimal handles for the LIFECYCLE benchmark: identical Rust
                // shape, one plain, one `.gc_managed()` — head-to-head rows in
                // `Bench.kt` price the GC-cleanup machinery (atomic cell, Cleaner
                // registration, CAS release ticket) per handle create/close/call.
                .class(
                    ptr_class!(Token)
                        .constructor(fun!(token_new))
                        .method(fun!(token_value)),
                )
                .class(
                    ptr_class!(TokenGc)
                        .gc_managed()
                        .constructor(fun!(token_gc_new))
                        .method(fun!(token_gc_value)),
                ),
        )
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
        .package(
            package!("storage")
                .fun(fun!(storage_new))
                .fun(fun!(storage_get))
                .fun(fun!(storage_put_by_take))
                .fun(fun!(storage_put_by_read))
                .fun(fun!(payload_handler_new))
                .fun(fun!(storage_callback))
                // Array (slice / Vec) API. `storage_put_slice(&[Payload])` takes a
                // `List<Payload>` (decoded element-by-element into an owned `Vec`, then
                // borrowed as a slice); `storage_get_vec() -> Option<Vec<Payload>>` returns
                // a `List<Payload>?`. The element is a `data class`, so the return is a
                // **fixed fold**: each element's fields cross as decoupled raw leaves and a
                // hoisted Kotlin folder reassembles them via `fromParts` and appends to the
                // list — no `ArrayList` and no per-element Java object on the Rust side.
                .fun(fun!(storage_put_slice))
                .fun(fun!(storage_get_vec))
                // Whole-batch callback: prepared once, fired with the entire `List<Payload>`.
                .fun(fun!(payload_vec_handler_new))
                .fun(fun!(storage_callback_vec)),
        );

    let registry = Registry::from_items(source.items_all()).expect("scan prebindgen items");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
    let rust_dest = std::path::Path::new(&crate_dir)
        .join("src")
        .join("generated_bindings.rs");
    let gen = registry.resolve(jni).expect("resolve failed");
    let rust_path = gen.write_rust(&rust_dest).expect("write_rust failed");
    println!(
        "cargo:warning=Generated bindings at: {}",
        rust_path.display()
    );

    // Kotlin classes → kotlin/generated/** (picked up by the Gradle source set).
    let kotlin_root = std::path::Path::new(&crate_dir)
        .join("kotlin")
        .join("generated");
    for path in gen.write_kotlin(&kotlin_root).expect("write_kotlin failed") {
        println!("cargo:warning=Wrote {}", path.display());
    }
}
