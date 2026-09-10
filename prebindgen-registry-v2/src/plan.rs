//! Planning: requests in, retained plans out.
//!
//! One recursive walk plans the value conversions, one
//! pass per requested output assembles the native wrappers, and one fixpoint
//! decides what survives. The registry owns all three; the target answers the
//! local questions in [`crate::target`].

use std::collections::{BTreeMap, HashMap};

use prebindgen_flat::flat::{Flat, Function, Struct, Type, TypeKind, TypeRef};

use crate::{
    body::{BodyBuilder, Instr, NodeBody, Operand, ValueId},
    decl::{DeclaredElement, ElementId, ElementKind},
    outcome::{EngineError, Outcome, Skip},
    report::{sort_entries, Entry, Report, SourceIdentity, SCHEMA_VERSION},
    run::{check_declarations, Generation, PIPELINE},
    target::{
        AbiSpec, ChildValue, Crossing, Direction, FailureCategory, FailureRoute, OperandRole,
        OutputPlacement, ParamRole, Part, PlanningError, PrimitiveId, PrimitiveSpec, Protocol,
        RecordRelation, Relation, RelationId, ResolvedShape, ResolvedValues, SelectionQuery,
        SiteDescriptor, SourceItem, SurfaceRequest, SurfaceSpec, Target, TargetAttempt,
        Unsupported,
    },
};

/// A configuration entry the target interprets. Valid inside one request set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PolicyId(pub usize);

/// One requested output.
#[derive(Clone, Debug)]
pub struct OutputRequest {
    pub element: DeclaredElement,
    /// The target's configuration for this output.
    pub policy: PolicyId,
}

/// What a frontend hands the engine: what to expose, and the choices that
/// apply.
///
/// Users never write this; a frontend builds it from its own recorded calls,
/// which is what lets two languages share everything after this point.
pub struct BindingRequests<Policy> {
    /// The target this binding generates for — `"c"`, `"jni"`.
    pub target: &'static str,
    /// The module generated code reaches the source items through.
    pub source_module: syn::Path,
    pub outputs: Vec<OutputRequest>,
    /// Configurations referenced by [`PolicyId`].
    pub policies: Vec<Policy>,
    /// The configuration for a value nothing more specific covers.
    pub default_policy: PolicyId,
    /// Per-source-type defaults: `Stamp` crosses this way wherever it appears.
    pub type_policies: BTreeMap<String, PolicyId>,
    /// Per-site overrides: this parameter of this exported function crosses
    /// differently. Keyed by element and the site's path, `param 0` or
    /// `return`.
    pub site_policies: BTreeMap<(ElementId, String), PolicyId>,
    /// Elements the user asked to leave alone.
    pub ignored: Vec<DeclaredElement>,
}

impl<Policy> BindingRequests<Policy> {
    /// A request set with one policy, which every value uses unless something
    /// more specific is recorded.
    pub fn new(target: &'static str, source_module: syn::Path, default: Policy) -> Self {
        BindingRequests {
            target,
            source_module,
            outputs: Vec::new(),
            policies: vec![default],
            default_policy: PolicyId(0),
            type_policies: BTreeMap::new(),
            site_policies: BTreeMap::new(),
            ignored: Vec::new(),
        }
    }

    /// Record a configuration and get its identity.
    ///
    /// Two values share a conversion only when they share the entry, never
    /// because two entries look alike — a policy can hold a closure, and two
    /// closures cannot be compared.
    pub fn policy(&mut self, policy: Policy) -> PolicyId {
        self.policies.push(policy);
        PolicyId(self.policies.len() - 1)
    }

    /// Ask for one element to be exposed.
    pub fn output(&mut self, element: DeclaredElement, policy: PolicyId) -> &mut Self {
        self.outputs.push(OutputRequest { element, policy });
        self
    }

    pub(crate) fn get(&self, id: PolicyId) -> &Policy {
        &self.policies[id.0]
    }
}

/// A retained conversion, reusable by every value that crosses the same way.
#[derive(Debug)]
pub struct ValuePlan<P> {
    pub id: NodeId,
    pub crossing: Crossing,
    pub relation: RelationId,
    pub repr: crate::target::ReprSpec<P>,
    pub children: Vec<NodeId>,
    pub body: NodeBody,
    /// Every failure category this conversion, or a conversion it uses, can
    /// raise. The boundary must route each of them.
    pub failures: Vec<FailureCategory>,
}

/// A retained conversion's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) usize);

/// A complete native wrapper.
#[derive(Debug)]
pub struct FunctionPlan<P> {
    pub element: ElementId,
    pub abi: AbiSpec,
    pub output: OutputPlacement,
    pub failures: Vec<FailureRoute<P>>,
    /// Native parameters, paired with the value identity that names each.
    pub params: Vec<(ValueId, crate::target::NativeParam)>,
    pub instrs: Vec<Instr>,
    /// The value delivered through [`Self::output`].
    pub result: Option<ValueId>,
}

/// What a conversion attempt is, while and after it is planned.
enum NodeState {
    /// Being resolved: meeting this again is a cycle.
    InProgress,
    Ready(NodeId),
    Unsupported(Unsupported),
}

#[derive(PartialEq, Eq, Hash)]
struct NodeKey {
    ty: prebindgen_flat::TypeKey,
    direction: Direction,
    relation: RelationId,
    policy: PolicyId,
}

/// A planned value, or the capability that stopped it.
enum Planned {
    Ready(NodeId),
    Unsupported(Unsupported),
}

/// The working state of one `generate` call.
struct Run<'a, T: Target> {
    flat: &'a Flat,
    target: &'a T,
    requests: &'a BindingRequests<T::Policy>,
    relations: Vec<Relation>,
    primitives: Vec<PrimitiveSpec<T::Payload>>,
    nodes: Vec<ValuePlan<T::Payload>>,
    cache: HashMap<NodeKey, NodeState>,
    /// The relations offered for a type, registered the first time it is
    /// planned. A relation's identity is part of a conversion's identity, so
    /// registering a fresh one per visit would give every value its own node
    /// and share nothing.
    offered: HashMap<prebindgen_flat::TypeKey, Vec<(RelationId, Relation)>>,
}

impl<'a, T: Target> Run<'a, T> {
    fn new(flat: &'a Flat, target: &'a T, requests: &'a BindingRequests<T::Policy>) -> Self {
        Run {
            flat,
            target,
            requests,
            relations: Vec::new(),
            primitives: Vec::new(),
            nodes: Vec::new(),
            cache: HashMap::new(),
            offered: HashMap::new(),
        }
    }

    /// The configuration for a value at this position: the site override, then
    /// the type default, then the request set's default.
    fn effective_policy(&self, position: &crate::target::Position, ty: &TypeRef) -> PolicyId {
        let requests = self.requests;
        let site = position.path.join(".");
        if let Some(policy) = requests
            .site_policies
            .get(&(position.element.clone(), site))
        {
            return *policy;
        }
        if let Some(name) = type_name(ty) {
            if let Some(policy) = requests.type_policies.get(&name) {
                return *policy;
            }
        }
        requests.default_policy
    }

    /// Register an operation, and hand back the identity instructions name it
    /// by.
    fn register(&mut self, primitive: PrimitiveSpec<T::Payload>) -> PrimitiveId {
        self.primitives.push(primitive);
        PrimitiveId(self.primitives.len() - 1)
    }

    /// The relations available for this type: today the implicit one, which is
    /// the record's fields for a record and the atomic conversion for anything
    /// carried whole.
    fn candidates(&mut self, ty: &TypeRef) -> Result<Vec<(RelationId, Relation)>, Unsupported> {
        if let Some(offered) = self.offered.get(&ty.key()) {
            return Ok(offered.clone());
        }
        let mut relations = vec![Relation::Atomic];
        if let TypeKind::Named { id, .. } = ty.kind() {
            match self.flat.resolve(id) {
                Some(Type::Struct(record)) => relations.push(Relation::Record(RecordRelation {
                    record: record.name.to_string(),
                    parts: record
                        .fields
                        .iter()
                        .map(|field| Part {
                            name: field.name.as_ref().map(|name| name.to_string()),
                            index: field.index,
                            ty: field.ty.clone(),
                        })
                        .collect(),
                })),
                // An opaque declaration has no parts to offer: a target that
                // carries it does so whole, through the atomic relation.
                Some(_) => {}
                None => {
                    return Err(Unsupported::new(
                        "unsupported.type.undeclared",
                        format!("`{}` names a type this model does not declare", ty.key()),
                    ))
                }
            }
        }
        let offered: Vec<(RelationId, Relation)> = relations
            .into_iter()
            .enumerate()
            .map(|(index, relation)| (RelationId(self.relations.len() + index), relation))
            .collect();
        self.relations
            .extend(offered.iter().map(|(_, relation)| relation.clone()));
        self.offered.insert(ty.key(), offered.clone());
        Ok(offered)
    }

    /// Plan one conversion, recursing into the parts of whatever relation the
    /// target selects.
    fn plan_value(
        &mut self,
        crossing: Crossing,
        position: &crate::target::Position,
    ) -> Result<Planned, PlanningError> {
        let target = self.target;
        let requests = self.requests;
        let policy_id = self.effective_policy(position, &crossing.ty);

        let candidates = match self.candidates(&crossing.ty) {
            Ok(candidates) => candidates,
            Err(unsupported) => return Ok(Planned::Unsupported(unsupported)),
        };

        let query = SelectionQuery {
            crossing: &crossing,
            position,
            candidates: &candidates,
            policy: requests.get(policy_id),
        };
        let relation_id = match target.select(&query)? {
            TargetAttempt::Ready(relation) => relation,
            TargetAttempt::Unsupported(unsupported) => {
                return Ok(Planned::Unsupported(unsupported))
            }
        };
        let relation = match candidates
            .iter()
            .find(|(id, _)| *id == relation_id)
            .map(|(_, relation)| relation.clone())
        {
            Some(relation) => relation,
            None => {
                return Err(PlanningError::InternalInvariant(format!(
                    "the target selected a relation that was not offered for `{}`",
                    crossing.ty.key()
                )))
            }
        };

        let key = NodeKey {
            ty: crossing.ty.key(),
            direction: crossing.direction,
            relation: relation_id,
            policy: policy_id,
        };
        match self.cache.get(&key) {
            Some(NodeState::Ready(id)) => return Ok(Planned::Ready(*id)),
            Some(NodeState::Unsupported(unsupported)) => {
                return Ok(Planned::Unsupported(unsupported.clone()))
            }
            Some(NodeState::InProgress) => {
                return Ok(Planned::Unsupported(Unsupported::new(
                    "unsupported.conversion.recursive",
                    format!(
                        "converting `{}` needs its own conversion; v2 has no recursive \
                         conversion yet",
                        crossing.ty.key()
                    ),
                )))
            }
            None => {}
        }
        self.cache.insert(
            NodeKey {
                ty: crossing.ty.key(),
                direction: crossing.direction,
                relation: relation_id,
                policy: policy_id,
            },
            NodeState::InProgress,
        );

        let planned = self.plan_relation(&crossing, position, relation_id, &relation, policy_id);
        let state = match &planned {
            Ok(Planned::Ready(id)) => NodeState::Ready(*id),
            Ok(Planned::Unsupported(unsupported)) => NodeState::Unsupported(unsupported.clone()),
            Err(_) => NodeState::InProgress,
        };
        self.cache.insert(key, state);
        planned
    }

    /// The body of `plan_value`, once the relation is known: children first,
    /// then the representation, then the instructions that put them together.
    fn plan_relation(
        &mut self,
        crossing: &Crossing,
        position: &crate::target::Position,
        relation_id: RelationId,
        relation: &Relation,
        policy_id: PolicyId,
    ) -> Result<Planned, PlanningError> {
        let target = self.target;
        let requests = self.requests;
        let parts = relation.parts().to_vec();

        let mut children = Vec::new();
        for part in &parts {
            let child = Crossing {
                ty: part.ty.clone(),
                direction: crossing.direction,
            };
            let child_position = position.child(format!("field {}", part.label()));
            match self.plan_value(child, &child_position)? {
                Planned::Ready(id) => children.push(id),
                // One unsupported part makes the whole conversion unsupported.
                // Nothing partial is recorded: a record missing a field is a
                // different type, not a reduced one.
                Planned::Unsupported(unsupported) => return Ok(Planned::Unsupported(unsupported)),
            }
        }

        let record = match relation {
            Relation::Record(record) => self.flat.struct_type(record.record.as_str()),
            Relation::Atomic => None,
        };
        let shape = ResolvedShape {
            crossing,
            position,
            relation,
            record,
        };
        let child_values: Vec<ChildValue<'_>> = parts
            .iter()
            .zip(&children)
            .map(|(part, id)| {
                let child = &self.nodes[id.0];
                ChildValue {
                    part,
                    layout: &child.repr.layout,
                    fallible: !child.failures.is_empty(),
                }
            })
            .collect();
        let repr = match target.represent(&shape, &child_values, requests.get(policy_id))? {
            TargetAttempt::Ready(repr) => repr,
            TargetAttempt::Unsupported(unsupported) => {
                return Ok(Planned::Unsupported(unsupported))
            }
        };
        drop(child_values);

        // Compose: obtain each part, convert it, construct the Rust value — or,
        // out of Rust, the target value.
        let mut failures: Vec<FailureCategory> = children
            .iter()
            .flat_map(|id| self.nodes[id.0].failures.clone())
            .collect();
        let mut body = BodyBuilder::new();
        let carrier = body.fresh();
        let result = match (&repr.protocol, crossing.direction) {
            (Protocol::Terminal { codec }, _) => {
                if codec.value_operands() != 1 {
                    return Err(PlanningError::InternalInvariant(
                        "a terminal conversion takes exactly one value operand".to_string(),
                    ));
                }
                failures.extend(codec.failure_category());
                self.apply(&mut body, (**codec).clone(), &[carrier])
            }
            (Protocol::Product { projections }, Direction::IntoRust) => {
                if projections.len() != parts.len() {
                    return Err(PlanningError::InternalInvariant(format!(
                        "the representation of `{}` describes {} projection(s) for {} part(s)",
                        crossing.ty.key(),
                        projections.len(),
                        parts.len()
                    )));
                }
                let record = match relation {
                    Relation::Record(record) => record.record.clone(),
                    Relation::Atomic => {
                        return Err(PlanningError::InternalInvariant(
                            "a product representation needs a relation with parts".to_string(),
                        ))
                    }
                };
                let mut converted = Vec::new();
                for (projection, child) in projections.iter().zip(&children) {
                    if projection.value_operands() != 1 {
                        return Err(PlanningError::InternalInvariant(
                            "a projection takes exactly one value operand".to_string(),
                        ));
                    }
                    failures.extend(projection.failure_category());
                    let obtained = self.apply(&mut body, projection.clone(), &[carrier]);
                    let obtained = match obtained {
                        Some(value) => value,
                        None => {
                            return Err(PlanningError::InternalInvariant(
                                "a projection must produce a value".to_string(),
                            ))
                        }
                    };
                    let child_body = self.nodes[child.0].body.clone();
                    converted.push(child_body.inline(obtained, &mut body));
                }
                let result = body.fresh();
                body.push(Instr::Construct {
                    record,
                    parts: converted,
                    result,
                });
                Some(result)
            }
            (Protocol::Product { .. }, Direction::OutOfRust) => {
                return Ok(Planned::Unsupported(Unsupported::new(
                    "unsupported.record.out_of_rust",
                    format!(
                        "`{}` leaves Rust as a composed value; v2 has no target construction \
                         operation yet",
                        crossing.ty.key()
                    ),
                )))
            }
        };
        let result = match result {
            Some(result) => result,
            None => {
                return Err(PlanningError::InternalInvariant(
                    "a value conversion must produce a value".to_string(),
                ))
            }
        };

        failures.sort();
        failures.dedup();
        let id = NodeId(self.nodes.len());
        self.nodes.push(ValuePlan {
            id,
            crossing: crossing.clone(),
            relation: relation_id,
            repr,
            children,
            body: NodeBody {
                carrier,
                instrs: body.into_instrs(),
                result,
            },
            failures,
        });
        Ok(Planned::Ready(id))
    }

    /// Apply one operation to already-available values.
    ///
    /// An infallible identity produces no instruction at all: the carrier and
    /// the converted value are one Rust value, so the conversion renders
    /// nothing and the caller keeps using the value it already had.
    fn apply(
        &mut self,
        body: &mut BodyBuilder,
        primitive: PrimitiveSpec<T::Payload>,
        values: &[ValueId],
    ) -> Option<ValueId> {
        if matches!(
            primitive.implementation,
            crate::target::Operation::Standard(crate::target::StandardOp::Identity)
        ) && matches!(
            primitive.failure,
            crate::target::PrimitiveFailure::Infallible
        ) {
            return values.first().copied();
        }
        let mut supplied = values.iter();
        let operands: Vec<Operand> = primitive
            .operands
            .iter()
            .map(|operand| match &operand.role {
                OperandRole::Value => Operand::Value(
                    *supplied
                        .next()
                        .expect("operand arity is checked before applying"),
                ),
                OperandRole::Context(name) => Operand::Context(name.clone()),
                OperandRole::Error => Operand::Context("__error".to_string()),
            })
            .collect();
        let produces = primitive.result.is_some();
        let id = self.register(primitive);
        let result = produces.then(|| body.fresh());
        body.push(Instr::Apply {
            primitive: id,
            operands,
            result,
        });
        result
    }
}

/// The name of a nominal type, for a policy lookup.
fn type_name(ty: &TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        _ => None,
    }
}

/// Plan `requests` over `flat` with `target`, and render what survives.
///
/// Every requested output leaves this with an outcome. A capability the engine
/// or the target has not implemented is a reported skip and the run continues;
/// contradictory input and violated invariants fail.
pub fn generate<T: Target>(
    flat: Flat,
    target: &T,
    requests: BindingRequests<T::Policy>,
    declaring_crate: impl Into<String>,
) -> Result<Generation<T::Payload>, EngineError> {
    let declared: Vec<DeclaredElement> = requests
        .outputs
        .iter()
        .map(|output| output.element.clone())
        .collect();
    check_declarations(&declared, &flat)?;

    let mut run = Run::new(&flat, target, &requests);
    let mut functions: Vec<FunctionPlan<T::Payload>> = Vec::new();
    let mut surfaces: Vec<SurfaceSpec<T::Payload>> = Vec::new();
    let mut outcomes: BTreeMap<ElementId, Outcome> = BTreeMap::new();

    for output in &requests.outputs {
        let element = &output.element;
        let policy = requests.get(output.policy);
        let planned = match element.kind {
            ElementKind::Function => {
                plan_function(&mut run, element, policy).map_err(EngineError::Planning)?
            }
            ElementKind::Type => {
                plan_record(&mut run, element, policy).map_err(EngineError::Planning)?
            }
            // One code per kind rather than one for the whole engine: the
            // report is how the next capability is chosen, and "everything is
            // unsupported" chooses nothing.
            kind => Err(Unsupported::new(
                format!("unsupported.{}.not_implemented", kind.as_str()),
                format!("the v2 engine has no {} lowering yet", kind.as_str()),
            )),
        };
        match planned {
            Ok(Emitted { function, surface }) => {
                if let Some(function) = function {
                    functions.push(function);
                }
                surfaces.push(surface);
                outcomes.insert(element.id.clone(), Outcome::Emitted);
            }
            Err(unsupported) => {
                outcomes.insert(
                    element.id.clone(),
                    Outcome::Skipped(Skip {
                        capability: unsupported.capability,
                        explanation: unsupported.explanation,
                        dependency_path: vec![element.id.to_string()],
                    }),
                );
            }
        }
    }

    // A public declaration can require another one. Propagate until a pass
    // changes nothing: one missing capability, several skipped outputs, each
    // keeping its own path to the cause.
    loop {
        let mut changed = false;
        for surface in &surfaces {
            if !matches!(outcomes.get(&surface.element), Some(Outcome::Emitted)) {
                continue;
            }
            for required in &surface.requires {
                let cause = match outcomes.get(required) {
                    Some(Outcome::Skipped(skip)) => skip.clone(),
                    Some(_) => continue,
                    None => Skip::direct(
                        "unsupported.requirement.unrequested",
                        format!("requires `{required}`, which this binding does not declare"),
                        required.to_string(),
                    ),
                };
                let mut path = vec![surface.element.to_string()];
                path.extend(cause.dependency_path.iter().cloned());
                outcomes.insert(
                    surface.element.clone(),
                    Outcome::Skipped(Skip {
                        capability: cause.capability,
                        explanation: cause.explanation,
                        dependency_path: path,
                    }),
                );
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }

    // Only what a retained output needs is published.
    let emitted = |id: &ElementId| matches!(outcomes.get(id), Some(Outcome::Emitted));
    functions.retain(|function| emitted(&function.element));
    surfaces.retain(|surface| emitted(&surface.element));

    let mut entries: Vec<Entry> = requests
        .outputs
        .iter()
        .map(|output| Entry {
            element: output.element.clone(),
            outcome: outcomes
                .get(&output.element.id)
                .cloned()
                .unwrap_or(Outcome::Emitted),
        })
        .chain(requests.ignored.iter().map(|element| Entry {
            element: element.clone(),
            outcome: Outcome::Ignored,
        }))
        .collect();
    sort_entries(&mut entries);

    let report = Report {
        pipeline: PIPELINE,
        target: requests.target,
        schema_version: SCHEMA_VERSION,
        source_identity: SourceIdentity {
            declaring_crate: declaring_crate.into(),
            sources: flat.source_modules().to_vec(),
            captured_items: flat.elements().count(),
        },
        elements: entries,
    };

    // Planning is over: the working state hands over its tables, and the model
    // moves into the frozen result.
    let nodes = std::mem::take(&mut run.nodes);
    let primitives = std::mem::take(&mut run.primitives);
    drop(run);

    // Only what a retained output needs is published: a helper is kept when an
    // emitted wrapper applies an operation that depends on it, and a type
    // declaration when its own public declaration survived.
    let mut artifacts: Vec<crate::target::Artifact> = Vec::new();
    let keep = |artifact: &crate::target::Artifact, artifacts: &mut Vec<_>| {
        if !artifacts
            .iter()
            .any(|kept: &crate::target::Artifact| kept.name == artifact.name)
        {
            artifacts.push(artifact.clone());
        }
    };
    for surface in &surfaces {
        for artifact in &surface.rust {
            keep(artifact, &mut artifacts);
        }
    }
    for function in &functions {
        for instr in &function.instrs {
            if let Instr::Apply { primitive, .. } = instr {
                for artifact in &primitives[primitive.0].dependencies {
                    keep(artifact, &mut artifacts);
                }
            }
        }
        for route in &function.failures {
            for artifact in route.report.iter().flat_map(|report| &report.dependencies) {
                keep(artifact, &mut artifacts);
            }
        }
    }

    let rust = crate::emit::render(
        &flat,
        target,
        &requests.source_module,
        &artifacts,
        &primitives,
        &functions,
    );

    Ok(Generation::new(
        flat, report, nodes, functions, surfaces, primitives, rust,
    ))
}

/// A requested output that survived planning.
struct Emitted<P> {
    function: Option<FunctionPlan<P>>,
    surface: SurfaceSpec<P>,
}

/// Plan one exported function: its input conversions, its call, its result and
/// the native interface around them.
fn plan_function<T: Target>(
    run: &mut Run<'_, T>,
    element: &DeclaredElement,
    policy: &T::Policy,
) -> Result<Result<Emitted<T::Payload>, Unsupported>, PlanningError> {
    let flat = run.flat;
    let function: &Function = flat
        .function(element.rust_origin.as_str())
        .expect("declarations are checked against the model before planning");
    let root = crate::target::Position::root(element.id.clone());

    let mut inputs = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        let position = root.child(format!("param {index}"));
        match run.plan_value(
            Crossing {
                ty: param.ty.clone(),
                direction: Direction::IntoRust,
            },
            &position,
        )? {
            Planned::Ready(id) => inputs.push(id),
            Planned::Unsupported(unsupported) => return Ok(Err(unsupported)),
        }
    }
    let output = if matches!(function.ret.kind(), TypeKind::Unit) {
        None
    } else {
        match run.plan_value(
            Crossing {
                ty: function.ret.clone(),
                direction: Direction::OutOfRust,
            },
            &root.child("return"),
        )? {
            Planned::Ready(id) => Some(id),
            Planned::Unsupported(unsupported) => return Ok(Err(unsupported)),
        }
    };

    let values = ResolvedValues {
        inputs: inputs.iter().map(|id| &run.nodes[id.0]).collect(),
        output: output.map(|id| &run.nodes[id.0]),
    };
    let site = SiteDescriptor { element, function };
    let boundary = match run.target.boundary(&site, &values, policy)? {
        TargetAttempt::Ready(boundary) => boundary,
        TargetAttempt::Unsupported(unsupported) => return Ok(Err(unsupported)),
    };
    // Every failure the conversions declare needs a terminal action here.
    for category in values.failure_categories() {
        if !boundary
            .failures
            .iter()
            .any(|route| route.category == category)
        {
            return Ok(Err(Unsupported::new(
                "unsupported.boundary.unrouted_failure",
                format!(
                    "a conversion can fail with a {} error and this boundary declares no route \
                     for it",
                    category.as_str()
                ),
            )));
        }
    }
    let surface = match run.target.surface(
        &SurfaceRequest {
            element,
            policy,
            item: SourceItem::Function(function),
        },
        &values,
    )? {
        TargetAttempt::Ready(surface) => surface,
        TargetAttempt::Unsupported(unsupported) => return Ok(Err(unsupported)),
    };
    drop(values);

    let plan = match assemble(run, element, function, &inputs, output, boundary)? {
        Ok(plan) => plan,
        Err(unsupported) => return Ok(Err(unsupported)),
    };
    Ok(Ok(Emitted {
        function: Some(plan),
        surface,
    }))
}

/// Put one wrapper together: native parameters in, conversions, one call, the
/// result out.
fn assemble<T: Target>(
    run: &mut Run<'_, T>,
    element: &DeclaredElement,
    function: &Function,
    inputs: &[NodeId],
    output: Option<NodeId>,
    boundary: crate::target::BoundarySpec<T::Payload>,
    // The boundary is consumed: its routes and its ABI belong to the plan.
) -> Result<Result<FunctionPlan<T::Payload>, Unsupported>, PlanningError> {
    let mut body = BodyBuilder::new();
    let mut params = Vec::new();
    let mut contexts: BTreeMap<String, ValueId> = BTreeMap::new();
    for param in &boundary.abi.params {
        let id = body.fresh();
        if let ParamRole::Context(name) = &param.role {
            contexts.insert(name.clone(), id);
        }
        params.push((id, param.clone()));
    }

    let mut arguments = Vec::new();
    for (index, node) in inputs.iter().enumerate() {
        let carrier = params
            .iter()
            .find(|(_, param)| matches!(param.role, ParamRole::Input(i) if i == index))
            .map(|(id, _)| *id);
        let carrier = match carrier {
            Some(carrier) => carrier,
            None => {
                return Err(PlanningError::InternalInvariant(format!(
                    "`{}` declares no native parameter for source parameter {index}",
                    element.id
                )))
            }
        };
        let node_body = run.nodes[node.0].body.clone();
        arguments.push(node_body.inline(carrier, &mut body));
    }

    let call = body.fresh();
    body.push(Instr::Call {
        function: function.name.to_string(),
        args: arguments,
        result: call,
    });

    let result = match output {
        Some(node) => {
            let node_body = run.nodes[node.0].body.clone();
            Some(node_body.inline(call, &mut body))
        }
        None => None,
    };
    let result = match boundary.output {
        OutputPlacement::Return => result,
        OutputPlacement::Void => None,
    };

    let instrs = body.into_instrs();
    // A conversion that asks for a runtime context the boundary does not
    // supply cannot be assembled — the route exists or the function is skipped.
    for instr in &instrs {
        if let Instr::Apply { operands, .. } = instr {
            for operand in operands {
                if let Operand::Context(name) = operand {
                    if name != "__error" && !contexts.contains_key(name) {
                        return Ok(Err(Unsupported::new(
                            "unsupported.boundary.missing_context",
                            format!(
                                "an operation needs the `{name}` runtime context and this \
                                 boundary supplies none"
                            ),
                        )));
                    }
                }
            }
        }
    }

    Ok(Ok(FunctionPlan {
        element: element.id.clone(),
        abi: boundary.abi,
        output: boundary.output,
        failures: boundary.failures,
        params,
        instrs,
        result,
    }))
}

/// Plan one exported record: the conversion everything taking it needs, and the
/// public declaration itself.
fn plan_record<T: Target>(
    run: &mut Run<'_, T>,
    element: &DeclaredElement,
    policy: &T::Policy,
) -> Result<Result<Emitted<T::Payload>, Unsupported>, PlanningError> {
    let flat = run.flat;
    let record: &Struct = match flat.struct_type(element.rust_origin.as_str()) {
        Some(record) => record,
        None => {
            return Ok(Err(Unsupported::new(
                "unsupported.type.not_a_record",
                format!(
                    "`{}` is declared as a data type, and v2 represents only records so far",
                    element.rust_origin
                ),
            )))
        }
    };
    let root = crate::target::Position::root(element.id.clone());
    let node = match run.plan_value(
        Crossing {
            ty: record.type_ref().clone(),
            direction: Direction::IntoRust,
        },
        &root,
    )? {
        Planned::Ready(id) => id,
        Planned::Unsupported(unsupported) => return Ok(Err(unsupported)),
    };

    let values = ResolvedValues {
        inputs: vec![&run.nodes[node.0]],
        output: None,
    };
    let surface = match run.target.surface(
        &SurfaceRequest {
            element,
            policy,
            item: SourceItem::Record(record),
        },
        &values,
    )? {
        TargetAttempt::Ready(surface) => surface,
        TargetAttempt::Unsupported(unsupported) => return Ok(Err(unsupported)),
    };
    drop(values);
    Ok(Ok(Emitted {
        function: None,
        surface,
    }))
}
