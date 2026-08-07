//! Where a node came from: the syntax it was built from, and the source that
//! syntax arrived in.
//!
//! One uniform property of **every** node in the model — item, parameter,
//! field, variant, type, array extent alike. Some levels know less than others
//! (a field has no line of its own), but the shape does not change with the
//! level, so nothing has to copy a piece of provenance downward by hand.

use std::rc::Rc;

use prebindgen::SourceLocation;
use quote::ToTokens;

use super::key::TypeKey;

/// The syntax a node was built from, plus where that syntax came from.
///
/// # Why the two travel together
///
/// `syn` tokens normally carry spans, so in principle the syntax alone could
/// answer "where was this written". Not here: the proc-macro serializes each
/// marked item as a **string** into JSONL, and `build.rs` re-parses it, so every
/// span in [`spell()`](Self::spell) points into an anonymous buffer.
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
/// cost. [`TypeKey`] made the same call for
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
/// # The syntax is sealed
///
/// > **You may output the source. You may not read it.**
///
/// [`spell`](Self::spell) hands out tokens and nothing else, which is all
/// generated Rust ever needed. The node is reachable only through one
/// crate-internal accessor, and the field itself is `pub(super)` — so the
/// model still reads it freely, while everything outside is limited to the
/// spelling.
///
/// It was a public field returning a `syn` node to anyone who asked.
/// Outside this crate, captured syntax is reachable only through
/// [`Emit`](crate::flat::emit::Emit), and the compiler enforces it.
#[derive(Clone, Debug)]
pub struct Origin<S> {
    /// The exact tokens this node was built from.
    ///
    /// `pub(super)` is the seal: inside the model this is the syntax being
    /// lowered, classified and round-tripped, and reading it is the work. Outside,
    /// see [`spell`](Self::spell) and [`as_syn`](Self::as_syn).
    pub(super) syntax: S,
    /// The captured item this node belongs to, shared with every sibling.
    pub location: Rc<SourceLocation>,
}

impl<S> Origin<S> {
    pub fn new(syntax: S, location: Rc<SourceLocation>) -> Self {
        Self { syntax, location }
    }

    /// The node as `syn` — **the escape**.
    ///
    /// Every place that takes the source apart instead of asking the model
    /// comes through here, and `pub(crate)` is what keeps that
    /// list short: only [`Emit`](crate::flat::emit::Emit) can reach it.
    ///
    /// Naming it is not an accusation. An emitter assembling a `syn::Item`, or a
    /// signature the generated crate must restate node for node, legitimately
    /// needs the node. What it stops is reaching for one *by default*.
    pub(crate) fn as_syn(&self) -> &S {
        &self.syntax
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

impl Origin<syn::Type> {
    /// This type's identity as a table key — the same answer
    /// [`TypeRef::key`](super::TypeRef::key) gives for a reading.
    ///
    /// Here because a **declaration** is an `Origin<syn::Type>`: the type a
    /// build script wrote, carried with a placeless location. Asking it for its
    /// identity is not reaching for the node, and it should not have to be
    /// spelled as one — every `TypeKey::from_type(decl.as_syn())` was a keying
    /// operation wearing an escape's clothes.
    pub fn key(&self) -> TypeKey {
        TypeKey::from_type(&self.syntax)
    }
}

impl<S: ToTokens> Origin<S> {
    /// The node's tokens, for generated Rust to spell.
    ///
    /// **The only output route, and deliberately not `ToTokens`.** Implementing
    /// that trait would make `quote!(#node)` work — and hand every consumer
    /// `to_token_stream().to_string()` with it, which is a classifier's input in
    /// a spelling's clothing. A `TokenStream` interpolates just as well one `let`
    /// earlier, and the string, if a site really wants one, is now a two-call
    /// pattern that says so.
    ///
    /// It does not make a token string *impossible* — `spell().to_string()`
    /// reaches one, and the ledger still lists that as open. What it makes is
    /// **visible**: `.to_token_stream().to_string()` was indistinguishable from
    /// the same call on a type an adapter built itself.
    ///
    /// `pub`, not `pub(crate)`: the registry pipeline's own
    /// tests (now in the separate `prebindgen-registry` crate) call this on a
    /// captured element's `origin` — see `TypeRef`'s doc for why this seal is
    /// now a convention rather than a compiler check.
    pub fn spell(&self) -> proc_macro2::TokenStream {
        self.syntax.to_token_stream()
    }
}

impl Origin<syn::Type> {
    /// A **declared** type's tokens.
    ///
    /// Public where [`Origin::spell`] is sealed, and the difference is what `S`
    /// is. An `Origin<syn::ItemFn>`'s tokens re-parse to the captured item, so
    /// handing them out is the item door under another name — that one is
    /// [`Emit`](crate::flat::emit::Emit)'s to open. An
    /// `Origin<syn::Type>` in an adapter's declaration holds a type the
    /// **build script wrote**, which was never captured syntax and which #280
    /// leaves the model no way to have a reading for.
    ///
    /// Still a token route, and still one C3 has to account for when
    /// [`TypeRef::spell`](super::TypeRef::spell) moves onto `Emit`: a
    /// declaration is an identity (`key()`), and the two sites that spell one
    /// do it to splice `#target` into generated Rust.
    pub fn declared_spelling(&self) -> proc_macro2::TokenStream {
        self.spell()
    }
}
