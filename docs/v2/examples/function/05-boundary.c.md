<!-- spec: {"example": "function", "kind": "variant", "language": "c", "stage": "05-boundary"} -->

# Function: stamp_sum — Assemble the native boundary — c

[Pipeline chapter](../../stages/05-boundary.md) · [Common contract](05-boundary.md) · [Example path](README.md)

## Input

Input node needs one StampC carrier; output node provides i64.

## Owner

C adapter supplies ABI description; registry assembles FunctionPlan.

## Result

Signature: `extern "C" fn stamp_sum_c(arg0: StampC) -> i64`. Body order: read secs, read nanos, construct source Stamp, call source stamp_sum, return its scalar value.

## Checks

No environment, class parameter, error helper, output pointer or runtime branch is needed. Check field reads against the target aggregate before emission.
