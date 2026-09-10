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

/// A record with a positional field, and one with a scalar neither adapter
/// carries: both are here so that a target refusing them is visible in the
/// report rather than only in a comment.
pub struct Pair(pub i64, pub i64);

pub struct Reading {
    pub level: i32,
}

/// A function that delivers nothing, so a wrapper with no native return — and,
/// through JNI, a failure route that must terminate without a value — is
/// compiled rather than described.
pub fn stamp_show(stamp: Stamp) {
    let _ = stamp;
}

/// Not part of the specification's fixture, and here for one reason: addition
/// is commutative, so a wrapper that read the two fields into the wrong
/// arguments would still compute `stamp_sum` correctly. Subtraction says which
/// field went where.
pub fn stamp_delta(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_sub(stamp.nanos)
}
