# Shared output cleanup helpers

This refactors #859 without changing its ownership-transfer policy or runtime library. JNI arrays, native bulk rows and bulk frames read through JNI now use one private cleanup-plan generator. Each resolved shape emits one named module-level function; parent helpers call shared child helpers. Shape identities include physical field order/layout, owned handle types, child identities and carrier mode. Occurrence-local wire names and declaration order do not enter the names.

Flat embedded records use the offsets of the actual whole row. A separately stored collection/frame has its own child helper. Optional/inactive wire slots retain #859’s zero/null/absent encodings and cleanup rules. Resource-free rows emit no helper. The registry checks name collisions and emits definitions in stable name order.

## Generated code

[generation.json](generation.json) records the comparison with #859 (`0736245004376cccd2d704a7d469c9ad86dbffab`). The measured implementation is `839a14f45`; later commits record snapshots/results.

| Generated Rust file | Lines before → after | Cleanup definitions before → after |
|---|---:|---:|
| `examples/covertest-kotlin/generated/bindings.rs` | 92,445 → 90,222 | 81 local → 30 shared |
| `examples/bench-bulk/generated/plain.rs` | 7,330 → 7,265 | 6 local → 3 shared |
| `examples/bench-bulk/generated/bulk.rs` | 7,791 → 7,743 | 4 local → 3 shared |

The Kotlin covertest loses 2,223 lines and 148,283 bytes of generated Rust. The runtime crate is unchanged. The generator gains a private identity/definition registry; this is a reduction of emitted duplication, rather than a claim that handwritten generator line count decreases.

## Successful-call code and allocations

The [syntax-tree checker](verify-callers/src/main.rs) removes local cleanup function declarations, omits the new module-level cleanup definitions and normalizes cleanup function references. After that normalization, all **492** remaining Kotlin-covertest functions and all **31 ordinary + 30 bulk** benchmark functions have identical Rust syntax trees. The comparison excludes cleanup bodies themselves; failure-cleanup behavior is checked separately through the generated JNI boundary.

All **3,535 JVM class/module entries** in the ordinary and allocation-instrumented before/after builds are identical; [class-identity.json](class-identity.json) records their content-map hash. Every generated and handwritten Kotlin source in the two example callers is also byte-identical to #859. Their existing compiled JARs were reused against the newly built native libraries for the complete `-Xcheck:jni` runs.

Native allocation probes use #859’s same driver and fixture: 19 cells, two fresh JVMs per revision, 100 warmup operations and ten counted calls per JVM (76 JVMs, 760 samples). Allocation counts, reallocation counts and requested native bytes are identical before/after in every cell. This covers empty/1/64/512-element flat and nested handle collections in ordinary and callback-return modes, plus scalar and primitive controls. [Full allocation results](counts/results.json), [raw samples](counts/samples.csv).

## Focused latency check

Separate uninstrumented release libraries, the same driver JAR and #859’s protocol: one case per JVM, CPUs 0–1, two active processors, fixed 256 MiB G1 heap, at least two seconds warmup and 500 ms without compilation-time changes, followed by twelve measured batches. Four JVMs per revision for three cases give 24 JVMs and 288 samples. No builds/tests ran alongside timing; all warmup checks passed and all samples were retained. Scores are aggregate ns/operation within each JVM, then median and full range across JVMs.

| Case | N | Before ns/call [range] | After ns/call [range] | Wall change |
|---|---:|---:|---:|---:|
| `control` | 1 | 13.4 [12.6–13.8] | 13.0 [12.6–13.1] | -2.42% |
| `fold_nested` | 64 | 387,428.6 [381,219.3–399,856.4] | 387,052.6 [382,603.7–492,324.7] | -0.10% |
| `nested` | 64 | 417,721.1 [401,333.3–864,777.7] | 407,827.2 [404,198.6–421,518.2] | -2.37% |

The scalar control moves by approximately the same proportion as nested arrays, and all ranges overlap. This check supports no throughput-speedup claim; source-size reduction and unchanged allocation/caller behavior are the established results. Hardware/JVM settings match the parent report (AMD EPYC 7713, Linux x86-64, Rust 1.98.0, Kotlin 2.4.21 and OpenJDK 11.0.29). CPU frequency and other users’ host activity are not controlled. [Latency metadata and per-JVM scores](latency/results.json), [raw samples](latency/samples.csv).

## Verification and review

Workspace tests, Clippy with warnings denied, formatting and the 270-file generated-example snapshot check passed. Both complete Kotlin callers passed against the refactored native code with `-Xcheck:jni`, covering typed errors, callback arguments/returns, optional/nested/mixed output cleanup, exact destruction counts and pending/secondary JVM exceptions. New generator tests cover helper reuse across functions/callbacks, local overrides, differing handle types, mode separation, deterministic ordering, input-only exclusion and collision rejection.

Claude Opus 5.5 reviewed the design through the Claude CLI and the source through Copilot using `claude-opus-5.5`. Source review found no confirmed blockers. Follow-up checks constrained collection lookup to sequence/frame carriers and verified that emitted typed-error converters come from the same function-root list, that all helpers compile in the same generated module, and that read/error/panic isolation follows #859. Flattened-field offsets and JNI/non-JNI mode separation were retained.

## Reproduction

Build release normal/count variants with the parent [prepare.py](../prepare.py) for #859 and the refactor, then use [measure.py](../measure.py) with `--counts --forks 2` and `--cases control,nested,fold_nested --forks 4`. Keep both source trees clean and committed and use fresh output directories. See the saved build metadata for the exact inputs. The syntax-tree checker accepts before/after generated Rust file paths:

```sh
cargo run --manifest-path examples/bench-bulk/jni-output-results/shared-helpers/verify-callers/Cargo.toml -- before.rs after.rs
```
