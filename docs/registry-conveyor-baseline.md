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
both adapters. It is judged at the plan's end against this baseline.

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
