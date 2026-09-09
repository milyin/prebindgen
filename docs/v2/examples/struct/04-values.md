<!-- spec: {"kind": "cell", "example": "struct", "stage": "04-values"} -->

# Record with scalar fields — Plan value conversions

[Stage chapter](../../stages/04-values.md) · [Element path][struct] · [Source crate](../../source.md)
Previous: [Record binding requests][struct_requests] · Next: [Retain supported output][struct_retain]

## Input

The crossing `Stamp` in the `IntoRust` direction, its selected relationship
`Stamp.fields`, and the effective policy for the record.

## Owner

The registry. It selects the relationship first, then asks Flat for the parts of
what was selected, then recursively plans a conversion for each part, and only
then asks the adapter to represent the whole — passing it the completed child
descriptions. The adapter answers about carriers and operations; it never
enumerates the fields itself.

## Result

One conversion node for the record, with two child nodes for `i64` in the same
direction. Its body obtains each part's carrier, converts it, and constructs
`source::Stamp { secs, nanos }`. The node's contract says it produces an owned
source `Stamp`, independent of anything the caller holds afterwards.

The child conversions are shared, not private to this record: an `i64` crossing
in the same direction under the same policy is one node wherever it occurs.

## Checks

Selection precedes traversal. Had the policy selected an opaque representation,
the fields would never have been inspected — which is what makes a record with
unreadable private fields representable as a handle. The parts come from the
selected relationship, so a constructor relationship would give one `millis`
argument instead of two fields, with no change to this stage's algorithm. Field
order in the construction is the declaration's order, and the mapping from parts
to target carriers is validated: a member or property the representation does not
account for is an error.

An unsupported child makes this node unsupported, which propagates to
[everything that requires the record][struct_retain] rather than producing a
record missing a field.

## Representation

```text
node(Stamp, IntoRust) {
  relation: Stamp.fields
  children: [ node(i64, IntoRust) as "secs",
              node(i64, IntoRust) as "nanos" ]
  body:     obtain secs carrier -> convert -> local
            obtain nanos carrier -> convert -> local
            construct source::Stamp { secs, nanos }
  contract: produces owned source Stamp, Independent
}
```

The two `i64` children are identity conversions in both targets: the source type
and the target carrier are the same Rust value, so they render no code of their
own. They still exist as nodes, because the record's representation refers to its
children by identity, and a child whose type needed real work would slot into the
same place.

## Language variants

- [C][struct_values_c]
- [Kotlin/JNI][struct_values_jni]

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_retain]: 06-retain.md
[struct_values_c]: 04-values.c.md
[struct_values_jni]: 04-values.jni.md
