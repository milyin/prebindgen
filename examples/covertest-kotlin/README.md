# covertest-kotlin

A Kotlin/JNI binding for [`perftest-flat`](../perftest-flat) whose goal is to
**use every feature** of prebindgen's [`lang::JniGen`] adapter and **verify each
one with `check(...)` asserts**. It is a correctness/coverage test, not a
benchmark.

| | `perftest-kotlin` | `covertest-kotlin` |
|---|---|---|
| Goal | performance | feature coverage |
| Surface | the lean perf API, mapped in the performance-optimal shape | the *same* flat library — including the coverage-only items in `perftest_flat::ext` — mapped through the full adapter surface |
| Verifies | throughput | behavior, via Kotlin `check(...)` |

The shared Rust library lives in `perftest-flat`. The coverage-only items
(`Priority`, `Freshness`, `Stamp`, `Annotated`, `StorageError`, `Summary`,
`Archive`, `StorageHandler`, `Millis`, and the analytics/shape functions) are
additive and opt-in: they sit in `perftest_flat::ext` and are only pulled in by
a binding that declares them, so `perftest-c` / `perftest-kotlin` are
unaffected.

## Running

```sh
cd examples/covertest-kotlin
./gradlew run
```

`./gradlew run` first runs `cargo build --release -p covertest-kotlin`, which
re-runs `build.rs` to regenerate both sides of the binding
(`src/generated_bindings.rs` and `kotlin/generated/**`), then compiles and runs
the Kotlin asserts. Expected output ends with:

```
PASS - 21 sections, every JniGen feature exercised
```

(One section deliberately provokes callback exceptions; the stack traces it
prints on the way are expected output, not failures.)

Requires a Rust toolchain and a JDK (Temurin 21 is configured via the Gradle
toolchain). The native library is loaded from `target/release` via
`java.library.path`.

## Layout

| Path | Hand-written? | Purpose |
|---|---|---|
| `build.rs` | yes | **The centerpiece.** Drives `JniGen` through every feature; its module doc-comment holds the authoritative coverage matrix. |
| `src/lib.rs` | yes | `include!`s the generated Rust wrappers + the hand-written `PayloadVecHandler::freePtr` (see below). |
| `kotlin/.../Test.kt` | yes | The assert harness — `main()` with one `section { … }` per feature group. |
| `kotlin/.../NativeLibrary.kt` | yes | Native-library loader invoked by `jni_native_init`. |
| `kotlin/.../PayloadVecHandler.kt` | yes | The typed handle for the `suppress_kotlin_code` ptr_class (see below). |
| `kotlin/.../model/Freshness.kt` | yes | The enum class for the `suppress_kotlin_code` enum_class (see below). |
| `src/generated_bindings.rs`, `kotlin/generated/**` | no — regenerated each build | The generated Rust JNI wrappers and matching typed Kotlin classes. |

## Feature coverage

`build.rs` exercises every public `JniGen` builder method. See its doc-comment
for the full table; in brief:

- **config:** `source_module`, `package_prefix`, `package` (subpackages
  `model` / `errors` / `analytics` / `storage`), `jni_native_init`, all six
  name-mangle closures.
- **types:** `data_class` (`Payload`; nested/Option-field `Annotated`),
  `ptr_class` (`Storage` / `Summary` / `StorageError` / `Archive` / handlers),
  `enum_class` (`Priority`; suppressed `Freshness`), `value_class` (`Stamp`),
  `kotlin_type` (`Millis` → `Long`).
- **members:** `accessor` / `method` / `constructor`.
- **flatten:** `flatten_input` / `flatten_output` (+ `variant`/`variant_self`,
  `field`/`field_self` — the latter delivering the owned `StorageError` handle
  to `onError`) and the per-fn overrides `flatten_input_suppress` /
  `flatten_output_suppress` / `flatten_input_with` / `flatten_output_with`.
- **per-fn:** `fun`, and `name` (renames `millis_add` → `addMillis`).
- **wrappers:** `input_wrapper` / `output_wrapper` (`Millis` ⇄ `Long`).
- **type mappings:** primitives, `String`/`&str` (incl. a bare `String`
  return), `Option<T>` (param / return / **field**, incl. `Option<enum>` in
  all three positions and `Option<Payload>` in both directions),
  `Vec<T>`/`&[T]`, `Vec<String>`, `Vec<value_class>` → `List<ByteArray>`,
  `Vec<handle>` / `Option<Vec<handle>>` (Kotlin-side handle fold),
  borrowed-opaque returns (`Option<&T>` → cloned owned handle),
  `Result<_, E>` → `onError`, enums, value classes, `impl Fn` callbacks
  (single + slice + **owned-handle**), N-ary sorted handle locking (3 handles,
  hammered from 4 threads), the `je != null` binding-error channel (malformed
  value-class bytes), the callback no-throw contract (exceptions described +
  cleared per upcall), and per-upcall local-frame hygiene (20k-upcall
  pressure run).

The asserts are grouped into these sections (run order):

1. `data_class Payload`
2. `enum_class Priority`
3. `value_class Stamp`
4. `Option<i64> payloadLabelLen`
5. `Storage members + Option/Vec round-trips`
6. `constructor Storage.withPayload`
7. `callbacks (impl Fn single + slice)`
8. `flatten_output (default / suppress / with)`
9. `flatten_input (default / with), leaves + handle`
10. `Result error channel storageTryWithLabel`
11. `input/output wrapper Millis -> Long (+ .name rename)`
12. `Vec<Storage> handle fold (storageShards / storageShardsOpt)`
13. `owned-handle callback (impl Fn(Storage))`
14. `nested data_class Annotated + Option fields`
15. `borrowed-opaque output archiveLatest`
16. `Vec<String> storageLabels + Option<Payload> input + String return`
17. `enum_class suppress_kotlin_code Freshness`
18. `binding error je != null (malformed Stamp bytes)`
19. `callback exceptions are swallowed (no-throw contract)`
20. `3-handle locking + 2-thread smoke`
21. `high-volume callback (localref pressure)`

## Configuration toggles

- **`suppress_kotlin_code`** — *exercised in both of its forms.* On the
  ptr_class `PayloadVecHandler` it drops **both** the generated Kotlin class
  and the generated Rust `freePtr` destructor extern, so both are hand-written
  (`kotlin/.../PayloadVecHandler.kt` and the `freePtr` extern in `src/lib.rs`).
  On the enum_class `Freshness` it drops the generated Kotlin `enum class`,
  hand-written as `kotlin/.../model/Freshness.kt` (wire-compatible `value` +
  `fromInt`). This is the flag's intended use: take over the type by hand
  while still generating the surrounding wrappers.
- **`disable_handle_locks`** — *kept at its default (locks ON).* This is a
  global, binary toggle: a single binding can only be in one lock mode.
  Keeping the default covers the richer `withSortedHandleLocks` scaffold that
  the generated wrappers emit (and that this example's handle round-trips and
  the 4-thread smoke exercise); calling `disable_handle_locks()` would simply
  omit it.

[`lang::JniGen`]: https://docs.rs/prebindgen
