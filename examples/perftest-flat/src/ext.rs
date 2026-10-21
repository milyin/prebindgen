//! Coverage source with primitive leaves, byte records and Report projections.
// Extended `#[prebindgen]` surface primarily used to exercise language-binding
// generator features that the lean performance surface in [`crate`] does not
// need. Most items exist so a *coverage* binding (e.g.
// `examples/covertest-kotlin`) can map one flat library through **every**
// adapter feature and assert the result; the `ObjectBoundary*` family also
// supports the flattened-vs-`JObject` JNI input micro-benchmark.
//
// Everything here is re-exported at the crate root (`pub use ext::*`), so a
// single `source_module = perftest_flat` reaches both the perf surface and this
// one. The items extend the same Storage/Payload domain with light
// "introspection / analytics" helpers:
//
// * [`Priority`] — a `#[repr(i32)]` enum (→ Kotlin `enum class`).
// * [`Stamp`] — a small `Copy` value crossing as its scalar fields (→ Kotlin
//   `data class`); `Vec<Stamp>` surfaces as `List<Stamp>`.
// * [`StorageError`] — the `E` of a fallible `Result` (→ the `onError` channel).
// * [`Summary`] — an opaque handle whose fields decompose at the boundary
//   (→ flatten-input / flatten-output).
// * [`Millis`] — a newtype crossing as a plain `Long` via a custom
//   input/output wrapper.
// * [`Duration`] — the standard-library semantic type crossing as bounded
//   milliseconds, with `Option<Duration>` using an invalid representation as
//   an allocation-free niche.

// Disabled: unsupported by the current binding subset
// /// Marked, so `Duration` is a name the flat API declares rather than one it
// /// merely mentions.
// #[prebindgen]
// pub type Duration = std::time::Duration;

// The handle types this module's flat API exports.
//
// Definitions live here and the flat API exports marked aliases to them, so
// each has a **name** every signature can resolve against without declaring
// its fields a boundary surface. See `lib.rs`'s `handles` for the same shape.
mod handles {
    use super::Stamp;
    // Disabled: unsupported by the current binding subset
    // use super::{Lookup, Reading, Stamp, Storage};

    #[derive(Clone)]
    pub struct Summary {
        pub(super) count: i64,
        pub(super) total: f64,
    }

    #[derive(Default)]
    pub struct Archive {
        pub(super) latest: Option<Summary>,
        // Unsupported: sum-valued archive fields.
        // pub(super) reading: Reading,
        // pub(super) fallback: Option<Reading>,
    }

    // Disabled: unsupported by the current binding subset
    // pub struct Ledger {
    //         pub(super) filed: Option<Report>,
    //         pub(super) archived: Option<i64>,
    //     }

    #[derive(Clone)]
    pub struct Report {
        pub(super) summary: Summary,
        // Unsupported field/value:         pub(super) taken: Option<Stamp>,
        pub(super) origin: Stamp,
        // Unsupported field/value:         pub(super) outcome: Lookup,
        pub(super) label: String,
    }

    // Disabled: unsupported by the current binding subset
    // /// The `Option<sum>` value-form field (#220), kept apart from [`Report`]
    //     /// on purpose: `Report` is embedded twice in [`Ledger`], so a field there
    //     /// would add three positional slots to `report_each` and six to
    //     /// `ledger_each` and bury the shape under signature churn.
    //     pub struct Probe {
    //         pub(super) seq: i64,
    //         pub(super) outcome: Option<Lookup>,
    //     }

    // Disabled: unsupported by the current binding subset
    // /// Two bounded-`convert!` leaves, one with a niche of its own and one
    //     /// without, behind an `Option` — see [`super::Span`] (#142).
    //     pub struct Span {
    //         pub(super) required: super::Duration,
    //         pub(super) delay: Option<super::Duration>,
    //     }

    // Disabled: unsupported by the current binding subset
    // pub struct SpanHolder {
    //         pub(super) span: Option<Span>,
    //     }

    // Disabled: unsupported by the current binding subset
    // pub struct EscapeProbe {
    //         pub(super) value: i64,
    //     }

    // Disabled: unsupported by the current binding subset
    // #[derive(Debug)]
    //
    //     pub struct StorageError {
    //         pub(super) message: String,
    //     }

    // Disabled: unsupported by the current binding subset
    // pub struct StorageHandler(pub(super) Box<dyn Fn(Storage) + Send + Sync>);
}

use prebindgen_proc_macro::prebindgen;

use crate::{Payload, Storage};

// ─────────────────────────────────────────────────────────────────────────────
// Priority — a primitive-repr enum (→ Kotlin `enum class`, jint wire).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Coarse importance bucket derived from a payload's `value`. A C-like
// /// `#[repr(i32)]` enum with explicit discriminants, mapped by the binding to a
// /// Kotlin `enum class` (and a C enum).
// #[prebindgen]
// #[repr(i32)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum Priority {
//     Low = 0,
//     Normal = 1,
//     High = 2,
// }

// Disabled: unsupported by the current binding subset
// /// Classify a payload by magnitude of its `value` (enum **return**).
// #[prebindgen]
// pub fn payload_priority(p: &Payload) -> Priority {
//     let v = p.value.abs();
//     if v < 10.0 {
//         Priority::Low
//     } else if v < 100.0 {
//         Priority::High
//     } else {
//         Priority::Normal
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Numeric weight of a priority (enum **by-value parameter**).
// #[prebindgen]
// pub fn priority_weight(p: Priority) -> i32 {
//     match p {
//         Priority::Low => 1,
//         Priority::Normal => 5,
//         Priority::High => 10,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Resolve an optional priority against a fallback (`Option<enum>` parameter).
// #[prebindgen]
// pub fn priority_or(p: Option<Priority>, fallback: Priority) -> Priority {
//     p.unwrap_or(fallback)
// }

// ─────────────────────────────────────────────────────────────────────────────
// Reading — a data-carrying enum, i.e. a sum type (→ Kotlin `sealed interface`).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A sensor reading: exactly **one** of these alternatives is live, and it
// /// carries that alternative's payload. Written as plain Rust — the "exactly
// /// one of" invariant is in the type, not in a doc comment on a struct of
// /// optional fields.
// ///
// /// All four variant shapes a sum can take are here: a payload-less variant, a
// /// single-payload tuple variant, a multi-field named variant, and a tuple
// /// variant whose payloads include a declared `enum_class`. The binding maps it
// /// to a Kotlin `sealed interface` with the variants nested inside
// /// (`lang::JniGenBuilder` `sealed_class!`).
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub enum Reading {
//     /// No reading — the empty payload group; only the tag is live.
//     Missing,
//     /// An exact value (single-payload tuple variant).
//     Exact(i64),
//     /// A bounded interval (multi-field named variant).
//     Range { low: i64, high: i64 },
//     /// A described reading: a `String` beside a declared `enum_class` payload.
//     Labeled(String, Priority),
//     /// A variant whose name collides with the Kotlin **companion object** the
//     /// binding emits to hold `fromParts`. The source crate keeps the name it
//     /// wants: `Companion` is not reserved by Kotlin, it is the generator's own
//     /// default, so the generator renames *its* companion instead.
//     Companion(i64),
// }

// Disabled: unsupported by the current binding subset
// /// A data class carrying a **sum** as a field — the position where "exactly one
// /// of" composes with ordinary product data.
// ///
// /// `reading` is required and `fallback` optional, so the binding has to gate a
// /// tag *and* a present flag independently; both sit beside already-flattened
// /// siblings (`id`, `note`) so the tag-gated groups must interleave correctly
// /// with ordinary leaves rather than only working in isolation. `Reading`'s
// /// `Labeled` arm carries a `String`, which is the payload that proves an inert
// /// group's object slot is wire-defaulted to null and therefore nullable in the
// /// generated `fromParts`.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub struct Observation {
//     pub id: i64,
//     pub reading: Reading,
//     pub fallback: Option<Reading>,
//     pub note: String,
// }

// Disabled: unsupported by the current binding subset
// /// The `Reading` alternative selected by `which` (declaration order, so it is
// /// the same numbering as the generated tag).
// fn reading_for(which: i32) -> Reading {
//     match which {
//         0 => Reading::Missing,
//         1 => Reading::Exact(42),
//         2 => Reading::Range { low: 1, high: 9 },
//         3 => Reading::Labeled("warm".to_string(), Priority::High),
//         _ => Reading::Companion(5),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Build an [`Observation`] carrying the selected alternative, optionally with
// /// a `fallback` (the next alternative round-robin) — a **sum as a struct
// /// field** crossing Rust → Kotlin, required and optional in one value.
// #[prebindgen]
// pub fn observation_new(which: i32, with_fallback: bool) -> Observation {
//     Observation {
//         id: 7,
//         reading: reading_for(which),
//         fallback: with_fallback.then(|| reading_for((which + 1) % 5)),
//         note: "obs".to_string(),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// A second sum whose payload is **not leaf-shaped**: `Option<Priority>` is an
// /// enum object (or null) in the JVM slot, which the tag-gated flat form cannot
// /// express. The binding therefore lets this one cross as a whole object through
// /// its own converter rather than failing — the degradation path — which is also
// /// what exercises the `Option<enum>` property read.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub enum Marker {
//     None_,
//     Ranked(Option<Priority>),
// }

// Disabled: unsupported by the current binding subset
// /// A data class carrying the object-shaped sum.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub struct Tagged {
//     pub id: i64,
//     pub marker: Marker,
// }

// Disabled: unsupported by the current binding subset
// /// Build a [`Tagged`]: `which` 0 = `None_`, 1 = `Ranked(None)`, 2 = `Ranked(Some(High))`.
// #[prebindgen]
// pub fn tagged_new(which: i32) -> Tagged {
//     Tagged {
//         id: 3,
//         marker: match which {
//             0 => Marker::None_,
//             1 => Marker::Ranked(None),
//             _ => Marker::Ranked(Some(Priority::High)),
//         },
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The same `Option<enum>` payload with the sum in **return** position rather
// /// than as a struct field. Only this reaches `synth_sum_leaves`, which hardcodes
// /// `nullable: false` on every group leaf and lets `plan_leaf_param` widen from
// /// the inert side; a struct field takes `PlanFieldKind::Sum` and, for this
// /// payload, degrades to the whole-object crossing instead.
// ///
// /// Two nullabilities meet in one slot and must not collapse into each other:
// /// the payload's own `None`, and the slot being inert because the other variant
// /// is live. Both arrive as a JVM null, so `Ranked(null)` and `None_` are only
// /// told apart by the tag.
// #[prebindgen]
// pub fn marker_of(which: i32) -> Marker {
//     match which {
//         0 => Marker::None_,
//         1 => Marker::Ranked(None),
//         _ => Marker::Ranked(Some(Priority::High)),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Read it back — the whole-object sum decode, including the `Option<enum>`
// /// payload, crossing Kotlin → Rust.
// #[prebindgen]
// pub fn tagged_rank(t: Tagged) -> i32 {
//     match t.marker {
//         Marker::None_ => -1,
//         Marker::Ranked(None) => 0,
//         Marker::Ranked(Some(p)) => priority_weight(p),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The selected alternative as the function's **own return** — a sum in
// /// return position, where nothing but the value's own tag says which group is
// /// live. Unlike a struct field (whose slots ride the parent's `fromParts`),
// /// there is no surrounding product to carry the tag, so the decomposition
// /// itself has to.
// #[prebindgen]
// pub fn reading_of(which: i32) -> Reading {
//     reading_for(which)
// }

// Disabled: unsupported by the current binding subset
// /// `Option<sum>` return: `which < 0` yields `None`. Optionality and choice stay
// /// independent — the present layer nulls the whole result rather than becoming
// /// an extra tag value.
// #[prebindgen]
// pub fn reading_maybe(which: i32) -> Option<Reading> {
//     (which >= 0).then(|| reading_for(which))
// }

// Disabled: unsupported by the current binding subset
// /// `Vec<sum>` return: alternatives `0..n`, each folded into the foreign list
// /// element by element.
// #[prebindgen]
// pub fn reading_series(n: i32) -> Vec<Reading> {
//     (0..n).map(reading_for).collect()
// }

// Disabled: unsupported by the current binding subset
// /// A sum as a **callback argument**: alternatives `0..n` delivered in turn.
// #[prebindgen]
// pub fn reading_each(n: i32, sink: impl Fn(Reading) + Send + Sync + 'static) {
//     for i in 0..n {
//         sink(reading_for(i));
//     }
// }

// Disabled: unsupported by the current binding subset
// /// A sum whose alternatives are a **payload-less** variant and one carrying an
// /// opaque **handle** — the shape a real lookup/reply result takes, and the one
// /// that proves a tag-gated group can own a native resource: the live group
// /// hands over a fresh handle, the inert group's slot stays a null pointer that
// /// is never wrapped.
// #[prebindgen]
// #[derive(Clone)]
// pub enum Lookup {
//     /// Nothing matched — only the tag is live.
//     Absent,
//     /// What matched, as a handle the caller owns (and must close).
//     Found(Summary),
//     /// Why the lookup could not run — a `String` beside the handle group, so an
//     /// inert object slot is exercised alongside an inert primitive one.
//     Failed(String),
// }

// Disabled: unsupported by the current binding subset
// /// Build a [`Lookup`]: `count < 0` is a failure, `count == 0` is absent,
// /// anything else is found.
// #[prebindgen]
// pub fn lookup_of(count: i64, total: f64) -> Lookup {
//     match count {
//         c if c < 0 => Lookup::Failed("negative count".to_string()),
//         0 => Lookup::Absent,
//         c => Lookup::Found(summary_new(c, total)),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// A handle-carrying sum as a **callback argument** — the same `Lookup` that
// /// [`lookup_of`] returns, arriving through `impl Fn` instead. Alternatives are
// /// delivered in `count` order starting at `-1`, so `n >= 3` covers all three:
// /// `Failed` (`i = 0`), `Absent` (`i = 1`), then `Found` with an increasing
// /// count. A live group hands a native resource to the callback while the inert
// /// groups' slots stay defaulted. The proxy closes the reassembled value after
// /// `run` returns — close-unless-taken, exactly as for a handle passed directly
// /// to a callback (#218), so a body that means to outlive the call must `take()`
// /// the payload.
// #[prebindgen]
// pub fn lookup_each(n: i64, total: f64, sink: impl Fn(Lookup) + Send + Sync + 'static) {
//     for i in 0..n {
//         sink(lookup_of(i - 1, total));
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The **third** position a handle can be reached through: a `data_class` field
// /// whose type is a handle-carrying sum. `Holder` covers the plain-handle field
// /// beside it, and the point of this one is that the two behave alike — the
// /// container is `AutoCloseable` and its `close()` cascades either way, because
// /// the field's type is an implementation detail and must not decide who frees
// /// the handle (#218).
// #[prebindgen]
// pub struct Verdict {
//     pub id: i64,
//     /// One alternative carries a `Summary` handle; closing the `Verdict`
//     /// closes it, through `Lookup`'s own `close()`.
//     pub outcome: Lookup,
// }

// Disabled: unsupported by the current binding subset
// /// Build a [`Verdict`] whose outcome comes from [`lookup_of`].
// #[prebindgen]
// pub fn verdict_new(id: i64, count: i64, total: f64) -> Verdict {
//     Verdict {
//         id,
//         outcome: lookup_of(count, total),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The **fourth** position, and the last row of the same table: a `data_class`
// /// field whose type is another `data_class` that carries the handle. Nothing
// /// here is a handle and nothing here is a sum — `Dossier` only *reaches* one,
// /// two levels down, and must still close it (#218).
// ///
// /// The cascade this emits is one line, `holder.close()`, which is correct only
// /// because [`Holder`] was independently rendered `AutoCloseable` by its own
// /// pass. An emission test cannot tell: it never compiles the inner class. This
// /// one is exercised from the JVM harness, where a `Dossier` that closed
// /// nothing, or an inner class without a `close()` to call, does not build.
// #[prebindgen]
// pub struct Dossier {
//     pub note: i64,
//     /// A plain data class whose own field is the `Summary` handle.
//     pub holder: Holder,
// }

// Disabled: unsupported by the current binding subset
// /// Build a [`Dossier`] over a fresh [`Summary`] — the two-level container.
// #[prebindgen]
// pub fn dossier_new(note: i64, tag: i64, count: i64, total: f64) -> Dossier {
//     Dossier {
//         note,
//         holder: Holder {
//             tag,
//             summary: Summary { count, total },
//         },
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Which alternative an [`Observation`]'s `reading` holds, by declaration
// /// order — the sum crossing back **in** as part of a data-class parameter.
// #[prebindgen]
// pub fn observation_which(o: Observation) -> i32 {
//     match o.reading {
//         Reading::Missing => 0,
//         Reading::Exact(_) => 1,
//         Reading::Range { .. } => 2,
//         Reading::Labeled(_, _) => 3,
//         Reading::Companion(_) => 4,
//     }
// }

// ─────────────────────────────────────────────────────────────────────────────
// Stamp — a small `Copy` value type (→ Kotlin data class over its fields).
// ─────────────────────────────────────────────────────────────────────────────

// A plain `Copy` timestamp. Declared `data_class` in the binding, so it
// crosses **by value as its two scalar fields** (no heap handle, no
// `close()`), and `Vec<Stamp>` surfaces as `List<Stamp>`.
#[prebindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

// Build a [`Stamp`] (data-class **return**).
#[prebindgen]
pub fn stamp_new(secs: i64, nanos: i64) -> Stamp {
    Stamp { secs, nanos }
}

// Seconds component (data-class **accessor**, receiver = its field leaves).
#[prebindgen]
pub fn stamp_secs(s: &Stamp) -> i64 {
    s.secs
}

// A value whose equality is **array-backed** on the JVM side: a byte-array
// field beside a nested data class.
//
// Kotlin arrays compare by identity, so the `Vec<u8>` field would make two
// equal-content values compare unequal unless the binding emits content-based
// operators. This mirrors the shape that broke downstream (a `Vec<u8>` struct
// field), which nothing else here exercised. Two fields are enough: `id` is
// array-backed and the nested [`Stamp`] — which itself crosses as its scalar
// fields — is not, so both comparison branches are covered, and a third of
// either kind would only repeat an emitted form.
//
// Field ORDER is deliberate: the array-backed fields come after `stamp`, so
// the generated `hashCode` folds them as `31 * result + id.contentHashCode()`
// rather than seeding the accumulator with them. That is the shape a real
// value takes (`Timestamp(ntp64, id)`), and it is a different emitted form
// from the array-first one.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlobValue {
    pub stamp: Stamp,
    pub id: Vec<u8>,
    // A CONTAINER of arrays. `List<ByteArray>` inherits `ByteArray`'s
    // identity equality just as a bare array does, so the generated operators
    // have to dig through the container rather than stopping at the property.
    // Unsupported field/value:     pub chunks: Vec<Vec<u8>>,
}

// Build a [`BlobValue`] (its equality is asserted from Kotlin).
#[prebindgen]
pub fn blob_value_new(secs: i64, id: Vec<u8> /* , chunks: Vec<Vec<u8>> */) -> BlobValue {
    BlobValue {
        stamp: Stamp { secs, nanos: 0 },
        id,
        // Unsupported field/value:         chunks,
    }
}

// Disabled: unsupported by the current binding subset
// /// Fixed-size arrays of every JNI-primitive element type.
// ///
// /// Each crosses as the matching Kotlin primitive array — bulk-copied, nothing
// /// boxed — rather than through the `Vec<T>` -> `List<T>` path. The wider
// /// unsigned field (`raw`) pins the raw-bit-pattern rule: `[u64; N]` carries its
// /// bits in a `LongArray`, exactly as a scalar `u64` crosses as a raw `jlong`.
// ///
// /// `flags` is the one element type that is NOT a cast: a `jboolean` is a `u8`,
// /// and reinterpreting an out-of-range byte as a Rust `bool` would be undefined
// /// behavior, so the decode normalizes instead.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub struct Arrays {
//     pub bytes: [u8; 4],
//     pub shorts: [i16; 2],
//     pub ints: [i32; 3],
//     pub longs: [i64; 2],
//     pub doubles: [f64; 2],
//     pub flags: [bool; 3],
//     pub raw: [u64; 2],
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip every fixed-size array shape, both directions.
// #[prebindgen]
// pub fn arrays_echo(a: Arrays) -> Arrays {
//     a
// }

// Round-trip a [`BlobValue`] through the WHOLE-OBJECT input decoder.
//
// The binding marks this class `.jobject_input()`, so the decoder reads each
// field off the Kotlin object by JVM descriptor — including the nested
// [`Stamp`], whose slot is its own class rather than the scalar leaves it
// flattens to everywhere else. Getting that descriptor wrong is a
// `NoSuchFieldError` on the first decode.
#[prebindgen]
pub fn blob_value_echo(value: BlobValue) -> BlobValue {
    value
}

// Nanoseconds component (data-class **accessor**).
#[prebindgen]
pub fn stamp_nanos(s: &Stamp) -> i64 {
    s.nanos
}

// Disabled: unsupported by the current binding subset
// /// A monotonically increasing run of stamps (`Vec<data-class>` →
// /// `List<Stamp>`).
// #[prebindgen]
// pub fn stamp_series(count: i64) -> Vec<Stamp> {
//     (0..count.max(0))
//         .map(|i| Stamp { secs: i, nanos: 0 })
//         .collect()
// }

// ─────────────────────────────────────────────────────────────────────────────
// StorageError — the `E` of a fallible `Result` (→ the `onError` channel).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Failure value for the fallible storage constructor. Never crosses as a
// /// value: the binding peels the `Result`, renders the message through
// /// [`storage_error_message`], and delivers it to the caller's `onError`.
// #[prebindgen]
// pub type StorageError = handles::StorageError;

// Disabled: unsupported by the current binding subset
// /// Render a [`StorageError`] as its message (the error's flatten-output
// /// **accessor**, fed to `onError`).
// #[prebindgen]
// pub fn storage_error_message(e: &StorageError) -> String {
//     e.message.clone()
// }

// Disabled: unsupported by the current binding subset
// /// Build a storage seeded with a single labelled payload, **failing** on an
// /// empty label (`Result<T, E>` routing + a `&str` input).
// #[prebindgen]
// pub fn storage_try_with_label(label: &str) -> Result<Storage, StorageError> {
//     if label.is_empty() {
//         return Err(StorageError {
//             message: "label must not be empty".to_string(),
//         });
//     }
//     Ok(Storage {
//         payloads: vec![Payload {
//             id: 0,
//             seq: 0,
//             value: 0.0,
//             flag: false,
//             label: Some(Box::new(label.to_string())),
//         }],
//     })
// }

// Disabled: unsupported by the current binding subset
// /// Build a storage stamped with `s`, **failing** on a non-positive `secs` (a
// /// domain [`StorageError`]).
// ///
// /// `tag` is a fixed-size array purely so the two error channels stay separately
// /// provable: a wrong-length array fails the input DECODE first — the binding
// /// channel — while a well-formed but rejected `secs` fails in the domain
// /// channel. It is the covertest exercise for issue #45's two-caller split: one
// /// wrapper, both `onBindingError` and `onError` provable independently.
// #[prebindgen]
// pub fn storage_try_from_stamp(s: Stamp, tag: [u8; 2]) -> Result<Storage, StorageError> {
//     let _ = tag;
//     if s.secs <= 0 {
//         return Err(StorageError {
//             message: "stamp secs must be positive".to_string(),
//         });
//     }
//     Ok(Storage {
//         payloads: vec![Payload {
//             id: s.secs,
//             seq: 0,
//             value: 0.0,
//             flag: false,
//             label: None,
//         }],
//     })
// }

// ─────────────────────────────────────────────────────────────────────────────
// Summary — an opaque handle whose fields decompose at the boundary.
// ─────────────────────────────────────────────────────────────────────────────

// An aggregate over a [`Storage`]'s payloads: how many there are and the sum of
// their `value`s. An opaque handle in the binding, but its default
// flatten-output decomposes it into `(count, total)` leaves and its
// flatten-input rebuilds it from the same leaves (via [`summary_new`]).
// `Clone` because [`archive_latest`] returns it *borrowed* (`Option<&Summary>`)
// and the JVM binding's only sound lowering of a borrowed handle is a clone
// into a fresh owned handle.
#[prebindgen]
pub type Summary = handles::Summary;

// Construct a [`Summary`] from its parts (declared a **constructor** /
// companion factory, and the build-from **variant** of the flatten-input).
#[prebindgen]
pub fn summary_new(count: i64, total: f64) -> Summary {
    Summary { count, total }
}

// Number of payloads (flatten-output **field** / **accessor**).
#[prebindgen]
pub fn summary_count(s: &Summary) -> i64 {
    s.count
}

// Sum of payload values (flatten-output **field** / **accessor**).
#[prebindgen]
pub fn summary_total(s: &Summary) -> f64 {
    s.total
}

// Disabled: unsupported by the current binding subset
// /// Total scaled by a factor (an instance **method**: `&Self` receiver + arg).
// #[prebindgen]
// pub fn summary_scaled(s: &Summary, factor: f64) -> f64 {
//     s.total * factor
// }

// Disabled: unsupported by the current binding subset
// /// A series of `count` summaries starting at `start`: element `i` is
// /// `(start + i, (start + i) * 10.0)`. A **record-built iterable fold** at the
// /// boundary: the caller supplies the accumulator and a per-element `fold`
// /// lambda receiving the decomposed `(count, total)` leaves.
// #[prebindgen]
// pub fn summary_series(count: i64, start: i64) -> Vec<Summary> {
//     (0..count)
//         .map(|i| summary_new(start + i, ((start + i) * 10) as f64))
//         .collect()
// }

// Disabled: unsupported by the current binding subset
// /// Like [`summary_series`] but `None` when `count < 0` — the record-built
// /// `Optional(Iterable)` shape (#105): `None` skips the fold and the JVM
// /// wrapper returns null; `Some(vec![])` returns the untouched accumulator.
// #[prebindgen]
// pub fn summary_series_opt(count: i64, start: i64) -> Option<Vec<Summary>> {
//     (count >= 0).then(|| summary_series(count, start))
// }

/// Summarize a storage (returns a `Summary`; the binding's **default
/// flatten-output** turns it into `(count, total)` leaves).
#[prebindgen]
pub fn storage_summary(s: &Storage) -> Summary {
    Summary {
        count: s.payloads.len() as i64,
        total: s.payloads.iter().map(|p| p.value).sum(),
    }
}

// Disabled: unsupported by the current binding subset
// /// Whether `expected` matches the storage's live summary (takes a `Summary`
// /// **parameter**; the binding's **default flatten-input** rebuilds it from
// /// `(count, total)` or accepts a handle).
// #[prebindgen]
// pub fn storage_matches_summary(s: &Storage, expected: Summary) -> bool {
//     let live = storage_summary(s);
//     live.count == expected.count && (live.total - expected.total).abs() < f64::EPSILON
// }

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

// Read a summary's total through a raw handle (per-fn **flatten-input-suppress**
// on the `Summary` parameter).
#[prebindgen]
pub fn summary_total_raw(s: Summary) -> f64 {
    s.total
}

// Disabled: unsupported by the current binding subset
// /// Like [`storage_summary`] but the binding decomposes it with a **custom**
// /// field set that also keeps the handle (per-fn **flatten-output-with**).
// #[prebindgen]
// pub fn storage_summary_full(s: &Storage) -> Summary {
//     storage_summary(s)
// }

// Disabled: unsupported by the current binding subset
// /// Like [`storage_summary`] but the binding's per-fn field set carries a
// /// **binding-local conditional field** (`field!("handle").with(ty!, path!)`):
// /// the handle leaf is delivered only when the binding-side predicate says
// /// re-using the value is worth it (the zenoh conditional-Encoding idiom).
// #[prebindgen]
// pub fn storage_summary_probe(s: &Storage) -> Summary {
//     storage_summary(s)
// }

// Disabled: unsupported by the current binding subset
// /// Set the storage's "expected" summary, accepting a `Summary` built via an
// /// explicit per-fn **flatten-input-with** variant list. Returns whether it
// /// matched the live summary before being consumed.
// #[prebindgen]
// pub fn storage_expect_summary(s: &mut Storage, expected: Summary) -> bool {
//     let live = storage_summary(s);
//     live.count == expected.count && (live.total - expected.total).abs() < f64::EPSILON
// }

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

// Build a storage holding a single payload (a **constructor** / companion
// factory on `Storage`).
#[prebindgen]
pub fn storage_with_payload(payload: Payload) -> Storage {
    Storage {
        payloads: vec![payload],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Millis — a newtype crossing as a plain `Long` via a custom wrapper.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A duration in milliseconds. The binding registers a custom
// /// `input_wrapper`/`output_wrapper` mapping it to a plain `Long` (no generated
// /// class), so it never appears as a Kotlin type of its own. It is intentionally
// /// **not** `#[prebindgen]`: the wrapper fully owns its boundary conversion, and
// /// marking it would make the Kotlin emitter try to render this tuple struct as a
// /// data class.
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// #[prebindgen]
// pub struct Millis(pub u64);

// Disabled: unsupported by the current binding subset
// /// Sum two durations (exercises the custom wrapper on both a **parameter** and
// /// the **return**).
// #[prebindgen]
// pub fn millis_add(a: Millis, b: Millis) -> Millis {
//     Millis(a.0 + b.0)
// }

// ─────────────────────────────────────────────────────────────────────────────
// Duration — std semantic type crossing as bounded u64 milliseconds.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Largest duration accepted by the covertest binding: one day in
// /// milliseconds. The binding's representation-domain declaration reserves all
// /// larger `u64` values, allowing `Option<Duration>` to use one as `None` while
// /// keeping the JNI carrier a primitive `jlong`.
// pub const DURATION_MAX_MILLIS: u64 = 86_400_000;

// Disabled: unsupported by the current binding subset
// /// Round-trip an optional standard-library duration. The source API remains
// /// semantic (`Option<Duration>`); only the binding declares its millisecond
// /// representation and range.
// #[prebindgen]
// pub fn duration_optional(value: Option<Duration>) -> Option<Duration> {
//     value
// }

// Disabled: unsupported by the current binding subset
// /// A transparent wrapper over a **`convert!`-declared** type, both directions.
// ///
// /// `Duration` reaches its Rust value through a staged chain
// /// (`jlong -> u64 -> Duration`), and the transparent bridge used to call the
// /// inner converter's function directly and leave `pre_stages` empty — so the
// /// stages were skipped and the rebuild put `Box::new` around a `u64`. Not a
// /// silent wrong value: `E0308` in the generated crate (#309).
// #[prebindgen]
// pub fn boxed_duration_echo(value: Box<Duration>) -> Box<Duration> {
//     value
// }

// Disabled: unsupported by the current binding subset
// /// Deliberately violate the binding's declared output domain so the Kotlin
// /// covertest can verify outbound validation and error routing.
// #[prebindgen]
// pub fn duration_out_of_range() -> Option<Duration> {
//     Some(Duration::from_millis(DURATION_MAX_MILLIS + 1))
// }

// Disabled: unsupported by the current binding subset
// /// Data-class composition probe for the bounded duration representation.
// /// The coverage binding deliberately marks this class `.jobject_input()` so
// /// its echo executes both the whole-object input decoder and the `fromParts`
// /// output encoder; the nullable duration itself still uses the raw `jlong`
// /// niche whenever it crosses a generated JNI call boundary.
// ///
// /// The two fields are the two shapes a converted leaf takes, and they exercise
// /// DIFFERENT emitter paths: `delay` goes through the `Option<_>` wrapper, which
// /// composes its inner conversion chain itself, while `required` is a bare leaf
// /// the data-class encoder/decoder has to compose for.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct DurationBoundary {
//     pub required: Duration,
//     pub delay: Option<Duration>,
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip [`DurationBoundary`] through the explicit object-input bridge.
// #[prebindgen]
// pub fn duration_boundary_echo(value: &DurationBoundary) -> DurationBoundary {
//     value.clone()
// }

// Disabled: unsupported by the current binding subset
// /// The same two converted leaves under an **optional ancestor** (#142) — the
// /// combination `DurationBoundary` alone cannot reach, because its own fields
// /// are never absent as a pair.
// ///
// /// A conditional value form (`Option<&Span>`) makes every leaf below it
// /// nullable, which crosses the two facts that decide a wrap: whether the leaf
// /// carries a niche of its OWN (`delay` does, `required` does not) and whether an
// /// ancestor can be absent (both, here). Only the first grants a sentinel — the
// /// ancestor's absence is carried by `?.`, and testing `required` against `-1`
// /// would ask about a value its own encoder can never produce.
// #[prebindgen]
// pub type Span = handles::Span;

// Disabled: unsupported by the current binding subset
// /// [`Span`]'s value form: one bounded leaf with a niche, one without.
// #[prebindgen]
// pub struct SpanStruct {
//     pub required: Duration,
//     pub delay: Option<Duration>,
// }

// Disabled: unsupported by the current binding subset
// /// The accessor `expand_return!(Span).fields(fields!(..))` names.
// #[prebindgen]
// pub fn span_to_struct(s: &Span) -> SpanStruct {
//     SpanStruct {
//         required: s.required,
//         delay: s.delay,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The holder whose span is reached **optionally** — the conditional hoist.
// #[prebindgen]
// pub type SpanHolder = handles::SpanHolder;

// Disabled: unsupported by the current binding subset
// /// `None` when `seq` is negative, so the absent case is reachable from the
// /// same argument that drives the present ones.
// #[prebindgen]
// pub fn span_holder_new(seq: i64, required_ms: u64, delay_ms: i64) -> SpanHolder {
//     SpanHolder {
//         span: (seq >= 0).then(|| Span {
//             required: Duration::from_millis(required_ms),
//             delay: (delay_ms >= 0).then(|| Duration::from_millis(delay_ms as u64)),
//         }),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The borrowed optional accessor that makes `Span`'s leaves nullable.
// #[prebindgen]
// pub fn span_holder_span(h: &SpanHolder) -> Option<&Span> {
//     h.span.as_ref()
// }

// Disabled: unsupported by the current binding subset
// /// Deliver a converted value through the generated typed/raw callback twin —
// /// the converted analogue of [`unsigned_emit`].
// ///
// /// A callback argument that crosses WHOLE (no deconstructor, so no leaf plan)
// /// is its own encoder path, independent of the data-class and sum emitters, so
// /// a converted type has to reach its representation here too.
// #[prebindgen]
// pub fn duration_emit(value: Duration, f: impl Fn(Duration) + Send + Sync + 'static) {
//     f(value)
// }

// Disabled: unsupported by the current binding subset
// /// How long something is held: for an explicit period, or indefinitely.
// ///
// /// A **sum whose payload is a converted type**. `Duration` crosses through the
// /// binding's `convert!` declaration, so this payload's boundary conversion is
// /// two steps (`Duration -> u64 -> jlong`, and back) rather than the single
// /// wire converter every other sum payload here uses — the position where an
// /// emitter that reads only the wire-facing converter builds code that hands
// /// the semantic value where the representation is expected.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum Hold {
//     /// Held with no end — the payload-less group.
//     Indefinite,
//     /// Held for this long.
//     For(Duration),
// }

// Disabled: unsupported by the current binding subset
// /// A retention policy: a required converted-payload sum beside an optional one.
// ///
// /// The data-class position for [`Hold`], so the converted payload is exercised
// /// both as a top-level sum and as a tag-gated group inside a `fromParts`
// /// bridge — the two encoders are separate code paths.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct HoldPolicy {
//     pub hold: Hold,
//     pub grace: Option<Hold>,
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip a converted-payload sum, whole.
// #[prebindgen]
// pub fn hold_echo(h: Hold) -> Hold {
//     h
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip a data class carrying converted-payload sums.
// #[prebindgen]
// pub fn hold_policy_echo(p: HoldPolicy) -> HoldPolicy {
//     p
// }

// ─────────────────────────────────────────────────────────────────────────────
// convert! source-kind fixtures — one type per conversion source. Like
// `Millis`, none of these types is `#[prebindgen]`-marked: each crosses the
// boundary only through its declared canonical conversion.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A temperature. Crosses via its `From`/`Into` impls
// /// (`convert!(Celsius).input_from(ty!(i32)).output_into(ty!(i32))`).
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// #[prebindgen]
// pub struct Celsius(pub i32);

// Disabled: unsupported by the current binding subset
// impl From<i32> for Celsius {
//     fn from(v: i32) -> Self {
//         Celsius(v)
//     }
// }
// Disabled: unsupported by the current binding subset
// impl From<Celsius> for i32 {
//     fn from(c: Celsius) -> Self {
//         c.0
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Double a temperature (exercises the `Into`-based conversion on a
// /// parameter and the return).
// #[prebindgen]
// pub fn celsius_double(c: Celsius) -> Celsius {
//     Celsius(c.0 * 2)
// }

// Disabled: unsupported by the current binding subset
// /// A percentage, range-invariant 0..=100. Crosses via a fallible
// /// `TryFrom<i32>` on input (out-of-range i32 from the JVM → the caller's
// /// error handler) and an infallible `Into<i32>` on output.
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// #[prebindgen]
// pub struct Percent(pub u8);

// Disabled: unsupported by the current binding subset
// impl TryFrom<i32> for Percent {
//     type Error = String;
//     fn try_from(v: i32) -> Result<Self, Self::Error> {
//         if (0..=100).contains(&v) {
//             Ok(Percent(v as u8))
//         } else {
//             Err(format!("percent out of range: {v} (expected 0..=100)"))
//         }
//     }
// }
// Disabled: unsupported by the current binding subset
// impl From<Percent> for i32 {
//     fn from(p: Percent) -> Self {
//         p.0 as i32
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Scale a percentage, saturating at 100 (exercises the `TryInto`-based
// /// input conversion — including its error path — and the `Into` output).
// #[prebindgen]
// pub fn percent_scale(p: Percent, factor: i32) -> Percent {
//     Percent(((p.0 as i32) * factor).clamp(0, 100) as u8)
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip an optional percentage. The covertest binding uses this to
// /// compose `Option` with the fallible `TryFrom<i32>` input conversion and its
// /// fallible output conversion.
// #[prebindgen]
// pub fn percent_optional(p: Option<Percent>) -> Option<Percent> {
//     p
// }

// Disabled: unsupported by the current binding subset
// /// Deliberately construct a value outside `Percent`'s semantic invariant so
// /// the covertest can verify a fallible output stage nested under `Option`.
// #[prebindgen]
// pub fn percent_invalid_output() -> Option<Percent> {
//     Some(Percent(101))
// }

// Disabled: unsupported by the current binding subset
// /// A text label. Crosses via plain conversion fns declared **in the binding
// /// crate** (`convert!(Label).input_with(ty!(String), path!(crate::label_in))…`)
// /// — no `#[prebindgen]` marking anywhere in the conversion.
// #[derive(Clone, Debug, PartialEq, Eq)]
// #[prebindgen]
// pub struct Label(pub String);

// Disabled: unsupported by the current binding subset
// /// Reverse a label's characters (exercises the binding-local conversion on
// /// a parameter and the return).
// #[prebindgen]
// pub fn label_reverse(l: Label) -> Label {
//     Label(l.0.chars().rev().collect())
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip a collection of labels — a `Vec` whose ELEMENT is a converted
// /// type.
// ///
// /// `Duration` cannot take this path (a `Vec` needs a JObject-shaped element
// /// wire, and its representation is a primitive `jlong`, so `Vec<Duration>` is
// /// refused at resolve time), but `Label` lowers to `String` and therefore does.
// /// The `Vec` converters build their element conversion inline, in both
// /// directions, so each has to compose the element's chain rather than call its
// /// wire-facing converter alone.
// #[prebindgen]
// pub fn label_series_echo(labels: Vec<Label>) -> Vec<Label> {
//     labels
// }

// ─────────────────────────────────────────────────────────────────────────────
// Option<scalar> — a nullable primitive return.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Length of a payload's label, or `None` when it is unlabeled. Exercises an
// /// `Option<i64>` (nullable primitive) return, distinct from the `Option<handle>`
// /// / `Option<data-class>` shapes elsewhere.
// #[prebindgen]
// pub fn payload_label_len(p: &Payload) -> Option<i64> {
//     p.label.as_ref().map(|s| s.len() as i64)
// }

// ─────────────────────────────────────────────────────────────────────────────
// Annotated — a data class with a NESTED data-class field and Option<scalar> /
// Option<enum> fields.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A [`Payload`] with optional delivery metadata. As a `data_class` it
// /// exercises the shapes flat `Payload` cannot: a **nested** data-class field
// /// (`payload`, recursive `fromParts` on output / recursive leaf decode on
// /// input) and `Option<primitive>` / `Option<enum>` **fields** (each crossing
// /// as a decoupled `(present, value)` leaf pair).
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq)]
// pub struct Annotated {
//     pub payload: Payload,
//     pub alternate: Option<Payload>,
//     pub ttl: Option<i64>,
//     pub priority: Option<Priority>,
// }

// Disabled: unsupported by the current binding subset
// /// Assemble an [`Annotated`] (nested data-class **output** + bare
// /// `Option<scalar>` / `Option<enum>` inputs).
// #[prebindgen]
// pub fn annotated_new(payload: Payload, ttl: Option<i64>, priority: Option<Priority>) -> Annotated {
//     Annotated {
//         payload,
//         alternate: None,
//         ttl,
//         priority,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The optional nested payload's value. Its `Option<data_class>` input leaves
// /// are guarded by one presence bit and recursively reconstructed only when
// /// present.
// #[prebindgen]
// pub fn annotated_alternate_value(a: &Annotated) -> Option<f64> {
//     a.alternate.as_ref().map(|payload| payload.value)
// }

// Disabled: unsupported by the current binding subset
// /// The metadata TTL (`Option<prim>` field read back through a data-class
// /// **input**).
// #[prebindgen]
// pub fn annotated_ttl(a: &Annotated) -> Option<i64> {
//     a.ttl
// }

// Disabled: unsupported by the current binding subset
// /// The metadata priority (`Option<enum>` **return**).
// #[prebindgen]
// pub fn annotated_priority(a: &Annotated) -> Option<Priority> {
//     a.priority
// }

// Disabled: unsupported by the current binding subset
// /// The nested payload's `value` (proves the nested field survived the
// /// input decode).
// #[prebindgen]
// pub fn annotated_payload_value(a: &Annotated) -> f64 {
//     a.payload.value
// }

// ─────────────────────────────────────────────────────────────────────────────
// CacheConfig / RepliesConfig — a non-null enum field reached through an outer
// `Option<data_class>` (the nullable-context input path, issue #144).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Inner, **non-optional** delivery config carrying a **non-null enum field**
// /// (`priority`). Nested inside [`CacheConfig`], which crosses as
// /// `Option<CacheConfig>`, so the outer optional's `nullable_context`
// /// propagates down to this non-optional struct — the exact shape that made the
// /// JNI input builder emit a dead double-Elvis default (`?.value ?: 0 ?: 0`)
// /// for a non-null enum field (#144).
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct RepliesConfig {
//     pub priority: Priority,
//     pub max_samples: i64,
// }

// Disabled: unsupported by the current binding subset
// /// Outer cache config crossed as `Option<CacheConfig>`. Its optional-ness
// /// propagates into the non-optional nested [`RepliesConfig`], whose non-null
// /// `priority` enum field must decode with exactly **one** Elvis default (#144).
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct CacheConfig {
//     pub replies: RepliesConfig,
//     pub ttl: i64,
// }

// Disabled: unsupported by the current binding subset
// /// The cache's replies-priority weight plus its `ttl`, or `-1` when the cache
// /// is absent (`Option<CacheConfig>` **input** — the #144 reproduction: a
// /// non-null enum field reached through the outer optional data class).
// #[prebindgen]
// pub fn cache_config_weight(cache: Option<CacheConfig>) -> i32 {
//     match cache {
//         Some(c) => priority_weight(c.replies.priority) + c.ttl as i32,
//         None => -1,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// One `i64` leaf in the deliberately wide [`ObjectBoundary`] tree.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ObjectBoundaryLeaf {
//     pub value: i64,
// }

// Disabled: unsupported by the current binding subset
// macro_rules! object_boundary_level {
//     ($name:ident, $child:ty) => {
//         #[prebindgen]
//         #[derive(Clone, Debug, PartialEq, Eq)]
//         pub struct $name {
//             pub left: $child,
//             pub right: $child,
//         }
//     };
// }

// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary2, ObjectBoundaryLeaf);
// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary4, ObjectBoundary2);
// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary8, ObjectBoundary4);
// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary16, ObjectBoundary8);
// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary32, ObjectBoundary16);
// Disabled: unsupported by the current binding subset
// object_boundary_level!(ObjectBoundary64, ObjectBoundary32);

// Disabled: unsupported by the current binding subset
// /// Structural twin of [`ObjectBoundary64`] used to benchmark an explicit
// /// whole-`JObject` input against recursive 64-leaf flattening.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ObjectBoundary64Object {
//     pub left: ObjectBoundary32,
//     pub right: ObjectBoundary32,
// }

// Disabled: unsupported by the current binding subset
// /// The right half of [`ObjectBoundary`]: 32 + 16 + 8 + 4 + 2 + 1 leaves.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ObjectBoundary63 {
//     pub leaves32: ObjectBoundary32,
//     pub leaves16: ObjectBoundary16,
//     pub leaves8: ObjectBoundary8,
//     pub leaves4: ObjectBoundary4,
//     pub leaves2: ObjectBoundary2,
//     pub leaf: ObjectBoundaryLeaf,
// }

// Disabled: unsupported by the current binding subset
// /// Deliberate object-boundary fixture for `data_class!(T).jobject_input()`.
// ///
// /// Its [`ObjectBoundary64`] and [`ObjectBoundary63`] children recursively
// /// contain 127 `i64` leaves. The generated Kotlin constructor/fromParts bridge
// /// remains legal at 254 JVM slots, but flattening a native input parameter
// /// would require 256: 254 for the leaves plus the `JNINative` receiver and
// /// binding-error sink. Because the JVM limit is 255, this otherwise-valid data
// /// class must cross Kotlin→Rust as one `JObject`.
// #[prebindgen]
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct ObjectBoundary {
//     pub left: ObjectBoundary64,
//     pub right: ObjectBoundary63,
// }

// Disabled: unsupported by the current binding subset
// trait ObjectBoundarySum {
//     fn boundary_sum(&self) -> i64;
// }

// Disabled: unsupported by the current binding subset
// impl ObjectBoundarySum for ObjectBoundaryLeaf {
//     fn boundary_sum(&self) -> i64 {
//         self.value
//     }
// }

// Disabled: unsupported by the current binding subset
// macro_rules! impl_object_boundary_sum {
//     ($($name:ty),+ $(,)?) => {
//         $(
//             impl ObjectBoundarySum for $name {
//                 fn boundary_sum(&self) -> i64 {
//                     self.left.boundary_sum() + self.right.boundary_sum()
//                 }
//             }
//         )+
//     };
// }

// Disabled: unsupported by the current binding subset
// impl_object_boundary_sum!(
//     ObjectBoundary2,
//     ObjectBoundary4,
//     ObjectBoundary8,
//     ObjectBoundary16,
//     ObjectBoundary32,
//     ObjectBoundary64,
//     ObjectBoundary64Object,
// );

// Disabled: unsupported by the current binding subset
// impl ObjectBoundarySum for ObjectBoundary63 {
//     fn boundary_sum(&self) -> i64 {
//         self.leaves32.boundary_sum()
//             + self.leaves16.boundary_sum()
//             + self.leaves8.boundary_sum()
//             + self.leaves4.boundary_sum()
//             + self.leaves2.boundary_sum()
//             + self.leaf.boundary_sum()
//     }
// }

// Disabled: unsupported by the current binding subset
// impl ObjectBoundarySum for ObjectBoundary {
//     fn boundary_sum(&self) -> i64 {
//         self.left.boundary_sum() + self.right.boundary_sum()
//     }
// }

// Disabled: unsupported by the current binding subset
// #[prebindgen]
// pub fn object_boundary_value(value: &ObjectBoundary) -> i64 {
//     value.boundary_sum()
// }

// Disabled: unsupported by the current binding subset
// /// Sum the 64 scalar leaves after recursive JNI parameter flattening.
// #[prebindgen]
// pub fn large_flat_input_sum(value: &ObjectBoundary64) -> i64 {
//     value.boundary_sum()
// }

// Disabled: unsupported by the current binding subset
// /// Sum the same 64-leaf shape after decoding one whole `JObject` input.
// #[prebindgen]
// pub fn large_object_input_sum(value: &ObjectBoundary64Object) -> i64 {
//     value.boundary_sum()
// }

// ─────────────────────────────────────────────────────────────────────────────
// Fixed-width unsigned integers — widened Kotlin scalars + ULong projection.
// ─────────────────────────────────────────────────────────────────────────────

// Every fixed-width Rust unsigned scalar in one generated Kotlin data class.
// The first three fields widen losslessly; `long`/`maybe_long` surface as
// `ULong`/`ULong?` over raw JNI `Long` bit patterns.
#[prebindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unsigned {
    pub byte: u8,
    pub short: u16,
    pub int: u32,
    pub long: u64,
    pub maybe_long: Option<u64>,
}

// Round-trip direct unsigned parameters through an unsigned data-class
// return, covering both checked widening and the `ULong` projection.
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

/// Read the optional `u64` field through a borrowed opaque handle in C.
/// JNI binding remains disabled until borrowed data-class inputs are supported.
#[prebindgen]
pub fn unsigned_data_maybe(value: &Unsigned) -> Option<u64> {
    value.maybe_long
}

// Disabled: unsupported by the current binding subset
// /// Deliver a `u64` through the generated typed/raw callback twin.
// #[prebindgen]
// pub fn unsigned_emit(value: u64, f: impl Fn(u64) + Send + Sync + 'static) {
//     f(value)
// }

// Disabled: unsupported by the current binding subset
// /// Output collection fold whose raw `jlong` leaves become `ULong` values on
// /// the Kotlin side.
// #[prebindgen]
// pub fn unsigned_series() -> Vec<u64> {
//     vec![0, u64::MAX]
// }

// ─────────────────────────────────────────────────────────────────────────────
// Vec<opaque-handle> outputs — the Kotlin-side handle fold.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// fn synthetic_storage(shard: i64, each: i64) -> Storage {
//     Storage {
//         payloads: (0..each.max(0))
//             .map(|k| Payload {
//                 id: shard * 1000 + k,
//                 seq: k as i32,
//                 value: k as f64,
//                 flag: false,
//                 label: None,
//             })
//             .collect(),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Build `count` independent storages of `each` payloads (a
// /// `Vec<opaque-handle>` **return** — each element crosses as a raw pointer the
// /// Kotlin folder wraps into a typed `Storage` handle).
// #[prebindgen]
// pub fn storage_shards(count: i64, each: i64) -> Vec<Storage> {
//     (0..count.max(0))
//         .map(|i| synthetic_storage(i, each))
//         .collect()
// }

// Disabled: unsupported by the current binding subset
// /// Like [`storage_shards`] but `None` when `count == 0`
// /// (`Option<Vec<opaque-handle>>` — the fold under the null niche).
// #[prebindgen]
// pub fn storage_shards_opt(count: i64, each: i64) -> Option<Vec<Storage>> {
//     if count <= 0 {
//         None
//     } else {
//         Some(storage_shards(count, each))
//     }
// }

// ─────────────────────────────────────────────────────────────────────────────
// StorageHandler — a callback receiving an OWNED opaque handle.
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A prepared callback receiving an **owned [`Storage`] handle** (`Fn(Storage)`,
// /// by value). Unlike [`PayloadHandler`](crate::PayloadHandler) (whose arg is a
// /// flattened data class),
// /// the handle crosses as a raw pointer and the generated Kotlin proxy wraps it
// /// into a typed `Storage` and `close()`s it after `run` (close-unless-taken).
// #[prebindgen]
// pub type StorageHandler = handles::StorageHandler;

// Disabled: unsupported by the current binding subset
// /// Wrap a `Fn(Storage)` closure into a reusable [`StorageHandler`].
// #[prebindgen]
// pub fn storage_handler_new(f: impl Fn(Storage) + Send + Sync + 'static) -> StorageHandler {
//     handles::StorageHandler(Box::new(f))
// }

// Disabled: unsupported by the current binding subset
// /// Build a synthetic storage of `n` payloads and hand **ownership** of it to
// /// the handler's callback.
// #[prebindgen]
// pub fn storage_emit(n: i64, h: &StorageHandler) {
//     (h.0)(synthetic_storage(0, n));
// }

// ─────────────────────────────────────────────────────────────────────────────
// Archive — a borrowed-opaque output (`Option<&Summary>` → cloned owned handle).
// ─────────────────────────────────────────────────────────────────────────────

// Holds the most recently stored [`Summary`]. Its accessor returns the summary
// **borrowed** — the shape zenoh-flat's `z_*` accessors use for the C tier's
// zero-copy borrows — which the JVM binding lowers by **cloning** into a fresh
// owned handle (the JVM keeps its handle past the call).
#[prebindgen]
pub type Archive = handles::Archive;

// Disabled: unsupported by the current binding subset
// impl Default for Archive {
//     fn default() -> Self {
//         Self {
//             latest: None,
//             reading: Reading::Missing,
//             fallback: None,
//         }
//     }
// }

// Create an empty archive.
#[prebindgen]
pub fn archive_new() -> Archive {
    Archive::default()
}

/// Store a summary, consuming it (owned-handle input).
#[prebindgen]
pub fn archive_store(a: &mut Archive, s: Summary) {
    a.latest = Some(s);
}

// Disabled: unsupported by the current binding subset
// /// Store the `which` alternative as the archive's own reading, and the same one
// /// as its optional fallback. A **negative** `which` clears the fallback and
// /// resets the reading to the first alternative, so the two borrow shapes can be
// /// exercised independently: `&Reading` always has a value, `Option<&Reading>`
// /// does not.
// #[prebindgen]
// pub fn archive_set_reading(a: &mut Archive, which: i32) {
//     a.reading = reading_for(which.max(0));
//     a.fallback = (which >= 0).then(|| reading_for(which));
// }

// Disabled: unsupported by the current binding subset
// /// A sum returned **borrowed** (`&Reading`). The value stays owned by the
// /// archive; the binding decomposes it in place — the encoder matches through
// /// the reference instead of consuming — and the caller gets an ordinary
// /// Kotlin value with no borrow to track.
// #[prebindgen]
// pub fn archive_reading(a: &Archive) -> &Reading {
//     &a.reading
// }

// Disabled: unsupported by the current binding subset
// /// The optional layer over the same borrow (`Option<&Reading>`): `None` nulls
// /// the whole result, exactly as for an owned `Option<Reading>`.
// #[prebindgen]
// pub fn archive_reading_maybe(a: &Archive) -> Option<&Reading> {
//     a.fallback.as_ref()
// }

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

// Disabled: unsupported by the current binding subset
// /// All present labels, in storage order (`Vec<String>` **return** — the
// /// single-leaf string fold).
// #[prebindgen]
// pub fn storage_labels(s: &Storage) -> Vec<String> {
//     s.payloads
//         .iter()
//         .filter_map(|p| p.label.as_deref().cloned())
//         .collect()
// }

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

// Disabled: unsupported by the current binding subset
// /// The storage capacity limit advertised to bindings (a primitive const).
// #[prebindgen]
// pub const COVER_MAGIC: i64 = 0xC0FFEE;

// Disabled: unsupported by the current binding subset
// /// The coverage surface's tag string (a string const).
// #[prebindgen]
// Kept as a Rust implementation constant; constant bindings are unsupported.
pub const COVER_TAG: &str = "covertest";

// The tag with a runtime-computed suffix — a constant value no Rust `const`
// can express (built through `format!`). Exercises
// `PackageDecl::constant_fun`: a nullary fn surfaced as a
// lazily-initialized Kotlin top-level `val`.
#[prebindgen]
pub fn cover_tag_runtime() -> String {
    format!("{COVER_TAG}-runtime")
}

// ─────────────────────────────────────────────────────────────────────────────
// EscapeProbe — JNI native-symbol escaping probe (#86).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// A tiny opaque handle whose covertest declaration puts underscores in every
// /// symbol component (#86): it lives in the underscored `esc_pkg` subpackage
// /// under the underscored Kotlin name `Esc_Probe`, and its accessor's harness
// /// extern is mangled to an underscored method name — so its `freePtr`
// /// destructor and accessor symbols only resolve at runtime if the generator
// /// applies the JNI spec's `_1` escaping.
// #[prebindgen]
// pub type EscapeProbe = handles::EscapeProbe;

// Disabled: unsupported by the current binding subset
// /// Construct an [`EscapeProbe`] (its covertest constructor).
// #[prebindgen]
// pub fn escape_probe_new(value: i64) -> EscapeProbe {
//     EscapeProbe { value }
// }

// Disabled: unsupported by the current binding subset
// /// Read the probe's value (mangled to an underscored harness extern in
// /// covertest, #86).
// #[prebindgen]
// pub fn escape_probe_value(p: &EscapeProbe) -> i64 {
//     p.value
// }

// ── Value form: a type's own accessors gathered into one struct (#213) ──────

// An opaque handle whose output boundary is declared from its **value form**
// ([`report_to_struct`]) instead of a restated field list — the
// `expand_return!(Report).fields(fields!(report_to_struct))` exercise.
//
// Its fields are chosen so each one lands on a different rule of the
// expansion, and so the derived boundary is the same one a hand-written
// `.field()` list would have produced:
//
// | field | rule |
// |---|---|
// | `summary` | its type has its own `expand_return!` ⇒ spliced into `(count, total)`, NOT handed over as a handle |
// | `taken` | `Option<data class>` ⇒ stays ONE leaf, its converter builds the object |
// | `origin` | a non-optional declared `data class` ⇒ INLINES into its own fields |
// | `outcome` | a `sealed_class!` ⇒ its selector plus one group per alternative, with a handle payload |
// | `label` | a plain leaf |
//
// `Clone` because a CONSUMING value form reached through a BORROW has nothing
// to take and must clone first — see [`ledger_filed`]. Only the borrowed
// position needs it; an owned payload is moved. The derive lives on the
// definition in `handles`, since this is only its name in the flat API.
#[prebindgen]
pub type Report = handles::Report;

// The value form of [`Report`]: its fields as data, handles staying handles.
#[prebindgen]
pub struct ReportStruct {
    // Decomposed by `Summary`'s own boundary decl, not delivered as a handle.
    pub summary: Summary,
    // Absent when the report was never stamped.
    // Unsupported field/value:     pub taken: Option<Stamp>,
    // Always present, so it inlines into `origin_secs` / `origin_nanos`.
    pub origin: Stamp,
    // A tag-gated group set, one alternative live, carrying a handle.
    // Unsupported field/value:     pub outcome: Lookup,
    // A plain string leaf beside the rest.
    pub label: String,
}

// Build a [`Report`]. `count < 0` makes the outcome a failure, `0` absent.
#[prebindgen]
pub fn report_new(count: i64, total: f64, /* taken: bool, */ label: String) -> Report {
    Report {
        summary: summary_new(count.max(0), total),
        // Unsupported field/value:         taken: taken.then(|| stamp_new(7, 8)),
        origin: stamp_new(1, 2),
        // Unsupported field/value:         outcome: lookup_of(count, total),
        label,
    }
}

// Decompose a [`Report`] into its value form — the accessor
// `expand_return!(Report).fields(fields!(...))` names. Cloning the fields is
// what makes this a *value* form; the generated code calls it ONCE per
// delivery and reads every leaf off that one result.
#[prebindgen]
pub fn report_to_struct(r: &Report) -> ReportStruct {
    ReportStruct {
        summary: r.summary.clone(),
        // Unsupported field/value:         taken: r.taken,
        origin: r.origin,
        // Unsupported field/value:         outcome: r.outcome.clone(),
        label: r.label.clone(),
    }
}

// The **consuming** value form of [`Report`] — the same fields, reached by
// destroying the report instead of cloning out of a borrow.
//
// This is the shape a hot receive path wants. Every callback hands its value
// over **owned** (`impl Fn(Report)`), so there is nothing to preserve: moving
// the fields out costs nothing, while [`report_to_struct`] pays a clone per
// handle field for a value it is about to drop. The binding picks whichever
// form it declares; `expand_return!(Report).fields(fields!(report_into_struct))`
// selects this one and the generated code then **moves** each field into its
// leaf rather than cloning it.
#[prebindgen]
pub fn report_into_struct(r: Report) -> ReportStruct {
    ReportStruct {
        summary: r.summary,
        // Unsupported field/value:         taken: r.taken,
        origin: r.origin,
        // Unsupported field/value:         outcome: r.outcome,
        label: r.label,
    }
}

// Disabled: unsupported by the current binding subset
// /// Deliver a [`Report`] to a callback — the decomposed value form arriving in
// /// ONE crossing, which is the whole point of deriving the boundary rather than
// /// handing over a handle the receiver must then query field by field.
// #[prebindgen]
// pub fn report_each(n: i64, sink: impl Fn(Report) + Send + Sync + 'static) {
//     for i in 0..n {
//         sink(report_new(
//             i - 1,
//             10.0 * i as f64,
//             i % 2 == 0,
//             format!("r{i}"),
//         ));
//     }
// }

// ─────────────────────────────────────────────────────────────────────────────
// Probe — a value form with an `Option<sum>` FIELD (#220).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// The handle whose boundary is derived from [`ProbeStruct`].
// #[prebindgen]
// pub type Probe = handles::Probe;

// Disabled: unsupported by the current binding subset
// /// [`Probe`]'s value form. `outcome` is the shape this exists for: a sum behind
// /// an `Option`, which used to be refused here while the very same field was
// /// accepted on a `data_class`.
// ///
// /// Absence rides the **selector's own nullability**, not a present flag beside
// /// it — `Lookup::Absent` is tag `0`, so a raw `jint` has no spelling left for
// /// "no sum at all" and the tag boxes. That is the same rule a sum under a
// /// conditional value form already crosses by, which is why this needed no new
// /// leaf kind.
// #[prebindgen]
// pub struct ProbeStruct {
//     /// A plain leaf beside the gated segment: gating is the segment's, not the
//     /// whole form's.
//     pub seq: i64,
//     /// Absent, or one of `Lookup`'s alternatives — including the one carrying
//     /// an opaque handle.
//     pub outcome: Option<Lookup>,
// }

// Disabled: unsupported by the current binding subset
// /// Build a [`Probe`]. `count < -1` leaves the outcome absent; anything else
// /// takes it from [`lookup_of`], so all four cases (absent, failed, empty,
// /// found-with-a-handle) are reachable from one argument.
// #[prebindgen]
// pub fn probe_new(seq: i64, count: i64, total: f64) -> Probe {
//     Probe {
//         seq,
//         outcome: (count >= -1).then(|| lookup_of(count, total)),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// The value form accessor `expand_return!(Probe).fields(fields!(..))` names.
// #[prebindgen]
// pub fn probe_to_struct(p: &Probe) -> ProbeStruct {
//     ProbeStruct {
//         seq: p.seq,
//         outcome: p.outcome.clone(),
//     }
// }

// Disabled: unsupported by the current binding subset
// /// Deliver [`Probe`]s to a callback, one per `i` starting at `-2`, so the first
// /// is the ABSENT case and the rest walk `Lookup`'s alternatives.
// #[prebindgen]
// pub fn probe_each(n: i64, total: f64, sink: impl Fn(Probe) + Send + Sync + 'static) {
//     for i in 0..n {
//         sink(probe_new(i, i - 2, total));
//     }
// }

// ─────────────────────────────────────────────────────────────────────────────
// Ledger — a value form reached through an `Option` (the CONDITIONAL hoist).
// ─────────────────────────────────────────────────────────────────────────────

// Disabled: unsupported by the current binding subset
// /// Holds reports a binding reaches **optionally**, so [`Report`]'s derived
// /// boundary is spliced in below an `Option` step rather than at a return of its
// /// own. That makes its value form conditional: the accessor runs only where the
// /// report is present, and every leaf under it is null otherwise.
// ///
// /// Both payload shapes are here because they need opposite treatment at the
// /// value-form call — the borrowed one is passed through (and cloned by a
// /// by-value accessor), the owned one is moved into it — and only one of the two
// /// is exercised by any single accessor.
// #[prebindgen]
// pub type Ledger = handles::Ledger;

// Disabled: unsupported by the current binding subset
// /// Build a [`Ledger`]; `n` selects which of the two slots are filled (bit 0 =
// /// `filed`, bit 1 = `archived`), so a caller can drive every arm of the
// /// conditional decomposition, both-present through both-absent.
// #[prebindgen]
// pub fn ledger_new(n: i64) -> Ledger {
//     Ledger {
//         filed: (n & 1 != 0).then(|| ledger_report(1)),
//         archived: (n & 2 != 0).then_some(2),
//     }
// }

// Disabled: unsupported by the current binding subset
// fn ledger_report(seed: i64) -> Report {
//     report_new(seed, 10.0 * seed as f64, seed % 2 == 0, format!("l{seed}"))
// }

// Disabled: unsupported by the current binding subset
// /// The filed report, **borrowed** (`Option<&Report>`) — the shape a `z_*`-style
// /// accessor hands back, where a by-value value form has nothing to take and
// /// clones first.
// #[prebindgen]
// pub fn ledger_filed(l: &Ledger) -> Option<&Report> {
//     l.filed.as_ref()
// }

// Disabled: unsupported by the current binding subset
// /// The archived report, **owned** (`Option<Report>`) — the equally ordinary
// /// shape whose payload is the caller's to move, so a by-value value form takes
// /// it directly. Built on demand precisely because `Report` is not `Clone`.
// #[prebindgen]
// pub fn ledger_archived(l: &Ledger) -> Option<Report> {
//     l.archived.map(ledger_report)
// }

// Disabled: unsupported by the current binding subset
// /// A **transparent wrapper**, and its unwrapped control.
// ///
// /// The model erases `Box`, so `Box<Box<Option<String>>>` classifies `Optional`
// /// exactly as a bare `Option<String>` does — one thing to every destination
// /// language, two spellings to Rust. That gap is the whole of #270: the adapter
// /// used to decide what a type *was* by rebuilding a pattern from its spelling,
// /// so a wrapped `Option` reconstructed as `Box<_>`, matched nothing, and got no
// /// converter at all.
// ///
// /// Declared here rather than only in a unit test because this crate's generated
// /// binding is `include!`d and **compiled**: a converter that named
// /// `Option<String>` for a `Box<Box<Option<String>>>` value, or bridged it with
// /// the wrong number of dereferences, fails to build. Nested deliberately — one
// /// dereference leaves a `Box<Option<String>>`, which still compiles as a
// /// *type* and would only fail here.
// ///
// /// A `Cow` payload is the other half and cannot appear in a compiled fixture:
// /// it must be REFUSED, which only
// /// `a_transparent_wrapper_is_bridged_only_where_it_can_be` can assert.
// ///
// /// The **parameter** is wrapped too, and that half is #273: nullability was
// /// decided by asking the spelling whether its last path segment read `Option`,
// /// so this rendered `note: String` while `plain_note_echo` rendered
// /// `note: String?`. A non-null Kotlin parameter for an optional value is a
// /// wrong contract rather than a cosmetic one — Kotlin rejects `null` at the
// /// call site, so the absent case becomes unexpressible. The two externs must
// /// come out **identical**.
// #[prebindgen]
// pub fn boxed_note_echo(note: Box<Option<String>>) -> Box<Box<Option<String>>> {
//     Box::new(note)
// }

/// Round-trip an optional string without transparent wrappers.
#[prebindgen]
pub fn plain_note_echo(note: Option<String>) -> Option<String> {
    note
}

// Disabled: unsupported by the current binding subset
// /// A transparent wrapper over a **decomposed** return — the shape `boxed_note_echo`
// /// does not reach.
// ///
// /// `boxed_note_echo`'s return takes an output *converter*, which is selected for
// /// the spelling and therefore names `Box<Option<String>>` itself. This one has
// /// no converter at all: `Summary` carries a declared output expansion, so the
// /// extern **binds the returned value and matches it** to deliver the leaves to a
// /// builder. Match ergonomics does not see through a `Box`, so the emitter has to
// /// move the value out of the wrappers the classification erased before it can
// /// destructure — the defect #292's audit found, and one this crate compiles.
// ///
// /// The unwrapped twin is [`archive_latest`], which crosses as the same
// /// `Summary?`.
// #[prebindgen]
// pub fn boxed_latest(a: &Archive) -> Box<Option<Summary>> {
//     Box::new(a.latest.clone())
// }

// Disabled: unsupported by the current binding subset
// /// An `Option<data class>` whose data class has a **required handle** field.
// ///
// /// The shape that proves an optional node's field decodes stay **inside** its
// /// presence gate. When the Kotlin object is null every leaf carries an inert
// /// placeholder, and a handle leaf's placeholder is pointer `0` — which the
// /// direct-handle decode reads as a closed handle, signals a binding error for,
// /// and returns from. Hoisting the decodes out of the gate therefore turns
// /// `null` into an error instead of `None`, and no fixture whose fields all
// /// decode successfully can tell the difference.
// ///
// /// `Summary` is consumed by value here, as a handle field is: the `Some` case
// /// hands over ownership, and the `None` case must never touch the slot.
// #[prebindgen]
// pub struct Holder {
//     pub tag: i64,
//     pub summary: Summary,
// }

// Disabled: unsupported by the current binding subset
// /// `tag` when the holder is present, `fallback` when it is absent — so the
// /// absent arm is observable as a **value** rather than as an error.
// #[prebindgen]
// pub fn holder_tag_or(h: Option<Holder>, fallback: i64) -> i64 {
//     match h {
//         Some(h) => h.tag + h.summary.count,
//         None => fallback,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// A data class whose **fields** carry transparent wrappers.
// ///
// /// This is what #289 changes and why it could not land alone. The field walk
// /// used to peel with `option_inner_type`, which reads the last path segment: a
// /// field spelled `Box<Option<i64>>` answered "not optional" and crossed as one
// /// boxed `java.lang.Long`. The model says `Optional`, so it now takes the
// /// decoupled `(present, value)` pair its bare twin does — and the emitter has to
// /// put the `Box` back when it rebuilds, or the migration turns a working boxed
// /// crossing into an `E0308`.
// ///
// /// `plain` is the control: the two fields must produce the same wire, since the
// /// model says they are the same type.
// #[prebindgen]
// pub struct WrappedFields {
//     pub id: i64,
//     pub boxed: Box<Option<i64>>,
//     pub plain: Option<i64>,
//     /// The same pairing over a **terminal**, which is the shape that had no
//     /// outbound route at all: `Box<Option<_>>` above rides the `Optional` layer
//     /// arm, while `Box<Priority>` classifies as `Named` and no arm claims it
//     /// (#309). Both must present as `Priority` in Kotlin — the wrapper is
//     /// invisible there, which is why the model erases it.
//     pub boxed_enum: Box<Priority>,
//     pub plain_enum: Priority,
// }

// Disabled: unsupported by the current binding subset
// /// Round-trip a [`WrappedFields`] so both field spellings cross in one call.
// #[prebindgen]
// pub fn wrapped_fields_sum(w: WrappedFields) -> i64 {
//     w.id + w.boxed.unwrap_or(0)
//         + w.plain.unwrap_or(0)
//         + i64::from(priority_weight(*w.boxed_enum))
//         + i64::from(priority_weight(w.plain_enum))
// }

// Disabled: unsupported by the current binding subset
// /// Transparent wrappers on the **input** side, one per specialized lowering.
// ///
// /// These lowerings do not *decode* their parameter, they **rebuild** it — a
// /// literal `Payload { .. }`, an `Option::Some(v)`, a `Vec<T>` pushed element by
// /// element — so the wrappers the classification erased have to go back on before
// /// the value reaches the signature. Rebuilding from the classification alone
// /// hands an `Option<Payload>` to a parameter spelled `Box<Option<Payload>>`:
// /// `E0308` (#292 item 3, which replaced #290's refusals).
// ///
// /// Declared here rather than only in unit tests for the reason
// /// [`boxed_note_echo`] is: this crate's generated binding is `include!`d and
// /// **compiled**, so a missing or misplaced `Box::new` fails the build. Each has
// /// an unwrapped twin already declared — the surfaces must come out identical,
// /// since the model says the two spellings are one type.
// ///
// /// The layers are covered separately because each is applied at a different
// /// point in the construction, and only a shape that exercises one can show it:
// /// the core wrap goes inside the present gate, and the optional wrap around it.
// ///
// /// **`Box<&Payload>` is deliberately absent.** The flatten lowering could build
// /// it — it owns a local and `Box::new(&local)` is well-typed — but a declared
// /// parameter also needs a general converter entry, and a converter *produces* an
// /// owned value: there is nothing for a `Box<&T>` to borrow from that outlives
// /// the call (`E0106` on the generated signature). So the shape is refused at
// /// resolution, by the converter's nature rather than the wrapper's.
// #[prebindgen]
// // A boxed parameter IS the point here — clippy is right that the `Box` buys
// // nothing, and that is what makes it a fixture: the binding must cross it as
// // the unwrapped type and put the wrapper back.
// #[allow(clippy::boxed_local)]
// pub fn boxed_payload_id(p: Box<Payload>) -> i64 {
//     p.id
// }

// Disabled: unsupported by the current binding subset
// /// The optional layer over the same rebuild — the wrap goes **around** the
// /// present gate, where the core wrap goes inside it.
// #[prebindgen]
// // A boxed parameter IS the point here — clippy is right that the `Box` buys
// // nothing, and that is what makes it a fixture: the binding must cross it as
// // the unwrapped type and put the wrapper back.
// #[allow(clippy::boxed_local)]
// pub fn boxed_opt_payload_id(p: Box<Option<Payload>>) -> i64 {
//     p.as_ref().as_ref().map(|p| p.id).unwrap_or(-1)
// }

// Disabled: unsupported by the current binding subset
// /// The option-scalar lowering (`(present, value)` raw pair) under a wrapper —
// /// the rebuilt `Option` is re-wrapped before it reaches the signature.
// #[prebindgen]
// // A boxed parameter IS the point here — clippy is right that the `Box` buys
// // nothing, and that is what makes it a fixture: the binding must cross it as
// // the unwrapped type and put the wrapper back.
// #[allow(clippy::boxed_local)]
// pub fn boxed_opt_priority_weight(p: Box<Option<Priority>>) -> i64 {
//     match *p {
//         Some(v) => priority_weight(v) as i64,
//         None => -1,
//     }
// }

// Disabled: unsupported by the current binding subset
// /// A wrapped **element** in the Vec-build path. The storage is the CANONICAL
// /// `Vec<Payload>` — one helper trio per Kotlin class, shared with every other
// /// spelling of the same element — and the `Box` goes back on where the Vec is
// /// consumed, in one pass (#296).
// ///
// /// Load-bearing: the refusal it replaced was silent and cost-only, so nothing
// /// failed while `Vec<Box<Payload>>` fell back to a per-element `JObject` plus a
// /// field read per field. Its generated Rust is the evidence — take the wrap off
// /// the consumption site with this declared and the crate does not build.
// #[prebindgen]
// pub fn boxed_elem_id_sum(ps: Vec<Box<Payload>>) -> i64 {
//     ps.iter().map(|p| p.id).sum()
// }

// Disabled: unsupported by the current binding subset
// /// A wrapped **run**, by value: `mem::take` yields the owned `Vec`, so the
// /// `Box` costs nothing. The borrowed twin (`&Box<Vec<Payload>>`) is deliberately
// /// **not** declared — interposing a `Box` between the caller's Vec and the
// /// callee would require copying it, so that shape keeps the ordinary path.
// #[prebindgen]
// // A boxed parameter IS the point here — clippy is right that the `Box` buys
// // nothing, and that is what makes it a fixture: the binding must cross it as
// // the unwrapped type and put the wrapper back.
// #[allow(clippy::boxed_local)]
// pub fn boxed_run_id_sum(ps: Box<Vec<Payload>>) -> i64 {
//     ps.iter().map(|p| p.id).sum()
// }

// Disabled: unsupported by the current binding subset
// /// The **borrowed** run spelled as a slice — the control half of the pair with
// /// [`ref_vec_id_sum`].
// ///
// /// Declared as a sum rather than reusing [`crate::storage_put_slice`] so the two
// /// spellings can be weighed against each other directly: same argument, same
// /// answer, or the claim is only that each compiles.
// #[prebindgen]
// pub fn slice_id_sum(ps: &[Payload]) -> i64 {
//     ps.iter().map(|p| p.id).sum()
// }

// Disabled: unsupported by the current binding subset
// /// The same borrowed run spelled `&Vec<T>`, which is **one type** to the model:
// /// `sequence_elem` answers for `&[T]` and `&Vec<T>` alike, so both reach the
// /// Vec-build path and the emitter has to serve both.
// ///
// /// It did not. The by-ref lowering hands the callee a borrow of the transient
// /// Rust-side `Vec`, and ascribing that borrow `&[T]` coerced it at the `let` —
// /// so this spelling got a `&[Payload]` and the generated crate did not build
// /// (`E0308`; the deref coercion runs `&Vec<T>` → `&[T]`, not back). #384.
// ///
// /// Load-bearing, and the only kind of fixture that can be: a lib test emits
// /// tokens and never compiles them, while this crate's binding is `include!`d
// /// and built. Put the ascription back and `cargo build -p covertest-kotlin`
// /// fails here.
// #[prebindgen]
// // A `&Vec` parameter IS the point here — clippy is right that it should be
// // `&[_]`, and that is what makes it a fixture: the two spellings are one type
// // to the model, so the binding must serve both.
// #[allow(clippy::ptr_arg)]
// pub fn ref_vec_id_sum(ps: &Vec<Payload>) -> i64 {
//     ps.iter().map(|p| p.id).sum()
// }

// Disabled: unsupported by the current binding subset
// /// Deliver a [`Ledger`] to a callback, so both conditional decompositions cross
// /// in ONE call — including the sum (`Report::outcome`) each one carries, whose
// /// `match` belongs inside the arm that binds the report.
// #[prebindgen]
// pub fn ledger_each(n: i64, sink: impl Fn(Ledger) + Send + Sync + 'static) {
//     for i in 0..n {
//         sink(ledger_new(i));
//     }
// }

// Disabled: unsupported by the current binding subset
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{storage_new, storage_put_slice};
//
//     fn payload(id: i64, value: f64, label: Option<&str>) -> Payload {
//         Payload {
//             id,
//             seq: id as i32,
//             value,
//             flag: id % 2 == 0,
//             label: label.map(|s| Box::new(s.to_string())),
//         }
//     }
//
//     #[test]
//     fn priority_classifies_by_magnitude() {
//         assert_eq!(payload_priority(&payload(1, 3.0, None)), Priority::Low);
//         assert_eq!(payload_priority(&payload(1, 50.0, None)), Priority::High);
//         assert_eq!(payload_priority(&payload(1, 500.0, None)), Priority::Normal);
//         assert_eq!(priority_weight(Priority::High), 10);
//         assert_eq!(priority_or(None, Priority::Normal), Priority::Normal);
//         assert_eq!(
//             priority_or(Some(Priority::Low), Priority::High),
//             Priority::Low
//         );
//     }
//
//     #[test]
//     fn stamps_roundtrip() {
//         let s = stamp_new(7, 42);
//         assert_eq!(stamp_secs(&s), 7);
//         assert_eq!(stamp_nanos(&s), 42);
//         let series = stamp_series(3);
//         assert_eq!(series.len(), 3);
//         assert_eq!(series[2], Stamp { secs: 2, nanos: 0 });
//         assert!(stamp_series(-1).is_empty());
//     }
//
//     #[test]
//     fn fallible_label_constructor() {
//         assert!(storage_try_with_label("").is_err());
//         let s = storage_try_with_label("hi").expect("non-empty label");
//         assert_eq!(storage_len(&s), 1);
//         let err = storage_try_with_label("").err().unwrap();
//         assert_eq!(storage_error_message(&err), "label must not be empty");
//     }
//
//     #[test]
//     fn summary_aggregates_storage() {
//         let mut s = storage_new();
//         storage_put_slice(
//             &mut s,
//             &[payload(1, 10.0, None), payload(2, 30.0, Some("x"))],
//         );
//         let sum = storage_summary(&s);
//         assert_eq!(summary_count(&sum), 2);
//         assert_eq!(summary_total(&sum), 40.0);
//         assert_eq!(summary_scaled(&sum, 2.0), 80.0);
//
//         assert!(storage_matches_summary(&s, summary_new(2, 40.0)));
//         assert!(!storage_matches_summary(&s, summary_new(1, 40.0)));
//         assert_eq!(summary_total_raw(storage_summary_handle(&s)), 40.0);
//         assert!(storage_expect_summary(&mut s, summary_new(2, 40.0)));
//     }
//
//     #[test]
//     fn storage_scalar_members() {
//         let s = storage_with_payload(payload(42, 1.0, Some("a")));
//         assert_eq!(storage_len(&s), 1);
//         assert!(storage_contains(&s, 42));
//         assert!(!storage_contains(&s, 7));
//     }
//
//     #[test]
//     fn millis_wrapper_arithmetic() {
//         assert_eq!(millis_add(Millis(100), Millis(50)), Millis(150));
//     }
//
//     #[test]
//     fn label_len_is_optional() {
//         assert_eq!(payload_label_len(&payload(1, 0.0, Some("abcd"))), Some(4));
//         assert_eq!(payload_label_len(&payload(1, 0.0, None)), None);
//     }
//
//     #[test]
//     fn annotated_roundtrips() {
//         let a = annotated_new(payload(1, 2.5, Some("x")), Some(30), Some(Priority::High));
//         assert_eq!(annotated_ttl(&a), Some(30));
//         assert_eq!(annotated_priority(&a), Some(Priority::High));
//         assert_eq!(annotated_payload_value(&a), 2.5);
//         assert_eq!(annotated_alternate_value(&a), None);
//         let with_alternate = Annotated {
//             alternate: Some(payload(2, 7.5, None)),
//             ..a.clone()
//         };
//         assert_eq!(annotated_alternate_value(&with_alternate), Some(7.5));
//         let b = annotated_new(payload(1, 0.0, None), None, None);
//         assert_eq!(annotated_ttl(&b), None);
//         assert_eq!(annotated_priority(&b), None);
//     }
//
//     #[test]
//     fn cache_config_weight_reaches_nested_enum() {
//         let cache = CacheConfig {
//             replies: RepliesConfig {
//                 priority: Priority::High,
//                 max_samples: 4,
//             },
//             ttl: 7,
//         };
//         // priority_weight(High) == 10, plus ttl 7.
//         assert_eq!(cache_config_weight(Some(cache)), 17);
//         assert_eq!(cache_config_weight(None), -1);
//     }
//
//     #[test]
//     fn shards_are_independent() {
//         let shards = storage_shards(3, 2);
//         assert_eq!(shards.len(), 3);
//         assert!(shards.iter().all(|s| storage_len(s) == 2));
//         assert!(storage_contains(&shards[2], 2001));
//         assert!(!storage_contains(&shards[0], 2001));
//         assert!(storage_shards(0, 2).is_empty());
//         assert!(storage_shards_opt(0, 2).is_none());
//         assert_eq!(storage_shards_opt(2, 1).unwrap().len(), 2);
//     }
//
//     #[test]
//     fn summary_series_shapes() {
//         let s = summary_series(3, 10);
//         assert_eq!(s.len(), 3);
//         assert_eq!(summary_count(&s[2]), 12);
//         assert_eq!(summary_total(&s[2]), 120.0);
//         assert!(summary_series(0, 5).is_empty());
//         assert!(summary_series_opt(-1, 0).is_none());
//         assert!(summary_series_opt(0, 0).unwrap().is_empty());
//         assert_eq!(summary_series_opt(2, 1).unwrap().len(), 2);
//     }
//
//     #[test]
//     fn storage_handler_receives_owned_storage() {
//         use std::sync::{
//             atomic::{AtomicI64, Ordering},
//             Arc,
//         };
//         let seen = Arc::new(AtomicI64::new(-1));
//         let seen2 = seen.clone();
//         let h = storage_handler_new(move |s| seen2.store(storage_len(&s), Ordering::SeqCst));
//         storage_emit(4, &h);
//         assert_eq!(seen.load(Ordering::SeqCst), 4);
//     }
//
//     #[test]
//     fn archive_borrows_latest() {
//         let mut a = archive_new();
//         assert!(archive_latest(&a).is_none());
//         archive_store(&mut a, summary_new(2, 40.0));
//         assert_eq!(summary_count(archive_latest(&a).unwrap()), 2);
//     }
//
//     #[test]
//     fn misc_shapes() {
//         let s1 = storage_with_payload(payload(1, 0.0, Some("a")));
//         let s2 = storage_with_payload(payload(2, 0.0, None));
//         let mut s3 = storage_new();
//         assert_eq!(storage_total_len(&s1, &s2, &s3), 2);
//         assert_eq!(storage_labels(&s1), vec!["a".to_string()]);
//         assert!(storage_labels(&s2).is_empty());
//         assert!(storage_put_opt(&mut s3, Some(payload(3, 0.0, None))));
//         assert!(!storage_put_opt(&mut s3, None));
//         assert_eq!(storage_len(&s3), 1);
//     }
// }

// Primitive boundary conformance: exercise every stable scalar in both directions.
#[prebindgen]
pub fn primitive_i8(value: i8) -> i8 {
    value
}
#[prebindgen]
pub fn primitive_i16(value: i16) -> i16 {
    value
}
#[prebindgen]
pub fn primitive_i32(value: i32) -> i32 {
    value
}
#[prebindgen]
pub fn primitive_i64(value: i64) -> i64 {
    value
}
#[prebindgen]
pub fn primitive_i128(value: i128) -> i128 {
    value
}
#[prebindgen]
pub fn primitive_isize(value: isize) -> isize {
    value
}
#[prebindgen]
pub fn primitive_u8(value: u8) -> u8 {
    value
}
#[prebindgen]
pub fn primitive_u16(value: u16) -> u16 {
    value
}
#[prebindgen]
pub fn primitive_u32(value: u32) -> u32 {
    value
}
#[prebindgen]
pub fn primitive_u64(value: u64) -> u64 {
    value
}
#[prebindgen]
pub fn primitive_u128(value: u128) -> u128 {
    value
}
#[prebindgen]
pub fn primitive_usize(value: usize) -> usize {
    value
}
#[prebindgen]
pub fn primitive_f32(value: f32) -> f32 {
    value
}
#[prebindgen]
pub fn primitive_f64(value: f64) -> f64 {
    value
}
#[prebindgen]
pub fn primitive_bool(value: bool) -> bool {
    value
}
#[prebindgen]
pub fn primitive_char(value: char) -> char {
    value
}
#[prebindgen]
pub fn primitive_unit() {}

// Consuming input followed by a fallible wire conversion: invalid text must
// leave the report owned by the caller, before any source function executes.
#[prebindgen]
pub fn report_relabel(mut report: Report, label: &str) -> Report {
    report.label = label.to_owned();
    report
}

// A constructor with an adapter-owned input nested under another conversion.
#[prebindgen]
pub fn report_from_summary(summary: Summary, label: &str) -> Report {
    Report {
        summary,
        origin: stamp_new(1, 2),
        label: label.to_owned(),
    }
}
#[prebindgen]
pub fn report_label(report: Report) -> String {
    report.label
}

// A value-only record for the replacement engine's data-class boundary coverage.
// No Clone derive: the output conversion observes individual fields.
#[prebindgen]
pub struct DataEnvelope {
    pub label: String,
    pub blob: BlobValue,
    pub unsigned: Unsigned,
    pub enabled: bool,
    pub code: char,
    pub signed: i128,
    pub wide: u128,
}
#[prebindgen]
pub fn data_envelope_echo(value: DataEnvelope) -> DataEnvelope {
    value
}
#[prebindgen]
pub fn blob_value_selected(value: BlobValue) -> BlobValue {
    value
}
#[prebindgen]
pub fn blob_value_views(value: BlobValue) -> BlobValue {
    value
}

// These floats exercise data-class equality semantics when byte fields require
// generated content-based equals/hashCode (NaN and signed zero in particular).
#[prebindgen]
pub struct DataFloats {
    pub bytes: Vec<u8>,
    pub f: f32,
    pub d: f64,
}
#[prebindgen]
pub fn data_floats_echo(value: DataFloats) -> DataFloats {
    value
}

#[prebindgen]
pub struct DataPrimitives {
    pub i8v: i8,
    pub i16v: i16,
    pub i32v: i32,
    pub i64v: i64,
    pub i128v: i128,
    pub isizev: isize,
    pub u8v: u8,
    pub u16v: u16,
    pub u32v: u32,
    pub u64v: u64,
    pub u128v: u128,
    pub usizev: usize,
    pub f32v: f32,
    pub f64v: f64,
    pub boolv: bool,
    pub charv: char,
}
#[prebindgen]
pub fn data_primitives_echo(value: DataPrimitives) -> DataPrimitives {
    value
}
#[prebindgen]
pub fn data_primitives_zero() -> DataPrimitives {
    DataPrimitives {
        i8v: 0,
        i16v: 0,
        i32v: 0,
        i64v: 0,
        i128v: 0,
        isizev: 0,
        u8v: 0,
        u16v: 0,
        u32v: 0,
        u64v: 0,
        u128v: 0,
        usizev: 0,
        f32v: 0.0,
        f64v: 0.0,
        boolv: false,
        charv: '\0',
    }
}

#[prebindgen]
pub fn data_with_storage(storage: Storage, value: DataEnvelope) -> Storage {
    drop(value);
    storage
}
#[prebindgen]
pub fn data_envelope_from_blob(blob: BlobValue) -> DataEnvelope {
    DataEnvelope {
        label: "from blob".into(),
        blob,
        unsigned: unsigned_round_trip(1, 2, 3, 4, None),
        enabled: true,
        code: 'x',
        signed: -1,
        wide: 1,
    }
}
#[prebindgen]
pub fn data_envelope_constructed(value: DataEnvelope) -> DataEnvelope {
    value
}

#[prebindgen]
pub fn blob_value_copy(value: &BlobValue) -> BlobValue {
    value.clone()
}

static OWNED_RESOURCE_DROPS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Non-cloneable resource used to verify owned record returns.
pub struct OwnedResourceImpl {
    id: i64,
}
impl Drop for OwnedResourceImpl {
    fn drop(&mut self) {
        OWNED_RESOURCE_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[prebindgen]
pub type OwnedResource = OwnedResourceImpl;
#[prebindgen]
pub struct ResourcePair {
    pub first: OwnedResource,
    pub second: OwnedResource,
}
#[prebindgen]
pub struct ResourceBundle {
    pub label: String,
    pub pair: ResourcePair,
    pub tail: OwnedResource,
    pub details: DataFloats,
}
#[prebindgen]
pub fn resource_bundle_new() -> ResourceBundle {
    ResourceBundle {
        label: "owned bundle".into(),
        pair: ResourcePair {
            first: OwnedResourceImpl { id: 1 },
            second: OwnedResourceImpl { id: 2 },
        },
        tail: OwnedResourceImpl { id: 3 },
        details: DataFloats {
            bytes: vec![1, 2, 3],
            f: 1.5,
            d: 2.5,
        },
    }
}
#[prebindgen]
pub fn resource_bundle_label(value: &ResourceBundle) -> String {
    value.label.clone()
}
#[prebindgen]
pub fn owned_resource_id(value: &OwnedResource) -> i64 {
    value.id
}
#[prebindgen]
pub fn owned_resource_drops() -> usize {
    OWNED_RESOURCE_DROPS.load(std::sync::atomic::Ordering::SeqCst)
}
#[prebindgen]
pub fn resource_bundle_views() -> ResourceBundle {
    resource_bundle_new()
}
#[prebindgen]
pub fn resource_bundle_default() -> ResourceBundle {
    resource_bundle_new()
}
#[prebindgen]
pub fn resource_bundle_fields() -> ResourceBundle {
    resource_bundle_new()
}
#[prebindgen]
pub fn resource_bundle_into(value: ResourceBundle) -> ResourceBundle {
    value
}

static PROJECTION_CLONES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static PROJECTION_DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub struct ProjectionHandleImpl(i64);
impl Clone for ProjectionHandleImpl {
    fn clone(&self) -> Self {
        PROJECTION_CLONES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self(self.0)
    }
}
impl Drop for ProjectionHandleImpl {
    fn drop(&mut self) {
        PROJECTION_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[prebindgen]
pub type ProjectionHandle = ProjectionHandleImpl;
#[prebindgen]
pub struct ProjectionPair {
    pub first: ProjectionHandle,
    pub second: ProjectionHandle,
}
#[prebindgen]
pub fn projection_pair_borrowed() -> ProjectionPair {
    ProjectionPair {
        first: ProjectionHandleImpl(1),
        second: ProjectionHandleImpl(2),
    }
}
#[prebindgen]
pub fn projection_pair_owned() -> ProjectionPair {
    projection_pair_borrowed()
}
#[prebindgen]
pub fn projection_pair_mixed() -> ProjectionPair {
    projection_pair_borrowed()
}
#[prebindgen]
pub fn projection_clones() -> usize {
    PROJECTION_CLONES.load(std::sync::atomic::Ordering::SeqCst)
}
#[prebindgen]
pub fn projection_drops() -> usize {
    PROJECTION_DROPS.load(std::sync::atomic::Ordering::SeqCst)
}
#[prebindgen]
pub fn projection_pair_views() -> ProjectionPair {
    projection_pair_borrowed()
}

// Option boundary fixtures shared by C and JNI.
#[prebindgen]
pub fn option_i64(value: Option<i64>) -> Option<i64> {
    value
}
#[prebindgen]
pub fn option_char(value: Option<char>) -> Option<char> {
    value
}
#[prebindgen]
pub fn option_u8(value: Option<u8>) -> Option<u8> {
    value
}
#[prebindgen]
pub fn option_bool(value: Option<bool>) -> Option<bool> {
    value
}
#[prebindgen]
pub fn option_text(value: Option<String>) -> Option<String> {
    value
}
#[prebindgen]
pub fn option_bytes(value: Option<Vec<u8>>) -> Option<Vec<u8>> {
    value
}
#[prebindgen]
pub fn option_unit(value: Option<()>) -> Option<()> {
    value
}
#[prebindgen]
pub fn option_nested(value: Option<Option<i64>>) -> Option<Option<i64>> {
    value
}
#[prebindgen]
pub fn option_nested_char(value: Option<Option<char>>) -> Option<Option<char>> {
    value
}
#[prebindgen]
pub fn option_resource_new(present: bool) -> Option<OwnedResource> {
    present.then(|| OwnedResourceImpl { id: 77 })
}
#[prebindgen]
pub fn option_resource_echo(value: Option<OwnedResource>, valid: char) -> Option<OwnedResource> {
    let _ = valid;
    value
}
#[prebindgen]
pub fn option_resource_id(value: Option<&OwnedResource>) -> i64 {
    value.map_or(-1, |v| v.id)
}
#[prebindgen]
pub struct OptionalFields {
    pub count: Option<i64>,
    pub text: Option<String>,
    pub letter: Option<char>,
}
#[prebindgen]
pub fn optional_fields_new(
    count: Option<i64>,
    text: Option<String>,
    letter: Option<char>,
) -> OptionalFields {
    OptionalFields {
        count,
        text,
        letter,
    }
}
#[prebindgen]
pub fn optional_fields_into(value: OptionalFields) -> OptionalFields {
    value
}
#[prebindgen]
pub fn optional_fields_echo(value: Option<OptionalFields>) -> Option<OptionalFields> {
    value
}
#[prebindgen]
pub struct OptionalResource {
    pub resource: Option<OwnedResource>,
    pub details: Option<OptionalFields>,
}
#[prebindgen]
pub fn optional_resource_new(present: bool) -> Option<OptionalResource> {
    Some(OptionalResource {
        resource: option_resource_new(present),
        details: present.then(|| optional_fields_new(Some(9), None, Some('é'))),
    })
}
#[prebindgen]
pub fn option_str_len(value: Option<&str>) -> Option<usize> {
    value.map(str::len)
}
#[prebindgen]
pub fn option_slice_len(value: Option<&[u8]>) -> Option<usize> {
    value.map(<[u8]>::len)
}
#[prebindgen]
pub fn option_string_len(value: Option<&String>) -> Option<usize> {
    value.map(String::len)
}
#[prebindgen]
pub fn option_i64_ref(value: Option<&i64>) -> Option<i64> {
    value.copied()
}
#[prebindgen]
pub fn option_projection_views(present: bool) -> Option<ProjectionHandle> {
    present.then(|| ProjectionHandleImpl(99))
}
#[prebindgen]
pub struct OptionChoice {
    pub id: i64,
}
#[prebindgen]
pub fn option_choice_new(id: i64) -> OptionChoice {
    OptionChoice { id }
}
#[prebindgen]
pub fn option_choice_id(value: Option<OptionChoice>) -> i64 {
    value.map_or(-1, |v| v.id)
}
#[prebindgen]
pub fn option_resource_pair(first: Option<OwnedResource>, second: Option<OwnedResource>) -> i64 {
    first.map_or(0, |v| v.id) + second.map_or(0, |v| v.id)
}
#[prebindgen]
pub struct OptionalBytes {
    pub data: Option<Vec<u8>>,
}
#[prebindgen]
pub fn optional_bytes_echo(value: OptionalBytes) -> OptionalBytes {
    value
}
#[prebindgen]
pub fn option_resource_mut(value: Option<&mut OwnedResource>) -> i64 {
    value.map_or(-1, |v| {
        v.id += 1;
        v.id
    })
}
#[prebindgen]
pub fn option_projection_ref(value: &ProjectionHandle, present: bool) -> Option<&ProjectionHandle> {
    present.then_some(value)
}
#[prebindgen]
pub struct OptionalPair {
    pub absent: Option<OwnedResource>,
    pub present: Option<OwnedResource>,
}
#[prebindgen]
pub fn optional_pair_new() -> OptionalPair {
    OptionalPair {
        absent: None,
        present: option_resource_new(true),
    }
}
#[prebindgen]
pub fn optional_pair_into(value: OptionalPair) -> OptionalPair {
    value
}
#[prebindgen]
pub fn option_str_echo(value: Option<&str>) -> Option<&str> {
    value
}
#[prebindgen]
pub fn option_slice_echo(value: Option<&[u8]>) -> Option<&[u8]> {
    value
}
#[prebindgen]
pub fn option_nested_resource(
    value: Option<Option<OwnedResource>>,
) -> Option<Option<OwnedResource>> {
    value
}
#[prebindgen]
pub struct OptionCtor {
    pub id: i64,
}
#[prebindgen]
pub fn option_ctor_new(value: Option<i64>) -> OptionCtor {
    OptionCtor {
        id: value.unwrap_or(-10),
    }
}
#[prebindgen]
pub fn option_ctor_id(value: OptionCtor) -> i64 {
    value.id
}
#[prebindgen]
pub struct ResourceChoice {
    pub id: i64,
}
#[prebindgen]
pub fn resource_choice_new(resource: OwnedResource, offset: i64) -> ResourceChoice {
    ResourceChoice {
        id: resource.id + offset,
    }
}
#[prebindgen]
pub fn resource_choice_id(value: Option<ResourceChoice>) -> i64 {
    value.map_or(-1, |v| v.id)
}
