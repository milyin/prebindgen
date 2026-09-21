<!-- spec: {"kind": "variant", "example": "typedef", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][typedef_requests] · [Element path][typedef]
Owner: the JNI frontend

# Type alias declaring an opaque handle — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .set_package_prefix("example")
    .package(
        package!()
            .class(ptr_class!(Ledger))
            .fun(prebindgen_registry::fun!(ledger_open))
            .fun(prebindgen_registry::fun!(ledger_close)),
    )
    .build_with(prebindgen_jni::pipeline::Pipeline::V2)
    .expect("generate the JNI binding");
```

## Result

```text
recorded in the JniTarget, under `type:Ledger` (JNI handle):
    representation: ptr_class
    kotlin:         example.Ledger                          // the class holding the address
    carrier:        jlong                                   // 64 bits on every platform
    native:         example.JNINative.freeLedger            // the release, on the harness
    symbol:         "Java_example_JNINative_freeLedger"     // derived from that placement
    null handle:    throw IllegalStateException, then return a default
```

`ptr_class!` declares a Kotlin class that holds a Rust address, as
`data_class!` declares one whose properties are read. The `external` method that
frees an address lives on `JNINative`, the generated object every `external`
method lands on, and is named after the class through the method-name hook —
so it has one namespace with every other `external` method, and its symbol is
built the way theirs are.

## Checks

- The [carrier](../../stages/05-represent.md#describing-target-values-and-operations) is `jlong` and not a pointer type, because JNI's wire is 64 bits
  whatever the platform's pointer is; the Kotlin side sees a `Long`.
- A null handle is a binding failure with a message; the adapter's convention
  turns the message into an exception, distinct from the runtime failures of
  JNI calls, which carry the `jni` crate's error.
- What Kotlin does with the `Long` — the class around it, the `take()` that
  empties it, the lock that makes closing race-free in the shipping adapter's
  v1 output — is the Kotlin writer's, and none of it changes the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
  boundary recorded here.

[typedef]: README.md
[typedef_requests]: 03-requests.md
