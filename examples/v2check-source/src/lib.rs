//! The source crate of `docs/v2`, compiled for real.
//!
//! `v2check/build.rs` parses this file and hands its items to Flat, exactly
//! as the chapters' capture stage would; nothing here knows about C or
//! Kotlin.
//!
//! A crate of its own, rather than a module of the binding: that is the
//! arrangement every real binding has, and some Rust only says no across a
//! crate boundary. A `#[non_exhaustive]` enum cannot be matched from here
//! without a wildcard arm, and a value of one cannot be named at all — which
//! a source module in the same crate would accept, and a compiled binding
//! would not.

pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}

/// A tally of stamps, reached through a handle. The type behind the alias is
/// unmarked, so a binding can hold one and call what takes it, and nothing
/// else.
pub type Ledger = crate::ledger::Ledger;

pub fn ledger_open(stamp: Stamp) -> Ledger {
    crate::ledger::Ledger {
        total: stamp_sum(stamp),
    }
}

pub fn ledger_close(ledger: Ledger) -> i64 {
    ledger.total
}

/// A fieldless enum, and one whose values carry the delimiters of a
/// constructor without carrying anything through them.
///
/// `Mul` numbers itself and `Add` takes the number before it, so a mirror has
/// to read both from the model rather than counting. `Twice()` and `Halve {}`
/// are fieldless too — the model says so — and every mention of them in
/// generated Rust has to keep its delimiters, which is what rustc checks
/// here by compiling this crate against what was generated from it.
pub enum Operation {
    Add,
    Mul = 7,
}

pub fn operation_flip(op: Operation) -> Operation {
    match op {
        Operation::Add => Operation::Mul,
        Operation::Mul => Operation::Add,
    }
}

pub enum Adjust {
    Twice(),
    Halve {},
}

pub fn adjust_invert(adjust: Adjust) -> Adjust {
    match adjust {
        Adjust::Twice() => Adjust::Halve {},
        Adjust::Halve {} => Adjust::Twice(),
    }
}

/// An enum only ever passed in. Nothing the C binding exports returns one, so
/// the header declares it only if the parameter that takes it is typed as it.
pub enum Gear {
    Low,
    High = 3,
}

pub fn gear_rank(gear: Gear) -> i64 {
    match gear {
        Gear::Low => 1,
        Gear::High => 30,
    }
}

/// An enum another crate may not match without an arm for a value it does not
/// know, and one whose values another crate may not name at all.
///
/// Both are fieldless, and both are refused: there is nothing for a wildcard
/// arm to produce going out of Rust, and a `#[non_exhaustive]` value cannot be
/// constructed from here — a unit value included, whose constructor is private
/// outside this crate. They are declared so that a target accepting one is a
/// failure of this crate to compile rather than a comment nobody reads.
#[non_exhaustive]
pub enum Sweep {
    Wide,
    Narrow = 3,
}

pub enum Detent {
    #[non_exhaustive]
    Soft,
    #[non_exhaustive]
    Hard(),
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

/// What [`Ledger`] is an alias of: a type this crate never marks, which is
/// the reason the alias exists.
pub(crate) mod ledger {
    pub struct Ledger {
        pub total: i64,
    }
}
