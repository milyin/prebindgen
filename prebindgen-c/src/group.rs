//! C adapter-owned descriptions of the wire tree.
use super::*;

use prebindgen_flat::WireNode;
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

fn values(nodes: &[WireNode<CWire, CGroup>]) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
    let mut out = vec![];
    read(nodes, &mut out, false);
    out.into_iter()
}
fn values_mut(
    nodes: &mut [WireNode<CWire, CGroup>],
) -> std::vec::IntoIter<&mut prebindgen_flat::Wire<CWire>> {
    fn visit<'a>(
        nodes: &'a mut [WireNode<CWire, CGroup>],
        out: &mut Vec<&'a mut prebindgen_flat::Wire<CWire>>,
    ) {
        for node in nodes {
            match node {
                WireNode::Wire(w) => out.push(w),
                WireNode::Group { children, .. } => visit(children.as_mut_slice(), out),
                WireNode::Optional(c) => {
                    if let Some(CGroup::Optional(o)) = &mut c.metadata {
                        out.extend(&mut o.controls);
                    }
                    visit(c.element.as_mut_slice(), out);
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
                        visit(arm.as_mut_slice(), out);
                    }
                }
                WireNode::Outcomes { alternatives } => {
                    if let Some(o) = alternatives
                        .iter_mut()
                        .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                    {
                        visit(o.nodes.as_mut_slice(), out);
                    }
                }
            }
        }
    }
    let mut out = vec![];
    visit(nodes, &mut out);
    out.into_iter()
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
            WireNode::Group { children, .. } => read(children.as_slice(), out, elements),
            WireNode::Optional(c) => {
                if let Some(CGroup::Optional(o)) = &c.metadata {
                    out.extend(&o.controls);
                }
                read(c.element.as_slice(), out, elements);
            }
            WireNode::Sequence(c) => {
                if let Some(CGroup::Sequence(w)) = &c.metadata {
                    out.push(w);
                }
                if elements {
                    read(c.element.as_slice(), out, elements);
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
                    read(arm.as_slice(), out, elements);
                }
            }
            WireNode::Outcomes { alternatives } => {
                if let Some(o) = alternatives
                    .iter()
                    .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                {
                    read(o.nodes.as_slice(), out, elements);
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
pub(crate) fn validate_layout(layout: &[prebindgen_flat::WireNode<CWire, CGroup>]) -> Result<()> {
    prebindgen_flat::validate_wire_nodes(layout.iter())?;
    if let [WireNode::Outcomes { alternatives }] = layout {
        for arm in alternatives {
            validate_layout(&arm.nodes)?;
        }
        return Ok(());
    }
    let mut definitions = vec![];
    read(layout, &mut definitions, true);
    prebindgen_conversions::validate_wire_names(definitions)?;
    for wire in crate::group::values(layout) {
        wire.kind.validate()?;
    }
    fn visit(
        layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<CNiche>>,
    ) -> Result<()> {
        for node in layout {
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
                    )) || crate::group::values(c.element.as_slice()).len() == 0
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
                        let wire = crate::group::values(c.element.as_slice())
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
                        validate_layout(&arm.nodes)?;
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
                            for child in children.as_slice() {
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

// These operations name the ABI role of the values they select. Traversal is
// private implementation; the logical tree itself defines no physical order.
// Native parameters contributed by one source function argument.
pub(crate) fn argument_parameters(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> Result<std::vec::IntoIter<&prebindgen_flat::Wire<CWire>>> {
    if layout
        .iter()
        .any(|n| matches!(n, WireNode::Outcomes { .. }))
    {
        return Err("function parameters cannot contain completion outcomes".into());
    }
    Ok(values(layout))
}
pub(crate) fn function_parameters<
    I: prebindgen_flat::Conversion<Wire = CWire, Group = CGroup>,
    O: prebindgen_flat::Conversion,
>(
    f: &prebindgen_flat::Function<I, O>,
) -> Result<std::vec::IntoIter<&prebindgen_flat::Wire<CWire>>> {
    let mut out = vec![];
    for input in f.input_conversions()? {
        out.extend(argument_parameters(input.wire_nodes())?);
    }
    Ok(out.into_iter())
}
pub(crate) fn result_packet_fields(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
    values(
        prebindgen_flat::outcome_nodes(layout, prebindgen_flat::OutcomeKind::Success)
            .expect("validated Success outcome"),
    )
}
pub(crate) fn result_packet<'a>(
    layout: &'a [prebindgen_flat::WireNode<CWire, CGroup>],
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a prebindgen_flat::Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(layout), bindings)
}
pub(crate) fn row_fields(
    element: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
    values(element)
}
pub(crate) fn row_values<'a>(
    element: &'a [prebindgen_flat::WireNode<CWire, CGroup>],
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a prebindgen_flat::Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(element), bindings)
}
pub(crate) fn callback_parameters(
    element: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
    values(element)
}

pub(crate) fn optional_niche(
    payload: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> Option<prebindgen_declarations::NicheReservation<CNiche>> {
    prebindgen_declarations::select_optional_niche(&CCodec, values(payload))
}
pub(crate) fn optional_presence(
    payload: &[prebindgen_flat::WireNode<CWire, CGroup>],
    reservation: &prebindgen_declarations::NicheReservation<CNiche>,
) -> Result<TokenStream> {
    prebindgen_declarations::optional_presence(&CCodec, values(payload), reservation)
}
pub(crate) fn absent_output_values(
    payload: &[prebindgen_flat::WireNode<CWire, CGroup>],
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<Vec<(syn::Ident, TokenStream)>> {
    prebindgen_declarations::absent_output_values(&CCodec, values(payload), reservation)
}
pub(crate) fn retire_niches(
    payload: &mut Vec<prebindgen_flat::WireNode<CWire, CGroup>>,
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(
        &CCodec,
        values_mut(payload.as_mut_slice()),
        reservation,
    )
}

// Physical fields occupied by a named record field in C's header/error record.
pub(crate) fn record_fields(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
) -> std::vec::IntoIter<&prebindgen_flat::Wire<CWire>> {
    values(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn enclose_optional_metadata(
        tree: &mut Vec<prebindgen_flat::WireNode<CWire, CGroup>>,
        mut metadata: CGroup,
    ) {
        let flag = matches!(&metadata,CGroup::Optional(o) if o.niche.is_none());
        let mut nodes = std::mem::take(tree);
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
        *tree = nodes;
        if let CGroup::Optional(o) = &mut metadata {
            o.controls = controls;
        }
        *tree = vec![prebindgen_flat::WireNode::Optional(
            prebindgen_flat::WireContainer {
                metadata: Some(metadata),
                element: std::mem::take(&mut *tree),
            },
        )];
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
        let mut carried: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        carried = vec![prebindgen_flat::WireNode::Optional(
            prebindgen_flat::WireContainer {
                metadata: Some(CGroup::Sequence(leaf("carrier"))),
                element: std::mem::take(&mut carried),
            },
        )];
        assert!(validate_layout(&carried)
            .unwrap_err()
            .contains("inline optional"));
        let mut sequence: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        sequence = vec![prebindgen_flat::WireNode::Sequence(
            prebindgen_flat::WireContainer {
                metadata: None,
                element: std::mem::take(&mut sequence),
            },
        )];
        assert!(validate_layout(&sequence)
            .unwrap_err()
            .contains("separate carrier"));
        let mut missing: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        missing = vec![prebindgen_flat::WireNode::Optional(
            prebindgen_flat::WireContainer {
                metadata: None,
                element: std::mem::take(&mut missing),
            },
        )];
        assert!(validate_layout(&missing)
            .unwrap_err()
            .contains("requires optional metadata"));
        let mut wrong: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        wrong = vec![prebindgen_flat::WireNode::Group {
            metadata: metadata(),
            children: std::mem::take(&mut wrong),
        }];
        assert!(validate_layout(&wrong)
            .unwrap_err()
            .contains("requires an Optional node"));
        let mut wrong_flag: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        let mut m = metadata();
        if let CGroup::Optional(o) = &mut m {
            o.controls.push(leaf("flag"));
        }
        wrong_flag = vec![prebindgen_flat::WireNode::Optional(
            prebindgen_flat::WireContainer {
                metadata: Some(m),
                element: std::mem::take(&mut wrong_flag),
            },
        )];
        assert!(validate_layout(&wrong_flag)
            .unwrap_err()
            .contains("sole inline control"));
        let mut wrong_choice: Vec<prebindgen_flat::WireNode<CWire, CGroup>> =
            vec![WireNode::Alternatives {
                metadata: Some(CGroup::Alternatives(leaf("tag"))),
                alternatives: vec![Vec::new()],
            }];
        assert!(validate_layout(&wrong_choice)
            .unwrap_err()
            .contains("integer selector"));
        if let WireNode::Alternatives {
            metadata: Some(CGroup::Alternatives(w)),
            ..
        } = &mut wrong_choice.as_mut_slice()[0]
        {
            w.kind = CWire::scalar(Primitive::I32);
        }
        validate_layout(&wrong_choice).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        prebindgen_flat::wire_definitions_mut(tree.iter_mut())
            .next()
            .expect("logical wire index out of range")
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
        assert_eq!(crate::group::values(tree.as_slice()).len(), 1);
        let mut bad = tree.clone();
        enclose_optional_metadata(&mut bad, optional("missing", "elsewhere", true));
        assert!(validate_layout(&bad)
            .unwrap_err()
            .contains("outside payload"));
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = vec![prebindgen_flat::WireNode::Wire(leaf("flag"))];
        crate::group::values_mut(flag.as_mut_slice())
            .next()
            .unwrap()
            .kind = CWire::scalar(Primitive::I32);
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
            let mut tree = vec![prebindgen_flat::WireNode::Wire(GenericWire {
                name: syn::parse_quote!(value),
                kind,
            })];
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
            prebindgen_flat::wire_definitions_mut(tree.iter_mut())
                .next()
                .expect("logical wire index out of range")
                .kind
                .clear_niche();
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
            let mut tree = vec![prebindgen_flat::WireNode::Wire(GenericWire {
                name: syn::parse_quote!(value),
                kind,
            })];
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
        let mut field: Vec<prebindgen_flat::WireNode<CWire, CGroup>> = Vec::new();
        field = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(x)),
            children: std::mem::take(&mut field),
        }];
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let mut duplicates = field.clone();
        duplicates.extend(field.clone());
        duplicates = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Fields,
            children: std::mem::take(&mut duplicates),
        }];
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        field = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Fields,
            children: std::mem::take(&mut field),
        }];
        validate_layout(&field).unwrap();
        let mut malformed = vec![prebindgen_flat::WireNode::Wire(leaf("value"))];
        malformed = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Fields,
            children: std::mem::take(&mut malformed),
        }];
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let mut empty: Vec<prebindgen_flat::WireNode<CWire, CGroup>> = Vec::new();
        empty = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Fields,
            children: std::mem::take(&mut empty),
        }];
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
        input.nodes = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(orphan)),
            children: std::mem::take(&mut input.nodes),
        }];
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
        output.nodes = vec![prebindgen_flat::WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(orphan)),
            children: std::mem::take(&mut output.nodes),
        }];
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
            crate::group::values_mut(plan.nodes.as_mut_slice())
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
            crate::group::values_mut(output.nodes.as_mut_slice())
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

#[cfg(test)]
mod abi_tests {
    use super::*;
    #[test]
    fn function_parameters_result_packets_and_sequence_fields_have_explicit_roles() {
        let scalar = |name: &str| prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: CWire::scalar(Primitive::I32),
        };
        let element: Vec<prebindgen_flat::WireNode<CWire, CGroup>> = vec![
            prebindgen_flat::WireNode::Wire(scalar("first")),
            prebindgen_flat::WireNode::Wire(scalar("second")),
        ];
        let mut sequence = element.clone();
        sequence = vec![prebindgen_flat::WireNode::Sequence(
            prebindgen_flat::WireContainer {
                metadata: Some(CGroup::Sequence(prebindgen_flat::Wire {
                    name: syn::parse_quote!(rows),
                    kind: CWire::Rows,
                })),
                element: std::mem::take(&mut sequence),
            },
        )];
        let mut result = sequence.clone();
        prebindgen_flat::set_outcome(
            &mut result,
            prebindgen_flat::OutcomeKind::SourceError,
            vec![prebindgen_flat::WireNode::Wire(scalar("error"))],
        );
        assert_eq!(
            result_packet_fields(&result)
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows"]
        );
        assert_eq!(
            row_fields(&element)
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert_eq!(
            callback_parameters(&element)
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        let bindings = vec![
            (syn::parse_quote!(second), quote!(2)),
            (syn::parse_quote!(first), quote!(1)),
        ];
        let ordered = row_values(&element, &bindings).unwrap();
        assert_eq!(ordered[0].1.to_string(), "1");
        assert_eq!(ordered[1].1.to_string(), "2");
        let packet = vec![(syn::parse_quote!(rows), quote!(buffer))];
        assert_eq!(result_packet(&result, &packet).unwrap().len(), 1);
        assert!(result_packet(&result, &bindings).is_err());
        let source = syn::parse_file("pub fn f(value:i32){}").unwrap();
        let mut flat: Bindings =
            Flat::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let mut input = CCodec
            .input_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(raw),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        input.nodes = sequence;
        flat.function_mut("f").unwrap().params[0].ty.slot = Some(input.clone());
        assert_eq!(
            function_parameters(flat.function("f").unwrap())
                .unwrap()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows"]
        );
        input.nodes = result;
        flat.function_mut("f").unwrap().params[0].ty.slot = Some(input);
        assert!(function_parameters(flat.function("f").unwrap()).is_err());
    }
}
