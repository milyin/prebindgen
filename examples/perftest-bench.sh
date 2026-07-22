#!/usr/bin/env bash
#
# Build, run, and compare the three `perftest` micro-benchmarks of the same
# `perftest-flat` API:
#
#   * Rust   — native, no FFI            (perftest-flat/examples/perftest.rs)
#   * C      — generated C ABI           (perftest-c, via cbindgen + cmake)
#   * Kotlin — generated JNI binding     (perftest-kotlin, via JniGen + gradle)
#
# All three emit the same normalized block:
#
#   BEGIN_PERFTEST lang=<rust|c|kotlin> n=<N>
#   <op> <variant> <ns_per_op> <mops>
#   END_PERFTEST
#
# Every op runs in two string categories — `.null` (no heap `label`) and `.str` (a
# real `label` string) — so the comparison is apples-to-apples. This script runs all
# three at one iteration count `N`, then prints, per category, a table of the BEST
# ns/op per operation (the single-payload put / get / callback and the whole-batch
# put_vec / get_vec / callback_vec) per language, followed by each language's full
# report. The vector ops process a batch of VEC_N payloads per call (ns reported per
# call); they run N / VEC_N iterations. The Kotlin report additionally compares a
# nested 64-Long input recursively flattened into JNI scalars (`large_flat`) with
# the identical shape passed as one `JObject` (`large_obj`).
#
# Usage:
#   examples/perftest-bench.sh            # full run (N = 5,000,000, VEC_N = 16)
#   examples/perftest-bench.sh --quick    # fast smoke (N = 100,000)
#   examples/perftest-bench.sh --n 20000000
#   examples/perftest-bench.sh --vec-n 64 # vector batch size (default 16)
#
# Requires: a Rust toolchain (always). C column needs cmake + a C compiler;
# Kotlin column needs a JDK (Gradle's toolchain). A missing toolchain is skipped
# with a warning — the merged table still renders the available columns.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)" # examples/.. = workspace root

N=5000000
VEC_N=16
while [ $# -gt 0 ]; do
    case "$1" in
        --quick) N=100000; shift ;;
        --n) N="${2:?--n needs a value}"; shift 2 ;;
        --n=*) N="${1#--n=}"; shift ;;
        --vec-n) VEC_N="${2:?--vec-n needs a value}"; shift 2 ;;
        --vec-n=*) VEC_N="${1#--vec-n=}"; shift ;;
        -h | --help)
            awk 'NR>=3 && /^#/ { sub(/^# ?/, ""); print; next } NR>=3 { exit }' \
                "${BASH_SOURCE[0]}"
            exit 0
            ;;
        *)
            echo "unknown argument: $1 (try --help)" >&2
            exit 2
            ;;
    esac
done
export PERFTEST_N="$N"
# Batch size for the vector ops (put_vec / get_vec / callback_vec). Rust & C read the
# env var; Kotlin gets it via `-PperftestVecN` (the Gradle daemon ignores the env).
export PERFTEST_VEC_N="$VEC_N"

have() { command -v "$1" >/dev/null 2>&1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
RUST_OUT="$TMP/rust.txt"
C_OUT="$TMP/c.txt"
KT_OUT="$TMP/kotlin.txt"
KT_ERR="$TMP/kotlin.err"

echo "==> perftest benchmark (N = $N)"

# ── Rust (native) ────────────────────────────────────────────────────────────
echo "--> Rust (native, no FFI)"
if (cd "$ROOT" && cargo run --release -q -p perftest-flat --example perftest) >"$RUST_OUT"; then
    :
else
    echo "    Rust runner FAILED" >&2
fi

# ── C (generated C ABI) ──────────────────────────────────────────────────────
if have cmake && { have cc || have clang; }; then
    echo "--> C (generated C ABI)"
    # Explicit cargo build first: cmake's cargo_cdylib custom command doesn't track
    # Rust source changes, so a stale dylib would otherwise be linked.
    (cd "$ROOT" && cargo build --release -q -p perftest-c)
    cmake -S "$ROOT/examples/perftest-c" -B "$ROOT/examples/perftest-c/build" >/dev/null
    cmake --build "$ROOT/examples/perftest-c/build" >/dev/null
    if ! "$ROOT/examples/perftest-c/build/perftest_bench" >"$C_OUT"; then
        echo "    C runner FAILED" >&2
    fi
else
    echo "--> C: SKIPPED (need cmake + a C compiler)"
fi

# ── Kotlin (generated JNI) ───────────────────────────────────────────────────
if have java; then
    echo "--> Kotlin (generated JNI)"
    # Gradle prints its own logs to stderr (suppressed with -q); the app's block
    # rides stdout. `-PperftestN` becomes `-Dperftest.n` for the forked app JVM.
    if ! (cd "$ROOT/examples/perftest-kotlin" &&
        ./gradlew -q --console=plain run -PperftestN="$N" -PperftestVecN="$VEC_N") >"$KT_OUT" 2>"$KT_ERR"; then
        echo "    Kotlin runner FAILED" >&2
        sed 's/^/    /' "$KT_ERR" >&2
    fi
else
    echo "--> Kotlin: SKIPPED (need a JDK for Gradle)"
fi

# ── Tabulate ─────────────────────────────────────────────────────────────────
echo
FILES=()
for f in "$RUST_OUT" "$C_OUT" "$KT_OUT"; do
    [ -s "$f" ] && FILES+=("$f")
done
if [ ${#FILES[@]} -eq 0 ]; then
    echo "No benchmark output captured — all runners failed or were skipped." >&2
    exit 1
fi
# Index of the last "." in s (0 if none) — splits a variant into <sub>.<category>.
awk -v want_n="$N" '
    function lastdot(s,   i) { for (i = length(s); i >= 1; i--) if (substr(s, i, 1) == ".") return i; return 0 }

    /^BEGIN_PERFTEST/ {
        inblk = 1; lang = "?"
        for (i = 1; i <= NF; i++) if ($i ~ /^lang=/) lang = substr($i, 6)
        order_seen[lang] = 1
        next
    }
    /^END_PERFTEST/ { inblk = 0; next }
    inblk && NF >= 4 && $1 != "" {
        op = $1; variant = $2; ns = $3 + 0
        full[lang] = full[lang] sprintf("  %-12s %-16s %9.2f ns/op  %9.1f Mops/s\n", $1, $2, $3, $4)
        # Split "<sub>.<cat>" (e.g. "by_take.null", "composition.str"); a variant with
        # no "." falls into the catch-all category "-".
        d = lastdot(variant)
        if (d > 0) { subv = substr(variant, 1, d - 1); cat = substr(variant, d + 1) }
        else       { subv = variant; cat = "-" }
        cats_seen[cat] = 1
        key = lang SUBSEP op SUBSEP cat
        if (!(key in best) || ns < best[key]) { best[key] = ns; bestsub[key] = subv }
    }
    END {
        nlangs = 0
        split("rust c kotlin", canon, " ")
        for (i = 1; i <= 3; i++) if (canon[i] in order_seen) langs[++nlangs] = canon[i]
        if (nlangs == 0) { print "No benchmark output captured." ; exit 1 }

        nops = split("put get callback put_vec get_vec callback_vec large_flat large_obj", ops, " ")
        # Apples-to-apples: one table per string category, null-label first.
        ncats = 0
        if ("null" in cats_seen) cattab[++ncats] = "null"
        if ("str"  in cats_seen) cattab[++ncats] = "str"
        if ("64long" in cats_seen) cattab[++ncats] = "64long"
        if ("-"    in cats_seen) cattab[++ncats] = "-"

        catdesc["null"] = "null-label (no heap string \xE2\x80\x94 FFI + ownership cost only)"
        catdesc["str"]  = "with-string (realistic \xE2\x80\x94 includes the label heap alloc)"
        catdesc["64long"] = "nested 64-Long Kotlin\xE2\x86\x92Rust input (flattened scalars vs JObject decode)"
        catdesc["-"]    = "uncategorized"

        for (c = 1; c <= ncats; c++) {
            cat = cattab[c]
            if (cat == "64long") { firstop = 7; lastop = 8 }
            else                 { firstop = 1; lastop = 6 }
            printf "================================================================\n"
            printf " perftest \xE2\x80\x94 best ns/op, %s\n", catdesc[cat]
            printf " (lower is better, N=%s)\n", want_n
            printf "================================================================\n"
            printf " %-12s", "operation"
            for (l = 1; l <= nlangs; l++) printf " %14s", langs[l]
            printf "\n"
            for (o = firstop; o <= lastop; o++) {
                op = ops[o]
                printf " %-12s", op
                for (l = 1; l <= nlangs; l++) {
                    key = langs[l] SUBSEP op SUBSEP cat
                    if (key in best) printf " %14.2f", best[key]; else printf " %14s", "-"
                }
                printf "\n"
            }
            printf "\n winning variant per cell:\n"
            for (o = firstop; o <= lastop; o++) {
                op = ops[o]
                printf "   %-12s", op
                for (l = 1; l <= nlangs; l++) {
                    key = langs[l] SUBSEP op SUBSEP cat
                    if (key in best) printf "  %s=%s", langs[l], bestsub[key]
                }
                printf "\n"
            }
            printf "\n"
        }

        printf "================================================================\n"
        printf " full reports\n"
        printf "================================================================\n"
        for (l = 1; l <= nlangs; l++) {
            printf "\n [%s]\n", langs[l]
            printf "%s", full[langs[l]]
        }
    }
' "${FILES[@]}"
