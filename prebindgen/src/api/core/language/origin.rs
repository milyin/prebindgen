//! Where a node came from: the syntax it was built from, and the source that
//! syntax arrived in.
//!
//! One uniform property of **every** node in the model — item, parameter,
//! field, variant, type, array extent alike. Some levels know less than others
//! (a field has no line of its own), but the shape does not change with the
//! level, so nothing has to copy a piece of provenance downward by hand.

use std::rc::Rc;

use quote::ToTokens;

use crate::SourceLocation;

/// The syntax a node was built from, plus where that syntax came from.
///
/// # Why the two travel together
///
/// `syn` tokens normally carry spans, so in principle the syntax alone could
/// answer "where was this written". Not here: the proc-macro serializes each
/// marked item as a **string** into JSONL, and `build.rs` re-parses it, so every
/// span in [`Self::syntax`] points into an anonymous buffer.
/// [`SourceLocation::from_span`] captures file, line and column at
/// macro-expansion time — while real rustc spans still exist — precisely because
/// they cannot survive that trip.
///
/// # Why the location is shared
///
/// One captured record is one item, so there is exactly one location per item
/// and none of its own for any component. An item and everything inside it —
/// parameters, fields, variants, types, extents — therefore point at the *same*
/// [`SourceLocation`], which is both the honest answer and the cheap one: a
/// struct with twenty fields keeps one location, not twenty copies of a path.
///
/// `Rc` rather than `Arc`: this holds `syn` values, which are `!Send`, so the
/// model can never cross a thread boundary and an atomic refcount would only
/// cost. [`TypeKey`](crate::api::core::registry::TypeKey) made the same call for
/// the same reason.
///
/// # The rule about origins
///
/// > A reference carries a name; the declaration carries the origin.
///
/// `location.crate_name` here is the crate whose source this node was written
/// *in* — the use site. It is never part of a referenced item's identity:
/// [`TypeId`](super::TypeId) is a name alone, because `#[prebindgen]` names live
/// in one flat namespace. [`ConstId`](super::ConstId) is not an exception — the
/// crate it records is the const's *declaring* crate, obtained by lookup, and
/// that is exactly what lets an array extent refuse a const from another source.
#[derive(Clone, Debug)]
pub struct Origin<S> {
    /// The exact tokens this node was built from. Feed it to `quote!` when
    /// generated Rust has to spell the node; never `match` on it to decide what
    /// the node is — see the [module docs](super) on classifying off `kind`.
    pub syntax: S,
    /// The captured item this node belongs to, shared with every sibling.
    pub location: Rc<SourceLocation>,
}

impl<S> Origin<S> {
    pub fn new(syntax: S, location: Rc<SourceLocation>) -> Self {
        Self { syntax, location }
    }

    /// The crate this node's source was written in.
    ///
    /// The **use site**, not the declaring crate of anything it names.
    pub fn crate_name(&self) -> Option<&str> {
        self.location.crate_name.as_deref()
    }

    /// The same location over different syntax — for building a component's
    /// origin from the item's.
    pub fn with<T>(&self, syntax: T) -> Origin<T> {
        Origin {
            syntax,
            location: Rc::clone(&self.location),
        }
    }
}

/// Spelling a node emits its syntax, so a site that re-states a whole node
/// needs no `.syntax` hop.
impl<S: ToTokens> ToTokens for Origin<S> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.syntax.to_tokens(tokens)
    }
}
