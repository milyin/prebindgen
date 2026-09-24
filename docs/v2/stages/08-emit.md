<!-- spec: {"kind": "stage", "stage": "08-emit"} -->

[Project contents](../README.md) · Previous: [Retain supported output](07-retain.md) · Next: [Implementation and acceptance](../implementation.md)

# Emit bindings

The Rust and Kotlin for this example are emitted through the V2 paths in the
real C and JNI frontends. `examples/v2check` compiles the generated Rust and
compares it with the examples in these pages; it also checks the Kotlin text.
That test does not yet load the library in a JVM or execute the Kotlin API.

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

The previous stage selected complete supported output and froze the plans.
**Emission** now turns those plans into source files. A writer does not retry
unsupported requests or invent another [representation](05-represent.md#represent-and-compose-values). It renders the decisions
already made, which keeps the C/JNI declarations consistent with their Rust
implementations. Three contributions combine to produce the files:

**Generated Rust** is produced for both targets by the same component, the common
Rust writer, which belongs to the registry. It walks the frozen instructions and
renders them: the locals, their order, the branches on failure, the construction
of the source value, the call, the return. It also allocates the [wrapper](06-boundary.md#assemble-the-wrapper-boundary)'s temporaries — `v0`, `v1`, … — from the plan, so
two operations rendered into the same wrapper cannot collide over a name. The
wrapper's *parameters* are the exception, and deliberately so: they are named in
the function form, because a convention that requires an environment operand
has to be able to say what that operand is called in the signature it states.

It also states the wrapper's own condition. Some
[source items](01-source.md#capture-source-items) arrive still carrying a
`#[cfg]` the
[capture reader could not answer](01-source.md#the-binding-crate-decides). A
wrapper names the items it calls and constructs and compiles only where all of
them exist, so it carries the condition of each, and a field written under one
puts it on the read of that field and on the initializer that consumes the read.
The writer never asks what a condition says: Rust conjoins repeated `#[cfg]`
attributes on one item, so carrying them side by side is the whole of it. The
condition holds on the Rust side only — the C prototype and the Kotlin
`external fun` for the same function are generated whatever it says, which
[the capture stage](01-source.md#the-binding-crate-decides) prices.

**Operation expressions** supply the target-specific parts of the Rust body.
Reading a C aggregate member renders as
`stamp.secs` through an operation the registry library already provides, so the C
adapter ships no renderer at all. Reading a JVM property renders as
`env.call_method(&stamp, "getSecs", "()J", &[]).and_then(|value| value.j())`,
which the JNI adapter knows how to produce. This expression returns a `Result`:
`call_method` invokes the getter and `and_then` extracts its long value if the
call succeeded. The expression does not decide how the exported function
reports an error. The common writer adds that control flow from the boundary
plan, including the `match` and early return shown below.

A target's own Rust — the `repr(C)` struct below — is conditioned the same way,
by the registry rather than by the target: the declaration carries the condition
of the item it was requested for, and each mirrored member carries the condition
of the field it mirrors. A target that declares members per field asks the
registry for those conditions and puts them where its members go; it neither
reads them nor decides anything by them.

**The foreign-language source** is what the other language compiles against —
a Kotlin class, a C prototype — and writing it belongs to the adapter for that
language: nothing else knows what such a thing should look like. The JNI adapter therefore renders the data
class, the `external fun` on its harness object and the Kotlin function that
calls it, from retained public descriptions. The adapter derives public
properties and JNI getter names from the same source fields and uses the
configured class name consistently. Tests check the agreement between those outputs.

C is the exception, and for a practical reason rather than an architectural one:
`cbindgen` already derives C headers from Rust source and is the established way
to do it. The public C API is expressed as generated Rust types and functions,
the build runs `cbindgen` over them, and the C adapter writes nothing. There is
no generated C source file either — the body of every C entry point *is* the Rust
wrapper.

The following JNI wrapper shows how these contributions meet. The adapter
describes the JNI signature and getter calls. The common writer names the
temporaries and surrounds each fallible read with the planned error path:

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampSum(  // symbol: JNI adapter
    mut env: jni::JNIEnv<'_>,                            // environment: JNI adapter
    _this: jni::objects::JObject<'_>,                    // the harness singleton
    stamp: jni::objects::JObject<'_>,                    // the source parameter's name
) -> jni::sys::jlong {
    let v0 = match env.call_method(&stamp, "getSecs", "()J", &[])
        .and_then(|value| value.j())                     // fragment: JNI adapter
    {                                                    // everything else: registry
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v1 = match env.call_method(&stamp, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

`v0` and `v1` hold the two integers, `v2` is the reconstructed Rust `Stamp`, and
`v3` is the source function's result. A failure returns before construction or
the source call. Central allocation gives each temporary a distinct name.

The call to `report_jni_error` needs a helper definition elsewhere in the module.
That definition is a generated [artifact](05-represent.md#individual-target-operations).
The JNI writer returns it beside the text of the reporting
[primitive](05-represent.md#represent-and-compose-values) that calls it, and the
common writer emits it once, before the wrappers, and the call in the planned
error branch. A binding whose operations never report a runtime failure never
writes the call, and so never emits the helper.

The C wrapper for the same function is the same shape with the branches gone,
because its member reads cannot fail:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64 {
    let v0 = stamp.secs;
    let v1 = stamp.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

On the Kotlin side, the public API is one function per exported function,
`stampSum(stamp: Stamp): Long`, delegating to the `external` method the JVM
binds the wrapper to. That method lives on one harness object, `JNINative`,
whichever package the function is declared in — the `Java_…` symbol names that
object, so the two have to agree — and because the struct crosses as an object,
it takes the data class itself. A future separate-arguments representation
would give the `external` method two `Long` arguments and make the public Kotlin
wrapper read them from `Stamp`. That representation is not implemented in this
V2 increment. It illustrates why a public wrapper may eventually do more than
delegate when the wrapper signature differs from the API Kotlin callers use.

## What lands on disk

The generated Rust is written into the binding crate's build output directory and
pulled into the crate the ordinary way:

```rust
// src/lib.rs of the binding crate
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

Everything the registry retained is emitted into that file: the wrappers, the
declarations the retained types'
[carriers](05-represent.md#describing-target-values-and-operations) need, and
any generated helper an
operation's text needed. The declarations come first, in the order the types
were declared, then the helpers, each once, then the wrappers; there is no
general dependency sort. Ahead of all of them go the model's guards — the
[feature assertions](01-source.md#capture-source-items) the capture reader
injected, which belong to no declaration and are emitted whatever this run
retained. The C build then runs `cbindgen` over the crate to
produce the header a C program includes. The JNI build instead runs its Kotlin
writer, which writes `.kt` files into a source directory the build script names,
where the Kotlin compiler finds them; checking those into the repository is a
reasonable choice, since it makes a change in the generated API visible in
review.

Generation and execution have different dependencies. The generator runs in
the build script. The compiled wrapper calls the source crate and any runtime
support required by its [conversions](04-select.md#select-conversion-relations). `prebindgen-c-runtime` and
`prebindgen-jni-runtime` provide that support, such as C opaque-value traits and
JNI string/byte-array helpers. The simple `Stamp` example needs only a small
subset of the overall binding machinery; it does not establish V2 support for
all the conversions provided by those runtime crates.

What the engine could not generate comes back beside the code as
`Generation::skipped`, each entry naming the capability that stopped it. It is
diagnostic output, not an input to generation, and what a build script makes
of it is its own business.

The [element paths](#elements-at-this-stage) below show each generated file in
full, including the parts elided above.

## What each writer contributes

The common Rust writer owns the shape of the generated Rust: the `let` bindings,
the order of operations, the branches on failure, the source construction, the
source call, and the return. Local names are allocated centrally from plan
identities, so no target invents a variable name and no two operations collide.

A target contributes fragments and declarations, never control flow, and
decides nothing while it writes. The C frontend states the extern calling
convention, the exported symbols and the carriers; its member
reads are standard operations the registry writes, so what the C target writes
is only its carriers' declarations — the `repr(C)` struct, the incomplete type
behind a handle, the enum mirror. The JNI frontend states the JNI symbols and
calling convention, the environment and receiver parameters, the carriers and
the error [convention](03-requests.md#what-a-choice-records); the JNI target
writes the JNI operations, one expression per operation and nothing around it.

The public declarations follow the same division: the JNI frontend's Kotlin
writer renders them from the retained outputs, their metadata and the carriers
their values resolved to. C declarations are expressed as generated Rust and
left to `cbindgen`.

## From description to generated code

These concrete contributions stay separate throughout planning and writing:

| Contribution | C aggregate | Kotlin/JNI object | Component responsible |
| --- | --- | --- | --- |
| Source facts | Two `i64` fields and `stamp_sum(Stamp) -> i64` | Same source facts | Flat |
| Requested public API | Explicit C type name `Stamp`; default function name `stamp_sum` | `example.Stamp`, `example.stampSum` | Language frontend records user choices. |
| Target [representation](05-represent.md#represent-and-compose-values) | A `Product` over a `repr(C)` `Stamp` carrier | A `Product` over a `JObject` carrier, read by getters | Language frontend states it in the binding. |
| The read of one part | `StandardOp::ReadMember` | `JniOp::Getter` | Language frontend states it; registry applies it once per part. |
| One primitive's written operation | `stamp.secs` | `env.call_method(...).and_then(...)` | Common Rust writer for C's standard read; the JNI target's writer for the getter, fed the part and its carrier. |
| A handle's operations | `Box::into_raw(Box::new(v3)) as *mut Ledger`; `NonNull::new(ledger as *mut source::Ledger)…` | The same, cast to and from `jlong` | Common Rust operation renderer: these spell a source type, which only the registry may. |
| Primitive application and result use | `let v0 = ...` | `let v0 = match ...` with error path | Registry plans instructions; common writer renders them. |
| Source construction and call | `source::Stamp { ... }`, then `stamp_sum` | Same source instructions | Registry plans; common Rust writer renders. |
| Error-reporting operation | Not needed by these field reads | Runtime helper using `exception_check` and `throw_new` | JNI supplies operation; registry places it and handles its failure. |
| Public foreign source | Header derived from Rust | Kotlin classes and `external` declarations | `cbindgen` for C; JNI's Kotlin writer for Kotlin. |

For the input struct, the registry reads the fields of the
[relation](04-select.md#what-a-relation-is) the rule named, resolves the child
[conversions](04-select.md#select-conversion-relations), and applies the
representation's `read` once per part. It registers each application with what
its writer will be fed, composes the wrapper and freezes the result. Writers
then render that result without discovering new conversions.

Adding a third supported field makes the registry visit another source child
and apply the same composition algorithm. The binding states nothing new, and
the target writes one more member or getter from what it is fed. Neither needs
another handwritten struct converter or wrapper-assembly algorithm.

The rendered output of each contribution above appears in the element paths: the
[C wrapper and header][fn_emit_c], the [Kotlin object and JNI wrapper][fn_emit_jni],
the [C aggregate][struct_emit_c], the [Kotlin data class][struct_emit_jni], and
the handle's [incomplete C type and release][typedef_emit_c] and
[Kotlin class and release][typedef_emit_jni].

## Elements at this stage

- [Function taking an owned struct][fn_emit] · [C][fn_emit_c] · [Kotlin/JNI][fn_emit_jni]
- [Struct with scalar fields][struct_emit] · [C][struct_emit_c] · [Kotlin/JNI][struct_emit_jni]
- [Type alias declaring an opaque handle][typedef_emit] · [C][typedef_emit_c] · [Kotlin/JNI][typedef_emit_jni]

[fn_emit]: ../examples/fn/08-emit.md
[fn_emit_c]: ../examples/fn/08-emit.c.md
[fn_emit_jni]: ../examples/fn/08-emit.jni.md
[struct_emit]: ../examples/struct/08-emit.md
[struct_emit_c]: ../examples/struct/08-emit.c.md
[struct_emit_jni]: ../examples/struct/08-emit.jni.md
[typedef_emit]: ../examples/typedef/08-emit.md
[typedef_emit_c]: ../examples/typedef/08-emit.c.md
[typedef_emit_jni]: ../examples/typedef/08-emit.jni.md
