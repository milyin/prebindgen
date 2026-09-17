<!-- spec: {"kind": "stage", "stage": "04-values"} -->

[Project contents](../README.md) · Previous: [Record binding requests](03-requests.md) · Next: [Assemble the native boundary](05-boundary.md)

# Plan value conversions

Status: implemented for what the two element paths need, which is a value
carried whole or a record read through its fields. The vocabulary below reaches
further than that, and every part of it the engine does not have is marked where
it is described.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

At this point the engine knows the Rust source and the requested foreign API.
It now needs a recipe for moving each value between them. A **plan** is that
recipe, stored as structured data during the build. The plan does not call your
Rust function during generation; it describes operations that generated code
will execute later when a foreign caller uses the binding.

A foreign caller does not call `stamp_sum`. It calls a generated entry point in
its own language: a C function declared in a generated header, taking a
`repr(C)` struct of two integers, or a Kotlin method taking a JVM object.
Behind that entry point is the generated
[wrapper](05-boundary.md#assemble-the-native-boundary), a Rust function whose
ABI the foreign side can reach. It receives what the caller sent, and has to
call `stamp_sum`, whose signature demands an owned `source::Stamp`.

So the wrapper builds that `Stamp` out of what arrived, calls `stamp_sum`, and
turns the returned `i64` into something it can send back. Under the
configuration these chapters use, building it means reading two fields —
members of the C struct, getters on the JVM object — and constructing
`Stamp { secs, nanos }`. That is the selected route, not the only one: a binding
configured to carry `Stamp` whole, as an opaque handle, reads no fields at all. Each of those
value-shaped problems is a **conversion**, and planning one is what this stage
does. Ordering them around the source call is
[the next stage](05-boundary.md#assemble-the-native-boundary); here each value
is planned on its own.

Three separate questions have to be answered for every conversion, and keeping
them apart is what lets one algorithm serve both languages. Everything here
happens while the binding is being generated, from what the build script
configured; no part of it is decided while the binding runs. Two of the
questions are answered by the **target**: the
[language adapter](../README.md#the-components) for the binding being generated
— `prebindgen-c` or `prebindgen-jni` — in the role it plays facing the engine,
answering for its own language item by item. The third is the registry's alone.

- **On the Rust side, what is this value made of?** A `Stamp` is made of its two
  fields, so obtaining one means obtaining two `i64`s and constructing
  `Stamp { secs, nanos }` from them. An `i64` is made of nothing smaller: it is
  converted whole. Each such answer is a
  [**relation**](#what-a-relation-is) — a link from a Rust type to the Rust
  values it is built from or read into, together with the source-level means of
  getting between them. Relations are pure source-side facts, and the registry
  is the only component that walks one.

  Which relation applies is the target's answer, though, because it follows from
  the configuration. Declared as a C struct or
  a Kotlin data class, `Stamp` is built from its fields. Declared as an opaque
  handle, it is carried whole and its fields are never read. So the registry
  lists what the source model offers for the type — the fields, if the type is a
  record, and the whole value in any case — and the target names the one its
  configuration calls for. When the configuration calls for something V2 has no
  lowering for, such as one of the C declarators it does not implement yet, the
  target answers that instead of naming a relation, and every output needing
  that value is
  [skipped](06-retain.md#unsupported-requests-and-public-api-dependencies) with
  that reason. A skip is the answer to missing support, not to bad input:
  configuration that contradicts the source fails the build instead.
- **On the foreign side, what carries the value, and how is it accessed?** A
  by-value C struct whose members are read with ordinary field reads, or a JVM
  object whose properties are read by calling `getSecs()` and `getNanos()`
  through JNI. This answer is a **representation**, and only the target can give
  it: nothing in the registry knows what a C struct or a JVM object is.
- **How are the pieces put together?** Read each part, convert it, construct the
  Rust value, in that order, stopping if a step fails. This is the registry's
  job, and it is identical in both languages.

Put together, planning one conversion is this recursion — the registry's own
loop, with the two target questions marked:

```text
plan(type, direction, position):
    policy   = effective policy for this position     # site path, else type policy,
                                                      # else run default
    relation = target.select(type, direction, applicable rules, policy)
                                                      # which way out of this type;
                                                      # cheap: no recursion yet
    mark (type, direction, relation, policy) as being resolved
                                                      # meeting this mark again is a cycle

    parts    = the source model's parts of that relation
                   # where that choice leads: the fields of a record, the
                   # arguments of a constructor; nowhere at all for a scalar
    children = [ plan(part.type, direction, that part's position)
                 for part in parts ]
    if any child is unsupported:
        this conversion is unsupported, and so is everything that needed it

    if a node exists for (type, direction, relation, policy, children):
        return it                                     # the identity is complete
                                                      # only once the children are

    repr = target.represent(relation, children, policy)
               # which carriers hold the value, and the operations that access them
    body = compose(relation, children, repr)
               # obtain each part, convert it, construct the Rust value — or the
               # reverse, when the direction is out of Rust

    record the node and return it
```

The algorithm is, in other words, a depth-first walk that starts at a requested
type, takes one outgoing relation per type it reaches, and builds its answer on
the way back up. The value it builds is a node, and equal nodes are made one.

The algorithm saves completed plans in a **cache** so it can reuse them. Reuse
requires more than matching the Rust type. Building `Stamp` from two fields is
different from calling a constructor with one integer. Even two field-based
conversions differ if they convert `secs` differently. The cache key therefore
includes the selected relation, the
[policy](03-requests.md#what-policy-means) for this value, and the completed
child plans. The recursion resolves the children before it has that full key.

The algorithm also receives the value's *position*, not just its type. In the
design vocabulary, a `SiteId`
(parameter 0 of this exported function) or a `PartId` (the `secs` field of this
relation) is what an override is recorded against, so the position is what turns
the recorded rules into this conversion's effective policy. Positions are how
overrides reach a nested child. Current code uses `Position { declaration, path }`
instead of separate `SiteId`/`PartId` structs. The resulting node is still shared by identity,
so two positions that resolve to the same key get the same node.

For `Stamp` the recursion is one level deep: two `i64` children that need no work
of their own. A record with a record field simply makes `plan` call itself again,
and neither adapter learns anything about the nesting — which is the point.

The unit the target supplies for the second question is a **primitive**: one
typed operation, such as "read the `secs` member of a `Stamp`" or "call the
`getSecs()` getter on this object with this environment". A primitive describes
an operation, not a use of it — it names no variable and belongs to no exported
function — so the same description can be applied wherever that operation is
needed. Along with the operation, the target states what the operation needs,
what it produces, whether it can fail, and any generated helper it depends on.
The extended contract will also describe validity and resource obligations.
Those facts let the registry compose
operations safely instead of pasting text together.

The result is a **node**, meaning one reusable conversion plan in the engine's
dependency graph. It records the type, direction, selected relation, child
conversions, target representation and instructions. The broader design also
gives it validity and resource contracts; those are not implemented for borrowed
or resource-bearing values yet. Two functions taking an owned `Stamp` in the
same way can share a node. A borrow such as `&Stamp` needs a different node, as
does a use with different field conversions or target policy.

Nodes and the children they name form the graph that every later stage reads,
and two of its properties are worth stating outright, because the algorithm
above is what produces them.

It is **acyclic**, by construction rather than by luck: a node is recorded only
once every child it needs already exists, so an edge out of a node always points
at something complete. Nothing in the graph can point back at a plan still being
built, because such a plan is not in the graph at all.

It is a graph and not a tree, because the cache makes equal subplans the same
node. Planning `stamp_sum` produces four edges over three nodes:

```text
   wrapper for stamp_sum
     |
     +-- input  --> node: Stamp, into Rust, Stamp.fields
     |                       |
     |                       +-- part secs  --+
     |                       |                +--> node: i64, into Rust, atomic
     |                       +-- part nanos --+
     |
     +-- output --> node: i64, out of Rust, atomic
```

Both fields converge on one node, applied twice: same type, same direction, same
relation, same policy, so the same plan. The two `i64` nodes do not merge,
because direction is part of a node's identity — reading an integer out of a
foreign argument and handing one back are different operations.

The registry walks the record and combines its child plans. The target answers
local representation questions, such as how to read one member. This division
lets supported nested combinations reuse the traversal algorithm. It does not
make a new kind of value automatically supported: an optional field still needs
the optional-value protocol, which this increment does not implement.

A conversion is planned all the way or not at all. If any child conversion is
unsupported — a field of a type nothing can carry yet — the node is unsupported,
and every request that needed it is skipped with that reason. That propagation
runs backwards along the edges just drawn: one refused leaf refuses every
conversion that reaches it, and then every request that needed one of those.

No half-built node is ever published: while planning is in progress the registry
marks the conversion as being resolved, which is how it detects a cycle, but
that mark is bookkeeping, not a plan, and it is replaced by a node or by an
unsupported result. No later stage sees a conversion that half exists.

That mark is also the reason the plan graph can promise to be acyclic while the
source graph it walks makes no such promise. A Rust type may perfectly well be
described in terms of itself:

```text
   struct Branch { pub child: Box<Branch>, pub id: i64 }

   plan(Branch, into Rust, Branch.fields)
     +-- part child --> plan(Branch, into Rust, Branch.fields)   <-- still open
```

Meeting a conversion that is already open is a cycle in the source graph, and
the registry answers it with `unsupported.conversion.recursive` rather than
descending forever. Acyclicity is therefore something planning enforces at this
one point, not something the source model guarantees; a recursive conversion
becomes supported by giving that case a real answer, not by relaxing the check.

## Describing source construction and decomposition

For `normalize(stamp: Stamp) -> Stamp`, the wrapper converts foreign input into a Rust `Stamp`, calls `normalize`, and converts its result for foreign code. Here, **value** means a runtime argument, result or field. Input **construction** can assemble `Stamp { secs, nanos }` from converted fields or call `stamp_from_millis` with one converted argument. Output **decomposition** can read fields or call an accessor. A constructor does not automatically provide an inverse accessor.

### What a relation is

A **relation** is a link inside the source domain: from one Rust type to the
Rust values it is built from or read into, together with the source-level means
of getting between them. `Stamp` is related to `(secs: i64, nanos: i64)` by its
fields; it would be related to `(millis: i64)` by `stamp_from_millis`, and to
`StampParts` by an accessor `stamp_parts(&Stamp)`. Each of those is a relation
of `Stamp`, and a type can stand in several at once. Nothing in a relation names
a carrier, a wire type, a Kotlin class or a C struct: it is the answer to "how is
the Rust value built or read?", and that answer is the same whichever language
is on the other side.

Relations are links between source types, so they form a graph over those types,
and planning is a walk across it. Drawing that graph exactly is worth the effort,
because three of its properties account for most of the algorithm above.

**A relation links one type to several others at once.** `Stamp` is not related
to `secs` and separately to `nanos`; it is related to the pair, and a conversion
needs both or neither. So the relation is not a single edge but a bundle of them
— a hyperedge — and the edge out to one type is a part of it. Drawing the
relation as a box of its own puts both on the page:

```text
              relations of Stamp        parts of each

    Stamp --+--> [ Stamp.fields ] --+--> i64   (secs,  part 0)
            |                       |
            |                       +--> i64   (nanos, part 1)
            |
            +--> [ atomic ]            (none)

      i64 ------> [ atomic ]            (none)
```

`Relation::Atomic` is the arity-zero case: a bundle of no edges, which is exactly
what makes a scalar a leaf. `Part` is the single edge, and that is why `Part`,
not `Relation`, is what the recursion iterates.

**A type has several outgoing relations, and they are distinct.** `Stamp` above
offers two; a build that declared `stamp_from_millis` as a constructor would
offer a third, landing on one `i64` instead of two. These are parallel hyperedges out of the same
type, so the source graph is a multigraph and its edges need labels. `RelationId` is that
label, which is why it belongs in a conversion's identity: `Stamp` through its
fields and `Stamp` through `stamp_from_millis` produce different code from the
same type and the same direction, and must never share a node.

**The graph may contain cycles.** Nothing stops a Rust type from being described
in terms of itself, directly or through a ring of types. The acyclicity the plan
graph enjoys is [enforced during the walk](#plan-value-conversions), by refusing
a conversion that is already open, and is not a property of the source.

As built, the engine has the two relations its element paths need:

```rust
/// How the registry constructs or reads a Rust value.
pub enum Relation {
    /// The whole value converted by one target operation: no parts, no
    /// recursion. An `i64`.
    Atomic,
    /// The record's fields. `Stamp` is `secs` and `nanos`.
    Record(RecordRelation),
}

pub struct RecordRelation {
    /// The record's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub record: String,
    pub parts: Vec<Part>,
}

/// One field of a record relation, or one argument of a constructor relation.
pub struct Part {
    pub name: Option<String>,  // `secs`; `None` for a positional field
    pub index: usize,
    pub ty: TypeRef,           // a source type — the part is another Rust value
}
```

`Part::ty` is a Flat `TypeRef`, a source type: every edge of a relation lands on
another Rust value, which the registry plans recursively with the same algorithm.
That is what keeps the walk the registry's and nobody else's — an edge that
landed on a C struct or a Kotlin class would need a different traversal. The
constructor and projector relations sketched
[below](#the-registry-validates-conversion-roles) are the same shape with
different edges and a source function as the means, which is why adding one
changes nothing under this type.

Flat stores no such links — its references are names, resolved on lookup — so the
graph is not a structure the model holds. The registry builds a type's outgoing
edges on demand and keeps them for the run. `Run::candidates` registers them the
first time a type is planned (the atomic relation for every type; the record one
when the model resolves the name to a struct) and hands the target the list as
`(RelationId, Relation)` pairs. Registering once per type rather than once per
visit is what makes the label stable: a fresh id on every visit would make every
parallel edge unique, and nothing would ever share a node.

The target answers `select` with a **`RelationId`**, an index into that run's
table under the same convention as every other `…Id` here — that is, it names
which outgoing edge the walk takes from this type. The id rather than the value
is what travels, because the id is what the cache key compares.

The important distinction is between the structure Rust declares and the
operation selected for a conversion. `Stamp` always has the same two fields,
but a future constructor-based binding could accept only milliseconds and let
a helper calculate those fields. A relation describes that selected way to
build or read the value, rather than treating the field list as the only option.

The same function can be exported directly, selected as a constructor, or used
to extract another value. Those are binding roles, so relation construction
belongs in the common registry library. Flat supplies the checked source facts
used to validate those roles.

### Flat provides neutral source views

The planned view-based API lets the registry read source through the same
[views everything else uses](02-flat.md#lookup-and-navigation) —
`FunctionView::parameters`, `TypeView::as_record`, `RecordView::fields` — with no
private channel of its own. Current V2 uses borrowed Flat records instead;
the view names in this section describe the intended API, not callable APIs today.

`ParameterView` and `FieldView` provide their exact `TypeView`s. Flat creates the views and keeps their constructors and storage indices private. A record view exposes structural fields only when Flat models them. Reaching a record through `&Stamp` requires an explicit `referent()` step; that inspection does not itself implement a borrow conversion. Details of [storage and model checks](02-flat.md#private-storage-and-model-consistency) and [incremental adoption](../implementation.md#flat-implementation-sequence-and-acceptance) belong to the Flat project.

For `parse_stamp(&str) -> Result<Stamp, Error>`, Flat reports one parameter and a `Result` return type. Whether `Ok` means successful construction is decided by the registry when that function is selected as a constructor.

### The registry validates conversion roles

The following is a proposed API, not the current public structs. It combines
the planned Flat views with checked constructors for three conversion roles.
In particular, this proposed private `RecordRelation` replaces the public-field
structure shown earlier; it is not a second implemented definition.

```rust
impl RecordRelation {
    pub fn new(record: RecordView) -> Self;
}
impl ConstructorRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}
impl ProjectionRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}

pub struct RecordRelation {
    record: RecordView, // Derive subject and fields from the checked record.
}
pub struct ConstructorRelation {
    function: FunctionView,      // Derive arguments from its signature.
    output: ConstructionOutput, // Checked interpretation of its return type.
}
enum ConstructionOutput {
    Value(TypeView), // Constructed type.
    Fallible { constructed: TypeView, error: TypeView },
}
pub struct ProjectionRelation {
    function: FunctionView, // Initially requires exactly one source argument.
    input: TypeView,        // Derived exact argument type: T, &T, or &mut T.
    output: TypeView,       // Derived complete return type, including wrappers.
}
```

All relation fields are private. Read-only accessors expose source types and arguments. A constructor's subject is derived from its result; a projector's subject/access requirements come from its input. Callers cannot pair an arbitrary subject with an unrelated operation. For `stamp_from_millis(i64) -> Stamp`, the constructor has one `i64` argument and produces `Stamp`, regardless of `Stamp`'s fields. For a helper `stamp_parts(&Stamp) -> StampParts`, where `StampParts` is a named record with `secs: i64` and `nanos: i64` fields, projection requires a shared borrow and produces that record.

`RelationError` reports a source function incompatible with the requested role. Validation establishes the role's internal consistency, not that every target supports it. The registry separately checks a selected role against the conversion's expected type, direction and ownership: producing `Stamp` alone does not satisfy `&Stamp` without a supported temporary-and-borrow step. Default fallible construction interprets `Result::Ok` as the value and `Err` as failure; a different treatment requires an explicit conversion role.

### Registering and selecting a relation

**Implemented: `Atomic` and `Record`** — [the two relations above](#what-a-relation-is).
The constructor and projector roles below are described, not built, so a target
has no such candidate to select and fallible construction never reaches a
boundary. Conversion rules are not built either: a request carries policies
recorded for a type and for a position, and the target derives its relation from
those, so a build script cannot yet name a relation directly. The sketch that follows is the
designed shape of the table, not the built one.

```rust
pub enum Relation {
    Record(RecordRelation),
    Construct(ConstructorRelation),
    Project(ProjectionRelation),
    // Later: checked atomic, optional, sequence, variant, Rust-to-Rust
    // conversion, representation-reuse and callable descriptions.
}

struct RelationDefinition {
    operation: Relation, // Checked role; no independently assignable subject field.
}
struct RelationId { /* private table identity and entry ID */ }
struct Selection {
    relation: RelationId, // Registered RelationDefinition containing the checked role.
    policy: PolicyId,     // Target representation settings.
}
struct ConversionRules {
    sites: Map<SiteId, Selection>, // Parameter/result overrides.
    parts: Map<PartId, Selection>, // Field/helper-argument selections.
    defaults: DefaultRules,       // Frontend-defined override precedence.
}
```

The user does not register a relation for each scalar or field-based record.
The registry discovers those edges from Flat. Constructor and projector
relations will additionally need explicit configuration identifying the helper:

A **record relation** is implicit: for any record the source model describes, the
registry registers the relation built from its fields, so `Stamp.fields` exists
without anyone asking for it. A **constructor or projector relation** is
explicit: it names a function, so someone has to say which function, and that is
a frontend declaration. A **scalar** has no parts at all — an `i64` is not built
from anything — so its relation is the atomic one, the whole value converted by a
single operation the target supplies. `Atomic` is implemented, as shown in the
earlier enum; the sketch above focuses on the additional constructor and
projector roles rather than repeating the complete implemented enum.

The relation for `Stamp` is its record: two parts, the fields.
Pinning the constructor instead is a rule recorded with the request, and changes
what the parts are without changing anything else:

```text
relation:  stamp_from_millis  = Relation::Construct(over the checked function view)
parts:     PartId { owner: stamp_from_millis, arm: None, position: Argument(0) }  // millis: i64
rule:      the Stamp type default selects that relation instead of Stamp.fields
```

The registry then converts one `i64`, calls `stamp_from_millis`, and has a
`Stamp` — one child instead of two, the same recursion, and a target that need
not know which happened.

So `select` chooses from the edges leaving that type: the implicit relation,
plus any explicit ones registered for it, and it must choose the one a conversion
rule pinned if the rules pinned any. Pinning does not add or remove edges — they
are all still there, and another position may take a different one — it fixes
which edge is taken from this position. A target that cannot work with a pinned
relation reports that as unsupported — the request is well formed, the capability is missing, and
the affected outputs are skipped with the reason. That is different from a rule
that contradicts the source, such as naming a constructor for a type it does not
construct, which is invalid input and fails the build.

Registry-library registration accepts checked relations and issues opaque `RelationId`s; request import validates their model and table context. Frontends may construct these descriptions without running recursive conversion planning. The earlier label `Stamp.fields` denotes a registered `Relation::Record` backed by the `Stamp` record view.

The registry follows `Selection.relation` to the checked operation, obtains its fields or helper arguments, and recursively plans their conversions. Projection calls its helper once and processes the saved result through the conversion rules. The adapter describes foreign representations; the registry assembles the source-side instructions executed by the wrapper.

Selection precedes child traversal: an atomic opaque representation does not inspect unused private fields. Helper arguments need not resemble fields. Child types retain wrappers, references and lifetimes; cloning needs an explicit operation.

Future callback planning must reverse direction for callback arguments. Rust
receives the callable as input, but later supplies values to the foreign
callback as output. Those values make the opposite
[crossing](03-requests.md#finding-an-existing-conversion-plan). The direction
follows from the callback's role, not from the user specifying an independent
direction for each argument. V2 does not implement callback conversion yet.

### Responsibility boundary with Flat

The [relation API](#describing-source-construction-and-decomposition) consumes
`FunctionView` and `RecordView`. The registry assigns conversion roles and
validates them against a requested direction and exact `TypeView`.

| Flat provides | Registry adds |
| --- | --- |
| Function parameters and complete return type | Whether the function constructs a value or projects one from an input. |
| Record shape and typed fields | Recursive field conversions and value construction/decomposition. |
| `Result` child types | Whether a selected constructor treats `Ok` as construction success and routes `Err` as failure. |
| Exact reference/wrapper structure and source access facts | Temporary lifetimes, borrow use and ownership in the generated conversion. |
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction, selected relation and target policy. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat has no
conversion-selection policy, target representation, recursive binding planner,
or dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.

## Describing target values and operations

The registry knows how source values are composed. It also needs a description of the values used on the target side and the operations that access them.

The **public surface** is the API foreign users see: C types/functions or Kotlin classes/methods. The **wire representation** is the set of values passed through the native calling interface, or **ABI** (application binary interface). A Kotlin `Stamp` object can have two integer arguments as its wire representation. A **carrier** is a value holding conversion data, either on that boundary or temporarily inside generated Rust code.

### Individual target operations

A **primitive** is one typed operation supplied by the target, such as converting a scalar, reading a JVM property, allocating a handle, or signaling an error. (The word means an indivisible *operation* here, not a primitive type; a scalar conversion is one of the things a primitive can do.) The registry schedules these operations within the complete conversion.

An operation may need a generated helper or type declaration. Each such unit is
an **artifact**. Current operation descriptions carry whole `Artifact` values,
identified by name; the registry deduplicates them. Several artifacts can share
one output file. `ArtifactId` in the design sketches below represents a proposed
table-based reference, not a current API type.

The following signature/contract types sketch an extended API. Current
operations have `operands`, one optional `result`, `failure`, `dependencies` and
`implementation`. Each operand also has a role: an ordinary value, an error,
or a named runtime context. Current carrier descriptions use inline
`WireType { ty, abi }` values, not `WireTypeId`; source types use `TypeRef`, not
the proposed `TypeView`. These differences matter when implementing an adapter.

```rust
struct PrimitiveSpec<Payload> {
    signature: PrimitiveSignature, // Operand/result types, including runtime environment values.
    failure: PrimitiveFailure,     // Possible failure and the type of error it produces.
    validity: ValidityContract,     // Inputs/scopes on which the result's validity depends.
    resources: ResourceContract,   // Acquire/release/transfer effects, or explicitly none.
    dependencies: Vec<ArtifactId>, // Generated helpers/types needed by this operation.
    implementation: Payload,       // Target-specific description of the operation to render.
}
```

The following design sketch describes a getter using those fields. It explains
the intended full contract; current `PrimitiveSpec` uses a smaller structure
without `validity` or `resources`. The getter operation itself is implemented.

```rust
PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![
            // The JNI environment: an operand, not an ambient variable.
            OperandSpec { ty: OperationType::Carrier(jni_environment), access: Access::Exclusive },
            // The object whose property is read.
            OperandSpec { ty: OperationType::Carrier(stamp_object), access: Access::Shared },
        ],
        results: vec![OperationType::Carrier(jni_long)],
    },
    // A JVM call can fail, and the error is the jni crate's.
    failure: PrimitiveFailure::Fallible {
        error: OperationType::Carrier(jni_error),
        category: FailureCategory::Runtime,
    },
    // The integer that comes back owes nothing to the object it came from.
    validity: ValidityContract { results: vec![ResultValidity::Independent] },
    resources: ResourceContract::none(),
    dependencies: vec![],
    implementation: JniOperation::CallLongGetter {
        name: "getSecs".into(),
        descriptor: "()J".into(),   // JVM descriptor: no arguments, returns a long
    },
}
```

Applied to an environment the boundary has named `env` and an object it has
named `stamp`, that one description renders exactly this much Rust:

```rust
env.call_method(&stamp, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

This expression returns either a JNI integer or a JNI error. `call_method`
invokes the getter, and `.j()` extracts the long result. The adapter renders
that single operation. The common writer supplies the surrounding `let`, branches
on success or failure, and follows the function's error route. For C, reading
the same field is simply `stamp.secs`, which cannot report an error and needs
no JNI environment. The registry combines either kind of read with the same
source-record construction algorithm.

`implementation` is `Operation<Payload>`: either a **standard** operation the
registry itself renders — the identity conversion, an aggregate member read — or
the adapter's own `Payload`. Reading a C aggregate member is the first, which is
why the C adapter ships no renderer at all and its payload type has no values.
`Payload` is rendering data whose type and interpretation are defined by the language implementation, such as a JVM property descriptor or a native operation description. The adapter supplies payload values; the registry stores those values in the plans and retains the required values in the completed `Generation`. Language-provided rendering code reads the retained payloads. Policy requests a representation; payload records the chosen implementation details after planning. Language implementations can use separate payload types for primitives, representations and foreign declarations.

#### The operation specification and its uses

`PrimitiveSpec` describes an operation during generation. For example, one
specification can describe reading the `secs` property from a JVM `Stamp` object.
The specification contains no particular caller's object, temporary name or
exported function. The registry can apply that operation in several conversion
plans, with different operands at each use.

The signature describes one application. An **operand** is a value supplied to
the operation; a **result** is a value available after successful execution.
Environment values are operands too, so a property read cannot silently depend
on a variable named `env` in its caller.

```rust
struct PrimitiveSignature {
    operands: Vec<OperandSpec>, // Ordered input positions, including runtime context.
    results: Vec<OperationType>, // Ordered values produced on success; empty for unit.
}

struct OperandSpec {
    ty: OperationType, // Exact type expected at this input position.
    access: Access,    // Whether this application consumes or borrows the input.
}

enum OperationType {
    Source(TypeView),     // An exact Rust type in its retained Flat snapshot.
    Carrier(WireTypeId), // A registered target/runtime type, described below.
}

enum Access {
    Owned,     // The operation may consume/move the value.
    Shared,    // It may read through a shared borrow.
    Exclusive, // It may modify through an exclusive borrow.
}
```

`Access` says what an operation is allowed to do with a value. Read on an operand
it is a demand — this operation will move the value, or only borrow it. Read on a
finished conversion's result, later in this chapter, it is a permission — this is
what a caller may do with what the conversion produced. One vocabulary, two ends
of the same value.

`WireTypeId` identifies a descriptor for one carrier type on the target-facing
side of a conversion: a JNI integer, an object reference, the JNI environment, or
a Rust-only intermediate that never leaves the wrapper. Not every carrier is an
ABI type, so the descriptor states whether this one may appear in an extern
signature. An environment
wrapper used inside generated Rust need not itself be an ABI argument type.
`Access` describes the operation's use of its operand and must agree with the
operand's exact Rust type. It does not authorize cloning or replacing a move
with a borrow. The same access vocabulary is used by [completed conversion
plans](#the-conversion-plans-the-registry-builds).

#### Failure and result validity

A primitive's failure describes what can go wrong while performing that
operation. The registry needs this information to compose fallible operations
and stop the success path before any unavailable result is used.

```rust
enum PrimitiveFailure {
    Infallible,
    Fallible {
        error: OperationType, // Typed value supplied on the failure path.
        category: FailureCategory, // Selects the containing boundary's error route.
    },
}

enum FailureCategory {
    Domain,  // Error reported by an explicitly selected source operation.
    Binding, // Invalid foreign value or failed representation conversion.
    Runtime, // Failure from target runtime operations, such as JNI access.
}
```

One specification has one error type; an operation with several error variants
uses a typed error enum. The common writer renders a fallible application with
separate success and error paths. The target's operation renderer supplies the
local operation that yields that result. The registry owns the branch, later
conversion calls, cleanup and final return or error-handler invocation. For JNI,
the error contract must also state how pending JVM exceptions are represented
and which recovery operations are legal. Unsupported exception handling skips
the affected binding; the planner cannot assume it is safe to continue calling
JNI methods after a failed read.

**Not implemented.** Validity is absent from a described operation for the same
reason: every result the engine plans is a value copied out. A borrowed or
scope-bound result is what needs the contract below.

Result validity answers a different question: how long can a successfully
produced value be used? A copied integer is independent of the object it came
from. A reference into a buffer remains tied to that buffer. A JNI local object
reference remains tied to its JNI reference scope.

```rust
struct ValidityContract {
    results: Vec<ResultValidity>, // One entry for each successful result position.
}

enum ResultValidity {
    Independent, // No lifetime dependency on an operand or ambient runtime scope.
    DependsOn {
        operands: Vec<OperandIndex>, // Inputs whose storage must remain valid.
        scopes: Vec<ScopeRequirement>, // Runtime scopes that must remain open.
    },
}
```

`OperandIndex` refers to a position in this primitive's signature, not an
exported function's parameter list. `ScopeRequirement` describes a required
runtime scope and the context operand through which the operation accesses it,
such as the JNI environment and its active local-reference frame. During
composition, the registry resolves that requirement to a concrete scope in the
wrapper. The checked specification constructor validates operand indices and
result counts. The registry rejects a use if the required scope is unavailable
or a result would escape its dependencies. `Independent` does not imply that a
value is copyable or that it has no destructor.

#### Runtime resources and generated dependencies

**Not implemented.** A described operation carries its failure and its generated
dependencies, and nothing about resources. Every operation the engine plans
acquires nothing, which is why the omission is safe today and why the first
handle-bearing operation cannot be written without this.

`ResourceContract` describes obligations introduced or discharged by an
operation, such as releasing a retained handle. It is separate from validity:
a value can have a long enough lifetime and still leak if nobody releases it.
The initial scalar/owned-record implementation has no resource-contract field;
it relies on ordinary Rust destruction for its owned locals and performs no
additional handle acquisition requiring explicit cleanup.

Resource-bearing operations require a later contract implementation with these
facts before the registry can support them:

| Fact in the resource contract | How the registry uses it |
| --- | --- |
| Acquisition on success, the result carrying the resource, and a release primitive with explicit operands | Register the cleanup obligation when acquisition succeeds. |
| Consumption or release of an input resource | Ensure the resource is live before the operation and cannot be consumed twice. |
| Transfer to the source API or foreign caller | Discharge local cleanup only on the path where the recipient actually takes ownership. |
| Resource state on each failure path | Preserve existing obligations and release partial acquisitions before routing the error. |
| Scope and cleanup order | Keep dependencies alive until release and finish scopes in a valid order. |

The adapter supplies runtime acquire/release operations. The registry tracks
those effects and schedules calls on success and failure paths. A primitive may
clean up a temporary allocation entirely inside its own implementation, provided
no ownership obligation escapes either execution path. Handles, callbacks and escaping
allocations remain unsupported until their effects can be represented and
validated — that is, until the table above can be filled in for them and the
registry can check what it says.

By comparison, `dependencies: Vec<ArtifactId>` concerns the generated program:
it retains helper functions and type declarations required to compile the
operation. It does not arrange runtime cleanup. A dependency on an external
runtime crate belongs in the target's build requirements; a generated helper
wrapping that runtime call is an artifact.

#### Registry registration and application

The adapter returns primitive specifications with its representation or boundary
description. The registry validates their types, local references, failure and
validity contracts, registers the descriptions, and issues opaque `PrimitiveId`
references. The specification fields above show the stored information; the
implementation should keep validated records private and expose checked
construction and read-only inspection.

A **primitive application** is an instruction in the registry's conversion or
function body. It names a registered primitive and the already available values
that supply its operands. The registry checks operand types/access, allocates
result identities, and records failure paths. These identities denote runtime
values in a plan; they are neither runtime values nor generated variable names.
Only the common writer chooses the final Rust names.

Contract validation can check the structure and composition of the description.
Target rendering code is still responsible for implementing the declared
operation correctly. Generated Rust compilation and focused runtime tests must
verify that responsibility, including error paths; metadata cannot prove that
arbitrary renderer code obeys its description.

The [C member reads][struct_values_c] and [JNI property getters][struct_values_jni]
of the record path are worked examples of these specifications, down to the
fragment each one renders.

## Target representations and composition

A target representation tells the registry which values carry converted data and
how to access them using the operations above.

**Implemented: two layouts, two protocols, one direction.** The engine has
`Layout::Scalar` and `Layout::Aggregate`, and `Protocol::Terminal` and
`Protocol::Product` — convert the whole value, or project one part per part of
the selected relation, reading a record on the way into Rust. A record leaving
Rust needs a target construction operation and is a reported skip until there is
one. C reports `unsupported.record.out_of_rust` from the registry; JNI refuses
earlier with `unsupported.jni.object_output`. Slots, nested member layouts,
guards, and the optional, sequence, choice and
callable protocols below are described rather than built.

A **layout** describes the values contained in a representation. A **protocol**
describes the operations used to read or construct them. The registry supplies
the recursion that converts the corresponding Rust fields. The following enum
is the broader design: `Empty`, `Slots`, nested layouts and id-based references
are not implemented. Current protocols carry operation specifications inline;
the registry assigns operation ids when it composes instructions.

```rust
enum Layout {
    Empty,                 // No carried values, as for a unit result.
    Scalar(WireTypeId),     // One scalar/reference/carrier value.
    Slots(Vec<SlotSpec>),   // Several independent ordered values.
    Aggregate {
        ty: WireTypeId,            // Type of the one containing struct/object carrier.
        members: Vec<MemberLayout>,// Named/indexed members and their child layouts.
    },
}

struct SlotSpec {
    id: SlotId,          // This value's identity within the layout.
    wire: WireTypeId,    // Target or intermediate type of the value.
    role: SlotRole,      // Payload, presence flag, variant selector, etc.
    active_when: GuardId,// Condition under which its payload can be converted.
}

struct ReprSpec<Payload> {
    layout: Layout,      // Values that carry this representation.
    protocol: Protocol,  // Operations used to access/construct those values.
    payload: Payload,    // Chosen target metadata needed for rendering.
}

enum Protocol {
    Terminal { codec: PrimitiveId }, // Converts the whole value in one operation, with no parts.
    Product(ProductOps),   // Project members and construct a target product.
    Optional(OptionalOps), // Detect/extract/inject presence or absence.
    Sequence(SequenceOps), // Read or append target sequence elements.
    Choice(ChoiceOps),     // Inspect/write a tag and its active payload.
    Callable(CallableOps), // Capture/invoke a target callable.
}
```

A carrier being expressible as a Rust type does not make it safe in an extern
signature; the `abi` flag states whether the adapter permits that use.
`MemberLayout` and `ReprSpec.payload` above belong to the design sketch. In
current code, the aggregate layout holds its `WireType`, and the C `repr(C)`
declaration is a Rust artifact attached to the public `SurfaceSpec`. A whole
function's calling convention belongs in `AbiSpec` at
[the boundary](05-boundary.md#assembling-an-exported-function).

A **slot** is one value in a multi-value representation — for a `Stamp` passed to JNI as two separate arguments rather than an object, the layout is two slots, and a function taking two such records has four native arguments in all. `SlotRole` states a slot's meaning, independent of its generated name. `GuardId` refers to an activation condition on a slot — “always,” “presence is true,” or “variant tag selects this arm” — and is unrelated to the guard items of [capture](01-source.md). Enclosing conditions also apply. Inactive slots can require valid wire defaults even though their source payload must not be read or constructed. When one layout is used for two function arguments, its slot identities are qualified by each use so their ABI positions remain separate.

`ProductOps` in the design would describe both member reads and a target
construction operation over converted children. The implemented form is
`Protocol::Product { projections }`: one read per part, in part order, for
input into Rust. It has no target-construction operation yet. A future C output
could use a struct literal, while a future separate-arguments JNI form would
map children to argument slots. The implemented JVM-object input uses getter
operations. These are different target operations around the same registry
algorithm for converting the selected parts.

For sequences, variants and callbacks, adapters supply runtime operations; the registry supplies loops, branches and child calls. C aggregates and JNI slots/object operations reuse the same relation.

A layout stays nested for as long as nesting is meaningful: an aggregate whose member is itself an aggregate is described that way, and only a place that requires a flat list of values — a native signature, where each slot becomes one ABI argument — flattens it, at that point, in that use. Keeping the nesting until then is what lets the same record representation be an argument in one function and a member of another.

### Optional values

**Not implemented.** Nothing carries an optional value yet: `Layout::Slots`,
`SlotRole`, `GuardId` and the encodings below are names. The section says what
the slot has to hold.

An optional value needs both a representation of its child and a way to distinguish absence. Different targets can encode that distinction differently:

```rust
enum AbsenceEncoding {
    Presence {
        flag: SlotId,         // Separate value indicating whether the child is present.
        inactive: DefaultsId, // Valid wire defaults for the absent child's slots.
    },
    Nullable {
        test: PrimitiveId,    // Test for absence in a nullable carrier.
        extract: PrimitiveId, // Obtain the present child's carrier.
        inject: PrimitiveId,  // Wrap a converted child as present.
    },
    Niche {
        domain: DomainId,     // Valid child values and a reserved absence encoding.
        test: PrimitiveId,    // Test for the reserved encoding.
        extract: PrimitiveId, // Recover the present child's carrier.
        inject: PrimitiveId,  // Encode a child without colliding with absence.
    },
}

struct OptionalOps {
    encoding: AbsenceEncoding, // The selected absence/presence convention.
    payload: LayoutId,         // Child representation when present.
    absent: PrimitiveId,       // Produce the complete representation of absence.
}
```

A **niche** is a reserved representation that cannot be a valid present child, such as zero for a handle whose valid values exclude zero. `DomainId` describes those validity facts. `DefaultsId` describes valid wire defaults, not fabricated Rust source values. `absent` builds the complete absent representation; `inactive` supplies the unused child slots for the separate-flag convention.

The registry branches on presence and invokes the child conversion only on the present path. It validates active inputs and supplies required inactive defaults. Nested optionals must preserve distinct states such as `None` and `Some(None)`; if the selected encoding cannot do that, the combination is unsupported.

## How the registry asks a target for decisions

The target interface has four planning operations. Each answers a local question
with a description; the registry decides when to ask it. The sketch below uses
the design's descriptor names. Current `represent` receives `ChildValue`
entries containing a part and layout, not full validity/resource contracts.
The same trait also has `render_operation`, used later during Rust emission.

```rust
trait Target {
    type Policy;  // Configuration choices recorded by the frontend.
    type Payload; // Owned operation/rendering descriptions.

    fn select(
        &self,
        query: SelectionQuery<'_, Self::Policy>, // Source value and applicable choices.
    ) -> TargetSupport<RelationId>; // One of the relations the query offered.

    fn represent(
        &self,
        shape: ResolvedShape<'_>, // Selected source operation and its direct children.
        children: &[ValueDescriptor<Self::Payload>], // Completed child conversion descriptions.
        policy: &Self::Policy,    // Effective choices for this value.
    ) -> TargetSupport<ReprSpec<Self::Payload>>;

    fn boundary(
        &self,
        site: &SiteDescriptor, // Exported call's signature and boundary roles.
        values: &ResolvedValues<Self::Payload>, // Its resolved input/output values.
        policy: &Self::Policy, // Calling and error-delivery choices.
    ) -> TargetSupport<BoundarySpec<Self::Payload>>;

    fn surface(
        &self,
        request: &SurfaceRequest<Self::Policy>, // Public name/placement, policy and promises.
        values: &ResolvedValues<Self::Payload>, // Values needed to describe that public API.
    ) -> TargetSupport<SurfaceSpec<Self::Payload>>;
}
```

Every method answers with `TargetSupport<Answer>`, which is one of three things:
a ready description, a specific unsupported reason, or a fatal planning error
([defined with the other support [outcomes](06-retain.md#retain-supported-output)](06-retain.md#unsupported-requests-and-public-api-dependencies)).
Two of the answers are specified in later chapters, because they are about later
stages: [`BoundarySpec`](05-boundary.md#assembling-an-exported-function) describes
where native arguments and results go, and
[`SurfaceSpec`](06-retain.md#dependencies-of-public-declarations) describes a
public foreign declaration and what it requires.

The method inputs and results serve different stages:

| Method | Information available | Target's answer | Registry's next job |
| --- | --- | --- | --- |
| `select` | Exact source type/direction, local source facts and applicable conversion rules and policy | The relation chosen for *this* value, and nothing else | Inspect that relation's parts and recursively resolve their conversions. |
| `represent` | `ResolvedShape`: source operation with model-derived child types; `ValueDescriptor`s: completed child layouts and contracts | Representation layout and target operations | Compose the complete value conversion. |
| `boundary` | `SiteDescriptor`: call signature/roles; `ResolvedValues`: its completed value descriptions | Argument placement, result delivery and error actions | Assemble and validate the complete native wrapper. |
| `surface` | `SurfaceRequest`: requested name, placement, policy and promises; required value descriptions | Public declaration description and requirements | Check dependencies before deciding whether to emit it. |

A selection speaks for one value. It cannot declare a choice for a child,
because a child is planned by a recursion that asks the target again, and two
ways to say the same thing would need a precedence rule; a choice for a child is
a conversion rule recorded at the child's position instead.

`select` is on this interface, even though a relation is a source-side
description, because the choice among the available relations depends on how the
target intends to carry the value: a representation that hands out an opaque
handle wants the atomic relation, not the fields. The target chooses; it does not
invent. It picks from the relations registered for that type, and where the
configuration pinned one, it either honours that choice or reports why it cannot.
Walking whatever it picked remains the registry's work.

These views are read-only. The adapter can inspect direct child descriptors but cannot invoke the registry's recursive compiler or modify the registry's plan tables. Relations and representations that recur — a record read through its members, a scalar carried unchanged — should be available to an adapter as ready-made descriptions it names rather than builds, so that a new target's first version is a handful of choices rather than a library.

Source-conversion dependencies come from the selected relation's children, and
target operations list generated helpers. Current public dependencies are
`DeclarationId`s in `SurfaceSpec.requires`. The more general design will need
an explicit request mechanism if a target requires additional conversions;
that mechanism is not implemented. Rendering must not discover new conversions.

Descriptions returned by a target can contain new primitive, layout, helper, or policy definitions with references local to that description. The registry validates and registers the definitions and assigns its own table IDs. Existing descriptors can reference IDs the registry already supplied. The target does not allocate entries in registry-owned tables itself.

The language-provided final rendering interface reads immutable plans and retained payloads. The common emission machinery supplies allocated operand names and the necessary rendering context; the rendering interface exposes no planning entry point. The original builder is translated before generation; subsequent decisions use requests and policies.

## The conversion plans the registry builds

The extended conversion contract below brings together the proposed validity,
resource and model-view APIs. Current `ValuePlan` instead stores `id`, `crossing`,
`relation`, `repr`, `children`, a `NodeBody`, and failure categories. It does
not yet contain `ValueContract`, `ResolvedRelation` or `ConversionBodyId`.

```rust
struct ValuePlan<Payload> {
    id: NodeId,                    // Reference used by callers of this conversion.
    crossing: Crossing,            // Exact source type and direction being converted.
    relation: ResolvedRelation,    // Source operation with its validated child node IDs.
    representation: ReprSpec<Payload>, // Target layout and access/construction operations.
    body: ConversionBodyId,        // Registry-owned structured instructions.
    contract: ValueContract,       // Result type, permitted use, validity and possible failures.
    dependencies: Vec<NodeId>,     // Derived index of the conversions this plan uses.
}

struct ValueContract {
    produced: ValueType, // Output endpoint: a source Rust value or target/intermediate layout.
    access: Access,     // Permitted use, checked against Rust types and selected helper signatures.
    validity: Validity, // Concrete lifetime/provenance guarantees of the produced value.
    failures: FailureSet, // Typed errors possible on the conversion's active execution paths.
}
```

`ResolvedRelation` is the chosen relation after its source references and child conversions are resolved. `ValueType` says what a finished conversion produces: either a source Rust type, or a layout of carriers holding zero, one or several values. It differs from the `OperationType` above in exactly that arity — one operand or result is always a single value, whereas a conversion's product can be a group of slots. `Validity` composes the individual primitives' validity rules: for example, a produced reference remains tied to a particular temporary. `FailureSet` collects possible error categories/types; the function boundary decides their eventual handling.

`dependencies` is the node's outgoing edge list, kept as an index: it is derived
from the resolved relation and the body, never authored, and it is what a later
stage follows to collect everything a retained conversion needs.

`ConversionBodyId` points to structured instructions for locals, field access, variant matching, source construction/calls, primitive applications, conditions, and later loops or callback invocation. The common writer renders these instructions as Rust. It allocates temporary names centrally from identities.

The instructions are three, and every value in them is an identity rather than a
name:

```rust
enum Instr {
    // Apply a registered operation to values this body already has, binding
    // its result when it produces one.
    Apply { primitive: PrimitiveId, operands: Vec<Operand>, result: Option<ValueId> },
    // Build a source record from converted parts, in field order.
    Construct { record: String, parts: Vec<ValueId>, result: ValueId },
    // Call the source function, once.
    Call { function: String, args: Vec<ValueId>, result: Option<ValueId> },
}

enum Operand {
    Value(ValueId),    // A value this body has.
    Context(String),   // A runtime scope the boundary supplies under this name.
}
```

A conversion's body is a **template**: one value identity is its carrier — the
value handed in when the conversion is used — and one is its result. Using a
conversion inlines its instructions under the caller's identities, so a
conversion belongs to no function and is still written down once. An identity
conversion is the degenerate template: no instructions, and the result *is* the
carrier, which is why a scalar child renders nothing between a member read and
the construction that uses it.

An operation that needs a runtime scope names it, and the boundary says which
native parameter supplies it; nothing in a rendered fragment can reach for a
variable its caller happens to have.

The registry generates all child calls and source traversal. A projector binds its intermediate result once. Optional/variant branches convert only active children. The dependency list is derived from these instructions and the resolved relation; it is a convenience index maintained by the registry.

Access follows the exact source type and operation. For example, generating `&Stamp` input may require constructing an owned temporary and borrowing it for the duration of the source call. Supporting the two scalar fields alone does not implement that borrow. The conversion contract must preserve the temporary's validity through its uses.

For initial scalar/record support, ordinary owned Rust temporaries can rely on Rust destruction at scope exit. Handles and callbacks need implemented acquisition, transfer and cleanup semantics before they are supported. As those features are added, the registry will schedule resource scopes and cleanup on success and failure paths; adapters provide the actual retain/free/runtime operations.

## What is not settled here

The scalar and owned-record cases above are implemented, and
[the first increment](../implementation.md#the-first-increment-as-built) records
what building them settled — the instruction set, the standard-operation
variant, the reach of a selection — and what it left open. The largest open
items belong to this chapter: the composition protocols for sequences, variants
and callables, the construction half of a product, optional values, and the
validity and resource contracts that `PrimitiveSpec` does not yet carry because
nothing in the increment produces a borrowed or resource-bearing value.

## Elements at this stage

- [Function taking an owned record][fn_values] · [C][fn_values_c] · [Kotlin/JNI][fn_values_jni]
- [Record with scalar fields][struct_values] · [C][struct_values_c] · [Kotlin/JNI][struct_values_jni]

[fn_values]: ../examples/fn/04-values.md
[fn_values_c]: ../examples/fn/04-values.c.md
[fn_values_jni]: ../examples/fn/04-values.jni.md
[struct_values]: ../examples/struct/04-values.md
[struct_values_c]: ../examples/struct/04-values.c.md
[struct_values_jni]: ../examples/struct/04-values.jni.md
