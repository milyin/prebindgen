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
pub fn input(
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
    let pointer = format_ident!("{w}_pointer");
    let validation = quote!(
        let #pointer = #w as *mut ::prebindgen_runtime::Allocation<#target>;
        if #condition {::prebindgen_runtime::validate_allocation(#pointer)?;}
    );
    let (acquire, statements) = match access {
        Access::Move => (
            quote!(let #v = if #condition {Some(unsafe {Box::from_raw(#pointer)})} else {None};),
            quote!(let #v: #ty = #v.expect("active allocation").value;),
        ),
        Access::Shared => (
            quote!(),
            quote!(let #v: #ty = unsafe { &(*#pointer).value };),
        ),
        Access::Exclusive => (
            quote!(),
            quote!(let #v: #ty = unsafe { &mut (*#pointer).value };),
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
    Ok(PreparedOutput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(BoxedWire {
                ty: ty.clone(),
                access: Access::Move,
            }),
            active: Condition::default(),
        }],
        statements: quote!(let mut #w = ::prebindgen_runtime::Allocation::new(#value);),
        values: vec![quote!((&mut *#w) as *mut ::prebindgen_runtime::Allocation<#ty>)],
        commit: quote!(let _ = Box::into_raw(#w);),
    })
}
