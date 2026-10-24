//! JNI adapter metadata and interpretation of logical value trees.
use super::*;
use prebindgen_flat::{Wire, WireNode};
use syn::ext::IdentExt;

/// Meaning and encoding of a parent with one child description.
#[derive(Clone, Debug)]
pub enum JniChain {
    /// Zero or one payload occurrence.
    Optional(JniOptional),
    /// Zero or more element occurrences.
    Sequence(JniSequence),
}
/// Meaning of an ordered collection of child descriptions.
#[derive(Clone, Debug)]
pub enum JniBranch {
    /// Related values or source fields, all belonging to the parent.
    Group(JniGroup),
    /// Exactly one selected arm; at least one arm is required.
    Alternatives,
}
/// Relationships between the child values of a group.
#[derive(Clone, Debug)]
pub enum JniGroup {
    /// Ordered conversion components, such as constructor arguments or output views.
    Values,
    /// Source record fields. JNI may associate the record with a Kotlin data class.
    Fields(Option<DataGroup>),
    /// One named source field.
    Field(syn::Ident),
}
pub(crate) fn validate_selector(wire: &Wire<JniWire>) -> Result<()> {
    if !matches!(wire.kind, JniWire::Selector { .. }) {
        return Err("selector requires the adapter's integer tag representation".into());
    }
    wire.kind.validate()
}
/// How this optional occurrence distinguishes absence from a present payload.
#[derive(Clone, Debug)]
pub enum JniOptional {
    /// A dedicated 0/1 tag. The binding is also the normalized presence value.
    Flag(syn::Ident),
    /// An excluded representation in one payload value.
    Niche {
        /// Normalized presence local used by conversion conditions.
        presence: syn::Ident,
        /// Payload binding and reserved absent representation.
        reservation: prebindgen_declarations::NicheReservation<JniNiche>,
    },
}
impl JniOptional {
    /// Binding of the normalized presence condition.
    pub fn presence(&self) -> &syn::Ident {
        match self {
            Self::Flag(flag) => flag,
            Self::Niche { presence, .. } => presence,
        }
    }
    /// Binding whose value distinguishes absent and present states.
    pub fn discriminator(&self) -> &syn::Ident {
        match self {
            Self::Flag(flag) => flag,
            Self::Niche { reservation, .. } => &reservation.discriminator,
        }
    }
    /// Reserved payload representation; a flag encoding has no reservation.
    pub fn niche(&self) -> Option<&JniNiche> {
        match self {
            Self::Flag(_) => None,
            Self::Niche { reservation, .. } => Some(&reservation.niche),
        }
    }
    pub(crate) fn from_parts(
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
        controls: &[Wire<JniWire>],
    ) -> Result<Self> {
        match niche {
            Some(niche) if controls.is_empty() => Ok(Self::Niche {
                presence,
                reservation: prebindgen_declarations::NicheReservation {
                    discriminator,
                    niche,
                },
            }),
            None if controls.len() == 1
                && presence == discriminator
                && controls[0].name == discriminator =>
            {
                validate_selector(&controls[0])?;
                Ok(Self::Flag(presence))
            }
            _ => {
                Err("optional encoding requires either one matching flag or a payload niche".into())
            }
        }
    }
}

// Definitions in one native scope; sequence element definitions belong to row scopes.
fn values(node: &WireNode<JniWire, JniChain, JniBranch>) -> std::vec::IntoIter<&Wire<JniWire>> {
    fn visit<'a>(
        node: &'a WireNode<JniWire, JniChain, JniBranch>,
        out: &mut Vec<&'a Wire<JniWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta: JniChain::Sequence(_),
                controls,
                ..
            } => out.extend(controls),
            WireNode::Chain {
                child, controls, ..
            } => {
                out.extend(controls);
                visit(child, out);
            }
            WireNode::Branch {
                children, controls, ..
            } => {
                out.extend(controls);
                for child in children {
                    visit(child, out);
                }
            }
        }
    }
    let mut out = vec![];
    visit(node, &mut out);
    out.into_iter()
}
fn values_mut(
    node: &mut WireNode<JniWire, JniChain, JniBranch>,
) -> std::vec::IntoIter<&mut Wire<JniWire>> {
    fn visit<'a>(
        node: &'a mut WireNode<JniWire, JniChain, JniBranch>,
        out: &mut Vec<&'a mut Wire<JniWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta: JniChain::Sequence(_),
                controls,
                ..
            } => out.extend(controls),
            WireNode::Chain {
                child, controls, ..
            } => {
                out.extend(controls);
                visit(child, out);
            }
            WireNode::Branch {
                children, controls, ..
            } => {
                out.extend(controls);
                for child in children {
                    visit(child, out);
                }
            }
        }
    }
    let mut out = vec![];
    visit(node, &mut out);
    out.into_iter()
}
pub(crate) fn validate_layout(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> Result<()> {
    prebindgen_conversions::validate_wire_names(prebindgen_flat::wire_definitions(
        std::iter::once(node),
    ))?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit(
        node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<JniNiche>>,
    ) -> Result<()> {
        match node {
            WireNode::Leaf(wires) => {
                for w in wires {
                    if matches!(w.kind, JniWire::Rows { .. }) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Branch {
                meta: JniBranch::Alternatives,
                children: alternatives,
                controls,
                ..
            } => {
                if alternatives.is_empty() {
                    return Err("selection has no alternatives".into());
                }
                if let [selector] = controls.as_slice() {
                    validate_selector(selector)?;
                } else {
                    return Err("alternatives require one selector control".into());
                }
                for arm in alternatives {
                    visit(arm, bindings, false, reserved)?;
                }
            }
            WireNode::Chain {
                child: element,
                controls,
                meta: JniChain::Sequence(metadata),
                ..
            } => {
                let [carrier] = controls.as_slice() else {
                    return Err("sequence requires one carrier control".into());
                };
                let valid = match metadata {
                    JniSequence::Buffered => matches!(carrier.kind, JniWire::Rows { .. }),
                    JniSequence::Callback => {
                        matches!(&carrier.kind, JniWire::Rows { condition,.. } if condition=="true")
                    }
                };
                if !valid {
                    return Err("sequence carrier control is incompatible with its delivery".into());
                }
                if values(element).len() == 0 {
                    return Err("sequence requires a nonempty element layout".into());
                }
                crate::sequence::validate_buffered(element)?;
                validate_layout(element)?;
            }
            WireNode::Chain {
                child: element,
                meta: JniChain::Optional(o),
                controls,
                ..
            } => {
                match o {
                    JniOptional::Flag(presence) => {
                        let [flag] = controls.as_slice() else {
                            return Err("flag optional requires one control".into());
                        };
                        validate_selector(flag)?;
                        if &flag.name != presence {
                            return Err("optional flag reference does not match its control".into());
                        }
                    }
                    JniOptional::Niche { .. } if !controls.is_empty() => {
                        return Err("niche optional must not have controls".into())
                    }
                    _ => {}
                }
                if !bindings.insert(o.presence().to_string()) {
                    return Err(format!(
                        "duplicate optional presence binding `{}`",
                        o.presence()
                    ));
                }
                if let Some(niche) = o.niche() {
                    let wire = crate::group::values(element)
                        .find(|w| w.name == *o.discriminator())
                        .ok_or("optional discriminator outside payload")?;
                    wire.kind.validate_niche(niche)?;
                    if wire.kind.available_niche().as_ref() == Some(niche) {
                        return Err(
                            "available JNI niche is already reserved by an optional node".into(),
                        );
                    }
                    let values = reserved.entry(o.discriminator().to_string()).or_default();
                    if values.contains(niche) {
                        return Err("duplicate JNI niche reservation on one wire".into());
                    }
                    values.push(niche.clone());
                }
                visit(element, bindings, false, reserved)?;
            }
            WireNode::Branch {
                children,
                meta: JniBranch::Group(metadata),
                controls,
                ..
            } => {
                if !controls.is_empty() {
                    return Err("group metadata does not support control wires".into());
                }
                if let JniGroup::Fields(Some(record)) = metadata {
                    if !bindings.insert(record.root.clone()) {
                        return Err(format!("duplicate Kotlin group binding `{}`", record.root));
                    }
                }
                match metadata {
                    JniGroup::Field(_) if !in_fields => {
                        return Err(
                            "named field must be a direct child of a field collection".into()
                        )
                    }
                    JniGroup::Fields(_) => {
                        let mut fields = std::collections::BTreeSet::new();
                        for child in children.as_slice() {
                            let WireNode::Branch {
                                meta: JniBranch::Group(JniGroup::Field(name)),
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
                        matches!(metadata, JniGroup::Fields(_)),
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
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    I: prebindgen_flat::InputConversion<Wire = JniWire, Chain = JniChain, Branch = JniBranch>,
    O: prebindgen_flat::Conversion,
>(
    f: &prebindgen_flat::Function<I, O>,
) -> Result<std::vec::IntoIter<&Wire<JniWire>>> {
    let mut out = vec![];
    for input in f.input_conversions()? {
        out.extend(argument_parameters(input.input_node()));
    }
    Ok(out.into_iter())
}
pub(crate) fn result_packet_fields(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn result_packet<'a>(
    node: &'a WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn row_values<'a>(
    node: &'a WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn optional_niche(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> Option<prebindgen_declarations::NicheReservation<JniNiche>> {
    prebindgen_declarations::select_optional_niche(&JniCodec, values(node))
}
pub(crate) fn optional_presence(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    reservation: &prebindgen_declarations::NicheReservation<JniNiche>,
) -> Result<TokenStream> {
    prebindgen_declarations::optional_presence(&JniCodec, values(node), reservation)
}
pub(crate) fn absent_output_values(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<Vec<(syn::Ident, TokenStream)>> {
    prebindgen_declarations::absent_output_values(&JniCodec, values(node), reservation)
}
pub(crate) fn retire_niches(
    node: &mut WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(&JniCodec, values_mut(node), reservation)
}
pub(crate) fn kotlin_result_fields(
    node: &WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selector(name: syn::Ident) -> prebindgen_flat::Wire<JniWire> {
        prebindgen_flat::Wire {
            name,
            kind: JniWire::selector(),
        }
    }
    fn enclose_optional_metadata(
        tree: &mut WireNode<JniWire, JniChain, JniBranch>,
        metadata: JniOptional,
    ) {
        let mut payload = std::mem::take(tree);
        let controls = if let JniOptional::Flag(name) = &metadata {
            if let WireNode::Leaf(wires) = &mut payload {
                if !wires.is_empty() {
                    vec![wires.remove(0)]
                } else {
                    vec![selector(name.clone())]
                }
            } else {
                vec![selector(name.clone())]
            }
        } else {
            vec![]
        };
        *tree = WireNode::Chain {
            controls,
            child: Box::new(payload),
            meta: JniChain::Optional(metadata),
        };
    }
    fn leaf(name: &str) -> prebindgen_flat::Wire<JniWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: Option<i64>) -> JniOptional {
        match niche.map(JniNiche::Integer) {
            Some(niche) => JniOptional::Niche {
                presence: syn::parse_str(presence).unwrap(),
                reservation: prebindgen_declarations::NicheReservation {
                    discriminator: syn::parse_str(discriminator).unwrap(),
                    niche,
                },
            },
            None => JniOptional::Flag(syn::parse_str(discriminator).unwrap()),
        }
    }
    #[test]
    fn optional_construction_rejects_inconsistent_flag_and_niche_parts() {
        let flag = selector(syn::parse_quote!(flag));
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[]
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag.clone(), flag.clone()]
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(other),
            syn::parse_quote!(flag),
            None,
            std::slice::from_ref(&flag)
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(present),
            syn::parse_quote!(payload),
            Some(JniNiche::Integer(-1)),
            std::slice::from_ref(&flag)
        )
        .is_err());
        let encoding = JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag],
        )
        .unwrap();
        assert!(matches!(encoding, JniOptional::Flag(_)));
        assert_eq!(encoding.presence(), encoding.discriminator());
    }
    #[test]
    fn selector_control_preserves_consumed_and_remaining_niches() {
        for remaining in [None, Some(JniNiche::Integer(-3))] {
            let mut carrier = selector(syn::parse_quote!(tag));
            carrier.kind.set_niche(remaining.clone()).unwrap();
            let node: WireNode<JniWire, JniChain, JniBranch> = WireNode::Branch {
                controls: vec![carrier],
                children: vec![WireNode::default()],
                meta: JniBranch::Alternatives,
            };
            validate_layout(&node).unwrap();
            assert_eq!(
                prebindgen_flat::wire_definitions(std::iter::once(&node))
                    .next()
                    .unwrap()
                    .kind
                    .available_niche(),
                remaining
            );
        }
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        assert!(validate_selector(&leaf("tag"))
            .unwrap_err()
            .contains("integer tag"));
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[leaf("flag")]
        )
        .unwrap_err()
        .contains("integer tag"));
        let sequence: WireNode<JniWire, JniChain, JniBranch> = WireNode::Chain {
            controls: vec![prebindgen_flat::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniChain::Sequence(JniSequence::Buffered),
        };
        validate_layout(&sequence).unwrap();
        let empty = WireNode::Chain {
            controls: vec![prebindgen_flat::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::default()),
            meta: JniChain::Sequence(JniSequence::Buffered),
        };
        assert!(validate_layout(&empty)
            .unwrap_err()
            .contains("nonempty element"));
        let choice = WireNode::Branch {
            controls: vec![selector(syn::parse_quote!(tag))],
            children: vec![WireNode::default()],
            meta: JniBranch::Alternatives,
        };
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn malformed_tree_controls_are_rejected() {
        let mut tree: WireNode<JniWire, JniChain, JniBranch> = WireNode::Branch {
            controls: vec![],
            children: vec![WireNode::default()],
            meta: JniBranch::Alternatives,
        };
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("one selector control"));
        let WireNode::Branch {
            meta: JniBranch::Alternatives,
            controls,
            ..
        } = &mut tree
        else {
            panic!()
        };
        controls.push(leaf("wrong_kind"));
        assert!(validate_layout(&tree).unwrap_err().contains("integer tag"));
        let empty_sequence = WireNode::Chain {
            controls: vec![],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniChain::Sequence(JniSequence::Buffered),
        };
        assert!(validate_layout(&empty_sequence)
            .unwrap_err()
            .contains("one carrier control"));
        let flag = WireNode::Chain {
            controls: vec![selector(syn::parse_quote!(actual))],
            child: Box::new(WireNode::default()),
            meta: JniChain::Optional(JniOptional::Flag(syn::parse_quote!(missing))),
        };
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("does not match"));
    }
    #[test]
    fn callback_control_condition_is_unconditional() {
        let mut node: WireNode<JniWire, JniChain, JniBranch> = WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: "false".into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniChain::Sequence(JniSequence::Callback),
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("incompatible with its delivery"));
        let WireNode::Chain {
            meta: JniChain::Sequence(metadata),
            ..
        } = &mut node
        else {
            panic!()
        };
        *metadata = JniSequence::Buffered;
        validate_layout(&node).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = prebindgen_flat::WireNode::Leaf(vec![leaf("value")]);
        prebindgen_flat::wire_definitions_mut(std::iter::once(&mut tree))
            .next()
            .expect("logical wire index out of range")
            .kind
            .set_niche(Some(JniNiche::Integer(-3)))
            .unwrap();
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        enclose_optional_metadata(&mut tree, optional("outer", "value", Some(-2)));
        validate_layout(&tree).unwrap();
        assert_eq!(crate::group::values(&tree).len(), 1);
        let mut bad = tree.clone();
        enclose_optional_metadata(&mut bad, optional("missing", "elsewhere", Some(-3)));
        assert!(validate_layout(&bad)
            .unwrap_err()
            .contains("outside payload"));
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = prebindgen_flat::WireNode::Leaf(vec![leaf("flag")]);
        crate::group::values_mut(&mut flag).next().unwrap().kind = JniWire::selector();
        enclose_optional_metadata(&mut flag, optional("flag", "flag", None));
        validate_layout(&flag).unwrap();
        assert!(JniOptional::from_parts(
            syn::parse_quote!(outer),
            syn::parse_quote!(flag),
            None,
            &[selector(syn::parse_quote!(flag))]
        )
        .unwrap_err()
        .contains("one matching flag"));
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let field = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Field(syn::parse_quote!(x))),
            children: vec![WireNode::default()],
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(None)),
            children: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(None)),
            children: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(None)),
            children: vec![WireNode::Leaf(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty = WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Fields(None)),
            children: vec![],
        };
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
        input.node = prebindgen_flat::WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Field(syn::parse_quote!(orphan))),
            children: vec![std::mem::take(&mut input.node)],
        };
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
        output.outcomes.success = prebindgen_flat::WireNode::Branch {
            controls: vec![],
            meta: JniBranch::Group(JniGroup::Field(syn::parse_quote!(orphan))),
            children: vec![std::mem::take(&mut output.outcomes.success)],
        };
        assert!(JniCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("direct child"));
    }
    #[test]
    fn reserved_niches_are_checked_independently_of_remaining_availability() {
        let mut wire = JniWire::scalar(Primitive::Char);
        wire.clear_niche();
        let mut tree = prebindgen_flat::WireNode::Leaf(vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(value),
            kind: wire,
        }]);
        enclose_optional_metadata(&mut tree, optional("present", "value", Some(-1)));
        validate_layout(&tree).unwrap();
        crate::group::values_mut(&mut tree).next().unwrap().kind = JniWire::scalar(Primitive::I64);
        assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        crate::group::values_mut(&mut tree).next().unwrap().kind = JniWire::Scalar {
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
        crate::group::values_mut(&mut input.node)
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
        crate::group::values_mut(&mut output.outcomes.success)
            .next()
            .unwrap()
            .kind = crate::group::values(&input.node)
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
        let mut tree = prebindgen_flat::WireNode::Leaf(vec![leaf("value")]);
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        prebindgen_flat::wire_definitions_mut(std::iter::once(&mut tree))
            .next()
            .expect("logical wire index out of range")
            .kind
            .set_niche(Some(JniNiche::Integer(-2)))
            .unwrap();
        validate_layout(&tree).unwrap();
        enclose_optional_metadata(&mut tree, optional("outer", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("duplicate JNI niche"));
    }
}

#[cfg(test)]
mod abi_tests {
    use super::*;
    #[test]
    fn function_parameters_result_packets_and_sequence_fields_have_explicit_roles() {
        let scalar = |name: &str| prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::I32),
        };
        let element: prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch> =
            prebindgen_flat::WireNode::Leaf(vec![scalar("first"), scalar("second")]);
        let mut sequence = element.clone();
        sequence = prebindgen_flat::WireNode::Chain {
            controls: vec![prebindgen_flat::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            meta: JniChain::Sequence(JniSequence::Buffered),
            child: Box::new(std::mem::take(&mut sequence)),
        };
        let result = prebindgen_flat::Outcomes {
            success: sequence.clone(),
            source_error: Some(WireNode::Leaf(vec![scalar("error")])),
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
        let mut input = JniCodec
            .input_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(raw),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        input.node = sequence;
        flat.function_mut("f").unwrap().params[0].ty.slot = Some(input.clone().into());
        assert_eq!(
            function_parameters(flat.function("f").unwrap())
                .unwrap()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows"]
        );
    }
}

#[cfg(test)]
mod structural_validation_tests {
    use super::*;
    #[test]
    fn adapter_rejects_empty_alternatives_and_unsupported_group_controls() {
        let node: WireNode<JniWire, JniChain, JniBranch> = WireNode::Branch {
            meta: JniBranch::Alternatives,
            children: vec![],
            controls: vec![],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("no alternatives"));
        let node = WireNode::Branch {
            meta: JniBranch::Group(JniGroup::Values),
            children: vec![],
            controls: vec![Wire {
                name: syn::parse_quote!(extra),
                kind: JniWire::scalar(Primitive::I32),
            }],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("control wires"));
    }
}
