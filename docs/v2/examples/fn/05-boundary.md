<!-- spec: {"kind": "cell", "example": "fn", "stage": "05-boundary"} -->

[Stage chapter](../../stages/05-boundary.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on the target's native interface · Previous: [Plan value conversions][fn_values] · Next: [Retain supported output][fn_retain]

# Function taking an owned record — Assemble the native boundary

## Input

The two conversion nodes, the function policy, and the source signature
`stamp_sum(Stamp) -> i64`.

## Result

```text
FunctionPlan {
    source:   CalleeId(crate::source::stamp_sum),
    inputs:   [ node(input) ],                    // in source parameter order
    output:   FunctionOutput::Single(node(output)),
    boundary: BoundarySpec { … },                 // per target, below
    body:     convert the input
              -> call the source function once
              -> convert the result
              -> deliver through the configured destination
}
```

## Checks

- The source function is called once, and only after every input conversion has
  succeeded.
- Every failure the input node declares needs a route here; a category with no
  route is an unsupported boundary, not a default.
- An unsupported destination skips the function. Its ABI is never quietly
  changed to make it fit.
- `FunctionOutput::Single` is the non-`Result` shape; a fallible source function
  resolves two conversions and a branching `OutputPlacement` instead, which is
  the `fn_fallible` path.

## Language variants

- [C][fn_boundary_c]
- [Kotlin/JNI][fn_boundary_jni]

[fn]: README.md
[fn_values]: 04-values.md
[fn_retain]: 06-retain.md
[fn_boundary_c]: 05-boundary.c.md
[fn_boundary_jni]: 05-boundary.jni.md
