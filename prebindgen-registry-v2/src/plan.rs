//! Planning: a binding in, retained plans out.
//!
//! One recursive walk plans the value conversions, one pass per requested
//! output assembles the wrappers, and one fixpoint decides what survives. All
//! three read the [`Binding`] and the source model and nothing else: no target
//! code runs until the common Rust writer writes the plan.

use std::collections::{BTreeMap, HashMap, HashSet};

use prebindgen_flat::{
    flat::{Flat, Function, Type, TypeKind, TypeRef},
    Conditioned, RustEmitter,
};

use crate::{
    binding::{
        Binding, CarrierId, Failure, FailureRoute, FunctionFormOf, Implementation, Operation,
        OutputForm, OutputId, ReprId, Representation, Scope, Step, ValuePath, Via,
    },
    body::{BodyBuilder, Instr, NodeBody, Operand, Stmt, ValueId},
    decl::Declaration,
    outcome::{EngineError, Outcome, Skip},
    run::{check_declarations, Generation},
    target::{
        Crossing, Direction, FailureCategory, Part, PlanningError, Relation, StructRelation,
        Target, Unsupported,
    },
};

/// A retained conversion, reusable by every value that crosses the same way.
#[derive(Debug)]
pub struct ValuePlan {
    pub id: NodeId,
    pub crossing: Crossing,
    /// How the Rust value is built or read, as its representation names it.
    pub relation: Relation,
    /// The representation it crosses as.
    pub representation: ReprId,
    /// The carrier it crosses in, in this direction.
    pub carrier: CarrierId,
    pub children: Vec<NodeId>,
    pub body: NodeBody,
    /// Every failure category this conversion, or a conversion it uses, can
    /// raise. The boundary must route each of them.
    pub failures: Vec<FailureCategory>,
}

/// A retained conversion's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) usize);

/// A registered application of an operation, valid inside one generation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimitiveId(pub(crate) usize);

/// Where an operand or result of an applied operation comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// The source type of the value the conversion serves.
    Source,
    /// A carrier the binding declared.
    Carrier(CarrierId),
    /// What a callback's `capture` produced, which only the target knows the
    /// type of.
    Captured,
}

/// One use of an operation, with everything its writer will be fed.
#[derive(Clone, Debug)]
pub struct Applied<Op> {
    pub implementation: Implementation<Op>,
    /// The source type of the value the conversion serves.
    pub subject: TypeRef,
    /// What the conversion hands the operation.
    pub value: Slot,
    /// The runtime contexts it needs, by name, in the order it asked.
    pub contexts: Vec<String>,
    /// What it produces, if anything.
    pub result: Option<Slot>,
    /// The part it is applied to, for a per-part read.
    pub part: Option<Part>,
    /// For a callback's `capture` and `invoke`, the arguments' carriers, in
    /// order; `invoke` is handed their values as operands after the
    /// contexts.
    pub args: Vec<CarrierId>,
    pub failure: Option<Failure>,
}

/// What a wrapper parameter is for.
#[derive(Clone, Debug)]
pub enum ParamRole {
    /// Feeds the conversion of this source parameter.
    Input(usize),
    /// Supplies a runtime context operations ask for by this name.
    Context(String),
    /// Required by the calling convention and used by nothing.
    Unused,
}

/// One parameter of a wrapper.
#[derive(Clone, Debug)]
pub struct WrapperParam {
    pub name: syn::Ident,
    pub ty: syn::Type,
    pub role: ParamRole,
    /// Whether the parameter binding needs `mut`.
    pub mutable: bool,
}

/// A complete wrapper: the exported function, as planned.
///
/// One per declaration that exports a source function, plus one per handle
/// type for its release. The common Rust writer renders it as the
/// `#[no_mangle] extern` function a foreign caller links against.
#[derive(Debug)]
pub struct FunctionPlan<Op> {
    /// The declaration this wrapper exports.
    pub declaration: Declaration,
    pub abi: String,
    pub symbol: String,
    pub attrs: Vec<syn::Attribute>,
    pub unsafety: bool,
    /// Wrapper parameters, paired with the value identity that names each.
    pub params: Vec<(ValueId, WrapperParam)>,
    /// The wrapper's return type, absent when it returns nothing.
    pub ret: Option<syn::Type>,
    pub routes: Vec<FailureRoute<Op>>,
    pub instrs: Vec<Stmt>,
    /// The value the wrapper returns.
    pub result: Option<ValueId>,
}

/// One output that survived, and the values planned for it.
#[derive(Clone, Debug)]
pub struct Retained {
    pub output: OutputId,
    pub declaration: Declaration,
    /// A function's parameters, in source order; a type's own value, into Rust.
    pub inputs: Vec<NodeId>,
    /// A function's result; a handle type's own value, out of Rust.
    pub output_value: Option<NodeId>,
}

/// Where a value sits: what a rule at a position is looked up by, and what a
/// diagnostic prints.
#[derive(Clone, Debug)]
pub(crate) struct Position {
    output: OutputId,
    declaration: Declaration,
    path: ValuePath,
}

impl Position {
    fn root(output: OutputId, declaration: &Declaration) -> Self {
        Position {
            output,
            declaration: declaration.clone(),
            path: ValuePath::root(),
        }
    }

    fn child(&self, step: Step) -> Self {
        Position {
            output: self.output,
            declaration: self.declaration.clone(),
            path: self.path.then(step),
        }
    }

    /// This position as a skip's dependency path: the declaration it is
    /// rooted at, then how the walk reached here.
    fn dependency_path(&self) -> Vec<String> {
        std::iter::once(self.declaration.to_string())
            .chain(self.path.0.iter().map(Step::to_string))
            .collect()
    }
}

/// A conversion whose planning has begun, for cycle detection.
///
/// Deliberately coarser than [`NodeKey`]: what makes a cycle is a type reaching
/// its own conversion, which its children cannot be known before.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ResolvingKey {
    ty: prebindgen_flat::TypeKey,
    direction: Direction,
    representation: ReprId,
}

/// What makes two conversions the same conversion.
///
/// The children are part of it. A rule recorded for a field is looked up at
/// that field's position, so two structs of one representation whose fields
/// were configured differently resolve to different children — and must not
/// share a node, or the second use would silently inherit the first one's
/// conversion. Everything in it is the registry's to compare: equal
/// representations share an id.
#[derive(PartialEq, Eq, Hash)]
struct NodeKey {
    ty: prebindgen_flat::TypeKey,
    direction: Direction,
    representation: ReprId,
    children: Vec<NodeId>,
}

/// A capability that stopped a conversion, and where it was met.
struct Refusal {
    reason: Unsupported,
    at: Position,
}

impl Refusal {
    fn at(reason: Unsupported, at: &Position) -> Self {
        Refusal {
            reason,
            at: at.clone(),
        }
    }
}

/// A planned value, or the capability that stopped it.
enum Planned {
    Ready(NodeId),
    Unsupported(Refusal),
}

impl Planned {
    fn refused(reason: Unsupported, at: &Position) -> Self {
        Planned::Unsupported(Refusal::at(reason, at))
    }
}

/// The binding's rules, by scope: a type output's representation is the rule
/// at its root, beside the rules the binding recorded.
struct Rules {
    /// Scope → the recorded rule's index, `None` for a type output's own, and
    /// the representation.
    by_scope: HashMap<Scope, (Option<usize>, ReprId)>,
}

impl Rules {
    fn new<T: Target>(binding: &Binding<T>) -> Result<Self, PlanningError> {
        let mut by_scope = HashMap::new();
        for (index, (_, form)) in binding.outputs().iter().enumerate() {
            if let OutputForm::Type { representation, .. } = form {
                by_scope.insert(
                    Scope::At(binding.output_id(index), ValuePath::root()),
                    (None, *representation),
                );
            }
        }
        for (index, (scope, representation)) in binding.rules().iter().enumerate() {
            if by_scope
                .insert(scope.clone(), (Some(index), *representation))
                .is_some()
            {
                return Err(PlanningError::InvalidInput(format!(
                    "two rules cover {}: the binding says two things about one value",
                    describe_scope(binding, scope)
                )));
            }
        }
        Ok(Rules { by_scope })
    }

    /// The representation of the value at `path` inside `output`, of type
    /// `ty`: the rule at its position, else the rule for its type.
    fn lookup(
        &self,
        output: OutputId,
        path: &ValuePath,
        ty: &TypeRef,
    ) -> Option<(Option<usize>, ReprId)> {
        self.by_scope
            .get(&Scope::At(output, path.clone()))
            .or_else(|| self.by_scope.get(&Scope::Type(ty.key())))
            .copied()
    }
}

/// How a diagnostic names a rule's scope.
fn describe_scope<T: Target>(binding: &Binding<T>, scope: &Scope) -> String {
    match scope {
        Scope::Type(key) => format!("every `{}`", key.as_str()),
        Scope::At(output, path) => match binding.outputs().get(output.index) {
            Some((declaration, _)) if path.0.is_empty() => format!("`{declaration}` itself"),
            Some((declaration, _)) => format!("`{path}` of `{declaration}`"),
            None => format!("an output this binding does not have ({})", output.index),
        },
    }
}

/// The name a value path gives a source parameter: its identifier without
/// Rust's raw prefix, as a build script author writes it.
fn param_name(ident: &syn::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}

/// The working state of one `generate` call.
struct Run<'a, T: Target> {
    flat: &'a Flat,
    binding: &'a Binding<T>,
    rules: Rules,
    /// The recorded rules some value was planned by.
    used: HashSet<usize>,
    primitives: Vec<Applied<T::Op>>,
    nodes: Vec<ValuePlan>,
    cache: HashMap<NodeKey, NodeId>,
    resolving: HashSet<ResolvingKey>,
    /// The relations of a type, worked out the first time it is planned.
    offered: HashMap<prebindgen_flat::TypeKey, Vec<Relation>>,
}

impl<'a, T: Target> Run<'a, T> {
    fn new(flat: &'a Flat, binding: &'a Binding<T>, rules: Rules) -> Self {
        Run {
            flat,
            binding,
            rules,
            used: HashSet::new(),
            primitives: Vec::new(),
            nodes: Vec::new(),
            cache: HashMap::new(),
            resolving: HashSet::new(),
            offered: HashMap::new(),
        }
    }

    /// Register an operation's use, and hand back the identity instructions
    /// name it by.
    fn register(&mut self, applied: Applied<T::Op>) -> PrimitiveId {
        self.primitives.push(applied);
        PrimitiveId(self.primitives.len() - 1)
    }

    /// The relations of this type: the atomic one, and the struct's fields for
    /// a struct.
    fn relations_of(&mut self, ty: &TypeRef) -> Result<Vec<Relation>, Unsupported> {
        if let Some(offered) = self.offered.get(&ty.key()) {
            return Ok(offered.clone());
        }
        let mut relations = vec![Relation::Atomic];
        if let TypeKind::Callback { args } = ty.kind() {
            relations.push(Relation::Callback(
                args.iter()
                    .enumerate()
                    .map(|(index, arg)| Part {
                        name: None,
                        index,
                        ty: arg.clone(),
                        conditions: Vec::new(),
                    })
                    .collect(),
            ));
        }
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
                // An opaque declaration has no parts to offer: it is carried
                // whole, through the atomic relation.
                Some(_) => {}
                None => {
                    return Err(Unsupported::new(
                        "unsupported.type.undeclared",
                        format!("`{}` names a type this model does not declare", ty.key()),
                    ))
                }
            }
        }
        self.offered.insert(ty.key(), relations.clone());
        Ok(relations)
    }

    /// Plan one conversion, recursing into the parts of the relation its
    /// representation names.
    fn plan_value(
        &mut self,
        crossing: Crossing,
        position: &Position,
    ) -> Result<Planned, PlanningError> {
        let relations = match self.relations_of(&crossing.ty) {
            Ok(relations) => relations,
            Err(reason) => return Ok(Planned::refused(reason, position)),
        };
        let Some((rule, representation)) =
            self.rules
                .lookup(position.output, &position.path, &crossing.ty)
        else {
            return Ok(Planned::refused(
                Unsupported::new(
                    "unsupported.conversion.no_rule",
                    format!(
                        "no rule covers `{}`, and this binding has no representation for it",
                        crossing.ty.key()
                    ),
                ),
                position,
            ));
        };
        if let Some(rule) = rule {
            self.used.insert(rule);
        }
        let relation = match self.binding.representation_of(representation) {
            Representation::Unsupported(reason) => {
                return Ok(Planned::refused(reason.clone(), position))
            }
            Representation::Terminal { .. } => Relation::Atomic,
            Representation::Callback { .. } => {
                // Rust receives a callable; handing one of its own out would
                // need a foreign closure built around a Rust one, which no
                // target states yet.
                if crossing.direction == Direction::OutOfRust {
                    return Ok(Planned::refused(
                        Unsupported::new(
                            "unsupported.callback.out_of_rust",
                            format!(
                                "`{}` leaves Rust as a callable; v2 builds callables only \
                                 into Rust",
                                crossing.ty.key()
                            ),
                        ),
                        position,
                    ));
                }
                match relations
                    .iter()
                    .find(|relation| matches!(relation, Relation::Callback(_)))
                {
                    Some(found) => found.clone(),
                    None => {
                        return Ok(Planned::refused(
                            Unsupported::new(
                                "unsupported.type.not_a_callback",
                                format!(
                                    "`{}` is represented as a callback, and is not an \
                                     `impl Fn(..)`",
                                    crossing.ty.key()
                                ),
                            ),
                            position,
                        ))
                    }
                }
            }
            Representation::Product {
                via: Via::Fields, ..
            } => {
                if crossing.direction == Direction::OutOfRust {
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
                    ));
                }
                match relations
                    .iter()
                    .find(|relation| matches!(relation, Relation::Struct(_)))
                {
                    Some(found) => found.clone(),
                    None => {
                        return Ok(Planned::refused(
                            Unsupported::new(
                                "unsupported.type.not_a_struct",
                                format!(
                                    "`{}` is read through its fields, and the model has no \
                                     fields for it",
                                    crossing.ty.key()
                                ),
                            ),
                            position,
                        ))
                    }
                }
            }
        };

        // Meeting a conversion that is already being resolved is a cycle: a
        // type whose conversion needs its own. The mark is bookkeeping, not a
        // plan, and it is dropped whichever way this call ends.
        let resolving = ResolvingKey {
            ty: crossing.ty.key(),
            direction: crossing.direction,
            representation,
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
        let planned = self.plan_relation(&crossing, position, &relation, representation);
        self.resolving.remove(&resolving);
        planned
    }

    /// The body of `plan_value`, once the relation is known: children first,
    /// then the instructions that put them together.
    fn plan_relation(
        &mut self,
        crossing: &Crossing,
        position: &Position,
        relation: &Relation,
        representation: ReprId,
    ) -> Result<Planned, PlanningError> {
        let parts = relation.parts().to_vec();

        // Children are planned before this conversion's identity is known,
        // because a rule recorded for one of them makes this a different
        // conversion.
        // A callback's arguments cross the other way: Rust hands them to the
        // callable it received.
        let (direction, step): (Direction, fn(&Part) -> Step) = match relation {
            Relation::Callback(_) => (crossing.direction.reversed(), |part| Step::Arg(part.index)),
            _ => (crossing.direction, |part| Step::Field(part.label())),
        };
        let mut children = Vec::new();
        for part in &parts {
            let child = Crossing {
                ty: part.ty.clone(),
                direction,
            };
            match self.plan_value(child, &position.child(step(part)))? {
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
            representation,
            children: children.clone(),
        };
        if let Some(id) = self.cache.get(&key) {
            return Ok(Planned::Ready(*id));
        }

        let mut failures: Vec<FailureCategory> = children
            .iter()
            .flat_map(|id| self.nodes[id.0].failures.clone())
            .collect();
        let mut body = BodyBuilder::new();
        let input = body.fresh();
        let (carrier, result) = match self.binding.representation_of(representation) {
            Representation::Terminal {
                into_rust,
                out_of_rust,
                ..
            } => {
                let codec = match crossing.direction {
                    Direction::IntoRust => into_rust,
                    Direction::OutOfRust => out_of_rust,
                };
                let Some(codec) = codec else {
                    return Ok(Planned::refused(
                        Unsupported::new(
                            "unsupported.conversion.direction",
                            format!(
                                "`{}` has no conversion {} in this binding",
                                crossing.ty.key(),
                                match crossing.direction {
                                    Direction::IntoRust => "into Rust",
                                    Direction::OutOfRust => "out of Rust",
                                }
                            ),
                        ),
                        position,
                    ));
                };
                let (value, result) = match crossing.direction {
                    Direction::IntoRust => (Slot::Carrier(codec.carrier), Slot::Source),
                    Direction::OutOfRust => (Slot::Source, Slot::Carrier(codec.carrier)),
                };
                failures.extend(codec.operation.failure.iter().map(|f| f.category));
                let produced = self.apply(
                    &mut body,
                    &codec.operation,
                    &crossing.ty,
                    (value, input),
                    Some(result),
                    None,
                    &[],
                );
                (codec.carrier, produced)
            }
            Representation::Product { carrier, read, .. } => {
                let (carrier, read) = (*carrier, read.clone());
                let strukt = match relation {
                    Relation::Struct(strukt) => strukt.name.clone(),
                    Relation::Atomic | Relation::Callback(_) => {
                        return Err(PlanningError::InternalInvariant(
                            "a product representation needs a relation with parts".to_string(),
                        ))
                    }
                };
                // What the carrier may hold is its own to say: a member whose
                // part resolved to a wire type it does not hold refuses the
                // value that would put it there.
                let accepts = self
                    .binding
                    .carrier_of(carrier)
                    .members
                    .clone()
                    .ok_or_else(|| {
                        PlanningError::InvalidInput(format!(
                            "`{}` is carried in a carrier with no members",
                            crossing.ty.key()
                        ))
                    })?;
                for (part, child) in parts.iter().zip(&children) {
                    let class = &self.binding.carrier_of(self.nodes[child.0].carrier).class;
                    if !accepts.holds(class) {
                        return Ok(Planned::refused(
                            Unsupported::new(
                                format!("unsupported.{}.member.{}", T::NAME, spell_class(class)),
                                format!(
                                    "member `{}` of `{}` is carried as {class:?}, which its \
                                     carrier does not hold",
                                    part.label(),
                                    crossing.ty.key()
                                ),
                            ),
                            &position.child(Step::Field(part.label())),
                        ));
                    }
                }
                failures.extend(read.failure.iter().map(|f| f.category));
                let mut converted = Vec::new();
                for (part, child) in parts.iter().zip(&children) {
                    // Everything planned for this part, from the read of the
                    // member to the last step of its conversion, exists exactly
                    // where the source field does.
                    let from = body.len();
                    let obtained = self.apply(
                        &mut body,
                        &read,
                        &crossing.ty,
                        (Slot::Carrier(carrier), input),
                        Some(Slot::Carrier(self.nodes[child.0].carrier)),
                        Some(part.clone()),
                        &[],
                    );
                    let child_body = self.nodes[child.0].body.clone();
                    converted.push(child_body.inline(obtained, &mut body));
                    body.condition(from, &part.conditions);
                }
                let result = body.fresh();
                body.push(Instr::Construct {
                    name: strukt,
                    parts: converted,
                    result,
                });
                (carrier, result)
            }
            Representation::Callback {
                carrier,
                capture,
                invoke,
                routes,
            } => {
                let (carrier, capture, invoke, routes) =
                    (*carrier, capture.clone(), invoke.clone(), routes.clone());
                if let Some(refusal) = self.check_callback(
                    crossing, position, carrier, &parts, &children, &invoke, &routes,
                )? {
                    return Ok(Planned::Unsupported(refusal));
                }
                // What a call raises stays inside the closure, which routes
                // it; the wrapper sees only what capturing can raise.
                failures = capture.failure.iter().map(|f| f.category).collect();
                // Capturing runs before any call, and is told what the calls
                // will carry.
                let arg_carriers: Vec<(CarrierId, Option<ValueId>)> = children
                    .iter()
                    .map(|child| (self.nodes[child.0].carrier, None))
                    .collect();
                let captured = self.apply(
                    &mut body,
                    &capture,
                    &crossing.ty,
                    (Slot::Carrier(carrier), input),
                    Some(Slot::Captured),
                    None,
                    &arg_carriers,
                );
                // An identity capture leaves the carrier itself to be moved
                // into the closure.
                let captured_slot = match captured == input {
                    true => Slot::Carrier(carrier),
                    false => Slot::Captured,
                };
                let from = body.len();
                let params: Vec<(ValueId, TypeRef)> = parts
                    .iter()
                    .map(|part| (body.fresh(), part.ty.clone()))
                    .collect();
                let mut wires = Vec::new();
                for ((param, _), child) in params.iter().zip(&children) {
                    let child_body = self.nodes[child.0].body.clone();
                    let wire = child_body.inline(*param, &mut body);
                    wires.push((self.nodes[child.0].carrier, Some(wire)));
                }
                self.apply(
                    &mut body,
                    &invoke,
                    &crossing.ty,
                    (captured_slot, captured),
                    None,
                    None,
                    &wires,
                );
                let instrs = body.split_off(from);
                let result = body.fresh();
                body.push(Instr::Closure {
                    representation,
                    captured,
                    params,
                    instrs,
                    result,
                });
                (carrier, result)
            }
            Representation::Unsupported(_) => {
                return Err(PlanningError::InternalInvariant(
                    "an unsupported representation reached composition".to_string(),
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
            representation,
            carrier,
            children,
            body: NodeBody {
                carrier: input,
                instrs: body.into_instrs(),
                result,
            },
            failures,
        });
        self.cache.insert(key, id);
        Ok(Planned::Ready(id))
    }

    /// Whether a callback can be built from what its arguments resolved to:
    /// each argument's carrier is one the callback's carrier holds, and every
    /// failure a call can meet has a route of the callback's own, needing no
    /// runtime context — inside a call nothing supplies one.
    #[allow(clippy::too_many_arguments)]
    fn check_callback(
        &self,
        crossing: &Crossing,
        position: &Position,
        carrier: CarrierId,
        parts: &[Part],
        children: &[NodeId],
        invoke: &Operation<T::Op>,
        routes: &[FailureRoute<T::Op>],
    ) -> Result<Option<Refusal>, PlanningError> {
        let refused = |code: String, explanation: String, at: &Position| {
            Ok(Some(Refusal::at(Unsupported::new(code, explanation), at)))
        };
        let accepts = self
            .binding
            .carrier_of(carrier)
            .members
            .clone()
            .ok_or_else(|| {
                PlanningError::InvalidInput(format!(
                    "`{}` is carried in a carrier with no members",
                    crossing.ty.key()
                ))
            })?;
        for (part, child) in parts.iter().zip(children) {
            let class = &self.binding.carrier_of(self.nodes[child.0].carrier).class;
            if !accepts.holds(class) {
                return refused(
                    format!("unsupported.{}.arg.{}", T::NAME, spell_class(class)),
                    format!(
                        "argument {} of `{}` is carried as {class:?}, which the callback's \
                         carrier does not hold",
                        part.index,
                        crossing.ty.key()
                    ),
                    &position.child(Step::Arg(part.index)),
                );
            }
        }
        // Every operation a call applies: the arguments' conversions, then
        // the invocation.
        let mut applied: Vec<(Option<&Failure>, &[String])> = Vec::new();
        for child in children {
            for step in &self.nodes[child.0].body.instrs {
                if let Instr::Apply { primitive, .. } = &step.instr {
                    let primitive = &self.primitives[primitive.0];
                    applied.push((primitive.failure.as_ref(), &primitive.contexts));
                }
            }
        }
        applied.push((invoke.failure.as_ref(), &invoke.context));
        let reporters = routes
            .iter()
            .filter_map(|route| route.report.as_ref())
            .map(|report| (None, report.operation.context.as_slice()));
        for (_, contexts) in applied.iter().copied().chain(reporters) {
            if let Some(name) = contexts.first() {
                return refused(
                    "unsupported.callback.missing_context".to_string(),
                    format!(
                        "a call of `{}` needs the `{name}` runtime context, and nothing \
                         supplies one inside a call",
                        crossing.ty.key()
                    ),
                    position,
                );
            }
        }
        for failure in applied.iter().filter_map(|(failure, _)| *failure) {
            let Some(route) = routes
                .iter()
                .find(|route| route.category == failure.category)
            else {
                return refused(
                    "unsupported.callback.unrouted_failure".to_string(),
                    format!(
                        "a call of `{}` can fail with a {} error and its representation \
                         declares no route for it",
                        crossing.ty.key(),
                        failure.category.as_str()
                    ),
                    position,
                );
            };
            if let Some(report) = &route.report {
                if spell(&report.error) != spell(&failure.error) {
                    return Err(PlanningError::InvalidInput(format!(
                        "the {} failure route of `{}` reports a `{}` where a call raises a `{}`",
                        failure.category.as_str(),
                        crossing.ty.key(),
                        spell(&report.error),
                        spell(&failure.error)
                    )));
                }
            }
        }
        Ok(None)
    }

    /// Apply one operation to an already-available value, and hand back what
    /// it produced — the value itself when it produces nothing new.
    ///
    /// An infallible identity produces no instruction at all: the carrier and
    /// the converted value are one Rust value, so the conversion renders
    /// nothing and the caller keeps using the value it already had.
    #[allow(clippy::too_many_arguments)]
    fn apply(
        &mut self,
        body: &mut BodyBuilder,
        operation: &Operation<T::Op>,
        subject: &TypeRef,
        (slot, value): (Slot, ValueId),
        result: Option<Slot>,
        part: Option<Part>,
        args: &[(CarrierId, Option<ValueId>)],
    ) -> ValueId {
        if matches!(
            operation.implementation,
            Implementation::Standard(crate::binding::StandardOp::Identity)
        ) && operation.failure.is_none()
        {
            return value;
        }
        let mut operands = vec![Operand::Value(value)];
        operands.extend(operation.context.iter().cloned().map(Operand::Context));
        operands.extend(args.iter().filter_map(|(_, arg)| arg.map(Operand::Value)));
        let id = self.register(Applied {
            implementation: operation.implementation.clone(),
            subject: subject.clone(),
            value: slot,
            contexts: operation.context.clone(),
            result,
            part,
            args: args.iter().map(|(carrier, _)| *carrier).collect(),
            failure: operation.failure.clone(),
        });
        let produced = result.map(|_| body.fresh());
        body.push(Instr::Apply {
            primitive: id,
            operands,
            result: produced,
        });
        produced.unwrap_or(value)
    }
}

/// How a refusal spells a wire type: its `Debug` form, lower-cased.
fn spell_class(class: &impl std::fmt::Debug) -> String {
    format!("{class:?}").to_lowercase()
}

fn spell(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

/// Check every rule addressed to a position against the output it names,
/// before anything is planned.
///
/// A rule at a position the output does not have — a parameter the function
/// does not take, a field of a value nothing reads through its fields — would
/// otherwise sit unused, and the binding would believe it had configured
/// something.
fn check_paths<T: Target>(
    binding: &Binding<T>,
    rules: &Rules,
    flat: &Flat,
) -> Result<(), PlanningError> {
    for (scope, _) in binding.rules() {
        let Scope::At(output, path) = scope else {
            continue;
        };
        let invalid = |why: String| {
            PlanningError::InvalidInput(format!(
                "a rule covers {}, and {why}",
                describe_scope(binding, scope)
            ))
        };
        let Some((declaration, _)) = binding.outputs().get(output.index) else {
            return Err(invalid("the binding has no such output".to_string()));
        };
        let mut steps = path.0.iter();
        let mut ty = match declaration {
            Declaration::Function(ident) => {
                let function = flat
                    .function(&ident.to_string())
                    .ok_or_else(|| invalid("its function is not in the model".to_string()))?;
                match steps.next() {
                    Some(Step::Param(name)) => function
                        .params
                        .iter()
                        .find(|param| param_name(&param.name) == *name)
                        .map(|param| param.ty.clone())
                        .ok_or_else(|| invalid(format!("the function takes no `{name}`")))?,
                    Some(Step::Return) if !matches!(function.ret.kind(), TypeKind::Unit) => {
                        function.ret.clone()
                    }
                    Some(Step::Return) => {
                        return Err(invalid("the function returns nothing".to_string()))
                    }
                    Some(Step::Field(_) | Step::Arg(_)) | None => {
                        return Err(invalid(
                            "a function's values start at a parameter or its return".to_string(),
                        ))
                    }
                }
            }
            Declaration::Type(key) | Declaration::Callback(key) => flat
                .reading_of(key)
                .map_err(|error| invalid(format!("its type cannot be read: {error}")))?,
            _ => return Err(invalid("that output has no values".to_string())),
        };
        let mut walked = ValuePath(path.0[..path.0.len() - steps.len()].to_vec());
        for step in steps {
            // The representation the value there takes decides which steps
            // lead into it.
            let taken = rules
                .lookup(*output, &walked, &ty)
                .map(|(_, representation)| binding.representation_of(representation));
            if let Step::Arg(index) = step {
                let args = match ty.kind() {
                    TypeKind::Callback { args } => args,
                    _ => return Err(invalid(format!("`{}` takes no arguments", ty.key()))),
                };
                if !matches!(taken, Some(Representation::Callback { .. })) {
                    return Err(invalid(format!(
                        "`{}` is not represented as a callback there",
                        ty.key()
                    )));
                }
                ty = args
                    .get(*index)
                    .cloned()
                    .ok_or_else(|| invalid(format!("`{}` has no argument {index}", ty.key())))?;
                walked = walked.then(step.clone());
                continue;
            }
            let Step::Field(label) = step else {
                return Err(invalid(format!("`{step}` does not follow a value")));
            };
            let through_fields = matches!(
                taken,
                Some(Representation::Product {
                    via: Via::Fields,
                    ..
                })
            );
            if !through_fields {
                return Err(invalid(format!(
                    "`{}` is not read through its fields there",
                    ty.key()
                )));
            }
            let field = match ty.kind() {
                TypeKind::Named { id, .. } => match flat.resolve(id) {
                    Some(Type::Struct(strukt)) => strukt
                        .fields
                        .iter()
                        .find(|field| match &field.name {
                            Some(name) => name == label,
                            None => field.index.to_string() == *label,
                        })
                        .map(|field| field.ty.clone()),
                    _ => None,
                },
                _ => None,
            };
            ty = field.ok_or_else(|| invalid(format!("`{}` has no field `{label}`", ty.key())))?;
            walked = walked.then(step.clone());
        }
    }
    Ok(())
}

/// Plan `binding` over `flat`, and have `target` write what survives.
///
/// Every requested output leaves this with an outcome. A capability the binding
/// records as unsupported, or the engine has not implemented, is a reported skip
/// and the run continues; contradictory input and violated invariants fail.
pub fn generate<T: Target>(
    flat: Flat,
    target: &T,
    binding: Binding<T>,
    source_module: syn::Path,
) -> Result<Generation<T>, EngineError> {
    check_declarations(binding.outputs(), &flat)?;
    let rules = Rules::new(&binding).map_err(EngineError::Planning)?;
    check_paths(&binding, &rules, &flat).map_err(EngineError::Planning)?;
    for (declaration, form) in binding.outputs() {
        let forms: Vec<&FunctionFormOf<T>> = match form {
            OutputForm::Type { release, .. } => release.iter().collect(),
            OutputForm::Function { form, .. } => vec![form],
            OutputForm::Unsupported(_) => Vec::new(),
        };
        for form in forms {
            check_form(declaration, form).map_err(EngineError::Planning)?;
        }
    }

    let mut run = Run::new(&flat, &binding, rules);
    let mut functions: Vec<(OutputId, FunctionPlan<T::Op>)> = Vec::new();
    let mut retained: Vec<Retained> = Vec::new();
    let mut outcomes: BTreeMap<OutputId, Outcome> = BTreeMap::new();

    for (index, (declaration, form)) in binding.outputs().iter().enumerate() {
        let id = binding.output_id(index);
        let root = Position::root(id, declaration);
        // What is planned follows from the declaration, which says both what the
        // target asked for and what the captured source holds for it.
        let planned = match declaration {
            Declaration::Type(_) => {
                plan_type(&mut run, id, declaration, form).map_err(EngineError::Planning)?
            }
            // Whether the function was captured or is the binding's own makes
            // no difference here: the model holds both, and the writer reaches
            // each through its own origin.
            Declaration::Function(ident) => {
                let function = flat
                    .function(&ident.to_string())
                    .expect("declarations are checked against the model before planning");
                plan_function(&mut run, id, declaration, form, function)
                    .map_err(EngineError::Planning)?
            }
            // A constant computed on the foreign side has no value in Rust to
            // plan from.
            Declaration::ComputedConst(name) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.const.computed",
                    format!(
                        "`{name}` is computed by the binding on the foreign side, and the v2 \
                         engine has no way to place a foreign expression yet"
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
            Declaration::Callback(key) => plan_callback(&mut run, id, declaration, form, key)
                .map_err(EngineError::Planning)?,
            Declaration::Conversion(_) => Err(Refusal::at(
                Unsupported::new(
                    "unsupported.conversion.not_implemented",
                    "the v2 engine has no conversion lowering yet",
                ),
                &root,
            )),
        };
        match planned {
            Ok(Emitted { function, values }) => {
                if let Some(function) = function {
                    functions.push((id, function));
                }
                retained.push(values);
                outcomes.insert(id, Outcome::Emitted);
            }
            Err(refusal) => {
                // The path is where the walk actually stopped — the exported
                // function, the parameter, the field — so the report says what
                // to look at rather than only which declaration vanished.
                outcomes.insert(
                    id,
                    Outcome::Skipped(Skip {
                        capability: refusal.reason.capability,
                        explanation: refusal.reason.explanation,
                        dependency_path: refusal.at.dependency_path(),
                    }),
                );
            }
        }
    }

    // What an output requires, and the fixpoint deciding what survives.
    retain(&run, &binding, &retained, &mut outcomes);

    // Only what a retained output needs is published.
    let emitted = |id: &OutputId| matches!(outcomes.get(id), Some(Outcome::Emitted));
    functions.retain(|(id, _)| emitted(id));
    retained.retain(|values| emitted(&values.output));

    // What the binding asked for and did not get, in the order it asked.
    let skipped: Vec<(Declaration, Skip)> = binding
        .outputs()
        .iter()
        .enumerate()
        .filter_map(
            |(index, (declaration, _))| match outcomes.get(&binding.output_id(index)) {
                Some(Outcome::Skipped(skip)) => Some((declaration.clone(), skip.clone())),
                _ => None,
            },
        )
        .collect();
    // A rule for a type no planned value had. Not an error: a type rule is a
    // default for many values, and a scalar table covers kinds a binding may
    // never mention.
    let unused_rules: Vec<Scope> = binding
        .rules()
        .iter()
        .enumerate()
        .filter(|(index, (scope, _))| matches!(scope, Scope::Type(_)) && !run.used.contains(index))
        .map(|(_, (scope, _))| scope.clone())
        .collect();

    // Planning is over: the working state hands over its tables, and the model
    // moves into the frozen result.
    let nodes = std::mem::take(&mut run.nodes);
    let primitives = std::mem::take(&mut run.primitives);
    drop(run);

    let functions: Vec<FunctionPlan<T::Op>> = functions.into_iter().map(|(_, plan)| plan).collect();
    let reach = crate::emit::Reach::new(&flat, source_module);
    let rust = crate::emit::render(
        &flat,
        target,
        &binding,
        &reach,
        &retained,
        &nodes,
        &primitives,
        &functions,
    );

    Ok(Generation::new(
        flat,
        T::NAME,
        binding,
        skipped,
        unused_rules,
        nodes,
        retained,
        functions,
        rust,
    ))
}

/// Check the part of a function form that is not a planned value.
fn check_form<Op, C>(
    declaration: &Declaration,
    form: &crate::binding::FunctionForm<Op, C>,
) -> Result<(), PlanningError> {
    // A symbol reaches generated Rust as a function name, so a frontend that
    // supplies something else is contradictory input rather than a name the
    // writer has to escape.
    if syn::parse_str::<syn::Ident>(&form.symbol).is_err() {
        return Err(PlanningError::InvalidInput(format!(
            "`{}` exports the symbol `{}`, which is not a Rust identifier",
            declaration, form.symbol
        )));
    }
    // The writer exports the wrapper under that symbol with `#[no_mangle]`; an
    // attribute restating or contradicting the linkage is not a form the
    // binding may ask for, and rustc would only report the clash later.
    for attr in &form.attrs {
        if attr.path().is_ident("no_mangle") || attr.path().is_ident("export_name") {
            return Err(PlanningError::InvalidInput(format!(
                "`{}` states `#[{}]` on its wrapper, whose linkage the writer owns: the \
                 symbol is `{}`",
                declaration,
                attr.path()
                    .require_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                form.symbol
            )));
        }
    }
    Ok(())
}

/// Decide what survives: an output whose values need a type declared is
/// skipped unless that declaration survives too, until a pass changes nothing.
///
/// A value of a named type requires the type output exposing the
/// representation the value crossed as, and no other: a foreign signature
/// naming the type needs the declaration of that representation. A value of a
/// type whose output the frontend refused inherits the refusal; a value no
/// output exposes the representation of requires what is not there. A
/// requirement does not export anything: whatever needed a missing declaration
/// is skipped instead.
fn retain<T: Target>(
    run: &Run<'_, T>,
    binding: &Binding<T>,
    retained: &[Retained],
    outcomes: &mut BTreeMap<OutputId, Outcome>,
) {
    // Every type and callback output, by the name a value of it requires it
    // under, with the representation it exposes — none for one the frontend
    // refused, which a value of the type still requires, and so goes down
    // with.
    let by_name: BTreeMap<String, Vec<(OutputId, Option<ReprId>)>> = binding
        .outputs()
        .iter()
        .enumerate()
        .filter_map(|(index, (declaration, form))| {
            let representation = match form {
                OutputForm::Type { representation, .. } => Some(*representation),
                _ => None,
            };
            let name = match declaration {
                Declaration::Type(_) => declaration.entity_name()?,
                Declaration::Callback(key) => key.as_str().to_string(),
                _ => return None,
            };
            Some((name, (binding.output_id(index), representation)))
        })
        .fold(BTreeMap::new(), |mut all, (name, id)| {
            all.entry(name).or_default().push(id);
            all
        });
    // The values of each surviving output that a declaration must stand
    // behind: a function's parameters and result, a type's parts.
    let required = |values: &Retained| -> Vec<NodeId> {
        match &values.declaration {
            Declaration::Type(_) | Declaration::Callback(_) => values
                .inputs
                .iter()
                .flat_map(|root| run.nodes[root.0].children.clone())
                .collect(),
            _ => values
                .inputs
                .iter()
                .copied()
                .chain(values.output_value)
                .collect(),
        }
    };
    let resolve = |node: NodeId| -> Option<Result<OutputId, Skip>> {
        let plan = &run.nodes[node.0];
        // A named type is required by its name, a callback by its signature;
        // a value of any other type names nothing a binding declares.
        let (name, shown, word) = match plan.crossing.ty.kind() {
            TypeKind::Named { id, .. } => (id.name.clone(), id.name.clone(), "type"),
            TypeKind::Callback { .. } => {
                let key = plan.crossing.ty.key();
                let shown = prebindgen_flat::close_up(key.as_str());
                (key.as_str().to_string(), shown, "callback")
            }
            _ => return None,
        };
        let declared = by_name.get(&name).map(Vec::as_slice).unwrap_or_default();
        // The output exposing the representation the value crossed as; failing
        // that, one the frontend refused, whose skip the value inherits.
        let found = declared
            .iter()
            .find(|(_, representation)| *representation == Some(plan.representation))
            .or_else(|| {
                declared
                    .iter()
                    .find(|(_, representation)| representation.is_none())
            })
            .map(|(output, _)| *output);
        Some(found.ok_or_else(|| {
            let explanation = match declared {
                [] => {
                    format!("requires {word} `{shown}`, which this binding declares no {word} for")
                }
                _ => format!(
                    "requires {word} `{shown}` as {}, which no {word} output of it exposes",
                    plan.representation
                ),
            };
            Skip::direct(
                "unsupported.requirement.unrequested",
                explanation,
                format!("{word} `{shown}`"),
            )
        }))
    };
    loop {
        let mut changed = false;
        for values in retained {
            if !matches!(outcomes.get(&values.output), Some(Outcome::Emitted)) {
                continue;
            }
            for node in required(values) {
                let cause = match resolve(node) {
                    None => continue,
                    Some(Ok(required)) => match outcomes.get(&required) {
                        Some(Outcome::Skipped(skip)) => skip.clone(),
                        _ => continue,
                    },
                    Some(Err(skip)) => skip,
                };
                let mut path = vec![values.declaration.to_string()];
                path.extend(cause.dependency_path.iter().cloned());
                outcomes.insert(
                    values.output,
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
}

/// A requested output that survived planning.
struct Emitted<Op> {
    function: Option<FunctionPlan<Op>>,
    values: Retained,
}

/// Plan a callback signature's own value: the closure its representation
/// builds, into Rust. It exports no function, and exists so that a function
/// taking the callback has a declaration to require — the foreign type its
/// parameter is written as — and so the target declares the carrier once.
fn plan_callback<T: Target>(
    run: &mut Run<'_, T>,
    id: OutputId,
    declaration: &Declaration,
    form: &crate::binding::OutputFormOf<T>,
    key: &prebindgen_flat::TypeKey,
) -> Result<Result<Emitted<T::Op>, Refusal>, PlanningError> {
    let root = Position::root(id, declaration);
    match form {
        OutputForm::Type { release: None, .. } => {}
        OutputForm::Type {
            release: Some(_), ..
        } => {
            return Err(PlanningError::InvalidInput(format!(
                "`{declaration}` is a callback, and a callback has nothing to release"
            )))
        }
        OutputForm::Unsupported(reason) => return Ok(Err(Refusal::at(reason.clone(), &root))),
        OutputForm::Function { .. } => {
            return Err(PlanningError::InvalidInput(format!(
                "`{declaration}` declares a callback, and is recorded as a function"
            )))
        }
    }
    let ty = match run.flat.reading_of(key) {
        Ok(ty) if matches!(ty.kind(), TypeKind::Callback { .. }) => ty,
        Ok(_) => {
            return Err(PlanningError::InvalidInput(format!(
                "`{declaration}` names `{}`, which is not an `impl Fn(..)`",
                key.as_str()
            )))
        }
        Err(error) => {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.key",
                    format!(
                        "`{}` is declared over a type this model cannot read: {error}",
                        key.as_str()
                    ),
                ),
                &root,
            )))
        }
    };
    let taken = match run.plan_value(
        Crossing {
            ty,
            direction: Direction::IntoRust,
        },
        &root,
    )? {
        Planned::Ready(node) => node,
        Planned::Unsupported(refusal) => return Ok(Err(refusal)),
    };
    Ok(Ok(Emitted {
        function: None,
        values: Retained {
            output: id,
            declaration: declaration.clone(),
            inputs: vec![taken],
            output_value: None,
        },
    }))
}

/// Plan the wrapper that exports one source function: the conversions of its
/// parameters, the call, the conversion of its result, and the wrapper's
/// interface around them.
fn plan_function<T: Target>(
    run: &mut Run<'_, T>,
    id: OutputId,
    declaration: &Declaration,
    form: &crate::binding::OutputFormOf<T>,
    function: &Function,
) -> Result<Result<Emitted<T::Op>, Refusal>, PlanningError> {
    let root = Position::root(id, declaration);

    // The wrapper is a safe function, and the writer renders a plain call.
    // Wrapping an `unsafe fn` would need the wrapper to state the caller's
    // obligations, which nothing here can do yet.
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
    for param in &function.params {
        let position = root.child(Step::Param(param_name(&param.name)));
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
            &root.child(Step::Return),
        )? {
            Planned::Ready(id) => Some(id),
            Planned::Unsupported(refusal) => return Ok(Err(refusal)),
        }
    };

    // A declarator the target does not lower is refused once every value it
    // takes has a carrier, so a value nothing can carry is what the report
    // names when it is the first thing missing.
    let form = match form {
        OutputForm::Function { form, .. } => form,
        OutputForm::Unsupported(reason) => return Ok(Err(Refusal::at(reason.clone(), &root))),
        OutputForm::Type { .. } => {
            return Err(PlanningError::InvalidInput(format!(
                "`{declaration}` exports a function, and is recorded as a type"
            )))
        }
    };
    let values: Vec<(NodeId, Option<usize>)> = inputs
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, Some(index)))
        .chain(output.map(|id| (id, None)))
        .collect();
    for (node, param) in &values {
        let class = &run.binding.carrier_of(run.nodes[node.0].carrier).class;
        let (holder, accepts, position) = match param {
            Some(index) => (
                "param",
                &form.params,
                root.child(Step::Param(param_name(&function.params[*index].name))),
            ),
            None => ("return", &form.ret, root.child(Step::Return)),
        };
        if !accepts.holds(class) {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    format!("unsupported.{}.{holder}.{}", T::NAME, spell_class(class)),
                    format!("a wrapper {holder} cannot be {class:?} here"),
                ),
                &position,
            )));
        }
    }
    let plan = match assemble(
        run,
        id,
        declaration,
        Body::Call(function),
        &inputs,
        output,
        form,
    )? {
        Ok(plan) => plan,
        Err(refusal) => return Ok(Err(refusal)),
    };
    Ok(Ok(Emitted {
        function: Some(plan),
        values: Retained {
            output: id,
            declaration: declaration.clone(),
            inputs,
            output_value: output,
        },
    }))
}

/// What a wrapper does with its converted inputs.
enum Body<'a, Op> {
    /// Call the source function once with them.
    Call(&'a Function),
    /// Apply this operation to the one carrier and produce nothing: a handle's
    /// release, which is a wrapper with no source function behind it.
    Release(Operation<Op>),
}

/// Put one wrapper together: wrapper parameters in, conversions, one call — or,
/// for a release, one drop — the result out.
fn assemble<T: Target>(
    run: &mut Run<'_, T>,
    id: OutputId,
    declaration: &Declaration,
    action: Body<'_, T::Op>,
    inputs: &[NodeId],
    output: Option<NodeId>,
    form: &FunctionFormOf<T>,
) -> Result<Result<FunctionPlan<T::Op>, Refusal>, PlanningError> {
    let root = Position::root(id, declaration);
    // Every failure the wrapper can meet needs a terminal action here: the
    // conversions' for a call, the release's own for a release, which drops
    // the carrier without converting it.
    let mut categories: Vec<FailureCategory> = match &action {
        Body::Call(_) => inputs
            .iter()
            .chain(output.iter())
            .flat_map(|node| run.nodes[node.0].failures.clone())
            .collect(),
        Body::Release(release) => release
            .failure
            .iter()
            .map(|failure| failure.category)
            .collect(),
    };
    categories.sort();
    categories.dedup();
    for category in categories {
        if !form.routes.iter().any(|route| route.category == category) {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.boundary.unrouted_failure",
                    format!(
                        "a conversion can fail with a {} error and this boundary declares no \
                         route for it",
                        category.as_str()
                    ),
                ),
                &root,
            )));
        }
    }
    if form.inputs.len() != inputs.len() {
        return Err(PlanningError::InvalidInput(format!(
            "`{declaration}` names {} wrapper parameter(s) for {} input(s)",
            form.inputs.len(),
            inputs.len()
        )));
    }

    let mut body = BodyBuilder::new();
    let mut params = Vec::new();
    let mut contexts: BTreeMap<String, ValueId> = BTreeMap::new();
    for context in &form.context {
        let value = body.fresh();
        let role = match &context.supplies {
            Some(name) => {
                contexts.insert(name.clone(), value);
                ParamRole::Context(name.clone())
            }
            None => ParamRole::Unused,
        };
        params.push((
            value,
            WrapperParam {
                name: context.name.clone(),
                ty: context.ty.clone(),
                role,
                mutable: context.mutable,
            },
        ));
    }
    let mut arguments = Vec::new();
    for (index, node) in inputs.iter().enumerate() {
        let carrier = body.fresh();
        params.push((
            carrier,
            WrapperParam {
                name: form.inputs[index].clone(),
                ty: run
                    .binding
                    .carrier_of(run.nodes[node.0].carrier)
                    .rust
                    .clone(),
                role: ParamRole::Input(index),
                mutable: false,
            },
        ));
        match &action {
            Body::Call(_) => {
                let node_body = run.nodes[node.0].body.clone();
                arguments.push(node_body.inline(carrier, &mut body));
            }
            // A release takes the carrier as the conversion would, and drops
            // what it holds instead of converting it.
            Body::Release(release) => {
                let subject = run.nodes[node.0].crossing.ty.clone();
                let slot = Slot::Carrier(run.nodes[node.0].carrier);
                run.apply(
                    &mut body,
                    release,
                    &subject,
                    (slot, carrier),
                    None,
                    None,
                    &[],
                );
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

    let instrs = body.into_instrs();
    // An operation that asks for a runtime context the boundary does not supply
    // cannot be assembled. That covers the reporting operations as much as the
    // conversions.
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
    for route in &form.routes {
        if let Some(report) = &route.report {
            needed.extend(report.operation.context.iter());
        }
    }
    // The reporter is handed the error the operation produced, so the two have
    // to be the same type; a route that reports something else would not
    // compile.
    for step in &instrs {
        let Instr::Apply { primitive, .. } = &step.instr else {
            continue;
        };
        let Some(failure) = &run.primitives[primitive.0].failure else {
            continue;
        };
        let Some(route) = form
            .routes
            .iter()
            .find(|route| route.category == failure.category)
        else {
            continue;
        };
        let Some(report) = &route.report else {
            continue;
        };
        if spell(&report.error) != spell(&failure.error) {
            return Err(PlanningError::InvalidInput(format!(
                "the {} failure route of `{}` reports a `{}` where the operation raises a `{}`",
                failure.category.as_str(),
                declaration,
                spell(&report.error),
                spell(&failure.error)
            )));
        }
    }
    for name in needed {
        if !contexts.contains_key(name) {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.boundary.missing_context",
                    format!(
                        "an operation needs the `{name}` runtime context and this boundary \
                         supplies none"
                    ),
                ),
                &root,
            )));
        }
    }

    let ret = output.map(|node| {
        run.binding
            .carrier_of(run.nodes[node.0].carrier)
            .rust
            .clone()
    });
    Ok(Ok(FunctionPlan {
        declaration: declaration.clone(),
        abi: form.abi.clone(),
        symbol: form.symbol.clone(),
        attrs: form.attrs.clone(),
        unsafety: form.unsafety,
        params,
        ret,
        routes: form.routes.clone(),
        instrs,
        result,
    }))
}

/// Plan one exported type: the conversion everything taking it needs — and,
/// for a representation the foreign side holds an obligation for, the
/// conversion handing one out and the release freeing it.
///
/// What says "handle" is the representation naming a release; the registry
/// then requires the out-of-Rust direction too — a handle is a promise that
/// what a function returns can be given back — and plans the release as a
/// wrapper under the type's own identity.
fn plan_type<T: Target>(
    run: &mut Run<'_, T>,
    id: OutputId,
    declaration: &Declaration,
    form: &crate::binding::OutputFormOf<T>,
) -> Result<Result<Emitted<T::Op>, Refusal>, PlanningError> {
    let flat = run.flat;
    let root = Position::root(id, declaration);
    let Declaration::Type(key) = declaration else {
        return Err(PlanningError::InternalInvariant(format!(
            "`{declaration}` was routed to the type planner"
        )));
    };
    let (representation, release_form) = match form {
        OutputForm::Type {
            representation,
            release,
            ..
        } => (*representation, release.as_ref()),
        OutputForm::Unsupported(reason) => return Ok(Err(Refusal::at(reason.clone(), &root))),
        OutputForm::Function { .. } => {
            return Err(PlanningError::InvalidInput(format!(
                "`{declaration}` declares a type, and is recorded as a function"
            )))
        }
    };
    let name = declaration
        .entity_name()
        .expect("a type declaration names an entity");
    // An alternative with a field makes a sum, which is a different lowering
    // from a named set of integers and has none yet.
    match flat.declared_type(&name) {
        Some(Type::Variant(_)) => {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.variant",
                    format!(
                        "`{name}` has alternatives that carry values, which v2 has no \
                         representation for yet"
                    ),
                ),
                &root,
            )))
        }
        Some(_) => {}
        None => {
            return Err(PlanningError::InternalInvariant(format!(
                "`{name}` was checked against the model and is not in it"
            )))
        }
    }
    // The item is looked up by its name, and the type is read from the key as
    // the binding declared it: `ptr_class!(Publisher<'static>)` names the item
    // `Publisher` and means values of `Publisher<'static>`.
    let ty = match flat.reading_of(key) {
        Ok(ty) => ty,
        Err(error) => {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.key",
                    format!(
                        "`{}` is declared over a type this model cannot read: {error}",
                        key.as_str()
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
    let release = match run.binding.representation_of(representation) {
        Representation::Terminal { release, .. } => release.clone(),
        _ => None,
    };
    let mut given = None;
    let mut release_plan = None;
    if let Some(release) = release {
        let handed_out = match run.plan_value(
            Crossing {
                ty,
                direction: Direction::OutOfRust,
            },
            &root,
        )? {
            Planned::Ready(id) => id,
            Planned::Unsupported(refusal) => return Ok(Err(refusal)),
        };
        given = Some(handed_out);
        let Some(release_form) = release_form else {
            return Ok(Err(Refusal::at(
                Unsupported::new(
                    "unsupported.type.no_release",
                    format!(
                        "`{declaration}` hands out a value to be released, and the binding \
                         names no function releasing it"
                    ),
                ),
                &root,
            )));
        };
        // The release frees what a function handed out, so it takes the
        // out-of-Rust carrier, which may differ from the one taken back in.
        release_plan = match assemble(
            run,
            id,
            declaration,
            Body::Release(release),
            &[handed_out],
            None,
            release_form,
        )? {
            Ok(plan) => Some(plan),
            Err(refusal) => return Ok(Err(refusal)),
        };
    }
    Ok(Ok(Emitted {
        function: release_plan,
        values: Retained {
            output: id,
            declaration: declaration.clone(),
            inputs: vec![taken],
            output_value: given,
        },
    }))
}
