//! Planning: requests in, retained plans out.
//!
//! One recursive walk plans the value conversions, one
//! pass per requested output assembles the wrappers, and one fixpoint
//! decides what survives. The registry owns all three; the target answers the
//! local questions in [`crate::target`].

use std::collections::{BTreeMap, HashMap};

use prebindgen_flat::{
    flat::{Flat, Function, Type, TypeKind, TypeRef},
    Conditioned, RustEmitter,
};

use crate::{
    body::{BodyBuilder, Instr, NodeBody, Operand, ValueId},
    decl::{CapturedName, Declaration},
    outcome::{EngineError, Outcome, Skip},
    report::{sort_entries, Entry, Report, SourceIdentity, SCHEMA_VERSION},
    run::{check_declarations, Generation, PIPELINE},
    target::{
        AbiSpec, ChildValue, Crossing, Direction, FailureCategory, FailureRoute, Layout,
        OperandRole, OutputPlacement, ParamRole, Part, PlanningError, PrimitiveFailure,
        PrimitiveId, PrimitiveSpec, Protocol, Relation, RelationId, ResolvedShape, ResolvedValues,
        Selection, SelectionQuery, SiteDescriptor, SourceItem, StructRelation, SurfaceRequest,
        SurfaceSpec, Target, TargetAttempt, Unsupported,
    },
};

/// One entry of a [`BindingRequests`] work list: something the binding said
/// about one item.
///
/// The two dispositions accept different things. An expose takes any
/// [`Declaration`] — a captured item, or one the binding defines itself. An
/// ignore takes only a [`CapturedName`]: nothing binding-local is in the
/// source to leave alone. The report accounts for both, so they are one list
/// rather than two: an id can then appear once, which is what the report's
/// "one id, one row" rests on. Exposing something and ignoring it is a
/// contradiction the engine catches for that reason, rather than emitting two
/// rows for it.
#[derive(Clone, Debug)]
pub enum Request {
    /// Expose this declaration, captured or binding-local. The target is
    /// asked what it is, and the registry plans it.
    Expose(Declaration),
    /// Leave this captured item alone. Nothing is planned and nothing is
    /// generated; the report carries it so that a decision and a gap read
    /// differently. A name and a kind are all an ignore has to say, so that is
    /// all it can carry.
    Ignore(CapturedName),
}

/// What a frontend hands the engine: what to expose, and what to leave alone.
///
/// Nothing about *how* anything crosses is here. A binding's choices — which
/// C name a type gets, which Kotlin class, which parameter is expanded — stay
/// in the frontend's own storage and are answered where the registry asks, in
/// [`Target`]. What this carries is the work list, one [`Request`] per entry,
/// so the registry can plan each, decide what survives, and account for every
/// one of them in the report.
///
/// Users never write this; a frontend builds it from its own recorded calls,
/// which is what lets two languages share everything after this point.
pub struct BindingRequests {
    /// The crate whose build script is generating, for the report.
    pub declaring_crate: String,
    /// The module generated code reaches the source items through.
    pub source_module: syn::Path,
    /// What the binding said, in the order it said it.
    pub requests: Vec<Request>,
}

impl BindingRequests {
    pub fn new(declaring_crate: impl Into<String>, source_module: syn::Path) -> Self {
        BindingRequests {
            declaring_crate: declaring_crate.into(),
            source_module,
            requests: Vec::new(),
        }
    }

    /// Ask for one declaration to be exposed.
    pub fn expose(&mut self, declaration: Declaration) -> &mut Self {
        self.requests.push(Request::Expose(declaration));
        self
    }

    /// Ask for one captured item to be left alone.
    pub fn ignore(&mut self, name: CapturedName) -> &mut Self {
        self.requests.push(Request::Ignore(name));
        self
    }

    /// The declarations to plan, and the ones only the report hears about —
    /// the latter as the declarations that would have exposed them, which is
    /// the id a report row and a duplicate check go by.
    ///
    /// Split once, here, so that nothing downstream can plan an ignore by
    /// forgetting to filter for it.
    fn split(&self) -> (Vec<&Declaration>, Vec<Declaration>) {
        let mut exposed = Vec::new();
        let mut ignored = Vec::new();
        for request in &self.requests {
            match request {
                Request::Expose(declaration) => exposed.push(declaration),
                Request::Ignore(name) => ignored.push(Declaration::from(name.clone())),
            }
        }
        (exposed, ignored)
    }
}

/// A retained conversion, reusable by every value that crosses the same way.
#[derive(Debug)]
pub struct ValuePlan<P> {
    pub id: NodeId,
    pub crossing: Crossing,
    /// How the Rust value is built or read, as the target chose it.
    pub relation: Relation,
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

/// A complete wrapper: the exported function, as planned.
///
/// One per declaration that exports a source function, plus one per handle
/// type for its release. The common Rust writer renders it as the
/// `#[no_mangle] extern` function a foreign caller links against.
#[derive(Debug)]
pub struct FunctionPlan<P> {
    /// The declaration this wrapper exports.
    pub declaration: Declaration,
    pub abi: AbiSpec,
    pub output: OutputPlacement,
    pub failures: Vec<FailureRoute<P>>,
    /// Wrapper parameters, paired with the value identity that names each.
    pub params: Vec<(ValueId, crate::target::WrapperParam)>,
    pub instrs: Vec<crate::body::Step>,
    /// The value delivered through [`Self::output`].
    pub result: Option<ValueId>,
}

/// A conversion whose planning has begun, for cycle detection.
///
/// Deliberately coarser than [`NodeKey`]: what makes a cycle is a type reaching
/// its own conversion, which its children cannot be known before.
///
/// It is the target's [`Target::ConversionKey`] that stands for the settings
/// here, which is why a target must not mint a fresh key per visit: two visits
/// to the same conversion would then look like two conversions, and a cycle
/// would recurse instead of being refused.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ResolvingKey<K> {
    ty: prebindgen_flat::TypeKey,
    direction: Direction,
    relation: RelationId,
    conversion: K,
}

/// What makes two conversions the same conversion.
///
/// The children are part of it. A choice recorded for a field is looked up at
/// that field's position, so two structs whose own conversion key is equal but
/// whose fields were configured differently resolve to different children —
/// and must not share a node, or the second use would silently inherit the
/// first one's conversion.
#[derive(PartialEq, Eq, Hash)]
struct NodeKey<K> {
    ty: prebindgen_flat::TypeKey,
    direction: Direction,
    relation: RelationId,
    conversion: K,
    children: Vec<NodeId>,
}

/// A capability that stopped a conversion, and where it was met.
struct Refusal {
    reason: Unsupported,
    at: crate::target::Position,
}

/// A planned value, or the capability that stopped it.
enum Planned {
    Ready(NodeId),
    Unsupported(Refusal),
}

impl Planned {
    fn refused(reason: Unsupported, at: &crate::target::Position) -> Self {
        Planned::Unsupported(Refusal::at(reason, at))
    }
}

impl Refusal {
    fn at(reason: Unsupported, at: &crate::target::Position) -> Self {
        Refusal {
            reason,
            at: at.clone(),
        }
    }
}

/// The working state of one `generate` call.
struct Run<'a, T: Target> {
    flat: &'a Flat,
    target: &'a T,
    relations: Vec<Relation>,
    primitives: Vec<PrimitiveSpec<T::Payload>>,
    nodes: Vec<ValuePlan<T::Payload>>,
    cache: HashMap<NodeKey<T::ConversionKey>, NodeId>,
    resolving: std::collections::HashSet<ResolvingKey<T::ConversionKey>>,
    /// The relations offered for a type, registered the first time it is
    /// planned. A relation's identity is part of a conversion's identity, so
    /// registering a fresh one per visit would give every value its own node
    /// and share nothing.
    offered: HashMap<prebindgen_flat::TypeKey, Vec<(RelationId, Relation)>>,
}

impl<'a, T: Target> Run<'a, T> {
    fn new(flat: &'a Flat, target: &'a T) -> Self {
        Run {
            flat,
            target,
            relations: Vec::new(),
            primitives: Vec::new(),
            nodes: Vec::new(),
            cache: HashMap::new(),
            resolving: std::collections::HashSet::new(),
            offered: HashMap::new(),
        }
    }

    /// Register an operation, and hand back the identity instructions name it
    /// by.
    fn register(&mut self, primitive: PrimitiveSpec<T::Payload>) -> PrimitiveId {
        self.primitives.push(primitive);
        PrimitiveId(self.primitives.len() - 1)
    }

    /// The relations available for this type: today the implicit one, which is
    /// the struct's fields for a struct and the atomic conversion for anything
    /// carried whole.
    fn candidates(&mut self, ty: &TypeRef) -> Result<Vec<(RelationId, Relation)>, Unsupported> {
        if let Some(offered) = self.offered.get(&ty.key()) {
            return Ok(offered.clone());
        }
        let mut relations = vec![Relation::Atomic];
        if let TypeKind::Named { id, .. } = ty.kind() {
            match self.flat.resolve(id) {
                Some(Type::Struct(strukt)) => relations.push(Relation::Struct(StructRelation {
                    name: strukt.name.to_string(),
                    parts: strukt
                        .fields
                        .iter()
                        .map(|field| Part {
                            name: field.name.as_ref().map(|name| name.to_string()),
                            index: field.index,
                            ty: field.ty.clone(),
                            conditions: crate::emit::Writer.conditions(Conditioned::Field(field)),
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

        let candidates = match self.candidates(&crossing.ty) {
            Ok(candidates) => candidates,
            Err(reason) => return Ok(Planned::refused(reason, position)),
        };

        // Everything the target needs to apply its own settings: the value,
        // and where it sits. What comes back says both how to read it and
        // which conversion that makes.
        let query = SelectionQuery {
            crossing: &crossing,
            position,
            candidates: &candidates,
        };
        let Selection {
            relation: relation_id,
            conversion,
        } = match target.select(&query)? {
            TargetAttempt::Ready(selection) => selection,
            TargetAttempt::Unsupported(reason) => return Ok(Planned::refused(reason, position)),
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

        // Meeting a conversion that is already being resolved is a cycle: a
        // type whose conversion needs its own. The mark is bookkeeping, not a
        // plan, and it is dropped whichever way this call ends.
        let resolving = ResolvingKey {
            ty: crossing.ty.key(),
            direction: crossing.direction,
            relation: relation_id,
            conversion: conversion.clone(),
        };
        if !self.resolving.insert(resolving.clone()) {
            return Ok(Planned::refused(
                Unsupported::new(
                    "unsupported.conversion.recursive",
                    format!(
                        "converting `{}` needs its own conversion; v2 has no recursive \
                         conversion yet",
                        crossing.ty.key()
                    ),
                ),
                position,
            ));
        }
        let planned = self.plan_relation(&crossing, position, relation_id, &relation, conversion);
        self.resolving.remove(&resolving);
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
        conversion: T::ConversionKey,
    ) -> Result<Planned, PlanningError> {
        let target = self.target;
        let parts = relation.parts().to_vec();

        // Children are planned before this conversion's identity is known,
        // because a choice recorded for one of them makes this a different
        // conversion: two values whose own conversion key is equal but whose
        // fields disagree must not share a node.
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
                // Nothing partial is recorded: a struct missing a field is a
                // different type, not a reduced one.
                Planned::Unsupported(refusal) => return Ok(Planned::Unsupported(refusal)),
            }
        }

        let key = NodeKey {
            ty: crossing.ty.key(),
            direction: crossing.direction,
            relation: relation_id,
            conversion: conversion.clone(),
            children: children.clone(),
        };
        if let Some(id) = self.cache.get(&key) {
            return Ok(Planned::Ready(*id));
        }

        let strukt = match relation {
            Relation::Struct(strukt) => self.flat.struct_type(strukt.name.as_str()),
            Relation::Atomic => None,
        };
        let shape = ResolvedShape {
            crossing,
            relation,
            strukt,
        };
        let child_values: Vec<ChildValue<'_>> = parts
            .iter()
            .zip(&children)
            .map(|(part, id)| ChildValue {
                part,
                layout: &self.nodes[id.0].repr.layout,
            })
            .collect();
        let repr = match target.represent(&shape, &child_values, &conversion)? {
            TargetAttempt::Ready(repr) => repr,
            TargetAttempt::Unsupported(reason) => return Ok(Planned::refused(reason, position)),
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
                check_operation(codec, repr.layout.wire(), repr.layout.wire())?;
                failures.extend(codec.failure_category());
                self.apply(&mut body, (**codec).clone(), &[carrier])?
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
                if let Layout::Aggregate { members, .. } = &repr.layout {
                    if members.len() != parts.len() {
                        return Err(PlanningError::InternalInvariant(format!(
                            "the representation of `{}` names {} member(s) for {} part(s)",
                            crossing.ty.key(),
                            members.len(),
                            parts.len()
                        )));
                    }
                }
                let strukt = match relation {
                    Relation::Struct(strukt) => strukt.name.clone(),
                    Relation::Atomic => {
                        return Err(PlanningError::InternalInvariant(
                            "a product representation needs a relation with parts".to_string(),
                        ))
                    }
                };
                let mut converted = Vec::new();
                for (index, (projection, child)) in projections.iter().zip(&children).enumerate() {
                    // Everything planned for this part, from the read of the
                    // member to the last step of its conversion, exists exactly
                    // where the source field does. The range is stamped rather
                    // than each instruction, because a child conversion's own
                    // instructions are inlined here and are equally the part's.
                    let from = body.len();
                    check_operation(
                        projection,
                        repr.layout.wire(),
                        self.nodes[child.0].repr.layout.wire(),
                    )?;
                    // Every projection reads the same carrier, so none of them
                    // may claim to consume it.
                    if projections.len() > 1 && projection.consumes_value() {
                        return Err(PlanningError::InternalInvariant(format!(
                            "a projection of `{}` consumes the value the other projections \
                             still read",
                            crossing.ty.key()
                        )));
                    }
                    // A member read has to name a member the representation
                    // declared, or the wrapper reads something the foreign
                    // caller never filled in.
                    if let (
                        crate::target::Operation::Standard(crate::target::StandardOp::ReadMember {
                            member,
                        }),
                        Layout::Aggregate { members, .. },
                    ) = (&projection.implementation, &repr.layout)
                    {
                        if !members.iter().any(|declared| same_member(declared, member)) {
                            return Err(PlanningError::InternalInvariant(format!(
                                "a projection of `{}` reads a member its representation does \
                                 not declare",
                                crossing.ty.key()
                            )));
                        }
                    }
                    failures.extend(projection.failure_category());
                    let obtained = self.apply(&mut body, projection.clone(), &[carrier])?;
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
                    body.condition(from, &parts[index].conditions);
                }
                let result = body.fresh();
                body.push(Instr::Construct {
                    name: strukt,
                    parts: converted,
                    result,
                });
                Some(result)
            }
            (Protocol::Product { .. }, Direction::OutOfRust) => {
                return Ok(Planned::refused(
                    Unsupported::new(
                        "unsupported.struct.out_of_rust",
                        format!(
                            "`{}` leaves Rust as a composed value; v2 has no target \
                             construction operation yet",
                            crossing.ty.key()
                        ),
                    ),
                    position,
                ))
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
            relation: relation.clone(),
            repr,
            children,
            body: NodeBody {
                carrier,
                instrs: body.into_instrs(),
                result,
            },
            failures,
        });
        self.cache.insert(key, id);
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
    ) -> Result<Option<ValueId>, PlanningError> {
        if matches!(
            primitive.implementation,
            crate::target::Operation::Standard(crate::target::StandardOp::Identity)
        ) && matches!(
            primitive.failure,
            crate::target::PrimitiveFailure::Infallible
        ) {
            return Ok(values.first().copied());
        }
        let mut supplied = values.iter();
        let mut operands = Vec::new();
        for operand in &primitive.operands {
            operands.push(match &operand.role {
                OperandRole::Value => match supplied.next() {
                    Some(value) => Operand::Value(*value),
                    None => {
                        return Err(PlanningError::InternalInvariant(
                            "an operation takes more values than this conversion supplies"
                                .to_string(),
                        ))
                    }
                },
                OperandRole::Context(name) => Operand::Context(name.clone()),
                // An error value exists on a failure route and nowhere else:
                // inside a conversion there is nothing to bind it to.
                OperandRole::Error => {
                    return Err(PlanningError::InternalInvariant(
                        "a conversion operation asks for an error value, which only a failure \
                         route has"
                            .to_string(),
                    ))
                }
            });
        }
        let produces = primitive.result.is_some();
        let id = self.register(primitive);
        let result = produces.then(|| body.fresh());
        body.push(Instr::Apply {
            primitive: id,
            operands,
            result,
        });
        Ok(result)
    }
}

/// Check one operation against the values it will be applied to.
///
/// An operation states the type of every operand and of its result, and the
/// registry is the only place those statements meet the carriers actually in
/// hand. `carrier` is what this conversion is given; `produces` is what the
/// application must yield — the child's carrier for a projection, the
/// conversion's own for a terminal codec.
fn check_operation<P>(
    primitive: &PrimitiveSpec<P>,
    carrier: &crate::target::WireType,
    produces: &crate::target::WireType,
) -> Result<(), PlanningError> {
    if primitive.value_operands() != 1 {
        return Err(PlanningError::InternalInvariant(format!(
            "this operation takes {} value operand(s); a conversion supplies exactly one",
            primitive.value_operands()
        )));
    }
    let value = primitive
        .operands
        .iter()
        .find(|operand| matches!(operand.role, OperandRole::Value))
        .expect("the arity check above found one");
    if let crate::target::OperationType::Carrier(wire) = &value.ty {
        if !same_type(&wire.ty, &carrier.ty) {
            return Err(PlanningError::InternalInvariant(format!(
                "this operation reads a `{}` and is applied to a `{}`",
                spell(&wire.ty),
                spell(&carrier.ty)
            )));
        }
    }
    match &primitive.result {
        Some(crate::target::OperationType::Carrier(wire)) if !same_type(&wire.ty, &produces.ty) => {
            Err(PlanningError::InternalInvariant(format!(
                "this operation produces a `{}` where a `{}` is expected",
                spell(&wire.ty),
                spell(&produces.ty)
            )))
        }
        _ => Ok(()),
    }
}

fn spell(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

fn same_type(left: &syn::Type, right: &syn::Type) -> bool {
    spell(left) == spell(right)
}

fn same_member(left: &syn::Member, right: &syn::Member) -> bool {
    use quote::ToTokens;
    left.to_token_stream().to_string() == right.to_token_stream().to_string()
}

/// Plan `requests` over `flat` with `target`, and render what survives.
///
/// Every requested output leaves this with an outcome. A capability the engine
/// or the target has not implemented is a reported skip and the run continues;
/// contradictory input and violated invariants fail.
pub fn generate<T: Target>(
    flat: Flat,
    target: &T,
    requests: BindingRequests,
) -> Result<Generation<T::Payload>, EngineError> {
    let (exposed, ignored) = requests.split();
    // Duplicates are checked over both dispositions, so declaring a thing and
    // ignoring it is caught here rather than printed as two rows for one id.
    // Existence is asked of the exposed only: an ignore says "if this is here,
    // leave it alone", and a binding may reasonably ignore an item its source
    // crate compiles out under a feature.
    check_declarations(&exposed, &ignored, &flat)?;

    let mut run = Run::new(&flat, target);
    let mut functions: Vec<FunctionPlan<T::Payload>> = Vec::new();
    let mut surfaces: Vec<SurfaceSpec<T::Payload>> = Vec::new();
    let mut outcomes: BTreeMap<Declaration, Outcome> = BTreeMap::new();

    for &declaration in &exposed {
        let root = crate::target::Position::root(declaration.clone());
        // What is planned follows from the declaration, which says both what the
        // target asked for and what the captured source holds for it. Each
        // planner is reached by its own variants and is given the captured item
        // they name; nothing below reads a field back to work out what it was
        // asked for.
        let planned = match declaration {
            Declaration::Type(_) | Declaration::LocalType(_) => {
                plan_type(&mut run, declaration).map_err(EngineError::Planning)?
            }
            // Two surfaces built the same way: a Kotlin `val` read through a
            // nullary function is planned as that function, and the target
            // renders the constant.
            Declaration::Function(ident) | Declaration::ConstFromFunction(ident) => {
                let function = flat
                    .function(&ident.to_string())
                    .expect("declarations are checked against the model before planning");
                plan_function(&mut run, declaration, function).map_err(EngineError::Planning)?
            }
            // A declaration the binding defines itself names no captured item,
            // so there is nothing to plan from: its signature or its value is
            // the binding's own, and reading one is a capability this engine
            // does not have.
            Declaration::LocalFunction(name) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.fn.binding_local",
                    format!(
                        "`{name}` is defined by the binding, not captured from the source, so \
                         there is no captured item to plan from"
                    ),
                ),
                &root,
            )),
            Declaration::LocalConst(name) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.const.binding_local",
                    format!(
                        "`{name}` is defined by the binding, not captured from the source, so \
                         there is no captured item to plan from"
                    ),
                ),
                &root,
            )),
            // One code per kind rather than one for the whole engine: the
            // report is how the next capability is chosen, and "everything is
            // unsupported" chooses nothing.
            Declaration::Const(_) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.const.not_implemented",
                    "the v2 engine has no const lowering yet",
                ),
                &root,
            )),
            Declaration::Callback(_) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.callback.not_implemented",
                    "the v2 engine has no callback lowering yet",
                ),
                &root,
            )),
            Declaration::Conversion(_) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.conversion.not_implemented",
                    "the v2 engine has no conversion lowering yet",
                ),
                &root,
            )),
        };
        match planned {
            Ok(Emitted { function, surface }) => {
                if let Some(function) = function {
                    functions.push(function);
                }
                surfaces.push(surface);
                outcomes.insert(declaration.clone(), Outcome::Emitted);
            }
            Err(refusal) => {
                // The path is where the walk actually stopped — the exported
                // function, the parameter, the field — so the report says what
                // to look at rather than only which declaration vanished.
                outcomes.insert(
                    declaration.clone(),
                    Outcome::Skipped(Skip {
                        capability: refusal.reason.capability,
                        explanation: refusal.reason.explanation,
                        dependency_path: refusal.at.dependency_path(),
                    }),
                );
            }
        }
    }

    // Which declaration represents which type. A target names a requirement by
    // type, because that is what the model told it about a value; matching the
    // type to the declaration covering it is the engine's side of that.
    let declared_types: BTreeMap<String, &Declaration> = exposed
        .iter()
        .filter(|declaration| declaration.is_type())
        .map(|declaration| (declaration.name(), *declaration))
        .collect();

    // A public declaration can require another one. Propagate until a pass
    // changes nothing: one missing capability, several skipped outputs, each
    // keeping its own path to the cause.
    loop {
        let mut changed = false;
        for surface in &surfaces {
            if !matches!(outcomes.get(&surface.declaration), Some(Outcome::Emitted)) {
                continue;
            }
            for required in &surface.requires {
                let declared = declared_types
                    .get(required.type_name())
                    .and_then(|id| outcomes.get(*id));
                let cause = match declared {
                    Some(Outcome::Skipped(skip)) => skip.clone(),
                    Some(_) => continue,
                    None => Skip::direct(
                        "unsupported.requirement.unrequested",
                        format!("requires {required}, which this binding declares no type for"),
                        required.to_string(),
                    ),
                };
                let mut path = vec![surface.declaration.to_string()];
                path.extend(cause.dependency_path.iter().cloned());
                outcomes.insert(
                    surface.declaration.clone(),
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
    let emitted =
        |declaration: &Declaration| matches!(outcomes.get(declaration), Some(Outcome::Emitted));
    functions.retain(|function| emitted(&function.declaration));
    surfaces.retain(|surface| emitted(&surface.declaration));

    let mut entries: Vec<Entry> = exposed
        .iter()
        .map(|declaration| Entry {
            declaration: (*declaration).clone(),
            described: target.describe(declaration),
            outcome: outcomes
                .get(*declaration)
                .cloned()
                .unwrap_or(Outcome::Emitted),
        })
        // An ignore is a decision the binding made about a captured item: it
        // is not an output, so the target is never asked to describe it, and
        // it lands nowhere. The id's prefix (`fn:`, `type:`, `const:`) already
        // says what was left alone.
        .chain(ignored.iter().map(|declaration| Entry {
            declaration: declaration.clone(),
            described: crate::target::Described::new("ignore", ""),
            outcome: Outcome::Ignored,
        }))
        .collect();
    sort_entries(&mut entries);

    let report = Report {
        pipeline: PIPELINE,
        target: T::NAME,
        schema_version: SCHEMA_VERSION,
        source_identity: SourceIdentity {
            declaring_crate: requests.declaring_crate.clone(),
            sources: flat.source_modules().to_vec(),
            captured_items: flat.elements().count(),
        },
        declarations: entries,
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
        // Rust a target contributes for its own public declaration of a
        // captured item — the `repr(C)` mirror of a struct — exists only where
        // that item does, by the rule a wrapper follows. A declaration the
        // binding defines itself has no captured item and no condition.
        let conditions = surface
            .declaration
            .captured(&flat)
            .map(|element| crate::emit::Writer.conditions(Conditioned::Item(element)))
            .unwrap_or_default();
        for artifact in &surface.rust {
            let artifact = match conditions.is_empty() {
                true => artifact.clone(),
                false => {
                    let rust = &artifact.rust;
                    crate::target::Artifact::new(
                        artifact.name.clone(),
                        quote::quote!(#(#conditions)* #rust),
                    )
                }
            };
            keep(&artifact, &mut artifacts);
        }
    }
    for function in &functions {
        for step in &function.instrs {
            if let Instr::Apply { primitive, .. } = &step.instr {
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

/// Plan the wrapper that exports one source function: the conversions of its
/// parameters, the call, the conversion of its result, and the wrapper's
/// interface around them.
fn plan_function<T: Target>(
    run: &mut Run<'_, T>,
    declaration: &Declaration,
    function: &Function,
) -> Result<Result<Emitted<T::Payload>, Refusal>, PlanningError> {
    let root = crate::target::Position::root(declaration.clone());

    // The wrapper is a safe function, and the writer renders a plain call.
    // Wrapping an `unsafe fn` would need the wrapper to state the caller's
    // obligations, which nothing here can do yet; hiding them in an `unsafe`
    // block would make a safe public function out of a contract it does not
    // uphold.
    if function.is_unsafe() {
        return Ok(Err(Refusal::at(
            Unsupported::new(
                "unsupported.fn.unsafe",
                format!(
                    "`{}` is an `unsafe fn`, and v2 has no way to carry its safety contract \
                     through a wrapper yet",
                    function.name
                ),
            ),
            &root,
        )));
    }

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
            Planned::Unsupported(refusal) => return Ok(Err(refusal)),
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
            Planned::Unsupported(refusal) => return Ok(Err(refusal)),
        }
    };

    let values = ResolvedValues {
        inputs: inputs.iter().map(|id| &run.nodes[id.0]).collect(),
        output: output.map(|id| &run.nodes[id.0]),
    };
    let site = SiteDescriptor {
        declaration,
        function: Some(function),
    };
    let boundary = match run.target.boundary(&site, &values)? {
        TargetAttempt::Ready(boundary) => boundary,
        TargetAttempt::Unsupported(reason) => return Ok(Err(Refusal::at(reason, &root))),
    };
    // Every failure the conversions declare needs a terminal action here.
    for category in values.failure_categories() {
        if !boundary
            .failures
            .iter()
            .any(|route| route.category == category)
        {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.boundary.unrouted_failure",
                    format!(
                    "a conversion can fail with a {} error and this boundary declares no route \
                     for it",
                    category.as_str()
                ),
                ),
                &root,
            )));
        }
    }
    let surface = match run.target.surface(
        &SurfaceRequest {
            declaration,
            item: SourceItem::Function(function),
        },
        &values,
    )? {
        TargetAttempt::Ready(surface) => surface,
        TargetAttempt::Unsupported(reason) => return Ok(Err(Refusal::at(reason, &root))),
    };
    drop(values);

    let plan = match assemble(
        run,
        declaration,
        Body::Call(function),
        &inputs,
        output,
        boundary,
    )? {
        Ok(plan) => plan,
        Err(unsupported) => return Ok(Err(unsupported)),
    };
    Ok(Ok(Emitted {
        function: Some(plan),
        surface,
    }))
}

/// What a wrapper does with its converted inputs.
enum Body<'a, P> {
    /// Call the source function once with them.
    Call(&'a Function),
    /// Apply this operation to the one carrier and produce nothing: a handle's
    /// release, which is a wrapper with no source function behind it.
    Release(Box<PrimitiveSpec<P>>),
}

/// Put one wrapper together: wrapper parameters in, conversions, one call — or,
/// for a release, one drop — the result out.
fn assemble<T: Target>(
    run: &mut Run<'_, T>,
    declaration: &Declaration,
    action: Body<'_, T::Payload>,
    inputs: &[NodeId],
    output: Option<NodeId>,
    boundary: crate::target::BoundarySpec<T::Payload>,
    // The boundary is consumed: its routes and its ABI belong to the plan.
) -> Result<Result<FunctionPlan<T::Payload>, Refusal>, PlanningError> {
    let refuse = |reason: Unsupported| {
        Ok(Err(Refusal::at(
            reason,
            &crate::target::Position::root(declaration.clone()),
        )))
    };
    // A symbol reaches generated Rust as a function name, so a target that
    // supplies something else is contradictory input rather than a name the
    // writer has to escape.
    if syn::parse_str::<syn::Ident>(&boundary.abi.symbol).is_err() {
        return Err(PlanningError::InvalidInput(format!(
            "`{}` exports the symbol `{}`, which is not a Rust identifier",
            declaration, boundary.abi.symbol
        )));
    }
    // The writer exports the wrapper under that symbol with `#[no_mangle]`; an
    // attribute restating or contradicting the linkage is not a form the
    // target may ask for, and rustc would only report the clash later.
    for attr in &boundary.abi.attrs {
        if attr.path().is_ident("no_mangle") || attr.path().is_ident("export_name") {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` states `#[{}]` on its wrapper, whose linkage the writer owns: the \
                 symbol is `{}`",
                declaration,
                attr.path()
                    .require_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                boundary.abi.symbol
            )));
        }
    }
    let mut body = BodyBuilder::new();
    let mut params = Vec::new();
    let mut contexts: BTreeMap<String, ValueId> = BTreeMap::new();
    for param in &boundary.abi.params {
        // A carrier that never leaves generated Rust cannot be an extern
        // parameter, whatever the adapter would like to pass through it.
        if !param.ty.abi {
            return Err(PlanningError::InternalInvariant(format!(
                "`{}` takes `{}` at its boundary, which is not an ABI carrier",
                declaration,
                spell(&param.ty.ty)
            )));
        }
        let id = body.fresh();
        if let ParamRole::Context(name) = &param.role {
            contexts.insert(name.clone(), id);
        }
        params.push((id, param.clone()));
    }
    if let Some(ret) = &boundary.abi.ret {
        if !ret.abi {
            return Err(PlanningError::InternalInvariant(format!(
                "`{}` returns `{}` at its boundary, which is not an ABI carrier",
                declaration,
                spell(&ret.ty)
            )));
        }
        let produced = output.map(|node| run.nodes[node.0].repr.layout.wire().ty.clone());
        match produced {
            Some(produced) if !same_type(&ret.ty, &produced) => {
                return Err(PlanningError::InternalInvariant(format!(
                    "`{}` returns `{}` at its boundary, and its result conversion produces a `{}`",
                    declaration,
                    spell(&ret.ty),
                    spell(&produced)
                )))
            }
            None => {
                return Err(PlanningError::InternalInvariant(format!(
                    "`{}` returns `{}` at its boundary and has no result conversion to fill it",
                    declaration,
                    spell(&ret.ty)
                )))
            }
            _ => {}
        }
    }

    let mut arguments = Vec::new();
    for (index, node) in inputs.iter().enumerate() {
        let carrier = params
            .iter()
            .find(|(_, param)| matches!(param.role, ParamRole::Input(i) if i == index))
            .map(|(id, param)| (*id, param.ty.clone()));
        let (carrier, wrapper_param) = match carrier {
            Some(carrier) => carrier,
            None => {
                return Err(PlanningError::InternalInvariant(format!(
                    "`{}` declares no wrapper parameter for source parameter {index}",
                    declaration
                )))
            }
        };
        // The conversion reads the carrier this parameter passes, so the two
        // are the same type or the wrapper reads something else entirely.
        let expected = run.nodes[node.0].repr.layout.wire();
        if !same_type(&wrapper_param.ty, &expected.ty) {
            return Err(PlanningError::InternalInvariant(format!(
                "`{}` passes parameter {index} as `{}`, and its conversion reads a `{}`",
                declaration,
                spell(&wrapper_param.ty),
                spell(&expected.ty)
            )));
        }
        match &action {
            Body::Call(_) => {
                let node_body = run.nodes[node.0].body.clone();
                arguments.push(node_body.inline(carrier, &mut body));
            }
            // A release takes the carrier as the conversion would, and drops
            // what it holds instead of converting it.
            Body::Release(release) => {
                check_operation(release, expected, expected)?;
                run.apply(&mut body, (**release).clone(), &[carrier])?;
            }
        }
    }

    let result = match &action {
        Body::Call(function) => {
            let call = output.map(|_| body.fresh());
            body.push(Instr::Call {
                function: function.name.to_string(),
                args: arguments,
                result: call,
            });
            match (output, call) {
                (Some(node), Some(call)) => {
                    let node_body = run.nodes[node.0].body.clone();
                    Some(node_body.inline(call, &mut body))
                }
                _ => None,
            }
        }
        Body::Release(_) => None,
    };
    let result = match boundary.output {
        OutputPlacement::Return => result,
        OutputPlacement::Void => None,
    };

    let instrs = body.into_instrs();
    // An operation that asks for a runtime context the boundary does not supply
    // cannot be assembled. That covers the reporting operations as much as the
    // conversions: a route whose reporter needs an environment nobody passes
    // would otherwise reach the writer and fail there.
    let mut needed: Vec<&String> = Vec::new();
    for step in &instrs {
        if let Instr::Apply { operands, .. } = &step.instr {
            for operand in operands {
                if let Operand::Context(name) = operand {
                    needed.push(name);
                }
            }
        }
    }
    for route in &boundary.failures {
        let Some(report) = &route.report else {
            continue;
        };
        for operand in &report.operands {
            match &operand.role {
                OperandRole::Context(name) => needed.push(name),
                OperandRole::Error => {}
                // A reporting operation is handed the error and its contexts,
                // and there is no converted value at that point to give it.
                OperandRole::Value => {
                    return Err(PlanningError::InternalInvariant(format!(
                        "the {} failure route of `{}` reports through an operation that reads a \
                         value",
                        route.category.as_str(),
                        declaration
                    )))
                }
            }
        }
    }
    // The reporter is handed the error the operation produced, so the two have
    // to be the same type; a route that reports something else would not
    // compile.
    for step in &instrs {
        let Instr::Apply { primitive, .. } = &step.instr else {
            continue;
        };
        let PrimitiveFailure::Fallible { error, category } = &run.primitives[primitive.0].failure
        else {
            continue;
        };
        let Some(route) = boundary
            .failures
            .iter()
            .find(|route| route.category == *category)
        else {
            continue;
        };
        let Some(report) = &route.report else {
            continue;
        };
        for operand in &report.operands {
            if !matches!(operand.role, OperandRole::Error) {
                continue;
            }
            if let (
                crate::target::OperationType::Carrier(reported),
                crate::target::OperationType::Carrier(raised),
            ) = (&operand.ty, &**error)
            {
                if !same_type(&reported.ty, &raised.ty) {
                    return Err(PlanningError::InternalInvariant(format!(
                        "the {} failure route of `{}` reports a `{}` where the operation raises \
                         a `{}`",
                        category.as_str(),
                        declaration,
                        spell(&reported.ty),
                        spell(&raised.ty)
                    )));
                }
            }
        }
    }
    for name in needed {
        if !contexts.contains_key(name) {
            return refuse(Unsupported::new(
                "unsupported.boundary.missing_context",
                format!(
                    "an operation needs the `{name}` runtime context and this boundary supplies \
                     none"
                ),
            ));
        }
    }

    Ok(Ok(FunctionPlan {
        declaration: declaration.clone(),
        abi: boundary.abi,
        output: boundary.output,
        failures: boundary.failures,
        params,
        instrs,
        result,
    }))
}

/// Plan one exported type: the conversion everything taking it needs, the
/// public declaration itself — and, for a type the foreign side holds an
/// obligation for, the conversion handing one out and the release freeing it.
///
/// Whether a type is a handle is the target's answer, not the item's kind: a
/// struct the binding declared opaque is carried whole as an address, exactly
/// as a declared alias is, and an alias a target carries by value would be no
/// handle. What says "handle" is the into-Rust representation naming a
/// release; the registry then requires the out-of-Rust direction too — a
/// handle is a promise that what a function returns can be given back — and
/// plans the release as a wrapper under the type's own identity.
fn plan_type<T: Target>(
    run: &mut Run<'_, T>,
    declaration: &Declaration,
) -> Result<Result<Emitted<T::Payload>, Refusal>, PlanningError> {
    let flat = run.flat;
    let root = crate::target::Position::root(declaration.clone());
    // A declared type need not be a captured item — a target may represent
    // `String` without the source exporting one — so this lookup, unlike a
    // function's, can find nothing.
    let (ty, item): (TypeRef, SourceItem<'_>) = match flat.declared_type(&declaration.name()) {
        Some(Type::Struct(strukt)) => (strukt.type_ref().clone(), SourceItem::Struct(strukt)),
        Some(Type::Extern(opaque)) => {
            // A declaration answers what reading names it; an extern keeps
            // none, so the model is asked to read its own name.
            let name = &opaque.name;
            match flat.classify(&syn::parse_quote!(#name)) {
                Ok(ty) => (ty, SourceItem::Extern(opaque)),
                Err(error) => {
                    return Err(PlanningError::InternalInvariant(format!(
                        "`{name}` is declared and does not classify as a reference to \
                             itself: {error}"
                    )))
                }
            }
        }
        None => {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.undeclared",
                    format!(
                        "`{}` is exposed as a type the source did not capture; v2 \
                             represents captured types only",
                        declaration.name()
                    ),
                ),
                &root,
            )))
        }
        Some(Type::Enum(_) | Type::Variant(_)) => {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.enum",
                    format!(
                        "`{}` is an enum, which v2 has no representation for yet",
                        declaration.name()
                    ),
                ),
                &root,
            )))
        }
    };

    let taken = match run.plan_value(
        Crossing {
            ty: ty.clone(),
            direction: Direction::IntoRust,
        },
        &root,
    )? {
        Planned::Ready(id) => id,
        Planned::Unsupported(refusal) => return Ok(Err(refusal)),
    };

    let mut given = None;
    let mut release_plan = None;
    if let Some(release) = run.nodes[taken.0].repr.release.clone() {
        given = match run.plan_value(
            Crossing {
                ty,
                direction: Direction::OutOfRust,
            },
            &root.child("out_of_rust"),
        )? {
            Planned::Ready(id) => Some(id),
            Planned::Unsupported(refusal) => return Ok(Err(refusal)),
        };
        // The release is a wrapper of its own, over the same carrier the
        // consuming conversion reads. Its boundary is the target's answer for
        // a site with no source function.
        let values = ResolvedValues {
            inputs: vec![&run.nodes[taken.0]],
            output: None,
        };
        let site = SiteDescriptor {
            declaration,
            function: None,
        };
        let boundary = match run.target.boundary(&site, &values)? {
            TargetAttempt::Ready(boundary) => boundary,
            TargetAttempt::Unsupported(reason) => return Ok(Err(Refusal::at(reason, &root))),
        };
        drop(values);
        if let Some(category) = release.failure_category() {
            if !boundary
                .failures
                .iter()
                .any(|route| route.category == category)
            {
                return Ok(Err(Refusal::at(
                    Unsupported::new(
                        "unsupported.boundary.unrouted_failure",
                        format!(
                            "releasing a handle can fail with a {} error and this boundary \
                             declares no route for it",
                            category.as_str()
                        ),
                    ),
                    &root,
                )));
            }
        }
        release_plan = match assemble(
            run,
            declaration,
            Body::Release(Box::new(release)),
            &[taken],
            None,
            boundary,
        )? {
            Ok(plan) => Some(plan),
            Err(refusal) => return Ok(Err(refusal)),
        };
    }

    let values = ResolvedValues {
        inputs: vec![&run.nodes[taken.0]],
        output: given.map(|id| &run.nodes[id.0]),
    };
    let surface = match run
        .target
        .surface(&SurfaceRequest { declaration, item }, &values)?
    {
        TargetAttempt::Ready(surface) => surface,
        TargetAttempt::Unsupported(reason) => return Ok(Err(Refusal::at(reason, &root))),
    };
    drop(values);
    Ok(Ok(Emitted {
        function: release_plan,
        surface,
    }))
}
