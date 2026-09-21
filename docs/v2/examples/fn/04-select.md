<!-- spec: {"kind": "cell", "example": "fn", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on the target adapter's selections · Previous: [Record binding requests][fn_requests] · Next: [Represent and compose values][fn_represent]

# Function taking an owned struct — Select conversion relations

## Input

```text
Crossing { source: Stamp, direction: IntoRust  }    // for Param(0)
Crossing { source: i64,   direction: OutOfRust }    // for Return
```

and, for each, the position it sits at. The
[policy](../../stages/03-requests.md#what-policy-means) recorded for that type
and position is not passed in: the target holds it, and looks it up.

## Result

The function needs two [conversions](../../stages/04-select.md#select-conversion-relations):
foreign argument to Rust `Stamp`, and Rust result to foreign integer. For each,
the registry offers the target the [relations](../../stages/04-select.md#what-a-relation-is)
registered for the type and asks it to choose one. The target answers with the
relation *and* a [conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan)
standing for the policy it applied; the registry then descends into that
relation's parts and asks again. What comes back up is a selection tree, each
entry carrying a key the later stages compare:

```text
   Param(0) --> Stamp, IntoRust
                  relation: Stamp.fields       // chosen from { Stamp.fields, atomic }
                  +-- part secs  --> i64, IntoRust,  relation: atomic
                  +-- part nanos --> i64, IntoRust,  relation: atomic

   Return   --> i64, OutOfRust
                  relation: atomic             // the only relation of a scalar
```

Nothing in the tree says how a `Stamp` arrives or how a field is read. It says
that `Stamp` will be built from two fields — so the next stage has to describe a
[carrier](../../stages/05-represent.md#describing-target-values-and-operations) with two members and a read for each — and that an `i64` is a leaf, so
the next stage converts it whole. The [C][fn_select_c] and
[Kotlin/JNI][fn_select_jni] pages show why each target chose the struct
relation here, and what it would choose instead.

The `Stamp` subtree is the struct's own selection,
[made on its own path][struct_select]; this function refers to it, as does
anything else taking an owned `Stamp` under the same policy.

## Checks

- Selection happens before any part is inspected: the target chooses
  `Stamp.fields` from the type and the policy it holds for it, and only then
  does the registry read the struct's fields. A target that chose `atomic`
  would leave them unread.
- A scalar offers one relation. The target does not get to invent a second;
  `select` must return one of the ids it was offered.
- The key returned with the relation is what makes the two `i64` leaves one
  conversion rather than two: the same
  [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan),
  the same relation, an equal key. A target that minted a fresh key per call
  would plan each of them separately and would not detect the recursion the
  last check names.
- If any part's conversion is unsupported — a field of a type nothing can carry
  yet — the `Stamp` conversion is unsupported, and so is this function, with
  that cause. Nothing partial is recorded.
- `Stamp` does not contain itself, so no position in the tree meets a
  conversion that is still open; a self-referential struct would be refused
  here with `unsupported.conversion.recursive`.

## Language variants

- [C][fn_select_c]
- [Kotlin/JNI][fn_select_jni]

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_represent]: 05-represent.md
[fn_select_c]: 04-select.c.md
[fn_select_jni]: 04-select.jni.md
[struct_select]: ../struct/04-select.md
