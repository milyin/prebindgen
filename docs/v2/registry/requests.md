# Binding requests and conversion identity

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Source-model companion: [Flat V2](../flat/overview.md).

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

This choice does not list `Stamp`'s fields or explain how to construct it. The registry obtains those facts through a **source relationship**: a description of how a Rust value is constructed or read, such as using fields or a helper (the `RelationDefinition` in section [4](source-relations.md)). The adapter interprets the policy when describing the target representation and its property/argument operations.

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

`A: Target` ties the adapter to its policy and rendering-payload types (sections [5](target-operations.md)–[6](target-interface.md)). The method borrows the registry and adapter, consumes requests, and builds private working state. The returned `Generation` owns retained plans and payloads. Unsupported requests appear in its report; invalid input or invariant failures return `PlanningError` (section [9](generation.md)). Rendering and I/O follow planning.

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

Conversion planning takes a `TypeView` and a direction, represented by `Crossing`. A `TypeView` is a read-only handle retaining an exact Rust type reading and the immutable Flat model in which it is interpreted; the [Flat V2 project](../flat/inspection.md#type-readings-and-type-views) defines its lookup and navigation API. Frontends and adapters supply source descriptions, not cache keys:

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

Model membership follows the [Flat V2 snapshot contract](../flat/model.md#private-storage-and-model-consistency). Flat publishes immutable source data after helper registration; its views preserve that snapshot through field and parameter navigation. The registry checks incoming views against its own model before planning. Flat owns these checks and private view construction. A valid view from another snapshot is rejected even when its key text matches. The registry accepts no detached reading or independently supplied model/type pair as a substitute for a view.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId` identifies a retained plan, and registry-issued node references must be validated within their generation context. Function sites retain separate overrides and diagnostic paths. Policies containing closures share identity only when equivalence is established. Reports use deterministic source/configuration identities.

---

Previous: [Registry purpose](overview.md) · Next: [Source relations](source-relations.md)
