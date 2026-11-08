# Type buckets versus one address array

**Keep JniGen's type buckets. C needs cross-type validation; one array is a strong baseline, with exceptions for large inputs in creation order.**
These measurements extend the [call-wide checker study](../call-wide/README.md)
on #824 after #890. They add benchmark fixtures and CI checks; production code
is unchanged.

## Kotlin: buckets have measurable value

The fixture passes four lists of different owning handle types, A/B/C/D. Each
list is expanded to primitive bulk rows by `a_read(&A)` or `a_mut(&mut A)` and
corresponding constructors for the other types. The source function receives
four `Vec<i64>` values and sums them. The shared case also borrows a mutable
`Token` of a fifth type. Handle counts below are the **total across the four
lists**; the shared case has one additional token.

Compare current generated wrappers against one combined `ArrayList<NativeHandle>`:

- Current wrappers sort each type bucket by address, check conflicting accesses
  within that type, then acquire monitors in stable type-ID order.
- The combined-array prototype copies the already-retained handle references
  into one exactly-sized list using indexed loops, sorts it **in place** by
  `(type ID, address)`, checks aliases, and acquires the same monitors in the
  same order. No per-handle tags or boxed sort keys are allocated.

Both implementations pack and encode identical bulk data and call the same
native symbol in the same loaded library. Bulk packing, input snapshots, sorting,
validation, monitor acquisition, JNI, Rust reconstruction and summing are timed.
Fixtures/native handles are created before timing. Every sample checks its sum;
all native owners are closed and the source allocation counter must return to zero.

**Complete JNI call, shuffled input, microseconds per call:**

| Access / total handles | Buckets µs | One array µs | Array / buckets time |
|---|---:|---:|---:|
| Mutable / 8 | 1.91 | 2.01 | 1.05 |
| Mutable / 128 | 11.22 | 19.44 | 1.75 |
| Mutable / 8,192 | 1455.92 | 1988.15 | 1.37 |
| Mutable / 65,536 | 16198.94 | 20582.03 | 1.32 |
| Shared + mutable token / 8 | 2.03 | 2.08 | 1.01 |
| Shared + mutable token / 128 | 11.21 | 18.03 | 1.67 |
| Shared + mutable token / 8,192 | 1487.92 | 1979.99 | 1.35 |
| Shared + mutable token / 65,536 | 16344.49 | 22018.61 | 1.28 |

With creation-order lists, one-array/bucket ratios at 8,192 handles are 1.34
(mutable) and 1.14 (shared); at 65,536 they are 1.30 and 1.19. At one total list
handle the combined prototype saves roughly 2–7%; eight-handle differences are
small, change direction with order, and fall within measured control variation. These tiny-call differences are not a
reason to add a second production strategy.

The combined array **allocates less**, despite being slower: at 8,192 shuffled
mutable handles it allocates 274,040 JVM bytes/call versus 294,824 for buckets
(7% less); at 65,536, 2,127,880 versus 2,281,816 (7% less). These are cumulative
calling-thread JVM allocations, not peak memory; they exclude native allocations.

Grouping avoids type-ID comparisons and virtual `handleType` getter calls inside
the sorting comparator, and sorts smaller same-type runs. Exclusive duplicate
checks can also compare addresses directly. These are plausible causes of the
observed difference, not individually isolated measurements. The result supports
keeping the current grouping for these signatures, not a claim that every possible
combined-array implementation must lose.

## C: cross-type validation changes the trade-off

C's direct opaque pointers can point into another object's storage. In the fixture,
`owner_child(owner)` returns the first field of a `#[repr(C)] Owner`; its `Child`
pointer has the **same address** as its owner. Thus `&mut Owner` and `&Child`
must still conflict under the current exact-address policy. Sorting only within
types would miss this. JniGen owning handles instead point to separate wrapped
allocations; the same independence assumption is valid for its declared types.

The C fixture uses the same four source constructors and sums as JNI. Native
objects and C rows are reused by all candidates within each process.

| Candidate | Validation |
|---|---|
| `current` | Existing checked C generation. |
| `flat` | One reserved address/access vector, one sort and conflict scan. |
| `typed` | Per-type vectors and local sorting/checks only. **Ineligible as general C validation**; conditional cost comparison only. |
| `cross` | Per-type sorting/checks plus a fixed-size tournament merge to detect cross-type conflicts. Skips the merge for at most one nonempty bucket. |

`flat`, `typed` and `cross` use the existing unchecked conversion after their
preflight; pointer/range checks remain. The preflight adds a descriptor/pointer
walk. This compares complete implementations, not isolated sorting algorithms.
The prototypes compare exact addresses, matching the existing checker; they do
not add general byte-range overlap analysis.

**Complete C call, microseconds per call:**

| Access / handles / input order | Current µs | One array µs | Type buckets + cross-check µs | Local-only buckets µs* |
|---|---:|---:|---:|---:|
| Mutable / 8 / creation | 0.437 | 0.209 | 0.455 | 0.284 |
| Mutable / 128 / creation | 2.583 | 1.846 | 3.211 | 0.905 |
| Mutable / 8,192 / creation | 515.810 | 161.746 | 186.585 | 43.952 |
| Mutable / 8,192 / shuffled | 554.869 | 165.120 | 300.104 | 156.815 |
| Mutable / 65,536 / creation | 4760.792 | 2005.725 | 1657.772 | 395.502 |
| Mutable / 65,536 / shuffled | 5154.250 | 2264.430 | 3225.427 | 1965.003 |
| Shared + token / 8,192 / shuffled | 488.887 | 166.231 | 307.369 | 10.238 |
| Shared + token / 65,536 / creation | 1909.877 | 1849.380 | 1613.555 | 197.352 |
| Shared + token / 65,536 / shuffled | 2318.926 | 2049.353 | 3185.556 | 189.767 |

\*Local-only results omit necessary general-C checks. For shared lists plus a
mutable token of another type, this candidate omits alias bookkeeping altogether.
Its speed is not evidence that those checks can safely be removed.

One array beats bucket-and-merge for the measured small/medium and shuffled
multi-type inputs. **At 65,536 creation-order handles, bucket-and-merge wins**:
1,658 versus 2,006 µs (mutable), and 1,614 versus 1,849 µs (shared plus token).
These exceptions show that grouping can help even with cross-type checks. For
the latter shared case, one array is indistinguishable from current validation
(paired ratio 0.985, within the identical-path control range).
Thus type grouping can help C too; its value must include the required cross-type
check, rather than assuming JNI's owning-type independence.

At 8,192 mutable handles, one array needs 5 Rust allocations versus 8 for
per-type vectors; both request 196,608 bytes. Four allocations hold the final
`Vec<i64>` inputs; the rest are validation storage. Existing checked generation
uses 11 allocations and requests 684,080 bytes in this fixture. Allocation
instrumentation runs in separate libraries outside latency measurements.

## Controls, scope and correctness

The primary Kotlin comparison uses **both strategies in one JVM**, the same
objects/lists/native library, equal iteration counts and alternating batch order.
Five fresh JVMs per input order, four warmup batches, eleven measured batches
targeting 30 ms. Tables show medians of fork medians; ratios are medians of paired
fork ratios and need not equal ratios of displayed medians. Unchanged primitive
and single-type paths are controls. Separate-JVM exploratory runs agreed on the
large-input direction but showed unchanged-path variation; they are not the
source of the published JNI table.

Three additional bucket-versus-identical-bucket A/A JVMs per order use 20 ms
batches. The largest deviation of a median ratio from 1 is 6.1%; for shuffled
mutable lists it stays below 0.7%. Do not infer a general tiny-call policy from this study.
The two strategies share JVM helper profiles and monitor state in the paired
comparison; these results describe this workload, not isolated production JVMs.

C uses three fresh process forks, nine measured batches targeting 20 ms,
rotated/alternating candidate order, identical native addresses and identical C
rows. A/A controls map four independent copies of the identical current library,
verify distinct function addresses, and reuse the same inputs. Ratios range
0.972–1.019 for the selected controls. Native functions use 64-byte alignment;
all timing libraries disable allocation counters.

All measurements run sequentially pinned to CPU 1 on an AMD EPYC 7713 VM,
OpenJDK 11.0.29, Kotlin 2.4.21, with no concurrent compilation. JVMs use `-Xbatch`,
`-Xms384m -Xmx384m`, Serial GC and `-Xss64m`. This is a shared VM without CPU
frequency isolation. Each fork uses the same reproducible shuffle permutation;
allocation addresses can differ between forks. CSV `ascending` means **creation
order**, not a verified monotonically increasing native address sequence.

Correctness probes cover repeated mutable handles in every C parameter bucket,
cross-type C aliases, the shared/mutable token conflict, accepted shared repeats,
and real owner/child address equality. Ineligible local-only C checking is never
used to invoke Rust with a known cross-type conflict. Kotlin probes cover shared
repeats, mutable repeats, generated type order, full multi-handle order and
concurrent opposite list permutations. The paired harness exercises both wrapper
implementations and verifies native owners are released.

This is **not a general Kotlin flat alias-checker implementation**. The measured
signatures are all-exclusive or have shared lists and one exclusive token of a
different type. Shared/exclusive uses of the same type, ownership transfer,
nested heterogeneous records, callbacks and contention are outside this comparison.
Keeping the existing `(type,address)` order also retains interoperability with
other generated wrappers. A project-wide change to address-only ordering is a
different alternative and is not measured here.

## Reproduce

From the repository root, with dependencies already fetched and Linux JDK/Kotlin:

```sh
python3 examples/bench-input-aliases/type-buckets/build.py --output /tmp/buckets-build
python3 examples/bench-input-aliases/type-buckets/kotlin.py --build /tmp/buckets-build --kotlinc /path/to/kotlinc
python3 examples/bench-input-aliases/type-buckets/measure.py --build /tmp/buckets-build --output /tmp/buckets-c --only c --forks 3 --target-ms 20
python3 examples/bench-input-aliases/type-buckets/measure.py --build /tmp/buckets-build --output /tmp/buckets-c-aa --only c --aa --forks 3 --target-ms 15
python3 examples/bench-input-aliases/type-buckets/paired.py --build /tmp/buckets-build --output /tmp/buckets-jni --forks 5 --target-ms 30
python3 examples/bench-input-aliases/type-buckets/paired.py --build /tmp/buckets-build --output /tmp/buckets-jni-aa --aa --forks 3 --target-ms 20
```

Compile sequentially and finish compilation before measuring. `--no-count` skips
instrumented native libraries for CI correctness-only builds. CI builds both
Kotlin surfaces and the paired harness, runs alias/order/concurrency checks and
executes the paired JNI boundary; it has no timing gates.

[Full JNI timings](results/kotlin-latency.csv), [JNI A/A](results/kotlin-aa.csv),
[C timings](results/c-latency.csv), [C A/A](results/c-aa.csv),
[C allocations](results/c-allocations.csv), [provenance](results/provenance.json).
Raw samples, compiler output and experiment logs remain outside Git.
