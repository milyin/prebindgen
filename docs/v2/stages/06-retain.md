<!-- spec: {"kind": "stage", "stage": "06-retain"} -->

# Retain supported output

[Project contents](../README.md)

## Input

Candidate conversions, function plans, public declarations and generated artifacts form a dependency graph. An artifact is a generated type, helper or wrapper that another output may require.

## Operation and owner

The registry propagates unsupported requirements and retains only complete public outputs plus all of their required artifacts. Association is not automatically a requirement: an optional method can be omitted without removing its class. A promised interface member is required. The registry freezes retained plans, referenced source views, primitive/layout/body records and outcome reports.

## Output and failure contract

Each requested element is emitted, skipped with causes, or explicitly ignored. An unselected source item is reported separately. A ready conversion alone is not an emitted public declaration. After freezing, writers cannot discover missing conversions or change the retained set. Language-specific dependencies are concrete in the preceding and following example cells; closure is a common algorithm.

## Detailed contracts

- [Support, registry state and output](06-retain/generation.md)

## Apply this stage

- [Function: stamp_sum](../examples/function/06-retain.md)
- [Struct: Stamp](../examples/struct/06-retain.md)

## Pipeline navigation

[Previous](05-boundary.md) [Next](07-emit.md)
