# Splitting `prebindgen` into layered crates

Record of the split, which landed on `main` as umbrella #371 — a chain of
stacked PRs, one per phase. Kept for the reasoning: why the layers sit where
they do, which predictions turned out wrong, and what the split cost.

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

prebindgen-flat           the flat model + TypeKey + the RustEmitter protocol
  deps: prebindgen

prebindgen-registry       language-agnostic pipeline + Emit key + shared decl vocabulary
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

## What the split costs — and how the emission boundary works

`prebindgen-flat` owns the facts required to generate Rust, while
`prebindgen-registry` owns the final-file policy and source-module map. Retained
frontend syntax remains private diagnostic/round-trip state; it is not an
emission source.

The boundary is therefore split into protocol and key:

- `prebindgen-flat::RustEmitter` is the object-safe, generate-only protocol. Its
  methods emit source types from `TypeKind`/identities, constructors from
  `FieldShape`, and private output fragments recorded by Flat. It has no method
  returning a captured spelling or typed source AST.
- `prebindgen-registry::Emit` is the concrete registry key. Its constructor
  is registry-private; `write_rust` and `RegistryBuilder::convert_with`
  hand references to it only to emission callbacks.
- `Emit` exposes a deliberately narrow forwarding surface such as
  `emit_source_type`; it does not implement or dereference to `RustEmitter`.

This preserves the dependency and mental model:

```text
prebindgen -> prebindgen-flat -> prebindgen-registry -> prebindgen-{c,jni}
  extract          parse              collect              convert
```

It also preserves independent use. A different collector can depend directly
on `prebindgen-flat` and implement `RustEmitter` for a key it owns, without
gaining access to retained source syntax.

The registry's default path is compiler-checked. `prebindgen-registry` does
not re-export `RustEmitter` through its `flat` model path, so an adapter that
depends only on the registry cannot name the protocol or construct
`prebindgen-registry::Emit`; it can render only after receiving `&Emit` in
an emission callback. A compile-fail doctest pins both restrictions.

An adapter could add a direct `prebindgen-flat` dependency and implement the
public protocol, but that only reproduces the same generate-only operations.
Workspace adapters still use registry `Emit`, keeping qualification and final
symbol policy centralized.

The accidental direct doors introduced by the split are closed:
`TypeRef::spell` and `Origin::spell` are crate-private, the test-only
`Flat::enum_item` syntax accessor is gone, and raw `syn` access remains
private. Compile-fail doctests cover direct spelling, raw-node access,
registry-key construction, and the hidden registry-to-flat protocol path.

The other widened methods are intentional flat-model API rather than emission
doors. `Flat::classify`, `Flat::add_local_function`,
`TypeRef::{borrowed, optional, vector, scalar}`, and `types_util::ident` let an
independent parser or collector build and compose the representation without
depending on the registry.

This addresses [#375](https://github.com/milyin/prebindgen/issues/375) without
reversing the crate dependency.

## Phases

| Phase | Content | Status |
|---|---|---|
| **B0** | `kotlin-codegen` → its own repo | done |
| **A1** | `TypeKey` → `flat`; breaks `flat → registry` | done |
| **A2** | `prebindgen-{jni,c}-runtime` carved out; emitted paths repointed | done |
| **A3** | shared decl vocabulary → `core::decl`; breaks `cbindgen → jnigen` | done |
| **A5** | drop the `unstable-cbindgen` feature | done |
| **A4** | rendering protocol → `flat`; registry-owned `Emit` key → `registry` | done |
| **B1** | carve `prebindgen-c` | done |
| **B2** | carve `prebindgen-jni` | done |
| **B3** | carve `prebindgen-registry` | done |
| **B4** | carve `prebindgen-flat`; `prebindgen` is what remains | done |
| **C** | workspace manifest, examples, docs, downstream repos | done |

Phase A is all in-place, so the tree stays green at every commit and the Phase B
moves are close to pure renames.

### The carves run top-down, not bottom-up

An earlier revision of this document had B1 strip `prebindgen` to the base and
then build the layers up from it. That order is impossible one crate at a time,
because every intermediate state would need a Cargo dependency cycle:

`flat` uses `crate::SourceLocation`, which stays in the base, so
`prebindgen-flat` → `prebindgen`. But `registry` depends on `flat`, and until
`registry` has moved out it is still *inside* `prebindgen` — so `prebindgen` →
`prebindgen-flat` at the same time. Cargo rejects that.

Removing the **topmost** layer each time has no such problem: what remains never
depends on what just left. So the adapters go first, then `registry`, then
`flat`, and the base crate is simply whatever is left over — it is never
"created" at all. The two adapters are independent siblings, so `c` before `jni`
is only a size choice: 9.8k lines against 40k, the smaller one first to shake
out the mechanics.

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

- workspace version stays **0.5.0**. `prebindgen` has never been published, so
  its shrinking API breaks no released contract and the split is the 0.5 shape
  rather than a bump away from it. `kotlin-codegen` is independent, released
  from its own repo, and consumed from crates.io.
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
- `./examples/regen-check.sh` — deterministic against committed goldens;
  intentional semantic-preserving output changes are reviewed and committed
  once, then the check pins their new form
- `./examples/smoke-asan.sh`, and the covertest-kotlin JVM harness
- layer DAG: `cargo tree -i -p prebindgen-jni` must not list `prebindgen-c`, and
  the reverse; `cargo tree -p prebindgen-c-runtime` must show no dependencies

### The one thing no gate catches

**Over-publicizing.** The natural way to make a cross-crate build compile is to
add `pub` until the errors stop — and that passes every check above while
dissolving the invariant `flat/mod.rs` spends a page defending. Every visibility
change in B2/B3 gets a deliberate diff review, and the rule when a privacy error
appears is to report it, not to widen the item.
