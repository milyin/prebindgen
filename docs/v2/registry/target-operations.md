# Individual target operations

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Source-model companion: [Flat V2](../flat/overview.md).

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
    Source(TypeView),     // An exact Rust type in its retained Flat snapshot.
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
in section [7](plans.md).

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

#### Concrete generated-code examples

The [C and Kotlin/JNI examples](primitive-examples.md) show a complete foreign declaration and native wrapper, the primitive specifications behind their input operations, and the exact ownership of each generated fragment.
