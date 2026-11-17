# Claude Opus 5.5 review

The final immutable typed-row design was reviewed through GitHub Copilot with model `claude-opus-5.5`, verified in session metadata. The reviewer received the full generator diff, shared decoding/cleanup implementation, stable name generator and actual-boundary tests. No blocking findings were reported.

Applied the final review’s recommendations to include both JVM descriptors and Kotlin field spellings in holder identity, and to expose immutable raw fields with `@JvmField`. Mixed scalar, optional, nested and bulk outputs, constructor/fold failure, native projection failure after an accepted prefix, rejected-row cleanup, reentry and concurrency pass actual Kotlin/JNI tests under `-Xcheck:jni`. The final generator has 162 passing unit tests plus 6 doc tests.

The index is reserved and each row is bound once before decoding. `callback_parameters`, `row_fields` and `kotlin_result_fields` use the same canonical wire traversal, so storage and reconstruction share field order. Primitive casts are removed only at complete type boundaries; required reference/nullability casts retain their existing semantics.

Three earlier reviews concern a measured column-array prototype, which was replaced because small wide rows and growth-boundary cases regressed. Those responses remain in [columns-review.md](columns-review.md); the custom column helpers and growth algorithm are absent from the final implementation. The reviewer did not run tests.

## Final design review

**No blocking findings.** I reviewed only the diff and files you provided. I didn't run any tools or tests, so the points below come from reading the code, not from running it.

**What looks right**
- **Ownership on failure:** If the holder allocation or `ArrayList.add` throws, nothing has been stored and `commit_element` hasn't run, so Rust still owns the rejected row. `add` fails during growth before it stores the element, so the accepted prefix stays consistent. Cleanup is counted with `__pb_processed`, so rows can't be released twice.
- **JVM limits:** The holder constructor has the same parameter-slot budget as `run`. The existing `> 254` check covers it, and a nested class has no hidden outer-instance parameter.
- **Remap:** When it can't safely replace a cast, it leaves the cast in place. That still compiles and only adds a redundant cast. Processing indices in reverse stops `[1]` from matching inside `[10]`.
- **Sharing:** Holders are immutable with no static state, so sharing them across functions and threads is safe.

**Non-blocking, should fix**
1. **Sharing key:** Holder classes are deduplicated by name, and the name comes only from the JVM signature, while the field types come from the Kotlin types. If two wires ever share a descriptor but differ in Kotlin type (for example `String` vs `String?`), the first declaration wins and the second use won't compile. Add the Kotlin types to the identity, or assert that the declaration text matches when reusing a name.
2. **Leftover row reads:** The tests only check that `__pb_fold_row[` is gone. Also check there are no bare `__pb_fold_row` uses other than `.cN`, and confirm the Kotlin for the bulk and nested outputs actually compiles.
3. **Field order:** The sink list switched from `row_fields` to `callback_parameters`. Assert that `output_kotlin`'s indices follow `callback_parameters` order, or the remap can map values to the wrong fields without any error.
4. **No new benchmark:** You have no numbers for this design yet. Rerun the 1/64/65/512/513-row cases, plus a wide row and a reference-heavy row, before claiming it's faster.
5. **Minor:**
   - Reference fields keep a redundant `as T?` cast, which produces warnings and breaks builds that treat warnings as errors.
   - Consider `@JvmField` on the holder fields to skip the generated getters.
   - Empty rows could reuse one singleton instead of allocating a holder per row.
