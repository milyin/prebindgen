//! Rendering captured Rust syntax through a pipeline-owned capability.
//!
//! The flat crate owns the captured syntax and therefore owns this rendering
//! protocol. A collecting pipeline owns the concrete key that implements it.
//! `prebindgen-registry`, for example, hands its unconstructable
//! `prebindgen_registry::Emit` only to Rust-emission callbacks.
//!
//! This direction preserves the crate pipeline:
//!
//! ```text
//! prebindgen -> prebindgen-flat -> prebindgen-registry -> adapters
//!   extract          parse              collect          convert
//! ```
//!
//! A different collector can use `prebindgen-flat` independently and
//! deliberately implement `RustEmitter` for its own callback key. Merely
//! holding a flat model node does not reveal captured syntax without explicitly
//! establishing another rendering boundary.
//!
//! # Direct syntax doors remain closed
//!
//! ```compile_fail
//! # use prebindgen_flat::flat;
//! fn leak(t: &flat::TypeRef) -> proc_macro2::TokenStream { t.spell() }
//! ```
//!
//! ```compile_fail
//! # use prebindgen_flat::flat;
//! fn leak(f: &flat::Function) -> proc_macro2::TokenStream { f.origin.spell() }
//! ```
//!
//! ```compile_fail
//! # use prebindgen_flat::{Element, flat};
//! fn leak(e: &Element) -> syn::Item { e.as_syn() }
//! ```
//!
//! ```compile_fail
//! # use prebindgen_flat::flat;
//! fn leak(t: &flat::TypeRef) -> &syn::Type { t.as_syn() }
//! ```
//!
//! ```compile_fail
//! # use prebindgen_flat::flat;
//! fn leak(t: &flat::TypeRef) -> syn::Type { t.stripped_syntax() }
//! ```
//!
//! ```compile_fail
//! # use prebindgen_flat::flat;
//! fn leak(k: &flat::TypeKind) -> syn::Type { k.to_syn() }
//! ```

use proc_macro2::TokenStream;

use super::{Alternative, Element, EnumValue, Struct, Type, TypeRef};

/// Rendering operations supplied by a pipeline-owned callback key.
///
/// All methods are renderings: they answer what the source wrote, never what
/// it means. Classification uses the flat model (`TypeRef::kind`, keys and
/// structural readings) and does not need this protocol.
///
/// This trait intentionally has no provided concrete key. Implementing it is a
/// collector's explicit decision to establish an emission boundary; adapters
/// using that collector receive its key only where that collector chooses.
/// The trait is object-safe so a collector wrapper can expose the full API by
/// delegation without reproducing every method.
///
/// An independent collector establishes its own boundary by implementing the
/// protocol for its own key:
///
/// ```
/// use prebindgen_flat::{Flat, RustEmitter};
///
/// struct MyCollectorKey;
/// impl RustEmitter for MyCollectorKey {}
///
/// let flat = Flat::builder().build()?;
/// let syntax: syn::Type = syn::parse_quote!(Option<String>);
/// let reading = flat.classify(&syntax)?;
/// assert_eq!(
///     MyCollectorKey.spell(&reading).to_string(),
///     "Option < String >"
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait RustEmitter {
    /// Spell a type exactly as captured.
    fn spell(&self, ty: &TypeRef) -> TokenStream {
        ty.spell()
    }

    /// Spell a type as a `syn::Type`.
    fn spell_ty(&self, ty: &TypeRef) -> syn::Type {
        let tokens = ty.spell();
        syn::parse_quote!(#tokens)
    }

    /// Spell the type below every transparent wrapper.
    fn spell_stripped(&self, ty: &TypeRef) -> syn::Type {
        ty.stripped_syntax()
    }

    /// Re-emit a captured item verbatim.
    fn item(&self, element: &Element) -> syn::Item {
        element.as_syn()
    }

    /// Re-emit a captured type declaration verbatim.
    fn type_item(&self, ty: &Type) -> syn::Item {
        ty.as_syn()
    }

    /// Re-emit a captured function verbatim.
    fn verbatim_fn(&self, function: &super::Function) -> syn::ItemFn {
        function.origin.as_syn().clone()
    }

    /// Re-emit a captured struct verbatim.
    fn verbatim_struct(&self, item: &Struct) -> syn::ItemStruct {
        item.origin.as_syn().clone()
    }

    /// Re-emit a captured payload enum verbatim.
    fn verbatim_variant(&self, item: &super::Variant) -> syn::ItemEnum {
        item.origin.as_syn().clone()
    }

    /// Re-emit a captured fieldless enum verbatim.
    fn verbatim_enum(&self, item: &super::Enum) -> syn::ItemEnum {
        item.origin.as_syn().clone()
    }

    /// Re-emit a constant as an alias to its source module.
    fn const_alias(&self, item: &super::Constant, source_module: &syn::Path) -> syn::ItemConst {
        let mut alias = item.origin.as_syn().clone();
        let ident = &alias.ident;
        alias.expr = Box::new(syn::parse_quote!(#source_module::#ident));
        alias
    }

    /// Re-emit a captured constant verbatim.
    fn const_verbatim(&self, item: &super::Constant) -> syn::ItemConst {
        item.origin.as_syn().clone()
    }

    /// Re-emit an anonymous feature guard.
    fn guard(&self, guard: &super::Guard) -> syn::ItemConst {
        guard.origin.as_syn().clone()
    }

    /// Return an enum discriminant exactly as written.
    fn discriminant(&self, value: &EnumValue) -> Option<TokenStream> {
        value
            .origin
            .as_syn()
            .discriminant
            .as_ref()
            .map(|(_, expr)| quote::quote!(#expr))
    }

    /// Render a struct pattern or constructor with its captured delimiters.
    fn shape_struct(&self, item: &Struct, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
        super::spell::fields(super::spell::Shaped::shape(item), head, parts)
    }

    /// Render an enum alternative with its captured delimiters.
    fn shape_alternative(
        &self,
        item: &Alternative,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        super::spell::fields(super::spell::Shaped::shape(item), head, parts)
    }

    /// Render an enum value with its captured delimiters.
    fn shape_enum_value(
        &self,
        item: &EnumValue,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        super::spell::fields(super::spell::Shaped::shape(item), head, parts)
    }
}
