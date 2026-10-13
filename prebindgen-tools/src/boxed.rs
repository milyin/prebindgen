//! Foreign conversion helpers for adapters using boxed allocations.
//! Adapters select a [`Storage`](crate::boxed::Storage) policy and map
//! [`BoxedWire`](crate::boxed::BoxedWire) data into their wire's `extra` field.
use crate::{Access, Condition, PreparedInput, PreparedOutput, Resource, Wire, WireKind};
use prebindgen_flat::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
/// Source type and access mode for a wire carrying a boxed allocation address.
#[derive(Clone, Debug)]
pub struct BoxedWire {
    /// Source value type inside the allocation, without the input reference or storage wrapper.
    pub ty: syn::Type,
    /// Whether the call moves, shares or exclusively borrows the stored value.
    pub access: Access,
}
/// Storage chosen by the adapter. A wrapper must expose a `value` field and
/// allow construction as `Wrapper { value }`.
pub enum Storage {
    /// The allocation is `Box<T>` and its address is exposed directly.
    Direct,
    /// The allocation is `Box<Wrapper<T>>`; the path identifies `Wrapper`.
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
/// Prepare a boxed-address input for `T`, `&T` or `&mut T`.
///
/// `T` transfers ownership; references borrow the allocation for the call.
/// Validation checks only nonnull/aligned encoding. The caller must guarantee
/// allocation provenance, type, liveness and synchronization. The wrapper must
/// run call-wide resource validation before acquisition (see [`PreparedInput`]).
/// The returned resource records the allocation address and required access.
///
/// `wire_extra` receives the pointee type and access mode as [`BoxedWire`]. Its
/// return value becomes [`Wire::extra`]. Generated code uses `prebindgen_runtime`.
///
/// ```
/// use prebindgen_tools::{boxed, Access, Condition};
/// let input = boxed::input(
///     &boxed::Storage::Direct, &syn::parse_quote!(&Counter),
///     syn::parse_quote!(raw), syn::parse_quote!(counter),
///     &Condition::default(), |boxed| boxed.access,
/// )?;
/// assert_eq!(input.wires[0].extra, Access::Shared);
/// assert!(input.acquire.is_empty());
/// assert_eq!(input.resources.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn input<W>(
    storage: &Storage,
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
    wire_extra: impl FnOnce(BoxedWire) -> W,
) -> Result<PreparedInput<W>> {
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
            kind: WireKind::Foreign,
            extra: wire_extra(BoxedWire {
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
/// Prepare an owned boxed-address output with deferred ownership transfer.
///
/// When `borrowed` is true, `value` is `&T` and generated code clones `T`; otherwise
/// it moves the value. A reference source type is rejected. `statements` retains
/// the box, `values` exposes its address, and `commit` relinquishes ownership only
/// after successful delivery. The adapter must provide a matching destructor.
///
/// `wire_extra` receives [`BoxedWire`] and returns the value stored in [`Wire::extra`].
/// Named source types must satisfy the nonzero-size guards from the flat model.
pub fn output<W>(
    storage: &Storage,
    ty: &syn::Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    wire_extra: impl FnOnce(BoxedWire) -> W,
) -> Result<PreparedOutput<W>> {
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
            kind: WireKind::Foreign,
            extra: wire_extra(BoxedWire {
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
