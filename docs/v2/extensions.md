<!-- spec: {"kind": "extensions"} -->

[Project contents](README.md)

# Extension contracts

Nothing on this page is built. The stage chapters describe what the engine does
to the two element paths, a function taking an owned struct and that struct;
this page holds the contracts designed for what it does not do yet, gathered
here so that a chapter reads as an account of the pipeline rather than as an
account interleaved with its own future. Each section names the stage it
extends and the element path that will demonstrate it, and
[the implementation page](implementation.md#what-it-does-not-settle) keeps the
list of what remains open. Rust API sketches here illustrate intended
contracts, not published APIs, and a name that also exists in current code —
`Relation`, `PrimitiveSpec`, `Layout`, `ValuePlan` — is shown in the shape the
extension needs, not the shape it has.

The extensions fall along the same seam as the chapters: what is known about
the source side of a [conversion](stages/04-select.md#select-conversion-relations),
what the target contributes to its
[representation](stages/05-represent.md#represent-and-compose-values), and the
contracts a finished plan carries.

## Source views

Extends [building the source model](stages/02-flat.md); used by
[selection](stages/04-select.md).

The planned view-based API lets the registry read source through the same
[views everything else uses](stages/02-flat.md#lookup-and-navigation) —
`FunctionView::parameters`, `TypeView::as_record`, `StructView::fields` — with no
private channel of its own. Current V2 uses borrowed Flat records instead; the
view names on this page describe the intended API, not callable APIs today.

`ParameterView` and `FieldView` provide their exact `TypeView`s. Flat creates the
views and keeps their constructors and storage indices private. A struct view
exposes structural fields only when Flat models them. Reaching a struct through
`&Stamp` requires an explicit `referent()` step; that inspection does not itself
implement a borrow conversion. Details of
[storage and model checks](stages/02-flat.md#private-storage-and-model-consistency)
and [incremental adoption](implementation.md#flat-implementation-sequence-and-acceptance)
belong to the Flat project.

For `parse_stamp(&str) -> Result<Stamp, Error>`, Flat reports one parameter and
a `Result` return type. Whether `Ok` means successful construction is decided by
the registry when that function is selected as a constructor, below.

## Constructor and projector relations

Extends [registering and selecting](stages/04-select.md#registering-and-selecting-a-relation)
a [relation](stages/04-select.md#what-a-relation-is). No element path
demonstrates a helper-based relation yet.

The engine has two relations: the
atomic one and a struct's fields. Two more are designed, both with a source
function as the means of getting between the type and its parts. Under a
**constructor**, `Stamp` is related to `(millis: i64)` by `stamp_from_millis`;
under a **projector**, it is related to `StampParts` by an accessor
`stamp_parts(&Stamp)`. A constructor does not automatically provide an inverse
accessor.

The same function can be exported directly, selected as a constructor, or used
to extract another value. Those are binding roles, so relation construction
belongs in the common registry library; Flat supplies the checked source facts
used to validate them. The proposed API combines the planned Flat views with
checked constructors for the three roles — and this proposed private
`StructRelation` replaces the public-field structure the chapter shows; it is
not a second implemented definition:

```rust
impl StructRelation {
    pub fn new(view: StructView) -> Self;
}
impl ConstructorRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}
impl ProjectionRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}

pub struct StructRelation {
    view: StructView,   // Derive subject and fields from the checked struct view.
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

All relation fields are private. Read-only accessors expose source types and
arguments. A constructor's subject is derived from its result; a projector's
subject and access requirements come from its input. Callers cannot pair an
arbitrary subject with an unrelated operation. For
`stamp_from_millis(i64) -> Stamp`, the constructor has one `i64` argument and
produces `Stamp`, regardless of `Stamp`'s fields. For a helper
`stamp_parts(&Stamp) -> StampParts`, where `StampParts` is a named struct with
`secs: i64` and `nanos: i64` fields, projection requires a shared borrow and
produces that struct.

`RelationError` reports a source function incompatible with the requested role.
Validation establishes the role's internal consistency, not that every target
supports it. The registry separately checks a selected role against the
conversion's expected type, direction and ownership: producing `Stamp` alone
does not satisfy `&Stamp` without a supported temporary-and-borrow step. Default
fallible construction interprets `Result::Ok` as the value and `Err` as failure;
a different treatment requires an explicit conversion role.

A struct relation is implicit — the registry registers it for any struct the
source model describes. A constructor or projector relation is explicit: it
names a function, so a frontend declaration has to say which. That declaration,
and the rule that pins the relation at a position, is the **conversion rule**
the chapter's `select` reads and that no build script can write yet:

```rust
pub enum Relation {
    Struct(StructRelation),
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

Pinning the constructor for `Stamp` is a rule recorded with the request, and it
changes what the parts are without changing anything else:

```text
relation:  stamp_from_millis  = Relation::Construct(over the checked function view)
parts:     PartId { owner: stamp_from_millis, arm: None, position: Argument(0) }  // millis: i64
rule:      the Stamp type default selects that relation instead of Stamp.fields
```

The registry then converts one `i64`, calls `stamp_from_millis`, and has a
`Stamp` — one child instead of two, the same recursion, and a target that need
not know which happened. Registry-library registration accepts checked relations
and issues opaque `RelationId`s; request import validates their model and table
context. Frontends may construct these descriptions without running recursive
conversion planning. A projection calls its helper once and processes the saved
result through the conversion rules; helper arguments need not resemble fields.

A target that cannot work with a pinned relation reports that as unsupported —
the request is well formed, the capability is missing, and the affected outputs
cannot be generated. That is different from a rule that contradicts the source,
such as naming a constructor for a type it does not construct, which is invalid
input and fails the build.

Future callback planning must reverse direction for callback arguments. Rust
receives the callable as input, but later supplies values to the foreign
callback as output. Those values make the opposite
[crossing](stages/03-requests.md#finding-an-existing-conversion-plan). The
direction follows from the callback's role, not from the user specifying an
independent direction for each argument.

## Validity of results

Extends [the operation contract](stages/05-represent.md#failure-of-an-operation).
Needed by the borrowed-parameter path.

Validity is absent from a described operation because every result the engine
plans is a value copied out. A borrowed or scope-bound result is what needs
this contract. It answers a different question from failure: how long can a
successfully produced value be used? A copied integer is independent of the
object it came from. A reference into a buffer remains tied to that buffer. A
JNI local object reference remains tied to its JNI reference scope.

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

`OperandIndex` refers to a position in this
[primitive](stages/05-represent.md#represent-and-compose-values)'s signature,
not an exported function's parameter list. `ScopeRequirement` describes a
required runtime scope and the context operand through which the operation
accesses it, such as the JNI environment and its active local-reference frame.
During composition, the registry resolves that requirement to a concrete scope
in the [wrapper](stages/06-boundary.md#assemble-the-native-boundary). The
checked specification constructor validates operand indices and result counts.
The registry rejects a use if the required scope is unavailable or a result
would escape its dependencies. `Independent` does not imply that a value is
copyable or that it has no destructor.

In the full contract a described operation carries validity beside its failure:

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

`ArtifactId` is a proposed table-based reference to an
[artifact](stages/05-represent.md#individual-target-operations); current
descriptions carry whole `Artifact` values, identified by name. Under that
contract the JNI getter the chapter shows would also state that the integer it
returns owes nothing to the object it came from —
`validity: ValidityContract { results: vec![ResultValidity::Independent] }` — and
`resources: ResourceContract::none()`.

## Runtime resources

Extends [the operation contract](stages/05-represent.md#failure-of-an-operation).
Needed by any handle-bearing path.

A described operation carries its failure and its generated dependencies, and
nothing about resources. Every operation the engine plans acquires nothing,
which is why the omission is safe today and why the first handle-bearing
operation cannot be written without this.

`ResourceContract` describes obligations introduced or discharged by an
operation, such as releasing a retained handle. It is separate from validity: a
value can have a long enough lifetime and still leak if nobody releases it. The
initial scalar/owned-struct implementation relies on ordinary Rust destruction
for its owned locals and performs no additional handle acquisition requiring
explicit cleanup.

Resource-bearing operations require these facts before the registry can support
them:

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
no ownership obligation escapes either execution path. Handles, callbacks and
escaping allocations remain unsupported until their effects can be represented
and validated — that is, until the table above can be filled in for them and
the registry can check what it says.

By comparison, `dependencies` concerns the generated program: it retains helper
functions and type declarations required to compile the operation. It does not
arrange runtime cleanup. A dependency on an external runtime crate belongs in
the target's build requirements; a generated helper wrapping that runtime call
is an artifact.

## Wrapper form

Extends [the boundary](stages/06-boundary.md#assembling-an-exported-function).
Needed by a target whose wrappers are not reached as `extern` functions under
exported symbols, or not in the form the writer gives them.

The common Rust writer renders every
[wrapper](stages/06-boundary.md#assemble-the-native-boundary) in one form:

```rust
#[no_mangle]
pub extern "<abi>" fn <symbol>(<params>) -> <ret> { … }
```

Of that, `AbiSpec` lets the target decide the calling convention string, the
symbol, and the native parameters and return. The attributes, the visibility
and the absence of `unsafe` are fixed, which the two targets built so far
accept. The chapter sketches `AbiSpec<Payload>` — "calling convention, symbol
and target signature requirements" — and the payload is what would carry the
rest: attributes the wrapper needs, an `unsafe` the target's convention
demands, a linkage other than `#[no_mangle]`, or a registration the target
performs instead of exporting a symbol. The writer would render what the
payload states around the body it already renders, and the registry would
check it as it checks the rest of the boundary — a symbol that is not a Rust
identifier is refused today, and a payload would be refused on the same terms.

The first known consumer is V1's JNI writer, whose wrapper is
`#[no_mangle] #[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn`: an `unsafe` signature and lint attributes the V2
form cannot state. Neither is required for the V2 JNI target as built, which
is why the payload waits.

## Multi-value layouts

Extends [target representations](stages/05-represent.md#target-representations).
Needed by the optional-field, sequence-field and enum paths.

The engine has two layouts, one scalar [carrier](stages/05-represent.md#describing-target-values-and-operations)
or one aggregate, and two protocols, converting the whole value or reading one
part per part. The broader design adds an empty layout, independent slots,
nested member layouts, id-based references and four more protocols:

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

enum Protocol {
    Terminal { codec: PrimitiveId }, // Converts the whole value in one operation, with no parts.
    Product(ProductOps),   // Project members and construct a target product.
    Optional(OptionalOps), // Detect/extract/inject presence or absence.
    Sequence(SequenceOps), // Read or append target sequence elements.
    Choice(ChoiceOps),     // Inspect/write a tag and its active payload.
    Callable(CallableOps), // Capture/invoke a target callable.
}
```

A **slot** is one value in a multi-value representation — for a `Stamp` passed
to JNI as two separate arguments rather than an object, the layout is two slots,
and a function taking two such structs has four native arguments in all.
`SlotRole` states a slot's meaning, independent of its generated name. `GuardId`
refers to an activation condition on a slot — "always," "presence is true," or
"variant tag selects this arm" — and is unrelated to the guard items of
[capture](stages/01-source.md). Enclosing conditions also apply. Inactive slots
can require valid wire defaults even though their source payload must not be
read or constructed. When one layout is used for two function arguments, its
slot identities are qualified by each use so their ABI positions remain
separate.

A layout stays nested for as long as nesting is meaningful: an aggregate whose
member is itself an aggregate is described that way, and only a place that
requires a flat list of values — a native signature, where each slot becomes one
ABI argument — flattens it, at that point, in that use. Keeping the nesting
until then is what lets the same struct representation be an argument in one
function and a member of another.

`ProductOps` in the design describes both member reads and a target
construction operation over converted children. The implemented form has no
target-construction operation, which is why a struct leaving Rust is a reported
skip. A future C output could use a struct literal, while a future
separate-arguments JNI form would map children to argument slots. For
sequences, variants and callbacks, adapters supply runtime operations; the
registry supplies loops, branches and child calls.

## Optional values

Extends [target representations](stages/05-represent.md#target-representations).
Demonstrated, when built, by the struct with an optional field.

Nothing carries an optional value yet: `Layout::Slots`, `SlotRole`, `GuardId`
and the encodings below are names. This section says what the slot has to hold.

An optional value needs both a representation of its child and a way to
distinguish absence. Different targets can encode that distinction differently:

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

A **niche** is a reserved representation that cannot be a valid present child,
such as zero for a handle whose valid values exclude zero. `DomainId` describes
those validity facts. `DefaultsId` describes valid wire defaults, not fabricated
Rust source values. `absent` builds the complete absent representation;
`inactive` supplies the unused child slots for the separate-flag convention.

The registry branches on presence and invokes the child conversion only on the
present path. It validates active inputs and supplies required inactive
defaults. Nested optionals must preserve distinct states such as `None` and
`Some(None)`; if the selected encoding cannot do that, the combination is
unsupported.

## Requesting further conversions

Extends [how the registry asks a target for decisions](stages/04-select.md#how-the-registry-asks-a-target-for-decisions).

Source-conversion dependencies come from the selected relation's children, and
target operations list generated helpers. Current public dependencies are
`DeclarationId`s in `SurfaceSpec.requires`. The more general design will need an
explicit request mechanism if a target requires additional conversions beyond
those; that mechanism is not implemented. Rendering must not discover new
conversions.

Descriptions returned by a target can contain new primitive, layout, helper, or
[policy](stages/03-requests.md#what-policy-means) definitions with references
local to that description. The registry validates and registers the
definitions and assigns its own table IDs. Existing descriptors can reference
IDs the registry already supplied. The target does not allocate entries in
registry-owned tables itself.

## The full value contract

Extends [the conversion plans the registry builds](stages/05-represent.md#the-conversion-plans-the-registry-builds).

The extended plan brings together the validity, resource and model-view
contracts above. The current [node](stages/05-represent.md#represent-and-compose-values)
stores `id`, `crossing`, `relation`, `repr`, `children`, a `NodeBody`, and
failure categories; it does not yet contain `ValueContract`,
`ResolvedRelation` or `ConversionBodyId`.

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

`ResolvedRelation` is the chosen relation after its source references and child
conversions are resolved. `ValueType` says what a finished conversion produces:
either a source Rust type, or a layout of carriers holding zero, one or several
values. It differs from an operation's `OperationType` in exactly that arity —
one operand or result is always a single value, whereas a conversion's product
can be a group of slots. `Validity` composes the individual primitives'
validity rules: for example, a produced reference remains tied to a particular
temporary. `FailureSet` collects possible error categories and types; the
function boundary decides their eventual handling.

`ConversionBodyId` points to structured instructions for locals, field access,
variant matching, source construction and calls, primitive applications,
conditions, and later loops or callback invocation — the three instructions the
chapter shows, grown to cover the protocols above. A projector binds its
intermediate result once. Optional and variant branches convert only active
children.

Access follows the exact source type and operation. Generating `&Stamp` input
may require constructing an owned temporary and borrowing it for the duration of
the source call. Supporting the two scalar fields alone does not implement that
borrow; the conversion contract must preserve the temporary's validity through
its uses. As handles and callbacks are added, the registry will schedule
resource scopes and cleanup on success and failure paths; adapters provide the
actual retain, free and runtime operations.
