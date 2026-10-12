//! Optional boxed-value conversions. Adapters opt in; other storage policies
//! continue to use their own foreign leaves and wire descriptions.
use crate::{Access, Condition, PreparedInput, PreparedOutput, Resource, Wire, WireKind};
use prebindgen_flat::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
#[derive(Clone, Debug)]
pub struct BoxedWire {
    pub ty: syn::Type,
    pub access: Access,
}
/// Storage chosen by the adapter. A wrapper must expose a `value` field and
/// allow construction as `Wrapper { value }`.
pub enum Storage {
    Direct,
    Wrapped(syn::Path),
}
impl Storage {
    fn ty(&self, ty: &syn::Type) -> TokenStream {
        match self {
            Self::Direct => quote!(#ty),
            Self::Wrapped(path) => quote!(#path<#ty>),
        }
    }
    fn field(&self) -> TokenStream {
        match self {
            Self::Direct => quote!(),
            Self::Wrapped(_) => quote!(.value),
        }
    }
    fn wrap(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Direct => value,
            Self::Wrapped(path) => quote!(#path { value: #value }),
        }
    }
}
pub fn input(
    storage: &Storage,
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
) -> Result<PreparedInput<BoxedWire>> {
    let (target, access) = match ty {
        syn::Type::Reference(r) => (
            &*r.elem,
            if r.mutability.is_some() {
                Access::Exclusive
            } else {
                Access::Shared
            },
        ),
        _ => (ty, Access::Move),
    };
    let condition = active.rust();
    let allocation = storage.ty(target);
    let field = storage.field();
    let pointer = format_ident!("{w}_pointer");
    let taken = match storage {
        Storage::Direct => quote!(*#v.expect("active allocation")),
        Storage::Wrapped(_) => quote!(#v.expect("active allocation").value),
    };
    let validation = quote!(
        let #pointer = #w as *mut #allocation;
        if #condition {::prebindgen_runtime::validate_pointer(#pointer)?;}
    );
    let (acquire, statements) = match access {
        Access::Move => (
            quote!(let #v = if #condition {Some(unsafe {Box::from_raw(#pointer)})} else {None};),
            quote!(let #v: #ty = #taken;),
        ),
        Access::Shared => (
            quote!(),
            quote!(let #v: #ty = unsafe { &(*#pointer)#field };),
        ),
        Access::Exclusive => (
            quote!(),
            quote!(let #v: #ty = unsafe { &mut (*#pointer)#field };),
        ),
    };
    Ok(PreparedInput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(BoxedWire {
                ty: target.clone(),
                access,
            }),
            active: active.clone(),
        }],
        validation,
        acquire,
        statements,
        argument: quote!(#v),
        resources: vec![Resource {
            identity: quote!(#pointer.addr()),
            access,
            active: active.clone(),
        }],
    })
}
pub fn output(
    storage: &Storage,
    ty: &syn::Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
) -> Result<PreparedOutput<BoxedWire>> {
    if matches!(ty, syn::Type::Reference(_)) {
        return Err("borrowed pointer outputs are outside this subset".into());
    }
    let value = if borrowed {
        quote!(::core::clone::Clone::clone(#value))
    } else {
        value
    };
    let value = storage.wrap(value);
    let allocation = storage.ty(ty);
    Ok(PreparedOutput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(BoxedWire {
                ty: ty.clone(),
                access: Access::Move,
            }),
            active: Condition::default(),
        }],
        statements: quote!(let mut #w = Box::new(#value);),
        values: vec![quote!((&mut *#w) as *mut #allocation)],
        commit: quote!(let _ = Box::into_raw(#w);),
    })
}
