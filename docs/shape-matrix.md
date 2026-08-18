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
| 2c | The runtime — C side, in pure Rust | **landed** ([#409](https://github.com/milyin/prebindgen/pull/409)) |
| 2d | The Kotlin compiler and the JVM runtime | not started — needs kotlinc and a JVM in CI |
| 3 | The guarantee ratchet — a floor per cell | **landed** ([#406](https://github.com/milyin/prebindgen/pull/406)) |
| 4 | The call axis | **landed** ([#407](https://github.com/milyin/prebindgen/pull/407)) |
| 5 | The declaration-policy axis | **landed** for the JNI vocabulary ([#408](https://github.com/milyin/prebindgen/pull/408)); C's own declarators blocked |
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

### 2c. The runtime, C side — landed

The C target's `extern "C"` wrappers are ordinary Rust functions, so running them
needs no C toolchain and no CI change: it is a `cargo test`. Three cases settle
what the call axis could not — the alias guard fires, spares, and runs *before*
ownership moves, the last proven by reclaiming the resource afterwards, which
would be a double free if a converter had run.

Two shapes of C binding are now in the corpus because the difference is
load-bearing: a `()`-returning one **aborts** on rejection (a panic cannot cross
`extern "C"`), while a fallible one reports through the error out-param. Only the
second is observable in-process, and only the second is what a real C API does.

### 2d. The Kotlin compiler and the JVM runtime

Still open, and the first step in this sequence that is not self-contained: the
Kotlin emitted in 2b is written but never compiled, and its wrappers are entered
from a JVM. Both need toolchains in CI.

### 3. The guarantee ratchet — landed

A floor per cell, in a committed `GUARANTEES.md`: the level it has been seen to
reach. Rising is free; falling fails a test naming the cell and both levels.

This is the gate the report cannot be. A byte-identity diff shows a cell getting
worse in the same shade as one getting better, so it catches a regression only if
a reviewer reads the diff and knows which direction is which. A floor does not
need a reviewer.

**Raising is automatic (`--update-guarantees`); lowering is a hand edit.** Giving
up on a shape that used to work should cost a visible line in a diff, not a
silently regenerated artifact.

The `must execute` level the issue also asks for waits on step 2c — there is no
runtime state for a floor to stand on yet.

### 4. The call axis — landed

Aliasing is a property of a **call**, not of a value: two parameters can name the
same resource, and both generators emit a preflight under a rule about the whole
parameter set. Eight call shapes now run through the same driver, compile check,
header stage and ratchet — pairs that must be guarded, a pair that must not be
(two shared borrows of one resource are legal), and pairs in different domains.

All eight are expressible in both targets, so the axis found nothing today. What
it buys is that a call shape becoming inexpressible now falls below a floor.

**Three claims it does not make**, all of which need running code and belong to
the runtime stage: that the guard rejects the aliased call, that it spares the
same pointers appearing in an **inactive** enum alternative, and that it runs
*before* ownership moves — provable by an invalid tag in a later argument, after
an earlier one would already have been consumed. None of these is asserted
against emitted text: a grep establishes that the text contains a guard, which is
not the property anyone cares about.

### 5. The declaration-policy axis — landed for the JNI vocabulary

The same Rust, in the same position, with its declared type declared as something
else. Twelve curated cases, each printing the varied answer beside the canonical
one.

Three rows where the policy decides the answer: `Vec<Rec>` as a parameter crosses
by value and is refused as handles; `Option<Rec>` as a C parameter emits broken
Rust by value and works as a handle, so that defect belongs to the by-value path;
and a fieldless enum returned as a handle is refused at compile time because the
tagged-pointer representation needs alignment ≥ 2.

That last one exposed a gap in this report's vocabulary rather than in the
generator: a *deliberate* compile-time refusal and genuinely broken output both
read as `bad rust`. The legend now says so and points at #191, whose subject is
exactly a refusal arriving later than declaration time.

**C's own declarators remain blocked** on the build-API rework
([#192](https://github.com/milyin/prebindgen/issues/192)). The four kinds varied
here are the JNI adapter's closed vocabulary, which is what makes them
enumerable; C is measured through the same four by translation, so its rows are
real but its coverage is not — `repr_c_struct`, `opaque_data_struct`, `callback`
and the rest have nothing to enumerate against.

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

## What it has found

The table is not the deliverable; the tickets are. As of `146eb348`:

| Issue | Finding | State |
|---|---|---|
| [#410](https://github.com/milyin/prebindgen/issues/410) | JNI emits an unqualified `Cow` into the consumer's scope | fixed ([#423](https://github.com/milyin/prebindgen/pull/423)) |
| [#411](https://github.com/milyin/prebindgen/issues/411) | JNI decodes an exclusive-borrow parameter as a shared or plain value | fixed ([#426](https://github.com/milyin/prebindgen/pull/426)) |
| [#412](https://github.com/milyin/prebindgen/issues/412) | C moves out of a raw pointer for an `Option<T>` by-value parameter | fixed ([#425](https://github.com/milyin/prebindgen/pull/425)) |
| [#413](https://github.com/milyin/prebindgen/issues/413) | C returns of borrowed elements: `&[T]` drops the value, `Vec<&T>` maps over an `unsafe fn` | fixed ([#427](https://github.com/milyin/prebindgen/pull/427)) |
| [#414](https://github.com/milyin/prebindgen/issues/414) | C qualifies std `Option` into the source module, and leaves the declared type bare | fixed ([#424](https://github.com/milyin/prebindgen/pull/424)) |
| [#428](https://github.com/milyin/prebindgen/issues/428) | C calls the composite marker for an `Option<&T>` **callback argument**, and the marker takes no arguments | open |
| [#191](https://github.com/milyin/prebindgen/issues/191) | a third of C's refusals and a fifth of JNI's arrive as panics — now a measured number | open |

Two of the five carried a diagnosis rather than a symptom, and both came from a
comparison no single cell could make: #412 was narrowed to the by-value decode
path because the same cell with the type declared as a handle compiles, and #414
got both halves of one expression's qualification wrong in opposite directions.

All five are merged, and this branch has merged `main` back. The report moved
the way the fixes predicted: C's `bad rust` column is 5 → 0 and Kotlin/JNI's
6 → 1, with two JNI cells falling from `bad rust` to `rejected` — `&mut T` over
anything but a handle now says so instead of emitting a wrapper that discards
the callee's writes. That fall tripped the ratchet, which is the ratchet working:
lowering those two floors is a hand edit in `146eb348`, visible in the diff.

**#428 is a finding about the corpus as much as about the generator.** It is
#413 with the directions swapped — a composite lowered structurally in one
direction and left to a marker in the other — and this crate did not catch it
because a **callback argument is not one of the four positions** it enumerates.
The fix for that is a corpus extension, not a generator change.

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
