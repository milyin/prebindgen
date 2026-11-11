# Complete input conversion in C and JNI

This implements the input contract discussed in #904. The original investigation
used #898 at `e78f4fd24ffaf85ccb342980f7dc33c377d39c34`; the historical failure
matrix in the overview describes that baseline. The updated probe accepts all
eight combinations of declaration-compiled and low-level converters.

## What drives generation

`prepare()` recursively compiles each configured input recipe into a converter
stored in Flat. `emit_call()` requests one complete conversion from each input
converter and wraps the remaining source call/output handling in parameter order.
The converter's supporting Rust storage stays alive through that remaining code.

```rust,ignore
pub struct InputFragment<'a, C> {
    pub context: &'a C,
    pub target: &'a syn::Ident,
    pub next: TokenStream,
}
```

`Conversion::code_fragment_slot(request, expected_type)` remains the emission
entry point. Input requests are no longer an enum of validation, acquisition,
construction, resource-query and acknowledgement operations. Output requests
retain preparation, delivery and ownership commit.

The standard `InputPlan` stores the authoritative wire tree, complete scoped
code, inert backing-storage declarations and fallibility. `InputPlan::new` builds
a value-producing conversion; `bind` composes a child during preparation without
resolving adapter bindings. The compiler hoists supporting storage outside
optional/selector branches, instead of duplicating the remaining call per branch.
The old `InputConstruction` modes and their C/JNI row-specific leaf rewrites are
removed. Sequence transport still binds/releases rows and constructs directly
into final storage, including nested/bulk collections.

## Ownership and failure

C and JNI derive possible incoming ownership from their existing wire kinds.
The separate `Resource` type/list and registration/counting flags are removed.
Kotlin's retained objects still drive preflight, locking and post-call invalidation.

The adapter establishes receipt handling before input conversion. Actual takes
record transfers in argument/row/field order. A later conversion can fail after
an earlier owner transferred: normal Rust destruction cleans constructed values,
and the existing prefix receipt tells C/Kotlin which owners were consumed.
An emptied input box is freed within its conversion, before the next argument or
source call; only the moved value and required borrowed backing storage survive.
Unread callback-result owners are reclaimed by the existing guards/failure scans.
A field failure may therefore consume fewer owners than the old grouped phases;
error paths promise correct destruction/reporting, not a fixed consumption count.

`InputTransfers` describes the boundary's actual reporting policy:

- `Report(counter)` emits updates to that local count.
- `Abort` omits updates for a C boundary whose errors abort.
- `Forbidden` rejects a transfer where no ownership receipt is available.

Shared-only calls emit no ownership counter. Custom converters must report each
actual take exactly once before later fallible work or unwinding. Arbitrary
emitted Rust cannot be proved safe by a second inventory or marker-presence test.
C pointer/alias/lifetime obligations remain with its caller; JNI keeps Kotlin
preflight and does not repeat native alias checks.

## Binding and composition

`record_input_transfer(count)` inserts a reserved operation in prepared Rust
tokens. `render_template(code, parameter, transfer)` substitutes named bindings
and translates that operation according to the boundary. Counts must be pure
`usize` expressions; aborting boundaries omit their evaluation.

Preparation-time composition preserves unresolved operations. At emission each
converter renders its own template before inserting its continuation. Callback
results use their own local reporting context. An aborting outer C wrapper
cannot erase the callback decoder's count. Obsolete input-counter/receipt binding
markers are removed; receipt setup belongs to the adapter.

## Compatibility and verification

Ordinary Cbindgen/JniGen declaration syntax stays the same. Existing boundary
ABIs are retained except for callback-result constructors borrowing an explicit
C inline owner: these now receive a pointer to caller storage, rather than a copy
of its descriptor. By-value borrows are rejected, preserving mutation and
interior-mutability behavior without implicit copies. Low-level plan construction,
input requests, leaf helper signatures and custom template contexts must migrate to the complete operation. A context
supplies `input_transfers`; JNI contexts no longer carry an unused input receipt
parameter binding. `SequenceInputRequest` carries the source collection view so
each adapter can enforce its existing borrowed-element ownership rules using its
own wire kinds.

The regression suite covers mixed shared/moving inputs, low-level moves followed
by constructor/source failure, exact prefix receipts and destruction, optional
backing storage, code size, collections and callback cached-read/cleanup paths.
C and Kotlin callers exercise the new mixed-input fixtures through the generated
native boundary. Generated artifacts and performance measurements accompany the
implementation PR; raw test/benchmark logs stay outside Git.
