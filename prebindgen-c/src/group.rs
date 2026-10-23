//! C adapter-owned descriptions of the wire tree.
use super::*;
use prebindgen_conversions::NativeReader;

use prebindgen_flat::{WireLayout, WireNode};
use syn::ext::IdentExt;

/// Metadata interpreted by the C binding writer.
#[derive(Clone, Debug)]
pub enum CGroup {
    /// Metadata for `WireNode::Optional`, including its reserved absence value.
    Optional(COptional),
    /// Physical carrier chosen for this logical sequence; elements stay in the tree.
    Sequence(prebindgen_flat::Wire<CWire>),
    /// Selector for this logical alternatives node; payloads stay in the tree.
    Alternatives(prebindgen_flat::Wire<CWire>),
    /// An ordered collection of field subtrees.
    Fields,
    /// A named source field, including empty and multi-wire fields.
    Field(syn::Ident),
}
/// Optional-value information retained by the C adapter.
#[derive(Clone, Debug)]
pub struct COptional {
    /// Explicit native presence controls; empty when a payload niche is used.
    pub controls: Vec<prebindgen_flat::Wire<CWire>>,
    /// Normalized presence binding used by payload conditions.
    pub presence: syn::Ident,
    /// Payload wire or inline control inspected for presence.
    pub discriminator: syn::Ident,
    /// Reserved absent representation; `None` means a separate 0/1 flag.
    pub niche: Option<CNiche>,
}

impl prebindgen_conversions::NativeReader<CWire, CGroup> for CCodec {
    fn wires(
        nodes: &[WireNode<CWire, CGroup>],
    ) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
        let mut out = vec![];
        read(nodes, &mut out, false);
        out.into_iter()
    }
    fn wires_mut(
        nodes: &mut [WireNode<CWire, CGroup>],
    ) -> std::vec::IntoIter<&mut prebindgen_flat::Wire<CWire>> {
        fn visit<'a>(
            nodes: &'a mut [WireNode<CWire, CGroup>],
            out: &mut Vec<&'a mut prebindgen_flat::Wire<CWire>>,
        ) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group { children, .. } => visit(children.nodes_mut(), out),
                    WireNode::Optional(c) => {
                        if let Some(CGroup::Optional(o)) = &mut c.metadata {
                            out.extend(&mut o.controls);
                        }
                        visit(c.element.nodes_mut(), out);
                    }
                    WireNode::Sequence(c) => {
                        if let Some(CGroup::Sequence(w)) = &mut c.metadata {
                            out.push(w);
                        }
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(CGroup::Alternatives(w)) = metadata {
                            out.push(w);
                        }
                        for arm in alternatives {
                            visit(arm.nodes_mut(), out);
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        if let Some(o) = alternatives
                            .iter_mut()
                            .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                        {
                            visit(o.layout.nodes_mut(), out);
                        }
                    }
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn validate(layout: &WireLayout<CWire, CGroup>) -> Result<()> {
        validate_layout(layout)
    }
}
// C/JNI parameters and row fields have different scopes. Validation separately
// reads all definitions to check the identifiers used by generated Rust code.
fn read<'a>(
    nodes: &'a [WireNode<CWire, CGroup>],
    out: &mut Vec<&'a prebindgen_flat::Wire<CWire>>,
    elements: bool,
) {
    for node in nodes {
        match node {
            WireNode::Wire(w) => out.push(w),
            WireNode::Group { children, .. } => read(children.nodes(), out, elements),
            WireNode::Optional(c) => {
                if let Some(CGroup::Optional(o)) = &c.metadata {
                    out.extend(&o.controls);
                }
                read(c.element.nodes(), out, elements);
            }
            WireNode::Sequence(c) => {
                if let Some(CGroup::Sequence(w)) = &c.metadata {
                    out.push(w);
                }
                if elements {
                    read(c.element.nodes(), out, elements);
                }
            }
            WireNode::Alternatives {
                metadata,
                alternatives,
            } => {
                if let Some(CGroup::Alternatives(w)) = metadata {
                    out.push(w);
                }
                for arm in alternatives {
                    read(arm.nodes(), out, elements);
                }
            }
            WireNode::Outcomes { alternatives } => {
                if let Some(o) = alternatives
                    .iter()
                    .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                {
                    read(o.layout.nodes(), out, elements);
                }
            }
        }
    }
}
impl CGroup {
    pub(crate) fn sequence_wire(&self) -> Option<&prebindgen_flat::Wire<CWire>> {
        if let Self::Sequence(w) = self {
            Some(w)
        } else {
            None
        }
    }
}
pub(crate) fn validate_layout(layout: &WireLayout<CWire, CGroup>) -> Result<()> {
    layout.validate()?;
    if let [WireNode::Outcomes { alternatives }] = layout.nodes() {
        for arm in alternatives {
            validate_layout(&arm.layout)?;
        }
        return Ok(());
    }
    let mut definitions = vec![];
    read(layout.nodes(), &mut definitions, true);
    prebindgen_conversions::validate_wire_names(definitions)?;
    for wire in CCodec::wires(layout.nodes()) {
        wire.kind.validate()?;
    }
    fn visit(
        layout: &WireLayout<CWire, CGroup>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<CNiche>>,
    ) -> Result<()> {
        for node in layout.nodes() {
            match node {
                WireNode::Alternatives {
                    metadata,
                    alternatives,
                } => {
                    let Some(CGroup::Alternatives(selector)) = metadata else {
                        return Err("alternatives require selector metadata".into());
                    };
                    if !matches!(
                        selector.kind,
                        CWire::Scalar {
                            primitive: Primitive::I32,
                            ..
                        }
                    ) {
                        return Err("alternatives require an integer selector".into());
                    }
                    for arm in alternatives {
                        visit(arm, bindings, false, reserved)?;
                    }
                }
                WireNode::Sequence(c) => {
                    let Some(CGroup::Sequence(carrier)) = &c.metadata else {
                        return Err("sequence requires a separate carrier".into());
                    };
                    if !(matches!(
                        carrier.kind,
                        CWire::Rows
                            | CWire::Scalar {
                                primitive: Primitive::Usize,
                                ..
                            }
                    )) || CCodec::wires(c.element.nodes()).len() == 0
                    {
                        return Err(
                            "sequence requires a supported carrier and nonempty element layout"
                                .into(),
                        );
                    }
                    validate_layout(&c.element)?;
                }
                WireNode::Optional(c) => {
                    let Some(CGroup::Optional(o)) = &c.metadata else {
                        return Err(
                            "optional node requires optional metadata and inline optional encoding"
                                .into(),
                        );
                    };
                    let controls = &o.controls;
                    if !bindings.insert(o.presence.to_string()) {
                        return Err(format!(
                            "duplicate optional presence binding `{}`",
                            o.presence
                        ));
                    }
                    if let Some(niche) = &o.niche {
                        if !controls.is_empty() {
                            return Err("niche optional must not have control wires".into());
                        }
                        let wire = CCodec::wires(c.element.nodes())
                            .find(|w| w.name == o.discriminator)
                            .ok_or("optional discriminator outside payload")?;
                        wire.kind.validate_niche(niche)?;
                        if wire.kind.available_niche().as_ref() == Some(niche) {
                            return Err(
                                "available C niche is already reserved by an optional node".into(),
                            );
                        }
                        let values = reserved.entry(o.discriminator.to_string()).or_default();
                        if values.contains(niche) {
                            return Err("duplicate C niche reservation on one wire".into());
                        }
                        values.push(niche.clone());
                    } else if !matches!(controls.as_slice(),[flag] if flag.name==o.discriminator && o.presence==o.discriminator && matches!(flag.kind,CWire::Scalar { primitive: Primitive::I32, .. }))
                    {
                        return Err("optional flag must be its sole inline control wire".into());
                    }
                    visit(&c.element, bindings, false, reserved)?;
                }
                WireNode::Outcomes { alternatives } => {
                    for arm in alternatives {
                        validate_layout(&arm.layout)?;
                    }
                }
                WireNode::Wire(w) if matches!(w.kind, CWire::Rows) => {
                    return Err("array carrier requires a container node".into())
                }
                WireNode::Wire(_) => {}
                WireNode::Group { metadata, children } => {
                    if matches!(metadata, CGroup::Optional(_)) {
                        return Err("optional metadata requires an Optional node".into());
                    }
                    if matches!(metadata, CGroup::Sequence(_) | CGroup::Alternatives(_)) {
                        return Err("encoding metadata must be on its matching logical node".into());
                    }
                    match metadata {
                        CGroup::Field(_) if !in_fields => {
                            return Err(
                                "named field must be a direct child of a field collection".into()
                            )
                        }
                        CGroup::Fields => {
                            let mut fields = std::collections::BTreeSet::new();
                            for child in children.nodes() {
                                let WireNode::Group {
                                    metadata: CGroup::Field(name),
                                    ..
                                } = child
                                else {
                                    return Err(
                                        "field collection requires named field children".into()
                                    );
                                };
                                if !fields.insert(name.unraw().to_string()) {
                                    return Err(format!("duplicate field `{name}`"));
                                }
                            }
                        }
                        _ => {}
                    }
                    visit(
                        children,
                        bindings,
                        matches!(metadata, CGroup::Fields),
                        reserved,
                    )?;
                }
            }
        }
        Ok(())
    }
    visit(
        layout,
        &mut std::collections::BTreeSet::new(),
        false,
        &mut std::collections::BTreeMap::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn enclose_optional_metadata(tree: &mut WireLayout<CWire, CGroup>, mut metadata: CGroup) {
        let flag = matches!(&metadata,CGroup::Optional(o) if o.niche.is_none());
        let mut nodes = std::mem::take(tree).into_nodes();
        // These fixtures describe an explicit flag as the first physical wire.
        // Move it into the representation without changing its wire type.
        let controls = if flag && matches!(nodes.first(), Some(WireNode::Wire(_))) {
            let WireNode::Wire(wire) = nodes.remove(0) else {
                unreachable!()
            };
            vec![wire]
        } else {
            vec![]
        };
        *tree = WireLayout::from_nodes(nodes);
        if let CGroup::Optional(o) = &mut metadata {
            o.controls = controls;
        }
        tree.enclose_optional(Some(metadata));
    }
    fn leaf(name: &str) -> prebindgen_flat::Wire<CWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: CWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: bool) -> CGroup {
        CGroup::Optional(COptional {
            controls: vec![],
            presence: syn::parse_str(presence).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche: niche.then_some(CNiche::Integer(0x110000)),
        })
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        let metadata = || {
            CGroup::Optional(COptional {
                controls: vec![],
                presence: syn::parse_quote!(flag),
                discriminator: syn::parse_quote!(flag),
                niche: None,
            })
        };
        let mut carried: WireLayout<CWire, CGroup> = vec![leaf("value")].into();
        carried.enclose_optional(Some(CGroup::Sequence(leaf("carrier"))));
        assert!(validate_layout(&carried)
            .unwrap_err()
            .contains("inline optional"));
        let mut sequence: WireLayout<CWire, CGroup> = vec![leaf("value")].into();
        sequence.enclose_sequence(None);
        assert!(validate_layout(&sequence)
            .unwrap_err()
            .contains("separate carrier"));
        let mut missing: WireLayout<CWire, CGroup> = vec![leaf("value")].into();
        missing.enclose_optional(None);
        assert!(validate_layout(&missing)
            .unwrap_err()
            .contains("requires optional metadata"));
        let mut wrong: WireLayout<CWire, CGroup> = vec![leaf("value")].into();
        wrong.enclose(metadata());
        assert!(validate_layout(&wrong)
            .unwrap_err()
            .contains("requires an Optional node"));
        let mut wrong_flag: WireLayout<CWire, CGroup> = vec![leaf("value")].into();
        let mut m = metadata();
        if let CGroup::Optional(o) = &mut m {
            o.controls.push(leaf("flag"));
        }
        wrong_flag.enclose_optional(Some(m));
        assert!(validate_layout(&wrong_flag)
            .unwrap_err()
            .contains("sole inline control"));
        let mut wrong_choice: WireLayout<CWire, CGroup> =
            WireLayout::from_nodes(vec![WireNode::Alternatives {
                metadata: Some(CGroup::Alternatives(leaf("tag"))),
                alternatives: vec![WireLayout::default()],
            }]);
        assert!(validate_layout(&wrong_choice)
            .unwrap_err()
            .contains("integer selector"));
        if let WireNode::Alternatives {
            metadata: Some(CGroup::Alternatives(w)),
            ..
        } = &mut wrong_choice.nodes_mut()[0]
        {
            w.kind = CWire::scalar(Primitive::I32);
        }
        validate_layout(&wrong_choice).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        tree[0]
            .kind
            .set_niche(Some(CNiche::Integer(0x110002)))
            .unwrap();
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        let mut outer = optional("outer", "value", true);
        let CGroup::Optional(o) = &mut outer else {
            unreachable!()
        };
        o.niche = Some(CNiche::Integer(0x110001));
        enclose_optional_metadata(&mut tree, outer);
        validate_layout(&tree).unwrap();
        assert_eq!(CCodec::wires(tree.nodes()).len(), 1);
        let mut bad = tree.clone();
        enclose_optional_metadata(&mut bad, optional("missing", "elsewhere", true));
        assert!(validate_layout(&bad)
            .unwrap_err()
            .contains("outside payload"));
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = WireLayout::from(vec![leaf("flag")]);
        CCodec::wires_mut(flag.nodes_mut()).next().unwrap().kind = CWire::scalar(Primitive::I32);
        enclose_optional_metadata(&mut flag, optional("flag", "flag", false));
        validate_layout(&flag).unwrap();
        enclose_optional_metadata(&mut flag, optional("outer", "flag", false));
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("inline control"));
    }
    #[test]
    fn every_niche_reservation_is_checked_for_compatibility_and_reuse() {
        for (kind, niche) in [
            (CWire::scalar(Primitive::Char), CNiche::Integer(0x110000)),
            (
                CWire::pointer(CPointer {
                    ty: syn::parse_quote!(Widget),
                    mutable: true,
                }),
                CNiche::Null,
            ),
        ] {
            let mut tree = WireLayout::from(vec![GenericWire {
                name: syn::parse_quote!(value),
                kind,
            }]);
            let group = |presence: &str, niche: CNiche| {
                CGroup::Optional(COptional {
                    controls: vec![],
                    presence: syn::parse_str(presence).unwrap(),
                    discriminator: syn::parse_quote!(value),
                    niche: Some(niche),
                })
            };
            enclose_optional_metadata(&mut tree, group("inner", niche.clone()));
            assert!(validate_layout(&tree)
                .unwrap_err()
                .contains("already reserved"));
            tree[0].kind.clear_niche();
            validate_layout(&tree).unwrap();
            enclose_optional_metadata(&mut tree, group("outer", niche));
            assert!(validate_layout(&tree)
                .unwrap_err()
                .contains("duplicate C niche"));
        }
        for (kind, niche) in [
            (CWire::Buffer, CNiche::Null),
            (CWire::scalar(Primitive::I64), CNiche::Integer(-1)),
            (CWire::scalar(Primitive::Char), CNiche::Integer(65)),
            (
                CWire::pointer(CPointer {
                    ty: syn::parse_quote!(Widget),
                    mutable: false,
                }),
                CNiche::Integer(0),
            ),
        ] {
            let mut tree = WireLayout::from(vec![GenericWire {
                name: syn::parse_quote!(value),
                kind,
            }]);
            enclose_optional_metadata(
                &mut tree,
                CGroup::Optional(COptional {
                    controls: vec![],
                    presence: syn::parse_quote!(present),
                    discriminator: syn::parse_quote!(value),
                    niche: Some(niche),
                }),
            );
            assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        }
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let mut field: WireLayout<CWire, CGroup> = WireLayout::default();
        field.enclose(CGroup::Field(syn::parse_quote!(x)));
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let mut duplicates = field.clone();
        duplicates.extend(field.clone());
        duplicates.enclose(CGroup::Fields);
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        field.enclose(CGroup::Fields);
        validate_layout(&field).unwrap();
        let mut malformed = WireLayout::from(vec![leaf("value")]);
        malformed.enclose(CGroup::Fields);
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let mut empty: WireLayout<CWire, CGroup> = WireLayout::default();
        empty.enclose(CGroup::Fields);
        validate_layout(&empty).unwrap();
    }
    #[test]
    fn malformed_metadata_is_rejected_before_final_slots_are_installed() {
        let mut input = CCodec
            .input_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        input
            .layout
            .enclose(CGroup::Field(syn::parse_quote!(orphan)));
        assert!(CCodec
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("direct child"));
        let mut output = CCodec
            .output_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                false,
            )
            .unwrap();
        output
            .layout
            .enclose(CGroup::Field(syn::parse_quote!(orphan)));
        assert!(CCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("direct child"));
    }
    #[test]
    fn public_variant_states_are_validated_before_emission() {
        for wire in [
            CWire::Scalar {
                primitive: Primitive::I64,
                next_integer_niche: Some(-1),
            },
            CWire::Scalar {
                primitive: Primitive::Char,
                next_integer_niche: Some(65),
            },
        ] {
            let mut plan = CCodec
                .input_leaf(
                    &syn::parse_quote!(i64),
                    &(),
                    syn::parse_quote!(wire),
                    syn::parse_quote!(value),
                    &prebindgen_declarations::Condition::default(),
                )
                .unwrap();
            CCodec::wires_mut(plan.layout.nodes_mut())
                .next()
                .unwrap()
                .kind = wire.clone();
            assert!(CCodec
                .finish_input(&mut plan, &())
                .unwrap_err()
                .contains("incompatible"));
            let mut output = CCodec
                .output_leaf(
                    &syn::parse_quote!(i64),
                    &(),
                    syn::parse_quote!(wire),
                    quote!(value),
                    false,
                )
                .unwrap();
            CCodec::wires_mut(output.layout.nodes_mut())
                .next()
                .unwrap()
                .kind = wire;
            assert!(CCodec
                .finish_output(&mut output, &())
                .unwrap_err()
                .contains("incompatible"));
        }
        let mut wire = CWire::enumeration(
            "code",
            prebindgen_declarations::EnumMapping {
                ty: syn::parse_quote!(Code),
                variants: vec![(syn::parse_quote!(A), 2)],
            },
        );
        let CWire::Enum { mapping, .. } = &mut wire else {
            panic!()
        };
        mapping.variants.push((syn::parse_quote!(D), 2));
        wire.clear_niche();
        assert!(wire.validate().is_err());
    }
}
