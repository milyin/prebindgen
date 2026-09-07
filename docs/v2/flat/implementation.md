# Flat V2 implementation and acceptance

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Binding-generation companion: [Registry V2](../registry/overview.md).

## Implementation sequence and acceptance

Implement this API incrementally in `prebindgen-flat`. A V2 module can coexist
with the current API during migration; `Flat` in this document denotes that V2
model. This proposal does not require a new standalone crate or an immediate
breaking rewrite of V1 consumers.

1. Add builder-to-snapshot ownership, helper registration checks and direct
   function lookup/enumeration. Preserve unsupported records and locations.
2. Add parameter/result `TypeView`s, exact record navigation and field views.
   Implement the independent inspection example using existing captured inputs.
3. Add snapshot checks and checked type composition. Have V2 registry requests
   retain views, and derive cache keys privately from those views.
4. Extend constants, enums and other type forms using the same conventions when
   their consumers need them. Existing inputs remain retained and reportable
   throughout; unsupported structural access is not a reason to lose an item.

Required validation for implementation:

- A standalone Flat consumer navigates function → parameter → record → field
  without a registry dependency or source reparsing.
- Lookup and enumeration agree; views remain usable after the original `Flat`
  value is dropped, and cloned views refer to the same snapshot.
- A view from another snapshot is rejected even when both snapshots contain a
  type with the same name and key.
- External callers cannot forge views or mutate their retained records.
- Local helpers are validated before publication, including duplicate names and
  source-module qualification; existing opaque helper types remain representable.
- Reference, optional and fallible navigation preserves wrappers and exact child
  types. Modeled lifetime arguments survive field navigation and emission.
- Opaque items, unsupported records, guards and locations survive migration.
- Derived type views preserve model association without changing source-item
  enumeration, and type/key consistency holds by construction.
- Flat tests validate source inspection and emission. The registry project's
  C/JNI tests separately validate conversion behavior and ownership.

---

Previous: [C and Kotlin examples](../registry/primitive-examples.md) · Next: [Registry implementation](../registry/implementation.md)
