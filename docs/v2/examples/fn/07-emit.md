<!-- spec: {"kind": "cell", "example": "fn", "stage": "07-emit"} -->

# Function taking an owned record — Emit bindings

[Stage chapter](../../stages/07-emit.md) · [Element path][fn] · [Source crate](../../source.md)

## Input

The frozen function plan, its two conversion nodes and the payloads the target
attached to their operations.

## Owner

The common Rust writer renders the wrapper. It allocates the local names, renders
the registry's instructions in order, and calls the target's operation renderer
for the fragments only that target can produce. The foreign side is then
`cbindgen` for C and the Kotlin writer for JNI.

## Result

One native Rust function per target, plus the foreign declaration that calls it.
The Rust body is the same shape in both: bind each field value to a local,
construct the source record, call `stamp_sum`, return its result. What differs is
the fragment each field read renders to, and the error handling around it.

## Checks

Local names come from the writer, not from any target, and are stable for a given
plan. The source function appears exactly once in the generated body. The writer
adds nothing that planning did not decide: no conversion appears here that has no
node, and no dependency is discovered while rendering.

## Representation

The instructions and what each becomes:

| Instruction | Owner | C | Kotlin/JNI |
| --- | --- | --- | --- |
| read the first member | adapter fragment | `arg0.secs` | `env.call_method(&arg0, "getSecs", "()J", &[])…` |
| bind it to a local | registry | `let v0 = …;` | `let v0 = match … { … };` |
| read the second member | adapter fragment | `arg0.nanos` | `env.call_method(&arg0, "getNanos", "()J", &[])…` |
| construct the record | registry | `source::Stamp { secs: v0, nanos: v1 }` | same |
| call the source once | registry | `source::stamp_sum(v2)` | same |
| deliver the result | boundary | `v3` returned | `v3` returned as `jlong` |

## Language variants

- [C][fn_emit_c]
- [Kotlin/JNI][fn_emit_jni]

## Along this element

Previous: [Retain supported output][fn_retain]

[fn]: README.md
[fn_retain]: 06-retain.md
[fn_emit_c]: 07-emit.c.md
[fn_emit_jni]: 07-emit.jni.md
