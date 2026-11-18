//! Errors must name the source occurrence before a conversion plan exists.
use super::*;

fn source(text: &str) -> Flat<<Scalars as Codec>::Schema> {
    Flat::parse(
        syn::parse_file(text)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
fn input() -> TestInput {
    TestInput::Leaf(TestInputLeafOp::Value)
}
fn output() -> TestOutput {
    TestOutput::Leaf(TestOutputLeafOp::Value)
}
fn optional(child: TestInput) -> TestInput {
    TestInput::Unary {
        operation: TestInputUnary::Optional,
        child: Box::new(child),
    }
}
fn error<T>(result: Result<T>, expected: &str) {
    let actual = result.err().expect("expected a generation error");
    println!("{actual}");
    assert_eq!(actual, expected);
}

#[test]
fn named_roots_calls_and_value_only_errors() {
    let flat =
        source("pub fn first(value:Option<i64>)->i32{} pub fn second(other:Option<i64>)->i64{}");
    let reason = "optional conversion is unsupported by this codec";
    error(
        compile_input(&flat, &Scalars, "first", "value", &optional(input()), &()),
        &format!("function `first` parameter `value`: {reason}"),
    );
    error(
        compile_input(&flat, &Scalars, "second", "other", &optional(input()), &()),
        &format!("function `second` parameter `other`: {reason}"),
    );
    let call = ResolvedCall {
        inputs: vec![ResolvedSlot::Tree(optional(input()))],
        output: ResolvedSlot::Tree(output()),
    };
    // The first parameter error wins over the unsupported i32 result.
    error(
        compile_call(&flat, &Scalars, "first", call, &()),
        &format!("function `first` parameter `value`: {reason}"),
    );
    let prepared =
        compile_input_value(&flat, &Scalars, &syn::parse_quote!(i64), &input(), &()).unwrap();
    error(
        compile_call(
            &flat,
            &Scalars,
            "first",
            ResolvedCall {
                inputs: vec![ResolvedSlot::Prepared(prepared)],
                output: ResolvedSlot::Tree(output()),
            },
            &(),
        ),
        "function `first` result: this codec only supports i64",
    );
    error(
        compile_output(&flat, &Scalars, "first", &output(), &()),
        "function `first` result: this codec only supports i64",
    );
    error(
        compile_input_value(
            &flat,
            &Scalars,
            &syn::parse_quote!(Option<i64>),
            &optional(input()),
            &(),
        ),
        reason,
    );
    error(
        compile_output_value(&flat, &Scalars, &syn::parse_quote!(i32), &output(), &()),
        "this codec only supports i64",
    );
    error(
        compile_input(&flat, &Scalars, "first", "missing", &input(), &()),
        "`first`: unknown parameter `missing`",
    );
}

#[test]
fn nested_optional_field_element_and_getter_errors() {
    let flat = source("pub struct Packet { pub items:Vec<i32> } pub fn nested(packet:Option<Option<i64>>){} pub fn send(packet:Packet){} pub fn read()->Packet{} pub fn observe(packet:&Packet)->i32{}");
    error(compile_input(&flat, &Scalars, "nested", "packet", &optional(optional(input())), &()),
        "function `nested` parameter `packet`: optional payload: optional conversion is unsupported by this codec");
    let sequence = TestInput::Unary {
        operation: TestInputUnary::Sequence,
        child: Box::new(input()),
    };
    let fields = TestInput::Branch {
        operation: TestInputBranch::Fields,
        children: vec![sequence],
    };
    error(
        compile_input(&flat, &Scalars, "send", "packet", &fields, &()),
        "function `send` parameter `packet`: field `items`: element: this codec only supports i64",
    );
    let sequence = TestOutput::Unary {
        operation: TestOutputUnary::Sequence {
            delivery: SequenceDelivery::Value,
            mode: None,
        },
        child: Box::new(output()),
    };
    for operation in [TestOutputBranch::Fields, TestOutputBranch::OwnedFields] {
        let fields = TestOutput::Branch {
            operation,
            children: vec![sequence.clone()],
        };
        error(
            compile_output(&flat, &Scalars, "read", &fields, &()),
            "function `read` result: field `items`: element: this codec only supports i64",
        );
    }
    let getter = TestOutput::Unary {
        operation: TestOutputUnary::Getter {
            function: "observe".into(),
        },
        child: Box::new(output()),
    };
    error(
        compile_output(&flat, &Scalars, "read", &getter, &()),
        "function `read` result: getter `observe` result: this codec only supports i64",
    );
    let optional = TestOutput::Unary {
        operation: TestOutputUnary::Optional,
        child: Box::new(output()),
    };
    error(
        compile_output_value(
            &flat,
            &Scalars,
            &syn::parse_quote!(Option<i32>),
            &optional,
            &(),
        ),
        "optional payload: this codec only supports i64",
    );
}

#[test]
fn callbacks_and_result_error_branch() {
    let flat = source("pub fn read()->Result<i64,i32>{}");
    error(
        compile_callback_argument(
            &flat,
            &Scalars,
            &syn::parse_quote!(i32),
            &output(),
            2,
            false,
            &(),
        ),
        "callback argument 2: this codec only supports i64",
    );
    error(
        compile_callback_return(&flat, &Scalars, &syn::parse_quote!(i32), &input(), &()),
        "callback result: this codec only supports i64",
    );
    let result = TestOutput::Unary {
        operation: TestOutputUnary::Result { error: None },
        child: Box::new(output()),
    };
    error(compile_output(&flat, &Scalars, "read", &result, &()),
        "function `read` result: Result Err: Result error `i32` requires an explicit error_mapper or on_error mapper");
}

#[test]
fn resolution_names_first_occurrence_and_preserves_prepared_slots() {
    let flat = source("pub fn send(first:i64,second:i64)->i64{}");
    let f = flat.function("send").unwrap();
    let call = || ConfiguredCall::<(), (), (), ()> {
        inputs: vec![ConfiguredSlot::Default, ConfiguredSlot::Recipe(())],
        output: ConfiguredSlot::Default,
        result_value: false,
        error: None,
    };
    let mut visited = 0;
    error(
        resolve_call(
            f,
            call(),
            |_, _| {
                visited += 1;
                Err::<(), _>("unsupported input".into())
            },
            |_, _, _, _| Ok(()),
        ),
        "function `send` parameter `first`: unsupported input",
    );
    assert_eq!(visited, 1);
    error(
        resolve_call(
            f,
            call(),
            |_, _| Ok(()),
            |_, _, _, _| Err::<(), _>("unsupported output".into()),
        ),
        "function `send` result: unsupported output",
    );
    let call = ConfiguredCall::<(), (), _, _> {
        inputs: vec![ConfiguredSlot::Prepared(11), ConfiguredSlot::Prepared(22)],
        output: ConfiguredSlot::Prepared(33),
        result_value: false,
        error: None,
    };
    let resolved: ResolvedCall<(), (), _, _> = resolve_call(
        f,
        call,
        |_, _| panic!("prepared input"),
        |_, _, _, _| panic!("prepared output"),
    )
    .unwrap();
    assert!(matches!(
        resolved.inputs.as_slice(),
        [ResolvedSlot::Prepared(11), ResolvedSlot::Prepared(22)]
    ));
    assert!(matches!(resolved.output, ResolvedSlot::Prepared(33)));
    let flat = source("pub fn send(first:i64,second:i64)->i64{}");
    let first = compile_input(&flat, &Scalars, "send", "first", &input(), &()).unwrap();
    let second = compile_input(&flat, &Scalars, "send", "second", &input(), &()).unwrap();
    let result = compile_output(&flat, &Scalars, "send", &output(), &()).unwrap();
    let prepared = compile_call(
        &flat,
        &Scalars,
        "send",
        ResolvedCall {
            inputs: vec![
                ResolvedSlot::Prepared(first.clone()),
                ResolvedSlot::Prepared(second),
            ],
            output: ResolvedSlot::Prepared(result.clone()),
        },
        &(),
    )
    .unwrap();
    let target = syn::parse_quote!(value);
    assert_eq!(
        prepared.inputs[0]
            .bind(&target, quote!(value))
            .unwrap()
            .to_string(),
        first.bind(&target, quote!(value)).unwrap().to_string()
    );
    assert_eq!(
        prepared.output.statements.to_string(),
        result.statements.to_string()
    );
}

#[test]
fn source_child_resolution_retains_names_and_callback_direction() {
    let flat = source("pub struct Packet { pub items:i32 } pub fn observe(p:&Packet)->i32{}");
    let ty = flat.resolve_type(&syn::parse_quote!(Packet)).unwrap();
    error(
        resolve_record_fields(&flat, &ty, &[()], |_, _| {
            Err::<(), _>("unsupported field".into())
        }),
        "field `items`: unsupported field",
    );
    error(
        resolve_named_fields(
            &flat,
            &ty,
            &std::collections::BTreeMap::<String, ()>::new(),
            &(),
            |_, _| Err::<(), _>("unsupported field".into()),
        ),
        "field `items`: unsupported field",
    );
    error(
        resolve_getter(&flat, &ty, "observe", false, &(), |_, _| {
            Err::<(), _>("unsupported getter".into())
        }),
        "getter `observe` result: unsupported getter",
    );
    for (ty, label) in [
        (syn::parse_quote!(Option<i32>), "optional payload"),
        (syn::parse_quote!(Vec<i32>), "element"),
    ] {
        error(
            resolve_container(
                &ty,
                true,
                |_| Err::<(), _>("unsupported child".into()),
                |x| x,
                |x| x,
            ),
            &format!("{label}: unsupported child"),
        );
        error(
            resolve_typed_expansion(
                &ty,
                false,
                &(),
                &(),
                &mut vec![],
                |_, _, _| Err::<(), _>("unsupported child".into()),
                |x| x,
                |x| x,
                Direction::Input,
            ),
            &format!("{label}: unsupported child"),
        );
    }
    let ty: Type = syn::parse_quote!(impl Fn(i32) -> i32);
    let operation = CallbackOperation::<()> {
        ty: ty.clone(),
        foreign: None,
        namespace: None,
        extension: None,
        explicit_return: true,
    };
    error(
        resolve_callback_arrow(
            &flat,
            &ty,
            &operation,
            &[()],
            &(),
            |_, _| Err::<(), _>("unsupported argument".into()),
            |_, _| Ok(()),
        ),
        "callback argument 0: unsupported argument",
    );
    error(
        resolve_callback_arrow(
            &flat,
            &ty,
            &operation,
            &[()],
            &(),
            |_, _| Ok(()),
            |_, _| Err::<(), _>("unsupported result".into()),
        ),
        "callback result: unsupported result",
    );
}
