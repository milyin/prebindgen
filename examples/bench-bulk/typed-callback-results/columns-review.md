# Claude Opus 5.5 review

Three passes used GitHub Copilot model `claude-opus-5.5`, verified in session metadata. The reviewer received generator/runtime definitions, the existing shared output decoders and new tests. The final growth revision has no blocking findings.

Addressed: use a reserved row-index name, preserve nullable/longer cast type boundaries, and assert that the old raw-row variable is absent. Existing `Names::name` sanitizes nested-list/byte-array labels; JVM descriptor and column types come from the same closed `JvmType` table. Actual Kotlin/JNI tests compile and exercise these shapes.

Growth was subsequently simplified: each successful column copy replaces its old buffer immediately. Capacity remains unchanged until all columns have grown; size is published after every row field is stored. A failure preserves accepted values but may change internal buffer identities/capacities. The final review validates this weaker, sufficient guarantee and retry behaviour.

The direct JVM growth-failure test injects a NullPointerException in the last copy by temporarily nulling a private field; it does not force heap exhaustion. JNI collector rejection, Rust conversion failure after an accepted prefix, constructor/fold errors and exact handle cleanup are covered separately by actual boundary tests. Native/JVM allocation and latency are measured independently; the review itself did not execute tests.

## Initial review

I found no blocking defect that the supplied code proves. Three issues should be closed before merge, because each could cause a compile failure or silently read the wrong row.

**Should fix before merge**

1. **Medium-high: unsafe class names** (`sequence_callback::capture`, label). The name label is built from `kotlin_type().trim_end_matches('?')`. A reference field such as `Array<Any?>?` becomes `Array<Any?>`, and a qualified type keeps its `.`. If one of the first three fields is a nested list, the class name contains `<`, `?`, `>` or `.`. Unless `Names::name` cleans the label, the generated Kotlin won't compile. Your tests only cover labels that are already safe, such as `Boolean_Byte_Short`. Clean the label, or build it from descriptor letters, and add a test with a `Vec<i64>` or `Vec<u8>` as the first field.

2. **Medium: the loop variable `index` can be shadowed** (`fold`: the column reads, `materialize` and `__pb_discard`). Each remapped read is `__pb_rows.cN[index]`, inside code produced elsewhere. If any nested or bulk output code declares its own `index`, the inner one wins. That compiles with only a warning and reads the wrong row, including in cleanup. Previously the row was bound once to `__pb_fold_row`, so this hazard is new. Use a reserved name such as `__pb_rowIndex`.

3. **Medium: `remap` matches casts by plain text** (`returns::remap`). Replacing `"{old} as {ty}"` also matches the start of `old as Long?`, which turns into `__pb_rows.c0[index]?` and breaks the build. Any other cast spelling stays as is: `as kotlin.Long`, `as Number`, `as? T`, or passing the value as `Any?` or into generic helpers. Those either box again or produce a useless-cast warning, which fails builds that treat warnings as errors. Add a generated-text check that `__pb_rows.c\d+\[...\]` is never followed by `as` or used where `Any?` is expected, or match on token boundaries.

**Lower severity**

4. **Low-medium: element nullability** (`JvmType::kotlin_column`). If any reference type's Kotlin name lacks `?`, `arrayOfNulls<T>(0)` and `copyOf(next)` produce `Array<T?>`, which can't be assigned to `Array<T>`. Always declare the column as `kotlin.Array<{trimmed}?>`.

5. **Low: sharing is keyed by JVM descriptor, but the class is built from Kotlin types.** This is correct only while the Kotlin parameter type depends on the JVM type alone. Assert that, or key on the column types instead.

6. **Low (performance):**
   - Each nested list read now repeats a getter call, an array load and a cast. Copy each column into a local before the loop.
   - `grow` briefly holds old and new copies of every column. That is what makes it atomic, but peak memory is about 3× for wide rows.
   - With no fields, the class still runs empty growth; a plain counter would do.

**Checked and fine**

- **Failure atomicity:** all columns are copied before any is replaced, assignments can't throw, and `size++` comes last. A failure leaves the accepted rows intact. A stale reference may stay at index `size`, but it is never counted.
- **Handles:** a rejected row stays owned by Rust, and accepted rows are committed only after `add` returns.
- **Exception identity:** an `OutOfMemoryError` from `add` stays pending in the JVM, so `binding_failure` skips `bindingError` and the original exception reaches the caller. The `!==` checks are unchanged.
- **Optional/presence checks:** these are now compared unboxed. The old `Any? == 1` comparison would have been wrong for a `Byte` flag.

**Still to test**

- A Rust conversion error after N handle rows were accepted. This checks that discard runs and `__pb_processed` stays at 0 when the native call fails.
- A first-field label containing a nested list.
- A bulk element that includes nested lists.

## Index/cast follow-up

**No blocking findings** in the supplied diff. These are review notes only; I didn't run anything.

**Index hygiene and casts**
- **Non-blocking, unverified shadowing risk.** If a nested materializer can also declare `__pb_rowIndex`, a top-level column read inside its body would use the inner index. Kotlin only warns about that, so it could compile and still read the wrong row. Add an assertion that each fold declares the name only in its main loop and discard loop.
- **Non-blocking: no left-boundary check in `remap`.** The match `__pb_fold_row[0]` would also hit `x__pb_fold_row[0]`. Nested rows look like they use distinct names, so this is just hardening.
- **Test gap.** Tests only check `__pb_fold_row[`. Assert that the bare name `__pb_fold_row` is absent.
- **Harmless leftover casts.** `as Long ?:` and safe calls after a newline keep their casts. That only adds warnings, but it matters if Kotlin builds turn warnings into errors.

**Capture sharing and cleanup**
- Sharing is correct. Names derive only from the JVM signature, and `BTreeSet` removes duplicates.
- The empty-array singletons are safe because `grow` always runs before the first write.
- Rollback is sound: columns are copied first and published afterwards. Rejected rows stay owned by Rust.

**Performance against #824**
- Primitive rows no longer allocate per row. Strings, byte arrays and nested lists still allocate objects, and nested `Vec<Vec<i64>>` values are still boxed.
- Growth starts at capacity 1. With k columns, n rows cost about k·log₂n array allocations, which is worse than `ArrayList`'s initial 10 for small, wide results.
- While growing, each column briefly holds both the old and new arrays, so peak memory is about 3× the old capacity. Up to half the final capacity can be unused, and nothing is trimmed.
- A Rust-side size hint (`reserve(n)` when the length is known) or an initial capacity of 8 would remove most copying.

## Final growth protocol

**No blocking findings.** I reviewed the supplied source only and found no counterexample or unsoundness in this revision.

**Why it holds:**
- **Accepted rows are kept.** `cN = cN.copyOf(next)` builds a new array before it's assigned, so a failed copy leaves that column as it was. A column already replaced has a copy at least as long, holding every row below `size`. `capacity` and `size` stay unchanged.
- **Retry works.** `next` is calculated again from the unchanged `capacity`. Copying a column that already grew to the same length is just a redundant copy.
- **Mixed capacities are safe.** After `grow` returns, every column is at least `next` long, which is more than `size`. Stores can't go out of bounds, and the primitive and typed stores can't otherwise throw.
- **Cleanup can't double-free.** Values past `size`, including a rejected row's leftover slot, are never read by the fold or `__pb_discard`. Rust still owns any rejected row, so nothing is freed twice.
- **Shared empty arrays stay safe.** They are only copied, never written.
- **Early failure is handled.** The accepted marker is published before the first row. A failure on the first row's growth leaves `size == 0`, so cleanup does nothing.

**Test:** making `c2` null fails after `c0` and `c1` have grown to length 2 while `capacity` stays 1. The checks match the intended behaviour, and the retry covers re-copying columns that already grew.

**Non-blocking notes:**
1. A retry re-copies the columns that already grew, which is an extra allocation but correct.
2. The test only fails the last column. A first-column failure is the simpler case and isn't tested, which is acceptable.
3. The test depends on reflection finding exactly one `SequenceRows_*` class with `add(Long, Long, Long)`. It will break if that capture is merged with another or no longer generated, but that wouldn't make the code wrong.
4. "Same unwind path as allocation failure" is only roughly true: it injects a `NullPointerException`, not an `OutOfMemoryError`. The thrown error is the same kind as far as JNI is concerned, so this doesn't matter.
