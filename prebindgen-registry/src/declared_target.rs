//! The one check every declarator owes: a declared function must be **about**
//! the type it was declared for.
//!
//! Both expansion directions ask it — an input constructor must *produce* the
//! parameter's target, an output accessor must *take* the deconstructor's
//! source — and each used to carry its own private copy. Two copies is how the
//! check went missing from a third declarator (prebindgen#223, the P2 from
//! [#221](https://github.com/milyin/prebindgen/pull/221)): the policy lived in
//! prose, so a new declarator started with zero checks and nothing noticed.
//!
//! **The comparison is shared here; being *asked* is enforced elsewhere.** A
//! helper alone would not have prevented that defect — a new declarator could
//! simply not call it. What prevents it is that the signature lookups
//! ([`ctor_signature`](crate::expand), `accessor_signature` in
//! [`crate::unfold`]) take the expected target as a **parameter** and run this
//! check themselves. There is no way to obtain a declared function's signature
//! without naming the type it must match, so the obligation is discharged by
//! the compiler rather than by remembering.

use crate::registry::TypeKey;

/// A declared function that is not about its declared type.
///
/// Deliberately vocabulary-free: it names the two keys that disagreed and
/// leaves the wording to whichever error type absorbs it, because the two
/// directions say it differently — a constructor *produces*, an accessor
/// *takes*. Those messages are the binding author's diagnostics and belong with
/// their own error enum, not here.
pub(crate) struct TargetMismatch {
    /// The declared function's ident.
    pub(crate) func: String,
    /// The type it is actually about.
    pub(crate) actual: String,
    /// The type it was declared for.
    pub(crate) expected: String,
}

/// The comparison itself: two [`TypeKey`]s, keyed so that spelling differences
/// which do not change identity cannot make a correct declaration fail.
pub(crate) fn check_declared_target(
    func: &syn::Ident,
    actual: &TypeKey,
    expected: &TypeKey,
) -> Result<(), TargetMismatch> {
    if actual == expected {
        Ok(())
    } else {
        Err(TargetMismatch {
            func: func.to_string(),
            actual: actual.to_string(),
            expected: expected.to_string(),
        })
    }
}
