# Splitting `prebindgen` into layered crates

Tracking document for the umbrella branch. Each phase below lands as its own
stacked PR targeting `umbrella-split-into-layered-crates`; the umbrella merges
to `main` once the chain is complete.

## Why

`prebindgen` is one ~75k-line crate carrying the JSONL source reader, the flat
model, the registry pipeline, the JNI/Kotlin adapter (40k lines, a `jni`
dependency) and the C/cbindgen adapter. That has four costs:

- A **source** crate like `zenoh-flat`, which only calls
  `init_prebindgen_out_dir()`, compiles the whole JNI adapter and links `jni`.
- A **shipped binding library** like `zenoh-flat-jni` depends on the entire
  generator — `syn`, `quote`, `prettyplease`, `toml` — purely to reach
  `JniBindingError` / `box_j*` / `CachedIfaceMethod`, which it needs at *run*
  time.
- The C adapter sits behind an `unstable-cbindgen` feature that made the
  committed goldens feature-sensitive: `--all-features` regenerated them into a
  variant CI then rejected, and a bare `cargo test` silently skipped ~60 tests.
- Two layers that should be independent form **dependency cycles**:
  `flat → registry` and `cbindgen → jnigen`.

## Target layout

```
~/PREBINDGEN_WORKSPACE/
  kotlin-codegen/         separate repo, own release cycle
  prebindgen/             the workspace below

prebindgen-proc-macro     unchanged

prebindgen                BASE. Source stream only. No adapters, no registry.
  └─ Source, Record, SourceLocation, init_prebindgen_out_dir / get_* ,
     RustEdition, TargetTriple, utils::jsonl, DEFAULT_GROUP_NAME

prebindgen-flat           the flat model + TypeKey + the Emit capability
  deps: prebindgen

prebindgen-registry       language-agnostic pipeline + shared decl vocabulary
  deps: prebindgen-flat

prebindgen-jni            JNI/Kotlin generator
  deps: prebindgen-registry, prebindgen-jni-runtime, kotlin-codegen, jni

prebindgen-c              C/cbindgen generator (no feature gate)
  deps: prebindgen-registry, prebindgen-c-runtime

prebindgen-jni-runtime    LEAF. ~350 lines. deps: jni
prebindgen-c-runtime      LEAF.  ~65 lines. deps: none

── separate repository ──
kotlin-codegen            LEAF. General-purpose Kotlin emitter. deps: none
```

The runtime crates are leaves: the generators depend on them only to name their
paths in emitted code, and nothing depends on a generator at run time. A shipped
binding library ends up depending on ~350 lines instead of 75k.

## Breaking the cycles

**`flat → registry` was `TypeKey` alone.** `TypeKey::from_type` already called
`flat::canonical_type`, so the type belonged in `flat`; `registry` re-exports it.

**`cbindgen → jnigen` was the declaration vocabulary.** `ConvertDecl`,
`ConvertSpec` and `local_path_prefix` are not JNI concepts — they are what a
build script writes. They moved down to `core::decl`.

One correction found while executing: the **expand decls are shared, not
JNI-only**. `fun!` expands to `FunctionDecl`, and `example-cbindgen/build.rs`
already writes `convert!(Millis).input(fun!(millis_from_raw))`, so
`FunctionDecl` must reach the C adapter — and it holds `ExpandParamDecl` /
`ExpandReturnDecl`, which drags `ExpandDecl` and `FieldsDecl` along. Harmless:
all four name only `TypeKey`, `Origin<syn::Type>`, `syn` types and each other,
never a Kotlin or JNI type.

## What the split costs

`Emit::new()` is `pub(in crate::api::core)` — *only core may mint the
capability*. Once `core` becomes `flat` + `registry` that is **inexpressible**:
`registry` must mint, and Rust has no cross-crate friend. It becomes `pub`.

The important half survives. `as_syn` / `spell` / `stripped_syntax` stay private
**inside `flat`**, so "classify off `kind`, spell with `spell()`" still holds.
What is lost is only the secondary control over *who mints*. `emit.rs`'s module
doc asserts the stronger claim today and must be amended when the `flat` carve
lands.

## Phases

| Phase | Content | Status |
|---|---|---|
| **B0** | `kotlin-codegen` → its own repo | done |
| **A1** | `TypeKey` → `flat`; breaks `flat → registry` | done |
| **A2** | `prebindgen-{jni,c}-runtime` carved out; emitted paths repointed | done |
| **A3** | shared decl vocabulary → `core::decl`; breaks `cbindgen → jnigen` | done |
| **A5** | drop the `unstable-cbindgen` feature | done |
| **A4** | `Emit` → `flat`; `flat/` reaches zero core-sibling refs | done |
| **B1** | strip `prebindgen` to the base crate | todo |
| **B2** | carve `prebindgen-flat` | todo |
| **B3** | carve `prebindgen-registry` | todo |
| **B4** | carve `prebindgen-c` | todo |
| **B5** | carve `prebindgen-jni` | todo |
| **C** | workspace manifest, examples, docs, downstream repos | todo |

Phase A is all in-place, so the tree stays green at every commit and the Phase B
moves are close to pure renames. B1–B5 are strictly sequential, bottom-up.

### Phase B notes

Each carve is a module-tree `git mv` plus a path rewrite
(`crate::api::core::flat::` → `prebindgen_flat::`, …). `api/test_util.rs` is used
by tests in several destinations — duplicate the ~30 lines per crate rather than
adding a shared test-only crate.

The visibility relaxations belong here, not earlier: done in place,
`pub(in crate::api::core)` still compiles, so relaxing before the carve would
weaken the seal with no compiler check that it was done right. At carve time the
compiler names exactly which items need `pub`.

### Phase C notes

- workspace version → **0.6.0**; `prebindgen`'s public API shrinks, so this is
  breaking
- `examples/{example-cbindgen,perftest-c}` → `prebindgen-c` build-dep;
  `examples/{covertest,perftest}-kotlin` → `prebindgen-jni` build-dep
- `examples/{example,perftest}-flat` and `covertest-helpers` are **unchanged** —
  they only call `init_prebindgen_out_dir()`, which stays in the base crate
- `README.md` + `lib.rs` rewritten for a base crate, with the C and JNI usage
  docs moved into their respective generator crates
- downstream, outside this repo: `zenoh-flat-jni` and `zenoh-flat-c` manifests,
  plus the stale `zenoh-flat-jni/kotlin/generated/.prebindgen-kotlin-output`
  marker → `.kotlin-codegen-output`. `zenoh-flat` needs no change.

## Verification, every phase

- `cargo test --all` — baseline **535 lib + 34 doc + 20 + 10**, 0 failures
- `cargo clippy --all-targets --all-features -- --deny warnings` and
  `cargo fmt --check -- --config "unstable_features=true,imports_granularity=Crate,group_imports=StdExternalCrate"`
  on **both 1.85.0 and stable** — stable-only clippy misses MSRV lints
- `./examples/regen-check.sh` — byte-identical, except where a phase changes an
  emitted path on purpose; that is what proves a move was a pure move
- `./examples/smoke-asan.sh`, and the covertest-kotlin JVM harness
- layer DAG: `cargo tree -i -p prebindgen-jni` must not list `prebindgen-c`, and
  the reverse; `cargo tree -p prebindgen-c-runtime` must show no dependencies

### The one thing no gate catches

**Over-publicizing.** The natural way to make a cross-crate build compile is to
add `pub` until the errors stop — and that passes every check above while
dissolving the invariant `flat/mod.rs` spends a page defending. Every visibility
change in B2/B3 gets a deliberate diff review, and the rule when a privacy error
appears is to report it, not to widen the item.
