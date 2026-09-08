<!-- spec: {"contract": "plans", "kind": "contract", "stage": "05-boundary"} -->

[Pipeline chapter](../05-boundary.md) · [Project contents](../../README.md)

# Conversion and exported-function plans

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

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
