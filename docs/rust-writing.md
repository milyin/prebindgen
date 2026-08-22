# Rust writing after planning

Here **planning** means every semantic decision about what will be generated and
how values cross the boundary; it excludes turning those decisions into Rust
syntax. This document proposes an implementation change. It does not redefine
the [binding model](model.md): Flat, crossings, recipes, fragments and site
plans keep their existing meanings. The proposal makes the implementation obey
the [Flat planning boundary](model.md#flat-is-the-planning-boundary): captured
Rust type syntax becomes available only after every generation decision is
complete and validated.

The target is semantic preservation, not textual stability. The public C
application binary interface (ABI), Java Native Interface (JNI) contract and
generated Kotlin API must remain compatible, and existing ownership, cleanup,
error, panic and concurrency behavior must keep working. Generated source and
private helper structure may change. In particular, the migration may reorder,
combine, inline or remove private converter functions when that improves the
result without changing the public contract.

Any change to a public symbol, calling convention, wire layout, Kotlin-visible
signature or documented behavior is separate binding work and must be proposed
and reviewed as such. A code-generation optimization is in scope when only its
private implementation, code size or performance changes.

## Terms used here

The model terms below are defined fully in [the model vocabulary](model.md#the-vocabulary).
Their short descriptions here say how this proposal uses them.

| Term | Meaning in this document |
|---|---|
| **Binding** | The generated foreign-language interface for one annotated Rust source. Its build is described in [where the crates sit](model.md#where-the-crates-sit). |
| **Adapter** | The target-specific generator: currently Cbindgen for C or JniGen for JNI and Kotlin. It decides how model values cross its target boundary. |
| **Flat** | The complete language-independent model parsed from captured source records. Planning asks Flat for facts instead of inspecting Rust syntax. |
| **`TypeRef`** | Flat's opaque description of one Rust type. Its structural reading is available during planning; its captured Rust spelling is not. See [the crossing](model.md#the-crossing). |
| **Declaration** | A build-script statement selecting a function, constant or way for a type to cross. Undeclared source items generate no binding output. |
| **Direction** | The orientation of a recursive shape walk: toward Rust (`Construct`) or toward the boundary (`Deconstruct`). See [parts and wire use different verbs](model.md#parts-and-wire-use-different-verbs). |
| **Crossing** | One `TypeRef` and one direction, used as a query into the recipe table. It does not itself say whether the next step is parts or wire. |
| **Recipe** | One named row owned by the recipe table, keyed by `(CrossingKey, RecipeId)` and containing a shape. The table may contain several rows under the same crossing key. |
| **Shape** | The value–parts step declared by a recipe row. A shape with parts constructs or deconstructs; `Atomic` ends the shape walk without naming wire types. |
| **Site** | One position where a value crosses, such as a function parameter, return or callback argument. |
| **Fragment** | The adapter-specific result of applying one selected recipe row to one spelled crossing and composing its child fragments. It is the first layer that records wire layout and terminal decoding or encoding. One normalized table row may produce different fragments for `T`, `&T` and `Box<T>`. |
| **Site plan** | The adapter-specific answer for one position in the generated interface, such as a function parameter or return. It selects a fragment for that position. |
| **Wire value** | A Rust value whose type has an exact representation in the target calling convention, such as JNI's 64-bit integer slot `jlong` or C's untyped raw pointer `*mut c_void`. |
| **Converter** | Internal Rust emitted from a fragment. It composes value–parts operations and child converters; at an `Atomic` terminal it decodes wire values into the Rust value or encodes the Rust value into wire values. |
| **Wrapper** | The exported generated Rust entry point for a declared function. It converts parameters, calls the source function and converts its result. See [what the foreign side calls](model.md#what-the-foreign-side-ends-up-calling). |
| **Callback** | A callable supplied by the foreign side and invoked later by Rust. Its argument crossings run opposite to the callback crossing. |

This proposal also uses implementation terms that are not part of the binding
model:

- Planning ends only when declarations, recipes, dependencies, fragments,
  sites, generated symbols, ownership and cleanup behavior are validated and
  immutable.
- A **generation plan** is that complete immutable result. Its Rust type is
  adapter-specific; C and JNI do not need a shared code-generation language.
- An **artifact plan** completely describes one output item, such as a converter
  function, exported wrapper, helper type or target-language declaration.
- **Final emission** turns artifact plans into Rust abstract syntax tree (AST)
  items and assembles the output file. It makes no generation decisions.
- **`Emit`** is the registry-owned capability that lets code render the captured
  source syntax behind `TypeRef`. It implements Flat's `RustEmitter` protocol.
- **`Compile`** is the current adapter trait for constructing fragments and site
  plans; **`recipe::Compiler`** drives that trait over recipes and sites.
- **`Answer`** is the registry-facing result for one crossing. It records which
  other crossings the adapter's fragment depends on, not the generated Rust.
- A **source feature guard** is an anonymous captured constant that makes a
  generated file fail to compile when generator and source features disagree.
- **`Prebindgen`** is the current shared trait containing item-kind Rust
  emission callbacks and the final whole-item rewrite.
- The **adapter renderer** translates one adapter-specific artifact plan into
  Rust AST items. It is allowed to receive `Emit` because it runs only during
  final emission.
- The **shared Rust writer** drives final emission for every adapter: it owns
  ordering, `Emit`, file assembly, formatting, error handling and destination
  writing.

## Baseline: rendering starts during resolution

[How the file is built today](model.md#how-the-file-is-built-today) describes
the two visible rounds: resolve conversions, then assemble the Rust file. The
implementation mixes Rust rendering into the first round.

The current call path is:

1. [`RegistryBuilder::convert_with`](../prebindgen-registry/src/registry/declare.rs)
   derives the crossing order, constructs an `Emit`, and passes it into the
   adapter's conversion closure.
2. [`recipe::Compiler`](../prebindgen-registry/src/recipe/compile.rs) constructs
   another `Emit` and exposes it through every `Compile` hook's context. C and
   JNI compile hooks use it to spell source-side `TypeRef`s while fragments and
   sites are still being planned.
3. A compiled fragment holds generated syntax, including complete
   `syn::ItemFn` AST nodes where its recipe emits converter functions. Both
   adapters keep the fragment memo in `Rc<RefCell<Compiled<_>>>`, a shared
   interior-mutable map. They repeatedly clone that map into
   `Compiler::resume`, run part of compilation, and store the resulting map
   again so later conversion generation can observe earlier generated code.
4. The [C builder](../prebindgen-c/src/trait_impl.rs) and
   [JNI builder](../prebindgen-jni/src/jni/trait_impl.rs) copy generated
   functions out of the fragment memo into a second `compiled_fns` collection.
   JNI includes a main converter and any pre-stage converter functions; a
   composed-only fragment contributes none.
5. Each adapter passes `compiled_fns`, itself and the resolved registry to the
   [shared writer](../prebindgen-registry/src/write.rs). The writer constructs a
   further `Emit`, appends the already-rendered converters, invokes item-kind
   callbacks, parses each callback's `TokenStream` (an unparsed Rust token
   sequence) back into `syn::Item` AST nodes, appends source feature guards and
   runs `post_process_item` over the complete AST before formatting and writing
   it.

`post_process_item` is the current cross-cutting rewrite that qualifies bare
source names with their module paths. It can inspect and rewrite every generated
item after the earlier rendering steps have finished.

### Finding

The spelling capability is private but not late. Ordinary adapter code cannot
construct `Emit`, but conversion planning explicitly receives it. The comment
on `convert_with` therefore calls its closure an emission callback.

The registry does not need generated Rust to resolve a binding. It needs a
conversion's semantic dependency edges, returned as `Answer::over(...)`.
Recipe composition already knows those edges; the registry never analyzes a
generated function body to discover them. Keeping `syn::ItemFn`s alive during
the dependency walk adds no model information.

The resumable compiler, shared mutable fragment memo, duplicate function list,
token reparsing and whole-file rewrite are consequences of rendering too early.
They are not required by crossings, recipes or recursive dependency ordering.

## Required boundary

The normative rule lives in
[Flat is the planning boundary](model.md#flat-is-the-planning-boundary). In
operational terms:

- Planning may inspect Flat's structural type class (`TypeKind`), normalized map
  identity (`TypeKey`), crossing mode, structural children, declared fields and
  functions, and source locations.
- Planning may store and pass a `TypeRef` as an opaque source-type reference.
- Planning may not receive `Emit`, obtain a source-side `syn::Type` Rust type
  AST, turn a `TypeRef` into tokens or text, or render a function merely to
  infer its dependencies.
- If planning needs a fact available only from captured Rust syntax, Flat is
  incomplete. Add a lossless fact and its recovery test to Flat.
- Adapter-authored wire syntax is allowed during planning. A C adapter can
  state `*mut c_void` and JNI can state `jlong`: those are output vocabulary,
  not captured source syntax.
- Final emission may spell `TypeRef`s stored in frozen plans, but may not select
  recipes, discover dependencies, rebuild site plans, choose fallback
  converters or reject a newly discovered shape.

## Target control flow

Generation has one semantic phase followed by one Rust-syntax phase:

```text
captured records
    -> Flat
    -> declarations, recipes and recipe selection at each site
    -> ordered crossings
    -> adapter-specific fragment and artifact plans
    -> validate and freeze the generation plan
    -> shared Rust writer + adapter renderer + Emit
    -> assemble, format and write the file
```

The roles are deliberately narrow:

| Component | Owns | Must not do |
|---|---|---|
| Flat and registry | Source facts, crossing demand and order, dependency completeness | Generate Rust syntax |
| Adapter planner | Target choices and adapter-specific semantic plans | Render captured source types |
| Generation plan | Immutable fragment, site and artifact decisions for one generation run | Lazily consult or mutate registry state |
| Adapter renderer | Translate one frozen artifact plan into `syn::Item`s | Receive `&Registry` or re-plan from a source signature |
| Shared Rust writer | Drive artifact iteration, mint `Emit`, collect items, append guards, format and write | Make target-language or crossing decisions |

The shared writer remains one pipeline. Cbindgen and JniGen may retain
convenience `write_rust` methods, but those delegate to it; adapters do not
reimplement ordering, assembly, formatting, destination handling or common
errors.

## The frozen generation plan

Plan types remain adapter-specific. Each immutable store must contain at least:

- one fragment plan per reached recipe, including semantic dependencies, wire
  slots, ownership and cleanup operations, a stable generated symbol and
  source-side positions stored as `TypeRef`s;
- one site plan for every crossing position in a declared function or callback,
  with its exact selected fragment;
- one artifact plan for every final top-level item. A private conversion may be
  represented as its own helper, an operation in another artifact or a shared
  helper; the plan need not preserve one node per converter function in the old
  output;
- an explicit artifact order, compatible identities for public symbols and
  deterministic identities for private artifacts, so ordering and duplicate
  detection do not depend on rendered function names;
- all prerequisites needed by the selected artifacts; and
- typed planning and validation errors, produced before an output path is
  opened.

The Rust and Kotlin sides of a JNI binding must read the same frozen plan
instance. They cannot rebuild plans independently or retain a mutable cache that
allows the two writers to observe different decisions.

## The final Rust-writing pipeline

The shared writer performs one fixed sequence:

1. Accept a validated generation plan and its adapter renderer.
2. Mint the only `Emit` that the generation run exposes.
3. Walk the already-ordered artifact plans.
4. Give the renderer one item-specific plan and `&Emit`; collect its
   `Vec<syn::Item>` result.
5. Append the source feature guards represented by the frozen plan.
6. Apply only syntax-level normalization that cannot affect a generation
   decision.
7. Format the full AST and write the destination.

Conceptually, not as a committed public API:

```rust,ignore
struct Generation<P> {
    artifacts: Vec<P>, // complete, validated and already ordered
}

trait RenderRust {
    type ArtifactPlan;

    fn render_artifact(
        &self,
        plan: &Self::ArtifactPlan,
        emit: &Emit,
    ) -> Result<Vec<syn::Item>, RenderError>;
}

fn write_rust<R: RenderRust>(
    generation: &Generation<R::ArtifactPlan>,
    renderer: &R,
    destination: impl AsRef<Path>,
) -> Result<PathBuf, WriteError>;
```

Calling `render_artifact` is the single target-specific boundary in the shared
pipeline. Unlike the current `Prebindgen` protocol, it is not split by source
item kind and it exposes neither the registry nor a global plan store from which
the renderer could select another answer.

The renderer returns `syn::Item`s directly, eliminating the current
`TokenStream -> syn::File -> syn::Item` parse round trip. Source qualification
should happen when the renderer spells each stored `TypeRef`. If a final
syntax-only normalizer remains, it receives no registry and cannot change which
artifacts or conversions exist.

## Optimizations this boundary permits

Requiring byte-identical generated files would preserve several artifacts that
the new boundary makes unnecessary. Semantic preservation allows the planner
and renderer to improve them:

- Emit fully qualified source types directly and delete the whole-file
  `post_process_item` rewrite, even when that changes generated spelling.
- Return `syn::Item`s directly and let ordinary formatting determine whitespace
  and item layout instead of preserving token-reparse artifacts.
- Inline or combine private converters and JNI pre-stage functions when a
  separate generated function provides no reuse or diagnostic value.
- Remove unreachable private helpers instead of retaining them to keep a golden
  file unchanged.
- Deduplicate artifacts by semantic identity before rendering rather than by a
  rendered function name afterwards.
- Use the complete frozen plan to choose reusable helpers versus inlined code,
  reducing generated code size or call overhead without exposing source syntax
  to planning.

An optimization that changes which artifacts exist or how they compose is a
planning decision and must be stored in the frozen plan. The renderer only
realizes that decision; semantic freedom does not permit it to start planning
again.

## What the change removes and preserves

The proposal removes accidental coordination:

- `Emit` from `RegistryBuilder::convert_with`, `recipe::Compiler` and all
  `Compile` contexts;
- generated `syn::ItemFn`s from fragment planning;
- the duplicate `compiled_fns` caches;
- the `Rc<RefCell<Compiled<_>>>` clone/resume/finish exchange used so generation
  can observe its own partial output;
- lazy or later compiler resumes used to construct site plans after conversion
  resolution;
- registry-bearing item-kind emission callbacks;
- generated-token reparsing; and
- name-based converter deduplication after rendering.

It preserves model complexity:

- Flat as the sole description of captured Rust;
- crossing dependency order and the explicit rule for recursive cycles;
- the distinction between reusable recipe fragments and per-site plans;
- adapter-specific wire layouts and artifact plans;
- typed planning and validation failures; and
- the callback rule in which argument crossings run opposite to the callback
  crossing, as described under [direction](model.md#the-direction).

## Migration

The migration uses semantically preserving vertical slices. A slice changes one
adapter all the way from planning through final emission and deletes the state
it replaces, rather than maintaining parallel semantic and pre-rendered
representations merely to keep generated text identical.

1. **State and test the observable contracts.** Record the exported C symbols,
   signatures and layouts; JNI native names and descriptors; Kotlin public
   signatures; and ownership, cleanup, error, panic and concurrency behavior.
   Generated-source snapshots remain change detectors, not an immutable
   specification: an intentional private-code diff is reviewed and committed.
2. **Migrate C end to end.** Replace generated functions inside C fragments with
   semantic operations, eagerly freeze every C fragment, site and artifact
   plan, and render them through the shared writer. Delete C's early `Emit`,
   shared mutable compiler memo and `compiled_fns` in the same slice. C is the
   smaller adapter and establishes the boundary and writer API first.
3. **Migrate JNI end to end.** Make Rust and Kotlin consume one frozen JNI plan,
   move source spelling to the final renderer and delete, rather than reproduce,
   the legacy carriers. `ConverterImpl` and `Stage` hold generated converter
   chains; `expand`, `unfold` and `fn_plan` are the older decomposition and
   per-function planning modules. Their deletion is tracked by
   [issue #506](https://github.com/milyin/prebindgen/issues/506). This slice may
   be several reviewable PRs, but no completed PR keeps duplicate planning and
   rendering stores solely for textual compatibility.
4. **Delete and seal the old path.** Remove `Emit` from `convert_with`,
   `recipe::Compiler` and every planning context. Reduce the shared writer to
   the final-assembly sequence above, replace item-kind token callbacks with the
   single artifact-rendering boundary, and remove the `Prebindgen` emission
   protocol and `post_process_item`. Compile-fail and call-graph checks then
   enforce that planning cannot obtain source spelling.

The Flat boundary is complete after both adapter slices stop rendering during
planning and stage 4 removes the generic escape. Generated output may evolve at
each slice; every externally visible difference still requires an explicit
contract change outside this proposal.

## Relation to existing work

- [Issue #195](https://github.com/milyin/prebindgen/issues/195) requires pure
  emission: renderers consume frozen plans rather than querying registry state.
  This proposal supplies the complementary rule that source Rust spelling is
  unavailable until that pure emission begins.
- [Issue #251](https://github.com/milyin/prebindgen/issues/251) moved crossing
  demand and answers out of the old callback protocol. Its remaining
  emission-out direction is completed here without duplicating the Rust writer.
- [Issue #506](https://github.com/milyin/prebindgen/issues/506) tracks deletion
  of JNI's legacy plan carriers. This proposal uses that deletion during
  migration but does not turn those carriers into the replacement architecture.

## Exit checks

The boundary and behavior are mechanically testable:

- No function reachable from planning or validation receives `Emit`, implements
  `RustEmitter`, or renders a source-side `TypeRef`.
- No function reachable from the final renderer receives `&Registry`, rebuilds
  a fragment or site plan, or classifies a source signature.
- Every source-side type in a generation plan remains a `TypeRef` until final
  emission.
- Validation and every artifact writer observe the same immutable plan store.
- All source-spelling calls are reachable only from the shared Rust writer's
  final-emission boundary.
- Exported C symbols, signatures and layouts remain compatible; JNI native names
  and descriptors remain compatible; generated Kotlin retains its public
  signatures and behavior.
- Generated-source changes are classified as private restructuring,
  qualification, formatting or an explicit optimization. No unexplained public
  delta is accepted.
- Workspace tests, the Rust linter (clippy), Rust documentation tests (rustdoc),
  reviewed regeneration goldens, AddressSanitizer smoke tests and the JNI
  end-to-end Java Virtual Machine covertest harness remain green.
