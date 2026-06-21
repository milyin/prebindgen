//! Flat, FFI-friendly example library demonstrating a **zero-copy** data struct.
//!
//! Every public function is annotated with `#[prebindgen]`, so `prebindgen`
//! captures this surface and a language adapter (here `prebindgen::lang::Cbindgen`,
//! driven by `perftest-c`) generates the FFI layer — no hand-written `extern "C"`
//! glue.
//!
//! [`Payload`] is `#[repr(C)]` and FFI-safe, so the C binding passes it **directly,
//! by reinterpret** (the C struct's memory *is* the Rust struct's memory) rather
//! than copying field-by-field. The string is carried as an **opaque pointer**
//! (`Option<Box<String>>`): a single nullable pointer that the C side holds as a
//! `string_t *` handle (because `String` is declared `opaque_ptr` in the binding
//! crate). This keeps the whole struct trivially `#[repr(C)]`/reinterpretable while
//! still carrying heap data.
//!
//! The functions operate on an opaque [`Storage`] handle (a `storage_t *` in C, a
//! `Storage` class in Kotlin) that owns the payload, so the matching Rust, C, and
//! Kotlin micro-benchmarks (`examples/perftest.rs`, `perftest-c/c/perftest.c`, and
//! `perftest-kotlin/.../Bench.kt`) measure the cost of the same operations natively
//! vs across the generated C ABI / JNI boundary — and exercise an opaque handle
//! crossing alongside the value struct.
//!
//! All three emit the same normalized `BEGIN_PERFTEST … END_PERFTEST` block;
//! `examples/perftest-bench.sh` builds, runs, and tabulates them into one comparison
//! (run `examples/perftest-bench.sh --quick` for a fast smoke).

use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
use std::mem::MaybeUninit;

/// Path to the directory where the `#[prebindgen]` macro records this crate's FFI
/// surface; read by consumers via `prebindgen::Source::new`.
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
/// The features enabled when this crate was built.
pub const FEATURES: &str = features!();

/// A by-value, FFI-safe payload. Scalars cross the C ABI as themselves; the
/// `label` string crosses as an opaque pointer (`Option<Box<String>>` ⇒ a nullable
/// `string_t *`). Being `#[repr(C)]`, the whole struct is passed by direct
/// reinterpret (zero-copy) — see `perftest-c`'s `.repr_c_struct(Payload)`.
// Field types are JNI-friendly (`i64`/`i32`/`f64`/`bool`): the JVM has no unsigned
// primitives, so the Kotlin consumer (`perftest-kotlin`) maps these directly. The C
// consumer treats them as the corresponding fixed-width C types.
#[prebindgen]
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Payload {
    pub id: i64,
    pub seq: i32,
    pub value: f64,
    pub flag: bool,
    pub label: Option<Box<String>>,
}

/// An opaque storage handle owning the **most recently stored batch** of
/// [`Payload`]s (a `Vec<Payload>`). The single-payload functions
/// ([`storage_put_by_take`], [`storage_get`], …) operate on this batch as the
/// array-of-one case ([`storage_put_by_take`] replaces it with a 1-element vec,
/// [`storage_get`] returns its first element); the array functions
/// ([`storage_put_slice`], [`storage_get_vec`]) store / return the whole batch.
///
/// The bindings expose it as an opaque pointer/handle (`storage_t *` in C, a
/// `Storage` class in Kotlin); its fields are never inspected across the FFI
/// boundary — the adapter boxes it and emits a typed destructor. (Not
/// `#[prebindgen]` and not `#[repr(C)]`: it is a boxed handle, like `Calculator`
/// in `example-flat`.)
#[derive(Default)]
pub struct Storage {
    payloads: Vec<Payload>,
}

/// An opaque, reusable handle wrapping a **prepared** `Fn(&Payload)` callback.
/// The foreign-side trampoline (e.g. the JNI global ref + method lookup that turn a
/// JVM callback into a Rust closure) is built **once** when the handle is created
/// ([`payload_handler_new`]); [`storage_callback`] then fires it many times without
/// rebuilding it. This is the registered-subscriber pattern: declare the handler
/// once, deliver events to it (cf. zenoh's `session_declare_subscriber` →
/// `Subscriber`). Like [`Storage`], it is a boxed handle (not `#[prebindgen]`/
/// `#[repr(C)]`); the adapter emits a typed destructor.
pub struct PayloadHandler(Box<dyn Fn(&Payload) + Send + Sync>);

/// Create a new, empty storage handle.
#[prebindgen]
pub fn storage_new() -> Storage {
    Storage::default()
}

/// Return a clone of the first stored payload (by value; crosses by reinterpret,
/// the `label` becoming a fresh owned `string_t *` the C caller must drop). A
/// freshly [`storage_new`]'d (empty) storage yields a [`Payload::default`].
#[prebindgen]
pub fn storage_get(s: &Storage) -> Payload {
    s.payloads.first().cloned().unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Five parameter-passing semantics for a `Payload` value struct, demonstrating the
// C wrappers the generator emits for each (see `perftest-c`):
//   - by_take            : `Payload`                  — by-value consume (`*mut`, move
//                                                       out + gravestone the owned ptr)
//   - by_read            : `&Payload`                 — shared read borrow (`const *`)
//   - by_read_and_update : `&mut Payload`             — read + write back in place
//   - get_into_init      : `&mut Payload`             — out-param; drops the old value
//                                                       (frees old label) before writing
//   - get_into_uninit    : `&mut MaybeUninit<Payload>`— out-param into uninit memory
//                                                       (writes without dropping)
// ─────────────────────────────────────────────────────────────────────────────

/// Move `payload` into the storage. Taken **by value**: across the C ABI this is a
/// consume — Rust reads the `payload_t` out through a `*mut` and writes a gravestone
/// back (nulling the owned `label` pointer) so the caller's later free is a no-op
/// (see `perftest-c`'s `.repr_c_struct(Payload)` — owned-ness is inferred from `label`).
#[prebindgen]
pub fn storage_put_by_take(s: &mut Storage, payload: Payload) {
    s.payloads = vec![payload];
}

/// Store a clone of `payload`, read through a shared borrow (`const payload_t *`).
/// The caller's payload is left untouched.
#[prebindgen]
pub fn storage_put_by_read(s: &mut Storage, payload: &Payload) {
    s.payloads = vec![payload.clone()];
}

/// Store a clone of `payload`, then **update the caller's payload in place** by
/// bumping its `seq` counter (a `&mut Payload` read/write borrow → `payload_t *`).
#[prebindgen]
pub fn storage_put_by_read_and_update(s: &mut Storage, payload: &mut Payload) {
    s.payloads = vec![payload.clone()];
    payload.seq += 1;
}

/// Write the stored payload into the caller's **already-initialized** `payload` slot.
/// The assignment drops the old value first (freeing its old `label`) — so the slot
/// must hold a valid payload (use [`storage_get_into_uninit`] for raw memory).
#[prebindgen]
pub fn storage_get_into_init(s: &Storage, payload: &mut Payload) {
    *payload = s.payloads.first().cloned().unwrap_or_default();
}

/// Write the stored payload into the caller's **uninitialized** `payload` slot,
/// without dropping whatever bytes were there (`&mut MaybeUninit<Payload>` →
/// `payload_t *`). The slot is initialized afterwards.
#[prebindgen]
pub fn storage_get_into_uninit(s: &Storage, payload: &mut MaybeUninit<Payload>) {
    payload.write(s.payloads.first().cloned().unwrap_or_default());
}

/// Prepare a reusable [`PayloadHandler`] from a callback `f`. The (foreign) closure
/// is decoded into the handler **once** here — reuse the handler across many
/// [`storage_callback`] calls instead of passing a fresh callback each time. This
/// is the "declare the subscriber once" step (its trampoline + per-call setup are
/// built here, amortized over every later delivery).
#[prebindgen]
pub fn payload_handler_new(f: impl Fn(&Payload) + Send + Sync + 'static) -> PayloadHandler {
    PayloadHandler(Box::new(f))
}

/// Invoke the prepared `handler` once **per stored payload** with a borrow of each
/// — reuses the handler's already-built foreign trampoline, so there is **no
/// per-call callback decoding** (only firing). After a single-payload put this
/// fires exactly once; after a [`storage_put_slice`] it fires once per slice
/// element. In C the closure receives a `const payload_t *` (zero-copy); in Kotlin
/// the borrowed `Payload` is delivered whole to the handler's
/// `PayloadCallback.run(Payload)` (its fields cross as decoupled leaves and are
/// reassembled on the Kotlin side — see `prebindgen::lang::JniGen`).
#[prebindgen]
pub fn storage_callback(s: &Storage, handler: &PayloadHandler) {
    for payload in &s.payloads {
        (handler.0)(payload);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Array (slice / Vec) API: store and retrieve a whole batch in one FFI call,
// amortizing per-call boundary overhead. The slice input crosses as
// `(const payload_t *, size_t)` in C (zero-copy reinterpret — `Payload` is
// `#[repr(C)]`) and as a `List<Payload>` in Kotlin; the `Vec` return crosses as a
// malloc'd `(payload_t *, size_t)` array in C and a `List<Payload>` in Kotlin.
// ─────────────────────────────────────────────────────────────────────────────

/// Replace the stored batch with a clone of `payloads`. The single-payload puts
/// are the array-of-one case of this.
#[prebindgen]
pub fn storage_put_slice(s: &mut Storage, payloads: &[Payload]) {
    s.payloads = payloads.to_vec();
}

/// Return a clone of the whole stored batch (each `label` becoming a fresh owned
/// `string_t *` the C caller must drop). [`storage_get`] is the first-element case
/// of this.
#[prebindgen]
pub fn storage_get_vec(s: &Storage) -> Vec<Payload> {
    s.payloads.clone()
}

/// Build the opaque string the C side stores in [`Payload::label`]. To C this
/// returns a `string_t *` (since `String` is declared `opaque_ptr`).
#[prebindgen]
pub fn string_new(s: &str) -> String {
    s.to_string()
}

/// Byte length of an opaque string — lets the C benchmark read it through the
/// `string_t *` handle.
#[prebindgen]
pub fn string_len(s: &String) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn payload(id: i64, label: Option<&str>) -> Payload {
        Payload {
            id,
            seq: id as i32,
            value: id as f64,
            flag: id % 2 == 0,
            label: label.map(|s| Box::new(s.to_string())),
        }
    }

    #[test]
    fn empty_storage_get_is_default() {
        let s = storage_new();
        assert_eq!(storage_get(&s), Payload::default());
        assert!(storage_get_vec(&s).is_empty());
    }

    #[test]
    fn single_put_get_roundtrip() {
        let mut s = storage_new();
        storage_put_by_take(&mut s, payload(1, Some("one")));
        let got = storage_get(&s);
        assert_eq!(got.id, 1);
        assert_eq!(got.label.as_deref().map(String::as_str), Some("one"));
        // A single put is the array-of-one case.
        assert_eq!(storage_get_vec(&s).len(), 1);
    }

    #[test]
    fn slice_put_vec_get_roundtrip() {
        let mut s = storage_new();
        let batch = vec![
            payload(1, Some("a")),
            payload(2, None),
            payload(3, Some("c")),
        ];
        storage_put_slice(&mut s, &batch);

        let out = storage_get_vec(&s);
        assert_eq!(out, batch);
        // The first element is what the single-payload `storage_get` returns.
        assert_eq!(storage_get(&s), batch[0]);
    }

    #[test]
    fn empty_slice_clears_batch() {
        let mut s = storage_new();
        storage_put_by_take(&mut s, payload(1, Some("one")));
        storage_put_slice(&mut s, &[]);
        assert!(storage_get_vec(&s).is_empty());
    }

    #[test]
    fn callback_fires_once_per_payload() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let handler = payload_handler_new(move |_p| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        let mut s = storage_new();
        // Single put → fires once.
        storage_put_by_take(&mut s, payload(1, None));
        storage_callback(&s, &handler);
        assert_eq!(count.load(Ordering::Relaxed), 1);

        // Slice of 3 → fires three more times.
        storage_put_slice(&mut s, &[payload(1, None), payload(2, None), payload(3, None)]);
        storage_callback(&s, &handler);
        assert_eq!(count.load(Ordering::Relaxed), 4);
    }
}
