#!/usr/bin/env bash
# Fails when rendered rustdoc holds a link that did not resolve.
#
# rustdoc does not always warn about one: an explicit link whose target does
# not resolve is kept as a plain URL (`href="prebindgen_flat::Flat"`), and a
# link into a crate whose docs are not in place is dropped. Both are visible
# only in the output, so this reads the output.
#
# Usage: check-links.sh <doc-dir> <crate>...
set -euo pipefail

doc="${1:?usage: check-links.sh <doc-dir> <crate>...}"
shift

dirs=()
for c in "$@"; do
  dirs+=("$doc/${c//-/_}")
done

status=0
report() {
  echo "check-links: $1" >&2
  sed 's/^/  /' >&2
  status=1
}

# An explicit link kept as a URL because its path did not resolve.
raw=$(grep -rHoE 'href="[A-Za-z_][A-Za-z0-9_]*::[^"]*"' "${dirs[@]}" --include='*.html' | sort -u || true)
[[ -z "$raw" ]] || report "links whose Rust path did not resolve:" <<<"$raw"

# A `[`Item`]` link rustdoc left as text.
literal=$(grep -rHoE '\[<code>[^<]*</code>\]' "${dirs[@]}" --include='*.html' | sort -u || true)
[[ -z "$literal" ]] || report "link text rustdoc left unlinked:" <<<"$literal"

# A crate with no docs of its own cannot be linked into.
for d in "${dirs[@]}"; do
  [[ -f "$d/index.html" ]] || { echo "check-links: no docs in $d" >&2; status=1; }
done

exit "$status"
