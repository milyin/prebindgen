# Coherence report for #660

> **Superseded.** This describes the tree at #660. Three of the things it
> measures have moved since: `convert_with` and `Answer` were the registry's API
> and #677 deleted them, and `unfold` left the registry for `prebindgen-jni` in
> #666 before #680 returned its neutral half as `prebindgen_registry::leaf`.
> The report that followed is
> [`registry-conveyor-baseline.md`](registry-conveyor-baseline.md), and its own
> closing section is superseded in turn: #676 reopened after it was written. For
> the live state read #676 until its close child rewrites that document. This one
> is kept for the measurements it took, which are still what they were.

What the umbrella cost and where the lines went, measured with
`tools/line-report` — the same tool `docs/production-line-report.md` used for
#613, which counts production lines by asking `syn` where each `#[cfg(test)]`
construct ends.

## Before and after

Production lines, so the figures are comparable with #613's:

| crate | #660's base | after #670 | delta |
|---|---|---|---|
| `prebindgen-registry` | 17,050 | 13,624 | **−3,426** |
| `prebindgen-c` | 8,757 | 8,755 | −2 |
| `prebindgen-jni` | 31,698 | 35,431 | **+3,733** |
| all crates | 66,491 | 66,796 | +305 |

**The registry shrank and the JNI adapter grew by slightly more.** That is the
honest headline, and it is what item 1 asked for rather than a disappointment:
`unfold` is 5,672 lines that `prebindgen-c` never referenced once, so it left
the shared crate for the adapter that declares it. Nothing was folded away.

The +305 net is what the move cost in new mechanism: `Requirement` and
`Unfolding` (the seam that let `unfold` stop taking a `&mut Registry`),
`ArtifactPlan::follows` and `GenerationPlanBuilder::root` (what let the assembly
be a projection), and `FragmentPlan::conversion` (what let the emitters' view
stop holding a compile carrier).

This umbrella did not promise a reduction. #613 promised one and delivered
carriers, which is why its report leads with **C +3, JNI +1,164**. #660 stated
its target as coherence, and items 3 and 4 were the only two with a reduction in
them.

## What was deleted

| deleted | where | PR |
|---|---|---|
| four `Representation` bridge associated types | registry | #661 |
| four plan accessors on `Conversions` | registry | #664 |
| four decomposition plan maps on `Registry` | registry | #664 |
| `WriteRustError::Unfold` | registry | #666 |
| `unfold` (5,672 lines) | registry → `prebindgen-jni` | #666 |
| `Conv::fragment` | `prebindgen-jni` | #668 |
| `Conv::converter_impl` | `prebindgen-jni` | #670 |
| the JNI plan's filler declared-surface artifact | `prebindgen-jni` | #667 |

## Where the boundary stands now

The table #660 opened with, re-measured. It counted references from each adapter
into each registry module:

| registry module | C uses (then) | JNI uses (then) | now |
|---|---|---|---|
| `unfold` | 0 | 110 | not in the registry |
| `expand` | 0 | 15 | unchanged |
| `chain` | 3 | 44 | unchanged |
| `recipe` | 25 | 101 | unchanged |
| `generation` | 27 | 83 | unchanged |

`expand` is the row that did not move and is worth naming rather than leaving
for a fresh reading to rediscover. It is the parameter-side twin of `unfold` and
`prebindgen-c` references it zero times, so the same argument applies — with one
difference that stopped it being folded into item 1: `expand::apply` still
mutates the registry directly, where `unfold`'s could be reduced to a list of
output registrations. Moving it needs the same `Requirement`-shaped seam on the
input side first.

## What each item settled

- **1** — `unfold` is `prebindgen-jni`'s. There was no second consumer to prove
  the abstraction with, and the `Representation`-style seam the alternative
  asked for already exists (`recipe` and the shape vocabulary) and is what C
  decomposes values through.
- **2** — the shared declaration vocabulary names a *target language*. The
  field it carried was `kotlin_name_override`, and `prebindgen-c` reached it
  through `From<FunctionDecl>` and got told its function "is never surfaced in
  Kotlin".
- **3** — one `Representation::Bridge`. Five names distinguished nothing that
  the `ShapePlan` variant holding the bridge did not already state, and no
  implementation had two distinct types to reject across.
- **4** — `Assembly` is `plan.artifacts()`. This needed the plan to admit an
  artifact whose existence *follows* a fragment rather than causing it, a
  reachability root that is not an artifact, and declaration order as emission
  order.
- **5** — the emitters' view holds no carrier. Three of the four names in the
  item turned out to be the `Compile` trait's own extension points, which both
  adapters fill; see below.
- **6** — a choice member is composed in place. Recorded on `Reach::Nested`,
  where the question is asked.

## Two items closed by measurement

Worth recording, because both were stated as work and neither was:

**Item 5 named four things to delete, and one was a carrier.** `Conv` — the
read-only view an emitter gets of one compiled crossing — held the compiler's
`JFrag`, and #668 and #670 removed it. The other three are the shape both
adapters have: `JFrag` is `Compile::Fragment`, which `prebindgen-c` fills with
`CFrag`; `JPlan` is `Compile::Plan`, which C fills with
`SitePlan<CRepresentation>`; `JLayout` sits in `JConverterArtifact::layout`,
which is the `AbiLayout<R::AbiLayout>` slot the plan provides.

`JPlan` is not a duplicate of the site plan either, which is the reading its
name invites: `freeze_site_of` puts the **same** `Rc` in the `SitePlan` that the
`JPlan` holds and asserts `Rc::ptr_eq`, so a later `clone()` cannot quietly make
it a copy.

A slice was drafted to move `out_wires`/`wires` onto `JConverterArtifact` and
withdrawn on measurement: `callback_input` and `freeze_callback_delivery` are
reached only from `JCompile::callback`, a composition hook whose signature the
`Compile` trait itself writes as `&[&Self::Fragment]`.

**Item 6 was a decision, not a mechanism.** The umbrella warned it had five
successive wrong diagnoses in #613 and should not be started without deciding
first. What decides it is where two steps sit: reading a shape's parts off the
model is a `&self` step with no adapter in reach, and composing anything needs
one — so a pre-built fragment would have to be built during that read, turning
the one purely-model step into a second composition site.

## What is left

Building the composed choice member item 6 settled on. It belongs with
`effective_callback_plan`'s removal rather than before it: the callback-argument
rows are `Deconstructing::Atomic` today and that function is what routes a
callback to such a row, so the row's shape and the function are one mechanism.

## Predecessor records

`docs/production-line-report.md` is #613's report. Its step-5c paragraph is
superseded here: it says two of `Conv`'s four remaining fragment reads are
composition, and only one was — `pipeline` and `output_abi` were blocked by
something else, that a fragment composed into its parent froze without an
artifact and `Conv::pipeline` was asked of one.
