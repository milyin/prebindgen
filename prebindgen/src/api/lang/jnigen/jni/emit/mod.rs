//! JNI `extern "C"` wrapper and converter-body emission (free fns).
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

// ──────────────────────────────────────────────────────────────────────
// Function-wrapper emission (JNI extern "C")
// ──────────────────────────────────────────────────────────────────────

use super::*;

mod callback;
mod convert;
mod delivery;
mod flat_input;
mod names;
mod struct_out;
mod vec_build;
mod wrapper;

pub(crate) use callback::*;
pub(crate) use convert::*;
pub(crate) use delivery::*;
pub(crate) use flat_input::*;
pub(crate) use names::*;
pub(crate) use struct_out::*;
pub(crate) use vec_build::*;
pub(crate) use wrapper::*;
