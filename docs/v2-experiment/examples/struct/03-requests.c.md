<!-- spec: {"example": "struct", "kind": "variant", "language": "c", "stage": "03-requests"} -->

# Struct: Stamp — Record binding requests — c

[Pipeline chapter](../../stages/03-requests.md) · [Common contract](03-requests.md) · [Example path](README.md)

## Input

Public Stamp declaration and owned record-input rule.

## Owner

C frontend records choices interpreted by C adapter.

## Result

Choose public aggregate `StampC`, repr(C), `secs` then `nanos`, each signed 64-bit. Map each source field identity to its target member identity.

## Checks

Members with identical spellings on another target type are different identities. No opaque handle is selected.

## Related example

[function: c at this stage](../function/03-requests.c.md)
