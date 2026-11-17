# Shared C inputs without implicit reconstruction

The same Rust `&[i64]` parameter previously decoded C rows into a temporary Rust
Vec. The new binding accepts a typed shared slice view and borrows the caller's
storage. This benchmark compares generated C-to-Rust calls against #824 before
the input change (the source-equivalent #888 checkout identified in provenance).
It does not measure packet throughput or JNI.

## Cases and allocation results

The same fixture builds both versions. Each has a matching compiled C caller:
legacy row structs become direct int64_t elements in the new header. Setup and
C buffer allocation/free are outside measurements. Assertions remain enabled.

- `inspect`: Rust returns the slice's address without traversing elements. C checks
  that nonempty new inputs keep the original address; old reconstructed inputs
  have a different address. One allocation of 8 × length requested bytes becomes
  zero for every nonzero measured length.
- `sum`: Rust reads and sums all elements; C verifies the result. The same one
  allocation/copy disappears, while the required O(length) summation remains.
- `array`: Rust sums `&[i64; 4]`. One allocation/32 requested bytes becomes zero.
- `text`: borrowed `&str`, already borrowing C storage in both versions. No
  native allocation; UTF-8 validation still reads the input bytes.
- `owned`: `String` input. Ownership requires constructing Rust-owned storage;
  one allocation of length bytes remains for nonempty input in both versions.
- `control`: unchanged primitive-only call, with length used only as its argument.
  It neither constructs nor traverses a collection and allocates zero.

Empty inputs allocate zero in both versions, and all reallocation counts are zero.
Five counted observations agree for each cell. At 65,536 i64 elements, borrowing
removes 512 KiB of requested native bytes per call. `allocations.csv` contains all
counts; these are native requested bytes, not retained or JVM memory.

Generated-boundary tests separately cover explicit reconstruction, native Vec/
String references, optional and fixed-array inputs, original pointer identity,
foreign views into live Rust owners, constructors, invalid ranges, and cleanup.
An explicitly reconstructed nonempty slice still allocates once; a nested slice
constructor allocates only its required final owned Vec. A later invalid field
following a consumed temporary reports one consumed handle and drops it once.
Strict-provenance Miri checks the direct view/owner and explicit-copy paths.

## Complete-call timing

Linux x86_64 AMD EPYC 7713 VM, two available CPUs, glibc 2.39. CPU 0 affinity;
Rust release opt-level 3 with debug assertions off; C `-O3`. Four fresh processes
per side, balanced before/after order, shuffled cases, five calibration rounds,
three warmup batches, nine measured batches targeting 10 ms. Calibration growth
and iteration count are bounded; the frozen runner records actual iterations.
The timing library is uninstrumented; allocation counts use separate builds.
A/A runs the identical baseline binary/library under both labels. CPU governor/
turbo settings are unavailable in this VM.

Times are **nanoseconds per complete call**, reported as medians of per-process
batch medians. The ratio is the median paired after/before ratio (lower is
better), which need not equal the ratio of the two displayed medians. All seven
variable lengths (0, 1, 16, 128, 512, 8,192, 65,536), fixed-array results and
per-process medians are in `latency.csv`; `aa-latency.csv` retains identical-library
controls. Provenance contains exact revisions, sources, library/caller hashes,
compiler versions, and settings.

| Case | Length | Before ns | After ns | Paired after/before | A/A ratio |
|---|---:|---:|---:|---:|---:|
| inspect | 0 | 5.95 | 6.64 | 1.1100 | 1.009 |
| inspect | 1 | 23.47 | 8.06 | 0.3425 | 1.002 |
| inspect | 512 | 123.38 | 8.01 | 0.0649 | 1.000 |
| inspect | 65,536 | 14239.43 | 8.07 | 0.0006 | 0.997 |
| sum | 0 | 7.52 | 7.54 | 1.0010 | 1.001 |
| sum | 1 | 23.62 | 8.88 | 0.3748 | 0.991 |
| sum | 512 | 166.47 | 58.02 | 0.3519 | 0.988 |
| sum | 65,536 | 21197.31 | 7518.06 | 0.3479 | 1.016 |
| array | 4 | 23.45 | 6.66 | 0.2815 | 1.000 |
| text | 0 | 12.14 | 12.18 | 1.0007 | 1.004 |
| text | 1 | 12.80 | 12.86 | 1.0082 | 1.034 |
| text | 512 | 22.54 | 23.23 | 1.0192 | 1.012 |
| text | 65,536 | 1573.13 | 1608.82 | 1.0193 | 1.001 |
| owned | 0 | 13.89 | 13.89 | 1.0010 | 1.000 |
| owned | 1 | 26.21 | 26.69 | 1.0092 | 1.006 |
| owned | 512 | 50.49 | 50.49 | 0.9966 | 1.005 |
| owned | 65,536 | 25565.04 | 25703.58 | 1.0058 | 1.018 |
| control | 0 | 3.27 | 2.96 | 0.8986 | 1.001 |
| control | 1 | 3.29 | 2.94 | 0.8996 | 0.999 |
| control | 512 | 3.34 | 2.98 | 0.8977 | 0.995 |
| control | 65,536 | 3.32 | 2.97 | 0.8959 | 0.999 |

The nonempty results reflect removing allocation and copying. The `inspect`
case intentionally avoids a full read, so its large-buffer difference is much
larger than `sum`. Primitive control is about 10% faster despite unchanged
conversion operations; code placement and the VM can affect small-call timing.
Thus exact speedup factors are not universal causal estimates. Text/owned controls
retain their required work and show small changes. Full-read speedups are the
relevant comparison when the Rust function must inspect every element.

## Empty-input slowdown investigation

The first implementation measured empty `inspect` at 6.64 → 8.00 ns and empty
`sum` at 8.31 → 10.23 ns, despite zero allocations on either side. Assembly showed
an outlined `validate_slice` call on the new empty path. That helper immediately
accepts empty descriptors; generation now skips that no-op call and still
normalizes null/misaligned pointers and preserves valid aligned empty pointers.

In the final run, empty `sum` is effectively unchanged (7.52 → 7.54 ns), while
empty pointer inspection still increases by about 0.7 ns (5.95 → 6.64 ns).
This regression is retained explicitly. The new inspection path preserves a
valid empty input pointer, whereas the old empty Vec can use its fixed dangling
pointer. Source/assembly inspection supports this explanation, but no isolated
causal estimate is claimed. Initial results and A/A controls remain in
`initial-latency.csv` / `initial-aa-latency.csv`. The final caller also uses longer
calibration (five rounds instead of three) to reach the requested duration for
very fast cases; the two sessions therefore are not directly paired experiments.

## Reproduce

Run from a clean committed checkout on Linux:

```sh
python3 examples/bench-c-shared-inputs/run.py --before /path/to/parent \
  --output /tmp/c-shared-input-study --build-only
python3 examples/bench-c-shared-inputs/run.py --before /path/to/parent \
  --output /tmp/c-shared-input-study --prepared
python3 examples/bench-c-shared-inputs/run.py --before /path/to/parent \
  --output /tmp/c-shared-input-study --prepared --aa
```

All four libraries freeze before timing. The runner verifies matching fixture
hashes, actual loaded library paths and frozen binaries. Source signatures are
the same; each C caller uses its version's actual header and ownership rules.
Raw samples, build logs and binaries stay outside Git. Committed files contain
only the reproducer, compact measurements and provenance.
