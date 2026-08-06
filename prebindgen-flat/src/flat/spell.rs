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
pub(crate) fn fields(shape: &syn::Fields, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
    match shape {
        syn::Fields::Unit => head,
        syn::Fields::Unnamed(_) => quote!(#head(#(#parts),*)),
        syn::Fields::Named(_) => quote!(#head { #(#parts),* }),
    }
}

/// The three elements that carry their own field delimiters.
///
/// `pub(crate)`, so it can bound
/// [`Emit::shape`](crate::flat::emit::Emit::shape) without becoming a door
/// itself: an out-of-crate consumer cannot name it, so cannot call through it.
pub(crate) trait Shaped {
    fn shape(&self) -> &syn::Fields;
}

impl Shaped for Alternative {
    fn shape(&self) -> &syn::Fields {
        &self.origin.as_syn().fields
    }
}
impl Shaped for EnumValue {
    fn shape(&self) -> &syn::Fields {
        &self.origin.as_syn().fields
    }
}
impl Shaped for Struct {
    fn shape(&self) -> &syn::Fields {
        &self.origin.as_syn().fields
    }
}

impl Field {
    /// How the field is addressed in a pattern or an initializer: by name when
    /// it has one, else by position.
    ///
    /// Ungated, and the reason is what it reads: [`name`](Self::name) and
    /// [`index`](Self::index), both model facts. No captured syntax is
    /// involved, so this is not a door — unlike the `spell` methods above,
    /// which read the delimiters the source wrote and are
    /// [`Emit::shape`](crate::flat::emit::Emit::shape)'s to hand out.
    pub fn member(&self) -> syn::Member {
        match &self.name {
            Some(id) => syn::Member::Named(id.clone()),
            None => syn::Member::Unnamed(syn::Index::from(self.index)),
        }
    }

    /// The field bound to `bind`, shaped for whichever address it uses —
    /// `id: __f0` for a named field, `__f0` for a positional one. The part
    /// `fields` takes.
    pub fn bind(&self, bind: &impl ToTokens) -> TokenStream {
        match &self.name {
            Some(id) => quote!(#id: #bind),
            None => quote!(#bind),
        }
    }
}
