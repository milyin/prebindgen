<!-- spec: {"kind": "cell", "example": "fn", "stage": "07-emit"} -->

[Stage chapter](../../stages/07-emit.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the common Rust writer, plus each target's writer · Previous: [Retain supported output][fn_retain]

# Function taking an owned record — Emit bindings

## Input

The frozen function plan, its two conversion nodes, and the payloads the target
attached to their operations.

## Result

One native Rust function per target, and the foreign declaration that calls it:
[C][fn_emit_c], [Kotlin/JNI][fn_emit_jni]. Both bodies have the same shape, and
each instruction has one owner:

| Instruction | Rendered by | C | Kotlin/JNI |
| --- | --- | --- | --- |
| read the first member | adapter fragment | `arg0.secs` | `env.call_method(&arg0, "getSecs", "()J", &[])…` |
| bind it to a local | registry | `let v0 = …;` | `let v0 = match … { … };` |
| read the second member | adapter fragment | `arg0.nanos` | `env.call_method(&arg0, "getNanos", "()J", &[])…` |
| construct the record | registry | `source::Stamp { secs: v0, nanos: v1 }` | same |
| call the source once | registry | `source::stamp_sum(v2)` | same |
| deliver the result | boundary | `v3` returned | `v3` returned as `jlong` |

## Checks

- Temporaries are allocated from the plan, so two operations in one wrapper
  cannot collide over a name; the parameters are named by the boundary, since a
  target that requires an environment operand has to name it.
- The source function appears exactly once in the generated body.
- The writer adds nothing planning did not decide: no conversion without a node,
  no dependency discovered while rendering.

## Language variants

- [C][fn_emit_c]
- [Kotlin/JNI][fn_emit_jni]

[fn]: README.md
[fn_retain]: 06-retain.md
[fn_emit_c]: 07-emit.c.md
[fn_emit_jni]: 07-emit.jni.md
