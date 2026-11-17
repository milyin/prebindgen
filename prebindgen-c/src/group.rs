//! C adapter metadata and interpretation of logical value trees.
use super::*;

use prebindgen_conversions::{Wire, WireNode};
use syn::ext::IdentExt;

/// Interpret exactly one child subtree while decoding C values into Rust.
/// Input includes a callback's return value; callback arguments use output metadata.
///
/// Output delivery cannot be selected on an input:
/// ```compile_fail
/// use prebindgen_c::{CInputWireContainer, CSequence};
/// let input = CInputWireContainer::Sequence(CSequence::Callback);
/// ```
/// Annotate one subtree without adding wires or changing its boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum CInputWireAnnotation {
    /// Name one source field's conversion subtree, directly inside Fields.
    /// The identifier names a Rust field or an enum tuple field's generated property.
    /// Adds no controls or native scope; the child may produce several ABI values.
    Field(syn::Ident),
}
/// Add optional presence controls to one subtree at the same boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum CInputWireChain {
    /// Decode an optional child payload. A flag lives in controls; a niche instead
    /// reserves one representation of a child wire, without a separate control.
    Optional(COptional),
}
/// Expose only the container wires; interpret its child in an internal boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum CInputWireContainer {
    /// Unpack the tagged C carrier in wires into the child's enum tag and
    /// payload wires. The child has its own native wire scope, independent of
    /// whether the carrier itself is inline or passed by pointer.
    Tagged(std::rc::Rc<crate::CTagged>),
    /// Decode a collection by repeating the child's element conversion.
    /// Wires contain one pointer/length row carrier; each row has its own
    /// native wire scope. The caller keeps the row storage live while it is read.
    Sequence,
}
/// Interpret exactly one child subtree while encoding Rust values for C.
/// Output includes arguments delivered to a callback; its return uses input metadata.
/// Annotate one subtree without adding wires or changing its boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum COutputWireAnnotation {
    /// Name one source field's conversion subtree, directly inside Fields.
    /// Adds no control wires or native scope; the child may produce several outputs.
    Field(syn::Ident),
}
/// Add optional presence controls to one subtree at the same boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum COutputWireChain {
    /// Encode an optional child payload. A flag lives in controls; a niche instead
    /// uses a reserved representation of a child wire, without a separate control.
    Optional(COptional),
}
/// Expose only the container wires; interpret its child in an internal boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum COutputWireContainer {
    /// Pack the child's enum tag and payload wires into the tagged C carrier in
    /// wires. The child has its own native wire scope; carrier metadata chooses
    /// whether C receives inline storage or a pointer to storage.
    Tagged(std::rc::Rc<crate::CTagged>),
    /// Repeat the child's element conversion using the selected delivery policy.
    /// Wires carry the buffered rows or callback-delivered count; each row has
    /// its own wire scope. Buffered output owns the generated row storage.
    Sequence(CSequence),
}
/// How C input components reconstruct a Rust value.
/// Field names annotate one subtree, so they belong to annotation metadata:
/// ```compile_fail
/// use prebindgen_c::CInputWireBranch;
/// let field = CInputWireBranch::Field(syn::parse_quote!(x));
/// ```
#[derive(Clone, Debug)]
pub enum CInputWireBranch {
    /// Decode every child in order, such as a constructor's arguments.
    /// Accepts an empty list; adds no controls, names, or native scope.
    Values,
    /// Decode source fields in declaration order. Every child is a Field annotation;
    /// empty records have no children. Adds no controls or native scope.
    Fields,
    /// Use the integer selector in controls to decode exactly one constructor arm.
    /// The nonempty child list is in selector order; inactive arms are not consumed.
    /// Constructors may produce the same source variant; this is not a source enum.
    Alternatives,
    /// Decode exactly one source enum variant. The control tag selects a child
    /// payload in wire-tag order. Metadata links these arms to the canonical source
    /// enum; an enclosing Tagged container may supply a single C carrier for all arms.
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
    /// Deliver every child in order, such as multiple projections of one Rust value.
    /// Accepts an empty list; adds no controls, names, or native scope.
    Values,
    /// Encode source fields in declaration order. Every child is a Field annotation;
    /// empty records have no children. Adds no controls or native scope.
    Fields,
    /// Encode the active source enum variant. The control tag selects a child
    /// payload in wire-tag order; inactive children contribute their absent wire
    /// values. An enclosing Tagged container may pack the result into one C carrier.
    Enum(std::rc::Rc<crate::CEnumGroup>),
}

// Private queries used by shared wire-scope, selection and layout algorithms.
pub(crate) trait AnnotationMetadata {
    fn field(&self) -> Option<&syn::Ident>;
}
impl AnnotationMetadata for CInputWireAnnotation {
    fn field(&self) -> Option<&syn::Ident> {
        let Self::Field(name) = self;
        Some(name)
    }
}
impl AnnotationMetadata for COutputWireAnnotation {
    fn field(&self) -> Option<&syn::Ident> {
        let Self::Field(name) = self;
        Some(name)
    }
}
pub(crate) trait ChainMetadata {
    fn optional(&self) -> Option<&COptional>;
}
impl ChainMetadata for CInputWireChain {
    fn optional(&self) -> Option<&COptional> {
        let Self::Optional(value) = self;
        Some(value)
    }
}
impl ChainMetadata for COutputWireChain {
    fn optional(&self) -> Option<&COptional> {
        let Self::Optional(value) = self;
        Some(value)
    }
}
pub(crate) trait ContainerMetadata {
    fn sequence(&self) -> Option<CSequence>;
    fn tagged(&self) -> Option<&std::rc::Rc<CTagged>>;
}
impl ContainerMetadata for CInputWireContainer {
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
impl ContainerMetadata for COutputWireContainer {
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
pub(crate) fn values<Chain, Branch, Annotation, Container>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    prebindgen_conversions::boundary_wires(std::iter::once(node))
        .collect::<Vec<_>>()
        .into_iter()
}
pub(crate) fn values_mut<Chain, Branch, Annotation, Container>(
    node: &mut CWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&mut Wire<CWire>> {
    prebindgen_conversions::boundary_wires_mut(std::iter::once(node))
        .collect::<Vec<_>>()
        .into_iter()
}
pub(crate) fn validate_layout<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
) -> Result<()> {
    prebindgen_conversions::validate_wire_names(prebindgen_conversions::wire_definitions(
        std::iter::once(node),
    ))?;
    for wire in values(node) {
        wire.kind.validate()?;
    }
    fn visit<
        Chain: group::ChainMetadata,
        Branch: group::BranchMetadata,
        Annotation: group::AnnotationMetadata,
        Container: group::ContainerMetadata,
    >(
        node: &CWireNode<Chain, Branch, Annotation, Container>,
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
            WireNode::Annotation { child, .. } => {
                if !in_fields {
                    return Err("named field must be a direct child of a field collection".into());
                }
                visit(child, bindings, false, reserved)?;
            }
            WireNode::Container {
                meta: encoding,
                wires: controls,
                child,
            } => {
                if let Some(value) = encoding.tagged() {
                    if controls.len() != 1
                        || !matches!(&controls[0].kind,CWire::Tagged{value:v,..} if v.name==value.name)
                    {
                        return Err("tagged carrier requires matching wire".into());
                    }
                    visit(child, bindings, false, reserved)?;
                } else if let Some(metadata) = encoding.sequence() {
                    let element = child;
                    let [carrier] = controls.as_slice() else {
                        return Err("sequence requires one carrier wire".into());
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
                            "sequence carrier wire is incompatible with its delivery".into()
                        );
                    }
                    if values(element).len() == 0 {
                        return Err("sequence requires a nonempty element layout".into());
                    }
                    crate::sequence::validate_buffered(element)?;
                    validate_layout(element)?;
                } else {
                    return Err("unsupported container metadata".into());
                }
            }
            WireNode::Chain {
                meta: encoding,
                controls,
                child,
            } => {
                let o = encoding.optional().expect("optional chain");
                let element = child;
                match o {
                    COptional::Flag(presence) => {
                        let [flag] = controls.as_slice() else {
                            return Err("flag optional requires one control".into());
                        };
                        validate_selector(flag)?;
                        if &flag.name != presence {
                            return Err("optional flag reference does not match its control".into());
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
                            "available C niche is already reserved by an optional node".into()
                        );
                    }
                    let values = reserved.entry(o.discriminator().to_string()).or_default();
                    if values.contains(niche) {
                        return Err("duplicate C niche reservation on one wire".into());
                    }
                    values.push(niche.clone());
                }
                visit(element, bindings, false, reserved)?;
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
                if metadata.is_field_collection() {
                    let mut fields = std::collections::BTreeSet::new();
                    for child in children {
                        let WireNode::Annotation { meta, .. } = child else {
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
pub(crate) fn argument_parameters<Chain, Branch, Annotation, Container>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    'a,
    I: prebindgen_conversions::InputConversion<
        Wire = CWire,
        Chain = CInputWireChain,
        Branch = CInputWireBranch,
        Annotation = CInputWireAnnotation,
        Container = CInputWireContainer,
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
pub(crate) fn result_packet_fields<Chain, Branch, Annotation, Container>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn result_packet<'a, Chain, Branch, Annotation, Container>(
    node: &'a CWireNode<Chain, Branch, Annotation, Container>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields<Chain, Branch, Annotation, Container>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<CWire>> {
    values(node)
}
pub(crate) fn row_values<'a, Chain, Branch, Annotation, Container>(
    node: &'a CWireNode<Chain, Branch, Annotation, Container>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<CWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters<Chain, Branch, Annotation, Container>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
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
pub(crate) fn retire_niches<
    Chain: group::ChainMetadata,
    Branch,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &mut CWireNode<Chain, Branch, Annotation, Container>,
    reservation: Option<&prebindgen_declarations::NicheReservation<CNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(
        values_mut(node),
        reservation,
        CWire::remaining_niche,
        CWire::set_niche,
    )
}

pub(crate) fn record_fields<
    Chain: group::ChainMetadata,
    Branch,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &CWireNode<Chain, Branch, Annotation, Container>,
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
        let sequence: CInputWireNode = WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: CInputWireContainer::Sequence,
        };
        validate_layout(&sequence).unwrap();
        let empty: CInputWireNode = WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            child: Box::new(WireNode::default()),
            meta: CInputWireContainer::Sequence,
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
        let empty_sequence: CInputWireNode = WireNode::Container {
            wires: vec![],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: CInputWireContainer::Sequence,
        };
        assert!(validate_layout(&empty_sequence)
            .unwrap_err()
            .contains("one carrier wire"));
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
        let field: CInputWireNode = WireNode::Annotation {
            meta: CInputWireAnnotation::Field(syn::parse_quote!(x)),
            child: Box::new(WireNode::default()),
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
        input.node = prebindgen_conversions::WireNode::Annotation {
            meta: CInputWireAnnotation::Field(syn::parse_quote!(orphan)),
            child: Box::new(std::mem::take(&mut input.node)),
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
        output.outcomes.success = prebindgen_conversions::WireNode::Annotation {
            meta: COutputWireAnnotation::Field(syn::parse_quote!(orphan)),
            child: Box::new(std::mem::take(&mut output.outcomes.success)),
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
        sequence = prebindgen_conversions::WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: CWire::Rows,
            }],
            meta: CInputWireContainer::Sequence,
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
