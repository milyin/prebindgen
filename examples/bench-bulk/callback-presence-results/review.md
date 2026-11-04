# Claude Opus 5.5 review

Reviewed through Copilot CLI with `--model claude-opus-5.5` on 2026-11-04.
The review contained repository source and diff; sharing was authorized by the
user. Claude read the supplied material and did not execute tests.

Three completed review passes covered the scalar carrier, option niches, direct
returns, fallback packets, acknowledgement and failure cleanup. The first request
was slow, so a focused second request was made; both ultimately completed.

- The initial prototype used i32::MIN as its only absent niche. Review confirmed
  correctness but identified fallback Integer.valueOf(MIN) allocation. The final
  implementation instead reserves -1 and explicitly diagnoses nested outer
  options; both 0 and -1 use JVM-cached integer boxes in packet mode.
- A focused review without the complete wrapper generator questioned failure-path
  presence reads and double cleanup. The follow-up reviewed the complete control
  flow and withdrew both concerns: native exceptions, captured binding errors and
  typed domain errors exit before fold materialization/cleanup. Accepted raw rows
  are discarded once by the native-call failure path; success-branch constructor
  and fold failures clean the remaining rows from their own progress position.
- Actual boundary tests include the JNI NewObjectArray counter, absent/empty
  statuses, native conversion failure after an accepted prefix, collector and
  reservation exceptions, constructor/fold exceptions, late drops, original
  exception identity and exact destruction counts. The final sanitizer/CheckJNI
  run passed with -1/0 sentinels.
- The public low-level wire-schema compatibility change is documented: callback
  sequences now require a Selector control instead of Rows.

The final review reported **no blockers**. It suggested an additional end-to-end
fixture forcing packet fallback for optional collections. This PR covers fallback
through generated-plan tests; the raw absent/empty runtime probes exercise the
direct-return path. No additional forced packet fixture was added.
