//! The [`Gravestone`] runtime trait for inline-opaque by-value FFI types.
//!
//! Unlike the rest of `core`, this is a *runtime* trait: it is implemented by
//! the (generated or hand-written) opaque counterpart of an inline-by-value Rust
//! type, and called from the `extern "C"` converters that
//! [`crate::lang::Cbindgen`] emits for a `value_opaque` declaration.
//!
//! An inline-opaque type is passed across the C ABI *by value* (no `Box`): the
//! Rust value's bytes live directly inside a `#[repr(C, align(_))]` opaque struct
//! of identical size/alignment. Because C "moves" by bitwise copy, a moved-from
//! value still occupies the caller's slot; a *gravestone* is the empty,
//! safely-droppable state written back into that slot after the live value is
//! moved out, so a later destructor call is a harmless no-op.
//!
//! ```ignore
//! // Generated consume (input) converter, schematically:
//! if (*v).is_gravestone() { return Err("moved-from / null value".into()); }
//! let live = ::core::ptr::read(v as *mut RustType);
//! ::core::ptr::write(v, <OpaqueType as Gravestone>::gravestone());
//! Ok(live)
//! ```

/// An FFI opaque value type with a representable *gravestone* (empty /
/// moved-from) state.
///
/// Implemented on the `#[repr(C, align(_))]` counterpart of an inline-by-value
/// Rust type. Enables safe drop-after-move (write a gravestone back on consume)
/// and a null-style niche for `Option<T>`.
pub trait Gravestone {
    /// A freshly-constructed gravestone: an empty value that is safe to drop.
    /// Written into a source slot after its live value is moved out.
    fn gravestone() -> Self;

    /// Whether `self` currently holds the gravestone state.
    ///
    /// For a sound `Option<Self>` niche this must be a state that **no live value
    /// ever holds** — otherwise a legitimate value resembling the gravestone is
    /// misread as `None`.
    fn is_gravestone(&self) -> bool;
}
