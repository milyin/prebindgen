<!-- spec: {"kind": "stage", "stage": "03-requests"} -->

# Record binding requests

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
pub struct Stamp { pub secs: i64, pub nanos: i64 }

pub fn stamp_sum(stamp: Stamp) -> i64;
```

The source model says what the crate contains. Nothing so far says what should
come out of it. That is what a user configures, and where the two targets first
part ways.

A binding crate is an ordinary Rust crate whose build script configures a
**language frontend** — the public Rust API of `prebindgen-c` or
`prebindgen-jni` — and asks it to generate. Exposing these two items through C is two declarations:

```rust
// build.rs of the C binding crate (schematic)
CbindgenBuilder::new()
    .data_struct("Stamp")     // Stamp crosses as a C struct passed by value
    .function("stamp_sum")    // and this function becomes a C entry point
    .build();
```

The generated C type and function keep the names the source used, `Stamp` and
`stamp_sum`, because that is the default: a foreign name is the source name
unless something changes it. A frontend offers naming hooks for the cases where
that is not what you want — a crate-wide prefix, or a symbol that would collide
with something already in the C namespace — and those hooks are configured here,
in the same builder, not anywhere downstream.

Through Kotlin it is the same two items with different answers — `Stamp` becomes
a Kotlin class in a package, and the function becomes a method on a Kotlin
object, reached through the Java Native Interface (JNI), the mechanism by which
JVM code calls native functions:

```rust
// build.rs of the JNI binding crate (schematic)
JniGen::builder()
    .package(package!("example").data_class(data_class!(Stamp)))
    .function("stamp_sum", placement!("example.Bindings.sum"))
    // What a native method does when a JVM call inside it fails:
    .runtime_errors(RuntimeErrors::PreservePendingElseThrow("java/lang/RuntimeException"))
    .build();
```

Kotlin cannot fall back on the source name the way C does, because a Kotlin
declaration needs somewhere to live: a package for the class, an object and a
method name for the function. That is what `package!` and `placement!` supply,
and it is why the JNI configuration says more than the C one about names. The
macros take Rust paths rather than strings, so `data_class!(Stamp)` fails to
compile if `Stamp` is not in scope. The `runtime_errors` call is the error
convention, and it is worth noticing that this is configuration rather than a
writer default: what
[the native boundary](05-boundary.md) does when a JVM property read fails is
decided here, and the wrapper it generates is only as good as this answer.

Each such call records a choice. Together they are the **binding
configuration**, and `.build()` is where the frontend turns it into
`BindingRequests` — the input the registry actually consumes. Users never write
that structure; frontends do, which is why the two builders above can be as
different as their languages while everything after this stage is shared.

A request set separates two kinds of statement. An **output request** names one
element to expose: this function, that type, at this foreign placement. A
**target policy** is the bag of target-specific choices that applies to it —
`Stamp` as a by-value C aggregate or as a JVM object whose properties are read,
this exported symbol, that error convention. Policy is data the target itself
interprets later; the registry only carries it and hands it back.

This stage also fixes the names by which everything is addressed afterwards. An
`ElementId` is one requested output — exposing the same Rust function at two
Kotlin placements makes two of them, with separate outcomes. A **site** is a
position inside such an element: parameter 0 of the exported `stamp_sum`, or its
return. A **part** is a position inside a source value: the `secs` field of
`Stamp`, or the single argument of a `stamp_from_millis` constructor. Overrides
attach to sites and parts, and so do diagnostics, which is why a skipped binding
can later say *which* parameter of *which* exported function was the problem.

Choices at different levels overlap, so they are looked up most specific first:
a choice recorded for this parameter of this function wins over one recorded for
this field of this record, which wins over the default for the type. The result
for one particular conversion is its **effective policy**, and since two
conversions of the same type under different effective policies are different
conversions, that lookup is not a detail — it decides what can be shared.

Recording a request claims nothing about feasibility; whether a well-formed
request can actually be generated is not known until the next stage tries.

## What V2 has to accept

V2 preserves the existing frontend APIs: the same builders and macros, the same
recorded choices, over the same captured Rust inputs. What it changes is the
engine behind `.build()`, which is why a request set can be built from a
configuration written for V1.
[#719](https://github.com/milyin/prebindgen/issues/719) covers introducing it and
switching the existing examples over. The types sketched below are schematic
contracts; capabilities not yet implemented are extension points, not features.

## What the registry does

The registry receives the [source model](02-flat.md) and requests to expose particular types, functions, and constants. For each request, it builds a **plan**: structured data describing the required conversions, calls and their execution order. The common Rust writer renders the native wrappers and supporting Rust types. The C build then uses `cbindgen` to derive C headers from that Rust output; the JNI implementation renders Kotlin declarations from the completed plans. Planning happens in the generator; the generated operations execute later when the bindings are used.

Take the record above and a second function over it — one that also
*returns* a `Stamp`, so that both directions are visible at once:

```rust
fn normalize(stamp: Stamp) -> Stamp;
```

Generating a binding for it requires several decisions and operations:

1. Choose how a foreign caller represents a `Stamp`.
2. Obtain and convert its `secs` and `nanos` values.
3. Construct the Rust `Stamp` and call `normalize`.
4. Read and convert the returned fields.
5. Return the result through the chosen foreign interface.

A C binding might represent `Stamp` as a C struct. A JNI binding might accept two native integer arguments produced by a Kotlin wrapper, or receive a JVM object whose properties must be read. Those representations need different target operations. The source-side work of discovering two fields, converting them, constructing `Stamp`, invoking the source function, and processing its result is common.

**The registry owns that common work.** When a record gains another nested record field, the shared recursive registry algorithm should process it using the representations supplied by the target. Each language should not need another implementation of record traversal or wrapper assembly.

A language implementation also owns its **foreign writer**: the component that renders the target language's own declarations from the completed plans. JNI's writes Kotlin. C's is the exception — [emission](07-emit.md) explains why it delegates to `cbindgen` instead. The registry library provides the **common Rust writer**, which emits native Rust wrappers and supporting Rust types for both targets.

The source crate and Flat have finished their work by the time requests exist.
These are the roles that act from here on — roles, not crates: the frontend and
the target adapter are the two faces of one language adapter crate, and the
common Rust writer belongs to the engine.

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
<td>Foreign writer</td>
<td>Render the target language's own declarations from the completed plans.</td>
<td>Delegated: the build invokes the external <code>cbindgen</code> tool on the generated Rust, which is where C headers come from.</td>
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

The C policy for the same value is a different shape entirely, because the
choices are different — there is no environment, no object, and no property to
read:

```rust
// Illustrative choices, again not a replacement for the existing builder API.
enum CRecordShape {
    Aggregate,   // A repr(C) struct, passed and returned by value.
    OpaquePtr,   // A pointer to a Rust-owned value, with a typed drop.
}

struct CValuePolicy {
    shape: CRecordShape, // How this record crosses the C boundary.
    c_name: String,      // Its name in the generated header.
}
```

Neither policy type is known to the registry: `Policy` is a generic parameter,
and each frontend fills it with whatever its own adapter will later have to
interpret. That is what lets one engine serve two languages whose choices have
nothing in common.

Neither choice lists `Stamp`'s fields or explains how to construct it. The registry obtains those facts through a **relation**: a description of how a Rust value is constructed or read, such as using its fields or calling a helper. Relations are the subject of [source construction and decomposition](04-values.md#describing-source-construction-and-decomposition); where the contrast with the target side matters, the chapters call one a *source relation*. The adapter interprets the policy when describing the target representation and its property/argument operations.

Three concepts stay separate throughout the design:

| Concept | Question it answers | `Stamp` example |
| --- | --- | --- |
| Relation | How can the Rust value be constructed or read? | Construct/read its `secs` and `nanos` fields. |
| Target representation | What values carry it, and how are those values accessed? | One C struct, two JNI integer arguments, or a JVM object. |
| Boundary delivery | Where do the converted values go at an exported call? | Native return, caller-provided output parameters, or a declared result callback. |

Policy guides the selection of these descriptions. The registry turns the descriptions into an executable plan.

### The registry API called by the frontend

The registry separates what should be generated from how values should be converted. An **output request** asks for one element, such as a function, type or constant. A **conversion rule** selects a relation and target policy for a particular type, parameter, result or child value. `BindingRequests` collects these requests and rules together with their policies and the information needed to report unsupported or ignored entries:

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

Inside the C frontend's build implementation after selecting v2 — the whole
chain from capture to planning, in internal pseudocode rather than user
`build.rs` code:

```rust
let mut builder = FlatBuilder::new();
builder.add_captures(Source::new(source_crate::PREBINDGEN_OUT_DIR).items_all())?;
let source_model = builder.build()?;              // stage 2: the snapshot

let requests = c_builder.into_requests(&source_model)?;  // this stage
let registry = Registry::new(source_model);
let generation = registry.generate(&c_adapter, requests)?; // stages 4 to 6
```

The frontend and registry independently use `prebindgen-flat` to inspect source items. The frontend interprets user declarations and validates their source references; the registry discovers required fields or helper arguments and plans their conversions. The registry supplies no separate source-inspection API to the frontend.

Request construction preserves all recorded frontend choices, including defaults, overrides, source mappings, helper signatures and ignore rules. Local helpers and declared Rust conversion operations are registered as typed source descriptions before planning. Unimplemented settings remain visible as unsupported requests. Naming closures may remain owned configuration objects; serialization is unnecessary.

## Identifying requests, value positions and reusable conversions

Names ending in `Id` follow one convention throughout. Each is a handle into a
table the registry owns, valid inside one generation run, and each is issued by
whoever owns that table: the frontend's request set issues `ElementId` and
`PolicyId`, the registry issues `RelationId`, `NodeId`, `PrimitiveId` and the
rest as it registers what a target described. A few are structured rather than
opaque — `SiteId` and `PartId` are positions, so they carry their owner and their
place in it — and those are shown below. No `Id` is a name a user writes, and
none can be constructed from a string.

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

A **part** is a field or argument converted within that relation. `Stamp.fields` (a descriptive label, not Rust syntax) has two parts; the constructor relation has one, `millis`. `PartId` identifies which part a conversion rule applies to.

For an enum such as `enum Event { At(Stamp), Count(u32) }`, the variant is also needed to identify a part. An **arm** is one alternative, and `ArmId` identifies it: here, `At` or `Count`. Each variant has a field at position 0, but those fields belong to different arms. A declared choice between constructors can also use arm IDs. Ordinary struct fields and a single constructor have no alternatives, so their arm is `None`.

```rust
struct PartId {
    owner: RelationId,       // Relationship defining the part, e.g. Stamp.fields.
    arm: Option<ArmId>,      // Enum variant/declared alternative; None without alternatives.
    position: PartPosition,  // Field identity, helper argument index, or projector result.
}
```

For example, `(Stamp.fields, None, Field("secs"))` identifies a struct field; `(Event.variants, Some(At), Field(0))` identifies `At`'s payload. These are illustrative IDs. `owner` refers to the containing relation, not Rust memory ownership. Function-specific overrides and diagnostics remain attached to `SiteId` positions.

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

A reusable conversion plan is a **node**. The registry finds nodes using a private `NodeKey`, derived internally from the accepted `Crossing`, selected relation and effective policy. No frontend/adapter conversion-planning API accepts `TypeKey` or `NodeKey`, or a caller-supplied type/key pair.

```rust
// Private to the registry's conversion cache module; not a public request type.
struct NodeKey {
    source: TypeKey,       // Derived internally from crossing.source.key().
    direction: Direction, // Copied from that crossing.
    relation: RelationId, // Validated selected relation.
    policy: PolicyId,     // Registered effective conversion settings.
}
```

The private cache operation accepts the validated crossing and selection, derives the key, and retains the same `TypeView` in the plan. Key construction and cache mutation are private; plan descriptors expose typed readings. Callers cannot submit mismatched type/key pairs, or ask for a conversion by handing in a key they built themselves.

The existing structural reading `TypeRef` has no `Eq`/`Hash`; its `key()` returns `prebindgen_flat::TypeKey`, which supplies both. `key()` preserves references/mutability, wrappers, generic arguments, array extents and lifetime spelling. Flat normalizes parentheses and known equivalent paths, such as `std::vec::Vec<T>` and `Vec<T>`, without equating arbitrary aliases. `stripped_key()` removes outer `Box`/`Cow` wrappers for declaration lookup: `Box<Stamp>` finds the `Stamp` declaration. The proposed `TypeView::key()` delegates to its retained reading. The conversion cache uses that key to retain wrappers. Plans retain the view for model-aware inspection and emission; key text cannot recreate a view.

For example, two owned `Stamp` inputs with the same two-integer JNI representation and field construction can share a node. An object-input override changes the policy; a return conversion changes direction. `Stamp`, `&Stamp` and `Option<&Stamp>` remain distinct.

Model membership follows the [snapshot contract](02-flat.md#private-storage-and-model-consistency). Flat publishes immutable source data after helper registration; its views preserve that snapshot through field and parameter navigation. The registry checks incoming views against its own model before planning. Flat owns these checks and private view construction. A valid view from another snapshot is rejected even when its key text matches. The registry accepts no detached reading or independently supplied model/type pair as a substitute for a view.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId` identifies a retained plan, and registry-issued node references must be validated within their generation context. Function sites retain separate overrides and diagnostic paths. Naming hooks are closures, and two closures cannot be compared, so two policies that contain them are distinct unless the frontend deliberately gives them the same `PolicyId`. Sharing a conversion between two configured values therefore requires sharing the policy entry, not writing an equal-looking one. Report entries are keyed by source and configuration identities that do not vary between runs over unchanged inputs, so two builds of the same crate produce the same report and a diff of it means something.

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
