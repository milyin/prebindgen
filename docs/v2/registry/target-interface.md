# The target adapter interface

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Source-model companion: [Flat V2](../flat/overview.md).

## 6. How the registry asks a target for decisions

The target interface has four planning operations. Each answers a local question with a description. The registry owns recursion and calls the next planning operation when its inputs are ready.

```rust
trait Target {
    type Policy;  // Configuration choices explained in section 2.
    type Payload; // Owned operation/rendering descriptions explained in section 5.

    fn select(
        &self,
        query: SelectionQuery<'_, Self::Policy>, // Source value and applicable choices.
    ) -> TargetSupport<RelationSelection>;

    fn represent(
        &self,
        shape: ResolvedShape<'_>, // Selected source operation and its direct children.
        children: &[ValueDescriptor<Self::Payload>], // Completed child conversion descriptions.
        policy: &Self::Policy,    // Effective choices for this value.
    ) -> TargetSupport<ReprSpec<Self::Payload>>;

    fn boundary(
        &self,
        site: &SiteDescriptor, // Exported call's signature and boundary roles.
        values: &ResolvedValues<Self::Payload>, // Its resolved input/output values.
        policy: &Self::Policy, // Calling and error-delivery choices.
    ) -> TargetSupport<BoundarySpec<Self::Payload>>;

    fn surface(
        &self,
        request: &SurfaceRequest<Self::Policy>, // Public name/placement, policy and promises.
        values: &ResolvedValues<Self::Payload>, // Values needed to describe that public API.
    ) -> TargetSupport<SurfaceSpec<Self::Payload>>;
}
```

`TargetSupport<Answer>` means a ready description, a specific unsupported reason, or a fatal planning error; its definition is in section [9](generation.md). `BoundarySpec` describes native argument/result placement (section [8](plans.md)). `SurfaceSpec` describes a public foreign declaration and its dependencies (section [9](generation.md)).

The method inputs and results serve different stages:

| Method | Information available | Target's answer | Registry's next job |
| --- | --- | --- | --- |
| `select` | Exact source type/direction, local source facts and applicable conversion rules and policy | `RelationSelection`: chosen source relationship and declared child/stage choices | Inspect that operation's children and recursively resolve their conversions. |
| `represent` | `ResolvedShape`: source operation with model-derived child types; `ValueDescriptor`s: completed child layouts and contracts | Representation layout and target operations | Compose the complete value conversion. |
| `boundary` | `SiteDescriptor`: call signature/roles; `ResolvedValues`: its completed value descriptions | Argument placement, result delivery and error actions | Assemble and validate the complete native wrapper. |
| `surface` | `SurfaceRequest`: requested name, placement, policy and promises; required value descriptions | Public declaration description and requirements | Check dependencies before deciding whether to emit it. |

These views are read-only. The adapter can inspect direct child descriptors but cannot invoke the registry's recursive compiler or modify the registry's plan tables. For an explicit conversion rule, `select` must honor that selection or report why it is unsupported. Common relationships and standard representations should have table-based helpers/defaults so each adapter supplies as little code as practical.

Source-conversion dependencies appear in selected relationships. Target operations list the generated helpers they need. When a target method needs an additional conversion, the method returns an explicit dependency request for the registry to resolve. Rendering operates on the completed result and cannot discover new conversions or support gaps.

Descriptions returned by a target can contain new primitive, layout, helper, or policy definitions with references local to that description. The registry validates and registers the definitions and assigns its own table IDs. Existing descriptors can reference IDs the registry already supplied. The target does not allocate entries in registry-owned tables itself.

The language-provided final rendering interface reads immutable plans and retained payloads. The common emission machinery supplies allocated operand names and the necessary rendering context; the rendering interface exposes no planning entry point. The original builder is translated before generation; subsequent decisions use requests and policies.
