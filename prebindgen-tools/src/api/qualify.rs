use std::collections::HashMap;

use prebindgen_flat::{
    flat::{ExtentSource, GenericArg, TypeKind, TypeRef},
    Flat,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Spells source items and types as Rust paths usable from a binding crate.
///
/// The qualifier borrows a [`Flat`] model and reads the declaring crate from
/// each item's source location. [`path`](Self::path) qualifies one item;
/// [`ty`](Self::ty) recursively qualifies a type expression. Both return
/// tokens for generated Rust without changing the model or choosing an ABI.
///
/// See the [qualification module](index.html) for path selection rules,
/// including nested containers and binding-local names.
///
/// # Example: call a source function from generated Rust
///
/// Build a model with the source crate recorded on its items, then qualify
/// both the function to call and its return type. The resulting tokens can
/// be placed in a generated wrapper:
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::Qualifier;
/// use quote::quote;
///
/// let source = syn::parse_file(
///     "pub struct Payload; pub fn make() -> Payload { Payload }",
/// ).unwrap();
/// let location = SourceLocation {
///     crate_name: Some("source-crate".into()),
///     ..Default::default()
/// };
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, location.clone())))
///     .build().unwrap();
/// let qualifier = Qualifier::new(&flat);
/// let function = flat.function("make").unwrap();
/// let callee = qualifier.path(&function.name);
/// let return_type = qualifier.ty(&function.ret);
/// let wrapper = quote! {
///     pub fn make_wrapper() -> #return_type { #callee() }
/// };
/// assert_eq!(wrapper.to_string(), quote! {
///     pub fn make_wrapper() -> source_crate::Payload { source_crate::make() }
/// }.to_string());
/// ```
#[derive(Clone)]
pub struct Qualifier<'a> {
    flat: &'a Flat,
    default_module: Option<syn::Path>,
    crate_paths: HashMap<String, syn::Path>,
}

impl<'a> Qualifier<'a> {
    /// A qualifier over `flat`. Items are qualified with the crate that
    /// captured them, as stamped by [`prebindgen::Source`].
    pub fn new(flat: &'a Flat) -> Self {
        Self {
            flat,
            default_module: None,
            crate_paths: HashMap::new(),
        }
    }

    /// The module used for an item whose capture carries no crate name — a
    /// synthetic stream fed to [`Flat::builder`] without a stamp.
    ///
    /// A recorded crate name or its configured override takes precedence.
    /// Unknown names never use this fallback. Pass `None` to remove a previously configured fallback.
    pub fn with_default_module(mut self, module: Option<syn::Path>) -> Self {
        self.default_module = module;
        self
    }

    /// Name items captured from `crate_name` through `path` in generated Rust.
    ///
    /// Use for a renamed dependency or a module re-exporting a source API.
    /// This overrides the recorded crate's path without changing source
    /// locations. Hyphens and underscores in `crate_name` are equivalent;
    /// a later override for the same crate replaces the earlier one.
    /// Items from other crates and unknown names are unaffected.
    pub fn with_crate_path(mut self, crate_name: impl Into<String>, path: syn::Path) -> Self {
        self.crate_paths
            .insert(crate_name.into().replace('-', "_"), path);
        self
    }

    /// The source prefix for a known item: a configured override, its recorded
    /// crate name (with hyphens replaced by underscores), or the fallback if no
    /// crate name was recorded.
    ///
    /// Returns `None` for an unknown name, a known item with no prefix, or
    /// a recorded crate name that cannot be parsed as a Rust path.
    fn module_of(&self, name: &str) -> Option<syn::Path> {
        let element = self.flat.element(name)?;
        match &element.location().crate_name {
            Some(c) => {
                let name = c.replace('-', "_");
                self.crate_paths
                    .get(&name)
                    .cloned()
                    .or_else(|| syn::parse_str(&name).ok())
            }
            None => self.default_module.clone(),
        }
    }

    /// The path generated code calls or names the flat item `name` by:
    /// `perftest_flat::storage_new`. A name the model does not know is
    /// returned unchanged.
    pub fn path(&self, name: &syn::Ident) -> TokenStream {
        match self.module_of(&crate::names::bare(name)) {
            Some(m) => quote!(#m::#name),
            None => quote!(#name),
        }
    }

    /// Spell a model type with named items qualified by their source crate.
    ///
    /// Recurses into containers, borrows, generic type arguments, and callback
    /// arguments. Named array lengths are qualified too. Standard types use
    /// absolute `::core` or `::std` paths, and source lifetimes are retained.
    /// The result still describes the source type, before ABI conversion.
    pub fn ty(&self, ty: &TypeRef) -> TokenStream {
        self.render_type(ty, false)
    }

    /// Qualify a type while eliding every explicit type lifetime recursively.
    ///
    /// Use for a `let` annotation or closure argument that cannot name the
    /// source's lifetime parameters. References lose their explicit lifetime
    /// (`&'a T` → `&T`); `Cow` and named generic lifetime arguments use `'_`.
    /// This applies inside every supported shape, including arrays, `Result`,
    /// generic type arguments and callback arguments, and includes `'static`.
    /// A callback's required `+ 'static` bound is retained.
    /// Use [`ty`](Self::ty) when explicit lifetime constraints must be retained.
    pub fn ty_elided(&self, ty: &TypeRef) -> TokenStream {
        self.render_type(ty, true)
    }

    fn render_type(&self, ty: &TypeRef, elide: bool) -> TokenStream {
        let child = |ty: &TypeRef| self.render_type(ty, elide);
        let render_lifetime = |l: &syn::Lifetime| {
            if elide {
                quote!('_)
            } else {
                quote!(#l)
            }
        };
        match ty.kind() {
            TypeKind::Scalar(k) => {
                let id = crate::names::ident(k.as_str());
                quote!(#id)
            }
            TypeKind::Str => quote!(str),
            TypeKind::String => quote!(::std::string::String),
            TypeKind::Unit => quote!(()),
            TypeKind::Optional(t) => {
                let t = child(t);
                quote!(::core::option::Option<#t>)
            }
            TypeKind::Vec(t) => {
                let t = child(t);
                quote!(::std::vec::Vec<#t>)
            }
            TypeKind::Slice(t) => {
                let t = child(t);
                quote!([#t])
            }
            TypeKind::Boxed(t) => {
                let t = child(t);
                quote!(::std::boxed::Box<#t>)
            }
            TypeKind::Cow { lifetime, inner } => {
                let t = child(inner);
                let lifetime = render_lifetime(lifetime);
                quote!(::std::borrow::Cow<#lifetime, #t>)
            }
            TypeKind::Uninit(t) => {
                let t = child(t);
                quote!(::core::mem::MaybeUninit<#t>)
            }
            TypeKind::Fallible { ok, err } => {
                let (ok, err) = (child(ok), child(err));
                quote!(::core::result::Result<#ok, #err>)
            }
            TypeKind::Ref {
                lifetime,
                mutable,
                inner,
            } => {
                let t = child(inner);
                let m = mutable.then(|| quote!(mut));
                let lifetime = lifetime.as_ref().filter(|_| !elide);
                quote!(& #lifetime #m #t)
            }
            TypeKind::Array { elem, extent } => {
                let e = child(elem);
                let len = match &extent.source {
                    ExtentSource::Literal => {
                        let n = proc_macro2::Literal::usize_unsuffixed(extent.value);
                        quote!(#n)
                    }
                    ExtentSource::Const(id) => {
                        let name = crate::names::ident(&id.name);
                        let path = self.path(&name);
                        quote!(#path)
                    }
                };
                quote!([#e; #len])
            }
            TypeKind::Named { id, args } => {
                let head: TokenStream = match id.ident() {
                    Some(name) => self.path(&name),
                    None => id.name.parse().expect("a type path the model built"),
                };
                if args.is_empty() {
                    head
                } else {
                    let args = args.iter().map(|a| match a {
                        GenericArg::Lifetime(l) => render_lifetime(l),
                        GenericArg::Type(t) => child(t),
                    });
                    quote!(#head<#(#args),*>)
                }
            }
            TypeKind::Callback { args } => {
                let args = args.iter().map(child);
                quote!(impl Fn(#(#args),*) + Send + Sync + 'static)
            }
        }
    }
}
