//! JNI adapter-owned descriptions of the wire tree.
use super::*;
use prebindgen_conversions::NativeReader;

use prebindgen_flat::{WireLayout, WireNode};
use syn::ext::IdentExt;

/// Metadata interpreted by the JNI binding writer.
#[derive(Clone, Debug)]
pub enum JniGroup {
    /// Metadata for `WireNode::Optional`, including its reserved absence value.
    Optional(JniOptional),
    /// Physical carrier chosen for this logical sequence; elements stay in the tree.
    Sequence(prebindgen_flat::Wire<JniWire>),
    /// Selector for this logical alternatives node; payloads stay in the tree.
    Alternatives(prebindgen_flat::Wire<JniWire>),
    /// An ordered collection of field subtrees.
    /// `Some` constructs a Kotlin data class; `None` exposes expanded values.
    Fields(Option<DataGroup>),
    /// A named source field, including empty and multi-wire fields.
    Field(syn::Ident),
}
/// Optional-value information retained by the JNI adapter.
#[derive(Clone, Debug)]
pub struct JniOptional {
    /// Explicit native presence controls; empty when a payload niche is used.
    pub controls: Vec<prebindgen_flat::Wire<JniWire>>,
    /// Normalized presence binding used by payload conditions.
    pub presence: syn::Ident,
    /// Payload wire or inline control inspected for presence.
    pub discriminator: syn::Ident,
    /// Reserved absent representation; `None` means a separate 0/1 flag.
    pub niche: Option<JniNiche>,
}

impl prebindgen_conversions::NativeReader<JniWire, JniGroup> for JniCodec {
    fn wires(
        nodes: &[WireNode<JniWire, JniGroup>],
    ) -> std::vec::IntoIter<&prebindgen_flat::Wire<JniWire>> {
        let mut out = vec![];
        read(nodes, &mut out, false);
        out.into_iter()
    }
    fn wires_mut(
        nodes: &mut [WireNode<JniWire, JniGroup>],
    ) -> std::vec::IntoIter<&mut prebindgen_flat::Wire<JniWire>> {
        fn visit<'a>(
            nodes: &'a mut [WireNode<JniWire, JniGroup>],
            out: &mut Vec<&'a mut prebindgen_flat::Wire<JniWire>>,
        ) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group { children, .. } => visit(children.nodes_mut(), out),
                    WireNode::Optional(c) => {
                        if let Some(JniGroup::Optional(o)) = &mut c.metadata {
                            out.extend(&mut o.controls);
                        }
                        visit(c.element.nodes_mut(), out);
                    }
                    WireNode::Sequence(c) => {
                        if let Some(JniGroup::Sequence(w)) = &mut c.metadata {
                            out.push(w);
                        }
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(JniGroup::Alternatives(w)) = metadata {
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
    fn validate(layout: &WireLayout<JniWire, JniGroup>) -> Result<()> {
        validate_layout(layout)
    }
}
// C/JNI parameters and row fields have different scopes. Validation separately
// reads all definitions to check the identifiers used by generated Rust code.
fn read<'a>(
    nodes: &'a [WireNode<JniWire, JniGroup>],
    out: &mut Vec<&'a prebindgen_flat::Wire<JniWire>>,
    elements: bool,
) {
    for node in nodes {
        match node {
            WireNode::Wire(w) => out.push(w),
            WireNode::Group { children, .. } => read(children.nodes(), out, elements),
            WireNode::Optional(c) => {
                if let Some(JniGroup::Optional(o)) = &c.metadata {
                    out.extend(&o.controls);
                }
                read(c.element.nodes(), out, elements);
            }
            WireNode::Sequence(c) => {
                if let Some(JniGroup::Sequence(w)) = &c.metadata {
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
                if let Some(JniGroup::Alternatives(w)) = metadata {
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
impl JniGroup {
    pub(crate) fn sequence_wire(&self) -> Option<&prebindgen_flat::Wire<JniWire>> {
        if let Self::Sequence(w) = self {
            Some(w)
        } else {
            None
        }
    }
}
pub(crate) fn validate_layout(layout: &WireLayout<JniWire, JniGroup>) -> Result<()> {
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
    for wire in JniCodec::wires(layout.nodes()) {
        wire.kind.validate()?;
    }
    fn visit(
        layout: &WireLayout<JniWire, JniGroup>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<JniNiche>>,
    ) -> Result<()> {
        for node in layout.nodes() {
            match node {
                WireNode::Alternatives {
                    metadata,
                    alternatives,
                } => {
                    let Some(JniGroup::Alternatives(selector)) = metadata else {
                        return Err("alternatives require selector metadata".into());
                    };
                    if !matches!(selector.kind, JniWire::Selector { .. }) {
                        return Err("alternatives require an integer selector".into());
                    }
                    for arm in alternatives {
                        visit(arm, bindings, false, reserved)?;
                    }
                }
                WireNode::Sequence(c) => {
                    let Some(JniGroup::Sequence(carrier)) = &c.metadata else {
                        return Err("sequence requires a separate carrier".into());
                    };
                    if !(matches!(carrier.kind, JniWire::Rows { .. }))
                        || JniCodec::wires(c.element.nodes()).len() == 0
                    {
                        return Err(
                            "sequence requires a supported carrier and nonempty element layout"
                                .into(),
                        );
                    }
                    validate_layout(&c.element)?;
                }
                WireNode::Optional(c) => {
                    let Some(JniGroup::Optional(o)) = &c.metadata else {
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
                        let wire = JniCodec::wires(c.element.nodes())
                            .find(|w| w.name == o.discriminator)
                            .ok_or("optional discriminator outside payload")?;
                        wire.kind.validate_niche(niche)?;
                        if wire.kind.available_niche().as_ref() == Some(niche) {
                            return Err(
                                "available JNI niche is already reserved by an optional node"
                                    .into(),
                            );
                        }
                        let values = reserved.entry(o.discriminator.to_string()).or_default();
                        if values.contains(niche) {
                            return Err("duplicate JNI niche reservation on one wire".into());
                        }
                        values.push(niche.clone());
                    } else if !matches!(controls.as_slice(),[flag] if flag.name==o.discriminator && o.presence==o.discriminator && matches!(flag.kind,JniWire::Selector { .. }))
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
                WireNode::Wire(w) if matches!(w.kind, JniWire::Rows { .. }) => {
                    return Err("array carrier requires a container node".into())
                }
                WireNode::Wire(_) => {}
                WireNode::Group { metadata, children } => {
                    if matches!(metadata, JniGroup::Optional(_)) {
                        return Err("optional metadata requires an Optional node".into());
                    }
                    if let JniGroup::Fields(Some(record)) = metadata {
                        if !bindings.insert(record.root.clone()) {
                            return Err(format!(
                                "duplicate Kotlin group binding `{}`",
                                record.root
                            ));
                        }
                    }
                    if matches!(metadata, JniGroup::Sequence(_) | JniGroup::Alternatives(_)) {
                        return Err("encoding metadata must be on its matching logical node".into());
                    }
                    match metadata {
                        JniGroup::Field(_) if !in_fields => {
                            return Err(
                                "named field must be a direct child of a field collection".into()
                            )
                        }
                        JniGroup::Fields(_) => {
                            let mut fields = std::collections::BTreeSet::new();
                            for child in children.nodes() {
                                let WireNode::Group {
                                    metadata: JniGroup::Field(name),
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
                        matches!(metadata, JniGroup::Fields(_)),
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
    fn enclose_optional_metadata(tree: &mut WireLayout<JniWire, JniGroup>, mut metadata: JniGroup) {
        let flag = matches!(&metadata,JniGroup::Optional(o) if o.niche.is_none());
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
        if let JniGroup::Optional(o) = &mut metadata {
            o.controls = controls;
        }
        tree.enclose_optional(Some(metadata));
    }
    fn leaf(name: &str) -> prebindgen_flat::Wire<JniWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: Option<i64>) -> JniGroup {
        JniGroup::Optional(JniOptional {
            controls: vec![],
            presence: syn::parse_str(presence).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche: niche.map(JniNiche::Integer),
        })
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        let metadata = || {
            JniGroup::Optional(JniOptional {
                controls: vec![],
                presence: syn::parse_quote!(flag),
                discriminator: syn::parse_quote!(flag),
                niche: None,
            })
        };
        let mut carried: WireLayout<JniWire, JniGroup> = vec![leaf("value")].into();
        carried.enclose_optional(Some(JniGroup::Sequence(leaf("carrier"))));
        assert!(validate_layout(&carried)
            .unwrap_err()
            .contains("inline optional"));
        let mut sequence: WireLayout<JniWire, JniGroup> = vec![leaf("value")].into();
        sequence.enclose_sequence(None);
        assert!(validate_layout(&sequence)
            .unwrap_err()
            .contains("separate carrier"));
        let mut missing: WireLayout<JniWire, JniGroup> = vec![leaf("value")].into();
        missing.enclose_optional(None);
        assert!(validate_layout(&missing)
            .unwrap_err()
            .contains("requires optional metadata"));
        let mut wrong: WireLayout<JniWire, JniGroup> = vec![leaf("value")].into();
        wrong.enclose(metadata());
        assert!(validate_layout(&wrong)
            .unwrap_err()
            .contains("requires an Optional node"));
        let mut wrong_flag: WireLayout<JniWire, JniGroup> = vec![leaf("value")].into();
        let mut m = metadata();
        if let JniGroup::Optional(o) = &mut m {
            o.controls.push(leaf("flag"));
        }
        wrong_flag.enclose_optional(Some(m));
        assert!(validate_layout(&wrong_flag)
            .unwrap_err()
            .contains("sole inline control"));
        let mut wrong_choice: WireLayout<JniWire, JniGroup> =
            WireLayout::from_nodes(vec![WireNode::Alternatives {
                metadata: Some(JniGroup::Alternatives(leaf("tag"))),
                alternatives: vec![WireLayout::default()],
            }]);
        assert!(validate_layout(&wrong_choice)
            .unwrap_err()
            .contains("integer selector"));
        if let WireNode::Alternatives {
            metadata: Some(JniGroup::Alternatives(w)),
            ..
        } = &mut wrong_choice.nodes_mut()[0]
        {
            w.kind = JniWire::selector();
        }
        validate_layout(&wrong_choice).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        tree[0].kind.set_niche(Some(JniNiche::Integer(-3))).unwrap();
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        enclose_optional_metadata(&mut tree, optional("outer", "value", Some(-2)));
        validate_layout(&tree).unwrap();
        assert_eq!(JniCodec::wires(tree.nodes()).len(), 1);
        let mut bad = tree.clone();
        enclose_optional_metadata(&mut bad, optional("missing", "elsewhere", Some(-3)));
        assert!(validate_layout(&bad)
            .unwrap_err()
            .contains("outside payload"));
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = WireLayout::from(vec![leaf("flag")]);
        JniCodec::wires_mut(flag.nodes_mut()).next().unwrap().kind = JniWire::selector();
        enclose_optional_metadata(&mut flag, optional("flag", "flag", None));
        validate_layout(&flag).unwrap();
        enclose_optional_metadata(&mut flag, optional("outer", "flag", None));
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("inline control"));
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let mut field: WireLayout<JniWire, JniGroup> = WireLayout::default();
        field.enclose(JniGroup::Field(syn::parse_quote!(x)));
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let mut duplicates = field.clone();
        duplicates.extend(field.clone());
        duplicates.enclose(JniGroup::Fields(None));
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        field.enclose(JniGroup::Fields(None));
        validate_layout(&field).unwrap();
        let mut malformed = WireLayout::from(vec![leaf("value")]);
        malformed.enclose(JniGroup::Fields(None));
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let mut empty: WireLayout<JniWire, JniGroup> = WireLayout::default();
        empty.enclose(JniGroup::Fields(None));
        validate_layout(&empty).unwrap();
    }
    #[test]
    fn malformed_metadata_is_rejected_before_final_slots_are_installed() {
        let mut input = JniCodec
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
            .enclose(JniGroup::Field(syn::parse_quote!(orphan)));
        assert!(JniCodec
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("direct child"));
        let mut output = JniCodec
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
            .enclose(JniGroup::Field(syn::parse_quote!(orphan)));
        assert!(JniCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("direct child"));
    }
    #[test]
    fn reserved_niches_are_checked_independently_of_remaining_availability() {
        let mut wire = JniWire::scalar(Primitive::Char);
        wire.clear_niche();
        let mut tree = WireLayout::from(vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(value),
            kind: wire,
        }]);
        enclose_optional_metadata(&mut tree, optional("present", "value", Some(-1)));
        validate_layout(&tree).unwrap();
        JniCodec::wires_mut(tree.nodes_mut()).next().unwrap().kind =
            JniWire::scalar(Primitive::I64);
        assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        JniCodec::wires_mut(tree.nodes_mut()).next().unwrap().kind = JniWire::Scalar {
            primitive: Primitive::Char,
            next_integer_niche: Some(0),
        };
        assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
    }
    #[test]
    fn incompatible_direct_wire_state_is_rejected_at_finalization() {
        let mut input = JniCodec
            .input_leaf(
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        JniCodec::wires_mut(input.layout.nodes_mut())
            .next()
            .unwrap()
            .kind = JniWire::Scalar {
            primitive: Primitive::I64,
            next_integer_niche: Some(-1),
        };
        assert!(JniCodec
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("incompatible"));
        let mut output = JniCodec
            .output_leaf(
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                false,
            )
            .unwrap();
        JniCodec::wires_mut(output.layout.nodes_mut())
            .next()
            .unwrap()
            .kind = JniCodec::wires(input.layout.nodes())
            .next()
            .unwrap()
            .kind
            .clone();
        assert!(JniCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("incompatible"));
    }
    #[test]
    fn optional_reservations_cannot_overlap_each_other_or_available_state() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        tree[0].kind.set_niche(Some(JniNiche::Integer(-2))).unwrap();
        validate_layout(&tree).unwrap();
        enclose_optional_metadata(&mut tree, optional("outer", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("duplicate JNI niche"));
    }
}
