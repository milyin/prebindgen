//! Migrated from main. Unsupported declarations remain commented below.
//! Build script generating Kotlin/JNI bindings for `perftest-flat` using the
//! separate `prebindgen-jni` crate's `JniGenBuilder` adapter. It produces:
//!   * `src/generated_bindings.rs` — the Rust-side JNI wrappers (included by
//!     `src/lib.rs`), and
//!   * `kotlin/generated/ **` — the matching typed Kotlin classes.
//!
//! The enabled surface projects `Payload` through its constructor and fields.
//! `storage_get` delivers four nullable fields to a caller-supplied builder;
//! `storage_put_opt` accepts an explicit presence flag and constructor arguments.
//! `Payload.label`, callbacks, collections, and whole-object input declarations
//! remain disabled pending their respective conversion support.

/* Unsupported: imports and helpers for the disabled declarations.
use prebindgen_jni::{data_class, package, ptr_class, JniGen};
use prebindgen_declarations::{expand_param, expand_return, fields, fun};


*/
use prebindgen_declarations::{expand_param, expand_return, fields, fun};
use prebindgen_jni::{package, ptr_class, JniGen};

fn main() {
    // Reads perftest-flat's `#[prebindgen]` output straight from its directory.
    let binding = JniGen::builder()
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
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
                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(Payload))
                */
                                // Matched large-input micro-benchmark. Both roots contain the
                                // same 64 Long leaves; only the JNI input representation differs.
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundaryLeaf))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary2))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary4))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary8))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary16))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary32))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary64))
                */
                                /* Unsupported: this class kind or its binding is not available yet.
                .class(data_class!(ObjectBoundary64Object).jobject_input())
                */
                                /* Unsupported: the signature or function options need unsupported conversions.
                .fun(fun!(large_flat_input_sum))
                */
                                /* Unsupported: the signature or function options need unsupported conversions.
                .fun(fun!(large_object_input_sum))
                */
                // `Storage` as an opaque Kotlin handle class (`NativeHandle`, closeable);
                // the functions read/write the payload it owns.
                .class(ptr_class!(Storage)), // `PayloadHandler` as an opaque Kotlin handle class: a prepared callback built
                                             // once via `payloadHandlerNew` and fired by `storageCallback` — the
                                             // registered-subscriber pattern (the JNI trampoline is built once, not per call).
                                             /* Unsupported: this class kind or its binding is not available yet.
                                             .class(ptr_class!(PayloadHandler))
                                             */
                                                             // `PayloadVecHandler`: a prepared WHOLE-BATCH callback fired by
                                                             // `storageCallbackVec` — its `PayloadListCallback.run(List<Payload>)` receives
                                                             // the entire batch in one upcall. Because `Payload` is a `data_class!`, the
                                                             // `List` is assembled by a **fold**: the trampoline allocates the list and
                                                             // folds each element's raw leaves through the hoisted folder (Kotlin does
                                                             // `fromParts` + `add`) — no per-element Java object is built on the Rust side.
                                                             /* Unsupported: this class kind or its binding is not available yet.
                                             .class(ptr_class!(PayloadVecHandler))
                                             */
                                                             // Twin minimal handles for the LIFECYCLE benchmark: identical Rust
                                                             // shape, one plain, one `.gc_managed()` — head-to-head rows in
                                                             // `Bench.kt` price the GC-cleanup machinery (atomic cell, Cleaner
                                                             // registration, CAS release ticket) per handle create/close/call.
                                                             /* Unsupported: this class kind or its binding is not available yet.
                                             .class(
                                                                 ptr_class!(Token)
                                                                     .constructor(fun!(token_new))
                                                                     .method(fun!(token_value)),
                                                             )
                                             */
                                                             /* Unsupported: this class kind or its binding is not available yet.
                                             .class(
                                                                 ptr_class!(TokenGc)
                                                                     .gc_managed()
                                                                     .constructor(fun!(token_gc_new))
                                                                     .method(fun!(token_gc_value)),
                                                             )
                                             */
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
                .fun(fun!(storage_get)),
            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_put_by_take))
            */
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_put_by_read))
            */
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(payload_handler_new))
            */
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_callback))
            */
                            // Array (slice / Vec) API. `storage_put_slice(&[Payload])` takes a
                            // `List<Payload>` (decoded element-by-element into an owned `Vec`, then
                            // borrowed as a slice); `storage_get_vec() -> Option<Vec<Payload>>` returns
                            // a `List<Payload>?`. The element is a `data class`, so the return is a
                            // **fixed fold**: each element's fields cross as decoupled raw leaves and a
                            // hoisted Kotlin folder reassembles them via `fromParts` and appends to the
                            // list — no `ArrayList` and no per-element Java object on the Rust side.
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_put_slice))
            */
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_get_vec))
            */
                            // Whole-batch callback: prepared once, fired with the entire `List<Payload>`.
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(payload_vec_handler_new))
            */
                            /* Unsupported: the signature or function options need unsupported conversions.
            .fun(fun!(storage_callback_vec))
            */
        );

    /* Unsupported: source-tree artifact publishing and the old surface report.
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
        let rust_dest = std::path::Path::new(&crate_dir)
            .join("src")
            .join("generated_bindings.rs");
        let jni = binding.build().expect("build failed");
        let rust_path = jni.write_rust(&rust_dest).expect("write_rust failed");
        println!(
            "cargo:warning=Generated bindings at: {}",
            rust_path.display()
        );

        // Kotlin classes → kotlin/generated/ ** (picked up by the Gradle source set).
        let kotlin_root = std::path::Path::new(&crate_dir)
            .join("kotlin")
            .join("generated");
        for path in jni.write_kotlin(&kotlin_root).expect("write_kotlin failed") {
            println!("cargo:warning=Wrote {}", path.display());
        }

    */
    let binding = binding
        .expand(expand_param!(Payload).variant(fun!(payload_new)))
        .expand(expand_return!(Payload).fields(fields!(payload_to_struct)))
        .package(
            package!()
                .fun(fun!(payload_new))
                .fun(fun!(storage_with_payload))
                .fun(fun!(storage_put_opt)),
        );
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let jni = binding.build().expect("build supported declarations");
    jni.write_rust("bindings.rs").unwrap();
    jni.write_kotlin(&out.join("kotlin")).unwrap();
}
