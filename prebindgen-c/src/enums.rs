//! Explicit enum declarations and the enum-aware C conversion codec.
use super::*;
use prebindgen_declarations::{Condition, EnumMapping};

/// C conversions read representation and error declarations from Flat.
pub(crate) struct EnumCodec;
impl EnumCodec {
    fn mapping<'a>(
        &self,
        flat: &'a Bindings,
        ty: &syn::Type,
    ) -> Option<(&'a str, &'a std::rc::Rc<EnumMapping>)> {
        let ty = if let syn::Type::Reference(r) = ty {
            &r.elem
        } else {
            ty
        };
        flat.type_info(ty)
            .ok()?
            .metadata
            .enumeration
            .as_ref()
            .map(|e| (e.name.as_str(), &e.mapping))
    }
    fn wire(name: &str, mapping: &std::rc::Rc<EnumMapping>) -> CWire {
        CWire::enumeration(name, mapping.clone())
    }
}
impl Codec for EnumCodec {
    type Schema = CSchema;
    type Wire = CWire;
    type Niche = CNiche;
    type Chain = CChain;
    type Branch = CBranch;
    fn group_metadata(&self) -> CBranch {
        CBranch::Group(CGroup::Values)
    }

    type Extra = ();
    type Context = CContext;
    fn named_output_group(&self, name: &str) -> Result<CBranch> {
        CCodec.named_output_group(name)
    }

    fn binding_failure_layout(
        &self,
        e: &(),
    ) -> Result<Option<prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>>> {
        CCodec.binding_failure_layout(e)
    }
    fn source_error_layout(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        mapped: bool,
        e: &(),
    ) -> Result<prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>> {
        if let Some(layout) = flat
            .type_info(ty)
            .ok()
            .and_then(|t| t.metadata.error.as_ref())
        {
            return Ok(layout.plan()?.outcomes.success.clone());
        }
        CCodec.source_error_layout(flat, ty, mapped, e)
    }
    fn optional_metadata(
        &self,
        p: syn::Ident,
        d: syn::Ident,
        n: Option<CNiche>,
        controls: &[prebindgen_flat::Wire<CWire>],
    ) -> Result<CChain> {
        CCodec.optional_metadata(p, d, n, controls)
    }
    fn alternatives_metadata(&self, selector: &prebindgen_flat::Wire<CWire>) -> Result<CBranch> {
        CCodec.alternatives_metadata(selector)
    }
    fn field_group(&self, n: syn::Ident) -> CBranch {
        CCodec.field_group(n)
    }
    fn fields_group(&self) -> CBranch {
        CCodec.fields_group()
    }
    fn finish_input(&self, p: &mut InputPlan<CWire>, e: &()) -> Result<()> {
        CCodec.finish_input(p, e)
    }
    fn finish_output(&self, p: &mut OutputPlan<CWire>, e: &()) -> Result<()> {
        CCodec.finish_output(p, e)
    }
    fn sequence_input(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        active: &Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<CWire, crate::CChain>> {
        CCodec.sequence_input(e, w, n, i, element, active)
    }
    fn sequence_output(
        &self,
        e: &(),
        w: syn::Ident,
        n: syn::Ident,
        i: syn::Ident,
        element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        values: &[(syn::Ident, TokenStream)],
        delivery: prebindgen_declarations::SequenceDelivery,
    ) -> Result<prebindgen_declarations::SequenceOutput<CWire, crate::CChain>> {
        CCodec.sequence_output(e, w, n, i, element, values, delivery)
    }
    fn result_error_value(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        value: TokenStream,
        message: Option<TokenStream>,
        extra: &(),
    ) -> Result<TokenStream> {
        if let Some(name) = flat
            .type_info(ty)
            .ok()
            .and_then(|t| t.metadata.error.as_ref())
            .map(|e| e.name.as_str())
        {
            if message.is_some() {
                return Err("structured C errors cannot also select a message mapper".into());
            }
            let helper = errors::helper(name);
            let pointer = prebindgen_conversions::template_parameter("error_pointer");
            let reported = prebindgen_conversions::template_parameter("error_reported");
            Ok(
                quote!({unsafe {#helper(#value,#pointer,&mut #reported)}?;#reported=true;String::new()}),
            )
        } else {
            CCodec.result_error_value(flat, ty, value, message, extra)
        }
    }
    fn result_error(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        message: TokenStream,
        extra: &(),
    ) -> Result<TokenStream> {
        CCodec.result_error(flat, ty, message, extra)
    }
    fn niche(&self, w: &CWire) -> Option<CNiche> {
        CCodec.niche(w)
    }
    fn set_niche(&self, w: &mut CWire, n: Option<CNiche>) {
        CCodec.set_niche(w, n)
    }
    fn remaining_niche(&self, w: &CWire, n: &CNiche) -> Option<CNiche> {
        CCodec.remaining_niche(w, n)
    }
    fn is_present(&self, w: &CWire, n: &CNiche, v: TokenStream) -> Result<TokenStream> {
        CCodec.is_present(w, n, v)
    }
    fn absent_value(&self, w: &CWire, n: Option<&CNiche>, input: bool) -> Result<TokenStream> {
        CCodec.absent_value(w, n, input)
    }
    fn selector(&self, e: &()) -> CWire {
        CCodec.selector(e)
    }
    fn consumes(&self, flat: &prebindgen_flat::Flat<Self::Schema>, ty: &syn::Type, e: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(flat, inner, e);
        }
        self.mapping(flat, ty).is_none() && CCodec.consumes(flat, ty, e)
    }
    fn input_leaf(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        e: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<CWire>> {
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        if let Some((name, m)) = self.mapping(flat, ty) {
            return prebindgen_declarations::enum_input(m, ty, w, v, active, Self::wire(name, m));
        }
        CCodec.input_leaf(flat, ty, e, w, v, active)
    }
    fn output_leaf(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &syn::Type,
        e: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<CWire>> {
        if prebindgen_declarations::option_inner(ty).is_some() {
            return Err("resolve Optional operations before leaf compilation".into());
        }
        if let Some((name, m)) = self.mapping(flat, ty) {
            if matches!(ty,syn::Type::Reference(r) if r.mutability.is_some()) {
                return Err("mutable-reference outputs are unsupported".into());
            }
            return Ok(prebindgen_declarations::enum_output(
                m,
                w,
                v,
                Self::wire(name, m),
            ));
        }
        CCodec.output_leaf(flat, ty, e, w, v, borrowed)
    }

    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<prebindgen_declarations::NicheReservation<Self::Niche>> {
        CCodec.optional_niche(payload)
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &prebindgen_declarations::NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        CCodec.optional_presence(payload, reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        CCodec.absent_output_values(payload, reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        CCodec.retire_niches(payload, reservation)
    }
    fn validate_input(&self, plan: &prebindgen_declarations::CodecInput<Self>) -> Result<()> {
        CCodec.validate_input(plan)
    }
    fn validate_output(&self, plan: &prebindgen_declarations::CodecOutput<Self>) -> Result<()> {
        CCodec.validate_output(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn flat(source: &str) -> Bindings {
        Flat::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn codec_reads_enum_declarations_from_flat_after_creation() {
        let codec = EnumCodec;
        let mut model = flat("pub enum E { A, B }pub fn echo(x:E)->E{}");
        let ty = model.resolve_type(&syn::parse_quote!(E)).unwrap();
        let mapping =
            std::rc::Rc::new(EnumMapping::new(model.enums.values().next().unwrap()).unwrap());
        for name in ["first", "updated"] {
            model.type_metadata_mut(&ty).unwrap().enumeration = Some(std::rc::Rc::new(CEnum {
                name: name.into(),
                mapping: mapping.clone(),
            }));
            let output = codec
                .output_leaf(
                    &model,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    false,
                )
                .unwrap();
            let wire = prebindgen_flat::wire_definitions(std::iter::once(&output.outcomes.success))
                .next()
                .unwrap();
            assert!(matches!(&wire.kind,CWire::Enum{name:actual,..} if actual==name));
        }
    }
    #[test]
    fn enum_declarations_emit_fixed_width_types_and_preserve_discriminants() {
        let output = Cbindgen::new(flat(
            "pub enum E { A = -2, B, C = 7, D, Min = -2147483648 } pub fn echo(x:E)->E{}",
        ))
        .enum_type("code", syn::parse_quote!(E))
        .function(prebindgen_declarations::fun!(echo))
        .panic()
        .build()
        .unwrap();
        assert!(output.header.contains("typedef int32_t pb_code;"));
        assert!(output.header.contains("#define pb_code_B ((pb_code)(-1))"));
        assert!(output.header.contains("#define pb_code_D ((pb_code)(8))"));
        assert!(output
            .header
            .contains("#define pb_code_Min ((pb_code)(INT32_MIN))"));
        syn::parse_file(&output.rust).unwrap();
    }
    #[test]
    fn enum_declarations_reject_wrong_types_duplicate_sources_names_and_symbols() {
        let source = "pub enum E { A, B } pub fn code(x:E)->E {}";
        assert!(Cbindgen::new(flat(source))
            .enum_type("code", syn::parse_quote!(E))
            .function(prebindgen_declarations::fun!(code))
            .panic()
            .build()
            .err()
            .unwrap()
            .contains("duplicate C export"));
        assert!(Cbindgen::new(flat(source))
            .enum_type("e", syn::parse_quote!(E))
            .enum_type("other", syn::parse_quote!(E))
            .build()
            .err()
            .unwrap()
            .contains("duplicate C enum source"));
        assert!(Cbindgen::new(flat("pub type Alias = hidden::E;"))
            .enum_type("alias", syn::parse_quote!(Alias))
            .build()
            .err()
            .unwrap()
            .contains("requires a fieldless enum"));
        assert!(Cbindgen::new(flat(source))
            .enum_type("e", syn::parse_quote!(E))
            .opaque_ptr("e", syn::parse_quote!(E))
            .build()
            .err()
            .unwrap()
            .contains("names collide"));
    }
    #[test]
    fn enum_symbols_cannot_shadow_runtime_or_generated_result_types() {
        for name in ["buffer", "int128", "H", "free_buffer"] {
            assert!(
                Cbindgen::new(flat("pub enum E { A, B }"))
                    .enum_type(name, syn::parse_quote!(E))
                    .build()
                    .is_err(),
                "{name}"
            );
        }
        assert!(
            Cbindgen::new(flat("pub enum E { A, B } pub fn echo(x:E)->E{}"))
                .enum_type("echo_result", syn::parse_quote!(E))
                .function(prebindgen_declarations::fun!(echo))
                .panic()
                .build()
                .err()
                .unwrap()
                .contains("duplicate C result type")
        );
    }
    #[test]
    fn enum_niches_skip_valid_values_and_survive_optional_composition() {
        let source = flat("pub enum E { A = -2, B, C = 0, D = 7 }");
        let mapping =
            std::rc::Rc::new(EnumMapping::new(source.enums.values().next().unwrap()).unwrap());
        let codec = EnumCodec;
        let wire = EnumCodec::wire("code", &mapping);
        assert!(matches!(wire.available_niche(), Some(CNiche::Integer(-3))));
        assert!(matches!(
            codec.remaining_niche(&wire, &CNiche::Integer(8)),
            Some(CNiche::Integer(6))
        ));
        let mut ty: syn::Type = syn::parse_quote!(E);
        for depth in 1..=6 {
            ty = syn::parse_quote!(Option<#ty>);
            let mut model = flat(&format!(
                "pub enum E {{ A = -2, B, C = 0, D = 7 }} pub fn echo(value:{})->{}{{}}",
                quote!(#ty),
                quote!(#ty)
            ));
            model
                .type_metadata_mut(&syn::parse_quote!(E))
                .unwrap()
                .enumeration = Some(std::rc::Rc::new(CEnum {
                name: "code".into(),
                mapping: mapping.clone(),
            }));
            prebindgen_declarations::bind_input(
                &mut model,
                &codec,
                "echo",
                "value",
                &prebindgen_declarations::Input::auto(),
                &(),
            )
            .unwrap();
            prebindgen_declarations::bind_output(
                &mut model,
                &codec,
                "echo",
                &prebindgen_declarations::Output::auto(),
                &(),
            )
            .unwrap();
            let function = model.function("echo").unwrap();
            let input = function.params[0].ty.slot.as_ref().unwrap();
            let output = function.ret.slot.as_ref().unwrap();
            assert_eq!(
                prebindgen_flat::wire_definitions(std::iter::once(&input.node)).count(),
                1
            );
            assert_eq!(
                crate::group::result_packet_fields(&output.outcomes.success).len(),
                1
            );
            assert!(
                matches!(prebindgen_flat::wire_definitions(std::iter::once(&input.node)).next().unwrap().kind.available_niche(),Some(CNiche::Integer(n)) if n == -3-depth)
            );
            super::super::group::validate_layout(&input.node).unwrap();
            super::super::group::validate_layout(&output.outcomes.success).unwrap();
        }
    }
    #[test]
    fn optional_enum_reservations_cannot_reuse_available_or_reserved_values() {
        let source = flat("pub enum E { A = -1, B = 0 }");
        let mapping =
            std::rc::Rc::new(EnumMapping::new(source.enums.values().next().unwrap()).unwrap());
        let wire = EnumCodec::wire("code", &mapping);
        let mut tree: prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch> =
            prebindgen_flat::WireNode::Leaf(vec![GenericWire {
                name: syn::parse_quote!(wire),
                kind: wire,
            }]);
        let group = |presence, value| {
            COptional::from_parts(
                syn::parse_str(presence).unwrap(),
                syn::parse_quote!(wire),
                Some(CNiche::Integer(value)),
                &[],
            )
            .unwrap()
        };
        tree = prebindgen_flat::WireNode::Chain {
            controls: vec![],
            meta: CChain::Optional(group("inner", -2)),
            child: Box::new(std::mem::take(&mut tree)),
        };
        assert!(super::super::group::validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        prebindgen_flat::wire_definitions_mut(std::iter::once(&mut tree))
            .next()
            .expect("logical wire index out of range")
            .kind
            .clear_niche();
        super::super::group::validate_layout(&tree).unwrap();
        tree = prebindgen_flat::WireNode::Chain {
            controls: vec![],
            meta: CChain::Optional(group("outer", -2)),
            child: Box::new(std::mem::take(&mut tree)),
        };
        assert!(super::super::group::validate_layout(&tree)
            .unwrap_err()
            .contains("duplicate"));
    }
    #[test]
    fn malformed_enum_niches_are_rejected_before_emission() {
        let source = flat("pub enum E { A = -1, B = 0 }");
        let mapping =
            std::rc::Rc::new(EnumMapping::new(source.enums.values().next().unwrap()).unwrap());
        for n in [-1, 0] {
            let mut wire = EnumCodec::wire("code", &mapping);
            let CWire::Enum {
                next_integer_niche, ..
            } = &mut wire
            else {
                unreachable!()
            };
            *next_integer_niche = Some(n);
            let layout: prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch> =
                prebindgen_flat::WireNode::Leaf(vec![GenericWire {
                    name: syn::parse_quote!(value),
                    kind: wire,
                }]);
            assert!(super::super::group::validate_layout(&layout).is_err());
        }
    }
}
