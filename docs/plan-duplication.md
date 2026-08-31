# What the generation plans say twice

The baseline for #613, which converges Cbindgen and JniGen on one frozen
generation-plan model. This records what is duplicated **before** any of that
work, so a later reader can tell what was deleted from what was moved.

Measured at `dea7a55` — #513's head, and #613's base.

## Vocabulary

- **Crossing** — one value moving across the boundary in one direction, e.g. a
  `Sample` returned to Kotlin.
- **Fragment** — the compiler's answer for one crossing: what it converts
  through, and what it composed from. Identified by a `FragmentId`.
- **Site** — one place a fragment is used: a parameter, a return, a callback
  argument. Identified by a `SiteId`.
- **Shape** — the structural form of a crossing: atomic, product, optional,
  sequence, choice, or invoke. The registry's word for "what kind of walk this
  needs".
- **Representation** (`R`) — the adapter's half of a plan, supplied as
  associated types on the `Representation` trait: intermediate type, terminal
  codec, and one bridge per shape.
- **Canonical plan** — `GenerationPlan<R>` and its `FragmentPlan<R>` /
  `SitePlan<R>` / `ArtifactPlan<R>` members, in `prebindgen-registry`.

## The same question, asked twice

### Shape

Eight enums name three or more of the six structural forms. Each is a place the
same structural question is asked again.

| enum | crate | what it is |
|---|---|---|
| `Shape` | registry `recipe.rs` | what the recipe compiler selected |
| `DerivedKind` | registry `recipe.rs` | how a derived recipe was obtained |
| `ShapePlan<R>` | registry `generation.rs` | the canonical frozen shape, with the adapter's bridge in each variant |
| `ComposedShape` | registry `generation.rs` | the composed-fragment subset |
| `CShape` | `prebindgen-c/src/compile.rs` | C's copy of the frozen shape, without bridges |
| `CBody` | `prebindgen-c/src/chain.rs` | the rendering half of the same split |
| `JLayout` | `prebindgen-jni/src/jni/compile.rs` | the shape of JNI's single intermediate over flattened ABI leaves |
| `JBody` | `prebindgen-jni/src/jni/chain.rs` | the rendering half of the same split |

`CShape` is the clearest case: `CFrag::freeze` translates it into `ShapePlan`
variant for variant, so the C compiler states the shape twice and neither
statement is derived from the other. Two fences pin the adapter half of this
census — `the_c_adapter_gains_no_third_shape_vocabulary` and
`jnigen_gains_no_third_shape_vocabulary` — so the list can shrink as #613
proceeds but cannot grow unnoticed.

### Fragment

| field | where | the canonical answer |
|---|---|---|
| `CFrag::id`, `JFrag` via `conv.converter` | adapters | `FragmentPlan::id` |
| `CFrag::source` | C | `FragmentPlan::source` |
| `CFrag::destination`, `ConverterImpl::destination` | C, shared | `FragmentPlan::intermediate` |
| `CFrag::yields`, `JFrag::yields` | both | `FragmentPlan::yields` |
| `CFrag::niches`, `ConverterImpl::niches` | C, shared | the converter plan's niche contract |
| `CFrag::subs`, `ConverterImpl::subs` | C, shared | `FragmentUse` edges inside `ShapePlan` |
| `CFrag::shape` | C | `FragmentPlan`'s `ShapePlan` |
| `CFrag::function`, `CFrag::value` | C | `FragmentPlan::artifact` plus the shape's bridges |
| `JFrag::layout` | JNI | `ShapePlan` — the same nesting, over ABI leaves |
| `JFrag::rust`, `JFrag::rust_stages` | JNI | `FragmentPlan::artifact` and the conversion chain |
| `JFrag::choice_arm` | JNI | `ShapePlan::Choice`'s arms |
| `JFrag::composed_only` | JNI | a fragment with no artifact |

`JFrag` also carries `wires` / `out_wires`, the several ABI values one crossing
occupies. The canonical plan has no multi-slot layout yet; #613 step 4 names
that as a missing general contract rather than a JNI concept.

### Decomposition structure

Two mechanisms state how a value comes apart, and only one of them is a recipe.

| | where | what it states |
|---|---|---|
| `Deconstruct` / `Reach` | registry `recipe.rs` | a product's parts as one-hop reaches; nesting is a part with its own recipe |
| `DeconRecord` / `UnfoldPlan` | registry `unfold.rs` | the same decomposition, flattened: `Acc` splices a child's records, `FieldRecord.members` chains, `Identity` names the value itself |

`unfold` predates the recipe table by two and a half months (`dacbd3ee`,
2026-06-08, against #450/#451's `cfd020e8`, 2026-08-20) and does three further
jobs no recipe does: lowering `expand_return!` declarations, registering each
leaf's `out_ty` so the resolver emits its converter, and generating the reach
code (`walk.rs`). Only the middle job is duplicated.

The bridge between the two is `prebindgen-jni`'s `value_form_of`, which mirrors
a declaration into a `Deconstruct::ValueForm` row and refuses three cases
outright — a nested override, a multi-hop member chain, a self-reach. What it
refuses has no row, so no site can name it.

**This row is what the census above missed.** Step 1 surveyed adapter carriers;
`unfold` sits in the registry, reads as language-neutral, and so never came up
as a second answer — although `prebindgen-c` uses it zero times in production
and 16 JNI files read `plan.leaves`. #622 names the seam rather than closing
it: a crossing a callback delivers by taking it apart now states a `parts` row,
so the argument's site has a row to name, and that row is `Deconstructing::Atomic`
for the reason the declared-type loop already gives — the adapter emits the
conversion itself. The structure is still stated once in `DeconRecord` and
pointed at from the table. Step 10 is the deletion.

### Stage order

`ConverterImpl::pre_stages` and `ConversionChain` both order the conversion
stages of one crossing. `pre_stages` is documented as running
rust-side-first for input and reversed for output; the chain states the same
order as an operation graph. Step 2 moves the remaining `pre_stages` decisions
and deletes `Stage`.

### Site

`JPlan` (`Param` / `Return` / `Decomposed`) is JniGen's per-site answer, held
beside the registry's `SitePlan<R>`, which JniGen does not use. `JPlan::Param`
boxes a `PlanLeaf` because the variants differ in size by hundreds of bytes —
which is itself a signal that the site vocabulary is carrying representation
detail the canonical plan would keep in `R`.

### The frozen whole

`JniGenerationPlan` holds `Compiled<JFrag>` plus the foreign-artifact sidecars
(functions, interfaces, structs, sums, vec builds) and an `Assembly`. The first
of those is what `GenerationPlan<R>` is for; the rest is the sidecar #613 keeps.

### Data-class decomposition

A data class is decomposed twice, by `struct_out_wires_at` and by `StructPlan`.

They had different coverage when this baseline was taken — the registry-facing
one refused a sum field, a nested data class behind an `Option`, a handle and an
`enum_class` field — which is why #602 carried the missing support and #603
pinned the agreement, where both applied, with a standing check.

That coverage gap is closed. #616 taught the walk a sum field, #617 an optional
nested class behind its presence flag, and #618 one selector inside another
(which also unrefused handle and `enum_class` fields, and with them `Option<sum>`
and a gated class that selects of its own). What is left to decline is
structural: a repeated nested class under a `Vec`, a field the model does not
hold as a named one, nesting past 16 — none of which `StructPlan` serves either.

**Resolved.** With the coverage gap closed the two walks duplicated one leaf
derivation, and #620 deleted the second: both the Rust whole-object encode and
the Kotlin `fromParts` factory now render from the decomposition, so
`assert_leaf_derivations_agree` — which existed to say they agreed — has nothing
left to compare and is gone with them. `StructPlan` remains, reduced to a data
class's own Kotlin **property** declarations, which is a different question from
the leaf list.

This row is kept rather than removed because the table above it is the #613
baseline: what it records is what was true at `dea7a55`, and this paragraph is
what became of it.

## Size baseline

Rust source lines under each crate's `src/`, from

```
cargo run --manifest-path tools/line-report/Cargo.toml
```

Three numbers, and every line is in exactly one of them: **production**, **test
items** (`#[cfg(test)]` items and statements inside production files), and
**test files** (anything under a `tests/` directory or named `tests.rs` /
`test_util.rs`).

| crate | #513 base `cecce967` | #613 base `dea7a55` | production delta |
|---|---|---|---|
| `prebindgen-c` | 7148 | 8754 | +1606 |
| `prebindgen-jni` | 29103 | 30534 | +1431 |
| `prebindgen-registry` | 12409 | 16639 | +4230 |
| `prebindgen-flat` | 5157 | 5270 | +113 |

#613's size gate is that the first two decrease from the `dea7a55` column.

Step 3's four children (#616, #617, #618, #620) leave `prebindgen-jni` at
**30,408** — below its baseline, so that half of the gate is met. `prebindgen-c`
is untouched so far; step 6 is where it moves.

These numbers are smaller than the ones quoted in #613's own description
(C 7,240 → 8,931, JNI 30,228 → 32,115), which were counted before this script
existed and by a different rule. The script is the definition from here on: it
counts `#[cfg(test)]` items at any indentation, so a production function moved
into an inline test module shows up as a production deletion **and** a test-item
addition rather than as a deletion alone.

**The boundary is syntactic.** `syn` parses each file and the walk asks every
`#[cfg(test)]`-attributed node — item, impl item, statement, field, variant —
for its own span. Nothing matches braces, indentation or line shapes.

Three rounds of #614 review each found another valid construct a delimiter
heuristic could not delimit, which is why the question is put to a parser: a
gated multiline call ending in `);` (the one at
`prebindgen-jni/src/jni/mod.rs:909`, which an indentation rule ran to line 998,
counting 81 production lines as test support); a `}` inside a multiline raw
string or a nested block comment, which a line-local lexer read as a delimiter;
and a gated `let x = if c { .. } else { .. };` or a chained `Thing { .. }
.method();`, whose first brace pair closes in the middle of the statement.

`--self-test` builds a fixture from all five and asserts the line split:

```
cargo run --manifest-path tools/line-report/Cargo.toml -- --self-test
```

The tool is deliberately outside the workspace: source locations on spans are a
Cargo feature of `proc-macro2`, features are additive, and a `Span` that stores
its location makes `syn::Type` large enough to trip `clippy::large_enum_variant`
across the C adapter in any `--all-features` build. Its `Cargo.lock` is
committed — an un-ignore in `.gitignore` — because the numbers depend on the
exact `syn` and `proc-macro2` that parse them, and CI runs its tests so the
pinned cases cannot rot unnoticed.

**A gate it cannot bound fails rather than guesses.** The walk names the node
kinds it understands; a separate pass sees every attribute in the file, whatever
owns it. A `#[cfg(test)]` that was met but not attributed — on a function
parameter, say — stops the report with the file and line, because counting what
it gates as production would understate the test lines silently.
