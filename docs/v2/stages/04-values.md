<!-- spec: {"kind": "stage", "stage": "04-values"} -->

# Plan value conversions

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

This is the stage that does the real work, and the one the whole architecture is
arranged around.

Consider what has to happen for a foreign caller to call `stamp_sum(Stamp) ->
i64`. The Rust function needs an owned `Stamp`. No foreign caller has one: a C
caller has a struct of two integers, a Kotlin caller has a JVM object. So the
generated wrapper has to obtain two field values from whatever the caller
actually passed, build `Stamp { secs, nanos }` out of them, call the function,
and turn the returned `i64` into something the caller can receive. Each of those
value-shaped problems is a **conversion**, and planning one is what this stage
does.

Three separate questions have to be answered for every conversion, and keeping
them apart is what lets one algorithm serve both languages:

- **How is the Rust value built or read?** For `Stamp`, from its two fields — or,
  if the configuration said so, by calling `stamp_from_millis`. This answer is a
  **relation**. Relations are described in source terms only, and the registry
  alone knows how to walk one; but *which* relation applies can still depend on
  the target, because a target that carries `Stamp` as an opaque handle needs no
  fields at all. So the target picks from the relations available, and must
  honour a relation the configuration pinned or say why it cannot.
- **What carries the value on the other side, and how is it accessed?** A
  by-value C struct whose members are read with ordinary field reads, or a JVM
  object whose properties are read by calling `getSecs()` and `getNanos()`
  through JNI. This answer is a **representation**, and only the target can give
  it.
- **How are the pieces put together?** Read each part, convert it, construct the
  Rust value, in that order, stopping if a step fails. This is the registry's
  job, and it is identical in both languages.

Put together, planning one conversion is this recursion — the registry's own
loop, with the two target questions marked:

```text
plan(type, direction, position):
    policy   = effective policy for this position     # site override, else part
                                                      # rule, else type default
    relation = target.select(type, direction, applicable rules, policy)
                                                      # cheap: no recursion yet
    if a node exists for (type, direction, relation, policy):
        return it                                     # the cache key is complete
                                                      # only once the relation is known
    mark (type, direction, relation, policy) as being resolved
                                                      # meeting this mark again is a cycle

    parts    = the source model's parts of that relation
                   # the fields of a record, the arguments of a constructor;
                   # none at all for a scalar
    children = [ plan(part.type, direction, that part's position)
                 for part in parts ]
    if any child is unsupported:
        this conversion is unsupported, and so is everything that needed it

    repr = target.represent(relation, children, policy)
               # which carriers hold the value, and the operations that access them
    body = compose(relation, children, repr)
               # obtain each part, convert it, construct the Rust value — or the
               # reverse, when the direction is out of Rust

    record the node and return it
```

Two details in that sketch matter more than they look. Selection happens before
the cache is consulted, because the relation is part of what identifies a node —
the same type converted through its fields and through a constructor are two
different conversions. And the recursion is parameterized by *position*, not just
by type: a `SiteId` (parameter 0 of this exported function) or a `PartId` (the
`secs` field of this relation) is what an override is recorded against, so the
position is what turns the recorded rules into this conversion's effective
policy. Positions are how overrides reach a nested child; the resulting node is
still shared by identity, so two positions that resolve to the same four-part key
get the same node.

For `Stamp` the recursion is one level deep: two `i64` children that need no work
of their own. A record with a record field simply makes `plan` call itself again,
and neither adapter learns anything about the nesting — which is the point.

The unit the target supplies for the second question is a **primitive**: one
typed operation, such as "read the `secs` member of a `StampC`" or "call the
`getSecs()` getter on this object with this environment". A primitive describes
an operation, not a use of it — it names no variable and belongs to no exported
function — so the same description can be applied wherever that operation is
needed. Along with the operation, the target states what the operation needs,
what it produces, whether it can fail, how long its result stays valid, and any
generated helper it depends on. Those facts are what let the registry compose
operations safely instead of pasting text together.

The result of planning one conversion is a **node**: a reusable plan holding the
exact source type, the relation, the child conversions, the representation, the
instructions and a contract stating what the node produces, how it may be used,
how long it remains valid and how it can fail. A node belongs to no function, so
two exported functions taking an owned `Stamp` the same way share one. Identity
is the exact type, the direction, the relation and the effective policy: `Stamp`
and `&Stamp` are different nodes, and so are the same `Stamp` under the C and the
Kotlin configuration.

The registry never asks a target to convert a record; it asks the target to
represent one, and walks the record itself. That is the division the rest of this
chapter specifies, and the reason a nested record, an optional field or a third
field costs the target nothing new: the same recursion visits one more child, and
asks the same local questions about it.

A conversion is planned all the way or not at all. If any child conversion is
unsupported — a field of a type nothing can carry yet — the node is unsupported,
and every request that needed it is skipped with that reason. No half-built node
is ever published: while planning is in progress the registry marks the
conversion as being resolved, which is how it detects a cycle, but that mark is
bookkeeping, not a plan, and it is replaced by a node or by an unsupported
outcome. No later stage sees a conversion that half exists.

## Describing source construction and decomposition

For `normalize(stamp: Stamp) -> Stamp`, the wrapper converts foreign input into a Rust `Stamp`, calls `normalize`, and converts its result for foreign code. Here, **value** means a runtime argument, result or field. Input **construction** can assemble `Stamp { secs, nanos }` from converted fields or call `stamp_from_millis` with one converted argument. Output **decomposition** can read fields or call an accessor. A constructor does not automatically provide an inverse accessor.

A **relation** is the registry's description of how to construct or read a Rust value for a conversion. The same function can be exported directly, selected as a constructor, or used to extract another value. Those are binding roles, so relation construction belongs in the common registry library. Flat supplies the checked source facts used to validate those roles.

### Flat provides neutral source views

The registry reads the source through the same
[views everything else uses](02-flat.md#lookup-and-navigation) —
`FunctionView::parameters`, `TypeView::as_record`, `RecordView::fields` — with no
private channel of its own.

`ParameterView` and `FieldView` provide their exact `TypeView`s. Flat creates the views and keeps their constructors and storage indices private. A record view exposes structural fields only when Flat models them. Reaching a record through `&Stamp` requires an explicit `referent()` step; that inspection does not itself implement a borrow conversion. Details of [storage and model checks](02-flat.md#private-storage-and-model-consistency) and [incremental adoption](../implementation.md#flat-implementation-sequence-and-acceptance) belong to the Flat project.

For `parse_stamp(&str) -> Result<Stamp, Error>`, Flat reports one parameter and a `Result` return type. Whether `Ok` means successful construction is decided by the registry when that function is selected as a constructor.

### The registry validates conversion roles

Registry-library constructors accept checked Flat views. The following prototypes and private internals show the first three roles:

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

Nothing in either build script registers a relation, yet the fixture needs three
of them, so it is worth being explicit about where each comes from.

A **record relation** is implicit: for any record the source model describes, the
registry registers the relation built from its fields, so `Stamp.fields` exists
without anyone asking for it. A **constructor or projector relation** is
explicit: it names a function, so someone has to say which function, and that is
a frontend declaration. A **scalar** has no parts at all — an `i64` is not built
from anything — so its relation is the atomic one, the whole value converted by a
single operation the target supplies. The enum above lists atomic among the roles
still to be added; the first increment needs it before anything else, because the
`i64` result and the `i64` fields of the fixture are exactly that case.

For the fixture, the relation for `Stamp` is its record: two parts, the fields.
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

So `select` chooses from: the implicit relation for that type, plus any explicit
ones registered for it, and it must choose the one a conversion rule pinned if
the rules pinned any. A target that cannot work with a pinned relation reports
that as unsupported — the request is well formed, the capability is missing, and
the affected outputs are skipped with the reason. That is different from a rule
that contradicts the source, such as naming a constructor for a type it does not
construct, which is invalid input and fails the build.

Registry-library registration accepts checked relations and issues opaque `RelationId`s; request import validates their model and table context. Frontends may construct these descriptions without running recursive conversion planning. The earlier label `Stamp.fields` denotes a registered `Relation::Record` backed by the `Stamp` record view.

The registry follows `Selection.relation` to the checked operation, obtains its fields or helper arguments, and recursively plans their conversions. Projection calls its helper once and processes the saved result through the conversion rules. The adapter describes foreign representations; the registry assembles the source-side instructions executed by the wrapper.

Selection precedes child traversal: an atomic opaque representation does not inspect unused private fields. Helper arguments need not resemble fields. Child types retain wrappers, references and lifetimes; cloning needs an explicit operation. Callback arguments reverse direction. Future roles follow the same checked-construction pattern and produce unsupported outcomes until implemented.

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

An operation may need a generated helper or type declaration. Each such generated unit is an **artifact**, referenced by an `ArtifactId` in the operation's dependency list. Several artifacts can share one output file.

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

Filled in, that is concrete. This is the JNI adapter's description of reading the
`secs` property of a `Stamp` object — one operation, complete:

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

Applied to an environment the registry has named `env` and an object it has named
`arg0`, that one description renders exactly this much Rust:

```rust
env.call_method(&arg0, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

An expression of type `Result<jlong, jni::errors::Error>`, and nothing more: no
`let`, no `match` on that result, no return from the enclosing function. Those
belong to the wrapper the registry composes. The C adapter's answer for the same
field is `arg0.secs`, infallible, with no environment operand — a different
`PrimitiveSpec` with the same purpose, which is why the composition around it can
be identical.

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

`ResourceContract` describes obligations introduced or discharged by an
operation, such as releasing a retained handle. It is separate from validity:
a value can have a long enough lifetime and still leak if nobody releases it.
The initial scalar/owned-record implementation uses an explicit no-extra-resource
contract and ordinary Rust destruction for owned locals.

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
no ownership obligation escapes either outcome. Handles, callbacks and escaping
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

A **layout** describes the values contained in a representation. A **protocol** describes the target operations through which the registry reads or constructs those values. Neither specifies how to recursively convert the corresponding Rust fields; the registry supplies that algorithm.

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

A Rust tuple or JVM wrapper object must not appear in an extern signature merely because it can be described as a Rust type — that is what the descriptor's ABI flag prevents. `MemberLayout` names an aggregate member and its child layout. Target-specific facts about the emitted type itself, such as C's `repr(C)` and the aggregate's C name, ride in `ReprSpec.payload`; the calling convention of a whole function rides in `AbiSpec` at [the boundary](05-boundary.md#assembling-an-exported-function) instead. `LayoutId` and `PrimitiveId`, used below, refer to registered layout and operation descriptions.

A **slot** is one value in a multi-value representation — for a `Stamp` passed to JNI as two separate arguments rather than an object, the layout is two slots, and a function taking two such records has four native arguments in all. `SlotRole` states a slot's meaning, independent of its generated name. `GuardId` refers to an activation condition on a slot — “always,” “presence is true,” or “variant tag selects this arm” — and is unrelated to the guard items of [capture](01-source.md). Enclosing conditions also apply. Inactive slots can require valid wire defaults even though their source payload must not be read or constructed. When one layout is used for two function arguments, its slot identities are qualified by each use so their ABI positions remain separate.

`ProductOps` describes member projections and a target construction operation over already converted children. For a C struct these can be ordinary member reads and a struct literal. For separate JNI arguments they map children to slots. For object input they can be JVM-property-read primitives. The registry can provide standard tuple/struct operations as reusable defaults.

For sequences, variants and callbacks, adapters supply runtime operations; the registry supplies loops, branches and child calls. C aggregates and JNI slots/object operations reuse the same relation.

A layout stays nested for as long as nesting is meaningful: an aggregate whose member is itself an aggregate is described that way, and only a place that requires a flat list of values — a native signature, where each slot becomes one ABI argument — flattens it, at that point, in that use. Keeping the nesting until then is what lets the same record representation be an argument in one function and a member of another.

### Optional values

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

The target interface has four planning operations. Each answers a local question with a description. The registry owns recursion and calls the next planning operation when its inputs are ready.

```rust
trait Target {
    type Policy;  // Configuration choices recorded by the frontend.
    type Payload; // Owned operation/rendering descriptions.

    fn select(
        &self,
        query: SelectionQuery<'_, Self::Policy>, // Source value and applicable choices.
    ) -> TargetSupport<RelationSelection>;

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
([defined with the other support outcomes](06-retain.md#unsupported-requests-and-public-api-dependencies)).
Two of the answers are specified in later chapters, because they are about later
stages: [`BoundarySpec`](05-boundary.md#assembling-an-exported-function) describes
where native arguments and results go, and
[`SurfaceSpec`](06-retain.md#dependencies-of-public-declarations) describes a
public foreign declaration and what it requires.

The method inputs and results serve different stages:

| Method | Information available | Target's answer | Registry's next job |
| --- | --- | --- | --- |
| `select` | Exact source type/direction, local source facts and applicable conversion rules and policy | `RelationSelection`: the chosen relation, plus any choice it declares for a child or for a later step of the same conversion | Inspect that relation's parts and recursively resolve their conversions. |
| `represent` | `ResolvedShape`: source operation with model-derived child types; `ValueDescriptor`s: completed child layouts and contracts | Representation layout and target operations | Compose the complete value conversion. |
| `boundary` | `SiteDescriptor`: call signature/roles; `ResolvedValues`: its completed value descriptions | Argument placement, result delivery and error actions | Assemble and validate the complete native wrapper. |
| `surface` | `SurfaceRequest`: requested name, placement, policy and promises; required value descriptions | Public declaration description and requirements | Check dependencies before deciding whether to emit it. |

`select` is on this interface, even though a relation is a source-side
description, because the choice among the available relations depends on how the
target intends to carry the value: a representation that hands out an opaque
handle wants the atomic relation, not the fields. The target chooses; it does not
invent. It picks from the relations registered for that type, and where the
configuration pinned one, it either honours that choice or reports why it cannot.
Walking whatever it picked remains the registry's work.

These views are read-only. The adapter can inspect direct child descriptors but cannot invoke the registry's recursive compiler or modify the registry's plan tables. Relations and representations that recur — a record read through its members, a scalar carried unchanged — should be available to an adapter as ready-made descriptions it names rather than builds, so that a new target's first version is a handful of choices rather than a library.

Source-conversion dependencies appear in selected relations. Target operations list the generated helpers they need. When a target method needs an additional conversion, the method returns an explicit dependency request for the registry to resolve. Rendering operates on the completed result and cannot discover new conversions or support gaps.

Descriptions returned by a target can contain new primitive, layout, helper, or policy definitions with references local to that description. The registry validates and registers the definitions and assigns its own table IDs. Existing descriptors can reference IDs the registry already supplied. The target does not allocate entries in registry-owned tables itself.

The language-provided final rendering interface reads immutable plans and retained payloads. The common emission machinery supplies allocated operand names and the necessary rendering context; the rendering interface exposes no planning entry point. The original builder is translated before generation; subsequent decisions use requests and policies.

## The conversion plans the registry builds

For each supported conversion node, the registry records the selected source operation, target representation, generated instructions, and the conditions under which the result is usable.

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

`ConversionBodyId` points to structured instructions for locals, field access, variant matching, source construction/calls, primitive applications, conditions, and later loops or callback invocation. The common writer renders these instructions as Rust. It allocates temporary names centrally from identities.

The registry generates all child calls and source traversal. A projector binds its intermediate result once. Optional/variant branches convert only active children. The dependency list is derived from these instructions and the resolved relation; it is a convenience index maintained by the registry.

Access follows the exact source type and operation. For example, generating `&Stamp` input may require constructing an owned temporary and borrowing it for the duration of the source call. Supporting the two scalar fields alone does not implement that borrow. The conversion contract must preserve the temporary's validity through its uses.

For initial scalar/record support, ordinary owned Rust temporaries can rely on Rust destruction at scope exit. Handles and callbacks need implemented acquisition, transfer and cleanup semantics before they are supported. As those features are added, the registry will schedule resource scopes and cleanup on success and failure paths; adapters provide the actual retain/free/runtime operations.

## What is not settled here

The contracts above are complete enough to divide the work and to plan the
fixture's conversions, and not complete enough to implement the planner from.
[The implementation plan](../implementation.md#what-this-design-does-not-settle-yet)
lists what the first increment still has to decide — chiefly the instruction set
behind `ConversionBodyId`, the shapes of the target interface's parameter types,
and the composition protocols for products, sequences, variants and callables.

## Elements at this stage

- [Function taking an owned record][fn_values] · [C][fn_values_c] · [Kotlin/JNI][fn_values_jni]
- [Record with scalar fields][struct_values] · [C][struct_values_c] · [Kotlin/JNI][struct_values_jni]

---

Previous: [Record binding requests](03-requests.md) · Next: [Assemble the native boundary](05-boundary.md)

[fn_values]: ../examples/fn/04-values.md
[fn_values_c]: ../examples/fn/04-values.c.md
[fn_values_jni]: ../examples/fn/04-values.jni.md
[struct_values]: ../examples/struct/04-values.md
[struct_values_c]: ../examples/struct/04-values.c.md
[struct_values_jni]: ../examples/struct/04-values.jni.md
