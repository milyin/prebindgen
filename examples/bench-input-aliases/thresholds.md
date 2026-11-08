# Configurable native alias-history threshold

Native input validation retains addresses already used by a call. The threshold
controls when an incremental sorted vector may become an exact hash index. It
counts accumulated addresses plus the incoming group’s potential size before
deduplication, not the outer array length. Every setting enforces
the same alias rules; changing the threshold does not disable checks.

The first batch reuses its sorted vector even above the threshold. Disjoint
ascending additions also retain the vector, because appending them requires no
movement of accepted addresses. This fixes the ascending-groups regression in the
[earlier complete-call study](remaining-paths.md). A hash index is considered only
when a later addition may require movement in a sufficiently large history.
Shared-only inputs, fixed inputs and disabled C checks use other paths.

## Configuration

The threshold is compiled into generated Rust. There is no per-call settings
lookup or extra runtime threshold field. Existing `DynamicInputs::default()`
uses `DEFAULT_INPUT_ALIAS_INDEX_THRESHOLD`; direct runtime users may select
`DynamicInputsWithThreshold::<N>::default()`.

C supports a project default and an override for the last declared function:

```rust
Cbindgen::builder()
    .set_input_alias_index_threshold(256)
    .function(fun!(accept_packets))
    .input_alias_index_threshold(usize::MAX)
```

The function override also applies when Rust decodes results from callbacks
registered by that function, including callbacks invoked later. `usize::MAX`
retains sorted vectors using the target platform’s maximum; zero allows immediate indexing when an addition needs
movement. Neither setting changes alias validity or ownership. Explicit numeric settings
must fit the target platform’s `usize`; otherwise compilation rejects them.

JNI supports a native-history setting:

```rust
JniGen::builder()
    .set_kotlin_input_preflight(false)
    .set_native_input_alias_index_threshold(256)
```

This covers native-checked arguments and callback results. Default JNI performs
preflight in Kotlin and normally omits native history, so this setting does not
change its locking, generated Kotlin or ABI. Conservative native-check fallbacks
still use the setting.

Low-level `CModule`, `CFunction`, `CContext`, `JniModule`, and `JniContext` now have
an optional `input_alias_index_threshold` field. `None` inherits the surrounding
policy or runtime default. Custom rendering contexts may continue using the
compatibility constructor macro, whose fallback uses the runtime default.

## Default decision: retain 128

The revised runtime keeps **128 accumulated addresses** as its default. This is a
workload compromise, not a universal optimum. Raising the limit does not fix
ascending inputs: keeping their append path does. The full-call sweep compares
64, 128, 256, 512, 1,024 and vector-only (`MAX`) on the same revised runtime.
`old128` means the previous hybrid before the append fix, not the #824 baseline.

Complete C-call medians, microseconds, for shuffled groups of eight:

| Handles | Limit 64 | 128 (default) | 256 | 512 | 1,024 | Vector only |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1.69 | 1.68 | 1.70 | 1.68 | 1.70 | 1.72 |
| 128 | 4.78 | 3.95 | 3.96 | 3.97 | 3.94 | 4.24 |
| 256 | 9.33 | 10.41 | 11.14 | 11.01 | 11.47 | 12.00 |
| 512 | 18.07 | 19.52 | 22.52 | 32.70 | 33.18 | 36.71 |
| 1024 | 36.30 | 37.00 | 39.95 | 54.52 | 96.77 | 106.00 |
| 8192 | 582.73 | 597.48 | 593.20 | 606.30 | 683.51 | 4907.21 |

At 128 handles, limit 64 is 20.8% slower than 128 in paired forks and adds two
allocations. At 512 handles, 256 and 512 are 15.5% and 67.6% slower than 128.
Limit 64 wins some larger cases, but choosing it globally would expose smaller
calls to promotion overhead. Raising to 256 buys little on these small cases and
delays useful indexing. Applications dominated by a different shape can override
individual functions rather than changing the general default.

Preserving ascending append removes the previous large regression:

| Ascending groups of eight | Previous hybrid, limit 128 | Revised, limit 128 | Revised, vector only |
|---|---:|---:|---:|
| 128 | 1.81 | 1.92 | 1.78 |
| 512 | 16.85 | 7.63 | 7.31 |
| 8192 | 588.94 | 209.99 | 198.82 |

The revised default is about 5% slower than vector-only for the listed 512/8,192
ascending cases; retaining the promotion decision still has a cost. At 128
ascending handles the measured difference is about 8%. The large shuffled case
instead favors the default by about 8.25× in paired forks. No setting wins every
cell. Shared-only and first-batch controls have the same allocations across limits;
the CSV retains their timing variation rather than attributing it to indexing.

Ascending groups have exactly the vector-only allocation counts at every tested
limit. At 8,192 handles they make 2,049 allocations and 10 reallocations, requesting
483,072 bytes cumulatively. Shuffled groups at limit 128 make 2,056 allocations and
4 reallocations, requesting 777,840 bytes. These count the complete C call, not
peak live memory. All 648 count rows were repeated with an explicit library search
path and loaded-library assertion; every observation agrees. Deliberate aliases
are rejected for all six limits in arguments and callback results.

Four balanced pinned-process forks per cell, seven measured batches targeting
8 ms, three calibration and three warmup batches, release Rust 1.98 on Linux
x86-64. All six shapes run at six sizes in three pointer orders: 108 cells per
limit, plus an original-128 reference. Counts use separately instrumented
libraries and five identical observations per cell. The host has two available
CPUs without interrupt isolation; fork spread is retained, and small effects are
not a promise of improvement. Some large-call forks have substantial outliers.
The threshold recommendation rests on reproducible 15–68% differences in the
mid-sized grouped cases, alongside the small-input allocation cost.

[Complete timing summaries](threshold-c-latency.csv) include every control,
per-fork medians, spread and paired ratios (candidate/reference time, lower is
better). [Allocation summaries](threshold-c-allocations.csv) and
[provenance](threshold-provenance.json) retain settings, source revisions and
frozen-artifact hashes. Raw samples and logs remain outside Git. The earlier
validator sweep also covers prefix-plus-array transitions; a 128-row array with
16 prior accesses can still pay late promotion. Such functions can select 256 or
`MAX`; this API does not promise to eliminate every transition cost.

## Native-checked JNI confirmation

The 128/256 comparison uses the same Kotlin JAR for both frozen native libraries.
All Kotlin sources match; generated Rust differs only in the configured history
constructor. Both libraries explicitly retain native checks and have allocation
instrumentation disabled. Four balanced fresh-JVM pairs use fixed 256 MiB heaps,
G1, `-Xss16m`, eight warmup and nine measured batches targeting 15 ms. The larger
stack is needed by the existing recursive Kotlin lock helper. No compilers or
other benchmark runs execute concurrently.

| Handles / bulk roundtrip | Limit 128 µs | Limit 256 µs | Paired 256/128 time |
|---|---:|---:|---:|
| 128 / singleton-list argument | 42.41 | 41.67 | 0.998 |
| 128 / singleton-list callback result | 43.23 | 43.83 | 1.025 |
| 512 / singleton-list argument | 156.86 | 164.53 | 1.030 |
| 512 / singleton-list callback result | 166.20 | 169.22 | 1.018 |
| 8192 / singleton-list argument | 2559.45 | 2542.81 | 0.998 |
| 8192 / singleton-list callback result | 2630.25 | 2642.94 | 0.999 |

Raising to 256 shows no consistent benefit here. Paired ratios are medians of
per-fork time ratios, so they need not equal the ratio of displayed medians.
[All JNI cells](threshold-jni-latency.csv), including single-element, first-batch,
flat and primitive controls, retain fork spread and JVM allocation. Some unchanged
controls vary more than the nested cases (the 128-handle flat control is 11.9%
slower at 256); these measurements do not establish an index-related cause.
Default JNI remains covered by Kotlin preflight and normally has no native history.

Reproduce with `examples/bench-bulk/run.py --build-only --native-input-checks
--alias-index-threshold 128` and a second build at 256, then use
`examples/bench-input-aliases/jni.py --policy native --compare-thresholds` with those
`--before`/`--after` directories, `--forks 4 --sizes 1,128,512,8192 --warmup 8
--batches 9 --target-ms 15`.

## Reproduction

The standalone validator runner accepts `limit0`, `limit64`, `limit128`,
`limit256`, `limit512`, `limit1024`, `limit2048`, `limit4096`, and `limitmax`.
These import the production implementation with different const arguments.

The complete-call runner builds C bindings for every selected limit, freezes
timing and allocation-counted libraries separately, and measures actual borrowed
arguments and owned callback results:

```sh
python3 examples/bench-input-aliases/thresholds.py \
  --output /tmp/threshold-c-sweep --limits 64,128,256,512,1024,max \
  --sizes 64,128,256,512,1024,8192 --forks 4 --batches 7 --target-ms 8 \
  --reference-build /tmp/original-128-build
```

The optional reference is the original-128 build recorded in the provenance
(`ec1a168d2`); omit it when comparing only the current configurable limits.
Run from a committed, clean checkout. Generated C ABI must match for all variants;
generated Rust may differ only in the history constructor. The same C executable
loads each frozen library and verifies its loaded path and result. Actual pointer
addresses are ascending, descending or deterministically shuffled. Build and count
work is separate from timing. Every fork randomizes cell/variant order and reverses
paired variant order on alternating forks. Raw logs remain outside Git.
