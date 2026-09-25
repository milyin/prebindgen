<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_callback_requests] · [Element path][fn_callback]
Owner: the JNI frontend

# Function taking a callback — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .set_package_prefix("example")
    .package(
        package!()
            .class(data_class!(Stamp))   // see the struct path
            .fun(prebindgen_registry::fun!(stamp_each)),
    )
    .build_with(prebindgen_jni::pipeline::Pipeline::V2)
    .expect("generate the JNI binding");
```

## Result

No declarator names the callback. The JNI frontend states one for every
`impl Fn(..)` an exported function takes, as a Kotlin `fun interface` in the
base package, named from its arguments — `LongCallback` here, `Long` being how
Kotlin spells an `i64`:

```rust
let callable = binding.wire_type(JniWireType::Callable {
    interface: "example.LongCallback".into(),   // so its descriptor is `Lexample/LongCallback;`
    raw: None,
});
let callback = binding.representation(Representation::Callback {
    wire_type: callable,
    capture: Operation::Target(JniOp::CaptureCallback),   // needs `jni.env`, can fail
    invoke: Operation::Target(JniOp::CallCallback),       // needs nothing, can fail
    routes: vec![FailureRoute {
        category: FailureCategory::Runtime,
        report: Some(Report {
            error: parse_quote!(jni::errors::Error),
            operation: Operation::Target(JniOp::ReportCallbackError),
        }),
        on_report_failure: Terminal::Abort,
        terminate: Terminal::Return(parse_quote!(())),
    }],
});
binding.rule(Scope::Type(key), callback);
binding.output(
    Declaration::Callback(key),
    OutputForm::Type {
        representation: callback,
        release: None,
        meta: JniOutput::Callback { package: "example".into(), class: "LongCallback".into(), raw: None },
    },
);
```

Capturing needs the JNI environment, which the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) has; a call does not,
and cannot — a call happens on whatever thread Rust calls from, after the
wrapper has returned — so the call attaches its own thread. Both can fail with
a JNI error. What a failed call does is the one route here: write the error
out, and return from the call.

## Checks

- `raw` is set when an argument is a handle or an enum: the `external` method then
  takes a second interface over the arguments' wire forms — `Long` for an
  address, `Int` for an enum's number — and the public function adapts one to
  the other. An `i64` is spelled the same on both sides, so `LongCallback` has
  none.
- Two functions taking `impl Fn(i64)` share `LongCallback`; the signature is
  the identity, not the parameter.
- A name built from short names can coincide for two signatures —
  `Fn(FooBar, Baz)` and `Fn(Foo, BarBaz)` — or with a declared class. Every
  callback whose name is taken twice is refused, as
  `unsupported.jni.callback_name` naming the others, and so is every function
  taking one: two Kotlin declarations of one name would not compile.
- A callback whose arguments would all be refused is still stated: the refusal
  is the registry's to find and report, where the argument is.

[fn_callback]: README.md
[fn_callback_requests]: 03-requests.md
