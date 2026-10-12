//! The flat surface under test: one field per **Rust spelling** of a shape the
//! model already reads representation-agnostically.
//!
//! This module is ordinary compiled Rust, and `build.rs` parses this very file
//! to build the flat model — so a shape added here is emitted, compiled, and
//! called against the same definitions, with nothing restated. Bodies are
//! `unimplemented!()`: the question is whether the *emitted* code type-checks
//! against these signatures, not what they compute.
//!
//! The generated bindings qualify every call as `myflat::…` (build.rs stamps
//! that crate name onto each item, exactly as the unit-test fixtures do), which
//! is why `lib.rs` mounts this file under that name.

// The shapes are chosen for the generator, not for idiomatic Rust — nobody
// writes a `Box<Option<T>>` field by hand.
#![allow(clippy::box_collection, clippy::redundant_allocation)]

use std::borrow::Cow;

/// An opaque handle. Every value-form field below reaches one of these, or a
/// leaf.
///
/// The field is not decoration: a handle crosses as a `jlong` whose bit 0 is
/// the closed tag, so the generated code asserts `align_of::<T>() >= 2` and a
/// zero-sized handle fails that at compile time.
pub struct ZKeyExpr(pub u64);

/// The subject whose value form carries the spellings.
pub struct ZSample(pub u64);

/// The child boundary: a `ZKeyExpr` field must still cross through *this*,
/// because the field's own type decides how it crosses — not the wrapper the
/// value form happens to spell it with.
pub fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str {
    let _ = k;
    unimplemented!()
}

/// The value form, one field per spelling. `prebindgen-flat`'s `acceptance.rs`
/// pins that each pair below *reads* the same; the emitted access for each one
/// is what this crate compiles.
///
/// The pairs, in the order the model's own acceptance test lists them:
///
/// | shape    | plain            | wrapped                |
/// |----------|------------------|------------------------|
/// | optional | `Option<T>`      | `Box<Option<T>>` (#268)|
/// | sequence | `Vec<u8>`        | `Cow<'_, [u8]>`        |
/// | `Str`    | `String`         | `Box<String>`, `Cow<'_, str>` |
///
/// `Box<Option<T>>` is the shape of #268 specifically: match ergonomics does
/// not deref a `Box`, so an access that destructures the raw place is `E0308`
/// here and compiles one field up.
pub struct ZSampleStruct {
    pub opt_plain: Option<ZKeyExpr>,
    pub opt_boxed: Box<Option<ZKeyExpr>>,
    pub seq_plain: Vec<u8>,
    pub seq_cow: Cow<'static, [u8]>,
    pub text_plain: String,
    pub text_boxed: Box<String>,
    pub text_cow: Cow<'static, str>,
}

pub fn z_sample_to_struct(s: &ZSample) -> ZSampleStruct {
    let _ = s;
    unimplemented!()
}

// The fourth spelling the model's acceptance test pins — a run of values
// spelled **bare**, `[T]` — is deliberately absent, and cannot be added. It is
// unsized, so it is not a struct field, not a return, and not a by-value
// callback argument: there is no signature carrying it that is itself valid
// Rust. A spelling that no compilable source can hold has no emitted access to
// compile, which is why `prebindgen-flat`'s acceptance test is the only place
// it can be pinned.

/// The one output position: `ZSample` crossing into this callback is what makes
/// the value form above get reached, and so emitted.
pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
    let _ = cb;
    unimplemented!()
}
