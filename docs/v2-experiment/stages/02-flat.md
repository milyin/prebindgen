<!-- spec: {"kind": "stage", "stage": "02-flat"} -->

# Build and inspect Flat

[Experiment contents](../README.md)

## Input

Flat receives the complete captures and any frontend-declared local helper signatures.

## Operation and owner

Flat lowers the source into typed records, validates source references, and publishes one immutable snapshot. A snapshot is a completed source model retained by read-only views. Function lookup returns FunctionView directly; parameter, field and result navigation returns TypeView in the same snapshot. The registry is not involved in inspection.

## Output and failure contract

The output is Flat plus checked views, source locations and explicit unsupported-item records. Keys derive from retained type readings; keys do not construct views. Reject a view from a different snapshot before registry planning. Capture acceptance does not imply that a binding conversion is supported.

## Apply this stage

- [Function: stamp_sum](../examples/function/02-flat.md)
- [Struct: Stamp](../examples/struct/02-flat.md)

## Pipeline navigation

[Previous](01-source.md) [Next](03-requests.md)
