//! The source crate of `docs/v2`, compiled for real.
//!
//! `build.rs` parses this file and hands its items to Flat, exactly as the
//! chapters' capture stage would; nothing here knows about C or Kotlin.

pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}

/// Not part of the specification's fixture, and here for one reason: addition
/// is commutative, so a wrapper that read the two fields into the wrong
/// arguments would still compute `stamp_sum` correctly. Subtraction says which
/// field went where.
pub fn stamp_delta(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_sub(stamp.nanos)
}
