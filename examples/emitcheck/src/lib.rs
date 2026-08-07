//! Compiles the JNI bindings `build.rs` emits for [`myflat`], and does nothing
//! else — see `build.rs` for why (#269).
//!
//! There is no test here on purpose. Success *is* `cargo build`: if the
//! generated file does not type-check against `myflat`'s definitions, this
//! crate fails to compile, and CI's `cargo build` at the workspace root is
//! already the gate.

// Generator findings belong to the generator, not to this file.
#![allow(clippy::all)]

// Mounted under the name the generated code qualifies its calls with
// (`build.rs`'s `SOURCE_CRATE`).
pub mod myflat;

include!("generated_bindings.rs");
