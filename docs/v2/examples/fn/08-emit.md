<!-- spec: {"kind": "cell", "example": "fn", "stage": "08-emit"} -->

[Stage chapter](../../stages/08-emit.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the common Rust writer, plus each target's writer · Previous: [Retain supported output][fn_retain]

# Function taking an owned struct — Emit bindings

## Input

The frozen plan for this function:

```text
FunctionPlan { declaration: fn:stamp_sum,
               abi, symbol, params, ret, routes: <from the form and the wire types>,
               instrs: <convert the input, call source::stamp_sum, convert the result>,
               result: <the converted result> }
```

The instructions refer to each
[node](../../stages/05-represent.md#represent-and-compose-values)'s operations.
The registry writes a standard operation itself and hands a target operation
to the target's writer with its operands, such as the part a getter reads. **Frozen** means
support decisions are finished; emission cannot add another [conversion](../../stages/04-select.md#select-conversion-relations). In
current V2, the common writer renders Rust before `generate` returns, and
`Generation` stores that text alongside the descriptions used for Kotlin output.

## Result

One generated Rust function per target, and the foreign declaration that calls it:
[C][fn_emit_c], [Kotlin/JNI][fn_emit_jni]. Both bodies have the same shape, and
each instruction has one owner:

| Instruction | Rendered by | C | Kotlin/JNI |
| --- | --- | --- | --- |
| read the first member | registry for C (`ReadMember`); JNI writer (`Getter`) | `stamp.secs` | `env.call_method(&stamp, "getSecs", "()J", &[])…` |
| bind it to a local | registry | `let v0 = …;` | `let v0 = match … { … };` |
| read the second member | registry for C (`ReadMember`); JNI writer (`Getter`) | `stamp.nanos` | `env.call_method(&stamp, "getNanos", "()J", &[])…` |
| construct the struct | registry | `source::Stamp { secs: v0, nanos: v1 }` | same |
| call the source once | registry | `source::stamp_sum(v2)` | same |
| deliver the result | registry, following the plan's result | `v3` returned | `v3` returned as `jlong` |

Read the table from top to bottom as one call. The input arrives in the target's
form, two integers are obtained, the source struct is built, and the source
function supplies the final integer. JNI additionally branches after each
fallible getter; the language-specific page shows those branches in full.

## Checks

- Temporaries are allocated from the plan, so two operations in one [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
  cannot collide over a name; the parameters are named by the boundary, since a
  target that requires an environment operand has to name it.
- The source function appears exactly once in the generated body.
- The writer adds nothing planning did not decide: no [conversion](../../stages/04-select.md#select-conversion-relations) without a node,
  no dependency discovered while rendering.

## Language variants

- [C][fn_emit_c]
- [Kotlin/JNI][fn_emit_jni]

[fn]: README.md
[fn_retain]: 07-retain.md
[fn_emit_c]: 08-emit.c.md
[fn_emit_jni]: 08-emit.jni.md
