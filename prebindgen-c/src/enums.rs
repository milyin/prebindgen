//! Explicit enum declarations and the enum-aware C conversion codec.
use super::*;
use prebindgen_declarations::{Condition, EnumMapping};

pub(crate) struct EnumCodec {
    pub(crate) errors: std::collections::BTreeMap<String, String>,
    pub(crate) enums: std::collections::BTreeMap<String, (String, EnumMapping)>,
}
impl EnumCodec {
    fn mapping(&self, ty: &syn::Type) -> Option<(&str, &EnumMapping)> {
        let ty = if let syn::Type::Reference(r) = ty {
            &r.elem
        } else {
            ty
        };
        self.enums
            .get(&prebindgen_flat::type_key(ty))
            .map(|(name, m)| (name.as_str(), m))
    }
    fn wire(name: &str, mapping: &EnumMapping) -> CWire {
        CWire::enumeration(name, mapping.clone())
    }
}
impl Codec for EnumCodec {
    type Wire = CWire;
    type Niche = CNiche;
    type Group = CGroup;
    type Extra = ();
    type Context = CContext;
    type Input = InputPlan<CWire>;
    type Output = OutputPlan<CWire>;
    fn optional_group(&self, p: syn::Ident, d: syn::Ident, n: Option<CNiche>) -> CGroup {
        CCodec.optional_group(p, d, n)
    }
    fn field_group(&self, n: syn::Ident) -> CGroup {
        CCodec.field_group(n)
    }
    fn fields_group(&self) -> CGroup {
        CCodec.fields_group()
    }
    fn finish_input(&self, p: &mut InputPlan<CWire>, e: &()) -> Result<()> {
        CCodec.finish_input(p, e)
    }
    fn finish_output(&self, p: &mut OutputPlan<CWire>, e: &()) -> Result<()> {
        CCodec.finish_output(p, e)
    }
    fn result_error_value(
        &self,
        ty: &syn::Type,
        value: TokenStream,
        message: Option<TokenStream>,
        extra: &(),
    ) -> Result<TokenStream> {
        if let Some(name) = self.errors.get(&prebindgen_flat::type_key(ty)) {
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
            CCodec.result_error_value(ty, value, message, extra)
        }
    }
    fn result_error(
        &self,
        ty: &syn::Type,
        message: TokenStream,
        extra: &(),
    ) -> Result<TokenStream> {
        CCodec.result_error(ty, message, extra)
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
    fn consumes(&self, ty: &syn::Type, e: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(inner, e);
        }
        self.mapping(ty).is_none() && CCodec.consumes(ty, e)
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        e: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<CWire>> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{w}_present");
            let child = self.input_leaf(
                inner,
                e,
                format_ident!("{w}_some"),
                format_ident!("{v}_some"),
                &active.arm(tag.clone(), 1),
            )?;
            return prebindgen_declarations::optional_input(self, e, tag, v, active, child);
        }
        if let Some((name, m)) = self.mapping(ty) {
            return prebindgen_declarations::enum_input(m, ty, w, v, active, Self::wire(name, m));
        }
        CCodec.input_leaf(ty, e, w, v, active)
    }
    fn output_leaf(
        &self,
        ty: &syn::Type,
        e: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<CWire>> {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = format_ident!("{w}_present");
            let payload = format_ident!("{w}_payload");
            let child = self.output_leaf(
                inner,
                e,
                format_ident!("{w}_some"),
                quote!(#payload),
                borrowed,
            )?;
            return prebindgen_declarations::optional_output(
                self, e, tag, payload, v, borrowed, child,
            );
        }
        if let Some((name, m)) = self.mapping(ty) {
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
        CCodec.output_leaf(ty, e, w, v, borrowed)
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
    fn enum_declarations_emit_fixed_width_types_and_preserve_discriminants() {
        let output = Cbindgen::new(flat(
            "pub enum E { A = -2, B, C = 7, D, Min = -2147483648 } pub fn echo(x:E)->E{}",
        ))
        .enum_type("code", syn::parse_quote!(E))
        .function(prebindgen_declarations::fun!(echo))
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
        for name in ["buffer", "int128", "H", "last_error", "free_buffer"] {
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
                .build()
                .err()
                .unwrap()
                .contains("duplicate C result type")
        );
    }
    #[test]
    fn enum_niches_skip_valid_values_and_survive_optional_composition() {
        let source = flat("pub enum E { A = -2, B, C = 0, D = 7 }");
        let mapping = EnumMapping::new(source.enums.values().next().unwrap()).unwrap();
        let codec = EnumCodec {
            errors: Default::default(),
            enums: std::collections::BTreeMap::from([(
                "crate :: E".into(),
                ("code".into(), mapping.clone()),
            )]),
        };
        let wire = EnumCodec::wire("code", &mapping);
        assert!(matches!(wire.available_niche(), Some(CNiche::Integer(-3))));
        assert!(matches!(
            codec.remaining_niche(&wire, &CNiche::Integer(8)),
            Some(CNiche::Integer(6))
        ));
        let mut ty: syn::Type = syn::parse_quote!(crate::E);
        for depth in 1..=6 {
            ty = syn::parse_quote!(Option<#ty>);
            let input = codec
                .input_leaf(
                    &ty,
                    &(),
                    syn::parse_quote!(wire),
                    syn::parse_quote!(value),
                    &Condition::default(),
                )
                .unwrap();
            let output = codec
                .output_leaf(&ty, &(), syn::parse_quote!(wire), quote!(value), false)
                .unwrap();
            assert_eq!(input.layout.len(), 1);
            assert_eq!(output.layout.len(), 1);
            assert!(
                matches!(input.layout[0].kind.available_niche(),Some(CNiche::Integer(n)) if n == -3-depth)
            );
            super::super::group::validate_layout(&input.layout).unwrap();
            super::super::group::validate_layout(&output.layout).unwrap();
        }
    }
    #[test]
    fn optional_enum_reservations_cannot_reuse_available_or_reserved_values() {
        let source = flat("pub enum E { A = -1, B = 0 }");
        let mapping = EnumMapping::new(source.enums.values().next().unwrap()).unwrap();
        let wire = EnumCodec::wire("code", &mapping);
        let mut tree: prebindgen_flat::WireLayout<CWire, CGroup> = vec![GenericWire {
            name: syn::parse_quote!(wire),
            kind: wire,
        }]
        .into();
        let group = |presence, value| {
            CGroup::Optional(COptional {
                presence: syn::parse_str(presence).unwrap(),
                discriminator: syn::parse_quote!(wire),
                niche: Some(CNiche::Integer(value)),
            })
        };
        tree.enclose(group("inner", -2));
        assert!(super::super::group::validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        tree[0].kind.clear_niche();
        super::super::group::validate_layout(&tree).unwrap();
        tree.enclose(group("outer", -2));
        assert!(super::super::group::validate_layout(&tree)
            .unwrap_err()
            .contains("duplicate"));
    }
    #[test]
    fn malformed_enum_niches_are_rejected_before_emission() {
        let source = flat("pub enum E { A = -1, B = 0 }");
        let mapping = EnumMapping::new(source.enums.values().next().unwrap()).unwrap();
        for n in [-1, 0] {
            let mut wire = EnumCodec::wire("code", &mapping);
            let CWire::Enum {
                next_integer_niche, ..
            } = &mut wire
            else {
                unreachable!()
            };
            *next_integer_niche = Some(n);
            let layout: prebindgen_flat::WireLayout<CWire, CGroup> = vec![GenericWire {
                name: syn::parse_quote!(value),
                kind: wire,
            }]
            .into();
            assert!(super::super::group::validate_layout(&layout).is_err());
        }
    }
}
