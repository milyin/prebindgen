//! Generating Rust field groups from explicit Flat shape facts.
//!
//! No helper here reads an origin or retained `syn` node. The frontend records
//! delimiter kind as [`FieldShape`], and final generation switches only on
//! that model fact.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::element::{Field, FieldShape};

/// Spell a field group: `head`, `head(parts…)` or `head { parts… }`, following
/// the delimiters the source wrote.
///
/// The one place those delimiters are chosen — for match patterns and
/// constructors alike, in either direction, for a struct and a variant alike.
/// `B()` carries no payload and still must be written `E::B()` wherever Rust
/// names it.
///
/// `head` is the type's or variant's path, and each part is an already-rendered
/// [`Field::bind`].
pub(crate) fn fields(shape: FieldShape, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
    match shape {
        FieldShape::Unit => head,
        FieldShape::Tuple => quote!(#head(#(#parts),*)),
        FieldShape::Named => quote!(#head { #(#parts),* }),
    }
}

impl FieldShape {
    /// Spell a group with no parts: `Add`, `Add()` or `Mul {}`.
    ///
    /// What a value of a fieldless enum needs — as a pattern and as a
    /// constructor alike, since for such a value the two are the same text.
    /// A consumer holding the shape and the path spells it without reaching
    /// for the captured syntax.
    pub fn spell_fieldless(self, head: TokenStream) -> TokenStream {
        fields(self, head, &[])
    }
}

impl Field {
    /// How the field is addressed in a pattern or an initializer: by name when
    /// it has one, else by position.
    ///
    /// Ungated, and the reason is what it reads: [`name`](Self::name) and
    /// [`index`](Self::index), both model facts. No captured syntax is
    /// involved.
    pub fn member(&self) -> syn::Member {
        match &self.name {
            Some(id) => syn::Member::Named(id.clone()),
            None => syn::Member::Unnamed(syn::Index::from(self.index)),
        }
    }

    /// The field bound to `bind`, shaped for whichever address it uses —
    /// `id: __f0` for a named field, `__f0` for a positional one. The part a
    /// spelled fields list is built from.
    pub fn bind(&self, bind: &impl ToTokens) -> TokenStream {
        match &self.name {
            Some(id) => quote!(#id: #bind),
            None => quote!(#bind),
        }
    }
}
