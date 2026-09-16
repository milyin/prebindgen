<!-- spec: {"kind": "cell", "example": "fn", "stage": "07-emit"} -->

[Stage chapter](../../stages/07-emit.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the common Rust writer, plus each target's writer · Previous: [Retain supported output][fn_retain]

# Function taking an owned record — Emit bindings

## Input

The frozen plan for this function:

```text
FunctionPlan { source: crate::source::stamp_sum,
               inputs: [ node(input) ], output: Single(node(output)),
               boundary: <symbol, convention, placements, failure routes>,
               body: <instructions> }
```

The summary also depends on each
[node](../../stages/04-values.md#plan-value-conversions)'s operations and the
adapter's retained rendering data, such as getter names. **Frozen** means
support decisions are finished; emission cannot add another [conversion](../../stages/04-values.md#plan-value-conversions). In
current V2, the common writer renders Rust before `generate` returns, and
`Generation` stores that text alongside the descriptions used for Kotlin output.

## Result

One native Rust function per target, and the foreign declaration that calls it:
[C][fn_emit_c], [Kotlin/JNI][fn_emit_jni]. Both bodies have the same shape, and
each instruction has one owner:

| Instruction | Rendered by | C | Kotlin/JNI |
| --- | --- | --- | --- |
| read the first member | common operation for C; adapter expression for JNI | `stamp.secs` | `env.call_method(&stamp, "getSecs", "()J", &[])…` |
| bind it to a local | registry | `let v0 = …;` | `let v0 = match … { … };` |
| read the second member | common operation for C; adapter expression for JNI | `stamp.nanos` | `env.call_method(&stamp, "getNanos", "()J", &[])…` |
| construct the record | registry | `source::Stamp { secs: v0, nanos: v1 }` | same |
| call the source once | registry | `source::stamp_sum(v2)` | same |
| deliver the result | common writer, following the boundary plan | `v3` returned | `v3` returned as `jlong` |

Read the table from top to bottom as one call. The input arrives in the target's
form, two integers are obtained, the source record is built, and the source
function supplies the final integer. JNI additionally branches after each
fallible getter; the language-specific page shows those branches in full.

## Checks

- Temporaries are allocated from the plan, so two operations in one [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary)
  cannot collide over a name; the parameters are named by the boundary, since a
  target that requires an environment operand has to name it.
- The source function appears exactly once in the generated body.
- The writer adds nothing planning did not decide: no [conversion](../../stages/04-values.md#plan-value-conversions) without a node,
  no dependency discovered while rendering.

## Language variants

- [C][fn_emit_c]
- [Kotlin/JNI][fn_emit_jni]

[fn]: README.md
[fn_retain]: 06-retain.md
[fn_emit_c]: 07-emit.c.md
[fn_emit_jni]: 07-emit.jni.md
