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
//! The three functions operate on a thread-local slot, so the matching Rust and C
//! micro-benchmarks (`examples/perftest.rs` and `perftest-c/c/perftest.c`) measure
//! the cost of the same operations natively vs across the generated C ABI.

use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
use std::cell::RefCell;

/// Path to the directory where the `#[prebindgen]` macro records this crate's FFI
/// surface; read by consumers via `prebindgen::Source::new`.
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
/// The features enabled when this crate was built.
pub const FEATURES: &str = features!();

/// A by-value, FFI-safe payload. Scalars cross the C ABI as themselves; the
/// `label` string crosses as an opaque pointer (`Option<Box<String>>` ⇒ a nullable
/// `string_t *`). Being `#[repr(C)]`, the whole struct is passed by direct
/// reinterpret (zero-copy) — see `perftest-c`'s `.repr_c_struct(Payload)`.
#[prebindgen]
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct Payload {
    pub id: u64,
    pub seq: u32,
    pub value: f64,
    pub flag: bool,
    pub label: Option<Box<String>>,
}

thread_local! {
    static STORED: RefCell<Payload> = RefCell::new(Payload::default());
}

/// Return a clone of the thread-local payload (by value; crosses by reinterpret,
/// the `label` becoming a fresh owned `string_t *` the C caller must drop).
#[prebindgen]
pub fn payload_get() -> Payload {
    STORED.with(|s| s.borrow().clone())
}

/// Store a copy of `p` into the thread-local slot. The `&Payload` borrow crosses as
/// a zero-copy `const payload_t *` pointer; the clone-into-storage happens in Rust.
#[prebindgen]
pub fn payload_put(p: &Payload) {
    STORED.with(|s| *s.borrow_mut() = p.clone());
}

/// Invoke `f` with a borrow of the stored payload — a pure zero-copy struct
/// crossing (the C closure receives a `const payload_t *`, no heap work).
#[prebindgen]
pub fn payload_callback(f: impl Fn(&Payload) + Send + Sync + 'static) {
    STORED.with(|s| f(&s.borrow()));
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
