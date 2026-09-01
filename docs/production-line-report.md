# Production-line report for #613

What the umbrella cost, measured rather than estimated. `tools/line-report`
counts production lines by asking `syn` where each `#[cfg(test)]` construct
ends, so test code is excluded from every figure here.

## Before and after

| crate | #613's base | after #658 | delta |
|---|---|---|---|
| `prebindgen-c` | 8,754 | 8,757 | **+3** |
| `prebindgen-jni` | 30,534 | 31,698 | **+1,164** |
| `prebindgen-registry` | — | 17,050 | — |

**The umbrella has not yet reduced the adapters.** That is the headline, and it
is the opposite of what steps 4 and 5 were meant to deliver together.

## Why, precisely

The additions landed and the deletion they were for did not.

- Steps 4, 5a and 5b **added carriers** so the canonical plan could answer what
  `JFrag` answered: the frozen plan built in production, the readers moved onto
  it, `JConverterArtifact` grown to hold wire, layout, metadata, niches,
  converter identity and stages.
- Step **5c is where those come back** — `JFrag`, `JPlan`, `Conv` and the
  Rust-graph half of `JLayout` — and it is blocked. Every reader that can move
  has moved (#626, #639, #640); what remains needs the compiler to produce
  `FragmentPlan` directly, because two of `Conv`'s four remaining fragment
  reads are composition, which happens before any plan exists.

So the ledger is honest about direction: this is a carrier-first refactor whose
payment is still outstanding, not a reduction that failed to materialise.

## What was actually deleted

| deleted | where | PR |
|---|---|---|
| `CShape` and its 24-line `freeze` translation | `prebindgen-c` | #627 |
| `CFunction::operation` (duplicate of `call.operation_id()`) | `prebindgen-c` | #633 |
| `Cx::table`, `Cx::recipe_names`, `Cx::recipes` | registry | #628 |
| `Compiled::record` (no production callers) | registry | #626 |
| two stale `allow(dead_code)` markers | `prebindgen-jni` | #628 |
| `asks_parts` route into the callback plan | `prebindgen-jni` | #657 |
| ~11,300 lines of uncalled converters, then 103 | generated output | #632 |

The last row is the largest single reduction in the umbrella and it is in
**generated** output, not the generator: once the JNI assembly was ordered by
the canonical plan, the converters nothing called stopped being emitted.

## Deleted rather than merely moved

Step 9 asks which abstractions were genuinely removed. Of the eight
shape-shaped enums step 1's census recorded:

- **`CShape` is gone** — a `CFrag` carries `ShapePlan<CRepresentation>` directly.
- **`CBody` is not a duplicate** and stays (#641). Its variants mirror
  `ShapePlan`'s, but `ProductField` carries the binding name, mode and
  uninit-holding that `FragmentUse` does not. Both routes out were tried:
  `ProductBridge = ProductPlan` is not total (a marker fragment carries a
  Product shape with no `ProductPlan`), and "move it onto the converter
  artifact" is already the case, since `ConverterArtifact` **is** `CFunction`.
- `Shape`, `DerivedKind`, `ShapePlan`, `ComposedShape` are the registry's own
  vocabulary, not adapter copies.
- `JLayout` and `JBody` remain, pending 5c.

So the census goes from eight to seven, and the remaining adapter entry is
documented as not-a-duplicate rather than left looking like unfinished work.

## Language-policy modules that remain

Named as step 9 asks, so the next reader does not have to find them:

- `prebindgen-jni/src/jni/recipes.rs` — declares the JNI binding's rows.
- `prebindgen-jni/src/jni/classify.rs` — which of the seven wire layouts a type
  takes. Narrowed to `&Flat` in #642.
- `prebindgen-jni/src/jni/struct_plan.rs` — close strategies. Also `&Flat`.
- `prebindgen-c/src/chain.rs` — `CBody`, the C rendering vocabulary.

## Predecessor records

`docs/plan-duplication.md` is #613's own census and is current: the shape table
lost `CShape`, the fragment table records `rust_stages` moving onto
`JConverterArtifact` (#629), and a **reachability** section was added (#631)
because the census listed vocabularies and fields but not computations — which
is why nobody had noticed that reachability was stated twice over one fragment
set.
