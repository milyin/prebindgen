<!-- spec: {"kind": "stage", "stage": "05-boundary"} -->

# Assemble the native boundary

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

Conversions are reusable and know nothing about the function that uses them. This
stage puts them into one exported function: which native arguments feed which
input conversion, where the converted result goes, and what happens on each way
the call can fail.

**Input.** The requested function, its input and output conversion nodes, and the
policy governing calling convention, result delivery and error handling.

**Owner.** The registry assembles and validates the wrapper; the target adapter
describes the native interface — symbol, calling convention, environment
operands, result placement and the terminal action for each failure category.

**Output.** A `FunctionPlan`: the source callee, its input nodes in parameter
order, its output conversions, a validated `BoundarySpec` and the complete body
instructions for the wrapper, plus the generated prerequisites it needs.

**Failure.** An unsupported result destination or error route skips the function.
Silently changing the ABI — dropping an out-parameter, returning a status the
configuration did not ask for — is not a substitute for supporting it.

Only exported callables reach this stage. A record has a conversion but no
boundary of its own; it crosses inside the functions that use it.

## Assembling an exported function

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

## Elements at this stage

- [Function taking an owned record][fn_boundary] · [C][fn_boundary_c] · [Kotlin/JNI][fn_boundary_jni]

The record path has no cell here: [its conversion][struct_values] is reached
through the function that uses it.

---

Previous: [Plan value conversions](04-values.md) · Next: [Retain supported output](06-retain.md)

[fn_boundary]: ../examples/fn/05-boundary.md
[fn_boundary_c]: ../examples/fn/05-boundary.c.md
[fn_boundary_jni]: ../examples/fn/05-boundary.jni.md
[struct_values]: ../examples/struct/04-values.md
