//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter — exercising **every**
//! JniGen feature so the hand-written `kotlin/.../Test.kt` can assert each one.
//!
//! Unlike `examples/perftest-kotlin` (which maps only the lean perf surface in
//! the performance-optimal shape), this binding maps the *same* flat library —
//! including the coverage-only items in `perftest_flat::ext` — through the full
//! adapter surface. `JniGen` accepts pre-built declaration objects (see
//! `prebindgen::lang::jnigen::jni::decl`) rather than a fluent typestate
//! chain — each row below is a `PackageDecl`/`ScalarTypeWrapperDecl`/etc. built
//! independently and then handed to `jni.package(...)` /
//! `jni.scalar_type_wrapper(...)`:
//!
//! | JniGen feature                       | Exercised by |
//! |--------------------------------------|--------------|
//! | `JniGenConfig::source_module`        | `perftest_flat` |
//! | `JniGenConfig::package_prefix`       | `io.prebindgen.covertest` |
//! | `JniGen::package` (subpackages)      | `model` / `errors` / `analytics` / `storage` |
//! | `JniGenConfig::jni_native_init`      | `NativeLibrary.ensureLoaded()` |
//! | 5 name-mangle closures               | harness (`Cov*`) + the four per-kind hooks |
//! | `DataClassDecl`                      | `Payload`; `Annotated` (NESTED field + `Option<prim>`/`Option<enum>` fields) |
//! | `PtrClassDecl`                       | `Storage` / `Summary` / `StorageError` / `Archive` / handlers |
//! | `EnumClassDecl`                      | `Priority` |
//! | `ValueClassDecl`                     | `Stamp` (+ `Vec<Stamp>` → `List<ByteArray>`) |
//! | `ScalarTypeWrapperDecl`              | `Millis` ⇄ `Long` |
//! | `.accessor()` / `.method()` / `.constructor()`| `Storage` + `Summary` + `Stamp` members |
//! | `.flatten_input()` (+`.variant()`/self)| `Summary` default input |
//! | `.flatten_output()` (+`.field()`/self)| `Summary` fields + `StorageError` `message` + `field_self` (error handle → `onError`) |
//! | `PackageDecl::fun` / `FunctionDecl::name`| every free function; `.name` renames `millis_add` → `addMillis` |
//! | per-class `.name()`                  | `Archive` → Kotlin `SummaryVault` (literal, bypasses mangles) |
//! | base-package functions               | `string_new` (declared in a `PackageDecl::new("")`) |
//! | `.flatten_input_suppress()`          | `summary_total_raw` |
//! | `.flatten_output_suppress()`         | `storage_summary_handle` / `archive_latest` |
//! | `.flatten_input_with()` (+`.variant()`/self)| `storage_expect_summary` |
//! | `.flatten_output_with()` (+`.field()`/self)| `storage_summary_full` |
//! | `Result<_, E>` → `onError`           | `storage_try_with_label` |
//! | `Option<T>`                          | `Option<Payload>` (in + out) / `Option<Vec>` / `Option<i64>` / `Option<enum>` (param + return + field) |
//! | `impl Fn` callbacks (single + slice) | `payload_handler_new` / `payload_vec_handler_new` |
//! | owned-handle callback (`Fn(Storage)`)| `storage_handler_new` / `storage_emit` |
//! | `Vec<handle>` / `Option<Vec<handle>>`| `storage_shards` / `storage_shards_opt` (Kotlin-side handle fold) |
//! | borrowed-opaque return (`Option<&T>`)| `archive_latest` (clone → fresh owned handle) |
//! | N-ary sorted handle locking          | `storage_total_len` (3 handles) + a 4-thread smoke |
//! | `Vec<String>` return                 | `storage_labels` (single-leaf string fold) |
//! | `String` return                      | `string_new` |
//! | binding-error channel (`je != null`) | malformed `Stamp` bytes (value-blob length guard) |
//! | callback no-throw contract           | a throwing `PayloadCallback` (described + cleared per upcall) |
//!
//! One feature is deliberately left at its default and documented rather than
//! toggled, because it is mutually exclusive with a richer path this example
//! prefers to keep covered:
//!   * `JniGenConfig::disable_handle_locks` — kept ENABLED (default). Toggling
//!     it OFF would remove the `withSortedHandleLocks` codegen this example
//!     asserts against; a single binding can only be in one lock mode, so we
//!     keep the locked one.
//!
//! `perftest-kotlin`'s declared surface is a strict subset of this binding
//! (verified 2026-07-03): its only unique configurations are the unset
//! defaults — the `JNINative` harness name (`Cov`-mangled here) and the unset
//! per-kind name hooks (≡ the identity closures registered here) — which are
//! binding-exclusive like the lock toggle above and add no code-path coverage.
//!
//! One perf-surface function stays undeclared like the `storage_get_into_*`
//! group: `string_len` (`&String` param / `usize` return are C-tier shapes with
//! no JVM mapping).

use prebindgen::{
    core::Registry,
    data_class, enum_class, flatten_input, flatten_output, fun, function_flatten_input,
    function_flatten_output,
    lang::{JniGen, JniGenConfig, PackageDecl, ScalarTypeWrapperDecl},
    ptr_class, value_class,
};
use syn::parse_quote as pq;

fn main() {
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new(
        JniGenConfig::new()
            .source_module(pq!(perftest_flat))
            .package_prefix("io.prebindgen.covertest")
            .jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
            // All five per-kind name-mangle hooks are registered. The harness hook
            // is a real transform (`Native` → `CovNative`, an internal symbol so it
            // needs no Kotlin-side coordination); the other four are the identity
            // (the domain names are already the desired Kotlin names) — registering
            // them still exercises the customization API and its `Some(closure)`
            // path.
            .kotlin_harness_name_mangle(|n| format!("Cov{n}"))
            .kotlin_fun_name_mangle(|n| n.to_string())
            .kotlin_ptr_class_name_mangle(|n| n.to_string())
            .kotlin_data_class_name_mangle(|n| n.to_string())
            .kotlin_enum_name_mangle(|n| n.to_string()),
    )
    // `Millis` newtype: a custom scalar wire mapping to a bare `Long` (no
    // generated class) — global, not tied to any package (see the `decl`
    // module doc for why a scalar wrapper never needs package placement).
    .scalar_type_wrapper(
        ScalarTypeWrapperDecl::new(pq!(Millis), pq!(jni::sys::jlong), "Long")
            .input(|v| pq!(perftest_flat::Millis(*#v as u64)))
            .output(|v| pq!(#v.0 as jni::sys::jlong)),
    )
    // ── Base-package types ──────────────────────────────────────────────
    // `Payload` as a Kotlin `data class` (fields cross as decoupled leaves,
    // reassembled via a generated `fromParts`).
    .package(PackageDecl::new("").class(data_class!(Payload)))
    // ── Subpackage `model`: enum + value class + nested data class ──────
    .package(
        PackageDecl::new("model")
            // `Priority` as a Kotlin `enum class` (jint wire, `fromInt` companion).
            .class(enum_class!(Priority))
            // `Annotated` exercises a NESTED data-class field (`payload`,
            // recursive fromParts / recursive leaf decode) plus Option<prim> and
            // Option<enum> FIELDS (each a decoupled `(present, value)` leaf pair).
            .class(data_class!(Annotated))
            // `Stamp` as a `@JvmInline value class` over its raw bytes; its readers
            // become instance methods (`secs()` / `nanos()`), and `Vec<Stamp>`
            // surfaces as `List<ByteArray>`.
            .class(
                value_class!(Stamp)
                    .accessor(fun!(stamp_secs).name("secs"))
                    .accessor(fun!(stamp_nanos).name("nanos")),
            ),
    )
    // ── Subpackage `errors`: the Result error channel ───────────────────
    .package(
        PackageDecl::new("errors").class(
            // `StorageError` is the `E` of a fallible `Result`. Declaring it a
            // ptr_class with a flatten-output makes the generated `onError`
            // handler receive the flattened fields: the `message` string plus —
            // via the TYPE-LEVEL `.field_self()` — the error handle itself (an
            // owned `StorageError` the handler must `close()`).
            ptr_class!(StorageError)
                .accessor(fun!(storage_error_message).name("message"))
                .flatten_output(flatten_output!().field("message").field_self()),
        ),
    )
    // ── Subpackage `analytics`: flatten input/output on `Summary` ───────
    .package(
        PackageDecl::new("analytics")
            // `Summary` is an opaque handle whose default boundary shape is its
            // `(count, total)` leaves: flatten-output decomposes it, flatten-input
            // rebuilds it (via the `of` constructor) or accepts a handle.
            .class(
                ptr_class!(Summary)
                    .constructor(fun!(summary_new).name("of"))
                    .accessor(fun!(summary_count).name("count"))
                    .accessor(fun!(summary_total).name("total"))
                    .method(fun!(summary_scaled).name("scaled"))
                    .flatten_input(flatten_input!().variant("of").variant_self())
                    .flatten_output(flatten_output!().field("count").field("total")),
            )
            // `Archive` holds the latest `Summary` and returns it BORROWED
            // (`Option<&Summary>`) — the JVM binding clones it into a fresh owned
            // handle (the zenoh-flat borrowed-accessor shape). Its Kotlin class is
            // RENAMED via the per-declaration `.name()` override (the type-level
            // dual of the per-fn `.name`; literal, bypasses the mangle closures).
            .class(ptr_class!(Archive).name("SummaryVault")),
    )
    // ── Base-package handle type: `Storage` + scalar members ────────────
    // Back in the base package so the typed handle classes live alongside
    // `Payload`.
    .package(
        PackageDecl::new("")
            .class(
                ptr_class!(Storage)
                    .accessor(fun!(storage_len).name("len"))
                    .method(fun!(storage_contains).name("contains"))
                    .constructor(fun!(storage_with_payload).name("withPayload")),
            )
            // The callback-handler handles (single payload / whole batch / owned
            // storage handle).
            .class(ptr_class!(PayloadHandler))
            // `StorageHandler`'s callback receives an OWNED opaque handle
            // (`Fn(Storage)`): the raw pointer crosses and the generated Kotlin
            // proxy wraps it into a typed `Storage` and closes it after `run`.
            .class(ptr_class!(StorageHandler))
            .class(ptr_class!(PayloadVecHandler)),
    )
    // ── Free functions, grouped by subpackage ───────────────────────────
    // model: enum return/param/option + value-class return + Vec<value> +
    //        Option<scalar>.
    .package(
        PackageDecl::new("model")
            .fun(fun!(payload_priority))
            .fun(fun!(priority_weight))
            .fun(fun!(priority_or))
            .fun(fun!(stamp_new))
            .fun(fun!(stamp_series))
            .fun(fun!(payload_label_len))
            .fun(fun!(annotated_new))
            .fun(fun!(annotated_ttl))
            .fun(fun!(annotated_priority))
            .fun(fun!(annotated_payload_value)),
    )
    // analytics: the flatten matrix (default / suppress / with, in + out).
    .package(
        PackageDecl::new("analytics")
            .fun(fun!(storage_summary))
            .fun(fun!(storage_matches_summary))
            .fun(fun!(storage_summary_handle).flatten_output_suppress())
            .fun(fun!(summary_total_raw).flatten_input_suppress(pq!(s)))
            .fun(fun!(storage_summary_full).flatten_output_with(
                function_flatten_output!()
                    .field(fun!(summary_count).name("count"))
                    .field(fun!(summary_total).name("total"))
                    .field_self(),
            ))
            .fun(fun!(storage_expect_summary).flatten_input_with(
                pq!(expected),
                function_flatten_input!()
                    .variant(fun!(summary_new))
                    .variant_self(),
            ))
            // The borrowed-accessor trio. `archive_latest` suppresses the default
            // Summary output flatten so the BORROWED handle path (clone into a
            // fresh owned handle, null when absent) is what crosses.
            .fun(fun!(archive_new))
            .fun(fun!(archive_store))
            .fun(fun!(archive_latest).flatten_output_suppress()),
    )
    // storage: the perf surface (handles, callbacks, Vec, Option) plus the
    // fallible constructor and the Millis wrapper.
    .package(
        PackageDecl::new("storage")
            .fun(fun!(storage_new))
            .fun(fun!(storage_get))
            .fun(fun!(storage_put_by_take))
            .fun(fun!(storage_put_by_read))
            .fun(fun!(storage_put_slice))
            .fun(fun!(storage_get_vec))
            .fun(fun!(payload_handler_new))
            .fun(fun!(storage_callback))
            .fun(fun!(payload_vec_handler_new))
            .fun(fun!(storage_callback_vec))
            .fun(fun!(storage_try_with_label))
            // Vec<opaque-handle> returns (plain + under the Option niche).
            .fun(fun!(storage_shards))
            .fun(fun!(storage_shards_opt))
            // Owned-handle-in-callback pair.
            .fun(fun!(storage_handler_new))
            .fun(fun!(storage_emit))
            // A 3-opaque-handle call (sorted N-ary handle locking).
            .fun(fun!(storage_total_len))
            // Vec<String> return (single-leaf string fold).
            .fun(fun!(storage_labels))
            // Option<data-class> input.
            .fun(fun!(storage_put_opt))
            // `.name(...)`: per-function Kotlin rename override. The default name
            // would be `millisAdd`; force it to `addMillis` to exercise the
            // override path (the Rust symbol/extern is unaffected).
            .fun(fun!(millis_add).name("addMillis")),
    )
    // Plain String return, declared in the BASE package (mirroring the
    // base-package classes). (`string_len` stays undeclared like the
    // `storage_get_into_*` group: its `&String` param / `usize` return are
    // C-tier shapes with no JVM mapping.)
    .package(PackageDecl::new("").fun(fun!(string_new)));

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
