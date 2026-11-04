# Claude review

Claude Opus 5.5 reviewed supplied source/diffs through Copilot in four passes.
The reviewer did not run builds or tests. Local/CI results are reported separately.

Findings addressed:

- Fixed the JNI test probe's result-array size after adding counters.
- Count reservation by its actual cached interface method ID, independently of
  row delivery order; assert reservation precedes every accepted row.
- Verify actual backing-array capacity before the first row; use an explicit
  `@JvmField` collector field for stable test inspection.
- Cover reserve failure before row acceptance with a synthetic IOException,
  checking exact native destruction and consumed-input state. This simulates the
  unwind from reservation failure; it does not trigger a real JVM heap OOM.
- Cover raw proxy dispatch (`reserve` versus `run`), reentry/concurrency,
  typed/native ownership rollback, Some(empty), None and source errors.
- Register the actual shared capacity interface once after library loading;
  pin it so its method ID remains valid. Re-registration checks class identity.
  Interface-method JNI virtual dispatch is valid for both generated collectors
  and custom proxies; the full JVM CheckJNI suite passes those calls.
- Avoid repeated method-name/signature allocation and lookup when reserving.
  Small collections avoid the hint entirely. Per-row signatures/store are unchanged.
- Check one shared initializer/static with multiple callback functions,
  loader-before-registration ordering, and absence when there are no callbacks.
- Include generated keep rules for capacity, collector and sink metadata.
  Conservative whole-type rules follow the existing generated-row policy.
  Android R8 was not run.
- Existing `fail()` preserves pending JVM exceptions. Missing/broken registration
  fails binding initialization, following existing row-metadata behavior.
  OnceLock fits the Rust 1.85 MSRV; no additional edition requirement is introduced.
- Pilot/expanded timing found a short-list cost. The final policy reserves at
  64 rows and above, with 63/64/65 boundary coverage. Metadata cache lookup and
  one reservation upcall occur once per large collection, never per row.

The final pass reported no blockers in the cutoff and tests and requested
verification on the final binary. The actual boundary suite passes that cutoff;
final performance results are recorded beside these notes.

Execution metadata:

```json
[
  {
    "pass_number": 1,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T14:36:39.664Z"
  },
  {
    "pass_number": 2,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T14:49:29.143Z"
  },
  {
    "pass_number": 3,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T15:20:50.697Z"
  },
  {
    "pass_number": 4,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T15:40:17.110Z"
  }
]
```
