# Configurable native alias-history threshold

Native input validation retains addresses already used by a call. The threshold
controls when an incremental sorted vector may become an exact hash index. It
counts accumulated addresses, not the outer array length. Every setting enforces
the same alias rules; changing the threshold does not disable checks.

The first batch reuses its sorted vector even above the threshold. Disjoint
ascending additions also retain the vector, because appending them requires no
movement of accepted addresses. This fixes the ascending-groups regression in the
[earlier complete-call study](remaining-paths.md). A hash index is considered only
when a later addition may require movement in a sufficiently large history.
Shared-only inputs, fixed inputs and disabled C checks use other paths.

## Configuration

The threshold is compiled into generated Rust. There is no per-call settings
lookup or extra runtime threshold field. Existing `DynamicInputs::default()`
uses `DEFAULT_INPUT_ALIAS_INDEX_THRESHOLD`; direct runtime users may select
`DynamicInputsWithThreshold::<N>::default()`.

C supports a project default and an override for the last declared function:

```rust
Cbindgen::builder()
    .set_input_alias_index_threshold(256)
    .function(fun!(accept_packets))
    .input_alias_index_threshold(usize::MAX)
```

The function override also applies when Rust decodes results from callbacks
registered by that function, including callbacks invoked later. `usize::MAX`
retains sorted vectors using the target platform’s maximum; zero allows immediate indexing when an addition needs
movement. Neither setting changes alias validity or ownership. Explicit numeric settings
must fit the target platform’s `usize`; otherwise compilation rejects them.

JNI supports a native-history setting:

```rust
JniGen::builder()
    .set_kotlin_input_preflight(false)
    .set_native_input_alias_index_threshold(256)
```

This covers native-checked arguments and callback results. Default JNI performs
preflight in Kotlin and normally omits native history, so this setting does not
change its locking, generated Kotlin or ABI. Conservative native-check fallbacks
still use the setting.

Low-level `CModule`, `CFunction`, `CContext`, `JniModule`, and `JniContext` now have
an optional `input_alias_index_threshold` field. `None` inherits the surrounding
policy or runtime default. Custom rendering contexts may continue using the
compatibility constructor macro, whose fallback uses the runtime default.

## Reproduction

The standalone validator runner accepts `limit0`, `limit64`, `limit128`,
`limit256`, `limit512`, `limit1024`, `limit2048`, `limit4096`, and `limitmax`.
These import the production implementation with different const arguments.

The complete-call runner builds C bindings for every selected limit, freezes
timing and allocation-counted libraries separately, and measures actual borrowed
arguments and owned callback results:

```sh
python3 examples/bench-input-aliases/thresholds.py \
  --output /tmp/threshold-c-sweep --limits 64,128,256,512,1024,max \
  --sizes 64,128,256,512,1024,8192 --forks 4 --batches 7 --target-ms 8
```

Run from a committed, clean checkout. Generated C ABI must match for all variants;
generated Rust may differ only in the history constructor. The same C executable
loads each frozen library and verifies its loaded path and result. Actual pointer
addresses are ascending, descending or deterministically shuffled. Build and count
work is separate from timing. Every fork randomizes cell/variant order and reverses
paired variant order on alternating forks. Raw logs remain outside Git.
