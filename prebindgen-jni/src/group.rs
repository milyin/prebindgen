//! JNI adapter metadata and interpretation of logical value trees.
use super::*;

use prebindgen_conversions::{Wire, WireNode};
use syn::ext::IdentExt;

/// Interpret exactly one child subtree while decoding Kotlin/JNI values into Rust.
/// Input includes a callback's return value; callback arguments use output metadata.
///
/// Output delivery cannot be selected on an input:
/// ```compile_fail
/// use prebindgen_jni::JniInputWireContainer;
/// let input = JniInputWireContainer::KCallbackRows;
/// ```
/// Annotate one subtree without adding wires or changing its boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniInputWireAnnotation {
    /// Name one source argument's conversion subtree for exposed Kotlin parameter paths.
    /// Adds no control wires and preserves the child's native representation and scope.
    RParam(String),
    /// Name one source field's conversion subtree, directly inside RFields or KDataClass.
    /// The identifier names a Rust field or an enum tuple field's generated property.
    /// Adds no control wires; the child may produce zero, one or several wire values.
    RField(syn::Ident),
}
/// Add optional presence controls to one subtree at the same boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniInputWireChain {
    /// Decode an optional child payload. A flag lives in controls; a niche instead
    /// reserves one representation of a child wire. Neither starts a new native scope.
    ROption(JniPresence),
}
/// Expose only the container wires; interpret its child in an internal boundary.
/// Input is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniInputWireContainer {
    /// Decode a collection by repeating the child's element conversion.
    /// The wires declare an object-array carrier, or a native offset/count range
    /// inside BulkFrame. The child describes one row in its own wire scope.
    KList,
    /// Decode the whole child subtree from one packed C-representation frame.
    /// Wires carry the byte buffer and its length; the child defines the frame's
    /// layout and native wire scope. Nested sequences use native rows in that frame.
    BulkFrame,
}
/// Interpret exactly one child subtree while encoding Rust values for Kotlin/JNI.
/// Output includes arguments delivered to a callback; its return uses input metadata.
/// Annotate one subtree without adding wires or changing its boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniOutputWireAnnotation {
    /// Name one source field's conversion subtree, directly inside RFields or KDataClass.
    /// Adds no control wires or native scope; the child may produce several outputs.
    RField(syn::Ident),
    /// Supply one base name for Kotlin callback, build and fold params. Multiple
    /// child results derive numeric suffixes. Adds no controls or native scope.
    KParamName(String),
}
/// Add optional presence controls to one subtree at the same boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniOutputWireChain {
    /// Encode an optional child payload. A flag lives in controls; a niche instead
    /// uses a reserved representation of a child wire, without a separate control.
    ROption(JniPresence),
}
/// Expose only the container wires; interpret its child in an internal boundary.
/// Output is relative to Rust; retained callbacks reverse the direction.
#[derive(Clone, Debug)]
pub enum JniOutputWireContainer {
    /// Reconstruct one Kotlin list from the child's repeated element conversion.
    /// The wires declare the array carrier; the child describes one row. The
    /// encoding selects JVM row objects; BulkFrame instead uses a native row range.
    KList(JniRowEncoding),
    /// Deliver the child's repeated elements through the generated Kotlin row
    /// callback. Wires carry the presence marker; each row has its own scope.
    /// User build/fold code runs after JNI returns, outside native handle locks.
    KCallbackRows,
    /// Pack the whole child subtree in one C-representation frame, carried by the
    /// wire byte buffer. The child defines the frame's layout and native scope.
    /// Callback sequences may have bulk elements but cannot be enclosed by BulkFrame.
    BulkFrame,
}
/// How Kotlin/JNI input components reconstruct a Rust value.
///
/// Input has no named output projection:
/// ```compile_fail
/// use prebindgen_jni::JniInputWireAnnotation;
/// let input = JniInputWireAnnotation::KParamName("size".into());
/// ```
/// RField names annotate one subtree, so they belong to annotation metadata:
/// ```compile_fail
/// use prebindgen_jni::JniInputWireBranch;
/// let field = JniInputWireBranch::RField(syn::parse_quote!(x));
/// ```
#[derive(Clone, Debug)]
pub enum JniInputWireBranch {
    /// Decode every child in order, such as a constructor's arguments.
    /// Accepts an empty list; adds no controls, names, or native scope.
    KValues,
    /// Decode Rust struct or enum-variant fields in declaration order. Every child
    /// is an RField annotation; unit/empty groups have no children. Adds no controls or scope.
    RFields,
    /// Accept one Kotlin data-class value and extract its properties before JNI.
    /// Children are RField annotations in source order, as for RFields. Metadata identifies
    /// the canonical source type and Kotlin binding; no native carrier is added.
    KDataClass(JniInputKDataClass),
    /// Use the integer selector in controls to decode exactly one conversion arm,
    /// including constructor and identity alternatives.
    /// The nonempty child list is in selector order; inactive arms are not consumed.
    /// Constructors may produce the same source variant; this is not a source enum.
    RChoices,
    /// Read one Kotlin sealed value and decode its selected Rust enum variant.
    /// The control tag selects the child payload in wire-tag order; metadata links
    /// the canonical enum and the Kotlin parameter/local holding its value.
    KSealed(std::rc::Rc<crate::sealed::JniInputKSealed>),
    /// Decode a Rust enum from separate Kotlin tag and payload parameters.
    /// The control tag selects one child payload in wire-tag order.
    KEnumParams {
        /// Canonical Rust enum in Flat, including its ordered variant definitions.
        r_type: prebindgen_flat::TypeRef,
    },
}
/// How Rust output components become Kotlin values.
///
/// Output branches have no constructor alternative operation:
/// ```compile_fail
/// use prebindgen_jni::JniOutputWireBranch;
/// let output = JniOutputWireBranch::RChoices;
/// ```
#[derive(Clone, Debug)]
pub enum JniOutputWireBranch {
    /// Deliver every child in order, such as multiple projections of one Rust value.
    /// Accepts an empty list; adds no controls, names, or native scope.
    KValues,
    /// Encode Rust struct or enum-variant fields in declaration order. Every child
    /// is an RField annotation; unit/empty groups have no children. Adds no controls or scope.
    RFields,
    /// Reconstruct one Kotlin data-class value from the RField children in source
    /// order. Metadata identifies the canonical type; reconstruction happens on
    /// Kotlin's side without adding a native object carrier or wire scope.
    /// Output labels belong to KParamName annotations; no input binding is needed here.
    KDataClass(JniOutputKDataClass),
    /// Reconstruct one Kotlin sealed value from the active Rust enum variant.
    /// The control tag selects a child payload in wire-tag order; inactive
    /// children contribute their absent wire values. No Kotlin input name is kept.
    KSealed(std::rc::Rc<crate::sealed::JniOutputKSealed>),
    /// Expose the Rust enum as separate Kotlin tag and payload parameters.
    /// The control tag selects a child payload in wire-tag order.
    KEnumParams {
        /// Canonical Rust enum in Flat, including its ordered variant definitions.
        r_type: prebindgen_flat::TypeRef,
    },
}

// Private queries used by shared wire-scope, selection and layout algorithms.
// The concrete enums above remain the complete input/output operation definitions.
pub(crate) trait AnnotationMetadata {
    fn field(&self) -> Option<&syn::Ident>;
    fn output_name(&self) -> Option<&str>;
}
impl AnnotationMetadata for JniInputWireAnnotation {
    fn field(&self) -> Option<&syn::Ident> {
        if let Self::RField(name) = self {
            Some(name)
        } else {
            None
        }
    }
    fn output_name(&self) -> Option<&str> {
        None
    }
}
impl AnnotationMetadata for JniOutputWireAnnotation {
    fn field(&self) -> Option<&syn::Ident> {
        if let Self::RField(name) = self {
            Some(name)
        } else {
            None
        }
    }
    fn output_name(&self) -> Option<&str> {
        if let Self::KParamName(name) = self {
            Some(name)
        } else {
            None
        }
    }
}
pub(crate) trait ChainMetadata {
    fn optional(&self) -> Option<&JniPresence>;
}
impl ChainMetadata for JniInputWireChain {
    fn optional(&self) -> Option<&JniPresence> {
        let Self::ROption(value) = self;
        Some(value)
    }
}
impl ChainMetadata for JniOutputWireChain {
    fn optional(&self) -> Option<&JniPresence> {
        let Self::ROption(value) = self;
        Some(value)
    }
}
pub(crate) trait ContainerMetadata {
    fn list_encoding(&self) -> Option<JniRowEncoding>;
    fn is_callback_rows(&self) -> bool;
    fn is_sequence(&self) -> bool {
        self.list_encoding().is_some() || self.is_callback_rows()
    }
    fn is_bulk(&self) -> bool;
}
impl ContainerMetadata for JniInputWireContainer {
    fn list_encoding(&self) -> Option<JniRowEncoding> {
        matches!(self, Self::KList).then_some(JniRowEncoding::ObjectArray)
    }
    fn is_callback_rows(&self) -> bool {
        false
    }
    fn is_bulk(&self) -> bool {
        matches!(self, Self::BulkFrame)
    }
}
impl ContainerMetadata for JniOutputWireContainer {
    fn list_encoding(&self) -> Option<JniRowEncoding> {
        if let Self::KList(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    fn is_callback_rows(&self) -> bool {
        matches!(self, Self::KCallbackRows)
    }
    fn is_bulk(&self) -> bool {
        matches!(self, Self::BulkFrame)
    }
}
pub(crate) trait BranchMetadata {
    fn is_fields(&self) -> bool;
    fn is_record(&self) -> bool;
    fn record_binding(&self) -> Option<&str>;
    fn r_enum(&self) -> Option<&prebindgen_flat::TypeRef>;
    fn is_alternatives(&self) -> bool;
    fn selects_arm(&self) -> bool {
        self.is_alternatives() || self.r_enum().is_some()
    }
    fn is_field_collection(&self) -> bool {
        self.is_fields() || self.is_record()
    }
}
impl BranchMetadata for JniInputWireBranch {
    fn is_fields(&self) -> bool {
        matches!(self, Self::RFields)
    }
    fn is_record(&self) -> bool {
        matches!(self, Self::KDataClass(_))
    }
    fn record_binding(&self) -> Option<&str> {
        if let Self::KDataClass(value) = self {
            Some(&value.k_name)
        } else {
            None
        }
    }
    fn r_enum(&self) -> Option<&prebindgen_flat::TypeRef> {
        match self {
            Self::KSealed(value) => Some(&value.r_type),
            Self::KEnumParams { r_type } => Some(r_type),
            _ => None,
        }
    }
    fn is_alternatives(&self) -> bool {
        matches!(self, Self::RChoices)
    }
}
impl BranchMetadata for JniOutputWireBranch {
    fn is_fields(&self) -> bool {
        matches!(self, Self::RFields)
    }
    fn is_record(&self) -> bool {
        matches!(self, Self::KDataClass(_))
    }
    fn record_binding(&self) -> Option<&str> {
        None
    }
    fn r_enum(&self) -> Option<&prebindgen_flat::TypeRef> {
        match self {
            Self::KSealed(value) => Some(&value.r_type),
            Self::KEnumParams { r_type } => Some(r_type),
            _ => None,
        }
    }
    fn is_alternatives(&self) -> bool {
        false
    }
}
pub(crate) fn validate_selector(wire: &Wire<JniWire>) -> Result<()> {
    if !matches!(wire.kind, JniWire::SelectorTag { .. }) {
        return Err("selector requires the adapter's integer tag representation".into());
    }
    wire.kind.validate()
}
/// How this optional occurrence distinguishes absence from a present payload.
#[derive(Clone, Debug)]
pub enum JniPresence {
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
impl JniPresence {
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
    pub(crate) fn is_present<
        Chain: group::ChainMetadata,
        Branch,
        Annotation: group::AnnotationMetadata,
        Container: group::ContainerMetadata,
    >(
        &self,
        child: &JniWireNode<Chain, Branch, Annotation, Container>,
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
pub(crate) fn values<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    prebindgen_conversions::boundary_wires(std::iter::once(node))
        .collect::<Vec<_>>()
        .into_iter()
}
pub(crate) fn values_mut<Chain, Branch, Annotation, Container>(
    node: &mut JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&mut Wire<JniWire>> {
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
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> Result<()> {
    validate_layout_in(node, false)
}
pub(crate) fn validate_layout_in<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
    bulk: bool,
) -> Result<()> {
    validate_scope(node, Some(bulk))
}
// Codec hooks also validate fragments before their enclosing scope is attached.
// They can check the physical shape, but cannot require a boundary context yet.
pub(crate) fn validate_fragment<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> Result<()> {
    validate_scope(node, None)
}
fn validate_scope<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
    bulk: Option<bool>,
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
        node: &JniWireNode<Chain, Branch, Annotation, Container>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut std::collections::BTreeMap<String, Vec<JniNiche>>,
        bulk: Option<bool>,
    ) -> Result<()> {
        match node {
            WireNode::Annotation {
                meta: encoding,
                child,
            } => {
                if encoding.field().is_some() && !in_fields {
                    return Err("named field must be a direct child of a field collection".into());
                }
                if let Some(name) = encoding.output_name() {
                    validate_name(name)?;
                }
                visit(child, bindings, false, reserved, bulk)?;
            }
            WireNode::Container {
                meta: encoding,
                wires: controls,
                child,
            } => {
                if encoding.is_bulk() {
                    if bulk == Some(true) {
                        return Err(
                            "nested bulk frames must be lowered into the enclosing bulk scope"
                                .into(),
                        );
                    }
                    if controls.is_empty()
                        || !matches!(controls[0].kind, JniWire::KBulkBytes { .. })
                    {
                        return Err("bulk scope requires a byte-array carrier".into());
                    }
                    visit(child, bindings, in_fields, reserved, Some(true))?;
                } else if encoding.is_sequence() {
                    let element = child;
                    let [carrier] = controls.as_slice() else {
                        return Err("sequence requires one carrier wire".into());
                    };
                    let valid = if encoding.is_callback_rows() {
                        if bulk == Some(true) {
                            return Err(
                                "bulk scopes cannot enclose callback sequence delivery".into()
                            );
                        }
                        matches!(&carrier.kind, JniWire::SelectorTag { .. })
                    } else {
                        match bulk {
                            Some(true) => matches!(carrier.kind, JniWire::FrameRowRange { .. }),
                            Some(false) => matches!(carrier.kind, JniWire::KRowArray { .. }),
                            None => matches!(
                                carrier.kind,
                                JniWire::KRowArray { .. } | JniWire::FrameRowRange { .. }
                            ),
                        }
                    };
                    if !valid {
                        return Err(
                            "sequence carrier wire is incompatible with its delivery".into()
                        );
                    }
                    if matches!(carrier.kind, JniWire::KRowArray { .. })
                        && encoding.list_encoding() == Some(JniRowEncoding::RowClass)
                        && !typed_rows::supported(element)
                    {
                        return Err(
                            "typed output row constructor exceeds 254 JVM parameter slots".into(),
                        );
                    }
                    if values(element).len() == 0
                        && matches!(carrier.kind, JniWire::KRowArray { .. })
                    {
                        return Err("sequence requires a nonempty element layout".into());
                    }
                    crate::sequence::validate_buffered(element)?;
                    validate_scope(element, bulk)?;
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
                    JniPresence::Flag(presence) => {
                        let [flag] = controls.as_slice() else {
                            return Err("flag optional requires one control".into());
                        };
                        validate_selector(flag)?;
                        if &flag.name != presence {
                            return Err("optional flag reference does not match its control".into());
                        }
                    }
                    JniPresence::Niche { .. } if !controls.is_empty() => {
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
            WireNode::Leaf(wires) => {
                for w in wires {
                    if let JniWire::KCallback { callback } = &w.kind {
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
                            JniWire::FrameBytes | JniWire::FrameUtf8 | JniWire::FrameInt128 { .. }
                        )
                    {
                        return Err("native bulk fields require an enclosing bulk scope".into());
                    }
                    if matches!(
                        w.kind,
                        JniWire::KRowArray { .. } | JniWire::FrameRowRange { .. }
                    ) {
                        return Err("array carrier requires a sequence node".into());
                    }
                }
            }
            WireNode::Branch {
                meta: encoding,
                children: alternatives,
                controls,
                ..
            } if (encoding.is_alternatives() || encoding.r_enum().is_some()) => {
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

            WireNode::Branch {
                children,
                meta: metadata,
                controls,
                ..
            } => {
                if !controls.is_empty() {
                    return Err("group metadata does not support control wires".into());
                }
                if let Some(binding) = metadata.record_binding() {
                    if !bindings.insert(binding.into()) {
                        return Err(format!("duplicate Kotlin group binding `{binding}`"));
                    }
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
                    visit(
                        child,
                        bindings,
                        metadata.is_field_collection(),
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
pub(crate) fn argument_parameters<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn function_parameters<
    'a,
    I: prebindgen_conversions::InputConversion<
        Wire = JniWire,
        Chain = JniInputWireChain,
        Branch = JniInputWireBranch,
        Annotation = JniInputWireAnnotation,
        Container = JniInputWireContainer,
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
pub(crate) fn result_packet_fields<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn result_packet<'a, Chain, Branch, Annotation, Container>(
    node: &'a JniWireNode<Chain, Branch, Annotation, Container>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(result_packet_fields(node), bindings)
}
pub(crate) fn row_fields<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn row_values<'a, Chain, Branch, Annotation, Container>(
    node: &'a JniWireNode<Chain, Branch, Annotation, Container>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> Result<Vec<(&'a Wire<JniWire>, &'a TokenStream)>> {
    prebindgen_conversions::match_output_fields(row_fields(node), bindings)
}
pub(crate) fn callback_parameters<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> std::vec::IntoIter<&Wire<JniWire>> {
    values(node)
}
pub(crate) fn prepare_optional<
    Chain: group::ChainMetadata,
    Branch,
    V: prebindgen_conversions::Conversion<Wire = JniWire, Chain = Chain, Branch = Branch>,
>(
    request: prebindgen_declarations::Optional<'_, JniCodec, V>,
    direction: prebindgen_declarations::Direction,
    wrap: impl FnOnce(JniPresence) -> Chain,
) -> Result<OptionalEncoding<JniWire, Chain>> {
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
    let metadata = JniPresence::from_parts(
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
    node: &mut JniWireNode<Chain, Branch, Annotation, Container>,
    reservation: Option<&prebindgen_declarations::NicheReservation<JniNiche>>,
) -> Result<()> {
    prebindgen_declarations::retire_optional_niches(
        values_mut(node),
        reservation,
        optional::remaining,
        JniWire::set_niche,
    )
}
pub(crate) fn kotlin_result_fields<Chain, Branch, Annotation, Container>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
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
    fn enclose_optional_metadata(tree: &mut JniInputWireNode, metadata: JniPresence) {
        let mut payload = std::mem::take(tree);
        let controls = if let JniPresence::Flag(name) = &metadata {
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
            meta: JniInputWireChain::ROption(metadata),
        };
    }
    fn leaf(name: &str) -> prebindgen_conversions::Wire<JniWire> {
        prebindgen_conversions::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: Option<i64>) -> JniPresence {
        match niche.map(JniNiche::Integer) {
            Some(niche) => JniPresence::Niche {
                presence: syn::parse_str(presence).unwrap(),
                reservation: prebindgen_declarations::NicheReservation {
                    discriminator: syn::parse_str(discriminator).unwrap(),
                    niche,
                },
            },
            None => JniPresence::Flag(syn::parse_str(discriminator).unwrap()),
        }
    }
    #[test]
    fn optional_construction_rejects_inconsistent_flag_and_niche_parts() {
        let flag = selector(syn::parse_quote!(flag));
        assert!(JniPresence::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[]
        )
        .is_err());
        assert!(JniPresence::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag.clone(), flag.clone()]
        )
        .is_err());
        assert!(JniPresence::from_parts(
            syn::parse_quote!(other),
            syn::parse_quote!(flag),
            None,
            std::slice::from_ref(&flag)
        )
        .is_err());
        assert!(JniPresence::from_parts(
            syn::parse_quote!(present),
            syn::parse_quote!(payload),
            Some(JniNiche::Integer(-1)),
            std::slice::from_ref(&flag)
        )
        .is_err());
        let encoding = JniPresence::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[flag],
        )
        .unwrap();
        assert!(matches!(encoding, JniPresence::Flag(_)));
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
                meta: JniInputWireBranch::RChoices,
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
    fn outer_optional_cannot_reserve_a_niche_from_container_internals() {
        let mut node: JniInputWireNode = WireNode::Container {
            meta: JniInputWireContainer::BulkFrame,
            wires: vec![Wire {
                name: syn::parse_quote!(bytes),
                kind: JniWire::KBulkBytes {
                    null_niche_available: false,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![selector(syn::parse_quote!(
                internal_tag
            ))])),
        };
        let prepared = prebindgen_declarations::prepare_optional_payload(
            values_mut(&mut node),
            prebindgen_declarations::Direction::Input,
            JniWire::available_niche,
            |w, n| w.set_niche(n),
            |w, n, v| Ok(n.present(w, v)),
            optional::remaining,
            |w, n| crate::optional::absent(w, n, false),
        )
        .unwrap();
        assert!(prepared.reservation.is_none());
        let inner = prebindgen_conversions::wire_definitions(std::iter::once(&node))
            .find(|w| w.name == "internal_tag")
            .unwrap();
        assert_eq!(inner.kind.available_niche(), Some(JniNiche::Integer(-1)));
    }
    #[test]
    fn structural_nodes_require_supported_explicit_representations() {
        assert!(validate_selector(&leaf("tag"))
            .unwrap_err()
            .contains("integer tag"));
        assert!(JniPresence::from_parts(
            syn::parse_quote!(flag),
            syn::parse_quote!(flag),
            None,
            &[leaf("flag")]
        )
        .unwrap_err()
        .contains("integer tag"));
        let sequence: JniInputWireNode = WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::KRowArray {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniInputWireContainer::KList,
        };
        validate_layout(&sequence).unwrap();
        let empty: JniInputWireNode = WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::KRowArray {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::default()),
            meta: JniInputWireContainer::KList,
        };
        assert!(validate_layout(&empty)
            .unwrap_err()
            .contains("nonempty element"));
        let choice: JniInputWireNode = WireNode::Branch {
            controls: vec![selector(syn::parse_quote!(tag))],
            children: vec![WireNode::default()],
            meta: JniInputWireBranch::RChoices,
        };
        validate_layout(&choice).unwrap();
    }
    #[test]
    fn malformed_tree_controls_are_rejected() {
        let mut tree: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            children: vec![WireNode::default()],
            meta: JniInputWireBranch::RChoices,
        };
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("one selector control"));
        let WireNode::Branch {
            meta: JniInputWireBranch::RChoices,
            controls,
            ..
        } = &mut tree
        else {
            panic!()
        };
        controls.push(leaf("wrong_kind"));
        assert!(validate_layout(&tree).unwrap_err().contains("integer tag"));
        let empty_sequence: JniInputWireNode = WireNode::Container {
            wires: vec![],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniInputWireContainer::KList,
        };
        assert!(validate_layout(&empty_sequence)
            .unwrap_err()
            .contains("one carrier wire"));
        let flag: JniInputWireNode = WireNode::Chain {
            controls: vec![selector(syn::parse_quote!(actual))],
            child: Box::new(WireNode::default()),
            meta: JniInputWireChain::ROption(JniPresence::Flag(syn::parse_quote!(missing))),
        };
        assert!(validate_layout(&flag)
            .unwrap_err()
            .contains("does not match"));
    }
    #[test]
    fn callback_control_requires_a_scalar_status() {
        let mut node: JniOutputWireNode = WireNode::Container {
            wires: vec![Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::KRowArray {
                    condition: "false".into(),
                    null_niche_available: true,
                },
            }],
            child: Box::new(WireNode::Leaf(vec![leaf("value")])),
            meta: JniOutputWireContainer::KCallbackRows,
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("incompatible with its delivery"));
        let WireNode::Container {
            wires: controls, ..
        } = &mut node
        else {
            panic!()
        };
        controls[0].kind = JniWire::selector();
        validate_layout(&node).unwrap();
        let WireNode::Container { meta, .. } = &mut node else {
            panic!()
        };
        *meta = JniOutputWireContainer::KList(JniRowEncoding::ObjectArray);
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
        assert!(JniPresence::from_parts(
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
        let field: JniInputWireNode = WireNode::Annotation {
            meta: JniInputWireAnnotation::RField(syn::parse_quote!(x)),
            child: Box::new(WireNode::default()),
        };
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let duplicates: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::RFields,
            children: vec![field.clone(), field.clone()],
        };
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        let record: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::RFields,
            children: vec![field],
        };
        validate_layout(&record).unwrap();
        let malformed: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::RFields,
            children: vec![WireNode::Leaf(vec![leaf("value")])],
        };
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let empty: JniInputWireNode = WireNode::Branch {
            controls: vec![],
            meta: JniInputWireBranch::RFields,
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
        input.node = prebindgen_conversions::WireNode::Annotation {
            meta: JniInputWireAnnotation::RField(syn::parse_quote!(orphan)),
            child: Box::new(std::mem::take(&mut input.node)),
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
        output.outcomes.success = prebindgen_conversions::WireNode::Annotation {
            meta: JniOutputWireAnnotation::RField(syn::parse_quote!(orphan)),
            child: Box::new(std::mem::take(&mut output.outcomes.success)),
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
        crate::group::values_mut(&mut tree).next().unwrap().kind = JniWire::RScalar {
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
            .kind = JniWire::RScalar {
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
        sequence = prebindgen_conversions::WireNode::Container {
            wires: vec![prebindgen_conversions::Wire {
                name: syn::parse_quote!(rows),
                kind: JniWire::KRowArray {
                    condition: ("true").into(),
                    null_niche_available: true,
                },
            }],
            meta: JniInputWireContainer::KList,
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
            meta: JniInputWireBranch::RChoices,
            children: vec![],
            controls: vec![],
        };
        assert!(validate_layout(&node)
            .unwrap_err()
            .contains("no alternatives"));
        let node: JniInputWireNode = WireNode::Branch {
            meta: JniInputWireBranch::KValues,
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
pub(crate) fn error_wire_shape<
    Chain: group::ChainMetadata,
    Branch: group::BranchMetadata,
    Annotation: group::AnnotationMetadata,
    Container: group::ContainerMetadata,
>(
    node: &JniWireNode<Chain, Branch, Annotation, Container>,
) -> Result<Vec<String>> {
    fn wires(wires: &[Wire<JniWire>], out: &mut Vec<String>) {
        for wire in wires {
            let detail = match &wire.kind {
                JniWire::RScalar { primitive, .. } | JniWire::FrameInt128 { primitive, .. } => {
                    format!("{primitive:?}")
                }
                JniWire::RHandle { handle, .. } => {
                    format!("{}:{:?}", type_key(&handle.ty), handle.access)
                }
                JniWire::REnumTag { enum_type, .. } => type_key(&enum_type.mapping.ty),
                JniWire::KCallback { callback } => callback.fqn(),
                _ => String::new(),
            };
            out.push(format!(
                "{:?}:{}:{detail}",
                std::mem::discriminant(&wire.kind),
                type_key(&wire.kind.rust_type())
            ));
        }
    }
    fn visit<
        Chain: group::ChainMetadata,
        Branch: group::BranchMetadata,
        Annotation: group::AnnotationMetadata,
        Container: group::ContainerMetadata,
    >(
        node: &JniWireNode<Chain, Branch, Annotation, Container>,
        out: &mut Vec<String>,
        bulk: bool,
    ) -> Result<()> {
        match node {
            WireNode::Leaf(fields) => wires(fields, out),
            WireNode::Branch {
                meta,
                controls,
                children,
            } if (meta.is_alternatives() || meta.r_enum().is_some()) => {
                // Packed enums use a union; selector alternatives store their
                // fields consecutively. Their JVM carriers alone are identical.
                out.push(if let Some(group) = meta.r_enum().filter(|_| bulk) {
                    format!("enum:{}[", group.key())
                } else {
                    "alternatives[".into()
                });
                wires(controls, out);
                for child in children {
                    out.push("arm[".into());
                    visit(child, out, bulk)?;
                    out.push("]".into());
                }
                out.push("]".into());
            }
            WireNode::Branch {
                controls, children, ..
            } => {
                wires(controls, out);
                for child in children {
                    visit(child, out, bulk)?;
                }
            }
            WireNode::Annotation { child, .. } => visit(child, out, bulk)?,
            WireNode::Chain {
                meta,
                controls,
                child,
            } => {
                let optional = meta.optional().expect("optional chain");
                let label = {
                    match optional {
                        JniPresence::Flag(_) => "optional:flag".into(),
                        JniPresence::Niche { reservation, .. } => {
                            let index = prebindgen_conversions::wire_definitions(std::iter::once(
                                child.as_ref(),
                            ))
                            .position(|wire| wire.name == reservation.discriminator);
                            format!("optional:niche:{:?}:{index:?}", reservation.niche)
                        }
                    }
                };
                out.push(label);
                out.push("[".into());
                wires(controls, out);
                visit(child, out, bulk)?;
                out.push("]".into());
            }
            WireNode::Container {
                meta,
                wires: controls,
                child,
            } => {
                let label = if meta.is_sequence() {
                    if bulk {
                        "sequence:native-rows".into()
                    } else {
                        format!(
                            "sequence:{}",
                            if meta.is_callback_rows() {
                                "Callback"
                            } else {
                                match meta.list_encoding().expect("list encoding") {
                                    JniRowEncoding::ObjectArray => "Buffered",
                                    JniRowEncoding::RowClass => "TypedBuffered",
                                }
                            }
                        )
                    }
                } else if meta.is_bulk() {
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
                } else {
                    return Err("unsupported JNI container metadata".into());
                };

                out.push(label);
                out.push("[".into());
                wires(controls, out);
                visit(child, out, bulk || meta.is_bulk())?;
                out.push("]".into());
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(node, &mut out, false)?;
    Ok(out)
}

use prebindgen_conversions::FunctionConversions as _;
