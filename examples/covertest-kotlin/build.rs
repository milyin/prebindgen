//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter — exercising **every**
//! JniGen feature so the hand-written `kotlin/.../Test.kt` can assert each one.
//!
//! Unlike `examples/perftest-kotlin` (which maps only the lean perf surface in
//! the performance-optimal shape), this binding maps the *same* flat library —
//! including the coverage-only items in `perftest_flat::ext` — through the full
//! adapter surface. `JniGen` accepts pre-built declaration objects (the
//! `prebindgen::lang` decl types, built by the root decl macros) rather than a fluent typestate
//! chain — each row below is a `PackageDecl`/`ConvertDecl`/etc. built
//! independently and then handed to `jni.package(...)` / `jni.convert(...)`:
//!
//! | JniGen feature                       | Exercised by |
//! |--------------------------------------|--------------|
//! | default module (first stream origin)  | `perftest_flat` |
//! | `JniGen::set_package_prefix`       | `io.prebindgen.covertest` |
//! | `JniGen::package` (subpackages)      | `model` / `errors` / `analytics` / `storage` |
//! | `JniGen::set_jni_native_init`      | `NativeLibrary.ensureLoaded()` |
//! | contextual name-mangle closures      | package-aware class/function hooks + package/class-aware method hook |
//! | `DataClassDecl`                      | `Payload`; `Annotated` (NESTED field + `Option<prim>`/`Option<enum>` fields) |
//! | `PtrClassDecl`                       | `Storage` / `Summary` / `StorageError` / `Archive` / handlers |
//! | `EnumClassDecl`                      | `Priority` |
//! | `ValueClassDecl`                     | `Stamp` (+ `Vec<Stamp>` → `List<ByteArray>`) |
//! | `convert!` + chained source streams   | `Millis` ⇄ `Long` via `covertest-helpers` fns |
//! | `Source::builder().crate_name()`      | the helpers dep is RENAMED to `cov_helpers` in Cargo.toml |
//! | `convert!` `.input(from!)`/`.output(into!)` | `Celsius` ⇄ `Int` via `From`/`Into` impls |
//! | `convert!` `.input(try_from!)` (fallible) | `Percent` ⇄ `Int`; out-of-range → `onError` |
//! | `convert!` sources `.with(path!)`/`.error(ty!)` | `Label` ⇄ `String` via binding-local fns (`crate::label_in`/`label_out`); empty label → `onError` |
//! | `.method()` / `.constructor()`       | `Storage` + `Summary` + `Stamp` members |
//! | `expand_param!` `.variant()` (+`_self`)| `Summary` default input (splittable, checked #52) |
//! | Optional combined-selector expansion  | `summary_total_opt(Option<&Summary>)` — selector `-1` = absent, borrow-identity arm clones |
//! | `FunctionDecl::split_on_param` (#52)  | single: `archiveStore`/`storageMatchesSummary` (class-default) + `storageExpectSummary` (per-fn); cartesian product: `summaryPrefer` (2 params); manual same-named overload in `ManualOverloads.kt` |
//! | `expand_return!` `.field()` (+`_self`) | `Summary` fields + `StorageError` `message` + self (error handle → `onError`) |
//! | `PackageDecl::fun` / `FunctionDecl::name`| every free function; `.name` renames `millis_add` → `addMillis` |
//! | `Generation::report()` (C7)           | `kotlin/REPORT.md` — the resolved surface, committed next to the regen |
//! | contextual method names               | method hook strips `storage`/`stamp` class prefixes; `summary_new`→`.name("of")` still overrides |
//! | per-class `.name()`                  | `Archive` → Kotlin `SummaryVault` (literal, bypasses mangles) |
//! | `.interface()` + `.implements(…)`      | `Storage`/`Payload` emit an Api interface; `CovResource`/`Timestamped` extend it (#54) |
//! | `.interface_name(…)`                  | `Priority` → generated `PriorityKind` interface (#54) |
//! | base-package functions               | `string_new` (declared in a `package!()`) |
//! | `constant!` (bare = `#[prebindgen]` const) | `COVER_MAGIC` (`Long`) + `COVER_TAG` (`String`) → top-level `val`s |
//! | `constant!(N).fun(fun!(…))`           | `cover_tag_runtime()` → eagerly-initialized `val COVER_TAG_RUNTIME` |
//! | `constant!(N).with(ty!, path!)`       | `val COVER_VERSION` from binding-local `crate::cover_version()` |
//! | `constant!(N).expr(ty!, expr!)`       | `COVER_BANNER` = binding-defined `format!` expression |
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
//! | `data_class` instance member          | `Payload.labelLen()` (receiver crosses as `this` field leaves) |
//! | `JniGen::ignore` (exact)              | `string_len` / `storage_put_by_read_and_update` (acknowledged-unbound, no skip warnings) |
//! | `JniGen::ignore` + `matching(…)`      | the `storage_get_into_*` group (one name predicate, any item kind) |
//!
//! One feature is deliberately left at its default and documented rather than
//! toggled, because it is mutually exclusive with a richer path this example
//! prefers to keep covered:
//!   * `JniGen::set_emit_handle_locks` — kept ENABLED (default). Toggling
//!     it OFF would remove the `withSortedHandleLocks` codegen this example
//!     asserts against; a single binding can only be in one lock mode, so we
//!     keep the locked one. (The toggle is a verification aid, not an
//!     optimization: benchmarks show the locks cost ~1 ns/call — see
//!     `set_emit_handle_locks` docs.)
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
//! read-write borrow). The two loners are acknowledged per-name via
//! `.ignore(fun!(…))`; the `storage_get_into_*` naming family via one
//! `.ignore(matching(…))` predicate. Both suppress the per-item
//! "skipping undeclared" build warning while emitting nothing.

use prebindgen::{
    constant, convert, core::Registry, data_class, enum_class, expand_param, expand_return, expr,
    from, fun, into, lang::JniGen, matching, package, path, ptr_class, try_from, ty, value_class,
};

fn strip_flat_class_prefix(class: &str, name: &str) -> String {
    if name
        .get(..class.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(class))
    {
        let rest = &name[class.len()..];
        let mut chars = rest.chars();
        if let Some(first) = chars.next() {
            return first.to_lowercase().chain(chars).collect();
        }
    }
    name.to_string()
}

fn main() {
    // Two prebindgen sources: the flat crate plus the binding-side helper
    // crate (conversion fns for `convert!`). The registry records each fn's
    // origin from the stream's `SourceLocation` stamps so generated calls
    // qualify with the defining crate (`perftest_flat::…` vs
    // `cov_helpers::…`). The helper dependency is RENAMED in Cargo.toml
    // (`cov_helpers = { package = "covertest-helpers", .. }`), so the stamp
    // recorded at capture time (`covertest-helpers`) would not resolve from
    // this crate — `.crate_name()` overrides it with the name this crate
    // actually uses.
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);
    let helpers = prebindgen::Source::builder(cov_helpers::PREBINDGEN_OUT_DIR)
        .crate_name("cov_helpers")
        .build();

    let jni = JniGen::new()
        .set_package_prefix("io.prebindgen.covertest")
        .set_jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
        // Every naming tier used here is configured. The harness hook is a
        // real transform: it receives the derived default `JNINative` and
        // replaces it wholesale with `CovNative` (an internal symbol, so no
        // Kotlin-side coordination is needed); four hooks are identity
        // (the domain names are already the desired Kotlin names) — registering
        // closures, and the method hook strips the flat class prefix. The
        // generated-interface hook deliberately keeps its `ClassApi` default.
        .set_harness_name_mangle(|_| "CovNative".to_string())
        .set_fun_name_mangle(|_, n| n.to_string())
        .set_ptr_class_name_mangle(|_, n| n.to_string())
        .set_data_class_name_mangle(|_, n| n.to_string())
        .set_enum_name_mangle(|_, n| n.to_string())
        .set_method_name_mangle(|_, class, n| strip_flat_class_prefix(class, n))
        // `Millis` newtype: a canonical single-value conversion to a bare
        // `Long` (no generated class) via two ordinary `#[prebindgen]` fns —
        // defined in the SEPARATE `covertest-helpers` source crate, proving
        // the multi-source model (generated calls carry the
        // `cov_helpers::` prefix). The Kotlin surface (`Long`) derives
        // from the fns' `i64` side; nothing is stated verbatim.
        .convert(
            convert!(Millis)
                .input(fun!(millis_from_long))
                .output(fun!(millis_value)),
        )
        // `Celsius`: canonical conversion via `From`/`Into` impls in the flat
        // crate — the repr (`i32`) is stated, the impls do the work.
        .convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
        // `Percent`: fallible input via `TryFrom<i32>` (out-of-range values
        // from the JVM route the impl's Error to onError); infallible output.
        .convert(convert!(Percent).input(try_from!(i32)).output(into!(i32)))
        // `Label`: conversions are plain fns in THIS binding crate (see
        // src/lib.rs) — no #[prebindgen], no helper crate. The input is
        // FALLIBLE (`fn(String) -> Result<Label, String>`; the error type is
        // stated — a bare path carries no signature to read); empty labels
        // route the Err to onError.
        .convert(
            convert!(Label)
                .input(
                    try_from!(String)
                        .with(path!(crate::label_in))
                        .error(ty!(String)),
                )
                .output(into!(String).with(path!(crate::label_out))),
        )
        // ── Base-package types ──────────────────────────────────────────────
        // `Payload` as a Kotlin `data class` (fields cross as decoupled leaves,
        // reassembled via a generated `fromParts`). A data class can carry
        // members like any re-enterable kind: the instance method's receiver
        // crosses as `this`'s field leaves (I5).
        // `Payload` also demos the `.interface()` hatch on a DATA class:
        // `PayloadApi` exposes its fields + `labelLen()`, and the
        // hand-written `Timestamped` interface extends it (#54).
        .package(
            package!().class(
                data_class!(Payload)
                    .interface()
                    .implements("io.prebindgen.covertest.Timestamped")
                    .method(fun!(payload_label_len)),
            ),
        )
        // ── Subpackage `model`: enum + value class + nested data class ──────
        .package(
            package!("model")
                // `Priority` as a Kotlin `enum class` (jint wire, `fromInt`
                // companion); `.interface_name("PriorityKind")` demos the
                // generated interface on an ENUM with a per-decl name, and the
                // hand-written `Ranked` (which extends `PriorityKind`) is
                // attached via `.implements` (#54).
                .class(
                    enum_class!(Priority)
                        .interface_name("PriorityKind")
                        .implements("io.prebindgen.covertest.Ranked"),
                )
                // `Annotated` exercises a NESTED data-class field (`payload`,
                // recursive fromParts / recursive leaf decode) plus Option<prim> and
                // Option<enum> FIELDS (each a decoupled `(present, value)` leaf pair).
                .class(data_class!(Annotated))
                // `Stamp` as a `@JvmInline value class` over its raw bytes; its readers
                // become instance methods (`secs()` / `nanos()`), and `Vec<Stamp>`
                // surfaces as `List<ByteArray>`.
                .class(
                    value_class!(Stamp)
                        .method(fun!(stamp_secs))
                        .method(fun!(stamp_nanos)),
                ),
        )
        // ── Subpackage `errors`: the Result error channel ───────────────────
        .package(package!("errors").class(
            // `StorageError` is the `E` of a fallible `Result`; its
            // boundary shape is declared with `expand_return!` below.
            ptr_class!(StorageError).method(fun!(storage_error_message)),
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
                        .method(fun!(summary_count))
                        .method(fun!(summary_total))
                        .method(fun!(summary_scaled)),
                )
                // `Archive` holds the latest `Summary` and returns it BORROWED
                // (`Option<&Summary>`) — the JVM binding clones it into a fresh owned
                // handle (the zenoh-flat borrowed-accessor shape). Its Kotlin class is
                // RENAMED via the per-declaration `.name()` override (the type-level
                // dual of the per-fn `.name`; literal, bypasses the mangle closures).
                .class(ptr_class!(Archive).name("SummaryVault")),
        )
        // `Summary` default input: rebuilt from the `of` constructor's
        // ingredients OR passed as an existing handle (runtime-selected). This
        // 2-variant set is verified *splittable* up front (#52): its arms
        // `(count, total)` vs `Summary` surface as distinct JVM signatures, so
        // functions may `.split_on_param(...)` it into typed overloads (see
        // `archive_store` / `storage_matches_summary` / `summary_prefer`).
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
                // Fn-sourced constant: a nullary `#[prebindgen]` fn surfaced
                // as an eagerly-initialized top-level `val`
                // (`COVER_TAG_RUNTIME: String`) — the value comes from the
                // fn at class-load, not from a Rust `const`.
                .constant(constant!(COVER_TAG_RUNTIME).fun(fun!(cover_tag_runtime)))
                // Binding-local-fn-sourced constant (`.with`, the const
                // analog of convert!'s `_with`): a nullary fn in THIS crate,
                // named by path, stated value type.
                .constant(constant!(COVER_VERSION).with(ty!(String), path!(crate::cover_version)))
                // Expression-sourced constant: an arbitrary binding-defined
                // expression (composing source-crate items via
                // `use perftest_flat::*;`) evaluated once at class-load —
                // no dedicated accessor fn in the source crate.
                .constant(
                    constant!(COVER_BANNER)
                        .expr(ty!(String), expr!(format!("{COVER_TAG}:{COVER_MAGIC:#x}"))),
                )
                .class(
                    ptr_class!(Storage)
                        // #54: emit the generated `StorageApi` interface (the
                        // class implements it, members get `override`) AND
                        // attach the hand-written `CovResource` which EXTENDS
                        // `StorageApi` — so its defaults call `len()`/`peek()`
                        // with full compiler checking, no hand-editing of
                        // generated code.
                        .interface()
                        .implements("io.prebindgen.covertest.CovResource")
                        .method(fun!(storage_len))
                        .method(fun!(storage_contains))
                        .constructor(fun!(storage_with_payload)),
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
                // The three convert!-source-kind fns (conversions declared
                // below): Into/From traits, TryFrom trait, binding-local fns.
                .fun(fun!(celsius_double))
                .fun(fun!(percent_scale))
                .fun(fun!(label_reverse))
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
                // Single split (#52) on the CLASS-DEFAULT `Summary` variants:
                // `storageMatchesSummary(count, total, …)` / `(expected, …)`.
                .fun(fun!(storage_matches_summary).split_on_param("expected"))
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
                // Per-fn split (#52): a per-fn `.expand_param` variant override
                // (demoing the override) whose `expected` param is then split
                // into typed overloads `storageExpectSummary(count, total, …)` /
                // `(expected, …)` on top of the selector form (which Test.kt
                // still calls directly).
                .fun(
                    fun!(storage_expect_summary)
                        .expand_param(
                            "expected",
                            expand_param!(Summary)
                                .variant(fun!(summary_new))
                                .variant_self(),
                        )
                        .split_on_param("expected"),
                )
                // Cartesian-product split (#52): TWO `Summary` params each split
                // → the 2×2 product of typed overloads (all combinations
                // distinct: build/build, build/handle, handle/build, handle/handle).
                .fun(
                    fun!(summary_prefer)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                )
                // Optional combined-selector expansion: `Option<&Summary>` under
                // the dual-arm type default — the selector also encodes absence
                // (`-1` = `None`); the borrow-identity arm clones, so the
                // caller's handle survives the call.
                .fun(fun!(summary_total_opt))
                // The borrowed-accessor trio. `archive_latest` suppresses the default
                // Summary return-field default so the BORROWED handle path (clone into a
                // fresh owned handle, null when absent) is what crosses.
                .fun(fun!(archive_new))
                // Single split (#52) on the CLASS-DEFAULT variants, consuming arm.
                .fun(fun!(archive_store).split_on_param("s"))
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
        .ignore(fun!(string_len))
        .ignore(matching(|name| name.starts_with("storage_get_into_")))
        .ignore(fun!(storage_put_by_read_and_update));

    let registry = Registry::from_items(source.items_all().chain(helpers.items_all()))
        .expect("scan prebindgen items");

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
    // The root is generator-owned: `write_kotlin` deletes and recreates it,
    // so no consumer-side cleanup is needed.
    for path in gen.write_kotlin(&kotlin_root).expect("write_kotlin failed") {
        println!("cargo:warning=Wrote {}", path.display());
    }

    // The resolved-surface report (C7): committed next to the regen so a
    // decl's effect is reviewable in a PR without reading generated Kotlin.
    std::fs::write(
        std::path::Path::new(&crate_dir)
            .join("kotlin")
            .join("REPORT.md"),
        gen.report(),
    )
    .expect("write REPORT.md");
}
