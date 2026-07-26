# Boundary planning — integration branch

Status: **in progress**, on the long-lived `boundary-planning` branch.

The design authority is umbrella issue
[#187](https://github.com/milyin/prebindgen/issues/187): the target invariant, the
tier invariants, the non-goals, and the completion criteria live there and are not
restated here. This file is the branch's own map — what has landed, what each child
PR is, and how the chain is reviewed and merged.

## Why a branch rather than a series onto `main`

The refactor is nine ordered stages that move the same code repeatedly. Landing
them straight onto `main` would mean `main` spends months in a half-migrated
state: two boundary-decomposition models live at once during Stages 1 and 3, the
legacy classifiers survive until 4B, and the shadow oracles that make the
byte-identical exits credible are load-bearing until then. None of that is a state
a release should be cut from.

So `main` takes the whole thing once, when the chain is complete and every exit
criterion in #187 holds. Until then this branch is the integration point.

`main` is still merged **into** this branch whenever it moves, so the chain never
drifts far enough to make the final merge a rewrite.

## How child PRs stack

Each stage is its own PR **targeting `boundary-planning`**, not `main`:

```
main ──────────────────────────────► (one merge, at the end)
  └─ boundary-planning ◄── stage PR ◄── stage PR ◄── …
```

CI runs on every PR whatever it targets — `.github/workflows/rust.yml` deliberately
filters only `push`, so a stacked PR is not exempt from clippy, fmt, the golden-diff
check, the JVM harness, or the sanitizer gate. `workflow_dispatch` is there to run
the full suite on this branch directly.

Each stage PR states its own exit in #187's vocabulary, and the three verdicts mean
what they say there:

- **Must not move** — byte-identical, enforced by `examples/regen-check.sh`. A diff
  is a bug.
- **Reviewed diff** — expected to change, with the cause stated up front. A diff
  outside that cause is a bug.
- **Asserted** — the invariant the stage adds.

A blanket "goldens unchanged" is not usable and is not claimed; Stage 0 already
moved generated Rust for `bool` inputs.

## Stages

| Stage | Issue | Owns | State |
|---|---|---|---|
| 0 | [#189](https://github.com/milyin/prebindgen/issues/189) | Close the remaining boundary safety gaps | **merged to `main`** ([#200](https://github.com/milyin/prebindgen/pull/200)) |
| T | [#190](https://github.com/milyin/prebindgen/issues/190) | The Tier 0 semantic shape tier | not started |
| 2A | [#191](https://github.com/milyin/prebindgen/issues/191) | Typed planning outcomes and pre-resolution recipes | not started |
| 1 | [#192](https://github.com/milyin/prebindgen/issues/192) | First-class C wire semantics and per-use value plans | not started |
| 3 | [#193](https://github.com/milyin/prebindgen/issues/193) | Canonical JNI value plan *(the long pole)* | not started |
| 2B.1 | [#194](https://github.com/milyin/prebindgen/issues/194) | Recipe-derived reachability accounting and manifests | not started |
| 4A | [#195](https://github.com/milyin/prebindgen/issues/195) | Pure emission — emitters consume plans only | not started |
| 2B.2 | [#196](https://github.com/milyin/prebindgen/issues/196) | Prune unreachable converters, delete requirement bookkeeping | not started |
| 4B | [#197](https://github.com/milyin/prebindgen/issues/197) | Delete shadow oracles, obsolete hooks, compatibility types | not started |
| 5A | [#186](https://github.com/milyin/prebindgen/issues/186) | `KtExpr` AST infrastructure *(lands before Stage 3)* | not started |
| 5B | [#199](https://github.com/milyin/prebindgen/issues/199) | Migrate remaining Kotlin expression emission onto the AST | not started |
| — | [#198](https://github.com/milyin/prebindgen/issues/198) | Cross-cutting: shape matrix and plan-level invariants | grows with every stage |

```
Stage 0 (#189) ── landed on main, independent

                          ┌──> Stage 1 (#192) ──┐
Stage T ──> Stage 2A ─────┤                     ├──> 2B.1 ──> 4A ──> 2B.2 ──> 4B
 (#190)      (#191)       │                     │    (#194)  (#195)  (#196)  (#197)
                          │                     │
Stage 5A (#186) ──────────┴──> Stage 3 (#193) ──┴──> Stage 5B (#199)

Matrix (#198) ── grows with every stage
```

T and 2A establish the shared vocabulary; **both** adapter migrations depend on
both. Stages 1 and 3 run concurrently. The tail is strictly ordered — 2B.1 changes
no artifact, 4A must precede pruning because `prerequisites` runs at step 0 of
`write_rust` with the resolved registry, 2B.2 prunes against 2B.1's manifests, and
4B removes.

Stage 0 is on `main` rather than here on purpose: it is the only stage that depends
on nothing, none of it is expected to survive Stage 1 unchanged, and the UB it
closes should not wait for the chain.

## Migration is by shadow planning

Stages 1 and 3 build the new plan alongside the legacy path, assert the two agree
on every matrix cell, switch one emission position at a time, and leave the oracle
live until 4B. That is what makes a "must not move" exit mechanically credible
rather than a promise resting on golden review.

## Debts Stage 0 knowingly left

Tactical by design — #189 accepted that none of its work survives Stage 1 unchanged.
Two items are owed to later stages and must not be lost in the shuffle:

- **`Cbindgen::assume_c_field_validity`** is an acknowledgement, not a fix. A
  `repr_c_struct` mirror is reinterpreted wholesale by one `Transmute`, so a field
  whose Rust type has restricted validity (`bool`, a declared `enum_type`) has no
  per-value hook. `perftest-c`'s `Payload::flag` is the only shipping instance.
  **Stage 1 (#192)** brings the raw-wire lowering that makes the escape hatch
  removable; deleting it is part of that stage's scope.
- **The alias preflight** must be reproduced **byte-identically** from `AliasPlan`
  by Stages 1 and 3 — not replaced with a rejection, which would remove supported
  surface and break Stage 3's byte-identical exit. Its four assertions
  (`lang::cbindgen::tests::aliasing`, `lang::jnigen::jni::tests::aliasing`, plus the
  runtime cases in `example-cbindgen`'s `boundary_tests`, `c/smoke.c` and
  `covertest-kotlin`) are the oracle for that.
