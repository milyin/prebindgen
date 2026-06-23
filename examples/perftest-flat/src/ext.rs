//! Extended `#[prebindgen]` surface used **only** to exercise language-binding
//! generator features that the lean performance surface in [`crate`] does not
//! need. None of these items are used by the `perftest-*` benchmarks; they exist
//! so a *coverage* binding (e.g. `examples/covertest-kotlin`) can map one flat
//! library through **every** adapter feature and assert the result.
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

use crate::{Payload, Storage};
use prebindgen_proc_macro::prebindgen;

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
        .map(|i| Stamp {
            secs: i,
            nanos: 0,
        })
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
    let mut s = Storage::default();
    s.payloads = vec![Payload {
        id: 0,
        seq: 0,
        value: 0.0,
        flag: false,
        label: Some(Box::new(label.to_string())),
    }];
    Ok(s)
}

// ─────────────────────────────────────────────────────────────────────────────
// Summary — an opaque handle whose fields decompose at the boundary.
// ─────────────────────────────────────────────────────────────────────────────

/// An aggregate over a [`Storage`]'s payloads: how many there are and the sum of
/// their `value`s. An opaque handle in the binding, but its default
/// flatten-output decomposes it into `(count, total)` leaves and its
/// flatten-input rebuilds it from the same leaves (via [`summary_new`]).
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
    let mut s = Storage::default();
    s.payloads = vec![payload];
    s
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
// Option<scalar> — a nullable primitive return.
// ─────────────────────────────────────────────────────────────────────────────

/// Length of a payload's label, or `None` when it is unlabeled. Exercises an
/// `Option<i64>` (nullable primitive) return, distinct from the `Option<handle>`
/// / `Option<data-class>` shapes elsewhere.
#[prebindgen]
pub fn payload_label_len(p: &Payload) -> Option<i64> {
    p.label.as_ref().map(|s| s.len() as i64)
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
        assert_eq!(priority_or(Some(Priority::Low), Priority::High), Priority::Low);
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
}
