//! Boundary smoke tests: call the generated `extern "C"` surface the way a C
//! caller would, including with values no Rust type has.
//!
//! A C `enum` is an `int` at the ABI, so a caller can pass a discriminant no
//! variant declares. The generated wrappers take such an enum as
//! `MaybeUninit<mirror>` — same ABI, same C spelling, but legal to hold any bit
//! pattern — and validate the raw integer before building the Rust enum. These
//! tests construct exactly that out-of-range value; written against the previous
//! (bare mirror enum) signature they would have been undefined behaviour, which
//! is the point.
//!
//! The `.panic()` route (`inside_foo_value`, which has no `char **e` to report
//! through) is deliberately not exercised: a panic out of an `extern "C"` fn
//! aborts the process rather than unwinding, so it cannot be asserted in-process.

use core::{
    ffi::{c_char, c_int, c_void, CStr},
    mem::{align_of, MaybeUninit},
    ptr,
};

use crate::{
    calculator_absorb, calculator_apply, calculator_drop, calculator_get_value, calculator_merge,
    calculator_new, caption_t, example_free, inside_foo_default, inside_foo_value, note_drop,
    note_emphatic, note_new_flagged, note_new_titled, note_t, note_value, operation_t,
};

/// The wire value a C caller passing `value` produces — including a `value`
/// that matches no variant.
unsafe fn raw_operation(value: c_int) -> MaybeUninit<operation_t> {
    let mut slot = MaybeUninit::<operation_t>::uninit();
    ptr::write(slot.as_mut_ptr().cast::<c_int>(), value);
    slot
}

#[test]
fn valid_operation_discriminant_applies() {
    unsafe {
        let c = calculator_new();
        assert!(!c.is_null());
        let mut out = 0.0f64;
        let mut err: *mut c_char = ptr::null_mut();

        assert!(calculator_apply(
            c,
            MaybeUninit::new(operation_t::Add),
            2.0,
            &mut out,
            &mut err
        ));
        assert!(err.is_null());
        assert_eq!(out, 2.0);
        assert_eq!(calculator_get_value(c), 2.0);

        calculator_drop(c);
    }
}

#[test]
fn out_of_range_operation_discriminant_reaches_the_error_channel() {
    unsafe {
        let c = calculator_new();
        assert!(!c.is_null());
        let mut out = 0.0f64;
        let mut err: *mut c_char = ptr::null_mut();
        assert!(calculator_apply(
            c,
            MaybeUninit::new(operation_t::Add),
            2.0,
            &mut out,
            &mut err
        ));
        let before = calculator_get_value(c);

        // 99 is not a discriminant of `operation_t`.
        let mut out = -1.0f64;
        assert!(!calculator_apply(
            c,
            raw_operation(99),
            3.0,
            &mut out,
            &mut err
        ));

        // Reported as a binding error, and the call had no effect: nothing was
        // constructed from the invalid value.
        assert!(!err.is_null());
        let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
        assert!(msg.contains("invalid discriminant 99"), "{msg}");
        assert_eq!(out, -1.0);
        assert_eq!(calculator_get_value(c), before);

        example_free(err.cast::<c_void>());
        calculator_drop(c);
    }
}

/// A negative value is out of range too — the raw discriminant is read as a
/// signed `c_int`, so it is compared, not truncated into some variant.
#[test]
fn negative_operation_discriminant_is_rejected() {
    unsafe {
        let c = calculator_new();
        let mut out = 0.0f64;
        let mut err: *mut c_char = ptr::null_mut();

        assert!(!calculator_apply(
            c,
            raw_operation(-1),
            3.0,
            &mut out,
            &mut err
        ));
        assert!(!err.is_null());
        let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
        assert!(msg.contains("invalid discriminant -1"), "{msg}");

        example_free(err.cast::<c_void>());
        calculator_drop(c);
    }
}

/// The wire a C caller produces by writing raw union bytes: tag `Flagged` with
/// `byte` in the payload slot.
///
/// A `#[repr(C)]` enum with payload variants is laid out as a leading `int`
/// discriminant followed by the variant union, padded to the union's alignment
/// — so the payload begins at the whole type's alignment. The `0`/`1` cases in
/// the test below pin that offset against the generated encoder.
unsafe fn raw_flagged(byte: u8) -> MaybeUninit<note_t> {
    const FLAGGED_TAG: c_int = 3;
    let mut slot = MaybeUninit::<note_t>::zeroed();
    ptr::write(slot.as_mut_ptr().cast::<c_int>(), FLAGGED_TAG);
    ptr::write(
        slot.as_mut_ptr().cast::<u8>().add(align_of::<note_t>()),
        byte,
    );
    slot
}

/// `bool` is the one scalar whose domain is restricted (`0`/`1`), so a `bool`
/// payload crosses behind `MaybeUninit` like a declared enum does: the byte C
/// wrote is read as a `u8` and normalised the way C converts to `_Bool`, never
/// materialised as a Rust `bool`. Written against a bare `bool` payload wire the
/// `2` case below would have been undefined behaviour at `assume_init` — before
/// any `match` — which is the point.
#[test]
fn out_of_domain_bool_payload_is_normalised_not_materialised() {
    unsafe {
        // Rust's own `true`/`false` round trip.
        assert_eq!(note_value(note_new_flagged(MaybeUninit::new(true))), 1);
        assert_eq!(note_value(note_new_flagged(MaybeUninit::new(false))), 0);
        // The hand-written wire agrees with the generated encoder.
        assert_eq!(note_value(raw_flagged(0)), 0);
        assert_eq!(note_value(raw_flagged(1)), 1);
        // Bytes no Rust `bool` may hold: accepted, normalised to `true`.
        assert_eq!(note_value(raw_flagged(2)), 1);
        assert_eq!(note_value(raw_flagged(0xff)), 1);
    }
}

/// The wire a C caller produces for a plain `bool` **parameter** with `byte` in
/// the slot — including a byte no Rust `bool` may hold.
unsafe fn raw_bool_arg(byte: u8) -> MaybeUninit<bool> {
    let mut slot = MaybeUninit::<bool>::uninit();
    ptr::write(slot.as_mut_ptr().cast::<u8>(), byte);
    slot
}

/// #170 instance 1 — the broadest one: a plain `bool` **parameter**. It shares
/// the payload's wire and normalising read, so the byte a caller supplies never
/// becomes a Rust `bool` unchecked. The C prototype is untouched: cbindgen
/// renders `MaybeUninit<bool>` as `bool`, so this is still `note_new_flagged(bool)`
/// in the header.
#[test]
fn out_of_domain_bool_parameter_is_normalised_not_materialised() {
    unsafe {
        assert_eq!(note_value(note_new_flagged(raw_bool_arg(0))), 0);
        assert_eq!(note_value(note_new_flagged(raw_bool_arg(1))), 1);
        assert_eq!(note_value(note_new_flagged(raw_bool_arg(2))), 1);
        assert_eq!(note_value(note_new_flagged(raw_bool_arg(0xff))), 1);
    }
}

/// #170 instance 2 — a `bool` field of a `data_struct`, reached through a
/// tagged-union payload. `caption_t` is built by the generated encoder and then
/// its `emphatic` slot is overwritten the way a C caller can (a `memcpy`, a
/// union, a struct read off the wire); the decode has to normalise it one level
/// down, inside `__cbg_in_Caption`. This is what makes the union's "no invalid
/// value is ever materialised" invariant transitive.
#[test]
fn out_of_domain_bool_data_struct_field_is_normalised_not_materialised() {
    unsafe {
        let text = c"chapter one";
        for (byte, expected) in [(0u8, false), (1, true), (2, true), (0xff, true)] {
            let mut note = note_new_titled(7, text.as_ptr(), raw_bool_arg(0));
            // Overwrite the payload's `bool` field in place, through raw bytes.
            // The payload begins at the union's alignment (see `raw_flagged`).
            let caption = note
                .as_mut_ptr()
                .cast::<u8>()
                .add(align_of::<note_t>())
                .cast::<caption_t>();
            ptr::write(ptr::addr_of_mut!((*caption).emphatic).cast::<u8>(), byte);

            // A union crosses BY VALUE, so the callee copies the label out and
            // the block stays ours — the same contract `smoke.c` relies on.
            assert_eq!(note_emphatic(ptr::read(&note)), expected, "byte {byte:#x}");
            assert_eq!(note_value(ptr::read(&note)), 7);
            note_drop(&mut note);
        }
    }
}

/// The round trip an enum takes across both directions: `inside_foo_default`
/// returns the mirror by value (Rust builds it, so it is always valid), and
/// `inside_foo_value` takes it back in through the validating wire.
#[test]
fn enum_round_trips_through_the_validating_wire() {
    unsafe {
        let v = inside_foo_default();
        assert_eq!(inside_foo_value(MaybeUninit::new(v)), v as i32);
    }
}

/// #189's alias preflight, in the shape it exists for: two **consumed** handles
/// of one type. `calculator_merge(x, x)` would reconstruct one allocation twice
/// — the second `Box::from_raw` on a pointer the first already owns.
///
/// Rejected *before* either conversion runs, so the handle is untouched and
/// still usable afterwards. That is the observable that distinguishes a
/// preflight from a check inside the converter: by then the first argument has
/// already been consumed and there is nothing left to keep.
#[test]
fn aliased_consumed_handles_are_rejected_before_any_conversion() {
    unsafe {
        let c = calculator_new();
        let mut err: *mut c_char = ptr::null_mut();

        assert!(calculator_merge(c, c, &mut err).is_null());
        assert!(!err.is_null());
        let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
        assert!(msg.contains("aliasing arguments"), "{msg}");
        assert!(msg.contains("Calculator"), "{msg}");
        example_free(err.cast::<c_void>());

        // Nothing was consumed: the handle is still live and still owns its
        // value. (Under ASan a consumed-then-reused handle is a use-after-free;
        // here it is simply the argument we passed.)
        assert_eq!(calculator_get_value(c), 0.0);
        calculator_drop(c);
    }
}

/// The mixed case: one **consumed** handle beside one **borrowed** handle. The
/// borrow dangles the moment the consume takes ownership, so it is rejected by
/// the same rule — which is why the generation predicate is "at least one
/// consume or exclusive borrow, and any other access in the same domain"
/// rather than "two or more consumed parameters".
#[test]
fn consumed_and_borrowed_alias_is_rejected() {
    unsafe {
        let c = calculator_new();
        let mut out = -1.0f64;
        let mut err: *mut c_char = ptr::null_mut();

        assert!(!calculator_absorb(c, c, &mut out, &mut err));
        assert!(!err.is_null());
        let msg = CStr::from_ptr(err).to_string_lossy().into_owned();
        assert!(msg.contains("aliasing arguments"), "{msg}");
        // The call had no effect at all.
        assert_eq!(out, -1.0);
        example_free(err.cast::<c_void>());

        assert_eq!(calculator_get_value(c), 0.0);
        calculator_drop(c);
    }
}

/// No false positives: two **distinct** resources of the same type still work,
/// in both the consume/consume and the consume/borrow shape. A preflight that
/// rejected these would have removed working surface, which is exactly what
/// #189 rules out.
#[test]
fn distinct_handles_are_not_treated_as_aliases() {
    unsafe {
        let mut err: *mut c_char = ptr::null_mut();

        let a = calculator_new();
        let b = calculator_new();
        assert!(calculator_apply(
            a,
            MaybeUninit::new(operation_t::Add),
            2.0,
            &mut 0.0,
            &mut err
        ));
        assert!(calculator_apply(
            b,
            MaybeUninit::new(operation_t::Add),
            5.0,
            &mut 0.0,
            &mut err
        ));

        // consume/borrow with distinct resources.
        let mut out = -1.0f64;
        assert!(calculator_absorb(a, b, &mut out, &mut err));
        assert!(err.is_null());
        assert_eq!(out, 7.0);

        // consume/consume with distinct resources. `a` is gone (absorbed), so
        // build two more.
        let c = calculator_new();
        let d = calculator_new();
        let merged = calculator_merge(c, d, &mut err);
        assert!(!merged.is_null());
        assert!(err.is_null());
        assert_eq!(calculator_get_value(merged), 0.0);

        calculator_drop(merged);
        calculator_drop(b);
    }
}
