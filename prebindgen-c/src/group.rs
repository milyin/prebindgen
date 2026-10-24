//! C adapter metadata and interpretation of logical value trees.
use super::*;
use prebindgen_flat::{Wire, WireNode};
use syn::ext::IdentExt;

/// Relationships between the child values of a group.
#[derive(Clone, Debug)]
pub enum CGroup {
    /// Ordered conversion components, such as constructor arguments or output views.
    Values,
    /// Source record fields.
    Fields,
    /// One named source field.
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

fn values(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    let mut out = vec![];
    read(node, &mut out, false);
    out.into_iter()
}
fn values_mut(
    node: &mut WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&mut Wire<CWire>> {
    fn visit<'a>(
        node: &'a mut WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
        out: &mut Vec<&'a mut Wire<CWire>>,
    ) {
        match node {
            WireNode::Wire(wires) => out.extend(wires),
            WireNode::Group { elements, .. } => {
                for child in elements {
                    visit(child, out);
                }
            }
            WireNode::Optional { element, metadata } => {
                out.extend(&mut metadata.controls);
                visit(element, out);
            }
            WireNode::Sequence { metadata, .. } => out.push(metadata),
            WireNode::Alternatives { elements, metadata } => {
                out.push(metadata);
                for arm in elements {
                    visit(arm, out);
                }
            }
        }
    }
    let mut out = vec![];
    visit(node, &mut out);
    out.into_iter()
}
fn read<'a>(
    node: &'a WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    out: &mut Vec<&'a Wire<CWire>>,
    elements: bool,
) {
    match node {
        WireNode::Wire(wires) => out.extend(wires),
        WireNode::Group {
            elements: children, ..
        } => {
            for child in children {
                read(child, out, elements);
            }
        }
        WireNode::Optional { element, metadata } => {
            out.extend(&metadata.controls);
            read(element, out, elements);
        }
        WireNode::Sequence { element, metadata } => {
            out.push(metadata);
            if elements {
                read(element, out, elements);
            }
        }
        WireNode::Alternatives {
            elements: children,
            metadata,
        } => {
            out.push(metadata);
            for child in children {
                read(child, out, elements);
            }
        }
    }
}
pub(crate) fn validate_layout(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> Result<()> {
    prebindgen_flat::validate_wire_nodes(std::iter::once(node))?;
    let mut definitions = vec![];
    read(node, &mut definitions, true);
    prebindgen_conversions::validate_wire_names(definitions)?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit(
        node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<CNiche>>,
    ) -> Result<()> {
        match node {
            WireNode::Wire(wires) => {
                for w in wires {
                    if matches!(w.kind, CWire::Rows) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Alternatives {
                elements: alternatives,
                metadata: selector,
            } => {
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
            WireNode::Sequence {
                element,
                metadata: carrier,
            } => {
                if !(matches!(
                    carrier.kind,
                    CWire::Rows
                        | CWire::Scalar {
                            primitive: Primitive::Usize,
                            ..
                        }
                )) || values(element).len() == 0
                {
                    return Err(
                        "sequence requires a supported carrier and nonempty element layout".into(),
                    );
                }
                validate_layout(element)?;
            }
            WireNode::Optional {
                element,
                metadata: o,
            } => {
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
                    let wire = crate::group::values(element)
                        .find(|w| w.name == o.discriminator)
                        .ok_or("optional discriminator outside payload")?;
                    wire.kind.validate_niche(niche)?;
                    if wire.kind.available_niche().as_ref() == Some(niche) {
                        return Err(
                            "available C niche is already reserved by an optional node".into()
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
                visit(element, bindings, false, reserved)?;
            }
            WireNode::Group {
                elements: children,
                metadata,
            } => {
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
                                return Err("field collection requires named field children".into());
                            };
                            if !fields.insert(name.unraw().to_string()) {
                                return Err(format!("duplicate field `{name}`"));
                            }
                        }
                    }
                    _ => {}
                }

                for child in children {
                    visit(
                        child,
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
        node,
        &mut Default::default(),
        false,
        &mut Default::default(),
    )
}

/// Native parameters contributed by a source argument.
pub(crate) fn argument_parameters(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    I: prebindgen_flat::InputConversion<
        Wire = CWire,
        Optional = COptional,
        Sequence = Wire<CWire>,
        Group = CGroup,
        Alternatives = Wire<CWire>,
    >,
    O: prebindgen_flat::Conversion,
>(
    f: &prebindgen_flat::Function<I, O>,
) -> Result<std::vec::IntoIter<&Wire<CWire>>> {
    let mut out = vec![];
    for input in f.input_conversions()? {
        out.extend(argument_parameters(input.input_node()));
    }
    Ok(out.into_iter())
}
pub(crate) fn result_packet_fields(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn result_packet<'a>(
    node: &'a WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn row_values<'a>(
    node: &'a WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn optional_niche(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> Option<prebindgen_declarations::NicheReservation<CNiche>> {
    prebindgen_declarations::select_optional_niche(&CCodec, values(node))
}
pub(crate) fn optional_presence(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    reservation: &prebindgen_declarations::NicheReservation<CNiche>,
) -> Result<TokenStream> {
    prebindgen_declarations::optional_presence(&CCodec, values(node), reservation)
}
pub(crate) fn absent_output_values(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<Vec<(syn::Ident, TokenStream)>> {
    prebindgen_declarations::absent_output_values(&CCodec, values(node), reservation)
}
pub(crate) fn retire_niches(
    node: &mut WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(&CCodec, values_mut(node), reservation)
}
pub(crate) fn record_fields(
    node: &WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn enclose_optional_metadata(
        tree: &mut WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>>,
        mut metadata: COptional,
    ) {
        let mut payload = std::mem::take(tree);
        if metadata.niche.is_none() {
            if let WireNode::Wire(wires) = &mut payload {
                if !wires.is_empty() {
                    metadata.controls = vec![wires.remove(0)];
                }
            }
        }
        *tree = WireNode::Optional {
            element: Box::new(payload),
            metadata,
        };
    }
    fn leaf(name: &str) -> prebindgen_flat::Wire<CWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: CWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: bool) -> COptional {
        COptional {
            controls: vec![],
            presence: syn::parse_str(presence).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche: niche.then_some(CNiche::Integer(0x110000)),
        }
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        let mut sequence: WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>> =
            WireNode::Sequence {
                element: Box::new(WireNode::Wire(vec![leaf("value")])),
                metadata: leaf("carrier"),
            };
        assert!(validate_layout(&sequence)
            .unwrap_err()
            .contains("supported carrier"));
        let WireNode::Sequence { metadata, .. } = &mut sequence else {
            panic!()
        };
        metadata.kind = CWire::Rows;
        validate_layout(&sequence).unwrap();
        let wrong_flag = WireNode::Optional {
            element: Box::new(WireNode::Wire(vec![leaf("value")])),
            metadata: COptional {
                controls: vec![leaf("flag")],
                presence: syn::parse_quote!(flag),
                discriminator: syn::parse_quote!(flag),
                niche: None,
            },
        };
        assert!(validate_layout(&wrong_flag)
            .unwrap_err()
            .contains("sole inline control"));
        let mut choice: WireNode<CWire, COptional, Wire<CWire>, CGroup, Wire<CWire>> =
            WireNode::Alternatives {
                elements: vec![WireNode::default()],
                metadata: leaf("tag"),
            };
        assert!(validate_layout(&choice)
            .unwrap_err()
            .contains("integer selector"));
        let WireNode::Alternatives { metadata, .. } = &mut choice else {
            panic!()
        };
        metadata.kind = CWire::scalar(Primitive::I32);
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = prebindgen_flat::WireNode::Wire(vec![leaf("value")]);
        prebindgen_flat::wire_definitions_mut(std::iter::once(&mut tree))
            .next()
            .expect("logical wire index out of range")
            .kind
            .set_niche(Some(CNiche::Integer(0x110002)))
            .unwrap();
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        let mut outer = optional("outer", "value", true);
        let o = &mut outer;
        o.niche = Some(CNiche::Integer(0x110001));
        enclose_optional_metadata(&mut tree, outer);
        validate_layout(&tree).unwrap();
        assert_eq!(crate::group::values(&tree).len(), 1);
        let mut bad = tree.clone();
        enclose_optional_metadata(&mut bad, optional("missing", "elsewhere", true));
        assert!(validate_layout(&bad)
            .unwrap_err()
            .contains("outside payload"));
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = prebindgen_flat::WireNode::Wire(vec![leaf("flag")]);
        crate::group::values_mut(&mut flag).next().unwrap().kind = CWire::scalar(Primitive::I32);
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
            let mut tree = prebindgen_flat::WireNode::Wire(vec![GenericWire {
                name: syn::parse_quote!(value),
                kind,
            }]);
            let group = |presence: &str, niche: CNiche| COptional {
                controls: vec![],
                presence: syn::parse_str(presence).unwrap(),
                discriminator: syn::parse_quote!(value),
                niche: Some(niche),
            };
            enclose_optional_metadata(&mut tree, group("inner", niche.clone()));
            assert!(validate_layout(&tree)
                .unwrap_err()
                .contains("already reserved"));
            prebindgen_flat::wire_definitions_mut(std::iter::once(&mut tree))
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
            let mut tree = prebindgen_flat::WireNode::Wire(vec![GenericWire {
                name: syn::parse_quote!(value),
                kind,
            }]);
            enclose_optional_metadata(
                &mut tree,
                COptional {
                    controls: vec![],
                    presence: syn::parse_quote!(present),
                    discriminator: syn::parse_quote!(value),
                    niche: Some(niche),
                },
            );
            assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        }
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let field = WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(x)),
            elements: vec![WireNode::default()],
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates = WireNode::Group {
            metadata: CGroup::Fields,
            elements: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record = WireNode::Group {
            metadata: CGroup::Fields,
            elements: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed = WireNode::Group {
            metadata: CGroup::Fields,
            elements: vec![WireNode::Wire(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty = WireNode::Group {
            metadata: CGroup::Fields,
            elements: vec![],
        };
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
        input.node = prebindgen_flat::WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(orphan)),
            elements: vec![std::mem::take(&mut input.node)],
        };
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
        output.outcomes.success = prebindgen_flat::WireNode::Group {
            metadata: CGroup::Field(syn::parse_quote!(orphan)),
            elements: vec![std::mem::take(&mut output.outcomes.success)],
        };
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
            crate::group::values_mut(&mut plan.node)
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
            crate::group::values_mut(&mut output.outcomes.success)
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
        let element: prebindgen_flat::WireNode<
            CWire,
            crate::COptional,
            prebindgen_flat::Wire<CWire>,
            CGroup,
            prebindgen_flat::Wire<CWire>,
        > = prebindgen_flat::WireNode::Wire(vec![scalar("first"), scalar("second")]);
        let mut sequence = element.clone();
        sequence = prebindgen_flat::WireNode::Sequence {
            metadata: prebindgen_flat::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            },
            element: Box::new(std::mem::take(&mut sequence)),
        };
        let result = prebindgen_flat::Outcomes {
            success: sequence.clone(),
            source_error: Some(WireNode::Wire(vec![scalar("error")])),
            binding_failure: None,
        };
        assert_eq!(
            result_packet_fields(&result.success)
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
        assert_eq!(result_packet(&result.success, &packet).unwrap().len(), 1);
        assert!(result_packet(&result.success, &bindings).is_err());
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
        input.node = sequence;
        flat.function_mut("f").unwrap().params[0].ty.slot = Some(input.clone());
        assert_eq!(
            function_parameters(flat.function("f").unwrap())
                .unwrap()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows"]
        );
    }
}
