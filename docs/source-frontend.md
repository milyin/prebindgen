# One Rust frontend and a common source IR — branch map

Integration branch for the frontend work tracked by
[#211](https://github.com/milyin/prebindgen/issues/211).
**Draft, and long-lived** — do not merge until every completion criterion in
#211 holds.

#211 remains the authority on the invariants, the frontend/adapter boundary, and
the completion criteria. This document does not restate them; it records what has
landed, what has not, and the order.

## Why a branch

Unlike [#187](https://github.com/milyin/prebindgen/issues/187), whose stages are
*not* independently landable, #211's steps mostly are — the issue says so
explicitly: "Each step should leave tests green and have a concrete consumer."

So this branch is not here to hide half-migrated states. It is here to keep one
reviewable diff against `main` for a change whose value is **structural**: "no
adapter independently parses source Rust" is a property of the whole, not of any
one step, and it is easy to lose one PR at a time.

The consequence is worth stating plainly: because the steps *are* independently
landable, any of them could go straight to `main`, exactly as Stage 0 (#200) did
for #187. Routing them through this branch is a choice about reviewability, not a
technical necessity — so if a stage's own fix becomes urgent, re-pointing it at
`main` costs nothing but a base change.

Child PRs target **`source-frontend`**.

## Size of the problem

`syn::Type::` / `syn::Expr::` match sites outside tests and outside the frontend,
i.e. places that classify source shape today:

| Where | Sites |
|---|---|
| `api/core` | 67 |
| `api/lang/jnigen` | 97 |
| `api/lang/cbindgen` | 25 |

Not all of these are *source*-syntax classifiers — many inspect types the adapter
itself synthesized (wire types, converter signatures), which is legitimately the
adapter's business. Separating the two populations is F1's job, and the number is
here as a scale marker, not a target.

## Stages

| Stage | Owns | State |
|---|---|---|
| F0 | Array-length subgrammar; the `core::frontend` seed | **done** — [#212](https://github.com/milyin/prebindgen/pull/212), closes #210 |
| F1 | Complete the source-language specification and classify the 189 sites | not started |
| F2 | `SourceModel` — the language-neutral item model | not started |
| F3 | Move the type-shape classifiers into the frontend | not started |
| F4 | `Registry` consumes `SourceModel` | not started |
| F5 | Migrate `Cbindgen` | not started |
| F6 | Migrate `JniGen` *(the long pole — 97 sites)* | not started |
| F7 | Close the open-syntax boundary in the `Prebindgen` trait | not started |
| F8 | Mechanical boundary check | not started |

## Checklist

### F0 — array lengths and the frontend seed — **done** (#212)

- [x] `core::frontend` module exists, with module docs stating the one-walk rule
- [x] `ArrayLen` — one closed representation (`Literal` / `SourceConst` / `ExternalConst`)
- [x] `lower_array_len` — a single fallible walk; acceptance is a consequence of lowering
- [x] Resolution runs at ingest (`Registry::from_items` pass 3), before any adapter exists
- [x] `ScanError::UnsupportedArrayLength`, naming the offending sub-expression
- [x] Transactional lowering — a refused item leaves no partially rewritten model
- [x] jnigen's `reject_unsupported_array_length`, `QualifyLengthPaths` and `length_names` deleted
- [x] Table-driven acceptance matrix, including the #210 `<Holder>::N` regression
- [x] `docs/source-language.md` — the written contract
- [x] Covertest fixture: a `#[prebindgen]` const used as a length, compiled from another crate

F0 is **self-contained**: CI-green, and it closes #210 on its own. It targets
this branch so the chain has one diff against `main`, but it is the one stage
that could be re-pointed at `main` at any time — as #187 did with its Stage 0 —
if #210 should be fixed before the rest of the chain lands. Note the cost of
keeping it here: `Closes #210` does not fire until this branch merges.

### F1 — specify the source language

`docs/source-language.md` currently inventories item kinds, function forms, and
type forms, but only the array-length row has actually moved into the frontend;
every other row records where the decision is made *today*, often inside an
adapter.

- [ ] Classify each of the 189 sites: **source-shape classifier** (must migrate) vs **adapter-synthesized-type inspection** (stays)
- [ ] Complete the acceptance inventory from the C and JNI covertests — supported / explicitly unsupported, no opportunistic expansion
- [ ] Decide the currently-unspecified rows: tuple-struct fields, `union`, `type` alias, generic parameters, raw pointers
- [ ] Nail down what a *passthrough* item may contain — today it is emitted verbatim and uninterpreted

### F2 — `SourceModel`

- [ ] A language-neutral item model: identity, origin, fields, variants, parameters, returns, ownership, borrows
- [ ] Preserve source locations and produce useful diagnostics
- [ ] One public frontend entry point from captured records to `SourceModel` (completion criterion 1)
- [ ] `ArrayLen` folds into it rather than sitting beside it

### F3 — move the type-shape classifiers into the frontend

- [ ] **Unify the two one-level type walkers**: `registry::immediate_subtype_positions` and `types_util::immediate_pattern_children` are near-duplicates that already diverge in their `Type::Path` handling
- [ ] `types_util::normalize_type` / `normalize_item_types` — ingest normalization belongs to the frontend
- [ ] `extract_fn_trait_args` — the `impl Fn(..) + Send + Sync + 'static` classifier
- [ ] `Registry::scan_fn_signature`'s receiver and parameter-pattern guards
- [ ] `types_util::match_pattern` and the `Type::Infer` unification

### F4 — `Registry` consumes `SourceModel`

- [ ] Registry indexes `SourceModel` items rather than raw `syn::Item` maps
- [ ] A temporary compatibility bridge is acceptable while adapters migrate
- [ ] The public `functions` / `structs` / `enums` / `consts` / `passthrough` fields stop being the adapter-facing contract
- [ ] Relate to [#92](https://github.com/milyin/prebindgen/issues/92) (split `Registry` into phase-specific types) — same seam, different cut

### F5 — migrate `Cbindgen`

- [ ] `lang/cbindgen/{trait_impl,convert,emit,builder,mod}.rs` consume `SourceModel`
- [ ] Generated C artifacts byte-identical, except where previously-accepted ambiguous syntax becomes an intentional frontend error
- [ ] Cbindgen gains array support, or refuses arrays explicitly — today it has neither

### F6 — migrate `JniGen`

- [ ] `prim_array_of` reads `ArrayLen` instead of re-matching `Type::Array`
- [ ] `builder.rs`, `emit/*`, `iface.rs`, `selector.rs`, `render.rs`, `fold.rs`, `overloads.rs` consume `SourceModel`
- [ ] `emit/names.rs`'s remaining source probes (`pat_match`, `pat_match_top`, `option_inner_ref_mutability`, `rust_short_name_opt`) classified per F1 and migrated or justified
- [ ] Generated Rust and Kotlin byte-identical, same exception as F5
- [ ] Relate to [#93](https://github.com/milyin/prebindgen/issues/93) and [#94](https://github.com/milyin/prebindgen/issues/94)

### F7 — close the open-syntax boundary

#211: "Keeping `syn` internally in the frontend is fine. Letting unclassified
`syn::Expr` or equivalent open syntax become an adapter-facing semantic contract
is not." These are the places where it currently does:

- [ ] `Niches { value: syn::Expr, matches: syn::Expr }` — a niche is a semantic fact carried as raw expression syntax
- [ ] `DomainScalar::rust_expr()` / `portable_expr()` returning `syn::Expr`
- [ ] `ConverterImpl::function` / `TypeEntry::function` as `syn::ItemFn`
- [ ] `Prebindgen::post_process_item(&mut syn::Item)` — the hook that let qualification live in an adapter in the first place
- [ ] `Prebindgen::prerequisites` / `local_functions` returning raw items

Note this one overlaps #187's emission work; F7 is about the **contract**, not
about how emission is planned.

### F8 — mechanical boundary check

- [ ] A test or CI check that fails when a new source-syntax classifier appears outside the frontend (completion criterion 6)
- [ ] Seeded from F1's classification, with an explicit allow-list that shrinks as F3–F6 land
- [ ] The allow-list is the honest scoreboard: it must not be possible to add an entry quietly

## Relationship to #187

Deliberately separate, and neither blocks the other.

```
#211:  source Rust ──> common source IR
#187:  common source IR ──> adapter boundary plans ──> emission
```

If #187 proceeds first, its Tier 0 and later plans should consume the frontend
model rather than introduce another source classifier. If this branch proceeds
first, F3–F6 reduce the surface #187's stages have to move.

The one concrete coupling: #187's Stage T (#190, merged) introduced a semantic
shape tier. F1 must classify whether Tier 0's shape reading is a *source* fact
(migrates here) or a *boundary* fact (stays there).

## Review protocol

Each stage PR states its own exit:

- **Must not move** — byte-identical, enforced by `examples/regen-check.sh`. A diff is a bug.
- **Reviewed diff** — expected to change, cause stated up front. A diff outside that cause is a bug.
- **Asserted** — the invariant the stage adds.

CI runs on every PR whatever it targets, so a stacked PR is not exempt from
clippy, fmt, the golden-diff check, the JVM harness, or the sanitizer gate.
