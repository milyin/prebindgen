# Input-only prototype

This implements the input portion of the [investigation](README.md). Built-in
and independent parameter converters use the existing JNI emitter. Outputs,
retained callback converters and configured error converters keep their existing
implementation. The first checkbox in #898 therefore remains partially open.

## What can now be evaluated

```rust,ignore
pub type Bindings = Flat<JniSchema>;
pub fn generate<I: KotlinInputConversion>(flat: &Flat<JniSchema<I>>) -> Result<JniGen>;
```

`KotlinInputConversion` now includes `InputConversion`: an implementation must
describe its native wire values as well as emit its Rust and Kotlin conversion.
The existing `JniInput` implements the same contract. The declaration builder
still prepares `Bindings` with the default input type; its `build()` uses the
same generic `generate` function as an independently prepared `Flat<JniSchema<MyInput>>`.
There is no alternate emitter or fallback path.

The schema fixes output and metadata types to their current JNI representations.
The existing `Bindings` alias remains concrete, preserving type inference for
source-inspection calls such as `let flat = Bindings::parse(items)?`.
Parameterizing the existing schema avoids introducing an additional schema
trait or repeating its metadata constraints on every traversal helper. Choose
the input type before parsing/preparing a custom model. This stage does not add
an operation that changes a prepared model's schema.

`JniSchema` is now a generic marker type rather than a unit struct. Existing
uses of the default schema as a type continue to work; directly constructing the
old unit-struct value is no longer supported. No schema value is needed by Flat.
Existing independent implementations of `KotlinInputConversion` must also
implement `InputConversion`. `JniFunction::default()` permits manual export
metadata to be configured using its public package, name and order fields.

## Observable example

The [covertest converter](../../../examples/covertest-kotlin/build_support/input_contracts.rs)
owns a wire tree and implements the conversion requests itself. It stores
neither `JniInput` nor `InputPlan`:

```text
Kotlin java.time.Duration
  → generated Kotlin calls toMillis()
  → one JNI long parameter
  → independent Rust converter constructs an i64 argument
  → Rust duration_millis(milliseconds: i64)
  → existing built-in output converter returns a Kotlin Long
```

The [Kotlin caller](../../../examples/covertest-kotlin/kotlin/io/prebindgen/covertest/InputContracts.kt)
checks zero, positive, negative and both `Long` boundary values through the
compiled native library on the suite's Java 17 runtime. It also checks
`Duration.toMillis()` semantics: sub-millisecond precision is discarded, and
overflow throws `ArithmeticException` on the Kotlin side before JNI is entered.
The new binding's Rust is recorded separately as
`input-contracts.rs`, so the established `bindings.rs` remains directly comparable.

The investigation's standalone probe also uses `Flat<JniSchema<IndependentInput>>` for
its input feature. That feature now compiles. The independent-output feature
still intentionally fails: output extensibility is not implemented in this stage.

## Shared input preparation

The emitter prepares each parameter's Kotlin conversion once. Kotlin ownership
updates determine whether a hidden ownership receipt parameter is needed before
return transport is selected. That receipt tells Kotlin which owning handles
Rust consumed, including partial consumption on failure.

After selecting the final emission context, the emitter obtains native resource
descriptions through `InputFragment::Resources`. The descriptions are reused for
Kotlin preflight validation and native symbol identity. The native
`InputFragment::Acknowledge` request must agree with the Kotlin ownership updates.
The ordinary emitter and handle validator no longer read `InputPlan` storage.
The shared call assembler still makes its own staged protocol requests when it
assembles the native body; this prototype does not redesign that assembler.
Resource and acknowledgement descriptions must therefore be consistent for the
same context. As with existing custom Rust plans, conversion authors remain
responsible for declaring every native handle their emitted Rust accesses.

These operations run while generating bindings. Runtime handle locking,
validation, ownership receipts, error routing and return transports are preserved.
Conditional resources remain present in the static description even when their
active expression is false. An incomplete custom handle description produces a
generation error rather than bypassing preflight.

## Evaluation criteria

- One emitter serves built-in and independent inputs. Output generation is
  unchanged rather than duplicated behind a new API.
- A separate converter supplies both Rust and Kotlin code through the real
  native boundary, rather than merely forwarding built-in behavior.
- Forwarding converters produce identical Rust, Kotlin and ProGuard declarations
  for shared, optional, consuming and bulk handle arguments. Kotlin input
  preparation runs once per parameter.
- Missing Kotlin handle snapshots, missing allocation types and disagreements
  between native and Kotlin consumption are rejected. Conditional handle
  resources retain the native method identity.
- Existing committed generated artifacts stay unchanged; the covertest adds
  only the new converter's generated files. No throughput improvement is claimed.

The implementation deliberately leaves return-delivery token recognition and
independent output/callback converters for the next stage. Those limitations are
visible in the diff rather than hidden behind a second implementation.
