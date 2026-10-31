//! Generate Rust wrappers and C declarations from conversion slots.
#![warn(missing_docs)]

use prebindgen_conversions::value::Value;
pub use prebindgen_conversions::Access;
use prebindgen_flat::{
    Conversion, Flat, InputConversion, OutputConversion, Primitive, Result, WireKind,
};

mod callback;
pub use callback::CCallback;
mod wire;
pub use wire::CWire;
/// Pointee type and Rust ownership/access mode of a C boundary pointer.
#[derive(Clone, Debug)]
pub struct CPointer {
    /// Rust pointee type used in the ABI and the opaque C type declaration.
    pub ty: syn::Type,
    /// Move transfers ownership; shared/exclusive access carries a non-owning borrow.
    pub access: Access,
}
impl From<prebindgen_conversions::boxed::BoxedWire> for CPointer {
    fn from(value: prebindgen_conversions::boxed::BoxedWire) -> Self {
        Self {
            ty: value.ty,
            access: value.access,
        }
    }
}
mod owned;
pub use owned::COwned;
mod layouts;
pub use layouts::{OpaqueLayout, OpaqueLayouts};
mod metadata;
pub use metadata::{CEnum, CFunction, CModule, CSchema, CType};
mod enums;
mod errors;
mod group;
mod sequence;
use group::validate_layout;
pub use group::{CBranch, CChain, CGroup, COptional};
use prebindgen_declarations::{Codec, InputRequest, OutputRequest};
use prebindgen_flat::Wire as GenericWire;
pub use sequence::CSequence;
mod emission;
pub use emission::CContext;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, CContext, crate::CChain, crate::CBranch>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, CContext, crate::CChain, crate::CBranch>;
/// Flat source model with C conversion data for wrapper emission.
pub type Bindings = Flat<CSchema>;
type Wire = GenericWire<CWire>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Conversions and slot glue used by [`Cbindgen`].
/// Prepared code uses explicit emission parameters for the output pointer and
/// result type. The writer supplies these bindings and the error boundary.
/// Use with [`prebindgen_declarations::bind_input`] and [`prebindgen_declarations::bind_output`]
/// to fill [`Bindings`] manually.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen, CCodec};
/// use prebindgen_declarations::{bind_input, bind_output, Input, Output};
/// let source = syn::parse_file("pub fn echo(x: i64) -> i64 { x }")?;
/// let mut flat = Bindings::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// bind_input(&mut flat, &CCodec, "echo", "x", &Input::Leaf, &())?;
/// bind_output(&mut flat, &CCodec, "echo", &Output::Leaf, &())?;
/// let generated = Cbindgen::new(flat).build()?;
/// assert!(generated.header.contains("pb_echo("));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CCodec;
impl Codec for CCodec {
    type Schema = CSchema;
    type Niche = CNiche;
    type Chain = CChain;
    type Branch = CBranch;
    fn input_collection_is_leaf(&self, flat: &Bindings, ty: &syn::Type, _: &()) -> bool {
        owned::native_input(flat, ty)
    }
    fn output_collection_is_leaf(&self, flat: &Bindings, ty: &syn::Type, _: &()) -> bool {
        owned::native_output(flat, ty)
    }
    fn group_metadata(&self) -> CBranch {
        CBranch::Group(CGroup::Values)
    }

    fn binding_failure_layout(
        &self,
        _: &(),
    ) -> Result<Option<prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>>> {
        Ok(Some(prebindgen_flat::WireNode::Leaf(vec![
            prebindgen_flat::Wire {
                name: syn::parse_quote!(message),
                kind: CWire::Buffer,
            },
        ])))
    }
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<CNiche>,
        controls: &[prebindgen_flat::Wire<CWire>],
    ) -> Result<CChain> {
        COptional::from_parts(presence, discriminator, niche, controls).map(CChain::Optional)
    }
    fn alternatives_metadata(&self, selector: &prebindgen_flat::Wire<CWire>) -> Result<CBranch> {
        group::validate_selector(selector)?;
        Ok(CBranch::Alternatives)
    }
    fn field_group(&self, name: syn::Ident) -> CBranch {
        CBranch::Group(CGroup::Field(name))
    }
    fn fields_group(&self) -> CBranch {
        CBranch::Group(CGroup::Fields)
    }
    type Extra = ();
    type Wire = CWire;
    type Context = CContext;
    fn finish_input(&self, plan: &mut InputPlan<CWire>, _: &()) -> Result<()> {
        validate_layout(&plan.node)?;
        if plan
            .resources
            .iter()
            .any(|r| r.access == prebindgen_conversions::Access::Move)
            || prebindgen_flat::wire_definitions(std::iter::once(&plan.node))
                .any(|w| matches!(w.kind, CWire::Callback { .. }))
        {
            plan.acknowledgement = Some(quote!());
        }
        Ok(())
    }
    fn finish_output(&self, plan: &mut OutputPlan<CWire>, _: &()) -> Result<()> {
        validate_layout(&plan.outcomes.success)?;
        plan.commit
            .extend(prebindgen_conversions::template_parameter(
                "output_committed",
            ));
        let mut values: Vec<_> = crate::group::result_packet(&plan.outcomes.success, &plan.values)?
            .into_iter()
            .enumerate()
            .map(|(i, (_, value))| {
                let field = format_ident!("v{i}");
                quote!(#field: #value)
            })
            .collect();
        if values.is_empty() {
            values.push(quote!(unused: 0));
        }
        let result_type = prebindgen_conversions::template_parameter("output_type");
        let out = prebindgen_conversions::template_parameter("output_pointer");
        plan.delivery = prebindgen_conversions::Delivery {
            statements: quote!(let result = #result_type { #(#values),* }; std::ptr::write(#out, result);),
            result: quote!(()),
        };
        Ok(())
    }
    fn sequence_input(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        _: &prebindgen_declarations::Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<CWire, crate::CChain>> {
        sequence::input(wire, length, index, element)
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        _: syn::Ident,
        element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        values: &[(syn::Ident, TokenStream)],
        delivery: prebindgen_declarations::SequenceDelivery,
    ) -> Result<prebindgen_declarations::SequenceOutput<CWire, crate::CChain>> {
        match delivery {
            prebindgen_declarations::SequenceDelivery::Auto
            | prebindgen_declarations::SequenceDelivery::Value => {
                sequence::output(wire, length, element, values)
            }
            prebindgen_declarations::SequenceDelivery::Callback => {
                sequence::callback_output(wire, values, element, length)
            }
        }
    }
    fn result_error(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _: &syn::Type,
        message: TokenStream,
        _: &(),
    ) -> Result<TokenStream> {
        let pointer = prebindgen_conversions::template_parameter("error_pointer");
        let reported = prebindgen_conversions::template_parameter("error_reported");
        Ok(
            quote!({if !#pointer.is_null(){unsafe {__pb_report_message(#message,#pointer)};}#reported=true;String::new()}),
        )
    }
    fn niche(&self, wire: &CWire) -> Option<CNiche> {
        wire.available_niche()
    }
    fn set_niche(&self, wire: &mut CWire, niche: Option<CNiche>) {
        wire.set_niche(niche)
            .expect("C codec supplied an incompatible niche");
    }
    fn is_present(&self, wire: &CWire, niche: &CNiche, value: TokenStream) -> Result<TokenStream> {
        wire.validate_niche(niche)?;
        Ok(niche.present(value))
    }
    fn remaining_niche(&self, wire: &CWire, niche: &CNiche) -> Option<CNiche> {
        wire.remaining_niche(niche)
    }
    fn absent_value(&self, wire: &CWire, niche: Option<&CNiche>, _: bool) -> Result<TokenStream> {
        if let Some(niche) = niche {
            wire.validate_niche(niche)?;
        }
        let buffer_type = prebindgen_conversions::template_parameter("buffer_type");
        let wide = prebindgen_conversions::template_parameter("wide_integer_type");
        Ok(match wire {
            CWire::Callback { callback } => callback.absent_value(),
            CWire::Pointer { pointer, .. } if pointer.access == Access::Shared => {
                quote!(std::ptr::null())
            }
            CWire::Pointer { .. } => quote!(std::ptr::null_mut()),
            CWire::View | CWire::Buffer => quote!(#buffer_type {data:std::ptr::null(),len:0}),
            CWire::Owned {
                owner,
                by_value: true,
                ..
            } => {
                let t = owner.ident();
                quote!(#t::empty())
            }
            CWire::Owned {
                access: Access::Shared,
                ..
            } => quote!(::core::ptr::null()),
            CWire::Owned { .. } => quote!(::core::ptr::null_mut()),
            CWire::Rows => quote!(__PbRows {
                data: std::ptr::null(),
                len: 0
            }),
            CWire::Scalar {
                primitive: Primitive::I128 | Primitive::U128,
                ..
            } => quote!(#wide {hi:0,lo:0}),
            CWire::Scalar {
                primitive: Primitive::F32 | Primitive::F64,
                ..
            } => quote!(0.0),
            CWire::Scalar { .. } | CWire::Enum { .. } => niche
                .map(|n| n.integer_value())
                .unwrap_or_else(|| quote!(0)),
        })
    }
    fn selector(&self, _: &()) -> CWire {
        CWire::scalar(Primitive::I32)
    }
    fn consumes(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
    ) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(_flat, inner, &());
        }
        // Text and bytes own their storage just like opaque objects. Treating
        // them as observations silently cloned them when another view was added.
        !matches!(ty, syn::Type::Reference(_))
            && (owned::native_output(_flat, ty) || !matches!(Value::of(ty), Some(Value::Scalar(_))))
    }

    fn input_leaf(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_declarations::Condition,
    ) -> Result<InputPlan<CWire>> {
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        if let Some(owner) = owned::declaration(_flat, ty).filter(|o| o.explicit) {
            return Ok(owned::input(owner, ty, w, v, active, false));
        }
        if owned::pointer_declared(_flat, ty) {
            return prebindgen_conversions::boxed::input(
                &prebindgen_conversions::boxed::Storage::Direct,
                ty,
                w,
                v,
                &active.rust(),
                |b| CWire::pointer(b.into()),
            );
        }
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        let read_buffer = prebindgen_conversions::template_parameter("read_buffer");
        let (decode, kind) = match Value::of(ty) {
            Some(Value::Scalar(p)) => (decode_scalar(p, &w), CWire::scalar(p)),
            // Text and bytes borrow the caller's buffer; owned targets copy it.
            Some(Value::Text) => (
                quote!(std::str::from_utf8(unsafe {#read_buffer(#w)?}).map_err(|_|"invalid UTF-8")?),
                CWire::Buffer,
            ),
            Some(Value::Bytes) => (quote!(unsafe {#read_buffer(#w)?}), CWire::Buffer),
            None => {
                return prebindgen_conversions::boxed::input(
                    &prebindgen_conversions::boxed::Storage::Direct,
                    ty,
                    w,
                    v,
                    &active.rust(),
                    |b| CWire::pointer(b.into()),
                )
            }
        };
        let mut plan =
            prebindgen_conversions::value::input(ty, w, v, &active.rust(), decode, kind)?;
        plan.fallible = !matches!(Value::of(ty), Some(Value::Scalar(p)) if p != Primitive::Char);
        Ok(plan)
    }
    fn output_leaf(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<CWire>> {
        if let Some(owner) = owned::declaration(_flat, ty) {
            if let syn::Type::Reference(r) = ty {
                let access = if r.mutability.is_some() {
                    Access::Exclusive
                } else {
                    Access::Shared
                };
                let value = if borrowed { quote!(*#v) } else { v };
                return Ok(owned::borrowed(owner, w, value, access));
            }
            if borrowed {
                // An explicit borrowed value projection requests an owning copy.
                return Ok(owned::output(
                    owner,
                    w,
                    quote!(::core::clone::Clone::clone(#v)),
                ));
            }
            return Ok(owned::output(owner, w, v));
        }
        if owned::pointer_declared(_flat, ty) {
            if let syn::Type::Reference(r) = ty {
                return Ok(self
                    .output_return_reference(_flat, r, &(), w, v)?
                    .expect("declared opaque reference"));
            }
            let value = if borrowed {
                quote!(::core::clone::Clone::clone(#v))
            } else {
                v
            };
            return Ok(owned_pointer_output(ty, w, value).0);
        }
        if let syn::Type::Reference(r) = ty {
            if r.mutability.is_none()
                && matches!(prebindgen_flat::type_key(&r.elem).as_str(), "str" | "[u8]")
            {
                return Ok(byte_view(w, v));
            }
        }
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if let syn::Type::Reference(reference) = ty {
            if reference.mutability.is_some() {
                return Err("mutable-reference outputs are unsupported".into());
            }
            match prebindgen_flat::type_key(&reference.elem).as_str() {
                "str" => {
                    return self.output_leaf(
                        _flat,
                        &syn::parse_quote!(String),
                        &(),
                        w,
                        quote!((#v).to_string()),
                        false,
                    )
                }
                "[u8]" => {
                    return self.output_leaf(
                        _flat,
                        &syn::parse_quote!(Vec<u8>),
                        &(),
                        w,
                        quote!((#v).to_vec()),
                        false,
                    )
                }
                _ => {}
            }
            let value = if borrowed { quote!(*#v) } else { v };
            return self.output_leaf(_flat, &reference.elem, &(), w, value, true);
        }
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        match Value::of(ty) {
            Some(Value::Scalar(p)) => {
                let mut plan = prebindgen_conversions::value::output(
                    ty,
                    w,
                    v,
                    borrowed,
                    |v| encode_scalar(p, v),
                    CWire::scalar(p),
                )?;
                plan.fallible = false;
                Ok(plan)
            }
            Some(_) => {
                // The buffer stays owned by Rust until every output is delivered.
                let mut out = prebindgen_conversions::value::output(
                    ty,
                    w.clone(),
                    v,
                    borrowed,
                    |v| quote!(::core::convert::Into::<Vec<u8>>::into(#v).into_boxed_slice()),
                    CWire::Buffer,
                )?;
                out.fallible = false;
                let buffer_type = prebindgen_conversions::template_parameter("buffer_type");
                let guard = format_ident!("{w}_owner");
                out.statements
                    .extend(quote!(let #w = Box::into_raw(#w); let #guard = __PbOutputOwner(#w);));
                out.values = vec![(
                    w.clone(),
                    quote!(#buffer_type {data: #w as *const u8, len: #w.len()}),
                )];
                out.commit = quote!(::core::mem::forget(#guard););
                Ok(out)
            }
            None => {
                let value = if borrowed {
                    quote!(::core::clone::Clone::clone(#v))
                } else {
                    v
                };
                Ok(owned_pointer_output(ty, w, value).0)
            }
        }
    }

    fn output_return_reference(
        &self,
        flat: &Bindings,
        reference: &syn::TypeReference,
        _: &(),
        wire: syn::Ident,
        value: TokenStream,
    ) -> Result<Option<OutputPlan<CWire>>> {
        let ty = &*reference.elem;
        if let Some(owner) = owned::declaration(flat, ty) {
            return Ok(Some(owned::borrowed(
                owner,
                wire,
                value,
                if reference.mutability.is_some() {
                    Access::Exclusive
                } else {
                    Access::Shared
                },
            )));
        }
        if reference.mutability.is_none()
            && matches!(prebindgen_flat::type_key(ty).as_str(), "str" | "[u8]")
        {
            return Ok(Some(byte_view(wire, value)));
        }
        // Scalars, enums and buffers retain their value conversions. Only an
        // opaque source type has an address-preserving C representation.
        if !owned::pointer_declared(flat, ty)
            && (!matches!(ty, syn::Type::Path(_))
                || prebindgen_flat::type_key(ty) == "str"
                || Value::of(ty).is_some()
                || prebindgen_flat::sequence_parts(ty).is_some()
                || prebindgen_flat::option_inner(ty).is_some()
                || flat
                    .type_info(ty)
                    .ok()
                    .is_some_and(|t| t.metadata.enumeration.is_some()))
        {
            return Ok(None);
        }
        let access = if reference.mutability.is_some() {
            Access::Exclusive
        } else {
            Access::Shared
        };
        let kind = CWire::pointer(CPointer {
            ty: ty.clone(),
            access,
        });
        let pointer_type = kind.rust_type();
        Ok(Some(OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            outcomes: prebindgen_flat::Outcomes {
                success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: wire.clone(),
                    kind,
                }]),
                source_error: None,
                binding_failure: None,
            },
            statements: quote!(let #wire: #pointer_type = #value as #pointer_type;),
            values: vec![(wire.clone(), quote!(#wire))],
        }))
    }

    fn output_owner(
        &self,
        flat: &Bindings,
        ty: &syn::Type,
        _: &(),
        wire: syn::Ident,
        value: TokenStream,
    ) -> Result<Option<(OutputPlan<CWire>, TokenStream)>> {
        if let Some(owner) = owned::declaration(flat, ty).filter(|_| owned::std_owner(ty)) {
            let ident = owner.ident();
            let mut plan = owned::output(owner, wire.clone(), value.clone());
            plan.statements = quote!(let mut #wire=#ident::guard(#value););
            let pointer = quote!(::core::ptr::addr_of_mut!(#wire.0).cast::<#ty>());
            return Ok(Some((plan, pointer)));
        }
        if !flat
            .type_info(ty)
            .ok()
            .is_some_and(|t| t.metadata.pointer.is_some() && t.metadata.enumeration.is_none())
        {
            return Ok(None);
        }
        Ok(Some(owned_pointer_output(ty, wire, value)))
    }

    fn validate_owner_getter(
        &self,
        flat: &Bindings,
        ty: &syn::Type,
        getter: &str,
        plan: &OutputPlan<CWire>,
    ) -> Result<()> {
        if let Some(owner) = owned::declaration(flat, ty) {
            let has_loan = crate::group::result_packet_fields(&plan.outcomes.success).any(|w| {
                matches!(
                    w.kind,
                    CWire::View
                        | CWire::Owned {
                            access: Access::Shared | Access::Exclusive,
                            ..
                        }
                ) || matches!(&w.kind,CWire::Pointer{pointer,..} if pointer.access!=Access::Move)
            });
            if has_loan && owner.helper("view") != getter {
                return Err("inline owner getter loans require a generated heap view accessor; a reference into the movable descriptor cannot escape".into());
            }
        }
        Ok(())
    }
    fn validate_getter_output(
        &self,
        _flat: &Bindings,
        _receiver: &syn::Type,
        borrowed: bool,
        ty: &syn::Type,
        plan: &OutputPlan<CWire>,
        _: &(),
    ) -> Result<()> {
        let fields = crate::group::result_packet_fields(&plan.outcomes.success).collect::<Vec<_>>();
        let mut reference_type = ty;
        while let Some(inner) = prebindgen_flat::option_inner(reference_type) {
            reference_type = inner;
        }
        if matches!(reference_type,syn::Type::Reference(r) if r.lifetime.as_ref().is_some_and(|l| l.ident=="static"))
        {
            return Ok(());
        }
        let loans = fields.iter().any(|w| {
            matches!(
                &w.kind,
                CWire::View
                    | CWire::Owned {
                        access: Access::Shared | Access::Exclusive,
                        ..
                    }
            ) || matches!(&w.kind,CWire::Pointer{pointer,..} if pointer.access != Access::Move)
        });
        if loans && !borrowed {
            return Err("a borrowed getter output requires its opaque owner to be returned with field_self_into(); a temporary owner cannot lend output storage; use a generated heap view accessor with its inline owner, or borrow an owner already stored by C".into());
        }
        Ok(())
    }

    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<prebindgen_declarations::NicheReservation<Self::Niche>> {
        crate::group::optional_niche(payload)
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &prebindgen_declarations::NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        crate::group::optional_presence(payload, reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        crate::group::absent_output_values(payload, reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        crate::group::retire_niches(payload, reservation)
    }
    fn validate_input(&self, plan: &prebindgen_declarations::CodecInput<Self>) -> Result<()> {
        sequence::validate_buffered(&plan.node)?;
        crate::group::validate_layout(&plan.node)
    }
    fn validate_output(&self, plan: &prebindgen_declarations::CodecOutput<Self>) -> Result<()> {
        sequence::callback_delivery(&plan.outcomes.success)?;
        for error in plan
            .outcomes
            .source_error
            .iter()
            .chain(plan.outcomes.binding_failure.iter())
        {
            sequence::validate_buffered(error)?;
        }
        for node in std::iter::once(&plan.outcomes.success)
            .chain(plan.outcomes.source_error.iter())
            .chain(plan.outcomes.binding_failure.iter())
        {
            crate::group::validate_layout(node)?;
        }
        crate::group::result_packet(&plan.outcomes.success, &plan.values)?;
        Ok(())
    }
}
// Every C opaque allocation has the direct Box<T> layout. Keep ownership in a
// raw-pointer guard so packet delivery and commit never retag a live Box after
// its address (or an interior address) has been placed in a result packet.
fn owned_pointer_output(
    ty: &syn::Type,
    wire: syn::Ident,
    value: TokenStream,
) -> (OutputPlan<CWire>, TokenStream) {
    let guard = format_ident!("{wire}_owner");
    (
        OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            outcomes: prebindgen_flat::Outcomes {
                success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: wire.clone(),
                    kind: CWire::pointer(CPointer {
                        ty: ty.clone(),
                        access: Access::Move,
                    }),
                }]),
                source_error: None,
                binding_failure: None,
            },
            statements: quote!(let #wire: *mut #ty = Box::into_raw(Box::new(#value)); let #guard = __PbOutputOwner(#wire);),
            values: vec![(wire.clone(), quote!(#wire))],
            commit: quote!(::core::mem::forget(#guard);),
        },
        quote!(#wire),
    )
}
fn decode_scalar(p: Primitive, w: &syn::Ident) -> TokenStream {
    let ty = p.rust_type();
    match p {
        Primitive::Bool => quote!(#w != 0),
        Primitive::Char => quote!(char::from_u32(#w).ok_or("invalid Unicode scalar")?),
        Primitive::I128 | Primitive::U128 => {
            quote!(((u128::from(#w.hi) << 64) | u128::from(#w.lo)) as #ty)
        }
        _ => quote!(#w),
    }
}
fn encode_scalar(p: Primitive, v: TokenStream) -> TokenStream {
    let wide_integer_type = prebindgen_conversions::template_parameter("wide_integer_type");
    match p {
        Primitive::Bool => quote!(u8::from(#v)),
        Primitive::Char => quote!((#v) as u32),
        Primitive::I128 | Primitive::U128 => {
            quote!({ let bits = (#v) as u128; #wide_integer_type { hi: (bits >> 64) as u64, lo: bits as u64 } })
        }
        _ => v,
    }
}
/// Configure C bindings, using typed opaque pointers for foreign values.
///
/// Start with captured source via [`Self::builder`] and [`Self::source`], or a
/// flat model via [`Self::new`]. With captured source, declare each
/// exported function. In either case, declare each opaque pointee type.
/// Conversion declarations use [`prebindgen_declarations::Declarations`]. When reading
/// captured source, add the source crate as both a build dependency and a normal
/// dependency: generation reads its captures, and generated code calls its functions.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen};
/// use prebindgen_declarations::fun;
/// let source = syn::parse_file("pub fn add(a: i64, b: i64) -> i64 { a + b }")?;
/// let flat = Bindings::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// let generated = Cbindgen::new(flat)
///     .prefix("demo")
///     .function(fun!(add))
///     .build()?;
/// assert!(generated.header.contains("demo_add("));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Non-Result calls return their first boundary value directly and write any
/// additional values through output pointers. Unit returns void. Fallible
/// conversions require explicit [`Self::panic`] and abort on failure; source
/// panics also abort. Destructors return void. Result calls report failures
/// through their explicit error output.
/// Each owned input transfers after its own validation and alias checks.
/// Later failures do not restore consumed inputs. References borrow for the
/// call and require the caller to obey Rust aliasing and lifetime rules.
/// Result calls use an optional explicit error output, with owned messages or a
/// declared error record. A success wire with an available niche is returned
/// directly; otherwise the return is bool and success values use output pointers.
/// Unit success needs no output pointer. Consuming Result calls additionally
/// require a `size_t` ownership output counting consumed owned values in
/// argument, row and field order, including on failure.
/// Owned returned pointers, buffers and errors require their matching destructors.
/// Direct reference returns of opaque types use non-owning pointers: `const` for
/// shared references and mutable for exclusive references. Options use null for absence.
/// Never destroy or consume these pointers; keep their
/// source storage alive and at a stable address, respecting Rust's aliasing rules.
/// Mutable loans require exclusive access while in use. Getter expansions retain
/// their existing ownership policy; mutable getter results remain unsupported.
pub struct Cbindgen {
    flat: Option<Bindings>,
    source: Option<std::path::PathBuf>,
    inputs: Vec<prebindgen_declarations::Input<InputRequest>>,
    outputs: Vec<prebindgen_declarations::Output<OutputRequest>>,
    error_mappers: Vec<(syn::Type, String)>,
    functions: Vec<prebindgen_declarations::FunctionDecl>,
    prefix: String,
    pointers: Vec<(String, syn::Type)>,
    owned: Vec<(String, syn::Type, OpaqueLayout, Option<String>)>,
    layout_file: Option<std::path::PathBuf>,
    layout_data: Option<OpaqueLayouts>,
    layout_sources: Vec<prebindgen::Source>,
    target: Option<String>,
    enums: Vec<(String, syn::Type)>,
    errors: Vec<(String, syn::Type)>,
    panics: std::collections::BTreeSet<String>,
}
impl Cbindgen {
    /// Create an unconfigured builder with the symbol prefix `pb`.
    pub fn builder() -> Self {
        Self {
            flat: None,
            source: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            error_mappers: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            pointers: Vec::new(),
            owned: Vec::new(),
            layout_file: None,
            layout_data: None,
            layout_sources: Vec::new(),
            target: None,
            enums: Vec::new(),
            errors: Vec::new(),
            panics: Default::default(),
        }
    }
    /// Read captures from a source crate's `PREBINDGEN_OUT_DIR`, passed as `path`.
    ///
    /// Only declared functions, referenced constructors/getters and required types
    /// are selected. Automatic type discovery follows unqualified source names
    /// (see [`Flat::parse_selected`]). Types referenced only by qualified paths
    /// can be selected through [`Self::opaque_ptr`] using their unqualified names.
    /// Supply exactly one source directory or flat model.
    pub fn source(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.source = Some(path.as_ref().to_owned());
        self
    }
    /// Register an input/output default using the production declaration syntax.
    /// The builder is consumed directly into its Input/Output tree.
    pub fn expand(self, decl: impl Into<prebindgen_declarations::ExpandDecl>) -> Self {
        use prebindgen_declarations::ExpandDecl;
        match decl.into() {
            ExpandDecl::Param(decl) => self.input(decl),
            ExpandDecl::Return(decl) => self.output(decl),
            ExpandDecl::Error(decl) => self.error_mapper(decl.ty, decl.mapper),
        }
    }
    /// Register a type-wide input or output expansion, built with `expand_param!`
    /// or `expand_return!`.
    ///
    /// Per-function overrides replace this default; duplicate defaults are rejected
    /// by [`Self::build`]. Registration order does not affect type lookup.
    pub fn input(mut self, tree: impl Into<prebindgen_declarations::Input<InputRequest>>) -> Self {
        self.inputs.push(tree.into());
        self
    }
    /// Register a type-wide direct output conversion tree.
    pub fn output(
        mut self,
        tree: impl Into<prebindgen_declarations::Output<OutputRequest>>,
    ) -> Self {
        self.outputs.push(tree.into());
        self
    }
    /// Select caller-owned C-string errors (`char **error`) for Result returns
    /// using `ty`, with a default source error-message mapper.
    /// The helper takes exactly E or &E and returns String. A function's
    /// `on_error(...)` overrides this default. Duplicate resolved types are rejected.
    pub fn error_mapper(
        mut self,
        ty: syn::Type,
        mapper: impl Into<prebindgen_declarations::FunctionDecl>,
    ) -> Self {
        self.error_mappers.push((ty, mapper.into().source_name()));
        self
    }

    /// Deliver this declared record error by value through an optional C
    /// error output pointer. Its fields use ordinary output declarations. The
    /// source type must implement `From<String>` for binding failures.
    pub fn error(mut self, name: &str, ty: syn::Type) -> Self {
        self.errors.push((name.into(), ty));
        self
    }
    /// Deliver an opaque source error as a caller-owned C string through
    /// `char **error`. The mapper takes E or &E and returns String.
    pub fn opaque_error(
        self,
        ty: syn::Type,
        mapper: impl Into<prebindgen_declarations::FunctionDecl>,
    ) -> Self {
        self.error_mapper(ty, mapper)
    }

    /// Export a function and build its occurrence converters from declarations.
    ///
    /// `decl` names the function, normally with `fun!`, and carries its overrides.
    ///
    /// Function renaming is currently unsupported; exports use the symbol prefix
    /// and original source name.
    pub fn function(mut self, decl: prebindgen_declarations::FunctionDecl) -> Self {
        self.functions.push(decl);
        self
    }

    /// Allow binding conversion errors to abort the last declared non-Result
    /// function. Required for fallible conversions without a Result error channel.
    /// Result functions continue to report errors through their explicit output.
    /// Source panics always abort non-Result calls at the C boundary.
    ///
    /// # Panics
    /// Panics if no function has been declared.
    pub fn panic(mut self) -> Self {
        let name = self
            .functions
            .last()
            .expect("panic() must follow function(...)")
            .name
            .clone();
        self.panics.insert(name);
        self
    }

    /// Start from a flat model with input and output conversion slots and prefix `pb`.
    ///
    /// `flat` is the parsed source, with any converters already attached.
    ///
    /// Attach converters with [`prebindgen_declarations::bind_input`] and
    /// [`prebindgen_declarations::bind_output`] with [`CCodec`], or fill defaults with
    /// [`Self::function`].
    /// Every fully bound function in the supplied model is exported.
    pub fn new(flat: Bindings) -> Self {
        Self {
            flat: Some(flat),
            source: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            error_mappers: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            pointers: Vec::new(),
            owned: Vec::new(),
            layout_file: None,
            layout_data: None,
            layout_sources: Vec::new(),
            target: None,
            enums: Vec::new(),
            errors: Vec::new(),
            panics: Default::default(),
        }
    }
    /// Set the prefix for exported functions, C types and destructors.
    ///
    /// Defaults to `pb`. The prefix is validated as an identifier during build.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    /// Select the binding target outside Cargo, or override Cargo's `TARGET`.
    /// Native measurements are used only for the generator's own platform.
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
    /// Read external target layouts during preparation. An incomplete artifact
    /// is an error even on native builds; no host fallback is used.
    pub fn opaque_layouts_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.layout_file = Some(path.as_ref().to_owned());
        self
    }
    /// Supply a parsed layout artifact instead of a file.
    pub fn opaque_layouts(mut self, data: OpaqueLayouts) -> Self {
        self.layout_data = Some(data);
        self
    }
    /// Register a captured source for layout-artifact feature verification when
    /// starting from a parsed Flat model. `.source(...)` registers automatically.
    pub fn layout_source(mut self, source: prebindgen::Source) -> Self {
        self.layout_sources.push(source);
        self
    }
    /// Declare an opaque C pointee and its typed destructor for a source type.
    ///
    /// Selects `Box<T>` storage for owned values in both directions, including
    /// built-in and vector types that otherwise have value/collection defaults.
    /// Named opaque types need an explicit pointer or inline-owner declaration.
    /// There is no `Send` requirement. This alone does not export constructors.
    ///
    /// # Arguments
    ///
    /// * `name` — the C name, without the prefix: the header declares the type
    ///   `{prefix}_{name}` and the destructor `{prefix}_drop_{name}`.
    /// * `ty` — the source type behind the pointer, as written in the source crate.
    pub fn opaque_ptr(mut self, name: &str, ty: syn::Type) -> Self {
        self.pointers.push((name.into(), ty));
        self
    }
    /// Declare an inline owning C value with generated target storage and drop.
    /// `T::default()` must be a valid, resource-free empty value: the C caller
    /// may discard it after a drop or consuming call. Use
    /// [`Self::opaque_owned_struct_with_empty`] when another empty factory is needed.
    /// Types without a resource-free empty state should use [`Self::opaque_ptr`].
    pub fn opaque_owned_struct(mut self, name: &str, ty: syn::Type, layout: OpaqueLayout) -> Self {
        self.owned.push((name.into(), ty, layout, None));
        self
    }
    /// Declare an inline owner whose empty value is produced by a nullary source
    /// function returning exactly this Rust type. The empty value must own no
    /// resources, because callers may discard it after destruction or consumption.
    /// This avoids a `Default` bound.
    pub fn opaque_owned_struct_with_empty(
        mut self,
        name: &str,
        ty: syn::Type,
        layout: OpaqueLayout,
        empty: prebindgen_declarations::FunctionDecl,
    ) -> Self {
        self.owned
            .push((name.into(), ty, layout, Some(empty.source_name())));
        self
    }
    /// Reference the generated length getter in an output expansion.
    pub fn owned_len(name: &str) -> prebindgen_declarations::FunctionDecl {
        prebindgen_declarations::FunctionDecl::new(owned::helper(name, "len"))
    }
    /// Reference the generated byte-view getter for a `String` or `Vec<u8>` owner.
    pub fn owned_view(name: &str) -> prebindgen_declarations::FunctionDecl {
        prebindgen_declarations::FunctionDecl::new(owned::helper(name, "view"))
    }
    /// Bind a fieldless source enum as a named `int32_t` C type and constants.
    /// Source integer discriminants are preserved and checked on input. Register
    /// the type explicitly; `name` is the unprefixed C typedef/constant prefix.
    pub fn enum_type(mut self, name: &str, ty: syn::Type) -> Self {
        self.enums.push((name.into(), ty));
        self
    }
    /// Resolve declarations and converters into a complete Flat model.
    ///
    /// Does not write files. Returns an error for conflicting sources, invalid or
    /// duplicate declarations, unsupported conversions, or missing opaque pointees.
    /// Rust layout/trait requirements are checked when the generated Rust compiles.
    pub fn prepare(self) -> Result<Bindings> {
        prebindgen_declarations::ident(&self.prefix)?;
        let target = self.target.unwrap_or_else(|| {
            std::env::var("TARGET").unwrap_or_else(|_| env!("PREBINDGEN_C_GENERATOR_TARGET").into())
        });
        if target.trim().is_empty() {
            return Err("binding target must not be empty".into());
        }
        let target_info = prebindgen::TargetTriple::parse(&target)
            .map_err(|e| format!("invalid binding target `{target}`: {e}"))?;
        let pointer_width = target_info
            .pointer_width()
            .ok_or("binding target has no pointer width")?;
        let artifact = match (self.layout_file, self.layout_data) {
            (Some(_), Some(_)) => {
                return Err("supply one opaque layout artifact, as a file or parsed data".into())
            }
            (Some(path), None) => Some(OpaqueLayouts::read_json(path)?),
            (None, data) => data,
        };
        let mut layout_sources = self.layout_sources;
        let mut flat = match (self.flat, self.source) {
            (Some(flat), None) => flat,
            (None, Some(path)) => {
                let source = prebindgen::Source::builder(path)
                    .enable_target_filtering(Some(&target))
                    .build();
                let flat = Flat::parse_selected(
                    source.items_all(),
                    &prebindgen_declarations::declarations::source_functions(
                        &self.inputs,
                        &self.outputs,
                        &self.error_mappers,
                        &self.functions,
                    )
                    .into_iter()
                    .chain(
                        self.owned
                            .iter()
                            .filter_map(|(_, _, _, empty)| empty.clone()),
                    )
                    .collect::<Vec<_>>(),
                    &self
                        .pointers
                        .iter()
                        .chain(&self.enums)
                        .chain(&self.errors)
                        .map(|(_, ty)| ty.clone())
                        .chain(self.error_mappers.iter().map(|(ty, _)| ty.clone()))
                        .chain(self.owned.iter().map(|(_, ty, _, _)| ty.clone()))
                        .collect::<Vec<_>>(),
                )?;
                layout_sources.push(source);
                flat
            }
            _ => return Err("supply exactly one source or flat model".into()),
        };
        flat.metadata.prefix = self.prefix;
        flat.metadata.target = Some((target.clone(), pointer_width, target_info.to_cfg_tokens()));
        let mut enum_names = std::collections::BTreeSet::new();
        for (name, ty) in self.enums {
            prebindgen_declarations::ident(&name)?;
            if !enum_names.insert(name.clone()) {
                return Err("duplicate C enum name".into());
            }
            let ty = flat.resolve_type(&ty)?;
            let source = flat
                .enums
                .get(&prebindgen_flat::type_key(&ty))
                .ok_or("enum_type requires a fieldless enum declaration")?;
            let mapping = std::rc::Rc::new(prebindgen_declarations::EnumMapping::new(source)?);
            for (variant, _) in &mapping.variants {
                prebindgen_declarations::ident(&format!(
                    "v_{}",
                    variant.to_string().trim_start_matches("r#")
                ))?;
            }
            flat.guards.extend(mapping.guards());
            let info = flat.type_info_mut(&ty)?;
            if info
                .metadata
                .enumeration
                .replace(std::rc::Rc::new(CEnum { name, mapping }))
                .is_some()
            {
                return Err("duplicate C enum source type".into());
            }
        }
        let mut error_names = std::collections::BTreeSet::new();
        for (order, (name, ty)) in self.errors.into_iter().enumerate() {
            prebindgen_declarations::ident(&name)?;
            if !error_names.insert(name.clone()) {
                return Err("duplicate C error declaration".into());
            }
            let ty = flat.resolve_type(&ty)?;
            let info = flat.type_metadata_mut(&ty)?;
            if info
                .error
                .replace(std::rc::Rc::new(errors::ErrorBinding::declared(name, ty)))
                .is_some()
            {
                return Err("duplicate C error declaration".into());
            }
            info.error_order = order;
        }
        for (ty, _) in &self.error_mappers {
            if flat
                .type_info(&flat.resolve_type(ty)?)
                .ok()
                .is_some_and(|t| t.metadata.error.is_some())
            {
                return Err("structured C errors cannot also select a message mapper".into());
            }
        }
        for (order, (name, ty)) in self.pointers.into_iter().enumerate() {
            prebindgen_declarations::ident(&name)?;
            let ty = flat.resolve_type(&ty)?;
            owned::validate_type(&ty)?;
            let info = flat.type_metadata_mut(&ty)?;
            if info.owned.is_some() || info.enumeration.is_some() {
                return Err(
                    "a source type cannot be both opaque_ptr and opaque_owned_struct".into(),
                );
            }
            if info.pointer.replace(name).is_some() {
                return Err("duplicate C opaque source type".into());
            }
            info.pointer_order = order;
        }
        for (name, ty, layout, empty) in self.owned {
            prebindgen_declarations::ident(&name)?;
            let ty = flat.resolve_type(&ty)?;
            owned::validate_type(&ty)?;
            let empty = if let Some(function) = empty {
                let f = flat.function(&function)?;
                if !f.params.is_empty() || !prebindgen_flat::same_type(&f.ret.ty, &ty) {
                    return Err(
                        "an opaque owner empty factory must be nullary and return its owner type"
                            .into(),
                    );
                }
                let path = &f.path;
                quote!(#path())
            } else {
                quote!(<#ty as ::core::default::Default>::default())
            };
            let info = flat.type_metadata_mut(&ty)?;
            if info.pointer.is_some() || info.owned.is_some() || info.enumeration.is_some() {
                return Err("duplicate or conflicting C owning representation".into());
            }
            info.owned = Some(std::rc::Rc::new(COwned {
                name,
                ty,
                layout,
                empty,
                explicit: true,
            }));
        }
        let mut automatic = std::collections::BTreeMap::new();
        for f in flat.functions.values() {
            owned::collect(&f.ret.ty, &mut automatic);
            for p in &f.params {
                owned::collect(&p.ty.ty, &mut automatic);
            }
        }
        for record in flat.records.values() {
            for field in &record.fields {
                owned::collect(&field.ty, &mut automatic);
            }
        }
        for ty in automatic.into_values() {
            let info = flat.type_metadata_mut(&ty)?;
            if info.owned.is_none() && info.pointer.is_none() {
                info.owned = Some(std::rc::Rc::new(COwned {
                    name: owned::auto_name(&ty),
                    ty: ty.clone(),
                    layout: layouts::builtin(&ty),
                    empty: quote!(<#ty as ::core::default::Default>::default()),
                    explicit: false,
                }));
            }
        }
        owned::accessors(&mut flat)?;
        let declarations = prebindgen_declarations::Declarations::new(
            &flat,
            self.inputs,
            self.outputs,
            self.error_mappers,
        )?;
        let codec = enums::EnumCodec::default();
        let mut names = std::collections::BTreeSet::new();
        for decl in &self.functions {
            if !names.insert(decl.name.clone()) {
                return Err("duplicate C function declaration".into());
            }
            if decl.foreign.is_some() {
                return Err("C function renaming is outside this subset".into());
            }
            flat.function_mut(&decl.name)?.metadata = Some(CFunction {
                abort_on_failure: self.panics.contains(&decl.name),
                requires_abort: false,
            });
        }
        errors::prepare(&mut flat, &codec, &declarations)
            .map_err(|e| format!("C error preparation: {e}"))?;
        for decl in self.functions {
            declarations.fill(&mut flat, &codec, &decl, &())?;
            if decl.callback_return.is_some() {
                let name = format!("{}_each", decl.name);
                let variant =
                    declarations.callback_variant(&mut flat, &codec, &decl, &name, &())?;
                declarations.fill(&mut flat, &codec, &variant, &())?;
                sequence::callback_target(
                    &(flat.function(&name)?.output_conversion()?)
                        .output_outcomes()
                        .success,
                )?;
                if decl.callback_return == Some(prebindgen_declarations::CallbackReturn::Only) {
                    let original = flat.functions.get_mut(&decl.name).unwrap();
                    original.ret.slot = None;
                    for param in &mut original.params {
                        param.ty.slot = None;
                    }
                }
            }
        }
        prebindgen_flat::validate_bindings(&flat)?;
        for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
            let is_result = prebindgen_declarations::result_parts(&f.ret.ty).is_some();
            if !is_result
                && !f.metadata.as_ref().is_some_and(|m| m.abort_on_failure)
                && (f
                    .params
                    .iter()
                    .filter_map(|p| p.ty.slot.as_ref())
                    .any(|p| p.fallible || !p.resources.is_empty())
                    || f.ret.slot.as_ref().is_some_and(|p| p.fallible))
            {
                let declaration = f.path.segments.last().unwrap().ident.to_string();
                return Err(format!("C function `{}` has a fallible binding conversion without Result; declare .panic() after .function(fun!({declaration})) to allow aborting", f.name));
            }
        }
        for f in flat.functions.values_mut() {
            if f.ret.slot.is_some() {
                let requires_abort = f
                    .params
                    .iter()
                    .filter_map(|p| p.ty.slot.as_ref())
                    .any(|p| p.fallible || !p.resources.is_empty())
                    || f.ret.slot.as_ref().is_some_and(|p| p.fallible);
                f.metadata
                    .get_or_insert_with(Default::default)
                    .requires_abort = requires_abort;
            }
        }
        layouts::prepare(&mut flat, &target, artifact.as_ref(), &layout_sources)?;
        Ok(flat)
    }
    /// Prepare a complete Flat model, then emit its C and Rust bindings.
    pub fn build(self) -> Result<Generation> {
        generate(&self.prepare()?)
    }
}
/// Generated Rust bindings and their corresponding C header.
pub struct Generation {
    /// Formatted Rust source to include in a binding crate with `prebindgen-runtime`
    /// and `konst` (used by captured source feature guards).
    pub rust: String,
    /// C declarations matching the generated Rust ABI, with C++ linkage guards.
    pub header: String,
}
impl Generation {
    /// Write Rust source to exactly `path`, replacing any existing file.
    ///
    /// Relative paths use the current directory, not `OUT_DIR`. Parent directories
    /// must already exist. Returns filesystem errors as messages.
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.rust).map_err(|e| e.to_string())
    }
    /// Write the C header to exactly `path`, replacing any existing file.
    ///
    /// Uses the same path and error rules as [`Self::write_rust`].
    pub fn write_header(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.header).map_err(|e| e.to_string())
    }
}
fn byte_view(wire: syn::Ident, value: TokenStream) -> OutputPlan<CWire> {
    let buffer = prebindgen_conversions::template_parameter("buffer_type");
    OutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: CWire::View,
            }]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(let #wire = #value; let #wire = #buffer {data:#wire.as_ptr().cast::<u8>(),len:#wire.len()};),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    }
}
fn c_scalar(p: Primitive, prefix: &str) -> String {
    match p {
        Primitive::Bool | Primitive::U8 => "uint8_t".into(),
        Primitive::Char | Primitive::U32 => "uint32_t".into(),
        Primitive::I8 => "int8_t".into(),
        Primitive::I16 => "int16_t".into(),
        Primitive::I32 => "int32_t".into(),
        Primitive::I64 => "int64_t".into(),
        Primitive::U16 => "uint16_t".into(),
        Primitive::U64 => "uint64_t".into(),
        Primitive::Isize => "intptr_t".into(),
        Primitive::Usize => "uintptr_t".into(),
        Primitive::F32 => "float".into(),
        Primitive::F64 => "double".into(),
        Primitive::I128 | Primitive::U128 => format!("{prefix}_int128"),
    }
}
fn c_ty(
    wire: &Wire,
    prefix: &str,
    pointers: &std::collections::BTreeMap<String, prebindgen_flat::TypeInfo<CType>>,
    context: &CContext,
) -> String {
    match &wire.kind {
        CWire::Callback { callback } => callback.name.clone(),
        CWire::Enum { name, .. } => format!("{prefix}_{name}"),
        CWire::Scalar { primitive: p, .. } => c_scalar(*p, prefix),
        CWire::Pointer { pointer: p, .. } => {
            let name = pointers
                .values()
                .find(|t| prebindgen_flat::same_type(&t.ty, &p.ty))
                .and_then(|t| t.metadata.pointer.as_ref())
                .expect("validated pointer declaration");
            let qualifier = if p.access == Access::Shared {
                "const "
            } else {
                ""
            };
            format!("{qualifier}{prefix}_{name} *")
        }
        CWire::View => format!("{prefix}_buffer_view"),
        CWire::Owned {
            owner,
            access,
            by_value,
        } => {
            if *by_value {
                format!("{prefix}_{}", owner.name)
            } else {
                format!(
                    "{}{prefix}_{} *",
                    if *access == Access::Shared {
                        "const "
                    } else {
                        ""
                    },
                    owner.name
                )
            }
        }
        CWire::Buffer => format!("{prefix}_buffer"),
        CWire::Rows => format!(
            "{prefix}_{}_rows",
            context.row_types[&wire.name.to_string()]
                .to_string()
                .trim_start_matches("__PbRow_")
        ),
    }
}
fn validate_wire_declaration(
    wire: &Wire,
    pointers: &std::collections::BTreeMap<String, prebindgen_flat::TypeInfo<CType>>,
    enums: &[(&String, &prebindgen_declarations::EnumMapping)],
) -> Result<()> {
    if let CWire::Enum { name, mapping, .. } = &wire.kind {
        if !enums.iter().any(|(declared, m)| {
            *declared == name
                && prebindgen_flat::same_type(&m.ty, &mapping.ty)
                && m.variants
                    .iter()
                    .map(|(n, v)| (n.to_string(), v))
                    .eq(mapping.variants.iter().map(|(n, v)| (n.to_string(), v)))
        }) {
            return Err("declare a matching C enum type for this wire".into());
        }
    }
    if let CWire::Owned { owner, .. } = &wire.kind {
        if !pointers.values().any(|t| {
            t.metadata.owned.as_ref().is_some_and(|o| {
                o.name == owner.name
                    && o.layout == owner.layout
                    && prebindgen_flat::same_type(&o.ty, &owner.ty)
            })
        }) {
            return Err("declare a matching opaque_owned_struct for this wire".into());
        }
    }
    if let CWire::Pointer { pointer, .. } = &wire.kind {
        if !pointers
            .values()
            .any(|t| t.metadata.pointer.is_some() && prebindgen_flat::same_type(&t.ty, &pointer.ty))
        {
            return Err(format!(
                "declare a C opaque pointer for `{}`",
                prebindgen_flat::type_key(&pointer.ty)
            ));
        }
    }
    Ok(())
}

/// Emit C/Rust bindings using only the prepared Flat model.
/// No builder, codec cache or separate declaration table is needed.
pub fn generate<S>(flat: &Flat<S>) -> Result<Generation>
where
    S: prebindgen_flat::Schema<
        Module = CModule,
        Function = Option<CFunction>,
        Type = CType,
        Field = (),
        Variant = (),
    >,
    S::Input: for<'a> prebindgen_flat::InputConversion<
        Wire = CWire,
        Chain = CChain,
        Branch = CBranch,
        SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, CContext>,
    >,
    S::Output: for<'a> prebindgen_flat::OutputConversion<
        Wire = CWire,
        Chain = CChain,
        Branch = CBranch,
        SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, CContext>,
    >,
{
    let prefix = &flat.metadata.prefix;
    prebindgen_declarations::ident(prefix)?;
    let pointers = &flat.types;
    let enums = flat
        .types
        .values()
        .filter_map(|t| t.metadata.enumeration.as_deref())
        .map(|e| (&e.name, e.mapping.as_ref()))
        .collect::<Vec<_>>();
    let mut error_entries = flat
        .types
        .values()
        .filter_map(|t| {
            t.metadata
                .error
                .as_deref()
                .map(|e| (t.metadata.error_order, e))
        })
        .collect::<Vec<_>>();
    error_entries.sort_by_key(|(order, _)| *order);
    let errors = error_entries
        .into_iter()
        .map(|(_, e)| e)
        .collect::<Vec<_>>();
    let mut pointer_entries = flat
        .types
        .values()
        .filter_map(|t| {
            t.metadata
                .pointer
                .as_ref()
                .map(|name| (t.metadata.pointer_order, name, &t.ty))
        })
        .collect::<Vec<_>>();
    pointer_entries.sort_by_key(|(order, _, _)| *order);
    let mut calls = Vec::new();
    let free = format_ident!("{prefix}_free_buffer");
    let free_error = format_ident!("{prefix}_free_error");
    let mut symbols = std::collections::BTreeSet::from([
        free.to_string(),
        free_error.to_string(),
        format!("{prefix}_buffer_view"),
        format!("{prefix}_buffer"),
        format!("{prefix}_int128"),
        format!("{prefix}_H"),
        format!("{prefix}_TARGET"),
    ]);
    for (name, mapping) in &enums {
        prebindgen_declarations::ident(name)?;
        if !symbols.insert(format!("{prefix}_{name}")) {
            return Err("duplicate C enum symbol".into());
        }
        for (variant, _) in &mapping.variants {
            let variant = variant.to_string();
            if !symbols.insert(format!(
                "{prefix}_{name}_{}",
                variant.trim_start_matches("r#")
            )) {
                return Err("duplicate C enum constant".into());
            }
        }
    }
    for name in pointers
        .values()
        .filter_map(|t| t.metadata.pointer.as_ref())
    {
        prebindgen_declarations::ident(name)?;
        if !symbols.insert(format!("{prefix}_{name}")) {
            return Err("C enum and pointer names collide".into());
        }
        if !symbols.insert(format!("{prefix}_drop_{name}")) {
            return Err("duplicate C destructor".into());
        }
    }
    for f in flat.functions.values() {
        if f.ret.slot.is_none() {
            continue;
        }
        let metadata = f
            .metadata
            .as_ref()
            .ok_or_else(|| format!("bound C function `{}` has no export metadata", f.name))?;
        if metadata.requires_abort
            && !metadata.abort_on_failure
            && prebindgen_flat::result_parts(&f.ret.ty).is_none()
        {
            return Err(format!(
                "C function `{}` requires an explicit abort policy",
                f.name
            ));
        }
        for slot in f.params.iter().filter_map(|p| p.ty.slot.as_ref()) {
            sequence::validate_buffered(slot.input_node())?;
            validate_layout(slot.input_node())?;
        }
        if let Some(slot) = f.ret.slot.as_ref() {
            let outcomes = slot.output_outcomes();
            for node in std::iter::once(&outcomes.success)
                .chain(outcomes.source_error.iter())
                .chain(outcomes.binding_failure.iter())
            {
                validate_layout(node)?;
            }
        }
        let result_type = format_ident!("{prefix}_{}_result", f.name);
        if !symbols.insert(result_type.to_string()) {
            return Err("duplicate C result type symbol".into());
        }
        let is_result = prebindgen_declarations::result_parts(&f.ret.ty).is_some();
        let outcomes = f.output_conversion()?.output_outcomes();
        validate_return_loans(f, &outcomes.success)?;
        for error in outcomes
            .source_error
            .iter()
            .chain(outcomes.binding_failure.iter())
        {
            sequence::validate_buffered(error)?;
        }
        let has_source_error = outcomes.source_error.is_some();
        let has_binding_failure = outcomes.binding_failure.is_some();
        if is_result != has_source_error {
            return Err("C source-error schema must match the declared Result return".into());
        }
        // Recoverable Result wrappers require a diagnostic channel. Other C
        // wrappers retain main's abort policy and may omit it in manual schemas.
        if is_result && !has_binding_failure {
            return Err("recoverable C delivery requires a binding-failure schema".into());
        }
        let context = CContext {
            row_types: {
                let mut schemas = Vec::new();
                for slot in f.input_conversions()? {
                    sequence::schemas(slot.input_node(), &mut schemas);
                }
                sequence::schemas(
                    &f.output_conversion()?.output_outcomes().success,
                    &mut schemas,
                );
                schemas
                    .into_iter()
                    .map(|(wire, _, _)| {
                        (
                            wire.name.to_string(),
                            format_ident!("__PbRow_{}_{}", f.name, wire.name),
                        )
                    })
                    .collect()
            },
            acknowledgement: if is_result {
                format_ident!("ownership_accepted")
            } else {
                format_ident!("_ownership_accepted")
            },
            output_pointer: format_ident!("output"),
            output_type: syn::parse_quote!(#result_type),
            error_pointer: format_ident!("error"),
            error_reported: format_ident!("error_reported"),
            output_committed: if prebindgen_declarations::result_parts(&f.ret.ty).is_some() {
                quote!(success_committed = true;)
            } else {
                TokenStream::new()
            },
            buffer_type: syn::parse_quote!(__PbBuffer),
            wide_integer_type: syn::parse_quote!(__Pb128),
            read_buffer: syn::parse_quote!(__pb_read),
        };
        let source = format_ident!("source_value");
        let body = prebindgen_conversions::staged::emit_call(
            f,
            &context,
            &source,
            &syn::parse_quote!(()),
            &crate::group::function_parameters(f)?
                .map(|w| w.name.clone())
                .collect::<Vec<_>>(),
        )?;
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        let mut all_wires = crate::group::function_parameters(f)?
            .chain(crate::group::result_packet_fields(
                &f.output_conversion()?.output_outcomes().success,
            ))
            .collect::<Vec<_>>();
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.input_node(), &mut schemas);
        }
        sequence::schemas(
            &f.output_conversion()?.output_outcomes().success,
            &mut schemas,
        );
        for (_, element, _) in schemas {
            all_wires.extend(crate::group::row_fields(element));
        }
        for wire in all_wires {
            validate_wire_declaration(wire, pointers, &enums)?;
        }
        calls.push((f, body, context));
    }
    let helpers = &flat.metadata.helpers;
    let mut tokens = quote! {
       #(#helpers)*
       // Raw ownership preserves the allocation address and provenance while
       // views are derived. Forgetting this guard commits the existing allocation.
       #[allow(dead_code)] struct __PbOutputOwner<T: ?Sized>(*mut T);
       impl<T: ?Sized> Drop for __PbOutputOwner<T> {
           fn drop(&mut self) { unsafe { drop(Box::from_raw(self.0)); } }
       }
       #[repr(C)] #[derive(Clone,Copy)] pub struct __Pb128 {pub hi:u64,pub lo:u64}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbRows {pub data:*const ::core::ffi::c_void,pub len:usize}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbBuffer {pub data:*const u8,pub len:usize}
       unsafe fn __pb_report_message(message:String,error:*mut *mut std::ffi::c_char){
           if !error.is_null(){std::ptr::write(error,std::ffi::CString::new(message.replace('\0',"?")).unwrap().into_raw());}
       }
       #[doc="# Safety\nThe pointer must be null or an unfreed error string returned by this library."]
       #[no_mangle]pub unsafe extern "C" fn #free_error(error:*mut std::ffi::c_char){if !error.is_null(){drop(std::ffi::CString::from_raw(error));}}
       // Input pointers are borrowed only during the call; output buffers are owned.
       unsafe fn __pb_read<'a>(b:__PbBuffer)->Result<&'a [u8],String>{
        if b.len==0{return Ok(&[]);}
        if b.data.is_null()||b.len>isize::MAX as usize{return Err("invalid buffer".into());}
        Ok(std::slice::from_raw_parts(b.data,b.len))
       }
       #[doc = "# Safety
    The buffer must have been returned by this library and not already freed."]
    #[no_mangle] pub unsafe extern "C" fn #free(b:__PbBuffer){if !b.data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8,b.len)));}}
      };
    let mut header=format!("#ifndef {prefix}_H\n#define {prefix}_H\n#include <stdint.h>\n#include <stddef.h>\n#include <stdbool.h>\n#ifdef __cplusplus\nextern \"C\" {{\n#endif\ntypedef struct {{ uint64_t hi; uint64_t lo; }} {prefix}_int128;\n typedef struct {{ const uint8_t *data; size_t len; }} {prefix}_buffer;\nvoid {free}({prefix}_buffer b);\nvoid {free_error}(char *error);\n");
    if let Some((target, width, cfg)) = &flat.metadata.target {
        tokens.extend(quote!(
            const _: () = {
                assert!(cfg!(#cfg) && usize::BITS == #width,
                "C binding target differs from the Rust compilation target");
            };
        ));
        let target_literal = serde_json::to_string(target).map_err(|e| e.to_string())?;
        header.push_str(&format!("/* Binding target identity; use the matching Rust library. */\n#define {prefix}_TARGET {target_literal}\n#ifdef __cplusplus\nstatic_assert(sizeof(void*) * 8 == {width}, \"prebindgen: header pointer width differs; regenerate for this target\");\n#else\n_Static_assert(sizeof(void*) * 8 == {width}, \"prebindgen: header pointer width differs; regenerate for this target\");\n#endif\n"));
    }
    for (name, mapping) in &enums {
        prebindgen_declarations::ident(name)?;
        header.push_str(&format!("typedef int32_t {prefix}_{name};\n"));
        for (variant, value) in &mapping.variants {
            let variant = variant.to_string();
            let value = if *value == i32::MIN {
                "INT32_MIN".into()
            } else {
                value.to_string()
            };
            header.push_str(&format!(
                "#define {prefix}_{name}_{} (({prefix}_{name})({value}))\n",
                variant.trim_start_matches("r#")
            ));
        }
    }
    header.push_str("/* Field paths below start at a source parameter or return and list their ABI values.\n * Optional groups below name their discriminator. Rules apply only in active\n * selector arms and present enclosing groups. Absent payload wires are ignored. */\n");
    for (_, name, ty) in &pointer_entries {
        let drop = format_ident!("{prefix}_drop_{name}");
        tokens.extend(quote!{
            #[doc = "# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call."]
            #[no_mangle] pub unsafe extern "C" fn #drop(ptr:*mut #ty){
                if let Err(error) = ::prebindgen_runtime::boundary(||{if !ptr.is_null(){drop(Box::from_raw(ptr));}Ok(())}) { panic!("C destructor failed: {}", error); }
            }
        });
        header.push_str(&format!(
            "typedef struct {prefix}_{name} {prefix}_{name};\nvoid {drop}({prefix}_{name} *ptr);\n"
        ));
    }
    header.push_str(&format!(
        "typedef struct {{ const uint8_t *data; size_t len; }} {prefix}_buffer_view;\n"
    ));
    for info in flat.types.values() {
        if let Some(owner) = &info.metadata.owned {
            prebindgen_declarations::ident(&owner.name)?;
            let layout = *flat
                .metadata
                .opaque_layouts
                .get(&owner.name)
                .ok_or_else(|| {
                    format!(
                        "opaque owner `{}` has no prepared target layout",
                        owner.name
                    )
                })?;
            layout.validate()?;
            if !symbols.insert(format!("{prefix}_{}", owner.name))
                || !symbols.insert(format!("{prefix}_drop_{}", owner.name))
            {
                return Err("duplicate C inline owner name or destructor".into());
            }
            owned::emit(owner, layout, prefix, &mut tokens, &mut header);
            owned::emit_accessors(
                owner,
                prefix,
                &flat.types,
                &mut symbols,
                &mut tokens,
                &mut header,
            )?;
        }
    }
    let mut callbacks = std::collections::BTreeMap::new();
    for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
        for wire in f.input_wire_definitions()? {
            if let CWire::Callback { callback } = &wire.kind {
                if let Some(previous) = callbacks.insert(callback.name.clone(), callback.clone()) {
                    if previous.signature != callback.signature {
                        return Err("C callback name collision".into());
                    }
                }
            }
        }
    }
    callback::emit(
        &callbacks,
        prefix,
        pointers,
        &enums,
        &mut symbols,
        &mut tokens,
        &mut header,
    )?;
    for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
        for param in &f.params {
            if let Some(slot) = &param.ty.slot {
                let callbacks =
                    prebindgen_flat::wire_definitions(std::iter::once(slot.input_node()))
                        .filter_map(|w| {
                            if let CWire::Callback { callback } = &w.kind {
                                Some((w, callback))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                for (wire, callback) in &callbacks {
                    let alias = if callbacks.len() == 1 {
                        format!("{prefix}_{}_{}", f.name, param.name)
                    } else {
                        format!("{prefix}_{}_{}_{}", f.name, param.name, wire.name)
                    };
                    if !symbols.insert(alias.clone()) {
                        return Err("C callback alias symbol collision".into());
                    }
                    header.push_str(&format!("typedef {} {alias};\n", callback.name));
                    let callback_context = callback.context();
                    let returned = callback.return_fields();
                    if !returned.is_empty() {
                        let ty = if returned.len() == 1 {
                            c_ty(returned[0], prefix, pointers, &callback_context)
                        } else {
                            format!("{}_result", callback.name)
                        };
                        let result_alias = format!("{alias}_return");
                        if !symbols.insert(result_alias.clone()) {
                            return Err("C callback return alias collision".into());
                        }
                        header.push_str(&format!("typedef {ty} {result_alias};\n"));
                        if returned.len() == 1 && matches!(returned[0].kind, CWire::Rows) {
                            let row = format!(
                                "{}_row",
                                ty.strip_suffix("_rows")
                                    .ok_or("invalid C callback row alias")?
                            );
                            header.push_str(&format!("typedef {row} {result_alias}_row;\n"));
                        }
                    }
                    for (index, arg) in callback.argument_fields().into_iter().enumerate() {
                        if matches!(arg.kind, CWire::Rows) {
                            let ty = c_ty(arg, prefix, pointers, &callback_context);
                            let arg_alias = format!("{alias}_arg{index}");
                            if !symbols.insert(arg_alias.clone()) {
                                return Err("C callback argument alias collision".into());
                            }
                            header.push_str(&format!("typedef {ty} {arg_alias};\nstatic inline void {arg_alias}_free({arg_alias} value) {{ {ty}_free(value); }}\n"));
                        }
                    }
                }
            }
        }
    }
    for error in &errors {
        validate_layout(&error.plan()?.outcomes.success)?;
        let mut schemas = Vec::new();
        sequence::schemas(&error.plan()?.outcomes.success, &mut schemas);
        for wire in crate::group::result_packet_fields(&error.plan()?.outcomes.success).chain(
            schemas
                .iter()
                .flat_map(|(_, element, _)| crate::group::row_fields(element)),
        ) {
            validate_wire_declaration(wire, pointers, &enums)?;
        }
    }
    errors::emit(
        &errors,
        prefix,
        pointers,
        &mut symbols,
        &mut tokens,
        &mut header,
    )?;
    header.push_str("/* Non-Result calls return their first value directly; additional values use output pointers.\n * Binding failures require explicit .panic() and abort. Destructors return void. */\n");
    for (f, body, context) in calls {
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.input_node(), &mut schemas);
        }
        let mut output_schemas = Vec::new();
        sequence::schemas(
            &f.output_conversion()?.output_outcomes().success,
            &mut output_schemas,
        );
        let output_carriers = output_schemas
            .iter()
            .map(|(wire, _, _)| wire.name.to_string())
            .collect::<std::collections::BTreeSet<_>>();
        schemas.extend(output_schemas);
        sequence::emit_schemas(
            schemas,
            &output_carriers,
            &context,
            prefix,
            pointers,
            &mut symbols,
            &mut tokens,
            &mut header,
        )?;
        let out = &context.output_pointer;
        let accepted = &context.acknowledgement;
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let inputs = crate::group::function_parameters(f)?.collect::<Vec<_>>();
        let outputs =
            crate::group::result_packet_fields(&f.output_conversion()?.output_outcomes().success)
                .collect::<Vec<_>>();
        if outputs.iter().any(|w| {
            matches!(&w.kind, CWire::Pointer { pointer, .. } if pointer.access != Access::Move)
                || matches!(
                    w.kind,
                    CWire::View
                        | CWire::Owned {
                            access: Access::Shared | Access::Exclusive,
                            ..
                        }
                )
        }) {
            header.push_str("/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */\n");
        }
        let mut next_option_label = 0;
        for param in &f.params {
            if let Some(slot) = &param.ty.slot {
                header.push_str(&describe_groups(
                    slot.input_node(),
                    &inputs,
                    false,
                    &mut next_option_label,
                    &param.name.to_string(),
                ));
            }
        }
        if prebindgen_declarations::result_parts(&f.ret.ty).is_none() {
            if let Some(slot) = &f.ret.slot {
                let mut comments = describe_groups(
                    &slot.output_outcomes().success,
                    &outputs,
                    true,
                    &mut next_option_label,
                    "return",
                );
                for i in (0..outputs.len()).rev() {
                    comments = comments.replace(
                        &format!("output->v{i}"),
                        &if i == 0 {
                            "return value".to_owned()
                        } else {
                            format!("*out{i}")
                        },
                    );
                }
                header.push_str(&comments);
            }
        }
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        for w in crate::group::function_parameters(f)? {
            let n = &w.name;
            let ty = w.kind.rust_type();
            params.push(quote!(#n:#ty));
            cparams.push(format!("{} {n}", c_ty(w, prefix, pointers, &context)));
        }
        let body = if sequence::callback_delivery(
            &f.output_conversion()?.output_outcomes().success,
        )? {
            let (_, element, _) =
                sequence::return_sequence(&f.output_conversion()?.output_outcomes().success)
                    .ok_or("missing callback element schema")?;
            let types = crate::group::callback_parameters(element)
                .map(|wire| wire.rust_type())
                .collect::<Vec<_>>();
            params.push(quote!(__pb_callback:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void,#(#types),*)>));
            params.push(quote!(__pb_callback_context:*mut ::core::ffi::c_void));
            let mut fields = vec!["void *context".to_string()];
            fields.extend(
                crate::group::callback_parameters(element)
                    .enumerate()
                    .map(|(i, wire)| format!("{} v{i}", c_ty(wire, prefix, pointers, &context))),
            );
            cparams.push(format!("void (*callback)({})", fields.join(", ")));
            cparams.push("void *context".into());
            header.push_str("/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */\n");
            quote!(if __pb_callback.is_none(){return Err("null sequence callback".into());}#body)
        } else {
            body
        };
        if let Some((_, error_ty)) = prebindgen_declarations::result_parts(&f.ret.ty) {
            let fields = outputs
                .iter()
                .enumerate()
                .map(|(i, wire)| (format_ident!("v{i}"), wire.rust_type()))
                .collect::<Vec<_>>();
            let fields = if fields.is_empty() {
                vec![(syn::parse_quote!(unused), syn::parse_quote!(u8))]
            } else {
                fields
            };
            let definition = prebindgen_reprc::Record::new(result.clone(), fields)?.definition();
            tokens.extend(quote!(#definition));
            let comments = describe_groups(
                &(f.ret.slot.as_ref().unwrap()).output_outcomes().success,
                &outputs,
                true,
                &mut next_option_label,
                "return",
            );
            let consumes=f.params.iter().filter_map(|p|p.ty.slot.as_ref()).any(|slot| !matches!(slot.code_fragment_slot(prebindgen_conversions::InputFragment::Acknowledge {context:&context},&syn::parse_quote!(())),Ok(code) if code.to_string()==quote!(()).to_string()));
            errors::result_function(
                &f.name,
                error_ty,
                &outputs,
                &f.output_conversion()?.output_outcomes().success,
                params,
                cparams,
                &body,
                &context,
                prefix,
                pointers,
                &errors,
                consumes,
                &comments,
                &mut tokens,
                &mut header,
            )?;
            continue;
        }
        let mut fields = Vec::new();
        let mut writes = TokenStream::new();
        let mut validate_outputs = TokenStream::new();
        for (i, wire) in outputs.iter().enumerate() {
            let field = format_ident!("v{i}");
            let ty = wire.rust_type();
            fields.push((field.clone(), ty.clone()));
            if i > 0 {
                let pointer = format_ident!("out{i}");
                params.push(quote!(#pointer:*mut #ty));
                cparams.push(format!(
                    "{} *{pointer}",
                    c_ty(wire, prefix, pointers, &context)
                ));
                validate_outputs.extend(
                    quote!(if #pointer.is_null(){return Err("null success output".into());}),
                );
                writes.extend(quote!(std::ptr::write(#pointer, value.#field);));
            }
        }
        if fields.is_empty() {
            fields.push((syn::parse_quote!(unused), syn::parse_quote!(u8)));
        }
        let (return_type, c_return, returned) = match outputs.first() {
            Some(wire) => {
                let ty = wire.rust_type();
                (
                    quote!(->#ty),
                    c_ty(wire, prefix, pointers, &context),
                    quote!(value.v0),
                )
            }
            None => (quote!(), "void".into(), quote!()),
        };
        let value = if outputs.is_empty() {
            quote!()
        } else {
            quote!(let value=storage.assume_init();)
        };
        header.push_str(&format!(
            "{c_return} {name}({});\n",
            if cparams.is_empty() {
                "void".into()
            } else {
                cparams.join(", ")
            }
        ));
        let definition = prebindgen_reprc::Record::new(result.clone(), fields)?.definition();
        tokens.extend(quote! {
         #definition
         #[doc = "# Safety\nAdditional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call."]
         #[allow(clippy::needless_borrow,clippy::let_unit_value,clippy::unnecessary_cast,clippy::useless_conversion)]
         #[no_mangle] pub unsafe extern "C" fn #name(#(#params),*) #return_type {
            #[allow(unused_mut)] let #accepted=::core::cell::Cell::new(0usize);
            let mut storage=std::mem::MaybeUninit::<#result>::uninit();
            let #out=storage.as_mut_ptr();
            if let Err(error)=::prebindgen_runtime::boundary(||{#validate_outputs #body}) {panic!("C binding failed: {}",error);}
            #value
            #writes
            #returned
         }
        });
    }
    let guards = &flat.guards;
    tokens.extend(quote!(#(#guards)*));
    header.push_str("#ifdef __cplusplus\n}\n#endif\n#endif\n");
    Ok(Generation {
        rust: prebindgen_flat::rust_source(tokens)?,
        header,
    })
}

// A Rust reference may refer to a wrapper-reconstructed input, rather than C's
// persistent allocation. Such a pointer must not escape the generated call.
fn pointer_owns_value(pointer: &syn::Type, returned: &syn::Type) -> bool {
    let mut ty = prebindgen_flat::result_parts(returned).map_or(returned, |(ok, _)| ok);
    while let Some(inner) = prebindgen_flat::option_inner(ty) {
        ty = inner;
    }
    // Flat currently permits only nongeneric named owners. Do not silently
    // extend this proof to future lifetime-bearing owner types.
    matches!(ty, syn::Type::Path(p) if p.qself.is_none() && p.path.segments.iter().all(|s| matches!(s.arguments,syn::PathArguments::None)))
        && prebindgen_flat::same_type(pointer, ty)
}
fn validate_return_loans<I, O, M>(
    function: &prebindgen_flat::Function<I, O, M>,
    output: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Result<()>
where
    I: InputConversion<Wire = CWire, Chain = CChain, Branch = CBranch>,
{
    if !crate::group::result_packet_fields(output).any(|w| {
        matches!(&w.kind, CWire::Pointer {pointer,..} if pointer.access != Access::Move)
            || matches!(
                &w.kind,
                CWire::View
                    | CWire::Owned {
                        access: Access::Shared | Access::Exclusive,
                        ..
                    }
            )
    }) {
        return Ok(());
    }
    // Owner-backed combinations create their loans only after this allocation
    // has reached its final address. Temporary source inputs cannot own it.
    // Getters take only the owner, and other non-static getter loans are rejected
    // by validate_getter_output; both restrictions are required for this bypass.
    if crate::group::result_packet_fields(output).any(|w| matches!(&w.kind,
        CWire::Pointer {pointer,..} if pointer.access == Access::Move && pointer_owns_value(&pointer.ty, &function.ret.ty))) {
        return Ok(());
    }
    fn reference(ty: &syn::Type) -> Option<&syn::TypeReference> {
        if let syn::Type::Reference(r) = ty {
            return Some(r);
        }
        prebindgen_flat::option_inner(ty).and_then(reference)
    }
    let returned = prebindgen_flat::result_parts(&function.ret.ty)
        .map_or(&function.ret.ty, |(success, _)| success);
    if reference(returned).is_some_and(|r| r.lifetime.as_ref().is_some_and(|l| l.ident == "static"))
    {
        return Ok(());
    }
    fn stable_input(
        ty: &syn::Type,
        node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    ) -> bool {
        if let (
            Some(inner),
            prebindgen_flat::WireNode::Chain {
                meta: CChain::Optional(_),
                child,
                ..
            },
        ) = (prebindgen_flat::option_inner(ty), node)
        {
            return stable_input(inner, child);
        }
        match (ty, node) {
            (syn::Type::Reference(r), prebindgen_flat::WireNode::Leaf(wires))
                if wires.len() == 1 =>
            {
                matches!(&wires[0].kind,CWire::Pointer {pointer,..} if prebindgen_flat::same_type(&pointer.ty,&r.elem))
                    || (matches!(prebindgen_flat::type_key(&r.elem).as_str(), "str" | "[u8]")
                        && matches!(wires[0].kind, CWire::Buffer))
                    || matches!(&wires[0].kind,CWire::Owned{owner,by_value:false,..} if prebindgen_flat::same_type(&owner.ty,&r.elem))
            }
            _ => false,
        }
    }
    for parameter in &function.params {
        if reference(&parameter.ty.ty).is_some()
            && !stable_input(
                &parameter.ty.ty,
                parameter
                    .ty
                    .slot
                    .as_ref()
                    .ok_or("missing C input conversion")?
                    .input_node(),
            )
        {
            return Err(format!("C function `{}` cannot return a borrowed pointer with reconstructed reference input `{}`; use a direct opaque pointer input, an explicit owning output projection, or a 'static return", function.name,parameter.name));
        }
    }
    Ok(())
}

fn describe_groups(
    layout: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    wires: &[&Wire],
    output: bool,
    next_label: &mut usize,
    root: &str,
) -> String {
    fn visit(
        node: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        names: &std::collections::BTreeMap<String, String>,
        parents: &mut Vec<String>,
        out: &mut String,
        next_label: &mut usize,
        field_path: &mut Vec<String>,
    ) {
        if let prebindgen_flat::WireNode::Branch {
            meta: CBranch::Alternatives,
            children: alternatives,
            ..
        } = node
        {
            for arm in alternatives {
                visit(arm, names, parents, out, next_label, field_path);
            }
        }
        if let prebindgen_flat::WireNode::Chain {
            controls,
            child: element,
            meta: CChain::Sequence(metadata),
            ..
        } = node
        {
            let [carrier] = controls.as_slice() else {
                return;
            };
            let path = field_path.join(".");
            if *metadata == CSequence::Buffered {
                out.push_str(&format!("/* Collection {path}: {} carries rows of {} ABI fields; element-owned fields are released separately from row storage. */\n",carrier.name,crate::group::row_fields(element).len()));
            } else {
                out.push_str(&format!("/* Callback sequence {path}: {} reports the delivered element count; the visitor receives {} fields per element. */\n",carrier.name,crate::group::row_fields(element).len()));
            }
            let row_names = crate::group::row_fields(element)
                .enumerate()
                .map(|(i, w)| (w.name.to_string(), format!("row.v{i}")))
                .collect();
            let original = field_path.last().cloned();
            if let Some(last) = field_path.last_mut() {
                last.push_str("[]");
            }
            visit(element, &row_names, parents, out, next_label, field_path);
            if let (Some(last), Some(original)) = (field_path.last_mut(), original) {
                *last = original;
            }
        }
        if let prebindgen_flat::WireNode::Chain {
            child: element,
            meta: CChain::Optional(g),
            ..
        } = node
        {
            let label = format!("option{}", *next_label);
            *next_label += 1;
            let discriminator = &names[&g.discriminator().to_string()];
            let rule = match g.niche() {
                    None => format!("{discriminator}: 0 = None, 1 = Some; other active flags are errors"),
                    Some(CNiche::Null) => format!("{discriminator} == NULL means None; non-null means Some"),
                    Some(CNiche::Integer(n)) => format!("{discriminator} == {n} means None; other values encode Some and must be valid payloads"),
                };
            let parent = if parents.is_empty() {
                String::new()
            } else {
                format!("; only when {} is present", parents.join(" and "))
            };
            out.push_str(&format!("/* Option {}: {rule}{parent}. */\n", label));
            parents.push(label);
            visit(element, names, parents, out, next_label, field_path);
            parents.pop();
        }
        if let prebindgen_flat::WireNode::Branch {
            meta: CBranch::Group(metadata),
            children,
            ..
        } = node
        {
            if let CGroup::Field(name) = metadata {
                field_path.push(name.to_string().trim_start_matches("r#").into());
                let values = crate::group::record_fields(node)
                    .map(|w| names[&w.name.to_string()].clone())
                    .collect::<Vec<_>>();
                let values = if values.is_empty() {
                    "no boundary values".into()
                } else {
                    values.join(", ")
                };
                out.push_str(&format!(
                    "/* Field {}: {values}. */\n",
                    field_path.join(".")
                ));
                for child in children {
                    visit(child, names, parents, out, next_label, field_path);
                }
                field_path.pop();
            } else {
                for child in children {
                    visit(child, names, parents, out, next_label, field_path);
                }
            }
        }
    }
    let names = wires
        .iter()
        .enumerate()
        .map(|(i, w)| {
            (
                w.name.to_string(),
                if output {
                    format!("output->v{i}")
                } else {
                    w.name.to_string()
                },
            )
        })
        .collect();
    let mut out = String::new();
    visit(
        layout,
        &names,
        &mut Vec::new(),
        &mut out,
        next_label,
        &mut vec![root.trim_start_matches("r#").into()],
    );
    out
}

#[cfg(test)]
mod generation_tests {
    use super::*;

    fn error_model(source: &str) -> Bindings {
        let source = syn::parse_file(source).unwrap();
        Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn direct_shared_returns_are_non_owning_nullable_const_pointers() {
        use prebindgen_declarations::fun;
        let flat = Cbindgen::new(error_model("pub type Owner=String;pub type Child=String;pub fn child(owner:&Owner)->&Child{}pub fn maybe(owner:&Owner)->Option<&Child>{}pub fn nested(owner:&Owner)->Option<Option<&Child>>{}pub fn fallible(owner:&Owner)->Result<&Child,String>{}pub fn optional(owner:&Owner)->Result<Option<&Child>,String>{}pub fn message(error:&String)->String{}"))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .opaque_ptr("child_t", syn::parse_quote!(Child))
            .function(fun!(child)).panic()
            .function(fun!(maybe)).panic()
            .function(fun!(nested)).panic()
            .function(fun!(fallible))
            .function(fun!(optional))
            .opaque_error(syn::parse_quote!(String), fun!(message))
            .prepare().unwrap();
        for name in ["child", "maybe", "nested", "fallible", "optional"] {
            let output = flat.function(name).unwrap().ret.slot.as_ref().unwrap();
            assert!(!output.statements.to_string().contains("Clone"));
            assert!(!output.statements.to_string().contains("Box"));
            let pointers = crate::group::result_packet_fields(&output.outcomes.success)
                .filter_map(|w| match &w.kind {
                    CWire::Pointer { pointer, .. } => Some(pointer),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(pointers.len(), 1);
            assert_eq!(pointers[0].access, Access::Shared);
        }
        assert_eq!(
            crate::group::result_packet_fields(
                &flat
                    .function("nested")
                    .unwrap()
                    .ret
                    .slot
                    .as_ref()
                    .unwrap()
                    .outcomes
                    .success
            )
            .count(),
            2
        );
        let generated = generate(&flat).unwrap();
        assert!(generated.header.contains("const pb_child_t * pb_child("));
        assert!(generated.header.contains("const pb_child_t * pb_maybe("));
        assert!(generated.header.contains("const pb_child_t * pb_fallible("));
        assert!(generated.header.contains("bool pb_optional("));
        assert!(generated.header.contains("const pb_child_t * *out0"));
        assert!(generated.header.contains("never be freed or consumed"));
        syn::parse_file(&generated.rust).unwrap();
    }

    #[test]
    fn mutable_returns_preserve_exclusive_access_without_ownership_or_copying() {
        use prebindgen_declarations::fun;
        let flat = Cbindgen::new(error_model("pub type Owner=String;pub type Child=String;pub fn child(owner:&mut Owner)->&mut Child{}pub fn maybe(owner:&mut Owner)->Option<&mut Child>{}pub fn fallible(owner:&mut Owner)->Result<&mut Child,String>{}pub fn optional(owner:&mut Owner)->Result<Option<&mut Child>,String>{}pub fn message(error:&String)->String{}"))
            .opaque_ptr("owner",syn::parse_quote!(Owner)).opaque_ptr("child_t",syn::parse_quote!(Child))
            .opaque_error(syn::parse_quote!(String),fun!(message))
            .function(fun!(child)).panic().function(fun!(maybe)).panic()
            .function(fun!(fallible)).function(fun!(optional)).prepare().unwrap();
        for name in ["child", "maybe", "fallible", "optional"] {
            let function = flat.function(name).unwrap();
            let output = function.ret.slot.as_ref().unwrap();
            assert!(!output.statements.to_string().contains("Clone"));
            assert!(!output.statements.to_string().contains("Box"));
            assert!(crate::group::result_packet_fields(&output.outcomes.success)
                .all(|w| matches!(&w.kind,CWire::Pointer {pointer,..} if pointer.access==Access::Exclusive)));
        }
        let generated = generate(&flat).unwrap();
        assert!(generated
            .header
            .contains("pb_child_t * pb_child(pb_owner *"));
        assert!(generated
            .header
            .contains("pb_child_t * pb_maybe(pb_owner *"));
        assert!(generated
            .header
            .contains("pb_child_t * pb_fallible(pb_owner *"));
        assert!(generated.header.contains("pb_child_t * *out0"));
        assert!(generated
            .header
            .contains("mutable borrows require exclusive access"));
        syn::parse_file(&generated.rust).unwrap();
    }

    #[test]
    fn mutable_value_and_collection_outputs_have_no_copying_fallback() {
        use prebindgen_declarations::fun;
        for returned in ["&mut i64", "&mut [u8]"] {
            let source =
                format!("pub type Owner=String;pub fn value(owner:&mut Owner)->{returned}{{}}");
            let error = Cbindgen::new(error_model(&source))
                .opaque_ptr("owner", syn::parse_quote!(Owner))
                .function(fun!(value))
                .panic()
                .build()
                .err()
                .unwrap();
            assert!(error.contains("mutable"), "{returned}: {error}");
        }
    }

    #[test]
    fn mutable_getter_expansions_are_rejected_instead_of_copied() {
        use prebindgen_declarations::{expand_return, fun};
        let error = Cbindgen::new(error_model("pub type Owner=String;pub type Child=String;pub fn make()->Owner{}pub fn child(owner:&Owner)->&mut Child{}"))
            .opaque_ptr("child_t",syn::parse_quote!(Child))
            .function(fun!(make).expand_return(expand_return!(Owner).view(prebindgen_declarations::Output::getter("child",prebindgen_declarations::Output::Projected(prebindgen_declarations::Projection::borrowed())))))
            .build().err().unwrap();
        assert!(
            error.contains("mutable-reference getter results are unsupported"),
            "{error}"
        );
    }

    #[test]
    fn owner_backed_getters_borrow_final_storage_and_keep_wire_order() {
        use prebindgen_declarations::{expand_return, fun, Output};
        for mutable in [false, true] {
            let qualifier = if mutable { "mut " } else { "" };
            let source=format!("pub type Owner=String;pub type Child=String;pub fn make()->Owner{{}}pub fn child(o:&{qualifier}Owner)->&{qualifier}Child{{}}");
            for reversed in [false, true] {
                let getter = Output::getter("child", Output::Leaf);
                let expansion = if reversed {
                    expand_return!(Owner).view(getter).field_self_into()
                } else {
                    expand_return!(Owner).field_self_into().view(getter)
                };
                let flat = Cbindgen::new(error_model(&source))
                    .opaque_ptr("owner_t", syn::parse_quote!(Owner))
                    .opaque_ptr("child_t", syn::parse_quote!(Child))
                    .function(fun!(make).expand_return(expansion))
                    .prepare()
                    .unwrap();
                let output = flat.function("make").unwrap().ret.slot.as_ref().unwrap();
                let code = output.statements.to_string();
                assert!(!code.contains("Clone"));
                assert!(
                    code.find("Box :: into_raw").unwrap() < code.find("crate :: child").unwrap()
                );
                let access = crate::group::result_packet_fields(&output.outcomes.success)
                    .map(|w| match &w.kind {
                        CWire::Pointer { pointer, .. } => pointer.access,
                        _ => panic!(),
                    })
                    .collect::<Vec<_>>();
                let loan = if mutable {
                    Access::Exclusive
                } else {
                    Access::Shared
                };
                assert_eq!(
                    access,
                    if reversed {
                        vec![loan, Access::Move]
                    } else {
                        vec![Access::Move, loan]
                    }
                );
                generate(&flat).unwrap();
            }
        }
    }

    #[test]
    fn owner_view_near_misses_are_diagnostics_not_implicit_clones() {
        use prebindgen_declarations::{expand_return, fun, Output};
        let source="pub type Owner=String;pub type Child=String;pub fn make()->Owner{}pub fn child(o:&Owner)->&Child{}pub fn mutate(o:&mut Owner)->&mut Child{}";
        let model = || {
            Cbindgen::new(error_model(source))
                .opaque_ptr("owner_t", syn::parse_quote!(Owner))
                .opaque_ptr("child_t", syn::parse_quote!(Child))
        };
        let cases = [
            (
                expand_return!(Owner).field(fun!(child)),
                "requires its opaque owner",
            ),
            (
                expand_return!(Owner)
                    .field_self_into()
                    .field(fun!(child))
                    .view(Output::Discard),
                "supports only",
            ),
            (
                expand_return!(Owner)
                    .field_self_into()
                    .view(Output::getter("mutate", Output::Leaf))
                    .field(fun!(child)),
                "only getter view",
            ),
        ];
        for (expansion, message) in cases {
            let error = model()
                .function(fun!(make).expand_return(expansion))
                .build()
                .err()
                .unwrap();
            assert!(error.contains(message), "{error}");
        }
    }

    #[test]
    fn getter_expansions_and_explicit_projections_keep_owned_clones() {
        use prebindgen_declarations::{expand_return, fun};
        let flat = Cbindgen::new(error_model("pub type Owner=String;pub type Child=String;pub fn make()->Owner{}pub fn child(owner:&Owner)->&Child{}"))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .opaque_ptr("child", syn::parse_quote!(Child))
            .function(fun!(make).expand_return(expand_return!(Owner).view(prebindgen_declarations::Output::getter("child",prebindgen_declarations::Output::Projected(prebindgen_declarations::Projection::borrowed())))))
            .function(fun!(child).expand_return(expand_return!(Child).field_self_ref()))
            .panic()
            .prepare().unwrap();
        for name in ["make", "child"] {
            let output = flat.function(name).unwrap().ret.slot.as_ref().unwrap();
            assert!(
                output.statements.to_string().contains("Clone"),
                "{name}: {}",
                output.statements
            );
            assert!(crate::group::result_packet_fields(&output.outcomes.success)
                .all(|w| matches!(&w.kind,CWire::Pointer { pointer,..} if pointer.access == Access::Move)));
        }
    }

    #[test]
    fn loans_reject_reconstructed_inputs_but_allow_static_returns_and_owned_copies() {
        use prebindgen_declarations::{expand_param, expand_return, fun};
        let source = "pub type Child=String;pub fn peek(value:&String)->&Child{}pub fn fixed(value:&String)->&'static Child{}pub fn first(values:&[Child])->&Child{}pub fn copy_child(child:&Child)->Child{}";
        let model =
            || Cbindgen::new(error_model(source)).opaque_ptr("child_t", syn::parse_quote!(Child));
        let bad = model().function(fun!(peek)).panic().build().err().unwrap();
        assert!(
            bad.contains("reconstructed reference input `value`"),
            "{bad}"
        );
        let bad = model()
            .expand(expand_param!(Child).variant(fun!(copy_child)))
            .function(fun!(first))
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(
            bad.contains("reconstructed reference input `values`"),
            "{bad}"
        );
        model().function(fun!(fixed)).panic().build().unwrap();
        model()
            .function(fun!(peek).expand_return(expand_return!(Child).field_self_ref()))
            .panic()
            .build()
            .unwrap();
    }

    #[test]
    fn expanded_reference_to_option_keeps_ordinary_value_conversion() {
        use prebindgen_declarations::{expand_return, fun};
        let source = "pub type Child=String;pub type Owner=String;pub fn maybe(owner:&Owner)->&Option<Child>{}pub fn length(child:&Child)->i64{}pub fn fallible(owner:&Owner)->Result<&Option<Child>,String>{}pub fn message(error:&String)->String{}";
        let generated = Cbindgen::new(error_model(source))
            .opaque_ptr("owner", syn::parse_quote!(Owner))
            .function(fun!(maybe).expand_return(expand_return!(Child).field(fun!(length))))
            .panic()
            .function(fun!(fallible).expand_return(expand_return!(Child).field(fun!(length))))
            .opaque_error(syn::parse_quote!(String), fun!(message))
            .build()
            .unwrap();
        assert!(generated
            .header
            .contains("int32_t pb_maybe(const pb_owner *"));
        syn::parse_file(&generated.rust).unwrap();
    }

    #[test]
    fn optional_callback_references_keep_their_owning_copy_contract() {
        use prebindgen_declarations::{compile_callback_argument, Output};
        let output = compile_callback_argument(
            &error_model("pub type Child=String;"),
            &CCodec,
            &syn::parse_quote!(Option<&Child>),
            &Output::Optional(Box::new(Output::Leaf)),
            0,
            false,
            &(),
        )
        .unwrap();
        assert!(output.statements.to_string().contains("Clone"));
        assert!(
            crate::group::result_packet_fields(&output.outcomes.success).all(
                |w| matches!(&w.kind, CWire::Pointer {pointer,..} if pointer.access == Access::Move)
            )
        );
    }

    #[test]
    fn reclaiming_undelivered_outputs_does_not_destroy_loans() {
        let context = CContext {
            row_types: Default::default(),
            acknowledgement: syn::parse_quote!(accepted),
            output_pointer: syn::parse_quote!(out),
            output_type: syn::parse_quote!(Returned),
            error_pointer: syn::parse_quote!(error),
            error_reported: syn::parse_quote!(reported),
            output_committed: quote!(),
            buffer_type: syn::parse_quote!(Buffer),
            wide_integer_type: syn::parse_quote!(Wide),
            read_buffer: syn::parse_quote!(read),
        };
        for reference in [syn::parse_quote!(&Child), syn::parse_quote!(&mut Child)] {
            let plan = CCodec
                .output_return_reference(
                    &error_model("pub type Child=String;"),
                    &reference,
                    &(),
                    syn::parse_quote!(child),
                    quote!(source),
                )
                .unwrap()
                .unwrap();
            let wire = crate::group::result_packet_fields(&plan.outcomes.success)
                .next()
                .unwrap();
            let release = sequence::release(
                &plan.outcomes.success,
                &[(wire, quote!(child))],
                &context,
                false,
            );
            assert!(release.is_empty());
            let mut owned = wire.clone();
            if let CWire::Pointer { pointer, .. } = &mut owned.kind {
                pointer.access = Access::Move;
            }
            assert!(!sequence::release(
                &plan.outcomes.success,
                &[(&owned, quote!(child))],
                &context,
                false
            )
            .is_empty());
        }
    }
    #[test]
    fn opaque_declarations_override_builtin_and_collection_types() {
        use prebindgen_declarations::fun;
        for ty in [syn::parse_quote!(String), syn::parse_quote!(Vec<Item>)] {
            let generated = Cbindgen::new(error_model("pub type Item=String;"))
                .opaque_ptr("native", ty)
                .build()
                .unwrap();
            assert!(generated.header.contains("pb_native"));
        }
        let generated = Cbindgen::new(error_model("pub fn f()->String{}"))
            .opaque_ptr("native", syn::parse_quote!(String))
            .function(fun!(f))
            .build()
            .unwrap();
        assert!(generated.header.contains("pb_native * pb_f("));
        for ty in [
            syn::parse_quote!(Option<Item>),
            syn::parse_quote!(&Item),
            syn::parse_quote!(Result<Item,Item>),
        ] {
            assert!(Cbindgen::new(error_model("pub type Item=String;"))
                .opaque_ptr("bad", ty)
                .prepare()
                .is_err());
        }
    }
    #[test]
    fn error_conversion_reads_current_flat_definition() {
        let codec = enums::EnumCodec::default();
        let mut flat =
            error_model("pub struct Failure{pub code:i64}pub fn f()->Result<i64,Failure>{}");
        let ty = flat.resolve_type(&syn::parse_quote!(Failure)).unwrap();
        flat.type_metadata_mut(&ty).unwrap().error = Some(std::rc::Rc::new(
            errors::ErrorBinding::declared("initial".into(), ty.clone()),
        ));
        let declarations =
            prebindgen_declarations::Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
        errors::prepare(&mut flat, &codec, &declarations).unwrap();
        let error = flat.type_metadata_mut(&ty).unwrap().error.as_mut().unwrap();
        std::rc::Rc::make_mut(error).name = "updated".into();
        let code = codec
            .result_error_value(&flat, &ty, quote!(source), None, &())
            .unwrap()
            .to_string();
        assert!(code.contains("__pb_report_updated"));
        assert!(!code.contains("__pb_report_initial"));
    }
    #[test]
    fn prebound_function_policy_is_read_from_flat() {
        use prebindgen_declarations::{bind_input, bind_output, Input, Output};
        let mut flat = error_model("pub type Item=String;pub fn inspect(value:&Item)->i64{}");
        flat.type_metadata_mut(&syn::parse_quote!(Item))
            .unwrap()
            .pointer = Some("item".into());
        flat.function_mut("inspect").unwrap().metadata = Some(CFunction {
            abort_on_failure: true,
            requires_abort: false,
        });
        bind_input(&mut flat, &CCodec, "inspect", "value", &Input::Leaf, &()).unwrap();
        bind_output(&mut flat, &CCodec, "inspect", &Output::Leaf, &()).unwrap();
        let prepared = Cbindgen::new(flat).prepare().unwrap();
        assert!(
            prepared
                .function("inspect")
                .unwrap()
                .metadata
                .as_ref()
                .unwrap()
                .abort_on_failure
        );
        assert!(generate(&prepared).unwrap().header.contains("pb_inspect"));
    }
    #[test]
    fn logical_outcomes_and_sequence_fields_are_independent_of_c_transport() {
        use prebindgen_declarations::{fun, Declarations};

        let mut flat=error_model("pub struct Failure {pub code:i64,pub info:Option<String>} pub fn f(v:Option<Vec<i64>>)->Result<Option<Vec<Option<i64>>>,Failure>{}");
        let ty = flat.resolve_type(&syn::parse_quote!(Failure)).unwrap();
        let codec = enums::EnumCodec::default();
        flat.type_metadata_mut(&ty).unwrap().error = Some(std::rc::Rc::new(
            errors::ErrorBinding::declared("failure".into(), ty.clone()),
        ));
        let declarations = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
        errors::prepare(&mut flat, &codec, &declarations).unwrap();
        let decl = fun!(f).callback_return();
        declarations.fill(&mut flat, &codec, &decl, &()).unwrap();
        let normal = flat.function("f").unwrap();
        assert_eq!(normal.input_wire_definitions().unwrap().count(), 3);
        assert_eq!(
            crate::group::function_parameters(normal).unwrap().count(),
            2
        );
        let output = normal.output_conversion().unwrap();
        let prebindgen_flat::Outcomes {
            success,
            source_error: Some(errors),
            binding_failure: Some(binding_failure),
        } = output.output_outcomes()
        else {
            panic!("all three completion channels must be declared")
        };

        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(success)).count(),
            4
        );

        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(errors)).count(),
            3
        );
        assert_eq!(
            crate::group::result_packet_fields(
                &flat
                    .type_info(&ty)
                    .unwrap()
                    .metadata
                    .error
                    .as_ref()
                    .unwrap()
                    .plan()
                    .unwrap()
                    .outcomes
                    .success
            )
            .count(),
            3
        );
        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(binding_failure)).count(),
            1
        );
        assert_ne!(
            prebindgen_flat::wire_definitions([success, errors, binding_failure]).count(),
            output.values.len()
        );
        assert!(matches!(
            prebindgen_flat::wire_definitions(std::iter::once(success))
                .last()
                .unwrap()
                .kind,
            CWire::Scalar {
                primitive: Primitive::I64,
                ..
            }
        ));
        let shape = |layout: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>| {
            let (_, element, _) = sequence::return_sequence(layout).unwrap();
            prebindgen_flat::wire_definitions_with_paths(std::iter::once(element))
                .map(|e| (e.path, prebindgen_flat::type_key(&e.wire.rust_type())))
                .collect::<Vec<_>>()
        };
        let expected = shape(&output.output_outcomes().success);
        let expected_paths = prebindgen_flat::wire_definitions_with_paths(std::iter::once(
            &output.output_outcomes().success,
        ))
        .map(|entry| entry.path)
        .collect::<Vec<_>>();
        sequence::callback_target(
            &(flat.function("f").unwrap().output_conversion().unwrap())
                .output_outcomes()
                .success,
        )
        .unwrap();
        let variant = declarations
            .callback_variant(&mut flat, &codec, &decl, "f_each", &())
            .unwrap();
        declarations.fill(&mut flat, &codec, &variant, &()).unwrap();
        let callback = flat
            .function("f_each")
            .unwrap()
            .output_conversion()
            .unwrap();
        assert_eq!(shape(&callback.output_outcomes().success), expected);
        assert_eq!(
            prebindgen_flat::wire_definitions_with_paths(std::iter::once(
                &callback.output_outcomes().success
            ))
            .map(|entry| entry.path)
            .collect::<Vec<_>>(),
            expected_paths
        );
        assert!(matches!(
            crate::group::result_packet_fields(&callback.output_outcomes().success)
                .nth(1)
                .unwrap()
                .kind,
            CWire::Scalar {
                primitive: Primitive::Usize,
                ..
            }
        ));
    }
    #[test]
    fn callback_return_rejects_unrelated_output_values() {
        use prebindgen_declarations::{expand_return, fun};
        let source=syn::parse_file("pub fn values()->Option<Vec<i64>>{} pub fn number(v:&Option<Vec<i64>>)->i64{} pub fn view(v:&Option<Vec<i64>>)->Vec<i64>{}").unwrap();
        let flat = Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let result = Cbindgen::new(flat)
            .function(
                fun!(values).callback_return().expand_return(
                    expand_return!(Option<Vec<i64>>)
                        .field(fun!(number))
                        .field(fun!(view)),
                ),
            )
            .panic()
            .build();
        assert!(result
            .err()
            .unwrap()
            .contains("callback_return requires exactly one outer sequence"));
    }

    #[test]
    fn callback_only_elements_require_declared_c_types() {
        let source =
            syn::parse_file("pub struct Undeclared {pub n:i32} pub fn values()->Vec<Undeclared>{}")
                .unwrap();
        let flat = Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let error = Cbindgen::new(flat)
            .function(prebindgen_declarations::fun!(values).callback_return_only())
            .panic()
            .build()
            .err()
            .unwrap();
        assert!(error.contains("declare a C opaque pointer"), "{error}");
    }

    #[test]
    fn callback_returns_preserve_buffers_and_require_explicit_abort_policy() {
        let flat = || {
            let source = syn::parse_file("pub fn values()->Vec<i64>{}").unwrap();
            Bindings::parse(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .unwrap()
        };
        let builder = || {
            Cbindgen::new(flat()).function(prebindgen_declarations::fun!(values).callback_return())
        };
        assert!(builder().build().err().unwrap().contains(".panic()"));
        let output = builder().panic().build().unwrap();
        assert!(output.header.contains("test_values") || output.header.contains("pb_values("));
        assert!(output.header.contains("pb_values_each("));
        assert!(output
            .header
            .contains("void (*callback)(void *context, int64_t v0)"));
        syn::parse_file(&output.rust).unwrap();
    }

    #[test]
    fn structured_error_registration_is_explicit_and_conflicts_are_rejected() {
        use prebindgen_declarations::fun;
        let source="pub struct Error {pub code:i64} pub fn message(error:&Error)->String{} pub fn f()->Result<i64,Error>{}";
        assert!(Cbindgen::new(error_model(source))
            .function(fun!(f))
            .build()
            .err()
            .unwrap()
            .contains("explicit error_mapper"));
        assert!(Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .error("other", syn::parse_quote!(Error))
            .function(fun!(f))
            .build()
            .is_err());
        assert!(Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .error_mapper(syn::parse_quote!(Error), fun!(message))
            .function(fun!(f))
            .build()
            .err()
            .unwrap()
            .contains("cannot also select"));
        let generated = Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .function(fun!(f))
            .build()
            .unwrap();
        assert!(generated
            .header
            .contains("bool pb_f(int64_t *out0, pb_error *error)"));
        assert!(!generated.header.contains("pb_f_result *"));
        assert!(generated.header.contains("int64_t code;"));
    }
    #[test]
    fn error_field_names_follow_wire_subtrees_and_avoid_c_keywords() {
        use prebindgen_declarations::fun;
        let generated = Cbindgen::new(error_model(
            "pub struct Error {pub empty:(),pub value:Option<i64>} pub fn f()->Result<(),Error>{}",
        ))
        .error("error", syn::parse_quote!(Error))
        .function(fun!(f))
        .build()
        .unwrap();
        assert!(generated
            .header
            .contains("Field error.empty: no boundary values"));
        assert!(generated
            .header
            .contains("Field error.value: error->v0, error->v1"));
        let generated = Cbindgen::new(error_model(
            "pub struct Error {pub char:String,pub v0:i64} pub fn f()->Result<(),Error>{}",
        ))
        .error("error", syn::parse_quote!(Error))
        .function(fun!(f))
        .build()
        .unwrap();
        assert!(generated.header.contains("pb_owned_string v0;"));
        assert!(generated.header.contains("int64_t v1;"));
        assert!(!generated.header.contains("pb_buffer char;"));
    }

    #[test]
    fn unit_and_optional_handle_results_use_the_expected_explicit_outputs() {
        use prebindgen_declarations::fun;
        let source="pub type Handle=String; pub struct Error {pub code:i64} pub fn unit()->Result<(),Error>{} pub fn make()->Result<Handle,Error>{} pub fn maybe()->Result<Option<Handle>,Error>{}";
        let generated = Cbindgen::new(error_model(source))
            .opaque_ptr("handle", syn::parse_quote!(Handle))
            .error("error", syn::parse_quote!(Error))
            .function(fun!(unit))
            .function(fun!(make))
            .function(fun!(maybe))
            .build()
            .unwrap();
        assert!(generated.header.contains("bool pb_unit(pb_error *error)"));
        assert!(generated
            .header
            .contains("pb_handle * pb_make(pb_error *error)"));
        assert!(generated
            .header
            .contains("bool pb_maybe(pb_handle * *out0, pb_error *error)"));
        assert!(generated.header.contains("NULL means failure"));
    }

    #[test]
    fn multiple_collection_observations_follow_the_actual_borrow() {
        use prebindgen_declarations::{bind_output, Output};
        let seq = || Output::Sequence {
            delivery: prebindgen_declarations::SequenceDelivery::Value,
            element: Box::new(Output::Leaf),
            mode: None,
        };
        for (source, recipe) in [
            ("pub fn f()->&Vec<i64> {}", Output::combine([seq(), seq()])),
            (
                "pub struct Record { pub values: Vec<i64> } pub fn f()->Record {}",
                Output::fields([Output::combine([seq(), seq()])]),
            ),
        ] {
            let source = syn::parse_file(source).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            bind_output(&mut flat, &CCodec, "f", &recipe, &()).unwrap();
            assert_eq!(
                crate::group::result_packet_fields(
                    &flat
                        .function("f")
                        .unwrap()
                        .ret
                        .slot
                        .as_ref()
                        .unwrap()
                        .outcomes
                        .success
                )
                .len(),
                2
            );
            Cbindgen::new(flat).build().unwrap();
        }
    }

    #[test]
    fn option_comment_labels_are_unique_across_parameters_and_result() {
        use prebindgen_declarations::{bind_input, bind_output, Input, Output};
        let source = syn::parse_file("pub fn echo(a: Option<Option<char>>, b: Option<Option<char>>) -> Option<Option<char>> { a }").unwrap();
        let mut flat =
            Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let input = Input::Optional(Box::new(Input::Optional(Box::new(Input::Leaf))));
        for param in ["a", "b"] {
            bind_input(&mut flat, &CCodec, "echo", param, &input, &()).unwrap();
        }
        let output = Output::Optional(Box::new(Output::Optional(Box::new(Output::Leaf))));
        bind_output(&mut flat, &CCodec, "echo", &output, &()).unwrap();
        let header = Cbindgen::new(flat)
            .function(prebindgen_declarations::fun!(echo))
            .panic()
            .build()
            .unwrap()
            .header;
        for i in 0..6 {
            assert_eq!(header.matches(&format!("Option option{i}:")).count(), 1);
        }
        for parent in [0, 2, 4] {
            assert!(header.contains(&format!("only when option{parent} is present")));
        }
    }

    #[test]
    fn header_describes_shared_nested_empty_and_multiwire_fields() {
        use prebindgen_declarations::{bind_output, Output};
        let source = syn::parse_file("pub struct Empty {} pub struct Pair { pub x: i32, pub y: i32 } pub struct Record { pub empty: Empty, pub pair: Pair, pub maybe: Option<i32> } pub fn make() -> Record { unreachable!() }").unwrap();
        let mut flat =
            Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let recipe = Output::Fields(vec![
            Output::Fields(vec![]),
            Output::Fields(vec![Output::Leaf, Output::Leaf]),
            Output::Optional(Box::new(Output::Leaf)),
        ]);
        bind_output(&mut flat, &CCodec, "make", &recipe, &()).unwrap();
        let header = Cbindgen::new(flat).build().unwrap().header;
        assert!(header.contains("Field return.empty: no boundary values."));
        assert!(header.contains("Field return.pair: return value, *out1."));
        assert!(header.contains("Field return.pair.x: return value."));
        assert!(header.contains("Field return.pair.y: *out1."));
        assert!(header.contains("Field return.maybe: *out2, *out3."));
    }

    #[test]
    fn character_niches_exhaust_at_the_carrier_limit() {
        let wire: CWire = CWire::scalar(Primitive::Char);
        assert!(CCodec
            .remaining_niche(&wire, &CNiche::Integer(u32::MAX as i64))
            .is_none());
        assert!(matches!(
            CCodec.remaining_niche(&wire, &CNiche::Integer(0x110000)),
            Some(CNiche::Integer(0x110001))
        ));
    }

    #[test]
    fn shared_compiler_suppresses_niches_in_selector_arms() {
        use prebindgen_declarations::{bind_input, Input};
        for (payload, arm) in [
            ("char", Input::Leaf),
            ("Option<char>", Input::Optional(Box::new(Input::Leaf))),
        ] {
            let source = syn::parse_file(&format!("pub fn echo(value: {payload}) {{}}")).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            let selector = Input::Selector(vec![arm.clone(), arm.clone()]);
            bind_input(&mut flat, &CCodec, "echo", "value", &selector, &()).unwrap();
            assert_eq!(
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .len(),
                3
            );
            // Check suppression before an outer option can clear the arm niches.
            for wire in crate::group::function_parameters(flat.function("echo").unwrap())
                .unwrap()
                .skip(1)
            {
                assert!(matches!(
                    wire.kind,
                    CWire::Scalar {
                        primitive: Primitive::Char,
                        ..
                    }
                ));
                assert!(CCodec.niche(&wire.kind).is_none());
            }
            let source =
                syn::parse_file(&format!("pub fn echo(value: Option<{payload}>) {{}}")).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            let recipe = Input::Optional(Box::new(Input::Selector(vec![arm.clone(), arm])));
            bind_input(&mut flat, &CCodec, "echo", "value", &recipe, &()).unwrap();
            let plan = flat.function("echo").unwrap().params[0]
                .ty
                .slot
                .as_ref()
                .unwrap();
            // Presence flag + selector + two char carriers. Even a remaining
            // niche of an optional arm cannot describe the outer option.
            assert_eq!(
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .len(),
                4
            );
            let prebindgen_flat::WireNode::Chain {
                meta: CChain::Optional(outer_metadata),
                ..
            } = &plan.node
            else {
                panic!()
            };
            let optional = outer_metadata;
            assert!(optional.niche().is_none());
            assert_eq!(
                *optional.discriminator(),
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .next()
                    .unwrap()
                    .name
            );
        }
    }

    #[test]
    fn public_emission_rejects_invalid_c_type_names_without_panicking() {
        let mut prepared = Cbindgen::new(error_model("pub type Handle=String;pub enum E{A,B}"))
            .opaque_ptr("handle", syn::parse_quote!(Handle))
            .enum_type("choice", syn::parse_quote!(E))
            .prepare()
            .unwrap();
        prepared
            .type_info_mut(&syn::parse_quote!(crate::Handle))
            .unwrap()
            .metadata
            .pointer = Some("bad-name".into());
        assert!(generate(&prepared).is_err());
        prepared
            .type_info_mut(&syn::parse_quote!(crate::Handle))
            .unwrap()
            .metadata
            .pointer = Some("handle".into());
        let enumeration = prepared
            .type_info_mut(&syn::parse_quote!(crate::E))
            .unwrap()
            .metadata
            .enumeration
            .as_mut()
            .unwrap();
        std::rc::Rc::make_mut(enumeration).name = "bad-name".into();
        assert!(generate(&prepared).is_err());
    }

    #[test]
    fn public_emission_validates_abort_policy_and_module_settings() {
        let flat = error_model("pub fn f(c:char)->char{}");
        let mut prepared = Cbindgen::new(flat)
            .function(prebindgen_declarations::fun!(f))
            .panic()
            .prepare()
            .unwrap();
        prepared
            .function_mut("f")
            .unwrap()
            .metadata
            .as_mut()
            .unwrap()
            .abort_on_failure = false;
        assert!(generate(&prepared).err().unwrap().contains("abort policy"));
        prepared
            .function_mut("f")
            .unwrap()
            .metadata
            .as_mut()
            .unwrap()
            .abort_on_failure = true;
        prepared.metadata.prefix = "invalid-prefix".into();
        assert!(generate(&prepared).is_err());
        prepared.metadata.prefix = "test".into();
        prepared.function_mut("f").unwrap().metadata = None;
        assert!(generate(&prepared).is_err());
    }

    #[test]
    fn emission_reads_only_the_prepared_flat() {
        let source = syn::parse_file("pub enum E { A, B } pub fn echo(e:E)->E{}").unwrap();
        let flat =
            Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let prepared = Cbindgen::new(flat)
            .prefix("custom")
            .enum_type("choice", syn::parse_quote!(E))
            .function(prebindgen_declarations::fun!(echo))
            .panic()
            .prepare()
            .unwrap();
        assert!(prepared
            .type_info(&syn::parse_quote!(crate::E))
            .unwrap()
            .metadata
            .enumeration
            .is_some());
        let first = generate(&prepared).unwrap();
        let second = generate(&prepared.clone()).unwrap();
        assert_eq!(first.rust, second.rust);
        assert_eq!(first.header, second.header);
        assert!(first.header.contains("custom_choice"));
        assert!(first.header.contains("custom_echo"));
    }

    #[test]
    fn rejects_invalid_pointer_names_during_preparation() {
        let source = syn::parse_file("pub type Handle = String;").unwrap();
        let flat = Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let result = Cbindgen::new(flat)
            .opaque_ptr("invalid-name", syn::parse_quote!(Handle))
            .build();
        assert_eq!(result.err().unwrap(), "invalid identifier `invalid-name`");
    }

    #[test]
    fn recoverable_c_results_require_a_declared_binding_failure_channel() {
        use prebindgen_declarations::{fun, Declarations};

        let mut flat = error_model("pub fn message(e:i64)->String{} pub fn f()->Result<i64,i64>{}");
        let declarations = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
        declarations
            .fill(&mut flat, &CCodec, &fun!(f).on_error(fun!(message)), &())
            .unwrap();
        let output = flat.function_mut("f").unwrap().ret.slot.as_mut().unwrap();
        output.outcomes.binding_failure = None;
        flat.function_mut("f").unwrap().metadata = Some(CFunction::default());
        let error = generate(&flat).err().unwrap();
        assert!(error.contains("binding-failure schema"));
    }

    #[test]
    fn emits_custom_slot_code_without_building_a_plan() {
        #[allow(clippy::type_complexity)] // Custom slot storage keeps every metadata type explicit.
        struct CountedOutput(
            std::rc::Rc<std::cell::Cell<usize>>,
            prebindgen_flat::Outcomes<
                prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
            >,
        );
        impl prebindgen_flat::Conversion for CountedOutput {
            type Wire = CWire;
            type Chain = CChain;
            type Branch = CBranch;
            type SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, CContext>;

            fn code_fragment_slot(
                &self,
                mode: Self::SlotFragment<'_>,
                _: &syn::Type,
            ) -> Result<TokenStream> {
                use prebindgen_conversions::OutputFragment;
                Ok(match mode {
                    OutputFragment::Prepare { next, .. } => {
                        self.0.set(self.0.get() + 1);
                        quote!({ #next })
                    }
                    OutputFragment::Deliver { next, .. } => quote!({ #next }),
                    OutputFragment::Commit { .. } | OutputFragment::Result { .. } => quote!(()),
                })
            }
        }
        impl prebindgen_flat::OutputConversion for CountedOutput {
            fn output_outcomes(
                &self,
            ) -> &prebindgen_flat::Outcomes<
                prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
            > {
                &self.1
            }
        }
        struct CountedSchema;
        impl prebindgen_flat::Schema for CountedSchema {
            type Input = InputPlan<CWire>;
            type Output = CountedOutput;
            type Module = CModule;
            type Function = Option<CFunction>;
            type Type = CType;
            type Field = ();
            type Variant = ();
        }
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let source = syn::parse_file("pub fn noop() {}").unwrap();
        let mut flat = Flat::<CountedSchema>::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        flat.function_mut("noop").unwrap().ret.slot =
            Some(CountedOutput(count.clone(), Default::default()));
        flat.metadata.prefix = "test".into();
        flat.function_mut("noop").unwrap().metadata = Some(CFunction::default());
        let output = generate(&flat).unwrap();
        assert!(output.header.contains("test_noop("));
        assert_eq!(count.get(), 1);
    }
}

/// A wire value excluded from the representation of a present value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CNiche {
    /// Null address (pointer or integer-encoded handle).
    Null,
    /// An invalid integer representation, within the wire's integer range.
    Integer(i64),
}
impl CNiche {
    /// Expression recognizing the present case without decoding the payload.
    pub(crate) fn present(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Null => quote!((#value as usize) != 0),
            Self::Integer(n) => {
                let n = syn::parse_str::<syn::Expr>(&n.to_string()).unwrap();
                quote!(#value != #n)
            }
        }
    }
    /// Absence expression for a numeric wire. Pointer codecs supply a null pointer.
    pub(crate) fn integer_value(&self) -> TokenStream {
        match self {
            Self::Null => quote!(0),
            Self::Integer(n) => syn::parse_str::<TokenStream>(&n.to_string()).unwrap(),
        }
    }
}

#[cfg(test)]
mod abort_policy_tests {
    use super::*;
    use prebindgen_declarations::{expand_param, fun};
    fn model(source: &str) -> Bindings {
        Bindings::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn direct_values_and_void_do_not_need_an_error_policy() {
        let generated = Cbindgen::new(model("pub fn identity(x:i64)->i64{} pub fn unit(){}"))
            .function(fun!(identity))
            .function(fun!(unit))
            .build()
            .unwrap();
        assert!(generated.header.contains("int64_t pb_identity(int64_t"));
        assert!(generated.header.contains("void pb_unit(void)"));
        assert!(!generated.header.contains("_result"));
        assert!(!generated.header.contains("last_error"));
        assert!(!generated.rust.contains("thread_local"));
        syn::parse_file(&generated.rust).unwrap();
    }
    #[test]
    fn fallible_leaves_and_option_flags_require_explicit_abort_policy() {
        for ty in ["char", "&str", "&[u8]", "Option<i64>"] {
            let source = format!("pub fn f(value:{ty}){{}}");
            let error = Cbindgen::new(model(&source))
                .function(fun!(f))
                .build()
                .err()
                .unwrap();
            assert!(error.contains(".panic()"), "{error}");
            Cbindgen::new(model(&source))
                .function(fun!(f))
                .panic()
                .build()
                .unwrap();
        }
    }
    #[test]
    fn constructor_alternatives_retain_their_own_validation_policy() {
        let source="pub struct Number {pub value:i64} pub fn new(value:i64)->Number{} pub fn other(value:i64)->Number{} pub fn consume(value:Number){}";
        let bind = || {
            Cbindgen::new(model(source)).function(
                fun!(consume).expand_param(
                    "value",
                    expand_param!(Number)
                        .variant(fun!(new))
                        .variant(fun!(other)),
                ),
            )
        };
        assert!(bind().build().err().unwrap().contains(".panic()"));
        bind().panic().build().unwrap();
    }
    #[test]
    fn manually_bound_fallible_output_requires_the_same_policy() {
        let mut flat = model("pub fn f()->i64{}");
        prebindgen_declarations::bind_output(
            &mut flat,
            &CCodec,
            "f",
            &prebindgen_declarations::Output::Leaf,
            &(),
        )
        .unwrap();
        flat.function_mut("f")
            .unwrap()
            .ret
            .slot
            .as_mut()
            .unwrap()
            .fallible = true;
        assert!(Cbindgen::new(flat.clone())
            .build()
            .err()
            .unwrap()
            .contains(".panic()"));
        Cbindgen::new(flat)
            .function(fun!(f))
            .panic()
            .build()
            .unwrap();
    }
}
