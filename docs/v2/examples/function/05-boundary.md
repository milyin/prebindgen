<!-- spec: {"example": "function", "kind": "cell", "stage": "05-boundary"} -->

# Function: stamp_sum — Assemble the native boundary

[Pipeline chapter](../../stages/05-boundary.md) · [Example path](README.md)

## Input

The registry has the function request, input/output NodeIds and the adapter's boundary description.

## Owner

The registry assembles the wrapper; the adapter specifies calling convention, placement and failure actions.

## Result

The FunctionPlan performs input conversion, calls `source::stamp_sum` once on success, and delivers the encoded `i64` through the native return. Any input runtime failure terminates its path before the source call. Native environment/class parameters are ABI requirements, not source parameters.

## Checks

Check operand placement and return types against resolved carriers. The source consumes the constructed Stamp. Unit, Result, callbacks, out-parameters and borrowed inputs are not part of this fixture; they require separate paths. The C and Kotlin variants below fix this fixture's exact ABI and error behavior.

## Language variants

- [c](05-boundary.c.md)
- [kotlin](05-boundary.kotlin.md)

## Along this example

[Previous](04-values.md) [Next](06-retain.md)
