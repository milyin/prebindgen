<!-- spec: {"kind": "cell", "example": "fn", "stage": "05-boundary"} -->

[Stage chapter](../../stages/05-boundary.md) · [Element path][fn] · [Source crate](../../source.md)
Previous: [Plan value conversions][fn_values] · Next: [Retain supported output][fn_retain]

# Function taking an owned record — Assemble the native boundary

## Input

The two conversion nodes from [value planning][fn_values], the requested
function's policy, and the source signature `stamp_sum(Stamp) -> i64`.

## Owner

The registry assembles and validates the wrapper. The adapter describes the
native interface: symbol, calling convention, any synthetic parameters, where the
result goes, and the terminal action for each failure category.

## Result

One `FunctionPlan`. Its callee is `crate::source::stamp_sum`; its inputs are the
single input node in source parameter order; its output is the scalar node
delivered through the native return. Its body is the complete wrapper: convert
the input, call the source function exactly once, convert the result, return it.

The plan is where the exported signature becomes fixed — the native arguments,
their order, the synthetic ones the target requires, and the return type.

## Checks

The source function is called exactly once, and only after every input conversion
has succeeded. A conversion failure never reaches the call. The return value is
delivered where the policy says: an unsupported destination skips the function
rather than quietly changing its ABI. Failures the input node declares must each
have a route; a category with no route is an unsupported boundary, not a default.

## Representation

```text
validate and convert inputs
 -> call the source function once
 -> convert the result
 -> deliver through the native return
```

```text
FunctionPlan {
  source:  CalleeId(crate::source::stamp_sum),
  inputs:  [ node(input) ],           // owned Stamp, in parameter order
  output:  FunctionOutput::Single(node(output)),
  boundary: BoundarySpec { ... },     // per target, below
}
```

`FunctionOutput::Single` is the shape for a non-`Result` return. A fallible source
function would instead resolve two conversions and a branching
`OutputPlacement`; that is the `fn_fallible` path, not this one.

## Language variants

- [C][fn_boundary_c]
- [Kotlin/JNI][fn_boundary_jni]

[fn]: README.md
[fn_values]: 04-values.md
[fn_retain]: 06-retain.md
[fn_boundary_c]: 05-boundary.c.md
[fn_boundary_jni]: 05-boundary.jni.md
