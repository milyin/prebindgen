//! Niche optimisation for FFI-wire encodings.
//!
//! A *niche* is a bit-pattern that the wire type *can* represent but that
//! a particular converter is guaranteed to never produce on output and
//! always reject on input. Wrappers like `Option<_>` and sum-typed enums
//! carve niches one at a time for their own discriminants and re-export
//! the remainder so further wrappers stack.
//!
//! Direct analogy with Rust's niche optimisation:
//!
//! | Rust                         | This crate                              |
//! | ---------------------------- | --------------------------------------- |
//! | `NonZeroU32` declares `{0}`  | converter sets `niches = Niches::one(…)`|
//! | `Option<NonZeroU32>` is u32  | `Option<T>` reuses inner's wire         |
//! | `Option<Option<NonZeroU32>>` | falls back unless inner exposes ≥2      |
//!
//! In the FFI setting the canonical example is a Rust value encoded as a
//! raw `Box::into_raw` pointer carried over the wire as an integer handle:
//! real `Box::into_raw` results are never `0`, so the converter declares
//! the single niche `{0}`. `Option<T>` then automatically reuses the same
//! integer wire with `0` meaning `None`, matching the C-pointer-with-null
//! ABI most native bindings already use.
//!
//! ## Cascading
//!
//! [`Niches::carve`] returns the next slot together with the remainder.
//! The wrapper places the carved value into its own emitted code (output:
//! `None` is encoded as `slot.value`; input: `slot.matches` is the
//! discriminator predicate) and stores `rest` on its own
//! [`crate::api::core::prebindgen::ConverterImpl::niches`] so any
//! enclosing wrapper can keep carving. Once `rest` is empty further
//! wrappers must fall back to a tag/box scheme.
//!
//! ## Soundness
//!
//! For the carve to be sound, the inner converter's outputs must
//! genuinely avoid the carved bit pattern, and its input must reject it
//! (typically by erroring). The adapter author guarantees this — `Niches`
//! is a *declaration* that the resolver and wrappers trust.
//!
//! ## Calling convention for `matches`
//!
//! The `matches` predicate is spliced into the input wrapper's body where
//! the wire-typed parameter `v` is in scope. The exact shape of `v`
//! depends on the wire kind:
//!
//! * By-reference wires (e.g. an integer handle, or an object-handle
//!   type): `v: &<wire>` — write `*v == 0`, or `v.is_null()` for a handle
//!   type that derefs to a null check.
//! * Raw-pointer wires (`*const T`): `v: <wire>` — write `v.is_null()`
//!   directly, no `*` deref.
//!
//! The adapter producing the niche knows which wire kind it is using and
//! must write `matches` accordingly.
//!
//! `value` is a wire-typed *constant* expression with no `v` and no other
//! locals in scope — just the bit pattern (e.g. `0i64`,
//! `std::ptr::null()`).

/// One free bit-pattern slot in the wire encoding.
///
/// See the module-level docs for the calling convention of `matches` and
/// `value`.
#[derive(Clone)]
pub struct NicheSlot {
    /// Wire-typed constant expression evaluating to this niche's bit
    /// pattern. Used by output wrappers to emit the discriminant.
    pub value: syn::Expr,
    /// Predicate testing whether the wire value `v` (in the local
    /// wrapper convention — see module docs) is *this* slot. Used by
    /// input wrappers to detect the discriminant.
    pub matches: syn::Expr,
}

/// An ordered set of [`NicheSlot`]s that a converter's wire type can
/// represent but that this converter never produces (output) and always
/// rejects (input).
///
/// Ordering: the *first* slot is the next one taken by [`Self::carve`].
/// Wrappers carve from the front; the remaining slots are passed up so
/// that further wrappers can stack their own discriminants.
#[derive(Clone, Default)]
pub struct Niches {
    pub slots: Vec<NicheSlot>,
}

impl Niches {
    /// No free bit-patterns. The default for converters whose wire
    /// encoding uses every bit-pattern as a valid value (e.g. a plain
    /// `i64` wire).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Convenience for the common single-niche case.
    pub fn one(value: syn::Expr, matches: syn::Expr) -> Self {
        Self {
            slots: vec![NicheSlot { value, matches }],
        }
    }

    /// Build from any iterable of slots; ordering is preserved.
    pub fn from_slots<I: IntoIterator<Item = NicheSlot>>(slots: I) -> Self {
        Self {
            slots: slots.into_iter().collect(),
        }
    }

    /// Take the first slot for use as a wrapper's discriminant. Returns
    /// the carved slot and the remaining niches (which the wrapper
    /// should re-export on its own
    /// [`ConverterImpl`](crate::api::core::prebindgen::ConverterImpl)).
    /// `None` if the
    /// set is empty — the caller must fall back to a tag/box scheme.
    pub fn carve(mut self) -> Option<(NicheSlot, Niches)> {
        if self.slots.is_empty() {
            None
        } else {
            let head = self.slots.remove(0);
            Some((head, self))
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[cfg(test)]
mod tests;
