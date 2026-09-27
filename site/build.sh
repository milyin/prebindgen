#!/usr/bin/env bash
# Builds the docs site into $1: rustdoc for the workspace crates plus the
# static home page. Output uses only relative links, so this same tree works
# whether it's deployed at the site root or nested under pr-preview/pr-<N>/.
set -euo pipefail

out="${1:?usage: build.sh <out-dir>}"

# The published workspace crates from .github/workflows/rust.yml. No examples/*.
#
# In dependency order, and documented one at a time: rustdoc links into
# another crate only when that crate's `target/doc/<crate>/` exists before it
# starts rendering. Documented together, a crate races its dependencies, and
# every link and signature type pointing into one that is not written yet
# renders as plain text, without a warning.
crates=(
  prebindgen
  prebindgen-c-runtime
  prebindgen-jni-runtime
  prebindgen-proc-macro
  prebindgen-flat
  prebindgen-tools
  prebindgen-c
  prebindgen-jni
)

rm -rf target/doc
for c in "${crates[@]}"; do
  cargo doc --no-deps -p "$c"
done

site/check-links.py target/doc "${crates[@]}"

rm -rf "$out"
mkdir -p "$out"
cp site/index.html "$out/index.html"
cp -r target/doc "$out/doc"
# cargo's own build lock, not part of the site.
rm -f "$out/doc/.lock"
