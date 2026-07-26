//! Extended `#[prebindgen]` surface primarily used to exercise language-binding
//! generator features that the lean performance surface in [`crate`] does not
//! need. Most items exist so a *coverage* binding (e.g.
//! `examples/covertest-kotlin`) can map one flat library through **every**
//! adapter feature and assert the result; the `ObjectBoundary*` family also
//! supports the flattened-vs-`JObject` JNI input micro-benchmark.
//!
//! Everything here is re-exported at the crate root (`pub use ext::*`), so a
//! single `source_module = perftest_flat` reaches both the perf surface and this
//! one. The items extend the same Storage/Payload domain with light
//! "introspection / analytics" helpers:
//!
//! * [`Priority`] — a `#[repr(i32)]` enum (→ Kotlin `enum class`).
//! * [`Stamp`] — a small `Copy` value (→ Kotlin `@JvmInline value class` over a
//!   `ByteArray`); `Vec<Stamp>` surfaces as `List<ByteArray>`.
//! * [`StorageError`] — the `E` of a fallible `Result` (→ the `onError` channel).
//! * [`Summary`] — an opaque handle whose fields decompose at the boundary
//!   (→ flatten-input / flatten-output).
//! * [`Millis`] — a newtype crossing as a plain `Long` via a custom
//!   input/output wrapper.
//! * [`Duration`] — the standard-library semantic type crossing as bounded
//!   milliseconds, with `Option<Duration>` using an invalid representation as
//!   an allocation-free niche.

pub use std::time::Duration;

use prebindgen_proc_macro::prebindgen;

use crate::{Payload, Storage};

// ─────────────────────────────────────────────────────────────────────────────
// Priority — a primitive-repr enum (→ Kotlin `enum class`, jint wire).
// ─────────────────────────────────────────────────────────────────────────────

/// Coarse importance bucket derived from a payload's `value`. A C-like
/// `#[repr(i32)]` enum with explicit discriminants, mapped by the binding to a
/// Kotlin `enum class` (and a C enum).
#[prebindgen]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// Classify a payload by magnitude of its `value` (enum **return**).
#[prebindgen]
pub fn payload_priority(p: &Payload) -> Priority {
    let v = p.value.abs();
    if v < 10.0 {
        Priority::Low
    } else if v < 100.0 {
        Priority::High
    } else {
        Priority::Normal
    }
}

/// Numeric weight of a priority (enum **by-value parameter**).
#[prebindgen]
pub fn priority_weight(p: Priority) -> i32 {
    match p {
        Priority::Low => 1,
        Priority::Normal => 5,
        Priority::High => 10,
    }
}

/// Resolve an optional priority against a fallback (`Option<enum>` parameter).
#[prebindgen]
pub fn priority_or(p: Option<Priority>, fallback: Priority) -> Priority {
    p.unwrap_or(fallback)
}

// ─────────────────────────────────────────────────────────────────────────────
// Reading — a data-carrying enum, i.e. a sum type (→ Kotlin `sealed interface`).
// ─────────────────────────────────────────────────────────────────────────────

/// A sensor reading: exactly **one** of these alternatives is live, and it
/// carries that alternative's payload. Written as plain Rust — the "exactly
/// one of" invariant is in the type, not in a doc comment on a struct of
/// optional fields.
///
/// All four variant shapes a sum can take are here: a payload-less variant, a
/// single-payload tuple variant, a multi-field named variant, and a tuple
/// variant whose payloads include a declared `enum_class`. The binding maps it
/// to a Kotlin `sealed interface` with the variants nested inside
/// (`lang::JniGen` `sealed_class!`).
#[prebindgen]
#[derive(Clone, Debug, PartialEq)]
pub enum Reading {
    /// No reading — the empty payload group; only the tag is live.
    Missing,
    /// An exact value (single-payload tuple variant).
    Exact(i64),
    /// A bounded interval (multi-field named variant).
    Range { low: i64, high: i64 },
    /// A described reading: a `String` beside a declared `enum_class` payload.
    Labeled(String, Priority),
    /// A variant whose name collides with the Kotlin **companion object** the
    /// binding emits to hold `fromParts`. The source crate keeps the name it
    /// wants: `Companion` is not reserved by Kotlin, it is the generator's own
    /// default, so the generator renames *its* companion instead.
    Companion(i64),
}

/// A data class carrying a **sum** as a field — the position where "exactly one
/// of" composes with ordinary product data.
///
/// `reading` is required and `fallback` optional, so the binding has to gate a
/// tag *and* a present flag independently; both sit beside already-flattened
/// siblings (`id`, `note`) so the tag-gated groups must interleave correctly
/// with ordinary leaves rather than only working in isolation. `Reading`'s
/// `Labeled` arm carries a `String`, which is the payload that proves an inert
/// group's object slot is wire-defaulted to null and therefore nullable in the
/// generated `fromParts`.
#[prebindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub id: i64,
    pub reading: Reading,
    pub fallback: Option<Reading>,
    pub note: String,
}

/// The `Reading` alternative selected by `which` (declaration order, so it is
/// the same numbering as the generated tag).
fn reading_for(which: i32) -> Reading {
    match which {
        0 => Reading::Missing,
        1 => Reading::Exact(42),
        2 => Reading::Range { low: 1, high: 9 },
        3 => Reading::Labeled("warm".to_string(), Priority::High),
        _ => Reading::Companion(5),
    }
}

/// Build an [`Observation`] carrying the selected alternative, optionally with
/// a `fallback` (the next alternative round-robin) — a **sum as a struct
/// field** crossing Rust → Kotlin, required and optional in one value.
#[prebindgen]
pub fn observation_new(which: i32, with_fallback: bool) -> Observation {
    Observation {
        id: 7,
        reading: reading_for(which),
        fallback: with_fallback.then(|| reading_for((which + 1) % 5)),
        note: "obs".to_string(),
    }
}

/// A second sum whose payload is **not leaf-shaped**: `Option<Priority>` is an
/// enum object (or null) in the JVM slot, which the tag-gated flat form cannot
/// express. The binding therefore lets this one cross as a whole object through
/// its own converter rather than failing — the degradation path — which is also
/// what exercises the `Option<enum>` property read.
#[prebindgen]
#[derive(Clone, Debug, PartialEq)]
pub enum Marker {
    None_,
    Ranked(Option<Priority>),
}

/// A data class carrying the object-shaped sum.
#[prebindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Tagged {
    pub id: i64,
    pub marker: Marker,
}

/// Build a [`Tagged`]: `which` 0 = `None_`, 1 = `Ranked(None)`, 2 = `Ranked(Some(High))`.
#[prebindgen]
pub fn tagged_new(which: i32) -> Tagged {
    Tagged {
        id: 3,
        marker: match which {
            0 => Marker::None_,
            1 => Marker::Ranked(None),
            _ => Marker::Ranked(Some(Priority::High)),
        },
    }
}

/// Read it back — the whole-object sum decode, including the `Option<enum>`
/// payload, crossing Kotlin → Rust.
#[prebindgen]
pub fn tagged_rank(t: Tagged) -> i32 {
    match t.marker {
        Marker::None_ => -1,
        Marker::Ranked(None) => 0,
        Marker::Ranked(Some(p)) => priority_weight(p),
    }
}

/// The selected alternative as the function's **own return** — a sum in
/// return position, where nothing but the value's own tag says which group is
/// live. Unlike a struct field (whose slots ride the parent's `fromParts`),
/// there is no surrounding product to carry the tag, so the decomposition
/// itself has to.
#[prebindgen]
pub fn reading_of(which: i32) -> Reading {
    reading_for(which)
}

/// `Option<sum>` return: `which < 0` yields `None`. Optionality and choice stay
/// independent — the present layer nulls the whole result rather than becoming
/// an extra tag value.
#[prebindgen]
pub fn reading_maybe(which: i32) -> Option<Reading> {
    (which >= 0).then(|| reading_for(which))
}

/// `Vec<sum>` return: alternatives `0..n`, each folded into the foreign list
/// element by element.
#[prebindgen]
pub fn reading_series(n: i32) -> Vec<Reading> {
    (0..n).map(reading_for).collect()
}

/// A sum as a **callback argument**: alternatives `0..n` delivered in turn.
#[prebindgen]
pub fn reading_each(n: i32, sink: impl Fn(Reading) + Send + Sync + 'static) {
    for i in 0..n {
        sink(reading_for(i));
    }
}

/// A sum whose alternatives are a **payload-less** variant and one carrying an
/// opaque **handle** — the shape a real lookup/reply result takes, and the one
/// that proves a tag-gated group can own a native resource: the live group
/// hands over a fresh handle, the inert group's slot stays a null pointer that
/// is never wrapped.
#[prebindgen]
#[derive(Clone)]
pub enum Lookup {
    /// Nothing matched — only the tag is live.
    Absent,
    /// What matched, as a handle the caller owns (and must close).
    Found(Summary),
    /// Why the lookup could not run — a `String` beside the handle group, so an
    /// inert object slot is exercised alongside an inert primitive one.
    Failed(String),
}

/// Build a [`Lookup`]: `count < 0` is a failure, `count == 0` is absent,
/// anything else is found.
#[prebindgen]
pub fn lookup_of(count: i64, total: f64) -> Lookup {
    match count {
        c if c < 0 => Lookup::Failed("negative count".to_string()),
        0 => Lookup::Absent,
        c => Lookup::Found(summary_new(c, total)),
    }
}

/// A handle-carrying sum as a **callback argument** — the same `Lookup` that
/// [`lookup_of`] returns, arriving through `impl Fn` instead. Alternatives are
/// delivered in turn (`Failed`, `Absent`, `Found`, then `Found` again), so a
/// live group hands a native resource to the callback while the inert groups'
/// slots stay defaulted. A sum payload is a plan LEAF, so the handle is wrapped
/// but not closed by the proxy: it is the callback body's to close, exactly as
/// for a returned sum.
#[prebindgen]
pub fn lookup_each(n: i64, total: f64, sink: impl Fn(Lookup) + Send + Sync + 'static) {
    for i in 0..n {
        sink(lookup_of(i - 1, total));
    }
}

/// Which alternative an [`Observation`]'s `reading` holds, by declaration
/// order — the sum crossing back **in** as part of a data-class parameter.
#[prebindgen]
pub fn observation_which(o: Observation) -> i32 {
    match o.reading {
        Reading::Missing => 0,
        Reading::Exact(_) => 1,
        Reading::Range { .. } => 2,
        Reading::Labeled(_, _) => 3,
        Reading::Companion(_) => 4,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stamp — a small `Copy` value type (→ Kotlin value class over raw bytes).
// ─────────────────────────────────────────────────────────────────────────────

/// A plain `Copy` timestamp. Declared `value_class` in the binding, so it
/// crosses **by value as its raw bytes** in a `ByteArray` (no heap handle, no
/// `close()`), and `Vec<Stamp>` surfaces as `List<ByteArray>`.
#[prebindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

/// Build a [`Stamp`] (value-class **return**).
#[prebindgen]
pub fn stamp_new(secs: i64, nanos: i64) -> Stamp {
    Stamp { secs, nanos }
}

/// Seconds component (value-class **accessor**, receiver = the value bytes).
#[prebindgen]
pub fn stamp_secs(s: &Stamp) -> i64 {
    s.secs
}

/// Nanoseconds component (value-class **accessor**).
#[prebindgen]
pub fn stamp_nanos(s: &Stamp) -> i64 {
    s.nanos
}

/// A monotonically increasing run of stamps (`Vec<value-class>` →
/// `List<ByteArray>`).
#[prebindgen]
pub fn stamp_series(count: i64) -> Vec<Stamp> {
    (0..count.max(0))
        .map(|i| Stamp { secs: i, nanos: 0 })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// StorageError — the `E` of a fallible `Result` (→ the `onError` channel).
// ─────────────────────────────────────────────────────────────────────────────

/// Failure value for the fallible storage constructor. Never crosses as a
/// value: the binding peels the `Result`, renders the message through
/// [`storage_error_message`], and delivers it to the caller's `onError`.
#[derive(Debug)]
pub struct StorageError {
    message: String,
}

/// Render a [`StorageError`] as its message (the error's flatten-output
/// **accessor**, fed to `onError`).
#[prebindgen]
pub fn storage_error_message(e: &StorageError) -> String {
    e.message.clone()
}

/// Build a storage seeded with a single labelled payload, **failing** on an
/// empty label (`Result<T, E>` routing + a `&str` input).
#[prebindgen]
pub fn storage_try_with_label(label: &str) -> Result<Storage, StorageError> {
    if label.is_empty() {
        return Err(StorageError {
            message: "label must not be empty".to_string(),
        });
    }
    Ok(Storage {
        payloads: vec![Payload {
            id: 0,
            seq: 0,
            value: 0.0,
            flag: false,
            label: Some(Box::new(label.to_string())),
        }],
    })
}

/// Build a storage stamped with `s`, **failing** on a non-positive `secs` (a
/// domain [`StorageError`]). This takes a `Stamp` **by value** (a value-blob
/// input), so a malformed `Stamp` blob fails the input decode FIRST — the
/// binding channel — while a well-formed but rejected value fails in the domain
/// channel. It is the covertest exercise for issue #45's two-caller split: one
/// wrapper, both `onBindingError` and `onError` provable independently.
#[prebindgen]
pub fn storage_try_from_stamp(s: Stamp) -> Result<Storage, StorageError> {
    if s.secs <= 0 {
        return Err(StorageError {
            message: "stamp secs must be positive".to_string(),
        });
    }
    Ok(Storage {
        payloads: vec![Payload {
            id: s.secs,
            seq: 0,
            value: 0.0,
            flag: false,
            label: None,
        }],
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Summary — an opaque handle whose fields decompose at the boundary.
// ─────────────────────────────────────────────────────────────────────────────

/// An aggregate over a [`Storage`]'s payloads: how many there are and the sum of
/// their `value`s. An opaque handle in the binding, but its default
/// flatten-output decomposes it into `(count, total)` leaves and its
/// flatten-input rebuilds it from the same leaves (via [`summary_new`]).
/// `Clone` because [`archive_latest`] returns it *borrowed* (`Option<&Summary>`)
/// and the JVM binding's only sound lowering of a borrowed handle is a clone
/// into a fresh owned handle.
#[derive(Clone)]
pub struct Summary {
    count: i64,
    total: f64,
}

/// Construct a [`Summary`] from its parts (declared a **constructor** /
/// companion factory, and the build-from **variant** of the flatten-input).
#[prebindgen]
pub fn summary_new(count: i64, total: f64) -> Summary {
    Summary { count, total }
}

/// Number of payloads (flatten-output **field** / **accessor**).
#[prebindgen]
pub fn summary_count(s: &Summary) -> i64 {
    s.count
}

/// Sum of payload values (flatten-output **field** / **accessor**).
#[prebindgen]
pub fn summary_total(s: &Summary) -> f64 {
    s.total
}

/// Total scaled by a factor (an instance **method**: `&Self` receiver + arg).
#[prebindgen]
pub fn summary_scaled(s: &Summary, factor: f64) -> f64 {
    s.total * factor
}

/// A series of `count` summaries starting at `start`: element `i` is
/// `(start + i, (start + i) * 10.0)`. A **record-built iterable fold** at the
/// boundary: the caller supplies the accumulator and a per-element `fold`
/// lambda receiving the decomposed `(count, total)` leaves.
#[prebindgen]
pub fn summary_series(count: i64, start: i64) -> Vec<Summary> {
    (0..count)
        .map(|i| summary_new(start + i, ((start + i) * 10) as f64))
        .collect()
}

/// Like [`summary_series`] but `None` when `count < 0` — the record-built
/// `Optional(Iterable)` shape (#105): `None` skips the fold and the JVM
/// wrapper returns null; `Some(vec![])` returns the untouched accumulator.
#[prebindgen]
pub fn summary_series_opt(count: i64, start: i64) -> Option<Vec<Summary>> {
    (count >= 0).then(|| summary_series(count, start))
}

/// Summarize a storage (returns a `Summary`; the binding's **default
/// flatten-output** turns it into `(count, total)` leaves).
#[prebindgen]
pub fn storage_summary(s: &Storage) -> Summary {
    Summary {
        count: s.payloads.len() as i64,
        total: s.payloads.iter().map(|p| p.value).sum(),
    }
}

/// Whether `expected` matches the storage's live summary (takes a `Summary`
/// **parameter**; the binding's **default flatten-input** rebuilds it from
/// `(count, total)` or accepts a handle).
#[prebindgen]
pub fn storage_matches_summary(s: &Storage, expected: Summary) -> bool {
    let live = storage_summary(s);
    live.count == expected.count && (live.total - expected.total).abs() < f64::EPSILON
}

/// Combine two summaries (#87 regression: BOTH parameters are splittable under
/// the `Summary` flatten-input default AND the `Summary` return is delivered
/// through the decomposed builder — the wrapper is generic over `<R>`, and
/// every split overload must re-declare it).
#[prebindgen]
pub fn summary_merge(primary: Summary, fallback: Summary) -> Summary {
    Summary {
        count: primary.count + fallback.count,
        total: primary.total + fallback.total,
    }
}

/// Like [`storage_summary`] but the binding keeps the result as a raw opaque
/// handle (per-fn **flatten-output-suppress**).
#[prebindgen]
pub fn storage_summary_handle(s: &Storage) -> Summary {
    storage_summary(s)
}

/// Read a summary's total through a raw handle (per-fn **flatten-input-suppress**
/// on the `Summary` parameter).
#[prebindgen]
pub fn summary_total_raw(s: Summary) -> f64 {
    s.total
}

/// Like [`storage_summary`] but the binding decomposes it with a **custom**
/// field set that also keeps the handle (per-fn **flatten-output-with**).
#[prebindgen]
pub fn storage_summary_full(s: &Storage) -> Summary {
    storage_summary(s)
}

/// Like [`storage_summary`] but the binding's per-fn field set carries a
/// **binding-local conditional field** (`field!("handle").with(ty!, path!)`):
/// the handle leaf is delivered only when the binding-side predicate says
/// re-using the value is worth it (the zenoh conditional-Encoding idiom).
#[prebindgen]
pub fn storage_summary_probe(s: &Storage) -> Summary {
    storage_summary(s)
}

/// Set the storage's "expected" summary, accepting a `Summary` built via an
/// explicit per-fn **flatten-input-with** variant list. Returns whether it
/// matched the live summary before being consumed.
#[prebindgen]
pub fn storage_expect_summary(s: &mut Storage, expected: Summary) -> bool {
    let live = storage_summary(s);
    live.count == expected.count && (live.total - expected.total).abs() < f64::EPSILON
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage scalar members — accessor / method / constructor on the existing
// opaque handle.
// ─────────────────────────────────────────────────────────────────────────────

/// Number of stored payloads (an **accessor** on `Storage`).
#[prebindgen]
pub fn storage_len(s: &Storage) -> i64 {
    s.payloads.len() as i64
}

/// Whether any stored payload has the given id (a **method** on `Storage`).
#[prebindgen]
pub fn storage_contains(s: &Storage, id: i64) -> bool {
    s.payloads.iter().any(|p| p.id == id)
}

/// Build a storage holding a single payload (a **constructor** / companion
/// factory on `Storage`).
#[prebindgen]
pub fn storage_with_payload(payload: Payload) -> Storage {
    Storage {
        payloads: vec![payload],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Millis — a newtype crossing as a plain `Long` via a custom wrapper.
// ─────────────────────────────────────────────────────────────────────────────

/// A duration in milliseconds. The binding registers a custom
/// `input_wrapper`/`output_wrapper` mapping it to a plain `Long` (no generated
/// class), so it never appears as a Kotlin type of its own. It is intentionally
/// **not** `#[prebindgen]`: the wrapper fully owns its boundary conversion, and
/// marking it would make the Kotlin emitter try to render this tuple struct as a
/// data class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Millis(pub u64);

/// Sum two durations (exercises the custom wrapper on both a **parameter** and
/// the **return**).
#[prebindgen]
pub fn millis_add(a: Millis, b: Millis) -> Millis {
    Millis(a.0 + b.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration — std semantic type crossing as bounded u64 milliseconds.
// ─────────────────────────────────────────────────────────────────────────────

/// Largest duration accepted by the covertest binding: one day in
/// milliseconds. The binding's representation-domain declaration reserves all
/// larger `u64` values, allowing `Option<Duration>` to use one as `None` while
/// keeping the JNI carrier a primitive `jlong`.
pub const DURATION_MAX_MILLIS: u64 = 86_400_000;

/// Round-trip an optional standard-library duration. The source API remains
/// semantic (`Option<Duration>`); only the binding declares its millisecond
/// representation and range.
#[prebindgen]
pub fn duration_optional(value: Option<Duration>) -> Option<Duration> {
    value
}

/// Deliberately violate the binding's declared output domain so the Kotlin
/// covertest can verify outbound validation and error routing.
#[prebindgen]
pub fn duration_out_of_range() -> Option<Duration> {
    Some(Duration::from_millis(DURATION_MAX_MILLIS + 1))
}

/// Data-class composition probe for the bounded duration representation.
/// The coverage binding deliberately marks this class `.jobject_input()` so
/// its echo executes both the whole-object input decoder and the `fromParts`
/// output encoder; the nullable duration itself still uses the raw `jlong`
/// niche whenever it crosses a generated JNI call boundary.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurationBoundary {
    pub delay: Option<Duration>,
}

/// Round-trip [`DurationBoundary`] through the explicit object-input bridge.
#[prebindgen]
pub fn duration_boundary_echo(value: &DurationBoundary) -> DurationBoundary {
    value.clone()
}

// ─────────────────────────────────────────────────────────────────────────────
// convert! source-kind fixtures — one type per conversion source. Like
// `Millis`, none of these types is `#[prebindgen]`-marked: each crosses the
// boundary only through its declared canonical conversion.
// ─────────────────────────────────────────────────────────────────────────────

/// A temperature. Crosses via its `From`/`Into` impls
/// (`convert!(Celsius).input_from(ty!(i32)).output_into(ty!(i32))`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Celsius(pub i32);

impl From<i32> for Celsius {
    fn from(v: i32) -> Self {
        Celsius(v)
    }
}
impl From<Celsius> for i32 {
    fn from(c: Celsius) -> Self {
        c.0
    }
}

/// Double a temperature (exercises the `Into`-based conversion on a
/// parameter and the return).
#[prebindgen]
pub fn celsius_double(c: Celsius) -> Celsius {
    Celsius(c.0 * 2)
}

/// A percentage, range-invariant 0..=100. Crosses via a fallible
/// `TryFrom<i32>` on input (out-of-range i32 from the JVM → the caller's
/// error handler) and an infallible `Into<i32>` on output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Percent(pub u8);

impl TryFrom<i32> for Percent {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        if (0..=100).contains(&v) {
            Ok(Percent(v as u8))
        } else {
            Err(format!("percent out of range: {v} (expected 0..=100)"))
        }
    }
}
impl From<Percent> for i32 {
    fn from(p: Percent) -> Self {
        p.0 as i32
    }
}

/// Scale a percentage, saturating at 100 (exercises the `TryInto`-based
/// input conversion — including its error path — and the `Into` output).
#[prebindgen]
pub fn percent_scale(p: Percent, factor: i32) -> Percent {
    Percent(((p.0 as i32) * factor).clamp(0, 100) as u8)
}

/// Round-trip an optional percentage. The covertest binding uses this to
/// compose `Option` with the fallible `TryFrom<i32>` input conversion and its
/// fallible output conversion.
#[prebindgen]
pub fn percent_optional(p: Option<Percent>) -> Option<Percent> {
    p
}

/// Deliberately construct a value outside `Percent`'s semantic invariant so
/// the covertest can verify a fallible output stage nested under `Option`.
#[prebindgen]
pub fn percent_invalid_output() -> Option<Percent> {
    Some(Percent(101))
}

/// A text label. Crosses via plain conversion fns declared **in the binding
/// crate** (`convert!(Label).input_with(ty!(String), path!(crate::label_in))…`)
/// — no `#[prebindgen]` marking anywhere in the conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label(pub String);

/// Reverse a label's characters (exercises the binding-local conversion on
/// a parameter and the return).
#[prebindgen]
pub fn label_reverse(l: Label) -> Label {
    Label(l.0.chars().rev().collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// Option<scalar> — a nullable primitive return.
// ─────────────────────────────────────────────────────────────────────────────

/// Length of a payload's label, or `None` when it is unlabeled. Exercises an
/// `Option<i64>` (nullable primitive) return, distinct from the `Option<handle>`
/// / `Option<data-class>` shapes elsewhere.
#[prebindgen]
pub fn payload_label_len(p: &Payload) -> Option<i64> {
    p.label.as_ref().map(|s| s.len() as i64)
}

// ─────────────────────────────────────────────────────────────────────────────
// Annotated — a data class with a NESTED data-class field and Option<scalar> /
// Option<enum> fields.
// ─────────────────────────────────────────────────────────────────────────────

/// A [`Payload`] with optional delivery metadata. As a `data_class` it
/// exercises the shapes flat `Payload` cannot: a **nested** data-class field
/// (`payload`, recursive `fromParts` on output / recursive leaf decode on
/// input) and `Option<primitive>` / `Option<enum>` **fields** (each crossing
/// as a decoupled `(present, value)` leaf pair).
#[prebindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Annotated {
    pub payload: Payload,
    pub alternate: Option<Payload>,
    pub ttl: Option<i64>,
    pub priority: Option<Priority>,
}

/// Assemble an [`Annotated`] (nested data-class **output** + bare
/// `Option<scalar>` / `Option<enum>` inputs).
#[prebindgen]
pub fn annotated_new(payload: Payload, ttl: Option<i64>, priority: Option<Priority>) -> Annotated {
    Annotated {
        payload,
        alternate: None,
        ttl,
        priority,
    }
}

/// The optional nested payload's value. Its `Option<data_class>` input leaves
/// are guarded by one presence bit and recursively reconstructed only when
/// present.
#[prebindgen]
pub fn annotated_alternate_value(a: &Annotated) -> Option<f64> {
    a.alternate.as_ref().map(|payload| payload.value)
}

/// The metadata TTL (`Option<prim>` field read back through a data-class
/// **input**).
#[prebindgen]
pub fn annotated_ttl(a: &Annotated) -> Option<i64> {
    a.ttl
}

/// The metadata priority (`Option<enum>` **return**).
#[prebindgen]
pub fn annotated_priority(a: &Annotated) -> Option<Priority> {
    a.priority
}

/// The nested payload's `value` (proves the nested field survived the
/// input decode).
#[prebindgen]
pub fn annotated_payload_value(a: &Annotated) -> f64 {
    a.payload.value
}

// ─────────────────────────────────────────────────────────────────────────────
// CacheConfig / RepliesConfig — a non-null enum field reached through an outer
// `Option<data_class>` (the nullable-context input path, issue #144).
// ─────────────────────────────────────────────────────────────────────────────

/// Inner, **non-optional** delivery config carrying a **non-null enum field**
/// (`priority`). Nested inside [`CacheConfig`], which crosses as
/// `Option<CacheConfig>`, so the outer optional's `nullable_context`
/// propagates down to this non-optional struct — the exact shape that made the
/// JNI input builder emit a dead double-Elvis default (`?.value ?: 0 ?: 0`)
/// for a non-null enum field (#144).
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepliesConfig {
    pub priority: Priority,
    pub max_samples: i64,
}

/// Outer cache config crossed as `Option<CacheConfig>`. Its optional-ness
/// propagates into the non-optional nested [`RepliesConfig`], whose non-null
/// `priority` enum field must decode with exactly **one** Elvis default (#144).
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheConfig {
    pub replies: RepliesConfig,
    pub ttl: i64,
}

/// The cache's replies-priority weight plus its `ttl`, or `-1` when the cache
/// is absent (`Option<CacheConfig>` **input** — the #144 reproduction: a
/// non-null enum field reached through the outer optional data class).
#[prebindgen]
pub fn cache_config_weight(cache: Option<CacheConfig>) -> i32 {
    match cache {
        Some(c) => priority_weight(c.replies.priority) + c.ttl as i32,
        None => -1,
    }
}

/// One `i64` leaf in the deliberately wide [`ObjectBoundary`] tree.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectBoundaryLeaf {
    pub value: i64,
}

macro_rules! object_boundary_level {
    ($name:ident, $child:ty) => {
        #[prebindgen]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            pub left: $child,
            pub right: $child,
        }
    };
}

object_boundary_level!(ObjectBoundary2, ObjectBoundaryLeaf);
object_boundary_level!(ObjectBoundary4, ObjectBoundary2);
object_boundary_level!(ObjectBoundary8, ObjectBoundary4);
object_boundary_level!(ObjectBoundary16, ObjectBoundary8);
object_boundary_level!(ObjectBoundary32, ObjectBoundary16);
object_boundary_level!(ObjectBoundary64, ObjectBoundary32);

/// Structural twin of [`ObjectBoundary64`] used to benchmark an explicit
/// whole-`JObject` input against recursive 64-leaf flattening.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectBoundary64Object {
    pub left: ObjectBoundary32,
    pub right: ObjectBoundary32,
}

/// The right half of [`ObjectBoundary`]: 32 + 16 + 8 + 4 + 2 + 1 leaves.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectBoundary63 {
    pub leaves32: ObjectBoundary32,
    pub leaves16: ObjectBoundary16,
    pub leaves8: ObjectBoundary8,
    pub leaves4: ObjectBoundary4,
    pub leaves2: ObjectBoundary2,
    pub leaf: ObjectBoundaryLeaf,
}

/// Deliberate object-boundary fixture for `data_class!(T).jobject_input()`.
///
/// Its [`ObjectBoundary64`] and [`ObjectBoundary63`] children recursively
/// contain 127 `i64` leaves. The generated Kotlin constructor/fromParts bridge
/// remains legal at 254 JVM slots, but flattening a native input parameter
/// would require 256: 254 for the leaves plus the `JNINative` receiver and
/// binding-error sink. Because the JVM limit is 255, this otherwise-valid data
/// class must cross Kotlin→Rust as one `JObject`.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectBoundary {
    pub left: ObjectBoundary64,
    pub right: ObjectBoundary63,
}

trait ObjectBoundarySum {
    fn boundary_sum(&self) -> i64;
}

impl ObjectBoundarySum for ObjectBoundaryLeaf {
    fn boundary_sum(&self) -> i64 {
        self.value
    }
}

macro_rules! impl_object_boundary_sum {
    ($($name:ty),+ $(,)?) => {
        $(
            impl ObjectBoundarySum for $name {
                fn boundary_sum(&self) -> i64 {
                    self.left.boundary_sum() + self.right.boundary_sum()
                }
            }
        )+
    };
}

impl_object_boundary_sum!(
    ObjectBoundary2,
    ObjectBoundary4,
    ObjectBoundary8,
    ObjectBoundary16,
    ObjectBoundary32,
    ObjectBoundary64,
    ObjectBoundary64Object,
);

impl ObjectBoundarySum for ObjectBoundary63 {
    fn boundary_sum(&self) -> i64 {
        self.leaves32.boundary_sum()
            + self.leaves16.boundary_sum()
            + self.leaves8.boundary_sum()
            + self.leaves4.boundary_sum()
            + self.leaves2.boundary_sum()
            + self.leaf.boundary_sum()
    }
}

impl ObjectBoundarySum for ObjectBoundary {
    fn boundary_sum(&self) -> i64 {
        self.left.boundary_sum() + self.right.boundary_sum()
    }
}

#[prebindgen]
pub fn object_boundary_value(value: &ObjectBoundary) -> i64 {
    value.boundary_sum()
}

/// Sum the 64 scalar leaves after recursive JNI parameter flattening.
#[prebindgen]
pub fn large_flat_input_sum(value: &ObjectBoundary64) -> i64 {
    value.boundary_sum()
}

/// Sum the same 64-leaf shape after decoding one whole `JObject` input.
#[prebindgen]
pub fn large_object_input_sum(value: &ObjectBoundary64Object) -> i64 {
    value.boundary_sum()
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixed-width unsigned integers — widened Kotlin scalars + ULong projection.
// ─────────────────────────────────────────────────────────────────────────────

/// Every fixed-width Rust unsigned scalar in one generated Kotlin data class.
/// The first three fields widen losslessly; `long`/`maybe_long` surface as
/// `ULong`/`ULong?` over raw JNI `Long` bit patterns.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unsigned {
    pub byte: u8,
    pub short: u16,
    pub int: u32,
    pub long: u64,
    pub maybe_long: Option<u64>,
}

/// Round-trip direct unsigned parameters through an unsigned data-class
/// return, covering both checked widening and the `ULong` projection.
#[prebindgen]
pub fn unsigned_round_trip(
    byte: u8,
    short: u16,
    int: u32,
    long: u64,
    maybe_long: Option<u64>,
) -> Unsigned {
    Unsigned {
        byte,
        short,
        int,
        long,
        maybe_long,
    }
}

/// Direct nullable `u64` projection in both directions.
#[prebindgen]
pub fn unsigned_optional(value: Option<u64>) -> Option<u64> {
    value
}

/// Read the optional `u64` field through the flattened data-class input ABI.
/// With no natural niche it crosses as `(present, raw Long)`, never a boxed
/// `java.lang.Long`/`JObject`.
#[prebindgen]
pub fn unsigned_data_maybe(value: &Unsigned) -> Option<u64> {
    value.maybe_long
}

/// Deliver a `u64` through the generated typed/raw callback twin.
#[prebindgen]
pub fn unsigned_emit(value: u64, f: impl Fn(u64) + Send + Sync + 'static) {
    f(value)
}

/// Output collection fold whose raw `jlong` leaves become `ULong` values on
/// the Kotlin side.
#[prebindgen]
pub fn unsigned_series() -> Vec<u64> {
    vec![0, u64::MAX]
}

// ─────────────────────────────────────────────────────────────────────────────
// Vec<opaque-handle> outputs — the Kotlin-side handle fold.
// ─────────────────────────────────────────────────────────────────────────────

fn synthetic_storage(shard: i64, each: i64) -> Storage {
    Storage {
        payloads: (0..each.max(0))
            .map(|k| Payload {
                id: shard * 1000 + k,
                seq: k as i32,
                value: k as f64,
                flag: false,
                label: None,
            })
            .collect(),
    }
}

/// Build `count` independent storages of `each` payloads (a
/// `Vec<opaque-handle>` **return** — each element crosses as a raw pointer the
/// Kotlin folder wraps into a typed `Storage` handle).
#[prebindgen]
pub fn storage_shards(count: i64, each: i64) -> Vec<Storage> {
    (0..count.max(0))
        .map(|i| synthetic_storage(i, each))
        .collect()
}

/// Like [`storage_shards`] but `None` when `count == 0`
/// (`Option<Vec<opaque-handle>>` — the fold under the null niche).
#[prebindgen]
pub fn storage_shards_opt(count: i64, each: i64) -> Option<Vec<Storage>> {
    if count <= 0 {
        None
    } else {
        Some(storage_shards(count, each))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StorageHandler — a callback receiving an OWNED opaque handle.
// ─────────────────────────────────────────────────────────────────────────────

/// A prepared callback receiving an **owned [`Storage`] handle** (`Fn(Storage)`,
/// by value). Unlike [`PayloadHandler`] (whose arg is a flattened data class),
/// the handle crosses as a raw pointer and the generated Kotlin proxy wraps it
/// into a typed `Storage` and `close()`s it after `run` (close-unless-taken).
pub struct StorageHandler(Box<dyn Fn(Storage) + Send + Sync>);

/// Wrap a `Fn(Storage)` closure into a reusable [`StorageHandler`].
#[prebindgen]
pub fn storage_handler_new(f: impl Fn(Storage) + Send + Sync + 'static) -> StorageHandler {
    StorageHandler(Box::new(f))
}

/// Build a synthetic storage of `n` payloads and hand **ownership** of it to
/// the handler's callback.
#[prebindgen]
pub fn storage_emit(n: i64, h: &StorageHandler) {
    (h.0)(synthetic_storage(0, n));
}

// ─────────────────────────────────────────────────────────────────────────────
// Archive — a borrowed-opaque output (`Option<&Summary>` → cloned owned handle).
// ─────────────────────────────────────────────────────────────────────────────

/// Holds the most recently stored [`Summary`]. Its accessor returns the summary
/// **borrowed** — the shape zenoh-flat's `z_*` accessors use for the C tier's
/// zero-copy borrows — which the JVM binding lowers by **cloning** into a fresh
/// owned handle (the JVM keeps its handle past the call).
pub struct Archive {
    latest: Option<Summary>,
    /// A sum the archive OWNS, so it can hand one back **borrowed** (`&Reading`)
    /// — the return shape whose encoder must match on the value behind the
    /// reference rather than moving it.
    reading: Reading,
    /// The same, optional, for the `Option<&Reading>` shape.
    fallback: Option<Reading>,
}

impl Default for Archive {
    fn default() -> Self {
        Self {
            latest: None,
            reading: Reading::Missing,
            fallback: None,
        }
    }
}

/// Create an empty archive.
#[prebindgen]
pub fn archive_new() -> Archive {
    Archive::default()
}

/// Store a summary, consuming it (owned-handle input).
#[prebindgen]
pub fn archive_store(a: &mut Archive, s: Summary) {
    a.latest = Some(s);
}

/// Store the `which` alternative as the archive's own reading, and the same one
/// as its optional fallback. A **negative** `which` clears the fallback and
/// resets the reading to the first alternative, so the two borrow shapes can be
/// exercised independently: `&Reading` always has a value, `Option<&Reading>`
/// does not.
#[prebindgen]
pub fn archive_set_reading(a: &mut Archive, which: i32) {
    a.reading = reading_for(which.max(0));
    a.fallback = (which >= 0).then(|| reading_for(which));
}

/// A sum returned **borrowed** (`&Reading`). The value stays owned by the
/// archive; the binding decomposes it in place — the encoder matches through
/// the reference instead of consuming — and the caller gets an ordinary
/// Kotlin value with no borrow to track.
#[prebindgen]
pub fn archive_reading(a: &Archive) -> &Reading {
    &a.reading
}

/// The optional layer over the same borrow (`Option<&Reading>`): `None` nulls
/// the whole result, exactly as for an owned `Option<Reading>`.
#[prebindgen]
pub fn archive_reading_maybe(a: &Archive) -> Option<&Reading> {
    a.fallback.as_ref()
}

/// The stored summary, borrowed (`Option<&Summary>` **return** — `None` when
/// empty, otherwise cloned into a fresh owned handle by the JVM binding).
#[prebindgen]
pub fn archive_latest(a: &Archive) -> Option<&Summary> {
    a.latest.as_ref()
}

// ─────────────────────────────────────────────────────────────────────────────
// Misc coverage shapes: 3-handle call, Vec<String> return, Option<data-class>
// input.
// ─────────────────────────────────────────────────────────────────────────────

/// Combined length of three storages (a **3-opaque-handle** call — the
/// generated wrapper must sort-lock all three).
#[prebindgen]
pub fn storage_total_len(a: &Storage, b: &Storage, c: &Storage) -> i64 {
    (a.payloads.len() + b.payloads.len() + c.payloads.len()) as i64
}

/// All present labels, in storage order (`Vec<String>` **return** — the
/// single-leaf string fold).
#[prebindgen]
pub fn storage_labels(s: &Storage) -> Vec<String> {
    s.payloads
        .iter()
        .filter_map(|p| p.label.as_deref().cloned())
        .collect()
}

/// Push `p` if present; whether it was pushed (`Option<data-class>` **input**).
#[prebindgen]
pub fn storage_put_opt(s: &mut Storage, p: Option<Payload>) -> bool {
    match p {
        Some(p) => {
            s.payloads.push(p);
            true
        }
        None => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Consts — declared via `PackageDecl::constant`, surfacing as generated JNI
// getters + lazily-initialized Kotlin top-level `val`s.
// ─────────────────────────────────────────────────────────────────────────────

/// The storage capacity limit advertised to bindings (a primitive const).
#[prebindgen]
pub const COVER_MAGIC: i64 = 0xC0FFEE;

/// The coverage surface's tag string (a string const).
#[prebindgen]
pub const COVER_TAG: &str = "covertest";

/// The tag with a runtime-computed suffix — a constant value no Rust `const`
/// can express (built through `format!`). Exercises
/// `PackageDecl::constant_fun`: a nullary fn surfaced as a
/// lazily-initialized Kotlin top-level `val`.
#[prebindgen]
pub fn cover_tag_runtime() -> String {
    format!("{COVER_TAG}-runtime")
}

// ─────────────────────────────────────────────────────────────────────────────
// EscapeProbe — JNI native-symbol escaping probe (#86).
// ─────────────────────────────────────────────────────────────────────────────

/// A tiny opaque handle whose covertest declaration puts underscores in every
/// symbol component (#86): it lives in the underscored `esc_pkg` subpackage
/// under the underscored Kotlin name `Esc_Probe`, and its accessor's harness
/// extern is mangled to an underscored method name — so its `freePtr`
/// destructor and accessor symbols only resolve at runtime if the generator
/// applies the JNI spec's `_1` escaping.
pub struct EscapeProbe {
    value: i64,
}

/// Construct an [`EscapeProbe`] (its covertest constructor).
#[prebindgen]
pub fn escape_probe_new(value: i64) -> EscapeProbe {
    EscapeProbe { value }
}

/// Read the probe's value (mangled to an underscored harness extern in
/// covertest, #86).
#[prebindgen]
pub fn escape_probe_value(p: &EscapeProbe) -> i64 {
    p.value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage_new, storage_put_slice};

    fn payload(id: i64, value: f64, label: Option<&str>) -> Payload {
        Payload {
            id,
            seq: id as i32,
            value,
            flag: id % 2 == 0,
            label: label.map(|s| Box::new(s.to_string())),
        }
    }

    #[test]
    fn priority_classifies_by_magnitude() {
        assert_eq!(payload_priority(&payload(1, 3.0, None)), Priority::Low);
        assert_eq!(payload_priority(&payload(1, 50.0, None)), Priority::High);
        assert_eq!(payload_priority(&payload(1, 500.0, None)), Priority::Normal);
        assert_eq!(priority_weight(Priority::High), 10);
        assert_eq!(priority_or(None, Priority::Normal), Priority::Normal);
        assert_eq!(
            priority_or(Some(Priority::Low), Priority::High),
            Priority::Low
        );
    }

    #[test]
    fn stamps_roundtrip() {
        let s = stamp_new(7, 42);
        assert_eq!(stamp_secs(&s), 7);
        assert_eq!(stamp_nanos(&s), 42);
        let series = stamp_series(3);
        assert_eq!(series.len(), 3);
        assert_eq!(series[2], Stamp { secs: 2, nanos: 0 });
        assert!(stamp_series(-1).is_empty());
    }

    #[test]
    fn fallible_label_constructor() {
        assert!(storage_try_with_label("").is_err());
        let s = storage_try_with_label("hi").expect("non-empty label");
        assert_eq!(storage_len(&s), 1);
        let err = storage_try_with_label("").err().unwrap();
        assert_eq!(storage_error_message(&err), "label must not be empty");
    }

    #[test]
    fn summary_aggregates_storage() {
        let mut s = storage_new();
        storage_put_slice(
            &mut s,
            &[payload(1, 10.0, None), payload(2, 30.0, Some("x"))],
        );
        let sum = storage_summary(&s);
        assert_eq!(summary_count(&sum), 2);
        assert_eq!(summary_total(&sum), 40.0);
        assert_eq!(summary_scaled(&sum, 2.0), 80.0);

        assert!(storage_matches_summary(&s, summary_new(2, 40.0)));
        assert!(!storage_matches_summary(&s, summary_new(1, 40.0)));
        assert_eq!(summary_total_raw(storage_summary_handle(&s)), 40.0);
        assert!(storage_expect_summary(&mut s, summary_new(2, 40.0)));
    }

    #[test]
    fn storage_scalar_members() {
        let s = storage_with_payload(payload(42, 1.0, Some("a")));
        assert_eq!(storage_len(&s), 1);
        assert!(storage_contains(&s, 42));
        assert!(!storage_contains(&s, 7));
    }

    #[test]
    fn millis_wrapper_arithmetic() {
        assert_eq!(millis_add(Millis(100), Millis(50)), Millis(150));
    }

    #[test]
    fn label_len_is_optional() {
        assert_eq!(payload_label_len(&payload(1, 0.0, Some("abcd"))), Some(4));
        assert_eq!(payload_label_len(&payload(1, 0.0, None)), None);
    }

    #[test]
    fn annotated_roundtrips() {
        let a = annotated_new(payload(1, 2.5, Some("x")), Some(30), Some(Priority::High));
        assert_eq!(annotated_ttl(&a), Some(30));
        assert_eq!(annotated_priority(&a), Some(Priority::High));
        assert_eq!(annotated_payload_value(&a), 2.5);
        assert_eq!(annotated_alternate_value(&a), None);
        let with_alternate = Annotated {
            alternate: Some(payload(2, 7.5, None)),
            ..a.clone()
        };
        assert_eq!(annotated_alternate_value(&with_alternate), Some(7.5));
        let b = annotated_new(payload(1, 0.0, None), None, None);
        assert_eq!(annotated_ttl(&b), None);
        assert_eq!(annotated_priority(&b), None);
    }

    #[test]
    fn cache_config_weight_reaches_nested_enum() {
        let cache = CacheConfig {
            replies: RepliesConfig {
                priority: Priority::High,
                max_samples: 4,
            },
            ttl: 7,
        };
        // priority_weight(High) == 10, plus ttl 7.
        assert_eq!(cache_config_weight(Some(cache)), 17);
        assert_eq!(cache_config_weight(None), -1);
    }

    #[test]
    fn shards_are_independent() {
        let shards = storage_shards(3, 2);
        assert_eq!(shards.len(), 3);
        assert!(shards.iter().all(|s| storage_len(s) == 2));
        assert!(storage_contains(&shards[2], 2001));
        assert!(!storage_contains(&shards[0], 2001));
        assert!(storage_shards(0, 2).is_empty());
        assert!(storage_shards_opt(0, 2).is_none());
        assert_eq!(storage_shards_opt(2, 1).unwrap().len(), 2);
    }

    #[test]
    fn summary_series_shapes() {
        let s = summary_series(3, 10);
        assert_eq!(s.len(), 3);
        assert_eq!(summary_count(&s[2]), 12);
        assert_eq!(summary_total(&s[2]), 120.0);
        assert!(summary_series(0, 5).is_empty());
        assert!(summary_series_opt(-1, 0).is_none());
        assert!(summary_series_opt(0, 0).unwrap().is_empty());
        assert_eq!(summary_series_opt(2, 1).unwrap().len(), 2);
    }

    #[test]
    fn storage_handler_receives_owned_storage() {
        use std::sync::{
            atomic::{AtomicI64, Ordering},
            Arc,
        };
        let seen = Arc::new(AtomicI64::new(-1));
        let seen2 = seen.clone();
        let h = storage_handler_new(move |s| seen2.store(storage_len(&s), Ordering::SeqCst));
        storage_emit(4, &h);
        assert_eq!(seen.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn archive_borrows_latest() {
        let mut a = archive_new();
        assert!(archive_latest(&a).is_none());
        archive_store(&mut a, summary_new(2, 40.0));
        assert_eq!(summary_count(archive_latest(&a).unwrap()), 2);
    }

    #[test]
    fn misc_shapes() {
        let s1 = storage_with_payload(payload(1, 0.0, Some("a")));
        let s2 = storage_with_payload(payload(2, 0.0, None));
        let mut s3 = storage_new();
        assert_eq!(storage_total_len(&s1, &s2, &s3), 2);
        assert_eq!(storage_labels(&s1), vec!["a".to_string()]);
        assert!(storage_labels(&s2).is_empty());
        assert!(storage_put_opt(&mut s3, Some(payload(3, 0.0, None))));
        assert!(!storage_put_opt(&mut s3, None));
        assert_eq!(storage_len(&s3), 1);
    }
}
