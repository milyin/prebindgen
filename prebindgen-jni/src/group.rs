//! JNI adapter metadata and interpretation of logical value trees.
use super::*;

use prebindgen_conversions::{Wire, WireNode};
use syn::ext::IdentExt;

/// Optional and sequence structure shared by the directional wire tree families.
/// `Sequence` is `()` for inputs and the adapter's delivery policy for outputs.
/// Use the concrete input/output aliases at converter boundaries.
#[derive(Clone, Debug)]
pub enum JniWireChain<Sequence> {
    /// Zero or one payload occurrence.
    Optional(JniOptional),
    /// Zero or more element occurrences.
    Sequence(Sequence),
    /// Expanded fields grouped into one C-representation bulk frame.
    /// Activation follows the enclosing optional/selector branches; layout comes from the child.
    Bulk,
}
/// Structural operations shared by the directional wire tree families.
/// Input-only payloads are `()` in inputs and `Infallible` in outputs.
/// Output-only payloads use the reverse assignment. These payloads occupy no space.
/// Use the concrete input/output aliases at converter boundaries.
#[derive(Clone, Debug)]
pub enum JniWireBranch<Input, Output> {
    /// Convert all ordered components, such as constructor arguments or output views.
    Values,
    /// Components of one source argument, preserving its Kotlin parameter path.
    Argument(String, Input),
    /// Deliver an output view under an explicit Kotlin name.
    Named(String, Output),
    /// Convert named source fields independently, in declaration order.
    Fields,
    /// Convert one named source field through its child subtree.
    Field(syn::Ident),
    /// Decompose or reconstruct one Kotlin data-class value through named child fields.
    Record(JniDataGroup),
    /// Convert exactly one constructor alternative selected by its incoming tag.
    Alternatives(Input),
    /// Convert the selected source enum variant as a sealed value or expanded tag/fields.
    Enum(std::rc::Rc<crate::sealed::JniEnumGroup>),
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
    // Read presence from the actual wire discriminator, including niche encoding.
    // Useful before the input converter has produced its normalized presence local.
    pub(crate) fn is_present<Sq: crate::sequence::RowEncoding, In, Out>(
        &self,
        child: &JniWireNode<Sq, In, Out>,
    ) -> Result<TokenStream> {
        let name = self.discriminator();
        if let Some(niche) = self.niche() {
            let wire = argument_parameters(child)
                .find(|wire| wire.name == *name)
                .ok_or("missing callback return optional discriminator")?;
            Ok(niche.present(&wire.kind, quote!(#name)))
        } else {
            Ok(quote!(#name == 1))
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
pub(crate) fn values<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    fn visit<'a, Sq: crate::sequence::RowEncoding, In, Out>(
        node: &'a JniWireNode<Sq, In, Out>,
        out: &mut Vec<&'a Wire<JniWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta: JniWireChain::Sequence(_) | JniWireChain::Bulk,
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
pub(crate) fn values_mut<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &mut JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&mut Wire<JniWire>> {
    fn visit<'a, Sq: crate::sequence::RowEncoding, In, Out>(
        node: &'a mut JniWireNode<Sq, In, Out>,
        out: &mut Vec<&'a mut Wire<JniWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta: JniWireChain::Sequence(_) | JniWireChain::Bulk,
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
pub(crate) fn validate_layout<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> Result<()> {
    validate_layout_in(node, false)
}
pub(crate) fn validate_layout_in<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
    bulk: bool,
) -> Result<()> {
    validate_scope(node, Some(bulk))
}
// Codec hooks also validate fragments before their enclosing scope is attached.
// They can check the physical shape, but cannot require a boundary context yet.
pub(crate) fn validate_fragment<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> Result<()> {
    validate_scope(node, None)
}
fn validate_scope<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
    bulk: Option<bool>,
) -> Result<()> {
    prebindgen_conversions::validate_wire_names(prebindgen_conversions::wire_definitions(
        std::iter::once(node),
    ))?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit<Sq: crate::sequence::RowEncoding, In, Out>(
        node: &JniWireNode<Sq, In, Out>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<JniNiche>>,
        bulk: Option<bool>,
    ) -> Result<()> {
        match node {
            WireNode::Chain {
                meta: JniWireChain::Bulk,
                child,
                controls,
            } => {
                if bulk == Some(true) {
                    return Err(
                        "nested bulk frames must be lowered into the enclosing bulk scope".into(),
                    );
                }
                if controls.is_empty() || !matches!(controls[0].kind, JniWire::BulkBuffer { .. }) {
                    return Err("bulk scope requires a byte-array carrier".into());
                }
                visit(child, bindings, in_fields, reserved, Some(true))?;
            }
            WireNode::Leaf(wires) => {
                for w in wires {
                    if let JniWire::Callback { callback } = &w.kind {
                        for root in callback.bulk_nodes() {
                            match root {
                                metadata::BoundaryNode::Input(node) => {
                                    validate_layout(node)?;
                                }
                                metadata::BoundaryNode::Output(node) => {
                                    validate_layout(node)?;
                                }
                            }
                        }
                    }
                    if bulk == Some(false)
                        && matches!(
                            w.kind,
                            JniWire::NativeBuffer { .. } | JniWire::Native128 { .. }
                        )
                    {
                        return Err("native bulk fields require an enclosing bulk scope".into());
                    }
                    if matches!(w.kind, JniWire::Rows { .. } | JniWire::NativeRows { .. }) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Branch {
                meta: JniWireBranch::Alternatives(_) | JniWireBranch::Enum(_),
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
                    visit(arm, bindings, false, reserved, bulk)?;
                }
            }
            WireNode::Chain {
                child: element,
                controls,
                meta: JniWireChain::Sequence(metadata),
                ..
            } => {
                let [carrier] = controls.as_slice() else {
                    return Err("sequence requires one carrier control".into());
                };
                let valid = match metadata.delivery() {
                    JniSequence::Buffered | JniSequence::TypedBuffered => match bulk {
                        Some(true) => matches!(carrier.kind, JniWire::NativeRows { .. }),
                        Some(false) => matches!(carrier.kind, JniWire::Rows { .. }),
                        None => matches!(
                            carrier.kind,
                            JniWire::Rows { .. } | JniWire::NativeRows { .. }
                        ),
                    },
                    JniSequence::Callback => {
                        if bulk == Some(true) {
                            return Err(
                                "bulk scopes cannot enclose callback sequence delivery".into()
                            );
                        }
                        matches!(&carrier.kind, JniWire::Selector { .. })
                    }
                };
                if !valid {
                    return Err("sequence carrier control is incompatible with its delivery".into());
                }
                if matches!(carrier.kind, JniWire::Rows { .. })
                    && metadata.delivery() == JniSequence::TypedBuffered
                    && !typed_rows::supported(element)
                {
                    return Err(
                        "typed output row constructor exceeds 254 JVM parameter slots".into(),
                    );
                }
                if values(element).len() == 0 && matches!(carrier.kind, JniWire::Rows { .. }) {
                    return Err("sequence requires a nonempty element layout".into());
                }
                crate::sequence::validate_buffered(element)?;
                validate_scope(element, bulk)?;
            }
            WireNode::Chain {
                child: element,
                meta: JniWireChain::Optional(o),
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
                visit(element, bindings, false, reserved, bulk)?;
            }
            WireNode::Branch {
                children,
                meta: metadata,
                controls,
                ..
            } => {
                if !controls.is_empty() {
                    return Err("group metadata does not support control wires".into());
                }
                if let JniWireBranch::Record(record) = metadata {
                    if !bindings.insert(record.root.clone()) {
                        return Err(format!("duplicate Kotlin group binding `{}`", record.root));
                    }
                }
                match metadata {
                    JniWireBranch::Named(name, _) => {
                        validate_name(name)?;
                    }
                    JniWireBranch::Field(_) if !in_fields => {
                        return Err(
                            "named field must be a direct child of a field collection".into()
                        )
                    }
                    JniWireBranch::Fields | JniWireBranch::Record(_) => {
                        let mut fields = std::collections::BTreeSet::new();
                        for child in children.as_slice() {
                            let WireNode::Branch {
                                meta: JniWireBranch::Field(name),
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
                        matches!(metadata, JniWireBranch::Fields | JniWireBranch::Record(_)),
                        reserved,
                        bulk,
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
        bulk,
    )
}

/// Native parameters contributed by a source argument.
pub(crate) fn argument_parameters<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    'a,
    I: prebindgen_conversions::InputConversion<
        Wire = JniWire,
        Chain = JniInputWireChain,
        Branch = JniInputWireBranch,
    >,
    O: prebindgen_conversions::Conversion,
    M,
>(
    f: &'a prebindgen_conversions::BoundFunction<'_, I, O, M>,
) -> Result<std::vec::IntoIter<&'a Wire<JniWire>>> {
    let mut out = vec![];
    for input in f.input_conversions()? {
        out.extend(argument_parameters(input.input_node()));
    }
    Ok(out.into_iter())
}
pub(crate) fn result_packet_fields<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn result_packet<'a, Sq: crate::sequence::RowEncoding, In, Out>(
    node: &'a JniWireNode<Sq, In, Out>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn row_values<'a, Sq: crate::sequence::RowEncoding, In, Out>(
    node: &'a JniWireNode<Sq, In, Out>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn prepare_optional<
    Sq: crate::sequence::RowEncoding,
    In,
    Out,
    V: prebindgen_conversions::Conversion<
        Wire = JniWire,
        Chain = JniWireChain<Sq>,
        Branch = JniWireBranch<In, Out>,
    >,
>(
    request: prebindgen_declarations::Optional<'_, JniCodec, V>,
    direction: prebindgen_declarations::Direction,
) -> Result<OptionalEncoding<JniWire, JniWireChain<Sq>>> {
    let prepared = prebindgen_declarations::prepare_optional_payload(
        values_mut(request.payload),
        direction,
        JniWire::available_niche,
        JniWire::set_niche,
        |wire, niche, value| Ok(niche.present(wire, value)),
        optional::remaining,
        |wire, niche| optional::absent(wire, niche, false),
    )?;
    let discriminator = prepared
        .reservation
        .as_ref()
        .map_or_else(|| request.presence.clone(), |r| r.discriminator.clone());
    let (presence, controls) = if prepared.reservation.is_some() {
        (
            Presence::Niche {
                input_test: prepared.input_test,
            },
            Vec::new(),
        )
    } else {
        let flag = Wire {
            name: request.presence.clone(),
            kind: JniWire::selector(),
        };
        (Presence::Flag(flag.clone()), vec![flag])
    };
    let metadata = JniOptional::from_parts(
        request.presence,
        discriminator,
        prepared.reservation.map(|r| r.niche),
        &controls,
    )
    .map(JniWireChain::Optional)?;
    Ok(OptionalEncoding {
        presence,
        metadata,
        absent_values: prepared.absent_values,
    })
}
pub(crate) fn retire_niches<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &mut JniWireNode<Sq, In, Out>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(
        values_mut(node),
        reservation,
        optional::remaining,
        JniWire::set_niche,
    )
}
pub(crate) fn kotlin_result_fields<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selector(name: syn::Ident) -> prebindgen_conversions::Wire<JniWire> {
        prebindgen_conversions::Wire {
            name,
            kind: JniWire::selector(),
        }
    }
    fn enclose_optional_metadata(tree: &mut JniInputWireNode, metadata: JniOptional) {
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
            meta: JniInputWireChain::Optional(metadata),
        };
    }
    fn leaf(name: &str) -> prebindgen_conversions::Wire<JniWire> {
        prebindgen_conversions::Wire {
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
            let node: JniInputWireNode = WireNode::Branch {
                controls: vec![carrier],
                children: vec![WireNode::default()],
                meta: JniInputWireBranch::Alternatives(()),
            };
            validate_layout(&node).unwrap();
            assert_eq!(
                prebindgen_conversions::wire_definitions(std::iter::once(&node))
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
        let sequence: JniInputWireNode = WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniInputWireChain::Sequence(()),
        };
        validate_layout(&sequence).unwrap();
        let empty: JniInputWireNode = WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::default()),
            meta: JniInputWireChain::Sequence(()),
        };
        assert!(validate_layout(&empty)
            .unwrap_err()
            .contains("nonempty element"));
        let choice: JniInputWireNode = WireNode::Branch {
            controls: vec![selector(syn::parse_quote!(tag))],
            children: vec![WireNode::default()],
            meta: JniInputWireBranch::Alternatives(()),
        };
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn malformed_tree_controls_are_rejected() {
        let mut tree: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            children: vec![WireNode::default()],
            meta: JniInputWireBranch::Alternatives(()),
        };
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("one selector control"));
        let WireNode::Branch {
            meta: JniInputWireBranch::Alternatives(()),
            controls,
            ..
        } = &mut tree
        else {
            panic!()
        };
        controls.push(leaf("wrong_kind"));
        assert!(validate_layout(&tree).unwrap_err().contains("integer tag"));
        let empty_sequence: JniInputWireNode = WireNode::Chain {
            controls: vec![],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniInputWireChain::Sequence(()),
        };
        assert!(validate_layout(&empty_sequence)
            .unwrap_err()
            .contains("one carrier control"));
        let flag: JniInputWireNode = WireNode::Chain {
            controls: vec![selector(syn::parse_quote!(actual))],
            child: Box::new(WireNode::default()),
            meta: JniInputWireChain::Optional(JniOptional::Flag(syn::parse_quote!(missing))),
        };
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("does not match"));
    }
    #[test]
    fn callback_control_requires_a_scalar_status() {
        let mut node: JniOutputWireNode = WireNode::Chain {
            controls: vec![Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: "false".into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniOutputWireChain::Sequence(JniSequence::Callback),
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("incompatible with its delivery"));
        let WireNode::Chain { controls, .. } = &mut node else {
            panic!()
        };
        controls[0].kind = JniWire::selector();
        validate_layout(&node).unwrap();
        let WireNode::Chain {
            meta: JniOutputWireChain::Sequence(metadata),
            ..
        } = &mut node
        else {
            panic!()
        };
        *metadata = JniSequence::Buffered;
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("incompatible with its delivery"));
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree: JniInputWireNode =
            prebindgen_conversions::WireNode::Leaf(vec![leaf("value")]);
        prebindgen_conversions::wire_definitions_mut(std::iter::once(&mut tree))
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
        let mut flag: JniInputWireNode = prebindgen_conversions::WireNode::Leaf(vec![leaf("flag")]);
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
        let field: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Field(syn::parse_quote!(x)),
            children: vec![WireNode::default()],
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Fields,
            children: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Fields,
            children: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Fields,
            children: vec![WireNode::Leaf(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Fields,
            children: vec![],
        };
        validate_layout(&empty).unwrap();
    }
    #[test]
    fn malformed_metadata_is_rejected_before_final_slots_are_installed() {
        let mut input = JniCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        input.node = prebindgen_conversions::WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::Field(syn::parse_quote!(orphan)),
            children: vec![std::mem::take(&mut input.node)],
        };
        assert!(JniCodec::raw()
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("direct child"));
        let mut output = JniCodec::raw()
            .output_leaf(prebindgen_declarations::OutputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                prebindgen_declarations::ValueAccess::Owned,
            ))
            .unwrap();
        output.outcomes.success = prebindgen_conversions::WireNode::Branch {
            controls: vec![],
            meta: JniOutputWireBranch::Field(syn::parse_quote!(orphan)),
            children: vec![std::mem::take(&mut output.outcomes.success)],
        };
        assert!(JniCodec::raw()
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("direct child"));
    }
    #[test]
    fn reserved_niches_are_checked_independently_of_remaining_availability() {
        let mut wire = JniWire::scalar(Primitive::Char);
        wire.clear_niche();
        let mut tree: JniInputWireNode =
            prebindgen_conversions::WireNode::Leaf(vec![prebindgen_conversions::Wire {
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
        let mut input = JniCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        crate::group::values_mut(&mut input.node)
            .next()
            .unwrap()
            .kind = JniWire::Scalar {
            primitive: Primitive::I64,
            next_integer_niche: Some(-1),
        };
        assert!(JniCodec::raw()
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("incompatible"));
        let mut output = JniCodec::raw()
            .output_leaf(prebindgen_declarations::OutputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                prebindgen_declarations::ValueAccess::Owned,
            ))
            .unwrap();
        crate::group::values_mut(&mut output.outcomes.success)
            .next()
            .unwrap()
            .kind = crate::group::values(&input.node)
            .next()
            .unwrap()
            .kind
            .clone();
        assert!(JniCodec::raw()
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("incompatible"));
    }
    #[test]
    fn optional_reservations_cannot_overlap_each_other_or_available_state() {
        let mut tree: JniInputWireNode =
            prebindgen_conversions::WireNode::Leaf(vec![leaf("value")]);
        enclose_optional_metadata(&mut tree, optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        prebindgen_conversions::wire_definitions_mut(std::iter::once(&mut tree))
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
        let scalar = |name: &str| prebindgen_conversions::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::I32),
        };
        let element: JniInputWireNode =
            prebindgen_conversions::WireNode::Leaf(vec![scalar("first"), scalar("second")]);
        let mut sequence = element.clone();
        sequence = prebindgen_conversions::WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::Rows {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            meta: JniInputWireChain::Sequence(()),
            child: Box::new(std::mem::take(&mut sequence)),
        };
        let result = prebindgen_conversions::Outcomes {
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
        let mut flat: JniBindings =
            Flat::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let mut input = JniCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(raw),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        input.node = sequence;
        bindings::set_input(&mut flat, "f", "value", input.clone().into()).unwrap();
        let output = prebindgen_declarations::compile_output(
            &flat,
            &JniCodec::raw(),
            "f",
            &JniOutput::None,
            &(),
        )
        .unwrap();
        bindings::set_output(&mut flat, "f", output).unwrap();
        assert_eq!(
            argument_parameters(&input.node)
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
        let node: JniInputWireNode = WireNode::Branch {
            meta: JniInputWireBranch::Alternatives(()),
            children: vec![],
            controls: vec![],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("no alternatives"));
        let node: JniInputWireNode = WireNode::Branch {
            meta: JniInputWireBranch::Values,
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

/// Physical/error decoding shape, ignoring wire names and Kotlin presentation
/// groups. Nested carriers and optional encodings must agree as well as the
/// top-level JVM signature, because the latter cannot describe rows in a buffer.
pub(crate) fn error_wire_shape<Sq: crate::sequence::RowEncoding, In, Out>(
    node: &JniWireNode<Sq, In, Out>,
) -> Vec<String> {
    fn wires(wires: &[Wire<JniWire>], out: &mut Vec<String>) {
        for wire in wires {
            let detail = match &wire.kind {
                JniWire::Scalar { primitive, .. } | JniWire::Native128 { primitive, .. } => {
                    format!("{primitive:?}")
                }
                JniWire::Handle { handle, .. } => {
                    format!("{}:{:?}", type_key(&handle.ty), handle.access)
                }
                JniWire::Enum { enum_type, .. } => type_key(&enum_type.mapping.ty),
                JniWire::NativeBuffer { text } => text.to_string(),
                JniWire::Callback { callback } => callback.fqn(),
                _ => String::new(),
            };
            out.push(format!(
                "{:?}:{}:{detail}",
                std::mem::discriminant(&wire.kind),
                type_key(&wire.kind.rust_type())
            ));
        }
    }
    fn visit<Sq: crate::sequence::RowEncoding, In, Out>(
        node: &JniWireNode<Sq, In, Out>,
        out: &mut Vec<String>,
        bulk: bool,
    ) {
        match node {
            WireNode::Leaf(fields) => wires(fields, out),
            WireNode::Branch {
                meta: meta @ (JniWireBranch::Alternatives(_) | JniWireBranch::Enum(_)),
                controls,
                children,
            } => {
                // Packed enums use a union; selector alternatives store their
                // fields consecutively. Their JVM carriers alone are identical.
                out.push(match meta {
                    JniWireBranch::Enum(group) if bulk => format!("enum:{}[", group.source.key()),
                    _ => "alternatives[".into(),
                });
                wires(controls, out);
                for child in children {
                    out.push("arm[".into());
                    visit(child, out, bulk);
                    out.push("]".into());
                }
                out.push("]".into());
            }
            WireNode::Branch {
                controls, children, ..
            } => {
                wires(controls, out);
                for child in children {
                    visit(child, out, bulk);
                }
            }
            WireNode::Chain {
                meta,
                controls,
                child,
            } => {
                let label = match meta {
                    JniWireChain::Optional(JniOptional::Flag(_)) => "optional:flag".into(),
                    JniWireChain::Optional(JniOptional::Niche { reservation, .. }) => {
                        let index = prebindgen_conversions::wire_definitions(std::iter::once(
                            child.as_ref(),
                        ))
                        .position(|wire| wire.name == reservation.discriminator);
                        format!("optional:niche:{:?}:{index:?}", reservation.niche)
                    }
                    JniWireChain::Sequence(_) if bulk => "sequence:native-rows".into(),
                    JniWireChain::Sequence(delivery) => {
                        format!("sequence:{:?}", delivery.delivery())
                    }
                    JniWireChain::Bulk => {
                        let mut fields = group::row_fields(child)
                            .map(|w| {
                                type_key(
                                    &bulk::field_type(&w.kind).expect("prepared bulk storage type"),
                                )
                            })
                            .collect::<Vec<_>>();
                        if fields.is_empty() {
                            fields.push("u8".into());
                        }
                        format!("bulk:{fields:?}")
                    }
                };
                out.push(label);
                out.push("[".into());
                wires(controls, out);
                visit(child, out, bulk || matches!(meta, JniWireChain::Bulk));
                out.push("]".into());
            }
        }
    }
    let mut out = Vec::new();
    visit(node, &mut out, false);
    out
}

use prebindgen_conversions::FunctionConversions as _;
