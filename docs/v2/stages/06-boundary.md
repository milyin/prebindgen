<!-- spec: {"kind": "stage", "stage": "06-boundary"} -->

[Project contents](../README.md) · Previous: [Represent and compose values](05-represent.md) · Next: [Retain supported output](07-retain.md)

# Assemble the wrapper boundary

Status: implemented. Delivery is a wrapper return or nothing: out-parameters,
`Result` branches and declared sinks are variants the engine does not have yet.

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

A value [conversion](04-select.md#select-conversion-relations) answers a local question, such as how to construct a Rust
`Stamp` from a JVM object. A callable binding needs more: an exported symbol,
a calling convention, parameters, a return value and error handling. This stage
combines the value conversions into a plan for that complete function, called a
**wrapper** because it surrounds the source call with conversion and
error-handling code. Its exported signature and calling convention — what the
foreign runtime actually calls — are the **wrapper boundary**.

For this example, the two targets end up with these signatures:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64

#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampSum(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    stamp: jni::objects::JObject<'_>,
) -> jni::sys::jlong
```

Both entry points eventually call the same source function. Their input
conversions use the same struct-construction algorithm but different reads:
C member access in one case, JNI getter calls in the other.

The C function uses the C calling convention and receives a C-compatible struct.
The JNI function uses the platform's JNI calling convention, spelled `system`
in Rust. Its exported `Java_...` name identifies the package, `JNINative`
object and `external` method. The JVM supplies `env`, the interface used to call
back into the JVM, and `_this`, the receiver of that instance method.
The `stamp` parameter is the actual user argument. These extra parameters have
no counterparts in `source::stamp_sum`.

The target adapter describes that signature. The registry connects its
parameters to the conversion plans. Wrapper parameter names come from the
boundary description; the common writer chooses names for internal temporaries.

Three things are decided here. **Where inputs come from**: which wrapper argument
feeds which input conversion, including arguments the target added for its own
reasons. **Where the result goes** — the *delivery* — which here is
the wrapper return, but could be a caller-provided out-parameter, or a callback
the configuration named; the same conversion is reused whichever destination
applies, so a conversion never has one version per destination. **What happens on
failure**: every failure a conversion declared needs a terminal action here, and
the actions differ by category — a `Result::Err` returned by the source function
is not the same event as a JNI call failing mid-conversion.

For example, `getSecs()` can fail before the source function runs. The V2 JNI
adapter reports that failure to the JVM and returns a placeholder zero from the
wrapper. With an exception pending, the Kotlin caller observes the exception,
not a successful result of zero. If reporting itself fails, the
generated code aborts. The adapter supplies the reporting operation, and the
registry plans the branch and terminal action around it. V1's handler-based
convention is a separate implementation and should not be confused with this
V2 example.

If a requested delivery or failure route is not supported, the function is
skipped, with the reason recorded in its [outcome](07-retain.md#retain-supported-output). It is never quietly given a different ABI than
the one the configuration asked for, because a caller compiled against the header
or the Kotlin declaration would then be calling something else.

Only exported callables reach this stage. A struct has conversions but no
boundary of its own; it crosses inside the functions that use it.

## Assembling an exported function

A complete binding function coordinates value plans: convert inputs, call the
source once, convert its result and handle failures. Future resource-bearing
conversions will also require cleanup on every relevant exit path.

One exported function has no source function behind it: the release of an
opaque handle. Its `SiteDescriptor` names the handle type's declaration and no
callee, its one wrapper parameter carries what the handle's consuming
conversion reads, and its body applies the release operation that conversion's
[representation](05-represent.md#represent-and-compose-values) declared, then delivers nothing. The target answers `boundary`
for it from what the binding [recorded for that type](03-requests.md#what-policy-means) — a symbol for C, an `external` method for Kotlin —
and the registry assembles and checks it exactly as it does a call, minus the
call. [The handle path][typedef_boundary] shows one.

**Delivery** specifies where converted values go. A conversion producing a struct can be reused whether the enclosing function returns that struct, writes it through output parameters, or passes it to a declared callback. The conversion itself need not contain a second version for each destination.

```rust
struct BoundarySpec<Payload> {
    abi: AbiSpec<Payload>,        // Calling convention, symbol and target signature requirements.
    inputs: Vec<InputPlacement>, // How wrapper arguments feed logical input conversions.
    output: OutputPlacement,     // Destinations for encoded success/error values.
    failures: FailureRoutes<Payload>, // Terminal actions for each category of failure.
}

enum OutputPlacement {
    Void, // This path delivers no value.
    Return(ValueMapping), // Deliver converted values through the wrapper return.
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
    boundary: BoundarySpec<Payload>, // Validated wrapper signature and delivery decisions.
    body: FunctionBodyId,     // Registry-owned instructions for the whole wrapper.
    artifacts: Vec<ArtifactId>, // Derived generated prerequisites for the wrapper.
}
```

All field shapes in this block are design sketches, not exact current API
definitions. Current `BoundarySpec` uses a non-generic `AbiSpec`, whose wrapper
parameters carry their placement roles, plus output and failure routes. Current
`FunctionPlan` stores those decisions with a flat instruction list. Names such
as `CalleeId`, `FunctionBodyId`, `SinkId` and `ValueMapping` below belong to the
design vocabulary. Current V2 supports `Void` and a
wrapper return; it does not yet support the `OutParameters`, `Branches` or
`Invoke` cases shown here. A **sink** is a configured recipient of a result,
such as a callback, rather than a wrapper return slot. These distinctions matter
when extending the engine: converting a value and choosing its recipient are
separate decisions.

`AbiSpec` describes the wrapper boundary, including target calling conventions and explicit environment operands such as a JNI environment. It also carries the part of the wrapper's *form* a target may state. The common Rust writer renders every wrapper as `#[no_mangle] <attrs> pub <unsafe> extern "<abi>" fn <symbol>(<params>) -> <ret>`, and what is in angle brackets is the target's: the calling convention, the symbol, the parameters and return, attributes beyond `#[no_mangle]` (a lint the generated signature would trip), and whether the signature is `unsafe`. The linkage is not: the writer exports the wrapper under the symbol, and a target restating it with `#[no_mangle]` or `#[export_name]` is contradictory input that fails generation. Both targets built so far state no attribute and no `unsafe`; V1's JNI wrapper, `#[allow(..)] pub unsafe extern "C"`, is the shape that needed the fields. This is the sketch's `AbiSpec<Payload>` with the payload made concrete, because the writer has to render it and the registry has to check it. Each wrapper parameter carries its role: it feeds one source parameter's conversion, it supplies a named runtime context that operations ask for, or the convention requires it and nothing uses it. That is what `InputPlacement` is as implemented, and it is also how an operation's `Context("jni.env")` operand finds the parameter that satisfies it — a conversion needing a context its boundary does not supply is skipped, with the reason. `ValueMapping` maps converted values/slots to a return, output location, or invocation argument. These are transport descriptions; they do not repeat source decomposition.

`SinkId` identifies a declared destination and signature, not the runtime callback pointer itself. `CalleeId` identifies the source operation being wrapped. `FunctionOutput` identifies the conversions for the wrapped source operation's result. `FunctionBodyId` refers to the complete structured wrapper instructions assembled by the registry.

The error categories distinguish source-domain, binding-conversion and runtime
failures. Current routes are a list of `FailureRoute` values. A route may run a
reporting operation, then either return an expression or abort. Throwing a JVM
exception is the reporting operation in this example, not a separate terminal
variant. Source `Result` delivery and declared handler destinations remain
extensions to this implemented route mechanism.

One route is a reporting operation the target supplies, what to do when reporting
itself fails, and how the route ends — returning an expression or aborting. The
registry emits the branch, the reporting call, the check on its result and the
terminal action; the target contributes only the operation that reports. A
category a conversion can raise and the boundary does not list makes the function
unsupported.

For a source `Result`, `OutputPlacement::Branches` maps the error value to its configured destination, while the domain failure route specifies how that path terminates. Both describe one consistent [boundary convention](03-requests.md#what-policy-means). Conversion failures can also happen before the source call or while encoding its result; those use their binding/runtime routes. Failure while encoding a domain error must itself have a defined route.

The complete design calls for this flow; choosing source `Result` branches and
finishing explicit resource scopes are future steps:

```text
validate and convert inputs
 -> call the source function once
 -> choose the source success/error path
 -> convert the selected value
 -> deliver through the configured destination
 -> finish resource scopes
```

The target declares extra wrapper parameters required by its convention, such
as the JNI environment. The registry binds and validates those parameters
against the operations that need them. A unit result requires no payload.

C and JNI retain configured calling conventions. Unsupported result destinations skip the function; changing its ABI is not a substitute for support.

## Elements at this stage

- [Function taking an owned struct][fn_boundary] · [C][fn_boundary_c] · [Kotlin/JNI][fn_boundary_jni]
- [Type alias declaring an opaque handle][typedef_boundary] · [C][typedef_boundary_c] · [Kotlin/JNI][typedef_boundary_jni]

The struct path has no cell here: [its conversion][struct_represent] is reached
through the function that uses it.

[fn_boundary]: ../examples/fn/06-boundary.md
[fn_boundary_c]: ../examples/fn/06-boundary.c.md
[fn_boundary_jni]: ../examples/fn/06-boundary.jni.md
[struct_represent]: ../examples/struct/05-represent.md
[typedef_boundary]: ../examples/typedef/06-boundary.md
[typedef_boundary_c]: ../examples/typedef/06-boundary.c.md
[typedef_boundary_jni]: ../examples/typedef/06-boundary.jni.md
