<!-- spec: {"kind": "implementation"} -->

# Implementation and acceptance

[Project contents](README.md)

The chapters describe the pipeline as a whole. This page is the plan for building
it: the order in which the pieces become real, the cases that would expose a
wrong architecture early, and what has to be true before the work counts as done.
It is not a pipeline stage, and no element path passes through it.

## Flat implementation sequence and acceptance

Implement the [source-model API](stages/02-flat.md) incrementally in
`prebindgen-flat`. A V2 module can coexist with the current API during migration;
`Flat` in these documents denotes that V2 model. This does not require a new
standalone crate or an immediate breaking rewrite of V1 consumers.

1. Add builder-to-snapshot ownership, helper registration checks and direct
   function lookup/enumeration. Preserve unsupported items and locations.
2. Add parameter/result `TypeView`s, exact record navigation and field views.
   Implement the independent inspection example using existing captured inputs.
3. Add snapshot checks and checked type composition. Have V2 registry requests
   retain views, and derive cache keys privately from those views.
4. Extend constants, enums and other type forms using the same conventions when
   their consumers need them. Existing inputs remain retained and reportable
   throughout; unsupported structural access is not a reason to lose an item.

Required validation for implementation:

- A standalone Flat consumer navigates function → parameter → record → field
  without depending on the registry.
- Lookup and enumeration agree; views remain usable after the original `Flat`
  value is dropped, and cloned views refer to the same snapshot.
- A view from another snapshot is rejected even when both snapshots contain a
  type with the same name and key.
- External callers cannot forge views or mutate their retained records.
- Local helpers are validated before publication, including duplicate names and
  source-module qualification; existing opaque helper types remain representable.
- Reference, optional and fallible navigation preserves wrappers and exact child
  types. Modeled lifetime arguments survive field navigation and emission.
- Opaque items, unsupported items, guards and locations survive migration.
- Derived type views preserve model association without changing source-item
  enumeration, and type/key consistency holds by construction.
- Flat tests validate source inspection and emission. The registry project's
  C/JNI tests separately validate conversion behavior and ownership.

## Integration and first implementation steps

### Switching existing examples

Keep v1 and v2 as parallel engines behind the existing frontend. Independent v2 registry and C/JNI implementation crates can share binding-configuration data modules and the source model. They must not depend on v1 conversion plans, recursive generators or emitters. Engine selection happens before v1 generation starts.

Engine selection and separate output paths already have an initial implementation in [#721](https://github.com/milyin/prebindgen/pull/721) and [#722](https://github.com/milyin/prebindgen/pull/722). The current V2 scaffold reports unsupported declarations; the conversion architecture in these chapters is the next implementation work.

The switching contract from [#719](https://github.com/milyin/prebindgen/issues/719) is:

- The frontend crate's Cargo feature `v2` — `prebindgen-c` or `prebindgen-jni` — makes the optional engine available. It does not select it by itself.
- `.build()` reads `PREBINDGEN_PIPELINE=v1|v2`; an unset variable selects v1, including under `--all-features`.
- `build_with(Pipeline::...)` selects an engine explicitly and bypasses the environment.
- Selecting v2 without its feature produces a clear error.
- Example scripts forward the selection and enable the feature as needed. Build scripts track environment changes so Cargo regenerates the selected output.

For example:

```sh
PREBINDGEN_PIPELINE=v2 cargo build -p covertest-kotlin --features v2
```

Both engines receive the complete current configuration through their respective paths. V2 generates its supported subset without falling back to v1 for individual items. Output paths and publication must support v1 → v2 → v1 without manual cleanup or stale generated declarations.

Existing Kotlin tests can directly refer to classes absent from v2 output. Select complete supported test sections before Kotlin compilation; runtime guards cannot hide missing symbols from the compiler. C tests need equivalent selection. The manifest must still require meaningful supported tests to execute, so skipping everything cannot pass a milestone.

### First executable increment

The initial implementation should demonstrate the architecture with both existing C and Kotlin examples:

1. Construct `BindingRequests` from all recorded frontend choices; implement identities, diagnostic causes and request accounting. Preserve unsupported configuration entries and settings from the start.
2. Implement one scalar function through target descriptors, registry conversion/function plans, frozen output and the normal output path: common Rust emission followed by C header generation or Kotlin emission. Execute it through both language boundaries.
3. Add named-field records with registry-owned field traversal, construction and decomposition. Demonstrate a C aggregate and a JNI representation using the same source relation algorithm.
4. Add plain optional representations and the temporary/borrow operations required by selected existing examples. Test present/absent behavior and temporary lifetime requirements.
5. Verify dependency-based skipping, existing test-section selection, and repeated switching between engines. Each preceding executable increment also produces its report and complete artifacts.

Steps 2 and 3 are exactly the two element paths specified in this document: the
[function path][fn] is the scalar function and its owned record argument, and the
[record path][struct] is the named-field record behind it.

Do not implement every proposed enum variant before the scalar case runs. Constructor/projector conversions, `Result`, resource-bearing handles, sequences and callbacks can be added incrementally through the same descriptions and registry algorithms. Full inputs remain accepted throughout that work.

### Cases that expose architectural mistakes

The cases below are drawn from the repository's own example crates — the C and
Kotlin coverage tests that both engines are run against, and whose declarations
include the awkward shapes real bindings have. `large_flat_input_sum` and its
sibling are two such existing functions, chosen because they take a record by
borrow and by JVM object respectively.

| Case | Expected behavior |
| --- | --- |
| `Stamp` under C aggregate, JNI separate arguments, and eventually JVM-object input | Source field discovery and Rust construction stay in the registry; target operations differ. An unimplemented requested representation is reported. |
| Two functions using the same `Stamp` representation | Share the conversion while retaining different parameter names and diagnostic paths. |
| A whole opaque representation of a type with unsupported private fields | Do not traverse the unused fields. |
| Accessor/value-form helper returning a compound value | Call it once, keep its exact result type, and let the registry process its selected children. |
| Existing `large_flat_input_sum(&ObjectBoundary64)` | Support requires an owned temporary and call-scoped borrow, beyond owned record conversion. Preserve the signature; skip with a borrow reason until implemented. |
| The existing JVM-object-input sibling of that function | Its independently selected representation may have a different support outcome. |
| Source `Result` with configured handler/builder | Preserve both branches and existing delivery conventions; skip if a required handler or destination is unimplemented. |
| Nested optional values | Preserve distinct states and never decode inactive payloads. |
| Unsupported required field or promised interface member | Propagate to the complete dependent public contract, while retaining unrelated output. |

A capability involving JNI is complete only when the existing Kotlin covertest exercises both its generated native boundary and Kotlin API. Unit tests for planning are useful, but they do not establish runtime ownership, JNI, or foreign-interface correctness.

## What this design does not settle yet

Reading these chapters is enough to explain the architecture and to see why the
work divides the way it does. It is not yet enough to implement the value
planner, and the honest list of what is missing is short enough to be worth
having in one place. Each entry is a decision the first increment has to make;
none of them is expected to change the division of labour.

1. **The instruction set.** `ConversionBodyId` and `FunctionBodyId` stand for the
   structured instructions the registry composes and the common Rust writer
   renders. Their vocabulary — locals, part access, primitive application,
   branches, construction, calls — is listed but not defined. It is the
   registry's own intermediate representation and needs a concrete enum.
2. **The target interface's parameter types.** `SelectionQuery`, `ResolvedShape`,
   `ValueDescriptor`, `ResolvedValues`, `SiteDescriptor` and `SurfaceRequest` are
   what an adapter author actually reads, and each is described in a line. They
   need real shapes before a second target can be written against them.
3. **`RelationSelection`'s reach.** A selection may declare a choice for a child,
   but children are planned by a recursion that asks the target again. Which wins
   has to be stated, and the answer should make one of the two impossible rather
   than establish a precedence.
4. **The composition protocols.** `ProductOps` is used throughout and defined
   nowhere; `SequenceOps`, `ChoiceOps` and `CallableOps` cover the deferred
   capabilities and are names only. With them goes the flattening rule: how a
   child that produces several slots is laid out where a flat argument list is
   required, and how its slot identities are qualified per use.
5. **How an adapter declares its types and artifacts.** `WireTypeId` and
   `ArtifactId` are referenced by every description, but the shapes an adapter
   submits to register a carrier type or a generated unit are not given.
6. **Registry-supplied operations inside an adapter's payload.** The C adapter is
   expected to select a common `ReadMember` operation, while `Payload` is the
   adapter's own type. Either the payload has a standard variant the registry
   understands, or standard operations are not payloads at all; the chapters
   assume both in different places.
7. **Fallible construction meeting the boundary.** A constructor relation can be
   fallible, and a source function can return `Result`. How a constructor's
   domain failure composes into the enclosing function's failure routes — and
   what a constructor relation means at all in the out-of-Rust direction, where
   it has no inverse — is not stated.

## Acceptance and feasibility evidence

Acceptance criteria:

- [ ] The design's boundaries are exercised by scalar and record bindings in existing C/JNI examples.
- [ ] Users configure the existing language frontends; frontend internals construct `BindingRequests` for the registry. Target policies have explicit local interpretation APIs.
- [ ] The registry owns recursive conversion, source calls, dependency resolution, control flow and Rust wrapper assembly.
- [ ] The [source model](stages/02-flat.md) supplies checked source views; the registry validates snapshot association and derives conversion keys privately.
- [ ] Targets retain their representation, runtime-operation and delivery choices without implementing another recursive source planner.
- [ ] Complete unsupported inputs produce actionable per-element outcomes; malformed configuration and generator defects fail generation.
- [ ] One immutable generation result supplies Rust output, optional foreign-writer output, reports and test selection; C headers are derived from the retained Rust output by `cbindgen`.
- [ ] Emitted output preserves logical behavior and declared interfaces without a byte-identity requirement.
- [ ] New nested combinations reuse the registry's composition algorithm instead of requiring a new per-language wrapper implementation.
- [ ] Remaining unsupported capabilities and any API refinements discovered during implementation are documented.

Earlier feasibility work inspected registry relations and composition and Flat emission at [e046546](https://github.com/milyin/prebindgen/tree/e04654679e7aa0e7c7d4b9bb4a1268f9943926ec). Type-key inspection at [a429662](https://github.com/milyin/prebindgen/tree/a429662bb450408f401ad8f52ff753c5f5a179d4/prebindgen-flat/src/flat) confirmed `TypeRef::key()` and its equality/hash contract. `cargo test -p prebindgen-flat --lib`: 92 passed. These checks covered existing Flat behavior, not the proposed V2 contracts.

Future resource, recursive and runtime capabilities require implementations and tests; until then, affected requests remain unsupported. Background: [#689](https://github.com/milyin/prebindgen/issues/689) / [#701](https://github.com/milyin/prebindgen/issues/701); earlier plans: [#713](https://github.com/milyin/prebindgen/issues/713) / [#717](https://github.com/milyin/prebindgen/issues/717).

---

Previous: [Emit bindings](stages/07-emit.md) · Next: [Project contents](README.md)

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
