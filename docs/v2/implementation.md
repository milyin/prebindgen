<!-- spec: {"kind": "implementation"} -->

[Project contents](README.md) · Previous: [Emit bindings](stages/07-emit.md)

# Implementation and acceptance

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

## The first increment, as built

The planning half of steps 2 and 3 above is implemented in
`prebindgen-registry-v2`, over the two element paths this document specifies:
both are planned, assembled and emitted, for both targets. What step 2 also asks
for — executing the binding through both language boundaries — is met on the C
side and not on the JNI side, where the evidence is that the generated Rust
compiles against the real `jni` crate and that the Kotlin says what this document
says. A JVM that loads the library and calls the method, which is what
[the JNI completion rule](#cases-that-expose-architectural-mistakes) requires,
comes with the covertest work rather than here. What follows records what
building this settled, so the chapters and the engine describe the same thing.

The engine is five modules. `target.rs` is the adapter interface and the
description vocabulary; `plan.rs` is the recursion, the conversion cache, the
wrapper assembly and the retention loop; `body.rs` is the instruction set;
`emit.rs` is the common Rust writer; `run.rs` holds the frozen `Generation` and
the report, which the reporting scaffold already had.

`examples/v2check` is the increment's evidence. It compiles
[the specification's source crate](source.md) for real, runs the engine over it
twice — through a small C adapter and a small JNI adapter, a few hundred lines
each — and compiles both generated files with rustc. Its tests read the expected
wrappers out of [the emit pages][fn_emit] themselves, item by item, so a chapter
and the engine cannot drift apart quietly, and read the Kotlin in order and
without duplicates. One test calls the generated C entry point, on a function
whose result changes if the two fields arrive in the wrong order — addition
would not notice.

The engine's own tests use a target that answers in one line, and cover what an
adapter cannot show: that two functions taking the same record share one
conversion, that an override on a parameter or on one of its fields makes a
different conversion whichever order the two are planned in, that a temporary
never takes the name of a parameter the wrapper still needs, that one
unsupported field skips its record and its callers with one cause and each one's
own path to it, that a public declaration the target refuses skips what requires
it, that a declared failure with no route — or a reporting operation needing a
context the boundary does not supply — skips its function, and that
contradictory configuration fails rather than becoming a capability claim.

### What the increment settles

1. **The instruction set.** `ConversionBodyId` and `FunctionBodyId` are three
   instructions over value identities — apply a registered operation, construct
   a source record, call the source function — described with the rest of the
   [conversion plans](stages/04-values.md#the-conversion-plans-the-registry-builds).
   A conversion's body is a template whose carrier is its input; using it inlines
   it under the caller's identities. Names are allocated by the writer from
   definition order, never by an adapter.
2. **Registry-supplied operations inside an adapter's payload.** An operation's
   implementation is `Operation<Payload>`: either a `Standard` operation the
   registry renders — identity, member read — or the adapter's own `Payload`.
   The payload has no standard variant to imitate, and C ships no operation
   renderer at all, which its adapter states by giving `Payload` no values.
3. **`RelationSelection`'s reach.** It has none: `select` answers with the
   relation for the value in front of it, and nothing else. A choice for a child
   is a conversion rule recorded at the child's position, which the recursion
   consults when it plans that child. The precedence question is gone because one
   of the two ways to express it no longer exists.

   For the same reason, `represent` is not told the position it is answering
   for. A representation is reused wherever a conversion of the same identity is
   needed — crossing, relation, effective policy, children — so a target that
   answered differently for two positions would have its second answer silently
   bypassed by the first one's node. Varying by position is what a policy
   recorded at that position is for, and that policy is in the identity.
4. **How an adapter declares its types and artifacts.** Neither is an id an
   adapter allocates. A carrier is a `WireType` — the Rust type it is spelled as,
   plus whether it may appear in an extern signature — carried inline in the
   description that uses it. A generated unit is an `Artifact`: a name and the
   Rust it contributes. The registry keeps one artifact per name and publishes
   only those a retained output needs.

   These descriptions are checked where they meet the values in hand, which is
   the registry and nowhere else. What is checked today, and nothing beyond it:
   a carrier that may not cross the ABI cannot be a native parameter or return;
   a native parameter must carry what its conversion reads, and a native return
   what the result conversion produces; where an operation states an operand or
   result *carrier*, it must be the carrier it is applied to and produces; a
   member read must name a member the representation declared; no projection may
   consume a carrier its siblings still read; and a failure route must report the
   error carrier the operation raises. An operand or result stated as a source
   type is carried and not compared, because nothing yet needs to relate a
   conversion's Rust type to an operation's.
5. **Runtime contexts.** `ScopeRequirement`'s concrete form is a named operand
   role: an operation declares `Context("jni.env")` where it needs the
   environment, the boundary names the native parameter that supplies it, and the
   registry binds the two at assembly. A conversion asking for a context its
   boundary does not supply is a reported skip, not a fragment reaching for a
   variable its caller happens to have.

### What it does not settle

- **The target interface's parameter types** are real for these two paths —
  `SelectionQuery`, `ResolvedShape`, `ChildValue` (the chapters' `ValueDescriptor`),
  `ResolvedValues`, `SiteDescriptor`, `SurfaceRequest` — and untested by a third
  target or a deferred capability.
- **The composition protocols.** `ProductOps` is one projection per part, in the
  into-Rust direction only: a record *leaving* Rust needs a target construction
  operation, and until there is one it is a reported skip
  (`unsupported.record.out_of_rust`). `SequenceOps`, `ChoiceOps`, `CallableOps`
  and the flattening rule for a child that produces several slots remain names.
- **Fallible construction meeting the boundary** is untouched, because
  constructor and projector relations are not implemented: the only relations are
  the record's fields and the atomic conversion.
- **Optional values** and everything that goes with them — `Layout::Slots`,
  `SlotRole`, `GuardId`, `AbsenceEncoding` — are not implemented.
- **Validity and resource contracts** are absent from `PrimitiveSpec`. Every
  operation in this increment produces an independent value and acquires nothing,
  which is why the omission is safe; the first borrowing or handle-bearing
  operation is what has to add them, and cannot be written without them.
- **Delivery** is a native return or nothing. Out-parameters, `Result` branches
  and declared sinks are `OutputPlacement` variants the increment does not have.
- **The source model's views.** Stage 2's `FunctionView`/`TypeView` are not
  built: the engine plans over today's borrowed `Flat` API and its frozen result
  owns the model, so a plan cannot outlive it and a view from another snapshot
  cannot be offered. That is enough for one model per run, and it is exactly what
  local helpers and cross-snapshot planning will break.
- **Node retention** keeps every conversion the run planned rather than only
  those a retained output reaches. Nodes are referenced by nothing after
  inlining, so this costs memory and no correctness; pruning them needs the
  reachability the retention loop does not yet track.
- **`ElementId` is `<kind>:<origin>`**, so exposing one Rust function at two
  foreign placements — which [the request chapter](stages/03-requests.md) uses to
  explain output identity — cannot be expressed yet. The `Unselected` outcome is
  likewise absent from the report.

## Acceptance and feasibility evidence

Acceptance criteria:

- [x] The design's boundaries are exercised by scalar and record bindings — in `examples/v2check`, over the specification's own source crate. Extending that to the existing C/JNI examples waits for the frontends to build `BindingRequests`.
- [ ] Users configure the existing language frontends; frontend internals construct `BindingRequests` for the registry. Target policies have explicit local interpretation APIs.
- [x] The registry owns recursive conversion, source calls, dependency resolution, control flow and Rust wrapper assembly.
- [ ] The [source model](stages/02-flat.md) supplies checked source views; the registry validates snapshot association and derives conversion keys privately.
- [x] Targets retain their representation, runtime-operation and delivery choices without implementing another recursive source planner: neither reference adapter walks a type or names a temporary.
- [x] Complete unsupported inputs produce actionable per-element outcomes; malformed configuration and generator defects fail generation.
- [ ] One immutable generation result supplies Rust output, optional foreign-writer output, reports and test selection; C headers are derived from the retained Rust output by `cbindgen`.
- [x] Emitted output preserves logical behavior and declared interfaces without a byte-identity requirement — checked item by item against the emit pages, compiled by rustc, and executed for C.
- [ ] New nested combinations reuse the registry's composition algorithm instead of requiring a new per-language wrapper implementation.
- [ ] Remaining unsupported capabilities and any API refinements discovered during implementation are documented.

Earlier feasibility work inspected registry relations and composition and Flat emission at [e046546](https://github.com/milyin/prebindgen/tree/e04654679e7aa0e7c7d4b9bb4a1268f9943926ec). Type-key inspection at [a429662](https://github.com/milyin/prebindgen/tree/a429662bb450408f401ad8f52ff753c5f5a179d4/prebindgen-flat/src/flat) confirmed `TypeRef::key()` and its equality/hash contract. `cargo test -p prebindgen-flat --lib`: 92 passed. These checks covered existing Flat behavior, not the proposed V2 contracts.

Future resource, recursive and runtime capabilities require implementations and tests; until then, affected requests remain unsupported. Background: [#689](https://github.com/milyin/prebindgen/issues/689) / [#701](https://github.com/milyin/prebindgen/issues/701); earlier plans: [#713](https://github.com/milyin/prebindgen/issues/713) / [#717](https://github.com/milyin/prebindgen/issues/717).

[fn]: examples/fn/README.md
[fn_emit]: examples/fn/07-emit.md
[struct]: examples/struct/README.md
