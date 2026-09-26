<!-- spec: {"kind": "stage", "stage": "07-retain"} -->

[Project contents](../README.md) · Previous: [Assemble the wrapper boundary](06-boundary.md) · Next: [Emit bindings](08-emit.md)

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
`Stamp` — the C struct, the Kotlin data class — has to exist for the [wrapper](06-boundary.md#assemble-the-wrapper-boundary) that
takes it to be usable at all. A Kotlin method needs the class it is declared on,
and if the configuration promised that class implements an interface, it needs
the members of that interface too.

So a missing capability propagates. Suppose one field of the struct had a type no
[representation](05-represent.md#represent-and-compose-values) covers yet. Its conversion is unsupported; the struct's conversion
is therefore unsupported; the public struct cannot be emitted; and the function
that takes it is skipped as well. Each dependent's skip copies the underlying
reason and keeps its own path to it. An unrelated function in
the same package is still generated. Nothing is emitted in a reduced form. A
struct is never emitted with a field left out, because a foreign type missing a
field is a different type, not a partial one.

An opaque-handle representation would have a different contract: the Rust value
stays behind a handle, and foreign callers do not receive its fields. Such a
representation need not convert unused fields. That does not permit a requested
data struct to lose a field silently, or permit V2 to replace an unsupported
data representation with a handle.

Every declaration finishes with exactly one **outcome**. *Emitted* means its
requirements succeeded and it can be generated. *Skipped* means the request needs support V2 does not yet provide — an outcome
that exists only while V2 is being brought up to V1's coverage, as
[the next section](#unsupported-requests-and-public-api-dependencies) sets
out.

A skipped declaration carries a **capability**: a stable code naming the
support it needs, such as `unsupported.conversion.no_rule`. An explanation and a path
to the failing dependency tell the developer where planning stopped. The
design also calls for *unselected*, to account for captured items nobody
requested, but that fourth outcome is not implemented.

Contradictory configuration and broken internal assumptions are different:
they fail generation rather than becoming ordinary skips. On a completed run,
`Generation::skipped` hands back every skip for inspection — a build script
prints them, a test reads them, and no stage reads them back to make a
generation decision.

What survives is then **frozen**: planning is over and retained plans determine
the output. Current `generate` renders Rust before returning `Generation`,
which stores that text, the plans, and what it left out. The Kotlin writer reads the
retained public descriptions. Neither writer may plan a missing conversion,
add a dependency or reverse a support decision.

## Unsupported requests and public API dependencies

**Specified behavior: a requested binding that cannot be generated fails the
build.** A build script asks for an exported API; quietly shipping less than it
asked for is not something the finished engine offers, and a consumer must not
have to read a diagnostic to learn that a function it declared does not exist.

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
// A missing capability is recorded in the binding by the frontend — a
// representation or an output it cannot lower — or met by the registry while
// it plans; the registry owns the final accounting.
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

A function also needs the public type used by its parameters, and a type
needs the public types of its fields. The registry works out both itself,
from the plan: every value of a named type a function takes or returns, and
every part of a type's own value or a callback's, requires the type output
exposing the
[representation](05-represent.md#represent-and-compose-values) that value's
[crossing](03-requests.md#finding-an-existing-conversion-plan) was planned
with, and only that one: `Stamp` exposed as a struct and as a handle is two
outputs, and a value crossing as the handle requires the second. A type output
the frontend refused exposes no representation; a value of that type is
skipped with the refused output's cause. A value of a type no output
exposes, or exposed only as other representations, produces
`unsupported.requirement.unrequested`: a foreign signature would otherwise
name a type the header or the Kotlin sources declare differently, or not at
all. A callback is required the same way, by its signature: a function taking
an `impl Fn(i64)` requires the `callback:` output exposing the representation
its parameter crossed as — the C closure struct or the Kotlin `fun interface`
its parameter is typed as — and a callback requires the type outputs of its
arguments.

Requiring a type does not export it: a type nothing requested is not emitted
because something needed it, and whatever needed it is skipped instead.

The full design requires the following dependency behavior. Struct/caller
propagation is implemented; interface and method promises extend that rule:

- An unsupported field skips the entire struct and callers that require that struct representation.
- An opaque handle is retained with its release or not at all, and a function returning one requires its public declaration as a function taking one does: a caller cannot hold what the header never names, nor free it through a method that was not emitted. [The handle path][typedef_retain] shows both.
- An unsupported optional method can be omitted independently; an interface-required method can prevent its owner from being emitted.
- An unsupported owner representation prevents methods requiring that owner.
- Unrelated free functions in a package can still be generated.
- A shared helper is retained if any emitted output needs it.
- An unimplemented semantic setting blocks the affected promise; it is not silently discarded.

Retaining a type output is not the same as writing it: its
[carriers](05-represent.md#describing-target-values-and-operations)'
declarations are written at [emission](08-emit.md), and whether it becomes a
header entry or a Kotlin class is the target's business then. An
[artifact](05-represent.md#individual-target-operations) — one generated unit,
such as a helper an operation's text needs — is emitted only when a retained
wrapper's operation needs it.

Public types referring to each other do not necessarily require an infinitely recursive conversion. Conversion-expansion cycles and public-declaration dependencies therefore need separate checks.

Retention may need several passes. Suppose declaration A needs B, and B needs
C. The engine cannot keep A merely because A's own conversion succeeded; it
must also know the outcome of B and C. It reevaluates dependent declarations
until the set of decisions stops changing, a process often called reaching a
fixed point. This check is separate from recursively planning a value's fields.

## Registry state, execution order and final output

Generation needs three kinds of storage: the source model with the binding,
mutable working state, and the completed result. `generate` uses a private
`Run` for the working state — the rules by scope, the conversion
[nodes](05-represent.md#represent-and-compose-values), the applications of
operations, the cache and the cycle marks — and returns a
`Generation`, which owns the model, the binding and the completed output.

The pipeline is:

```text
existing source captures + C/JNI frontend configured through its Rust API
 -> user calls the frontend build method
 -> frontend selects v1 or v2
 -> v2 frontend builds its binding — carriers, representations, rules,
    outputs — and calls generate with its target
 -> registry checks declarations against the model, and rules against the
    outputs they address
 -> for each requested value: registry finds the rule that applies and the
    relation its representation names
 -> registry resolves that relation's children, then composes the value's
    instructions from the representation's operations
 -> registry assembles each wrapper from its function form
 -> registry propagates skips and retains complete supported output
 -> common Rust writer renders the retained plans, calling the target's
    writers for its operations and for its carriers' declarations
 -> return Generation with Rust text, the binding, retained outputs and skips
 -> C: cbindgen derives headers from generated Rust
    JNI: the frontend's Kotlin writer reads the retained outputs
 -> publish generated files
 -> the binding prints or publishes the skips as it chooses
```

Rule lookup, child resolution and composition happen together for each
node. No target code runs until
the plan is complete. Wrapper and requirement failures can remove candidate
outputs before the result is frozen.

```rust
struct Generation<T: Target> {
    flat: Flat,                          // The model the run planned over.
    binding: Binding<T>,                 // What planning read.
    skipped: Vec<(Declaration, Skip)>,   // What the binding asked for and did not get.
    unused_rules: Vec<Scope>,            // Type rules no planned value used.
    values: Vec<ValuePlan>,              // Planned conversions, by NodeId.
    retained: Vec<Retained>,             // Surviving outputs, with the values planned for each.
    functions: Vec<FunctionPlan<T::Op>>, // Retained wrappers.
    rust: String,                        // The generated Rust.
}
```

**Frozen** means planning is complete and consumers cannot mutate the result.
The retention loop keeps an output only when the outputs it requires succeed.
It retains all planned conversion nodes, including ones no surviving wrapper
needs; the instructions have already been inlined, so this is extra storage
rather than a dangling reference.

The common Rust writer renders before `Generation` is returned;
`Generation::write_rust` writes the stored text. The JNI Kotlin writer reads
the retained outputs, their metadata and the carriers their values resolved
to. The C build passes generated Rust to `cbindgen` for headers. Writers must
not introduce a newly discovered dependency or reverse a support decision.
Files should be published only after output generation succeeds.

What a binding makes of a skip — a cargo warning, a file beside the generated
code, a test's expectation — is the frontend's business, and separate from the
retention decisions described here.

## Elements at this stage

- [Function taking an owned struct][fn_retain]
- [Struct with scalar fields][struct_retain]
- [Type alias declaring an opaque handle][typedef_retain]
- [Function taking a callback][fn_callback_retain]

[fn_retain]: ../examples/fn/07-retain.md
[struct_retain]: ../examples/struct/07-retain.md
[typedef_retain]: ../examples/typedef/07-retain.md
[fn_callback_retain]: ../examples/fn_callback/07-retain.md
