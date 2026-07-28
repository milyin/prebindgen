//! The Rust frontend: the single authority for what captured `#[prebindgen]`
//! source **means**.
//!
//! Everything downstream of this module — the registry's planning phases and
//! every language adapter — consumes the frontend's decisions rather than the
//! captured syntax. An adapter chooses how a value crosses ITS boundary and
//! how it appears in the destination language; it does not get to decide what
//! the source Rust means, or whether its spelling is allowed.
//!
//! The accepted subset is written down in `docs/source-language.md`.
//!
//! ## Why one walk per construct
//!
//! The frontend's rule is that **validation is a consequence of lowering**, not
//! a separate judgement. For each source construct there is one fallible
//! function from captured syntax to a closed representation; a form it cannot
//! lower is, by construction, a form the language does not accept.
//!
//! That is not a stylistic preference. Array lengths previously had two
//! independent walks — a whitelist deciding what was accepted and a rewriter
//! deciding what it could qualify — with nothing tying them together. They
//! disagreed eight times (issue #210), most sharply over `[u8; <Holder>::N]`,
//! which the whitelist accepted and the rewriter silently declined to qualify,
//! emitting a path the generated crate cannot resolve. With one walk that shape
//! is not representable.
//!
//! ## Scope today
//!
//! * [`array_len`] — the fixed-size-array length subgrammar.
//! * [`model`] — a closed [`SourceType`](model::SourceType) covering the type
//!   grammar in `docs/source-language.md`, plus
//!   [`SourceStruct`](model::SourceStruct) for indexed structs. A struct
//!   field's `syn` type is written from the model, so the model is authoritative
//!   and the syntax is a projection of it.
//!
//! Functions and enums still keep their `syn` items, and every adapter except
//! cbindgen's struct-field path still classifies types at its use site. Issue
//! #211 tracks migrating the rest.

mod array_len;
pub mod model;

/// The mechanical boundary check (#211 completion criterion 6): a committed
/// ledger of every classification site outside this module.
#[cfg(test)]
mod boundary;
#[cfg(test)]
mod tests;

pub(crate) use self::array_len::{resolve_array_lengths, ConstIndex};
/// The frontend's **public** surface is the diagnostics — what a caller has to
/// handle. The decided values are read from the registry
/// ([`Registry::array_len`](crate::api::core::registry::Registry::array_len)),
/// not from here.
///
/// Lowering itself is crate-private on purpose. The entry point is
/// [`Registry::from_items`](crate::api::core::registry::Registry::from_items),
/// which runs the frontend over the whole stream; there is no supported way to
/// lower one construct in isolation, because the decisions need the complete
/// const index and the item's origin. Exposing the machinery would offer
/// callers a second, unvalidated route to a `SourceModel` — the very thing #211
/// exists to prevent.
///
/// A length's decided model is an
/// [`ArrayExtent`](model::ArrayExtent): its **value**, and which const — if any
/// — the use site named. The value is what a type-keyed table stores, since
/// equal-valued lengths are one Rust type; the spelling lives on the use site
/// in [`SourceStruct`](model::SourceStruct), which is what lets a C header emit
/// `uint8_t tag[MARKER_TAG_LEN]`.
pub use self::array_len::{ArrayLenReason, UnsupportedArrayLen};
