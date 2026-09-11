<!-- spec: {"kind": "stage", "stage": "05-boundary"} -->

[Project contents](../README.md) · Previous: [Plan value conversions](04-values.md) · Next: [Retain supported output](06-retain.md)

# Assemble the native boundary

Status: implemented. Delivery is a native return or nothing: out-parameters,
`Result` branches and declared sinks are variants the engine does not have yet.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

A conversion knows how to turn a value into another value. It does not know which
exported function it serves, where that function's result is supposed to go, or
what should happen if a conversion inside it fails. That is this stage: taking
the conversions planned for one requested function and assembling the actual
native function a foreign caller will call.

For this example, the two targets end up with these signatures:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64

#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampSum(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    stamp: jni::objects::JObject<'_>,
) -> jni::sys::jlong
```

Both wrap the same Rust function through the same input conversion, yet neither
signature can be derived from the other. The C one is what a C caller can write;
the JNI one is what the JVM demands — a symbol built from the Kotlin package, the
harness object holding the native declaration and the method name, plus two
parameters, the environment and the calling class, that the JVM passes to every
native method and that no source parameter corresponds to. The target supplies
those conventions; the registry places the conversions inside them. The source
parameter keeps its name in both, because the boundary said so — a parameter is
the one name the writer does not allocate.

Three things are decided here. **Where inputs come from**: which native argument
feeds which input conversion, including arguments the target added for its own
reasons. **Where the result goes** — the *delivery* — which here is
the native return, but could be a caller-provided out-parameter, or a callback
the configuration named; the same conversion is reused whichever destination
applies, so a conversion never has one version per destination. **What happens on
failure**: every failure a conversion declared needs a terminal action here, and
the actions differ by category — a `Result::Err` returned by the source function
is not the same event as a JNI call failing mid-conversion.

The JNI wrapper shows why that last part is not a detail. Reading `getSecs()` can
fail; the conversion says so but decides nothing. The route comes from the
adapter, and the one this example's adapter takes is to report the failure to the
JVM and return zero — zero being merely what a native method must return while an
exception is pending, since the caller sees the exception rather than a result.
Reporting can itself fail, and that path terminates by aborting. The registry
emits the branch, the reporting call, the check on its result and the terminal
return; the adapter supplies the operation that reports. (The shipping JNI
adapter routes errors differently — through a handler object the Kotlin caller
passes — which is a choice in the same slot, not a different pipeline.)

If a requested delivery or failure route is not supported, the function is
skipped and the report says why. It is never quietly given a different ABI than
the one the configuration asked for, because a caller compiled against the header
or the Kotlin declaration would then be calling something else.

Only exported callables reach this stage. A record has conversions but no
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

`AbiSpec` describes the native interface, including target calling conventions and explicit environment operands such as a JNI environment. Each native parameter carries its role: it feeds one source parameter's conversion, it supplies a named runtime context that operations ask for, or the convention requires it and nothing uses it. That is what `InputPlacement` is as implemented, and it is also how an operation's `Context("jni.env")` operand finds the parameter that satisfies it — a conversion needing a context its boundary does not supply is skipped, with the reason. `ValueMapping` maps converted values/slots to a return, output location, or invocation argument. These are transport descriptions; they do not repeat source decomposition.

`SinkId` identifies a declared destination and signature, not the runtime callback pointer itself. `CalleeId` identifies the source operation being wrapped. `FunctionOutput` identifies the conversions for the wrapped source operation's result. `FunctionBodyId` refers to the complete structured wrapper instructions assembled by the registry.

There are three relevant error categories: a domain error returned by the source API, a binding/conversion error, and a runtime error such as a JNI failure. `FailureRoutes` selects their terminal actions: return a configured status, throw, call a declared handler, or abort according to the binding's policy.

One route is a reporting operation the target supplies, what to do when reporting
itself fails, and how the route ends — returning an expression or aborting. The
registry emits the branch, the reporting call, the check on its result and the
terminal action; the target contributes only the operation that reports. A
category a conversion can raise and the boundary does not list makes the function
unsupported.

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

[fn_boundary]: ../examples/fn/05-boundary.md
[fn_boundary_c]: ../examples/fn/05-boundary.c.md
[fn_boundary_jni]: ../examples/fn/05-boundary.jni.md
[struct_values]: ../examples/struct/04-values.md
