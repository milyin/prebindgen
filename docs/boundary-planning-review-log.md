# Boundary planning — review log for #187

Verbatim archive of the design discussion that ran on
[#187](https://github.com/milyin/prebindgen/issues/187) between
2026-07-26 and 2026-07-27: **31 comments, ~132 KB**.

The comments were moved here and removed from the issue so that #187's body —
the invariants, the stage graph, and the completion criteria — stays the thing
you read when you open it. Nothing is summarised or edited below; each entry is
the comment as posted, with its author, timestamp and original comment id.

## Why this is worth keeping

Three of these rounds found a representable-but-invalid state *in a document
arguing against representable-but-invalid states* — a product where a sum
belongs, then a sum plus a redundant discriminant, then a root-only use
qualifier. #187's body cites that history as the reason its plan-level
invariants (#198) run against the plan's own types rather than only against
generated code. Deleting the record would leave that justification unsupported.

Several stage issues also cite decisions taken here rather than in their own
threads — the rejection of a shared core `SlotPlan` over a shared `Wire`
(#179/#183), the preflight-not-rejection call for Stage 0's aliasing, and the
`T` vs `Option<T>` resource-domain rule propagated into #192 and #193.

## Index

| # | Author | Date | Size |
|---|---|---|---|
| [1](#comment-1) | milyin | 2026-07-26 15:36 | 8 KB |
| [2](#comment-2) | milyin | 2026-07-26 15:40 | 6 KB |
| [3](#comment-3) | milyin | 2026-07-26 15:53 | 9 KB |
| [4](#comment-4) | milyin | 2026-07-26 15:57 | 6 KB |
| [5](#comment-5) | milyin | 2026-07-26 16:07 | 8 KB |
| [6](#comment-6) | milyin | 2026-07-26 16:11 | 6 KB |
| [7](#comment-7) | milyin | 2026-07-26 16:22 | 7 KB |
| [8](#comment-8) | milyin | 2026-07-26 16:25 | 5 KB |
| [9](#comment-9) | milyin | 2026-07-26 16:36 | 1 KB |
| [10](#comment-10) | milyin | 2026-07-26 16:48 | 4 KB |
| [11](#comment-11) | milyin | 2026-07-26 16:52 | 5 KB |
| [12](#comment-12) | milyin | 2026-07-26 16:58 | 3 KB |
| [13](#comment-13) | milyin | 2026-07-26 17:00 | 3 KB |
| [14](#comment-14) | milyin | 2026-07-26 17:06 | 2 KB |
| [15](#comment-15) | milyin | 2026-07-26 17:08 | 3 KB |
| [16](#comment-16) | milyin | 2026-07-26 17:12 | 2 KB |
| [17](#comment-17) | milyin | 2026-07-26 17:14 | 2 KB |
| [18](#comment-18) | milyin | 2026-07-26 17:18 | 2 KB |
| [19](#comment-19) | milyin | 2026-07-26 17:21 | 3 KB |
| [20](#comment-20) | milyin | 2026-07-26 18:23 | 3 KB |
| [21](#comment-21) | milyin | 2026-07-26 18:25 | 4 KB |
| [22](#comment-22) | milyin | 2026-07-26 18:36 | 2 KB |
| [23](#comment-23) | milyin | 2026-07-26 18:39 | 3 KB |
| [24](#comment-24) | milyin | 2026-07-26 18:51 | 2 KB |
| [25](#comment-25) | milyin | 2026-07-26 18:53 | 3 KB |
| [26](#comment-26) | milyin | 2026-07-26 19:49 | 3 KB |
| [27](#comment-27) | milyin | 2026-07-26 19:52 | 3 KB |
| [28](#comment-28) | milyin | 2026-07-26 20:10 | 2 KB |
| [29](#comment-29) | milyin | 2026-07-26 20:12 | 3 KB |
| [30](#comment-30) | milyin | 2026-07-26 20:25 | 2 KB |
| [31](#comment-31) | milyin | 2026-07-26 20:27 | 2 KB |

---

## Comment 1
<a id="comment-1"></a>

**milyin** — 2026-07-26 15:36 UTC — original id `5084161860`

---

review by codex

## Verdict

The updated [#187](https://github.com/milyin/prebindgen/issues/187) is substantially better. The three-tier direction is sound, the rejection of a shared C/JNI physical wire is well justified, and the stages now have measurable exits.

I would approve the direction, but not consider the plan implementation-ready yet. Two phase/granularity questions are architectural blockers.

## 1. Stage 2 risks a dependency cycle with Stage 3

Today the pipeline does:

```text
scan → apply expansion/unfold plans → resolve converters
     → build/validate resolved JniFunctionPlan → emit
```

That ordering is visible in [registry.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/registry.rs:1311). `JniFunctionPlan` itself is built from an already-resolved registry and consults converter entries and expansion plans in [fn_plan.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/fn_plan.rs:384).

But Stage 2 says converter roots will be derived from plans and deletes `boundary_only_types` before Stage 3 creates the complete canonical JNI plans. That is potentially circular:

```text
resolved plan needs converters
converters are reachable because of plan dependencies
```

The text hints at “converter/lowering recipes,” but does not distinguish them from resolved `ValuePlan`.

The plan needs two explicit types/phases:

```text
BoundaryRecipe
    semantic and pre-resolution
    exposes dependencies

        ↓ resolve dependencies

ResolvedValuePlan
    contains wire, surface, validity, ownership, symbols

        ↓ derive once

WireLayout / signature view
    consumed by emitters
```

I would split Stage 2:

- Stage 2A: typed `Ready`/`Needs`/`Unsupported`, dependency provenance and pre-resolution recipes.
- Stage 3: migrate JNI to resolved value plans.
- Stage 2B: switch all reachability roots to recipes/plans, then remove `unrequire_*` and `boundary_only_types`.

That permits an incremental compatibility period without preserving mutable requirements in the end-state.

## 2. `Cbindgen::Metadata = CWireMeta` is at the wrong granularity

`TypeEntry::metadata` is attached to one converter cell for a type and direction, as shown in [registry.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/registry.rs:138). It is not attached to a particular function parameter, field, payload or callback argument.

However, Stage 1’s proposed metadata contains position-sensitive properties:

```rust
ownership: Ownership,
arity: WireArity,
```

The same C type can have different ownership at different uses. A clear example is callback `takeable_param`, which is configured per callback argument index in [cbindgen/mod.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/cbindgen/mod.rs:173).

Therefore `CWireMeta` should contain only reusable converter/type facts, such as intrinsic ABI representation and validity domain. A separate per-use `CValuePlan` should contain:

- direction and position;
- borrowed versus consumed versus produced ownership;
- callback takeability;
- physical layout and arity;
- decode/encode operation;
- recursive drop action.

Otherwise Stage 1 will move position-specific inference into metadata without actually eliminating it.

## 3. `SemanticShape` needs nominal identity and recursion semantics

The proposed nodes:

```rust
Product(Vec<FieldShape>)
Choice(Vec<VariantShape>)
```

lose the identity of the declared struct or enum. Existing `SumSpec` correctly retains its `TypeKey`, source identifier and members. That information is needed for:

- Kotlin/C artifact naming;
- constructors and match paths;
- diagnostics;
- recursive-type detection;
- artifact identity.

Inlining products recursively also risks infinite expansion for recursive or mutually recursive source types.

Prefer something like:

```rust
Product {
    key: TypeKey,
    fields: Vec<FieldShape>,
}
Choice {
    key: TypeKey,
    variants: Vec<VariantShape>,
}
```

or intern shapes in a graph and refer to them by `ShapeId`. The plan should define where recursion stops—opaque declarations, unsupported cycles, or graph back-references.

There is also no stage explicitly responsible for delivering Tier 0. Stage 1 and Stage 3 are declared independent, but both require the shared semantic tier. Add a Tier 0 milestone or explicitly state that each stage extends one shared builder incrementally.

## 4. The Tier 1 sketch does not cover the promised contexts

Tier 0 includes `Borrow`, `Result` and `Callback`, but the displayed `ValuePlan` has no corresponding nodes. Perhaps these belong to `JniFunctionPlan`, but that boundary needs to be explicit.

Similarly, `SlotPlan` contains both:

```rust
inbound: DecodePlan,
outbound: EncodePlan,
```

despite the invariant saying a plan is resolved per context and direction. Usually only one applies. Keeping both risks another representable-but-invalid product.

Define a context key, for example:

```rust
struct PlanContext {
    direction: Direction,
    position: Position,
    ownership: OwnershipMode,
    delivery: DeliveryMode,
}
```

Then make the crossing operation direction-specific. The plan should also say where JNI locking, consumption, invalidation and `close()` live; `DropPlan` alone does not describe all of them.

## 5. Flattening should be derived once per function

The updated plan says flattening is not canonical—which is correct—but proposes running the flattener per emitter.

A deterministic shared function is better than duplicate classification, but JNI ABI layout is important enough to materialize once as a derived view:

```text
hierarchical ValuePlan          canonical
        ↓
JniSignatureLayout              cached derivative
        ↓
Rust extern + JNINative + wrapper
```

This preserves hierarchy while guaranteeing identical slot order, descriptors, inert defaults and JVM parameter counts. `JniFunctionPlan` is already memoized, so it is the natural owner.

## 6. The matrix is good in intent but not yet well-defined

The declared Cartesian product includes meaningless combinations—for example input consts or arbitrary `Result` wrappers in fields. It also calls data classes and sums “leaves,” contradicting Tier 0, where they are products and choices.

It misses important current distinctions such as value blobs and `u64`, and a single wrapper dimension does not cover the nested shapes motivating the refactor:

- `Option<Vec<T>>`;
- `Vec<Option<T>>`;
- choice inside a choice;
- product containing an owning choice.

Define a bounded grammar of legal semantic shapes plus a set of boundary contexts, rather than blindly multiplying enums. The report should be generated by invoking the real planner; it must not become a second authoritative `support()` implementation.

Also specify how “Supported-and-exercised” is proven. Ideally every supported matrix fixture runs plan construction automatically, while selected representatives additionally compile through the real C/JVM toolchains.

## 7. Some completion metrics need tightening

“Late-classification calls in jnigen: 73 → 0” is too broad. Tier 1 plan builders must still consult resolved converter/type information. The correct target is:

> zero registry classification or converter lookup from Tier 2 emitters.

Likewise:

- “one matrix row” conflicts with the multi-dimensional matrix; say “classify the newly introduced cells.”
- Stage 2 does have a partial counterpart in #184; the dependency/reachability graph is the new part.
- The sequencing diagram makes Stage 2 appear dependent on Stage 0, while its stage declaration says it depends on nothing.

## What the updated plan gets especially right

- The shared layer ends at semantic structure, not physical wire layout.
- `Choice` remains hierarchical.
- Existing `StructPlan` and `JniFunctionPlan` are used as migration seeds.
- Known UB and leak detection are handled before the long refactor.
- #186 remains separate instead of being incompletely absorbed.
- Unsupported shapes become typed planning outcomes.
- Artifact identity and pure-emitter boundaries are explicit.
- Stage 3 is divided into landable slices.

My strongest recommendation is to add the pre-resolution `BoundaryRecipe` versus post-resolution `ResolvedValuePlan` distinction. Once that and the per-use versus per-converter metadata distinction are resolved, the rest of the plan becomes much more mechanically executable.


## Comment 2
<a id="comment-2"></a>

**milyin** — 2026-07-26 15:40 UTC — original id `5084175580`

---

Accepted, with two divergences from the prescription. Every code citation in the review checks out — I verified all four against `b1e7f46` before editing. The issue body is updated.

## Accepted as stated

**§2 (metadata granularity) — this is the strongest finding, above §1.** `TypeEntry` is a per-cell entry (`registry.rs:138`, keyed type × direction) and `CbCfg.takeable: BTreeSet<usize>` is per callback *argument index*, so the same C type is takeable at arg 0 and not at arg 1. No per-type cell can hold that. Putting `ownership`/`arity` in `Metadata` would have moved position-specific inference into metadata while claiming to eliminate it — the plan's own defect, one tier down. Stage 1 now splits `CWireMeta` (intrinsic ABI shape + validity domain, per cell) from `CValuePlan` (per use, keyed by `PlanContext`), and callback takeability becomes a `delivery` value rather than an index set consulted at emit time.

**§1 (phase separation).** Confirmed: `JniFunctionPlan::build` takes an already-resolved `&Registry<KotlinMeta>`, and `fn_plan`'s doc records that an unresolved-converter `Err` "only occurs at the validation phase, which reports it and fails `resolve` before any emitter runs." `BoundaryRecipe` / `ResolvedValuePlan` added, and the stages resequenced to **2A → 3 → 2B**.

I'd frame it as under-specification rather than circularity: the resolver is already a rank-based fixed point, so recipes and resolution co-iterating is fine — what was missing is a pre-resolution form that can state a dependency *without holding a resolved converter*. That's what makes the iteration well-founded, and it's the reason 2B lands after 3 rather than the reverse.

**§3 (identity and recursion).** `SumSpec` already carries `key: TypeKey` and `source: syn::Ident`, so the first draft's `Product(Vec<FieldShape>)` would have regressed against existing code. Both `Product` and `Choice` now carry `TypeKey`, shapes are interned by `ShapeId`, and cycles are graph back-references with a per-adapter stopping rule. Worth noting the recursion concern is not hypothetical: cycle detection already exists at `jni/emit/flat_input.rs:1165` and is already known-wrong — **#136**, open, keys on the bare struct ident. Tier 0 interning subsumes it, so #136 is now marked "do not fix separately."

**§3's closing point was the real gap**: nothing owned Tier 0. Stages 1 and 3 were declared independent while both required it, which would have produced two private copies — this plan's thesis, self-inflicted. There is now an explicit **Stage T** that both depend on.

**§4.** `SlotPlan { inbound, outbound }` was a representable-but-invalid product inside the plan built to eliminate representable-but-invalid products — same shape as #180. It's now `crossing: Crossing` (decode XOR encode), with `PlanContext { direction, position, ownership, delivery }` as the per-use key.

On "perhaps these belong to `JniFunctionPlan`" — yes, and I'd resolve it more sharply than "perhaps":

- `Result` and `Callback` are **function-shaped**, not value-shaped. They describe a signature, not a value crossing. Both are removed from `SemanticShape` and confined to the function plan tier.
- `Borrow` is **not a node at all** — it's an ownership mode of a *use*, so it moves into `PlanContext`. Keeping a `Borrow` node *and* an `OwnershipMode` field would have been a third instance of the same duplication.

Also accepted: `DropPlan` does not describe locking, consumption/invalidation or `close()`. Those are a separate `LifecyclePlan` derived from `ownership` + `delivery`.

**§7.** All correct. The metric is now scoped to Tier 2 emitters (`jni/emit/**`, `render.rs`, `kotlin_emit.rs`) — Tier 1 plan builders must consult resolved converter info, that's their job, so 73 → 0 was measuring the wrong set. "One matrix row" → "classify the newly introduced cells." #184 is now credited as Stage 2A's partial predecessor. Sequencing diagram fixed.

## Two divergences

**§5 — materialize once, yes, but caching isn't what makes it safe.** A cached derived signature view is *exactly* what `jvm_parameter_slots` is, and #135 tracks its drift. `fn_plans`' current guard is explicit that drift is caught "externally by the byte-identity regen check" — an external guard, which is not sufficient for an end state that claims correctness by construction. So `JniSignatureLayout` is adopted and owned by `JniFunctionPlan`, but it is **keyed with and constructed alongside the plan it derives from**, not stored beside it and refreshed. The guarantee comes from single derivation; the memo is an optimization on top.

**§6 — a hand-written grammar of legal shapes is itself a second authority**, which the same section objects to two paragraphs earlier. Resolved by splitting the roles: the **grammar bounds enumeration; the planner defines legality**. Enumerate the bounded grammar, run every cell through the real recipe/lowering path, record `Supported`/`Unsupported(reason)` from the planner's own answer. No standalone `support()` table — that was #185's design and it is now explicitly not adopted.

Everything else in §6 is in: the meaningless cells are gone via position-legal filtering, the "leaf" naming that contradicted Tier 0 is fixed, value blobs and `u64` are added, and the four nested shapes are now closure requirements on the grammar — including *product containing an owning choice*, which is the zenoh-flat#30 shape this whole plan targets. Proof of "supported-and-exercised" is two-tier: plan construction for every supported cell, toolchain compilation + covertest for sampled representatives, and a cell that constructs with no compiling representative is reported as such rather than counted.

## One thing the review didn't catch

My stage exits were partly false as written. "Committed goldens unchanged" was the reviewability guarantee for Stages 1 and 3, and it holds for C *headers* — cbindgen simplifies `MaybeUninit<T>` to `T` — but not for generated *Rust*: Stage 0's #170 fix changes `bool` input materialization, so Rust goldens necessarily move. A behaviour-preservation criterion that is known-false on contact is worse than none, because the first legitimate diff teaches everyone to wave the check through.

Every stage exit is now split three ways: **must not move** (byte-identical, enforced by the regen check — a diff is a bug), **reviewed diff** (expected to change, with the cause stated up front — a diff outside that cause is a bug), and **asserted** (the invariant the stage adds).

## One process change

§1's 2A/2B split creates a window where recipes and `unrequire_*` coexist, and "two mechanisms for one fact" is the disease this plan treats. Accepted anyway — 2B is small once Stage 3 lands — but the window is closed by a machine, not a checklist: 2A freezes new `unrequire_*` call sites, and 2B's deletion is enforced by a CI grep over `api/`. Same for `boundary_only_types`.


## Comment 3
<a id="comment-3"></a>

**milyin** — 2026-07-26 15:53 UTC — original id `5084227424`

---

codex review:
## Verdict

The latest [#187](https://github.com/milyin/prebindgen/issues/187) is now architecturally coherent and much closer to implementation-ready. It successfully addresses nearly every concern from the previous review:

- pre-resolution `BoundaryRecipe` is separated from `ResolvedValuePlan`;
- Stage 2 is correctly split into 2A → 3 → 2B;
- Tier 0 has an explicit owner, Stage T;
- C converter metadata is separated from per-use planning;
- crossing direction is a sum, not `{ inbound, outbound }`;
- JNI lifecycle is separated from dropping;
- `JniSignatureLayout` is derived once;
- the matrix invokes the real planner;
- emitter metrics and artifact-diff expectations are scoped properly.

One conceptual issue remains significant: the ownership/reference model is still too coarse. There is also a concrete contradiction in Stage 2B’s byte-identity promise.

## 1. `PlanContext.ownership` still collapses distinct facts

The proposed context is:

```rust
struct PlanContext {
    direction: Direction,
    position: Position,
    ownership: OwnershipMode, // Borrowed | Consumed | Produced
    delivery: DeliveryMode,
}
```

This still admits invalid combinations such as `Input + Produced` and `Output + Consumed`. More importantly, “ownership” is ambiguous about which side it describes.

Consider a borrowed opaque output:

```rust
fn get(&self) -> &T
```

JNI currently clones the borrowed Rust value into a newly owned foreign handle. That crossing has two simultaneous facts:

- Rust source access is borrowed;
- the foreign wire receives an owned produced value.

A single `Borrowed | Produced` choice cannot express both.

The same problem appears with:

- `&T` versus `&mut T`;
- `&[T]` versus `Vec<T>`;
- consumed input versus shared borrowed input;
- an owned outer sequence containing borrowed elements;
- callback arguments borrowed for the invocation but represented by owned wrapper objects.

I agree that ownership policy should not be duplicated as a generic `Borrow` node, but the semantic recipe must still preserve Rust reference structure and mutability. Current C lowering explicitly distinguishes shared slices and rejects mutable ones in [cbindgen/mod.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/cbindgen/mod.rs:543).

Prefer a direction-specific sum:

```rust
enum CrossingContext {
    Input {
        access: InputAccess, // SharedBorrow | ExclusiveBorrow | Consume
    },
    Output {
        transfer: OutputTransfer, // Move | Copy | CloneFromBorrow | BorrowedView
    },
}
```

Then model the foreign release obligation separately in `DropPlan`/`LifecyclePlan`.

This preserves the good decision that wire ownership belongs to the use while avoiding another representable-but-invalid product.

## 2. Stage 2B cannot safely promise byte-identical Rust output

Stage 2B says:

> emit only reachable converters and artifacts

but its exit says all generated artifacts must remain byte-identical and any emitted-set difference is a bug.

Today the writer emits every successfully resolved converter, regardless of its `required` flag, in [write.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/write.rs:145). Current `unrequire_*` removes the requirement but deliberately leaves the table cell available for resolution. Therefore the generated file may contain converters that a reachability-rooted resolver will no longer resolve or emit.

Consequently, reachable-only resolution can legitimately shrink the converter set.

There are two defensible options:

- Treat dead-converter removal as Stage 2B’s reviewed diff and generate a before/after converter manifest.
- Split Stage 2B:

  1. switch requirement accounting to recipe reachability while temporarily preserving resolve-all emission;
  2. prune unreachable converters in a separate reviewed cleanup.

The current “any emitted-set diff is a bug” condition conflicts with the stage’s stated purpose.

## 3. The dependency structure should not be assumed to be a DAG

Stage 2B still says to follow a dependency DAG, while Stage T explicitly introduces legal cyclic semantic graphs. Existing resolution also has cross-direction callback dependencies and uses a fixed point rather than topological evaluation.

Unless acyclicity is proven for converter dependencies after opaque cut points, call this a dependency graph and specify:

- strongly connected component handling;
- which cycles are legal;
- which cycles stall as `Unsupported`;
- how a stalled cycle is distinguished from a missing leaf;
- diagnostic provenance around the cycle.

`Needs(Vec<Dependency>)` is a good protocol, but termination and cycle diagnostics need to be part of its contract.

## 4. Stage 2B’s dependency on the C migration is unclear

Stage 2B deletes global core APIs:

- `unrequire_input`;
- `unrequire_output`;
- `boundary_only_types`;
- `Prebindgen::boundary_only_types`.

Yet the sequence says Stage 2B depends only on Stage 3, while Stage 1—the C per-use-plan migration—is independent. If recipe-based reachability is the universal resolver model, every adapter must provide appropriate roots before the global compatibility APIs disappear.

Either:

- make Stage 2B depend on both Stage 1 and Stage 3; or
- explicitly state that core can derive sufficient C recipes from declared signatures without `CValuePlan`, and explain why.

The sequence diagram currently permits Stage 2B to land before Stage 1, which does not obviously satisfy the completion criterion that reachability derives from adapter plans.

## 5. Keep adapter policy out of `PlanContext` and Stage T

`DeliveryMode` currently includes examples such as:

```text
ByValue | Takeable | Handle | …
```

Those are not one cross-language algebra:

- `Takeable` is a C callback protocol;
- `Handle` is a JNI projection kind;
- `ByValue` is a general physical mode.

A shared enum will gradually become the adapter-policy union that Tier 0 was designed to avoid.

Use a small neutral use context plus adapter-specific policy:

```rust
struct UseContext {
    crossing: CrossingContext,
    position: Position,
}

struct CPlanContext {
    base: UseContext,
    delivery: CDelivery,
}

struct JniPlanContext {
    base: UseContext,
    delivery: JniDelivery,
}
```

Likewise, the “per-adapter stopping-rule hook” mentioned in Stage T should live in Tier 1 lowering. Tier 0 should represent graph cycles without calling adapter policy.

Also clarify that the displayed `ValuePlan` is a conceptual pattern, not one shared core physical type. Otherwise separate C/JNI plans may accidentally converge back into the rejected universal wire IR.

## 6. Ownership correctness needs failure-path invariants

The plan thoroughly covers foreign-facing destructors, but recursive ownership also matters during failed input decoding.

For a consumed composite input:

1. field A may be decoded and ownership transferred;
2. field B validation may fail;
3. the generated wrapper exits through an error path.

The plan must establish whether A is:

- rolled back and released;
- retained by the foreign caller;
- committed to a partially constructed Rust value.

A recursive output `DropPlan` alone does not answer that. `DecodePlan` should contain an explicit ownership-transition protocol—effectively prepare/commit/rollback—or otherwise prove decoding orders validation before any irreversible transfer.

Add invariants such as:

- every consumed input resource reaches exactly one commit or rollback path;
- validation cannot fail after an untracked ownership transfer;
- inactive choice payloads are never consumed;
- partial product decoding releases exactly the fields already acquired.

This is important for making ownership correct by construction rather than merely centralizing typed drops.

## 7. The shape matrix still needs practical bounds

The grammar is much better than the earlier Cartesian product, but “exhaustive” remains underspecified:

```text
Product(shape, …)
Choice(shape, …)
depth ≤ 3
```

Without bounded product/choice arity and canonical fixture selection, enumeration grows extremely quickly. It should define, for example:

- product arities tested: 0, 1, 2;
- choice variants: unit plus one- and two-field payload representatives;
- wrapper nesting depth and whether composite depth counts named nodes;
- adapter-specific runs—C and JNI will return different support answers;
- elimination of isomorphic cases.

Also distinguish these report states:

```text
Unsupported
PlanSupported
ToolchainCompiled
RuntimeExercised
```

A plan-supported cell with no compiling representative is useful evidence, but it should not be described as fully exercised.

## Execution recommendation

The plan is large enough that #187 should remain an umbrella with one child issue per stage. Stage 3 already promises one PR per bullet; Stage T, Stage 1 and Stage 2B need similar decomposition.

For Stage 1 and Stage 3, use shadow planning:

1. construct the new plan without changing emission;
2. compare its derived layout/policy against the legacy path;
3. add differential assertions;
4. switch one emission position at a time;
5. delete the legacy classifier only after equivalence is established.

That would make the “must not move” exits mechanically credible rather than depending solely on golden review.

## Final assessment

The previous blockers are resolved. I would now classify the plan as:

- architectural direction: sound;
- phase structure: sound;
- migration strategy: mostly sound;
- remaining blocker: direction/reference/ownership semantics;
- required correction: Stage 2B artifact-diff policy;
- recommended clarifications: dependency cycles, cross-adapter 2B dependency, matrix bounds and failure-path cleanup.

Once the ownership context is made direction-specific and Stage 2B’s expected converter-set change is acknowledged, the plan is ready to decompose into implementation issues.


## Comment 4
<a id="comment-4"></a>

**milyin** — 2026-07-26 15:57 UTC — original id `5084240633`

---

All seven accepted; body updated. Both load-bearing code claims verified against `b1e7f46` first, and §2 is confirmed by the code's own doc comment rather than by inspection.

## §2 — the contradiction is real, and worse than stated

`unrequire_output`'s doc says it outright:

> The type's table entry is left intact (so resolve's PASS A still resolves it if it can, **and emits it when resolved**), but a `None` resolution no longer counts as an unresolved-required error.

And `collect_converter_items` (`core/write.rs:145`) walks `walk_resolved` over both tables with **no `required` filter** — every resolved entry is emitted. So the current output demonstrably contains unrequired-but-resolvable converters, and reachability-rooted resolution will drop them. "Any emitted-set diff is a bug" was not merely optimistic; it contradicted the stage's purpose, and it was mine to catch when I wrote it.

Adopted **both** of your options rather than choosing, because they compose into something stronger than either:

- **2B.1** switches requirement accounting to recipe reachability, keeps resolve-all emission, and emits a **converter manifest** (name → reached-by provenance, or `unreachable`). Exit: all artifacts byte-identical, including the converter set. The manifest is what proves the *considered* set moved while the *emitted* set did not.
- **2B.2** prunes and deletes. The 2B.1 manifest is the review artifact: every removed converter must already be marked `unreachable`, and a removal that wasn't so marked is a bug.

That turns "review a shrinking converter set" from a judgment call into a diff against a machine-generated expectation.

## §1 — accepted, and it removes a field rather than adding one

`CrossingContext::Input { access } | Output { transfer }` makes direction the *discriminant* instead of a field sitting beside ownership, so `Input + Produced` and `Output + Consumed` stop being expressible. Same move as #180, one tier down.

The borrowed-opaque case is the clearest proof and it's already shipping: `&T` → cloned owned `jlong` handle is *borrowed on the Rust side* and *owned on the foreign side* simultaneously, and `Borrowed | Consumed | Produced` cannot say both. `Output { transfer: CloneFromBorrow }` says exactly it.

On preserving reference structure and mutability — agreed, and it resolves cleanly against my removal of the `Borrow` node: reference structure is a property of the **use**, not of the value shape. `scalar_slice_elem` (`cbindgen/mod.rs:543`) already rejects `&mut [E]` explicitly, so that distinction is load-bearing today; it becomes `InputAccess::SharedBorrow` vs `ExclusiveBorrow`, stated once instead of re-derived per policy.

**One residual I've stated rather than solved:** `position` sits beside `crossing` rather than inside its arms, so `Input + Const` is still representable. Nesting `Position` per direction duplicates most of the enum to kill one illegal pair. It's rejected by a validated constructor and by matrix filtering, not by the type. Recorded in the body as the single knowingly-retained representable-but-invalid combination — I'd rather name it than quietly leave it.

## §5 — accepted, plus the failure mode in the opposite direction

`UseContext` + `CDelivery` / `JniDelivery` adopted; `Takeable` (a C callback protocol) and `Handle` (a JNI projection kind) in one enum was the Tier-0 policy union reintroduced one tier down. The Tier T stopping-rule hook moves to Tier 1 lowering — Tier 0 now *represents* cycles and does nothing else about them.

Your point about `ValuePlan` accidentally converging back into the universal wire IR is the one I'd have been most likely to lose during implementation, so it's now stated as a non-goal: **the pattern is shared, the type is not**, and nothing in `api/core` declares a `ValuePlan`.

But there's a cost in the other direction you didn't price: if C and JNI plans share nothing, the plan-level invariant suite gets written twice, and the second copy drifts — which is this plan's own thesis turned on its test code. Resolved by sharing the *harness*, not the types: invariants are written once against a small read-only trait each adapter's plan implements. A shared trait for asking questions is safe; a shared type for storing answers is not.

## §6 — the strongest new content in this round

Partial-decode failure is a genuine hole and a recursive output `DropPlan` does not touch it. `DecodePlan` now carries an explicit prepare/commit/rollback protocol **or** proves validation is ordered before any irreversible transfer — the cheaper answer where it holds, and the one I'd expect to cover most cells. All four invariants are in, and "every consumed input resource reaches exactly one commit or rollback path" is promoted to a completion criterion.

This is the difference between centralizing typed drops and making ownership correct by construction, and the plan was claiming the latter while delivering the former.

## §3, §4, §7 — accepted as stated

**§3:** "DAG" was wrong on two counts — Stage T admits legal cyclic semantic graphs, and resolution is already a rank-based fixed point rather than a topological walk. Now "graph", with SCC handling, legal-vs-stalling cycles, stall-vs-missing-leaf discrimination and cycle provenance written into `Needs(Vec<Dependency>)`'s contract rather than left to implementation.

**§4:** 2B now depends on Stages 1 **and** 3. The old diagram permitted deleting `boundary_only_types` while C still had no per-use plans to root reachability in — which plainly fails completion criterion 3. I took the first of your two options; claiming core could derive sufficient C recipes from declared signatures alone would need a proof I don't have.

**§7:** arities 0/1/2, unit/one-field/two-field variant representatives, composite depth ≤ 3 counting named nodes only, wrapper depth ≤ 2, isomorphism elimination, **per-adapter runs** (C and JNI legitimately disagree; one merged verdict hides the gaps). The four report states replace my two-tier proof, and `PlanSupported` is explicitly not "exercised".

## Execution

Shadow planning adopted as the migration strategy for Stages 1 and 3 — construct alongside, assert differentially across the matrix, switch one position at a time, delete the legacy classifier last. You're right that this is what makes "must not move" mechanically credible instead of a promise resting on golden review, and it gives each stage a natural PR decomposition where every step is independently revertible.

Decomposition into child issues per stage is the next step and not yet done — Stage T, 1, 2A, 3, 2B.1, 2B.2, 4, with #186 already standing as Stage 5.


## Comment 5
<a id="comment-5"></a>

**milyin** — 2026-07-26 16:07 UTC — original id `5084280141`

---

codex review:
## Verdict

The updated [#187](https://github.com/milyin/prebindgen/issues/187) resolves the previous review well. The architecture is now sound in direction, and the safety model is notably stronger with direction-specific crossing, partial-decode rollback, adapter-specific delivery, SCC-aware dependencies, shadow planning, and the 2B.1/2B.2 split.

I still see three substantive issues before implementation:

1. nested reference/ownership structure is not represented by the shown types;
2. Stage 2B.2 prunes resolution before Stage 4 makes every emitter plan-only;
3. the converter manifest is keyed too weakly to prove safe pruning.

## 1. Root `UseContext` cannot represent nested reference structure

The new model correctly handles a top-level crossing:

```rust
Input  { access: SharedBorrow | ExclusiveBorrow | Consume }
Output { transfer: Move | Copy | CloneFromBorrow | BorrowedView }
```

But the issue also claims this handles “an owned sequence of borrowed elements.” The displayed model cannot express that:

```rust
Vec<&T>
```

requires two different use facts:

- the `Vec` itself is consumed;
- every element is a shared borrow.

One root `CrossingContext` cannot describe both. The same applies to references nested under:

- `Optional`;
- product fields;
- choice payloads;
- sequences;
- callbacks such as `impl Fn(&[E])`.

If Tier 0 removes `Borrow` and recursively reduces `Vec<&T>` to `Sequence(Leaf(T))`, the reference information is lost. If it leaves `&T` inside `Leaf(TypeKey)`, then Tier 0 has not actually classified reference structure once and adapters must re-peel it.

The clean solution is to place source-use qualifiers on graph edges rather than introducing a generic ownership-shaped `Borrow` node:

```rust
struct SemanticUse {
    shape: ShapeId,
    source: SourceUse,
}

enum SourceUse {
    Value,
    SharedRef,
    ExclusiveRef,
}

enum SemanticShape {
    Leaf(TypeKey),
    Product {
        key: TypeKey,
        fields: Vec<FieldUse>,
    },
    Choice {
        key: TypeKey,
        variants: Vec<VariantUse>,
    },
    Optional(SemanticUse),
    Sequence {
        container: SequenceKind, // Vec, slice, etc.
        element: SemanticUse,
    },
}
```

The root `UseContext` then describes the boundary crossing, while every nested edge preserves its Rust reference/mutability semantics. Tier 1 recursively derives child contexts.

Without something equivalent, the prose promises recursive ownership that the types cannot carry.

## 2. Crossing direction is still stored twice

A resolved plan contains:

```rust
context: CPlanContext // contains UseContext.crossing
crossing: Crossing    // Decode | Encode
```

That permits:

```text
UseContext::Input + Crossing::Encode
UseContext::Output + Crossing::Decode
```

This is another representable-but-invalid state, immediately after the plan explains why direction must be a discriminant.

`CrossingContext` expresses semantic direction and `Crossing` contains executable code, but they still must agree. Encode them together:

```rust
enum CValuePlan {
    Input {
        context: CInputContext,
        decode: DecodePlan,
        layout: CLayout,
        rollback: RollbackPlan,
    },
    Output {
        context: COutputContext,
        encode: EncodePlan,
        layout: CLayout,
        drop: DropPlan,
    },
}
```

The same principle should apply to JNI leaf plans. A generic `ValuePlan<D>` with direction-specific associated payloads would also work, provided it remains adapter-local.

This also removes the awkward presence of an output-facing `DropPlan` on every input leaf.

## 3. Stage 2B.2 currently happens before pure emission

The sequence remains:

```text
Stage 1 + Stage 3 → Stage 2B.1 → Stage 2B.2 → Stage 4
```

But Stage 4 is where emitters finally stop querying the registry and converter tables. If 2B.2 changes the resolver so unreachable entries are no longer resolved, a remaining late emitter lookup can observe `None` and silently:

- skip an artifact;
- choose a fallback;
- omit runtime support;
- produce a different declaration.

That is precisely the defect class this refactor addresses.

The writer also calls adapter prerequisites with the resolved registry before emitting converters. Runtime support can therefore depend on which cells exist, not only which converter functions are selected.

Safer sequencing:

```text
Stage 1 + Stage 3
        ↓
Stage 2B.1 — recipe accounting and manifest, resolve-all retained
        ↓
Stage 4A — all emitters and prerequisite selection consume plans
        ↓
Stage 2B.2 — prune unreachable resolution/emission
        ↓
Stage 4B — delete compatibility hooks and old classifiers
```

Alternatively, define 2B.2 narrowly as an emission filter while preserving the fully resolved registry, then perform actual resolver pruning after pure emission. The issue currently conflates those two operations.

## 4. A name-keyed converter manifest is insufficient

Stage 2B.1 proposes:

```text
name → reached-by provenance, or unreachable
```

But [collect_converter_items](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/write.rs:145) deduplicates functions by name after walking multiple type-table cells. A single converter symbol may be contributed by:

- more than one `TypeKey`;
- both directions;
- wrapper and inner entries;
- multiple entries sharing a pre-stage.

One contributing cell may be reachable while another is not. A name-only manifest cannot express that and could incorrectly authorize removal.

Use two levels:

```text
CellId = (Direction, TypeKey)

cell manifest:
  CellId → {
    dependencies,
    reached_by[],
    emitted_symbols[],
    status
  }

symbol manifest:
  Symbol → contributing CellId[]
```

A symbol is removable only when every contributing cell is unreachable. Include pre-stage functions and reachability-dependent prerequisite/support artifacts as well as primary converter functions.

## 5. The C pre-resolution recipe is still not assigned clearly

Stage 2A introduces `BoundaryRecipe` and blocks Stage 3, but Stage 1 remains independent of Stage 2A and only explicitly creates a resolved `CValuePlan`.

Stage 2B later requires recipe roots from every adapter. The plan should say where C’s pre-resolution recipes are created:

- make Stage 1 depend on Stage 2A and require both `CBoundaryRecipe` and `CValuePlan`; or
- add an explicit C-recipe task to 2B.1;
- split Stage 1 into a resolved-plan migration and later recipe integration.

A post-resolution `CValuePlan` alone cannot seed pre-resolution reachability.

## 6. Several stale statements should be corrected

These are small edits, but they describe architectural contracts and should not remain contradictory:

- Stage T still says “cycle representation and the per-adapter stopping-rule hook,” although the architecture correctly moved that hook to Tier 1. Stage T should only represent back-references.
- The lifecycle paragraph still refers to `PlanContext.ownership` and `delivery`; those fields/types no longer exist. It should reference `JniPlanContext.base.crossing` and `JniPlanContext.delivery`, plus the handle nodes being traversed.
- Stage 1 says a drop exists when a slot is `Produced`, but `Produced` was removed. It should name the relevant `OutputTransfer` cases and the resulting foreign release obligation.
- Stage 1 asks for one wire-policy entry point per direction but exits with “one wire function remains.” That is either two direction-specific functions or one function parameterized by direction.

## What is now especially strong

The latest revision made good architectural choices:

- it explicitly represents the borrowed-Rust/owned-foreign output case;
- C and JNI delivery policies no longer share an accidental union;
- Tier 0 cycles are policy-free;
- partial decode has commit/rollback obligations;
- 2B pruning is now machine-reviewed through a manifest;
- dependency cycles are part of the solver contract;
- C and JNI both gate global reachability deletion;
- the matrix is bounded, adapter-specific and honest about coverage level;
- shadow planning provides a credible incremental migration.

## Final assessment

The plan is very close. I would classify it as:

- semantic/adaptor tiering: approved;
- resolution phases: approved;
- ownership safety goals: approved;
- remaining model blocker: nested reference/use qualifiers;
- remaining sequencing blocker: resolver pruning before all emitters are plan-only;
- required implementation clarification: cell-keyed reachability manifest and C recipe ownership.

After those are addressed, #187 is ready to become an umbrella over stage-specific implementation issues.


## Comment 6
<a id="comment-6"></a>

**milyin** — 2026-07-26 16:11 UTC — original id `5084296048`

---

All six accepted; body updated. Verified the two load-bearing claims against `b1e7f46` before editing — both hold, and §3's is worse than the review states.

## §1 — the prose outran the types, and edges are the right fix

Correct, and it was my error specifically: last round I answered "reference structure is a property of the use, not the shape" and then supplied only a *root* use. `Vec<&T>` needs two facts — the container consumed, the elements shared-borrowed — and a root `CrossingContext` carries one. The claim about "an owned sequence of borrowed elements" was unbacked.

`SemanticUse { shape: ShapeId, source: SourceUse }` on edges adopted, with `SequenceKind` on `Sequence`. The reason this is not a retreat to the `Borrow` node I removed is worth stating explicitly, and now is in the body: a `Borrow(Box<SemanticShape>)` **node** makes borrowing a shape, so `&T` and `T` become different interned types and every consumer peels. An **edge qualifier** keeps one interned shape for `T` and records reference-ness exactly where it varies — at the point of use. Tier 1 derives each child context from `(parent context, edge SourceUse, container kind)`, so `impl Fn(&[E])` and `Option<&T>` fall out of the same recursion.

Added a third Tier 0 constraint while I was there: `SourceUse` records what the *source Rust type says*; what that obliges either side to release stays Tier 1.

## §2 — direction was stored twice, immediately after arguing it must not be

Accepted without qualification. `UseContext.crossing` and `crossing: Crossing` were two representations of one fact that had to agree, which is the third round of the same defect in this document — product where a sum belongs, then a sum plus a redundant discriminant.

I took the `ValuePlan<D>` variant of your suggestion rather than splitting only the leaf, because a bare `enum CValuePlan { Input, Output }` still permits a `Product` of input leaves containing an output child. Parameterizing the whole plan makes direction uniform through the tree by construction:

```rust
struct SlotPlan<D: Dir> {
    wire: WireType,
    surface: ForeignType,
    context: D::Context,     // CInputContext | COutputContext
    op: D::Op,               // DecodePlan   | EncodePlan
    obligation: D::Release,  // RollbackPlan | DropPlan
}
```

`obligation` is the point you raised about output-facing `DropPlan` on input leaves, generalized: an input leaf's release obligation is *what to release if a later field fails validation*, an output leaf's is *what the foreign side must eventually free*. Different obligations, no longer one field.

## §3 — verified, and the hazard is at step 0

This is the sharpest catch in the round. `write_rust` calls `ext.prerequisites(registry)` **first** — `core/write.rs:70`, before any converter is emitted — and `on_function` / `on_struct` / `on_enum` / `on_const` all take `registry` as well. So pruning resolution while emitters still query it doesn't just risk a different converter set; it risks *runtime support* silently disappearing, upstream of everything, with no failure.

Resequenced to your ordering: `2B.1 → 4A → 2B.2 → 4B`. Stage 4 is now split — **4A** makes emitters plan-only (including `prerequisites` selecting from plans, called out by name as the specific hazard), **4B** deletes legacy classifiers and hooks after pruning. I also moved "Tier 2 emitter lookups at 0" off Stage 3's exit onto 4A's, where it belongs: Stage 3 builds the plan, 4A switches emitters onto it.

I did not take the alternative (2B.2 as emission-filter-only, resolver pruning later). It leaves a resolver that resolves things nothing emits, which is a third state to reason about during the window that already has two.

## §4 — name-keyed manifest confirmed insufficient

`collect_converter_items` dedupes on `function.sig.ident` *after* walking both tables and every entry's `pre_stages`, so one symbol genuinely can have several contributing `(Direction, TypeKey)` cells. A name-keyed manifest would have authorized removing a symbol still reachable through a cell it didn't know about — and the failure mode is a missing function at compile time if lucky, a different overload selected if not.

Two-level manifest adopted as specified, with the removability rule stated as a gate: **a symbol is removable only when every contributing cell is unreachable.** Coverage explicitly includes pre-stage functions and `prerequisites` output, since both are reachability-dependent and neither is a primary converter.

## §5 — Stage 1 now depends on 2A and produces both

Taking your first option. Stage 1 builds `CBoundaryRecipe` (pre-resolution, dependency-exposing) *and* `CValuePlan` (post-resolution), and the body states why the recipe isn't optional: a post-resolution plan cannot seed pre-resolution reachability — the two-phase rule applied to C, which I'd stated as a general principle and then not applied to the C adapter.

Consequence: 2A now blocks Stages 1 and 3 both, rather than only 3. The sequencing is cleaner for it — T and 2A establish shared vocabulary, then the two adapter migrations run concurrently.

## §6 — all four were stale, all four fixed

Artifacts of editing sections independently across rounds:

- Stage T's checklist still requested the stopping-rule hook the architecture had already moved to Tier 1. Now representation-only.
- The lifecycle paragraph referenced `PlanContext.ownership`/`delivery`, types that no longer exist → `JniPlanContext.base.crossing` and `.delivery`, on the handle-bearing nodes traversed.
- `Produced` was gone → the drop condition now names `OutputTransfer` cases (`Move` and `CloneFromBorrow` create a foreign obligation; `Copy` and `BorrowedView` do not).
- "One entry point per direction" vs "one wire function remains" → **one** function taking direction and position as parameters, with the exit criterion matching.

The last one mattered more than it looks: "per direction" would have left two policies, which is where this started with four.

## Status

Model blocker (nested use qualifiers), sequencing blocker (pruning before pure emission), and both clarifications (cell-keyed manifest, C recipe ownership) are addressed. Decomposition into stage issues is next and not yet done — Stage T, 2A, 1, 3, 2B.1, 4A, 2B.2, 4B, with #186 standing as Stage 5.


## Comment 7
<a id="comment-7"></a>

**milyin** — 2026-07-26 16:22 UTC — original id `5084336364`

---

codex review

## Verdict

The latest [#187](https://github.com/milyin/prebindgen/issues/187) now resolves the previous architectural blockers. The target design is coherent:

- nested source references live on semantic graph edges;
- direction is uniform through `ValuePlan<D>`;
- C and JNI both produce pre-resolution recipes;
- pruning follows pure emission;
- manifests are keyed by registry cell and symbol;
- failure-path ownership is explicit.

I would approve it as the umbrella architecture. I see two remaining required consistency fixes and several smaller refinements—none requires changing the central design.

## 1. The matrix does not enumerate `SemanticUse`

The architecture’s most important new addition is:

```rust
SemanticUse {
    shape: ShapeId,
    source: Value | SharedRef | ExclusiveRef,
}
```

But the matrix still enumerates only shapes:

```text
Optional(shape)
Sequence(shape)
Product(shape, …)
```

and applies one root `UseContext`. It therefore does not generate the cases that justified edge-qualified uses:

- `Option<&T>`;
- `Vec<&T>`;
- `&[T]`;
- `&mut [T]`;
- a product with borrowed and owned fields;
- a choice whose alternatives have different source-use qualifiers.

The grammar should enumerate uses:

```text
use   := Value(shape)
       | SharedRef(shape)
       | ExclusiveRef(shape)

shape := Leaf(...)
       | Product(field-use, ...)
       | Choice(variant-use, ...)
       | Optional(use)
       | Sequence(kind, use)

kind  := Vec | Slice | CowSlice | ...
```

Then the root cell is a `use × position × adapter policy`, and nested qualifiers are generated recursively.

Without this, the matrix can pass while the new `SemanticUse` recursion is wrong—the exact sort of unexercised architectural cell the matrix is intended to prevent.

## 2. Stage 3, Stage 4A and shadow migration have overlapping ownership

The plan currently says:

- Stage 3 deletes `FlatLeaf`;
- Stage 3 migrates one emission position at a time;
- Stage 4A switches emitters to plans;
- Stage 4B deletes legacy classifiers;
- shadow-planning step 5 deletes the legacy classifier after equivalence.

These statements can coexist, but the ownership boundary is unclear. In particular, if Stage 3 deletes `FlatLeaf`, some emitters must already have switched to the new plan before Stage 4A.

Define the boundary explicitly:

- **Stages 1/3:** build plans and migrate all value-crossing emission paths; keep legacy classifiers only as differential oracles if still needed.
- **Stage 4A:** remove remaining registry-dependent orchestration and artifact selection, especially `prerequisites` and top-level `on_*` decisions.
- **Stage 4B:** remove shadow oracles, obsolete hooks and compatibility types.

Then adjust shadow-planning step 5 to point to the owning stage. This matters because each stage’s exit determines whether pruning in 2B.2 is safe.

There is also a small sequencing typo: 2B.2 is reviewed against the **2B.1 manifests**, not “4A’s manifests.” The manifests should be regenerated or revalidated after 4A, but their provenance begins in 2B.1.

## 3. Preserve structured source access in `FieldUse`

The semantic sketch currently describes:

```text
FieldUse = name + SemanticUse
```

Existing `SumSpec` carries `syn::Member`, which distinguishes named and tuple fields and supports structured patterns/access. Replacing it with a string name would regress toward the textual-access problem tracked by #186.

`FieldUse` should retain at least:

```rust
struct FieldUse {
    member: syn::Member,
    diagnostic_name: String,
    value: SemanticUse,
}
```

Potentially also retain source location. Variant identity should likewise preserve the source identifier independently from destination-language naming.

## 4. Decide whether unit enums are `Leaf` or `Choice`

Tier 0 says `SumSpec` becomes the `Choice` constructor, and `SumSpec` can describe unit-only enums as a tag with empty payload groups. The matrix, however, classifies `unit-enum` as a `Leaf`.

Either choice is defensible:

- Tier 0 models every enum as `Choice`, with adapters collapsing unit choices to tag-only leaves; or
- Tier 0 treats unit enums as terminal `Leaf` values and reserves `Choice` for payload-carrying enums.

But it must be one rule. Otherwise the matrix will exercise a different semantic node from the real shape builder.

Similarly, `handle` and `value-blob` are adapter declaration kinds, not source-semantic leaf kinds. The matrix should call them leaf fixtures with adapter configuration rather than imply Tier 0 intrinsically knows them.

## 5. Manifest provenance should admit plan/global artifacts

The two-level cell/symbol manifest correctly handles converter deduplication. One extension is worth stating: not every reachability-dependent prerequisite necessarily belongs naturally to a converter cell.

Use a provenance key such as:

```text
ReachabilityRoot =
    ConverterCell(Direction, TypeKey)
  | FunctionPlan(FunctionId)
  | TypePlan(TypeKey)
  | UnconditionalInfrastructure
```

Then an emitted support symbol is removable only if every contributing root is unreachable. This avoids forcing harnesses, helper types or package-level infrastructure into a fictitious converter cell.

## 6. “Exactly one wire-policy function” is too syntactic

The real invariant is:

> Every C use is lowered through one authoritative policy entry point; no emitter or position owns an independent policy.

Requiring exactly one Rust function can encourage a large direction-and-position match that is harder to understand than typed input/output helpers. One public plan-construction entry point with private typed helpers still gives one authority.

Measure absence of the four independent call paths, not literal function count.

## 7. Root recipe compatibility deserves an invariant

`CBoundaryRecipe` contains both:

```rust
root: SemanticUse
context: UseContext
```

These are different facts—source syntax versus adapter crossing policy—but only compatible combinations should produce a resolved plan. Add an invariant or constructor contract covering examples such as:

- `SharedRef` input cannot become `Consume` without an explicit clone/copy policy;
- `ExclusiveRef` cannot silently become shared access;
- `CloneFromBorrow` requires a borrowed source and a valid clone operation;
- `BorrowedView` requires a lifetime-safe adapter representation.

This need not make every unsupported recipe unrepresentable; `Unsupported` is the intended channel. It should ensure no incompatible pair reaches `ResolvedValuePlan`.

## Process recommendation

The architecture is now stable enough to create the child issues listed in the latest comment:

- Stage T
- Stage 2A
- Stage 1
- Stage 3
- Stage 2B.1
- Stage 4A
- Stage 2B.2
- Stage 4B
- #186 / Stage 5

After that, shorten #187 to the tier invariants, stage graph and completion criteria, moving detailed evidence and PR-sized acceptance criteria into the child issues. The repeated stale references found during these review rounds are a practical sign that the umbrella body has become too large to maintain atomically.

## Final assessment

- Architecture: approved.
- Safety model: approved.
- Resolution and pruning sequence: approved.
- Required before implementation: extend the matrix to `SemanticUse` and clarify Stage 3/4A/4B ownership.
- Recommended before decomposition: preserve `syn::Member`, settle unit-enum classification, and generalize manifest provenance.

No further foundational redesign appears necessary.


## Comment 8
<a id="comment-8"></a>

**milyin** — 2026-07-26 16:25 UTC — original id `5084348531`

---

All seven applied. Both code-grounded claims verified against `b1e7f46` first, and §4 turned out to be settled by existing code rather than a judgment call.

## §1 — the matrix would have passed while the new recursion was wrong

Accepted, and this is the one that mattered most. `SemanticUse` is the newest and least-exercised part of the architecture, and a shape-only grammar would have gone green without ever generating `Option<&T>`, `Vec<&T>`, or a product mixing borrowed and owned fields. A matrix that cannot fail on the feature added two rounds ago to fix an expressiveness gap is worse than no matrix, because it certifies the gap closed.

The grammar now enumerates **uses**:

```text
use   := Value(shape) | SharedRef(shape) | ExclusiveRef(shape)
cell  := use × position × adapter policy
```

with the six use-qualified nestings added as explicit closure requirements alongside the four shape ones, and `SequenceKind` (`Vec | Slice | CowSlice`) enumerated rather than assumed.

## §4 — settled by `SumSpec`, not by preference

You framed this as "either is defensible, but pick one." The codebase already picked: `SumSpec::from_item_enum` says a unit enum *"yields all-empty groups, which is exactly the 'tag only' lowering — so this never fails and never consults `enum_shape`."* Tier 0 modelling every enum as `Choice` is therefore the rule that matches what exists, and the matrix's `unit-enum`-as-leaf was the side that was wrong. `EnumShape::{Unit, Sum}` survives as a *classifier over* a `Choice`, not as a second modelling of one; adapters may collapse an all-empty `Choice` to a tag-only leaf.

Your second half was a cleaner catch than the first: `handle` and `value-blob` are `ptr_class!` / `value_class!` **declaration kinds applied to a leaf**, not source-semantic leaf kinds. Having them in the leaf grammar implied Tier 0 intrinsically knows about opaque handles, which is the policy leak this tier exists to prevent — in the test harness rather than the code, where it would have been harder to notice. They are now leaf fixtures plus adapter configuration.

## §3 — `syn::Member` retained

Correct, and `SumSpec` carries it at `types_util.rs:567` for exactly the stated reason. `FieldUse` now holds `member: syn::Member` plus a separate `diagnostic_name: String`, and `VariantUse` keeps `ident: syn::Ident` independent of destination-language naming. Collapsing member access to a string would have rebuilt #186's problem one tier lower, in the tier meant to be structural.

## §2 — boundary defined by *what* is emitted, not *when*

The overlap was real and the resolving distinction is: **Stages 1/3 own how a value crosses; 4A owns which artifacts exist at all.** That is why Stage 3 can delete `FlatLeaf` before 4A — its consumers are value-crossing emitters, already migrated — while `prerequisites` and the top-level `on_*` selection survive into 4A, because they are orchestration rather than crossing. It is exactly that residue that makes pruning before 4A unsafe.

Added as a table, and shadow-planning step 5 now points at Stage 4B rather than implying deletion at step 5's own site — the oracle must stay live through 4A and 2B.2.

Typo fixed: 2B.2 is reviewed against the **2B.1** manifests, regenerated and revalidated after 4A but originating in 2B.1.

## §5 — provenance generalized

Accepted as specified. Forcing a harness type or package-level helper into a fictitious converter cell would make the manifest lie about why a symbol exists, and the manifest's whole job is to be trustworthy enough to authorize deletion:

```text
ReachabilityRoot = ConverterCell(Direction, TypeKey)
                 | FunctionPlan(FunctionId)
                 | TypePlan(TypeKey)
                 | UnconditionalInfrastructure
```

`UnconditionalInfrastructure` is never unreachable, so anything it contributes is never a removal candidate — which is the property that keeps the always-emitted `konst` guard and similar out of the pruning path by construction rather than by exception.

## §6 — you were correcting your own previous round, and rightly

Last round's finding was that "one entry point per direction" and "one wire function remains" contradicted each other; I resolved it toward the literal count. You're right that this optimizes the wrong thing — a single giant `direction × position` match satisfies a function-count metric while being worse code than typed helpers behind one entry point.

Restated as the real invariant: **no position or emitter owns an independent wire policy**, measured as absence of the four call paths (`c_field_wire`, `mirror_field_wire`, `data_field_wire`, `payload_field_wire`), with one public plan-construction entry free to delegate to private typed helpers. Still mechanically checkable, no longer syntactic.

## §7 — recipe compatibility

Added as a constructor contract plus plan-level invariant, with your four cases named. Keeping `Unsupported(path, reason)` as the channel rather than making every bad pair unrepresentable is the right call — `SharedRef` + `Consume` *with* an explicit clone policy is legal, so the pair isn't inherently invalid, only silently-accepted ones are. The invariant is that no incompatible pair reaches `ResolvedValuePlan` without going through the rejection channel.

## On the process recommendation

Agreed on both counts, and the diagnosis is fair: four stale references across three rounds is the umbrella telling us it has outgrown atomic maintenance. The plan is to create the nine child issues, then cut #187 down to tier invariants, the stage graph, and completion criteria — with evidence and PR-sized acceptance criteria moving into the children. Awaiting the go-ahead before restructuring the backlog.


## Comment 9
<a id="comment-9"></a>

**milyin** — 2026-07-26 16:36 UTC — original id `5084391375`

---

Decomposed. #187 is now the umbrella: tier invariants, stage graph, baseline metrics, completion criteria and non-goals — 129 lines, down from 658. Evidence and PR-sized acceptance criteria moved into the children.

| Stage | Issue |
|---|---|
| 0 — boundary safety gaps | #189 |
| T — Tier 0 semantic shape tier | #190 |
| 2A — typed outcomes + recipes | #191 |
| 1 — C wire semantics | #192 |
| 3 — canonical JNI value plan | #193 |
| 2B.1 — reachability accounting + manifests | #194 |
| 4A — pure emission | #195 |
| 2B.2 — prune + delete bookkeeping | #196 |
| 4B — delete oracles and hooks | #197 |
| 5 — Kotlin access paths | #186 (pre-existing) |
| cross-cutting — matrix + invariants | #198 |

Two decomposition decisions worth flagging, since neither was in the recommended list:

**Stage 0 got its own issue (#189)** rather than being left to #170 and #158. It has items belonging to neither — the ASan/LSan CI gate, the `Box::from_raw` null check, and the generator invariant test — and it is the only stage that can start today, so it needed a home.

**The matrix became a cross-cutting issue (#198), not a stage.** It has no single position in the graph: the grammar's vocabulary comes from Stage T, but every subsequent stage must classify its own new cells before exiting. Filing it as a stage would have implied it lands once and is done, which is how the #152 coverage gaps happened.

The stale-reference problem you diagnosed should now be structurally harder: each invariant lives in exactly one child, and the umbrella holds only what must stay true across all of them.


## Comment 10
<a id="comment-10"></a>

**milyin** — 2026-07-26 16:48 UTC — original id `5084436313`

---

codex review:

## Verdict

The revised [#187](https://github.com/milyin/prebindgen/issues/187) remains architecturally sound, and the decomposition into child issues is a major improvement. I would approve the overall direction.

I still see three implementation-readiness issues, one of them architectural.

### P1 — Stage 4A lacks a concrete plan-carriage interface

[Stage 4A](https://github.com/milyin/prebindgen/issues/195) says `prerequisites` and `on_*` will consume plans rather than the registry, but it does not define how adapter-specific plans cross the generic core interface.

Currently:

- `Prebindgen` has only `type Metadata`, and every emission hook receives `Registry` ([prebindgen.rs:156](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/prebindgen.rs:156)).
- `Generation` stores only the registry and adapter ([registry.rs:1514](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/registry.rs:1514)).
- `write_rust` passes the registry to every hook ([write.rs:61](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/write.rs:61)).
- C and JNI intentionally have different plan types, so core cannot simply accept one shared `ValuePlan`.

The plan needs to choose an interface, for example:

```rust
trait Prebindgen {
    type Metadata;
    type ResolvedPlans;

    fn prerequisites(&self, plans: &Self::ResolvedPlans) -> Vec<syn::Item>;
    // on_function/on_struct/... likewise
}

struct Generation<E: Prebindgen> {
    registry: Registry<E::Metadata>,
    adapter: E,
    plans: E::ResolvedPlans,
}
```

An equivalent frozen adapter-owned plan store is possible, but its construction, generation isolation, and sharing between Rust and Kotlin writers must be explicit. Otherwise Stage 4A could merely hide registry lookups behind lazy adapter caches while claiming emitters are pure.

### P1 — the matrix still contradicts its unit-enum rule

In [#198](https://github.com/milyin/prebindgen/issues/198), the grammar currently says:

```text
leaf := scalar | bool | u64 | string | unit-enum-as-Choice
```

But the same issue later correctly says unit enums enumerate as `Choice`, “not as a leaf kind.” Those cannot both be true.

It should instead be:

```text
leaf := scalar | bool | u64 | string
```

with unit enums generated through the ordinary `Choice` production using all-empty variants. This is the previous review finding only partially applied.

### P1 — the matrix is still largely self-certifying

The real planner both decides legality and generates the four-state report. Therefore every new cell immediately receives some answer—even if the planner accidentally rejects a previously supported case. That does not substantiate:

> adding a kind, position or grammar production … fails until newly introduced cells are classified.

The issue should require:

- `REPORT.md` to be committed and byte-identity checked in CI, so new or changed classifications require reviewed diffs;
- a small set of explicit minimum guarantees such as “must plan,” “must compile,” and “must execute” for established acceptance shapes;
- compiled/runtime states to name the fixture or test that supplies the evidence.

Those guarantees are regression expectations, not a second lowering or `support()` implementation, so they do not violate the non-goal.

### P2 — Stage 5 is not truly parallel with Stage 3

[Stage 3](https://github.com/milyin/prebindgen/issues/193) deletes `FlatLeaf`; [Stage 5/#186](https://github.com/milyin/prebindgen/issues/186) still defines acceptance in terms of replacing `FlatLeaf::kt_access_prefix` and `kt_access_tail`.

“Whichever lands first defines the replacement” acknowledges the conflict but does not make the work independently landable. Pick an explicit order:

- land the `KtExpr` infrastructure before Stage 3 and have the new JNI plan use it; or
- make #186 depend on Stage 3 and rewrite its acceptance criteria around the new slot/value plan;
- or split #186 into AST infrastructure and remaining expression migrations.

## Final assessment

- Core architecture: approved.
- Safety and ownership model: approved.
- Stage ordering and reachability sequence: approved.
- Decomposition: successful.
- Still required before implementation: define the generation-level resolved-plan carrier, correct the unit-enum grammar, and give the matrix an independent regression gate.

No further foundational redesign appears necessary.


## Comment 11
<a id="comment-11"></a>

**milyin** — 2026-07-26 16:52 UTC — original id `5084453118`

---

All four applied. The three code claims verified against `b1e7f46` first, and the P1 on plan carriage is worse than stated — the codebase already contains the failure mode it warns about.

## P1 — plan carriage: the lazy-cache hole is not hypothetical

Confirmed: `Prebindgen` carries only `type Metadata` (`core/prebindgen.rs:156`), `Generation<E>` is `{ registry, adapter }` (`core/registry.rs:1514`), and `write_rust` hands the registry to every hook (`core/write.rs:61`).

The sharper point is that **two lazy plan memos already exist**, and Stage 4A as written was fully satisfiable by keeping them:

- `JniGen::fn_plans: RefCell<HashMap<Ident, Rc<JniFunctionPlan>>>` (`jni/mod.rs:576`), populated by `fn_plan(&self, registry, f)` on first touch;
- `JniGen::iface_spec`, whose own doc says *"populated lazily … first touch may be the"* resolve phase.

An emitter calling `fn_plan(registry, f)` "consumes a plan" by any reading of the old criterion while still holding the registry. So the stage would have passed its own exit while changing nothing that matters — which is precisely the "hide registry lookups behind lazy adapter caches" outcome you named.

#195 now specifies `Prebindgen::ResolvedPlans` with `Generation { registry, adapter, plans }`, plus three explicit properties, since the associated type alone doesn't prevent the hole:

- **Construction** — built once inside `resolve`, after validation, before `Generation` exists. A plan that fails to build is a resolve-time error, never an emit-time `None`.
- **Generation isolation** — the store belongs to one `Generation`; no adapter-held `RefCell` survives as a back channel.
- **Writer sharing** — `write_rust` and `write_kotlin` read the *same* frozen instance, so the Rust and Kotlin sides cannot observe different plans.

Converting `fn_plans` and `iface_spec` from lazy memos to eager members is now a named work item, and the exit adds a separately checkable assertion: **no `RefCell`/`OnceCell` plan memo remains reachable from an emitter.** That is what distinguishes a real migration from a wrapped lookup. The frozen adapter-owned store remains acceptable as an alternative, but must state the same three properties.

## P1 — unit-enum grammar

Correct, and it was a half-application on my part: I added the prose rule and left `unit-enum-as-Choice` in the `leaf` production, so #198 asserted both. Fixed — `leaf := scalar | bool | u64 | string`, with unit enums generated through the ordinary `Choice` production using all-empty variants, and a note that the `leaf` production must not mention enums at all.

## P1 — self-certification

Accepted, and this was the weakest part of the matrix design. The planner deciding legality *and* generating the report means a regression that silently flips a supported cell to `Unsupported` gets reported as a successful classification. "Fails until newly introduced cells are classified" was vacuous.

Three mechanisms added to #198:

- **`REPORT.md` committed and byte-identity checked in CI** — the primary gate. Any new or changed classification becomes a reviewed diff, so a silent flip needs someone to approve it.
- **A committed minimum-guarantee table** — `must plan` / `must compile` / `must execute` for established acceptance shapes. A cell listed `must execute` that reports only `PlanSupported` fails the build. Seeded with the zenoh-flat#30/#31 acceptance shapes, the sum positions pinned by #166, and every cell the covertest currently touches.
- **`ToolchainCompiled` and `RuntimeExercised` name their evidence** — the fixture or test supplying it. A state claimed without a named fixture is a reporting bug, not a coverage level.

Your framing is right that these are regression expectations about the planner's output rather than an independent authority on legality, so the non-goal stands. I've recorded that reasoning in the issue so it isn't relitigated later.

## P2 — Stage 5 split

Agreed that "whichever lands first" was an acknowledgement rather than a plan. I took a combination of your first and third options:

- **#186 is rescoped to Stage 5A** — `KtExpr` / `KtStmt` / `KtType`, the renderer, scope-tracked temporaries, tree-derived imports. **Lands before Stage 3.**
- **#199 is Stage 5B** — `AccessPath` from the plan, deleting `replace_ident`, `render_handle_close`, the #159 hoisted-singleton reassembly text. **Depends on #186 and #193.**

Ordering rationale, now in #193 and the umbrella: Stage 3's Kotlin emission targets the AST directly. Emitting strings there and migrating afterwards writes that emission twice, and Stage 3 deletes `FlatLeaf` — whose `kt_access_prefix`/`kt_access_tail` are the textual template being removed — so the string version would be born dead. Making #186 depend on Stage 3 instead (your second option) has the same problem in reverse.

Both halves are now independently landable: 5A touches no call sites, 5B has a concrete input in the value plan.

## Umbrella

Updated: stage table and graph carry 5A/5B and the new ordering; the Tier 2 invariant now states the plan-carriage requirement and names the two existing lazy caches as the specific thing it rules out.


## Comment 12
<a id="comment-12"></a>

**milyin** — 2026-07-26 16:58 UTC — original id `5084477642`

---

codex review:

## Verdict

The previous plan-carriage and matrix findings are now addressed well. The central architecture remains approved.

Two new sequencing contradictions were introduced by the revisions and should be fixed before implementation.

### P1 — Stage 5A was not actually rescoped, creating a dependency cycle

The umbrella calls [#186](https://github.com/milyin/prebindgen/issues/186) infrastructure-only and makes Stage 3 depend on it. But #186 still requires:

- deleting `replace_ident`;
- removing `kt_access_prefix`/`kt_access_tail`;
- migrating all generated expressions;
- eliminating manual import registration.

Those migrations are now assigned to [#199](https://github.com/milyin/prebindgen/issues/199), which depends on Stage 3. As written:

```text
Stage 3 depends on #186
#186 acceptance includes #199 work
#199 depends on Stage 3
```

Rewrite #186’s work and acceptance criteria to cover only `KtExpr`/`KtStmt`/`KtType`, rendering, scope allocation, import collection, and isolated renderer tests. All production-call-site migration and global “no string emission remains” criteria belong in #199.

### P1 — ownership of structured access paths is still contradictory

[Stage 3](https://github.com/milyin/prebindgen/issues/193) says:

- `FlatLeaf` is deleted;
- its facts move into `SlotPlan`;
- Kotlin output targets `KtExpr` directly.

But #199 says plan-carried `AccessPath` is introduced only after Stage 3.

That leaves no coherent intermediate representation: once `FlatLeaf` and its prefix/tail template are gone, Stage 3 already needs structured information to emit expressions such as the tag-gated variant access. It cannot safely reconstruct that information in the emitter and still satisfy the plan-only invariant.

`AccessPath` should therefore be owned by Stage 3 and carried by `SlotPlan` or another derived plan view. Stage 5B can then migrate the remaining non-plan expressions—`replace_ident`, close chains, hoisted reassembly—and delete obsolete text utilities.

### P1 — frozen plans are ordered after the validation that currently builds them

[Stage 4A](https://github.com/milyin/prebindgen/issues/195) says `ResolvedPlans` is built once “after validation.” Today, however, `validate_resolved` calls `validate_bindings`, which builds every `JniFunctionPlan` and uses those plans for symbol-collision validation ([registry.rs:1343](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/registry.rs:1343), [fn_plan.rs:271](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/fn_plan.rs:271)).

Removing the lazy caches requires this order:

```text
resolve converters
→ build ResolvedPlans exactly once
→ validate that same frozen store
→ construct Generation
```

Alternatively, one operation can build and validate the store atomically. Building it after `validate_resolved` would either rebuild the plans or require validation to retain the `RefCell` caches that Stage 4A promises to remove.

### Minor consistency fix

The umbrella’s “Superseded” section still says “#186 stays open as Stage 5.” It should identify #186 as 5A and #199 as 5B.

Once those ownership and ordering points are corrected, the plan looks implementation-ready; I see no remaining foundational architecture problem.


## Comment 13
<a id="comment-13"></a>

**milyin** — 2026-07-26 17:00 UTC — original id `5084488787`

---

All four fixed. Two of them were execution errors on my side rather than design gaps, which is worth saying plainly.

## P1 — #186 was never actually rescoped

Correct, and this one is on me: last round I *commented* the rescope on #186 and edited the umbrella and #193 to match, but left #186's body untouched. So the issue still carried the migration acceptance criteria while the umbrella called it infrastructure-only — producing exactly the cycle you traced:

```text
Stage 3 depends on #186
#186 acceptance includes #199 work
#199 depends on Stage 3
```

A comment describing a rescope is not a rescope. #186's body is now rewritten and retitled *"Stage 5A: Kotlin expression AST infrastructure (KtExpr)"*, scoped to `KtExpr`/`KtStmt`/`KtType`, the renderer, scope-tracked name allocation, tree-derived imports, and the hole-filling/substitution primitives — plus isolated renderer unit tests. It has a closing section listing the five criteria that are **deliberately not there**, so the cycle cannot reappear by someone reading the old acceptance list.

## P1 — `AccessPath` ownership

The contradiction is real and follows directly from Stage 3 deleting `FlatLeaf`. Once `kt_access_prefix`/`kt_access_tail` are gone, the emitter still has to produce `(<base>.field as? Reading.Exact)?.v0 ?: 0L`, and the only way to do that without a plan-carried path is to reconstruct it in the emitter — which is precisely the plan-only invariant Stage 3 exists to establish. Deferring `AccessPath` to 5B would have made Stage 3's exit criterion unsatisfiable in principle.

`AccessPath` now belongs to **#193**, carried by `SlotPlan` (or a derived view), with a work item and an assertion that no emitter reconstructs a Kotlin access from field names or templates.

#199 is rescoped to what has no plan to carry it: `replace_ident` deletion, `render_handle_close`, the #159 hoisted-singleton reassembly text, and the *global* "no string emission / no manual imports remain" properties — which are only checkable once every producer has moved. All three issues now carry an explicit division-of-labour table so no criterion is owned twice.

## P1 — build/validate ordering

Verified, and the dependency is stronger than "validation happens to run first": `validate_bindings` **builds every `JniFunctionPlan`** and reads each plan's `native_symbol` for the `#[no_mangle]` collision table (`core/registry.rs:1343`, `jni/fn_plan.rs:271`). Validation is a plan *consumer*, so "built after validation" would have meant either building every plan twice or keeping the `RefCell` caches Stage 4A exists to delete — the lazy-cache hole re-entering through the validation path.

#195 now specifies the order explicitly:

```text
resolve converters -> build ResolvedPlans exactly once -> validate that frozen store -> construct Generation
```

with `validate_bindings` consuming the store rather than building plans, a build-and-validate atomic operation named as an acceptable alternative, and a new exit assertion: **validation and emission observe the same store instance — no path rebuilds a plan.**

## Minor

Umbrella's Superseded section now reads #186 as Stage 5A and #199 as Stage 5B.

## Note on the pattern

Three of the last five rounds found an inconsistency introduced by the previous round's edits rather than by the original design — the stale `PlanContext` references, then `unit-enum-as-Choice` left in the `leaf` production, now #186's unrewritten body. The decomposition was supposed to make that structurally harder, and it did for the umbrella, but cross-issue edits still have the same failure mode: I changed three of four places. Worth watching for as the children start moving; a criterion that appears in two issues should appear in a division-of-labour table in both, as it now does for 5A/3/5B.


## Comment 14
<a id="comment-14"></a>

**milyin** — 2026-07-26 17:06 UTC — original id `5084511458`

---

codex review
## Verdict

The previous three blockers are now resolved correctly:

- #186 is genuinely infrastructure-only;
- `AccessPath` is owned by Stage 3;
- frozen plans are built before validation and shared with emission.

The architecture is approved. Two implementation-interface details remain.

### P1 — `ResolvedPlans` has no construction hook

[Stage 4A](https://github.com/milyin/prebindgen/issues/195) adds the associated type and changes consumers, but the trait sketch provides no operation by which generic `Registry::resolve` can construct an adapter-chosen opaque type.

The interface needs something like:

```rust
fn build_resolved_plans(
    &self,
    registry: &Registry<Self::Metadata>,
) -> Result<Self::ResolvedPlans, String>;

fn validate_resolved(
    &self,
    registry: &Registry<Self::Metadata>,
    plans: &Self::ResolvedPlans,
) -> Result<(), String>;
```

Alternatively, `build_resolved_plans` can return an already-validated store and replace `validate_resolved`. The important contract is explicit:

```text
resolve converters
→ build store once
→ validate that store
→ place the same store in Generation
```

Without a producer hook, the associated type solves carriage but leaves construction undefined.

### P1 — the proposed Kotlin AST does not yet make scope safety structural

[Stage 5A/#186](https://github.com/milyin/prebindgen/issues/186) promises capture-safe substitution and renderer-allocated names, but its sketch still represents both references and binders textually:

```rust
Ident(String)
Lambda { params: Vec<String>, ... }
Lit(String)
```

That leaves two escape hatches:

- substitution must still decide which textual `Ident("e0")` belongs to which lambda;
- `Lit(String)` can contain an arbitrary expression, allowing producers to bypass the AST while technically emitting a `KtExpr`.

Use structural binder identity, for example:

```rust
Local(BindingId)
Name(KtName)                 // validated external identifier
Lambda { params: Vec<BindingId>, ... }
Literal(KtLiteral)           // Null, Bool, Int, String, ...
```

The renderer can then assign printable names to `BindingId`s without capture. If raw expressions remain temporarily necessary, represent them with an explicit restricted `Raw` node and require Stage 5B to eliminate it. That makes the global “no string-built expressions” exit mechanically checkable.

### P2 — one stale umbrella sentence remains

The umbrella correctly assigns `AccessPath` to [Stage 3](https://github.com/milyin/prebindgen/issues/193), but still says:

> Plan-carried access paths follow in #199.

That should say that the remaining **non-plan-carried expressions** follow in [#199](https://github.com/milyin/prebindgen/issues/199).

After these corrections, I consider both the architecture and stage decomposition implementation-ready.


## Comment 15
<a id="comment-15"></a>

**milyin** — 2026-07-26 17:08 UTC — original id `5084520284`

---

All three applied.

## P1 — construction hook

Correct: the associated type solved carriage and left construction undefined, which is not implementable — generic `Registry::resolve` has no way to build an adapter-chosen opaque type. #195 now carries `build_resolved_plans(&self, registry) -> Result<Self::ResolvedPlans, String>` as the producer, with `validate_resolved` taking `(&registry, &plans)` so it validates the frozen store rather than building its own copy.

I kept them as two hooks rather than folding validation into the producer, for a reason specific to this codebase: `validate_resolved`'s existing work is not all plan-based — `validate_split_declarations(registry)` is independent of any plan. Collapsing them would either drag registry-only validation into the plan producer or leave it homeless. The atomic build-and-validate alternative is named as acceptable, with the same ordering contract stated either way.

Trait-size note recorded, since #91 is tracking the opposite direction: this adds one method and changes one signature while removing `&Registry` from five emission hooks.

## P1 — structural binder identity

This is the sharpest of the three, and it's the same principle the rest of the plan runs on, applied to the AST itself: `Ident(String)` + `Lambda { params: Vec<String> }` leaves capture-safety to convention, so substitution would still have to reason about which textual `Ident("e0")` belongs to which binder — the exact reasoning `replace_ident` gets wrong, reintroduced by the type that replaces it.

Adopted as specified: `Local(BindingId)`, `Name(KtName)`, `Literal(KtLiteral)`, `Lambda { params: Vec<BindingId> }`. `BindingId` is opaque and tree-allocated; the renderer assigns printable names with real scope tracking, so `e0`/`e1` conventions disappear and capture cannot occur. Typed `KtLiteral` also moves string escaping into the renderer instead of trusting producers to pre-escape.

Your `Lit(String)` observation is the load-bearing half. It was a hole wide enough for any expression, which would have made 5B's global "no string-built expressions" exit **unfalsifiable** — a producer could emit arbitrary text and still be emitting a `KtExpr`. So `Raw(String)` is now the single, conspicuous, crate-private escape hatch, and 5B's exit is mechanical: **every construction site enumerated in #199, then the variant deleted.** The check is that `Raw` does not exist, not that someone inspected the emitters.

Added to 5A's exit: a capture test — substituting an expression containing a `Local` into a lambda binding a same-printed name must not capture it.

## P2 — and a process correction

Fixed, and it was introduced by the very edit that moved `AccessPath` to Stage 3 — I updated the ownership and left the trailing clause describing the old split.

That is the fourth consecutive round where a finding was a stale sentence from my previous round's edits. Noting the pattern twice hasn't stopped it, so I've stopped fixing them one at a time: I pulled all thirteen issue bodies and swept them for the cross-references that have churned — bare `Stage 5` / `Stage 4` / `Stage 2B` without their suffixes, `whichever lands first`, `PlanContext.ownership`, `OwnershipMode`, `slot is Produced`, `unit-enum-as-Choice`, and `after validation`.

Two hits, both benign: #198 and #195 each mention the rejected earlier wording *while correcting it*. No stale statement survives. I'll re-run that sweep after any future cross-issue edit rather than relying on having remembered every location.


## Comment 16
<a id="comment-16"></a>

**milyin** — 2026-07-26 17:12 UTC — original id `5084536383`

---

codex review
## Verdict

The previous findings are now correctly resolved. The plan-carriage interface, validation order, Stage 5 split, and `AccessPath` ownership are coherent.

One required Kotlin-AST correction remains.

### P1 — deleting `KtExpr::Raw` does not close every raw-expression path

[Stage 5B/#199](https://github.com/milyin/prebindgen/issues/199) says the global “no string-built expressions” property is mechanically enforced when `KtExpr::Raw` disappears. But the existing Kotlin model has several other expression-bearing string/code fields:

- `KtBody::{Expr(Code), Block(Code)}` ([model.rs:321](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:321));
- enum-entry arguments ([model.rs:191](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:191));
- constructor and function parameter defaults ([model.rs:199](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:199), [model.rs:404](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:404));
- property initializers, delegates and accessors ([model.rs:426](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:426));
- raw declarations.

Consequently, a producer could still concatenate an expression into `Code` or one of those `String` fields after `KtExpr::Raw` is deleted.

Stage 5A should introduce typed AST-bearing replacements for every expression position. Stage 5B should migrate or remove the legacy fields and assert:

> No expression-position API accepts `String` or `Code`; any remaining raw-code facility is declaration-only and cannot embed generated user-facing expressions.

### P2 — function parameters are still textual binders

[Stage 5A/#186](https://github.com/milyin/prebindgen/issues/186) says binder identity is structural, but its sketch classifies a parameter under:

```rust
Name(KtName) // class, member, param
```

A function parameter is a binder, just like a lambda parameter. Representing it as `Name("x")` means inserting that expression under a lambda that prints the same name can still capture it.

Either:

- give function/constructor parameters `BindingId`s and reference them through `Local(BindingId)`; or
- define `Name` as a free-name set that the renderer must reserve when allocating binders, with an explicit capture test.

The first option better matches the stated “binder identity is structural” invariant.

With those AST-boundary corrections, I consider the entire plan implementation-ready. The core boundary-planning architecture itself has no remaining blocker.


## Comment 17
<a id="comment-17"></a>

**milyin** — 2026-07-26 17:14 UTC — original id `5084543863`

---

Both applied. The P1 is a straightforward correction of a claim I made too strongly, and the verification is unambiguous — the existing model's own doc comments say "raw Kotlin text."

## P1 — `Raw` deletion proves nothing on its own

Verified all five positions, and the field docs state the problem better than a summary can:

| Position | Field | Doc says |
|---|---|---|
| function body | `KtBody::Expr(Code)` / `Block(Code)` (`model.rs:321`) | single-expression / block body |
| enum entry args | `KtEnumEntry.args: Option<String>` (`model.rs:191`) | *"Constructor argument text"* |
| ctor param default | `KtCtorParam` (`model.rs:199`) | — |
| fn param default | `KtParam.default: Option<String>` (`model.rs:404`) | *"raw Kotlin text, e.g. a lambda literal"* |
| property init / delegate | `KtProperty` (`model.rs:426`) | *"Raw initializer expression text"* |

All five predate `KtExpr` entirely, so a producer can concatenate into any of them without ever constructing one. My "the check is that `Raw` does not exist" was measuring the newest and narrowest hole while five older ones stayed open — and worse, it would have read as *proof* while being merely necessary.

- **#186 (5A)** now introduces a typed replacement for **every** expression position, alongside the legacy fields.
- **#199 (5B)** migrates the callers, deletes the `String`/`Code` variants, and carries the corrected exit:

  > No expression-position API accepts `String` or `Code`; any remaining raw-code facility is declaration-only and cannot embed a generated user-facing expression.

`Raw`'s deletion is now one clause of that, not the whole test.

## P2 — function parameters are binders

Correct, and it was the position most likely to appear in a hole, which makes it the worst one to get wrong. I took your first option: **every binder carries a `BindingId`** — lambda parameters, function parameters, constructor parameters, local `val`/`var` — and `KtName` is now a **free-name set only** (class, member, type reference).

I also kept a weakened form of your second option as a secondary guard rather than an alternative: the renderer reserves every free `KtName` reachable in a tree before allocating printable names for its binders, so a machine-allocated binder cannot shadow a referenced free name either. Two capture tests in the exit, one per direction.

## The sweep earned its keep

Fixing this produced exactly the failure I described last round: I added the prose saying parameters are binders and left `// validated external identifier (class, member, param)` sitting in the code sketch twenty lines above it. The sweep caught it before this reply, not a review round later.

That is the first time one of these has been caught by process rather than by you, which is the outcome I was aiming for. The sweep now runs after every cross-issue edit.


## Comment 18
<a id="comment-18"></a>

**milyin** — 2026-07-26 17:18 UTC — original id `5084557974`

---

codex review
## Verdict

The core #187 architecture and stage graph are now sound. The previous findings are fixed.

Two remaining issues are confined to the Stage 5 migration contract.

### P1 — the raw-expression inventory is still incomplete

[#186](https://github.com/milyin/prebindgen/issues/186) and [#199](https://github.com/milyin/prebindgen/issues/199) enumerate five expression positions, but the existing model has additional paths:

- `KtClass::supertypes` contains constructor-argument text such as `NativeHandle(initialPtr)` ([model.rs:253](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:253)).
- `KtDecl::Raw { code: Code }` can contain entire declarations, including bodies and initializers ([model.rs:70](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:70)).
- Property accessors are not expressions; they need a structured getter/setter model containing typed bodies, rather than simply becoming `KtExpr`.
- Any temporary `KtStmt::Raw` would be another bypass and must be covered alongside `KtExpr::Raw`.

Therefore “raw code survives only at declaration position” is insufficient: a raw declaration can itself contain generated expressions. Either remove `KtDecl::Raw`, or restrict it to something mechanically static such as audited `&'static str` with no generated fragments.

The exit should cover every API capable of embedding expression text, not only fields whose top-level grammatical category is “expression.”

### P1 — adding typed replacements “alongside” legacy fields creates two authorities

Stage 5A says typed replacements are introduced alongside the legacy `String`/`Code` fields. If that means parallel optional fields, both representations can be populated simultaneously, forcing the renderer to choose which wins—the same duplicated-authority problem #187 is eliminating.

Use exclusive migration types instead:

```rust
enum ExprSlot<T> {
    Legacy(Code),
    Ast(T),
}
```

For properties, model the existing mutual exclusion structurally:

```rust
enum PropertyValue {
    None,
    Initializer(ExprSlot<KtExpr>),
    Delegate(ExprSlot<KtExpr>),
}
```

Bodies, defaults, supertype arguments and accessors should follow the same pattern. Stage 5B then deletes the `Legacy` variants. At no point can one expression position contain both textual and structured answers.

With these two Stage 5 corrections, I would approve the complete plan as implementation-ready. No remaining issue affects the boundary-planning architecture itself.


## Comment 19
<a id="comment-19"></a>

**milyin** — 2026-07-26 17:21 UTC — original id `5084567034`

---

Both applied. Everything verified against `b1e7f46` — and `KtDecl`'s own doc confirms the worst case.

## P1 — inventory now covers seven positions plus two structural cases

The two missing rows are confirmed, and the accessor point is well taken:

| Added | Field | Doc says |
|---|---|---|
| supertype ctor args | `KtClass.supertypes` (`model.rs:253`) | *"Supertypes with optional constructor-argument text: `(NativeHandle, Some("initialPtr"))`"* |
| property accessors | `KtProperty.accessors: Option<Code>` (`model.rs:435`) | *"Raw accessor text … e.g. a custom getter `get() = …`"* |

Accessors correctly get a structured `KtAccessor { kind, body: KtBody }` rather than a `KtExpr` — they are declarations carrying bodies, and flattening them to an expression would have been the wrong shape even after migration.

**`KtDecl::Raw` is the decisive one.** Its own doc reads *"A top-level (or member-level, for `Raw`) declaration"* — so it reaches member position and can contain bodies and initializers. That makes "raw code survives only at declaration position" not merely weak but **vacuous**: the declaration position is precisely where generated expressions can hide. #199 now either deletes it or narrows it to `&'static str`, so the restriction is enforced by the compiler rather than by audit, and the exit says explicitly that the declaration-position formulation is not acceptable on its own.

`KtStmt` doesn't exist yet — 5A introduces it — so any `Raw` variant it needs is covered by the same enumerate-then-delete contract.

## P1 — "alongside" was two authorities, and it fixed an older one too

Correct, and the word was mine: parallel optional fields mean both can be populated and the renderer picks a winner, which is the defect #187 exists to remove, recreated inside its own migration. `ExprSlot<T> { Legacy(Code), Ast(T) }` adopted for bodies, defaults, supertype arguments, enum-entry arguments and accessors; #199 deletes the `Legacy` variants.

`PropertyValue` is worth calling out beyond its migration role. `KtProperty` currently holds `initializer: Option<String>` **and** `delegate: Option<String>` with a doc comment saying they are *"Mutually exclusive with `initializer`"* — a product where a sum belongs, exclusivity enforced by prose. That is the #180 defect verbatim, sitting in the Kotlin model since before any of this work, and your suggested enum fixes it structurally rather than just carrying it across the migration.

## One distinction, stated so it doesn't become round ten

The sweep flagged three surviving uses of "alongside the legacy" — in #187, #192 and #197 — all describing **shadow planning**. They are not the defect you found, and the difference is worth pinning down:

- **Shadow planning:** the legacy path drives emission, the new plan is built beside it and asserted *equal* on every matrix cell, then emission switches in one step. There is always exactly one authority, plus a differential check on a candidate.
- **The `ExprSlot` case:** two fields on one struct, either or both populated, no rule about which wins and no assertion that they agree. Two authorities.

So shadow planning stays, and the flag was mine to resolve rather than a finding. I'm noting it because "build the new thing alongside the old" reads identically in both and the distinction is the presence of a defined winner and an equality assertion.


## Comment 20
<a id="comment-20"></a>

**milyin** — 2026-07-26 18:23 UTC — original id `5084797132`

---

fresh codex review
## Verdict

The core boundary-planning architecture is sound, but the plan is not yet implementation-ready. Four P1 design gaps remain.

### P1 — Stage 4A misses registry-backed emission paths

[#195](https://github.com/milyin/prebindgen/issues/195) migrates five emission hooks, but `post_process_item` is a sixth hook that still receives the registry and can rewrite every emitted item:

- Trait: [prebindgen.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/prebindgen.rs:443)
- Invocation: [write.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/write.rs:131)
- JNI implementation: [trait_impl.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/trait_impl.rs:1543)

The Kotlin writer also still receives the registry directly: [kotlin_emit.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/kotlin_emit.rs:66).

Stage 4A should explicitly migrate `post_process_item` and both artifact writers to the frozen plan/symbol store. Its exit check should cover the complete emission call graph, not only selected directories or five named hooks.

### P1 — the C plan contradicts the direction-typed invariant

The umbrella requires `ValuePlan<D>` so direction is uniform throughout a plan tree. But [#192](https://github.com/milyin/prebindgen/issues/192) proposes:

```rust
enum CValuePlan {
    Input { ... },
    Output { ... },
}
```

That makes direction a root discriminant rather than a type parameter and does not structurally prevent mixed-direction recursive children—the exact invalid state the umbrella says must be impossible.

Use `CValuePlan<D: CDir>`, or separate `CInputPlan`/`COutputPlan` types with direction-specific recursive nodes.

### P1 — `BindingId` needs provenance and a spelling policy

[#186](https://github.com/milyin/prebindgen/issues/186) says IDs are “allocated by the tree,” while substitution and hole-filling combine trees. Two independently built trees can both allocate the same numeric ID; grafting one beneath the other can then cause structural capture despite scope-aware rendering.

Also, renderer-selected names cannot replace public function/constructor parameter spellings. Those names are part of Kotlin’s named-argument source API; the existing model stores them explicitly at [model.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:404).

The binder model needs both:

```rust
struct Binder {
    id: BindingId,                 // arena-qualified or globally generative
    spelling: Fixed(KtName) | Fresh(NameHint),
}
```

Cross-tree grafting must alpha-remap IDs. Add tests for independently allocated colliding IDs and byte-identical preservation of fixed public parameter names.

### P1 — Stage 5’s “mechanical” raw-expression exit is still false

Two holes remain in [#186](https://github.com/milyin/prebindgen/issues/186)/[#199](https://github.com/milyin/prebindgen/issues/199):

- Narrowing `KtDecl::Raw` to `&'static str` does not prove it came from a literal; dynamically generated text can be leaked into a `'static` reference. Delete `Raw`, or use a private newtype constructible only through a literal-accepting macro.
- Annotations remain `String` APIs in constructor parameters, classes, functions, and properties—for example [model.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/gen/kotlin/model.rs:339). Annotation text can contain expressions, as existing `Suppress("UNCHECKED_CAST")` calls demonstrate.

To make “no expression-bearing API accepts `String` or `Code`” true, introduce a typed `KtAnnotation { name, args: Vec<KtExpr> }` or restrict annotations to mechanically static audited tokens.

No files or GitHub issues were changed during this review.


## Comment 21
<a id="comment-21"></a>

**milyin** — 2026-07-26 18:25 UTC — original id `5084804693`

---

All four applied. Every claim verified against `b1e7f46` first; two of the four were internal contradictions in my own decomposition rather than gaps in the design.

## P1 — the emission call graph, not a list of hooks

`post_process_item` is the one that matters most, and its own doc explains why: *"Walks the entire AST, not just signatures, so type ascriptions and casts inside function bodies are covered."* It receives the registry (`core/prebindgen.rs:443`), runs over **every** emitted item at `core/write.rs:131`, and is implemented at `jni/trait_impl.rs:1543`. A registry-backed rewrite pass over finished output makes plan-only emission upstream of it worthless — pruning could change what that pass sees, silently, exactly as with `prerequisites`.

`write_kotlin` is a second independent path: `self.adapter().write_kotlin(self.registry(), kotlin_root)` (`jni/kotlin_emit.rs:66`) hands the registry straight to the Kotlin writer, bypassing every hook I had listed.

The root error was the *metric*, not the omissions. Scoping "Tier 2 emitter lookups" to directories (`jni/emit/**`, `render.rs`, `kotlin_emit.rs`) meant the check could pass while both of these kept their registry. #195 now scopes it by **reachability from `write_rust` / `write_kotlin`**, with an explicit exit that no function reachable from an artifact writer takes `&Registry`.

## P1 — `CValuePlan` contradicted the umbrella it implements

Correct, and this is mine: when I applied the direction-as-type-parameter fix I updated the umbrella's Tier 1 section and #193, and left #192 carrying the earlier `enum CValuePlan { Input, Output }`. A root discriminant fixes direction only at the root — nothing stops a `Product`'s child being the other variant, which is precisely the state the umbrella says must be unrepresentable.

#192 now has `CValuePlan<D: CDir>` with recursive nodes at the same `D`, and records why the enum form is insufficient so it isn't reintroduced.

My sweep didn't catch this because I was grepping for stale *phrases*, not for structural divergence between an umbrella invariant and a child's type sketch. I've added a check for root-discriminant plan enums; it now reports none, and `ValuePlan<D>` appears in #187, #192 and #193 consistently.

## P1 — binder provenance and spelling

Both halves are right, and the second is the more dangerous:

- **Collision across arenas.** Grafting is exactly what substitution and hole-filling do, and two independently-built trees that each allocated `BindingId(0)` are indistinguishable at render time — scope-aware rendering cannot detect a collision it has no way to see. IDs are now arena-qualified or globally generative, with **alpha-remapping on graft**.
- **Not every binder may be renamed.** I had said the renderer assigns printable names to all binders, which would have silently broken every `foo(bar = …)` call site: function and constructor parameter names are Kotlin's named-argument surface, stored explicitly at `model.rs:404`. `Binder { id, spelling: Fixed(KtName) | Fresh(NameHint) }` — identity structural everywhere, naming renderer-controlled only for `Fresh`, and `Fresh` allocation must avoid colliding with any `Fixed` or free name in scope.

Exit now carries four tests: two capture directions, colliding-`BindingId` graft, and byte-identical preservation of `Fixed` spellings.

## P1 — the raw exit was still false, twice

- **`&'static str` proves lifetime, not literal origin.** `String::leak` and `Box::leak` produce one from dynamically built text, so the narrowing I proposed was decorative. `KtDecl::Raw` is now deleted, or its payload becomes a private newtype constructible only through a literal-accepting macro — which does make origin a compile-time property.
- **Annotations were missing from the inventory entirely**, and they carry expressions: `.annotation("Suppress(\"UNCHECKED_CAST\")")` at `jni/overloads.rs:659` and `jni/render.rs:800` is a call with a string-literal argument, built by concatenation today. `KtFun.annotations: Vec<String>` (`model.rs:339`) and the same on classes, properties and ctor params. Typed `KtAnnotation { name, args: Vec<KtExpr> }` added to 5A, migration and deletion to 5B.

That is the third round in which the "complete" raw-expression inventory grew. The pattern is consistent: each time I enumerated the positions I could find rather than deriving the closure of "APIs that accept text which reaches generated output." The exit is now written as that closure — every API able to embed expression text — with the table as evidence rather than as the definition.


## Comment 22
<a id="comment-22"></a>

**milyin** — 2026-07-26 18:36 UTC — original id `5084846723`

---

codex review
## Verdict

The four previous findings are resolved. Three P1 gaps remain.

### P1 — reachability manifests track symbol names, not artifact identity

[#194](https://github.com/milyin/prebindgen/issues/194) records:

```text
Symbol -> contributing ReachabilityRoot[]
```

But converter collection deduplicates by name and keeps the first body encountered: [write.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/write.rs:145).

If two roots contribute the same symbol with different bodies, pruning the root that currently wins changes the surviving body without removing the symbol. The proposed manifest would report that symbol as still reachable and therefore miss the change.

Record `(root, symbol, normalized-item hash)` and assert that all contributions sharing a symbol are byte/AST-identical before deduplication. Stage 2B.2 should compare artifact hashes, not only symbol presence.

### P1 — cycle representation is promised but never exercised

[#190](https://github.com/milyin/prebindgen/issues/190) promises interned back-references and closes #136, but its only exit assertion checks the Tier 0 module boundary. [#198’s matrix](https://github.com/milyin/prebindgen/issues/198) generates bounded trees and has no back-reference production, while #191 tests stalled dependency cycles rather than whether Tier 0 built the correct cyclic graph.

Stage T needs direct tests for:

- self-recursion, such as `Node { children: Vec<Node> }`;
- mutual recursion;
- distinct types with the same short name not being mistaken for a cycle;
- finite graph construction with the expected `ShapeId` back-edges.

Then Stage 2A should test both an opaque-cut legal cycle and a genuinely stalled SCC.

### P1 — annotation migration has two stage owners and no exclusive bridge

[#186](https://github.com/milyin/prebindgen/issues/186) says infrastructure only, with no production call-site migration, but its work list says `KtAnnotation` replaces the existing `Vec<String>` APIs. [#199](https://github.com/milyin/prebindgen/issues/199) separately owns migrating annotations and deleting those same APIs.

As written, either 5A must migrate callers—contradicting its scope—or it introduces the typed API alongside `Vec<String>`, recreating the two-authority problem already fixed for other expression slots.

Give annotations the same exclusive migration structure:

```rust
enum AnnotationSlot {
    Legacy(StaticAnnotationText),
    Ast(KtAnnotation),
}
```

Stage 5A changes the field to `Vec<AnnotationSlot>` and mechanically wraps existing producers as `Legacy`; Stage 5B migrates each entry and deletes `Legacy`.

No files or GitHub state were changed.


## Comment 23
<a id="comment-23"></a>

**milyin** — 2026-07-26 18:39 UTC — original id `5084855880`

---

All three applied.

## P1 — symbol name is not artifact identity

Verified, and the mechanism is `or_insert_with` in `collect_converter_items` (`core/write.rs:145`): the **first** contribution of a name wins and later ones are discarded silently, in walk order `input_types` then `output_types`.

The consequence you describe is the part my manifest could not see. Pruning the root that currently wins doesn't remove the symbol — it **substitutes the body**, and a presence-keyed manifest reports the symbol as still reachable. A body swap with no diff in the symbol set, passing every exit criterion I had written.

`symbol manifest: Symbol -> [ (ReachabilityRoot, ItemHash) ]` now, with 2B.1 asserting **no symbol has two contributions with differing `ItemHash`**, and 2B.2 comparing surviving hashes rather than names.

Worth stating explicitly: that identity assertion is also a check on **current** behaviour. Nothing today guarantees the discarded duplicates were equivalent to the winner — the dedup is silent, so a divergence would already be shipping. If the assertion fires on first run, it has found a pre-existing bug rather than a migration regression, and I've noted that in #194 so it isn't mistaken for one.

## P1 — cycles promised, never exercised

Correct, and the gap is exactly the shape this plan claims to close. #190 promised interned back-references and *closes #136*, while its only assertion was the Tier 0 module boundary. Neither of the other candidates covers it: #198 generates bounded trees with no back-reference production, and #191 tests stalled **dependency** cycles at resolution time — a different graph entirely from Tier 0's shape graph.

So the feature that motivated interning had no test, in the tier that introduces it. #190 now asserts all four cases, and the third is the one that earns the #136 closure, since #136's actual defect is bare-ident keying:

- self-recursion `Node { children: Vec<Node> }` interns finitely with a `ShapeId` back-edge;
- mutual recursion `A { b: B }` / `B { a: A }` yields two nodes and two back-edges;
- **two distinct types sharing a short name are not conflated into a cycle**;
- back-edges are asserted structurally as the expected `ShapeId`s, not inferred from termination.

#191 adds the SCC pair: a legal opaque-cut cycle resolves; a genuinely stalled SCC reports `Unsupported` naming the path, distinguishable from a missing leaf.

## P1 — annotations had no exclusive bridge

Right, and it is the same omission twice over: annotations entered the inventory one round after `ExprSlot` was introduced, so they got the typed replacement without the bridge, and the work item landed in 5A while migration lived in 5B — the two-owner contradiction on top of the two-authority one.

`AnnotationSlot { Legacy(StaticAnnotationText), Ast(KtAnnotation) }` adopted as specified. 5A changes the field to `Vec<AnnotationSlot>` and mechanically wraps existing producers as `Legacy` — a type change, no call-site migration, so infrastructure-only scope holds. 5B migrates each entry and deletes `Legacy`. `Legacy` carries the literal-origin newtype rather than a bare `String`, matching the `KtDecl::Raw` constraint.

Every expression-bearing position now has an exclusive bridge: `ExprSlot` for bodies, defaults, supertype args and enum-entry args; `PropertyValue` for initializer/delegate; `AnnotationSlot` for annotations; structured `KtAccessor` for accessors.

## Sweep

Extended with checks for the two structural classes these rounds have surfaced — root-discriminant plan enums contradicting `ValuePlan<D>`, and presence-keyed manifests — plus a check that no typed API is introduced without a bridge type. All clean. The surviving "alongside" matches are shadow planning and ordinary module coexistence, neither of which is the two-authority pattern.


## Comment 24
<a id="comment-24"></a>

**milyin** — 2026-07-26 18:51 UTC — original id `5084899186`

---

codex review

## Verdict

The previous three findings are fixed. Two P1 gaps and one P2 test defect remain.

### P1 — lifecycle plans do not model aliasing between parameters

The invariant “every consumed input resource reaches exactly one commit or rollback” is only enforceable per slot. Two slots can reference the same runtime resource.

Examples:

- C reconstructs ownership independently with `Box::from_raw` for each consumed argument: [trait_impl.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/cbindgen/trait_impl.rs:21).
- JNI collects each consuming parameter independently and marks handles consumed only after the native call: [render.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/render.rs:1567), [render.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/render.rs:1711).
- Existing fixtures already declare functions with two owned handles: [flatten.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/tests/flatten.rs:1513).

Passing the same handle twice can therefore reconstruct the same allocation twice. Aliasing is also invalid for `ExclusiveBorrow + anything` and `Consume + borrow`.

Stages 1/3 need a function-level `AliasPlan` derived from all active lifecycle/access plans:

- shared/shared may alias;
- exclusive or consumed resources must be pairwise distinct from every other active access;
- nullable and inactive choice slots are ignored.

Stage 0 should tactically reject affected declarations or add preflight pointer-identity checks. [#198](https://github.com/milyin/prebindgen/issues/198) also needs multi-parameter alias cases; its current `use × position` cells cannot exercise relationships between roots.

### P1 — a stalled SCC is incorrectly reclassified as `Unsupported`

[#191](https://github.com/milyin/prebindgen/issues/191) introduces `Ready / Needs / Unsupported` specifically to distinguish deferral from rejection. Its amended exit now says a genuinely stalled SCC reports `Unsupported`, collapsing those channels again.

A leftover `Needs` graph is a resolution failure, not an adapter decision that the semantic shape is unsupported. Otherwise a missing dependency or resolver regression can appear in the matrix as a legitimate `Unsupported` cell.

Keep it separate:

```rust
ResolveError::StalledCycle { path, blockers }
ResolveError::MissingDependency { dependency, provenance }
```

Matrix generation should fail on either; only an explicit planner rejection should produce `Unsupported(reason)`.

### P2 — the mutual-recursion fixture is not legal Rust

[#190](https://github.com/milyin/prebindgen/issues/190) proposes:

```rust
A { b: B }
B { a: A }
```

That is infinitely sized and cannot pass through the real source-crate pipeline. Use legal indirection, for example:

```rust
A { bs: Vec<B> }
B { as_: Vec<A> }
```

The test should compile through the actual capture path, not only parse as `syn` syntax.

No files or GitHub state were changed.


## Comment 25
<a id="comment-25"></a>

**milyin** — 2026-07-26 18:53 UTC — original id `5084907834`

---

All three applied. The aliasing finding is the first one in these rounds that is a live soundness bug rather than a plan defect, and the fixture that demonstrates it is already committed.

## P1 — aliasing

Verified end to end. Each consumed C argument reconstructs ownership independently (`cbindgen/trait_impl.rs:21`), JNI collects each consuming parameter independently and marks handles consumed only after the native call (`jni/render.rs:1567`, `:1711`), and `z_combine(primary: ZThing, fallback: ZThing)` (`jni/tests/flatten.rs:1513`) is a committed fixture with two owned handles of one type. `z_combine(x, x)` reconstructs one allocation twice.

The framing is what I had wrong, not the checklist: my invariant is quantified over slots, and aliasing is a property of a **call**. No amount of per-slot rigour reaches it — the two slots are each individually correct. And it is broader than double-consume, as you note: `ExclusiveBorrow` aliased with any other active access is UB, and `Consume` aliased with a borrow frees memory the borrow still points at.

- **#192 / #193** gain a function-level `AliasPlan` derived from all active lifecycle/access plans, with your three rules: shared/shared may alias; every exclusive or consumed resource pairwise distinct from every other active access; nullable slots and inactive choice groups excluded.
- **#189 (Stage 0)** gains the stopgap, since this is exploitable now: reject declarations with two or more consumed/exclusive parameters of the same type, or emit a preflight pointer-identity check. Tactical and superseded by 1/3.
- **#198** gains alias fixtures as a **function-level** dimension, with the reason stated — `use × position` describes one root and structurally cannot express a relationship between roots, so this could never have been a cell.

## P1 — stalled SCC is not `Unsupported`

Correct, and it was my regression from last round: I introduced `Ready`/`Needs`/`Unsupported` in 2A precisely to separate deferral from rejection, then wrote an exit that routes a stalled SCC to `Unsupported` — collapsing the channels in the issue that exists to keep them apart.

The consequence is worse than the category error. #198 classifies cells from the planner's own answer, so a resolver regression or a missing dependency would have appeared as a legitimate `Unsupported` cell — a resolution failure laundered into a supported-shape verdict, in the report whose byte-identity check is supposed to catch exactly that.

```rust
ResolveError::StalledCycle      { path, blockers }
ResolveError::MissingDependency { dependency, provenance }
```

Only an explicit planner rejection yields `Unsupported(reason)`; a `ResolveError` **fails matrix generation** rather than being recorded. Both #191 and #198 now say so.

## P2 — the fixture was not Rust

`A { b: B }` / `B { a: A }` is infinitely sized and would not compile, so the test I specified could only ever have exercised `syn` parsing. Replaced with `A { bs: Vec<B> }` / `B { as_: Vec<A> }`, and #190 now states that cycle fixtures must compile through the real capture path rather than merely parse — which matters because interning is meant to work on captured items, not on hand-built syntax trees.

## Sweep

Extended with checks for both new classes: a stalled SCC routed to `Unsupported`, and the illegal mutual-recursion fixture. Both clean; the single remaining match is #198's corrective statement.


## Comment 26
<a id="comment-26"></a>

**milyin** — 2026-07-26 19:49 UTC — original id `5085112942`

---

Next codex review:

## Verdict

The previous three findings are fixed. Three P1 contract gaps remain.

### P1 — Stage 0’s alias fix is optional and absent from its exit

[#189](https://github.com/milyin/prebindgen/issues/189) permits either rejecting multi-resource declarations or emitting a preflight check, but its reviewed-diff and exit sections mention neither aliasing tests nor affected artifacts.

Rejection is incompatible with existing supported declarations such as `z_combine(ZThing, ZThing)` and with Stage 3’s byte-identical exit. If Stage 0 rejects it, Stage 3 must later restore it and change artifacts.

Stage 0 should require preflight for existing supported shapes and assert:

- same resource in consume/consume, consume/borrow, or exclusive/anything is rejected before conversion;
- distinct resources still work;
- `T` and `Option<T>` are compared by underlying resource domain, not syntactic parameter type;
- the test discriminates when the preflight is removed.

Its reviewed diffs must include the generated Rust/Kotlin wrapper changes. Stages 1/3 should then reproduce that behavior byte-identically from `AliasPlan`.

### P1 — “pure emission” still receives raw source signatures

[#195](https://github.com/milyin/prebindgen/issues/195) removes `&Registry`, but its proposed interface remains:

```rust
fn on_function(&self, item: &syn::ItemFn, plans: &Self::ResolvedPlans)
```

The current hooks expose the complete raw source declarations: [prebindgen.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/prebindgen.rs:483).

An emitter can therefore re-peel `item.sig.inputs` or `item.sig.output` without any registry lookup and still satisfy every current Stage 4A exit assertion. It can also select the wrong entry from the global plan store by identifier.

Pass an item-specific `FunctionArtifactPlan`/`TypeArtifactPlan` containing the resolved crossing plan and only the source tokens needed for emission. Add an exit asserting no writer-reachable path classifies boundary `syn::Type`; zero registry access alone is insufficient.

### P1 — `AccessPath` retains a source identifier, not the resolved Kotlin member

[#193](https://github.com/milyin/prebindgen/issues/193) proposes:

```rust
Field(Box<AccessPath>, Ident)
```

Current emission converts Rust field identifiers to escaped camel-case Kotlin names in multiple locations—for example [flat_input.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/emit/flat_input.rs:22) and [render.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/lang/jnigen/jni/render.rs:101).

If `Ident` is the source `syn::Ident`, the emitter must repeat that destination-name derivation, violating the plan’s stated purpose. If it means a Kotlin identifier, the type is misleading and does not enforce that interpretation.

`AccessPath` should carry a resolved `KtMemberRef`/`KtName`, including escaping and final spelling. Add differential tests for snake-case fields, tuple fields, Kotlin keywords, and any applicable field-name override.

No files or GitHub state were changed.


## Comment 27
<a id="comment-27"></a>

**milyin** — 2026-07-26 19:52 UTC — original id `5085127714`

---

All three applied.

## P1 — Stage 0 must preflight, not reject

The argument that settles it is the one I'd missed: `z_combine(ZThing, ZThing)` is a **supported declaration today**. Rejecting it removes working surface, and Stage 3 would then have to restore it — moving artifacts and breaking its own byte-identical exit. So "reject *or* preflight" wasn't two acceptable options; one of them was incompatible with a later stage's exit criterion, and I offered it as a coin flip.

#189 now requires the preflight, with all four assertions including the discrimination check, and — the part that was simply missing — **reviewed diffs now list the generated Rust and Kotlin wrappers** for every call with two or more consumed/exclusive parameters. A stage that inserts a runtime check into generated wrappers and declares no artifact change was self-contradictory.

The `T` vs `Option<T>` point is the subtlest of the four and I've propagated it to #192 and #193 as well: comparison is by **underlying resource domain**, not syntactic parameter type. Both stages now also state that they must reproduce Stage 0's preflight **byte-identically** from `AliasPlan` rather than replacing it with a rejection.

## P1 — removing `&Registry` was the weaker property

Correct, and this one invalidates an exit criterion rather than merely weakening it. `on_function(&self, f: &syn::ItemFn, registry: …)` (`core/prebindgen.rs:483`) hands over the complete source declaration. An emitter holding the whole `ItemFn` can re-peel `sig.inputs` / `sig.output` and re-derive a boundary classification **with zero registry access**, passing every assertion I had written — while doing exactly what the Tier 2 rule forbids.

Hooks now take item-specific `FunctionArtifactPlan` / `TypeArtifactPlan` carrying the resolved crossing plan for that item plus only the tokens emission needs, with selection done once by the writer. Two exits added:

- no writer-reachable path classifies a boundary `syn::Type` — the property that actually matters, stated separately because zero-registry-access does not imply it;
- hooks receive item-specific plans, so an emitter cannot index the global store by identifier and pick the wrong entry.

## P1 — `AccessPath` carried the wrong name

Also right, and the derivation already exists twice: `mangle_kotlin_ident(&snake_to_camel(&fname))` (`jni/emit/flat_input.rs:22`) and the data-class renderer (`jni/render.rs:101`). A source `syn::Ident` in the path would force a third. Your second horn is the sharper one — if `Ident` were meant as "Kotlin identifier", the type documents the wrong contract and enforces neither reading.

`Field(Box<AccessPath>, KtMemberRef)` now, carrying the final spelling with escaping and any override applied. Differential tests required for snake_case fields, tuple fields, camel-cased names colliding with Kotlin hard keywords, and explicit name overrides.

## Sweep

Extended with three more checks — `AccessPath` carrying a bare `Ident`, hooks taking a raw `syn` item alongside plans, and Stage 0 offering rejection as an option. All clean, along with the previously added classes.

Worth noting what these last rounds have had in common: each found an exit criterion that could be satisfied without the property it was meant to establish — zero registry access without plan-only classification, `Raw` deleted without string-free emission, symbol presence without artifact identity, per-slot obligations without call-level soundness. That is a more useful failure mode to have surfaced repeatedly than a wrong design would have been, and it is the same class of defect the plan targets in the generator: a check that passes for the wrong reason.


## Comment 28
<a id="comment-28"></a>

**milyin** — 2026-07-26 20:10 UTC — original id `5085193293`

---

next codex review

The previous three findings are resolved in the latest amendments. This pass found three new gaps.

1. **P1 — Alias safety still lacks an explicit execution-order plan.**
   [Stage 0](https://github.com/milyin/prebindgen/issues/189) requires alias rejection before conversion, but [Stages 1](https://github.com/milyin/prebindgen/issues/192) and [3](https://github.com/milyin/prebindgen/issues/193) model `AliasPlan`, decoding, and rollback without defining their function-level ordering. Byte-identical migration protects existing shapes, but not future nested products/choices.

   Specify a schedule such as:

   `validate tags/presence → enumerate active resources → alias preflight → ownership-bearing decode → call → commit/rollback`

   Also extend [#198](https://github.com/milyin/prebindgen/issues/198) with runtime-discriminating cases for:

   - aliasing inside an active choice;
   - the same pointers in an inactive choice;
   - an invalid later tag/null after an earlier consumed argument, proving ownership was never acquired.

2. **P1 — Stage 4A collapses the typed planning errors Stage 2A introduces.**
   [#191](https://github.com/milyin/prebindgen/issues/191) deliberately distinguishes `Unsupported(Diagnostic)`, `ResolveError::StalledCycle`, and `ResolveError::MissingDependency`. But [#195](https://github.com/milyin/prebindgen/issues/195) sketches:

   ```rust
   fn build_resolved_plans(...) -> Result<Self::ResolvedPlans, String>;
   ```

   That permits store construction to erase the distinction which the matrix depends on. Use a typed aggregate error preserving `Unsupported` versus `ResolveError`; stringify only at the public reporting edge.

3. **P2 — The short-name collision exit cannot exercise the real registry path.**
   [#190](https://github.com/milyin/prebindgen/issues/190) claims closure of #136 through a test with two distinct types sharing a short name. The actual registry keys source items by bare identifier and rejects duplicate names because the source namespace is flat ([registry.rs](/home/milyin/PREBINDGEN_WORKSPACE/prebindgen/prebindgen/src/api/core/registry.rs:3)). A synthetic shape-graph test can pass without proving the capture/registry pipeline handles that scenario.

   Either:

   - describe this as a defensive `TypeKey` unit invariant and avoid claiming end-to-end coverage; or
   - expand source-item identity beyond bare identifiers, which is a substantially larger scope.

The latest response correctly incorporates the preceding review’s alias-preflight, item-specific artifact-plan, and resolved `KtMemberRef` changes: [maintainer update](https://github.com/milyin/prebindgen/issues/187#issuecomment-5085127714).


## Comment 29
<a id="comment-29"></a>

**milyin** — 2026-07-26 20:12 UTC — original id `5085202993`

---

All three applied.

## P1 — alias safety is an ordering property

Right, and byte-identical reproduction was covering for the gap: it pins today's shapes and says nothing about a nested product or choice added later. The schedule is now stated in #192 and #193, with each dependency spelled out because the order is the whole safety argument:

```text
1. validate tags and presence flags
2. enumerate the ACTIVE resource set (inactive choice groups excluded)
3. alias preflight over that set
4. ownership-bearing decode
5. call
6. commit / rollback
```

1–2 precede 3 because liveness depends on tags — two pointers colliding in an *inactive* group are not an alias. 3 precedes 4 because a preflight that runs after the first `Box::from_raw` has already double-owned; failing *before* any acquisition is the entire point.

Your three runtime cases are in #198, and the third is the one that actually tests the ordering rather than the plan: an invalid tag or null in a *later* argument, after an earlier argument would have been consumed, proves ownership was never acquired. A plan-level check cannot distinguish "preflight exists" from "preflight runs first"; that case can.

The second — same pointers in an **inactive** group must *not* be flagged — is the one I'd have been most likely to get wrong in the conservative direction, where a false positive silently removes working surface.

## P1 — `Result<_, String>` erased 2A's distinction

Correct, and the placement makes it worse than a lost type: `build_resolved_plans` is the single point where every plan is constructed, so stringifying there would launder `ResolveError` back into an undifferentiated failure immediately after 2A separated the channels — and #198 classifies cells from the planner's answer, so the very distinction the matrix depends on would be destroyed at the one call site it flows through.

```rust
struct PlanBuildErrors { unsupported: Vec<Diagnostic>, resolve: Vec<ResolveError> }
```

Stringification only at the public reporting edge (`ScanError::AdapterInvariant`), never inside the pipeline. It also aggregates rather than short-circuits, which fits 2A's collect-every-failure behaviour.

## P2 — the #136 claim was overstated

You're right, and the issue text refutes me directly: *"In prebindgen's flat namespace idents are unique, so this is currently safe."* `check_no_duplicate` rejects duplicate idents across kinds, so two distinct declared types sharing a short name cannot reach capture. Any test of that collision is synthetic by construction.

Taking your first option. #190 now carries it as a **defensive `TypeKey` unit invariant**, stated explicitly as not end-to-end, and the auto-closing `Closes: #136` is gone — replaced by a paragraph distinguishing what is delivered from what is tested:

- **delivered:** interning keys by full `TypeKey`, and Stage 3 deletes `flat_input.rs` outright, so the bare-ident cycle stack at `:650` ceases to exist rather than being corrected. That is #136's first suggested fix, structurally.
- **not claimed:** coverage of the collision scenario, which needs source-item identity beyond bare identifiers — separate scope, substantially larger than this refactor, and worth its own issue only if someone wants it.

I've posted the correction on #136 as well, since my earlier comment there claimed the stronger version.


## Comment 30
<a id="comment-30"></a>

**milyin** — 2026-07-26 20:25 UTC — original id `5085249670`

---

The previous three findings are resolved. This pass found four new gaps.

1. **P1 — Stage 0’s generation predicate misses consume/borrow aliasing.**
   [#189](https://github.com/milyin/prebindgen/issues/189) requires rejecting consume/borrow and exclusive/anything aliases, but its reviewed-diff criterion covers calls with “two or more consumed/exclusive parameters.” That excludes one consumed parameter plus one shared borrow—and one exclusive borrow plus one shared borrow.

   Generate preflight whenever a call contains:

   - at least one `Consume` or `ExclusiveBorrow`; and
   - any other active access in the same resource domain.

2. **P1 — The new schedule does not explicitly prevalidate nullability.**
   [#192](https://github.com/milyin/prebindgen/issues/192) and [#193](https://github.com/milyin/prebindgen/issues/193) begin with “validate tags and presence flags,” while [#198](https://github.com/milyin/prebindgen/issues/198) requires a later invalid null to prove an earlier resource was never acquired. A non-optional null can otherwise remain undiscovered until ownership-bearing decode.

   Make the opening phases explicit:

   `validate tags, presence, nullability and resource eligibility → enumerate active resources → alias check → acquire/decode`

   Alternatively, change the test to prove successful rollback rather than claiming ownership was never acquired.

3. **P2 — `RuntimeExercised` can still be asserted by stale metadata.**
   [#198](https://github.com/milyin/prebindgen/issues/198) requires the state to name its fixture, but a hand-written `cell → fixture-name` mapping can claim coverage even when that fixture never touches the cell—the same failure class cited from #175.

   Couple evidence mechanically: runtime fixtures should emit stable `MatrixCellId` receipts only after the relevant assertion executes, and CI should derive or verify `RuntimeExercised` from those receipts. Apply the same principle to `ToolchainCompiled`.

4. **P2 — Stage 5A still has conflicting migration ownership.**
   [#186](https://github.com/milyin/prebindgen/issues/186) opens with “All production call-site migration is #199,” but later assigns plan-carried emitter migration to [#193](https://github.com/milyin/prebindgen/issues/193); [#199](https://github.com/milyin/prebindgen/issues/199) also documents that split correctly.

   Change the opener to: plan-carried migration belongs to #193; remaining non-plan-carried migration belongs to #199.

The maintainer’s latest response confirms the preceding ordering, typed-error, and flat-namespace findings were incorporated: [latest amendment summary](https://github.com/milyin/prebindgen/issues/187#issuecomment-5085202993).


## Comment 31
<a id="comment-31"></a>

**milyin** — 2026-07-26 20:27 UTC — original id `5085256880`

---

All four applied. These are internal inconsistencies rather than design gaps, and three of the four are the same shape: a criterion narrower than the rule it was meant to enforce.

## P1 — the preflight predicate was narrower than the rejection rule

The rule rejects consume/borrow and exclusive/anything; the reviewed-diff criterion said "two or more consumed/exclusive parameters." Those disagree on exactly the mixed cases — one consumed plus one shared borrow, one exclusive borrow plus one shared borrow — so a call that the rule rejects would have had no preflight generated, and the artifact diff would have looked complete.

#189 now states the generation predicate directly: **at least one `Consume` or `ExclusiveBorrow`, and any other active access in the same resource domain.** The reviewed-diff criterion points at the predicate rather than restating it in weaker terms, which is what let the two drift apart.

## P1 — nullability had to move into step 1

Also right, and it is what makes #198's third case provable. That case asserts a later invalid null shows an earlier resource was **never acquired** — which only holds if every non-optional null is rejected before step 4. With nullability checked at decode time, the strongest available claim would have been successful rollback, a weaker property that does not distinguish "never acquired" from "acquired then released."

Both schedules now open with **validate tags, presence flags, nullability and resource eligibility**, with the reason recorded inline so the step isn't trimmed later by someone who reads it as redundant with decode-time checks.

## P2 — declared evidence is not evidence

This is the self-certification problem one level down, and the #175 citation is apt: a hand-written `cell -> fixture-name` mapping is exactly a test claiming a precondition it never established.

`ToolchainCompiled` and `RuntimeExercised` are now **derived from receipts**, not declared. A fixture emits a stable `MatrixCellId` receipt *only after the relevant assertion executes*; CI derives or verifies the state from the collected receipts; the compile receipt comes from the build that actually compiled the representative. **A cell with no receipt is `PlanSupported`, whatever any mapping claims** — so the failure mode is under-reporting coverage, which is safe, rather than over-reporting it.

## P2 — the 5A opener

Stale from the round where `AccessPath` moved to Stage 3: I updated the body and the division-of-labour table and left the opening sentence saying all call-site migration is #199. Corrected to name both destinations — plan-carried to #193, non-plan-carried to #199, neither here.

That is the second time an opening line has outlived the body beneath it. I've added both this and the predicate/rule divergence to the sweep, which now covers fourteen classes and is clean.

