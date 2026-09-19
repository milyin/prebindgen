<!-- spec: {"kind": "cell", "example": "struct", "stage": "05-represent"} -->

[Stage chapter](../../stages/05-represent.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, on descriptions from the target adapter · Previous: [Select conversion relations][struct_select] · Next: [Retain supported output][struct_retain]

# Struct with scalar fields — Represent and compose values

## Input

The [selection][struct_select] for the struct, with its leaves already planned:

```text
Stamp, IntoRust, relation Stamp.fields
  +-- secs  -> node(i64, IntoRust)     // finished: identity conversion
  +-- nanos -> node(i64, IntoRust)     // the same node
policy:   this target's struct policy
```

## Result

```text
node(Stamp, IntoRust) {
    relation: Stamp.fields
    children: [ node(i64, IntoRust) as "secs",
                node(i64, IntoRust) as "nanos" ]
    repr:     layout   — the target's carrier for one Stamp
              protocol — Product { projections: [read secs, read nanos] }
    body:     apply read secs  to carrier -> convert -> local
              apply read nanos to carrier -> convert -> local
              construct source::Stamp { secs, nanos }
    contract: produces an owned source Stamp, validity Independent
}
```

With both children finished, the registry asks the target for the struct's
[representation](../../stages/05-represent.md#represent-and-compose-values):
which [carrier](../../stages/05-represent.md#describing-target-values-and-operations)
holds a `Stamp` on its side — a C member value or a JVM object here — and one
[primitive](../../stages/05-represent.md#represent-and-compose-values) per
part that reads a field out of it. The [C][struct_represent_c] and
[Kotlin/JNI][struct_represent_jni] pages show those reads. The registry then
composes the body: each read is applied to the carrier, its result is passed
through the child's template, and the converted parts construct the source
struct.

Both field positions resolve to the same cached `i64` input
[node](../../stages/05-represent.md#represent-and-compose-values) under this
[policy](../../stages/03-requests.md#what-policy-means); the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) applies
that plan once per field. Each carrier already has the required Rust `i64`
value, so the child [conversion](../../stages/04-select.md#select-conversion-relations)
is **identity**: it passes the value through without generating another
operation. `Independent` describes the owned result, which borrows neither
input; the full validity-contract structure remains a proposed extension.

## Checks

- The registry checks the number and types of projections against the parts
  of the selected [relation](../../stages/04-select.md#what-a-relation-is);
  the adapter is responsible for associating each read with the intended
  source field.
- Construction follows declaration order, and a read that fails stops the body
  before the construction: no `Stamp` is built from a partial set of fields.
- The node is recorded only once its children exist, so its identity — type,
  direction, relation, policy, children — is complete when it is cached and
  another struct with the same identity shares it.
- This is the struct entering Rust. Leaving Rust needs a construction
  operation on the target side that `Product` does not have yet, and is a
  reported skip.

## Language variants

- [C][struct_represent_c]
- [Kotlin/JNI][struct_represent_jni]

[struct]: README.md
[struct_select]: 04-select.md
[struct_retain]: 07-retain.md
[struct_represent_c]: 05-represent.c.md
[struct_represent_jni]: 05-represent.jni.md
