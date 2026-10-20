use super::*;
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
    assert!(functions.contains("public fun `make`(p0_w1: ULong): ULong"));
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
    assert!(kt.contains("public fun `h`(p0_w3: test.Handle?): test.Handle?"));
    assert!(kt.contains("build: (Boolean, test.Handle?) -> R"));
    assert!(kt.contains("invalid option payload"));
    assert!(kt.contains("if (result[0] != 0L)"));
    assert!(kt.contains("if (result[0] == 1)"));
    // Validate raw widths independently from the Kotlin nullable surface.
    let mut flat=flat("pub type Handle=String;pub fn h(x:Option<Handle>)->Option<Handle>{}pub fn n(x:Option<Option<Handle>>)->Option<Option<Handle>>{}");
    let d = prebindgen_declarations::Declarations::new(&flat, vec![]).unwrap();
    for (name, count) in [("h", 1), ("n", 2)] {
        d.fill(
            &mut flat,
            &JniCodec,
            &FunctionDecl::new(syn::parse_str(name).unwrap()),
            &(),
        )
        .unwrap();
        use prebindgen_flat::Conversion;
        let f = flat.function(name).unwrap();
        assert_eq!(f.params[0].ty.slot.as_ref().unwrap().wires().len(), count);
        assert_eq!(f.ret.slot.as_ref().unwrap().wires().len(), count);
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
