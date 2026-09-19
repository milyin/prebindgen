<!-- spec: {"kind": "stage", "stage": "07-retain"} -->

[Project contents](../README.md) · Previous: [Assemble the native boundary](06-boundary.md) · Next: [Emit bindings](08-emit.md)

# Retain supported output

Status: implemented. Outcomes, causes and the frozen result are what this stage
produces; `Unselected` and the pruning of unreachable [conversions](04-select.md#select-conversion-relations)
are not there yet.

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

By now the engine has attempted to plan the requested bindings. Some plans are
complete; others need a feature V2 does not implement. **Retention** chooses
the complete set that can safely be published. It checks dependencies before
letting a writer produce files, so the generated API does not contain functions
whose required types or helpers are missing.

There are both conversion dependencies and public API dependencies. The exported `stamp_sum` needs
the conversion of `Stamp`, which needs a conversion for each field. The public
`Stamp` — the C struct, the Kotlin data class — has to exist for the [wrapper](06-boundary.md#assemble-the-native-boundary) that
takes it to be usable at all. A Kotlin method needs the class it is declared on,
and if the configuration promised that class implements an interface, it needs
the members of that interface too.

So a missing capability propagates. Suppose one field of the struct had a type no
[representation](05-represent.md#represent-and-compose-values) covers yet. Its conversion is unsupported; the struct's conversion
is therefore unsupported; the public struct cannot be emitted; and the function
that takes it is skipped as well. Current reports copy the underlying reason
and preserve each dependent's path to it. An unrelated function in
the same package is still generated. Nothing is emitted in a reduced form. A
struct is never emitted with a field left out, because a foreign type missing a
field is a different type, not a partial one.

An opaque-handle representation would have a different contract: the Rust value
stays behind a handle, and foreign callers do not receive its fields. Such a
representation need not convert unused fields. That does not permit a requested
data struct to lose a field silently, or permit V2 to replace an unsupported
data representation with a handle.

Every declaration finishes with exactly one **outcome**. *Emitted* means its
requirements succeeded and it can be generated. *Ignored* means the user
explicitly excluded it; that is a configuration decision, not a gap in support.
*Skipped* means the request needs support V2 does not yet provide — an outcome
that exists only while V2 is being brought up to V1's coverage, as
[the next section](#unsupported-requests-and-public-api-dependencies) sets
out.

A skipped declaration carries a **capability**: a stable code naming the
support it needs, such as `unsupported.jni.carrier`. An explanation and a path
to the failing dependency tell the developer where planning stopped. The
design also calls for *unselected*, to account for captured items nobody
requested, but that fourth outcome is not implemented.

Contradictory configuration and broken internal assumptions are different:
they fail generation rather than becoming ordinary skips. On a completed run,
[the report](../report.md) records the outcomes for inspection. It is diagnostic
output; the pipeline does not read it back to make generation decisions.

What survives is then **frozen**: planning is over and retained plans determine
the output. Current `generate` renders Rust before returning `Generation`,
which stores that text, the plans and the report. The Kotlin writer reads the
retained public descriptions. Neither writer may plan a missing conversion,
add a dependency or reverse a support decision.

## Unsupported requests and public API dependencies

**Specified behavior: a requested binding that cannot be generated fails the
build.** A build script asks for an exported API; quietly shipping less than it
asked for is not something the finished engine offers, and a consumer must not
have to read a report to learn that a function it declared does not exist.

**Skipping is transitional.** V2's coverage is still smaller than V1's, so while
that gap lasts the engine accepts a request it cannot generate, records it as a
skipped declaration with its reason, and generates the rest. That is what lets
an existing V1 configuration be pointed at V2 without being rewritten first, and
what lets the three element paths here be built before the rest of the surface
exists. It is a migration device with an end: as coverage completes, each
remaining skip becomes a build failure, and nothing in a consumer's build should
be arranged to depend on skipping. Invalid configuration and generator defects
already fail generation today.

The rest of this section describes that transitional accounting. The following
sketch describes the fuller diagnostic design, including source locations,
shared causes and the future `Unselected` outcome. Current error payloads and
reports are simpler, as explained below.

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

In the sketch, `Diagnostic` combines a message and source location, and
`CauseId` refers to a shared cause table. Current `PlanningError` carries a
message string; a skipped declaration carries a `Skip` with a capability,
explanation and dependency path. There is no current cause-id table. Propagation
copies the reason and extends its path to explain why another output was skipped.

The distinction between an operation result and an output outcome matters: a scalar conversion can be ready while its enclosing function is skipped because another parameter is unsupported. Several skipped outputs can share one underlying cause, while reports retain each output's dependency path.

Planning itself determines support; there is no separate recursive
`supports(type)` pass. The engine tracks active conversions to detect cycles
and caches completed plans for reuse. Unsupported attempts are not cached today
and may be attempted again. Cycle detection follows selected
[relations](04-select.md#what-a-relation-is), so a future atomic handle can stop
expansion without traversing fields. Unsupported recursive conversion is a skip;
contradictory configuration, panics and I/O failures are not ordinary skips.

### Dependencies of public declarations

A function also needs the public type used by its parameters. Current
`SurfaceSpec.requires` lists required `DeclarationId`s. A required declaration
that was never requested produces `unsupported.requirement.unrequested`.
The design sketch below extends these requirements to interface promises and
member associations; those richer fields are not implemented yet.

```rust
struct SurfaceSpec<Payload> {
    declaration: DeclarationId,  // Public type/function/member this description implements.
    requires: Vec<Requirement>, // Required public types, conversions, interfaces or helpers.
    members: Vec<DeclarationId>, // Associated declarations, such as a class's methods.
    payload: Payload,            // Chosen target name/package/modifiers and rendering metadata.
}
```

`Requirement` is a typed reference to a dependency that must succeed. It is the resolved obligation derived from an output request's semantic promises or actual usage. `members` describes association; a member that is essential to a promised interface must also be a requirement.

The full design requires the following dependency behavior. Struct/caller
propagation is implemented; interface and method promises extend that rule:

- An unsupported field skips the entire struct and callers that require that struct representation.
- An opaque handle is retained with its release or not at all, and a function returning one requires its public declaration as a function taking one does: a caller cannot hold what the header never names, nor free it through a method that was not emitted. [The handle path][typedef_retain] shows both.
- An unsupported optional method can be omitted independently; an interface-required method can prevent its owner from being emitted.
- An unsupported owner representation prevents methods requiring that owner.
- Unrelated free functions in a package can still be generated.
- A shared helper is retained if any emitted output needs it.
- An unimplemented semantic setting blocks the affected promise; it is not silently discarded.

A `SurfaceSpec` describes one public declaration and its requirements. Describing it is not the same as writing it: whether that description becomes a header entry or a Kotlin class is [emission](08-emit.md)'s business, and a target without a foreign writer still produces these descriptions. An [artifact](05-represent.md#individual-target-operations) — one generated unit, as defined with [the operations that depend on them](05-represent.md#individual-target-operations) — is what actually gets emitted: one public declaration can require a foreign wrapper, a native extern, converter helpers and runtime helpers, each its own artifact, several of which may end up in one file. The registry keeps candidate artifacts during planning and publishes only those needed by complete supported outputs.

Public types referring to each other do not necessarily require an infinitely recursive conversion. Conversion-expansion cycles and public-declaration dependencies therefore need separate checks. Public dependencies may require repeated readiness evaluation until the retained set stops changing. A new public requirement discovered after value planning must still propagate before output is finalized.

Retention may need several passes. Suppose declaration A needs B, and B needs
C. The engine cannot keep A merely because A's own conversion succeeded; it
must also know the outcome of B and C. It reevaluates dependent declarations
until the set of decisions stops changing, a process often called reaching a
fixed point. This check is separate from recursively planning a value's fields.
The current implementation propagates public requirements over its candidate
declarations; the broader design also accounts for newly discovered requirements.

## Registry state, execution order and final output

Generation needs three kinds of storage: the source model, mutable working
state, and the completed result. The following design sketch calls the first
two `Registry` and `GenerationRun`. In the implementation, `generate` uses a
private `Run` rather than these exact public structs. It returns `Generation`,
which owns the model and completed output. An **arena** below means a table of
records addressed by ids, not a separate planning algorithm:

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
    outcomes: OutcomeTable, // Per-declaration decisions and shared unsupported causes.
}
```

`T` implements `Target`. Each `*Arena` is a registry-owned table addressed by typed IDs. `OutcomeTable` holds classifications and diagnostic causes. [Primitive](05-represent.md#represent-and-compose-values)/layout/body tables are omitted here. The registry updates these tables; adapters receive immutable descriptions.

The pipeline is:

```text
existing source captures + C/JNI frontend configured through its Rust API
 -> user calls the frontend build method
 -> frontend selects v1 or v2
 -> v2 frontend creates BindingRequests internally and calls generate
 -> registry validates/imports source references, policies and requests
 -> for each requested value: select its source relation
 -> registry resolves the selected source operation's children
 -> target describes the requested value's representation
 -> registry composes value instructions and contracts
 -> target describes native delivery and public declarations
 -> registry assembles functions and resolves all required dependencies
 -> registry propagates skips and retains complete supported output
 -> common Rust writer renders the retained native plans
 -> return Generation with Rust text, retained descriptions and report
 -> C: cbindgen derives headers from generated Rust
    JNI: optional foreign-writer interface is implemented by the Kotlin writer
 -> publish generated files
 -> optionally write the diagnostic report beside them
```

Selection, child resolution and representation happen together for each [node](05-represent.md#represent-and-compose-values). A complete conversion table is not required before relation choices are known. Boundary/public-declaration failures can remove candidate outputs before the result is frozen.

The following is the proposed storage organization. Current `Generation` owns
Flat, vectors of value/function/surface/primitive records, a `Report` and the
already-rendered Rust string. It does not contain an ordered artifact arena.

```rust
struct Generation<Payload> {
    values: FrozenArena<ValuePlan<Payload>>, // Retained reusable conversions.
    functions: FrozenArena<FunctionPlan<Payload>>, // Retained complete native functions.
    surface: FrozenArena<SurfaceSpec<Payload>>, // Complete retained public declarations.
    artifacts: OrderedArtifacts<Payload>, // Generated units with a validated emission order.
    outcomes: Outcomes,     // Every declaration's outcome, with its diagnostic path.
}
```

Freezing retains all referenced tables (bodies, primitives, layouts, helpers) and the required `Flat` source-emission data, directly or through shared ownership. Rendering uses these retained records and language-provided rendering code. The original registry, borrowed adapter and temporary working tables need not remain alive.

**Frozen** means planning is complete and consumers cannot mutate the result.
The implemented retention loop keeps a public declaration only when its required
declarations succeed. It retains all planned conversion nodes, including ones no
surviving wrapper needs; the instructions have already been inlined, so this is
extra storage rather than a dangling reference.

Artifacts are currently collected and deduplicated by name before Rust rendering.
There is no general dependency sort or forward-declaration planner. The
`FrozenArena` and `OrderedArtifacts` structures above describe a proposed
extension, not types the current implementation exposes.

The common Rust writer renders before `Generation` is returned;
`Generation::write_rust` writes the stored text. The JNI writer reads retained
public descriptions for Kotlin. The C build passes generated Rust to `cbindgen`
for headers. Writers must not introduce a newly discovered dependency or reverse
a support decision. Files should be published only after output generation succeeds.

The report's contents, frontend accessors and planned use for test selection
are explained on [its own page](../report.md). Those are separate from the
retention decisions described here.

## Elements at this stage

- [Function taking an owned struct][fn_retain]
- [Struct with scalar fields][struct_retain]
- [Type alias declaring an opaque handle][typedef_retain]

[fn_retain]: ../examples/fn/07-retain.md
[struct_retain]: ../examples/struct/07-retain.md
[typedef_retain]: ../examples/typedef/07-retain.md
