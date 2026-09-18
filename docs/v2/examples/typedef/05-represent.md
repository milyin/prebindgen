<!-- spec: {"kind": "cell", "example": "typedef", "stage": "05-represent"} -->

[Stage chapter](../../stages/05-represent.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry, on descriptions from the target adapter · Previous: [Select conversion relations][typedef_select] · Next: [Assemble the native boundary][typedef_boundary]

# Type alias declaring an opaque handle — Represent and compose values

## Input

The two [selections][typedef_select], each with nothing under it:

```text
Ledger, IntoRust,  relation atomic
Ledger, OutOfRust, relation atomic
policy:   this target's handle policy
```

## Result

Two [nodes](../../stages/05-represent.md#represent-and-compose-values), one
per direction, and a release that rides with the first:

```text
node(Ledger, IntoRust) {
    relation: atomic
    children: []
    repr:     layout   — Scalar(<the target's carrier for an address>)
              protocol — Terminal(FromRaw)          // *Box::from_raw(v as *mut source::Ledger)
              release  — Release                     // drop(Box::from_raw(..)); null releases nothing
    body:     apply FromRaw to carrier -> owned source Ledger
    contract: produces an owned source Ledger; the handle is consumed
    failures: { Binding: String }                    // a null address
}

node(Ledger, OutOfRust) {
    relation: atomic
    children: []
    repr:     layout   — Scalar(<the same carrier>)
              protocol — Terminal(IntoRaw)          // Box::into_raw(Box::new(v)) as <carrier>
    body:     apply IntoRaw to the source value -> carrier
    contract: produces the carrier; the foreign side owns the allocation
    failures: {}
}
```

With no children to wait for, the registry asks the target for each
[representation](../../stages/05-represent.md#represent-and-compose-values)
at once: which
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
holds an address on its side, and which
[primitive](../../stages/05-represent.md#represent-and-compose-values)
turns a source value into one and back. All three primitives — `IntoRaw`,
`FromRaw`, `Release` — are standard ones the registry renders itself, beside
`Identity` and `ReadMember`. They are the registry's because each spells a
*source* type, `*mut source::Ledger`, and only the registry may do that; what
the target contributes is the carrier the address is cast to and from, and
nothing else. [C][typedef_represent_c] and [Kotlin/JNI][typedef_represent_jni]
each name theirs.

A `Release` produces no value, so it is never part of a [conversion](../../stages/04-select.md#select-conversion-relations) body. It is
stated on the into-Rust representation — the one whose carrier it takes — and
the registry plans it as a
[wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) of its own
at [the next stage][typedef_boundary]. Naming a release is also what tells the
registry the type is a handle: a representation the foreign side owes nothing
for names none.

## Checks

- A null address into Rust is a failure of the *binding* category, carrying a
  `String`: the caller broke the contract, not the JVM and not the source
  function. The boundary has to route it like any other; a release is
  infallible and null is a no-op there, as with `free(NULL)`.
- Ownership is discharged by construction, without a resource contract: handing
  out is the last operation of a wrapper that returns the carrier, taking back
  moves the value into an ordinary owned local that Rust drops on every path,
  and releasing is a wrapper of its own that converts nothing. Nothing acquires
  a resource that a later failing operation could leak. A borrowed handle
  (`&Ledger`) breaks this argument, and is what
  [the extensions page](../../extensions.md) reserves the contract for.
- The same two nodes serve the type's own request, `ledger_open` and
  `ledger_close`: node identity is (type, direction, [relation](../../stages/04-select.md#what-a-relation-is), effective
  [policy](../../stages/03-requests.md#what-policy-means)), and all three name the same policy.

## Language variants

- [C][typedef_represent_c]
- [Kotlin/JNI][typedef_represent_jni]

[typedef]: README.md
[typedef_select]: 04-select.md
[typedef_boundary]: 06-boundary.md
[typedef_represent_c]: 05-represent.c.md
[typedef_represent_jni]: 05-represent.jni.md
