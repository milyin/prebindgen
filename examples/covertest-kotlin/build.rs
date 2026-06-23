//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter — exercising **every**
//! JniGen feature so the hand-written `kotlin/.../Test.kt` can assert each one.
//!
//! Unlike `examples/perftest-kotlin` (which maps only the lean perf surface in
//! the performance-optimal shape), this binding maps the *same* flat library —
//! including the coverage-only items in `perftest_flat::ext` — through the full
//! adapter surface:
//!
//! | JniGen feature                       | Exercised by |
//! |--------------------------------------|--------------|
//! | `source_module`                      | `perftest_flat` |
//! | `package_prefix`                     | `io.prebindgen.covertest` |
//! | `package` (subpackages)              | `model` / `errors` / `analytics` / `storage` |
//! | `jni_native_init`                    | `NativeLibrary.ensureLoaded()` |
//! | 6 name-mangle closures               | harness (`Cov*`) + the five per-kind hooks |
//! | `data_class`                         | `Payload` |
//! | `ptr_class`                          | `Storage` / `Summary` / `StorageError` / handlers |
//! | `enum_class`                         | `Priority` |
//! | `value_class`                        | `Stamp` (+ `Vec<Stamp>` → `List<ByteArray>`) |
//! | `kotlin_type`                        | `Millis` → `Long` |
//! | `accessor` / `method` / `constructor`| `Storage` + `Summary` + `Stamp` members |
//! | `flatten_input` (+`variant`/self)    | `Summary` default input |
//! | `flatten_output` (+`field`/self)     | `Summary` default + `StorageError` message |
//! | `fun` / `name`                       | every free function; `.name` renames `millis_add` → `addMillis` |
//! | `flatten_input_suppress`             | `summary_total_raw` |
//! | `flatten_output_suppress`            | `storage_summary_handle` |
//! | `flatten_input_with` (+`variant`/self)| `storage_expect_summary` |
//! | `flatten_output_with` (+`field`/self)| `storage_summary_full` |
//! | `input_wrapper` / `output_wrapper`   | `Millis` ⇄ `Long` |
//! | `Result<_, E>` → `onError`           | `storage_try_with_label` |
//! | `Option<T>`                          | `Option<Payload>` / `Option<Vec>` / `Option<i64>` / `Option<enum>` |
//! | `impl Fn` callbacks (single + slice) | `payload_handler_new` / `payload_vec_handler_new` |
//!
//! Two features are deliberately left at their default and documented rather
//! than toggled, because each is mutually exclusive with a richer path that this
//! example prefers to keep covered:
//!   * `disable_handle_locks` — kept ENABLED (default). Toggling it OFF would
//!     remove the `withSortedHandleLocks` codegen this example asserts against;
//!     a single binding can only be in one lock mode, so we keep the locked one.
//!   * `suppress_kotlin_code` — see the `PayloadVecHandler` declaration below,
//!     where it IS exercised: the flag drops both the generated Kotlin class and
//!     the Rust `freePtr` destructor, and both are hand-written instead.

use prebindgen::core::Registry;
use prebindgen::lang::JniGen;
use syn::parse_quote as pq;

fn main() {
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new()
        // ── Global configuration ────────────────────────────────────────────
        .source_module(pq!(perftest_flat))
        .package_prefix("io.prebindgen.covertest")
        .jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
        // All six per-kind name-mangle hooks are registered. The harness hook is
        // a real transform (`Native` → `CovNative`, an internal symbol so it
        // needs no Kotlin-side coordination); the other five are the identity
        // (the domain names are already the desired Kotlin names) — registering
        // them still exercises the customization API and its `Some(closure)`
        // path.
        .kotlin_harness_name_mangle(|n| format!("Cov{n}"))
        .kotlin_fun_name_mangle(|n| n.to_string())
        .kotlin_ptr_class_name_mangle(|n| n.to_string())
        .kotlin_data_class_name_mangle(|n| n.to_string())
        .kotlin_enum_name_mangle(|n| n.to_string())
        .kotlin_wrapper_name_mangle(|n| n.to_string())
        // ── Base-package types ──────────────────────────────────────────────
        // `Payload` as a Kotlin `data class` (fields cross as decoupled leaves,
        // reassembled via a generated `fromParts`). Declared while no subpackage
        // is active, so it lands in the base package.
        .data_class(pq!(Payload))
        // `Millis` newtype: a custom input/output wrapper maps it to a bare
        // `Long` wire (no generated class). `.kotlin_type("Long")` overrides the
        // phantom class name the rank-0 wrapper registration would otherwise
        // stamp, so it surfaces as Kotlin `Long`.
        .input_wrapper(
            pq!(Millis),
            |_r: &Registry<_>| -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
                Some((pq!(jni::sys::jlong), None, pq!(perftest_flat::Millis(*v as u64))))
            },
        )
        .output_wrapper(
            pq!(Millis),
            |_r: &Registry<_>| -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
                Some((pq!(jni::sys::jlong), None, pq!(v.0 as jni::sys::jlong)))
            },
        )
        .kotlin_type("Long")
        // ── Subpackage `model`: enum + value class ──────────────────────────
        .package("model")
        // `Priority` as a Kotlin `enum class` (jint wire, `fromInt` companion).
        .enum_class(pq!(Priority))
        // `Stamp` as a `@JvmInline value class` over its raw bytes; its readers
        // become instance methods (`secs()` / `nanos()`), and `Vec<Stamp>`
        // surfaces as `List<ByteArray>`.
        .value_class(pq!(Stamp))
        .accessor(pq!(stamp_secs), "secs")
        .accessor(pq!(stamp_nanos), "nanos")
        // ── Subpackage `errors`: the Result error channel ───────────────────
        .package("errors")
        // `StorageError` is the `E` of a fallible `Result`. Declaring it a
        // ptr_class with a single flatten-output `message` field makes the
        // generated `onError` handler receive that message string.
        .ptr_class(pq!(StorageError))
        .accessor(pq!(storage_error_message), "message")
        .flatten_output()
        .field("message")
        // ── Subpackage `analytics`: flatten input/output on `Summary` ───────
        .package("analytics")
        // `Summary` is an opaque handle whose default boundary shape is its
        // `(count, total)` leaves: flatten-output decomposes it, flatten-input
        // rebuilds it (via the `of` constructor) or accepts a handle.
        .ptr_class(pq!(Summary))
        .constructor(pq!(summary_new), "of")
        .accessor(pq!(summary_count), "count")
        .accessor(pq!(summary_total), "total")
        .method(pq!(summary_scaled), "scaled")
        .flatten_input()
        .variant("of")
        .variant_self()
        .flatten_output()
        .field("count")
        .field("total")
        // ── Base-package handle type: `Storage` + scalar members ────────────
        // Cleared back to the base package so the typed handle classes live
        // alongside `Payload`.
        .package("")
        .ptr_class(pq!(Storage))
        .accessor(pq!(storage_len), "len")
        .method(pq!(storage_contains), "contains")
        .constructor(pq!(storage_with_payload), "withPayload")
        // The two callback-handler handles (single payload + whole batch).
        .ptr_class(pq!(PayloadHandler))
        // `PayloadVecHandler` exercises `suppress_kotlin_code`: this flag drops
        // BOTH its generated Kotlin typed-handle class AND its Rust `freePtr`
        // destructor, so both are hand-written (see
        // kotlin/io/prebindgen/covertest/PayloadVecHandler.kt and the matching
        // extern in src/lib.rs). The handle's constructor/converters are still
        // generated, so `payloadVecHandlerNew` / `storageCallbackVec` work.
        .ptr_class(pq!(PayloadVecHandler))
        .suppress_kotlin_code()
        // ── Free functions, grouped by subpackage ───────────────────────────
        // model: enum return/param/option + value-class return + Vec<value> +
        //        Option<scalar>.
        .package("model")
        .fun(pq!(payload_priority))
        .fun(pq!(priority_weight))
        .fun(pq!(priority_or))
        .fun(pq!(stamp_new))
        .fun(pq!(stamp_series))
        .fun(pq!(payload_label_len))
        // analytics: the flatten matrix (default / suppress / with, in + out).
        .package("analytics")
        .fun(pq!(storage_summary))
        .fun(pq!(storage_matches_summary))
        .fun(pq!(storage_summary_handle))
        .flatten_output_suppress()
        .fun(pq!(summary_total_raw))
        .flatten_input_suppress(pq!(s))
        .fun(pq!(storage_summary_full))
        .flatten_output_with()
        .field(pq!(summary_count), "count")
        .field(pq!(summary_total), "total")
        .field_self()
        .fun(pq!(storage_expect_summary))
        .flatten_input_with(pq!(expected))
        .variant(pq!(summary_new))
        .variant_self()
        // storage: the perf surface (handles, callbacks, Vec, Option) plus the
        // fallible constructor and the Millis wrapper.
        .package("storage")
        .fun(pq!(storage_new))
        .fun(pq!(storage_get))
        .fun(pq!(storage_put_by_take))
        .fun(pq!(storage_put_by_read))
        .fun(pq!(storage_put_slice))
        .fun(pq!(storage_get_vec))
        .fun(pq!(payload_handler_new))
        .fun(pq!(storage_callback))
        .fun(pq!(payload_vec_handler_new))
        .fun(pq!(storage_callback_vec))
        .fun(pq!(storage_try_with_label))
        // `.name(...)`: per-function Kotlin rename override. The default name
        // would be `millisAdd`; force it to `addMillis` to exercise the
        // override path (the Rust symbol/extern is unaffected).
        .fun(pq!(millis_add))
        .name("addMillis");

    let mut registry = Registry::from_items(source.items_all()).expect("scan prebindgen items");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
    let rust_dest = std::path::Path::new(&crate_dir)
        .join("src")
        .join("generated_bindings.rs");
    let rust_path = registry
        .write_rust(&jni, &rust_dest)
        .expect("write_rust failed");
    println!("cargo:warning=Generated bindings at: {}", rust_path.display());

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
