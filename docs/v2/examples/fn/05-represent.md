<!-- spec: {"kind": "cell", "example": "fn", "stage": "05-represent"} -->

[Stage chapter](../../stages/05-represent.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on descriptions from the target adapter · Previous: [Select conversion relations][fn_select] · Next: [Assemble the wrapper boundary][fn_boundary]

# Function taking an owned struct — Represent and compose values

## Input

The [selection tree][fn_select] the previous stage left, read from the leaves:

```text
   Param(0) --> Stamp, IntoRust,  relation Stamp.fields
                  +-- secs  --> i64, IntoRust,  atomic
                  +-- nanos --> i64, IntoRust,  atomic
   Return   --> i64, OutOfRust, atomic
```

## Result

Each [conversion](../../stages/04-select.md#select-conversion-relations) in the
tree becomes a reusable plan, called a
[node](../../stages/05-represent.md#represent-and-compose-values). The leaves
are finished first, because a parent's identity and its
[representation](../../stages/05-represent.md#represent-and-compose-values)
both depend on its children. Neither plan calls `stamp_sum`; the next stage places
that source call between them.

```text
node(input)  = Crossing { Stamp, IntoRust }
               relation: Stamp.fields
               children: [ node(i64, IntoRust), node(i64, IntoRust) ] // same node twice
               repr:     the target's carrier for Stamp, with one read per part
               body:     obtain secs carrier -> convert
                         obtain nanos carrier -> convert
                         construct source::Stamp { secs, nanos }

node(output) = Crossing { i64, OutOfRust }
               relation: atomic
               children: []
               repr:     the target's scalar carrier
               body:     identity — source i64 and the target carrier are one value
```

Taken together the two plans are a small graph — three nodes, and one edge per
part, with both fields landing on the same child:

```text
   Param(0) --> node(input)  Stamp, IntoRust, Stamp.fields
                     |
                     +-- secs  --+
                     |           +--> node(i64, IntoRust, atomic)
                     +-- nanos --+

   Return   --> node(output) i64, OutOfRust, atomic
```

The two `i64` plans stay apart because direction is part of a node's identity,
while the two fields share one plan because everything in theirs agrees. The
selection tree had two `i64` leaves under `Stamp`; the cache, consulted once
each leaf is complete, made them one node.

The input node is the struct's own conversion,
[planned on its own path][struct_represent]; this function refers to it, and so
does anything else taking an owned `Stamp` under the same
[policy](../../stages/03-requests.md#what-policy-means). Their `NodeId`s are
what [the boundary][fn_boundary] assembles.

## Checks

- Reuse depends on type, direction, [relation](../../stages/04-select.md#what-a-relation-is),
  effective policy and child conversions. A second owned `Stamp` input with
  the same choices can reuse this plan. C and JNI run separate generation jobs.
- The cache is checked after the children exist, not before: the key includes
  them.
- An unsupported child made the input conversion unsupported in the previous
  stage; nothing reaches this one for a refused subtree, and no partial node is
  recorded.

## Language variants

- [C][fn_represent_c]
- [Kotlin/JNI][fn_represent_jni]

[fn]: README.md
[fn_select]: 04-select.md
[fn_boundary]: 06-boundary.md
[fn_represent_c]: 05-represent.c.md
[fn_represent_jni]: 05-represent.jni.md
[struct_represent]: ../struct/05-represent.md
