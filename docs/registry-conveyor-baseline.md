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

## Closing figures

Measured on `umbrella/registry-conveyor` when #676 closed, by the same script
and the same `grep`.

| figure | baseline (`eb4aa007`) | closing | change |
|---|---|---|---|
| 1 — adapter production lines in registry-facing files | 28,348 | **28,140** | −208 |
| 2 — production lines across the three crates | 57,844 | **57,756** | −88 |

Per crate:

| crate | production | test items | test files | total |
|---|---|---|---|---|
| `prebindgen-registry` | 15,153 | 724 | 7,708 | 23,585 |
| `prebindgen-c` | 8,658 | 81 | 5,826 | 14,565 |
| `prebindgen-jni` | 33,945 | 1,582 | 21,826 | 57,353 |

Both figures are below the baseline, which is the criterion #675 states. The
registry grew and both adapters shrank, which is what the criterion is for.

## What the umbrella did, and what it did not

Four children landed:

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

Steps 4, 5, 6's second half and 7 did not land. Each was built or priced, and
each rests on something #675 assumes about this code that is not so:

1. **The registry cannot build a `Compile::Fragment`.** It is the adapter's
   type and the registry never looks inside one. Step 4's "the registry wraps
   the bridge into the fragment" therefore needs a second hook per shape, and
   step 3's `GenerationPlan<C: Compile>` needs the adapter's `Compile` type to
   be lifetime-free, which costs more than deleting `Representation` saves.
2. **The declaration applier runs before the recipe table exists.**
   `declare_into` runs it, and `recipes()` is built afterwards *from the plans it
   produces*. So step 6's "readers of `Product` recipes" has nothing to read;
   what is duplicated is the derivation, and one shared derivation fixes it.
3. **Site enumeration is not language-neutral.** `prebindgen-c` skips a `()`
   return because C has nothing to hand back there, and `prebindgen-jni` plans
   one because the JVM does; JniGen answers a callback parameter whole, and has
   synthesized const-getter sites C has no equivalent of. Step 7's "the registry
   compiles every site" holds only with a per-adapter rule for each.

One of #675's assumptions was wrong and is now fixed: the registry **can** be
told which recipe a site takes, by asking. `Compile::site_recipe` is on the
branch `feat/676-step7-registry-owns-the-plan`, beside a `Compile::plan` that
derives its per-site context from the `Bound` rather than carrying it. Neither
shipped, because on their own they move lines without moving the first figure.
