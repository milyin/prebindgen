# The shape matrix — branch map

Integration branch for the coverage work tracked by
[#198](https://github.com/milyin/prebindgen/issues/198), itself part of
[#399](https://github.com/milyin/prebindgen/issues/399).

**This document is where step state is edited.** The umbrella PR's body mirrors
it — change the doc, then re-sync the body, never the other way round.

Child PRs target **`shape-coverage`**, not `main`.

## The problem

Which Rust shapes survive the trip to C or to Kotlin — `Option<&T>`,
`Vec<Option<T>>`, a payload-carrying enum inside a struct — is knowable today
only by reading the generators, or by writing the Rust and finding out. So a user
discovers a gap by hitting it, a reviewer cannot tell whether a change closed a
hole or moved it, and a regression that quietly drops a shape looks like nothing
at all: no test fails, because no test knew the shape was supported.

The answer is a **generated table**: one row per (shape, position, target), the
answer produced by running the real generators, never by a hand-maintained list
of what is supposed to work. There is deliberately no second authority on
legality anywhere in this work — a second opinion can disagree with the first,
and then neither is trustworthy.

## Why a branch

Unlike the abandoned #187, the steps here **are** independently landable: each
one leaves the tree green and has a concrete consumer, and step 1 is already
useful on its own.

The branch is not here to hide half-migrated states. It is here to keep one
reviewable thread for a change whose value is cumulative: *"every cell's state is
evidence rather than assertion"* is a property of the whole, and it is easy to
lose one PR at a time. Any step could go straight to `main` if it became urgent;
re-pointing it costs a base change.

`main` is merged **into** this branch whenever it moves, so the final merge is
not a rewrite.

## Steps

| # | Step | State |
|---|---|---|
| 1 | The enumerator, both targets, committed report + regen gate | **landed** ([#400](https://github.com/milyin/prebindgen/pull/400)) |
| 2 | Receipts: rustc accepts the emitted Rust | **landed** ([#403](https://github.com/milyin/prebindgen/pull/403)) |
| 2b | Kotlin emitted, and cbindgen asked for the header | **landed** ([#405](https://github.com/milyin/prebindgen/pull/405)) |
| 2c | The Kotlin compiler, and `RuntimeExercised` | not started |
| 3 | The minimum-guarantees table | not started |
| 4 | Multi-parameter aliasing fixtures | not started |
| 5 | The adapter-policy axis | JNI half ready; C half blocked |
| 6 | Plan-level invariants | blocked |

### 1. The enumerator — landed

`examples/shape-matrix` enumerates (shape × position × target), synthesizes a
fixture per cell, runs it through the real `prebindgen-c` and `prebindgen-jni`,
and writes `REPORT.md`. Two gates hold it honest, both of the same shape — a
closed vocabulary matched exhaustively, plus a test that the vocabulary is
exercised:

* **the type axis** — `tag_of` over `prebindgen_flat::flat::TypeKind`, so a new
  accepted Rust form stops the crate compiling until it has a fixture;
* **the declaration axis** — `kind_of` over `prebindgen_jni::ClassDecl`, so a
  fifth class kind does the same.

Three of the four report states exist: `rejected`, `plan`, and a third the issue
did not anticipate — **`panic`**, for a shape the generator refuses *without* a
diagnosis. That is [#191](https://github.com/milyin/prebindgen/issues/191)'s
evidence, per cell.

`REPORT.md` is committed and diffed by `examples/regen-check.sh`, which CI
already runs. That is the primary regression gate, and it is not decoration: the
generators both decide legality **and** write the report, so "the build fails
until new cells are classified" would be vacuous on its own — a regression that
flips a working cell to `rejected` would be recorded as a successful
classification.

### 2. Receipts — rustc — landed

Every cell that produced Rust is compiled, and the state is a **receipt**: the
cell is written to its own file, the crate is checked in one pass, and each
diagnostic is attributed back by the file rustc names. Nothing maps a cell to a
fixture by hand — that mapping is what let #175's test pass without creating its
own precondition.

Compiler messages stay out of the committed report: they vary by toolchain, and
the report has to be identical on every one that builds it.

Turning the compiler on immediately found ten cells whose generated Rust does not
compile, and three defects in this harness — each of which had been reporting a
confident wrong answer. That is the argument for this step in one sentence:
`plan` was worth less than it looked.

### 2b. Both halves, and the header — landed

Two stages cells were passing without reaching. **JNI produced no Kotlin at all**
— the driver wrote the Rust and stopped, so a passing cell had shown the half a
Kotlin caller never sees. And **C stopped at rustc**, which is the wrong finish
line: what a C consumer gets is a header, and a signature rustc accepts can be
one cbindgen skips or cannot name.

The header receipt is that the wrapper is *declared*, not that cbindgen returned
`Ok` — it returns `Ok` for a header declaring nothing.

The ladders differ by target now, and the report says so rather than levelling to
the shorter one: `header` for C, `rustc` for JNI.

### 2c. The Kotlin compiler, and the runtime

The Kotlin emitted in 2b is written but never compiled, so the JNI ladder still
ends one stage short of C's; closing it needs kotlinc in CI.
`RuntimeExercised` needs the JVM covertest and the C smoke tests, under the same
receipt rule — a fixture emits its cell id only *after* the relevant assertion
has executed, and a cell with no receipt keeps the weaker state whatever any
table claims.

### 3. The minimum-guarantees table

A committed table of `must plan` / `must compile` / `must execute` for shapes
already shipped. A cell listed `must execute` that reports only `PlanSupported`
fails the build. Seeded from what downstream depends on and from every cell the
covertest already touches.

This is the second gate on top of the committed report: the report catches a
*changed* answer, this catches an answer that was never strong enough.

### 4. Aliasing fixtures

Aliasing is a property of a **call**, not of a value: two parameters can name the
same resource. Three cases that must discriminate at runtime, not merely plan —
the same resource passed twice where one side consumes it (rejected); the same
pointers in an **inactive** enum alternative (**not** flagged, or working surface
silently disappears); and an invalid tag in a *later* argument after an earlier
one would already have been consumed, proving the check precedes decoding.

### 5. The adapter-policy axis

Vary *how* a type is declared, not just what it is: the same struct as a handle
and as a value type answers differently, and that difference is currently
invisible.

The JNI half is available now — its class kinds are a closed enum. The C half is
**blocked on the build-API rework** ([#192](https://github.com/milyin/prebindgen/issues/192)):
that API has eleven declarator methods and no type unifying them, so there is
nothing to enumerate against. Hand-writing a list of C declarators in the harness
would recreate precisely the drift the type axis exists to prevent, so the
harness models the JNI vocabulary and translates to C in one function, which the
rework is expected to rewrite wholesale.

### 6. Plan-level invariants

Every enum tag maps to exactly one alternative; every owned output is reached by
exactly one release path; every consumed input reaches exactly one commit or
rollback; the three views of one JNI function flatten to identical descriptors.
Written once against a small read-only interface each adapter's plan implements —
a shared way to *ask* questions is safe where a shared type for *storing* answers
is not.

Blocked on the plans existing at all
([#192](https://github.com/milyin/prebindgen/issues/192),
[#193](https://github.com/milyin/prebindgen/issues/193)).

## Exits

Each step states which of the two it is, up front:

* **answer-preserving** — `REPORT.md` gains or loses no cell state. A refactor of
  a measuring instrument must not move the measurements, and step 1's follow-up
  commit is the pattern: 11 insertions, 0 changed cells.
* **answers move** — the diff **is** the review. Each moved cell is named in the
  PR body with the cause, because a moved cell is either a capability change or a
  regression, and no one can tell which from the diff alone.

A blanket "the report never changes" would be false on contact and would train
reviewers to wave the diff through.
