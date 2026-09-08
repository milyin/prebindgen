<!-- spec: {"example": "function", "kind": "variant", "language": "c", "stage": "03-requests"} -->

# Function: stamp_sum — Record binding requests — c

[Pipeline chapter](../../stages/03-requests.md) · [Common contract](03-requests.md) · [Example path](README.md)

## Input

Function request for stamp_sum; the struct rule selects StampC by-value input.

## Owner

C frontend.

## Result

Record public symbol `stamp_sum_c`, extern C convention, one by-value StampC parameter, native i64 return, and infallible input conversion.

## Checks

Do not choose an out-parameter or alter ownership. The source argument is owned even though member reads borrow the temporary carrier locally.

## Related example

[struct: c at this stage](../struct/03-requests.c.md)
