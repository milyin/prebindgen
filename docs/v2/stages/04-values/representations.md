<!-- spec: {"contract": "representations", "kind": "contract", "stage": "04-values"} -->

[Pipeline chapter](../04-values.md) · [Project contents](../../README.md)

# Target representations and composition

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

A target representation tells the registry which values carry converted data
and how to access them using [primitive operations](target-operations.md). The
[C and Kotlin examples](../07-emit/primitive-examples.md) show those operations in generated code.

## Representation shape and composition protocol

A **layout** describes the values contained in a representation. A **protocol** describes the target operations through which the registry reads or constructs those values. Neither specifies how to recursively convert the corresponding Rust fields; the registry supplies that algorithm.

```rust
enum Layout {
    Empty,                 // No carried values, as for a unit result.
    Scalar(WireTypeId),     // One scalar/reference/carrier value.
    Slots(Vec<SlotSpec>),   // Several independent ordered values.
    Aggregate {
        ty: WireTypeId,            // Type of the one containing struct/object carrier.
        members: Vec<MemberLayout>,// Named/indexed members and their child layouts.
    },
}

struct SlotSpec {
    id: SlotId,          // This value's identity within the layout.
    wire: WireTypeId,    // Target or intermediate type of the value.
    role: SlotRole,      // Payload, presence flag, variant selector, etc.
    active_when: GuardId,// Condition under which its payload can be converted.
}

struct ReprSpec<Payload> {
    layout: Layout,      // Values that carry this representation.
    protocol: Protocol,  // Operations used to access/construct those values.
    payload: Payload,    // Chosen target metadata needed for rendering.
}

enum Protocol {
    Terminal { codec: PrimitiveId }, // One whole-value conversion operation.
    Product(ProductOps),   // Project members and construct a target product.
    Optional(OptionalOps), // Detect/extract/inject presence or absence.
    Sequence(SequenceOps), // Read or append target sequence elements.
    Choice(ChoiceOps),     // Inspect/write a tag and its active payload.
    Callable(CallableOps), // Capture/invoke a target callable.
}
```

`WireTypeId` refers to a type descriptor that distinguishes a valid extern ABI type from a Rust-only intermediate carrier. A Rust tuple or JVM wrapper object must not appear in an extern signature merely because it can be described as a Rust type. `MemberLayout` names an aggregate member and its child layout. `LayoutId` and `PrimitiveId`, used below, refer to registered layout and operation descriptions.

A **slot** is one value in a multi-value representation. `SlotRole` states its meaning, independent of its generated name. `GuardId` refers to a condition such as “always,” “presence is true,” or “variant tag selects this arm.” Enclosing conditions also apply. Inactive slots can require valid wire defaults even though their source payload must not be read or constructed. When one layout is used for two function arguments, its slot identities are qualified by each use so their ABI positions remain separate.

`ProductOps` describes member projections and a target construction operation over already converted children. For a C struct these can be ordinary member reads and a struct literal. For separate JNI arguments they map children to slots. For object input they can be JVM-property-read primitives. The registry can provide standard tuple/struct operations as reusable defaults.

For sequences, variants and callbacks, adapters supply runtime operations; the registry supplies loops, branches and child calls. C aggregates and JNI slots/object operations reuse the source relationship. Layouts remain nested until flattening is needed.

## Optional values

An optional value needs both a representation of its child and a way to distinguish absence. Different targets can encode that distinction differently:

```rust
enum AbsenceEncoding {
    Presence {
        flag: SlotId,         // Separate value indicating whether the child is present.
        inactive: DefaultsId, // Valid wire defaults for the absent child's slots.
    },
    Nullable {
        test: PrimitiveId,    // Test for absence in a nullable carrier.
        extract: PrimitiveId, // Obtain the present child's carrier.
        inject: PrimitiveId,  // Wrap a converted child as present.
    },
    Niche {
        domain: DomainId,     // Valid child values and a reserved absence encoding.
        test: PrimitiveId,    // Test for the reserved encoding.
        extract: PrimitiveId, // Recover the present child's carrier.
        inject: PrimitiveId,  // Encode a child without colliding with absence.
    },
}

struct OptionalOps {
    encoding: AbsenceEncoding, // The selected absence/presence convention.
    payload: LayoutId,         // Child representation when present.
    absent: PrimitiveId,       // Produce the complete representation of absence.
}
```

A **niche** is a reserved representation that cannot be a valid present child, such as zero for a handle whose valid values exclude zero. `DomainId` describes those validity facts. `DefaultsId` describes valid wire defaults, not fabricated Rust source values. `absent` builds the complete absent representation; `inactive` supplies the unused child slots for the separate-flag convention.

The registry branches on presence and invokes the child conversion only on the present path. It validates active inputs and supplies required inactive defaults. Nested optionals must preserve distinct states such as `None` and `Some(None)`; if the selected encoding cannot do that, the combination is unsupported.
