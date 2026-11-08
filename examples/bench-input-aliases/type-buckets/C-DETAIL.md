# Why C type buckets have mixed results

The C result depends on three things: whether grouping makes sorting cheaper,
how cross-group conflicts are checked, and how types occupy the address space.
The earlier bucket prototype combined cheap local sorting with an expensive
merge. That is one implementation of bucket checking, not its inherent cost.

This extends the [original comparison](README.md). Production code is unchanged.
All candidates preserve its exact-address conflict policy: equal addresses conflict
if either access is exclusive. Different types can share an address through a
borrowed field, so a type distinction cannot replace the cross-group check.

## Real C calls: allocation history changes the winner

Three fresh process forks, seven batches targeting 8 ms, identical objects and C
rows for all candidates in each process, rotated/alternating order, CPU 1. Every
call verifies its checksum; every fixture releases all its native owners.

**65,536 total mutable handles in four lists; complete call times in µs:**

| Allocation / list order | One array | Original bucket merge | Compact merge | Direct intersections |
|---|---:|---:|---:|---:|
| Fresh interleaved / creation | 1856.9 | 1491.3 | 1323.0 | 642.7 |
| Fresh interleaved / shuffled | 2064.8 | 2955.4 | 2858.4 | 2193.1 |
| Allocated by type / creation | 347.5 | 1171.4 | 1213.9 | 357.1 |
| Fragmented / creation | 2160.3 | 3001.9 | 2891.7 | 2987.1 |

Fresh interleaved allocation makes each list monotonically increasing in native
address: A/B/C/D are allocated in turn. Concatenating the lists creates four
sorted runs, with overlapping address ranges. This is unusually favorable to
sorting buckets. Allocating all A values, then all B values, etc. instead produces
one globally sorted sequence, where one array and direct intersections are close
and both are much faster than the tournament merges.

Fragmentation is reproduced by allocating 2N filler handles, freeing a shuffled
half, creating the N actual handles, then freeing the remaining fillers. Its
creation-order lists have about half their adjacent address pairs descending.
Thus **creation order does not promise sorted pointers** when an allocator reuses
storage. No row sorting/shuffling/fixture preparation is included in call timings.

The direct-intersection candidate improves the favorable case substantially,
but it also loses on fragmented allocation. It is not a universal replacement.

## Where the original prototype spends time

A separate checker-only test uses numeric addresses, never dereferenced. With
65,536 addresses arranged as four increasing, interleaved runs:

- One global unstable sort makes **1,126,836 comparisons**.
- Sorting the four buckets makes **65,532 comparisons**.
- After independently shuffling each bucket, comparison counts are roughly
  1.08 million versus 0.95 million. Most of the sorting advantage disappears.

Separate phase timers, in µs, for the sorted-run case (the original-bucket cross
stage invokes an equivalent extracted merge helper):

| Checker | Sorting | Local duplicate scan | Cross-group check |
|---|---:|---:|---:|
| One array | 1667.0 | 35.6 | — |
| Original buckets | 31.9 | 36.9 | 1199.5 |
| Buckets without local rescan | 35.6 | — | 1192.1 |
| Compact tournament | 35.3 | — | 947.5 |
| Direct intersections | 38.8 | 41.1 | 261.3 |

These are separately instrumented diagnostic stages, not numbers to subtract
from complete native-call latency. Removing the redundant local scan helps little;
**the original merge dominates**. Its eight-leaf tournament updates three tree
levels for every emitted address, even for only four nonempty groups. It stores
optional address/group pairs and maintains positions and winner metadata. None of
the merged output is needed by the Rust function: validation only needs to know
whether a conflict exists.

The compact prototype uses a tree sized to the active groups, simpler sentinel
keys, and one conflict scan covering both local and cross-group duplicates. This
helps the sorted/interleaved case, but remains expensive on other layouts.

Direct intersections sort/check each bucket, skip pairs that are shared-only or
have disjoint address ranges, binary-search singleton pairs, and otherwise walk
two sorted lists with two cursors. That avoids producing a global merged stream.
For four types it can be much cheaper. Its pair scheduling is O(k²), with O(kN)
worst-case scanning, so this result does not justify applying it to arbitrarily
many types.

## Physical type placement matters independently of list order

A controlled numeric-address experiment changes only which type owns each rank
in the address space. All four lists are already sorted in both cases.

| 65,536 addresses / cross-group stage | Regular A/B/C/D address cycle | Random type placement |
|---|---:|---:|
| Original tournament | 1136.9 µs | 1071.3 µs |
| Compact tournament | 962.0 µs | 942.4 µs |
| Direct intersections | 293.1 µs | 1118.1 µs |

For direct intersections, sorted input alone is insufficient: the pattern in
which the two cursors advance matters too. A regular type cycle produces a
regular cursor pattern; random placement removes that regularity. Branch
prediction is a plausible contributor, but hardware counters were unavailable
(`perf_event_paranoid=4`), so no branch-miss/cache attribution is claimed.

This is consistent with shuffled freshly interleaved objects and fragmented objects
can behave differently even though both have unordered input lists. Shuffling a
list changes traversal order; it does not change the type distribution by address.

## Access modes can remove most of the work

Four shared lists plus one mutable token do not need a complete ordered set of
all addresses. The only possible conflict is equality with that token. A direct
scan validates each pointer and compares it to the token before constructing Rust
inputs. Shared duplicates remain valid; a token/shared alias is rejected.

At 65,536 shuffled handles, complete C calls take:

| Check | µs |
|---|---:|
| One array and sort | 1898.3 |
| Original bucket merge | 3168.9 |
| Direct bucket intersections | 1844.5 |
| Specialized token scan | 311.3 |

The token scan allocates **no validation metadata**. The four source `Vec<i64>`
inputs are still reconstructed normally. This specialization is valid only for
one exclusive scalar plus otherwise shared inputs; it is not a general alias
checker or an omission of runtime guarantees.

## Allocations and alternative sorts

At 65,536 mutable addresses, checker-only metadata requests are 1 MiB for both
one array (one allocation) and four buckets (four allocations). Allocation/copy
stages are much smaller than the old merge. Allocation count alone does not
explain the result.

A global stable sort was also measured. It loses in the four 65,536-mutable-handle
rows above, but helps other native cases:

| Interleaved native input | Global unstable sort | Global stable sort |
|---|---:|---:|
| 65,536 shared handles + token / creation order | 1824.9 µs | 884.3 µs |
| 262,144 mutable handles / address-sorted lists | 8702.2 µs | 7063.4 µs |

The mutable/shared difference under comparable layouts is not explained by this
investigation. It may involve generated-code specialization and compiler behavior;
no root cause is established, and these measurements should not select a universal
sort default.

At 1,048,576 numeric addresses arranged as four increasing interleaved runs,
**complete checker-only** times (copy, sort and scan) are 11.2 ms for stable versus
31.6 ms for unstable sorting. Stable requests 24 MiB of metadata versus 16 MiB:
8 MiB extra scratch, a 1.5× total. At 65,536 it requests 2 MiB versus 1 MiB.
Scratch size is not a fixed multiplier. The million-address result is not a
measured million-element full C-call speedup.

## Production direction

Keep one call-wide collection as a baseline, while retaining access-mode and
source-group information. Groups can be ranges in that one buffer; they do not
inherently require separate heap allocations. Prefer a specialized conflict scan
when the declared signature permits it, such as one exclusive token plus shared
lists. For general inputs, preserve cross-type checking and avoid unconditional
bucket-and-merge behavior.

Any adaptive sorting/intersection policy needs further measurements, especially
for many types and reused allocations. Reuse information obtained during required
wire collection; do not add a separate preparatory pass solely to choose a checker.
No production policy or generator default is changed by this investigation.

## Verification and reproduction

The algorithms are checked against the existing exact checker on exhaustive
small inputs, including `usize::MAX`, random mixed-access sets, larger valid sets
and injected cross-group conflicts. Real generated C probes reject duplicates
in every parameter, cross-type aliases and token aliases; accept shared repeats;
and verify owner/borrowed-child equality. Toolchain equality and unchanged
production sources are verified against the frozen baseline.

Stage timings and comparison counters are separate from uninstrumented total
latency. Allocation counters use a separate executable. Fixed-source wire values
and resource checks remain in every generated call. The microbenchmark is built
with `rustc -O`; native libraries use Cargo release, both rustc 1.98.0. The host is
the same shared AMD EPYC VM as the earlier study; CPU frequency is not isolated.
Selected identical-path controls stay within 2.2% of a paired median ratio of 1
(both checker-only and native calls). The in-run mutable `token` candidate uses the same validation code as `flat`:
its paired median ratios range 0.986–1.041, with individual fork ratios
0.888–1.110. That wider control spread also matters; small differences should not
select an implementation. Main and spatial datasets have separate source/executable hashes;
the spatial run adds fixture distributions without changing the checkers.

After following the original fixture build instructions:

```sh
python3 examples/bench-input-aliases/type-buckets/c-detail-build.py --build /tmp/buckets-build --output /tmp/c-detail-build
python3 examples/bench-input-aliases/type-buckets/c-detail-study.py --build /tmp/c-detail-build --output /tmp/c-detail --forks 3 --target-ms 8
python3 examples/bench-input-aliases/type-buckets/c-detail-study.py --build /tmp/c-detail-build --output /tmp/c-spatial --spatial-only --forks 3 --target-ms 10
python3 examples/bench-input-aliases/type-buckets/c-detail-study.py --build /tmp/c-detail-build --output /tmp/c-detail-aa --aa --forks 3 --target-ms 20
```

`--verify-only` runs correctness and generated-boundary smoke checks for CI.
[Compact results and provenance](results/c-detail/) retain fork medians and input
shape evidence. Raw timing batches and logs stay outside Git.
