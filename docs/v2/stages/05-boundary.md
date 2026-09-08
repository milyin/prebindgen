<!-- spec: {"kind": "stage", "stage": "05-boundary"} -->

# Assemble the native boundary

[Project contents](../README.md)

## Input

A requested source call has resolved input/output conversions. The adapter supplies its native calling convention, parameter placement, return placement and failure actions.

## Operation and owner

The registry builds a FunctionPlan: obtain input carriers, execute conversions, call the source once, encode the selected result, deliver it, and finish scopes. Primitive renderers supply local operations; the registry owns surrounding branches and early termination. Declaring a public type does not itself create a native callable.

## Output and failure contract

The result is a complete candidate function body with required generated artifacts. A missing boundary convention or failure route skips the function, even when its input conversions are ready. Failure paths must never execute later success operations or expose unavailable result values.

## Detailed contracts

- [Conversion and exported-function plans](05-boundary/plans.md)

## Apply this stage

- [Function: stamp_sum](../examples/function/05-boundary.md) · [c](../examples/function/05-boundary.c.md) · [kotlin](../examples/function/05-boundary.kotlin.md)

## Pipeline navigation

[Previous](04-values.md) [Next](06-retain.md)
