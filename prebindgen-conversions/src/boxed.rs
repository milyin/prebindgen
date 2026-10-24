//! Foreign conversion helpers for adapters using boxed allocations.
//! Adapters select a [`Storage`] policy and map
//! [`BoxedWire`] data into their own wire kind.
use crate::{Access, InputPlan, OutputPlan, Resource};
use prebindgen_flat::{Result, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
/// Source type and access mode for a wire carrying a boxed allocation address.
/// This is generation metadata used by adapters to declare pointer/handle types;
/// allocation and conversion construction remain preparation responsibilities.
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
/// run call-wide resource validation before acquisition (see [`InputPlan`]).
/// The returned resource records the allocation address and required access.
/// Generated code uses `prebindgen_runtime`.
///
/// # Arguments
///
/// * `storage` — the allocation layout. It must match the layout the adapter's
///   outputs and destructors use for the same type.
/// * `ty` — the source parameter type: `T` moves the value out of the allocation,
///   `&T` and `&mut T` borrow it for the call.
/// * `wire` — identifier of the boundary parameter carrying the address. Its type
///   is the adapter's choice, an integer or a pointer, as long as `wire as *mut _`
///   converts it to the allocation pointer.
/// * `var` — identifier of the Rust local that holds the argument.
/// * `active` — an emitted Rust boolean expression guarding validation and
///   ownership acquisition. Its meaning is private to the converter.
/// * `wire_kind` — builds [`Wire::kind`] from the pointee type and access mode.
///
/// ```
/// use prebindgen_conversions::{boxed};
/// use prebindgen_conversions::Access;
/// let input = boxed::input::<syn::Type, (), (), ()>(
///     &boxed::Storage::Direct, &syn::parse_quote!(&Counter),
///     syn::parse_quote!(raw), syn::parse_quote!(counter),
///     &quote::quote!(true), |boxed| {
///         let ty = boxed.ty;
///         syn::parse_quote!(*const #ty)
///     },
/// )?;
/// assert_eq!(input.resources[0].access, Access::Shared);
/// assert_eq!(prebindgen_flat::type_key(&prebindgen_flat::wire_definitions(std::iter::once(&input.node)).next().unwrap().rust_type()), "* const Counter");
/// assert!(input.acquire.is_empty());
/// assert_eq!(input.resources.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn input<W: prebindgen_flat::WireKind, C, N, B>(
    storage: &Storage,
    ty: &syn::Type,
    wire: syn::Ident,
    var: syn::Ident,
    active: &TokenStream,
    wire_kind: impl FnOnce(BoxedWire) -> W,
) -> Result<InputPlan<W, C, N, B>> {
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
    let condition = active.clone();
    let allocation = storage.ty(target);
    let field = storage.field();
    let pointer = format_ident!("{wire}_pointer");
    let taken = match storage {
        Storage::Direct => quote!(*#var.expect("active allocation")),
        Storage::Wrapped(_) => quote!(#var.expect("active allocation").value),
    };
    let validation = quote!(
        let #pointer = if #condition { #wire as *mut #allocation } else { ::core::ptr::null_mut() };
        if #condition {::prebindgen_runtime::validate_pointer(#pointer)?;}
    );
    let (acquire, statements) = match access {
        Access::Move => (
            quote!(let #var = if #condition {Some(unsafe {Box::from_raw(#pointer)})} else {None};),
            quote!(let #var: #ty = #taken;),
        ),
        Access::Shared => (
            quote!(),
            quote!(let #var: #ty = unsafe { &(*#pointer)#field };),
        ),
        Access::Exclusive => (
            quote!(),
            quote!(let #var: #ty = unsafe { &mut (*#pointer)#field };),
        ),
    };
    Ok(InputPlan {
        fallible: true,
        context: Default::default(),
        acknowledgement: None,
        node: prebindgen_flat::WireNode::Leaf(vec![Wire {
            name: wire.clone(),
            kind: wire_kind(BoxedWire {
                ty: target.clone(),
                access,
            }),
        }]),
        validation,
        acquire,
        statements,
        argument: quote!(#var),
        resources: vec![Resource {
            identity: crate::ResourceIdentity::Single(quote!(#pointer.addr())),
            active: active.clone(),
            access,
        }],
    })
}
/// Prepare an owned boxed-address output with deferred ownership transfer.
///
/// A reference source type is rejected. `statements` retains the box, `values`
/// exposes its address as a `*mut` pointer to the allocation, and `commit`
/// relinquishes ownership only after successful delivery. The adapter must provide
/// a matching destructor. Named source types must satisfy the nonzero-size guards
/// from the flat model.
///
/// # Arguments
///
/// * `storage` — the allocation layout, matching the adapter's inputs and destructor.
/// * `ty` — the owned source type of the value.
/// * `wire` — identifier of the delivered wire, also used for the local box.
/// * `value` — Rust expression producing the value.
/// * `borrowed` — `true` when `value` is a shared reference to `ty`; the generated
///   code then boxes a clone, which requires `ty: Clone`. `false` moves the value.
/// * `wire_kind` — builds [`Wire::kind`] from the source type and access mode.
pub fn output<W: prebindgen_flat::WireKind, C, N, B>(
    storage: &Storage,
    ty: &syn::Type,
    wire: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    wire_kind: impl FnOnce(BoxedWire) -> W,
) -> Result<OutputPlan<W, C, N, B>> {
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
    Ok(OutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: wire_kind(BoxedWire {
                    ty: ty.clone(),
                    access: Access::Move,
                }),
            }]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(let mut #wire = Box::new(#value);),
        values: vec![(wire.clone(), quote!((&mut *#wire) as *mut #allocation))],
        commit: quote!(let _ = Box::into_raw(#wire);),
    })
}
