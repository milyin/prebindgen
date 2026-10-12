//! JNI owns pointer encoding; Kotlin owns lifetime and serializes access.
use super::*;
use prebindgen_tools::{Access, Condition, PreparedInput, PreparedOutput, Resource, Wire};
#[derive(Clone, Debug)]
pub struct Handle;
#[derive(Clone, Debug)]
pub struct HandleWire {
    pub ty: syn::Type,
    pub access: Access,
}
pub(super) fn input(
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
) -> Result<PreparedInput<HandleWire>> {
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
    let validation =
        quote!(if #condition {::prebindgen_jni_runtime::validate_pointer::<#target>(#w)?;});
    let (acquire, statements) = match access {
        Access::Move => (
            quote!(let #v = if #condition {Some(unsafe {Box::from_raw(#w as *mut ::prebindgen_jni_runtime::Allocation<#target>)})} else {None};),
            quote!(let #v: #ty = #v.expect("active handle").value;),
        ),
        Access::Shared => (
            quote!(),
            quote!(let #v: #ty = unsafe { &(*( #w as *const ::prebindgen_jni_runtime::Allocation<#target>)).value };),
        ),
        Access::Exclusive => (
            quote!(),
            quote!(let #v: #ty = unsafe { &mut (*( #w as *mut ::prebindgen_jni_runtime::Allocation<#target>)).value };),
        ),
    };
    Ok(PreparedInput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(HandleWire {
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
            identity: quote!(#w),
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
) -> Result<PreparedOutput<HandleWire>> {
    if matches!(ty, syn::Type::Reference(_)) {
        return Err("borrowed handle outputs are outside this subset".into());
    }
    let value = if borrowed {
        quote!(::core::clone::Clone::clone(#value))
    } else {
        value
    };
    Ok(PreparedOutput {
        wires: vec![Wire {
            name: w.clone(),
            kind: WireKind::Foreign(HandleWire {
                ty: ty.clone(),
                access: Access::Move,
            }),
            active: Condition::default(),
        }],
        statements: quote!(let mut #w = ::prebindgen_jni_runtime::Allocation::new(#value);),
        values: vec![quote!((&mut *#w) as *mut ::prebindgen_jni_runtime::Allocation<#ty> as u64)],
        commit: quote!(let _ = Box::into_raw(#w);),
    })
}
