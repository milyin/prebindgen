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

A data class is decomposed twice, by `struct_out_wires_at` and by `StructPlan`,
with different coverage — the registry-facing one refuses a nested data class
behind `Option` or `Vec`. #602 carries the missing support and #603 pinned the
disagreement with a standing check; step 3 removes the second derivation.

## Size baseline

Rust source lines under each crate's `src/`, from
`python3 examples/line-report.py`. Three numbers, and every line is in exactly
one of them: **production**, **test items** (`#[cfg(test)]` items inside
production files), and **test files** (anything under a `tests/` directory or
named `tests.rs` / `test_util.rs`).

| crate | #513 base `cecce967` | #613 base `dea7a55` | production delta |
|---|---|---|---|
| `prebindgen-c` | 7148 | 8755 | +1607 |
| `prebindgen-jni` | 29120 | 30585 | +1465 |
| `prebindgen-registry` | 12417 | 16647 | +4230 |
| `prebindgen-flat` | 5157 | 5271 | +114 |

#613's size gate is that the first two decrease from the `dea7a55` column.

These numbers are smaller than the ones quoted in #613's own description
(C 7,240 → 8,931, JNI 30,228 → 32,115), which were counted before this script
existed and by a different rule. The script is the definition from here on: it
counts `#[cfg(test)]` items at any indentation, so a production function moved
into an inline test module shows up as a production deletion **and** a test-item
addition rather than as a deletion alone.

An item's end is found by tracking delimiter depth — the `}` closing the brace
body it opened, or a `;` when it opened none — over lines whose comments and
literals have been blanked out by a lexer that carries its state across lines.
Both halves were needed: a gated multiline call ending in `);` ends there
rather than at the next closing brace, and a `}` inside a multiline raw string
or a nested block comment is text rather than the item's end.
`python3 examples/line-report.py --self-test` pins that on a fixture built from
the call at `prebindgen-jni/src/jni/mod.rs:909`. That item is nine lines; the
earlier indentation-based rule ran it to line 998 and counted the 81 production
lines in between as test support, understating the JNI baseline by that much
(#614 review). The raw-string and block-comment cases are pinned there too,
each from the review probe that found it.

Only a bare `#[cfg(test)]` counts as a test item: an item behind
`#[cfg(any(test, feature = "testing"))]` ships to other crates under that
feature and is production by this rule.
