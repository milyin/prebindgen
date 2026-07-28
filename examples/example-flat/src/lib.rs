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

/// A **data-carrying enum** (a sum type): exactly one alternative is live, and it
/// carries that alternative's payload. Written as plain Rust — the invariant is in
/// the type, not in a doc comment on a struct of optional fields.
///
/// The four variant shapes a sum can take are all here: a unit variant, a
/// single-payload tuple variant, a multi-field named variant, and a tuple variant
/// with an **owning** payload (a `String`, which crosses to C as a malloc'd
/// `char *`) beside a payload that is itself a declared enum. The C adapter
/// lowers the whole thing to a `#[repr(C)]` enum, which cbindgen renders as the
/// idiomatic tag + `union`. (`lang::Cbindgen` `.tagged_union`.)
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Unit variant: the empty payload group — only the tag is live.
    Empty,
    /// Single-payload tuple variant.
    Circle(f64),
    /// Multi-field named variant.
    Rect { width: f64, height: f64 },
    /// Owning payload: the `String` is handed to C as a malloc'd `char *` that the
    /// generated `shape_drop` releases, beside a declared-`enum_type` payload.
    Labeled(String, Operation),
}

/// A by-value data struct used as a **union payload** — the shape zenoh-flat's
/// `ReplyResult` needs, where an alternative carries a whole record rather than a
/// scalar. Its `String` field owns memory, so the union's typed drop has to reach
/// through the payload and release it.
///
/// The `bool` field is the second half of #170: a `data_struct` field crosses by
/// value through a per-field wire, so a byte C wrote has to be normalised there
/// too — and because this struct is also a union payload, that is what makes the
/// tagged union's "no invalid value is ever materialised" invariant hold
/// *transitively*, one level down.
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Caption {
    pub id: u64,
    pub text: String,
    pub emphatic: bool,
}

/// Build a [`Caption`].
#[prebindgen]
pub fn caption_new(id: u64, text: &str, emphatic: bool) -> Caption {
    Caption {
        id,
        text: text.to_string(),
        emphatic,
    }
}

/// A sum whose payloads are the shapes the zero-copy mirror policy could not
/// express: a nested `data_struct` (by value, owning a `char *`) and a
/// **converted leaf** whose wire is its converter's destination rather than its
/// own layout.
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    /// No payload — only the tag.
    Silent,
    /// A whole record by value, owning a `char *`.
    Titled(Caption),
    /// A converted leaf: `Millis` crosses as the `u64` its conversion produces.
    After(Millis),
    /// A `bool` payload — the one scalar whose domain is restricted (`0`/`1`),
    /// so it crosses behind `MaybeUninit` like a declared enum does and a byte
    /// C wrote is normalised, never materialised as a Rust `bool`.
    Flagged(bool),
    /// A record whose own field is ANOTHER union with an owning arm. The
    /// payload crosses by value, so `note_drop` has to reach through it and
    /// release the nested union's active arm — nothing else can: a union arm
    /// is not a top-level struct field the C caller drops by hand.
    Sketched(Drawing),
}

/// A newtype whose C wire is the `u64` its declared conversion produces — not
/// its own layout. As a union payload it exercises the converter-destination
/// rule directly.
#[prebindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millis(pub u64);

/// `Millis` from its raw value (the conversion's **input**).
#[prebindgen]
pub fn millis_from_raw(v: u64) -> Millis {
    Millis(v)
}

/// `Millis` to its raw value (the conversion's **output**).
#[prebindgen]
pub fn millis_to_raw(v: &Millis) -> u64 {
    v.0
}

/// A titled note (nested-struct payload).
#[prebindgen]
pub fn note_new_titled(id: u64, text: &str, emphatic: bool) -> Note {
    Note::Titled(caption_new(id, text, emphatic))
}

/// A delayed note (converted-leaf payload).
#[prebindgen]
pub fn note_new_after(millis: u64) -> Note {
    Note::After(Millis(millis))
}

/// The silent note.
#[prebindgen]
pub fn note_new_silent() -> Note {
    Note::Silent
}

/// A flagged note (`bool` payload).
#[prebindgen]
pub fn note_new_flagged(flag: bool) -> Note {
    Note::Flagged(flag)
}

/// A sketched note: a record payload whose own `shape` field is another union,
/// with an arm that owns a `char *`.
#[prebindgen]
pub fn note_new_sketched(id: u64, label: &str) -> Note {
    Note::Sketched(Drawing {
        id,
        shape: Shape::Labeled(label.to_string(), Operation::Add),
    })
}

/// Read a note back: its caption id, its delay, or 0 — the sum in **parameter**
/// position, so both new payload kinds cross inbound as well as outbound.
#[prebindgen]
pub fn note_value(n: Note) -> u64 {
    match n {
        Note::Silent => 0,
        Note::Titled(c) => c.id,
        Note::After(Millis(m)) => m,
        Note::Flagged(f) => f as u64,
        Note::Sketched(d) => d.id,
    }
}

/// Whether a titled note's caption is emphatic. The observable of #170's
/// second instance: the `bool` rides in as a `data_struct` field of a union
/// payload, two per-field wires down from the C caller's bytes.
#[prebindgen]
pub fn note_emphatic(n: Note) -> bool {
    match n {
        Note::Titled(c) => c.emphatic,
        _ => false,
    }
}

/// A by-value data struct carrying a sum as a **field** — the position that makes
/// "exactly one of" compose with ordinary product data.
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Drawing {
    pub id: u64,
    pub shape: Shape,
}

/// The empty shape (the unit variant).
#[prebindgen]
pub fn shape_new_empty() -> Shape {
    Shape::Empty
}

/// A circle of the given radius (single-payload tuple variant).
#[prebindgen]
pub fn shape_new_circle(radius: f64) -> Shape {
    Shape::Circle(radius)
}

/// A rectangle (multi-field named variant).
#[prebindgen]
pub fn shape_new_rect(width: f64, height: f64) -> Shape {
    Shape::Rect { width, height }
}

/// A labeled shape (owning `String` payload beside an enum payload).
#[prebindgen]
pub fn shape_new_labeled(label: &str, op: Operation) -> Shape {
    Shape::Labeled(label.to_string(), op)
}

/// Area of a shape — the sum in **parameter** position, consumed by value. Every
/// alternative is handled; there is no "both set" or "neither set" case to guard.
#[prebindgen]
pub fn shape_area(s: Shape) -> f64 {
    match s {
        Shape::Empty => 0.0,
        Shape::Circle(r) => std::f64::consts::PI * r * r,
        Shape::Rect { width, height } => width * height,
        Shape::Labeled(_, _) => f64::NAN,
    }
}

/// Area of a shape, reporting the alternative that has none through the error
/// channel — the sum in parameter position on a **fallible** function. The C
/// binding routes a rejected tag (a discriminant no variant has, which a C
/// caller can always supply) to this same `char **e`, so the boundary check is
/// observable instead of aborting.
#[prebindgen]
pub fn shape_try_area(s: Shape) -> Result<f64, Error> {
    match s {
        Shape::Labeled(_, _) => Err("a labeled shape has no area".to_string().into()),
        other => Ok(shape_area(other)),
    }
}

/// The label of a `Labeled` shape, or the empty string for any other alternative.
#[prebindgen]
pub fn shape_get_label(s: Shape) -> String {
    match s {
        Shape::Labeled(label, _) => label,
        _ => String::new(),
    }
}

/// Wrap a shape into a drawing (the sum crossing back out as a struct field).
#[prebindgen]
pub fn drawing_new(id: u64, shape: Shape) -> Drawing {
    Drawing { id, shape }
}

/// Take a drawing's shape back out (the sum crossing in as a struct field).
#[prebindgen]
pub fn drawing_get_shape(d: Drawing) -> Shape {
    d.shape
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

/// Fold one accumulator into another, consuming **both**.
///
/// Two consumed handles of the same type is a supported shape, and it is the
/// aliasing hazard: called as `calculator_merge(x, x)` a C caller would have
/// one allocation reconstructed twice, so the generated wrapper runs a
/// pointer-identity **preflight** before either conversion and reports the
/// alias through this `Result`'s error channel.
#[prebindgen]
pub fn calculator_merge(a: Calculator, b: Calculator) -> Result<Calculator, Error> {
    let mut history = a.history;
    history.extend(b.history);
    Ok(Calculator {
        value: a.value + b.value,
        history,
    })
}

/// Consume one accumulator while **borrowing** another — the mixed case the
/// "two or more consumed parameters" reading of the rule would have skipped.
/// The borrow dangles the moment the consume takes ownership, so the same
/// preflight covers it.
#[prebindgen]
pub fn calculator_absorb(a: Calculator, b: &Calculator) -> Result<f64, Error> {
    Ok(a.value + b.value)
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

/// Width of [`Marker::tag`] — a `#[prebindgen]` const used as an ARRAY EXTENT.
///
/// The value must be a plain integer literal: a generator runs in `build.rs`,
/// where it cannot evaluate Rust, and the C header needs the number to define
/// the symbol. Being `#[prebindgen]` is what makes it visible at all — the
/// generated crate sees only what the macro exposed.
#[prebindgen]
pub const MARKER_TAG_LEN: usize = 4;

/// A data struct with a fixed-size array field, sized by a NAMED const.
///
/// The const spelling is the point. `tag` must surface in C as
/// `uint8_t tag[MARKER_TAG_LEN]`, not `uint8_t tag[4]`: a symbolic extent is
/// part of the API's meaning, because it makes changing the size one edit
/// rather than a hunt through literals. Carrying that fact from the source to
/// the header is what the frontend's typed model exists for — the adapter reads
/// the extent as a decided fact, never by re-parsing Rust syntax.
///
/// `[u8; N]` is already `#[repr(C)]`-compatible, so the array is its own C wire
/// and the field copies with no conversion. An array of `bool` deliberately is
/// NOT supported: its domain is `0`/`1`, and a mirror is reinterpreted wholesale
/// with no per-element hook to normalise what C wrote.
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub tag: [u8; MARKER_TAG_LEN],
    pub weight: u32,
}

/// Build a [`Marker`] from its bytes.
#[prebindgen]
pub fn marker_new(b0: u8, b1: u8, b2: u8, b3: u8, weight: u32) -> Marker {
    Marker {
        tag: [b0, b1, b2, b3],
        weight,
    }
}

/// Sum a [`Marker`]'s tag bytes (consumes it by value, so the array crosses
/// both ways).
#[prebindgen]
pub fn marker_tag_sum(m: Marker) -> u32 {
    m.tag.iter().map(|b| *b as u32).sum::<u32>() + m.weight
}
