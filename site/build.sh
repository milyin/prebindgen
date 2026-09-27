#!/usr/bin/env bash
# Builds the docs site into $1: rustdoc for the workspace crates plus the
# static home page. Output uses only relative links, so this same tree works
# whether it's deployed at the site root or nested under pr-preview/pr-<N>/.
set -euo pipefail

out="${1:?usage: build.sh <out-dir>}"

# The published workspace crates from .github/workflows/rust.yml. No examples/*.
crates=(
  prebindgen
  prebindgen-flat
  prebindgen-tools
  prebindgen-c
  prebindgen-jni
  prebindgen-proc-macro
  prebindgen-c-runtime
  prebindgen-jni-runtime
)

args=()
for c in "${crates[@]}"; do
  args+=(-p "$c")
done

cargo doc --no-deps "${args[@]}"

rm -rf "$out"
mkdir -p "$out"
cp site/index.html "$out/index.html"
cp -r target/doc "$out/doc"
# cargo's own build lock, not part of the site.
rm -f "$out/doc/.lock"
