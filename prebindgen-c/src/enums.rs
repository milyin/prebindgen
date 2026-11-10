//! Declared C enum metadata and conversion helpers.
use super::*;
use prebindgen_declarations::EnumMapping;
impl CCodec {
    pub(super) fn enum_mapping<'a>(
        &self,
        flat: &'a Bindings,
        ty: &syn::Type,
    ) -> Option<(&'a str, &'a std::rc::Rc<EnumMapping>)> {
        if !self.declared {
            return None;
        }
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
    pub(super) fn enum_wire(name: &str, mapping: &std::rc::Rc<EnumMapping>) -> CWire {
        CWire::enumeration(name, mapping.clone())
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
        let codec = CCodec::declared();
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
                .output_leaf(prebindgen_declarations::OutputLeaf::new(
                    &model,
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    prebindgen_declarations::ValueAccess::Owned,
                ))
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
            .contains("both opaque_ptr and opaque_owned_struct"));
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
        let codec = CCodec::declared();
        let wire = CCodec::enum_wire("code", &mapping);
        assert!(matches!(wire.available_niche(), Some(CNiche::Integer(-3))));
        assert!(matches!(
            wire.remaining_niche(&CNiche::Integer(8)),
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
        let wire = CCodec::enum_wire("code", &mapping);
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
            let mut wire = CCodec::enum_wire("code", &mapping);
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
