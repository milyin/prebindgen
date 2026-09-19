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

/// A tally of stamps, reached through a handle. The type behind the alias is
/// crate-private and unmarked, so a binding can hold one and call what takes
/// it, and nothing else.
pub type Ledger = crate::ledger::Ledger;

pub fn ledger_open(stamp: Stamp) -> Ledger {
    crate::ledger::Ledger {
        total: stamp_sum(stamp),
    }
}

pub fn ledger_close(ledger: Ledger) -> i64 {
    ledger.total
}

/// A struct with a positional field, and one with a scalar neither adapter
/// carries: both are here so that a target refusing them is visible in the
/// report rather than only in a comment.
pub struct Pair(pub i64, pub i64);

pub struct Reading {
    pub level: i32,
}

/// A struct with no fields at all. The model keeps it as a struct, so it
/// reaches both adapters — and neither an empty `repr(C)` aggregate nor a
/// Kotlin data class with no properties is a thing that exists.
pub struct Marker;

pub fn marker_value(marker: Marker) -> i64 {
    let _ = marker;
    7
}

/// A function that delivers nothing, so a wrapper with no return — and,
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

/// A struct with a field written under a condition nothing in this build can
/// answer, and a function taking it.
///
/// `v2check_conditional_field` is never set, so `extra` is absent here — and
/// the point is that the generated mirror, the read of its member and the
/// initializer that fills it are absent with it. Setting the flag compiles the
/// other half. Either way this crate builds, which is what a wrapper naming a
/// field the source does not have could not do.
pub struct Sample {
    pub level: i64,
    #[cfg(v2check_conditional_field)]
    pub extra: i64,
}

pub fn sample_total(sample: Sample) -> i64 {
    sample.level
}

/// A function written under a condition nothing in this build can answer.
///
/// `v2check_conditional_fn` is never set, so neither this nor the wrapper
/// generated for it is compiled — and the Kotlin declared for it is, because
/// Kotlin cannot say what `#[cfg]` says. What the JNI writer can do is put the
/// condition in the function's documentation, which is what this checks.
#[cfg(v2check_conditional_fn)]
pub fn stamp_ratio(stamp: Stamp) -> i64 {
    stamp.secs / stamp.nanos.max(1)
}
