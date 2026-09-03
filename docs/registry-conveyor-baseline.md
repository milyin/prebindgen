# Baseline for #675

The figures the plan in #675 compares against, recorded once because the
`tools/line-report` script bakes in its workspace path at build time and cannot
be pointed at an older commit without being rebuilt inside a worktree of it.

Measured on `main` at `eb4aa007`, the #513 merge.

## Production lines

The `production` column of `tools/line-report`, which counts the lines left
after both kinds of test code are removed: every **test file** — one under a
`tests/` directory, or named `tests.rs` or `test_util.rs` — and every
`#[cfg(test)]` item or statement inside the files that remain.

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
grep -rlE 'prebindgen_registry::(recipe|generation|chain|write)\b|use prebindgen_registry::\{[^}]*\b(recipe|generation|chain|write)\b' --include='*.rs' src \
  | grep -vE '/tests/|/tests\.rs$|/test_util\.rs$'
```

The three exclusions match what `tools/line-report` itself removes, so the set
this selects and the lines it sums are chosen by one rule.

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

## Closing figures

The umbrella closed after a final review reopened it. Both figures, measured
against the `eb4aa007` baselines by the method above:

| | baseline | close | |
|---|---|---|---|
| figure 1 — adapter production lines in registry-facing files | 28,348 | **28,576** | +228 |
| figure 2 — production lines across the three crates | 57,844 | **58,589** | +745 |

**Both figures are indicators, and neither gates anything.** Figure 1 was the
gate until the final review, and holding it as one had the effect the umbrella's
first rule had: it made the umbrella's own remaining work disqualifying. The
child that put every C artifact in the generation plan — which is what #675's
step 7 asks for — raises figure 1, because stating twelve artifact kinds with
their identities, resolving their prerequisites and declaring their retention is
larger than the hand-written list it replaces. A number that answers that
question wrongly should not decide it.

So what the umbrella bought is not fewer adapter lines. It is that the decisions
an adapter used to make by writing a walk are now made by stating something:

- **one crossing walk and one site walk**, both the registry's, where each
  adapter had its own;
- an adapter that wants a position planned differently says so through
  `Compile::plans_site`, `Compile::site_recipe`, or a `Bindings` entry, rather
  than enumerating the functions itself;
- **one artifact type per adapter**, and the generated file's layout is the order
  its artifacts were stated in the plan, not a second list placed beside it;
- a runtime helper is emitted because a kept artifact requires it, which the plan
  decides, rather than because a scan over `calls()` said so.

A third adapter inherits all of that, and inherits no walk to copy.

## What landed

The children, by what each did rather than by how many there were — a count
drifts every time one lands, and this document has already corrected one:

- **#677 + #678** — `RegistryBuilder::generate` walks the crossings and drives
  every `Compile` hook through one `Compiler` the registry holds, and the
  registry composes every `ShapePlan`. `convert_with`, `Answer`,
  `Compiler::recipe_of`, both adapters' crossing walks, `CFrag::shape` and the
  shape helpers are gone.
- **#679** — `Intermediate` and `Niche`, which both adapters spelled the same
  way, become registry types.
- **#680** — the leaf model and the walk over it become
  `prebindgen_registry::leaf`. Which **delivery** those leaves take stays with
  the adapter, because it follows from what the target can receive.
- **#681 – #686** — the record of what building the remaining steps showed. Their
  findings are below.
- **#687** — the registry compiles every site. `Registry::compile_sites` walks
  the exported functions in one fixed order; `Cbindgen::compile_sites` and
  `Cbindgen::fallible_return_plans` are deleted, and `fn_plan.rs` reads the plan
  the walk produced. `Compile::plans_site` lets an adapter decline a position
  without pruning the positions inside it, `Compile::site_recipe` lets the
  registry ask which recipe a site takes, and `Bindings` is where a binding says
  it crosses something other than what the model states — a converted return, a
  decomposed one, a peeled error arm, C's ok arm.
- **#690 – #696** — the final review's findings. JniGen stopped discarding the
  site refusals the registry reports (#690); `prebindgen-c` went from two artifact
  types and a lookup handle to one (#691) and then stated every artifact of its
  file in the generation plan, runtime helpers included, so the assembly is
  `plan.artifacts()` and the hand-written placement and the `calls()` scan are
  gone (#696); a kept artifact keeps what it requires (#692); one `SiteWalk`
  replaced four copies of the same five lines (#693); the documents describing a
  tree the registry has moved past say so (#694); and a constant's getter is a
  model function of this binding, so the walk reaches its return and
  `return_site` is a lookup and nothing else (#695).

## The waivers

Two positions are still planned by their adapter, and both are named rather than
quietly left:

**A no-`parts` callback-delivered return.** A callback-delivered expansion whose
source has no `parts` row is planned on its default row, so the walk's plan
carries no wires and `build_output` falls back to `freeze_out_wires`.
`annotated_new` in `examples/perftest-flat` is one. It closes with step 6b, which
is deferred: `value_form_of` declines exactly those decompositions, so there is
no `parts` row for the site to ask for.

**JniGen's declared-decomposition loop.** `build_with` compiles a `parts` row for
a `sealed_class` deconstruct crossing, because a sum hands out a tag and its
groups — which is not a value — so it has no whole-value crossing for the derived
order to offer. The final review asked for this to move into the walk, by letting
a marked row add its crossing. Built, that widens the walk to construct crossings
nothing else compiles and emits an unreferenced converter into
`covertest-kotlin`. The loop's guard is why: it skips a type unless every part of
it already crosses, which is a question about what has been compiled and can only
be asked after the walk has run. Closing it needs either pruning what the widened
walk compiles or a row that is compiled once its parts have crossed — a model
change, recorded here for the successor.

Every other site of every exported function is enumerated and compiled by the
registry.

## What a successor inherits

Steps 3b, 4, 5 and 6b are deferred to a successor issue. Each was built or
priced, and each rests on something #675 assumes that this code does not bear
out:

1. **The registry cannot build a `Compile::Fragment`.** It is the adapter's type
   and the registry never looks inside one. Step 4's "the registry wraps the
   bridge into the fragment" therefore needs a second hook per shape, and step
   3b's `GenerationPlan<C: Compile>` needs the adapter's `Compile` type to be
   lifetime-free, which costs more than deleting `Representation` saves.
2. **The declaration applier runs before the recipe table exists.**
   `declare_into` runs it, and `recipes()` is built afterwards *from the plans it
   produces*. So step 6's "readers of `Product` recipes" has nothing to read;
   what is duplicated is the derivation.

Two more things the plan assumed were already done before this umbrella opened,
which is why the deletions it budgeted for were not there to make.

**#620 had already deleted the duplicate derivation** — `StructPlan`'s conversion
half, the second data-class leaf derivation, and its old readers. It did not
delete `prebindgen-jni/src/jni/struct_plan.rs`, which is still 640 production
lines: what remains is a data class's own Kotlin **property** declarations, one
per field, which is a different question from one leaf per slot and has no
duplicate to remove. The distinction is the diagnosis a successor needs — the
duplication step 6 was written to delete was already gone, and what is left there
is not duplication.

**#660 had already made `JniGenerationPlan::freeze`** state every artifact to the
registry as an `ArtifactPlan` and derive the assembly from the plan, so step 7's
fourth named deletion was not available either.

Step 6b was built as far as an unresolved converter: reading the applier's
already-flattened leaves instead of re-walking the records agrees wherever both
answer, with the fixtures byte-identical, but declaring the decompositions the
record walk declined leaves the `impl Fn(Probe)` and `impl Fn(Report)` callback
arguments with no construct converter. The shared derivation has to keep or
rebuild that conversion. The prototype is on
`feat/676-step6b-one-derivation`; a branch can change or go away.

## Superseded: the figures while the umbrella was open

Kept because the measurements cost something to take, not because they still
describe the tree. Every conclusion drawn from them at the time — that the
criterion was undecided, that step 7 could not land, that a choice remained open
— was answered when the goal owner amended the criterion on 2026-09-02 and
accepted step 7's cost with the waiver above.

| point | figure 1 | figure 2 |
|---|---|---|
| baseline `eb4aa007` | 28,348 | 57,844 |
| after #677 + #678 | 28,142 | 57,764 |
| after #679 | 28,138 | 57,759 |
| after #680 | 28,140 | 57,756 |
| step 7 prototype, mid-review | 28,325 | 58,165 |
| step 7 merged (#687) | 28,346 | 58,272 |
| close, after the final review | **28,576** | **58,589** |

The shape those numbers showed, and the reason the criterion was amended: every
remaining step moves complexity from an adapter into the registry, figure 1
counts adapter code and figure 2 counts all three crates, so a move lowers the
first and raises the second by whatever the boundary costs. The plan expected
that boundary cost to be paid for out of large deletions, and those had already
happened.
