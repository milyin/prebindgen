<!-- spec: {"kind": "cell", "example": "fn", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Record binding requests][fn_requests] · Next: [Represent and compose values][fn_represent]

# Function taking an owned struct — Select conversion relations

## Input

```text
Crossing { source: Stamp, direction: IntoRust  }    // at `param stamp`
Crossing { source: i64,   direction: OutOfRust }    // at `return`
rules:    Type(Stamp) -> this target's Stamp representation, read through its fields
          Type(i64)   -> this target's i64 representation, whole
```

and, for each value, the position it sits at, which is what a rule at a
position would be looked up by. This binding records none, so every value
takes its type's rule.

## Result

The function needs two [conversions](../../stages/04-select.md#select-conversion-relations):
foreign argument to Rust `Stamp`, and Rust result to foreign integer. For each,
the registry looks up the rule that applies — the one at the value's position,
else its type's — and resolves the
[relation](../../stages/04-select.md#what-a-relation-is) its
[representation](../../stages/05-represent.md#represent-and-compose-values) names; then it descends into that relation's parts and looks up again. No
target code runs. What comes back up is a selection tree, each entry carrying
the representation the later stages compare:

```text
   param stamp --> Stamp, IntoRust
                     relation: Stamp.fields    // what the rule's `Fields` names
                     +-- field secs  --> i64, IntoRust,  relation: atomic
                     +-- field nanos --> i64, IntoRust,  relation: atomic

   return      --> i64, OutOfRust
                     relation: atomic          // a `Terminal` representation
```

Nothing in the tree says how a field is read. It says that `Stamp` will be built
from two fields — so the `Stamp` [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
has two members, each carried as the `i64` rule says — and that an `i64` is a
leaf, converted whole. The [C][fn_select_c] and [Kotlin/JNI][fn_select_jni]
pages show each frontend's rules.

The `Stamp` subtree is the struct's own selection,
[made on its own path][struct_select]; this function refers to it, as does
anything else taking an owned `Stamp` under the same rule.

## Checks

- Selection happens before any part is inspected: the rule says `Fields` from
  the type alone, and only then does the registry read the struct's fields. A
  rule making `Stamp` a handle would leave them unread.
- The two `i64` leaves are one conversion rather than two: the same
  [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan)
  and the same representation id — the identity the registry compares, and
  can compare itself.
- If any part's conversion is unsupported — a field of a type no rule covers —
  the `Stamp` conversion is unsupported, and so is this function, with that
  cause and the path to it. Nothing partial is recorded.
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
