//! Spelling an element back as Rust: the one place the model turns into tokens.
//!
//! The other half of **classify off `kind`, spell with `spell()`**. Every helper
//! here reads an element's retained syntax and emits Rust; none of them decides
//! what anything *means*. Keeping them out of [`element`](super::element) is
//! what lets that module describe structure alone — a `Field` is a name, a
//! position and a type, and whether Rust writes it `id: v` or `v` is answered
//! here.
//!
//! Nothing outside generated Rust reads any of this: a destination language
//! cannot tell `E::B` from `E::B()`, which is exactly why the delimiters are
//! spelling rather than a modelled shape.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::element::{Alternative, EnumValue, Field, Struct};

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
pub fn fields(shape: &syn::Fields, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
    match shape {
        syn::Fields::Unit => head,
        syn::Fields::Unnamed(_) => quote!(#head(#(#parts),*)),
        syn::Fields::Named(_) => quote!(#head { #(#parts),* }),
    }
}

impl Alternative {
    /// [`fields`] over this alternative's own delimiters.
    pub fn spell(&self, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
        fields(&self.origin.as_syn().fields, head, parts)
    }
}

impl EnumValue {
    /// [`fields`] over this value's own delimiters.
    ///
    /// A fieldless alternative still has them: `A`, `B()` and `C {}` carry no
    /// payload alike, and Rust demands the delimiters wherever the last two are
    /// named. `parts` is therefore always empty — the signature matches
    /// [`Alternative::spell`] so one caller can spell either.
    pub fn spell(&self, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
        fields(&self.origin.as_syn().fields, head, parts)
    }
}

impl Struct {
    /// [`fields`] over this struct's own delimiters — the dual of
    /// [`Variant::spell`], and the reason neither needs a modelled shape.
    pub fn spell(&self, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
        fields(&self.origin.as_syn().fields, head, parts)
    }
}

impl Field {
    /// How the field is addressed in a pattern or an initializer: by name when
    /// it has one, else by position.
    pub fn member(&self) -> syn::Member {
        match &self.name {
            Some(id) => syn::Member::Named(id.clone()),
            None => syn::Member::Unnamed(syn::Index::from(self.index)),
        }
    }

    /// The field bound to `bind`, shaped for whichever address it uses —
    /// `id: __f0` for a named field, `__f0` for a positional one. The part
    /// [`fields`] takes.
    pub fn bind(&self, bind: &impl ToTokens) -> TokenStream {
        match &self.name {
            Some(id) => quote!(#id: #bind),
            None => quote!(#bind),
        }
    }
}
