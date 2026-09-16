<!-- spec: {"kind": "stage", "stage": "07-emit"} -->

[Project contents](../README.md) · Previous: [Retain supported output](06-retain.md) · Next: [Implementation and acceptance](../implementation.md)

# Emit bindings

The Rust and Kotlin for this example are emitted through the V2 paths in the
real C and JNI frontends. `examples/v2check` compiles the generated Rust and
compares it with the examples in these pages; it also checks the Kotlin text.
That test does not yet load the library in a JVM or execute the Kotlin API.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

The previous stage selected complete supported output and froze the plans.
**Emission** now turns those plans into source files. A writer does not retry
unsupported requests or invent another representation. It renders the decisions
already made, which keeps the C/JNI declarations consistent with their Rust
implementations. Three contributions combine to produce the files:

**Native Rust** is generated for both targets by the same component, the common
Rust writer, which belongs to the registry. It walks the frozen instructions and
renders them: the locals, their order, the branches on failure, the construction
of the source value, the call, the return. It also allocates the wrapper's temporaries — `v0`, `v1`, … — from the plan, so
two operations rendered into the same wrapper cannot collide over a name. The
wrapper's *parameters* are the exception, and deliberately so: they are named in
the boundary description, because a target that requires an environment operand
has to be able to say what that operand is called in the signature it dictated.

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

**The foreign declaration** is what the other language compiles against, and
writing it belongs to the adapter for that language: nothing else knows what a
declaration in it should look like. The JNI adapter therefore renders the data
class, the `external fun` on its harness object and the Kotlin function that
calls it, from retained public descriptions. The adapter derives public
properties and native getter names from the same source fields and uses the
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
That definition is a generated **artifact**, a unit of Rust contributed by the
target, such as a helper or a public type definition. The JNI adapter supplies the helper and declares
the dependency; the common writer emits its use in the error branch.

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
`stampSum(stamp: Stamp): Long`, delegating to the native method the JVM binds
the wrapper to. The native method lives on one harness object, `JNINative`,
whichever package the function is declared in — the `Java_…` symbol names that
object, so the two have to agree — and because the record crosses as an object,
it takes the data class itself. A future separate-arguments representation
would give the native method two `Long` arguments and make the public Kotlin
wrapper read them from `Stamp`. That representation is not implemented in this
V2 increment. It illustrates why a public wrapper may eventually do more than
delegate when the native signature differs from the API Kotlin callers use.

## What lands on disk

The generated Rust is written into the binding crate's build output directory and
pulled into the crate the ordinary way:

```rust
// src/lib.rs of the binding crate
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

Everything the registry retained is emitted into that file: the wrappers, the
type declarations the target needed, and any generated helper an operation
depended on. Current generation collects and deduplicates the supporting Rust
items, then emits them before wrappers; it has no general artifact-dependency
sort. The C build then runs `cbindgen` over the crate to
produce the header a C program includes. The JNI build instead runs its Kotlin
writer, which writes `.kt` files into a source directory the build script names,
where the Kotlin compiler finds them; checking those into the repository is a
reasonable choice, since it makes a change in the generated API visible in
review.

Generation and execution have different dependencies. The generator runs in
the build script. The compiled wrapper calls the source crate and any runtime
support required by its conversions. `prebindgen-c-runtime` and
`prebindgen-jni-runtime` provide that support, such as C opaque-value traits and
JNI string/byte-array helpers. The simple `Stamp` example needs only a small
subset of the overall binding machinery; it does not establish V2 support for
all the conversions provided by those runtime crates.

The report is published alongside the code. It says which declarations were
emitted or skipped and why. Selecting example test sections from that report
is planned work; current text checks are not a substitute for executing JNI.

The [element paths](#elements-at-this-stage) below show each generated file in
full, including the parts elided above.

## What each writer contributes

The common Rust writer owns the shape of the generated Rust: the `let` bindings,
the order of operations, the branches on failure, the source construction, the
source call, and the return. Local names are allocated centrally from plan
identities, so no target invents a variable name and no two operations collide.

A target contributes fragments and declarations, never control flow. The C
adapter contributes `repr(C)`, the extern calling convention, the exported symbol
and member identities; its member reads render through a common Rust operation,
so C ships no field-read renderer of its own. The JNI adapter contributes the JNI
symbol and calling convention, the environment and class parameters, the carrier
types, the getter descriptors and the error policy — and a renderer for the JNI
operations, which produces one expression per operation and nothing around it.

The public declarations follow the same division: the JNI adapter renders them
from retained descriptions derived from the same source fields and naming
configuration as its native operations. C declarations are expressed as
generated Rust and left to `cbindgen`.

## From description to generated code

The operation payload is only one field of `PrimitiveSpec`. The other fields
let the registry validate where and how that operation can be used. These
concrete contributions stay separate throughout planning and writing:

| Contribution | C aggregate | Kotlin/JNI object | Component responsible |
| --- | --- | --- | --- |
| Source facts | Two `i64` fields and `stamp_sum(Stamp) -> i64` | Same source facts | Flat |
| Requested public API | Explicit C type name `Stamp`; default function name `stamp_sum` | `example.Stamp`, `example.stampSum` | Language frontend records user choices. |
| Target representation | `repr(C)` struct with members | JVM object, getters and JNI integer carriers | Target adapter describes it from policy and direct child descriptors. |
| `PrimitiveSpec.implementation` | Common `StandardOp::ReadMember` plus member identity | `JniPayload::Getter` plus getter name and descriptor | Adapter selects operation; registry retains it. |
| One primitive's rendered operation | `stamp.secs` | `env.call_method(...).and_then(...)` | Common Rust operation renderer for C; JNI operation renderer for the getter. |
| Primitive application and result use | `let v0 = ...` | `let v0 = match ...` with error path | Registry plans instructions; common writer renders them. |
| Source construction and call | `source::Stamp { ... }`, then `stamp_sum` | Same source instructions | Registry plans; common Rust writer renders. |
| Error-reporting operation | Not needed by these field reads | Runtime helper using `exception_check` and `throw_new` | JNI supplies operation; registry places it and handles its failure. |
| Public foreign source | Header derived from Rust | Kotlin classes and native declaration | `cbindgen` for C; JNI's Kotlin writer for Kotlin. |

For the input record, the registry asks the selected [relation](04-values.md#what-a-relation-is) for its fields,
resolves the child conversions, and asks the target for a representation using
those child descriptions. The target returns the member/getter mappings and
primitive specifications. The registry registers their definitions, creates
applications with concrete operand identities, composes the wrapper and freezes
the result. Writers then render that result without discovering new conversions.

Adding a third supported field makes the registry visit another source child
and apply the same composition algorithm. The target describes one more member
or getter through its existing local representation interface. The target does
not need another handwritten record converter or wrapper-assembly algorithm.

The rendered output of each contribution above appears in the element paths: the
[C wrapper and header][fn_emit_c], the [Kotlin object and JNI wrapper][fn_emit_jni],
the [C aggregate][struct_emit_c] and the [Kotlin data class][struct_emit_jni].

## Elements at this stage

- [Function taking an owned record][fn_emit] · [C][fn_emit_c] · [Kotlin/JNI][fn_emit_jni]
- [Record with scalar fields][struct_emit] · [C][struct_emit_c] · [Kotlin/JNI][struct_emit_jni]

[fn_emit]: ../examples/fn/07-emit.md
[fn_emit_c]: ../examples/fn/07-emit.c.md
[fn_emit_jni]: ../examples/fn/07-emit.jni.md
[struct_emit]: ../examples/struct/07-emit.md
[struct_emit_c]: ../examples/struct/07-emit.c.md
[struct_emit_jni]: ../examples/struct/07-emit.jni.md
