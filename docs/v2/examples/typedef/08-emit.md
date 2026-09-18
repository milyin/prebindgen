<!-- spec: {"kind": "cell", "example": "typedef", "stage": "08-emit"} -->

[Stage chapter](../../stages/08-emit.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the common Rust writer, then `cbindgen` or the Kotlin writer · Previous: [Retain supported output][typedef_retain]

# Type alias declaring an opaque handle — Emit bindings

## Input

The frozen public declaration for this alias, and the frozen release plan:

```text
SurfaceSpec  { declaration: public Ledger, payload: <the target's declaration metadata> }
FunctionPlan { declaration: public Ledger, boundary: <the release symbol and carrier>,
               body: [ apply Release(this_) ] }
```

with the two [conversion](../../stages/04-select.md#select-conversion-relations)
[nodes](../../stages/05-represent.md#represent-and-compose-values), whose
standard operations the writer renders inside the
[wrappers](../../stages/06-boundary.md#assemble-the-native-boundary) of
`ledger_open` and `ledger_close`.

## Result

One public type per target, one release per target, and one expression in each
wrapper that touches the handle:

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct Ledger { _private: [u8; 0] }` | none; the value is a `jlong` |
| Foreign declaration | header `typedef struct Ledger Ledger;` | `class Ledger(ptr: Long)` with `take()` and `free()` |
| Release | `void ledger_drop(Ledger *this_)` | `JNINative.freeLedger(ptr: Long)`, called by `free()` |
| Handing out, in `ledger_open` | `Box::into_raw(Box::new(v3)) as *mut Ledger` | the same, cast to `jlong`; Kotlin wraps the `Long` in a `Ledger` |
| Taking back, in `ledger_close` | `NonNull::new(ledger as *mut source::Ledger)…` then abort on `Err` | the same, then throw on `Err`; Kotlin passes `ledger.take()` |

Rendered: [C][typedef_emit_c], [Kotlin/JNI][typedef_emit_jni].

Every expression that names `source::Ledger` is the registry's; the adapters
contributed the [carrier](../../stages/05-represent.md#describing-target-values-and-operations)
spelling (`*mut Ledger`, `jlong`) and the release symbol, and no Rust of their
own beyond the incomplete C type. The release wrapper has no `let`, no call
and no return value: one statement, then the end of the function.

## Checks

- The writer emits the release only if the type was retained, and the type
  only if the release was planned — the two arrive together or not at all.
- A wrapper consuming a handle binds the taken value like any owned local, so
  on the failure path of a *later* conversion in the same wrapper the value is
  dropped: a handle passed by value is consumed by the call whether or not the
  call succeeds, as a Rust argument is.
- A `#[cfg]` on the alias reaches every wrapper that spells `source::Ledger`,
  the release included, by the rule a wrapper follows for the items it names.

## Language variants

- [C][typedef_emit_c]
- [Kotlin/JNI][typedef_emit_jni]

[typedef]: README.md
[typedef_retain]: 07-retain.md
[typedef_emit_c]: 08-emit.c.md
[typedef_emit_jni]: 08-emit.jni.md
