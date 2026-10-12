//! C's terminal conversion is an opaque, typed pointer, with no handle runtime.
use super::*;
use prebindgen_tools::{Access, Condition, PreparedInput, PreparedOutput, Resource, Wire};
#[derive(Clone, Debug)]
pub struct OpaquePointer;
#[derive(Clone, Debug)]
pub struct PointerWire {
    pub ty: syn::Type,
    pub access: Access,
}
pub(super) fn input(
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
) -> Result<PreparedInput<PointerWire>> {
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
    let invalid = active
        .and(quote!(#w.is_null() || (#w as usize) & (::core::mem::align_of::<#target>() - 1) != 0));
    let validation = quote!(if #invalid {return Err("null or misaligned pointer".into());});
    let (acquire, statements) = match access {
        Access::Move => (
            quote!(let #v = if #condition {Some(unsafe {Box::from_raw(#w)})} else {None};),
            quote!(let #v: #ty = *#v.expect("active pointer");),
        ),
        Access::Shared => (quote!(), quote!(let #v: #ty = unsafe { &*#w };)),
        Access::Exclusive => (quote!(), quote!(let #v: #ty = unsafe { &mut *#w };)),
    };
    Ok(PreparedInput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(PointerWire {
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
            identity: quote!(#w.addr()),
            access,
            active: active.clone(),
        }],
    })
}
pub(super) fn output(
    ty: &syn::Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
) -> Result<PreparedOutput<PointerWire>> {
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
            kind: WireKind::Foreign(PointerWire {
                ty: ty.clone(),
                access: Access::Move,
            }),
            active: Condition::default(),
        }],
        statements: quote!(let mut #w = Box::new(#value);),
        values: vec![quote!((&mut *#w) as *mut #ty)],
        commit: quote!(let _ = Box::into_raw(#w);),
    })
}
