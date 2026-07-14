//! Binding-side conversion helpers for `covertest-kotlin` — a **separate
//! `#[prebindgen]` source crate** layered on top of [`perftest_flat`].
//!
//! This crate exists to prove the multi-source model: a binding crate's
//! `build.rs` chains SEVERAL prebindgen source streams into one registry
//! (`Registry::from_items(flat.items_all().chain(helpers.items_all()))`) and
//! the generated Rust qualifies each function with its origin crate.
//! covertest-kotlin additionally RENAMES this dependency
//! (`cov_helpers = { package = "covertest-helpers", .. }`) and overrides the
//! stamped origin via `Source::builder(..).crate_name("cov_helpers")` — so
//! generated calls read `perftest_flat::…` vs `cov_helpers::…`, proving the
//! per-source rename escape hatch.
//!
//! Why a separate crate at all: `#[prebindgen]` markers are captured during a
//! crate's own compilation, which happens *after* its `build.rs` runs — so a
//! binding crate can never feed its own markers into its own generation.
//! Helper conversions therefore live either in the flat crate itself or in a
//! small helper crate like this one, consumed as both a normal and a build
//! dependency (exactly like the flat crate).

use perftest_flat::Millis;
use prebindgen_proc_macro::{features, prebindgen};

/// Output directory with the prebindgen data of this crate.
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_proc_macro::prebindgen_out_dir!();

/// Enabled Cargo features of this crate (referenced by the generated
/// `konst` feature guard, like every prebindgen source crate).
pub const FEATURES: &str = features!();

/// Canonical input conversion for [`Millis`]: build it from the raw
/// millisecond count. Referenced by covertest's
/// `convert!(Millis).input(fun!(millis_from_long))`.
#[prebindgen]
pub fn millis_from_long(v: i64) -> Millis {
    Millis(v as u64)
}

/// Canonical output conversion for [`Millis`]: read the raw millisecond
/// count. Referenced by covertest's
/// `convert!(Millis).output(fun!(millis_value))`.
#[prebindgen]
pub fn millis_value(m: &Millis) -> i64 {
    m.0 as i64
}
