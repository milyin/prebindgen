# Claude review of typed JNI object rows

Requested Claude Opus 5.5 through Copilot. Three completed passes reviewed the
production implementation and the follow-up ownership tests. The reviewer read
provided source/diffs; it did not build or run the project. Local/CI verification
is reported separately.

Addressed findings:

- Cleanup helpers now include the exact generated row class/cache identity in
  their key, so a future Kotlin spelling difference cannot reuse another
  class's field IDs even when JVM descriptors match.
- `JniGen::proguard_rules()` supplies keep rules for raw row constructors,
  fields and native registration; the example commits the generated rules.
  Consumer integration is documented. Android R8 execution was not tested.
- Hand-built typed schemas exceeding 254 constructor slots are rejected. Legacy
  `Buffered` layouts retain their decoder. Custom typed writers must construct
  the registered row class; the public metadata documents this requirement.
- Cache registration rejects a different class or descriptor schema, including
  an initialization race. Class references are retained by the native cache.
- Primitive-only rows use one local object reference instead of a pushed local
  frame. Reference-valued rows retain frames.
- Fault injection includes constructor failure before/after a completed prefix,
  array insertion failure, a typed parent failing after its own legacy children
  completed, and a legacy child-array allocation failing after an earlier child.
  The mixed-parent probe checks the exact two-row root prefix.
- The native-worker fixture explicitly transfers its collection to Kotlin and
  closes its owned handle fields in `finally`. The compiled boundary test checks
  200 worker deliveries, no captured assertion failures and zero live resources.

The second pass found no production ownership/descriptor/slot-limit defects
remaining. The final pass requested the worker cleanup and exact prefix assertion;
those changes are included and the full actual JNI/Kotlin boundary suite passed.
No per-call alias or lifetime checks were introduced. The nonempty package prefix
is already validated by the builder.

Review execution metadata:

```json
[
  {
    "pass": 1,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T12:54:16.785Z"
  },
  {
    "pass": 2,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T13:00:13.155Z"
  },
  {
    "pass": 3,
    "model": "claude-opus-5.5",
    "completed": "2026-11-04T13:02:56.901Z"
  }
]
```
