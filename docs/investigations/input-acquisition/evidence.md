# Evidence for the conversion-pipeline proposal

Baseline: #898 after merged #903,
`e78f4fd24ffaf85ccb342980f7dc33c377d39c34`. These observations describe the current
implementation at that baseline. The [contract](contract.md) is now implemented;
see the [migration guide](implementation.md). The updated probe verifies all eight
combinations generate, replacing the historical rejection assertions.

## The architectural mismatch comes before the reproduction

The low-level [boxed input helper](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/boxed.rs#L87)
returns separate validation, acquisition and construction fragments. Borrowed
inputs have empty acquisition; their reference is created in construction.
The helper describes resources with `registered = false`.

The declaration compiler's
[`input_named`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/lib.rs#L910) combines those
fragments into `validation`, retains the constructed argument there and clears
later fragments. It sets `registered = true` even for shared borrows. Scalar
conversion can execute early without a resource descriptor at all.

These are different representations of the same operation. A phase name no
longer tells an independent converter author when construction runs. Meanwhile,
the assembler still nests all validation, then all acquisition, then all
construction. It chooses receipt readiness and automatic counting using resource
flags. C's by-value inline-owner leaf is another warning against interpreting
`registered` as a schedule: it marks itself registered/accounted while acquisition
remains in its `acquire` buffer until composition.

An existing [execution-order test](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/generation_tests.rs#L60)
asserts separated phases. The proposal deliberately changes that public contract.
Tests must establish the new operation/lifetime boundaries rather than merely
remove assertions that reject mixed implementations.

### Some existing distinctions represent real lifetimes

The solution cannot simply concatenate code and forget storage scope:

- [`Input::Temporary`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/lib.rs) retains an
  owned value behind a borrowed argument. In grouped input, its storage is
  declared outside optional/selector branches and filled during construction.
- [`record_input`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/record.rs) and
  [`optional_input`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/optional.rs) compose
  fields and active alternatives. A borrowed field's backing owner must survive
  through the enclosing record's use.
- [`sequence_input`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/sequence.rs) opens a
  carrier, binds a row, constructs into final storage and releases row-local
  state. It currently combines phased fragments itself. Release must precede
  propagation of an ordinary row error.
- The JNI [callback-result decoder](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/callback.rs)
  establishes owner protection and supplies cached reads, then composes the
  current input fragments. Its unread-owner cleanup depends on actual progress.

These requirements justify storage/transport operations, not a global
validate/acquire/construct schedule. The proposed storage rewrite needs a
prototype proving explicit backing-storage composition across codec/compiler
crates without duplicating continuations
through alternatives or adding per-row closures.

## Reproduce with the public C preparation and emission APIs

Run from the repository root:

```sh
cargo run --manifest-path docs/investigations/input-acquisition/probe/Cargo.toml
```

The standalone [probe](probe/src/main.rs) has its own Cargo manifest outside the
production workspace, so workspace CI does not build it. This command and its
explicit Clippy check are separate investigation verification. The probe:

1. Parses a Rust source API into `Flat<CSchema>`.
2. Declares opaque `Item`/`Packet` representations and an exported function through
   `Cbindgen`, then calls `prepare()`.
3. Optionally replaces selected parameter slots with `CCodec::raw().input_leaf`,
   completing the normal `finish_input` and `validate_input` hooks.
4. Invokes `prebindgen_c::generate` and parses the generated Rust with `syn`.
   The current fixture expects success for every combination. The table below
   preserves the outcomes observed at the baseline.

This exercises the actual public generation path. It does **not** compile or
execute the emitted C/Rust boundary, measure performance, or demonstrate the
runtime behavior. Current implementation execution is covered separately by the
C/Kotlin boundary fixtures. The following rejections are historical observations.

| Source parameters | Converters | Current `registered` flags | Result |
| --- | --- | --- | --- |
| `&Item, Item` | Both declaration-compiled | `true, true` | Generation succeeds. |
| `&Item, Item` | First low-level | `false, true` | Expected mixed-mode rejection. |
| `&Item, Item` | Second low-level | `true, false` | Expected mixed-mode rejection. |
| `&Item, Item` | Both low-level | `false, false` | Generation succeeds. |
| `&Item, &Item` | First low-level | `false, true` | Rejected despite no moves. |
| `&Item, i64` | First low-level | Only the borrow has a descriptor: `false` | Succeeds despite early scalar construction. |
| `&Item, Packet` | Both declaration-compiled | `true, true` | Generation succeeds. |
| `&Item, Packet` | First low-level | `false, true` | Rejected despite unrelated pointee types. |

All resource descriptors in the probe have `records_consumption = false`.
For compiled inputs, the declaration compiler already inserted counts where
needed and set `registered`; it did not set `records_consumption`.

The shared [call assembler](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/staged.rs#L318)
currently computes, schematically:

```rust,ignore
let eager = resources.iter().any(|r| r.registered);
if eager && resources.iter().any(|r| !r.registered) {
    return Err("cannot mix deferred resource conversions with eagerly constructed inputs".into());
}
```

The condition compares neither source types nor actual pointer values. It does
not distinguish moves from borrows. An existing
[assembler test](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/generation_tests.rs#L787)
checks rejection of mixed owning resource metadata. Removing that condition
alone leaves mixed scheduling and automatic counting inconsistent; it is not
the proposed fix.

## Existing access information and runtime accounting

The adapters already describe native type and access in their wire kinds:

- [`CWire::Pointer` and `CWire::Owned`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/wire.rs) carry
  pointer/inline-owner type, access and storage representation.
- [`JniWire::Handle`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/wire.rs) carries boxed allocation
  type, access and active condition.
- C's [incoming callback leaf](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/callback.rs#L232)
  has an empty `resources` list but increments the consumed count when adopting
  the callback context. Its `CWire::Callback` already identifies that carrier.
  JNI's Java callback reference does not transfer an analogous native owner.

Today, C's `finish_input` separately inspects resources and callback wires to
supply `Acknowledge`. The C emitter uses that request to decide receipt needs.
JNI compares resource metadata with handle wires and also compares receipt
availability with Kotlin's actual consumption projection. The proposal replaces
the resource/setup queries with adapter-owned traversal of existing wires, while
retaining the real Kotlin sources used for locking and invalidation.

### Runtime progress remains necessary at some boundaries

The current wrapper contexts bind `input_consumed` to a local `Cell`. Recoverable
C results return an ownership count; consuming JNI calls encode it in their
receipt array. Callback-result guards/scans use their own invocation-local count
to avoid destroying acquired owners a second time. These are runtime ownership
protocols, not build-time diagnostics.

Ordinary input/field/row takes must follow the same owner traversal as the caller's
receipt handling. Kotlin builds `KotlinInput.consumed` in input-wire order; the
emitter joins parameters in source order. Failure limits that walk by the decoded
prefix. Success invalidates all consumed inputs. For carriers,
`NativeInputCarrier.consume` in
[`native_handle.kt`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/native_handle.kt) walks the
ordered `owners` list, populated while packing in
[`bulk.kt`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/bulk.kt).

`HandleLocks` in [`handles.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/handles.rs) deliberately
keeps consumption separate from allocation-type/access grouping and lock order.
Those existing traversals remain authoritative; type buckets must not reorder
ownership takes.

The current [`TemplateContext`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/emission.rs)
renders named binding markers, including `input_consumed`. It does **not** yet
implement the proposed receipt-update operation or an explicit no-report policy.
That renderer extension would let an aborting boundary emit no updates while
a nested callback-result decoder retains its independently rendered reporting.
This is an implementation requirement, not a capability demonstrated by the probe.

### C recovery and receipt coverage

The current C emitter selects [Result handling](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/lib.rs)
from the source return type. For that branch, `consumes` is derived from the input
`Acknowledge` requests, and [result_function](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/errors.rs)
adds a required ownership output when `consumes` is true. C's `finish_input`
sets acknowledgement for moving resources or callback input wires, including
potential transfers in optional/repeated inputs. Replacing that setup query with
a wire traversal must preserve this potential-transfer rule.

The other branch emits a non-Result wrapper whose safety documentation states
that binding errors and Rust panics abort. It has no recoverable failure receipt.
A Result with no potential incoming owner has no ownership output and cannot
partially transfer an input owner. There is no supported built-in recoverable
consuming category relying on global validation instead of an ownership output.
Custom code omitting metadata today is not proof that such a contract is safe;
the proposed adapter must require a receipt for every recoverable consuming path.

## Source inventory

These are implementation locations, not proposed tracking structures:

| Location | Current responsibility relevant to migration |
| --- | --- |
| [`prebindgen-conversions/src/plan.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/plan.rs) | Phased `InputPlan` storage and duplicate `Resource` metadata; both must change. |
| [`prebindgen-conversions/src/staged.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/staged.rs) | Five input requests, global execution schedule, automatic counts and mixed-mode rejection. |
| [`prebindgen-conversions/src/emission.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/emission.rs) | Renders binding templates; proposed complete conversion/reporting operations need explicit rendering rules. |
| [`prebindgen-conversions/src/boxed.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/boxed.rs) | Boxed-pointer leaf; moves currently rely on enclosing automatic counting. |
| [`prebindgen-declarations/src/lib.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/lib.rs) | Recipe compilation, temporary backing storage and early construction marked registered. |
| [`prebindgen-declarations/src/record.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/record.rs), [`optional.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/optional.rs) | Field/alternative composition and enclosing lifetime requirements. |
| [`prebindgen-declarations/src/sequence.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-declarations/src/sequence.rs) | Direct row construction; currently emits validation, automatic count, acquisition. Preserve grouping while reporting actual takes. |
| [`prebindgen-c/src/sequence.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/sequence.rs) | Specialized grouped leaves and per-leaf counting. |
| [`prebindgen-c/src/owned.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/owned.rs) | Inline owners, empty replacement and tracked by-value transfers. |
| [`prebindgen-c/src/callback.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/callback.rs) | Incoming callback context and callback-return protection/decoding. |
| [`prebindgen-c/src/lib.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/lib.rs), [`errors.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/errors.rs) | Counter binding, receipt policy/delivery and assembler calls for normal/Result wrappers. |
| [`prebindgen-c/src/emission.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-c/src/emission.rs) | C emission context and counter rendering. |
| [`prebindgen-jni/src/handles.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/handles.rs) | Duplicate resource/wire comparison and genuine Kotlin handle-source coverage. |
| [`prebindgen-jni/src/lib.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/lib.rs), [`emission.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/emission.rs) | JNI counter/receipt readiness and delivery, Kotlin invalidation and context rendering. |
| [`prebindgen-jni/src/kotlin.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/kotlin.rs) | Input request forwarding and callback-result cached-read specialization. |
| [`prebindgen-jni/src/callback.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/callback.rs), [`bulk.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/bulk.rs) | Callback-local accounting, protection, cached reads and complete-result composition. |
| [`prebindgen-jni/src/callback_rows.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni/src/callback_rows.rs) | Failure-only unread-owner scans using the consumed prefix. |
| [`prebindgen-conversions/src/generation_tests.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-conversions/src/generation_tests.rs) | Current phase-order/mixed-mode tests to replace with new contract and failure coverage. |
| [`prebindgen-runtime/src/callback.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-runtime/src/callback.rs) | Prefix-aware reclamation, preserving already acquired values. |

`registered`, `records_consumption`, `count` and `active` have users throughout
these paths. Remove their resource-inventory consumers together with the public
queries; do not leave unused support behind or substitute a new effects list.

Independent-converter fixtures also require migration:
[`input_contracts.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/examples/covertest-kotlin/build_support/input_contracts.rs),
[`output_contracts.rs`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/examples/covertest-kotlin/build_support/output_contracts.rs)
and the existing [JNI contract probe](../jni-converter-contracts/probe/src/main.rs).

The JNI runtime audit finds no direct critical-array access in the adapter or
runtime. [`text` and `bytes`](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni-runtime/src/lib.rs#L26)
produce owned `String`/`Vec<u8>` values; string access ends inside the helper.
[Bulk input](https://github.com/milyin/prebindgen/blob/e78f4fd24ffaf85ccb342980f7dc33c377d39c34/prebindgen-jni-runtime/src/bulk.rs#L319) reads a byte-array
region into Rust-owned storage. Those values can span later JNI calls.
This observation concerns current built-ins, not arbitrary custom fragments.

## Limits of this evidence

The source audit and eight-case probe establish the stage mismatch and its
composition restriction. They do not validate the proposed expression storage,
lifetime composition, receipt-marker rendering or cleanup schedule. No new
runtime behavior or speedup is claimed.

Existing boundary suites cover owned inputs, collections, callbacks and failure
cleanup. Their existence does not prove a new schedule correct. Implementation
requires the scope prototype, mixed-converter execution matrix and performance
checks in the [acceptance plan](contract.md#acceptance-and-measurement).
