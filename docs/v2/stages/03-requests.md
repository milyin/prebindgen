<!-- spec: {"kind": "stage", "stage": "03-requests"} -->

# Record binding requests

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

Flat says what the source contains. This stage says what the user wants out of
it: which items become part of the foreign API, under which names, and with which
representation. The result is the registry's input — a request set, not yet a
plan.

**Input.** The published source model and its views, plus the choices the user
recorded through a language frontend: builders and macros called from `build.rs`,
naming hooks, per-argument overrides, ignore rules.

**Owner.** The language frontend, which is the public Rust API of `prebindgen-c`
or `prebindgen-jni`. Users configure the frontend; the frontend translates its
recorded configuration into `BindingRequests` internally. Users never construct
requests themselves.

**Output.** `BindingRequests`: output requests naming what to expose, conversion
rules selecting how particular values are built and represented, a policy table
holding the target-specific choices, and the entries the frontend could not
translate or was told to ignore. It also fixes the identities the rest of the
pipeline reports against — the requested element, the site inside it, the part of
a source relationship.

**Failure.** A setting the frontend cannot translate becomes an
`UnsupportedRequest`, carried forward so the report can name it. A configuration
that is malformed or contradictory is invalid input and fails generation.
Recording a request never asserts that the registry can generate it.

## Purpose and scope

Build a second generation pipeline around a common **registry**: an engine that takes the Rust source model and the consumer's choices, determines the required conversions, and assembles instructions for generating bindings. Language implementations describe how foreign code represents Rust values and provide the operations specific to their runtime. The registry combines those descriptions into complete conversions and exported functions using algorithms shared by C, JNI/Kotlin, and future languages.

V2 accepts the existing examples' complete Rust inputs and preserves their existing language-frontend APIs. Users configure a C or JNI frontend through Rust builders and macros, typically called from `build.rs`. The choices recorded by those calls—items to expose, names, representations and overrides—are the **binding configuration**. V2 initially generates only supported elements and reports why the others were skipped. The resulting Rust wrappers, C headers and Kotlin sources are the **generated bindings**. For an emitted element, correctness means preserving its declared behavior, ownership, error handling, and public/native interface. Generated source does not have to be byte-identical to v1.

[#719](https://github.com/milyin/prebindgen/issues/719) covers introducing v2 and switching existing examples. The types below are proposed schematic contracts; future capabilities are extension points, not implemented features.

## What the registry does

The registry receives the [source model](02-flat.md) and requests to expose particular types, functions, and constants. For each request, it builds a **plan**: structured data describing the required conversions, calls and their execution order. The common Rust writer renders the native wrappers and supporting Rust types. The C build then uses `cbindgen` to derive C headers from that Rust output; the JNI implementation renders Kotlin declarations from the completed plans. Planning happens in the generator; the generated operations execute later when the bindings are used.

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

## Binding requests and target policy

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

This choice does not list `Stamp`'s fields or explain how to construct it. The registry obtains those facts through a **source relationship**: a description of how a Rust value is constructed or read, such as using fields or a helper (the `RelationDefinition` in [source construction and decomposition](04-values.md#describing-source-construction-and-decomposition)). The adapter interprets the policy when describing the target representation and its property/argument operations.

Three concepts stay separate throughout the design:

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
    outputs: Vec<OutputRequest>,    // Explicit output requests; defined below.
    conversion_rules: ConversionRules, // Source-operation selections and applicability.
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

`A: Target` ties the adapter to its [policy and rendering-payload types](04-values.md#how-the-registry-asks-a-target-for-decisions). The method borrows the registry and adapter, consumes requests, and builds private working state. The returned `Generation` owns retained plans and payloads. Unsupported requests appear in [its report](06-retain.md#unsupported-requests-and-public-api-dependencies); invalid input or invariant failures return `PlanningError`. Rendering and I/O follow planning.

Inside the C frontend's build implementation after selecting v2 (internal pseudocode, not user `build.rs` code):

```rust
let requests = c_builder.into_requests(&source_model)?;
let registry = Registry::new(source_model);
let generation = registry.generate(&c_adapter, requests)?;
```

The frontend and registry independently use `prebindgen-flat` to inspect source items. The frontend interprets user declarations and validates their source references; the registry discovers required fields or helper arguments and plans their conversions. The registry supplies no separate source-inspection API to the frontend.

Request construction preserves all recorded frontend choices, including defaults, overrides, source mappings, helper signatures and ignore rules. Local helpers and declared Rust conversion operations are registered as typed source descriptions before planning. Unimplemented settings remain visible as unsupported requests. Naming closures may remain owned configuration objects; serialization is unnecessary.

## Identifying requests, value positions and reusable conversions

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

Conversion planning takes a `TypeView` and a direction, represented by `Crossing`. A `TypeView` is a read-only handle retaining an exact Rust type reading and the immutable Flat model in which it is interpreted; [type readings and type views](02-flat.md#type-readings-and-type-views) define its lookup and navigation API. Frontends and adapters supply source descriptions, not cache keys:

```rust
enum Direction {
    IntoRust,  // Produce the Rust value expected by a source function.
    OutOfRust, // Encode a Rust result or callback argument for foreign code.
}

struct Crossing {
    source: TypeView,      // Exact type and retained Flat snapshot, including wrappers.
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

The private cache operation accepts the validated crossing and selection, derives the key, and retains the same `TypeView` in the plan. Key construction and cache mutation are private; plan descriptors expose typed readings. Callers cannot submit mismatched type/key pairs or request conversions using parsed keys.

The existing structural reading `TypeRef` has no `Eq`/`Hash`; its `key()` returns `prebindgen_flat::TypeKey`, which supplies both. `key()` preserves references/mutability, wrappers, generic arguments, array extents and lifetime spelling. Flat normalizes parentheses and known equivalent paths, such as `std::vec::Vec<T>` and `Vec<T>`, without equating arbitrary aliases. `stripped_key()` removes outer `Box`/`Cow` wrappers for declaration lookup: `Box<Stamp>` finds the `Stamp` declaration. The proposed `TypeView::key()` delegates to its retained reading. The conversion cache uses that key to retain wrappers. Plans retain the view for model-aware inspection and emission; key text cannot recreate a view.

For example, two owned `Stamp` inputs with the same two-integer JNI representation and field construction can share a node. An object-input override changes the policy; a return conversion changes direction. `Stamp`, `&Stamp` and `Option<&Stamp>` remain distinct.

Model membership follows the [snapshot contract](02-flat.md#private-storage-and-model-consistency). Flat publishes immutable source data after helper registration; its views preserve that snapshot through field and parameter navigation. The registry checks incoming views against its own model before planning. Flat owns these checks and private view construction. A valid view from another snapshot is rejected even when its key text matches. The registry accepts no detached reading or independently supplied model/type pair as a substitute for a view.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId` identifies a retained plan, and registry-issued node references must be validated within their generation context. Function sites retain separate overrides and diagnostic paths. Policies containing closures share identity only when equivalence is established. Reports use deterministic source/configuration identities.

## Elements at this stage

- [Function taking an owned record][fn_requests] · [C][fn_requests_c] · [Kotlin/JNI][fn_requests_jni]
- [Record with scalar fields][struct_requests] · [C][struct_requests_c] · [Kotlin/JNI][struct_requests_jni]

---

Previous: [Build and inspect the source model](02-flat.md) · Next: [Plan value conversions](04-values.md)

[fn_requests]: ../examples/fn/03-requests.md
[fn_requests_c]: ../examples/fn/03-requests.c.md
[fn_requests_jni]: ../examples/fn/03-requests.jni.md
[struct_requests]: ../examples/struct/03-requests.md
[struct_requests_c]: ../examples/struct/03-requests.c.md
[struct_requests_jni]: ../examples/struct/03-requests.jni.md
