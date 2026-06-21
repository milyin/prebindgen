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
#[derive(Clone, Debug, Default)]
pub struct Payload {
    pub id: i64,
    pub seq: i32,
    pub value: f64,
    pub flag: bool,
    pub label: Option<Box<String>>,
}

/// An opaque storage handle owning a single [`Payload`]. The bindings expose it as
/// an opaque pointer/handle (`storage_t *` in C, a `Storage` class in Kotlin); its
/// fields are never inspected across the FFI boundary — the adapter boxes it and
/// emits a typed destructor. (Not `#[prebindgen]` and not `#[repr(C)]`: it is a
/// boxed handle, like `Calculator` in `example-flat`.)
#[derive(Default)]
pub struct Storage {
    payload: Payload,
}

/// Create a new, empty storage handle.
#[prebindgen]
pub fn storage_new() -> Storage {
    Storage::default()
}

/// Return a clone of the stored payload (by value; crosses by reinterpret, the
/// `label` becoming a fresh owned `string_t *` the C caller must drop).
#[prebindgen]
pub fn storage_get(s: &Storage) -> Payload {
    s.payload.clone()
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
    s.payload = payload;
}

/// Store a clone of `payload`, read through a shared borrow (`const payload_t *`).
/// The caller's payload is left untouched.
#[prebindgen]
pub fn storage_put_by_read(s: &mut Storage, payload: &Payload) {
    s.payload = payload.clone();
}

/// Store a clone of `payload`, then **update the caller's payload in place** by
/// bumping its `seq` counter (a `&mut Payload` read/write borrow → `payload_t *`).
#[prebindgen]
pub fn storage_put_by_read_and_update(s: &mut Storage, payload: &mut Payload) {
    s.payload = payload.clone();
    payload.seq += 1;
}

/// Write the stored payload into the caller's **already-initialized** `payload` slot.
/// The assignment drops the old value first (freeing its old `label`) — so the slot
/// must hold a valid payload (use [`storage_get_into_uninit`] for raw memory).
#[prebindgen]
pub fn storage_get_into_init(s: &Storage, payload: &mut Payload) {
    *payload = s.payload.clone();
}

/// Write the stored payload into the caller's **uninitialized** `payload` slot,
/// without dropping whatever bytes were there (`&mut MaybeUninit<Payload>` →
/// `payload_t *`). The slot is initialized afterwards.
#[prebindgen]
pub fn storage_get_into_uninit(s: &Storage, payload: &mut MaybeUninit<Payload>) {
    payload.write(s.payload.clone());
}

/// Invoke `f` with a borrow of the stored payload. In C this is a pure zero-copy
/// struct crossing (the closure receives a `const payload_t *`, no heap work). In
/// Kotlin the borrowed `Payload` is delivered whole to a generated
/// `PayloadCallback.run(Payload)` (its fields cross as decoupled leaves and are
/// reassembled on the Kotlin side — see `prebindgen::lang::JniGen`).
#[prebindgen]
pub fn storage_callback(s: &Storage, f: impl Fn(&Payload) + Send + Sync + 'static) {
    f(&s.payload);
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
