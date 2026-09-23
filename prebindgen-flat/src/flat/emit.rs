//! Final Rust generation through a pipeline-owned capability.
//!
//! Flat owns the model-driven generation protocol. A collecting pipeline owns
//! the concrete key that implements it.
//! `prebindgen-registry`, for example, hands its unconstructable
//! private receiver inside the `prebindgen_registry::RustWriter` handed to
//! final callbacks.
//!
//! This direction preserves the crate pipeline:
//!
//! ```text
//! prebindgen -> prebindgen-flat -> prebindgen-registry -> adapters
//!   extract          parse              collect          convert
//! ```
//!
//! A different collector can use `prebindgen-flat` independently and
//! deliberately implement `RustEmitter` for its own callback key. The protocol
//! contains generation operations only; implementing it does not expose
//! retained source syntax.
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

use super::{
    Alternative, Element, Enum, EnumValue, Extern, Field, Function, Struct, Type, TypeRef,
};

/// Rendering operations supplied by a pipeline-owned callback key.
///
/// All methods are renderings. Classification uses the flat model
/// (`TypeRef::kind`, keys and structural readings) and does not need this
/// protocol. Source types are generated from those facts as inert tokens;
/// retained syntax is never returned as a typed AST.
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
/// use std::collections::HashMap;
///
/// struct MyCollectorKey;
/// impl RustEmitter for MyCollectorKey {}
///
/// let flat = Flat::builder().build()?;
/// let syntax: syn::Type = syn::parse_quote!(Option<String>);
/// let reading = flat.classify(&syntax)?;
/// let module: syn::Path = syn::parse_quote!(source);
/// assert_eq!(MyCollectorKey
///     .emit_source_type(&reading, &HashMap::new(), &module)
///     .to_string(), ":: core :: option :: Option < :: std :: string :: String >");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
/// A model node that may have been written under a condition.
///
/// The variants are the nodes a generator holds when it needs one: a captured
/// item reached by name, the function or record behind a requested declaration,
/// or one field of that record.
#[derive(Clone, Copy)]
pub enum Conditioned<'a> {
    /// A whole captured item, whichever kind it is.
    Item(&'a Element),
    Function(&'a Function),
    Type(&'a Type),
    Struct(&'a Struct),
    Extern(&'a Extern),
    /// A fieldless enum, which a target may re-declare on its own side.
    Enum(&'a Enum),
    Field(&'a Field),
}

impl Conditioned<'_> {
    fn conditions(self) -> Vec<TokenStream> {
        match self {
            Conditioned::Item(element) => element.conditions(),
            Conditioned::Function(function) => function.conditions(),
            Conditioned::Type(ty) => ty.conditions(),
            Conditioned::Struct(record) => record.conditions(),
            Conditioned::Extern(opaque) => opaque.conditions(),
            Conditioned::Enum(unit) => unit.conditions(),
            Conditioned::Field(field) => field.conditions(),
        }
    }
}

pub trait RustEmitter {
    /// Generate a source type from Flat facts, qualifying model-known nominal
    /// types and const extents to their declaration modules.
    ///
    /// This returns an inert output fragment. It neither exposes nor walks the
    /// captured Rust node; all structural decisions come from `TypeKind`,
    /// `TypeId`, and `ConstId`.
    fn emit_source_type(
        &self,
        ty: &TypeRef,
        modules: &std::collections::HashMap<String, syn::Path>,
        default_module: &syn::Path,
    ) -> TokenStream {
        ty.emit_source_type(modules, default_module)
    }

    /// Copy the pipeline-owned anonymous guard into the final file.
    ///
    /// The item is private model output, not an adapter-visible source node.
    fn guard(&self, guard: &super::Guard) -> syn::ItemConst {
        guard.output.clone()
    }

    /// Copy a node's unevaluated `#[cfg]` conditions into the final output —
    /// one token stream per attribute.
    ///
    /// The capture reader answers the conditions it has rules for and rewrites
    /// the rest back onto the item; those are what comes back here, verbatim.
    /// Re-apply them to everything generated for that node, so what is
    /// generated exists exactly where the node it names does. Nothing — not the
    /// model, not planning, not this call — asks what a condition means.
    ///
    /// Empty when the node carries none, which is the ordinary case.
    ///
    /// One call rather than one per node kind: an item's condition and a
    /// field's are the same fact at two levels, and they are re-applied in
    /// different places only because different things are generated from them.
    fn conditions(&self, node: Conditioned<'_>) -> Vec<TokenStream> {
        node.conditions()
    }

    /// Copy an explicit enum discriminant into the final output.
    fn discriminant(&self, value: &EnumValue) -> Option<TokenStream> {
        value.discriminant_output.clone()
    }

    /// Generate a struct pattern or constructor from its modeled shape.
    fn shape_struct(&self, item: &Struct, head: TokenStream, parts: &[TokenStream]) -> TokenStream {
        super::spell::fields(item.shape, head, parts)
    }

    /// Generate an enum alternative from its modeled shape.
    fn shape_alternative(
        &self,
        item: &Alternative,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        super::spell::fields(item.shape, head, parts)
    }

    /// Generate a fieldless enum value from its modeled shape.
    fn shape_enum_value(
        &self,
        item: &EnumValue,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        super::spell::fields(item.shape, head, parts)
    }
}
