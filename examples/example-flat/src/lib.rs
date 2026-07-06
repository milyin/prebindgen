//! Flat, FFI-friendly example library — a miniature in the style of `zenoh-flat`.
//!
//! Every public function is annotated with `#[prebindgen]`, so `prebindgen`
//! captures this surface and a language adapter (here `prebindgen::lang::Cbindgen`,
//! driven by `example-cbindgen`) generates the FFI layer — no hand-written
//! `extern "C"` glue, and **no `#[repr(C)]`** in this crate.
//!
//! The API is plain idiomatic Rust:
//!
//! - [`Calculator`] is an opaque handle returned **by value**; the adapter boxes it
//!   and emits a typed `calculator_drop`.
//! - [`Error`] is a boxed `std` error rendered to a message by [`error_get_message`];
//!   fallible calls return `Result<T, Error>`.
//! - [`Operation`] is a primitive-repr enum (`#[repr(i32)]`, like zenoh-flat's
//!   `Priority`).
//! - Items are delivered to a C closure through an `impl Fn(..)` callback
//!   ([`calculator_for_each`]).
//!
//! Function names encode their receiver and role: `calculator_new*` construct,
//! `calculator_get_*` read, `calculator_to_string` converts.

use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};

/// Path to the directory where the `#[prebindgen]` macro records this crate's FFI
/// surface; read by consumers via `prebindgen::Source::new`.
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
/// The features enabled when this crate was built; consumers verify their own
/// feature set against it.
pub const FEATURES: &str = features!();

/// Boxed error type, mirroring zenoh-flat's `Error`. It is the `E` of every
/// fallible `Result` and never crosses the FFI boundary as a value; the adapter
/// marshals it to C as a `char*` message obtained from [`error_get_message`].
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Render an error as its display string. Wired into the C adapter as the
/// `opaque_error` message function.
#[prebindgen]
pub fn error_get_message(e: &Error) -> String {
    e.to_string()
}

/// Arithmetic operation selector — a primitive-repr enum (like zenoh-flat's
/// `Priority`); the adapter lowers it to a C enum.
#[prebindgen]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}

/// A stateful accumulator. This is a plain Rust type used as an opaque handle:
/// the binding holds it behind a pointer and frees it with `calculator_drop`.
pub struct Calculator {
    value: f64,
    history: Vec<f64>,
}

/// Build a fresh accumulator initialized to zero.
#[prebindgen]
pub fn calculator_new() -> Calculator {
    Calculator {
        value: 0.0,
        history: Vec::new(),
    }
}

/// Parse an initial value from a string, returning an error on bad input
/// (demonstrates a `&str` input plus `Result` error routing).
#[prebindgen]
pub fn calculator_new_from_str(s: &str) -> Result<Calculator, Error> {
    let value: f64 = s.parse().map_err(|e| format!("parse error: {e}"))?;
    Ok(Calculator {
        value,
        history: vec![value],
    })
}

/// Clone an accumulator handle. Use before passing one to a consuming call when
/// the caller needs to keep the original.
#[prebindgen]
pub fn calculator_new_clone(c: &Calculator) -> Calculator {
    Calculator {
        value: c.value,
        history: c.history.clone(),
    }
}

/// Apply `op` with `operand`, updating the accumulator and returning the new
/// value. Division by zero returns an error (its fallible `&mut` input routes
/// through the error channel of the `Result`).
#[prebindgen]
pub fn calculator_apply(c: &mut Calculator, op: Operation, operand: f64) -> Result<f64, Error> {
    let next = match op {
        Operation::Add => c.value + operand,
        Operation::Sub => c.value - operand,
        Operation::Mul => c.value * operand,
        Operation::Div => {
            if operand == 0.0 {
                return Err("division by zero".to_string().into());
            }
            c.value / operand
        }
    };
    c.value = next;
    c.history.push(next);
    Ok(next)
}

/// The current accumulator value.
#[prebindgen]
pub fn calculator_get_value(c: &Calculator) -> f64 {
    c.value
}

/// How many operations have been applied so far.
#[prebindgen]
pub fn calculator_get_count(c: &Calculator) -> u64 {
    c.history.len() as u64
}

/// Whether the accumulator currently holds exactly `value`.
#[prebindgen]
pub fn calculator_is(c: &Calculator, value: f64) -> bool {
    c.value == value
}

/// Render the accumulator as an owned string (`char*` to C, freed by the
/// adapter's `example_free`).
#[prebindgen]
pub fn calculator_to_string(c: &Calculator) -> String {
    format!("Calculator({})", c.value)
}

/// Copy the recorded history out as an array.
#[prebindgen]
pub fn calculator_get_history(c: &Calculator) -> Vec<f64> {
    c.history.clone()
}

/// Invoke `f` once per recorded value in application order — replays the history
/// into a C closure (demonstrates callback / closure-struct generation).
#[prebindgen]
pub fn calculator_for_each(c: &Calculator, f: impl Fn(f64) + Send + Sync + 'static) {
    for v in &c.history {
        f(*v);
    }
}

/// Reset the accumulator to zero (feature-gated, mirroring zenoh-flat's
/// `unstable` slices of the API).
#[cfg(feature = "unstable")]
#[prebindgen(cfg = "feature = \"unstable\"")]
pub fn calculator_reset(c: &mut Calculator) {
    c.value = 0.0;
    c.history.clear();
}

// ─────────────────────────────────────────────────────────────────────────────
// Multi-target cfg demonstration.
//
// These items show that `#[prebindgen]` captures per-target `cfg` and that the C
// binding crate (`example-cbindgen`) then generates *different* code per target:
// `InsideFoo`'s discriminants and `Foo`'s field set change with `target_arch`
// (and `Foo` also varies by feature). Build for x86_64 vs aarch64 to get two
// different `inside_foo_t` / `foo_t` in the generated header.
// ─────────────────────────────────────────────────────────────────────────────

/// A fieldless enum whose **discriminants differ by target architecture**. The two
/// definitions are mutually exclusive — the `#[prebindgen(cfg = ...)]` macro emits a
/// matching real `#[cfg]`, so each target compiles exactly one and the generated C
/// enum carries that target's values. (`lang::Cbindgen` `.enum_type`.)
#[prebindgen("structs", cfg = "target_arch = \"x86_64\"")]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideFoo {
    DouddleDee = 42,
    DouddleDum = 24,
}
#[prebindgen("structs", cfg = "target_arch = \"aarch64\"")]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsideFoo {
    DouddleDee = 14,
    DouddleDum = 88,
}

/// A by-value data struct whose **field set varies by target architecture and by
/// feature**. `#[prebindgen]` records every `cfg`-gated field; the binding crate
/// keeps only those matching the build target, so the generated `#[repr(C)] foo_t`
/// differs per target. (`lang::Cbindgen` `.data_struct`.)
#[prebindgen("structs")]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Foo {
    /// Always present.
    pub id: u64,
    #[cfg(target_arch = "x86_64")]
    pub x86_64_field: u64,
    #[cfg(target_arch = "aarch64")]
    pub aarch64_field: u64,
    #[cfg(feature = "unstable")]
    pub unstable_field: u64,
    #[cfg(not(feature = "unstable"))]
    pub stable_field: u64,
}

/// Construct a `Foo` (the target-specific fields default to zero).
#[prebindgen]
pub fn foo_new(id: u64) -> Foo {
    Foo {
        id,
        ..Foo::default()
    }
}

/// Read a `Foo`'s always-present field (consumes the value-struct by value).
#[prebindgen]
pub fn foo_get_id(f: Foo) -> u64 {
    f.id
}

/// The default `InsideFoo` variant (its numeric value is target-specific).
#[prebindgen]
pub fn inside_foo_default() -> InsideFoo {
    InsideFoo::DouddleDee
}

/// The numeric value of an `InsideFoo` (consumes the enum by value).
#[prebindgen]
pub fn inside_foo_value(x: InsideFoo) -> i32 {
    x as i32
}
