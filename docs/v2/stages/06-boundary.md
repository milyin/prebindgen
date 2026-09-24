<!-- spec: {"kind": "stage", "stage": "06-boundary"} -->

[Project contents](../README.md) · Previous: [Represent and compose values](05-represent.md) · Next: [Retain supported output](07-retain.md)

# Assemble the wrapper boundary

Status: implemented. Delivery is a wrapper return or nothing: out-parameters,
`Result` branches and a result handed to a callback the caller passes in are
variants the engine does not have yet. A callback taken as a parameter is an
input like any other.

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

The binding states that signature's form, as the **function form** it records
with the function's output; the registry connects its parameters to the
conversion plans. Wrapper parameter names come from the form; the common
writer chooses names for internal temporaries.

Three things are decided here. **Where inputs come from**: which wrapper argument
feeds which input conversion, including arguments the convention adds for its
own reasons. **Where the result goes** — the *delivery* — which here is the
wrapper return; delivering it through out-parameters or a callback the caller
passes in is [designed](../extensions.md#multi-value-layouts), and reuses the
same conversion, so a conversion never has one version per destination. **What
happens on failure**: every failure a conversion declared needs a terminal action
here, and the actions differ by category — a `Result::Err` returned by the
source function is not the same event as a JNI call failing mid-conversion.

For example, `getSecs()` can fail before the source function runs. The JNI
form reports that failure to the JVM and returns a placeholder zero from the
wrapper. With an exception pending, the Kotlin caller observes the exception,
not a successful result of zero. If reporting itself fails, the generated code
aborts. The form names the reporting operation, and the registry plans the
branch and terminal action around it. V1's handler-based convention is a
separate implementation and should not be confused with this V2 example.

If a failure has no route, or a value is of a kind the target cannot pass or
return, the function is skipped, with the reason recorded in its
[outcome](07-retain.md#retain-supported-output). It is never quietly given a
different ABI than the one the configuration asked for, because a caller
compiled against the header or the Kotlin declaration would then be calling
something else.

Only exported callables reach this stage. A struct has conversions but no
boundary of its own; it crosses inside the functions that use it.

## Assembling an exported function

A complete binding function coordinates value plans: convert inputs, call the
source once, convert its result and handle failures. Future resource-bearing
conversions will also require cleanup on every relevant exit path.

One exported function has no source function behind it: the release of an
opaque handle. A type output exposing a
[representation](05-represent.md#represent-and-compose-values) with a release
names a function form for it — a symbol for C, an `external` method for Kotlin
— and the registry assembles a wrapper whose one parameter carries the handle,
whose body applies the release operation to it, and which delivers nothing. It
is checked exactly as a call is, minus the call. [The handle path][typedef_boundary]
shows one.

What the binding states about one exported function:

```rust
struct FunctionForm<Op> {
    abi: String,                   // The `extern` string: "C", "system".
    symbol: String,                // The symbol the wrapper is exported under.
    context: Vec<ContextParam>,    // Parameters the convention adds, before the inputs.
    inputs: Vec<syn::Ident>,       // The names of the parameters carrying the inputs.
    routes: Vec<FailureRoute<Op>>, // One per failure category a conversion can raise.
    attrs: Vec<syn::Attribute>,    // Attributes beyond `#[no_mangle]`.
    unsafety: bool,                // Whether the wrapper is an `unsafe fn`.
}

struct ContextParam {
    name: syn::Ident,
    ty: syn::Type,
    supplies: Option<String>, // The runtime context it supplies, or none: the
                              // convention requires it and nothing uses it.
    mutable: bool,
}

struct FailureRoute<Op> {
    category: FailureCategory,
    report: Option<Report<Op>>,  // An operation reporting the error, and the error
                                 // type it takes.
    on_report_failure: Terminal, // What to do when reporting itself fails.
    terminate: Terminal,         // How the route ends: return an expression, or abort.
}
```

And what the registry makes of it, together with the planned conversions:

```rust
struct FunctionPlan<Op> {
    declaration: Declaration,              // The declaration this wrapper exports.
    abi: String,
    symbol: String,
    attrs: Vec<syn::Attribute>,
    unsafety: bool,
    params: Vec<(ValueId, WrapperParam)>,  // The context parameters, then one per input,
                                           // each typed as its conversion's wire type.
    ret: Option<syn::Type>,                // The result conversion's wire type, if any.
    routes: Vec<FailureRoute<Op>>,
    instrs: Vec<Stmt>,                     // The whole wrapper, as instructions.
    result: Option<ValueId>,               // The value the wrapper returns.
}
```

The common Rust writer renders every wrapper as
`#[no_mangle] <attrs> pub <unsafe> extern "<abi>" fn <symbol>(<params>) -> <ret>`.
What is in angle brackets comes from the form and the plan: the calling
convention, the symbol, the parameters and return, attributes beyond
`#[no_mangle]` (a lint the generated signature would trip), and whether the
signature is `unsafe`. The linkage is not the binding's to state: the writer
exports the wrapper under the symbol, and a form restating it with
`#[no_mangle]` or `#[export_name]` is contradictory input that fails
generation. Both targets built so far state no attribute and no `unsafe`; V1's
JNI wrapper, `#[allow(..)] pub unsafe extern "C"`, is the shape that needed the
fields. The registry makes a wrapper `unsafe` on its own account when one of
its conversions reads a
[wire type](05-represent.md#describing-target-values-and-operations)'s bits, as
C's enum input does: that caller
owes initialized storage.

A wrapper parameter's type is not the binding's to state either. Each input's
parameter is typed as the wire type its conversion resolved to, so a
parameter and the conversion reading it cannot disagree. Which kinds of wire
type a wrapper can take and return at all is not a binding's choice but a
fact about the target, its `PARAMS` and `RETURNS`. A context parameter supplies a named runtime context
that operations ask for — an operation's `jni.env` finds the parameter that
supplies it, and one that needs a context no parameter supplies is skipped,
with the reason.

A parameter that takes a callback is placed like any input: its conversion
captures the callable and builds the closure the source function receives.
What the wrapper routes is what capturing can raise — a JNI global reference
that could not be taken, say. A failure inside a later call of the closure
never reaches the wrapper, which may have returned by then; it takes a route
the callback's representation states, and a call has no runtime context to
report with. Planning refuses a callback whose calls can fail in a category
its own routes do not cover, as `unsupported.callback.unrouted_failure`, and
one whose calls, or whose reporters, need a runtime context, as
`unsupported.callback.missing_context`.

The error categories distinguish source-domain, binding-conversion and runtime
failures. A route may run a reporting operation, then either return an
expression or abort. Throwing a JVM exception is the reporting operation in
this example, not a separate terminal variant. The registry emits the branch,
the reporting call, the check on its result and the terminal action; the
target writes only the reporting operation, fed the error and the contexts it
asked for. The registry also checks that a route reports the error type the
operation raises, since a mismatch would not compile. A category a conversion
can raise and the form does not route makes the function unsupported.

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

C and JNI retain configured calling conventions. An unsupported destination
skips the function; changing its ABI is not a substitute for support.

## Elements at this stage

- [Function taking an owned struct][fn_boundary] · [C][fn_boundary_c] · [Kotlin/JNI][fn_boundary_jni]
- [Type alias declaring an opaque handle][typedef_boundary] · [C][typedef_boundary_c] · [Kotlin/JNI][typedef_boundary_jni]
- [Function taking a callback][fn_callback_boundary] · [C][fn_callback_boundary_c] · [Kotlin/JNI][fn_callback_boundary_jni]

The struct path has no cell here: [its conversion][struct_represent] is reached
through the function that uses it.

[fn_boundary]: ../examples/fn/06-boundary.md
[fn_boundary_c]: ../examples/fn/06-boundary.c.md
[fn_boundary_jni]: ../examples/fn/06-boundary.jni.md
[struct_represent]: ../examples/struct/05-represent.md
[typedef_boundary]: ../examples/typedef/06-boundary.md
[typedef_boundary_c]: ../examples/typedef/06-boundary.c.md
[typedef_boundary_jni]: ../examples/typedef/06-boundary.jni.md
[fn_callback_boundary]: ../examples/fn_callback/06-boundary.md
[fn_callback_boundary_c]: ../examples/fn_callback/06-boundary.c.md
[fn_callback_boundary_jni]: ../examples/fn_callback/06-boundary.jni.md
