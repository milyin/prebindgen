#!/usr/bin/env bash
#
# Run the C smoke test against the generated `example_flat` ABI under
# AddressSanitizer + LeakSanitizer + UndefinedBehaviorSanitizer.
#
# `c/smoke.c` is the test that exists to prove the ownership contract — every
# arm C receives is C's to release, and the typed drops release exactly the live
# one. Running it without a leak detector meant it could violate that contract
# and still print PASS, which is what happened (#154 review). This is the gate
# that keeps it honest.
#
# What is and is not instrumented: the Rust cdylib is built by a stable
# toolchain and is therefore NOT compiled with `-Zsanitizer=address`, so ASan's
# redzone checks apply to the C side only. LeakSanitizer, however, interposes
# `malloc` for the whole process, and Rust's default allocator on Linux is the
# system allocator — so a block the generated Rust layer hands out and nobody
# frees IS reported, with the generated file and line in the stack. That is the
# property this gate is for.
#
# Usage:
#   examples/smoke-asan.sh
#
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

cc="${CC:-cc}"
out_dir="${TMPDIR:-/tmp}/prebindgen-smoke-asan.$$"
mkdir -p "$out_dir"
trap 'rm -rf "$out_dir"' EXIT

arch="$(uname -m)"
case "$arch" in
    x86_64 | amd64) header_arch=x86_64 ;;
    aarch64 | arm64) header_arch=aarch64 ;;
    *)
        echo "no committed example_flat header for architecture '$arch'" >&2
        exit 1
        ;;
esac

echo "== cargo build -p example-cbindgen (cdylib + regenerated header)"
cargo build -p example-cbindgen

lib_dir="$repo_root/target/debug"
case "$(uname -s)" in
    Darwin) dylib="$lib_dir/libexample_cbindgen.dylib" ;;
    *) dylib="$lib_dir/libexample_cbindgen.so" ;;
esac
if [[ ! -f "$dylib" ]]; then
    echo "expected cdylib at $dylib" >&2
    exit 1
fi

# `-fno-sanitize-recover=all` turns every UBSan finding into a non-zero exit;
# without it an unaligned load or a signed overflow only prints and continues.
sanitizers=(
    -fsanitize=address,undefined
    -fno-sanitize-recover=all
    -fno-omit-frame-pointer
)

echo "== compiling c/smoke.c for $header_arch under ASan/LSan/UBSan"
"$cc" "${sanitizers[@]}" -g -O1 -std=c11 -Wall -Wextra -Werror \
    -I "$repo_root/examples/example-cbindgen/include" \
    "$repo_root/examples/example-cbindgen/c/smoke.c" \
    -o "$out_dir/smoke" \
    -L "$lib_dir" -lexample_cbindgen -lm \
    -Wl,-rpath,"$lib_dir"

echo "== running"
ASAN_OPTIONS="detect_leaks=1:detect_stack_use_after_return=1:strict_string_checks=1" \
    UBSAN_OPTIONS="print_stacktrace=1:halt_on_error=1" \
    "$out_dir/smoke"

echo "PASS - C smoke test clean under ASan/LSan/UBSan"
