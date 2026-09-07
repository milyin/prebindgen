# V2 binding-generation architecture

Status: proposed design for [issue #720](https://github.com/milyin/prebindgen/issues/720). This document describes the architecture and implementation plan; its Rust examples are schematic API sketches.

## Purpose and scope

Build a second generation pipeline around a common **registry**: an engine that takes the Rust source model and the consumer's choices, determines the required conversions, and assembles instructions for generating bindings. Language implementations describe how foreign code represents Rust values and provide the operations specific to their runtime. The registry combines those descriptions into complete conversions and exported functions using algorithms shared by C, JNI/Kotlin, and future languages.

V2 accepts the existing examples' complete Rust inputs and preserves their existing language-frontend APIs. Users configure a C or JNI frontend through Rust builders and macros, typically called from `build.rs`. The choices recorded by those calls—items to expose, names, representations and overrides—are the **binding configuration**. V2 initially generates only supported elements and reports why the others were skipped. The resulting Rust wrappers, C headers and Kotlin sources are the **generated bindings**. For an emitted element, correctness means preserving its declared behavior, ownership, error handling, and public/native interface. Generated source does not have to be byte-identical to v1.

[#719](https://github.com/milyin/prebindgen/issues/719) covers introducing v2 and switching existing examples. The types below are proposed schematic contracts; future capabilities are extension points, not implemented features.

## 1. What the registry does

The registry receives the existing [`Flat` source model](https://github.com/milyin/prebindgen/blob/main/prebindgen-flat/src/flat/mod.rs) and requests to expose particular types, functions, and constants. For each request, it builds a **plan**: structured data describing the required conversions, calls and their execution order. The common Rust writer renders the native wrappers and supporting Rust types. The C build then uses `cbindgen` to derive C headers from that Rust output; the JNI implementation renders Kotlin declarations from the completed plans. Planning happens in the generator; the generated operations execute later when the bindings are used.

For example, consider this illustrative source API:

```rust
struct Stamp {
    secs: i64,
    nanos: i64,
}

fn normalize(stamp: Stamp) -> Stamp;
```

Generating a binding requires several decisions and operations:

1. Choose how a foreign caller represents a `Stamp`.
2. Obtain and convert its `secs` and `nanos` values.
3. Construct the Rust `Stamp` and call `normalize`.
4. Read and convert the returned fields.
5. Return the result through the chosen foreign interface.

A **target** is a language and its native calling interface, such as C or Kotlin through JNI. Each language implementation provides a **target adapter**, which supplies the representation choices and runtime operations needed by the registry.

A C binding might represent `Stamp` as a C struct. A JNI binding might accept two native integer arguments produced by a Kotlin wrapper, or receive a JVM object whose properties must be read. Those representations need different target operations. The source-side work of discovering two fields, converting them, constructing `Stamp`, invoking the source function, and processing its result is common.

**The registry owns that common work.** When a record gains another nested record field, the shared recursive registry algorithm should process it using the representations supplied by the target. Each language should not need another implementation of record traversal or wrapper assembly.

Each language implementation provides a **frontend**: the public Rust API through which users configure that language's bindings. When the user calls the frontend's build method, the frontend creates **binding requests** for the registry internally. These requests are the registry's input API for frontend implementations; users configure the frontend and do not construct requests themselves. A language implementation may also provide a **foreign writer**, an optional component that renders foreign-language source from the completed plans. The JNI implementation provides a Kotlin writer. The C implementation needs no foreign writer: the external `cbindgen` tool generates C headers from the generated Rust types and functions. The registry library provides the **common Rust writer**, which emits native Rust wrappers and supporting Rust types for both targets.

<table>
<thead>
<tr><th>Component</th><th>Responsibility</th><th>Example C</th><th>Example Kotlin</th></tr>
</thead>
<tbody>
<tr>
<td>Language frontend</td>
<td>Provide the public language-specific Rust API and pass the recorded choices to the registry as binding requests.</td>
<td>Expose <code>Stamp</code> as a C data struct and <code>normalize</code> as a C function.</td>
<td>Expose <code>Stamp</code> as a Kotlin data class in a specified package and place <code>normalize</code> in the requested API.</td>
</tr>
<tr>
<td>Registry</td>
<td>Plan Rust field access, value construction and helper calls; combine child conversions and track dependencies.</td>
<td colspan="2">Convert both fields, construct the Rust <code>Stamp</code>, call <code>normalize</code> once, and process its result using the target's representation operations.</td>
</tr>
<tr>
<td>Target adapter</td>
<td>Describe representations, runtime operations and boundary conventions.</td>
<td>Represent <code>Stamp</code> as a C struct with member-access operations; select the declared C return/out-parameter and error conventions.</td>
<td>Represent <code>Stamp</code> as separate JNI arguments or a JVM object with property-read operations; select the declared JNI result and error-handler conventions.</td>
</tr>
<tr>
<td>Common Rust writer</td>
<td>Render the registry's completed native conversion and function plans.</td>
<td colspan="2">Emit Rust locals, field accesses, source calls, branches and the extern wrapper from the common plans, using the chosen native calling convention and target operations.</td>
</tr>
<tr>
<td>Foreign writer (optional)</td>
<td>Render foreign-language source when the language implementation needs a custom writer.</td>
<td>Not needed. The build invokes the external <code>cbindgen</code> tool on generated Rust to produce C headers.</td>
<td>Emit the Kotlin <code>Stamp</code> class and typed wrappers that call the generated JNI boundary.</td>
</tr>
</tbody>
</table>

The implementation divides the registry's data between two structures:

- **`Registry`** uses an existing `Flat` model and adds the binding-generation operation, `generate(adapter, requests)`. `adapter` is the language implementation's object implementing the proposed `Target` interface; `requests` contains the choices recorded by the frontend. The registry determines how to construct or read Rust values—through their fields, constructors, accessors or conversion helpers—then combines the required conversions, checks dependencies and assembles binding plans. Source types, fields and signatures remain described by `Flat`.
- **`GenerationRun`** holds the temporary planning state inside `Registry::generate`: binding requests, conversion plans being built, dependencies and skip reasons. The registry creates this state internally and processes the complete request set in it. On success, the registry returns a **`Generation`** containing the completed plans and report; on failure, the registry returns an error.

A binding crate normally builds one configured frontend. The frontend calls `Registry::generate` once for all requests. `Flat` supplies source facts; the registry plans conversions using `GenerationRun` working state.

## 2. Binding requests and target policy

The common [`prebindgen-flat` library](https://github.com/milyin/prebindgen/blob/main/prebindgen-flat/src/lib.rs) supplies Rust source facts through `Flat`. Users call the C or JNI frontend's API to choose the generated interface: for example, exposing `Stamp` as a C data struct or a Kotlin class. The configured frontend creates `BindingRequests` and calls the registry.

A declaration or setting recorded by the frontend is a **configuration entry**. A frontend call exposing a function records an output request; an argument override records the requested representation choice. The frontend also preserves naming hooks, ignored items and unsupported settings. Recording a request does not establish that the registry can generate it.

### What policy means

A **target policy** records the target-specific choices applicable to a request or a particular value conversion. It is configuration data. It does not contain a recursive conversion algorithm or a finished wrapper.

Examples include JNI object versus separate-argument input, C struct versus handle, and function error handling or public placement.

An illustrative portion of JNI policy could be:

```rust
// Illustrative choices, not a replacement for the existing builder/macro API.
enum JniRecordInput {
    SeparateArguments, // Kotlin supplies the fields as individual JNI arguments.
    ObjectProperties,  // JNI receives an object and reads its properties.
}

struct JniValuePolicy {
    record_input: JniRecordInput, // How this record reaches the native wrapper.
}
```

This choice does not list `Stamp`'s fields or explain how to construct it. The registry obtains those facts through a **source relationship**: a description of how a Rust value is constructed or read, such as using fields or a helper (the `RelationDefinition` in section 4). The adapter interprets the policy when describing the target representation and its property/argument operations.

Three concepts stay separate throughout the proposal:

| Concept | Question it answers | `Stamp` example |
| --- | --- | --- |
| Source relationship | How can the Rust value be constructed or read? | Construct/read its `secs` and `nanos` fields. |
| Target representation | What values carry it, and how are those values accessed? | One C struct, two JNI integer arguments, or a JVM object. |
| Boundary delivery | Where do the converted values go at an exported call? | Native return, caller-provided output parameters, or a declared result callback. |

Policy guides the selection of these descriptions. The registry turns the descriptions into an executable plan.

### The registry API called by the frontend

The registry separates what should be generated from how values should be converted. An **output request** asks for one element, such as a function, type or constant. A **conversion rule** selects a source relationship and target policy for a particular type, parameter, result or child value. `BindingRequests` collects these requests and rules together with their policies and the information needed to report unsupported or ignored entries:

```rust
struct BindingRequests<Policy> {
    outputs: Vec<OutputRequest>,    // Explicit output requests; defined in section 3.
    conversion_rules: ConversionRules, // Source-operation selections and applicability; section 4.
    policies: PolicyTable<Policy>,// Target configurations referenced by PolicyId.
    unsupported: Vec<UnsupportedRequest>, // Requests the frontend cannot yet fully translate.
    ignored: Vec<ElementId>,      // Explicit user opt-outs retained for reporting.
}
```

`Policy` is a Rust generic type parameter supplied by the language implementation. The C frontend and JNI frontend produce the same `BindingRequests` structure with different policy types. `PolicyTable` stores their configurations; `PolicyId` is a typed reference to one entry. A concrete implementation can use separate policy types for value conversion, function boundaries and public declarations instead of one large enum.

The registry plans `outputs`, applies `conversion_rules`, reports `unsupported`/`ignored` entries, and asks the adapter to interpret `policies`.

Suppose the user configures the JNI frontend to accept `Stamp` as two integer arguments by default, then overrides the `Stamp` parameter of function `f` to accept a JVM object. The registry uses the explicit object choice for `f`. Function `g`, which has no override, keeps the two-argument default. The settings selected for a particular conversion are its **effective policy**. A choice recorded for a particular field or constructor argument is also applied where that child is converted, following the frontend API's documented override rules.

Identical type, construction and representation choices can share a converter; the object override needs a different converter.

`UnsupportedRequest` retains request identity, location and reason when a frontend cannot honor a setting. The registry reports and propagates that failure.

`Registry` exposes this generation method (signature only):

```rust
pub fn generate<A: Target>(
    &self,
    adapter: &A,
    requests: BindingRequests<A::Policy>,
) -> Result<Generation<A::Payload>, PlanningError>;
```

`A: Target` ties the adapter to its policy and rendering-payload types (sections 5–6). The method borrows the registry and adapter, consumes requests, and builds private working state. The returned `Generation` owns retained plans and payloads. Unsupported requests appear in its report; invalid input or invariant failures return `PlanningError` (section 9). Rendering and I/O follow planning.

Inside the C frontend's build implementation after selecting v2 (internal pseudocode, not user `build.rs` code):

```rust
let requests = c_builder.into_requests(&source_model)?;
let registry = Registry::new(source_model);
let generation = registry.generate(&c_adapter, requests)?;
```

The frontend and registry independently use `prebindgen-flat` to inspect source items. The frontend interprets user declarations and validates their source references; the registry discovers required fields or helper arguments and plans their conversions. The registry supplies no separate source-inspection API to the frontend.

Request construction preserves all recorded frontend choices, including defaults, overrides, source mappings, helper signatures and ignore rules. Local helpers and declared Rust conversion operations are registered as typed source descriptions before planning. Unimplemented settings remain visible as unsupported requests. Naming closures may remain owned configuration objects; serialization is unnecessary.

## 3. Identifying requests, value positions and reusable conversions

### Where planning starts

To generate `normalize(stamp: Stamp) -> Stamp`, the registry needs an input conversion, the source call and an output conversion. The request to expose `normalize` is the starting point, called a **root**. The conversions required to implement that request are its **dependencies**. A request to expose a public type is also a root, even if no function uses that type.

```rust
struct OutputRequest {
    id: ElementId,        // This requested output, e.g. normalize at one Kotlin placement.
    source: SourceItemId, // The Rust function/type/constant or registered local helper.
    policy: PolicyId,     // Entry in BindingRequests.policies configuring this output.
    requirements: Vec<SemanticRequirement>, // Promises that must hold for this output.
}
```

`ElementId` identifies the requested output; `SourceItemId` identifies the source item behind it. Exposing one Rust function at two foreign placements gives two output identities. `SemanticRequirement` records promises such as implementing an interface or preserving an ownership/error-handling convention. Required helpers do not automatically become public exports.

### A value's position in an exported function

The registry needs to locate the parameter affected by a per-function override. A **site** is such a position: for example, parameter 0 of the requested `normalize` binding. `SiteId` identifies that position so the registry can apply its override and report problems there.

```rust
struct SiteId {
    owner: ElementId, // Requested function containing the position, e.g. normalize.
    path: SitePath,   // Param(0), Return, or a nested callback argument position.
}
```

### Fields, constructor arguments and enum variants

`Stamp` can be built from its `secs` and `nanos` fields or by calling `stamp_from_millis(millis: i64) -> Stamp`. Each choice has a separate `RelationId`. Constructor parameters need not match the fields in name, type or number: the registry converts `millis` and calls the helper; the helper computes the fields.

A **part** is a field or argument converted within that relationship. `Stamp.fields` (a descriptive label, not Rust syntax) has two parts; the constructor relationship has one, `millis`. `PartId` identifies which part a conversion rule applies to.

For an enum such as `enum Event { At(Stamp), Count(u32) }`, the variant is also needed to identify a part. An **arm** is one alternative, and `ArmId` identifies it: here, `At` or `Count`. Each variant has a field at position 0, but those fields belong to different arms. A declared choice between constructors can also use arm IDs. Ordinary struct fields and a single constructor have no alternatives, so their arm is `None`.

```rust
struct PartId {
    owner: RelationId,       // Relationship defining the part, e.g. Stamp.fields.
    arm: Option<ArmId>,      // Enum variant/declared alternative; None without alternatives.
    position: PartPosition,  // Field identity, helper argument index, or projector result.
}
```

For example, `(Stamp.fields, None, Field("secs"))` identifies a struct field; `(Event.variants, Some(At), Field(0))` identifies `At`'s payload. These are illustrative IDs. `owner` refers to the containing relationship, not Rust memory ownership. Function-specific overrides and diagnostics remain attached to `SiteId` positions.

### Finding an existing conversion plan

Conversion planning takes a model-validated `TypeRef` and a direction, represented by `Crossing`. Frontends and adapters supply source descriptions, not cache keys:

```rust
enum Direction {
    IntoRust,  // Produce the Rust value expected by a source function.
    OutOfRust, // Encode a Rust result or callback argument for foreign code.
}

struct Crossing {
    source: TypeRef,       // Model-validated type, including references/generics.
    direction: Direction, // Whether the value enters or leaves the Rust API.
}
```

A reusable conversion plan is a **node**. The registry finds nodes using a private `NodeKey`, derived internally from the accepted `Crossing`, selected relationship and effective policy. No frontend/adapter conversion-planning API accepts `TypeKey` or `NodeKey`, or a caller-supplied type/key pair.

```rust
// Private to the registry's conversion cache module; not a public request type.
struct NodeKey {
    source: TypeKey,       // Derived internally from crossing.source.key().
    direction: Direction, // Copied from that crossing.
    relation: RelationId, // Validated selected relationship.
    policy: PolicyId,     // Registered effective conversion settings.
}
```

The private cache operation accepts the validated crossing and selection, derives the key, and retains the same `TypeRef` in the plan. Key construction and cache mutation are private; plan descriptors expose typed readings. Callers cannot submit mismatched type/key pairs or request conversions using parsed keys.

`TypeRef` has no `Eq`/`Hash`; its existing `key()` returns `prebindgen_flat::TypeKey`, which supplies both. `key()` preserves references/mutability, wrappers, generic arguments, array extents and lifetime spelling. Flat normalizes parentheses and known equivalent paths, such as `std::vec::Vec<T>` and `Vec<T>`, without equating arbitrary aliases. `stripped_key()` removes outer `Box`/`Cow` wrappers for declaration lookup: `Box<Stamp>` finds the `Stamp` declaration. The conversion cache uses `key()` to retain those wrappers. Plans retain the typed readings needed for inspection and emission.

For example, two owned `Stamp` inputs with the same two-integer JNI representation and field construction can share a node. An object-input override changes the policy; a return conversion changes direction. `Stamp`, `&Stamp` and `Option<&Stamp>` remain distinct.

Model membership is part of the API contract. Extend `prebindgen-flat` with model-scoped type references or checked import operations that bind a reading to a specific `Flat` model. Flat owns source identity, lookup and validation; the registry accepts the resulting checked references. Private constructors prevent forged model identity, and derived child types preserve that context. A mismatched reference must be rejected before planning; matching key text alone is insufficient. Design this contract jointly across Flat and the registry rather than duplicating source validation in the registry.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId` identifies a retained plan, and registry-issued node references must be validated within their generation context. Function sites retain separate overrides and diagnostic paths. Policies containing closures share identity only when equivalence is established. Reports use deterministic source/configuration identities.

## 4. Describing source construction and decomposition

For `normalize(stamp: Stamp) -> Stamp`, the wrapper converts foreign input into a Rust `Stamp`, calls `normalize`, and converts its result for foreign code. Here, **value** means a runtime argument, result or field. Input **construction** can assemble `Stamp { secs, nanos }` from converted fields or call `stamp_from_millis` with one converted argument. Output **decomposition** can read fields or call an accessor. A constructor does not automatically provide an inverse accessor.

A **relation** is the registry's description of how to construct or read a Rust value for a conversion. The same function can be exported directly, selected as a constructor, or used to extract another value. Those are binding roles, so relation construction belongs in the common registry library. Flat supplies the checked source facts used to validate those roles.

### Flat provides neutral source views

The proposed Flat API returns structural views, without choosing conversion roles:

```rust
impl Flat {
    pub fn record(&self, ty: &TypeRef) -> Result<RecordView, ModelError>;
    pub fn function(&self, reference: &FunctionRef) -> Result<FunctionView, ModelError>;
}

// Internals private to Flat; public accessors provide read-only source facts.
pub struct RecordView {
    model: ModelIdentity, // Checked model association, retained with the source data.
    record: RecordRef,    // Record type, named/tuple/unit form and typed fields.
}
pub struct FunctionView {
    model: ModelIdentity,
    function: FunctionRef, // Checked signature: parameters and complete return type.
}
```

`RecordRef`/`FunctionRef` are model-issued references. Flat checks membership and source shape; failures return `ModelError`. Private constructors and retained immutable source data preserve validity. Frontend and registry share this API; required Flat extensions are in scope.

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
    Value(TypeRef), // Constructed type.
    Fallible { constructed: TypeRef, error: TypeRef },
}
pub struct ProjectionRelation {
    function: FunctionView, // Initially requires exactly one source argument.
    input: TypeRef,        // Derived exact argument type: T, &T, or &mut T.
    output: TypeRef,       // Derived complete return type, including wrappers.
}
```

All relation fields are private. Read-only accessors expose source types and arguments. A constructor's subject is derived from its result; a projector's subject/access requirements come from its input. Callers cannot pair an arbitrary subject with an unrelated operation. For `stamp_from_millis(i64) -> Stamp`, the constructor has one `i64` argument and produces `Stamp`, regardless of `Stamp`'s fields. For `stamp_parts(&Stamp) -> (i64, i64)`, projection requires a shared borrow and produces a tuple.

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

Registry-library registration accepts checked relations and issues opaque `RelationId`s; request import validates their model and table context. Frontends may construct these descriptions without running recursive conversion planning. The earlier label `Stamp.fields` denotes a registered `Relation::Record` backed by the `Stamp` record view.

The registry follows `Selection.relation` to the checked operation, obtains its fields or helper arguments, and recursively plans their conversions. Projection calls its helper once and processes the saved result through the conversion rules. The adapter describes foreign representations; the registry assembles the source-side instructions executed by the wrapper.

Selection precedes child traversal: an atomic opaque representation does not inspect unused private fields. Helper arguments need not resemble fields. Child types retain wrappers, references and lifetimes; cloning needs an explicit operation. Callback arguments reverse direction. Future roles follow the same checked-construction pattern and produce unsupported outcomes until implemented.

## 5. Describing target values and operations

The registry knows how source values are composed. It also needs a description of the values used on the target side and the operations that access them.

The **public surface** is the API foreign users see: C types/functions or Kotlin classes/methods. The **wire representation** is the set of values passed through the native calling interface, or **ABI** (application binary interface). A Kotlin `Stamp` object can have two integer arguments as its wire representation. A **carrier** is a value holding conversion data, either on that boundary or temporarily inside generated Rust code.

### Individual target operations

A **primitive** is one typed operation supplied by the target, such as converting a scalar, reading a JVM property, allocating a handle, or signaling an error. The registry schedules these operations within the complete conversion.

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
    Source(TypeRef),     // A checked Rust source type supplied by Flat.
    Carrier(WireTypeId), // A registered target/runtime type, described below.
}

enum Access {
    Owned,     // The operation may consume/move the value.
    Shared,    // It may read through a shared borrow.
    Exclusive, // It may modify through an exclusive borrow.
}
```

`WireTypeId` identifies a descriptor for a target value, such as a JNI integer,
object reference or environment, or an intermediate Rust carrier. The descriptor
also states whether the type may appear in an extern signature. An environment
wrapper used inside generated Rust need not itself be an ABI argument type.
`Access` describes the operation's use of its operand and must agree with the
operand's exact Rust type. It does not authorize cloning or replacing a move
with a borrow. The same access vocabulary is used by completed conversion plans
in section 7.

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
validated. Adding an enum variant named `Acquire` alone does not implement this
behavior.

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

#### Worked example: reading a JVM record

Suppose the selected input representation receives a JVM `Stamp` object. The
JNI adapter describes a property read with these facts:

| `PrimitiveSpec` field | Read `Stamp.secs` |
| --- | --- |
| `signature` | Operand 0: mutable access to the JNI environment carrier. Operand 1: shared access to the `Stamp` object carrier. One successful result: JNI's signed 64-bit integer carrier. |
| `failure` | A typed JNI access error in the runtime category. The operation has no successful integer result on failure. |
| `validity` | The integer result is independent of the environment and object after the read succeeds. Both operands must be valid during the call. |
| `resources` | No extra ownership obligation escapes this integer read. |
| `dependencies` | Any generated access helper needed by this implementation; empty when the operation directly calls an existing runtime API. |
| `implementation` | JNI getter name/signature and result-extraction metadata for `secs`; no Rust local names or nested converter body. |

Reading `nanos` uses another specification with that property's metadata. The
registry discovers both fields from the selected record relation and assembles:

```text
read secs from (environment, object)
 -> on success, convert the integer carrier to Rust i64
 -> read nanos from (environment, object)
 -> on success, convert the integer carrier to Rust i64
 -> construct Stamp { secs, nanos }
 -> call normalize once
```

Each failing operation branches to the wrapper's configured runtime-error
handling; later success operations are not executed on that path. Output
conversion follows the same registry composition rules using the target's
chosen output operations. Changing `Stamp` to contain another supported field
makes the registry plan another child conversion; the JNI adapter supplies the
new field's property operation through the same local interface.

For a C aggregate, the corresponding inputs come from C-struct member access.
The common struct-operation defaults can describe those reads without a
language-specific renderer for each field. JNI separate-argument input instead
obtains already available slots. All three use the same registry record
construction algorithm.


### Representation shape and composition protocol

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
    Terminal { codec: PrimitiveId }, // One whole-value conversion operation.
    Product(ProductOps),   // Project members and construct a target product.
    Optional(OptionalOps), // Detect/extract/inject presence or absence.
    Sequence(SequenceOps), // Read or append target sequence elements.
    Choice(ChoiceOps),     // Inspect/write a tag and its active payload.
    Callable(CallableOps), // Capture/invoke a target callable.
}
```

`WireTypeId` refers to a type descriptor that distinguishes a valid extern ABI type from a Rust-only intermediate carrier. A Rust tuple or JVM wrapper object must not appear in an extern signature merely because it can be described as a Rust type. `MemberLayout` names an aggregate member and its child layout. `LayoutId` and `PrimitiveId`, used below, refer to registered layout and operation descriptions.

A **slot** is one value in a multi-value representation. `SlotRole` states its meaning, independent of its generated name. `GuardId` refers to a condition such as “always,” “presence is true,” or “variant tag selects this arm.” Enclosing conditions also apply. Inactive slots can require valid wire defaults even though their source payload must not be read or constructed. When one layout is used for two function arguments, its slot identities are qualified by each use so their ABI positions remain separate.

`ProductOps` describes member projections and a target construction operation over already converted children. For a C struct these can be ordinary member reads and a struct literal. For separate JNI arguments they map children to slots. For object input they can be JVM-property-read primitives. The registry can provide standard tuple/struct operations as reusable defaults.

For sequences, variants and callbacks, adapters supply runtime operations; the registry supplies loops, branches and child calls. C aggregates and JNI slots/object operations reuse the source relationship. Layouts remain nested until flattening is needed.

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

## 6. How the registry asks a target for decisions

The target interface has four planning operations. Each answers a local question with a description. The registry owns recursion and calls the next planning operation when its inputs are ready.

```rust
trait Target {
    type Policy;  // Configuration choices explained in section 2.
    type Payload; // Owned operation/rendering descriptions explained in section 5.

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

`TargetSupport<Answer>` means a ready description, a specific unsupported reason, or a fatal planning error; its definition is in section 9. `BoundarySpec` describes native argument/result placement (section 8). `SurfaceSpec` describes a public foreign declaration and its dependencies (section 9).

The method inputs and results serve different stages:

| Method | Information available | Target's answer | Registry's next job |
| --- | --- | --- | --- |
| `select` | Exact source type/direction, local source facts and applicable conversion rules and policy | `RelationSelection`: chosen source relationship and declared child/stage choices | Inspect that operation's children and recursively resolve their conversions. |
| `represent` | `ResolvedShape`: source operation with model-derived child types; `ValueDescriptor`s: completed child layouts and contracts | Representation layout and target operations | Compose the complete value conversion. |
| `boundary` | `SiteDescriptor`: call signature/roles; `ResolvedValues`: its completed value descriptions | Argument placement, result delivery and error actions | Assemble and validate the complete native wrapper. |
| `surface` | `SurfaceRequest`: requested name, placement, policy and promises; required value descriptions | Public declaration description and requirements | Check dependencies before deciding whether to emit it. |

These views are read-only. The adapter can inspect direct child descriptors but cannot invoke the registry's recursive compiler or modify the registry's plan tables. For an explicit conversion rule, `select` must honor that selection or report why it is unsupported. Common relationships and standard representations should have table-based helpers/defaults so each adapter supplies as little code as practical.

Source-conversion dependencies appear in selected relationships. Target operations list the generated helpers they need. When a target method needs an additional conversion, the method returns an explicit dependency request for the registry to resolve. Rendering operates on the completed result and cannot discover new conversions or support gaps.

Descriptions returned by a target can contain new primitive, layout, helper, or policy definitions with references local to that description. The registry validates and registers the definitions and assigns its own table IDs. Existing descriptors can reference IDs the registry already supplied. The target does not allocate entries in registry-owned tables itself.

The language-provided final rendering interface reads immutable plans and retained payloads. The common emission machinery supplies allocated operand names and the necessary rendering context; the rendering interface exposes no planning entry point. The original builder is translated before generation; subsequent decisions use requests and policies.

## 7. The conversion plans the registry builds

For each supported conversion node, the registry records the selected source operation, target representation, generated instructions, and the conditions under which the result is usable.

```rust
struct ValuePlan<Payload> {
    id: NodeId,                    // Reference used by callers of this conversion.
    crossing: Crossing,            // Exact source type and direction being converted.
    relation: ResolvedRelation,    // Source operation with validated child/stage node IDs.
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

`ResolvedRelation` is the chosen relationship after its source references and child conversions are resolved. `ValueType` can describe a source type or a carrier layout containing zero, one or several values. `Validity` composes the individual primitives' validity rules: for example, a produced reference remains tied to a particular temporary. `FailureSet` collects possible error categories/types; the function boundary decides their eventual handling.

`ConversionBodyId` points to structured instructions for locals, field access, variant matching, source construction/calls, primitive applications, conditions, and later loops or callback invocation. The common writer renders these instructions as Rust. It allocates temporary names centrally from identities.

The registry generates all child calls and source traversal. A projector binds its intermediate result once. Optional/variant branches convert only active children. The dependency list is derived from these instructions and the resolved relationship; it is a convenience index maintained by the registry.

Access follows the exact source type and operation. For example, generating `&Stamp` input may require constructing an owned temporary and borrowing it for the duration of the source call. Supporting the two scalar fields alone does not implement that borrow. The conversion contract must preserve the temporary's validity through its uses.

For initial scalar/record support, ordinary owned Rust temporaries can rely on Rust destruction at scope exit. Handles and callbacks need implemented acquisition, transfer and cleanup semantics before they are supported. As those features are added, the registry will schedule resource scopes and cleanup on success and failure paths; adapters provide the actual retain/free/runtime operations.

## 8. Assembling an exported function

A complete binding function coordinates several value plans. It converts inputs, calls the source once, converts the selected result, handles failures, and finishes resource scopes.

**Delivery** specifies where converted values go. A conversion producing a record can be reused whether the enclosing function returns that record, writes it through output parameters, or passes it to a declared callback. The conversion itself need not contain a second version for each destination.

```rust
struct BoundarySpec<Payload> {
    abi: AbiSpec<Payload>,        // Calling convention, symbol and target signature requirements.
    inputs: Vec<InputPlacement>, // How native arguments feed logical input conversions.
    output: OutputPlacement,     // Destinations for encoded success/error values.
    failures: FailureRoutes<Payload>, // Terminal actions for each category of failure.
}

enum OutputPlacement {
    Void, // This path delivers no value.
    Return(ValueMapping), // Deliver converted values through the native return.
    OutParameters(Vec<ValueMapping>), // Write converted values to caller-provided locations.
    Branches {
        ok: Box<OutputPlacement>, // Delivery for a source Result::Ok value.
        err: Box<OutputPlacement>,// Delivery for a source Result::Err value.
    },
    Invoke {
        sink: SinkId,         // Resolved callback/builder/error-handler destination.
        values: ValueMapping, // Converted values mapped to its arguments.
    },
}

struct FunctionPlan<Payload> {
    source: CalleeId,          // Source function, local helper or modeled constant getter.
    inputs: Vec<NodeId>,      // Input conversions in source parameter order.
    output: FunctionOutput,   // No result, one conversion, or success/error conversions.
    boundary: BoundarySpec<Payload>, // Validated native signature and delivery decisions.
    body: FunctionBodyId,     // Registry-owned instructions for the whole wrapper.
    artifacts: Vec<ArtifactId>, // Derived generated prerequisites for the wrapper.
}
```

`AbiSpec` describes the native interface, including target calling conventions and explicit environment operands such as a JNI environment. `InputPlacement` maps native argument values to a logical conversion input. `ValueMapping` maps converted values/slots to a return, output location, or invocation argument. These are transport descriptions; they do not repeat source decomposition.

`SinkId` identifies a declared destination and signature, not the runtime callback pointer itself. `CalleeId` identifies the source operation being wrapped. `FunctionOutput` identifies the conversions for the wrapped source operation's result. `FunctionBodyId` refers to the complete structured wrapper instructions assembled by the registry.

There are three relevant error categories: a domain error returned by the source API, a binding/conversion error, and a runtime error such as a JNI failure. `FailureRoutes` selects their terminal actions: return a configured status, throw, call a declared handler, or abort according to the binding's policy.

For a source `Result`, `OutputPlacement::Branches` maps the error value to its configured destination, while the domain failure route specifies how that path terminates. Both describe one consistent boundary policy. Conversion failures can also happen before the source call or while encoding its result; those use their binding/runtime routes. Failure while encoding a domain error must itself have a defined route.

The registry constructs this flow:

```text
validate and convert inputs
 -> call the source function once
 -> choose the source success/error path
 -> convert the selected value
 -> deliver through the configured destination
 -> finish resource scopes
```

The registry allocates synthetic parameters required by the boundary, validates type/value mappings and ensures that partial results and errors obey the declared contract. Unit results explicitly require no result payload.

C and JNI retain configured calling conventions. Unsupported result destinations skip the function; changing its ABI is not a substitute for support.

## 9. Unsupported requests and public API dependencies

V2 accepts more input than it initially knows how to generate. Unsupported functionality is a normal, explicit planning result. Invalid configuration and generator defects are errors that fail generation.

```rust
// Answer returned by a target operation; the registry owns the final accounting.
enum TargetAttempt<Answer> {
    Ready(Answer),                 // Complete description usable by the registry.
    Unsupported(UnsupportedReason), // Valid request requiring an unimplemented capability.
}
type TargetSupport<Answer> = Result<TargetAttempt<Answer>, PlanningError>;

enum PlanningError {
    InvalidInput(Diagnostic),      // Malformed or contradictory configuration.
    InternalInvariant(Diagnostic),// A generator defect or violated internal contract.
}

struct UnsupportedReason {
    capability: CapabilityCode, // Stable identifier such as unsupported.handle.borrowed_input.
    explanation: String,        // Human-readable explanation of the missing capability.
}

struct Cause {
    reason: UnsupportedReason,  // The missing capability and explanation.
    origin: SourceLocation,     // Source/configuration location for the diagnostic.
    at: FailureLocation,        // A root, boundary site, or reusable relationship part.
    dependencies: Vec<CauseId>, // Underlying causes when failure is propagated.
}

enum ElementOutcome {
    Emitted { artifacts: Vec<ArtifactId> }, // Complete retained output for the request.
    Skipped { causes: Vec<CauseId> },       // Requested output omitted with explicit reasons.
    Ignored,    // Explicitly excluded by the consumer.
    Unselected, // Present in source but not selected by this configuration.
}
```

`Diagnostic` contains an error message and its relevant location. `CapabilityCode` is a stable reason code for reports and tests. `FailureLocation` identifies the root/site/part being planned; `SourceLocation` points the reader to the source or configuration entry. The target returns an unsupported description, and the registry attaches planning context and stores it as a `Cause`. `CauseId` is a reference into that registry-owned table. The registry uses the same cause representation for capabilities missing in its own algorithms.

The distinction between an operation result and an output outcome matters: a scalar conversion can be ready while its enclosing function is skipped because another parameter is unsupported. Several skipped outputs can share one underlying cause, while reports retain each output's dependency path.

Planning itself determines support. There is no separate recursive `supports(type)` pass that could disagree with generation. The registry tracks each attempted conversion as unseen, currently being resolved, ready, or unsupported. Encountering a currently active conversion can reveal an expansion cycle. Cycle detection follows selected relationships: an atomic handle can stop expansion of a recursive source type. A recursive conversion not yet implemented in v2 is reported as unsupported; a contradictory conversion rule remains invalid input. Panics and I/O failures are not converted into skip reasons.

### Dependencies of public declarations

Conversion dependencies are not the only conditions for valid output. A public method can require its owning class, a parameter type, and an interface member promised by configuration.

```rust
struct SurfaceSpec<Payload> {
    element: ElementId,          // Public type/function/member this description implements.
    requires: Vec<Requirement>, // Required public types, conversions, interfaces or helpers.
    members: Vec<ElementId>,     // Associated declarations, such as a class's methods.
    payload: Payload,            // Chosen target name/package/modifiers and rendering metadata.
}
```

`Requirement` is a typed reference to a dependency that must succeed. It is the resolved obligation derived from an output request's semantic promises or actual usage. `members` describes association; a member that is essential to a promised interface must also be a requirement.

The registry checks the complete set of direct and indirect requirements before deciding to emit an output:

- An unsupported field skips the entire record and callers that require that record representation.
- An unsupported optional method can be omitted independently; an interface-required method can prevent its owner from being emitted.
- An unsupported owner representation prevents methods requiring that owner.
- Unrelated free functions in a package can still be generated.
- A shared helper is retained if any emitted output needs it.
- An unimplemented semantic setting blocks the affected promise; it is not silently discarded.

A `SurfaceSpec` describes one public declaration and its requirements; providing this description does not require a foreign writer. For C, the public declaration is expressed through generated Rust types/functions that `cbindgen` translates into a header. For Kotlin, the JNI implementation's writer renders the public declaration directly. An artifact is an emission unit: that declaration can require a foreign wrapper, native extern, converter helpers and runtime helpers. The registry keeps candidate artifacts during planning and publishes only those needed by complete supported outputs.

Public types referring to each other do not necessarily require an infinitely recursive conversion. Conversion-expansion cycles and public-declaration dependencies therefore need separate checks. Public dependencies may require repeated readiness evaluation until the retained set stops changing. A new public requirement discovered after value planning must still propagate before output is finalized.

## 10. Registry state, execution order and final output

During generation, `Registry` provides access to the `Flat` source model. `GenerationRun` holds the temporary plans, dependencies and diagnostics being assembled for the configured frontend. The registry returns the retained results as `Generation`. The following structures separate source information from mutable planning state:

```rust
struct Registry {
    model: Flat, // Existing source model used by generate(); source facts stay in Flat.
}

struct GenerationRun<'a, T: Target> {
    registry: &'a Registry, // Source facts used throughout this generate call.
    target: &'a T,         // Read-only target decision/operation provider.
    requests: BindingRequests<T::Policy>, // Complete translated requests and configuration.
    nodes: NodeArena<T::Payload>,       // Conversion attempts and completed value plans.
    functions: FunctionArena<T::Payload>, // Candidate complete native wrapper plans.
    surfaces: SurfaceArena<T::Payload>, // Candidate public declaration descriptions.
    artifacts: ArtifactArena<T::Payload>, // Candidate generated units and dependencies.
    outcomes: OutcomeTable, // Per-element decisions and shared unsupported causes.
}
```

`T` implements `Target`. Each `*Arena` is a registry-owned table addressed by typed IDs. `OutcomeTable` holds classifications and diagnostic causes. Primitive/layout/body tables are omitted here. The registry updates these tables; adapters receive immutable descriptions.

The pipeline is:

```text
existing source captures + C/JNI frontend configured through its Rust API
 -> user calls the frontend build method
 -> frontend selects v1 or v2
 -> v2 frontend creates BindingRequests internally and calls Registry::generate
 -> registry validates/imports source references, policies and requests
 -> for each requested value: select source relationship
 -> registry resolves the selected source operation's children
 -> target describes the requested value's representation
 -> registry composes value instructions and contracts
 -> target describes native delivery and public declarations
 -> registry assembles functions and resolves all required dependencies
 -> registry propagates skips and retains complete supported output
 -> freeze Generation
 -> common Rust writer emits native wrappers and supporting Rust types
 -> C: cbindgen derives headers from generated Rust
    JNI: optional foreign-writer interface is implemented by the Kotlin writer
 -> publish generated artifacts, report and test-selection manifest
```

Selection, child resolution and representation happen together for each node. A complete conversion table is not required before relationship choices are known. Boundary/public-declaration failures can remove candidate outputs before the result is frozen.

```rust
struct Generation<Payload> {
    values: FrozenArena<ValuePlan<Payload>>, // Retained reusable conversions.
    functions: FrozenArena<FunctionPlan<Payload>>, // Retained complete native functions.
    surface: FrozenArena<SurfaceSpec<Payload>>, // Complete retained public declarations.
    artifacts: OrderedArtifacts<Payload>, // Generated units with a validated emission order.
    report: GenerationReport, // All requested/source outcomes and their diagnostic paths.
}
```

Freezing retains all referenced tables (bodies, primitives, layouts, helpers) and the required `Flat` source-emission data, directly or through shared ownership. Rendering uses these retained records and language-provided rendering code. The original registry, borrowed adapter and temporary working tables need not remain alive.

**Frozen** means planning is complete and the records are immutable. `FrozenArena` retains ID-based lookup without insertion or replanning. `OrderedArtifacts` provides an emission order appropriate to generated dependencies, including any required forward declarations. `GenerationReport` records emitted/skipped/ignored/unselected outcomes, pipeline identity, capability causes, native/foreign artifacts and symbol identities. Helper-only items remain distinguishable from explicitly requested exports.

The common Rust writer reads `Generation`; JNI's optional writer reads the same result for Kotlin. C passes generated Rust to `cbindgen` for headers. Writers cannot add dependencies or change support decisions. Publish after output generation succeeds. The report also selects existing test sections, ensuring tests match the emitted API.

## 11. Integration and first implementation steps

### Switching existing examples

Keep v1 and v2 as parallel engines behind the existing frontend. Independent v2 registry and C/JNI implementation crates can share binding-configuration data modules and the source model. They must not depend on v1 conversion plans, recursive generators or emitters. Engine selection happens before v1 generation starts.

Engine selection and separate output paths already have an initial implementation in [#721](https://github.com/milyin/prebindgen/pull/721) and [#722](https://github.com/milyin/prebindgen/pull/722). The current V2 scaffold reports unsupported declarations; the conversion architecture in this document is the next implementation work.

The switching contract from [#719](https://github.com/milyin/prebindgen/issues/719) is:

- Cargo feature `v2` makes the optional engine available. It does not select it by itself.
- `.build()` reads `PREBINDGEN_PIPELINE=v1|v2`; an unset variable selects v1, including under `--all-features`.
- `build_with(Pipeline::...)` selects an engine explicitly and bypasses the environment.
- Selecting v2 without its feature produces a clear error.
- Example scripts forward the selection and enable the feature as needed. Build scripts track environment changes so Cargo regenerates the selected output.

For example:

```sh
PREBINDGEN_PIPELINE=v2 cargo build -p covertest-kotlin --features v2
```

Both engines receive the complete current configuration through their respective paths. V2 generates its supported subset without falling back to v1 for individual items. Output paths and publication must support v1 → v2 → v1 without manual cleanup or stale generated declarations.

Existing Kotlin tests can directly refer to classes absent from v2 output. Select complete supported test sections before Kotlin compilation; runtime guards cannot hide missing symbols from the compiler. C tests need equivalent selection. The manifest must still require meaningful supported tests to execute, so skipping everything cannot pass a milestone.

### First executable increment

The initial implementation should demonstrate the architecture with both existing C and Kotlin examples:

1. Construct `BindingRequests` from all recorded frontend choices; implement identities, diagnostic causes and request accounting. Preserve unsupported configuration entries and settings from the start.
2. Implement one scalar function through target descriptors, registry conversion/function plans, frozen output and the normal output path: common Rust emission followed by C header generation or Kotlin emission. Execute it through both language boundaries.
3. Add named-field records with registry-owned field traversal, construction and decomposition. Demonstrate a C aggregate and a JNI representation using the same source relationship algorithm.
4. Add plain optional representations and the temporary/borrow operations required by selected existing examples. Test present/absent behavior and temporary lifetime requirements.
5. Verify dependency-based skipping, existing test-section selection, and repeated switching between engines. Each preceding executable increment also produces its report and complete artifacts.

Do not implement every proposed enum variant before the scalar case runs. Constructor/projector conversions, `Result`, resource-bearing handles, sequences and callbacks can be added incrementally through the same descriptions and registry algorithms. Full inputs remain accepted throughout that work.

### Cases that expose architectural mistakes

| Case | Expected behavior |
| --- | --- |
| `Stamp` under C aggregate, JNI separate arguments, and eventually JVM-object input | Source field discovery and Rust construction stay in the registry; target operations differ. An unimplemented requested representation is reported. |
| Two functions using the same `Stamp` representation | Share the conversion while retaining different parameter names and diagnostic paths. |
| A whole opaque representation of a type with unsupported private fields | Do not traverse the unused fields. |
| Accessor/value-form helper returning a compound value | Call it once, keep its exact result type, and let the registry process its selected children. |
| Existing `large_flat_input_sum(&ObjectBoundary64)` | Support requires an owned temporary and call-scoped borrow, beyond owned record conversion. Preserve the signature; skip with a borrow reason until implemented. |
| The existing JVM-object-input sibling of that function | Its independently selected representation may have a different support outcome. |
| Source `Result` with configured handler/builder | Preserve both branches and existing delivery conventions; skip if a required handler or destination is unimplemented. |
| Nested optional values | Preserve distinct states and never decode inactive payloads. |
| Unsupported required field or promised interface member | Propagate to the complete dependent public contract, while retaining unrelated output. |

A capability involving JNI is complete only when the existing Kotlin covertest exercises both its generated native boundary and Kotlin API. Unit tests for planning are useful, but they do not establish runtime ownership, JNI, or foreign-interface correctness.

## 12. Acceptance and feasibility evidence

Acceptance criteria:

- [ ] The proposal's boundaries are exercised by scalar and record bindings in existing C/JNI examples.
- [ ] Users configure the existing language frontends; frontend internals construct `BindingRequests` for the registry. Target policies have explicit local interpretation APIs.
- [ ] The registry owns recursive conversion, source calls, dependency resolution, control flow and Rust wrapper assembly.
- [ ] Flat and registry jointly enforce model-scoped references and private key derivation; update Flat APIs as needed.
- [ ] Targets retain their representation, runtime-operation and delivery choices without implementing another recursive source planner.
- [ ] Complete unsupported inputs produce actionable per-element outcomes; malformed configuration and generator defects fail generation.
- [ ] One immutable generation result supplies Rust output, optional foreign-writer output, reports and test selection; C headers are derived from the retained Rust output by `cbindgen`.
- [ ] Emitted output preserves logical behavior and declared interfaces without a byte-identity requirement.
- [ ] New nested combinations reuse the registry's composition algorithm instead of requiring a new per-language wrapper implementation.
- [ ] Remaining unsupported capabilities and any API refinements discovered during implementation are documented.

Earlier feasibility work inspected registry relationships/composition and Flat emission at [e046546](https://github.com/milyin/prebindgen/tree/e04654679e7aa0e7c7d4b9bb4a1268f9943926ec). Type-key inspection at [a429662](https://github.com/milyin/prebindgen/tree/a429662bb450408f401ad8f52ff753c5f5a179d4/prebindgen-flat/src/flat) confirmed `TypeRef::key()` and its equality/hash contract. `cargo test -p prebindgen-flat --lib`: 92 passed. These checks covered existing Flat behavior, not the proposed V2 contracts.

Future resource, recursive and runtime capabilities require implementations and tests; until then, affected requests remain unsupported. Background: [#689](https://github.com/milyin/prebindgen/issues/689) / [#701](https://github.com/milyin/prebindgen/issues/701); earlier plans: [#713](https://github.com/milyin/prebindgen/issues/713) / [#717](https://github.com/milyin/prebindgen/issues/717).
