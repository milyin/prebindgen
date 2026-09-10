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

/// A record with a positional field, and one with a scalar the JVM cannot
/// carry: both are here so that a target refusing them is visible in the
/// report rather than only in a comment.
pub struct Pair(pub i64, pub i64);

pub struct Reading {
    pub level: usize,
}

/// A record with no fields at all. The model keeps it as a record, so it
/// reaches both adapters — and neither an empty `repr(C)` aggregate nor a
/// Kotlin data class with no properties is a thing that exists.
pub struct Marker;

pub fn marker_value(marker: Marker) -> i64 {
    let _ = marker;
    7
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

/// Every scalar both C and the JVM carry, in one record, so that each one is
/// planned as an aggregate member as well as on its own.
///
/// `usize`/`isize` are not here: the JVM has no type of the platform's width.
/// [`size_shift`] carries them where only C is asked.
pub struct Scalars {
    pub flag: bool,
    pub tiny: i8,
    pub small: i16,
    pub medium: i32,
    pub large: i64,
    pub byte: u8,
    pub word: u16,
    pub dword: u32,
    pub qword: u64,
    pub single: f32,
    pub double: f64,
}

/// A scalar that needs no conversion in either direction: the carrier is the
/// Rust value itself.
pub fn scalars_large(scalars: Scalars) -> i64 {
    scalars.large
}

/// A `bool` in both directions: it crosses C as storage rather than as itself,
/// and the JVM's `jboolean` is a `u8`.
pub fn scalars_flag(scalars: Scalars) -> bool {
    !scalars.flag
}

/// An unsigned result, which the JVM has no type for: it rides home in the
/// signed carrier of the same width.
pub fn scalars_qword(scalars: Scalars) -> u64 {
    scalars.qword
}

/// A float result, so a carrier that is neither an integer nor `i64` is
/// returned as well as read.
pub fn scalars_double(scalars: Scalars) -> f64 {
    scalars.double
}

/// Every narrower integer, summed in a width the sum cannot overflow — so a
/// wrapper that read a member at the wrong width would be caught by the value,
/// not only by the compiler.
pub fn scalars_narrow(scalars: Scalars) -> i64 {
    i64::from(scalars.tiny)
        + i64::from(scalars.small)
        + i64::from(scalars.medium)
        + i64::from(scalars.byte)
        + i64::from(scalars.word)
        + i64::from(scalars.dword)
        + scalars.single as i64
}

/// The two platform-width scalars. C carries both; the JVM has no stable type
/// for either, so a JNI binding that declares this function is told so.
pub fn size_shift(offset: isize, count: usize) -> usize {
    count.wrapping_add(offset as usize)
}
