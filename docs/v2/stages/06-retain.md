<!-- spec: {"kind": "stage", "stage": "06-retain"} -->

[Project contents](../README.md) · Previous: [Assemble the native boundary](05-boundary.md) · Next: [Emit bindings](07-emit.md)

# Retain supported output

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

Planning produces candidates: conversion nodes, wrapper plans, public
declarations, generated helpers. Not all of them can be emitted, because they
depend on each other, and V2 deliberately accepts more input than it can yet
generate. This stage decides what survives.

The dependencies run both ways through a binding. The exported `stamp_sum` needs
the conversion of `Stamp`, which needs a conversion for each field. The public
`Stamp` — the C struct, the Kotlin data class — has to exist for the wrapper that
takes it to be usable at all. A Kotlin method needs the class it is declared on,
and if the configuration promised that class implements an interface, it needs
the members of that interface too.

So a missing capability propagates. Suppose one field of the record had a type no
representation covers yet. Its conversion is unsupported; the record's conversion
is therefore unsupported; the public record cannot be emitted; and the function
that takes it is skipped as well — one underlying cause, recorded once, reached by
several dependency paths. What is *not* affected stays: an unrelated function in
the same package is still generated. Nothing is emitted in a reduced form. A
record is never emitted with a field left out, because a foreign type missing a
field is a different type, not a partial one.

One clarification about "reduced": a record represented as an opaque handle is
not a reduced record. Its fields never cross at all — the whole value stays on the
Rust side behind a pointer — so there is no field to leave out. The rule above
bites only where a representation promised to carry the parts.

Being unsupported is a normal outcome, not a failure. Each skipped element is
recorded with a stable capability code, a human-readable explanation and the
source or configuration location that provoked it, so the report can say what to
implement rather than just that something is missing. Two other outcomes exist so that
the report can account for every captured item, not only the requested ones: an
element the user explicitly excluded is *ignored*, and one that was captured but
that no configuration asked for is *unselected*. The second is expected in bulk —
a source crate typically captures more than any one binding exposes — so a report
groups those rather than listing them beside real skips; their value is answering
"why is this not in my header" without reading the build script. Genuine
errors — contradictory configuration, or a violated internal invariant — are not
turned into skip reasons; they fail generation.

What survives is then **frozen**: planning is over, the records are immutable,
and the result — retained conversions, wrappers, public declarations, and the
generated units in a valid emission order — is handed to the writers together
with the report. A writer reads it. It cannot plan a conversion that is missing,
add a dependency, or change any of these decisions.

## Unsupported requests and public API dependencies

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
    at: FailureLocation,        // A root, boundary site, or reusable relation part.
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

Planning itself determines support. There is no separate recursive `supports(type)` pass that could disagree with generation. The registry tracks each attempted conversion as unseen, currently being resolved, ready, or unsupported. Encountering a currently active conversion can reveal an expansion cycle. Cycle detection follows selected relations: an atomic handle can stop expansion of a recursive source type. A recursive conversion not yet implemented in v2 is reported as unsupported; a contradictory conversion rule remains invalid input. Panics and I/O failures are not converted into skip reasons.

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

A `SurfaceSpec` describes one public declaration and its requirements. Describing it is not the same as writing it: whether that description becomes a header entry or a Kotlin class is [emission](07-emit.md)'s business, and a target without a foreign writer still produces these descriptions. An artifact — one generated unit, as defined with [the operations that depend on them](04-values.md#individual-target-operations) — is what actually gets emitted: one public declaration can require a foreign wrapper, a native extern, converter helpers and runtime helpers, each its own artifact, several of which may end up in one file. The registry keeps candidate artifacts during planning and publishes only those needed by complete supported outputs.

Public types referring to each other do not necessarily require an infinitely recursive conversion. Conversion-expansion cycles and public-declaration dependencies therefore need separate checks. Public dependencies may require repeated readiness evaluation until the retained set stops changing. A new public requirement discovered after value planning must still propagate before output is finalized.

That is a second loop, and it is worth separating from the first. Value planning
is one recursive walk that terminates because each conversion is either found in
the cache, resolved, or refused. Deciding what to retain is an outer loop over
the candidate set: a public declaration can require a conversion that was not
planned yet, planning it can produce another public requirement, and the loop
runs until a pass adds nothing. It terminates for the same reason the walk does —
the set of requestable elements and reachable conversions is finite, and every
pass either adds to the retained set or ends it — but it is iteration, not
recursion, and a design that assumed one pass here would be wrong.

## Registry state, execution order and final output

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
 -> for each requested value: select its source relation
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

Selection, child resolution and representation happen together for each node. A complete conversion table is not required before relation choices are known. Boundary/public-declaration failures can remove candidate outputs before the result is frozen.

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

The report has a second consumer. The example crates' test suites are written
against the full API, so when V2 emits a subset, tests referring to what it
skipped would not compile — a runtime guard cannot hide a missing class from
`kotlinc` or a missing symbol from a C compiler. Each suite is therefore divided
into **test sections**, each naming the emitted elements it needs, and the report
selects the sections whose elements were all emitted. A milestone still has to
require that meaningful sections run, so that skipping everything cannot pass.

The common Rust writer reads `Generation`; JNI's optional writer reads the same result for Kotlin. C passes generated Rust to `cbindgen` for headers. Writers cannot add dependencies or change support decisions. Publish after output generation succeeds. The report also selects existing test sections, ensuring tests match the emitted API.

## Elements at this stage

- [Function taking an owned record][fn_retain]
- [Record with scalar fields][struct_retain]

[fn_retain]: ../examples/fn/06-retain.md
[struct_retain]: ../examples/struct/06-retain.md
