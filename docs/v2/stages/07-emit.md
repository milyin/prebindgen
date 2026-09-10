<!-- spec: {"kind": "stage", "stage": "07-emit"} -->

[Project contents](../README.md) · Previous: [Retain supported output](06-retain.md) · Next: [Implementation and acceptance](../implementation.md)

# Emit bindings

Status: proposed design. Code shown as generated output illustrates required
behavior, not bytes produced by the current scaffold.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

Everything is decided by now; what is left is producing files. There are three
kinds of output, and understanding which component makes each is most of this
stage.

**Native Rust** is generated for both targets by the same component, the common
Rust writer, which belongs to the registry. It walks the frozen instructions and
renders them: the locals, their order, the branches on failure, the construction
of the source value, the call, the return. It also allocates the wrapper's temporaries — `v0`, `v1`, … — from the plan, so
two operations rendered into the same wrapper cannot collide over a name. The
wrapper's *parameters* are the exception, and deliberately so: they are named in
the boundary description, because a target that requires an environment operand
has to be able to say what that operand is called in the signature it dictated.

**A fragment inside that Rust** is the one thing a target contributes, and only
where the operation is target-specific. Reading a C aggregate member renders as
`arg0.secs` through an operation the registry library already provides, so the C
adapter ships no renderer at all. Reading a JVM property renders as
`env.call_method(&arg0, "getSecs", "()J", &[]).and_then(|value| value.j())`,
which only the JNI adapter can produce. A fragment is one expression: this one
evaluates to a `Result`, and the `and_then` is part of performing the read, not
part of handling its failure. Nothing decides what happens when that `Result` is
an error — no `match` on it, no early return — because deciding that is the
wrapper's job, and the wrapper is the registry's.

**The foreign declaration** is what the other language compiles against, and
writing it belongs to the adapter for that language: nothing else knows what a
declaration in it should look like. The JNI adapter therefore renders the data
class and the `external fun` itself, from the same retained plans, using the same
class metadata that the property-read operations used — so a renamed getter moves
in both places or neither.

C is the exception, and for a practical reason rather than an architectural one:
`cbindgen` already derives C headers from Rust source and is the established way
to do it. The public C API is expressed as generated Rust types and functions,
the build runs `cbindgen` over them, and the C adapter writes nothing. There is
no generated C source file either — the body of every C entry point *is* the Rust
wrapper.

Put together, the JNI wrapper for the example comes out like this — every line
attributable to one of the three contributions above:

```rust
#[no_mangle]
pub unsafe extern "C" fn Java_example_JNINative_stampSum<'a>(  // symbol: JNI adapter
    mut env: jni::JNIEnv<'a>,                                  // environment: JNI adapter
    _class: jni::objects::JClass<'a>,
    arg0: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,                   // the caller's error handler
) -> jni::sys::jlong {
    let v0 = match env.call_method(&arg0, "getSecs", "()J", &[])
        .and_then(|value| value.j())                // fragment: JNI adapter
    {                                               // everything else: registry
        Ok(value) => value,
        Err(error) => {                             // the route: JNI adapter
            signal_binding_error(&mut env, &__error_sink, …, &error.to_string());
            return 0;
        }
    };
    let v1 = match env.call_method(&arg0, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            signal_binding_error(&mut env, &__error_sink, …, &error.to_string());
            return 0;
        }
    };
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

Both reads render the same way and neither name collides, because the
temporaries came from the plan rather than from the operation.
`signal_binding_error` is neither a fragment nor part of the wrapper: it is a
runtime helper the JNI side provides, called through a described operation whose
dependencies name what it needs — a cached identifier for the handler's method
among them — which is how retention knows to keep those.

The C wrapper for the same function is the same shape with the branches gone,
because its member reads cannot fail:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(arg0: Stamp) -> i64 {
    let v0 = arg0.secs;
    let v1 = arg0.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

On the Kotlin side, the public API here is a single declaration: because the
record crosses as an object, the `external fun` can take the data class itself,
and there is nothing to wrap. Had the configuration chosen to pass the two fields
as separate JNI arguments, the native declaration would take two `Long`s, and the
writer would add a Kotlin method taking a `Stamp` in front of it — that is the
"typed wrapper" the component table mentions, and it exists only when the native
signature is not the one Kotlin callers should see.

## What lands on disk

The generated Rust is written into the binding crate's build output directory and
pulled into the crate the ordinary way:

```rust
// src/lib.rs of the binding crate
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

Everything the registry retained is emitted into that file: the wrappers, the
type declarations the target needed, and any generated helper an operation
depended on — several such units per file, in the order retention fixed, so a
declaration precedes its uses. The C build then runs `cbindgen` over the crate to
produce the header a C program includes. The JNI build instead runs its Kotlin
writer, which writes `.kt` files into a source directory the build script names,
where the Kotlin compiler finds them; checking those into the repository is a
reasonable choice, since it makes a change in the generated API visible in
review.

What the generated code calls at run time is not the generator. Beyond the
source crate itself, a wrapper reaches for the small runtime crate that matches
its target — `prebindgen-c-runtime` or `prebindgen-jni-runtime` — for the
operations that are the same in every binding: encoding a string or a byte array
for the JVM, the binding error type, the traits behind an opaque value passed by
value across the C ABI. A binding crate therefore depends on a few hundred lines
at run time and on the generator only while building.

The report is published alongside the code. It is what says which requested
elements were emitted, which were skipped and why — and, for the example crates,
which test sections may be compiled against this output.

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
from the retained plans, with the same class metadata its primitive renderer
uses, so a naming override reaches both consistently; C's are expressed as
generated Rust and left to `cbindgen`.

## From description to generated code

The operation payload is only one field of `PrimitiveSpec`. The other fields
let the registry validate where and how that operation can be used. These
concrete contributions stay separate throughout planning and writing:

| Contribution | C aggregate | Kotlin/JNI object | Component responsible |
| --- | --- | --- | --- |
| Source facts | Two `i64` fields and `stamp_sum(Stamp) -> i64` | Same source facts | Flat |
| Requested public API | `Stamp`, `stamp_sum` — the source names, since nothing renamed them | `example.Stamp`, the top-level `example.stampSum` | Language frontend records user choices. |
| Target representation | `repr(C)` struct with members | JVM object, getters and JNI integer carriers | Target adapter describes it from policy and direct child descriptors. |
| `PrimitiveSpec.implementation` | Common `ReadMember` plus member identity | `CallLongGetter` plus getter metadata | Adapter selects payload; registry retains it. |
| One primitive's rendered operation | `arg0.secs` | `env.call_method(...).and_then(...)` | Common Rust operation renderer for C; JNI operation renderer for the getter. |
| Primitive application and result use | `let v0 = ...` | `let v0 = match ...` with error path | Registry plans instructions; common writer renders them. |
| Source construction and call | `source::Stamp { ... }`, then `stamp_sum` | Same source instructions | Registry plans; common Rust writer renders. |
| Error-reporting operation | Not needed by these field reads | Signalling the caller's error handler | JNI supplies the operation and the convention; the registry places it. |
| Public foreign source | Header derived from Rust | Kotlin classes and native declaration | `cbindgen` for C; JNI's Kotlin writer for Kotlin. |

For the input record, the registry asks the selected relation for its fields,
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
