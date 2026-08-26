#!/usr/bin/env bash
#
# Golden-diff check: regenerate every example binding that commits its
# generated output, then fail if regeneration changed (or added) anything.
#
# The committed generated files pin deterministic output. A semantic-preserving
# generator change may intentionally update them (for example, by qualifying a
# Rust path); review that diff once, commit it, then this check proves a second
# generation is byte-identical to the accepted goldens. Pair it with
# `cargo test --all --all-features` (codegen snapshots) and
# `examples/covertest-kotlin$ ./gradlew run` (JVM-runtime behavior).
#
# Usage:
#   examples/regen-check.sh                        # the in-repo bindings
#   examples/regen-check.sh --with-zenoh-flat-jni [path]
#       Also regenerate the sibling zenoh-flat-jni checkout (default
#       ../../zenoh-flat-jni relative to this script) and diff ITS committed
#       generated files. Requires that checkout to be clean in those paths.
#
# The in-repo check requires the `aarch64-unknown-linux-gnu` Rust target because
# example-cbindgen commits target-specific golden Rust files.
#
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

# Committed generated outputs, per example crate.
generated_paths=(
    examples/emitcheck/src/generated_bindings.rs
    examples/covertest-kotlin/src/generated_bindings.rs
    examples/covertest-kotlin/kotlin/generated
    examples/perftest-kotlin/src/generated_bindings.rs
    examples/perftest-kotlin/kotlin/generated
    examples/perftest-c/generated
    examples/perftest-c/include
    examples/example-cbindgen/generated
    examples/example-cbindgen/include
)

echo "== regenerating in-repo bindings (cargo build)"
cargo build --release \
    -p emitcheck \
    -p covertest-kotlin \
    -p perftest-kotlin \
    -p perftest-c \
    -p example-cbindgen

# example-cbindgen commits one artifact per VARIANT (arch × features), and the
# feature set changes the generated surface as much as the arch does. The
# default-feature build above covers one of them; this covers the other. Without
# it the all-features artifacts would be committed but never checked.
echo "== regenerating example-cbindgen (--all-features)"
cargo build --release --all-features -p example-cbindgen

# The host builds above cannot touch the committed aarch64 variants. `check`
# runs the target-aware build script without requiring an aarch64 linker, and
# the three invocations cover every aarch64 feature variant committed here.
echo "== regenerating example-cbindgen (aarch64 variants)"
cargo check -p example-cbindgen --target aarch64-unknown-linux-gnu
cargo check -p example-cbindgen --target aarch64-unknown-linux-gnu --features unstable
cargo check -p example-cbindgen --target aarch64-unknown-linux-gnu --all-features

echo "== diffing committed generated files"
# `git status --porcelain` catches modified AND newly created files (a new
# generated file never shows in `git diff`).
drift="$(git status --porcelain -- "${generated_paths[@]}")"
if [[ -n "$drift" ]]; then
    echo "REGEN DRIFT — generated output changed:" >&2
    echo "$drift" >&2
    git --no-pager diff --stat -- "${generated_paths[@]}" >&2
    exit 1
fi
echo "ok: in-repo generated output matches committed goldens"

if [[ "${1:-}" == "--with-zenoh-flat-jni" ]]; then
    jni_dir="${2:-$repo_root/../zenoh-flat-jni}"
    echo "== regenerating zenoh-flat-jni at $jni_dir"
    (cd "$jni_dir" && cargo build --release)
    jni_drift="$(git -C "$jni_dir" status --porcelain -- src/generated_bindings.rs kotlin/generated)"
    if [[ -n "$jni_drift" ]]; then
        echo "REGEN DRIFT — zenoh-flat-jni generated output changed:" >&2
        echo "$jni_drift" >&2
        git -C "$jni_dir" --no-pager diff --stat -- src/generated_bindings.rs kotlin/generated >&2
        exit 1
    fi
    echo "ok: zenoh-flat-jni generated output matches committed goldens"
fi

echo "PASS - regen check clean"
