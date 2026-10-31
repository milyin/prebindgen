//! Active coverage: primitives, strings/bytes, owned handles, selectors and field outputs.
//! The original coverage inventory below includes features now commented out.
//!
//! Unsupported declarations remain commented below.
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
//! | Optional combined-selector expansion  | `summary_total_opt(Option<&Summary>)` — selector `-1` = absent, borrow-identity arm clones |
//! | `FunctionDecl::split_on_param` (#52)  | single: `archiveStore`/`storageMatchesSummary` (class-default) + `storageExpectSummary` (per-fn); cartesian product: `summaryPrefer` (2 params); manual same-named overload in `ManualOverloads.kt` |
//! | split × builder-delivered return (#87) | `summaryMerge` — cartesian split + generic `<R>` wrapper; every overload re-declares `<R>` |
//! | JNI native-symbol escaping (#86)      | `esc_pkg.Esc_Probe` — underscored subpackage + class (escaped `freePtr` symbol) + hook-mangled `escape_probe_value` harness extern |
//! | `expand_return!` `.field()` (+`_self`) | `Summary` fields + `StorageError` `message` + self (error handle → `onError`) |
//! | `expand_return!` `.fields(fields!(…))` (#213) | `Report` — boundary DERIVED from the value form instead of restated; covers every per-field rule (spliced `Summary`, inlined `Stamp`, `Option<data class>`, a sum with a handle payload, a plain leaf) |
//! | `expand_return!` `.fields_into(fields!(…))` | `report_into_struct(r: Report)` — the CONSUMING value form: the value is given away and its fields MOVED out, so the clones the borrowing `report_to_struct` pays are not emitted at all |
//! | `PackageDecl::fun` / `FunctionDecl::name`| every free function; `.name` renames `millis_add` → `addMillis` |
//! | `JniGen::report()` (C7)               | `kotlin/REPORT.md` — the resolved surface, committed next to the regen |
//! | contextual method names               | method hook strips `storage`/`stamp` class prefixes; `summary_new`→`.name("of")` still overrides |
//! | per-class `.name()`                  | `Summary` → Kotlin `SummaryValue` (literal, bypasses mangles) |
//! | `.interface()` + `.implements(…)`      | `Storage`/`Payload` emit an Api interface; `CovResource`/`Timestamped` extend it (#54) |
//! | `.interface_name(…)`                  | `Priority` → generated `PriorityKind` interface (#54) |
//! | base-package functions               | `string_new` (declared in a `package!()`) |
//! | `constant!` (bare = `#[prebindgen]` const) | `COVER_MAGIC` (`Long`) + `COVER_TAG` (`String`) → top-level `val`s |
//! | `constant!(N).fun(fun!(…))`           | `cover_tag_runtime()` → eagerly-initialized `val COVER_TAG_RUNTIME` |
//! | `constant!(N).with(ty!, path!)`       | `val COVER_VERSION` from binding-local `crate::cover_version()` |
//! | `constant!(N).expr(ty!, expr!)`       | `COVER_BANNER` = binding-defined `format!` expression |
//! | per-fn `.expand_param(name, …)` identity-only | `summary_total_raw` (raw handle param, overrides the type default) |
//! | per-fn `.expand_return(…)` identity-only | `storage_summary_handle` (raw handle return) |
//! | per-fn `.expand_param(name, …)` variants | `storage_expect_summary` |
//! | per-fn `.expand_return(…)` fields+self | `storage_summary_full` |
//! | binding-local field `fun!(crate::…).sig(sig!).name(…)` | `storage_summary_probe` — custom field, here a conditional handle via `crate::summary_if_nonempty` |
//! | binding-local fn `fun!(crate::…)` `.sig(sig!)` as free fn | `describeSummary` ← `crate::summary_describe` |
//! | binding-local fn as `.method()` / `.constructor()` | `Summary.mean()` ← `crate::summary_mean` (NO `.name` — derived by the strip hook); `Summary.fromMean` ← `crate::summary_from_mean` (FALLIBLE — sig `Result` → `onError`) |
//! | `Result<_, E>` → typed domain `onError` | `storage_try_with_label` |
//! | two-caller split (#45): `onBindingError` + `onError` on one fallible wrapper | `storage_try_from_stamp` (wrong-length `tag` → binding; bad `secs` → domain) |
//! | fixed-width unsigned scalars (#108) | `Unsigned` + direct/optional/callback/collection max-value round trips |
//! | `Option<T>`                          | `Option<Payload>` (in + out) / `Option<Vec>` / `Option<i64>` / `Option<enum>` (param + return + field) |
//! | non-null enum field under nullable-context (#144) | `Option<CacheConfig>` → nested `RepliesConfig.priority` (single Elvis default) |
//! | `impl Fn` callbacks (single + slice) | `payload_handler_new` / `payload_vec_handler_new` |
//! | owned-handle callback (`Fn(Storage)`)| `storage_handler_new` / `storage_emit` |
//! | `Vec<handle>` / `Option<Vec<handle>>`| `storage_shards` / `storage_shards_opt` (Kotlin-side handle fold) |
//! | record-built `<A>` fold (bare + `Option`) | `summary_series` / `summary_series_opt` (caller `acc`/`fold`; `A?` return, null = `None`) |
//! | borrowed-opaque return (`Option<&T>`)| `archive_latest` (clone → fresh owned handle) |
//! | N-ary sorted handle locking          | `storage_total_len` (3 handles) + a 4-thread smoke |
//! | `Vec<String>` return                 | `storage_labels` (single-leaf string fold) |
//! | `String` return                      | `string_new` |
//! | `&String` param (converted, then borrowed) | `string_len` |
//! | shared primitive param (`&i64`) as a Kotlin scalar | `i64_ref_double` |
//! | `variant_self()` / `field_self()` on `String` and `Vec<u8>` → text/bytes | `text_echo` / `bytes_echo` |
//! | binding-error channel (`JniErrorHandler`) | wrong-length `[u8; 2]` (fixed-size array length guard) |
//! | callback no-throw contract           | a throwing `PayloadCallback` (described + cleared per upcall) |
//! | `data_class` instance member          | `Payload.labelLen()` (receiver crosses as `this` field leaves) |
//! | `JniGenBuilder::ignore` (exact)              | `storage_put_by_read_and_update` (acknowledged-unbound, no skip warnings) |
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
//! Some functions are deliberately NOT wrapped — their shapes are C-tier
//! with no JVM mapping (the `storage_get_into_*` out-param group,
//! `storage_put_by_read_and_update`'s read-write borrow). The loner is
//! acknowledged by name via `.ignore(fun!(…))`; the `storage_get_into_*` naming family via one
//! `.ignore(matching(…))` predicate. Both suppress the per-item
//! "skipping undeclared" build warning while emitting nothing.

/* Unsupported: imports and helpers for the disabled declarations.
use prebindgen_jni::{
    constant, data_class, enum_class, matching, package, ptr_class, sealed_class, variant, JniGen,
};
use prebindgen_declarations::{
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


*/
use prebindgen_declarations::{expand_param, expand_return, fields, fun};
use prebindgen_jni::{callback, data_class, enum_class, package, ptr_class, Input, JniGen};

fn main() {
    let getter_views = || {
        expand_return!(GetterOwner)
            .field(fun!(getter_owner_child))
            .field(fun!(getter_owner_optional))
            .field(fun!(getter_owner_container))
            .field(fun!(getter_owner_rows))
            .field(fun!(getter_owner_leaf))
    };
    let wide_buffer_views = (0..48).fold(expand_return!(JniBufferRow), |views, _| {
        views.field(fun!(jni_buffer_row_text))
    });
    let wide_return = (0..130).fold(expand_return!(i64), |views, _| views.field_self_ref());
    let binding = JniGen::builder()
        .package(package!("collectioninputs")
            .fun(fun!(collection_input_plain))
            .fun(fun!(collection_input_bulk).expand_param("values", expand_param!(Vec<i64>).variant_self().bulk()))
            .fun(fun!(collection_input_custom).expand_param("values", expand_param!(i64).variant(fun!(collection_input_value))))
            .fun(fun!(collection_input_custom_bulk).expand_param("values", expand_param!(i64).variant(fun!(collection_input_value)).bulk()))
            .fun(fun!(collection_input_narrow).expand_param("values", expand_param!(i64).variant(fun!(collection_input_narrow_value))))
            .fun(fun!(collection_input_narrow_bulk).expand_param("values", expand_param!(i64).variant(fun!(collection_input_narrow_value)).bulk()))
            .fun(fun!(collection_input_constructions))
            .fun(fun!(collection_input_slice))
            .fun(fun!(collection_input_array))
            .fun(fun!(collection_input_optional_array))
            .fun(fun!(collection_input_optional_array_bulk).expand_param("values", expand_param!(Option<[i64; 4]>).variant_self().bulk()))
            .fun(fun!(collection_input_optional).expand_param("values", expand_param!(Option<Vec<i64>>).variant_self().bulk())))
        .package(package!("defaults")
            .class(data_class!(DefaultBulkRow))
            .fun(fun!(default_bulk_echo))
            .fun(fun!(default_bulk_array).callback_return())
            .fun(fun!(default_bulk_optional).callback_return())
            .fun(fun!(default_bulk_nested))
            .fun(fun!(default_bulk_borrowed))
            .fun(fun!(default_bulk_numbers))
            .fun(fun!(default_bulk_units))
            .fun(fun!(default_bulk_texts))
            .fun(fun!(default_bulk_optional_texts).expand_return(expand_return!(String).field_self_ref()))
            .fun(fun!(default_bulk_buffers))
            .fun(fun!(default_bulk_plain).expand_return(expand_return!(Vec<DefaultBulkRow>).field_self()))
            .fun(fun!(default_bulk_callback))
            .fun(fun!(default_bulk_callback_mixed).expand_param("callback",
                callback!(impl Fn(Vec<Vec<i64>>,Vec<Vec<i64>>)).name("DefaultBulkMixedCallback")
                    .expand_param(0,expand_return!(Vec<Vec<i64>>).field_self()))))
        .expand(expand_param!(BulkPrimitives).variant_self().bulk())
        .expand(expand_return!(BulkPrimitives).field_self_into().bulk())
        .expand(expand_param!(BulkNative).variant_self().bulk())
        .expand(expand_return!(BulkNative).field_self_into().bulk())
        .expand(expand_param!(BulkExpanded).variant(fun!(bulk_expanded_from)).bulk())
        .expand(expand_return!(BulkExpanded).field(fun!(bulk_expanded_number)).field(fun!(bulk_expanded_text)).bulk())
        .expand(expand_return!(BulkFailure).field(fun!(bulk_failure_code)).bulk())
        .expand(expand_param!(BulkPoint).variant_self().bulk())
        .expand(expand_return!(BulkPoint).field_self_into().bulk())
        .package(package!("bulk")
            .class(data_class!(BulkPoint)).class(data_class!(BulkEnvelope)).class(data_class!(BulkPrimitives)).class(enum_class!(BulkKind))
            .class(data_class!(BulkNative)).class(data_class!(BulkExpanded))
            .class(ptr_class!(BulkHandle))
            .class(data_class!(BulkHandleText)).class(data_class!(BulkPairText))
            .fun(fun!(bulk_pair_text_callback).expand_param("callback",callback!(impl Fn()->Vec<BulkPairText>).name("BulkPairTextCallback").expand_return(Input::Sequence(Box::new(Input::Record(vec![Input::Constructor{function:"bulk_handle_copy".into(),args:vec![Input::Leaf]},Input::Leaf,Input::Leaf]))).bulk())))
            .fun(fun!(bulk_handle_texts).expand_param("values",Input::Sequence(Box::new(Input::Record(vec![Input::Leaf;2]))).bulk()))
            .fun(fun!(bulk_collection_lock_groups).expand_param("probe",callback!(impl Fn()->bool).name("CollectionLocksProbe")))
            .fun(fun!(bulk_handle_nested).expand_param("values",Input::Sequence(Box::new(Input::Sequence(Box::new(Input::Leaf)).bulk()))))
            .fun(fun!(bulk_handle_text_callback).expand_param("callback",callback!(impl Fn()->Vec<BulkHandleText>).name("BulkHandleTextCallback").expand_return(Input::Sequence(Box::new(Input::Record(vec![Input::Leaf;2]))).bulk())))
            .fun(fun!(bulk_optional_borrow_pair).expand_param("value",Input::Optional(Box::new(Input::Constructor {function:"bulk_borrow_pair_new".into(),args:vec![Input::Leaf;2]})).bulk()))
            .fun(fun!(bulk_select).expand_param("value",expand_param!(i64).variant(fun!(bulk_scalar)).variant_self().bulk()).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_kinds).expand_param("value",expand_param!(BulkKind).variant_self().bulk()).expand_return(expand_return!(BulkKind).field_self().bulk()))
            .fun(fun!(bulk_units).expand_param("value",expand_param!(Vec<()>).variant_self().bulk()).expand_return(expand_return!(Vec<()>).field_self_into().bulk()))
            .fun(fun!(bulk_flags).expand_param("value",expand_param!(bool).variant_self().bulk()).expand_return(expand_return!(bool).field_self().bulk()))
            .fun(fun!(bulk_envelope)).fun(fun!(bulk_primitives))
            .fun(fun!(bulk_optional_wide).expand_param("value",expand_param!(i128).variant_self().bulk()).expand_return(expand_return!(i128).field_self().bulk()))
            .fun(fun!(bulk_optional_callback).expand_param("value",callback!(impl Fn()->Option<Vec<BulkHandle>>).name("BulkOptionalCallback").expand_return(expand_param!(BulkHandle).variant_self().bulk())))
            .fun(fun!(bulk_native)).fun(fun!(bulk_expanded)).fun(fun!(bulk_expanded_rows).expand_param("values",expand_param!(BulkExpanded).variant_self().bulk()).expand_return(expand_return!(BulkExpanded).field_self_into().bulk())).fun(fun!(bulk_result))
            .fun(fun!(bulk_handle_new)).fun(fun!(bulk_handle_value)).fun(fun!(bulk_drops))
            .fun(fun!(bulk_late).expand_return(expand_return!(BulkLate).field(fun!(bulk_late_handle)).bulk()))
            .fun(fun!(bulk_handle_views).expand_return(expand_return!(BulkHandle).field_self_into().field(fun!(bulk_handle_value)).bulk()))
            .fun(fun!(bulk_handles).expand_param("values",expand_param!(BulkHandle).variant_self().bulk()).expand_return(expand_return!(BulkHandle).field_self_into().bulk()))
            .fun(fun!(bulk_borrow).expand_param("value",expand_param!(&BulkHandle).variant_self().bulk()))
            .fun(fun!(bulk_checked).expand_param("value",expand_param!(BulkHandle).variant_self().bulk()).expand_param("number",expand_param!(u8).variant_self().bulk()).expand_return(expand_return!(BulkHandle).field_self_into().bulk()))
            .fun(fun!(bulk_wide).expand_param("values",expand_param!(i128).variant_self().bulk()).expand_return(expand_return!(i128).field_self().bulk()))
            .fun(fun!(bulk_unsigned_wide).expand_param("values",expand_param!(u128).variant_self().bulk()).expand_return(expand_return!(u128).field_self().bulk()))
            .fun(fun!(bulk_callback_worker).expand_param("value",callback!(impl Fn(Vec<i64>)->Vec<i64>+Send+'static).name("BulkWorkerCallback").expand_param(0,expand_return!(i64).field_self().bulk()).expand_return(expand_param!(i64).variant_self().bulk())).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_callback).expand_param("value",callback!(impl Fn(Vec<i64>)->Vec<i64>).name("BulkNumbersCallback").expand_param(0,expand_return!(i64).field_self().bulk()).expand_return(expand_param!(i64).variant_self().bulk())).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_handle_callback).expand_param("value",callback!(impl Fn(Vec<BulkHandle>)->Vec<BulkHandle>).name("BulkHandlesCallback").expand_param(0,expand_return!(BulkHandle).field_self_into().bulk()).expand_return(expand_param!(BulkHandle).variant_self().bulk())).expand_return(expand_return!(BulkHandle).field_self_into().bulk()))
            .fun(fun!(bulk_point)).fun(fun!(bulk_points)).fun(fun!(bulk_point_array)).fun(fun!(bulk_slice))
            .fun(fun!(bulk_point_plain).expand_param("value",expand_param!(BulkPoint).variant_self()).expand_return(expand_return!(BulkPoint).field_self_into()))
            .fun(fun!(bulk_optional_handles).expand_param("value",expand_param!(BulkHandle).variant_self().bulk()).expand_return(expand_return!(BulkHandle).field_self_into().bulk()))
            .fun(fun!(bulk_array).expand_param("value",expand_param!(i64).variant_self().bulk()).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_array_borrow).expand_param("value",expand_param!(i64).variant_self().bulk()))
            .fun(fun!(bulk_scalar).expand_param("value", expand_param!(i64).variant_self().bulk()).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_numbers).expand_param("values", expand_param!(i64).variant_self().bulk()).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_nested).expand_param("values", expand_param!(i64).variant_self().bulk()).expand_return(expand_return!(i64).field_self().bulk()))
            .fun(fun!(bulk_text).expand_param("value", expand_param!(String).variant_self().bulk()).expand_return(expand_return!(String).field_self_into().bulk()))
            .fun(fun!(bulk_strings).expand_param("values", expand_param!(String).variant_self().bulk()).expand_return(expand_return!(String).field_self_into().bulk()))
            .fun(fun!(bulk_optional).expand_param("value", expand_param!(String).variant_self().bulk()).expand_return(expand_return!(String).field_self_into().bulk()))
            .fun(fun!(bulk_bytes).expand_param("value", expand_param!(Vec<u8>).variant_self().bulk()).expand_return(expand_return!(Vec<u8>).field_self_into().bulk())))
        .expand(expand_return!(ReturnFailure).field(fun!(return_failure_code)))
        .package(package!("returns")
            .class(data_class!(ReturnArgA)).class(data_class!(ReturnArgB)).class(data_class!(ReturnArgOuter))
            .class(data_class!(ReturnWideA)).class(data_class!(ReturnWideB)).class(data_class!(ReturnWideError))
            .fun(fun!(return_limited)).fun(fun!(return_wide_error))
            .fun(fun!(return_ownership).expand_return(expand_return!(ReturnOwner)
                .field(fun!(return_owner_value)).field(fun!(return_owner_resource))))
            .fun(fun!(return_single_ownership).expand_return(expand_return!(ReturnOwner).field(fun!(return_owner_resource))))
            .fun(fun!(return_domain)).fun(fun!(return_pair).expand_return(expand_return!(ReturnPair).field(fun!(return_pair_first)).field(fun!(return_pair_second))))
            .fun(fun!(return_wide).expand_return(wide_return)))
        .package(package!("runtimecosts")
            .fun(fun!(buffer_views_optional_text).expand_return(expand_return!(Option<String>).field_self().field(fun!(buffer_views_optional_text_len))))
            .fun(fun!(buffer_views_optional_bytes).expand_return(expand_return!(Option<Vec<u8>>).field_self().field(fun!(buffer_views_optional_bytes_len))))
            .fun(fun!(buffer_views_text).expand_return(expand_return!(String).field_self().field(fun!(buffer_views_text_len))))
            .fun(fun!(buffer_views_bytes).expand_return(expand_return!(Vec<u8>).field_self().field(fun!(buffer_views_bytes_len)))))
        .package(package!("buffers")
            .class(data_class!(JniBufferRow)).class(data_class!(JniBufferResourceRow))
            .fun(fun!(jni_buffer_resource)).fun(fun!(jni_buffer_domain_failure)).fun(fun!(jni_buffer_resource_rows).expand_return(expand_return!(Vec<JniBufferResourceRow>).field_self()))
            .fun(fun!(jni_buffer_text)).fun(fun!(jni_buffer_string))
            .fun(fun!(jni_buffer_bytes)).fun(fun!(jni_buffer_vec))
            .fun(fun!(jni_buffer_optional_text)).fun(fun!(jni_buffer_optional_bytes))
            .fun(fun!(jni_buffer_owned_optional))
            .fun(fun!(jni_buffer_owned).expand_return(expand_return!(JniBufferRow)
                .field(fun!(jni_buffer_row_text)).field(fun!(jni_buffer_row_bytes))))
            .fun(fun!(jni_buffer_rows))
            .fun(fun!(jni_buffer_wide).expand_return(wide_buffer_views))
            .fun(fun!(jni_buffer_callback)).fun(fun!(jni_buffer_failure)).fun(fun!(jni_buffer_worker))
            .fun(fun!(jni_buffer_owned_callback).expand_param("callback",
                callback!(impl Fn(Option<JniBufferRow>)).name("OwnedBufferViews")
                    .expand_param(0, expand_return!(JniBufferRow)
                        .field(fun!(jni_buffer_row_text)).field(fun!(jni_buffer_row_bytes))))))
        .expand(expand_return!(GetterChild)
            .field(fun!(getter_child_number)).field(fun!(getter_child_name)))
        .package(package!("getters")
            .class(ptr_class!(GetterOwner)).class(ptr_class!(GetterLeaf))
            .class(data_class!(GetterDetails))
            .fun(fun!(getter_owner_new).expand_return(expand_return!(GetterOwner)
                .field_self_into()
                .field(fun!(getter_owner_child)).field(fun!(getter_owner_optional))
                .field(fun!(getter_owner_container)).field(fun!(getter_owner_rows))
                .field(fun!(getter_owner_leaf))))
            .fun(fun!(getter_owner_failure).expand_return(expand_return!(GetterOwner)
                .field_self_into().field(fun!(getter_owner_leaf)).field(fun!(getter_owner_fail))))
            .fun(fun!(getter_owner_id)).fun(fun!(getter_leaf_id))
            .fun(fun!(getter_static_leaf))
            .fun(fun!(getter_owner_read).expand_return(expand_return!(GetterChild)
                .field(fun!(getter_child_number)).field(fun!(getter_child_name))))
            .fun(fun!(getter_counts))
            .fun(fun!(getter_two_clones_new).expand_return(expand_return!(GetterOwner)
                .view(prebindgen_declarations::Output::getter("getter_owner_leaf",
                    prebindgen_declarations::Output::combine([
                        prebindgen_declarations::Output::Leaf, prebindgen_declarations::Output::Leaf])))))
            .fun(fun!(getter_fields_new).expand_return(expand_return!(GetterOwner)
                .field_self_into().fields(fields!(getter_owner_details))))
            .fun(fun!(getter_callback).expand_param("callback",
                callback!(impl Fn(&GetterOwner)).name("GetterObserver")
                    .expand_param(0, getter_views()))))
        .expand(expand_param!(CallbackConfigValue).variant(fun!(callback_config_from_number)))
        .expand(expand_return!(CallbackConfigValue).field(fun!(callback_config_number)))
        .expand(expand_return!(CallbackBoundaryPanic).field(fun!(callback_boundary_view)))
        .expand(expand_return!(CallbackBoundaryError).fields_into(fields!(callback_boundary_error_into)))
        .package(package!("callbackboundaries")
            .class(ptr_class!(CallbackBoundaryHandler))
            .fun(fun!(callback_boundary_register)).fun(fun!(callback_boundary_fire))
            .fun(fun!(callback_boundary_fold).callback_return())
            .fun(fun!(callback_boundary_faults)).fun(fun!(callback_boundary_conversion))
            .fun(fun!(callback_boundary_borrowed)).fun(fun!(callback_boundary_strings)))
        .package(package!("callbackconfig")
            .class(callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue)
                .name("ConfiguredHandler")
                .expand_param(0, expand_return!(CallbackConfigValue).field(fun!(callback_config_number).name("number")))
                .expand_return(expand_param!(CallbackConfigValue).variant(fun!(callback_config_from_flag))))
            .class(callback!(impl Fn(bool) -> i16).name("UnusedHandler"))
            .class(callback!(impl Fn(CallbackConfigValue) -> i64).name("TypeDefaultHandler"))
            .class(ptr_class!(CallbackConfigHandler))
            .fun(fun!(callback_config_default))
            .fun(fun!(callback_config_type_default))
            .fun(fun!(callback_config_argument).expand_param("callback",
                callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue)
                    .name("FlagArgumentHandler")
                    .expand_param(0, expand_return!(CallbackConfigValue).field(fun!(callback_config_positive)))))
            .fun(fun!(callback_config_return).expand_param("callback",
                callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue)
                    .name("NumberReturnHandler")
                    .expand_return(expand_param!(CallbackConfigValue).variant(fun!(callback_config_from_number)))))
            .fun(fun!(callback_config_pair)
                .expand_param("first", callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue))
                .expand_param("second", callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue)
                    .name("FlagArgumentHandler")
                    .expand_param(0, expand_return!(CallbackConfigValue).field(fun!(callback_config_positive)))))
            .fun(fun!(callback_config_nested).expand_param("handlers",
                Input::for_type(syn::parse_quote!(CallbackConfigHandler), Input::constructor("callback_config_handler", [
                    callback!(impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue)
                        .name("NestedHandler")
                        .expand_param(0, expand_return!(CallbackConfigValue).field(fun!(callback_config_positive)))
                        .expand_return(expand_param!(CallbackConfigValue).variant(fun!(callback_config_from_number)))
                        .into()
                ]))))
            .fun(fun!(callback_config_optional).expand_param("handler", expand_param!(CallbackConfigHandler).variant(fun!(callback_config_handler))))
            .fun(fun!(callback_config_ordinary)))
        .expand(expand_param!(CallbackConstructor).variant(fun!(callback_constructor_from)))
        .expand(expand_param!(CallbackConstructorEnvelope).variant(fun!(callback_constructor_wrap)))
        // Clone-failure coverage explicitly opts these row fields into copying.
        .expand(expand_return!(CallbackRow).field_self_ref())
        .expand(expand_return!(CallbackTransferred).field_self_into())
        .expand(expand_return!(Vec<CallbackTransferredRow>).field_self_into())
        .expand(expand_return!(Vec<CallbackObservedRow>).field_self_ref())
        .expand(expand_return!(CallbackProjected).field(fun!(callback_projected_id).name("id")).field_self_into())
        .error_mapper(syn::parse_quote!(String),fun!(callback_return_error_message))
        .package(package!("callbackreturns")
            .class(data_class!(CallbackWireRow))
            .fun(fun!(callback_return_wire_rows_catch).expand_param("callback", callback!(impl Fn() -> Vec<CallbackWireRow>).expand_return(Input::Sequence(Box::new(Input::Record(vec![Input::Leaf; 2]))))))
            .fun(fun!(callback_return_wire_row_catch).expand_param("callback", callback!(impl Fn() -> CallbackWireRow).expand_return(prebindgen_jni::Input::Record(vec![prebindgen_jni::Input::Leaf; 2]))))
            .class(data_class!(CallbackReturnRecord)).class(enum_class!(CallbackReturnChoice))
            .class(ptr_class!(CallbackReturnResource)).class(ptr_class!(CallbackReturnJob)).class(ptr_class!(CallbackReturnHandler))
            .fun(fun!(callback_return_i64)).fun(fun!(callback_return_i8)).fun(fun!(callback_return_i16)).fun(fun!(callback_return_f32)).fun(fun!(callback_return_u128)).fun(fun!(callback_return_zero)).fun(fun!(callback_return_bool))
            .fun(fun!(callback_return_char)).fun(fun!(callback_return_float)).fun(fun!(callback_return_unsigned)).fun(fun!(callback_return_wide))
            .fun(fun!(callback_return_text)).fun(fun!(callback_return_bytes)).fun(fun!(callback_return_optional)).fun(fun!(callback_return_vector))
            .fun(fun!(callback_return_record)).fun(fun!(callback_return_enum))
            .fun(fun!(callback_return_resource_new)).fun(fun!(callback_return_resource_live))
            .fun(fun!(callback_return_owned)).fun(fun!(callback_return_take)).fun(fun!(callback_return_owned_optional)).fun(fun!(callback_return_owned_vector))
            .fun(fun!(callback_return_spawn)).fun(fun!(callback_return_join))
            .fun(fun!(callback_return_handler_new)).fun(fun!(callback_return_handler_call)).fun(fun!(callback_return_catch))
            .fun(fun!(callback_return_handler_many).expand_param("handlers",expand_param!(CallbackReturnHandler).variant(fun!(callback_return_handler_wrap))))
            .fun(fun!(callback_return_char_catch)).fun(fun!(callback_return_owned_vector_catch)).fun(fun!(callback_return_unit))
            .fun(fun!(callback_return_handler_value).expand_param("handler",expand_param!(CallbackReturnHandler).variant(fun!(callback_return_handler_wrap)))))
        // Malformed-row probes below deliberately exercise ordinary row-array ABI.
        .package(package!("callbacks")
            .class(data_class!(CallbackRecord)).class(data_class!(CallbackEnvelope))
            .class(ptr_class!(CallbackSet)).class(ptr_class!(CallbackJob))
            .class(ptr_class!(CallbackOwned)).class(ptr_class!(CallbackTransferred))
            .class(ptr_class!(CallbackProjected))
            .class(ptr_class!(CallbackConstructor)).class(ptr_class!(CallbackConstructorEnvelope))
            .class(ptr_class!(CallbackConstructorJob))
            .class(data_class!(CallbackObservedRow)).fun(fun!(callback_rows_ref)).fun(fun!(callback_row_return_cleanup_failure))
            .class(ptr_class!(CallbackUniqueResource)).class(data_class!(CallbackUniqueRow))
            .fun(fun!(callback_unique_resource_id)).fun(fun!(callback_rows_unique)).fun(fun!(callback_rows_unique_error))
            .class(ptr_class!(CallbackRowResource))
            .class(data_class!(CallbackRow)).class(data_class!(CallbackRowDetail))
            .class(data_class!(CallbackNestedRow)).class(data_class!(CallbackTransferredRow))
            .fun(fun!(callback_row_resource_id)).fun(fun!(callback_row_resource_live)).fun(fun!(callback_row_resource_drops))
            .fun(fun!(callback_row_resource_hold))
            // Raw ordinary-array probes are separate from default scoped callback tests.
            .fun(fun!(callback_rows_owned_probe).expand_param("callback",callback!(impl Fn(Vec<CallbackRow>)).name("OrdinaryCallbackRowVecCallback").expand_param(0,expand_return!(Vec<CallbackRow>).field_self())))
            .fun(fun!(callback_rows_optional_probe).expand_param("callback",callback!(impl Fn(Option<Vec<Option<CallbackRow>>>)).name("OrdinaryOptionalOptionalCallbackRowVecCallback").expand_param(0,expand_return!(Option<Vec<Option<CallbackRow>>>).field_self())))
            .fun(fun!(callback_rows_nested_probe).expand_param("callback",callback!(impl Fn(Vec<Vec<Option<CallbackNestedRow>>>)).name("OrdinaryOptionalCallbackNestedRowVecVecCallback").expand_param(0,expand_return!(Vec<Vec<Option<CallbackNestedRow>>>).field_self())))
            .fun(fun!(callback_rows_pair_probe).expand_param("callback",callback!(impl Fn(Vec<CallbackRow>, Vec<CallbackRow>)).name("OrdinaryCallbackRowVecCallbackRowVecCallback").expand_param(0,expand_return!(Vec<CallbackRow>).field_self()).expand_param(1,expand_return!(Vec<CallbackRow>).field_self())))
            .fun(fun!(callback_rows_transferred_probe).expand_param("callback",callback!(impl Fn(Vec<CallbackTransferredRow>)).name("OrdinaryCallbackTransferredRowVecCallback").expand_param(0,expand_return!(Vec<CallbackTransferredRow>).field_self())))
            .fun(fun!(callback_rows_transferred_pair_probe).expand_param("callback",callback!(impl Fn(Vec<CallbackTransferredRow>, Vec<CallbackRow>)).name("OrdinaryCallbackTransferredRowVecCallbackRowVecCallback").expand_param(0,expand_return!(Vec<CallbackTransferredRow>).field_self()).expand_param(1,expand_return!(Vec<CallbackRow>).field_self())))
            .fun(fun!(callback_rows_owned)).fun(fun!(callback_rows_borrowed))
            .fun(fun!(callback_rows_optional)).fun(fun!(callback_rows_nested))
            .fun(fun!(callback_rows_pair)).fun(fun!(callback_row_handles))
            .fun(fun!(callback_rows_transferred)).fun(fun!(callback_rows_transferred_pair)).fun(fun!(callback_rows_conversion_failure))
            .fun(fun!(callback_rows_spawn)).fun(fun!(callback_row_job_finished))
            .class(ptr_class!(CallbackReentry))
            .fun(fun!(callback_reentry_new)).fun(fun!(callback_reentry_read))
            .fun(fun!(callback_reentry_write)).fun(fun!(callback_reentry_take))
            .fun(fun!(callback_reentry_shared)).fun(fun!(callback_reentry_exclusive))
            .fun(fun!(callback_reentry_consuming)).fun(fun!(callback_reentry_pair))
            .fun(fun!(callback_reentry_mixed)).fun(fun!(callback_reentry_worker))
            .fun(fun!(callback_reentry_probe))
            .fun(fun!(callback_reentry_optional))
            .fun(fun!(callback_reentry_constructed).expand_param("value",expand_param!(CallbackReentry).variant(fun!(callback_reentry_clone))))
            .fun(fun!(callback_constructor_new)).fun(fun!(callback_constructor_fire))
            .fun(fun!(callback_constructor_owned_fire).expand_param("handler",expand_param!(CallbackConstructor).variant(fun!(callback_constructor_owned))))
            .fun(fun!(callback_constructor_pair)).fun(fun!(callback_constructor_mixed))
            .fun(fun!(callback_constructor_set_fire).expand_param("callbacks",expand_param!(CallbackSet).variant(fun!(callback_set_new))))
            .fun(fun!(callback_constructor_nested))
            .fun(fun!(callback_constructor_spawn))
            .fun(fun!(callback_constructor_run))
            .fun(fun!(callback_constructor_selected).expand_param("handler",expand_param!(CallbackConstructor).variant(fun!(callback_constructor_new)).variant_self()))
            .fun(fun!(callback_constructor_optional)).fun(fun!(callback_constructor_list))
            .fun(fun!(callback_constructor_checked))
            .fun(fun!(callback_set_new)).fun(fun!(callback_set_fire))
            .fun(fun!(callback_set_spawn)).fun(fun!(callback_job_join))
            .fun(fun!(callback_owned_new)).fun(fun!(callback_owned_id)).fun(fun!(callback_owned_drops))
            .fun(fun!(callback_owned_fire)).fun(fun!(callback_owned_borrow))
            .fun(fun!(callback_transferred_id)).fun(fun!(callback_transferred_drops))
            .fun(fun!(callback_transferred_fire)).fun(fun!(callback_projection_fire))
            .fun(fun!(callback_projected_id)).fun(fun!(callback_preflight))
            .fun(fun!(callback_scalars)).fun(fun!(callback_texts))
            .fun(fun!(callback_optional_owned)).fun(fun!(callback_envelope)).fun(fun!(callback_primitives))
            .fun(fun!(callback_enums)))
        .configure(|flat| {
            use prebindgen_flat::{OutputConversion, Outcomes, WireNode};
            let mut probe = flat.clone();
            let declarations = prebindgen_declarations::Declarations::new(&probe, vec![],vec![],vec![])?;
            declarations.fill(&mut probe, &prebindgen_jni::JniCodec, &fun!(collection_probe).expand_return(expand_return!(Vec<char>).field_self()), &())?;
            // Observe a clone so the builder still installs its own codec.
            // Collections.kt exercises the separately generated JNI boundary.
            let output = probe.function("collection_probe")?.output_conversion()?;
            let Outcomes { success, source_error: None, binding_failure: Some(_) } = output.output_outcomes() else {
                return Err("collection probe must declare success and binding failure only".into());
            };
            let logical = success;
            let WireNode::Chain {meta: prebindgen_jni::JniChain::Sequence(_), child: element, controls}=logical else {return Err("expected sequence".into());};
            assert_eq!(controls.len(),1);
            assert_eq!(prebindgen_flat::wire_definitions(std::iter::once(logical)).count(),2);
            assert!(matches!(prebindgen_flat::wire_definitions(std::iter::once(element.as_ref())).next().unwrap().kind, prebindgen_jni::JniWire::Scalar {
                primitive: prebindgen_flat::Primitive::Char, ..
            }));
            assert!(matches!(output.result_packet_fields()?.next().unwrap().kind, prebindgen_jni::JniWire::Rows { .. }));
            Ok(())
        })
        .expand(expand_param!(Vec<i64>).variant_self())
        .expand(expand_return!(CollectionError).fields_into(fields!(collection_error_into)))
        .expand(expand_return!(ResultFailure).field(fun!(result_error_code).name("code")))
        .error_mapper(syn::parse_quote!(StorageError), fun!(storage_error_message))
        .error_class(syn::parse_quote!(StorageError), "java.lang.IllegalStateException")
        .expand(expand_return!(CDropError).fields_into(fields!(result_drop_error_into)))
        .expand(expand_return!(CPanicOutput).field(fun!(c_panic_output_value)))
        .expand(expand_return!(CResultError).fields_into(fields!(result_pair_error_into)))
        .expand(expand_return!(CResultPair).fields_into(fields!(c_result_pair_into)))
        .expand(expand_return!(CResourceError).fields_into(fields!(result_resource_error_into)))
        .expand(expand_return!(CRichError).fields_into(fields!(result_rich_error_into)))
        .package(package!("collections").class(data_class!(Payload))
            .class(data_class!(SequenceScalars))
            .fun(fun!(collection_scalars).callback_return())
            .fun(fun!(collection_u128).callback_return())
            .fun(fun!(collection_construct_all).expand_param("values",expand_param!(CollectionItem).variant(fun!(collection_construct))))
.fun(fun!(collection_make_items).expand_return(expand_return!(CollectionItem).fields_into(fields!(collection_item_into))))
.fun(fun!(collection_bundle).expand_return(expand_return!(CollectionBundle).fields_into(fields!(collection_bundle_into))))
.fun(fun!(collection_optional_handles).callback_return())
.fun(fun!(collection_nested_handles).callback_return())
.fun(fun!(collection_numbers).expand_return(expand_return!(CollectionNumbers).fields(fields!(collection_numbers_view))))
.fun(fun!(collection_resource_ids).expand_param("values",expand_param!(OwnedResource).variant(fun!(collection_resource_clone))))
.fun(fun!(collection_i64).callback_return())
            .fun(fun!(collection_text).expand_param("values",expand_param!(Vec<String>).variant_self()).callback_return())
            .fun(fun!(collection_bytes).callback_return())
            .fun(fun!(collection_nested).callback_return())
            .fun(fun!(collection_options).callback_return())
            .fun(fun!(collection_optional).callback_return())
            .fun(fun!(collection_optional_items).callback_return())
            .fun(fun!(collection_enum).callback_return())
            .fun(fun!(collection_probe))
            .fun(fun!(collection_borrow).callback_return())
            .fun(fun!(collection_slice).callback_return())
            .fun(fun!(collection_resource_new))
            .fun(fun!(collection_handles).callback_return())
            .fun(fun!(collection_alias))
            .fun(fun!(collection_result).callback_return())
            .fun(fun!(collection_fail))
            .fun(fun!(collection_result_optional).callback_return())
            .fun(fun!(sequence_late_drop).callback_return_only().expand_return(expand_return!(Vec<CPanicOutput>).field(fun!(sequence_late_view))))
            .fun(fun!(sequence_scalars).callback_return_only().expand_return(expand_return!(SequenceScalars).fields_into(fields!(sequence_scalars_into))))
            .fun(fun!(sequence_parts).callback_return_only().expand_return(expand_return!(SequencePart).fields_into(fields!(sequence_part_into))))

            .fun(fun!(storage_put_slice).expand_param("payloads",expand_param!(Payload).variant_self()))
            .fun(fun!(storage_get_vec).callback_return().expand_return(expand_return!(Payload).field_self_into())))
        .package(package!("results")
            .fun(fun!(c_result_error_drop))
            .fun(fun!(c_result_late_niche))
            .fun(fun!(c_result_resource_error))
            .fun(fun!(c_result_rich_error))
            .fun(fun!(result_scalar))
            .fun(fun!(result_unit_error))
            .fun(fun!(result_unit))
            .fun(fun!(result_optional))
            .fun(fun!(result_text))
            .fun(fun!(result_record))
            .fun(fun!(result_owned))
            .fun(fun!(result_other).on_error(fun!(result_string_message)).expand_return(expand_return!(i64).field_self()))
            .fun(fun!(storage_try_with_label))
            .fun(fun!(result_error_drops)))
        .package(package!("enums")
            .fun(fun!(enum_boundary_probe)).class(enum_class!(Priority))
            .class(ptr_class!(PayloadHandle).name("EnumPayload"))
            .fun(fun!(enum_payload_new).expand_return(expand_return!(PayloadHandle).field_self_into()))
            .fun(fun!(payload_priority)).class(enum_class!(SparseCode).name("Code"))
            .class(enum_class!(EnumNames).variant_name("name","Name").variant_name("ordinal","Ordinal").variant_name("entries","Entries").variant_name("kotlin","Kotlin")).class(data_class!(EnumRecord))
            .fun(fun!(priority_weight)).fun(fun!(priority_or))
            .fun(fun!(sparse_echo)).fun(fun!(sparse_optional)).fun(fun!(sparse_nested))
            .fun(fun!(sparse_ref)).fun(fun!(sparse_borrowed)).fun(fun!(enum_record_echo)).fun(fun!(enum_names_echo)))
        .expand(expand_param!(TextNicheInput).variant(fun!(text_niche_input_new)))
        .expand(expand_param!(BytesNicheInput).variant(fun!(bytes_niche_input_new)))
        .expand(expand_param!(ResourceChoice).variant(fun!(resource_choice_new)))
        .expand(expand_param!(NicheChoiceHandle).variant(fun!(niche_choice_new)).variant_self())
        .expand(expand_param!(OptionCtor).variant(fun!(option_ctor_new)).variant_self())
        .expand(expand_param!(OptionChoice).variant(fun!(option_choice_new)).variant_self())
        .package(package!("options")
            .class(ptr_class!(NicheChoiceHandle)).fun(fun!(niche_choice_new)).fun(fun!(niche_choice_id))
            .fun(fun!(option_nested_u32))
            .fun(fun!(option_triple_text)).fun(fun!(option_triple_bytes))
            .fun(fun!(option_triple_i64)).fun(fun!(option_triple_unit))
            .fun(fun!(optional_text_input_len)).fun(fun!(optional_bytes_input_len))
            .fun(fun!(nested_niche_choice_id))
            .fun(fun!(option_nested_projection_views).expand_return(expand_return!(ProjectionHandle).field_self_into().field_self_ref()))
            .class(ptr_class!(OptionChoice))
            .class(ptr_class!(OptionCtor)).fun(fun!(option_ctor_new)).fun(fun!(option_ctor_id)).fun(fun!(option_nested_resource))
            .fun(fun!(option_choice_id))
            .fun(fun!(option_str_echo)).fun(fun!(option_slice_echo)).fun(fun!(option_str_len)).fun(fun!(option_slice_len)).fun(fun!(option_string_len)).fun(fun!(option_i64_ref))
            .fun(fun!(option_resource_pair)).fun(fun!(resource_choice_id))
            .fun(fun!(optional_pair_new).expand_return(expand_return!(OptionalPair).fields_into(fields!(optional_pair_into).field("present",expand_return!(OwnedResource).field_self_into()))))
            .fun(fun!(option_projection_views).expand_return(expand_return!(ProjectionHandle).field_self_into().field_self_ref()))
            .class(data_class!(OptionalFields))
            .class(data_class!(OptionalBytes)).fun(fun!(optional_bytes_echo))
            .fun(fun!(option_resource_mut)).fun(fun!(option_projection_ref))
            .class(data_class!(OptionalResource))
            .fun(fun!(option_i64)).fun(fun!(option_char)).fun(fun!(option_u8)).fun(fun!(option_bool))
            .fun(fun!(option_text)).fun(fun!(option_bytes)).fun(fun!(option_unit))
            .fun(fun!(option_nested)).fun(fun!(option_nested_char))
            .fun(fun!(option_resource_new)).fun(fun!(option_resource_echo).expand_param("value",expand_param!(OwnedResource).variant_self())).fun(fun!(option_resource_id))
            .fun(fun!(optional_fields_echo))
            .fun(fun!(optional_field_new).expand_return(expand_return!(OptionalField).fields(fields!(optional_field_fields))))
            .fun(fun!(optional_resource_new)))
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
        /* Unsupported: multiple source streams.
.source_named(cov_helpers::PREBINDGEN_OUT_DIR, "cov_helpers")
*/
        .set_package_prefix("io.prebindgen.covertest")
        .set_jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
        // Every naming tier used here is configured. The harness hook is a
        // real transform: it receives the derived default `JNINative` and
        // replaces it wholesale with `CovNative` (an internal symbol, so no
        // Kotlin-side coordination is needed); four hooks are identity
        // (the domain names are already the desired Kotlin names) — registering
        // closures, and the method hook strips the flat class prefix. The
        // generated-interface hook deliberately keeps its `ClassApi` default.
        /* Unsupported: this builder feature is not implemented.
.set_harness_name_mangle(|_| "CovNative".to_string())
*/
        /* Unsupported: this builder feature is not implemented.
.set_fun_name_mangle(|_, n| n.to_string())
*/
        /* Unsupported: this builder feature is not implemented.
.set_ptr_class_name_mangle(|_, n| n.to_string())
*/
        /* Unsupported: this builder feature is not implemented.
.set_data_class_name_mangle(|_, n| n.to_string())
*/
        /* Unsupported: this builder feature is not implemented.
.set_enum_name_mangle(|_, n| n.to_string())
*/
        /* Unsupported: this builder feature is not implemented.
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
*/
        // `Millis` newtype: a canonical single-value conversion to a bare
        // `Long` (no generated class) via two ordinary `#[prebindgen]` fns —
        // defined in the SEPARATE `covertest-helpers` source crate, proving
        // the multi-source model (generated calls carry the
        // `cov_helpers::` prefix). The Kotlin surface (`Long`) derives
        // from the fns' `i64` side; nothing is stated verbatim.
        /* Unsupported: this builder feature is not implemented.
.convert(
            convert!(Millis)
                .input(fun!(millis_from_long))
                .output(fun!(millis_value)),
        )
*/
        // `Celsius`: canonical conversion via `From`/`Into` impls in the flat
        // crate — the repr (`i32`) is stated, the impls do the work.
        /* Unsupported: this builder feature is not implemented.
.convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
*/
        // `Percent`: fallible in BOTH directions. `Option<Percent>` below
        // forces both raw stage error types through the structural Option
        // converter, where they normalize to the binding error channel.
        /* Unsupported: this builder feature is not implemented.
.convert(
            convert!(Percent)
                .input(try_from!(i32))
                .output(fun!(crate::percent_out).sig(sig!((p: Percent) -> Result<i32, String>))),
        )
*/
        // `Label`: conversions are plain fns in THIS binding crate (see
        // src/lib.rs) — no #[prebindgen], no helper crate — declared with
        // the ONE binding-local vocabulary, `fun!(crate::…).sig(sig!(…))`.
        // The input is FALLIBLE: the sig's `Result<Label, String>` return IS
        // the error channel (empty labels route the Err to onError).
        /* Unsupported: this builder feature is not implemented.
.convert(
            convert!(Label)
                .input(fun!(crate::label_in).sig(sig!((s: String) -> Result<Label, String>)))
                .output(fun!(crate::label_out).sig(sig!((l: Label) -> String))),
        )
*/
        // Keep Rust's standard `Duration` as the semantic type while exposing
        // bounded u64 milliseconds. Values above one day are invalid; one of
        // those representations becomes the `Option<Duration>` None marker,
        // so nullable Kotlin `ULong?` crosses JNI as a raw primitive `Long`
        // without JObject/boxed-Long allocation.
        /* Unsupported: this builder feature is not implemented.
.convert(
            convert!(Duration)
                .input(fun!(crate::duration_from_millis).sig(sig!((ms: u64) -> Duration)))
                .output(
                    fun!(crate::duration_to_millis).sig(sig!((d: Duration) -> Result<u64, String>)),
                )
                .valid_range(0u64..=86_400_000u64),
        )
*/
        // ── Base-package types ──────────────────────────────────────────────
        // `Payload` as a Kotlin `data class` (fields cross as decoupled leaves,
        // reassembled via a generated `fromParts`). A data class can carry
        // members like any re-enterable kind: the instance method's receiver
        // crosses as `this`'s field leaves (I5).
        // `Payload` also demos the `.interface()` hatch on a DATA class:
        // `PayloadApi` exposes its fields + `labelLen()`, and the
        // hand-written `Timestamped` interface extends it (#54).
        .package(
            package!()/* Unsupported: this class kind or its binding is not available yet.
.class(
                data_class!(Payload)
                    .interface()
                    .implements("io.prebindgen.covertest.Timestamped")
                    .method(fun!(payload_label_len)),
            )
*/,
        )
        // `Option<Holder>` where `Holder` has a REQUIRED handle field: the
        // absent case passes pointer 0 for it, so the field decodes must stay
        // inside the presence gate or `null` becomes a binding error instead of
        // `None` (PR#294 review).
        .package(package!()/* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Holder))
*/)
        // #218's last row: a data class that only REACHES a handle, through
        // another data class. `Dossier`'s cascade is a one-liner that assumes
        // `Holder` above was independently made `AutoCloseable` — the JVM
        // harness is what ties the two decisions together, since an emission
        // test never compiles the inner class.
        .package(package!()/* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Dossier))
*/)
        // A data class whose FIELDS carry transparent wrappers (#289 + #292):
        // `boxed: Box<Option<i64>>` must cross exactly as `plain: Option<i64>`
        // does — the decoupled `(present, value)` pair — with the `Box` put back
        // on the Rust side. Peeling the field by path segment answered "not
        // optional" and boxed it instead.
        .package(package!()/* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(WrappedFields))
*/)
        // ── Subpackage `model`: enum + value class + nested data class ──────
        .package(
            package!("model")
                // `Priority` as a Kotlin `enum class` (jint wire, `fromInt`
                // companion); `.interface_name("PriorityKind")` demos the
                // generated interface on an ENUM with a per-decl name, and the
                // hand-written `Ranked` (which extends `PriorityKind`) is
                // attached via `.implements` (#54).
                /* Unsupported: enum interface/implements customization.
.class(
                    enum_class!(Priority)
                        .interface_name("PriorityKind")
                        .implements("io.prebindgen.covertest.Ranked"),
                )
*/
                // `Reading` as a Kotlin `sealed interface` — a data-carrying
                // enum whose alternatives are nested classes, with the
                // payload-less one a `data object`. `variant!(Labeled)
                // .name("Tagged")` exercises the per-variant rename (which
                // also renames its `fromParts` slots).
                /* Unsupported: this class kind or its binding is not available yet.
.class(sealed_class!(Reading).variant(variant!(Labeled).name("Tagged")))
*/
                // `Lookup` is the sum in RETURN position whose groups own
                // resources: one alternative carries an opaque handle, one
                // carries nothing at all. Because it reaches an owned handle it
                // is emitted `AutoCloseable`, and each variant class overrides
                // `close()` — that is what lets a container cascade into it
                // with the same one-liner a handle field gets (#218).
                /* Unsupported: this class kind or its binding is not available yet.
.class(sealed_class!(Lookup))
*/
                // …and `Verdict` is that container: the sum in DATA-CLASS FIELD
                // position, the third place a handle is reached through.
                // `Holder` above is the plain-handle field it must match.
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Verdict))
*/
                // Report now projects the reduced value form through a borrowing
                // getter, then delivers its owned handle. The original consuming
                // expansion remains commented below.
                .class(ptr_class!(Report))
                .class(ptr_class!(ReportStruct))
                // #220: the same derived-boundary idea with an `Option<sum>`
                // FIELD — the shape that used to be refused on a value form
                // while a `data_class` accepted it.
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(Probe))
*/
                // #142: two bounded-`convert!` leaves — one with a niche of its
                // own, one without — reached through a CONDITIONAL hoist, so both
                // are nullable. Only the niche-carrying one keeps a sentinel in
                // its wrap; the other's absence is the ancestor's `?.`.
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(Span))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(SpanHolder))
*/
                // `Hold`'s payload is a CONVERTED type, so its leaf crosses
                // through the `convert!(Duration)` chain; `HoldPolicy` puts
                // that same payload in the data-class-field position.
                /* Unsupported: this class kind or its binding is not available yet.
.class(sealed_class!(Hold))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(HoldPolicy))
*/
                // `Observation` carries that sum as a data-class FIELD —
                // required (`reading`) and optional (`fallback`) — beside
                // ordinary flattened leaves, so the tag-gated groups must
                // interleave with them on one `fromParts` call.
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Observation))
*/
                // `Marker`'s `Option<enum>` payload is not leaf-shaped, so the
                // sum degrades to a whole-object crossing instead of failing —
                // and that path is what reads an enum property back.
                /* Unsupported: this class kind or its binding is not available yet.
.class(sealed_class!(Marker))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Tagged))
*/
                // `Annotated` exercises a NESTED data-class field (`payload`,
                // recursive fromParts / recursive leaf decode) plus Option<prim> and
                // Option<enum> FIELDS (each a decoupled `(present, value)` leaf pair).
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Annotated))
*/
                // #144: a NON-NULL enum field (`RepliesConfig.priority`) reached
                // through an outer `Option<CacheConfig>` param. The outer
                // optional propagates `nullable_context` into the non-optional
                // nested struct, so its enum field must decode with exactly one
                // Elvis default (regression guard for the dead `?: 0 ?: 0`).
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(RepliesConfig))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(CacheConfig))
*/
                // Compose the bounded `Option<Duration>` niche through a
                // data-class field. Explicit JObject input makes the runtime
                // execute the whole-object decoder as well as the primitive-
                // niche `fromParts` encoder (#138).
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(DurationBoundary).jobject_input())
*/
                // These small nested classes form a 127-Long-leaf tree. Its
                // constructor is legal, but flattening the root function input
                // would consume 256 JVM slots, so it keeps one JObject input.
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
.class(data_class!(ObjectBoundary63))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(ObjectBoundary).jobject_input())
*/
                // Fixed-width unsigned mappings: Int / Long widening plus
                // ULong over a raw jlong bit pattern.
                .class(data_class!(Unsigned))
                // `Stamp` is a small `Copy` struct of two scalars, so it crosses
                // as its FIELDS — no array, no raw-memory image. Its readers stay
                // instance methods (`secs()` / `nanos()`) whose receiver crosses
                // as those field leaves, and `Vec<Stamp>` surfaces as
                // `List<Stamp>`.
                /* Unsupported: this class kind or its binding is not available yet.
.class(
                    data_class!(Stamp)
                        .method(fun!(stamp_secs))
                        .method(fun!(stamp_nanos)),
                )
*/
                // `BlobValue` is the array-backed EQUALITY probe: a raw-bytes
                // field beside a scalar, plus a nested data class. Both compare
                // by identity in Kotlin unless the binding says otherwise.
                /* Unsupported: direct jobject input is not available yet.
.class(data_class!(BlobValue).jobject_input())
*/
                // Value records cross as fields and are reconstructed automatically.
                .class(data_class!(BlobValue))
                .class(data_class!(Stamp))
                // Fixed-size arrays of every JNI-primitive element.
                /* Unsupported: this class kind or its binding is not available yet.
.class(data_class!(Arrays))
*/,
        )
        // ── Subpackage `errors`: the Result error channel ───────────────────
        .package(package!("errors")/* Unsupported: this class kind or its binding is not available yet.
.class(
            // `StorageError` is the `E` of a fallible `Result`; its
            // boundary shape is declared with `expand_return!` below.
            ptr_class!(StorageError).method(fun!(storage_error_message)),
        )
*/)
        // `StorageError`'s default return fields make the generated `onError`
        // handler receive the decomposed error: the `message` string (name
        // inherited from the class member) plus — via `.field_self()` — the
        // error handle itself (an owned `StorageError` the handler must
        // `close()`).
        /* Unsupported: this expansion needs unsupported conversions.
.expand(
            expand_return!(StorageError)
                .field(fun!(storage_error_message))
                .field_self(),
        )
*/
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
                /* Unsupported: this class kind or its binding is not available yet.
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
*/
                // `Archive` holds the latest `Summary` and returns it BORROWED
                // (`Option<&Summary>`) — the JVM binding clones it into a fresh owned
                // handle (the zenoh-flat borrowed-accessor shape). Its Kotlin class is
                // RENAMED via the per-declaration `.name()` override (the type-level
                // dual of the per-fn `.name`; literal, bypasses the mangle closures).
                .class(ptr_class!(Archive).name("SummaryVault"))
                .class(ptr_class!(Summary).name("SummaryValue")),
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

        // `Report` default output DERIVED from its value form (#213): the
        // leaves come from `ReportStruct`'s fields, so the list cannot drift
        // from the struct the way a restated one does. Each field still crosses
        // by ITS OWN type's boundary — `summary` splices `Summary`'s decl above
        // into `(count, total)` rather than becoming a handle, `origin` inlines
        // its `Stamp` fields, `taken` stays one `Stamp?` leaf, and `outcome`
        // decomposes into a selector plus one group per alternative.
        /* Unsupported: this expansion needs unsupported conversions.
.expand(expand_return!(Report).fields_into(fields!(report_into_struct)))
*/
        // #142: `Span`'s value form is two bounded leaves; `SpanHolder` reaches
        // it through an `Option`, so both become nullable. `delay` keeps its own
        // niche sentinel, `required` takes none — a sentinel is the leaf's own
        // `None`, never an ancestor's.
        /* Unsupported: this expansion needs unsupported conversions.
.expand(expand_return!(Span).fields(fields!(span_to_struct)))
*/
        /* Unsupported: this expansion needs unsupported conversions.
.expand(expand_return!(SpanHolder).field(fun!(span_holder_span)))
*/
        // #220: `ProbeStruct.outcome` is `Option<Lookup>`. Its whole segment
        // gates together — one tuple bind whose absent arm defaults every slot
        // — because a sum's leaves are not independent. The selector boxes, so
        // JVM null is "no sum" and cannot be read as `Lookup.Absent` (tag 0).
        /* Unsupported: this expansion needs unsupported conversions.
.expand(expand_return!(Probe).fields(fields!(probe_to_struct)))
*/
        // `Ledger` reaches that same derived boundary through an `Option`, so
        // `Report`'s value form is hoisted CONDITIONALLY — built in the `Some`
        // arm, its leaves (the sum among them) sharing one `match`, null in the
        // absent arm. The two accessors differ in what the arm binds: `filed`
        // hands over a borrow, `archived` an owned report the by-value form
        // takes directly — `Report` is not `Clone`, so cloning it here would not
        // compile.
        /* Unsupported: this expansion needs unsupported conversions.
.expand(
            expand_return!(Ledger)
                .field(fun!(ledger_filed))
                .field(fun!(ledger_archived)),
        )
*/
        // ── Base-package handle type: `Storage` + scalar members ────────────
        // Back in the base package so the typed handle classes live alongside
        // `Payload`.
        .package(
            package!()
                // `#[prebindgen]` consts: each surfaces as a generated nullary JNI
                // getter extern + an eagerly-initialized top-level Kotlin `val`
                // (`COVER_MAGIC: Long`, `COVER_TAG: String`) in the base package.
                /* Unsupported: this builder feature is not implemented.
.constant(constant!(COVER_MAGIC))
*/
                /* Unsupported: this builder feature is not implemented.
.constant(constant!(COVER_TAG))
*/
                // Fn-sourced constant: a nullary `#[prebindgen]` fn surfaced
                // as an eagerly-initialized top-level `val`
                // (`COVER_TAG_RUNTIME: String`) — the value comes from the
                // fn at class-load, not from a Rust `const`.
                /* Unsupported: this builder feature is not implemented.
.constant(constant!(COVER_TAG_RUNTIME).fun(fun!(cover_tag_runtime)))
*/
                // Binding-local-fn-sourced constant (`.with`, the const
                // analog of convert!'s `_with`): a nullary fn in THIS crate,
                // named by path, stated value type.
                /* Unsupported: this builder feature is not implemented.
.constant(constant!(COVER_VERSION).with(ty!(String), path!(crate::cover_version)))
*/
                // Expression-sourced constant: an arbitrary binding-defined
                // expression (composing source-crate items via
                // `use perftest_flat::*;`) evaluated once at class-load —
                // no dedicated accessor fn in the source crate.
                /* Unsupported: this builder feature is not implemented.
.constant(
                    constant!(COVER_BANNER)
                        .expr(ty!(String), expr!(format!("{COVER_TAG}:{COVER_MAGIC:#x}"))),
                )
*/
                .class(
                    ptr_class!(Storage)
                        // #54: emit the generated `StorageApi` interface (the
                        // class implements it, members get `override`) AND
                        // attach the hand-written `CovResource` which EXTENDS
                        // `StorageApi` — so its defaults call `len()`/`peek()`
                        // with full compiler checking, no hand-editing of
                        // generated code.
                        /* Unsupported: this builder feature is not implemented.
.interface()
*/
                        /* Unsupported: this builder feature is not implemented.
.implements("io.prebindgen.covertest.CovResource")
*/
                        /* Unsupported: this builder feature is not implemented.
.method(fun!(storage_len))
*/
                        /* Unsupported: this builder feature is not implemented.
.method(fun!(storage_contains))
*/
                        /* Unsupported: this builder feature is not implemented.
.constructor(fun!(storage_with_payload))
*/,
                )
                // The callback-handler handles (single payload / whole batch / owned
                // storage handle).
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(PayloadHandler))
*/
                // `StorageHandler`'s callback receives an OWNED opaque handle
                // (`Fn(Storage)`): the raw pointer crosses and the generated Kotlin
                // proxy wraps it into a typed `Storage` and closes it after `run`.
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(StorageHandler))
*/
                /* Unsupported: this class kind or its binding is not available yet.
.class(ptr_class!(PayloadVecHandler))
*/,
        )
        // ── JNI native-symbol escaping probe (#86) ──────────────────────────
        // Underscores in EVERY symbol component: the `esc_pkg` subpackage and
        // the `Esc_Probe` class name put `_1` escapes into the `freePtr`
        // destructor symbol (`Java_…_esc_1pkg_Esc_1Probe_freePtr`), and the
        // method-mangle hook above puts one into the accessor's harness
        // extern (`…_escape_1probe_1value`). Kotlin names stay verbatim; the
        // JVM only resolves these if the generator escapes per the JNI spec.
        .package(
            package!("esc_pkg")/* Unsupported: this class kind or its binding is not available yet.
.class(
                ptr_class!(EscapeProbe)
                    .name("Esc_Probe")
                    .constructor(fun!(escape_probe_new))
                    .method(fun!(escape_probe_value)),
            )
*/,
        )
        // ── Free functions, grouped by subpackage ───────────────────────────
        // model: value-class return, Vec<value> and Option<scalar>.
        // Enum entry points are bound in the `enums` package above.
        .package(
            package!("model")
                .fun(fun!(stamp_new))
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(stamp_series))
*/
                // The three convert!-source-kind fns (conversions declared
                // below): Into/From traits, TryFrom trait, binding-local fns.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(celsius_double))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(percent_scale))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(percent_optional))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(percent_invalid_output))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(label_reverse))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(label_series_echo))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(annotated_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(annotated_alternate_value))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(annotated_ttl))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(annotated_priority))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(annotated_payload_value))
*/
                // A sum as a data-class field, crossing OUT: the tag plus every
                // variant's group ride the parent's single `fromParts`.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(observation_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(observation_which))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(tagged_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(tagged_rank))
*/
                // The same `Option<enum>` payload in RETURN position — the
                // `synth_sum_leaves` path, which the struct field above does
                // not reach (it degrades to the whole-object crossing).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(marker_of))
*/
                // A sum as the function's OWN return (and callback argument):
                // the tag + groups ride the hoisted builder / folder singleton
                // instead of a parent's `fromParts`. All four positions —
                // bare, `Option`, `Vec`, callback — plus a group owning a
                // native handle.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(reading_of))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(reading_maybe))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(reading_series))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(reading_each))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(lookup_of))
*/
                // #161: the two positions the four above do not reach — a
                // handle-carrying sum arriving through a CALLBACK, and a sum
                // returned BORROWED (`&E` / `Option<&E>`).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(lookup_each))
*/
                // #218: the same handle reached through a data-class FIELD, so
                // the JVM harness can assert the container's cascade closes it.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(verdict_new))
*/
                // …and reached one level deeper still, through a nested data
                // class rather than a sum.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(dossier_new))
*/
                // #213: the output boundary DERIVED from the type's value form
                // rather than restated. `report_each` delivers the decomposed
                // `Report` in one crossing; the leaf list comes from
                // `ReportStruct`'s fields, so it cannot drift from it.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(report_each))
*/
                // #220: the value form whose `outcome` field is `Option<sum>`.
                // `probe_each` walks the absent case and every alternative, so
                // one crossing covers the gate and the groups it gates.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(probe_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(probe_each))
*/
                // The same derived boundary reached through an `Option` — a
                // CONDITIONAL hoist. `ledger_each` delivers both at once, so one
                // crossing covers the borrowed payload, the owned one, and the
                // sum each report carries.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(ledger_each))
*/
                // #142: the holder whose span is optional.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(span_holder_new))
*/
                // A transparent wrapper (`Box<Option<String>>`) in and out. The
                // model erases the `Box`, so this must cross exactly as a
                // `String?` — and because this crate compiles its generated
                // binding, a converter that named the wrong type or bridged it
                // with the wrong number of dereferences fails the build (#270).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_note_echo))
*/
                .fun(fun!(plain_note_echo))
                // Transparent wrappers on the INPUT side (#292 item 3), one per
                // specialized lowering. These rebuild their parameter rather
                // than decoding it, so the erased wrapper has to go back on
                // before the value reaches the signature — and each layer is
                // applied at a different point in the construction, which is why
                // one shape cannot cover them all. Compiling this crate is the
                // check: a missing `Box::new` is an `E0308`, invisible to any
                // text assertion.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(wrapped_fields_sum))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(holder_tag_or))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_payload_id))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_opt_payload_id))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_opt_priority_weight))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_elem_id_sum))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_run_id_sum))
*/
                // The two spellings of a borrowed run (#384). One type to the
                // model, so both take the Vec-build path — and the compiled
                // binding is what proves the emitter serves both, since the
                // failure was an `E0308` rather than a wrong answer.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(slice_id_sum))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(ref_vec_id_sum))
*/
                // The same wrapper over a DECOMPOSED return (#292). `Summary`
                // has an output expansion, so this return takes no converter to
                // name the spelling for it — the extern binds the value and
                // matches it, and a `Box` match ergonomics cannot see through
                // is an `E0308` that only compiling this crate catches.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_latest))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(ledger_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(archive_set_reading))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(archive_reading))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(archive_reading_maybe))
*/
                // A sum payload that is a CONVERTED type (`convert!(Duration)`),
                // so its boundary conversion is a chain rather than one wire
                // converter — exercised as the function's own return and as a
                // (required + optional) data-class field, the two encoders.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(hold_echo))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(hold_policy_echo))
*/
                // #144: `Option<CacheConfig>` input reaching a non-null enum
                // field through the nested `RepliesConfig`.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(cache_config_weight))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(object_boundary_value))
*/
                .fun(fun!(unsigned_round_trip))
                .fun(fun!(unsigned_optional))
                /* Unsupported: borrowed data-class inputs.
.fun(fun!(unsigned_data_maybe))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(unsigned_emit))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(unsigned_series))
*/
                .fun(fun!(blob_value_new))
                .fun(fun!(blob_value_echo))
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(arrays_echo))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(duration_optional))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(boxed_duration_echo))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(duration_boundary_echo))
*/
                // The converted analogue of `unsigned_emit`: a whole-value
                // callback argument, which encodes on its own path rather than
                // through the data-class or sum emitters.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(duration_emit))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(duration_out_of_range))
*/,
        )
        // analytics: the param-variant / return-field matrix (type default /
        // per-fn override, in + out). Per-fn overrides reuse the SAME
        // expand-decl objects as the type defaults (complete-set rule): an
        // identity-only set is the plain form.
        .package(
            package!("analytics")
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_summary))
*/
                // Binding-local FREE FUNCTION: exported like any package fn;
                // its `&Summary` param resolves through the ordinary borrow
                // converter, its String return through the ordinary output
                // converter.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(crate::summary_describe)
                        .sig(sig!((s: &Summary, verbose: bool) -> String))
                        .name("describeSummary"),
                )
*/
                // Single split (#52) on the CLASS-DEFAULT `Summary` variants:
                // `storageMatchesSummary(count, total, …)` / `(expected, …)`.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_matches_summary).split_on_param("expected"))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(storage_summary_handle)
                        .expand_return(expand_return!(Summary).field_self()),
                )
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(summary_total_raw)
                        .expand_param("s", expand_param!(Summary).variant_self()),
                )
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(storage_summary_full).expand_return(
                        expand_return!(Summary)
                            .field(fun!(summary_count).name("count"))
                            .field(fun!(summary_total).name("total"))
                            .field_self(),
                    ),
                )
*/
                // Binding-local CONDITIONAL field (`field!` + `.with(ty, path)`):
                // the handle leaf is delivered only when the binding-side
                // predicate (`crate::summary_if_nonempty`, src/lib.rs) says
                // re-using the value is worth it — nullable identity leaf,
                // null when the condition fails. The condition is binding
                // policy, so it lives in THIS crate, not the source crate.
                /* Unsupported: the signature or function options need unsupported conversions.
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
*/
                // Per-fn split (#52): a per-fn `.expand_param` variant override
                // (demoing the override) whose `expected` param is then split
                // into typed overloads `storageExpectSummary(count, total, …)` /
                // `(expected, …)` on top of the selector form (which Test.kt
                // still calls directly).
                /* Unsupported: the signature or function options need unsupported conversions.
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
*/
                // Cartesian-product split (#52): TWO `Summary` params each split
                // → the 2×2 product of typed overloads (all combinations
                // distinct: build/build, build/handle, handle/build, handle/handle).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(summary_prefer)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                )
*/
                // Split × builder-delivered return (#87): both params split AND
                // the `Summary` return crosses via the decomposed builder, so
                // the wrapper is generic — every overload must re-declare `<R>`.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(
                    fun!(summary_merge)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                )
*/
                // Optional combined-selector expansion: `Option<&Summary>` under
                // the dual-arm type default — the selector also encodes absence
                // (`-1` = `None`); the borrow-identity arm clones, so the
                // caller's handle survives the call.
                /* Unsupported: helper source requires multiple source streams.
.fun(fun!(summary_total_opt))
*/
                // Record-built generic folds (#105): `Vec<Summary>` and
                // `Option<Vec<Summary>>` returns cross as the caller-supplied
                // `<A>(acc, fold)` surface — decomposed `(count, total)`
                // leaves per element. The `Option` form returns `A?`: null =
                // `None` (the fold never invoked), `Some(empty)` = the
                // untouched accumulator.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(summary_series))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(summary_series_opt))
*/
                // The borrowed accessor returns a fresh cloned handle when present.
                // The owned Summary expansion does not apply to &Summary.
                .fun(fun!(archive_new))
                .fun(fun!(archive_store))
                .fun(fun!(archive_latest))
                // Single split (#52) on the CLASS-DEFAULT variants, consuming arm.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(archive_store).split_on_param("s"))
*/,
        )
        // storage: the perf surface (handles, callbacks, Vec, Option) plus the
        // fallible constructor and the Millis wrapper.
        .package(
            package!("storage")
                .fun(fun!(storage_new))
                .fun(fun!(storage_get))
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_put_by_take))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_put_by_read))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_put_slice))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_get_vec).callback_return())
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(payload_handler_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_callback))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(payload_vec_handler_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_callback_vec))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_try_with_label))
*/
                // Two-caller error split (#45): both channels on one wrapper.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_try_from_stamp))
*/
                // Vec<opaque-handle> returns (plain + under the Option niche).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_shards))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_shards_opt))
*/
                // Owned-handle-in-callback pair.
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_handler_new))
*/
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_emit))
*/
                // A 3-opaque-handle call (sorted N-ary handle locking).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_total_len))
*/
                // Vec<String> return (single-leaf string fold).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(storage_labels))
*/
                // Option<data-class> input.
                .fun(fun!(storage_put_opt))
                // `.name(...)`: per-function Kotlin rename override. The default name
                // would be `millisAdd`; force it to `addMillis` to exercise the
                // override path (the Rust symbol/extern is unaffected).
                /* Unsupported: the signature or function options need unsupported conversions.
.fun(fun!(millis_add).name("addMillis"))
*/,
        )
        // Plain String return, declared in the BASE package (mirroring the
        // base-package classes).
        .package(
            package!()
                .fun(fun!(string_new))
                .fun(fun!(string_len))
                .fun(fun!(i64_ref_double))
                // Explicit identity declarations give text and bytes, like the defaults.
                .fun(
                    fun!(text_echo)
                        .expand_param("s", expand_param!(String).variant_self())
                        .expand_return(expand_return!(String).field_self()),
                )
                .fun(
                    fun!(bytes_echo)
                        .expand_param("b", expand_param!(Vec<u8>).variant_self())
                        .expand_return(expand_return!(Vec<u8>).field_self()),
                ),
        )
        // Report projects its supported Summary, Stamp and label fields.
        // Project those fields before handing ownership of the Report to Kotlin.
        .expand(expand_return!(Report).fields(fields!(report_to_struct).field("origin", expand_return!(Stamp).field_self())).field_self())
        .expand(expand_param!(Report).variant_self().variant(fun!(report_new)))
        .package(package!().fun(fun!(report_new)).fun(fun!(report_into_struct)))
        .expand(expand_return!(Stamp).field(fun!(stamp_secs)).field(fun!(stamp_nanos)))
        .expand(expand_return!(Payload).fields(fields!(payload_to_struct)))
        .expand(expand_param!(Payload).variant(fun!(payload_new)))
        .package(package!().fun(fun!(payload_new)).fun(fun!(storage_with_payload)).fun(fun!(summary_new)).fun(fun!(summary_total_raw)))
        .package(
            package!("primitives")
                .fun(fun!(primitive_i8))
                .fun(fun!(primitive_i16))
                .fun(fun!(primitive_i32))
                .fun(fun!(primitive_i64))
                .fun(fun!(primitive_i128))
                .fun(fun!(primitive_isize))
                .fun(fun!(primitive_u8))
                .fun(fun!(primitive_u16))
                .fun(fun!(primitive_u32))
                .fun(fun!(primitive_u64))
                .fun(fun!(primitive_u128))
                .fun(fun!(primitive_usize))
                .fun(fun!(primitive_f32))
                .fun(fun!(primitive_f64))
                .fun(fun!(primitive_bool))
                .fun(fun!(primitive_char))
                .fun(fun!(primitive_unit)),
        )
        .package(package!()
            .fun(fun!(report_label).expand_param("report", expand_param!(Report).variant(fun!(report_from_summary))))
            .fun(fun!(report_relabel).expand_param("report", expand_param!(Report).variant_self()))
            .fun(fun!(storage_len)).fun(fun!(storage_contains)).fun(fun!(storage_total_len)).fun(fun!(storage_accumulate_lengths))
            .fun(fun!(storage_put_by_take)).fun(fun!(summary_merge))
            .fun(fun!(storage_summary_handle).expand_return(expand_return!(Summary).field_self())))
        .package(package!("resources")
            .class(data_class!(ProjectionPair)).class(ptr_class!(ProjectionHandle))
            .fun(fun!(projection_pair_borrowed).expand_return(expand_return!(ProjectionPair).field_self_ref()))
            .fun(fun!(projection_pair_owned)).fun(fun!(projection_pair_callback))
            .fun(fun!(projection_pair_mixed).expand_return(expand_return!(ProjectionPair).project_self(
                prebindgen_jni::Projection::owned().field("first", prebindgen_jni::Projection::borrowed()))))
            .fun(fun!(projection_pair_views).expand_return(expand_return!(ProjectionPair).field_self_into().field_self_ref()))
            .fun(fun!(projection_clones)).fun(fun!(projection_drops)))
        .package(package!("resources")
            .class(data_class!(ResourceBundle))
            .class(data_class!(ResourcePair))
            .class(ptr_class!(OwnedResource))
            .fun(fun!(resource_bundle_new).expand_return(expand_return!(ResourceBundle).project_self(
                prebindgen_jni::Projection::owned().field("details", prebindgen_jni::Projection::borrowed()))))
            .fun(fun!(resource_bundle_default))
            .fun(fun!(resource_bundle_fields).expand_return(expand_return!(ResourceBundle).fields_into(
                fields!(resource_bundle_into)
                    .field("pair", expand_return!(ResourcePair).field_self_into())
                    .field("details", expand_return!(DataFloats).field_self_ref()))))
            .fun(fun!(api_record_new).expand_return(
                expand_return!(ApiRecord).fields_into(
                    fields!(api_record_into)
                    .field("omitted", expand_return!(OwnedResource))
                    .field("omitted_opt", expand_return!(OwnedResource))
                    .field("omitted_vec", expand_return!(OwnedResource)))))
            .fun(fun!(api_record_views).expand_return(expand_return!(ApiRecord)
                .field(fun!(api_record_count).name("count"))
                .field(fun!(api_record_label).name("label"))))
            .fun(fun!(api_record_rows).callback_return_only().expand_return(expand_return!(ApiRecord)
                .field(fun!(api_record_count).name("count"))
                .field(fun!(api_record_label).name("label"))))
            .fun(fun!(owned_resource_id))
            .fun(fun!(owned_resource_drops)))
        .package(package!("resourceViews")
            .fun(fun!(resource_bundle_views).expand_return(
                expand_return!(ResourceBundle).field_self_into().field(fun!(resource_bundle_label)))))
        .package(package!("values")
            .class(data_class!(DataEnvelope).name("Envelope"))
            .class(data_class!(DataFloats))
                .class(data_class!(DataPrimitives))
                .fun(fun!(data_primitives_echo).expand_param("value", expand_param!(DataPrimitives).variant_self().variant(fun!(data_primitives_zero))))
                .fun(fun!(data_primitives_zero))
            .fun(fun!(data_envelope_echo))
                .fun(fun!(data_with_storage))
                .fun(fun!(data_envelope_constructed).expand_param("value", expand_param!(DataEnvelope).variant(fun!(data_envelope_from_blob))))
            .fun(fun!(data_floats_echo))
            .fun(fun!(blob_value_selected).expand_param("value",
                expand_param!(BlobValue).variant_self().variant(fun!(blob_value_new))))
            .fun(fun!(blob_value_views).expand_return(
                expand_return!(BlobValue).field_self_ref().field_self().field(fun!(blob_value_copy)))))
        // The deliberately-unbound group (C-tier shapes with no JVM mapping):
        // acknowledged so the build log stays free of "skipping undeclared"
        // warnings without emitting anything.
        /* Unsupported: ignore diagnostics are not implemented.
.ignore(matching(|name| name.starts_with("storage_get_into_")))
*/
        /* Unsupported: ignore diagnostics are not implemented.
.ignore(fun!(storage_put_by_read_and_update))
*/;

    // Two prebindgen sources: the flat crate plus the binding-side helper crate
    // (conversion fns for `convert!`). The registry records each fn's origin from
    // the `SourceLocation` stamps so generated calls qualify with the defining
    // crate (`perftest_flat::…` vs `cov_helpers::…`). The helper dependency is
    // RENAMED in Cargo.toml (`cov_helpers = { package = "covertest-helpers", .. }`),
    // so the stamp recorded at capture time (`covertest-helpers`) would not
    // resolve from this crate — `source_named` overrides it with the name this
    // crate actually uses, per directory.
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

    */
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let jni = binding.build().expect("build supported declarations");
    jni.write_rust("bindings.rs").unwrap();
    let kotlin = out.join("kotlin");
    if kotlin.exists() {
        std::fs::remove_dir_all(&kotlin).unwrap();
    }
    jni.write_kotlin(&kotlin).unwrap();
}
