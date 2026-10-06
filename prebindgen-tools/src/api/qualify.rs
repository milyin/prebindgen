use prebindgen_flat::{
    flat::{ExtentSource, GenericArg, TypeKind, TypeRef},
    Flat,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// The parameter types of the closure an adapter builds for an `impl Fn(..)`
/// parameter with arguments `args`, lifetimes elided: `|a0: &Payload| ..`.
///
/// Rust needs the annotations to give the closure the borrowed signature the
/// callee expects. Source items are qualified as their [`ItemName`]s say;
/// standard types are spelled with absolute paths.
///
/// [`ItemName`]: prebindgen_flat::flat::ItemName
pub fn callback_arg_types(flat: &Flat, args: &[TypeRef]) -> Vec<TokenStream> {
    args.iter().map(|t| render(flat, t, true)).collect()
}

/// A reference to the item named `name`: its qualified path when the model
/// declares it, else the name as written.
fn item(flat: &Flat, name: &str) -> TokenStream {
    match flat.element(name).and_then(|e| e.name()) {
        Some(n) => n.to_token_stream(),
        None => name.parse().expect("a type path the model built"),
    }
}

/// A model type spelled for generated Rust; `elide` drops lifetimes, for a
/// position that cannot name the source's lifetime parameters.
pub(crate) fn render(flat: &Flat, ty: &TypeRef, elide: bool) -> TokenStream {
    let child = |ty: &TypeRef| render(flat, ty, elide);
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
                ExtentSource::Const(id) => item(flat, &id.name),
            };
            quote!([#e; #len])
        }
        TypeKind::Named { id, args } => {
            let head = item(flat, &id.name);
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
