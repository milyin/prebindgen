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
— branches that build the step far enough to see what it runs into, named at the
end of this section, and part of no merged tree — and record where each stopped
and what it left open:

1. **The registry cannot build a `Compile::Fragment`.** It is the adapter's
   type and the registry never looks inside one. Step 4's "the registry wraps
   the bridge into the fragment" therefore needs a second hook per shape, and
   step 3's `GenerationPlan<C: Compile>` needs the adapter's `Compile` type to
   be lifetime-free, which costs more than deleting `Representation` saves.
2. **The declaration applier runs before the recipe table exists.**
   `declare_into` runs it, and `recipes()` is built afterwards *from the plans it
   produces*. So step 6's "readers of `Product` recipes" has nothing to read;
   what is duplicated is the derivation, and one shared derivation fixes it.
3. **Step 7's prototype did not share site enumeration, and one difference
   between the adapters is still open.** Three of the four differences between what the two adapters
   call a site are about which positions each of them plans, and a hook that
   asks the adapter settles all three: `prebindgen-c` skips a `()` return
   because C has nothing to hand back there while `prebindgen-jni` plans one;
   JniGen answers a callback parameter whole; JniGen's synthesized const getters
   are not `#[prebindgen]` functions, so they sit in no export set and a walk
   over the exports never reaches them. The fourth difference is about what one
   site *crosses*: `prebindgen-c` enumerates a `Result` return as **two** sites,
   the ok arm and the error arm, and JniGen as **one**, whose crossing is the
   whole `Result`.

   The prototype kept C's split and stopped there, so what it shows is a limit
   of the prototype rather than a property of the registry. #675 defines a site
   as a source position and a crossing as that Rust type with a direction, which
   makes the whole `Result` the one semantic return crossing. C's split is its
   **delivery policy** — how the ok value and the error each reach the caller —
   sitting inside C's private enumeration, which is the duplication step 7
   exists to remove. The open work is to normalize the site and its crossing in
   the registry, and to let each adapter lower that one `Result` crossing:
   JniGen to a single JVM return, `prebindgen-c` to its out-parameter and its
   error channel.
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

Findings 3 and 4 were reached by building them, on the branches
`feat/676-step7-registry-owns-the-plan` and `feat/676-step6b-one-derivation`.
Both are **prototypes, not part of this tree**: neither has had review, and a
branch can change or go away. They are recorded here as the directions a
successor should try first; this document otherwise describes the committed tree
(#681 review).
