#!/usr/bin/env bash
#
# Run the C behaviour tests against the generated ABIs under AddressSanitizer +
# LeakSanitizer + UndefinedBehaviorSanitizer.
#
# Two programs, two generated surfaces:
#
#   example-cbindgen `c/smoke.c`  — the handle / tagged-union / error-channel /
#                                   owned-return / closure surface, asserted
#                                   value by value.
#   perftest-c `c/perftest.c`     — the by-value `.repr_c_struct()` surface: its
#                                   `correctness()` asserts the five
#                                   parameter-passing semantics. Those asserts
#                                   already existed but only ran when somebody
#                                   benchmarked by hand; here they run at a tiny
#                                   iteration count so CI executes them.
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

echo "== cargo build -p example-cbindgen -p perftest-c (cdylibs + regenerated headers)"
cargo build -p example-cbindgen -p perftest-c

lib_dir="$repo_root/target/debug"

# `-fno-sanitize-recover=all` turns every UBSan finding into a non-zero exit;
# without it an unaligned load or a signed overflow only prints and continues.
sanitizers=(
    -fsanitize=address,undefined
    -fno-sanitize-recover=all
    -fno-omit-frame-pointer
)

# Compile one C program against one generated cdylib and run it under the
# sanitizers: $1 = cdylib crate name, $2 = C source, $3 = include dir.
run_under_sanitizers() {
    local libname="$1" src="$2" incdir="$3"
    local name dylib
    name="$(basename "${src%.c}")"

    case "$(uname -s)" in
        Darwin) dylib="$lib_dir/lib${libname}.dylib" ;;
        *) dylib="$lib_dir/lib${libname}.so" ;;
    esac
    if [[ ! -f "$dylib" ]]; then
        echo "expected cdylib at $dylib" >&2
        exit 1
    fi

    # `-std=c11` is strict ISO, which hides POSIX declarations `perftest.c`
    # needs (`clock_gettime`); `_POSIX_C_SOURCE` puts them back without
    # loosening the language dialect to `gnu11`.
    echo "== compiling $src under ASan/LSan/UBSan"
    "$cc" "${sanitizers[@]}" -g -O1 -std=c11 -D_POSIX_C_SOURCE=200809L \
        -Wall -Wextra -Werror \
        -I "$incdir" "$src" -o "$out_dir/$name" \
        -L "$lib_dir" -l"$libname" -lm \
        -Wl,-rpath,"$lib_dir"

    echo "== running $name"
    ASAN_OPTIONS="detect_leaks=1:detect_stack_use_after_return=1:strict_string_checks=1" \
        UBSAN_OPTIONS="print_stacktrace=1:halt_on_error=1" \
        "$out_dir/$name"
}

echo "== example_flat header variant: $header_arch"
run_under_sanitizers example_cbindgen \
    "$repo_root/examples/example-cbindgen/c/smoke.c" \
    "$repo_root/examples/example-cbindgen/include"

# The benchmark's timings are meaningless under ASan and at this iteration
# count — `correctness()` and the leak detector are what this run is for, so the
# measured block goes to /dev/null and only the asserts (and any sanitizer
# report) decide the exit status.
export PERFTEST_N="${PERFTEST_N:-200}"
run_under_sanitizers perftest_c \
    "$repo_root/examples/perftest-c/c/perftest.c" \
    "$repo_root/examples/perftest-c/include" >/dev/null

echo "PASS - C behaviour tests clean under ASan/LSan/UBSan"
