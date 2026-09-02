# Baseline for #675

The figures the plan in #675 compares against, recorded once because the
`tools/line-report` script bakes in its workspace path at build time and cannot
be pointed at an older commit without being rebuilt inside a worktree of it.

Measured on `main` at `eb4aa007`, the #513 merge.

## Production lines

The `production` column of `tools/line-report`, which counts the lines left
after every `#[cfg(test)]` item and every file under a `tests/` directory are
removed.

| crate | production | test items | test files | total |
|---|---|---|---|---|
| `prebindgen` | 2,821 | 468 | 666 | 3,955 |
| `prebindgen-flat` | 5,270 | 132 | 2,919 | 8,321 |
| `prebindgen-registry` | 13,658 | 705 | 7,505 | 21,868 |
| `prebindgen-c` | 8,755 | 81 | 5,826 | 14,662 |
| `prebindgen-jni` | 35,431 | 1,578 | 21,832 | 58,841 |
| `prebindgen-c-runtime` | 65 | 0 | 0 | 65 |
| `prebindgen-jni-runtime` | 368 | 0 | 0 | 368 |
| `prebindgen-proc-macro` | 462 | 0 | 0 | 462 |
| all | 66,830 | 2,964 | 38,748 | 108,542 |

The plan's second figure, production lines across `prebindgen-registry`,
`prebindgen-c` and `prebindgen-jni` together, is **57,844** here.

## Registry-facing files

An adapter source file is registry-facing when it names any of
`prebindgen_registry::recipe`, `prebindgen_registry::generation`,
`prebindgen_registry::chain` or `prebindgen_registry::write`. The set is
re-derived at every measurement with:

```
grep -rlE 'prebindgen_registry::(recipe|generation|chain|write)\b|use prebindgen_registry::\{[^}]*\b(recipe|generation|chain|write)\b' --include='*.rs' src | grep -v '/tests/'
```

At `eb4aa007` that selects, in `prebindgen-c/src`: `assembly.rs`, `chain.rs`,
`compile.rs`, `lib.rs`, `recipes.rs`, `trait_impl.rs`.

In `prebindgen-jni/src/jni`: `chain.rs`, `compile.rs`, `emit/callback.rs`,
`emit/delivery.rs`, `emit/flat_input.rs`, `emit/wrapper.rs`, `fn_plan.rs`,
`generation.rs`, `iface.rs`, `kotlin_emit.rs`, `mod.rs`, `recipes.rs`,
`trait_impl.rs` — about 21,000 production lines, of which `kotlin_emit.rs` and
`iface.rs` are Kotlin emission that names registry types.

The plan's first figure is the production lines of those files, summed over
both adapters. At `eb4aa007` it is **28,348**, which is what the plan's end is
judged against. Per-file production counts come from the same script:

```
cargo run --manifest-path tools/line-report/Cargo.toml -- --files prebindgen-c prebindgen-jni
```

## Gates

Every step passes, before its figures are read:

```
cargo build
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
RUSTFLAGS="-D warnings" cargo build
```

plus the CI formatting configuration, and a full build after which the
committed generated bindings under `examples/` are byte-for-byte unchanged.

## Figures so far

Measured on `umbrella/registry-conveyor` after #680, by the same script and the
same `grep`. **Interim, not closing**: #676 is open and its checklist still has
steps 4, 5, 6's second half and 7 unchecked. The two figures are gates on those
deliverables, not a substitute for them, and re-scoping the umbrella is the goal
owner's call rather than this document's (#681 review).

| figure | baseline (`eb4aa007`) | after #680 | change |
|---|---|---|---|
| 1 — adapter production lines in registry-facing files | 28,348 | **28,140** | −208 |
| 2 — production lines across the three crates | 57,844 | **57,756** | −88 |

Per crate:

| crate | production | test items | test files | total |
|---|---|---|---|---|
| `prebindgen-registry` | 15,153 | 724 | 7,708 | 23,585 |
| `prebindgen-c` | 8,658 | 81 | 5,826 | 14,565 |
| `prebindgen-jni` | 33,945 | 1,582 | 21,826 | 57,353 |

Both figures are below the baseline. The registry grew and both adapters
shrank, which is the direction the criterion asks for; whether the umbrella has
met it is decided when its steps are done, not here.

## What has landed, and what each remaining step is blocked on

Four children so far:

- **#677** — `RegistryBuilder::generate` walks the crossings and drives every
  `Compile` hook through one `Compiler` the registry holds. `convert_with`,
  `Answer`, `Compiler::recipe_of` and both adapters' crossing walks are gone.
- **#678** — the registry composes every `ShapePlan` and hands it to the
  fragment. `Representation` loses `Bridge`, `TerminalCodec` and `Step`, and
  both adapters' shape helpers go.
- **#679** — `Intermediate` and `Niche`, which both adapters spelled the same
  way, become registry types.
- **#680** — the leaf model and the walk over it become
  `prebindgen_registry::leaf`. Which **delivery** those leaves take stays with
  the adapter, because it follows from what the target can receive.

Steps 4, 5, 6's second half and 7 have not landed. Each was built or priced
against the committed tree. The first two rest on something #675 assumes about
this code that the code does not bear out. The last two come from **prototypes**
— branches that build the step, named at the end of this section, and part of no
merged tree — and record what building it showed:

1. **The registry cannot build a `Compile::Fragment`.** It is the adapter's
   type and the registry never looks inside one. Step 4's "the registry wraps
   the bridge into the fragment" therefore needs a second hook per shape, and
   step 3's `GenerationPlan<C: Compile>` needs the adapter's `Compile` type to
   be lifetime-free, which costs more than deleting `Representation` saves.
2. **The declaration applier runs before the recipe table exists.**
   `declare_into` runs it, and `recipes()` is built afterwards *from the plans it
   produces*. So step 6's "readers of `Product` recipes" has nothing to read;
   what is duplicated is the derivation, and one shared derivation fixes it.
3. **Step 7 is built as far as it goes, and it costs 185 lines on figure 1.**
   Every difference between what the two adapters
   call a site is addressed. Three are about which positions each adapter plans,
   and a hook that asks the adapter settles those: `prebindgen-c` skips a `()`
   return because C has nothing to hand back there while `prebindgen-jni` plans
   one; JniGen answers a callback parameter whole; JniGen's synthesized const
   getters are not `#[prebindgen]` functions, so they sit in no export set and a
   walk over the exports never reaches them.

   The fourth is about what one site *crosses*. The walk enumerates **one**
   return site, crossing the return as the model states it, so a fallible return
   crosses whole — which is what #675's own definitions give, a site being a
   source position and a crossing that Rust type with a direction. Where a
   binding crosses something else there, it says so rather than declining:
   `Bindings` is where a binding states what crosses at a site.

   JniGen states all three of its cases that way, in **one pass per function**
   and in the precedence `build_output` applies. An **expanded** return — one the
   binding converts or takes apart — does not cross the value its signature
   names, and only the binding's own output expansion says what it does cross: a
   convert crosses the converted value; a return handed to a callback crosses the
   value it takes apart, and asks for the row naming its parts; a fallible return
   whose error the binding peels crosses only its ok arm, since the error is
   thrown rather than crossed there. A fallible return with no error plan still
   crosses whole, which is what the model already says.

   One pass rather than one per kind, because two would answer the same site
   twice and be refused as a rebind (#684 review). Whether a function can hold
   both an output expansion and an error plan is unsettled: `peel` does not peel
   `Result`, so a fallible return takes no output expansion, and an assertion
   that no function holds both held across every fixture and every test. The
   single pass is the right shape either way.

   With the right crossing in hand a return's plan is complete in the hook: the
   delivery a convert converts through is built there, `fn_plan.rs` reads that
   plan instead of rebuilding it, and `ValueOutputPlan::site` — which existed
   only to carry the site through that rebuild — is deleted.
   `JCompile::plans_site` is left with one rule, the callback parameter it
   answers whole, and `return_site` compiles a site for no exported function; an
   assertion to that effect held across every fixture and every test. What still
   reaches its private fallback is the one thing with no site to reach: a getter
   synthesised for a declared constant, which is not a `#[prebindgen]` function.

   `prebindgen-c` states its own case the same way. A fallible return's **error
   arm is a position in the model**, not a channel a target invented — a `Result`
   has two arms whatever reads it — so the walk enumerates it, and whether a
   binding crosses anything there is that binding's answer, given through
   `plans_site`: JniGen throws the error, so the JVM receives no value at that
   position and it declines; `prebindgen-c` hands the error back and plans it.
   Declining rather than refusing is the distinction that matters — a refusal is
   a diagnostic about something that should have compiled, and `build_with`
   drops refusals, so a position left to refuse would look handled while nothing
   had decided anything (#685 review). What C
   crosses at the **return** is then the ok arm alone, which is a fact about the
   C binding, so it is bound rather than re-derived. `CCompile::plans_site` is
   back to one rule, the `()` return C has nothing to hand back at, and
   `Cbindgen::fallible_return_plans` is deleted.

   **One adapter-private planner remains**, and it closes with finding 4 rather
   than on its own. A callback-delivered expansion whose source has no `parts`
   row is planned on its default row, so the walk's plan carries no wires;
   `build_output` falls back to `freeze_out_wires` and compiles the leaves
   itself. Confirmed rather than reasoned: a probe that panicked on that fallback
   fired for `annotated_new` in `examples/perftest-flat` (#684 review).
   `value_form_of` declines exactly those decompositions, so there is no `parts`
   row for the site to ask for. Where a row does exist, the emitter reads the
   site's own list, and `a_decomposed_return_shares_one_wire_list_with_its_site`
   asserts the two are the same allocation.

   So every site of every exported function is enumerated and compiled by the
   registry, bar that one, and what each adapter still decides it decides through
   a hook or a binding rather than by walking the functions itself.

   The branch is green: every gate passes with the generated fixtures
   byte-identical. It costs 185 lines on figure 1 (28,325) and 409 on figure 2
   (58,165), measured against this umbrella's head the same way.

   **The artifact half is already done, and step 7's last deletion is not
   available.** Of the four deletions step 7 names, `prebindgen-c`'s
   `compile_sites` and `fallible_return_plans` are gone.
   `JniGenerationPlan::freeze` was the candidate for a third — 328 lines,
   against 185 that have to come out for the step to pay — but it is not an
   artifact assembly waiting to be moved. It already states every artifact to
   the registry's `GenerationPlanBuilder` as an `ArtifactPlan`, with the
   fragments whose converters it calls as `ArtifactInput::Fragment` and a
   private converter's fragments as `follows`, and it derives the `Assembly`
   from `plan.artifacts()` rather than beside it. That is step 7's "every
   artifact kind becomes an `ArtifactPlan` that `follows` its fragments", and
   #660 did it.

   What is left in `freeze` is not artifact assembly: it is the memo hand-off —
   `fn_plans`, `iface_specs`, `struct_plans`, `sum_plans` and `vec_build_plans`
   moved out of the mutable planning store — the pre-population that has to
   happen before that store is drained, and the operation-to-fragment map the
   plan's pruning needs. None of it is duplicated in the registry, so deleting
   it would move it rather than remove it. This is the same shape as step 6's
   `struct_plan.rs`, which #620 had already deleted before step 6 asked for it.

   So the 185 lines are not duplication awaiting removal. They are what the
   mechanism costs: the hooks and the `Bindings` entries that let each adapter
   state its own answers instead of walking the functions itself. Whether that
   is worth 185 lines on figure 1 is a judgement about the architecture rather
   than a measurement still to be taken, and it is the goal owner's to make.
   **Step 7 adds to both sides rather than moving between them.** Measured
   per crate against this umbrella's head:

   | crate | umbrella | step 7 | delta |
   |---|---|---|---|
   | `prebindgen-registry` | 15,153 | 15,376 | +223 |
   | `prebindgen-c` | 8,658 | 8,699 | +41 |
   | `prebindgen-jni` | 33,945 | 34,090 | +145 |
   | **total (figure 2)** | **57,756** | **58,165** | **+409** |

   The adapters grew by 186 between them. So this is not a move whose boundary
   cost is the problem — the thing step 7 exists to shrink got bigger. A hook the
   registry calls and a `Bindings` entry the adapter writes are both adapter
   code, and they cost more than the walks they replaced.

4. **Step 6b's prototype shares the derivation, and leaves the migration
   around it unfinished.** Both are derived from one declaration's
   records: `Declarations::value_form_of` builds the recipe's list of `Reach`es,
   and the declaration applier flattens the same records into leaves. Reading
   the leaves instead of re-walking the records agrees wherever both answer,
   with the generated files byte-identical.

   The two part on a value form whose records state their own decomposition —
   the record walk declines such a value form, the flattening handles it — which
   is step 6's "`recipes.rs` stops declining". Declaring those decompositions
   moves the `impl Fn(Probe)` and `impl Fn(Report)` callback arguments off the
   path that was converting them, and the build stops at resolution:
   `Unresolved { key: TypeKey("impl Fn (Probe) …"), direction: Construct }`.
   Resolution stops before anything is generated, so there is no second set of
   files to compare and this says nothing about whether the output would change.
   It is the work step 6b has to do: the shared derivation has to keep, or
   rebuild, the construct conversion those two crossings were selecting.

A fifth assumption — that the registry cannot be told which recipe a site takes
— looks wrong: a hook that asks the adapter works in the prototype. Whether that
hook is the right seam is a question for review, which the prototype has not
had.

Neither prototype is a candidate to merge. Step 7's is green and complete as far
as the step goes, and costs 185 lines on figure 1, which a child of #676 may not
do; step 6b's stops at an unresolved converter. What each entry above says is
what a successor needs.

Findings 3 and 4 were reached by building them, on the branches
`feat/676-step7-registry-owns-the-plan` and `feat/676-step6b-one-derivation`.
Both are **prototypes, not part of this tree**: neither has had review, and a
branch can change or go away. They are recorded here as the directions a
successor should try first; this document otherwise describes the committed tree
(#681 review).

## What the figures settle, and what they leave open

**Two criteria are in play, and which one governs has not been decided.**

- **#675 as written**: figure 2 is bounded on *every step*, and figure 1 is
  judged over the *completed plan* against the 28,348 baseline.
- **#676's amendment**: figure 1 must not rise in any child, and figure 2 is
  bounded once, at the umbrella's close, below 57,844.

The amendment was written in this umbrella after four measurements, and **no goal
owner has approved it** — #681's review said so, and #676's own description still
says the decision has not been made. So the arithmetic below is given under both.

### What is measured

| step | figure 1 | figure 2 | what the number covers |
|---|---|---|---|
| 7, as prototyped | 28,325 (+185) | **+409** | the prototype that exists, built and green — **not** the whole step: one exported return still plans its leaves privately |
| 6b | falls | ~+10 | the shared derivation only; the migration that would finish it is unpriced |
| 4 | falls | ~+40 | the `Optional` shape only, not the rest of the step |

Every one of these is a floor for work that is not finished. Step 7's is the
firmest, because the thing it measures compiles and passes every gate — but
#675's step 7 is "the registry compiles **every** site", and this prototype
compiles all but one.

### What is settled

**The step 7 prototype cannot land as a child under either criterion**, because
figure 2 rises by 409: #675 bounds figure 2 on every step, and the amendment
leaves 88 lines of headroom at the close. Under #675 its figure 1 of 28,325 would
have passed the 28,348 baseline; under the amendment it would not. That much the
figures decide on their own.

Why it costs what it does, per crate against this umbrella's head:

| crate | umbrella | prototype | delta |
|---|---|---|---|
| `prebindgen-registry` | 15,153 | 15,376 | +223 |
| `prebindgen-c` | 8,658 | 8,699 | +41 |
| `prebindgen-jni` | 33,945 | 34,090 | +145 |
| **total** | **57,756** | **58,165** | **+409** |

The adapters grew by 186 between them, so this is not a move whose boundary cost
is the problem — the thing step 7 exists to shrink got bigger. The plan assumed
each move's boundary cost would be paid out of large deletions, and those had
already happened: #513 took the Rust side, #620 took `struct_plan.rs`, #660 made
the generation plan the assembly.

### What is not settled

**Whether a *completed* step 7 could come in under budget is unmeasured.** The
completion — removing the `freeze_out_wires` fallback, which needs step 6b's
migration — is unbuilt, and it can delete as well as add: the fallback and the
duplicated derivation behind it are both code it would remove. So the figures
prove this prototype cannot be the step 7 child; they do not prove that no
completion or regrouping lets #676 close on its original terms.

## The answers available

1. **Build the completion and measure it.** The only path that might let #676
   close on #675's terms. It requires step 6b's migration, which is unpriced
   beyond the ~10 lines its shared derivation costs, and which finding 4 records
   as blocked on the construct conversion those crossings were selecting.
2. **Accept the prototype's cost as a re-scoped step 7.** Relax whichever
   criterion governs, and waive the one exported return that still plans its
   leaves privately — explicitly, as a known exception, since #675's step 7 says
   every site. Then +409 is the price of the re-scoped step rather than evidence
   about the original one.
3. **Close on what landed.** Figure 1 is 28,140 against a baseline of 28,348 and
   figure 2 is 57,756 against 57,844. **Under #675 this closes cleanly**: figure
   1 is judged over the completed plan and passes, and every step lowered figure
   2 or held it. **Under the amendment it needs one waiver**: the per-child rule
   says no child may raise figure 1, and #680 took it from 28,138 to 28,140.
   Being 208 below the baseline at the end does not repair a child that rose by
   2, so closing this way under the amendment means treating #680 as a known
   exception — it returned the leaf model to the registry and deleted nothing,
   which is what a move looks like when the deletion has already happened
   elsewhere.

Nine children have merged under any of the three, and the vocabulary, the shared
crossing walk, the registry's `ShapePlan`, the neutral leaf model and the
findings above are all in the tree.

This document chooses none of the three, and does not choose which criterion
governs. It records what the figures decide and what they leave to be decided.
