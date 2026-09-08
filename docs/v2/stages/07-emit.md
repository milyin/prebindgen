<!-- spec: {"kind": "stage", "stage": "07-emit"} -->

# Emit bindings

[Project contents](../README.md)

Status: proposed design. Code shown as generated output illustrates required
behavior, not bytes produced by the current scaffold.

Everything is decided by now. The last stage turns the frozen result into files:
native Rust for both targets, a C header derived from that Rust, and Kotlin
declarations rendered from the same plans.

**Input.** The frozen `Generation`: retained value plans, function plans, public
declarations, generated artifacts in emission order, and the payloads the targets
attached to them.

**Owner.** The common Rust writer, which is part of the registry library and
renders every native wrapper and supporting Rust type. It allocates the local
names, renders the registry's instructions, and calls a target's operation
renderer for the fragments only that target can produce. C then runs the external
`cbindgen` tool over the generated Rust; JNI runs its own Kotlin writer over the
same `Generation`.

**Output.** The generated Rust module, the C header, the Kotlin sources, and the
report published alongside them.

**Failure.** None of its own that concerns support. A writer cannot discover a new
conversion, add a dependency or change a support decision — anything it would
need was decided before freezing, and its absence at this point is a generator
defect, not a skip.

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

For C there is no foreign writer at all. The public C API is expressed as
generated Rust types and functions, and `cbindgen` derives the header from them;
there is no generated C implementation file, because the function body is the
Rust wrapper. For Kotlin the JNI implementation renders the public declarations
directly from the retained plans, using the same class metadata the primitive
renderer uses, so a naming override reaches both consistently.

## From description to generated code

The operation payload is only one field of `PrimitiveSpec`. The other fields
let the registry validate where and how that operation can be used. These
concrete contributions stay separate throughout planning and writing:

| Contribution | C aggregate | Kotlin/JNI object | Component responsible |
| --- | --- | --- | --- |
| Source facts | Two `i64` fields and `stamp_sum(Stamp) -> i64` | Same source facts | Flat |
| Requested public API | `StampC`, `stamp_sum_c` | `example.Stamp`, `Bindings.sum` | Language frontend records user choices. |
| Target representation | `repr(C)` struct with members | JVM object, getters and JNI integer carriers | Target adapter describes it from policy and direct child descriptors. |
| `PrimitiveSpec.implementation` | Common `ReadMember` plus member identity | `CallLongGetter` plus getter metadata | Adapter selects payload; registry retains it. |
| One primitive's rendered operation | `arg0.secs` | `env.call_method(...).and_then(...)` | Common Rust operation renderer for C; JNI operation renderer for the getter. |
| Primitive application and result use | `let v0 = ...` | `let v0 = match ...` with error path | Registry plans instructions; common writer renders them. |
| Source construction and call | `source::Stamp { ... }`, then `stamp_sum` | Same source instructions | Registry plans; common Rust writer renders. |
| Error-reporting operation | Not needed by these field reads | Runtime helper using `exception_check` and `throw_new` | JNI supplies operation; registry places it and handles its failure. |
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

---

Previous: [Retain supported output](06-retain.md) · Next: [Implementation and acceptance](../implementation.md)

[fn_emit]: ../examples/fn/07-emit.md
[fn_emit_c]: ../examples/fn/07-emit.c.md
[fn_emit_jni]: ../examples/fn/07-emit.jni.md
[struct_emit]: ../examples/struct/07-emit.md
[struct_emit_c]: ../examples/struct/07-emit.c.md
[struct_emit_jni]: ../examples/struct/07-emit.jni.md
