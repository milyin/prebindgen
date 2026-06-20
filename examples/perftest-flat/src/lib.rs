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
//! `Storage` class in Kotlin) that owns the payload, so the matching Rust and C
//! micro-benchmarks (`examples/perftest.rs` and `perftest-c/c/perftest.c`) measure
//! the cost of the same operations natively vs across the generated C ABI — and
//! exercise an opaque handle crossing alongside the value struct.

use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};

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

/// Store a copy of `p` into the storage. The `&Payload` borrow crosses as a
/// zero-copy `const payload_t *` pointer; the clone-into-storage happens in Rust.
#[prebindgen]
pub fn storage_put(s: &mut Storage, p: &Payload) {
    s.payload = p.clone();
}

/// Invoke `f` with a borrow of the stored payload — a pure zero-copy struct
/// crossing (the C closure receives a `const payload_t *`, no heap work).
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

// ─────────────────────────────────────────────────────────────────────────────
// Per-field getters reading one field of a storage's payload — the "naive"
// baseline for the Kotlin benchmark: fetching all fields takes N separate FFI
// crossings, vs the single crossing of `storage_get` (which composes the whole
// struct on the foreign side).
// ─────────────────────────────────────────────────────────────────────────────

/// `id` of the stored payload.
#[prebindgen]
pub fn storage_get_id(s: &Storage) -> i64 {
    s.payload.id
}

/// `seq` of the stored payload.
#[prebindgen]
pub fn storage_get_seq(s: &Storage) -> i32 {
    s.payload.seq
}

/// `value` of the stored payload.
#[prebindgen]
pub fn storage_get_value(s: &Storage) -> f64 {
    s.payload.value
}

/// `flag` of the stored payload.
#[prebindgen]
pub fn storage_get_flag(s: &Storage) -> bool {
    s.payload.flag
}

/// `label` of the stored payload (an owned copy; `None` when unset).
#[prebindgen]
pub fn storage_get_label(s: &Storage) -> Option<String> {
    s.payload.label.as_deref().map(|s| s.to_string())
}
