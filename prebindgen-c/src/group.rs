//! C adapter metadata and interpretation of logical value trees.
use super::*;

use prebindgen_conversions::{Wire, WireNode};
use syn::ext::IdentExt;

/// Decode one nested input from C into Rust.
///
/// Output delivery cannot be selected on an input:
/// ```compile_fail
/// use prebindgen_c::{CInputWireChain, CSequence};
/// let input = CInputWireChain::Sequence(CSequence::Callback);
/// ```
#[derive(Clone, Debug)]
pub enum CInputWireChain {
    /// Use a flag or payload niche to decode zero or one Rust value.
    Optional(COptional),
    /// Read a tagged carrier; the child describes its converted payload fields.
    Tagged(std::rc::Rc<crate::CTagged>),
    /// Read buffered C rows. The child describes how one row becomes a Rust element.
    Sequence,
}
/// Encode one nested Rust output for C delivery.
#[derive(Clone, Debug)]
pub enum COutputWireChain {
    /// Encode absence with a flag or an excluded payload representation.
    Optional(COptional),
    /// Build a tagged carrier; the child describes its converted payload fields.
    Tagged(std::rc::Rc<crate::CTagged>),
    /// Buffer rows or deliver each row through a callback, according to this policy.
    Sequence(CSequence),
}
/// How C input components reconstruct a Rust value.
#[derive(Clone, Debug)]
pub enum CInputWireBranch {
    /// Decode every ordered child, such as the arguments of a Rust constructor.
    Values,
    /// Decode named Rust fields in source declaration order.
    Fields,
    /// Decode the child for this named Rust field.
    Field(syn::Ident),
    /// An incoming selector chooses exactly one Rust constructor alternative.
    /// The selector belongs to the node's controls; children describe the alternatives.
    Alternatives,
    /// Decode the selected source enum variant; children follow wire-tag order.
    Enum(std::rc::Rc<crate::CEnumGroup>),
}
/// How Rust output components become C values.
///
/// Output branches have no constructor alternative operation:
/// ```compile_fail
/// use prebindgen_c::COutputWireBranch;
/// let output = COutputWireBranch::Alternatives;
/// ```
#[derive(Clone, Debug)]
pub enum COutputWireBranch {
    /// Deliver every ordered child, such as multiple projections of one Rust value.
    Values,
    /// Project named Rust fields in source declaration order.
    Fields,
    /// Encode this named Rust field through its child subtree.
    Field(syn::Ident),
    /// Encode the selected source enum variant; children follow wire-tag order.
    Enum(std::rc::Rc<crate::CEnumGroup>),
}

// Private queries used by shared wire-scope, selection and layout algorithms.
pub(crate) trait ChainMetadata {
    fn optional(&self) -> Option<&COptional>;
    fn sequence(&self) -> Option<CSequence>;
    fn tagged(&self) -> Option<&std::rc::Rc<CTagged>>;
    fn is_scope_boundary(&self) -> bool {
        self.tagged().is_some() || self.sequence().is_some()
    }
}
impl ChainMetadata for CInputWireChain {
    fn optional(&self) -> Option<&COptional> {
        if let Self::Optional(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn sequence(&self) -> Option<CSequence> {
        matches!(self, Self::Sequence).then_some(CSequence::Buffered)
    }
    fn tagged(&self) -> Option<&std::rc::Rc<CTagged>> {
        if let Self::Tagged(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
impl ChainMetadata for COutputWireChain {
    fn optional(&self) -> Option<&COptional> {
        if let Self::Optional(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn sequence(&self) -> Option<CSequence> {
        if let Self::Sequence(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    fn tagged(&self) -> Option<&std::rc::Rc<CTagged>> {
        if let Self::Tagged(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
pub(crate) trait BranchMetadata {
    fn is_values(&self) -> bool;
    fn is_fields(&self) -> bool;
    fn field(&self) -> Option<&syn::Ident>;
    fn enumeration(&self) -> Option<&std::rc::Rc<CEnumGroup>>;
    fn is_alternatives(&self) -> bool;
    fn selects_arm(&self) -> bool {
        self.is_alternatives() || self.enumeration().is_some()
    }
    fn is_field_collection(&self) -> bool {
        self.is_fields()
    }
}
impl BranchMetadata for CInputWireBranch {
    fn is_values(&self) -> bool {
        matches!(self, Self::Values)
    }
    fn is_fields(&self) -> bool {
        matches!(self, Self::Fields)
    }
    fn field(&self) -> Option<&syn::Ident> {
        if let Self::Field(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn enumeration(&self) -> Option<&std::rc::Rc<CEnumGroup>> {
        if let Self::Enum(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn is_alternatives(&self) -> bool {
        matches!(self, Self::Alternatives)
    }
}
impl BranchMetadata for COutputWireBranch {
    fn is_values(&self) -> bool {
        matches!(self, Self::Values)
    }
    fn is_fields(&self) -> bool {
        matches!(self, Self::Fields)
    }
    fn field(&self) -> Option<&syn::Ident> {
        if let Self::Field(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn enumeration(&self) -> Option<&std::rc::Rc<CEnumGroup>> {
        if let Self::Enum(value) = self {
            Some(value)
        } else {
            None
        }
    }
    fn is_alternatives(&self) -> bool {
        false
    }
}
pub(crate) fn validate_selector(wire: &Wire<CWire>) -> Result<()> {
    if !matches!(
        wire.kind,
        CWire::Scalar {
            primitive: Primitive::I32,
            ..
        }
    ) {
        return Err("selector requires the adapter's integer tag representation".into());
    }
    wire.kind.validate()
}
/// How this optional occurrence distinguishes absence from a present payload.
#[derive(Clone, Debug)]
pub enum COptional {
    /// A dedicated 0/1 tag. The binding is also the normalized presence value.
    Flag(syn::Ident),
    /// An excluded representation in one payload value.
    Niche {
        /// Normalized presence local used by conversion conditions.
        presence: syn::Ident,
        /// Payload binding and reserved absent representation.
        reservation: prebindgen_declarations::NicheReservation<CNiche>,
    },
}
impl COptional {
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
    pub fn niche(&self) -> Option<&CNiche> {
        match self {
            Self::Flag(_) => None,
            Self::Niche { reservation, .. } => Some(&reservation.niche),
        }
    }
    pub(crate) fn from_parts(
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<CNiche>,
        controls: &[Wire<CWire>],
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
pub(crate) fn values<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    fn visit<'a, Chain: group::ChainMetadata, Branch>(
        node: &'a CWireNode<Chain, Branch>,
        out: &mut Vec<&'a Wire<CWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta,
                controls,
                child,
            } => {
                out.extend(controls);
                if !meta.is_scope_boundary() {
                    visit(child, out);
                }
            }
            WireNode::Branch {
                controls, children, ..
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
pub(crate) fn values_mut<Chain: group::ChainMetadata, Branch>(
    node: &mut CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&mut Wire<CWire>> {
    fn visit<'a, Chain: group::ChainMetadata, Branch>(
        node: &'a mut CWireNode<Chain, Branch>,
        out: &mut Vec<&'a mut Wire<CWire>>,
    ) {
        match node {
            WireNode::Leaf(wires) => out.extend(wires),
            WireNode::Chain {
                meta,
                controls,
                child,
            } => {
                out.extend(controls);
                if !meta.is_scope_boundary() {
                    visit(child, out);
                }
            }
            WireNode::Branch {
                controls, children, ..
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
pub(crate) fn validate_layout<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
    node: &CWireNode<Chain, Branch>,
) -> Result<()> {
    prebindgen_conversions::validate_wire_names(prebindgen_conversions::wire_definitions(
        std::iter::once(node),
    ))?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
        node: &CWireNode<Chain, Branch>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<CNiche>>,
    ) -> Result<()> {
        match node {
            WireNode::Leaf(wires) => {
                for w in wires {
                    if matches!(w.kind, CWire::Rows) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Branch {
                meta: encoding,
                children: alternatives,
                controls,
                ..
            } if (encoding.is_alternatives() || encoding.enumeration().is_some()) => {
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
                meta: encoding,
                controls,
                child,
            } => {
                if let Some(value) = encoding.tagged() {
                    if controls.len() != 1
                        || !matches!(&controls[0].kind,CWire::Tagged{value:v,..} if v.name==value.name)
                    {
                        return Err("tagged carrier requires matching control".into());
                    }
                    visit(child, bindings, false, reserved)?;
                } else if let Some(metadata) = encoding.sequence() {
                    let element = child;
                    let [carrier] = controls.as_slice() else {
                        return Err("sequence requires one carrier control".into());
                    };
                    let valid = match metadata {
                        CSequence::Buffered => matches!(carrier.kind, CWire::Rows),
                        CSequence::Callback => matches!(
                            carrier.kind,
                            CWire::Scalar {
                                primitive: Primitive::Usize,
                                ..
                            }
                        ),
                    };
                    if !valid {
                        return Err(
                            "sequence carrier control is incompatible with its delivery".into()
                        );
                    }
                    if values(element).len() == 0 {
                        return Err("sequence requires a nonempty element layout".into());
                    }
                    crate::sequence::validate_buffered(element)?;
                    validate_layout(element)?;
                } else if let Some(o) = encoding.optional() {
                    let element = child;
                    match o {
                        COptional::Flag(presence) => {
                            let [flag] = controls.as_slice() else {
                                return Err("flag optional requires one control".into());
                            };
                            validate_selector(flag)?;
                            if &flag.name != presence {
                                return Err(
                                    "optional flag reference does not match its control".into()
                                );
                            }
                        }
                        COptional::Niche { .. } if !controls.is_empty() => {
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
                                "available C niche is already reserved by an optional node".into(),
                            );
                        }
                        let values = reserved.entry(o.discriminator().to_string()).or_default();
                        if values.contains(niche) {
                            return Err("duplicate C niche reservation on one wire".into());
                        }
                        values.push(niche.clone());
                    }
                    visit(element, bindings, false, reserved)?;
                } else {
                    return Err("unsupported chain metadata".into());
                }
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
                if metadata.field().is_some() && !in_fields {
                    return Err("named field must be a direct child of a field collection".into());
                }
                if metadata.is_field_collection() {
                    let mut fields = std::collections::BTreeSet::new();
                    for child in children {
                        let WireNode::Branch { meta, .. } = child else {
                            return Err("field collection requires named field children".into());
                        };
                        let Some(name) = meta.field() else {
                            return Err("field collection requires named field children".into());
                        };
                        if !fields.insert(name.unraw().to_string()) {
                            return Err(format!("duplicate field `{name}`"));
                        }
                    }
                }

                for child in children {
                    visit(child, bindings, metadata.is_fields(), reserved)?;
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
pub(crate) fn argument_parameters<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    'a,
    I: prebindgen_conversions::InputConversion<
        Wire = CWire,
        Chain = CInputWireChain,
        Branch = CInputWireBranch,
    >,
    O: prebindgen_conversions::Conversion,
    M,
>(
    f: &'a prebindgen_conversions::BoundFunction<'_, I, O, M>,
) -> Result<std::vec::IntoIter<&'a Wire<CWire>>> {
    let mut out = vec![];
    for input in f.input_conversions()? {
        out.extend(argument_parameters(input.input_node()));
    }
    Ok(out.into_iter())
}
pub(crate) fn result_packet_fields<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn result_packet<'a, Chain: group::ChainMetadata, Branch>(
    node: &'a CWireNode<Chain, Branch>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn row_values<'a, Chain: group::ChainMetadata, Branch>(
    node: &'a CWireNode<Chain, Branch>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn prepare_optional<
    Chain: group::ChainMetadata,
    Branch,
    V: prebindgen_conversions::Conversion<Wire = CWire, Chain = Chain, Branch = Branch>,
>(
    request: prebindgen_declarations::Optional<'_, CCodec, V>,
    direction: prebindgen_declarations::Direction,
    wrap: impl FnOnce(COptional) -> Chain,
) -> Result<OptionalEncoding<CWire, Chain>> {
    let prepared = prebindgen_declarations::prepare_optional_payload(
        values_mut(request.payload),
        direction,
        CWire::available_niche,
        CWire::set_niche,
        |wire, niche, value| {
            wire.validate_niche(niche)?;
            Ok(niche.present(value))
        },
        CWire::remaining_niche,
        wire::absent_output_value,
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
            kind: CWire::scalar(Primitive::I32),
        };
        (Presence::Flag(flag.clone()), vec![flag])
    };
    let metadata = COptional::from_parts(
        request.presence,
        discriminator,
        prepared.reservation.map(|r| r.niche),
        &controls,
    )
    .map(wrap)?;
    Ok(OptionalEncoding {
        presence,
        metadata,
        absent_values: prepared.absent_values,
    })
}
pub(crate) fn retire_niches<Chain: group::ChainMetadata, Branch>(
    node: &mut CWireNode<Chain, Branch>,
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(
        values_mut(node),
        reservation,
        CWire::remaining_niche,
        CWire::set_niche,
    )
}

pub(crate) fn record_fields<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selector(name: syn::Ident) -> prebindgen_conversions::Wire<CWire> {
        prebindgen_conversions::Wire {
            name,
            kind: CWire::scalar(Primitive::I32),
        }
    }
    fn enclose_optional_metadata(tree: &mut CInputWireNode, metadata: COptional) {
        let mut payload = std::mem::take(tree);
        let controls = if let COptional::Flag(name) = &metadata {
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
            meta: CInputWireChain::Optional(metadata),
        };
    }
    fn leaf(name: &str) -> prebindgen_conversions::Wire<CWire> {
        prebindgen_conversions::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: CWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: bool) -> COptional {
        match niche.then_some(CNiche::Integer(0x110000)) {
            Some(niche) => COptional::Niche {
                presence: syn::parse_str(presence).unwrap(),
                reservation: prebindgen_declarations::NicheReservation {
                    discriminator: syn::parse_str(discriminator).unwrap(),
                    niche,
                },
            },
            None => COptional::Flag(syn::parse_str(discriminator).unwrap()),
        }
    }
    #[test]
    fn optional_construction_rejects_inconsistent_flag_and_niche_parts() {
        let flag = selector(syn::parse_quote!(flag));
        assert!(
            COptional::from_parts(syn::parse_quote!(flag), syn::parse_quote!(flag), None, &[])
                .is_err()
        );
        assert!(COptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag.clone(), flag.clone()]
        )
        .is_err());
        assert!(COptional::from_parts(
            syn::parse_quote!(other),
            syn::parse_quote!(flag),
            None,
            std::slice::from_ref(&flag)
        )
        .is_err());
        assert!(COptional::from_parts(
            syn::parse_quote!(present),
            syn::parse_quote!(payload),
            Some(CNiche::Integer(-1)),
            std::slice::from_ref(&flag)
        )
        .is_err());
        let encoding = COptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag],
        )
        .unwrap();
        assert!(matches!(encoding, COptional::Flag(_)));
        assert_eq!(encoding.presence(), encoding.discriminator());
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        assert!(validate_selector(&leaf("tag"))
            .unwrap_err()
            .contains("integer tag"));
        assert!(COptional::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[leaf("flag")]
        )
        .unwrap_err()
        .contains("integer tag"));
        let sequence: CInputWireNode = WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: CInputWireChain::Sequence,
        };
        validate_layout(&sequence).unwrap();
        let empty: CInputWireNode = WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            child: Box::new(WireNode::default()),
            meta: CInputWireChain::Sequence,
        };
        assert!(validate_layout(&empty)
            .unwrap_err()
            .contains("nonempty element"));
        let choice: CInputWireNode = WireNode::Branch {
            controls: vec![selector(syn::parse_quote!(tag))],
            children: vec![WireNode::default()],
            meta: CInputWireBranch::Alternatives,
        };
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn malformed_tree_controls_are_rejected() {
        let mut tree: CInputWireNode = WireNode::Branch {
            controls: vec![],
            children: vec![WireNode::default()],
            meta: CInputWireBranch::Alternatives,
        };
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("one selector control"));
        let WireNode::Branch {
            meta: CInputWireBranch::Alternatives,
            controls,
            ..
        } = &mut tree
        else {
            panic!()
        };
        controls.push(leaf("wrong_kind"));
        assert!(validate_layout(&tree).unwrap_err().contains("integer tag"));
        let empty_sequence: CInputWireNode = WireNode::Chain {
            controls: vec![],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: CInputWireChain::Sequence,
        };
        assert!(validate_layout(&empty_sequence)
            .unwrap_err()
            .contains("one carrier control"));
        let flag: CInputWireNode = WireNode::Chain {
            controls: vec![selector(syn::parse_quote!(actual))],
            child: Box::new(WireNode::default()),
            meta: CInputWireChain::Optional(COptional::Flag(syn::parse_quote!(missing))),
        };
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("does not match"));
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree: CInputWireNode = prebindgen_conversions::WireNode::Leaf(vec![leaf("value")]);
        prebindgen_conversions::wire_definitions_mut(std::iter::once(&mut tree))
            .next()
            .expect("logical wire index out of range")
            .kind
            .set_niche(Some(CNiche::Integer(0x110002)))
            .unwrap();
        enclose_optional_metadata(&mut tree, optional("inner", "value", true));
        let mut outer = optional("outer", "value", true);
        let o = &mut outer;
        let COptional::Niche { reservation, .. } = o else {
            panic!()
        };
        reservation.niche = CNiche::Integer(0x110001);
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
        let mut flag: CInputWireNode = prebindgen_conversions::WireNode::Leaf(vec![leaf("flag")]);
        crate::group::values_mut(&mut flag).next().unwrap().kind = CWire::scalar(Primitive::I32);
        enclose_optional_metadata(&mut flag, optional("flag", "flag", false));
        validate_layout(&flag).unwrap();
        assert!(COptional::from_parts(
            syn::parse_quote!(outer),
            syn::parse_quote!(flag),
            None,
            &[selector(syn::parse_quote!(flag))]
        )
        .unwrap_err()
        .contains("one matching flag"));
    }
    #[test]
    fn every_niche_reservation_is_checked_for_compatibility_and_reuse() {
        for (kind, niche) in [
            (CWire::scalar(Primitive::Char), CNiche::Integer(0x110000)),
            (
                CWire::pointer(CPointer {
                    ty: syn::parse_quote!(Widget),
                    access: Access::Move,
                }),
                CNiche::Null,
            ),
        ] {
            let mut tree: CInputWireNode =
                prebindgen_conversions::WireNode::Leaf(vec![GenericWire {
                    name: syn::parse_quote!(value),
                    kind,
                }]);
            let group = |presence: &str, niche: CNiche| {
                COptional::from_parts(
                    syn::parse_str(presence).unwrap(),
                    syn::parse_quote!(value),
                    Some(niche),
                    &[],
                )
                .unwrap()
            };
            enclose_optional_metadata(&mut tree, group("inner", niche.clone()));
            assert!(validate_layout(&tree)
                .unwrap_err()
                .contains("already reserved"));
            prebindgen_conversions::wire_definitions_mut(std::iter::once(&mut tree))
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
                    access: Access::Shared,
                }),
                CNiche::Integer(0),
            ),
        ] {
            let mut tree: CInputWireNode =
                prebindgen_conversions::WireNode::Leaf(vec![GenericWire {
                    name: syn::parse_quote!(value),
                    kind,
                }]);
            enclose_optional_metadata(
                &mut tree,
                COptional::from_parts(
                    syn::parse_quote!(present),
                    syn::parse_quote!(value),
                    Some(niche),
                    &[],
                )
                .unwrap(),
            );
            assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        }
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let field: CInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Field(syn::parse_quote!(x)),
            children: vec![WireNode::default()],
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates: CInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Fields,
            children: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record: CInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Fields,
            children: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed: CInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Fields,
            children: vec![WireNode::Leaf(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty: CInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Fields,
            children: vec![],
        };
        validate_layout(&empty).unwrap();
    }
    #[test]
    fn malformed_metadata_is_rejected_before_final_slots_are_installed() {
        let mut input = CCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &CBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        input.node = prebindgen_conversions::WireNode::Branch {
            controls: vec![],
            meta: CInputWireBranch::Field(syn::parse_quote!(orphan)),
            children: vec![std::mem::take(&mut input.node)],
        };
        assert!(CCodec::raw()
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("direct child"));
        let mut output = CCodec::raw()
            .output_leaf(prebindgen_declarations::OutputLeaf::new(
                &CBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                prebindgen_declarations::ValueAccess::Owned,
            ))
            .unwrap();
        output.outcomes.success = prebindgen_conversions::WireNode::Branch {
            controls: vec![],
            meta: COutputWireBranch::Field(syn::parse_quote!(orphan)),
            children: vec![std::mem::take(&mut output.outcomes.success)],
        };
        assert!(CCodec::raw()
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
            let mut plan = CCodec::raw()
                .input_leaf(prebindgen_declarations::InputLeaf::new(
                    &CBindings::parse([]).unwrap(),
                    &syn::parse_quote!(i64),
                    &(),
                    syn::parse_quote!(wire),
                    syn::parse_quote!(value),
                    &prebindgen_declarations::Condition::default(),
                ))
                .unwrap();
            crate::group::values_mut(&mut plan.node)
                .next()
                .unwrap()
                .kind = wire.clone();
            assert!(CCodec::raw()
                .finish_input(&mut plan, &())
                .unwrap_err()
                .contains("incompatible"));
            let mut output = CCodec::raw()
                .output_leaf(prebindgen_declarations::OutputLeaf::new(
                    &CBindings::parse([]).unwrap(),
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
                .kind = wire;
            assert!(CCodec::raw()
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
        std::rc::Rc::make_mut(mapping)
            .variants
            .push((syn::parse_quote!(D), 2));
        wire.clear_niche();
        assert!(wire.validate().is_err());
    }
}

#[cfg(test)]
mod abi_tests {
    use super::*;
    #[test]
    fn function_parameters_result_packets_and_sequence_fields_have_explicit_roles() {
        let scalar = |name: &str| prebindgen_conversions::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: CWire::scalar(Primitive::I32),
        };
        let element: CInputWireNode =
            prebindgen_conversions::WireNode::Leaf(vec![scalar("first"), scalar("second")]);
        let mut sequence = element.clone();
        sequence = prebindgen_conversions::WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            meta: CInputWireChain::Sequence,
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
        let mut flat: CBindings =
            Flat::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let mut input = CCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &CBindings::parse([]).unwrap(),
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(raw),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        input.node = sequence;
        bindings::set_input(&mut flat, "f", "value", input.clone()).unwrap();
        let output = prebindgen_declarations::compile_output(
            &flat,
            &CCodec::raw(),
            "f",
            &COutput::None,
            &(),
        )
        .unwrap();
        bindings::set_output(&mut flat, "f", output).unwrap();
        assert_eq!(
            argument_parameters(input.input_node())
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
        let node: CInputWireNode = WireNode::Branch {
            meta: CInputWireBranch::Alternatives,
            children: vec![],
            controls: vec![],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("no alternatives"));
        let node: CInputWireNode = WireNode::Branch {
            meta: CInputWireBranch::Values,
            children: vec![],
            controls: vec![Wire {
                name: syn::parse_quote!(extra),
                kind: CWire::scalar(Primitive::I32),
            }],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("control wires"));
    }
}

use prebindgen_conversions::FunctionConversions as _;
