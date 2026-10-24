//! JNI adapter metadata and interpretation of logical value trees.
use super::*;
use prebindgen_flat::{Wire, WireNode};
use syn::ext::IdentExt;

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
/// An integer tag binding used for alternative selection or presence.
/// The carrier is private so callers cannot replace it with an unrelated wire kind.
#[derive(Clone, Debug)]
pub struct JniSelector {
    carrier: Wire<JniWire>,
}
impl JniSelector {
    /// Create a tag binding with the adapter's initial niche availability.
    pub fn new(name: syn::Ident) -> Self {
        Self {
            carrier: Wire {
                name,
                kind: JniWire::selector(),
            },
        }
    }
    /// The single integer value passed across the boundary.
    pub fn carrier(&self) -> &Wire<JniWire> {
        &self.carrier
    }
    pub(crate) fn carrier_mut(&mut self) -> &mut Wire<JniWire> {
        &mut self.carrier
    }
}
impl TryFrom<Wire<JniWire>> for JniSelector {
    type Error = String;
    fn try_from(carrier: Wire<JniWire>) -> Result<Self> {
        if !matches!(carrier.kind, JniWire::Selector { .. }) {
            return Err("selector requires the adapter's integer tag representation".into());
        }
        carrier.kind.validate()?;
        Ok(Self { carrier })
    }
}
/// How this optional occurrence distinguishes absence from a present payload.
#[derive(Clone, Debug)]
pub enum JniOptional {
    /// A dedicated 0/1 tag. The binding is also the normalized presence value.
    Flag(JniSelector),
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
            Self::Flag(flag) => &flag.carrier.name,
            Self::Niche { presence, .. } => presence,
        }
    }
    /// Binding whose value distinguishes absent and present states.
    pub fn discriminator(&self) -> &syn::Ident {
        match self {
            Self::Flag(flag) => &flag.carrier.name,
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
    /// Dedicated presence wire, if this occurrence uses one.
    pub fn control(&self) -> Option<&Wire<JniWire>> {
        match self {
            Self::Flag(flag) => Some(flag.carrier()),
            Self::Niche { .. } => None,
        }
    }
    fn control_mut(&mut self) -> Option<&mut Wire<JniWire>> {
        match self {
            Self::Flag(flag) => Some(flag.carrier_mut()),
            Self::Niche { .. } => None,
        }
    }
    pub(crate) fn from_parts(
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<JniNiche>,
        controls: Vec<Wire<JniWire>>,
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
                Ok(Self::Flag(JniSelector::try_from(
                    controls.into_iter().next().unwrap(),
                )?))
            }
            _ => {
                Err("optional encoding requires either one matching flag or a payload niche".into())
            }
        }
    }
}

fn values(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    let mut out = vec![];
    read(node, &mut out, false);
    out.into_iter()
}
fn values_mut(
    node: &mut WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&mut Wire<JniWire>> {
    fn visit<'a>(
        node: &'a mut WireNode<
            JniWire,
            JniOptional,
            crate::JniSequence,
            JniGroup,
            crate::JniSelector,
        >,
        out: &mut Vec<&'a mut Wire<JniWire>>,
    ) {
        match node {
            WireNode::Wire(wires) => out.extend(wires),
            WireNode::Group { elements, .. } => {
                for child in elements {
                    visit(child, out);
                }
            }
            WireNode::Optional { element, metadata } => {
                out.extend(metadata.control_mut());
                visit(element, out);
            }
            WireNode::Sequence { metadata, .. } => out.push(metadata.carrier_mut()),
            WireNode::Alternatives { elements, metadata } => {
                out.push(metadata.carrier_mut());
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
    node: &'a WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    out: &mut Vec<&'a Wire<JniWire>>,
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
            out.extend(metadata.control());
            read(element, out, elements);
        }
        WireNode::Sequence { element, metadata } => {
            out.push(metadata.carrier());
            if elements {
                read(element, out, elements);
            }
        }
        WireNode::Alternatives {
            elements: children,
            metadata,
        } => {
            out.push(metadata.carrier());
            for child in children {
                read(child, out, elements);
            }
        }
    }
}
pub(crate) fn validate_layout(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> Result<()> {
    prebindgen_flat::validate_wire_nodes(std::iter::once(node))?;
    let mut definitions = vec![];
    read(node, &mut definitions, true);
    prebindgen_conversions::validate_wire_names(definitions)?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit(
        node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<JniNiche>>,
    ) -> Result<()> {
        match node {
            WireNode::Wire(wires) => {
                for w in wires {
                    if matches!(w.kind, JniWire::Rows { .. }) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Alternatives {
                elements: alternatives,
                metadata: _,
            } => {
                for arm in alternatives {
                    visit(arm, bindings, false, reserved)?;
                }
            }
            WireNode::Sequence {
                element,
                metadata: _,
            } => {
                if values(element).len() == 0 {
                    return Err("sequence requires a nonempty element layout".into());
                }
                crate::sequence::validate_buffered(element)?;
                validate_layout(element)?;
            }
            WireNode::Optional {
                element,
                metadata: o,
            } => {
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
            WireNode::Group {
                elements: children,
                metadata,
            } => {
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
                            let WireNode::Group {
                                metadata: JniGroup::Field(name),
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
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    I: prebindgen_flat::InputConversion<
        Wire = JniWire,
        Optional = JniOptional,
        Sequence = JniSequence,
        Group = JniGroup,
        Alternatives = JniSelector,
    >,
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
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn result_packet<'a>(
    node: &'a WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn row_values<'a>(
    node: &'a WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn optional_niche(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> Option<prebindgen_declarations::NicheReservation<JniNiche>> {
    prebindgen_declarations::select_optional_niche(&JniCodec, values(node))
}
pub(crate) fn optional_presence(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    reservation: &prebindgen_declarations::NicheReservation<JniNiche>,
) -> Result<TokenStream> {
    prebindgen_declarations::optional_presence(&JniCodec, values(node), reservation)
}
pub(crate) fn absent_output_values(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<Vec<(syn::Ident, TokenStream)>> {
    prebindgen_declarations::absent_output_values(&JniCodec, values(node), reservation)
}
pub(crate) fn retire_niches(
    node: &mut WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(&JniCodec, values_mut(node), reservation)
}
pub(crate) fn kotlin_result_fields(
    node: &WireNode<JniWire, JniOptional, crate::JniSequence, JniGroup, crate::JniSelector>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn enclose_optional_metadata(
        tree: &mut WireNode<JniWire, JniOptional, JniSequence, JniGroup, JniSelector>,
        mut metadata: JniOptional,
    ) {
        let mut payload = std::mem::take(tree);
        if matches!(metadata, JniOptional::Flag(_)) {
            if let WireNode::Wire(wires) = &mut payload {
                if !wires.is_empty() {
                    metadata = JniOptional::Flag(JniSelector::try_from(wires.remove(0)).unwrap());
                }
            }
        }
        *tree = WireNode::Optional {
            element: Box::new(payload),
            metadata,
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
            None => JniOptional::Flag(JniSelector::new(syn::parse_str(discriminator).unwrap())),
        }
    }
    #[test]
    fn optional_construction_rejects_inconsistent_flag_and_niche_parts() {
        let flag = JniSelector::new(syn::parse_quote!(flag)).carrier().clone();
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            vec![]
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            vec![flag.clone(), flag.clone()]
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(other),
            syn::parse_quote!(flag),
            None,
            vec![flag.clone()]
        )
        .is_err());
        assert!(JniOptional::from_parts(
            syn::parse_quote!(present),
            syn::parse_quote!(payload),
            Some(JniNiche::Integer(-1)),
            vec![flag.clone()]
        )
        .is_err());
        let encoding = JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            vec![flag],
        )
        .unwrap();
        assert!(matches!(encoding, JniOptional::Flag(_)));
        assert_eq!(encoding.presence(), encoding.discriminator());
    }
    #[test]
    fn selector_construction_preserves_consumed_and_remaining_niches() {
        for remaining in [None, Some(JniNiche::Integer(-3))] {
            let mut carrier = JniSelector::new(syn::parse_quote!(tag)).carrier().clone();
            carrier.kind.set_niche(remaining.clone()).unwrap();
            let selector = JniSelector::try_from(carrier).unwrap();
            assert_eq!(selector.carrier().kind.available_niche(), remaining);
        }
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        assert!(JniSelector::try_from(leaf("tag"))
            .unwrap_err()
            .contains("integer tag"));
        assert!(JniOptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            vec![leaf("flag")]
        )
        .unwrap_err()
        .contains("integer tag"));
        let sequence: WireNode<JniWire, JniOptional, JniSequence, JniGroup, JniSelector> =
            WireNode::Sequence {
                element: Box::new(WireNode::Wire(vec![leaf("value")])),
                metadata: JniSequence::buffered(syn::parse_quote!(rows), "true"),
            };
        validate_layout(&sequence).unwrap();
        let empty = WireNode::Sequence {
            element: Box::new(WireNode::default()),
            metadata: JniSequence::buffered(syn::parse_quote!(rows), "true"),
        };
        assert!(validate_layout(&empty)
            .unwrap_err()
            .contains("nonempty element"));
        let choice = WireNode::Alternatives {
            elements: vec![WireNode::default()],
            metadata: JniSelector::new(syn::parse_quote!(tag)),
        };
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = prebindgen_flat::WireNode::Wire(vec![leaf("value")]);
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
        let mut flag = prebindgen_flat::WireNode::Wire(vec![leaf("flag")]);
        crate::group::values_mut(&mut flag).next().unwrap().kind = JniWire::selector();
        enclose_optional_metadata(&mut flag, optional("flag", "flag", None));
        validate_layout(&flag).unwrap();
        assert!(JniOptional::from_parts(
            syn::parse_quote!(outer),
            syn::parse_quote!(flag),
            None,
            vec![JniSelector::new(syn::parse_quote!(flag)).carrier().clone()]
        )
        .unwrap_err()
        .contains("one matching flag"));
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let field = WireNode::Group {
            metadata: JniGroup::Field(syn::parse_quote!(x)),
            elements: vec![WireNode::default()],
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates = WireNode::Group {
            metadata: JniGroup::Fields(None),
            elements: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record = WireNode::Group {
            metadata: JniGroup::Fields(None),
            elements: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed = WireNode::Group {
            metadata: JniGroup::Fields(None),
            elements: vec![WireNode::Wire(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty = WireNode::Group {
            metadata: JniGroup::Fields(None),
            elements: vec![],
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
        input.node = prebindgen_flat::WireNode::Group {
            metadata: JniGroup::Field(syn::parse_quote!(orphan)),
            elements: vec![std::mem::take(&mut input.node)],
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
        output.outcomes.success = prebindgen_flat::WireNode::Group {
            metadata: JniGroup::Field(syn::parse_quote!(orphan)),
            elements: vec![std::mem::take(&mut output.outcomes.success)],
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
        let mut tree = prebindgen_flat::WireNode::Wire(vec![prebindgen_flat::Wire {
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
        let mut tree = prebindgen_flat::WireNode::Wire(vec![leaf("value")]);
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
        let element: prebindgen_flat::WireNode<
            JniWire,
            crate::JniOptional,
            crate::JniSequence,
            JniGroup,
            crate::JniSelector,
        > = prebindgen_flat::WireNode::Wire(vec![scalar("first"), scalar("second")]);
        let mut sequence = element.clone();
        sequence = prebindgen_flat::WireNode::Sequence {
            metadata: JniSequence::buffered(syn::parse_quote!(rows), "true"),
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
