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
    /// A syntax-free identity for a private Rust carrier in a converter graph.
    type Intermediate: Clone + Eq;
    /// One adapter-declared conversion between two graph values.
    type Step;
    /// Terminal conversion at an [`Atomic`](ShapePlan::Atomic) shape.
    type TerminalCodec;
    /// Packing or unpacking a fixed product.
    type ProductBridge;
    /// Absent/present control flow.
    type OptionalBridge;
    /// Builder or traversal control flow for a sequence.
    type SequenceBridge;
    /// Arm/tag control flow for a choice.
    type ChoiceBridge;
    /// Callable construction and invocation control flow.
    type CallbackBridge;
    /// One semantic niche identity. Equal values denote the same bit domain.
    type Niche: Clone + Eq + Hash;
    /// A cleanup operation.
    type Cleanup;
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
pub struct ConverterStep<R: Representation> {
    from: ChainValue<R::Intermediate>,
    into: ChainValue<R::Intermediate>,
    operation: R::Step,
    failure: Failure,
    cleanup: Cleanup<R::Cleanup>,
}

impl<R: Representation> ConverterStep<R> {
    /// Describe one directional conversion step.
    pub fn new(
        from: ChainValue<R::Intermediate>,
        into: ChainValue<R::Intermediate>,
        operation: R::Step,
        failure: Failure,
        cleanup: Cleanup<R::Cleanup>,
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
    pub fn from(&self) -> &ChainValue<R::Intermediate> {
        &self.from
    }

    /// Value produced by this step.
    pub fn into(&self) -> &ChainValue<R::Intermediate> {
        &self.into
    }

    /// Adapter-owned semantic operation.
    pub fn operation(&self) -> &R::Step {
        &self.operation
    }

    /// Whether this step can fail.
    pub fn failure(&self) -> Failure {
        self.failure
    }

    /// Cleanup attached to this step's failure and success edges.
    pub fn cleanup(&self) -> &Cleanup<R::Cleanup> {
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
pub enum ConversionChain<R: Representation> {
    /// No adapter conversion lies between the shape and source value.
    Direct,
    /// One or more explicit internal conversions.
    Steps(Vec<ConverterStep<R>>),
}

impl<R: Representation> ConversionChain<R> {
    /// Explicit steps in execution order.
    pub fn steps(&self) -> &[ConverterStep<R>] {
        match self {
            Self::Direct => &[],
            Self::Steps(steps) => steps,
        }
    }
}

/// The registry-composed converter operation for one fragment.
pub enum ShapePlan<R: Representation> {
    /// Convert one wire leaf to or from the fragment's intermediate.
    ///
    /// The codec remains present when the fragment has a staged
    /// [`ConversionChain`]; it may be an identity operation at this boundary.
    Atomic(R::TerminalCodec),
    /// Pack or unpack all fixed positions.
    Product {
        /// Adapter representation operation.
        bridge: FixedArity<R::ProductBridge>,
        /// Ordered source parts.
        parts: Vec<FragmentUse>,
    },
    /// Absent/present control flow around one value.
    Optional {
        /// Adapter representation operation.
        bridge: R::OptionalBridge,
        /// The present value.
        value: FragmentUse,
    },
    /// Builder or traversal control flow around one element type.
    Sequence {
        /// Adapter representation operation.
        bridge: R::SequenceBridge,
        /// The repeated element.
        element: FragmentUse,
    },
    /// Tagged selection among ordered arms.
    Choice {
        /// Adapter representation operation and arm contracts.
        bridge: ChoiceArity<R::ChoiceBridge>,
        /// Parts in every arm, in tag and then position order.
        arms: Vec<Vec<FragmentUse>>,
    },
    /// Foreign callable construction and later argument delivery.
    Invoke {
        /// Adapter callable operation.
        bridge: FixedArity<R::CallbackBridge>,
        /// Callback arguments. Their direction is opposite the callable's.
        arguments: Vec<FragmentUse>,
    },
}

impl<R: Representation> ShapePlan<R> {
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
pub struct ConverterPlan<R: Representation> {
    shape: ShapePlan<R>,
    chain: ConversionChain<R>,
    niches: NichePlan<R::Niche>,
    failure: Failure,
    cleanup: Cleanup<R::Cleanup>,
}

impl<R: Representation> ConverterPlan<R> {
    /// Freeze one converter operation graph.
    pub fn new(
        shape: ShapePlan<R>,
        niches: NichePlan<R::Niche>,
        failure: Failure,
        cleanup: Cleanup<R::Cleanup>,
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
        shape: ShapePlan<R>,
        chain: ConversionChain<R>,
        niches: NichePlan<R::Niche>,
        failure: Failure,
        cleanup: Cleanup<R::Cleanup>,
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
    pub fn chain(&self) -> &ConversionChain<R> {
        &self.chain
    }

    /// The selected shape and representation operation.
    pub fn shape(&self) -> &ShapePlan<R> {
        &self.shape
    }

    /// Consumed and exposed niche domains.
    pub fn niches(&self) -> &NichePlan<R::Niche> {
        &self.niches
    }

    /// Whether this graph has a failure edge.
    pub fn failure(&self) -> Failure {
        self.failure
    }

    /// Cleanup attached to this graph.
    pub fn cleanup(&self) -> &Cleanup<R::Cleanup> {
        &self.cleanup
    }
}

/// Immutable plan for one selected, spelled recipe fragment.
pub struct FragmentPlan<R: Representation> {
    id: FragmentId,
    source: TypeRef,
    intermediate: R::Intermediate,
    converter: ConverterPlan<R>,
    yields: Yield,
}

impl<R: Representation> FragmentPlan<R> {
    /// Describe one fragment completely, without rendering it.
    pub fn new(
        id: FragmentId,
        source: TypeRef,
        intermediate: R::Intermediate,
        converter: ConverterPlan<R>,
        yields: Yield,
    ) -> Self {
        Self {
            id,
            source,
            intermediate,
            converter,
            yields,
        }
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
    pub fn intermediate(&self) -> &R::Intermediate {
        &self.intermediate
    }

    /// Registry-composed converter operation graph.
    pub fn converter(&self) -> &ConverterPlan<R> {
        &self.converter
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
    cleanup: Cleanup<R::Cleanup>,
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
        cleanup: Cleanup<R::Cleanup>,
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
    pub fn cleanup(&self) -> &Cleanup<R::Cleanup> {
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
            payload,
        }
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
    errors: Vec<PlanError>,
}

impl<R: Representation> Default for GenerationPlanBuilder<R> {
    fn default() -> Self {
        Self {
            fragments: HashMap::new(),
            sites: HashMap::new(),
            artifacts: HashMap::new(),
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
    pub fn artifact(&mut self, plan: ArtifactPlan<R>) -> &mut Self {
        let id = plan.id.clone();
        if self.artifacts.insert(id.clone(), plan).is_some() {
            self.errors.push(PlanError::DuplicateArtifact(id));
        }
        self
    }

    /// Validate, compute deterministic dependency order, and freeze the plan.
    pub fn build(mut self) -> Result<GenerationPlan<R>, PlanErrors> {
        self.validate_fragments();
        self.validate_sites();
        self.validate_artifacts();
        let fragment_order = topo_fragments(&self.fragments, &mut self.errors);
        let artifact_order = topo_artifacts(&self.artifacts, &mut self.errors);
        if !self.errors.is_empty() {
            return Err(PlanErrors(self.errors));
        }

        let mut roots: Vec<_> = self.sites.values().map(|s| s.fragment.clone()).collect();
        roots.extend(self.artifacts.values().flat_map(|artifact| {
            artifact.inputs.iter().filter_map(|input| match input {
                ArtifactInput::Fragment(id) => Some(id.clone()),
                ArtifactInput::Site { .. } => None,
            })
        }));
        roots.sort_by_cached_key(FragmentId::stable_key);
        roots.dedup();
        let reachable = reachable_fragments(&self.fragments, roots);
        let fragment_order = fragment_order
            .into_iter()
            .filter(|id| reachable.contains(id))
            .collect();
        self.fragments.retain(|id, _| reachable.contains(id));

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

fn topo_artifacts<R: Representation>(
    plans: &HashMap<ArtifactId, ArtifactPlan<R>>,
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

    let mut roots: Vec<_> = plans.keys().cloned().collect();
    roots.sort();
    let mut state = HashMap::new();
    let mut order = Vec::new();
    for root in roots {
        visit(&root, plans, &mut state, &mut order, errors);
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
        if !reached.insert(id.clone()) {
            continue;
        }
        if let Some(plan) = plans.get(&id) {
            pending.extend(
                plan.converter()
                    .shape()
                    .uses()
                    .into_iter()
                    .map(|usage| usage.fragment().clone()),
            );
        }
    }
    reached
}
