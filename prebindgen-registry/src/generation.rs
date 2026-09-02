//! Immutable, syntax-free plans shared by registry-driven generators.
//!
//! Planning may retain an opaque [`TypeRef`], but none of the types in this
//! module can spell it. Adapter-owned associated types describe target
//! representations and operations; the registry owns identity, composition,
//! reachability, ordering, and the contracts that can be checked without Rust
//! source syntax. Those opaque payloads may use syntax for types invented by
//! the target adapter, such as a C wire type; they must not contain source Rust
//! syntax obtained by spelling a [`TypeRef`] before final emission.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
};

use crate::{
    flat::{TypeKey, TypeRef},
    recipe::{Bound, Crossing, Direction, Mode, RecipeKey, Site, Yield},
};

#[cfg(test)]
mod tests;

/// One selected recipe row applied to one spelled crossing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FragmentId {
    spelling: TypeKey,
    recipe: RecipeKey,
}

/// Stable identity of one private operation retained by a recipe fragment.
///
/// This is model identity, not a rendered Rust identifier. The final writer
/// allocates its private symbol through `RustWriter`; an adapter only supplies its
/// target namespace while rendering.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OperationId {
    owner: OperationOwner,
    role: OperationRole,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum OperationOwner {
    Fragment(FragmentId),
    Composed {
        shape: ComposedShape,
        carrier: TypeKey,
        mode: Option<Mode>,
        representation: Option<ArtifactId>,
        direction: Direction,
    },
    ModelArtifact {
        carrier: TypeKey,
        artifact: ArtifactId,
        direction: Direction,
    },
    Shared {
        artifact: ArtifactId,
        direction: Direction,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ComposedShape {
    Product,
    Optional,
    Sequence,
    Choice,
}

/// Position of a private operation inside one fragment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OperationRole {
    /// The fragment's wire-facing converter.
    Converter,
    /// One source-side conversion step, in declared chain order.
    Stage(usize),
}

impl OperationId {
    /// Identify the wire-facing converter of `fragment`.
    pub fn converter(fragment: FragmentId) -> Self {
        Self {
            owner: OperationOwner::Fragment(fragment),
            role: OperationRole::Converter,
        }
    }

    /// Identify source-side stage `index` of `fragment`.
    pub fn stage(fragment: FragmentId, index: usize) -> Self {
        Self {
            owner: OperationOwner::Fragment(fragment),
            role: OperationRole::Stage(index),
        }
    }

    /// Identify an adapter operation intentionally shared by several
    /// fragments. `artifact` is semantic adapter vocabulary, not a Rust name.
    pub fn shared(artifact: ArtifactId, direction: Direction) -> Self {
        Self {
            owner: OperationOwner::Shared {
                artifact,
                direction,
            },
            role: OperationRole::Converter,
        }
    }

    /// Identify an adapter operation by the model carrier it produces or
    /// consumes and by the adapter representation operation it performs.
    /// Crossings that reach the same contract intentionally share one helper.
    pub fn model_artifact(carrier: &TypeRef, artifact: ArtifactId, direction: Direction) -> Self {
        Self {
            owner: OperationOwner::ModelArtifact {
                carrier: carrier.key(),
                artifact,
                direction,
            },
            role: OperationRole::Converter,
        }
    }

    /// Identify a Product converter by its model carrier and adapter-declared
    /// intermediate representation.
    pub fn product_converter(
        carrier: &TypeRef,
        mode: Mode,
        representation: ArtifactId,
        direction: Direction,
    ) -> Self {
        Self::composed_converter(
            ComposedShape::Product,
            carrier,
            Self::deconstruction_mode(mode, direction),
            Some(representation),
            direction,
        )
    }

    /// Identify an Optional converter by its model carrier and
    /// adapter-declared intermediate representation.
    pub fn optional_converter(
        carrier: &TypeRef,
        mode: Mode,
        representation: ArtifactId,
        direction: Direction,
    ) -> Self {
        Self::composed_converter(
            ComposedShape::Optional,
            carrier,
            Self::deconstruction_mode(mode, direction),
            Some(representation),
            direction,
        )
    }

    /// Identify the Sequence converter for a model-selected intermediate
    /// carrier. Owned collections and borrowed views that produce the same
    /// carrier intentionally share this operation.
    pub fn sequence_converter(carrier: &TypeRef, direction: Direction) -> Self {
        Self::composed_converter(ComposedShape::Sequence, carrier, None, None, direction)
    }

    /// Identify a Choice converter by its model carrier and adapter-declared
    /// intermediate representation.
    pub fn choice_converter(
        carrier: &TypeRef,
        mode: Mode,
        representation: ArtifactId,
        direction: Direction,
    ) -> Self {
        Self::composed_converter(
            ComposedShape::Choice,
            carrier,
            Self::deconstruction_mode(mode, direction),
            Some(representation),
            direction,
        )
    }

    fn composed_converter(
        shape: ComposedShape,
        carrier: &TypeRef,
        mode: Option<Mode>,
        representation: Option<ArtifactId>,
        direction: Direction,
    ) -> Self {
        Self {
            owner: OperationOwner::Composed {
                shape,
                carrier: carrier.key(),
                mode,
                representation,
                direction,
            },
            role: OperationRole::Converter,
        }
    }

    fn deconstruction_mode(mode: Mode, direction: Direction) -> Option<Mode> {
        match direction {
            // A construct converter's result is already the model carrier;
            // the asking site's later use of that value does not change this
            // operation's signature or body.
            Direction::Construct => None,
            Direction::Deconstruct => Some(mode),
        }
    }

    /// Fragment that owns this operation, or `None` for a contract-owned or
    /// explicitly shared operation.
    pub fn fragment(&self) -> Option<&FragmentId> {
        match &self.owner {
            OperationOwner::Fragment(fragment) => Some(fragment),
            OperationOwner::Composed { .. } => None,
            OperationOwner::ModelArtifact { .. } => None,
            OperationOwner::Shared { .. } => None,
        }
    }

    /// Conversion direction of this operation.
    pub fn direction(&self) -> Direction {
        match &self.owner {
            OperationOwner::Fragment(fragment) => fragment.direction(),
            OperationOwner::Composed { direction, .. } => *direction,
            OperationOwner::ModelArtifact { direction, .. } => *direction,
            OperationOwner::Shared { direction, .. } => *direction,
        }
    }

    /// Operation position inside the fragment.
    pub fn role(&self) -> &OperationRole {
        &self.role
    }

    /// Stable writer-only fingerprint used to allocate a private Rust symbol.
    ///
    /// Keeping this implementation beside the model identity prevents an
    /// adapter from reading or reinterpreting the `TypeKey` values contained
    /// by a fragment. The fingerprint deliberately has no public accessor;
    /// final emission exposes only the completed identifier through `RustWriter`.
    pub(crate) fn stable_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in self.stable_key().bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    /// Human-readable semantic owner used as the prefix of the final private
    /// Rust symbol. This is deliberately available only inside the registry:
    /// adapters retain the identity, while the writer decides how model
    /// identity is presented in generated Rust.
    pub(crate) fn semantic_label(&self) -> String {
        match &self.owner {
            OperationOwner::Fragment(fragment) => fragment.spelling().as_str().to_owned(),
            OperationOwner::Composed {
                shape,
                carrier,
                representation,
                ..
            } => match shape {
                ComposedShape::Product | ComposedShape::Optional | ComposedShape::Choice => {
                    match representation {
                        Some(representation) => format!(
                            "{}_{}_{}",
                            carrier.as_str(),
                            representation.kind(),
                            representation.name()
                        ),
                        None => carrier.as_str().to_owned(),
                    }
                }
                ComposedShape::Sequence => format!("sequence_{}", carrier.as_str()),
            },
            OperationOwner::ModelArtifact {
                carrier, artifact, ..
            } => format!(
                "{}_{}_{}",
                carrier.as_str(),
                artifact.kind(),
                artifact.name()
            ),
            OperationOwner::Shared { artifact, .. } => {
                format!("{}_{}", artifact.kind(), artifact.name())
            }
        }
    }

    fn stable_key(&self) -> String {
        let owner = match &self.owner {
            OperationOwner::Fragment(fragment) => format!("fragment\0{}", fragment.stable_key()),
            OperationOwner::Composed {
                shape,
                carrier,
                mode,
                representation,
                direction,
            } => {
                let representation = representation
                    .as_ref()
                    .map(|representation| {
                        format!("{}\0{}", representation.kind(), representation.name())
                    })
                    .unwrap_or_default();
                format!(
                    "composed\0{shape:?}\0{}\0{mode:?}\0{representation}\0{direction}",
                    carrier.as_str(),
                )
            }
            OperationOwner::ModelArtifact {
                carrier,
                artifact,
                direction,
            } => format!(
                "model-artifact\0{}\0{}\0{}\0{direction}",
                carrier.as_str(),
                artifact.kind(),
                artifact.name()
            ),
            OperationOwner::Shared {
                artifact,
                direction,
            } => format!(
                "shared\0{}\0{}\0{}",
                artifact.kind(),
                artifact.name(),
                direction
            ),
        };
        let role = match self.role {
            OperationRole::Converter => "converter".to_string(),
            OperationRole::Stage(index) => format!("stage\0{index}"),
        };
        format!("{owner}\0{role}")
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.role {
            OperationRole::Converter => match &self.owner {
                OperationOwner::Fragment(fragment) => write!(f, "converter of {fragment}"),
                OperationOwner::Composed {
                    shape,
                    carrier,
                    mode,
                    representation,
                    direction,
                } => write!(
                    f,
                    "{direction} {mode:?} {shape:?} converter for `{carrier}` via {representation:?}"
                ),
                OperationOwner::ModelArtifact {
                    carrier,
                    artifact,
                    direction,
                } => write!(f, "{direction} {artifact} converter for `{carrier}`"),
                OperationOwner::Shared {
                    artifact,
                    direction,
                } => {
                    write!(f, "shared {direction} converter {artifact}")
                }
            },
            OperationRole::Stage(index) => match &self.owner {
                OperationOwner::Fragment(fragment) => write!(f, "stage {index} of {fragment}"),
                OperationOwner::Composed {
                    shape,
                    carrier,
                    mode,
                    representation,
                    direction,
                } => write!(
                    f,
                    "{direction} {mode:?} {shape:?} stage {index} for `{carrier}` via {representation:?}"
                ),
                OperationOwner::ModelArtifact {
                    carrier,
                    artifact,
                    direction,
                } => write!(f, "{direction} {artifact} stage {index} for `{carrier}`"),
                OperationOwner::Shared {
                    artifact,
                    direction,
                } => {
                    write!(f, "shared {direction} stage {index} {artifact}")
                }
            },
        }
    }
}

impl FragmentId {
    /// Identify the fragment for an exact source spelling and recipe row.
    pub fn new(spelling: TypeKey, recipe: RecipeKey) -> Self {
        Self { spelling, recipe }
    }

    /// The exact source type spelling this fragment was planned for.
    pub fn spelling(&self) -> &TypeKey {
        &self.spelling
    }

    /// The complete recipe-table row selected for the crossing.
    pub fn recipe(&self) -> &RecipeKey {
        &self.recipe
    }

    /// The direction encoded by the selected row.
    pub fn direction(&self) -> Direction {
        self.recipe.crossing().direction
    }

    fn stable_key(&self) -> String {
        format!(
            "{}\0{}\0{}\0{}",
            self.spelling.as_str(),
            self.direction(),
            self.recipe.crossing().ty.as_str(),
            self.recipe.name()
        )
    }
}

impl fmt::Display for FragmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fragment `{}` using {}", self.spelling, self.recipe)
    }
}

/// Stable identity of one crossing position in the generated interface.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SiteId(Site);

impl SiteId {
    /// Use the model's semantic site as a generation-plan identity.
    pub fn new(site: Site) -> Self {
        Self(site)
    }

    /// The model site this identity denotes.
    pub fn site(&self) -> &Site {
        &self.0
    }

    fn stable_key(&self) -> String {
        format!("{}\0{}", self.0.owner, self.0.role)
    }
}

impl fmt::Display for SiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Semantic identity of one final output artifact.
///
/// These fields are adapter vocabulary, not rendered Rust identifiers. A
/// renderer remains free to name, combine, or inline private artifacts.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtifactId {
    kind: String,
    name: String,
}

impl ArtifactId {
    /// Make an adapter-scoped artifact identity.
    pub fn new(kind: impl Into<String>, name: impl Into<String>) -> Result<Self, IdentityError> {
        let kind = kind.into();
        let name = name.into();
        if kind.is_empty() || name.is_empty() {
            return Err(IdentityError::EmptyArtifact { kind, name });
        }
        Ok(Self { kind, name })
    }

    /// The adapter-defined class of artifact.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// The semantic key within [`Self::kind`].
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} `{}`", self.kind, self.name)
    }
}

/// Failure to construct a semantic identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// An artifact identity must have both components.
    EmptyArtifact {
        /// The invalid kind.
        kind: String,
        /// The invalid name.
        name: String,
    },
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyArtifact { kind, name } => write!(
                f,
                "artifact identity needs a non-empty kind and name (got `{kind}` / `{name}`)"
            ),
        }
    }
}

impl std::error::Error for IdentityError {}

/// Adapter-owned representation vocabulary stored by the shared plan.
///
/// Associated values describe semantics, never rendered source items. The
/// registry compares niche identities and treats every other payload as opaque.
/// A payload may retain target-language syntax owned by the adapter. Source
/// Rust syntax remains forbidden until final emission spells an opaque
/// [`TypeRef`].
pub trait Representation {
    /// Frozen adapter artifact that renders this fragment's private converter.
    ///
    /// The registry owns its dependency-ordered placement but treats the
    /// payload as opaque. [`FragmentPlan`] represents fragments without a
    /// standalone converter explicitly.
    type ConverterArtifact;
    /// A typed route for a failed site conversion.
    type FailureRoute;
    /// The ordered wire layout of one site.
    type AbiLayout;
    /// One final adapter-specific output item.
    type Artifact;
}

/// A child fragment together with the source-value contract this edge needs.
#[derive(Clone, Debug)]
pub struct FragmentUse {
    fragment: FragmentId,
    required: Yield,
}

impl FragmentUse {
    /// Require `fragment` to produce `required`.
    pub fn new(fragment: FragmentId, required: Yield) -> Self {
        Self { fragment, required }
    }

    /// The child fragment.
    pub fn fragment(&self) -> &FragmentId {
        &self.fragment
    }

    /// The type, ownership mode, and validity required by this edge.
    pub fn required(&self) -> &Yield {
        &self.required
    }
}

/// An operation whose positional arity is part of its frozen contract.
#[derive(Clone)]
pub struct FixedArity<P> {
    arity: usize,
    payload: P,
}

impl<P> FixedArity<P> {
    /// Declare that `payload` consumes exactly `arity` ordered values.
    pub fn new(arity: usize, payload: P) -> Self {
        Self { arity, payload }
    }

    /// Required positional arity.
    pub fn arity(&self) -> usize {
        self.arity
    }

    /// Adapter operation payload.
    pub fn payload(&self) -> &P {
        &self.payload
    }
}

/// Choice bridge payload plus the exact arity of every arm.
#[derive(Clone)]
pub struct ChoiceArity<P> {
    arm_arities: Vec<usize>,
    payload: P,
}

impl<P> ChoiceArity<P> {
    /// Declare one ordered arity per choice arm.
    pub fn new(arm_arities: Vec<usize>, payload: P) -> Self {
        Self {
            arm_arities,
            payload,
        }
    }

    /// Required arity of every arm, in tag order.
    pub fn arm_arities(&self) -> &[usize] {
        &self.arm_arities
    }

    /// Adapter operation payload.
    pub fn payload(&self) -> &P {
        &self.payload
    }
}

/// Niche domains consumed and exposed by one converter.
pub struct NichePlan<N> {
    discriminants: usize,
    consumed: Vec<N>,
    exposed: Vec<N>,
}

impl<N> NichePlan<N> {
    /// State the niches used for discriminants and those left for parents.
    pub fn new(discriminants: usize, consumed: Vec<N>, exposed: Vec<N>) -> Self {
        Self {
            discriminants,
            consumed,
            exposed,
        }
    }

    /// A plan that neither consumes nor exposes a niche.
    pub fn none() -> Self {
        Self::new(0, Vec::new(), Vec::new())
    }

    /// Number of distinct discriminant values the bridge needs.
    pub fn discriminants(&self) -> usize {
        self.discriminants
    }

    /// Niche domains consumed by this bridge.
    pub fn consumed(&self) -> &[N] {
        &self.consumed
    }

    /// Niche domains this bridge leaves available to its parent.
    pub fn exposed(&self) -> &[N] {
        &self.exposed
    }
}

/// Whether a converter can fail.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No failure edge exists.
    Infallible,
    /// Failure must be routed at every site using this fragment.
    Fallible,
}

/// When an adapter cleanup operation runs.
#[derive(Clone)]
pub enum Cleanup<C> {
    /// Explicitly no cleanup is required.
    None,
    /// Run only if conversion fails.
    OnFailure(C),
    /// Run on both success and failure.
    Always(C),
    /// Destroy an owned value unless the foreign side takes it.
    UnlessTransferred(C),
}

/// One endpoint in a fragment's directional conversion chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainValue<I> {
    /// The source Rust value denoted by the fragment's opaque `TypeRef`.
    Source,
    /// An adapter-declared internal carrier, without source type syntax.
    Intermediate(I),
}

/// One explicit adapter conversion between values in a fragment graph.
///
/// A step has no fragment identity of its own: it is an internal node of the
/// selected fragment, which is how conversions such as
/// `jint -> i32 -> Percent` remain one recipe answer.
#[derive(Clone)]
pub struct ConverterStep {
    from: ChainValue<TypeKey>,
    into: ChainValue<TypeKey>,
    operation: OperationId,
    failure: Failure,
    cleanup: Cleanup<()>,
}

impl ConverterStep {
    /// Describe one directional conversion step.
    pub fn new(
        from: ChainValue<TypeKey>,
        into: ChainValue<TypeKey>,
        operation: OperationId,
        failure: Failure,
        cleanup: Cleanup<()>,
    ) -> Self {
        Self {
            from,
            into,
            operation,
            failure,
            cleanup,
        }
    }

    /// Value consumed by this step.
    pub fn from(&self) -> &ChainValue<TypeKey> {
        &self.from
    }

    /// Value produced by this step.
    pub fn into(&self) -> &ChainValue<TypeKey> {
        &self.into
    }

    /// Adapter-owned semantic operation.
    pub fn operation(&self) -> &OperationId {
        &self.operation
    }

    /// Whether this step can fail.
    pub fn failure(&self) -> Failure {
        self.failure
    }

    /// Cleanup attached to this step's failure and success edges.
    pub fn cleanup(&self) -> &Cleanup<()> {
        &self.cleanup
    }
}

/// Directional steps between the source value and the selected shape.
///
/// [`ShapePlan`] owns the wire-to-intermediate boundary. An atomic shape
/// retains its terminal codec even when that operation is an identity. This
/// chain never converts wire values: its steps are stored in execution order
/// from the fragment's shape-adjacent intermediate to [`ChainValue::Source`]
/// when constructing, and in the opposite direction when deconstructing.
/// [`Self::Direct`] means the shape itself consumes or produces the source value.
#[derive(Clone)]
pub enum ConversionChain {
    /// No adapter conversion lies between the shape and source value.
    Direct,
    /// One or more explicit internal conversions.
    Steps(Vec<ConverterStep>),
}

impl ConversionChain {
    /// Explicit steps in execution order.
    pub fn steps(&self) -> &[ConverterStep] {
        match self {
            Self::Direct => &[],
            Self::Steps(steps) => steps,
        }
    }
}

/// The registry-composed converter operation for one fragment.
#[derive(Clone)]
pub enum ShapePlan {
    /// Convert one wire leaf to or from the fragment's intermediate.
    Atomic(OperationId),
    /// Pack or unpack all fixed positions.
    Product {
        /// This fragment's converter identity.
        bridge: FixedArity<OperationId>,
        /// Ordered source parts.
        parts: Vec<FragmentUse>,
    },
    /// Absent/present control flow around one value.
    Optional {
        /// This fragment's converter identity.
        bridge: OperationId,
        /// The present value.
        value: FragmentUse,
    },
    /// Builder or traversal control flow around one element type.
    Sequence {
        /// This fragment's converter identity.
        bridge: OperationId,
        /// The repeated element.
        element: FragmentUse,
    },
    /// Tagged selection among ordered arms.
    Choice {
        /// This fragment's converter identity, and the arm contracts.
        bridge: ChoiceArity<OperationId>,
        /// Parts in every arm, in tag and then position order.
        arms: Vec<Vec<FragmentUse>>,
    },
    /// Foreign callable construction and later argument delivery.
    Invoke {
        /// This fragment's converter identity.
        bridge: FixedArity<OperationId>,
        /// Callback arguments. Their direction is opposite the callable's.
        arguments: Vec<FragmentUse>,
    },
}

impl ShapePlan {
    fn uses(&self) -> Vec<&FragmentUse> {
        match self {
            Self::Atomic(_) => Vec::new(),
            Self::Product { parts, .. } => parts.iter().collect(),
            Self::Optional { value, .. } => vec![value],
            Self::Sequence { element, .. } => vec![element],
            Self::Choice { arms, .. } => arms.iter().flatten().collect(),
            Self::Invoke { arguments, .. } => arguments.iter().collect(),
        }
    }

    fn child_direction(&self, parent: Direction) -> Direction {
        match self {
            Self::Invoke { .. } => parent.swap(),
            _ => parent,
        }
    }
}

/// Complete syntax-free converter graph for one fragment.
pub struct ConverterPlan {
    shape: ShapePlan,
    chain: ConversionChain,
    niches: NichePlan<String>,
    failure: Failure,
    cleanup: Cleanup<()>,
}

impl ConverterPlan {
    /// Freeze one converter operation graph.
    pub fn new(
        shape: ShapePlan,
        niches: NichePlan<String>,
        failure: Failure,
        cleanup: Cleanup<()>,
    ) -> Self {
        Self {
            shape,
            chain: ConversionChain::Direct,
            niches,
            failure,
            cleanup,
        }
    }

    /// Freeze a converter whose selected shape is joined to the source by
    /// explicit adapter conversion steps.
    pub fn with_chain(
        shape: ShapePlan,
        chain: ConversionChain,
        niches: NichePlan<String>,
        failure: Failure,
        cleanup: Cleanup<()>,
    ) -> Self {
        Self {
            shape,
            chain,
            niches,
            failure,
            cleanup,
        }
    }

    /// Directional internal conversions between the shape and source value.
    pub fn chain(&self) -> &ConversionChain {
        &self.chain
    }

    /// The selected shape and representation operation.
    pub fn shape(&self) -> &ShapePlan {
        &self.shape
    }

    /// Consumed and exposed niche domains.
    pub fn niches(&self) -> &NichePlan<String> {
        &self.niches
    }

    /// Whether this graph has a failure edge.
    pub fn failure(&self) -> Failure {
        self.failure
    }

    /// Cleanup attached to this graph.
    pub fn cleanup(&self) -> &Cleanup<()> {
        &self.cleanup
    }
}

/// Immutable plan for one selected, spelled recipe fragment.
pub struct FragmentPlan<R: Representation> {
    id: FragmentId,
    source: TypeRef,
    intermediate: TypeKey,
    converter: ConverterPlan,
    conversion: Option<R::ConverterArtifact>,
    /// Whether [`Self::conversion`] is also emitted into the file.
    renders: bool,
    yields: Yield,
}

impl<R: Representation> FragmentPlan<R> {
    /// Describe one fragment completely, without rendering it.
    pub fn new(
        id: FragmentId,
        source: TypeRef,
        intermediate: TypeKey,
        converter: ConverterPlan,
        yields: Yield,
    ) -> Self {
        Self {
            id,
            source,
            intermediate,
            converter,
            conversion: None,
            renders: false,
            yields,
        }
    }

    /// Attach the adapter's frozen private-converter artifact, which the file
    /// emits.
    pub fn with_artifact(mut self, artifact: R::ConverterArtifact) -> Self {
        self.conversion = Some(artifact);
        self.renders = true;
        self
    }

    /// Attach a conversion the file does **not** emit.
    ///
    /// A fragment can have a conversion and render nothing: one composed into
    /// its parent emits no converter of its own, and a deferred callable is
    /// invoked where it lands rather than through a function of its own. Both
    /// adapters have such fragments, and both still have to say what the
    /// conversion *is* — an emitter asking how such a value crosses has a fair
    /// question, and before this the plan could not answer it and the adapter's
    /// own compile-time carrier had to (#660 item 5).
    pub fn with_composed_conversion(mut self, conversion: R::ConverterArtifact) -> Self {
        self.conversion = Some(conversion);
        self.renders = false;
        self
    }

    /// Semantic fragment identity.
    pub fn id(&self) -> &FragmentId {
        &self.id
    }

    /// Opaque source type, spellable only by final emission.
    pub fn source(&self) -> &TypeRef {
        &self.source
    }

    /// Adapter-selected carrier adjacent to the shape.
    pub fn intermediate(&self) -> &TypeKey {
        &self.intermediate
    }

    /// Registry-composed converter operation graph.
    pub fn converter(&self) -> &ConverterPlan {
        &self.converter
    }

    /// Adapter-owned converter artifact in registry dependency order.
    ///
    /// `None` for a fragment that renders nothing, whether or not it has a
    /// conversion — this is the file's question. [`Self::conversion`] is the
    /// other one.
    pub fn artifact(&self) -> Option<&R::ConverterArtifact> {
        self.conversion.as_ref().filter(|_| self.renders)
    }

    /// How this fragment's value crosses, whether or not the file emits a
    /// converter for it — see [`Self::with_composed_conversion`].
    pub fn conversion(&self) -> Option<&R::ConverterArtifact> {
        self.conversion.as_ref()
    }

    /// Source-value contract produced by this fragment.
    pub fn yields(&self) -> &Yield {
        &self.yields
    }
}

/// Ordered wire layout assigned to one boundary site.
pub struct AbiLayout<L> {
    slots: usize,
    payload: L,
}

impl<L> AbiLayout<L> {
    /// State how many ordered ABI leaves `payload` occupies.
    pub fn new(slots: usize, payload: L) -> Self {
        Self { slots, payload }
    }

    /// Number of ABI leaves.
    pub fn slots(&self) -> usize {
        self.slots
    }

    /// Adapter layout payload.
    pub fn payload(&self) -> &L {
        &self.payload
    }
}

/// Immutable plan for one declared boundary position.
pub struct SitePlan<R: Representation> {
    id: SiteId,
    bound: Bound,
    fragment: FragmentId,
    required: Yield,
    abi: AbiLayout<R::AbiLayout>,
    failure_route: Option<R::FailureRoute>,
    cleanup: Cleanup<()>,
}

// Cloned where a site travels with the plan being collected, for the same
// reason [`ShapePlan`]'s impl is written out: `derive` would ask `R: Clone`,
// and `R` names an adapter's associated types rather than being a value.
impl<R: Representation> Clone for SitePlan<R>
where
    R::AbiLayout: Clone,
    R::FailureRoute: Clone,
    (): Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            bound: self.bound.clone(),
            fragment: self.fragment.clone(),
            required: self.required.clone(),
            abi: AbiLayout::new(self.abi.slots(), self.abi.payload().clone()),
            failure_route: self.failure_route.clone(),
            cleanup: self.cleanup.clone(),
        }
    }
}

impl<R: Representation> SitePlan<R> {
    /// Freeze the fragment, ABI, error route, and cleanup selected for a site.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: SiteId,
        bound: Bound,
        fragment: FragmentId,
        required: Yield,
        abi: AbiLayout<R::AbiLayout>,
        failure_route: Option<R::FailureRoute>,
        cleanup: Cleanup<()>,
    ) -> Self {
        Self {
            id,
            bound,
            fragment,
            required,
            abi,
            failure_route,
            cleanup,
        }
    }

    /// Semantic site identity.
    pub fn id(&self) -> &SiteId {
        &self.id
    }

    /// Resolved recipe-table binding.
    pub fn bound(&self) -> &Bound {
        &self.bound
    }

    /// Reusable fragment selected for this site.
    pub fn fragment(&self) -> &FragmentId {
        &self.fragment
    }

    /// Source-value contract this position requires.
    pub fn required(&self) -> &Yield {
        &self.required
    }

    /// Ordered target ABI layout.
    pub fn abi(&self) -> &AbiLayout<R::AbiLayout> {
        &self.abi
    }

    /// Typed route used if conversion fails.
    pub fn failure_route(&self) -> Option<&R::FailureRoute> {
        self.failure_route.as_ref()
    }

    /// Site-level cleanup policy.
    pub fn cleanup(&self) -> &Cleanup<()> {
        &self.cleanup
    }
}

/// One dependency of a final artifact.
#[derive(Clone, Debug)]
pub enum ArtifactInput {
    /// The artifact consumes a converter fragment directly.
    Fragment(FragmentId),
    /// The artifact consumes a site's exact number of ABI leaves.
    Site {
        /// Site being consumed.
        site: SiteId,
        /// Arity expected by this artifact's adapter payload.
        slots: usize,
    },
}

/// Immutable adapter-owned final item plus its semantic dependencies.
pub struct ArtifactPlan<R: Representation> {
    id: ArtifactId,
    prerequisites: Vec<ArtifactId>,
    inputs: Vec<ArtifactInput>,
    follows: Vec<FragmentId>,
    payload: R::Artifact,
}

impl<R: Representation> ArtifactPlan<R> {
    /// Describe one output item before rendering.
    pub fn new(
        id: ArtifactId,
        prerequisites: Vec<ArtifactId>,
        inputs: Vec<ArtifactInput>,
        payload: R::Artifact,
    ) -> Self {
        Self {
            id,
            prerequisites,
            inputs,
            follows: Vec::new(),
            payload,
        }
    }

    /// State that this artifact exists only while one of `fragments` is reached.
    ///
    /// The inverse of an [`ArtifactInput::Fragment`]. An input is a *reason* a
    /// fragment is kept: the artifact needs it, so the plan must not prune it.
    /// This is a *consequence* of a fragment being kept: the file emits a
    /// private converter because its fragment survived, and emitting one is no
    /// reason to keep the fragment. So the plan drops such an artifact when it
    /// has dropped every fragment it follows.
    ///
    /// Several fragments can render one converter — an operation shared by two
    /// crossings is emitted once — which is why this is a list: the artifact
    /// stays while any one of them does.
    ///
    /// Left unset, the artifact is unconditional: the binding declared it, and
    /// no fragment's fate withdraws it.
    pub fn follows(mut self, fragments: Vec<FragmentId>) -> Self {
        self.follows = fragments;
        self
    }

    /// Semantic artifact identity.
    pub fn id(&self) -> &ArtifactId {
        &self.id
    }

    /// Artifacts that must be emitted first.
    pub fn prerequisites(&self) -> &[ArtifactId] {
        &self.prerequisites
    }

    /// Fragment and site contracts this item consumes.
    pub fn inputs(&self) -> &[ArtifactInput] {
        &self.inputs
    }

    /// Fragments whose reachability this item's existence follows — see
    /// [`Self::follows`]. Empty for an unconditional artifact.
    pub fn followed(&self) -> &[FragmentId] {
        &self.follows
    }

    /// Adapter-owned artifact description.
    pub fn payload(&self) -> &R::Artifact {
        &self.payload
    }
}

/// Mutable collection phase preceding an immutable [`GenerationPlan`].
pub struct GenerationPlanBuilder<R: Representation> {
    fragments: HashMap<FragmentId, FragmentPlan<R>>,
    sites: HashMap<SiteId, SitePlan<R>>,
    artifacts: HashMap<ArtifactId, ArtifactPlan<R>>,
    /// Artifacts in the order they were added, which is the order the file
    /// emits them in. A prerequisite still hoists ahead of its dependent.
    declared: Vec<ArtifactId>,
    roots: Vec<FragmentId>,
    errors: Vec<PlanError>,
}

impl<R: Representation> Default for GenerationPlanBuilder<R> {
    fn default() -> Self {
        Self {
            fragments: HashMap::new(),
            sites: HashMap::new(),
            artifacts: HashMap::new(),
            declared: Vec::new(),
            roots: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl<R: Representation> GenerationPlanBuilder<R> {
    /// Start an empty plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one fragment. Duplicate semantic identities are retained as errors.
    pub fn fragment(&mut self, plan: FragmentPlan<R>) -> &mut Self {
        let id = plan.id.clone();
        if self.fragments.insert(id.clone(), plan).is_some() {
            self.errors.push(PlanError::DuplicateFragment(id));
        }
        self
    }

    /// Add one boundary site. Duplicate semantic identities are errors.
    pub fn site(&mut self, plan: SitePlan<R>) -> &mut Self {
        let id = plan.id.clone();
        if self.sites.insert(id.clone(), plan).is_some() {
            self.errors.push(PlanError::DuplicateSite(id));
        }
        self
    }

    /// Add one final artifact. Duplicate semantic identities are errors.
    ///
    /// The order artifacts are added in is the order the file emits them in.
    pub fn artifact(&mut self, plan: ArtifactPlan<R>) -> &mut Self {
        let id = plan.id.clone();
        if self.artifacts.insert(id.clone(), plan).is_some() {
            self.errors.push(PlanError::DuplicateArtifact(id));
        } else {
            self.declared.push(id);
        }
        self
    }

    /// Declare `fragment` reached because the binding's surface says so.
    ///
    /// Sites and artifact inputs are the reasons a fragment is kept that the
    /// plan can see for itself. This is the one an adapter has to state: a
    /// declared class crosses in both directions whether or not this build
    /// happens to export a function that mentions it, so its converters are
    /// binding surface and not dead code.
    pub fn root(&mut self, fragment: FragmentId) -> &mut Self {
        self.roots.push(fragment);
        self
    }

    /// Validate, compute deterministic dependency order, and freeze the plan.
    pub fn build(mut self) -> Result<GenerationPlan<R>, PlanErrors> {
        self.validate_fragments();
        self.validate_sites();
        self.validate_artifacts();
        let fragment_order = topo_fragments(&self.fragments, &mut self.errors);
        let artifact_order = topo_artifacts(&self.artifacts, &self.declared, &mut self.errors);
        if !self.errors.is_empty() {
            return Err(PlanErrors(self.errors));
        }

        // What the binding asks for outright: the fragment each site names, and
        // the roots the adapter declared.
        let declared_roots: Vec<_> = self
            .sites
            .values()
            .map(|s| s.fragment.clone())
            .chain(self.roots.iter().cloned())
            .collect();
        // An artifact that follows fragments is kept only while one of them is
        // reached, and until it is kept it asks for nothing: a private
        // converter the file will not emit is no reason to keep the converters
        // it would have called. But a kept one's inputs are reasons, and
        // keeping it can reach a fragment that keeps another — so this runs to
        // a fixed point rather than in one pass. It terminates because the kept
        // set only grows.
        let mut kept: HashSet<ArtifactId> = HashSet::new();
        let mut reachable;
        loop {
            let mut roots = declared_roots.clone();
            roots.extend(
                self.artifacts
                    .values()
                    .filter(|artifact| artifact.follows.is_empty() || kept.contains(&artifact.id))
                    .flat_map(|artifact| {
                        artifact.inputs.iter().filter_map(|input| match input {
                            ArtifactInput::Fragment(id) => Some(id.clone()),
                            ArtifactInput::Site { .. } => None,
                        })
                    }),
            );
            roots.sort_by_cached_key(FragmentId::stable_key);
            roots.dedup();
            reachable = reachable_fragments(&self.fragments, roots);
            let grown: HashSet<ArtifactId> = self
                .artifacts
                .values()
                .filter(|artifact| {
                    artifact.follows.is_empty()
                        || artifact.follows.iter().any(|id| reachable.contains(id))
                })
                .map(|artifact| artifact.id.clone())
                .collect();
            if grown == kept {
                break;
            }
            kept = grown;
        }
        let fragment_order = fragment_order
            .into_iter()
            .filter(|id| reachable.contains(id))
            .collect();
        self.fragments.retain(|id, _| reachable.contains(id));
        let artifact_order: Vec<_> = artifact_order
            .into_iter()
            .filter(|id| kept.contains(id))
            .collect();
        self.artifacts.retain(|id, _| kept.contains(id));

        let mut site_order: Vec<_> = self.sites.keys().cloned().collect();
        site_order.sort_by_cached_key(SiteId::stable_key);
        Ok(GenerationPlan {
            fragments: self.fragments,
            fragment_order,
            sites: self.sites,
            site_order,
            artifacts: self.artifacts,
            artifact_order,
        })
    }

    fn validate_fragments(&mut self) {
        for fragment in self.fragments.values() {
            let id = fragment.id();
            if fragment.source().key() != *id.spelling() {
                self.errors.push(PlanError::FragmentSpelling(id.clone()));
            }
            let crossing = Crossing::new(fragment.source().clone(), id.direction());
            if crossing.key() != *id.recipe().crossing() {
                self.errors.push(PlanError::FragmentCrossing(id.clone()));
            }
            if fragment.yields().ty != id.recipe().crossing().ty {
                self.errors.push(PlanError::YieldType(id.clone()));
            }
            match fragment.converter().shape() {
                ShapePlan::Product { bridge, parts } if bridge.arity() != parts.len() => {
                    self.errors.push(PlanError::Arity(id.clone()));
                }
                ShapePlan::Choice { bridge, arms } => {
                    if bridge.arm_arities().len() != arms.len()
                        || bridge
                            .arm_arities()
                            .iter()
                            .zip(arms)
                            .any(|(arity, arm)| *arity != arm.len())
                    {
                        self.errors.push(PlanError::Arity(id.clone()));
                    }
                }
                ShapePlan::Invoke { bridge, arguments } if bridge.arity() != arguments.len() => {
                    self.errors.push(PlanError::Arity(id.clone()));
                }
                _ => {}
            }
            validate_chain(&mut self.errors, fragment);
            let niches = fragment.converter().niches();
            if niches.consumed().len() < niches.discriminants() {
                self.errors.push(PlanError::InsufficientNiches(id.clone()));
            }
            let mut seen = HashSet::new();
            if niches
                .consumed()
                .iter()
                .chain(niches.exposed())
                .any(|niche| !seen.insert(niche))
            {
                self.errors.push(PlanError::OverlappingNiches(id.clone()));
            }
            if matches!(
                fragment.converter().cleanup(),
                Cleanup::UnlessTransferred(_)
            ) && fragment.yields().mode != Mode::Owned
            {
                self.errors.push(PlanError::TransferOfBorrowed(id.clone()));
            }

            let direction = fragment.converter().shape().child_direction(id.direction());
            for usage in fragment.converter().shape().uses() {
                let Some(child) = self.fragments.get(usage.fragment()) else {
                    self.errors.push(PlanError::UnknownChild {
                        parent: id.clone(),
                        child: usage.fragment().clone(),
                    });
                    continue;
                };
                if usage.fragment().direction() != direction {
                    self.errors.push(PlanError::ChildDirection(id.clone()));
                }
                validate_yield(
                    &mut self.errors,
                    ContractAt::Fragment(id.clone()),
                    child.yields(),
                    usage.required(),
                );
            }
        }
    }

    fn validate_sites(&mut self) {
        for site in self.sites.values() {
            let id = site.id();
            if id.site() != &site.bound().site {
                self.errors.push(PlanError::SiteIdentity(id.clone()));
            }
            let Some(fragment) = self.fragments.get(site.fragment()) else {
                self.errors.push(PlanError::UnknownSiteFragment(id.clone()));
                continue;
            };
            if site.fragment().direction() != site.bound().crossing.direction() {
                self.errors.push(PlanError::SiteDirection(id.clone()));
            }
            if site.fragment().spelling() != &site.bound().crossing.spelled().key() {
                self.errors.push(PlanError::SiteSpelling(id.clone()));
            }
            if site.fragment().recipe() != &site.bound().recipe {
                self.errors.push(PlanError::SiteRecipe(id.clone()));
            }
            validate_yield(
                &mut self.errors,
                ContractAt::Site(id.clone()),
                fragment.yields(),
                site.required(),
            );
            match (
                fragment.converter().failure(),
                site.failure_route().is_some(),
            ) {
                (Failure::Fallible, false) => {
                    self.errors.push(PlanError::MissingFailureRoute(id.clone()));
                }
                (Failure::Infallible, true) => {
                    self.errors
                        .push(PlanError::UnexpectedFailureRoute(id.clone()));
                }
                _ => {}
            }
            if matches!(site.cleanup(), Cleanup::UnlessTransferred(_))
                && fragment.yields().mode != Mode::Owned
            {
                self.errors
                    .push(PlanError::SiteTransferOfBorrowed(id.clone()));
            }
        }
    }

    fn validate_artifacts(&mut self) {
        for artifact in self.artifacts.values() {
            for dependency in artifact.prerequisites() {
                if !self.artifacts.contains_key(dependency) {
                    self.errors.push(PlanError::UnknownPrerequisite {
                        artifact: artifact.id().clone(),
                        prerequisite: dependency.clone(),
                    });
                }
            }
            for input in artifact.inputs() {
                match input {
                    ArtifactInput::Fragment(id) if !self.fragments.contains_key(id) => self
                        .errors
                        .push(PlanError::UnknownArtifactFragment(artifact.id().clone())),
                    ArtifactInput::Site { site, slots } => match self.sites.get(site) {
                        None => self
                            .errors
                            .push(PlanError::UnknownArtifactSite(artifact.id().clone())),
                        Some(plan) if plan.abi().slots() != *slots => self
                            .errors
                            .push(PlanError::AbiArity(artifact.id().clone(), site.clone())),
                        Some(_) => {}
                    },
                    ArtifactInput::Fragment(_) => {}
                }
            }
            // A followed fragment is not an input, so the loop above does not
            // see it — and an unknown one is worse than an unknown input:
            // `reachable_fragments` reaches whatever it is handed, so an
            // artifact following an id no fragment has would be kept, and a
            // kept artifact roots its own inputs (#660 review).
            for fragment in artifact.followed() {
                if !self.fragments.contains_key(fragment) {
                    self.errors.push(PlanError::UnknownFollowedFragment {
                        artifact: artifact.id().clone(),
                        fragment: fragment.clone(),
                    });
                }
            }
        }
        for root in &self.roots {
            if !self.fragments.contains_key(root) {
                self.errors.push(PlanError::UnknownRoot(root.clone()));
            }
        }
    }
}

/// A validated generation plan. It has no mutating or registry lookup API.
pub struct GenerationPlan<R: Representation> {
    fragments: HashMap<FragmentId, FragmentPlan<R>>,
    fragment_order: Vec<FragmentId>,
    sites: HashMap<SiteId, SitePlan<R>>,
    site_order: Vec<SiteId>,
    artifacts: HashMap<ArtifactId, ArtifactPlan<R>>,
    artifact_order: Vec<ArtifactId>,
}

impl<R: Representation> fmt::Debug for GenerationPlan<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationPlan")
            .field("fragments", &self.fragments.len())
            .field("sites", &self.sites.len())
            .field("artifacts", &self.artifacts.len())
            .finish()
    }
}

impl<R: Representation> GenerationPlan<R> {
    /// Reached fragments in dependency-first deterministic order.
    pub fn fragments(&self) -> impl ExactSizeIterator<Item = &FragmentPlan<R>> {
        self.fragment_order.iter().map(|id| &self.fragments[id])
    }

    /// Look up a reached fragment by semantic identity.
    pub fn fragment(&self, id: &FragmentId) -> Option<&FragmentPlan<R>> {
        self.fragments.get(id)
    }

    /// Sites in deterministic semantic order.
    pub fn sites(&self) -> impl ExactSizeIterator<Item = &SitePlan<R>> {
        self.site_order.iter().map(|id| &self.sites[id])
    }

    /// Look up one site plan.
    pub fn site(&self, id: &SiteId) -> Option<&SitePlan<R>> {
        self.sites.get(id)
    }

    /// Final artifacts in prerequisite-first deterministic order.
    pub fn artifacts(&self) -> impl ExactSizeIterator<Item = &ArtifactPlan<R>> {
        self.artifact_order.iter().map(|id| &self.artifacts[id])
    }

    /// Look up one artifact plan.
    pub fn artifact(&self, id: &ArtifactId) -> Option<&ArtifactPlan<R>> {
        self.artifacts.get(id)
    }
}

/// All errors discovered before a plan is frozen.
#[derive(Debug)]
pub struct PlanErrors(Vec<PlanError>);

impl PlanErrors {
    /// Individual typed validation errors.
    pub fn errors(&self) -> &[PlanError] {
        &self.0
    }
}

impl fmt::Display for PlanErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "generation plan has {} error(s)", self.0.len())?;
        for error in &self.0 {
            write!(f, ":\n- {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for PlanErrors {}

/// One syntax-free generation-plan validation failure.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    DuplicateFragment(FragmentId),
    DuplicateSite(SiteId),
    DuplicateArtifact(ArtifactId),
    FragmentSpelling(FragmentId),
    FragmentCrossing(FragmentId),
    YieldType(FragmentId),
    EmptyConversionChain(FragmentId),
    BrokenConversionChain(FragmentId),
    UnreportedStepFailure(FragmentId),
    Arity(FragmentId),
    InsufficientNiches(FragmentId),
    OverlappingNiches(FragmentId),
    TransferOfBorrowed(FragmentId),
    UnknownChild {
        parent: FragmentId,
        child: FragmentId,
    },
    ChildDirection(FragmentId),
    ContractType(ContractAt),
    ContractMode(ContractAt),
    ContractValidity(ContractAt),
    SiteIdentity(SiteId),
    UnknownSiteFragment(SiteId),
    SiteDirection(SiteId),
    SiteSpelling(SiteId),
    SiteRecipe(SiteId),
    MissingFailureRoute(SiteId),
    UnexpectedFailureRoute(SiteId),
    SiteTransferOfBorrowed(SiteId),
    UnknownPrerequisite {
        artifact: ArtifactId,
        prerequisite: ArtifactId,
    },
    UnknownArtifactFragment(ArtifactId),
    UnknownArtifactSite(ArtifactId),
    /// An artifact follows a fragment the plan does not hold.
    UnknownFollowedFragment {
        artifact: ArtifactId,
        fragment: FragmentId,
    },
    /// A declared reachability root names a fragment the plan does not hold.
    UnknownRoot(FragmentId),
    AbiArity(ArtifactId, SiteId),
    FragmentCycle(FragmentId),
    ArtifactCycle(ArtifactId),
}

/// Location of a source-value contract mismatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractAt {
    /// A recursive part edge in this fragment.
    Fragment(FragmentId),
    /// This boundary site.
    Site(SiteId),
}

impl fmt::Display for ContractAt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fragment(id) => id.fmt(f),
            Self::Site(id) => write!(f, "site {id}"),
        }
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PlanError::*;
        match self {
            DuplicateFragment(id) => write!(f, "duplicate {id}"),
            DuplicateSite(id) => write!(f, "duplicate site {id}"),
            DuplicateArtifact(id) => write!(f, "duplicate artifact {id}"),
            FragmentSpelling(id) => write!(f, "{id} stores a different source spelling"),
            FragmentCrossing(id) => write!(f, "{id} stores a source type outside its crossing"),
            YieldType(id) => write!(f, "{id} yields a type outside its crossing"),
            EmptyConversionChain(id) => write!(f, "{id} declares an empty conversion chain"),
            BrokenConversionChain(id) => {
                write!(
                    f,
                    "{id} has a disconnected or misplaced conversion-chain endpoint"
                )
            }
            UnreportedStepFailure(id) => {
                write!(
                    f,
                    "{id} contains a fallible step but is declared infallible"
                )
            }
            Arity(id) => write!(f, "{id} operation arity does not match its children"),
            InsufficientNiches(id) => write!(f, "{id} has too few niches for its discriminants"),
            OverlappingNiches(id) => write!(f, "{id} consumes or exposes a niche more than once"),
            TransferOfBorrowed(id) => write!(f, "{id} cannot transfer a borrowed yield"),
            UnknownChild { parent, child } => {
                write!(f, "{parent} references unknown child {child}")
            }
            ChildDirection(id) => write!(f, "{id} has a child in the wrong direction"),
            ContractType(at) => write!(f, "{at} has a child with the wrong type"),
            ContractMode(at) => write!(f, "{at} has a child with insufficient ownership"),
            ContractValidity(at) => write!(f, "{at} has a child with insufficient validity"),
            SiteIdentity(id) => write!(f, "site plan {id} stores a different bound site"),
            UnknownSiteFragment(id) => write!(f, "site {id} references an unknown fragment"),
            SiteDirection(id) => write!(f, "site {id} and its fragment have different directions"),
            SiteSpelling(id) => write!(f, "site {id} and its fragment use different spellings"),
            SiteRecipe(id) => write!(f, "site {id} and its fragment select different recipes"),
            MissingFailureRoute(id) => write!(f, "fallible site {id} has no failure route"),
            UnexpectedFailureRoute(id) => write!(f, "infallible site {id} has a failure route"),
            SiteTransferOfBorrowed(id) => write!(f, "site {id} cannot transfer a borrowed yield"),
            UnknownPrerequisite {
                artifact,
                prerequisite,
            } => write!(
                f,
                "artifact {artifact} references unknown prerequisite {prerequisite}"
            ),
            UnknownArtifactFragment(id) => {
                write!(f, "artifact {id} references an unknown fragment")
            }
            UnknownArtifactSite(id) => write!(f, "artifact {id} references an unknown site"),
            UnknownFollowedFragment { artifact, fragment } => {
                write!(f, "artifact {artifact} follows unknown fragment {fragment}")
            }
            UnknownRoot(id) => write!(f, "declared root names unknown fragment {id}"),
            AbiArity(artifact, site) => write!(
                f,
                "artifact {artifact} disagrees with the ABI arity of site {site}"
            ),
            FragmentCycle(id) => write!(f, "fragment dependency cycle reaches {id}"),
            ArtifactCycle(id) => write!(f, "artifact dependency cycle reaches {id}"),
        }
    }
}

fn validate_yield(errors: &mut Vec<PlanError>, at: ContractAt, got: &Yield, need: &Yield) {
    if got.ty != need.ty {
        errors.push(PlanError::ContractType(at.clone()));
    }
    if !got.mode.satisfies(need.mode) {
        errors.push(PlanError::ContractMode(at.clone()));
    }
    if !got.validity.satisfies(need.validity) {
        errors.push(PlanError::ContractValidity(at));
    }
}

fn validate_chain<R: Representation>(errors: &mut Vec<PlanError>, fragment: &FragmentPlan<R>) {
    let id = fragment.id();
    let converter = fragment.converter();
    let ConversionChain::Steps(steps) = converter.chain() else {
        return;
    };
    if steps.is_empty() {
        errors.push(PlanError::EmptyConversionChain(id.clone()));
        return;
    }

    let (start, end) = match id.direction() {
        Direction::Construct => (
            ChainValue::Intermediate(fragment.intermediate().clone()),
            ChainValue::Source,
        ),
        Direction::Deconstruct => (
            ChainValue::Source,
            ChainValue::Intermediate(fragment.intermediate().clone()),
        ),
    };
    let mut cursor = start;
    let mut broken = false;
    let mut unreported_failure = false;
    for (index, step) in steps.iter().enumerate() {
        if step.from() != &cursor {
            broken = true;
        }
        if matches!(step.from(), ChainValue::Source)
            && !(id.direction() == Direction::Deconstruct && index == 0)
        {
            broken = true;
        }
        if matches!(step.into(), ChainValue::Source)
            && !(id.direction() == Direction::Construct && index + 1 == steps.len())
        {
            broken = true;
        }
        if step.failure() == Failure::Fallible && converter.failure() == Failure::Infallible {
            unreported_failure = true;
        }
        cursor = step.into().clone();
    }
    if cursor != end {
        broken = true;
    }
    if broken {
        errors.push(PlanError::BrokenConversionChain(id.clone()));
    }
    if unreported_failure {
        errors.push(PlanError::UnreportedStepFailure(id.clone()));
    }
}

fn topo_fragments<R: Representation>(
    plans: &HashMap<FragmentId, FragmentPlan<R>>,
    errors: &mut Vec<PlanError>,
) -> Vec<FragmentId> {
    let mut roots: Vec<_> = plans.keys().cloned().collect();
    roots.sort_by_cached_key(FragmentId::stable_key);
    let mut state = HashMap::new();
    let mut order = Vec::new();
    for root in roots {
        visit_fragment(&root, plans, &mut state, &mut order, errors);
    }
    order
}

fn visit_fragment<R: Representation>(
    id: &FragmentId,
    plans: &HashMap<FragmentId, FragmentPlan<R>>,
    state: &mut HashMap<FragmentId, u8>,
    order: &mut Vec<FragmentId>,
    errors: &mut Vec<PlanError>,
) {
    match state.get(id) {
        Some(1) => {
            errors.push(PlanError::FragmentCycle(id.clone()));
            return;
        }
        Some(2) => return,
        _ => {}
    }
    state.insert(id.clone(), 1);
    if let Some(plan) = plans.get(id) {
        let mut children: Vec<_> = plan
            .converter()
            .shape()
            .uses()
            .into_iter()
            .map(|usage| usage.fragment().clone())
            .filter(|child| plans.contains_key(child))
            .collect();
        children.sort_by_cached_key(FragmentId::stable_key);
        children.dedup();
        for child in children {
            visit_fragment(&child, plans, state, order, errors);
        }
    }
    state.insert(id.clone(), 2);
    order.push(id.clone());
}

/// Artifacts in emission order: the order they were declared in, with each
/// artifact's prerequisites hoisted ahead of it.
fn topo_artifacts<R: Representation>(
    plans: &HashMap<ArtifactId, ArtifactPlan<R>>,
    declared: &[ArtifactId],
    errors: &mut Vec<PlanError>,
) -> Vec<ArtifactId> {
    fn visit<R: Representation>(
        id: &ArtifactId,
        plans: &HashMap<ArtifactId, ArtifactPlan<R>>,
        state: &mut HashMap<ArtifactId, u8>,
        order: &mut Vec<ArtifactId>,
        errors: &mut Vec<PlanError>,
    ) {
        match state.get(id) {
            Some(1) => {
                errors.push(PlanError::ArtifactCycle(id.clone()));
                return;
            }
            Some(2) => return,
            _ => {}
        }
        state.insert(id.clone(), 1);
        if let Some(plan) = plans.get(id) {
            let mut dependencies = plan.prerequisites().to_vec();
            dependencies.sort();
            dependencies.dedup();
            for dependency in dependencies {
                if plans.contains_key(&dependency) {
                    visit(&dependency, plans, state, order, errors);
                }
            }
        }
        state.insert(id.clone(), 2);
        order.push(id.clone());
    }

    let mut state = HashMap::new();
    let mut order = Vec::new();
    for root in declared {
        visit(root, plans, &mut state, &mut order, errors);
    }
    order
}

fn reachable_fragments<R: Representation>(
    plans: &HashMap<FragmentId, FragmentPlan<R>>,
    roots: Vec<FragmentId>,
) -> HashSet<FragmentId> {
    let mut reached = HashSet::new();
    let mut pending = roots;
    while let Some(id) = pending.pop() {
        // Only an id the plan holds enters the reached set. Inserting first and
        // looking up after would let an unknown id be "reached", which is a
        // claim about a fragment that does not exist — validation rejects such
        // an id, and this makes the walk itself not depend on that (#660 review).
        let Some(plan) = plans.get(&id) else {
            continue;
        };
        if !reached.insert(id) {
            continue;
        }
        pending.extend(
            plan.converter()
                .shape()
                .uses()
                .into_iter()
                .map(|usage| usage.fragment().clone()),
        );
    }
    reached
}

/// Every production Rust source under `dir`, as (label, text) pairs.
///
/// A **production** source is one that is not test support: anything under a
/// `tests/` directory, or named `tests.rs` or `test_util.rs`, is skipped. The
/// label is the path relative to `dir`'s parent, which is what a fence prints
/// when it names where it found something.
///
/// Discovered by walking the directory rather than by a list a test carries,
/// because a fence over a list only fences the files someone remembered to add
/// to it — and a new module is the most natural way to introduce the thing a
/// fence exists to reject.
#[cfg(any(test, feature = "testing"))]
pub fn production_sources(dir: &std::path::Path) -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("read source directory {}: {error}", dir.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|error| {
                        panic!("read source entry under {}: {error}", dir.display())
                    })
                    .path()
            })
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "tests") {
                    continue;
                }
                walk(&path, root, out);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            if path
                .file_name()
                .is_some_and(|name| name == "tests.rs" || name == "test_util.rs")
            {
                continue;
            }
            let label = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read source {}: {error}", path.display()));
            out.push((label, text));
        }
    }

    let mut out = Vec::new();
    let root = dir.parent().unwrap_or(dir);
    walk(dir, root, &mut out);
    out
}

/// The names of the shape-shaped enums a set of sources declares.
///
/// A **shape-shaped** enum is one whose variants name three or more of the
/// structural forms this crate's model already has words for — `Atomic`,
/// `Product`, `Optional`, `Sequence`, `Choice`, `Invoke`, `Leaf`. Each such
/// enum is a place where the same structural question is asked again, and #613
/// exists to reduce their number rather than let it grow: an adapter fences
/// this against the list it has, so a new one fails a test and a deleted one is
/// a deliberate edit to that list.
///
/// `sources` pairs a label — the path a reader should open — with the file's
/// text, since a fence names what it found.
#[cfg(any(test, feature = "testing"))]
pub fn shape_like_enums(sources: &[(&str, &str)]) -> Vec<(String, String)> {
    const FORMS: [&str; 7] = [
        "Atomic", "Product", "Optional", "Sequence", "Choice", "Invoke", "Leaf",
    ];
    let mut found = Vec::new();
    for (label, source) in sources {
        let file: syn::File = match syn::parse_str(source) {
            Ok(file) => file,
            // A fence reports what it could not read rather than passing
            // silently on a file it failed to parse.
            Err(error) => panic!("shape-enum fence cannot parse {label}: {error}"),
        };
        // Inline modules too: an enum does not stop being a second shape
        // vocabulary by being declared one `mod` deeper.
        fn walk(items: &[syn::Item], forms: &[&str], label: &str, out: &mut Vec<(String, String)>) {
            for item in items {
                match item {
                    syn::Item::Enum(item) => {
                        let named = item
                            .variants
                            .iter()
                            .filter(|variant| forms.contains(&variant.ident.to_string().as_str()))
                            .count();
                        if named >= 3 {
                            out.push((item.ident.to_string(), label.to_string()));
                        }
                    }
                    syn::Item::Mod(module) => {
                        if let Some((_, items)) = &module.content {
                            walk(items, forms, label, out);
                        }
                    }
                    _ => {}
                }
            }
        }
        walk(&file.items, &FORMS, label, &mut found);
    }
    found.sort();
    found
}
