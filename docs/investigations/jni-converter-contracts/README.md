# Let JNI generation accept independently implemented converters

Investigation and proposed design for the first checkbox in
[#898](https://github.com/milyin/prebindgen/pull/898). Baseline: that PR's
`5afe625b0`, whose source tree matches #808 at `58d6f368c`.
This document and its probe do not change the generators or their runtime code.

The [completed converter implementation](complete-stage.md), built above the
[input-only prototype](input-stage.md), now accepts independent inputs and
outputs. Both probe features compile with the JNI schema. The restrictions and
expected failures below describe the investigation baseline.

## The problem

Flat, the prepared source model, stores a converter for each function parameter
and return value. A converter
describes the values crossing the foreign boundary and emits the Rust code that
translates them. For JNI, the converter also emits Kotlin code to take public
arguments apart or reconstruct public results.

The public interfaces describe those operations, but JNI's generation entry
point accepts only the built-in converter types:

```rust,ignore
pub type Bindings = Flat<JniSchema>; // Input = JniInput; Output = JniOutput
pub fn generate(flat: &Bindings) -> Result<JniGen>;
```

Consequently, a separately implemented converter cannot reach the JNI emitter,
even if its Rust and Kotlin behavior is identical to the built-in behavior.
Implementing `InputConversion` plus `KotlinInputConversion`, or
`OutputConversion` plus `KotlinOutputConversion`, is insufficient.

For example, an integration might need a converter that exposes a Kotlin domain
object and supplies its own packing and Rust decoding. It can implement the
interfaces, but it must still fit its behavior into `JniInput`/`JniOutput` or
change prebindgen-jni. The probe below uses forwarding wrappers instead of such
a domain-specific implementation to isolate the type restriction.

This is an extension limitation. Ordinary declarations, local overrides and
customized built-in conversion plans already work. `JniInput` and `JniOutput`
can be constructed from `InputPlan` and `OutputPlan`; however, their Kotlin
emission is the built-in interpretation of the value tree. There is no supported
injection point for an independent implementation of `kotlin(...)`.

C already accepts independent parameter and return converters through `generate<S>`.
That is also bounded support: the converters must use C's wire descriptions,
metadata and staged emission protocol, which requests Rust fragments for
successive conversion and ownership steps. It does not mean arbitrary foreign ABIs
can be plugged into the C emitter.

## Evidence and reproduction

The [standalone probe](probe/src/main.rs) defines `IndependentInput` and
`IndependentOutput`. Both forward every conversion operation to the built-in
converter. Their schema retains exactly the JNI module, function, type, field
and variant metadata; only the parameter or return converter type changes.

Run these commands from the repository root:

```sh
cargo run --manifest-path docs/investigations/jni-converter-contracts/probe/Cargo.toml
cargo check --manifest-path docs/investigations/jni-converter-contracts/probe/Cargo.toml --features independent-input
cargo check --manifest-path docs/investigations/jni-converter-contracts/probe/Cargo.toml --features independent-output
```

The first command passes. It checks that the independent types implement the
public contracts and instantiates an empty Flat model with their schema. It also
prepares a function with a modified built-in input plan, generates Rust and Kotlin, and parses the
generated Rust syntax. It does not compile or execute that generated JNI boundary.

At the investigation baseline, the next two commands failed separately with Rust error `E0308`:
`generate` expects `&Flat<JniSchema>`, but receives a model with the independent
input or output type. These are expected investigation results, not regression
tests at that baseline. Both commands now pass with the completed implementation.
The probe is outside the production workspace.

The corresponding C positive control is an existing test:

```sh
cargo test -p prebindgen-c --lib generation_tests::emits_custom_slot_code_without_building_a_plan -- --exact
```

It generates a C export using an independently implemented output converter
without constructing an `OutputPlan`.

## Why changing the function signature alone is insufficient

A wire is one primitive, handle or carrier described by a converter. The logical
value tree groups wires into records, alternatives and collections. An ownership
receipt tells Kotlin which owning handles Rust consumed, including a consumed
prefix when conversion fails partway through a call.
Kotlin handle preflight retains input handles, checks for conflicting accesses
and arranges their locks before native conversion uses their addresses.

| Dependency in the JNI emitter | Operation the emitter actually needs |
| --- | --- |
| Fixed `Bindings` in `generate` and several traversal helpers | Inspect source declarations, JNI metadata and converter value trees |
| `input.rust.acknowledgement` in signature selection | Determine whether the native call needs an ownership receipt parameter |
| `input.rust.resources` in handle preflight and native name selection | Obtain resource allocation types and access modes; verify Kotlin snapshots cover them |
| `HandleLocks::new` accepts `&JniInput` | Combine prepared Kotlin handle sources in the existing lock order |
| `JniOutput::native_return` and concrete output methods | Select a native result protocol supported by that converter |
| Concrete callback and error converters stored in JNI metadata | Discover and emit nested boundaries, including those without a direct exported occurrence |

The physical parameter traversal is already generic in
[`group::function_parameters`](../../../prebindgen-jni/src/group.rs).
The shared call assembler already requests Rust fragments through `Conversion`.
The missing work is to make the remaining JNI operations use the same supported
boundary rather than inspect plan storage.

There is a separate output concern. Today `returns::select` recognizes the
built-in result packet by comparing delivery tokens, and `JniOutput` can clone
its plan and replace delivery for a synchronous return capture. Here a packet is
the JVM array carrying result fields; a capture is an internal callback that
stores those fields for reconstruction after the native call. A generic emitter
cannot assume an independent converter permits that replacement. The second
checkbox in #898 tracks redesigning this selection; the first checkbox must
respect the converter's existing protocol while making the entry point generic.

## Recommended fix

Support independent **function parameter and return converters** first. Keep the
current JNI wire model, staged Rust protocol and metadata. Keep `Bindings`,
`JniSchema` and the ordinary declaration builder as convenient defaults.

1. **Accept a schema through the public JNI emitter.** Change `generate` and
   its source/tree traversal helpers to be generic over the converter types,
   retaining JNI-specific metadata constraints. Make `KotlinInputConversion`
   extend `InputConversion`, and `KotlinOutputConversion` extend
   `OutputConversion`, with the existing JNI wire and request types. These
   operations describe both halves of the same foreign conversion. Adding these
   supertraits is a source compatibility change for external implementations
   that currently implement only the Kotlin trait; document the new requirement.

   The intended signature is schematically:

   ```rust,ignore
   pub fn generate<S>(flat: &Flat<S>) -> Result<JniGen>
   where
       S: Schema<
           Module = JniModule,
           Function = Option<JniFunction>,
           Type = JniType,
           Field = JniField,
           Variant = (),
       >,
       S::Input: KotlinInputConversion,
       S::Output: KotlinOutputConversion;
   ```

   A custom schema is selected before parsing/preparation. This proposal does
   not require changing an already prepared model's schema or making every
   builder and codec generic.

2. **Reuse existing ownership operations in a defined order.** Move creation of
   the input `KotlinContext` and preparation of Kotlin inputs ahead of native
   return selection: the current input context has no dependency on that choice.
   Prepare each input once and derive the receipt parameter from its `consumed`
   operations. Include that parameter in the JVM signature budget, select the
   candidate return transport, and construct the final `JniContext`.

   With that context, request resource descriptions using
   `InputFragment::Resources` and acknowledgement code using
   `InputFragment::Acknowledge`. The latter already specifies exactly `()` when
   no receipt is needed. Reject a disagreement with the Kotlin ownership updates
   before emitting the signature or wrapper. This avoids depending on a context
   whose return transport has not yet been chosen.

   Compare resources with the existing Kotlin handle sources and reuse the
   collected summary for preflight and native-name selection. The existing
   resource request renders active-condition expressions but does not filter
   descriptors. Keep that behavior, including descriptors in inactive branches;
   otherwise static resource counts or native symbol names could change.
   Preserve all current generation-time rejection rules. The current protocol
   still uses an exact unit-expression sentinel; redesigning that protocol is
   separate from removing reads of `InputPlan` fields. Do not introduce a public
   plan accessor or request resources again for every generated-code use.

3. **Make permission to select return transport explicit.** Add a narrowly
   scoped function-return capability to `KotlinOutputConversion`, with a
   conservative packet-only default. A provisional operation is
   `supports_function_return(delivery)`, where delivery identifies the existing
   packet, direct return or internal capture protocol. Existing built-in
   converters preserve their selection behavior; they may initially delegate
   this decision to the current private implementation. Existing signature
   limits and typed-error compatibility still constrain selection.

   Initially, independently emitted outputs use the public packet contract:
   `JniContext.return_sink` is absent, and `KotlinContext.result` names the result
   array. That also works for the probe's forwarding output, whose built-in
   delegate leaves delivery unchanged when no capture is selected. Do not treat
   the capability method alone as a sufficient public protocol for independently
   emitted direct returns or captures. Enabling that opt-in requires documenting
   the native result/commit conventions and supplying the selected transport to
   both the Rust and Kotlin contexts. Direct wire types must come from
   `output_outcomes()` and the adapter's field traversal, rather than concrete
   plan storage. That protocol work and removal of delivery-token recognition
   belong to the second checkbox, rather than being hidden in this change.

4. **State the nested-converter boundary explicitly.** Retained source callbacks
   and configured domain-error handlers already store prepared built-in
   converters inside JNI metadata. Keep those artifacts and their behavior in
   this first change, and include them in generic traversal. Independent
   converters inside those artifacts are not part of this initial extension.
   Customizing a callback's Kotlin projection would also require updating its
   generated interface and lifetime/cleanup protocol; that deserves a separate
   concrete use case. Do not claim that making root slots generic makes every
   nested converter replaceable.

This initial root-only proposal did not require a new registry, trait objects,
downcasting or a parallel Kotlin configuration table. The completed implementation
also supports nested callback/error converters through private generation-time
containers; see [the final design](complete-stage.md). Source and binding data remain in Flat.
The new work happens when bindings are generated; it does not require additional
handle locks, native checks, JNI calls or per-call allocation in generated code.
That is a design constraint to verify, not a measured speedup.

An alternative is to document that JNI supports only built-in plans and stop
presenting the Kotlin conversion interfaces as an independent extension point.
That is simpler, but retains the C/JNI asymmetry and prevents integration-specific
Kotlin projections. Accepting arbitrary untyped hooks or exposing `.rust` publicly
would weaken the boundary without solving those problems.

## Compatibility and implementation acceptance criteria

The production implementation inspected at `main` commit `5e149f869` uses a
different registry-based architecture, so it does not establish that #808 lost
this particular trait-based extension point. It remains the reference for
observable binding behavior. In particular, preserve typed domain-error
handlers and the separate binding-error callback, and invoke user result/error
handlers after the native call returns.

An implementation should meet these checks:

- The two independent converter probes generate successfully; an independently
  implemented Kotlin projection is exercised rather than only forwarded.
- A custom packet-only output remains on its chosen protocol. Built-in direct
  returns, captures and wide-signature fallbacks retain their behavior.
- Resource-free, borrowed and consuming handle inputs retain complete Kotlin
  preflight, locking, partial-consumption receipts and failure cleanup. Missing
  allocation/access or snapshot information still fails generation without a
  generator panic. A handle-free converter may omit an empty handle-input list;
  a handle-bearing converter may not. Conditionally gated handle inputs retain
  their static resource counts and native symbol names.
- Ordinary outputs, configured typed errors, retained callbacks, callback
  returns and bulk/non-bulk collections remain in boundary discovery.
- Existing built-in generated artifacts remain unchanged unless a separately
  reviewed behavior correction requires a difference. C/JNI wire mixing remains
  a compile-time error. Preserve C's trust in caller arguments and Kotlin's
  responsibility for checking conflicting JNI handle accesses; extending the
  emitter must not add native checks that repeat Kotlin's work or change C's
  runtime contract.
- The Kotlin covertest compiles and runs a real custom conversion through the
  generated native boundary. Compare allocations and throughput if emitted
  runtime code changes; generation-only refactoring is not a performance claim.

The original investigation PR records this proposal. The subsequent
[implementation](complete-stage.md) addresses the first two checkboxes in #898.

## Source map

- [Flat converter contracts](../../../prebindgen-flat/src/conversion.rs)
- [JNI entry point and call assembly](../../../prebindgen-jni/src/lib.rs)
- [Kotlin projections and concrete JNI converters](../../../prebindgen-jni/src/kotlin.rs)
- [JNI handle preflight](../../../prebindgen-jni/src/handles.rs)
- [Prepared callback/error metadata](../../../prebindgen-jni/src/metadata.rs)
- [Retained source callbacks](../../../prebindgen-jni/src/callback.rs)
- [Native return selection](../../../prebindgen-jni/src/returns.rs)
- [Shared staged ownership requests](../../../prebindgen-conversions/src/staged.rs)
- [Generic C entry point and its independent-converter test](../../../prebindgen-c/src/lib.rs)
