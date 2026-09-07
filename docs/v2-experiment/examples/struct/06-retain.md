<!-- spec: {"example": "struct", "kind": "cell", "stage": "06-retain"} -->

# Struct: Stamp — Retain supported output

[Pipeline chapter](../../stages/06-retain.md) · [Example path](README.md)

## Input

The record's candidate public declaration contains two required scalar fields. The separate record conversion can also be required by the function.

## Owner

The registry resolves public-declaration and conversion requirements separately.

## Result

Retain the public type when both field representations and its declared semantic promises are supported. Retain the input conversion when a retained function needs it. A missing public field representation removes the entire public type and callers depending on it. A missing input-only conversion operation need not remove a still-valid public type.

## Checks

For this fixture, removing the function root must not remove the independently requested public type. No half-record with only one field may be emitted. Shared helpers survive only when retained outputs require them. Failure of a JNI getter operation can skip the function while the Kotlin data class remains supported.


## Along this example

[Previous](04-values.md) [Next](07-emit.md)

## Related example

[function at this stage](../function/06-retain.md)
