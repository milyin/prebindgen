# Evidence for the input-acquisition proposal

Baseline: #898 after merged #903,
`e78f4fd24ffaf85ccb342980f7dc33c377d39c34`. These observations describe the current
implementation. The [proposed contract](contract.md) is not implemented by this
investigation.

## Reproduce with the public C preparation and emission APIs

Run from the repository root:

```sh
cargo run --manifest-path docs/investigations/input-acquisition/probe/Cargo.toml
```

The standalone [probe](probe/src/main.rs) has its own Cargo manifest outside the
production workspace, so the workspace CI does not build it. The commands above
and its explicit Clippy check are separate investigation verification. It performs these steps:

1. Parse a Rust source API into `Flat<CSchema>`.
2. Declare the opaque `Item` representation and the exported function through
   `Cbindgen`, then call `prepare()`.
3. Optionally replace selected parameter slots with `CCodec::raw().input_leaf`.
   Call the normal `finish_input` and `validate_input` hooks on those leaves.
4. Invoke `prebindgen_c::generate` and verify success or the expected rejection.
   Parse the generated Rust source for each successful case with `syn`.

This exercises the actual public generation path. It does **not** compile or
execute the emitted C/Rust boundary, measure performance, or demonstrate the
proposed behavior. The expected rejections are observations, not the desired
future behavior. Revisit the fixture when the implementation changes them.

The probe varies only converter preparation, except for the last two controls,
which change the second source parameter to a borrow or scalar:

| Source parameters | Converters | `registered` flags on resources | Result |
| --- | --- | --- | --- |
| `&Item, Item` | Both declaration-compiled | `true, true` | Generation succeeds. |
| `&Item, Item` | First low-level | `false, true` | Expected mixed-mode rejection. |
| `&Item, Item` | Second low-level | `true, false` | Expected mixed-mode rejection. |
| `&Item, Item` | Both low-level | `false, false` | Generation succeeds. |
| `&Item, &Item` | First low-level | `false, true` | Expected mixed-mode rejection despite no moves. |
| `&Item, i64` | First low-level | Only the borrow has a descriptor: `false` | Generation succeeds despite the scalar being compiled early. |

All resource descriptors in these cases have `records_consumption = false`.
For compiled inputs, the declaration compiler already inserted the count where
needed and set `registered`; it did not set `records_consumption`. This is one
reason those flags need explanation before an author can compose helpers safely.

## The relevant current control flow

The low-level [boxed input helper](../../../prebindgen-conversions/src/boxed.rs#L87)
returns separate validation, acquisition and construction fragments. It sets
`registered = false`. Borrowed inputs have empty acquisition; their reference
is created in construction. Direct C storage trusts the caller; wrapped JNI
storage performs its existing pointer-encoding checks.

The declaration compiler's
[`input_named`](../../../prebindgen-declarations/src/lib.rs#L910) moves those
fragments into `validation`, retains the constructed value there and clears the
later acquisition/construction fragments. It sets `registered = true` even for
shared borrows. It inserts transfer counts for moves not already accounted for.
Grouped collection inputs have specialized composition to keep row values live.

The shared [call assembler](../../../prebindgen-conversions/src/staged.rs#L318)
currently computes, schematically:

```rust,ignore
let eager = resources.iter().any(|r| r.registered);
if eager && resources.iter().any(|r| !r.registered) {
    return Err("cannot mix deferred resource conversions with eagerly constructed inputs".into());
}
```

The condition does not distinguish moves from borrows. With no such mixture,
the assembler generally nests all validation, then all acquisition, then all
construction fragments around the source call. It selects receipt readiness and
automatic consumption counting using the resource flags.

The C inline-owner by-value leaf is another warning against interpreting
`registered` as a complete schedule: it marks itself registered/accounted while
its acquisition code is still in the `acquire` buffer until composition.

An existing [assembler test](../../../prebindgen-conversions/src/generation_tests.rs#L787)
checks rejection of mixed owning resource metadata. An existing
[execution-order test](../../../prebindgen-conversions/src/generation_tests.rs#L60)
checks the separated phase ordering. The proposed ordering deliberately changes
that contract; updating the tests must document the new behavior rather than
simply deleting their assertions.

## Source inventory

The paths below are implementation locations, not separate registries. Flat
remains the authoritative source/binding model.

| Location | Current responsibility relevant to migration |
| --- | --- |
| [`prebindgen-conversions/src/plan.rs`](../../../prebindgen-conversions/src/plan.rs) | `InputPlan` code storage and `Resource`'s access, registration, counting and activation fields. |
| [`prebindgen-conversions/src/staged.rs`](../../../prebindgen-conversions/src/staged.rs) | Requests, template rendering, execution scheduling, readiness placement, automatic prefix counts and mixed-mode rejection. |
| [`prebindgen-conversions/src/boxed.rs`](../../../prebindgen-conversions/src/boxed.rs) | Shared boxed-pointer leaf; its move acquisition currently relies on parent/assembler counting. |
| [`prebindgen-declarations/src/lib.rs`](../../../prebindgen-declarations/src/lib.rs) | Compiles complete arguments into early code and marks resources registered. |
| [`prebindgen-declarations/src/sequence.rs`](../../../prebindgen-declarations/src/sequence.rs) | Builds rows directly; currently emits `validation`, then an automatic count, then `acquire`. Replace that pre-acquisition count with reporting at each actual take; preserve grouped decoding. |
| [`prebindgen-c/src/sequence.rs`](../../../prebindgen-c/src/sequence.rs) | Specializes grouped row leaves and appends per-leaf counting where needed. |
| [`prebindgen-c/src/owned.rs`](../../../prebindgen-c/src/owned.rs) | Inline-owner acquisition, empty replacement, and already tracked by-value transfers. |
| [`prebindgen-c/src/callback.rs`](../../../prebindgen-c/src/callback.rs) | Incoming callback-context acquisition and callback-return decoding/cleanup. |
| [`prebindgen-c/src/lib.rs`](../../../prebindgen-c/src/lib.rs) | Creates the local counter for non-Result wrappers and invokes the assembler; supplies its name in `CContext`. |
| [`prebindgen-c/src/errors.rs`](../../../prebindgen-c/src/errors.rs) | Creates the recoverable Result wrapper's local counter and writes its ownership count on return. |
| [`prebindgen-c/src/emission.rs`](../../../prebindgen-c/src/emission.rs) | `CContext` renders `input_consumed` as a reference to the wrapper-local counter. |
| [`prebindgen-jni/src/handles.rs`](../../../prebindgen-jni/src/handles.rs) | Native resource/Kotlin handle coverage; currently rejects unregistered resources in `require_preflight`. |
| [`prebindgen-jni/src/lib.rs`](../../../prebindgen-jni/src/lib.rs) | Creates the ordinary JNI counter, receipt-array readiness and output, assembler invocation and Kotlin consumed-handle updates. |
| [`prebindgen-jni/src/emission.rs`](../../../prebindgen-jni/src/emission.rs) | Renders `input_consumed` to that invocation's local cell. |
| [`prebindgen-jni/src/kotlin.rs`](../../../prebindgen-jni/src/kotlin.rs) | Forwards the staged protocol and implements callback-return fragment/cached-read behavior. |
| [`prebindgen-jni/src/callback.rs`](../../../prebindgen-jni/src/callback.rs) | Directly composes callback-result phases; protects or rescans unread owners on failure. |
| [`prebindgen-jni/src/bulk.rs`](../../../prebindgen-jni/src/bulk.rs) | Bulk carrier code and callback cached-read preparation; preserve single decoding under `Convert`. |
| [`prebindgen-jni/src/callback_rows.rs`](../../../prebindgen-jni/src/callback_rows.rs) | Failure-only scans of unread ordinary/bulk callback owners; uses the consumed prefix. |
| [`prebindgen-conversions/src/generation_tests.rs`](../../../prebindgen-conversions/src/generation_tests.rs) | Current phase-order and mixed-mode tests; migrate to the new schedule and expanded failure matrix. |
| [`prebindgen-runtime/src/callback.rs`](../../../prebindgen-runtime/src/callback.rs) | Uses the consumed prefix to avoid reclaiming values already acquired by Rust. |

`registered` is read by the assembler, declaration composition, collection
composition, C row specialization and JNI preflight. `records_consumption` is
read by the counting paths. `count` and `active` are rendered/used by those same
automatic counting paths. The proposed metadata reduction depends on migrating
all of these users, not only the mixed-mode conditional.

The independent-converter fixtures also use this protocol: migrate
[`input_contracts.rs`](../../../examples/covertest-kotlin/build_support/input_contracts.rs),
[`output_contracts.rs`](../../../examples/covertest-kotlin/build_support/output_contracts.rs)
and the existing
[JNI contract probe](../jni-converter-contracts/probe/src/main.rs).

The JNI runtime audit finds no direct critical-array access in the adapter or
runtime. [`text` and `bytes`](../../../prebindgen-jni-runtime/src/lib.rs#L26)
produce owned `String`/`Vec<u8>` values; string access ends inside the helper.
[Bulk input](../../../prebindgen-jni-runtime/src/bulk.rs#L319) reads a byte-array
region into Rust-owned storage. Those lifetimes can span later JNI calls.
This observation concerns the current built-ins, not arbitrary custom fragments.

## Ownership traversal and lock grouping are separate

For ordinary values, `kotlin.rs` builds `KotlinInput.consumed` in input-wire
order. The writer in `lib.rs` joins those statements in parameter order and
limits their visitor by the decoded prefix on failure. For carriers,
`NativeInputCarrier.consume` in
[`native_handle.kt`](../../../prebindgen-jni/src/native_handle.kt) walks its
ordered `owners` list. The bulk writer appends owners to that list while packing
in [`bulk.kt`](../../../prebindgen-jni/src/bulk.kt).

`HandleLocks` in `handles.rs` explicitly keeps those consumption walks separate
from its allocation-type/access grouping and lock order. Preserve these actual
adapter traversals as the authority. The proposed two-type interleaving test
checks Rust takes, Kotlin invalidation and callback cleanup against them.

## Scope of the evidence

The source audit and C probe establish the composition restriction and the
coupling between scheduling and accounting. The invalid prefix execution in the
overview is a counterexample to removing the guard; the probe does not execute
that invalid scenario.

Existing boundary suites cover many owned-input, collection, callback and
failure-cleanup cases. Their existence is not proof that a new schedule is
correct. The implementation needs the explicit mixed-converter failure matrix
and performance checks listed in the [acceptance plan](contract.md#acceptance-and-measurement).
