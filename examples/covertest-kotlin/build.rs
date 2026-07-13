//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter — exercising **every**
//! JniGen feature so the hand-written `kotlin/.../Test.kt` can assert each one.
//!
//! Unlike `examples/perftest-kotlin` (which maps only the lean perf surface in
//! the performance-optimal shape), this binding maps the *same* flat library —
//! including the coverage-only items in `perftest_flat::ext` — through the full
//! adapter surface. `JniGen` accepts pre-built declaration objects (see
//! `prebindgen::lang::jnigen::jni::decl`) rather than a fluent typestate
//! chain — each row below is a `PackageDecl`/`ConvertDecl`/etc. built
//! independently and then handed to `jni.package(...)` / `jni.convert(...)`:
//!
//! | JniGen feature                       | Exercised by |
//! |--------------------------------------|--------------|
//! | `JniGen::set_source_module`        | `perftest_flat` |
//! | `JniGen::set_package_prefix`       | `io.prebindgen.covertest` |
//! | `JniGen::package` (subpackages)      | `model` / `errors` / `analytics` / `storage` |
//! | `JniGen::set_jni_native_init`      | `NativeLibrary.ensureLoaded()` |
//! | 5 name-mangle closures               | harness (`Cov*`) + the four per-kind hooks |
//! | `DataClassDecl`                      | `Payload`; `Annotated` (NESTED field + `Option<prim>`/`Option<enum>` fields) |
//! | `PtrClassDecl`                       | `Storage` / `Summary` / `StorageError` / `Archive` / handlers |
//! | `EnumClassDecl`                      | `Priority` |
//! | `ValueClassDecl`                     | `Stamp` (+ `Vec<Stamp>` → `List<ByteArray>`) |
//! | `convert!` + multi-source (`from_sources`) | `Millis` ⇄ `Long` via `covertest-helpers` fns |
//! | `.fun()` / `.constructor()`          | `Storage` + `Summary` + `Stamp` members |
//! | `expand_param!` `.variant()` (+`_self`)| `Summary` default input |
//! | `expand_return!` `.field()` (+`_self`) | `Summary` fields + `StorageError` `message` + self (error handle → `onError`) |
//! | `PackageDecl::fun` / `FunctionDecl::name`| every free function; `.name` renames `millis_add` → `addMillis` |
//! | per-class `.name()`                  | `Archive` → Kotlin `SummaryVault` (literal, bypasses mangles) |
//! | base-package functions               | `string_new` (declared in a `package!()`) |
//! | `PackageDecl::constant` (`constant!`) | `COVER_MAGIC` (`Long`) + `COVER_TAG` (`String`) → top-level `val`s |
//! | `PackageDecl::constant_fun` | `cover_tag_runtime()` → eagerly-initialized `val COVER_TAG_RUNTIME` |
//! | `PackageDecl::constant_expr` (`constant_expr!`) | `COVER_BANNER` = binding-defined `format!` expression |
//! | per-fn `.expand_param(name, …)` identity-only | `summary_total_raw` (raw handle param, overrides the type default) |
//! | per-fn `.expand_return(…)` identity-only | `storage_summary_handle` / `archive_latest` (raw handle return) |
//! | per-fn `.expand_param(name, …)` variants | `storage_expect_summary` |
//! | per-fn `.expand_return(…)` fields+self | `storage_summary_full` |
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
//! | `JniGen::ignore_fun`                 | `string_len` / `storage_get_into_*` / `storage_put_by_read_and_update` (acknowledged-unbound, no skip warnings) |
//!
//! One feature is deliberately left at its default and documented rather than
//! toggled, because it is mutually exclusive with a richer path this example
//! prefers to keep covered:
//!   * `JniGen::set_emit_handle_locks` — kept ENABLED (default). Toggling
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
//! Four functions are deliberately NOT wrapped — their shapes are C-tier
//! with no JVM mapping (`string_len`'s `&String` param / `usize` return, the
//! `storage_get_into_*` out-param group, `storage_put_by_read_and_update`'s
//! read-write borrow) — and are acknowledged via `JniGen::ignore_fun`, which
//! suppresses the per-item "skipping undeclared" build warning while
//! emitting nothing.

use prebindgen::{
    constant, constant_expr, convert, core::Registry, data_class, enum_class, expand_param,
    expand_return, fun, lang::JniGen, package, ptr_class, value_class,
};
use syn::parse_quote as pq;

fn main() {
    // Two prebindgen sources: the flat crate plus the binding-side helper
    // crate (conversion fns for `convert!`). The registry records each fn's
    // origin so generated calls qualify with the defining crate
    // (`perftest_flat::…` vs `covertest_helpers::…`).
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);
    let helpers = prebindgen::Source::new(covertest_helpers::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new()
        .set_source_module(pq!(perftest_flat))
        .set_package_prefix("io.prebindgen.covertest")
        .set_jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
        // All five per-kind name-mangle hooks are registered. The harness hook
        // is a real transform (`Native` → `CovNative`, an internal symbol so it
        // needs no Kotlin-side coordination); the other four are the identity
        // (the domain names are already the desired Kotlin names) — registering
        // them still exercises the customization API and its `Some(closure)`
        // path.
        .set_harness_name_mangle(|n| format!("Cov{n}"))
        .set_fun_name_mangle(|n| n.to_string())
        .set_ptr_class_name_mangle(|n| n.to_string())
        .set_data_class_name_mangle(|n| n.to_string())
        .set_enum_name_mangle(|n| n.to_string())
        // `Millis` newtype: a canonical single-value conversion to a bare
        // `Long` (no generated class) via two ordinary `#[prebindgen]` fns —
        // defined in the SEPARATE `covertest-helpers` source crate, proving
        // the multi-source model (generated calls carry the
        // `covertest_helpers::` prefix). The Kotlin surface (`Long`) derives
        // from the fns' `i64` side; nothing is stated verbatim.
        .convert(
            convert!(Millis)
                .input(fun!(millis_from_long))
                .output(fun!(millis_value)),
        )
        // ── Base-package types ──────────────────────────────────────────────
        // `Payload` as a Kotlin `data class` (fields cross as decoupled leaves,
        // reassembled via a generated `fromParts`).
        .package(package!().class(data_class!(Payload)))
        // ── Subpackage `model`: enum + value class + nested data class ──────
        .package(
            package!("model")
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
                        .fun(fun!(stamp_secs).name("secs"))
                        .fun(fun!(stamp_nanos).name("nanos")),
                ),
        )
        // ── Subpackage `errors`: the Result error channel ───────────────────
        .package(package!("errors").class(
            // `StorageError` is the `E` of a fallible `Result`; its
            // boundary shape is declared with `expand_return!` below.
            ptr_class!(StorageError).fun(fun!(storage_error_message).name("message")),
        ))
        // `StorageError`'s default return fields make the generated `onError`
        // handler receive the decomposed error: the `message` string (name
        // inherited from the class member) plus — via `.field_self()` — the
        // error handle itself (an owned `StorageError` the handler must
        // `close()`).
        .expand(
            expand_return!(StorageError)
                .field(fun!(storage_error_message))
                .field_self(),
        )
        // ── Subpackage `analytics`: param-variant / return-field defaults on `Summary`
        .package(
            package!("analytics")
                // `Summary` is an opaque handle; its default boundary shape —
                // decomposed `(count, total)` leaves out, rebuilt via the `of`
                // constructor (or an existing handle) in — is declared with
                // `expand_param!` / `expand_return!` below.
                .class(
                    ptr_class!(Summary)
                        .constructor(fun!(summary_new).name("of"))
                        .fun(fun!(summary_count).name("count"))
                        .fun(fun!(summary_total).name("total"))
                        .fun(fun!(summary_scaled).name("scaled")),
                )
                // `Archive` holds the latest `Summary` and returns it BORROWED
                // (`Option<&Summary>`) — the JVM binding clones it into a fresh owned
                // handle (the zenoh-flat borrowed-accessor shape). Its Kotlin class is
                // RENAMED via the per-declaration `.name()` override (the type-level
                // dual of the per-fn `.name`; literal, bypasses the mangle closures).
                .class(ptr_class!(Archive).name("SummaryVault")),
        )
        // `Summary` default input: rebuilt from the `of` constructor's
        // ingredients OR passed as an existing handle (runtime-selected).
        .expand(
            expand_param!(Summary)
                .variant(fun!(summary_new))
                .variant_self(),
        )
        // `Summary` default output: decomposed `(count, total)` leaves, names
        // inherited from the class members.
        .expand(
            expand_return!(Summary)
                .field(fun!(summary_count))
                .field(fun!(summary_total)),
        )
        // ── Base-package handle type: `Storage` + scalar members ────────────
        // Back in the base package so the typed handle classes live alongside
        // `Payload`.
        .package(
            package!()
                // `#[prebindgen]` consts: each surfaces as a generated nullary JNI
                // getter extern + an eagerly-initialized top-level Kotlin `val`
                // (`COVER_MAGIC: Long`, `COVER_TAG: String`) in the base package.
                .constant(constant!(COVER_MAGIC))
                .constant(constant!(COVER_TAG))
                // Function-backed constant: a nullary `#[prebindgen]` fn
                // surfaced as an eagerly-initialized top-level `val`
                // (`COVER_TAG_RUNTIME: String`) — the value comes from the
                // fn at class-load, not from a Rust `const`.
                .constant_fun(fun!(cover_tag_runtime).name("COVER_TAG_RUNTIME"))
                // Expression-backed constant: an arbitrary binding-defined
                // expression (composing source-crate items via
                // `use perftest_flat::*;`) evaluated once at class-load —
                // no dedicated accessor fn in the source crate.
                .constant_expr(
                    constant_expr!(COVER_BANNER: String = format!("{COVER_TAG}:{COVER_MAGIC:#x}")),
                )
                .class(
                    ptr_class!(Storage)
                        .fun(fun!(storage_len).name("len"))
                        .fun(fun!(storage_contains).name("contains"))
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
            package!("model")
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
        // analytics: the param-variant / return-field matrix (type default /
        // per-fn override, in + out). Per-fn overrides reuse the SAME
        // expand-decl objects as the type defaults (complete-set rule): an
        // identity-only set is the plain form.
        .package(
            package!("analytics")
                .fun(fun!(storage_summary))
                .fun(fun!(storage_matches_summary))
                .fun(
                    fun!(storage_summary_handle)
                        .expand_return(expand_return!(Summary).field_self()),
                )
                .fun(
                    fun!(summary_total_raw)
                        .expand_param("s", expand_param!(Summary).variant_self()),
                )
                .fun(
                    fun!(storage_summary_full).expand_return(
                        expand_return!(Summary)
                            .field(fun!(summary_count).name("count"))
                            .field(fun!(summary_total).name("total"))
                            .field_self(),
                    ),
                )
                .fun(
                    fun!(storage_expect_summary).expand_param(
                        "expected",
                        expand_param!(Summary)
                            .variant(fun!(summary_new))
                            .variant_self(),
                    ),
                )
                // The borrowed-accessor trio. `archive_latest` suppresses the default
                // Summary return-field default so the BORROWED handle path (clone into a
                // fresh owned handle, null when absent) is what crosses.
                .fun(fun!(archive_new))
                .fun(fun!(archive_store))
                .fun(fun!(archive_latest).expand_return(expand_return!(Summary).field_self())),
        )
        // storage: the perf surface (handles, callbacks, Vec, Option) plus the
        // fallible constructor and the Millis wrapper.
        .package(
            package!("storage")
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
        // base-package classes).
        .package(package!().fun(fun!(string_new)))
        // The deliberately-unbound group (C-tier shapes with no JVM mapping):
        // acknowledged so the build log stays free of "skipping undeclared"
        // warnings without emitting anything.
        .ignore_fun(fun!(string_len))
        .ignore_fun(fun!(storage_get_into_init))
        .ignore_fun(fun!(storage_get_into_uninit))
        .ignore_fun(fun!(storage_put_by_read_and_update));

    let mut registry = Registry::from_sources([&source, &helpers]).expect("scan prebindgen items");

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
