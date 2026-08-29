#!/usr/bin/env bash
# Rust source lines per crate, split so adapter growth cannot hide in tests.
#
# #613's size gate is "prebindgen-c and prebindgen-jni production Rust line
# counts both decrease", which is only checkable if everyone counts the same
# way. This is that way. It lives beside the other repository scripts and is
# run from the repository root:
#
#     bash examples/line-report.sh              # every crate
#     bash examples/line-report.sh prebindgen-c prebindgen-jni
#
# Three numbers per crate, and every line of every `src/**/*.rs` file is in
# exactly one of them:
#
#   test files  — a file under a `tests/` directory, or named `tests.rs` or
#                 `test_util.rs`. Wholly test support.
#   test items  — `#[cfg(test)]` items inside the remaining files, counted from
#                 the attribute to the item's end. Test support that would
#                 otherwise read as production, which is the number that stops
#                 a move into an inline test module from looking like a
#                 deletion.
#   production  — what is left, and what the gate is about.
#
# Only a bare `#[cfg(test)]` counts as a test item. An item behind
# `#[cfg(any(test, feature = "testing"))]` ships to other crates under that
# feature, so it is production by this rule — `assert_edges_cover_rendered_calls`
# and the shape-enum fence helper are counted that way.
#
# An item's end is its closing `}` at column 0, or its own line when the item
# is a one-liner ending in `;` (`#[cfg(test)] mod tests;`). Both hold for
# rustfmt-formatted sources, which the repository's format gate enforces.
set -euo pipefail

cd "$(dirname "$0")/.."

crates=("$@")
if [ ${#crates[@]} -eq 0 ]; then
  crates=(prebindgen prebindgen-flat prebindgen-registry prebindgen-c prebindgen-jni \
          prebindgen-c-runtime prebindgen-jni-runtime prebindgen-proc-macro)
fi

printf '%-24s %10s %10s %10s %10s\n' crate production "test items" "test files" total

total_prod=0
total_items=0
total_files=0

for crate in "${crates[@]}"; do
  [ -d "$crate/src" ] || continue

  test_files=0
  prod_files=()
  while IFS= read -r file; do
    case "$file" in
      */tests/*|*/tests.rs|*/test_util.rs)
        test_files=$((test_files + $(wc -l < "$file")))
        ;;
      *)
        prod_files+=("$file")
        ;;
    esac
  done < <(find "$crate/src" -name '*.rs' | sort)

  read -r production test_items <<<"$(
    awk '
      FNR == 1 { in_item = 0 }
      {
        if (in_item) {
          items++
          if (in_item == 2) {
            # The item is a one-liner when its first line closes it.
            in_item = ($0 ~ /[;}][[:space:]]*$/) ? 0 : 1
            next
          }
          if ($0 == closer) in_item = 0
          next
        }
        if ($0 ~ /^[[:space:]]*#\[cfg\(test\)\][[:space:]]*$/) {
          items++
          match($0, /^[[:space:]]*/)
          closer = substr($0, 1, RLENGTH) "}"
          in_item = 2
          next
        }
        production++
      }
      END { print production + 0, items + 0 }
    ' "${prod_files[@]}"
  )"

  printf '%-24s %10d %10d %10d %10d\n' \
    "$crate" "$production" "$test_items" "$test_files" \
    "$((production + test_items + test_files))"

  total_prod=$((total_prod + production))
  total_items=$((total_items + test_items))
  total_files=$((total_files + test_files))
done

printf '%-24s %10d %10d %10d %10d\n' \
  TOTAL "$total_prod" "$total_items" "$total_files" \
  "$((total_prod + total_items + total_files))"
