use super::*;
use prebindgen_flat::InputConversion;
fn flat(source: &str) -> Bindings {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap()
}
fn fail(builder: JniGenBuilder) -> String {
    match builder.build() {
        Ok(_) => panic!("expected generation error"),
        Err(e) => e,
    }
}
#[test]
fn rejects_unknown_parameter_override() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Key))
                .fun(fun!(f).expand_param("missing", expand_param!(Key).variant_self())),
        );
    assert!(fail(b).contains("unknown parameter"));
}
#[test]
fn rejects_recursive_expansion() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .expand(expand_param!(Key).variant(fun!(f)))
        .package(package!().class(ptr_class!(Key)).fun(fun!(f)));
    assert!(fail(b).contains("recursive input"));
}
#[test]
fn rejects_missing_handle_class() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(f)));
    assert!(fail(b).contains("no handle class"));
}
#[test]
fn rejects_jvm_signature_overflow() {
    let params = (0..128)
        .map(|i| format!("k{i}:Key"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("pub type Key=String;pub fn f({params})->Key{{}}");
    let b = JniGen::builder()
        .flat(flat(&source))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Key)).fun(fun!(f)));
    assert!(fail(b).contains("limit is 255"));
}

#[test]
fn scalar_jvm_slots_include_wide_numeric_carriers() {
    for ty in ["i64", "u32", "u64", "isize", "usize", "f64"] {
        for (count, accepted) in [(127, true), (128, false)] {
            let params = (0..count)
                .map(|i| format!("p{i}:{ty}"))
                .collect::<Vec<_>>()
                .join(",");
            let source = format!("pub fn f({params}){{}}");
            let result = JniGen::builder()
                .flat(flat(&source))
                .set_package_prefix("test")
                .package(package!().fun(fun!(f)))
                .build();
            assert_eq!(result.is_ok(), accepted, "{ty}: {count}");
        }
    }
}

#[test]
fn data_classes_resolve_forward_declarations_and_keep_overrides() {
    let generated = JniGen::builder()
        .flat(flat("pub struct Inner { pub bytes: Vec<u8> } pub struct Outer { pub inner: Inner, pub id: u64 } pub fn echo(x: Outer) -> Outer {} pub fn make(id: u64) -> Outer {} pub fn id(x: &Outer) -> u64 {}"))
        .set_package_prefix("test")
        .package(package!().class(data_class!(Outer).name("Envelope"))
            .fun(fun!(echo))
            .fun(fun!(make).expand_return(expand_return!(Outer).field(fun!(id)))))
        .package(package!("model").class(data_class!(Inner)))
        .build().unwrap();
    assert!(generated.kotlin["test/Envelope.kt"].contains("val `inner`: test.model.Inner"));
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains("p0_w1: test.Envelope"));
    assert!(functions.contains("p0_w1.`inner`.`bytes`"));
    assert!(functions.contains("test.Envelope(test.model.Inner("));
    assert!(
        functions.contains("public fun `make`(p0_w1: ULong, onError: test.JniErrorHandler<ULong>")
    );
    assert!(!generated.rust.contains("drop_pointer"));
    assert!(generated.kotlin["test/model/Inner.kt"].contains("contentEquals"));
}

#[test]
fn data_classes_reject_unsupported_shapes_before_emission() {
    for (source, expected) in [
        ("pub type Value = String;", "named-field record"),
        ("pub struct Value {}", "at least one field"),
        (
            "pub type Handle = String; pub struct Value { pub handle: Handle }",
            "field `handle` has unsupported type",
        ),
        (
            "pub struct Value { pub child: Value }",
            "recursive data class",
        ),
        (
            "pub struct Value { pub item: &i64 }",
            "cannot be a reference",
        ),
        (
            "pub struct Value { pub r#class: i64 }",
            "invalid Kotlin name",
        ),
        (
            "pub struct Value { pub x: i64, pub X: i64 }",
            "conflicting JVM getter",
        ),
        (
            "pub struct Value { pub Class: i64 }",
            "conflicting JVM getter",
        ),
    ] {
        let error = fail(
            JniGen::builder()
                .flat(flat(source))
                .set_package_prefix("test")
                .package(package!().class(data_class!(Value))),
        );
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn data_classes_reject_borrowed_occurrences_and_duplicate_representations() {
    for signature in [
        "pub fn f(v: &Value) {}",
        "pub fn f(v: &mut Value) {}",
        "pub fn f() -> &Value {}",
    ] {
        let source = format!("pub struct Value {{ pub n: i64 }} {signature}");
        assert!(fail(
            JniGen::builder()
                .flat(flat(&source))
                .set_package_prefix("test")
                .package(package!().class(data_class!(Value)).fun(fun!(f)))
        )
        .contains("owned occurrences only"));
    }
    assert!(fail(
        JniGen::builder()
            .flat(flat("pub struct Value { pub n: i64 }"))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(data_class!(Value))
                    .class(ptr_class!(Value).name("Handle"))
            )
    )
    .contains("duplicate class"));
}

#[test]
fn data_classes_check_copy_and_flattened_call_parameter_limits() {
    let fields = (0..126)
        .map(|i| format!("pub f{i}: i64"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("pub struct Value {{ {fields} }}");
    assert!(fail(
        JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().class(data_class!(Value)))
    )
    .contains("copy method"));
    let source = "pub struct Value { pub n: i64 }".to_owned()
        + &format!(
            "pub fn f({}) {{}}",
            (0..128)
                .map(|i| format!("v{i}: Value"))
                .collect::<Vec<_>>()
                .join(",")
        );
    assert!(fail(
        JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().class(data_class!(Value)).fun(fun!(f)))
    )
    .contains("limit is 255"));
}

#[test]
fn explicit_data_expansion_replaces_a_configured_handle_slot() {
    for replace in [false, true] {
        let mut function = fun!(echo);
        if replace {
            function = function.expand_param("value", expand_param!(Value).variant_self());
        }
        let result = JniGen::builder()
            .flat(flat(
                "pub struct Value { pub n: i64 } pub fn echo(value: Value) -> Value {}",
            ))
            .set_package_prefix("test")
            .configure(|flat| {
                prebindgen_declarations::bind_input(
                    flat,
                    &JniCodec,
                    "echo",
                    "value",
                    &prebindgen_declarations::Input::Leaf,
                    &(),
                )
            })
            .package(package!().class(data_class!(Value)).fun(function))
            .build();
        if replace {
            assert!(result.unwrap().kotlin["test/Functions.kt"].contains("p0_w1: test.Value"));
        } else {
            assert!(result.err().unwrap().contains("no handle class"));
        }
    }
}

#[test]
fn data_class_properties_and_operators_use_qualified_standard_types() {
    let generated = JniGen::builder()
        .flat(flat("pub struct Value { pub n: i64, pub bytes: Vec<u8> }"))
        .set_package_prefix("test")
        .package(package!().class(data_class!(Value).name("Long")))
        .build()
        .unwrap();
    let class = &generated.kotlin["test/Long.kt"];
    assert!(class.contains("val `n`: kotlin.Long"));
    assert!(class.contains("equals(other: kotlin.Any?): kotlin.Boolean"));
    assert!(class.contains("hashCode(): kotlin.Int"));
    assert!(class.contains("toString(): kotlin.String"));
}

#[test]
fn handle_records_require_owned_returns_and_reject_duplicate_consumption() {
    let source = "pub struct Handle { pub id: i64 } pub struct Inner { pub handle: Handle } pub struct Bundle { pub inner: Inner, pub label: String } pub fn make() -> Bundle {} pub fn echo(value: Bundle) -> Bundle {} pub struct Outer { pub value: Bundle } pub fn outer() -> Outer {} pub fn outer_fields(value: &Outer) -> Outer {}";
    let base = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .expand(expand_return!(Bundle).field_self_into())
            .package(
                package!()
                    .class(data_class!(Bundle))
                    .class(data_class!(Inner))
                    .class(ptr_class!(Handle)),
            )
    };
    let generated = base().package(package!().fun(fun!(make))).build().unwrap();
    assert!(generated.kotlin["test/Inner.kt"].contains("test.Handle"));
    assert!(generated.kotlin["test/Functions.kt"].contains(".close()"));
    assert!(!generated.rust.contains("Clone::clone"));
    assert!(fail(base().package(package!().fun(fun!(echo)))).contains("returns only"));
    assert!(fail(base().package(package!().fun(
        fun!(make).expand_return(expand_return!(Bundle).field_self_into().field_self_into())
    )))
    .contains("consume the same value twice"));
    assert!(fail(
        base().package(
            package!().fun(
                fun!(outer).expand_return(expand_return!(Outer).fields(fields!(outer_fields)))
            )
        )
    )
    .contains("shared-reference occurrence"));
}

#[test]
fn explicit_projection_checks_fields_and_never_moves_from_a_borrow() {
    let source = "pub struct Point { pub x:i64 } pub struct Outer { pub point:Point } pub fn make()->Outer {}";
    let base = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(data_class!(Outer))
                    .class(data_class!(Point)),
            )
    };
    for projection in [
        Projection::owned(),
        Projection::borrowed(),
        Projection::owned().field("point", Projection::borrowed()),
    ] {
        base()
            .package(
                package!()
                    .fun(fun!(make).expand_return(expand_return!(Outer).project_self(projection))),
            )
            .build()
            .unwrap();
    }
    assert!(fail(
        base().package(
            package!().fun(
                fun!(make)
                    .expand_return(expand_return!(Outer).project_self(
                        Projection::owned().field("missing", Projection::borrowed())
                    ))
            )
        )
    )
    .contains("unknown projection field"));
    assert!(fail(
        base().package(
            package!().fun(
                fun!(make).expand_return(
                    expand_return!(Outer)
                        .project_self(Projection::borrowed().field("point", Projection::owned()))
                )
            )
        )
    )
    .contains("cannot consume a shared-reference"));
    assert!(fail(base().package(package!().fun(fun!(make).expand_return(
        expand_return!(Outer).project_self(Projection::owned().field(
            "point",
            Projection::owned().field(
                "x",
                Projection::borrowed().field("bad", Projection::borrowed())
            )
        ))
    ))))
    .contains("does not support field projection overrides"));
}

#[test]
fn optional_niches_and_nested_presence_preserve_distinct_states() {
    let source=flat("pub type Handle=String; pub fn h(x:Option<Handle>)->Option<Handle>{} pub fn n(x:Option<Option<Handle>>)->Option<Option<Handle>>{} pub fn scalar(x:Option<i64>)->Option<i64>{} pub fn ch(x:Option<char>)->Option<char>{}");
    let generated = JniGen::builder()
        .flat(source)
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Handle))
                .fun(fun!(h))
                .fun(fun!(n))
                .fun(fun!(scalar))
                .fun(fun!(ch)),
        )
        .build()
        .unwrap();
    let kt = &generated.kotlin["test/Functions.kt"];
    assert!(kt.contains(
        "public fun `h`(p0_w3: test.Handle?, onError: test.JniErrorHandler<test.Handle?>"
    ));
    assert!(kt.contains("build: (Boolean, test.Handle?) -> R"));
    assert!(kt.contains("invalid option payload"));
    assert!(kt.contains("if (result[1] != 0L)"));
    assert!(kt.contains("if (result[1] == 1)"));
    // Validate raw widths independently from the Kotlin nullable surface.
    let mut flat=flat("pub type Handle=String;pub fn h(x:Option<Handle>)->Option<Handle>{}pub fn n(x:Option<Option<Handle>>)->Option<Option<Handle>>{}pub fn ch2(x:Option<Option<char>>)->Option<Option<char>>{}pub fn u2(x:Option<Option<u32>>)->Option<Option<u32>>{}");
    let d = prebindgen_declarations::Declarations::new(&flat, vec![]).unwrap();
    for (name, count) in [("h", 1), ("n", 2), ("ch2", 1), ("u2", 1)] {
        d.fill(
            &mut flat,
            &JniCodec,
            &FunctionDecl::new(syn::parse_str(name).unwrap()),
            &(),
        )
        .unwrap();
        let f = flat.function(name).unwrap();
        assert_eq!(
            crate::group::argument_parameters(f.params[0].ty.slot.as_ref().unwrap().input_node())
                .len(),
            count
        );
        assert_eq!(
            crate::group::result_packet_fields(
                &(f.ret.slot.as_ref().unwrap()).output_outcomes().success
            )
            .len(),
            count
        );
    }
}

#[test]
fn owned_optional_projection_cannot_move_a_reference_payload() {
    let error =
        fail(
            JniGen::builder()
                .flat(flat(
                    "pub type Handle=String;pub fn view(x:&Handle)->Option<&Handle>{}",
                ))
                .set_package_prefix("test")
                .package(package!().class(ptr_class!(Handle)).fun(
                    fun!(view).expand_return(expand_return!(Option<&Handle>).field_self_into()),
                )),
        );
    assert!(
        error.contains("owned projection cannot consume a reference payload"),
        "{error}"
    );
}

#[test]
fn scalar_niches_exhaust_at_the_carrier_limit() {
    for (primitive, min) in [
        (Primitive::Char, i32::MIN as i64),
        (Primitive::U32, i64::MIN),
    ] {
        let wire: JniWire = JniWire::scalar(primitive);
        assert!(JniCodec
            .remaining_niche(&wire, &JniNiche::Integer(min))
            .is_none());
        assert!(matches!(
            JniCodec.remaining_niche(&wire, &JniNiche::Integer(-1)),
            Some(JniNiche::Integer(-2))
        ));
    }
}

#[test]
fn optional_unannotated_fields_keep_outer_presence_in_both_directions() {
    use prebindgen_declarations::{bind_input, Input};
    let mut source = flat("pub struct Wrapped { pub value: Option<i64> } pub fn wrapped_fields(x: &Wrapped) -> Wrapped {} pub fn echo(value: Option<Wrapped>) -> Option<Wrapped> {}");
    let input = Input::Optional(Box::new(Input::Fields(vec![Input::Optional(Box::new(
        Input::Leaf,
    ))])));
    bind_input(&mut source, &JniCodec, "echo", "value", &input, &()).unwrap();
    let generated =
        JniGen::builder()
            .flat(source)
            .set_package_prefix("test")
            .package(package!().fun(
                fun!(echo).expand_return(expand_return!(Wrapped).fields(fields!(wrapped_fields))),
            ))
            .build()
            .unwrap();
    let kt = &generated.kotlin["test/Functions.kt"];
    assert!(kt.contains(": Boolean,"), "{kt}");
    assert!(kt.contains(": Long?"), "{kt}");
    assert!(kt.contains("build: (Boolean, Long?) -> R"), "{kt}");
}

#[test]
fn reference_and_presence_flag_niches_keep_nested_wire_counts_bounded() {
    use prebindgen_flat::WireNode;
    fn reservations(
        layout: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    ) -> Vec<Option<JniNiche>> {
        let mut result = vec![];
        match layout {
            WireNode::Chain {
                child: element,
                meta: JniChain::Optional(metadata),
                ..
            } => {
                result.push(metadata.niche().cloned());
                result.extend(reservations(element));
            }
            WireNode::Branch {
                meta: JniBranch::Group(_),
                children: elements,
                ..
            }
            | WireNode::Branch {
                meta: JniBranch::Alternatives,
                children: elements,
                ..
            } => {
                for child in elements {
                    result.extend(reservations(child));
                }
            }
            _ => {}
        }
        result
    }
    for base in ["String", "Vec<u8>", "i64", "()"] {
        for depth in 1..=6 {
            let mut ty = base.to_string();
            for _ in 0..depth {
                ty = format!("Option<{ty}>");
            }
            let mut source = flat(&format!("pub fn echo(x:{ty})->{ty}{{}}"));
            let declarations = prebindgen_declarations::Declarations::new(&source, vec![]).unwrap();
            declarations
                .fill(&mut source, &JniCodec, &fun!(echo), &())
                .unwrap();
            let function = source.function("echo").unwrap();
            let input = function.params[0].ty.slot.as_ref().unwrap();
            let output = function.ret.slot.as_ref().unwrap();
            let reference = base == "String" || base == "Vec<u8>";
            let count = if base == "()" || (reference && depth == 1) {
                1
            } else {
                2
            };
            assert_eq!(
                crate::group::argument_parameters(input.input_node()).len(),
                count,
                "input {ty}"
            );
            assert_eq!(
                crate::group::result_packet_fields(&output.output_outcomes().success).len(),
                count,
                "output {ty}"
            );
            let mut expected = if reference {
                vec![Some(JniNiche::Null)]
            } else {
                vec![None]
            };
            for i in 1..depth {
                expected.push(if reference && i == 1 {
                    None
                } else {
                    Some(JniNiche::Integer(-((i - usize::from(reference)) as i64)))
                });
            }
            expected.reverse();
            assert_eq!(reservations(input.input_node()), expected, "input {ty}");
            assert_eq!(
                reservations(&output.output_outcomes().success),
                expected,
                "output {ty}"
            );
        }
    }
}

#[test]
fn nested_optional_selectors_reserve_distinct_tags_and_disable_arm_niches() {
    use prebindgen_declarations::{bind_input, Input};
    let mut source = flat("pub fn echo(x:Option<Option<char>>)->char {} pub fn first(x:char)->char{} pub fn second(x:char)->char{}");
    let selector = Input::selector([
        Input::constructor("first", [Input::Leaf]),
        Input::constructor("second", [Input::Leaf]),
    ]);
    bind_input(
        &mut source,
        &JniCodec,
        "echo",
        "x",
        &Input::Optional(Box::new(Input::Optional(Box::new(selector)))),
        &(),
    )
    .unwrap();
    let input = source.function("echo").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap();
    assert_eq!(
        crate::group::argument_parameters(input.input_node()).len(),
        3
    ); // one tag + two arm payloads, no option flags
    assert_eq!(
        crate::group::argument_parameters(input.input_node())
            .next()
            .unwrap()
            .kind
            .available_niche(),
        Some(JniNiche::Integer(-3))
    );
    assert!(crate::group::argument_parameters(input.input_node())
        .skip(1)
        .all(|w| w.kind.available_niche().is_none()));
}

#[test]
fn result_callbacks_support_explicit_message_mapping_and_optional_throwing_defaults() {
    let source = "pub type E=String; pub fn message(e:&E)->String{} pub fn read()->Result<i64,E>{}";
    let builder = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(read)))
            .error_mapper(syn::parse_quote!(E), fun!(message))
    };
    let callback = builder().build().unwrap();
    assert!(callback.kotlin["test/Functions.kt"].contains("onError: test.EMessageHandler<Long>"));
    for class in ["", "bad/name", "bad.Errors$", "bad.Errors$$Nested"] {
        assert!(builder()
            .error_class(syn::parse_quote!(E), class)
            .build()
            .is_err());
    }
    assert!(builder()
        .error_class(syn::parse_quote!(E), "java.io.IOException")
        .error_class(syn::parse_quote!(E), "java.lang.Exception")
        .build()
        .is_err());
    assert!(builder()
        .error_class(syn::parse_quote!(E), "java.lang.IllegalArgumentException")
        .build()
        .is_ok());
    let nested = builder()
        .error_class(syn::parse_quote!(E), "test.Errors$Nested")
        .build()
        .unwrap();
    assert!(!nested.rust.contains("test/Errors$Nested"));
    assert!(nested
        .kotlin
        .values()
        .any(|s| s.contains("test.Errors.Nested::class")));
    let output = builder()
        .error_class(syn::parse_quote!(E), "java.io.IOException")
        .build()
        .unwrap();
    let kotlin = output.kotlin.values().cloned().collect::<String>();
    assert!(kotlin.contains("@kotlin.jvm.Throws(java.io.IOException::class)"));
    assert!(!output.rust.contains("java/io/IOException"));
    assert!(kotlin.contains("throw java.io.IOException(message)"));
    syn::parse_file(&output.rust).unwrap();
}

#[test]
fn collections_reject_unsupported_element_and_view_contracts() {
    for source in [
        "pub fn f(x:&mut Vec<i64>) {}",
        "pub fn f(x:Vec<&i64>) {}",
        "pub fn f(x:Vec<()>) {}",
        "pub fn f(x:Vec<Result<i64,i64>>) {}",
        "pub fn f()->Vec<Result<i64,i64>> {}",
        "pub fn f()->Vec<()> {}",
        "pub fn f()->Vec<&i64> {}",
    ] {
        let b = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(f)));
        assert!(!fail(b).is_empty());
    }
    let borrowed = JniGen::builder()
        .flat(flat("pub type Handle=String; pub fn f(x:&[Handle]) {}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handle)).fun(fun!(f)));
    assert!(fail(borrowed).contains("cannot consume elements"));
}

#[test]
fn explicit_collection_projection_cannot_move_borrowed_values() {
    let b = JniGen::builder()
        .flat(flat("pub fn f()->&Vec<i64> {}"))
        .set_package_prefix("test")
        .package(
            package!().fun(fun!(f).expand_return(expand_return!(&Vec<i64>).field_self_into())),
        );
    assert!(fail(b).contains("cannot consume a borrowed collection"));
}

#[test]
fn result_errors_use_declared_typed_output_projections_without_a_mapper() {
    let generated = JniGen::builder()
        .flat(flat("pub struct E { pub code: i64, pub message: Option<String> } pub fn parts(e:E)->E{} pub fn read()->Result<i64,E>{} pub fn another()->Result<i64,E>{}"))
        .set_package_prefix("test")
        .expand(expand_return!(E).fields_into(fields!(parts)))
        .package(package!().fun(fun!(read)).fun(fun!(another)))
        .build().unwrap();
    let handler = &generated.kotlin["test/EHandler.kt"];
    assert!(handler.contains("v0: Long, v1: String?"));
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains("onError: test.EHandler<Long>"));
    assert!(functions.contains("onBindingError: test.JniErrorHandler<Long>"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn result_unit_error_has_a_zero_argument_domain_handler() {
    let generated = JniGen::builder()
        .flat(flat("pub fn empty()->Result<i64,()>{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(empty)))
        .build()
        .unwrap();
    assert!(generated.kotlin["test/EmptyErrorHandler.kt"].contains("run(): R"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn sequence_callback_variants_are_explicit_and_support_decomposed_elements() {
    let source="pub struct Item { pub n:i64, pub text:String } pub fn parts(v:Item)->Item{} pub fn values()->Vec<Item>{}";
    let generated = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!().fun(
                fun!(values)
                    .callback_return_only()
                    .expand_return(expand_return!(Item).fields_into(fields!(parts))),
            ),
        )
        .build()
        .unwrap();
    let kotlin = &generated.kotlin["test/Functions.kt"];
    assert!(kotlin.contains("valuesFold"));
    assert!(kotlin.contains("fold: (A, Long, String) -> A"));
    assert!(!kotlin.contains("`values`("));
    assert!(generated.rust.contains("call_method_unchecked"));
    assert!(generated.rust.contains("(JLjava/lang/String;)V"));
    syn::parse_file(&generated.rust).unwrap();
    let both = JniGen::builder()
        .flat(flat("pub fn values()->Vec<i64>{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(values).callback_return()))
        .build()
        .unwrap();
    assert!(both.kotlin["test/Functions.kt"].contains("`values`("));
    assert!(both.kotlin["test/Functions.kt"].contains("`valuesFold`("));
    for source in [
        "pub fn f()->i64{}",
        "pub fn f()->Vec<i64>{} pub fn f_fold(){}",
    ] {
        assert!(JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(f).callback_return()))
            .build()
            .is_err());
    }
}

#[test]
fn callback_returns_do_not_guess_prebound_or_extra_output_projections() {
    let mut model = flat("pub fn values()->Vec<i64>{}");
    prebindgen_declarations::bind_output(
        &mut model,
        &JniCodec,
        "values",
        &prebindgen_declarations::Output::Sequence {
            element: Box::new(prebindgen_declarations::Output::Leaf),
            mode: None,
        },
        &(),
    )
    .unwrap();
    assert!(fail(
        JniGen::builder()
            .flat(model)
            .set_package_prefix("test")
            .package(package!().fun(fun!(values).callback_return()))
    )
    .contains("prebound"));
    let source="pub fn values()->Option<Vec<i64>>{} pub fn number(v:&Option<Vec<i64>>)->i64{} pub fn view(v:&Option<Vec<i64>>)->Vec<i64>{}";
    assert!(fail(
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!().fun(
                    fun!(values).callback_return().expand_return(
                        expand_return!(Option<Vec<i64>>)
                            .field(fun!(number))
                            .field(fun!(view))
                    )
                )
            )
    )
    .contains("additional return"));
}

#[test]
fn callback_return_uses_projected_sequence_presence_from_the_tree() {
    let source="pub struct Batch { pub values: Vec<i64> } pub fn read()->Batch{} pub fn view(v:&Batch)->Option<Vec<i64>>{}";
    let generated = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!().fun(
                fun!(read)
                    .callback_return_only()
                    .expand_return(expand_return!(Batch).field(fun!(view))),
            ),
        )
        .build()
        .unwrap();
    let kotlin = &generated.kotlin["test/Functions.kt"];
    assert!(kotlin.contains("`readFold`"));
    assert!(kotlin.contains("fold: (A, Long) -> A): A?"));
}

#[test]
fn logical_outcomes_keep_typed_errors_and_match_both_jni_sequence_deliveries() {
    use prebindgen_declarations::{expand_return, fields, fun, Declarations};

    let mut flat=flat("pub struct Failure {pub code:i64,pub info:Option<String>} pub fn parts(e:Failure)->Failure{} pub fn f(v:Option<Vec<i64>>)->Result<Option<Vec<Option<i64>>>,Failure>{}");
    let defaults = Declarations::new(
        &flat,
        vec![expand_return!(Failure).fields_into(fields!(parts)).into()],
    )
    .unwrap();
    let classes = BTreeMap::new();
    let mut codec = data::DataCodec::new(&flat, &classes).unwrap();
    let decl = fun!(f).callback_return();
    let errors = errors::prepare(
        &mut flat,
        &mut codec,
        &defaults,
        std::slice::from_ref(&decl),
        &classes,
        "test",
        &mut BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(errors.len(), 1);
    defaults.fill(&mut flat, &codec, &decl, &()).unwrap();
    let output = flat.function("f").unwrap().output_conversion().unwrap();
    let prebindgen_flat::Outcomes {
        success,
        source_error: Some(error),
        binding_failure: Some(binding_failure),
    } = output.output_outcomes()
    else {
        panic!("all three completion channels must be declared")
    };

    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(success)).count(),
        3
    );

    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(error)).count(),
        2
    );
    assert!(matches!(
        prebindgen_flat::wire_definitions(std::iter::once(error))
            .next()
            .unwrap()
            .kind,
        JniWire::Scalar {
            primitive: Primitive::I64,
            ..
        }
    ));
    assert!(matches!(
        prebindgen_flat::wire_definitions(std::iter::once(error))
            .nth(1)
            .unwrap()
            .kind,
        JniWire::Text { .. }
    ));
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(binding_failure)).count(),
        1
    );
    assert_eq!(
        prebindgen_flat::wire_definitions([success, error, binding_failure]).count(),
        6
    );
    assert_eq!(
        crate::group::result_packet_fields(&output.output_outcomes().success).count(),
        1
    );
    let shape = |layout: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>| {
        prebindgen_flat::wire_definitions_with_paths(std::iter::once(layout))
            .map(|e| (e.path, prebindgen_flat::type_key(&e.wire.rust_type())))
            .collect::<Vec<_>>()
    };
    let expected = shape(&output.output_outcomes().success);
    let wire = sequence::callback_target(
        &(flat.function("f").unwrap().output_conversion().unwrap())
            .output_outcomes()
            .success,
    )
    .unwrap();
    let variant = defaults
        .callback_variant(&mut flat, &decl, "f_fold")
        .unwrap();
    codec.callback_wire = Some(wire);
    defaults.fill(&mut flat, &codec, &variant, &()).unwrap();
    assert_eq!(
        shape(
            &(flat
                .function("f_fold")
                .unwrap()
                .output_conversion()
                .unwrap())
            .output_outcomes()
            .success
        ),
        expected
    );
}

#[test]
fn manual_jni_plans_share_one_tree_through_flat_and_configure() {
    fn install(model: &mut Bindings, binding_failure: bool) -> Result<()> {
        let mut input = JniCodec.input_leaf(
            &syn::parse_quote!(i64),
            &(),
            syn::parse_quote!(native_input),
            syn::parse_quote!(value),
            &Default::default(),
        )?;
        JniCodec.finish_input(&mut input, &())?;

        let mut output = JniCodec.output_leaf(
            &syn::parse_quote!(i64),
            &(),
            syn::parse_quote!(native_output),
            prebindgen_conversions::source_result_expr(),
            false,
        )?;
        JniCodec.finish_output(&mut output, &())?;

        if binding_failure {
            output.outcomes.binding_failure = Some(JniCodec.binding_failure_layout(&())?.unwrap());
        }
        let function = model.function_mut("echo")?;
        function.params[0].ty.slot = Some(input.into());
        function.ret.slot = Some(output.into());
        let input = function.input_conversions()?[0];
        assert!(std::ptr::eq(
            prebindgen_flat::wire_definitions(std::iter::once(input.input_node()))
                .next()
                .unwrap(),
            crate::group::argument_parameters(input.input_node())
                .next()
                .unwrap()
        ));
        let output = function.output_conversion()?;
        assert!(std::ptr::eq(
            prebindgen_flat::wire_definitions(std::iter::once(&output.output_outcomes().success))
                .next()
                .unwrap(),
            crate::group::result_packet_fields(&output.output_outcomes().success)
                .next()
                .unwrap()
        ));
        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(
                function.input_conversions()?[0].input_node()
            ))
            .next()
            .expect("logical wire index out of range")
            .name,
            "native_input"
        );
        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(
                &function.output_conversion()?.output_outcomes().success
            ))
            .next()
            .expect("logical wire index out of range")
            .name,
            "native_output"
        );
        assert_eq!(
            crate::group::result_packet_fields(
                &function.output_conversion()?.output_outcomes().success
            )
            .next()
            .unwrap()
            .name,
            "native_output"
        );
        Ok(())
    }
    for configure in [false, true] {
        let mut model = flat("pub fn echo(value:i64)->i64{}");
        if !configure {
            install(&mut model, true).unwrap();
        }
        let mut builder = JniGen::builder()
            .flat(model)
            .set_package_prefix("test")
            .package(package!().fun(fun!(echo)));
        if configure {
            builder = builder.configure(|flat| install(flat, true));
        }
        let generated = builder.build().unwrap();
        assert!(generated.kotlin["test/Functions.kt"].contains("native_input: Long"));
    }
    let builder = JniGen::builder()
        .flat(flat("pub fn echo(value:i64)->i64{}"))
        .set_package_prefix("test")
        .configure(|flat| install(flat, false))
        .package(package!().fun(fun!(echo)));
    assert!(fail(builder).contains("binding-failure schema"));
}
