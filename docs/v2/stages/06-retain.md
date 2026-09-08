<!-- spec: {"kind": "stage", "stage": "06-retain"} -->

# Retain supported output

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

Planning produces candidates. This stage decides which of them actually become
output: it resolves every dependency a public declaration has, drops the requests
whose dependencies are missing, records why, and freezes what remains.

**Input.** The candidate value plans, function plans and public declarations, the
dependencies between them, and the causes collected wherever the target or the
registry reported a missing capability.

**Owner.** The registry. Targets answer local questions and never decide what is
emitted; a writer, later, cannot revisit the decision either.

**Output.** A frozen `Generation`: retained conversions, native functions, public
declarations and generated artifacts in a valid emission order, plus a report
saying what was emitted, skipped, ignored or never selected, and for each skip,
the capability that was missing and where it was needed.

**Failure.** An unsupported request is a normal outcome, not an error. Malformed
configuration and violated internal invariants are `PlanningError` and fail the
build. Nothing partially supported is retained: a function whose input conversion
is unsupported is skipped whole, and the record it needed is skipped along with
every other declaration that required that record.

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

## Elements at this stage

- [Function taking an owned record][fn_retain]
- [Record with scalar fields][struct_retain]

---

Previous: [Assemble the native boundary](05-boundary.md) · Next: [Emit bindings](07-emit.md)

[fn_retain]: ../examples/fn/06-retain.md
[struct_retain]: ../examples/struct/06-retain.md
