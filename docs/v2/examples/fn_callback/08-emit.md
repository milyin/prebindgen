<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "08-emit"} -->

[Stage chapter](../../stages/08-emit.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the common Rust writer, then `cbindgen` or the Kotlin writer · Previous: [Retain supported output][fn_callback_retain]

# Function taking a callback — Emit bindings

## Input

The retained callback and the frozen plan of the function taking it:

```text
Retained     { output: callback:impl Fn(i64)+Send+Sync+'static, output_value: node(each) }
               wire type: the callable wire type, with its argument wire types as members
FunctionPlan { declaration: fn:stamp_each,
               instrs: <convert stamp, capture each and build the closure, call source::stamp_each> }
```

## Result

One declaration for the callable per target, and one [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary). The wrapper's
body has the shape of [the function path's][fn_emit], with one statement more:
the closure.

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct closure_i64 { context, call, drop }`, with `Drop` calling `drop` | none; the callable is a JVM object |
| Foreign declaration | header `typedef struct closure_i64 { … } closure_i64;` | `public fun interface LongCallback { fun run(value: Long) }` |
| Capture, in the wrapper | none | `let v3 = match (|| { … })() { … }` over the JVM, a global reference and `run` |
| The closure | `let v4 = move \|v3: i64\| { … call(v3, closure.context) … };` | `let v5 = move \|v4: i64\| { … call_method_unchecked(…) … };` |
| A failure inside a call | none can happen | written to standard error; the call returns |

Rendered: [C][fn_callback_emit_c], [Kotlin/JNI][fn_callback_emit_jni].

The common writer renders the closure's body as it renders a wrapper's —
`let` bindings, operations, failure branches — with the callback's routes in
place of the form's. The closure's parameter is typed as the source type,
`i64`, because it is what the source function calls it with; its argument
leaves Rust inside.

## Checks

- The closure is `move`: it owns the capture, so it lives as long as the
  source function keeps it, independently of the wrapper's frame.
- The closure is `Send + Sync + 'static` because what it owns is: the C
  closure struct declares itself both, and the JNI capture holds a `JavaVM`, a
  `GlobalRef` and a method id, which are. A capture that were not would fail to
  compile against the source function's bounds rather than run unsoundly.
- Nothing is emitted for a refused callback, and nothing for a function taking
  one.

## Language variants

- [C][fn_callback_emit_c]
- [Kotlin/JNI][fn_callback_emit_jni]

[fn_callback]: README.md
[fn_callback_retain]: 07-retain.md
[fn_callback_emit_c]: 08-emit.c.md
[fn_callback_emit_jni]: 08-emit.jni.md
[fn_emit]: ../fn/08-emit.md
