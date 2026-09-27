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
/// use prebindgen_tools::{flat::Flat, Qualifier};
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
}

impl<'a> Qualifier<'a> {
    /// A qualifier over `flat`. Items are qualified with the crate that
    /// captured them, as stamped by [`prebindgen::Source`].
    pub fn new(flat: &'a Flat) -> Self {
        Self {
            flat,
            default_module: None,
        }
    }

    /// The module used for an item whose capture carries no crate name — a
    /// synthetic stream fed to [`Flat::builder`] without a stamp.
    ///
    /// A recorded crate name takes precedence. Unknown names never use this
    /// fallback. Pass `None` to remove a previously configured fallback.
    pub fn with_default_module(mut self, module: Option<syn::Path>) -> Self {
        self.default_module = module;
        self
    }

    /// The flat model this qualifier reads.
    pub fn flat(&self) -> &'a Flat {
        self.flat
    }

    /// The source prefix for a known item: its recorded crate name (with
    /// hyphens replaced by underscores), or the configured fallback if no
    /// crate name was recorded.
    ///
    /// Returns `None` for an unknown name, a known item with no prefix, or
    /// a recorded crate name that cannot be parsed as a Rust path.
    pub fn module_of(&self, name: &str) -> Option<syn::Path> {
        let element = self.flat.element(name)?;
        match &element.location().crate_name {
            Some(c) => syn::parse_str(&c.replace('-', "_")).ok(),
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
        match ty.kind() {
            TypeKind::Scalar(k) => {
                let id = crate::names::ident(k.as_str());
                quote!(#id)
            }
            TypeKind::Str => quote!(str),
            TypeKind::String => quote!(::std::string::String),
            TypeKind::Unit => quote!(()),
            TypeKind::Optional(t) => {
                let t = self.ty(t);
                quote!(::core::option::Option<#t>)
            }
            TypeKind::Vec(t) => {
                let t = self.ty(t);
                quote!(::std::vec::Vec<#t>)
            }
            TypeKind::Slice(t) => {
                let t = self.ty(t);
                quote!([#t])
            }
            TypeKind::Boxed(t) => {
                let t = self.ty(t);
                quote!(::std::boxed::Box<#t>)
            }
            TypeKind::Cow { lifetime, inner } => {
                let t = self.ty(inner);
                quote!(::std::borrow::Cow<#lifetime, #t>)
            }
            TypeKind::Uninit(t) => {
                let t = self.ty(t);
                quote!(::core::mem::MaybeUninit<#t>)
            }
            TypeKind::Fallible { ok, err } => {
                let (ok, err) = (self.ty(ok), self.ty(err));
                quote!(::core::result::Result<#ok, #err>)
            }
            TypeKind::Ref {
                lifetime,
                mutable,
                inner,
            } => {
                let t = self.ty(inner);
                let m = mutable.then(|| quote!(mut));
                quote!(& #lifetime #m #t)
            }
            TypeKind::Array { elem, extent } => {
                let e = self.ty(elem);
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
                        GenericArg::Lifetime(l) => quote!(#l),
                        GenericArg::Type(t) => self.ty(t),
                    });
                    quote!(#head<#(#args),*>)
                }
            }
            TypeKind::Callback { args } => {
                let args = args.iter().map(|a| self.ty(a));
                quote!(impl Fn(#(#args),*) + Send + Sync + 'static)
            }
        }
    }

    /// Qualify a type while eliding lifetimes in supported borrowing shapes.
    ///
    /// Use for a `let` annotation or closure argument that cannot name the
    /// source's lifetime parameters. References lose their explicit lifetime
    /// (`&'a T` → `&T`); `Cow<'a, T>` becomes `Cow<'_, T>`. This recurses
    /// through references, `Cow`, `Option`, `Vec`, `Box`, and slices.
    ///
    /// Other shapes delegate to [`ty`](Self::ty) unchanged. In particular,
    /// this is not a general lifetime eraser for named generic arguments,
    /// arrays, `Result`, or callback arguments.
    pub fn ty_elided(&self, ty: &TypeRef) -> TokenStream {
        match ty.kind() {
            TypeKind::Ref { mutable, inner, .. } => {
                let t = self.ty_elided(inner);
                let m = mutable.then(|| quote!(mut));
                quote!(& #m #t)
            }
            TypeKind::Cow { inner, .. } => {
                let t = self.ty_elided(inner);
                quote!(::std::borrow::Cow<'_, #t>)
            }
            TypeKind::Optional(t) => {
                let t = self.ty_elided(t);
                quote!(::core::option::Option<#t>)
            }
            TypeKind::Vec(t) => {
                let t = self.ty_elided(t);
                quote!(::std::vec::Vec<#t>)
            }
            TypeKind::Boxed(t) => {
                let t = self.ty_elided(t);
                quote!(::std::boxed::Box<#t>)
            }
            TypeKind::Slice(t) => {
                let t = self.ty_elided(t);
                quote!([#t])
            }
            _ => self.ty(ty),
        }
    }
}
