<!-- spec: {"kind": "extensions"} -->

[Project contents](README.md)

# Extension contracts

Nothing on this page is built. The stage chapters describe what the engine does
to the three element paths — a function taking an owned struct, that struct,
and a type alias declaring an opaque handle; this page holds the contracts designed for what it does not do yet, gathered
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
names a function, so a frontend declaration has to say which. It says so in
the `Via` of a `Product` representation in a
[conversion rule](stages/03-requests.md#conversion-rules), which gains one
variant per relation:

```rust
pub enum Relation {
    Struct(StructRelation),
    Construct(ConstructorRelation),
    Project(ProjectionRelation),
    // Later: checked atomic, optional, sequence, variant, Rust-to-Rust
    // conversion, representation-reuse and callable descriptions.
}

pub enum Via {
    Fields,
    Construct(syn::Ident), // Into Rust through this source function.
    Project(syn::Ident),   // Out of Rust through this source function.
}

pub enum Step {
    Param(String),
    Return,
    Field(String),
    Arg(String), // An argument of the constructor the value is built through.
}
```

Pinning the constructor for `Stamp` is a rule recorded with the request, and it
changes what the parts are without changing anything else:

```text
rule:      Type(Stamp) -> Product { via: Construct(stamp_from_millis), carrier, read, build }
relation:  Relation::Construct, over the checked function view, registered for Stamp
parts:     millis: i64, addressed as `….arg millis`
```

The registry resolves `Construct(stamp_from_millis)` the way it resolves
`Fields`: it checks the function against the type before planning, and a
function that does not construct the type fails the build as invalid input.
The [carrier](stages/05-represent.md#describing-target-values-and-operations)'s
members are then the constructor's arguments, so its writer
is fed one `millis` member instead of `secs` and `nanos`.

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
in the [wrapper](stages/06-boundary.md#assemble-the-wrapper-boundary). The
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
Needed by a borrowed handle, an optional handle, and a retained callback.

A described operation carries its failure and its generated dependencies, and
nothing about resources. The one resource the engine hands across a boundary
today — [an owned opaque handle][typedef_represent] — is safe without a
contract because its obligations are discharged by construction: handing out
is the last operation of a wrapper that returns the
[carrier](stages/05-represent.md#describing-target-values-and-operations),
taking back moves
the value into an ordinary owned local that Rust drops on every path, and
releasing is a wrapper of its own that converts nothing. Every other operation
acquires nothing. The first operation that breaks that shape — a borrowed
handle whose referent must stay alive across the source call, a handle inside
an optional, a callback that retains a JVM reference — cannot be written
without this.

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
no ownership obligation escapes either execution path. Borrowed handles,
callbacks and escaping allocations remain unsupported until their effects can
be represented and validated — that is, until the table above can be filled in for them and
the registry can check what it says.

By comparison, `dependencies` concerns the generated program: it retains helper
functions and type declarations required to compile the operation. It does not
arrange runtime cleanup. A dependency on an external runtime crate belongs in
the target's build requirements; a generated helper wrapping that runtime call
is an artifact.

## Containers

Extends [conversion rules](stages/03-requests.md#conversion-rules). Needed by
the sequence-field and optional-field paths.

A `Type` rule names one exact type, and a binding cannot list every
`Vec<Stamp>`, `Vec<Ledger>` or `Option<i64>` its source happens to use. What
makes a finite answer possible is that the target side is finite: an adapter
has a handful of wire types, and a handful of ways to pack wire values into a
container. So a generic instance is matched one container level at a time,
against the wire type its element *resolved to* — never against a pattern of
Rust types.

Each carrier states which of the adapter's wire types it is, as its
`class`, and the adapter declares its containers:

```rust
pub struct WireType<T: Target> {
    pub rust: syn::Type,
    pub abi: bool,
    pub class: T::WireClass, // JNI: Long, Int, Byte, …, Object. C: I64, …, Pointer, Aggregate.
    pub base: String,        // Its naming base: `stamp`, `i64`.
    pub meta: T::CarrierMeta,
}

/// The Rust containers the registry knows the relation of.
pub enum SourceContainer { Vec, Slice, Array, Option, Box }

pub struct Container<T: Target> {
    pub source: SourceContainer,     // Which Rust container it serves.
    pub accepts: T::WireClass,       // Which element wire type it holds.
    pub role: String,                // Its naming role: `vec`, `opt`.
    pub carrier: CarrierTemplate<T>, // The result's Rust type and class; its metadata
                                     // is derived per instance from the element's.
    pub ops: ContainerOps<T::Op>,    // A sequence: len, get, build. An optional:
                                     // test, extract, inject.
    pub release: Option<Operation<T::Op>>,
    pub niche: Option<Niche>,        // What the result leaves free; see optional values.
}

impl<T: Target> Binding<T> {
    pub fn container(&mut self, container: Container<T>);
}
```

The registry owns each container's relation, since it is a source-side fact
and the same for every target: a `Vec<T>`, a slice or an array is one part,
the element, converted once per element; an `Option<T>` is one part, present
or absent. A value of a container type takes, in order:

1. the rule at its position;
2. the `Type` rule for its exact type — `Type(Vec<u8>)`, say, for a byte array
   carried whole;
3. the container table: its element is planned first, at its own position,
   and the registry looks up the container declared for this Rust container
   and the element's wire class.

A missing entry refuses the value, naming the Rust container and the wire
class: `Vec` of `Aggregate` has no container in JNI.

For JNI the table is:

| Rust container | Element wire class | Container | Instance carrier |
| --- | --- | --- | --- |
| `Vec`, slice | `Long` | `jlongArray`, read in bulk | `[J` |
| `Vec`, slice | `Byte` | `jbyteArray`, read in bulk | `[B` |
| `Vec`, slice | `Object` | `jobjectArray`, element by element | `[` + the element's descriptor |
| `Option` | `Long` | a boxed `java.lang.Long` | `Ljava/lang/Long;` |

and for C:

| Rust container | Element wire class | Container | Instance carrier |
| --- | --- | --- | --- |
| `Vec`, slice | any | `{ ptr, len }` | a `repr(C)` struct per instance |
| `Option` | a class with no niche | `{ bool present; E value }` | a `repr(C)` struct per instance |

Three things follow from matching the element's wire class rather than a
Rust pattern. Two entries cannot overlap, since each covers one container
level and one class, so there is no precedence among them to define. An
element that changes the container's Rust type is simply a different key:
`Vec<i64>` becomes a `jlongArray` and `Vec<Stamp>` a `jobjectArray`. And an
override on an element flows up by itself: a rule at `param xs.element` that
makes `Stamp` a handle gives the element class `Long`, and the container
follows.

A container instance is a carrier the registry builds during planning, from
the container and the element's carrier. It holds it as that pair, compared
structurally, and feeds both to the writers: the JNI writer spells
`[Lexample/Stamp;` from the element's descriptor, and the C writer writes the
instance's `repr(C)` struct from its element's carrier. Planning still calls
no target code; the instance is data the binding's declarations determine.

An instance's name is composed the way a callback's closure struct is named
today, from bases. Its base is the container's role and the element's base —
`Vec<Stamp>` is `vec_stamp`, and nesting composes, so `Vec<Option<Stamp>>` is
`vec_opt_stamp`. The target turns a base into a name with the frontend's own
manglers, as it does for a declared type: `mangle_type_name` gives the C
struct its name and `mangle_destructor` gives a `Vec` handed out of Rust its
release symbol, `vec_stamp_drop` by default. No planning decision depends on
a name, so the target applies the manglers when it writes, through
`Target::write_name`, and the registry checks the names for collisions once
they are written.

## Optional values

Extends [containers](#containers). Demonstrated, when built, by the struct
with an optional field.

An `Option<T>` is cheapest when the element's wire value has a value no real
element ever takes — a **niche** — and absence can be that value. Whether one
exists depends on the representation, not on the wire type alone. A `jlong`
carrying a handle is never 0, because `Box::into_raw` never returns null; a
`jlong` carrying an `i64` can be anything. A C `*mut ledger_t` handle is never
null; a `JObject` a data-class conversion produces is never null. So the
representation that produces a value declares its niche:

```rust
pub enum Niche {
    Null, // A null pointer or a null object reference.
    Zero, // An integer 0.
}

// On `Representation::Terminal` and `Representation::Product`:
pub niche: Option<Niche>,
```

An `Option<X>` resolves from its element's representation:

| The element's representation | `Option<X>` |
| --- | --- |
| has a niche | The element's carrier. Absent is the niche value; the test for it and the conversion each way are standard operations the registry writes. The result has no niche left. |
| has none | The container declared for `Option` and the element's wire class: C's `{ bool present; E value }`, JNI's boxed `java.lang.Long`. The result's niche is the container's. |

Nesting falls out of that. `Option<Ledger>` takes the pointer's null.
`Option<Option<Ledger>>` finds that niche consumed and takes the flag
container, so `None` and `Some(None)` stay distinct. In JNI,
`Option<i64>` is a boxed `Long`, whose own niche is `null`, and an
`Option<Option<i64>>` then needs a container for class `Object` or is
refused.

A container's `inject` writes the whole absent form, including a valid value
for a member that is not read: C's `{ false, 0 }`. Its `extract` is only
reached on the present path; the registry branches on `test` and converts
the element only there.

## Multi-value layouts

Extends [the wrapper boundary](stages/06-boundary.md#assemble-the-wrapper-boundary).
Needed by the sequence-field path, and by JNI's `expand_param`.

Inside a plan one value is always one carrier. A C slice is one `{ ptr, len }`
aggregate, and a `Stamp` read from a JVM object is one `JObject`, wherever
the value sits: as a parameter, a field, or an element of another container.
Only the wrapper boundary may split a value into several wrapper parameters,
and the function form says which:

```rust
pub struct FunctionForm<T: Target> {
    // … the calling convention, symbol, context parameters, routes …
    /// The parameters, or the return, that cross as their members rather
    /// than as one value.
    pub flatten: Vec<Step>, // Step::Param("xs"), Step::Return
}
```

A flattened value must be carried in an aggregate: a carrier with members,
such as a `repr(C)` struct. The registry checks that before planning. At the
boundary it replaces the one wrapper parameter with one per member, each typed
as that member's carrier and named by the target's writer — `xs` becomes
`xs_ptr` and `xs_len` — and binds each member directly where the plan would
have read it out of the aggregate. A flattened return becomes one
out-parameter per member, in a convention that has out-parameters: C does, and
a JNI method returns one value.

The aggregate need not be one the foreign side ever sees whole. A carrier
declared with `abi: false` is a Rust-only intermediate, and flattening is how
its members reach the wrapper signature. That is how JNI's `expand_param`
fits: a rule at `param stamp` selects a `Product` over an internal aggregate
of `Stamp`'s two parts instead of the `JObject`, and the function form
flattens `param stamp`.

| Frontend setting | Declarations | Wrapper parameters |
| --- | --- | --- |
| C, a slice parameter | `xs: &[i64]` takes the slice container, carried in `slice_i64 { ptr, len }`; the form flattens `param xs` | `xs_ptr: *const i64, xs_len: usize` |
| JNI, `expand_param(stamp)` | `At(f, param stamp)` selects a `Product` over an internal `{ secs: jlong, nanos: jlong }`; the form flattens `param stamp` | `secs: jlong, nanos: jlong` |

Neither the slice container nor `Stamp`'s other representations know about
the flattening, and everywhere else the same `Stamp` is still one `JObject`.

## Requesting further conversions

Extends [what the target writes](stages/04-select.md#what-the-target-writes).

A value's conversions come from its representation's relation and the rules
for its parts, and a writer's needs are its helpers. A target that needs a
conversion beyond those — a helper that takes a value the plan has no reason
to convert — states it in the binding, as another carrier and rule, so it is
planned and checked like any other. It is never requested while writing: a
writer that discovered a conversion would be a decision the plan could not
see, and planning would stop being a function of the model and the binding.

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

[typedef_represent]: examples/typedef/05-represent.md
