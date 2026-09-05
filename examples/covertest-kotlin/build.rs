//! Build script generating Kotlin/JNI bindings for `perftest-flat` using the
//! separate `prebindgen-jni` crate's `JniGenBuilder` adapter — exercising **every**
//! JniGenBuilder feature so the hand-written `kotlin/.../Test.kt` can assert each one.
//!
//! Unlike `examples/perftest-kotlin` (which maps only the lean perf surface in
//! the performance-optimal shape), this binding maps the *same* flat library —
//! including the coverage-only items in `perftest_flat::ext` — through the full
//! adapter surface. `JniGenBuilder` accepts pre-built declaration objects (the
//! `prebindgen-jni` decl types, built by its root decl macros) rather than a fluent typestate
//! chain — each row below is a `PackageDecl`/`ConvertDecl`/etc. built
//! independently and then handed to `jni.package(...)` / `jni.convert(...)`:
//!
//! | JniGenBuilder feature                       | Exercised by |
//! |--------------------------------------|--------------|
//! | default module (first stream origin)  | `perftest_flat` |
//! | `JniGenBuilder::set_package_prefix`       | `io.prebindgen.covertest` |
//! | `JniGenBuilder::package` (subpackages)      | `model` / `errors` / `analytics` / `storage` |
//! | `JniGenBuilder::set_jni_native_init`      | `NativeLibrary.ensureLoaded()` |
//! | contextual name-mangle closures      | package-aware class/function hooks + package/class-aware method hook |
//! | `DataClassDecl`                      | `Payload`; `Annotated` (recursive direct + optional nested fields) |
//! | `DataClassDecl::jobject_input()`     | `ObjectBoundary` (127 `Long` leaves plus JNI infrastructure exceed the JVM's 255-slot method limit) |
//! | `PtrClassDecl`                       | `Storage` / `Summary` / `StorageError` / `Archive` / handlers |
//! | `EnumClassDecl`                      | `Priority` |
//! | `convert!` + chained source streams   | `Millis` ⇄ `Long` via `covertest-helpers` fns |
//! | `.source_named(dir, "cov_helpers")`   | the helpers dep is RENAMED to `cov_helpers` in Cargo.toml |
//! | `convert!` `.input(from!)`/`.output(into!)` | `Celsius` ⇄ `Int` via `From`/`Into` impls |
//! | fallible conversion stages under `Option` | `Option<Percent>` ⇄ `Int?`; raw `TryFrom::Error` input and binding-local `String` output errors normalize to `JniErrorHandler` |
//! | `convert!` sources `fun!(crate::…).sig(sig!)` | `Label` ⇄ `String` via binding-local fns (`crate::label_in`/`label_out`); the sig's `Result` = error channel, empty label → `onError` |
//! | bounded conversion domains + niches | `Option<Duration>` ⇄ bounded millisecond `ULong?`; raw JNI remains primitive `Long`, `None` uses an invalid `u64`, invalid input/output routes to `onError`; `DurationBoundary` composes the niche through a data-class field and whole-object decode |
//! | `.method()` / `.constructor()`       | `Storage` + `Summary` + `Stamp` members |
//! | `expand_param!` `.variant()` (+`_self`)| `Summary` default input (splittable, checked #52) |
//! | recursive `expand_param!` constructor input | `SummaryEnvelope` folds the nested Summary build/identity selector before its outer constructor |
//! | Optional combined-selector expansion  | `summary_total_opt(Option<&Summary>)` — selector `-1` = absent, borrow-identity arm clones; `selector_code_score(Option<&SelectorCode>)` lowers its synthetic `Option<u16>` build-arm leaf to a primitive presence/value pair |
//! | `FunctionDecl::split_on_param` (#52)  | single: `archiveStore`/`storageMatchesSummary` (class-default) + `storageExpectSummary` (per-fn); cartesian product: `summaryPrefer` (2 params); manual same-named overload in `ManualOverloads.kt` |
//! | split × builder-delivered return (#87) | `summaryMerge` — cartesian split + generic `<R>` wrapper; every overload re-declares `<R>` |
//! | JNI native-symbol escaping (#86)      | `esc_pkg.Esc_Probe` — underscored subpackage + class (escaped `freePtr` symbol) + hook-mangled `escape_probe_value` harness extern |
//! | `expand_return!` `.field()` (+`_self`) | `Summary` fields + `StorageError` `message` + self (error handle → `onError`) |
//! | `expand_return!` `.fields(fields!(…))` (#213) | `Report` — boundary DERIVED from the value form instead of restated; covers every per-field rule (spliced `Summary`, inlined `Stamp`, `Option<data class>`, a sum with a handle payload, a plain leaf) |
//! | `expand_return!` `.fields_self_into(fields!(…))` | `report_into_struct(r: Report)` — the CONSUMING value form: the value is given away and its fields MOVED out, so the clones the borrowing `report_to_struct` pays are not emitted at all |
//! | `PackageDecl::fun` / `FunctionDecl::name`| every free function; `.name` renames `millis_add` → `addMillis` |
//! | `JniGen::report()` (C7)               | `kotlin/REPORT.md` — the resolved surface, committed next to the regen |
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
//! | binding-local field `fun!(crate::…).sig(sig!).name(…)` | `storage_summary_probe` — custom field, here a conditional handle via `crate::summary_if_nonempty` |
//! | binding-local fn `fun!(crate::…)` `.sig(sig!)` as free fn | `describeSummary` ← `crate::summary_describe` |
//! | binding-local fn as `.method()` / `.constructor()` | `Summary.mean()` ← `crate::summary_mean` (NO `.name` — derived by the strip hook); `Summary.fromMean` ← `crate::summary_from_mean` (FALLIBLE — sig `Result` → `onError`) |
//! | `Result<_, E>` → typed domain `onError` | `storage_try_with_label` |
//! | two-caller split (#45): `onBindingError` + `onError` on one fallible wrapper | `storage_try_from_stamp` (wrong-length `tag` → binding; bad `secs` → domain) |
//! | fixed-width unsigned scalars (#108) | `Unsigned` + direct/optional/callback/collection max-value round trips |
//! | owned `Option<opaque>` input         | `ingot_optional_grams`: null niche or consuming handle through the shared registry Optional chain |
//! | `Option<T>`                          | `Option<Payload>` (in + out) / `Option<Vec>` / `Option<i64>` / `Option<enum>` (param + return + field) |
//! | borrowed `Option<&data_class>`       | `payload_optional_borrow_id`: Kotlin-side flattening → registry-owned `Option<Payload>` carrier → final borrow |
//! | borrowed `&[T]` Sequence input      | `label_borrowed_concat`: registry Sequence chain builds an owned `Vec<Label>` carrier → final borrow |
//! | non-null enum field under nullable-context (#144) | `Option<CacheConfig>` → nested `RepliesConfig.priority` (single Elvis default) |
//! | `impl Fn` callbacks (single + slice) | `payload_handler_new` / `payload_vec_handler_new` |
//! | owned-handle callback (`Fn(Storage)`)| `storage_handler_new` / `storage_emit` |
//! | `Vec<handle>` / `Option<Vec<handle>>`| `storage_shards` / `storage_shards_opt` (Kotlin-side handle fold) |
//! | record-built `<A>` fold (bare + `Option`) | `summary_series` / `summary_series_opt` (caller `acc`/`fold`; `A?` return, null = `None`) |
//! | borrowed-opaque return (`Option<&T>`)| `archive_latest` (clone → fresh owned handle) |
//! | N-ary sorted handle locking          | `storage_total_len` (3 handles) + a 4-thread smoke |
//! | `Vec<String>` return                 | `storage_labels` (single-leaf string fold) |
//! | `String` return                      | `string_new` |
//! | fixed-size primitive arrays          | every JNI scalar element + `[u8; CONST_ARRAY_LEN]` from the renamed helper source |
//! | binding-error channel (`JniErrorHandler`) | wrong-length `[u8; 2]` (fixed-size array length guard) |
//! | callback no-throw contract           | a throwing `PayloadCallback` (described + cleared per upcall) |
//! | `data_class` instance member          | `Payload.labelLen()` (receiver crosses as `this` field leaves) |
//! | `JniGenBuilder::ignore` (exact)              | `string_len` / `storage_put_by_read_and_update` (acknowledged-unbound, no skip warnings) |
//! | `JniGenBuilder::ignore` + `matching(…)`      | the `storage_get_into_*` group (one name predicate, any item kind) |
//!
//! One feature is deliberately left at its default and documented rather than
//! toggled, because it is mutually exclusive with a richer path this example
//! prefers to keep covered:
//!   * `JniGenBuilder::set_emit_handle_locks` — kept ENABLED (default). Toggling
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

use prebindgen_jni::{
    constant, data_class, enum_class, matching, package, ptr_class, sealed_class, variant, JniGen,
};
use prebindgen_registry::{
    convert, expand_param, expand_return, expr, fields, from, fun, into, path, sig, try_from, ty,
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
    let binding = JniGen::builder()
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
        .source_named(cov_helpers::PREBINDGEN_OUT_DIR, "cov_helpers")
        // The two halves of what the row differential does with this binding's
        // decompositions: the ones it does not compare, each named with a stable
        // reason code, and how many it does. Every decomposition is in exactly one,
        // so a build fails if either moves — a decomposition leaving the comparison,
        // or leaving the population. Each entry is a part binding #701 step 3 owes.
        .expect_parity_skips([
            "`storage_labels`'s return: whole-element-fold",
            "`storage_shards_opt`'s return: whole-element-fold",
            "`storage_shards`'s return: whole-element-fold",
            "`storage_summary_probe`'s return: row-states-no-parts",
            "`unsigned_series`'s return: whole-element-fold",
            "the callback argument `Report`: consuming-value-form-handle-field",
        ])
        .expect_parity_compared(53)
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
        .set_method_name_mangle(|_, class, n| {
            // #86: force an underscored method name onto ONE harness extern —
            // the Kotlin `external fun` keeps this verbatim name while the
            // Rust export symbol needs the JNI `_1` escape
            // (`…_escape_1probe_1value`) to resolve at runtime.
            if class == "CovNative" && n == "escapeProbeValue" {
                return "escape_probe_value".to_string();
            }
            strip_flat_class_prefix(class, n)
        })
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
        // `Ticks`: a `u64` representation, so its Kotlin view is `ULong` and its
        // wire a `Long` — a leaf with a real conversion, which is what
        // `ticks_emit` needs to show that a callback argument's LAYERS are
        // peeled (#438). `u64` itself would have been the obvious leaf and
        // collides with the existing `impl Fn(u64)` interface name (#440).
        .convert(
            convert!(Ticks)
                .input(fun!(ticks_from_raw))
                .output(fun!(ticks_value)),
        )
        // `Celsius`: canonical conversion via `From`/`Into` impls in the flat
        // crate — the repr (`i32`) is stated, the impls do the work.
        .convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
        // `Percent`: fallible in BOTH directions. `Option<Percent>` below
        // forces both raw stage error types through the structural Option
        // converter, where they normalize to the binding error channel.
        .convert(
            convert!(Percent)
                .input(try_from!(i32))
                .output(fun!(crate::percent_out).sig(sig!((p: Percent) -> Result<i32, String>))),
        )
        // `Label`: conversions are plain fns in THIS binding crate (see
        // src/lib.rs) — no #[prebindgen], no helper crate — declared with
        // the ONE binding-local vocabulary, `fun!(crate::…).sig(sig!(…))`.
        // The input is FALLIBLE: the sig's `Result<Label, String>` return IS
        // the error channel (empty labels route the Err to onError).
        .convert(
            convert!(Label)
                .input(fun!(crate::label_in).sig(sig!((s: String) -> Result<Label, String>)))
                .output(fun!(crate::label_out).sig(sig!((l: Label) -> String))),
        )
        // Keep Rust's standard `Duration` as the semantic type while exposing
        // bounded u64 milliseconds. Values above one day are invalid; one of
        // those representations becomes the `Option<Duration>` None marker,
        // so nullable Kotlin `ULong?` crosses JNI as a raw primitive `Long`
        // without JObject/boxed-Long allocation.
        .convert(
            convert!(Duration)
                .input(fun!(crate::duration_from_millis).sig(sig!((ms: u64) -> Duration)))
                .output(
                    fun!(crate::duration_to_millis).sig(sig!((d: Duration) -> Result<u64, String>)),
                )
                .valid_range(0u64..=86_400_000u64),
        )
        // Output-only semantic wrapper over an owned opaque handle, used to
        // verify callback cleanup through an Optional(Product) chain.
        .convert(convert!(CallbackToken).output(fun!(callback_token_into_ingot)))
        .ignore(ty!(CallbackToken))
        // The helper source is private to this JVM covertest. Its array length
        // is a marked const, so final Rust emission must qualify the path with
        // the helper crate's configured `cov_helpers` name.
        .package(
            package!("model")
                .class(data_class!(ConstArray))
                .fun(fun!(const_array_echo)),
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
            package!()
                .class(
                    data_class!(Payload)
                        .interface()
                        .implements("io.prebindgen.covertest.Timestamped")
                        .method(fun!(payload_label_len)),
                )
                .fun(fun!(payload_optional_borrow_id)),
        )
        // `Option<Holder>` where `Holder` has a REQUIRED handle field: the
        // absent case passes pointer 0 for it, so the field decodes must stay
        // inside the presence gate or `null` becomes a binding error instead of
        // `None` (PR#294 review).
        .package(
            package!()
                .class(data_class!(Holder))
                .class(data_class!(CallbackHolder))
                .fun(fun!(callback_holder_optional_emit)),
        )
        // #218's last row: a data class that only REACHES a handle, through
        // another data class. `Dossier`'s cascade is a one-liner that assumes
        // `Holder` above was independently made `AutoCloseable` — the JVM
        // harness is what ties the two decisions together, since an emission
        // test never compiles the inner class.
        .package(package!().class(data_class!(Dossier)))
        // #430: the same handle field with an `Option` in front of it. The
        // present arm mints the handle, and it has to do so through the
        // generated factory — the constructor is private (#404). Only a
        // compiled run says which one the factory names, so the shape lives
        // here rather than only in an emission test.
        .package(package!().class(data_class!(MaybeHolder)))
        // A data class whose FIELDS carry transparent wrappers (#289 + #292):
        // `boxed: Box<Option<i64>>` must cross exactly as `plain: Option<i64>`
        // does — the decoupled `(present, value)` pair — with the `Box` put back
        // on the Rust side. Peeling the field by path segment answered "not
        // optional" and boxed it instead.
        .package(package!().class(data_class!(WrappedFields)))
        // ── Subpackage `model`: enum + data/sealed classes ─────────────────
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
                // `Reading` as a Kotlin `sealed interface` — a data-carrying
                // enum whose alternatives are nested classes, with the
                // payload-less one a `data object`. `variant!(Labeled)
                // .name("Tagged")` exercises the per-variant rename (which
                // also renames its `fromParts` slots).
                .class(sealed_class!(Reading).variant(variant!(Labeled).name("Tagged")))
                // `Lookup` is the sum in RETURN position whose groups own
                // resources: one alternative carries an opaque handle, one
                // carries nothing at all. Because it reaches an owned handle it
                // is emitted `AutoCloseable`, and each variant class overrides
                // `close()` — that is what lets a container cascade into it
                // with the same one-liner a handle field gets (#218).
                .class(sealed_class!(Lookup))
                // #429: the sum whose payloads have LAYERS — an optional
                // scalar, an optional handle, a list of optionals — beside two
                // controls that must not gain one. The builder that reassembles
                // them is Kotlin, so only a compiled run says whether each
                // layer was carried.
                .class(sealed_class!(Layered))
                // …and `Verdict` is that container: the sum in DATA-CLASS FIELD
                // position, the third place a handle is reached through.
                // `Holder` above is the plain-handle field it must match.
                .class(data_class!(Verdict))
                // `Report`'s output boundary is DERIVED from its value form
                // (`.fields_self_into(fields!(report_into_struct))` below) instead of
                // being restated field by field — #213. The form it names is
                // the CONSUMING one, so the fields are moved, not cloned.
                .class(ptr_class!(Report))
                // #220: the same derived-boundary idea with an `Option<sum>`
                // FIELD — the shape that used to be refused on a value form
                // while a `data_class` accepted it.
                .class(ptr_class!(Probe))
                // #142: two bounded-`convert!` leaves — one with a niche of its
                // own, one without — reached through a CONDITIONAL hoist, so both
                // are nullable. Only the niche-carrying one keeps a sentinel in
                // its wrap; the other's absence is the ancestor's `?.`.
                .class(ptr_class!(Span))
                .class(ptr_class!(SpanHolder))
                // #433: the same matrix on an opaque handle leaf, whose niche
                // is `0L` rather than a declared sentinel. `VaultHolder` is what
                // reaches the fourth row — an `Option<handle>` under an absent
                // ancestor — where the descriptor and the encoder can disagree
                // while both halves still compile.
                .class(ptr_class!(Vault))
                .class(ptr_class!(VaultHolder))
                .class(
                    ptr_class!(Ingot)
                        .constructor(fun!(ingot_new))
                        .method(fun!(ingot_grams)),
                )
                .fun(fun!(ingot_optional_grams))
                // `Hold`'s payload is a CONVERTED type, so its leaf crosses
                // through the `convert!(Duration)` chain; `HoldPolicy` puts
                // that same payload in the data-class-field position.
                .class(sealed_class!(Hold))
                .class(data_class!(HoldPolicy))
                // `Observation` carries that sum as a data-class FIELD —
                // required (`reading`) and optional (`fallback`) — beside
                // ordinary flattened leaves, so the tag-gated groups must
                // interleave with them on one `fromParts` call.
                .class(data_class!(Observation))
                // `Marker`'s `Option<enum>` payload uses one primitive jint;
                // an enum niche represents null while the sum tag independently
                // identifies the active variant.
                .class(sealed_class!(Marker))
                .class(data_class!(Tagged))
                // `Annotated` exercises a NESTED data-class field (`payload`,
                // recursive fromParts / recursive leaf decode) plus Option<prim> and
                // Option<enum> FIELDS (one jint carrying an unused discriminant for null).
                .class(data_class!(Annotated))
                // #144: a NON-NULL enum field (`RepliesConfig.priority`) reached
                // through an outer `Option<CacheConfig>` param. The outer
                // optional propagates `nullable_context` into the non-optional
                // nested struct, so its enum field must decode with exactly one
                // Elvis default (regression guard for the dead `?: 0 ?: 0`).
                .class(data_class!(RepliesConfig))
                .class(data_class!(CacheConfig))
                // Compose the bounded `Option<Duration>` niche through a
                // data-class field. Explicit JObject input makes the runtime
                // execute the whole-object decoder as well as the primitive-
                // niche `fromParts` encoder (#138).
                .class(data_class!(DurationBoundary).jobject_input())
                // These small nested classes form a 127-Long-leaf tree. Its
                // constructor is legal, but flattening the root function input
                // would consume 256 JVM slots, so it keeps one JObject input.
                .class(data_class!(ObjectBoundaryLeaf))
                .class(data_class!(ObjectBoundary2))
                .class(data_class!(ObjectBoundary4))
                .class(data_class!(ObjectBoundary8))
                .class(data_class!(ObjectBoundary16))
                .class(data_class!(ObjectBoundary32))
                .class(data_class!(ObjectBoundary64))
                .class(data_class!(ObjectBoundary63))
                .class(data_class!(ObjectBoundary).jobject_input())
                // Fixed-width unsigned mappings: Int / Long widening plus
                // ULong over a raw jlong bit pattern.
                .class(data_class!(Unsigned))
                // `Stamp` is a small `Copy` struct of two scalars, so it crosses
                // as its FIELDS — no array, no raw-memory image. Its readers stay
                // instance methods (`secs()` / `nanos()`) whose receiver crosses
                // as those field leaves, and `Vec<Stamp>` surfaces as
                // `List<Stamp>`.
                // The optional nested class #602 names first: `Envelope`'s
                // stamp is sometimes there, and the decomposition says so with
                // a presence flag ahead of the child's leaves.
                .class(data_class!(Envelope))
                .class(data_class!(Window))
                .class(data_class!(Frame))
                .class(data_class!(Meter))
                .class(data_class!(Rack))
                .class(
                    data_class!(Stamp)
                        .method(fun!(stamp_secs))
                        .method(fun!(stamp_nanos)),
                )
                // `BlobValue` is the array-backed EQUALITY probe: a raw-bytes
                // field beside a scalar, plus a nested data class. Both compare
                // by identity in Kotlin unless the binding says otherwise.
                .class(data_class!(BlobValue).jobject_input())
                // Fixed-size arrays of every JNI-primitive element.
                .class(data_class!(Arrays)),
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
                // `expand_param!` / `expand_return!` below. It is also the
                // `.gc_managed()` exercise: unreachable Summary handles are
                // freed by the shared Cleaner; close/take/by-value consumption
                // settle the release ticket first (see the Test.kt section).
                .class(
                    ptr_class!(Summary)
                        .gc_managed()
                        .constructor(fun!(summary_new).name("of"))
                        .method(fun!(summary_count))
                        .method(fun!(summary_total))
                        .method(fun!(summary_scaled))
                        // Binding-local INSTANCE METHOD and COMPANION
                        // CONSTRUCTOR (`fun!(crate::…).sig(sig!(…))`): fns
                        // defined in THIS crate (src/lib.rs), no source-crate
                        // item — same member machinery as registry fns.
                        // NO .name(): the strip-class-prefix method hook
                        // derives `mean` from the path's LAST segment
                        // (`summary_mean` on `Summary` → strip → `mean`) —
                        // automatic mangling covers binding-local fns too.
                        .method(fun!(crate::summary_mean).sig(sig!((s: &Summary) -> f64)))
                        // FALLIBLE binding-local constructor: the sig's
                        // `Result<Summary, String>` return is the error
                        // channel — a negative count routes the Err message
                        // to onError, exactly like a registry fn's Result.
                        .constructor(
                            fun!(crate::summary_from_mean)
                                .sig(sig!((count: i64, mean: f64) -> Result<Summary, String>)),
                        ),
                )
                // A multi-variant Optional input whose required `u16` build-arm
                // leaf is nullable on the public surface. Its synthetic
                // `Option<u16>` reading must still select the allocation-free
                // registry pair recipe at the native ABI (#525 follow-up).
                .class(ptr_class!(SelectorCode).constructor(fun!(selector_code_new)))
                .fun(fun!(selector_code_score))
                // `SummaryEnvelope` has no Kotlin class. Its sole constructor
                // takes `Summary`, so the registry recursively folds Summary's
                // own selector expansion before building the outer Rust value.
                .fun(fun!(summary_envelope_score))
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
        .expand(expand_param!(SummaryEnvelope).variant(fun!(summary_envelope_new)))
        .expand(
            expand_param!(SelectorCode)
                .variant(fun!(selector_code_new))
                .variant_self(),
        )
        // `Summary` default output: decomposed `(count, total)` leaves, names
        // inherited from the class members.
        .expand(
            expand_return!(Summary)
                .field(fun!(summary_count))
                .field(fun!(summary_total)),
        )
        // `Report` default output DERIVED from its value form (#213): the
        // leaves come from `ReportStruct`'s fields, so the list cannot drift
        // from the struct the way a restated one does. Each field still crosses
        // by ITS OWN type's boundary — `summary` splices `Summary`'s decl above
        // into `(count, total)` rather than becoming a handle, `origin` inlines
        // its `Stamp` fields, `taken` stays one `Stamp?` leaf, and `outcome`
        // decomposes into a selector plus one group per alternative.
        .expand(expand_return!(Report).fields_self_into(fields!(report_into_struct)))
        // #142: `Span`'s value form is two bounded leaves; `SpanHolder` reaches
        // it through an `Option`, so both become nullable. `delay` keeps its own
        // niche sentinel, `required` takes none — a sentinel is the leaf's own
        // `None`, never an ancestor's.
        .expand(expand_return!(Span).fields(fields!(span_to_struct)))
        .expand(expand_return!(SpanHolder).field(fun!(span_holder_span)))
        .expand(expand_return!(Vault).fields(fields!(vault_to_struct)))
        .expand(expand_return!(VaultHolder).field(fun!(vault_holder_vault)))
        // #220: `ProbeStruct.outcome` is `Option<Lookup>`. Its whole segment
        // gates together — one tuple bind whose absent arm defaults every slot
        // — because a sum's leaves are not independent. The selector boxes, so
        // JVM null is "no sum" and cannot be read as `Lookup.Absent` (tag 0).
        .expand(expand_return!(Probe).fields(fields!(probe_to_struct)))
        // `Ledger` reaches that same derived boundary through an `Option`, so
        // `Report`'s value form is hoisted CONDITIONALLY — built in the `Some`
        // arm, its leaves (the sum among them) sharing one `match`, null in the
        // absent arm. The two accessors differ in what the arm binds: `filed`
        // hands over a borrow, `archived` an owned report the by-value form
        // takes directly — `Report` is not `Clone`, so cloning it here would not
        // compile.
        .expand(
            expand_return!(Ledger)
                .field(fun!(ledger_filed))
                .field(fun!(ledger_archived)),
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
        // ── JNI native-symbol escaping probe (#86) ──────────────────────────
        // Underscores in EVERY symbol component: the `esc_pkg` subpackage and
        // the `Esc_Probe` class name put `_1` escapes into the `freePtr`
        // destructor symbol (`Java_…_esc_1pkg_Esc_1Probe_freePtr`), and the
        // method-mangle hook above puts one into the accessor's harness
        // extern (`…_escape_1probe_1value`). Kotlin names stay verbatim; the
        // JVM only resolves these if the generator escapes per the JNI spec.
        .package(
            package!("esc_pkg").class(
                ptr_class!(EscapeProbe)
                    .name("Esc_Probe")
                    .constructor(fun!(escape_probe_new))
                    .method(fun!(escape_probe_value)),
            ),
        )
        // ── Free functions, grouped by subpackage ───────────────────────────
        // model: enum return/param/option + value-class return + Vec<value> +
        //        Option<scalar>.
        .package(
            package!("model")
                .fun(fun!(payload_priority))
                .fun(fun!(priority_weight))
                .fun(fun!(priority_or))
                .fun(fun!(priority_nested))
                .fun(fun!(priority_nested_state))
                .fun(fun!(stamp_new))
                .fun(fun!(stamp_series))
                // The three convert!-source-kind fns (conversions declared
                // below): Into/From traits, TryFrom trait, binding-local fns.
                .fun(fun!(celsius_double))
                .fun(fun!(percent_scale))
                .fun(fun!(percent_optional))
                .fun(fun!(percent_invalid_output))
                .fun(fun!(label_reverse))
                .fun(fun!(label_series_echo))
                .fun(fun!(label_borrowed_concat))
                .fun(fun!(annotated_new))
                .fun(fun!(annotated_alternate_value))
                .fun(fun!(annotated_ttl))
                .fun(fun!(annotated_priority))
                .fun(fun!(annotated_payload_value))
                // A sum as a data-class field, crossing OUT: the tag plus every
                // variant's group ride the parent's single `fromParts`.
                .fun(fun!(observation_new))
                .fun(fun!(observation_which))
                .fun(fun!(tagged_new))
                .fun(fun!(tagged_rank))
                // The same niche-backed `Option<enum>` payload in RETURN
                // position exercises the hoisted sum builder.
                .fun(fun!(marker_of))
                // A sum as the function's OWN return (and callback argument):
                // the tag + groups ride the hoisted builder / folder singleton
                // instead of a parent's `fromParts`. All four positions —
                // bare, `Option`, `Vec`, callback — plus a group owning a
                // native handle.
                .fun(fun!(reading_of))
                .fun(fun!(reading_maybe))
                .fun(fun!(reading_series))
                .fun(fun!(reading_each))
                .fun(fun!(lookup_of))
                // #161: the two positions the four above do not reach — a
                // handle-carrying sum arriving through a CALLBACK, and a sum
                // returned BORROWED (`&E` / `Option<&E>`).
                .fun(fun!(lookup_each))
                // …and #429's layered payloads, one call per case.
                .fun(fun!(layered_of))
                // #438: the same layers as a CALLBACK ARGUMENT, which is a
                // different emitter and kept its own one-shot wrap.
                .fun(fun!(ticks_emit))
                // #218: the same handle reached through a data-class FIELD, so
                // the JVM harness can assert the container's cascade closes it.
                .fun(fun!(verdict_new))
                // …and the same data class handed to a CALLBACK, which
                // reassembles through the interface path rather than the
                // return path — a different question about the same leaves
                // (#616 review).
                .fun(fun!(verdict_each))
                // The optional nested class on both delivery routes.
                .fun(fun!(envelope_new))
                .fun(fun!(envelope_each))
                .fun(fun!(frame_new))
                .fun(fun!(frame_each))
                .fun(fun!(rack_new))
                .fun(fun!(rack_each))
                // …and reached one level deeper still, through a nested data
                // class rather than a sum.
                .fun(fun!(dossier_new))
                // #430: the handle field with an `Option` in front of it. The
                // return is what runs the factory, so both arms — minted and
                // absent — are compiled and exercised.
                .fun(fun!(maybe_holder_new))
                // #213: the output boundary DERIVED from the type's value form
                // rather than restated. `report_each` delivers the decomposed
                // `Report` in one crossing; the leaf list comes from
                // `ReportStruct`'s fields, so it cannot drift from it.
                .fun(fun!(report_each))
                // #220: the value form whose `outcome` field is `Option<sum>`.
                // `probe_each` walks the absent case and every alternative, so
                // one crossing covers the gate and the groups it gates.
                .fun(fun!(probe_new))
                .fun(fun!(probe_each))
                // The same derived boundary reached through an `Option` — a
                // CONDITIONAL hoist. `ledger_each` delivers both at once, so one
                // crossing covers the borrowed payload, the owned one, and the
                // sum each report carries.
                .fun(fun!(ledger_each))
                // #142: the holder whose span is optional.
                .fun(fun!(span_holder_new))
                .fun(fun!(vault_holder_new))
                // A transparent wrapper (`Box<Option<String>>`) in and out. The
                // model erases the `Box`, so this must cross exactly as a
                // `String?` — and because this crate compiles its generated
                // binding, a converter that named the wrong type or bridged it
                // with the wrong number of dereferences fails the build (#270).
                .fun(fun!(boxed_note_echo))
                .fun(fun!(plain_note_echo))
                // Transparent wrappers on the INPUT side (#292 item 3), one per
                // specialized lowering. These rebuild their parameter rather
                // than decoding it, so the erased wrapper has to go back on
                // before the value reaches the signature — and each layer is
                // applied at a different point in the construction, which is why
                // one shape cannot cover them all. Compiling this crate is the
                // check: a missing `Box::new` is an `E0308`, invisible to any
                // text assertion.
                .fun(fun!(wrapped_fields_sum))
                .fun(fun!(holder_tag_or))
                .fun(fun!(boxed_payload_id))
                .fun(fun!(boxed_opt_payload_id))
                .fun(fun!(boxed_opt_priority_weight))
                .fun(fun!(boxed_elem_id_sum))
                .fun(fun!(boxed_run_id_sum))
                // The two spellings of a borrowed run (#384). One type to the
                // model, so both take the Vec-build path — and the compiled
                // binding is what proves the emitter serves both, since the
                // failure was an `E0308` rather than a wrong answer.
                .fun(fun!(slice_id_sum))
                .fun(fun!(ref_vec_id_sum))
                // The same wrapper over a DECOMPOSED return (#292). `Summary`
                // has an output expansion, so this return takes no converter to
                // name the spelling for it — the extern binds the value and
                // matches it, and a `Box` match ergonomics cannot see through
                // is an `E0308` that only compiling this crate catches.
                .fun(fun!(boxed_latest))
                .fun(fun!(ledger_new))
                .fun(fun!(archive_set_reading))
                .fun(fun!(archive_reading))
                .fun(fun!(archive_reading_maybe))
                // A sum payload that is a CONVERTED type (`convert!(Duration)`),
                // so its boundary conversion is a chain rather than one wire
                // converter — exercised as the function's own return and as a
                // (required + optional) data-class field, the two encoders.
                .fun(fun!(hold_echo))
                .fun(fun!(hold_policy_echo))
                // #144: `Option<CacheConfig>` input reaching a non-null enum
                // field through the nested `RepliesConfig`.
                .fun(fun!(cache_config_weight))
                .fun(fun!(object_boundary_value))
                .fun(fun!(unsigned_round_trip))
                .fun(fun!(unsigned_optional))
                .fun(fun!(unsigned_data_maybe))
                .fun(fun!(unsigned_emit))
                .fun(fun!(unsigned_series))
                .fun(fun!(blob_value_new))
                .fun(fun!(blob_value_echo))
                .fun(fun!(arrays_echo))
                .fun(fun!(duration_optional))
                .fun(fun!(boxed_duration_echo))
                .fun(fun!(duration_boundary_echo))
                // The converted analogue of `unsigned_emit`: a whole-value
                // callback argument, which encodes on its own path rather than
                // through the data-class or sum emitters.
                .fun(fun!(duration_emit))
                .fun(fun!(duration_out_of_range)),
        )
        // analytics: the param-variant / return-field matrix (type default /
        // per-fn override, in + out). Per-fn overrides reuse the SAME
        // expand-decl objects as the type defaults (complete-set rule): an
        // identity-only set is the plain form.
        .package(
            package!("analytics")
                .fun(fun!(storage_summary))
                // Binding-local FREE FUNCTION: exported like any package fn;
                // its `&Summary` param resolves through the ordinary borrow
                // converter, its String return through the ordinary output
                // converter.
                .fun(
                    fun!(crate::summary_describe)
                        .sig(sig!((s: &Summary, verbose: bool) -> String))
                        .name("describeSummary"),
                )
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
                // Binding-local CONDITIONAL field (`field!` + `.with(ty, path)`):
                // the handle leaf is delivered only when the binding-side
                // predicate (`crate::summary_if_nonempty`, src/lib.rs) says
                // re-using the value is worth it — nullable identity leaf,
                // null when the condition fails. The condition is binding
                // policy, so it lives in THIS crate, not the source crate.
                .fun(
                    fun!(storage_summary_probe).expand_return(
                        expand_return!(Summary)
                            .field(fun!(summary_count).name("count"))
                            .field(fun!(summary_total).name("total"))
                            .field(
                                fun!(crate::summary_if_nonempty)
                                    .sig(sig!((s: &Summary) -> Option<&Summary>))
                                    .name("handle"),
                            ),
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
                // Split × builder-delivered return (#87): both params split AND
                // the `Summary` return crosses via the decomposed builder, so
                // the wrapper is generic — every overload must re-declare `<R>`.
                .fun(
                    fun!(summary_merge)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                )
                // Optional combined-selector expansion: `Option<&Summary>` under
                // the dual-arm type default — the selector also encodes absence
                // (`-1` = `None`); the borrow-identity arm clones, so the
                // caller's handle survives the call.
                .fun(fun!(summary_total_opt))
                // Record-built generic folds (#105): `Vec<Summary>` and
                // `Option<Vec<Summary>>` returns cross as the caller-supplied
                // `<A>(acc, fold)` surface — decomposed `(count, total)`
                // leaves per element. The `Option` form returns `A?`: null =
                // `None` (the fold never invoked), `Some(empty)` = the
                // untouched accumulator.
                .fun(fun!(summary_series))
                .fun(fun!(summary_series_opt))
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
                // Two-caller error split (#45): both channels on one wrapper.
                .fun(fun!(storage_try_from_stamp))
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
                // Option<data-class> callback output in both presence states.
                .fun(fun!(payload_optional_emit))
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

    // Two prebindgen sources: the flat crate plus the binding-side helper crate
    // (conversion fns for `convert!`). The registry records each fn's origin from
    // the `SourceLocation` stamps so generated calls qualify with the defining
    // crate (`perftest_flat::…` vs `cov_helpers::…`). The helper dependency is
    // RENAMED in Cargo.toml (`cov_helpers = { package = "covertest-helpers", .. }`),
    // so the stamp recorded at capture time (`covertest-helpers`) would not
    // resolve from this crate — `source_named` overrides it with the name this
    // crate actually uses, per directory.
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
    let rust_dest = std::path::Path::new(&crate_dir)
        .join("src")
        .join("generated_bindings.rs");
    let jni = binding.build().expect("build failed");
    let rust_path = jni
        .write_rust(&rust_dest)
        .unwrap_or_else(|error| panic!("write_rust failed: {error}"));
    println!(
        "cargo:warning=Generated bindings at: {}",
        rust_path.display()
    );

    // Kotlin classes → kotlin/generated/** (picked up by the Gradle source set).
    let kotlin_root = std::path::Path::new(&crate_dir)
        .join("kotlin")
        .join("generated");
    // The root is prebindgen-owned: `write_kotlin` replaces marked output,
    // so no consumer-side cleanup is needed.
    for path in jni.write_kotlin(&kotlin_root).expect("write_kotlin failed") {
        println!("cargo:warning=Wrote {}", path.display());
    }

    // The resolved-surface report (C7): committed next to the regen so a
    // decl's effect is reviewable in a PR without reading generated Kotlin.
    std::fs::write(
        std::path::Path::new(&crate_dir)
            .join("kotlin")
            .join("REPORT.md"),
        jni.report(),
    )
    .expect("write REPORT.md");
}
