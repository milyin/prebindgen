# Minimal public-API adapter

This independent crate implements a small conversion compiler using public APIs:
`ResolvedCall` → `compile_call` → `PreparedCall::view` → `staged::emit_call`.
It has its own input/output operation vocabulary and no C/JNI dependencies or
private test frontend imports. Its separate workspace keeps the external-crate
boundary visible; the root workspace commands do not run it.

From the repository root, choose an output directory and run:

```sh
cargo run --offline --manifest-path examples/adapter-minimal/Cargo.toml -- "$TMPDIR/adapter-minimal"
```

Set `TMPDIR` to your scratch directory first. Cargo and `rustc` must be available;
offline execution requires dependencies to have been fetched. The program writes
`generated.rs` and an executable into the supplied directory, invokes
`rustc --edition=2021`, and runs the executable. Successful output includes:

```text
expected rejection for maybe's optional input: function `maybe` parameter `value`: optional conversion is unsupported by this codec
generated scalar/unit assertions passed
```

Read [src/main.rs](src/main.rs) in order:

1. `Incoming` and `Outgoing` define a scalar leaf, one input optional operation,
   and uninhabited unsupported branch/arrow operations. `None` represents unit.
2. `Scalars` selects unit source attachments independently of its input/output
   plan types. Its interpreters use public leaf and unit composition helpers.
3. `finish_output` selects the scalar wire or `()` as final result delivery.
   The default root compiler applies finalization and validation.
4. `bridge` compiles a resolved call without changing source occurrence slots,
   borrows the original signature through `view`, and emits a Rust wrapper.
5. `main` checks the named optional-input rejection, compiles the emitted Rust,
   and runs assertions for scalar arguments/results, unit results and a unit
   argument with no boundary wires.

The boundary is deliberately Rust `i64`/unit with `Result<_, String>` wrappers.
It demonstrates adapter authoring and executable emission, not a foreign ABI.
Optional encoding is rejected through the codec's default; records, other scalar
types, collections, callbacks, borrowed values, source `Result` errors and bulk
transport are outside this example. A successful generated Rust assertion does
not establish C/JNI runtime behavior or the broader release criteria.

Continue with the [adapter guide](../../docs/declaration-codec.md), especially
[richer conversions](../../docs/declaration-codec.md#progressing-to-richer-conversions)
and [root/Result migration](../../docs/declaration-codec.md#root-and-result-migration).
