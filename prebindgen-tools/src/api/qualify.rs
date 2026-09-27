use prebindgen_flat::{
    flat::{ExtentSource, GenericArg, TypeKind, TypeRef},
    Flat,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Rewrites flat-namespace spellings into paths the generated crate can name.
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
    pub fn with_default_module(mut self, module: Option<syn::Path>) -> Self {
        self.default_module = module;
        self
    }

    /// The flat model this qualifier reads.
    pub fn flat(&self) -> &'a Flat {
        self.flat
    }

    /// The module that declares `name`, if `name` is a flat item.
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

    /// `ty` spelled with every flat item qualified by its declaring crate.
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

    /// `ty` with its lifetimes erased (`&'a T` → `&T`), for a position that
    /// cannot name the source's lifetime parameters: a `let` annotation or a
    /// closure argument in generated code.
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
