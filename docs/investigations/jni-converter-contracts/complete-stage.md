# Complete JNI converter contracts

The [input prototype](input-stage.md) established one input-emission path.
The same emitter now accepts independent inputs and outputs:

```rust,ignore
pub fn generate<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
) -> Result<JniGen>;
```

`Bindings` remains the concrete default alias. The declaration builder still
prepares that model, and its existing built-in converters implement the same
public contracts used by independent converters. There is no second renderer.

## Use custom converters in normal bindings

`JniInput::new(my_input)` and `JniOutput::new(my_output)` retain independent
implementations in ordinary occurrence slots. A `configure` hook can install
them before the normal declaration compiler fills the other slots. Consumers
can also choose concrete converter types in `JniSchema<I, O>` from the outset.

The containers also retain converters inside callback and error metadata, where
different occurrences may need different implementation types. Their private
adapters forward the fixed JNI request family to the public conversion traits;
they do not generate conversion code themselves. The containers use `Rc` and
dynamic dispatch **during binding generation only**. Generated libraries do not
depend on these objects and gain no per-call allocations or dynamic dispatch.
The standard `From<InputPlan>`/`From<OutputPlan>` implementations retain private
plan providers, which exercise these same contracts.

## Explicit return delivery

`DeliveryPolicy::Fixed` preserves a converter's own delivery code. Independent
outputs default to this policy. `AdapterSelected` permits the adapter to select
its standard packet, direct return or synchronous capture, subject to JVM
signature limits and error-channel compatibility.

`JniContext.return_delivery` states the selected mode. The adapter requests
`prepare_return_delivery` once for each applicable generated return body and
shares that prepared `Delivery` across its staged output fragments. A converter
advertising adapter-selected delivery must provide that operation. Built-in
plans use `JniContext::output_delivery`, which is also available to independent
implementations.

The shared plan's `emit_with_delivery` operation emits preparation, delivery,
commit and result using borrowed delivery code. It replaces the old whole-plan
clone. Return selection no longer generates a candidate packet and compares its
token strings with stored delivery code. `JniOutput::packet_only()` explicitly
keeps a compatible prepared converter on its packet protocol.

When editing a raw plan's delivery code, set `plan.delivery.policy =
DeliveryPolicy::Fixed` before converting it to `JniOutput`. This preserves the
edited delivery instead of permitting the adapter to prepare its standard one.
The prepared delivery in `JniContext` belongs to the active output converter;
prepare it again before requesting fragments from a different converter.

`KotlinOutputConversion::kotlin_return` receives a `KotlinReturnContext` with the
selected delivery, successful native expressions and any prepared sequence
projection. Its default implementation places the converter's canonical Kotlin
projection in that transport through `context.project(...)`. Both built-in and
custom function outputs use this operation.
When wrapping a converter with a different `kotlin` projection, keep the default
`kotlin_return` implementation or call `context.project` with the wrapper's own
projection. Forwarding `kotlin_return` to the inner converter would instead use
that converter's projection. Forward the delivery policy/preparation operations
as well when retaining its direct/capture capability; the conservative default
uses packets.

Callback-based collection returns use `kotlin_sequence_element` for row
construction, argument types and cleanup. This prevents collection folds from
bypassing custom projections. The default implementation uses the declared JNI
layout; custom implementations can adapt it while preserving its ownership code.

## Retained callbacks and typed errors

`JniCallback::set_argument_converter` replaces one Rust-to-Kotlin argument
converter. `set_return_converter` replaces its Kotlin-to-Rust owned result
converter. `into_input` recompiles the callback's closure and interface from the
updated description; changing metadata without recompiling is not sufficient.
The source types and previously declared scoped/owning lifetime remain in force.
If a shared callback interface needs a different Kotlin signature for one
occurrence, call `set_interface_name` on that occurrence before recompiling it.
Generation rejects conflicting signatures under the same interface name.
`into_input` constructs a top-level function parameter, with the supplied wire
and target names subject to the normal generated-name checks.

Retained callback argument delivery requests `callback_arguments` for the Rust
wire expressions inside the converter's preparation scope. The emitter no longer
reads a concrete output plan's value list. Callback return construction requests
`callback_return_fragment` for its stages. The cache flag tells the converter
whether native bulk carriers were already read. The standard plan provider uses
`emit_with_validation` to reuse those cached reads without cloning or editing a
plan; wrappers should forward this operation to preserve that optimization.
An independent implementation can use `JniContext::cached_callback_validation`
with its validation code. The default callback-return implementation rejects a
buffered conversion that requires this operation instead of silently reading
the same JNI carrier again. Custom Rust fragments remain responsible for
constructing the declared source type; generated Rust compilation checks those
types.

`bind_error_output(&mut flat, "function", converter)` replaces a prepared
function's typed domain-error converter. Stable helper/handler names use the
source function identity, so a local override does not change another function's
handler for the same Rust error type. Generation checks that its wire ABI and
native handle allocation/access descriptions match the result slot's declared
error channel, including recursive bulk-row and optional layouts. A custom result
converter can declare a different matching error channel when a different wire
representation is required.
This checks declared layout compatibility, not semantic equivalence of arbitrary
custom Rust/Kotlin fragments.

User result and error handlers continue running after the ordinary native call.
Retained source callbacks continue to run when Rust invokes them. Kotlin handle
preflight, ownership receipts, failure cleanup and the C trust policy are unchanged.

## Observable integration coverage

The [independent converter fixture](../../../examples/covertest-kotlin/build_support/output_contracts.rs)
reuses native `i64` conversions and independently reconstructs
`java.time.Duration` in Kotlin. Its compiled caller exercises:

- Direct and explicitly fixed packet returns.
- Two expanded getter results and an optional result through capture delivery.
- Bulk lists and callback collection folds with `Duration` elements.
- A retained `Fn(i64) -> i64` exposed as `Duration -> Duration`, beside an
  unchanged `Long -> Long` use of the same source signature.
- A local typed error projection to `Duration`, beside an unchanged `Long` error
  handler for another function using the same Rust error type.

The [input fixture](../../../examples/covertest-kotlin/build_support/input_contracts.rs)
still supplies its own Rust construction without storing a built-in plan. The
standalone investigation's independent input and output features now both compile.

Tests compare complete generated output for independent forwarding converters,
check preparation once, preserve explicit custom packet code, reject missing
delivery and inconsistent error wires, and reuse one prepared output across
packet/direct/capture contexts without mutation. Existing generated Rust,
Kotlin and C bindings remain unchanged. The new integration fixture adds its
own bindings and entries in the aggregate consumer rules file.

The first two architecture tasks in #898 are addressed by this implementation.
The codec extension-burden review, mixed acquisition/receipt work and prepared
model readiness tasks remain separate. No runtime speedup is claimed for this
generation-time refactoring.

## Compatibility

Low-level `Delivery` literals now specify their policy; custom code should use
`Fixed` (or `..Default::default()`) unless it implements the adapter-selected
protocol. Direct `JniContext` literals also specify the selected mode and any
prepared delivery. `KotlinOutputConversion` now requires `OutputConversion`.
These are source changes to the unpublished API, not changes to emitted ABIs.
The supported wire model and adapter-owned declaration metadata remain fixed;
custom converters do not introduce arbitrary JNI carrier types.
