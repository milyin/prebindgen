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

#[test]
fn bulk_resource_rows_replay_while_object_rows_decode_once() {
    let source = "pub type Handle=String;pub enum Kind{A,B}pub struct Row{pub h:Handle,pub text:Option<String>,pub bytes:Option<Vec<u8>>,pub kind:Option<Kind>}pub fn rows(values:Vec<Row>)->usize{}";
    let recipe = Input::Sequence(Box::new(Input::Record(vec![Input::Leaf; 4])));
    for bulk in [false, true] {
        let input = if bulk {
            recipe.clone().bulk()
        } else {
            recipe.clone()
        };
        let generated = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(ptr_class!(Handle))
                    .class(data_class!(Row))
                    .class(enum_class!(Kind))
                    .fun(fun!(rows).expand_param("values", input)),
            )
            .build()
            .unwrap();
        syn::parse_file(&generated.rust).unwrap();
        assert_eq!(generated.rust.contains("_prepared"), !bulk);
        if bulk {
            assert_eq!(generated.rust.matches("for p0_w").count(), 2);
            assert!(generated.rust.contains("accept_group"));
        }
    }
}

#[test]
fn gc_managed_is_opt_in_and_preserves_native_call_generation() {
    let generate_handle = |managed: bool| {
        let class = ptr_class!(Handle).name("Resource");
        let class = if managed { class.gc_managed() } else { class };
        JniGen::builder()
            .flat(flat("pub type Handle=String;pub fn make()->Handle{}pub fn read(v:&Handle)->u64{}pub fn consume(v:Handle)->u64{}"))
            .set_package_prefix("example")
            .package(package!("objects").class(class))
            .package(package!().fun(fun!(make)).fun(fun!(read)).fun(fun!(consume)))
            .prepare().unwrap()
    };
    let plain = generate(&generate_handle(false)).unwrap();
    let managed_flat = generate_handle(true);
    let managed = generate(&managed_flat.clone()).unwrap();
    assert_eq!(plain.rust, managed.rust);
    assert_eq!(
        plain.kotlin["example/Functions.kt"],
        managed.kotlin["example/Functions.kt"].replace("withGcHandleLocks", "withHandleLocks")
    );
    assert_eq!(
        plain.kotlin["example/NativeHandle.kt"],
        managed.kotlin["example/NativeHandle.kt"].replace(
            "internal open fun markConsumed()",
            "internal fun markConsumed()"
        )
    );
    assert!(!plain
        .kotlin
        .keys()
        .any(|p| p.ends_with("/GcNativeHandle.kt")));
    assert!(!plain
        .kotlin
        .values()
        .any(|s| s.contains("java.lang.ref.Cleaner")));
    assert!(plain.kotlin["example/objects/Resource.kt"].contains("example.NativeHandle(raw)"));
    assert!(managed.kotlin["example/objects/Resource.kt"]
        .contains("example.GcNativeHandle(raw, { example.JNINative.drop"));
    let helper = &managed.kotlin["example/GcNativeHandle.kt"];
    assert!(helper.contains("register(this, release)"));
    assert!(helper.contains("internal class Release(raw: Long"));
    assert!(!helper.contains("inner class"));
    assert!(helper.contains("release.disarm()"));
    assert!(helper.contains("cleanable?.clean()"));
}

#[test]
fn gc_support_class_names_are_reserved() {
    for name in ["GcNativeHandle", "NativeCleaner"] {
        let error = JniGen::builder()
            .flat(flat("pub type Handle=String;"))
            .set_package_prefix("test")
            .package(package!().class(ptr_class!(Handle).name(name)))
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reserved"), "{name}: {error}");
    }
}

#[test]
fn gc_managed_callback_returns_reject_borrowed_constructor_inputs() {
    use crate::callback;
    for managed in [false, true] {
        let class = ptr_class!(Handle);
        let class = if managed { class.gc_managed() } else { class };
        let generated = JniGen::builder()
            .flat(flat("pub type Handle=String;pub type Produced=String;pub fn construct(h:&Handle)->Produced{}pub fn call(cb:impl Fn()->Produced){}"))
            .set_package_prefix("test")
            .package(package!().class(class).fun(fun!(call).expand_param("cb",
                callback!(impl Fn()->Produced).expand_return(expand_param!(Produced).variant(fun!(construct))))))
            .build();
        if managed {
            assert!(generated
                .err()
                .unwrap()
                .contains("cannot borrow gc-managed handles"));
        } else {
            generated.unwrap();
        }
    }
}

#[test]
fn gc_managed_handles_in_borrowed_collection_constructors_are_kept_reachable() {
    let generated = JniGen::builder()
        .flat(flat("pub type Handle=String;pub type Item=String;pub fn item(value:&Handle)->Item{}pub fn read(values:Vec<Item>)->u64{}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handle).gc_managed())
            .fun(fun!(read).expand_param("values", expand_param!(Item).variant(fun!(item)))))
        .build().unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("withGcHandleLocks(handles)"));
    assert!(!generated.kotlin["test/Functions.kt"].contains("markConsumed"));
}

#[test]
fn mutable_reference_returns_never_fall_back_to_cloned_jni_handles() {
    for returned in ["&mut Child", "Option<&mut Child>", "Result<&mut Child,i64>"] {
        let source=format!("pub type Owner=String;pub type Child=String;pub fn child(owner:&mut Owner)->{returned}{{}}");
        let error = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(ptr_class!(Owner))
                    .class(ptr_class!(Child))
                    .fun(fun!(child)),
            )
            .build()
            .err()
            .unwrap();
        assert!(
            error.contains("mutable-reference outputs"),
            "{returned}: {error}"
        );
    }
}

#[test]
fn expanded_reference_to_option_keeps_jni_value_conversion() {
    let source = "pub type Child=String;pub type Owner=String;pub fn maybe(owner:&Owner)->&Option<Child>{}pub fn fallible(owner:&Owner)->Result<&Option<Child>,i64>{}pub fn length(child:&Child)->i64{}";
    let generated = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Owner))
                .fun(fun!(maybe).expand_return(expand_return!(Child).field(fun!(length))))
                .fun(fun!(fallible).expand_return(expand_return!(Child).field(fun!(length)))),
        )
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("Long?"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn static_reference_outputs_keep_independent_jni_handles() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub type Child=String;pub fn fixed()->&'static Child{}",
        ))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Child))
                .fun(fun!(fixed).expand_return(expand_return!(Child).field_self_ref())),
        )
        .build()
        .unwrap();
    assert!(generated.rust.contains("Clone"));
    assert!(generated.kotlin["test/Functions.kt"].contains("Child"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn borrowed_opaque_defaults_are_diagnosed_in_results_callbacks_and_rows() {
    for signature in [
        "pub fn f(v:&Handle)->&Handle{}",
        "pub fn f(v:&Handle)->Option<&Handle>{}",
        "pub fn f(cb:impl Fn(&Handle)){}",
        "pub fn f(cb:impl Fn(Option<&Handle>)){}",
        "pub fn f(cb:impl Fn(&[Row])){}",
    ] {
        let error = fail(
            JniGen::builder()
                .flat(flat(&format!(
                    "pub type Handle=String; pub struct Row {{pub owner:Handle}} {signature}"
                )))
                .set_package_prefix("test")
                .package(
                    package!()
                        .class(ptr_class!(Handle))
                        .class(data_class!(Row))
                        .fun(fun!(f)),
                ),
        );
        assert!(
            error.contains("would clone and allocate"),
            "{signature}: {error}"
        );
        assert!(error.contains("field_self_ref()"));
    }
}

#[test]
fn reference_copy_defaults_and_callback_local_views_are_independent() {
    for local in [false, true] {
        let callback = if local {
            fun!(call).expand_param(
                "cb",
                callback!(impl Fn(&Handle))
                    .expand_param(0, expand_return!(Handle).field(fun!(length))),
            )
        } else {
            fun!(call)
        };
        let generated=JniGen::builder()
            .flat(flat("pub type Handle=String;pub fn read(v:&Handle)->&Handle{}pub fn call(cb:impl Fn(&Handle)){}pub fn length(v:&Handle)->i64{}"))
            .set_package_prefix("test")
            .expand(expand_return!(&Handle).field_self_ref())
            .package(package!().class(ptr_class!(Handle)).fun(fun!(read)).fun(callback))
            .build().unwrap();
        assert_eq!(
            generated.rust.matches("Clone::clone").count(),
            if local { 1 } else { 2 }
        );
        syn::parse_file(&generated.rust).unwrap();
    }
}

#[test]
fn explicit_borrowed_record_and_collection_copies_work_in_bulk_and_ordinary_outputs() {
    for bulk in [false, true] {
        for container in [false, true] {
            let mut expansion = if container {
                expand_return!(&'static [Row]).field_self_ref()
            } else {
                expand_return!(Row).field_self_ref()
            };
            if bulk {
                expansion = expansion.bulk();
            }
            let generated=JniGen::builder()
                .flat(flat("pub type Handle=String;pub struct Row {pub owner:Handle,pub text:String,pub number:i64}pub fn rows()->&'static [Row]{}"))
                .set_package_prefix("test")
                .set_bulk_collection_outputs(false)
                .package(package!().class(ptr_class!(Handle)).class(data_class!(Row))
                    .fun(fun!(rows).expand_return(expansion)))
                .build().unwrap();
            assert!(generated.rust.contains("Clone::clone"));
            assert_eq!(generated.kotlin.contains_key("test/BulkLayouts.kt"), bulk);
            syn::parse_file(&generated.rust).unwrap();
        }
    }
}

#[test]
fn unrelated_declarations_and_reordering_preserve_existing_native_bindings() {
    fn generate_example(extra: bool, reverse: bool) -> JniGen {
        let source = "pub type Handle=String;pub type Added=String;pub type Error=String;pub fn code(e:&Error)->i64{}pub fn message(e:&Error)->String{}pub fn keep(value:&Handle)->i64{}pub fn values()->Result<Vec<i64>,Error>{}pub fn mapped()->Result<i64,Error>{}pub fn added()->Result<i64,i64>{}";
        let mut builder = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .expand(expand_return!(Error).field(fun!(code)));
        if extra {
            builder = builder.package(package!("added").class(ptr_class!(Added)).fun(fun!(added)));
        }
        let mut functions = vec![
            fun!(keep),
            fun!(values).callback_return(),
            fun!(mapped).on_error(fun!(message)),
        ];
        if reverse {
            functions.reverse();
        }
        let mut api = package!("api").class(ptr_class!(Handle));
        for function in functions {
            api = api.fun(function);
        }
        builder.package(api).build().unwrap()
    }
    fn rust_functions(generation: &JniGen) -> BTreeMap<String, String> {
        syn::parse_file(&generation.rust)
            .unwrap()
            .items
            .into_iter()
            .filter_map(|item| {
                if let syn::Item::Fn(function) = item {
                    Some((
                        function.sig.ident.to_string(),
                        quote!(#function).to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
    let original = generate_example(false, false);
    let functions = rust_functions(&original);
    assert!(original.rust.contains("__pb_jni_error_Error_typed_"));
    assert!(original.rust.contains("__pb_jni_error_Error_mapped_"));
    assert!(original.kotlin["test/JNINative.kt"].contains("SequenceSink_values_fold_"));
    assert!(original.kotlin["test/api/Handle.kt"].contains("JNINative.drop_Handle_"));
    for reverse in [false, true] {
        let extended = generate_example(true, reverse);
        for line in original.kotlin["test/JNINative.kt"].lines() {
            assert!(
                extended.kotlin["test/JNINative.kt"]
                    .lines()
                    .any(|candidate| candidate == line),
                "native declaration changed: {line}"
            );
        }
        let extended_functions = rust_functions(&extended);
        for (name, body) in &functions {
            assert_eq!(
                Some(body),
                extended_functions.get(name),
                "binding changed: {name}"
            );
        }
        for (path, text) in &original.kotlin {
            if path != "test/JNINative.kt" && !(reverse && path == "test/api/Functions.kt") {
                assert_eq!(
                    Some(text),
                    extended.kotlin.get(path),
                    "Kotlin changed: {path}"
                );
            }
        }
    }
}

#[test]
fn native_name_identity_has_a_fixed_encoding_and_ignores_destination_renaming() {
    let generate = |name: &str| {
        JniGen::builder()
            .flat(flat("pub fn read()->i64{}"))
            .set_package_prefix("test")
            .package(package!().fun(fun!(read).name(name)))
            .build()
            .unwrap()
    };
    let original = generate("read");
    let renamed = generate("readRenamed");
    assert!(original.kotlin["test/JNINative.kt"].contains("call_read_e42fa77bcd21ca1a("));
    assert!(original
        .rust
        .contains("Java_test_JNINative_call_1read_1e42fa77bcd21ca1a"));
    assert_eq!(original.rust, renamed.rust);
    assert_eq!(
        original.kotlin["test/JNINative.kt"],
        renamed.kotlin["test/JNINative.kt"]
    );
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
    assert!(functions.contains("x: test.Envelope"));
    assert!(functions.contains("x.`inner`.bytes"));
    assert!(functions.contains("test.Envelope(test.model.Inner("));
    assert!(functions.contains("public fun make(id: ULong, onError: test.JniErrorHandler<ULong>"));
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
fn borrowed_getter_outputs_reject_moves_and_mutable_views() {
    for returned in ["&Child", "Option<&Child>", "&Option<Child>"] {
        let source = format!("pub type Owner=String;pub type Child=String;pub fn child(v:&Owner)->{returned}{{}}pub fn read()->Owner{{}}");
        let error = fail(
            JniGen::builder()
                .flat(flat(&source))
                .set_package_prefix("test")
                .expand(expand_return!(Child).field_self_into())
                .package(
                    package!()
                        .class(ptr_class!(Owner))
                        .class(ptr_class!(Child))
                        .fun(fun!(read).expand_return(expand_return!(Owner).field(fun!(child)))),
                ),
        );
        assert!(error.contains("owned projection cannot consume"), "{error}");
    }
    let error = fail(JniGen::builder()
        .flat(flat("pub type Owner=String;pub type Child=String;pub fn child(v:&Owner)->&mut Child{}pub fn read()->Owner{}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Owner)).class(ptr_class!(Child))
            .fun(fun!(read).expand_return(expand_return!(Owner).field(fun!(child))))));
    assert!(
        error.contains("mutable-reference getter results are unsupported"),
        "{error}"
    );
}

#[test]
fn explicit_shared_data_outputs_resolve_ordinary_record_defaults() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub struct Value { pub n:i64,pub text:String }pub fn read()->&Value{}",
        ))
        .set_package_prefix("test")
        .package(
            package!()
                .class(data_class!(Value))
                .fun(fun!(read).expand_return(expand_return!(Value).field_self_ref())),
        )
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains(": test.Value"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn borrowed_handles_require_explicit_copies_while_views_can_expand() {
    let generated = JniGen::builder()
        .flat(flat("pub type Handle=String;pub fn label(v:&Handle)->&str{}pub fn plain()->Option<&Handle>{}pub fn projected()->&Handle{}"))
        .set_package_prefix("test")
        .expand(expand_return!(Handle).field(fun!(label)))
        .package(package!().class(ptr_class!(Handle)).fun(fun!(plain)
            .expand_return(expand_return!(Handle).field_self_ref()))
            .fun(fun!(projected).expand_return(expand_return!(Handle).field(fun!(label)))))
        .build().unwrap();
    let kotlin = &generated.kotlin["test/Functions.kt"];
    assert!(kotlin.contains(": test.Handle?"), "{kotlin}");
    assert!(kotlin.contains(": String"), "{kotlin}");
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
            function = function.input("value", expand_param!(Value).variant_self());
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
            assert!(result.unwrap().kotlin["test/Functions.kt"].contains("`value`: test.Value"));
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
    assert!(class.contains("val n: kotlin.Long"));
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
    assert!(fail(base().package(package!().fun(fun!(echo))))
        .contains("cannot be passed into Rust as a record"));
    assert!(fail(base().package(package!().fun(
        fun!(make).expand_return(expand_return!(Bundle).field_self_into().field_self_into())
    )))
    .contains("consume the same value twice"));
    assert!(fail(
        base()
            .expand(expand_return!(Bundle).field_self_into())
            .package(package!().fun(
                fun!(outer).expand_return(expand_return!(Outer).fields(fields!(outer_fields)))
            ))
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
    assert!(
        kt.contains("public fun h(x: test.Handle?, onError: test.JniErrorHandler<test.Handle?>")
    );
    assert!(kt.contains("build: (Boolean, test.Handle?) -> R"));
    assert!(kt.contains("invalid option payload"));
    assert!(kt.contains("if (nativeResult != 0L)"));
    assert!(kt.contains("if (returnCapture.s0 == 1)"));
    // Validate raw widths independently from the Kotlin nullable surface.
    let mut flat=flat("pub type Handle=String;pub fn h(x:Option<Handle>)->Option<Handle>{}pub fn n(x:Option<Option<Handle>>)->Option<Option<Handle>>{}pub fn ch2(x:Option<Option<char>>)->Option<Option<char>>{}pub fn u2(x:Option<Option<u32>>)->Option<Option<u32>>{}");
    let d = prebindgen_declarations::Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
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
            let declarations =
                prebindgen_declarations::Declarations::new(&source, vec![], vec![], vec![])
                    .unwrap();
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
        "pub fn f()->Vec<&i64> {}",
    ] {
        let b = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(f)));
        assert!(!fail(b).is_empty());
    }
    // Unit rows have a bulk representation, but no ordinary object-array element.
    let units = JniGen::builder()
        .flat(flat("pub fn f()->Vec<()> {}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().fun(fun!(f)));
    assert!(!fail(units).is_empty());
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
    let handler = generated
        .kotlin
        .iter()
        .find(|(path, _)| path.starts_with("test/ErrorHandler_Unit_typed_"))
        .unwrap();
    assert!(handler.1.contains("run(): R"));
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
    assert!(!kotlin.contains("values("));
    assert!(generated.rust.contains("call_method_unchecked"));
    assert!(generated.rust.contains("(JLjava/lang/String;)V"));
    syn::parse_file(&generated.rust).unwrap();
    let both = JniGen::builder()
        .flat(flat("pub fn values()->Vec<i64>{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(values).callback_return()))
        .build()
        .unwrap();
    assert!(both.kotlin["test/Functions.kt"].contains("values("));
    assert!(both.kotlin["test/Functions.kt"].contains("valuesFold("));
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
            delivery: prebindgen_declarations::SequenceDelivery::Value,
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
    assert!(kotlin.contains("readFold"));
    assert!(kotlin.contains("fold: (A, Long) -> A): A?"));
}

#[test]
fn callback_collection_presence_is_a_direct_integer_without_a_jvm_marker() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub fn rows()->Vec<i64>{} pub fn optional()->Option<Vec<i64>>{}",
        ))
        .set_package_prefix("test")
        .package(
            package!()
                .fun(fun!(rows).callback_return_only())
                .fun(fun!(optional).callback_return_only()),
        )
        .build()
        .unwrap();
    let mut checked = 0;
    for item in syn::parse_file(&generated.rust).unwrap().items {
        if let syn::Item::Fn(function) = item {
            if function.sig.ident.to_string().contains("call_1") {
                assert_eq!(function.sig.output, syn::parse_quote!(-> i32));
                let block = &function.block;
                let body = quote!(#block).to_string();
                assert!(!body.contains("prebindgen_jni_runtime :: array"));
                assert!(!body.contains("new_local_ref"));
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 2);
    let kotlin = &generated.kotlin["test/Functions.kt"];
    assert!(kotlin.contains("if (nativeResult == 0)"));
    assert!(!kotlin.contains("if (nativeResult != null)"));
    // Keep the pre-existing diagnostic instead of collapsing two absent states
    // into the fold's one nullable accumulator.
    assert!(fail(
        JniGen::builder()
            .flat(flat("pub fn nested()->Option<Option<Vec<i64>>>{}"))
            .set_package_prefix("test")
            .package(package!().fun(fun!(nested).callback_return_only()))
    )
    .contains("additional return"));
}

#[test]
fn logical_outcomes_keep_typed_errors_and_match_both_jni_sequence_deliveries() {
    use prebindgen_declarations::{expand_return, fields, fun, Declarations};

    let mut flat=flat("pub struct Failure {pub code:i64,pub info:Option<String>} pub fn parts(e:Failure)->Failure{} pub fn f(v:Option<Vec<i64>>)->Result<Option<Vec<Option<i64>>>,Failure>{}");
    let defaults = Declarations::new(
        &flat,
        vec![],
        vec![expand_return!(Failure).fields_into(fields!(parts)).into()],
        vec![],
    )
    .unwrap();
    let codec = data::DataCodec::default();
    let decl = fun!(f).callback_return();
    flat.metadata.package = "test".into();
    flat.function_mut("f").unwrap().metadata = Some(JniFunction {
        package: "test".into(),
        name: "f".into(),
        order: 0,
        error: None,
    });
    errors::prepare(&mut flat, &codec, &defaults, std::slice::from_ref(&decl)).unwrap();
    assert!(flat
        .function("f")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .is_some());
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
    sequence::callback_target(
        &(flat.function("f").unwrap().output_conversion().unwrap())
            .output_outcomes()
            .success,
    )
    .unwrap();
    let variant = defaults
        .callback_variant(&mut flat, &codec, &decl, "f_fold", &())
        .unwrap();
    defaults.fill(&mut flat, &codec, &variant, &()).unwrap();
    let callback_shape = shape(
        &flat
            .function("f_fold")
            .unwrap()
            .output_conversion()
            .unwrap()
            .output_outcomes()
            .success,
    );
    // Both deliveries retain the logical scopes and row fields. Only the
    // collection carrier changes from an array to scalar presence status.
    assert_eq!(callback_shape.len(), expected.len());
    assert_eq!(callback_shape[0].0, expected[0].0);
    assert_eq!(
        callback_shape[0].1,
        prebindgen_flat::type_key(&syn::parse_quote!(i32))
    );
    assert_ne!(callback_shape[0].1, expected[0].1);
    assert_eq!(callback_shape[1..], expected[1..]);
}

#[test]
fn manual_jni_plans_share_one_tree_through_flat_and_configure() {
    fn install(model: &mut Bindings, binding_failure: bool) -> Result<()> {
        let mut input = JniCodec.input_leaf(
            &Bindings::parse([]).unwrap(),
            &syn::parse_quote!(i64),
            &(),
            syn::parse_quote!(native_input),
            syn::parse_quote!(value),
            &Default::default(),
        )?;
        JniCodec.finish_input(&mut input, &())?;

        let mut output = JniCodec.output_leaf(
            &Bindings::parse([]).unwrap(),
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
        assert!(generated.kotlin["test/Functions.kt"].contains("`value`: Long"));
    }
    let builder = JniGen::builder()
        .flat(flat("pub fn echo(value:i64)->i64{}"))
        .set_package_prefix("test")
        .configure(|flat| install(flat, false))
        .package(package!().fun(fun!(echo)));
    assert!(fail(builder).contains("binding-failure schema"));
}

#[test]
fn emission_reads_only_the_prepared_flat_and_shared_source_metadata() {
    let prepared = JniGen::builder()
        .flat(flat("pub enum E { A, B } pub struct R{pub e:E,pub text:Option<String>} pub fn echo(r:R)->Result<R,String>{}"))
        .set_package_prefix("test")
        .set_jni_native_init("// configured initializer")
        .package(package!("model").class(enum_class!(E)).class(data_class!(R)).fun(fun!(echo)))
        .prepare().unwrap();
    assert!(prepared
        .type_info(&syn::parse_quote!(crate::R))
        .unwrap()
        .metadata
        .class
        .is_some());
    assert_eq!(
        prepared.records["crate :: R"].fields[1]
            .metadata
            .kotlin_type,
        "kotlin.String?"
    );
    let error = prepared
        .function("echo")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    let shared = &prepared
        .type_info(&syn::parse_quote!(String))
        .unwrap()
        .metadata
        .errors[&false];
    assert!(std::rc::Rc::ptr_eq(error, shared));
    let first = generate(&prepared).unwrap();
    let second = generate(&prepared.clone()).unwrap();
    assert_eq!(first.rust, second.rust);
    assert_eq!(first.kotlin, second.kotlin);
    assert!(first.kotlin["test/JNINative.kt"].contains("// configured initializer"));
    assert!(first.kotlin.contains_key("test/model/R.kt"));
}

#[test]
fn public_emission_rejects_invalid_settings_and_unplaced_bindings() {
    let mut prepared = JniGen::builder()
        .flat(flat("pub fn f()->i64{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(f)))
        .prepare()
        .unwrap();
    prepared.metadata.package = "invalid-package".into();
    assert!(generate(&prepared).is_err());
    prepared.metadata.package = "test".into();
    prepared.function_mut("f").unwrap().metadata = None;
    assert!(generate(&prepared).is_err());
}

#[test]
fn production_declarations_write_direct_trees_and_preserve_named_error_views() {
    let input: ExpandParamDecl = expand_param!(Key).variant(fun!(new_key)).variant_self();
    let output: ExpandReturnDecl = expand_return!(Key)
        .fields(fields!(key_parts).field("value", expand_return!(i64).field_self()));
    let prepared = JniGen::builder()
        .flat(flat("pub type Key=String;pub struct Parts{pub value:i64}pub fn new_key(v:i64)->Key{}pub fn key_parts(k:&Key)->Parts{}pub fn err_message(e:&Key)->String{}pub fn echo(k:Key)->Result<Key,Key>{}"))
        .set_package_prefix("test")
        .expand(input)
        .expand(output)
        .package(package!().class(ptr_class!(Key)).fun(fun!(echo)
            .expand_param("k",expand_param!(Key).variant_self())
            .expand_return(expand_return!(Key).field_self())))
        .expand(ErrorDecl::new(syn::parse_quote!(Key),fun!(err_message)))
        .prepare().unwrap();
    let input = prepared.function("echo").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap();
    assert!(matches!(
        input.input_node(),
        prebindgen_flat::WireNode::Leaf(_)
    ));
    let generated = generate(&prepared).unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("test.Key"));

    let prepared = JniGen::builder()
        .flat(flat(
            "pub type E=String;pub fn message(e:&E)->String{}pub fn fail()->Result<i64,E>{}",
        ))
        .set_package_prefix("test")
        .expand(expand_return!(E).field(fun!(message).name("message")))
        .package(package!().fun(fun!(fail)))
        .prepare()
        .unwrap();
    let files = generate(&prepared).unwrap().kotlin;
    assert!(files["test/EHandler.kt"].contains("run(message: String)"));
}

#[test]
fn explicit_view_names_reach_builders_and_duplicate_parameters_are_rejected() {
    let source="pub type E=String;pub fn code(e:&E)->i64{}pub fn text(e:&E)->String{}pub fn read()->E{}pub fn fail()->Result<i64,E>{}";
    let build = |second: &str| {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .expand(
                expand_return!(E)
                    .field(fun!(code).name("code"))
                    .field(fun!(text).name(second)),
            )
            .package(package!().fun(fun!(read)).fun(fun!(fail)))
            .build()
    };
    let generated = build("message").unwrap();
    assert!(
        generated.kotlin["test/Functions.kt"].contains("build: (code: Long, message: String) -> R")
    );
    assert!(generated.kotlin["test/EHandler.kt"].contains("run(code: Long, message: String)"));
    assert!(build("code")
        .err()
        .unwrap()
        .contains("duplicate output parameter"));
    let positional = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .expand(
            expand_return!(E)
                .field(fun!(code).name("v1"))
                .field(fun!(text)),
        )
        .package(package!().fun(fun!(read)))
        .build();
    assert!(positional
        .err()
        .unwrap()
        .contains("duplicate output parameter"));
    assert!(std::panic::catch_unwind(
        || JniGen::builder().error_mapper(syn::parse_quote!(E), fun!(text).name("renamed"))
    )
    .is_err());
}

#[test]
fn preparation_uses_module_placement_and_error_selection_from_flat() {
    use std::rc::Rc;
    let source="pub type E=String;pub fn code(e:&E)->i64{}pub fn message(e:&E)->String{}pub fn rows()->Result<Vec<i64>,E>{}pub fn mapped()->Result<i64,E>{}";
    let prepared = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("initial")
        .expand(expand_return!(E).field(fun!(code)))
        .package(
            package!()
                .fun(fun!(rows).callback_return_only())
                .fun(fun!(mapped).on_error(fun!(message))),
        )
        .configure(|flat| {
            assert_eq!(flat.metadata.package, "initial");
            assert_eq!(
                flat.function("rows")?.metadata.as_ref().unwrap().name,
                "rows"
            );
            flat.metadata.package = "canonical".into();
            flat.metadata.native_init = "/* canonical init */".into();
            for f in flat.functions.values_mut() {
                if let Some(m) = &mut f.metadata {
                    m.package = "canonical".into();
                }
            }
            flat.function_mut("mapped")?.metadata.as_mut().unwrap().name = "renamed".into();
            Ok(())
        })
        .prepare()
        .unwrap();
    let ty = prepared.resolve_type(&syn::parse_quote!(E)).unwrap();
    let errors = &prepared.type_info(&ty).unwrap().metadata.errors;
    assert_eq!(errors.len(), 2);
    let rows = prepared
        .function("rows_fold")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap();
    let mapped = prepared
        .function("mapped")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap();
    assert!(Rc::ptr_eq(rows.error.as_ref().unwrap(), &errors[&false]));
    assert!(Rc::ptr_eq(mapped.error.as_ref().unwrap(), &errors[&true]));
    assert!(!prepared
        .functions
        .keys()
        .any(|name| name.starts_with("__pb_jni_error_")));
    let generated = generate(&prepared).unwrap();
    assert!(generated.kotlin["canonical/Functions.kt"].contains("renamed"));
    assert!(generated.kotlin["canonical/Functions.kt"].contains("rowsFold"));
    assert!(!generated.kotlin["canonical/Functions.kt"].contains("rows("));
    assert!(generated.kotlin["canonical/JNINative.kt"].contains("canonical init"));
    assert!(generated.kotlin.contains_key("canonical/EHandler.kt"));
    assert!(generated
        .kotlin
        .contains_key("canonical/EMessageHandler.kt"));
    assert!(!generated.kotlin.contains_key("initial/EHandler.kt"));
}

#[test]
fn error_exception_policy_is_available_in_flat_before_preparation() {
    let mut prepared = JniGen::builder()
        .flat(flat(
            "pub type E=String;pub fn message(e:&E)->String{}pub fn f()->Result<i64,E>{}",
        ))
        .set_package_prefix("test")
        .error_class(syn::parse_quote!(E), "java.lang.IllegalStateException")
        .package(package!().fun(fun!(f).on_error(fun!(message))))
        .configure(|flat| {
            let ty = flat.resolve_type(&syn::parse_quote!(E))?;
            let metadata = flat.type_metadata_mut(&ty)?;
            assert_eq!(
                metadata.error_class.as_deref(),
                Some("java.lang.IllegalStateException")
            );
            metadata.error_class = Some("java.lang.IllegalArgumentException".into());
            Ok(())
        })
        .prepare()
        .unwrap();
    let generated = generate(&prepared).unwrap();
    let function = &generated.kotlin["test/Functions.kt"];
    assert!(function.contains("java.lang.IllegalArgumentException"));
    assert!(!function.contains("java.lang.IllegalStateException"));
    let ty = prepared.resolve_type(&syn::parse_quote!(E)).unwrap();
    prepared.type_metadata_mut(&ty).unwrap().error_class =
        Some("java.lang.RuntimeException".into());
    let regenerated = generate(&prepared).unwrap();
    assert!(regenerated.kotlin["test/Functions.kt"].contains("java.lang.RuntimeException"));
    assert!(!regenerated.kotlin["test/Functions.kt"].contains("java.lang.IllegalArgumentException"));
}

#[test]
fn configured_function_names_are_validated_from_flat() {
    let error = JniGen::builder()
        .flat(flat("pub fn first()->i64{}pub fn second()->i64{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(first)).fun(fun!(second)))
        .configure(|flat| {
            flat.function_mut("second")?.metadata.as_mut().unwrap().name = "first".into();
            Ok(())
        })
        .prepare()
        .err()
        .unwrap();
    assert!(error.contains("duplicate foreign function name"));
}

#[test]
fn completing_flat_again_preserves_prepared_enum_overrides() {
    let mut prepared = JniGen::builder()
        .flat(flat("pub enum E{A,B}pub fn f(x:E)->E{}"))
        .set_package_prefix("test")
        .package(
            package!()
                .class(enum_class!(E).variant_name("A", "Renamed"))
                .fun(fun!(f)),
        )
        .prepare()
        .unwrap();
    let before = generate(&prepared).unwrap();
    let guard_count = prepared.guards.len();
    data::prepare(&mut prepared).unwrap();
    let after = generate(&prepared).unwrap();
    assert_eq!(before.rust, after.rust);
    assert_eq!(before.kotlin, after.kotlin);
    assert_eq!(guard_count, prepared.guards.len());
    assert!(after.kotlin["test/E.kt"].contains("Renamed"));
}

#[test]
fn extending_prepared_flat_keeps_error_helpers_unique_and_existing_projections() {
    let source="pub type E=String;pub type F=String;pub fn code(e:&E)->i64{}pub fn message(f:&F)->String{}pub fn first()->Result<i64,E>{}pub fn second()->Result<i64,F>{}";
    let initial = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .expand(expand_return!(E).field(fun!(code)))
        .package(package!().fun(fun!(first)))
        .prepare()
        .unwrap();
    let original = initial
        .function("first")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap()
        .clone();
    let extended = JniGen::builder()
        .flat(initial)
        .set_package_prefix("test")
        .expand(expand_return!(F).field(fun!(message)))
        .package(package!().fun(fun!(second)))
        .prepare()
        .unwrap();
    let first = extended
        .function("first")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    let second = extended
        .function("second")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    assert!(std::rc::Rc::ptr_eq(first, &original));
    assert_ne!(first.helper, second.helper);
    let generated = generate(&extended).unwrap();
    assert!(generated.kotlin["test/EHandler.kt"].contains("Long"));
    assert!(generated.kotlin["test/FHandler.kt"].contains("String"));
}

#[test]
fn clearing_declared_metadata_and_editing_invalid_package_return_errors() {
    for source in ["pub fn f()->Result<i64,String>{}", "pub fn f()->Vec<i64>{}"] {
        let result = std::panic::catch_unwind(|| {
            JniGen::builder()
                .flat(flat(source))
                .set_package_prefix("test")
                .package(package!().fun(fun!(f).callback_return()))
                .configure(|flat| {
                    flat.function_mut("f")?.metadata = None;
                    Ok(())
                })
                .prepare()
        });
        let error = result
            .expect("preparation must return an error, not panic")
            .err()
            .unwrap();
        assert!(error.contains("has no export metadata"));
    }
    let error = JniGen::builder()
        .flat(flat("pub fn f()->i64{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(f)))
        .configure(|flat| {
            flat.metadata.package = "bad package".into();
            Ok(())
        })
        .prepare()
        .err()
        .unwrap();
    assert!(error.contains("package"));
}

#[test]
fn emission_rejects_placed_functions_with_missing_converters() {
    let mut prepared = JniGen::builder()
        .flat(flat("pub fn f()->i64{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(f)))
        .prepare()
        .unwrap();
    prepared.function_mut("f").unwrap().ret.slot = None;
    assert!(generate(&prepared).is_err());
}

#[test]
fn retained_callback_names_follow_source_types_and_prepared_placement() {
    let model = flat("pub type Original=String; pub fn first(cb: impl Fn(Original) + Send + Sync + 'static) {} pub fn second(cb: impl Fn(&Original)) {} pub fn empty(cb: impl Fn()) {}");
    let prepared = JniGen::builder()
        .flat(model)
        .set_package_prefix("example")
        .package(package!("values").class(ptr_class!(Original).name("Renamed")))
        .package(
            package!("calls")
                .fun(fun!(first))
                .fun(
                    fun!(second).expand_param(
                        "cb",
                        callback!(impl Fn(&Original))
                            .name("OriginalBorrowedCopy")
                            .expand_param(0, expand_return!(Original).field_self_ref()),
                    ),
                )
                .fun(fun!(empty)),
        )
        .prepare()
        .unwrap();
    let output = generate(&prepared).unwrap();
    assert!(output.kotlin["example/values/OriginalCallback.kt"]
        .contains("arg0_0: example.values.Renamed"));
    assert!(output.kotlin["example/VoidCallback.kt"].contains("public fun run()"));
    assert!(
        output.kotlin["example/calls/Functions.kt"].contains("example.values.OriginalCallback,")
    );
    syn::parse_file(&output.rust).unwrap();
}

#[test]
fn callback_rejections_are_preparation_errors() {
    for (source, message) in [
        (
            "pub fn f(cb:impl Fn(Result<i64,String>)) {}",
            "Result-valued callback",
        ),
        ("pub fn f(cb:impl Fn(&mut i64)) {}", "mutable callback"),
    ] {
        let builder = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test");
        let error = fail(builder.package(package!().fun(fun!(f))));
        assert!(error.contains(message), "{error}");
    }
    let error = fail(
        JniGen::builder()
            .flat(flat("pub fn f(cb:impl Fn()) {}"))
            .set_package_prefix("test")
            .package(package!().fun(fun!(f).expand_param("cb", expand_param!(i64).variant_self()))),
    );
    assert!(
        error.contains("input expansion type does not match occurrence"),
        "{error}"
    );
}

#[test]
fn callback_instance_signature_counts_receiver_slots() {
    for (count, accepted) in [(127, true), (128, false)] {
        let args = std::iter::repeat_n("i64", count)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("pub fn f(cb: impl Fn({args})) {{}}");
        let result = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(f)))
            .build();
        assert_eq!(result.is_ok(), accepted);
        if let Err(error) = result {
            assert!(error.contains("callback exceeds JVM parameter limit"));
        }
    }
}

#[test]
fn callback_signature_markers_distinguish_options_and_sequence_elements() {
    let source = "pub fn a(cb:impl Fn(i64)){} pub fn b(cb:impl Fn(Option<i64>)){} pub fn c(cb:impl Fn(Vec<i64>)){} pub fn d(cb:impl Fn(Vec<String>)){}";
    let output = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!()
                .fun(fun!(a))
                .fun(fun!(b))
                .fun(fun!(c))
                .fun(fun!(d)),
        )
        .build()
        .unwrap();
    for name in [
        "i64Callback",
        "Optionali64Callback",
        "i64VecCallback",
        "StringVecCallback",
    ] {
        assert!(output.kotlin.contains_key(&format!("test/{name}.kt")));
    }
}
#[test]
fn callback_signature_limit_counts_handle_slots() {
    for (count, accepted) in [(126, true), (127, false)] {
        let scalars = std::iter::repeat_n("i64", count)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("pub type Handle=String; pub fn f(cb:impl Fn(Handle,{scalars})) {{}}");
        let result = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().class(ptr_class!(Handle)).fun(fun!(f)))
            .build();
        assert_eq!(result.is_ok(), accepted);
    }
}

#[test]
fn callback_constructor_is_resolved_without_exporting_the_helper() {
    let source = "pub type Handler=String; pub fn make(cb:impl Fn(i64))->Handler{} pub fn use_handler(value:Handler){} pub fn pair(first:Handler,second:Handler){} pub fn unused(cb:impl Fn(Vec<Handler>)){}";
    let prepared = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .expand(expand_param!(Handler).variant(fun!(make)))
        .package(
            package!()
                .class(ptr_class!(Handler))
                .fun(fun!(use_handler))
                .fun(fun!(pair)),
        )
        .prepare()
        .expect("callback-taking helpers must compose with input defaults");
    assert!(prepared.function("make").unwrap().metadata.is_none());
    let generated = generate(&prepared).unwrap();
    assert!(generated.kotlin.contains_key("test/i64Callback.kt"));
    assert!(!generated.kotlin["test/Functions.kt"].contains("make("));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn callback_constructor_can_also_be_exported_and_selected_in_alternatives() {
    let prepared = JniGen::builder()
        .flat(flat("pub type Handler=String; pub fn make(cb:impl Fn(i64))->Handler{} pub fn use_handler(value:Handler){}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handler)).fun(fun!(make)).fun(
            fun!(use_handler).expand_param("value",expand_param!(Handler).variant(fun!(make)).variant_self()),
        ))
        .prepare()
        .expect("callback conversion must have a fresh occurrence in each constructor");
    let generated = generate(&prepared).unwrap();
    assert_eq!(
        generated
            .kotlin
            .keys()
            .filter(|p| p.ends_with("i64Callback.kt"))
            .count(),
        1
    );
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn c_supports_callbacks_in_direct_and_constructor_parameters() {
    for constructor in [false, true] {
        let source = syn::parse_file("pub type Handler=String; pub fn make(cb:impl Fn(i64))->Handler{} pub fn use_handler(value:Handler){}").unwrap();
        let model = prebindgen_c::Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let declaration = if constructor {
            fun!(use_handler).expand_param("value", expand_param!(Handler).variant(fun!(make)))
        } else {
            fun!(make)
        };
        let generated = prebindgen_c::Cbindgen::new(model)
            .opaque_ptr("handler", syn::parse_quote!(Handler))
            .function(declaration)
            .panic()
            .build()
            .unwrap();
        assert!(generated
            .header
            .contains("(*run)(void *context, int64_t arg0)"));
        syn::parse_file(&generated.rust).unwrap();
    }
}

#[test]
fn callback_constructor_rejects_a_selector_for_the_callable_itself() {
    let recipe: Input<InputRequest> =
        Input::constructor("make", [Input::selector([Input::Leaf, Input::Leaf])]);
    let error = fail(JniGen::builder()
        .flat(flat("pub type Handler=String; pub fn make(cb:impl Fn(i64))->Handler{} pub fn use_handler(value:Handler){}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handler)).fun(fun!(use_handler).input("value", recipe))));
    assert!(
        error.contains("callback parameters only support leaf inputs"),
        "{error}"
    );
}

#[test]
fn wrapped_callable_source_types_are_rejected_before_callback_preparation() {
    for callback in ["Option<impl Fn(i64)>", "Vec<impl Fn(i64)>"] {
        for constructor in [false, true] {
            let source = syn::parse_file(&format!("pub type Handler=String; pub fn make(cb:{callback})->Handler{{}} pub fn use_handler(value:Handler){{}}" )).unwrap();
            let declaration = if constructor {
                fun!(use_handler).expand_param("value", expand_param!(Handler).variant(fun!(make)))
            } else {
                fun!(make)
            };
            let error = fail(
                JniGen::builder()
                    .items(
                        source
                            .items
                            .into_iter()
                            .map(|item| (item, Default::default())),
                    )
                    .set_package_prefix("test")
                    .package(package!().class(ptr_class!(Handler)).fun(declaration)),
            );
            assert!(error.contains("unsupported source type"), "{error}");
        }
    }
}

#[test]
fn callbacks_accept_handle_containing_sequence_arguments() {
    let generated = JniGen::builder()
        .flat(flat("pub type Handle=String; pub struct Row { pub handle:Handle, pub optional:Option<Handle>, pub label:String } pub fn owned(cb:impl Fn(Vec<Row>)){} pub fn borrowed(cb:impl Fn(&[Row])){} pub fn nested(cb:impl Fn(Option<Vec<Option<Row>>>)){} pub fn handles(cb:impl Fn(Vec<Handle>)){}"))
        .set_package_prefix("test")
        .package(package!("rows").class(ptr_class!(Handle)).class(data_class!(Row)))
        .package(package!().fun(fun!(owned)).fun(fun!(borrowed).expand_param("cb",callback!(impl Fn(&[Row]))
            .expand_param(0,expand_return!(Row).field_self_ref())))
            .fun(fun!(nested)).fun(fun!(handles)))
        .build().unwrap();
    assert!(generated.kotlin["test/RowVecCallback.kt"]
        .contains("kotlin.collections.List<test.rows.Row>"));
    assert!(generated.kotlin["test/RowListCallback.kt"]
        .contains("kotlin.collections.List<test.rows.Row>"));
    assert!(generated.kotlin["test/OptionalOptionalRowVecCallback.kt"]
        .contains("kotlin.collections.List<test.rows.Row?>?"));
    assert!(generated.kotlin["test/HandleVecCallback.kt"]
        .contains("kotlin.collections.List<test.rows.Handle>"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn callbacks_return_typed_values_and_keep_unit_interfaces() {
    let generated=JniGen::builder().flat(flat("pub fn unit(cb:impl Fn(i64)) {} pub fn number(cb:impl Fn(i64)->i64)->i64{} pub fn flag(cb:impl Fn(i64)->bool)->bool{} pub fn zero(cb:impl Fn()->String)->String{}"))
        .set_package_prefix("test").package(package!().fun(fun!(unit)).fun(fun!(number)).fun(fun!(flag)).fun(fun!(zero))).build().unwrap();
    assert!(generated.kotlin["test/i64Callback.kt"].contains("public fun run(arg0_0: Long)"));
    assert!(generated.kotlin["test/i64Toi64Callback.kt"].contains(": Long"));
    assert!(generated.kotlin["test/i64ToboolCallback.kt"].contains(": Boolean"));
    assert!(generated.kotlin["test/VoidToStringCallback.kt"].contains("run(): String\n"));
    assert!(generated.rust.contains("invoke_return"));
    syn::parse_file(&generated.rust).unwrap();
}
#[test]
fn callback_returns_reject_borrows_and_unrepresented_results_in_preparation() {
    for (source, message) in [
        ("pub fn f(cb:impl Fn(&i64)->&i64) {}", "must be owned"),
        (
            "pub fn f(cb:impl Fn()->Result<i64,String>) {}",
            "Result inputs",
        ),
    ] {
        let error = fail(
            JniGen::builder()
                .flat(flat(source))
                .set_package_prefix("test")
                .package(package!().fun(fun!(f))),
        );
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn c_callback_return_failures_require_the_ordinary_c_error_policy() {
    let source = syn::parse_file("pub fn f(cb:impl Fn(i64)->i64)->i64{}").unwrap();
    let model = prebindgen_c::Bindings::parse(
        source
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap();
    let error = prebindgen_c::Cbindgen::new(model)
        .function(fun!(f))
        .build()
        .err()
        .expect("non-Result callback binding needs explicit panic policy");
    assert!(error.contains("declare .panic()"), "{error}");
}

#[test]
fn unnamed_error_handlers_are_independent_of_their_first_caller() {
    let source = "pub fn first()->Result<i64,()>{}pub fn second()->Result<i64,()>{}";
    let generate = |reverse| {
        let package = if reverse {
            package!().fun(fun!(second)).fun(fun!(first))
        } else {
            package!().fun(fun!(first)).fun(fun!(second))
        };
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(package)
            .build()
            .unwrap()
    };
    let original = generate(false);
    let reordered = generate(true);
    let (name, text) = original
        .kotlin
        .iter()
        .find(|(path, _)| path.starts_with("test/ErrorHandler_Unit_typed_"))
        .unwrap();
    assert_eq!(Some(text), reordered.kotlin.get(name));
    assert!(!original
        .kotlin
        .keys()
        .any(|path| path.contains("FirstError") || path.contains("SecondError")));
}

#[test]
fn generic_error_types_get_distinct_readable_handlers() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub fn first()->Result<i64,Vec<i64>>{}pub fn second()->Result<i64,Vec<String>>{}",
        ))
        .set_package_prefix("test")
        .package(package!().fun(fun!(first)).fun(fun!(second)))
        .build()
        .unwrap();
    assert!(generated
        .kotlin
        .keys()
        .any(|name| name.starts_with("test/ErrorHandler_Vec_i64_typed_")));
    assert!(generated
        .kotlin
        .keys()
        .any(|name| name.starts_with("test/ErrorHandler_Vec_String_typed_")));
}

#[test]
fn default_record_outputs_follow_source_ownership_and_allow_copy_overrides() {
    let records="pub struct Handle {pub id:i64}pub struct Inner {pub handle:Handle}pub struct Bundle {pub inner:Inner,pub text:String,pub bytes:Vec<u8>,pub maybe:Option<Handle>}";
    let classes = || {
        package!()
            .class(data_class!(Bundle))
            .class(data_class!(Inner))
            .class(ptr_class!(Handle))
    };
    for returned in [
        "Bundle",
        "Option<Bundle>",
        "Vec<Bundle>",
        "Result<Bundle,i64>",
    ] {
        let source = format!("{records}pub fn make()->{returned}{{}}");
        let generated = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(classes().fun(fun!(make)))
            .build()
            .unwrap();
        assert!(
            !generated.rust.contains("Clone::clone"),
            "{returned}: {}",
            generated.rust
        );
    }
    let source = format!("{records}pub fn make()->Bundle{{}}");
    for local in [0, 1, 2] {
        let function = match local {
            1 => fun!(make).expand_return(expand_return!(Bundle).field_self_into()),
            2 => fun!(make).expand_return(expand_return!(Bundle).field_self()),
            _ => fun!(make),
        };
        let generated = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .expand(expand_return!(Bundle).field_self_ref())
            .package(classes().fun(function))
            .build()
            .unwrap();
        assert_eq!(generated.rust.contains("Clone::clone"), local == 0);
    }
    let error = fail(
        JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(classes().fun(
                fun!(make).expand_return(expand_return!(Bundle).field_self().field_self_into()),
            )),
    );
    assert!(error.contains("field_self_ref()"));
    // Two views of optional references are observations, never two moves.
    let source =
        format!("{records}pub type Owner=String;pub fn maybe(owner:&Owner)->Option<&Bundle>{{}}");
    let view = prebindgen_declarations::Output::Optional(Box::new(
        prebindgen_declarations::Output::Record {
            mode: Some(prebindgen_declarations::ProjectionMode::Borrowed),
            fields: vec![prebindgen_declarations::Output::Leaf; 4],
        },
    ));
    JniGen::builder()
        .flat(flat(&source))
        .set_package_prefix("test")
        .package(classes().class(ptr_class!(Owner)).fun(fun!(maybe).output(
            prebindgen_declarations::Output::combine([view.clone(), view]),
        )))
        .build()
        .unwrap();
}

#[test]
fn owned_buffer_identity_views_move_and_independent_copies_are_explicit() {
    for bytes in [false, true] {
        let ty = if bytes { "Vec<u8>" } else { "String" };
        let source = format!("pub fn make()->{ty}{{}}pub fn observe(value:&{ty})->usize{{}}");
        for copy in [false, true] {
            let expansion = match (bytes, copy) {
                (false, false) => expand_return!(String).field_self().field(fun!(observe)),
                (true, false) => expand_return!(Vec<u8>).field_self().field(fun!(observe)),
                (false, true) => expand_return!(String).field_self_ref().field_self(),
                (true, true) => expand_return!(Vec<u8>).field_self_ref().field_self(),
            };
            let jni = JniGen::builder()
                .flat(flat(&source))
                .set_package_prefix("test")
                .package(package!().fun(fun!(make).expand_return(expansion.clone())))
                .build()
                .unwrap();
            let c = prebindgen_c::Cbindgen::new(
                prebindgen_flat::Flat::parse(
                    syn::parse_file(&source)
                        .unwrap()
                        .items
                        .into_iter()
                        .map(|i| (i, Default::default())),
                )
                .unwrap(),
            )
            .function(fun!(make).expand_return(expansion))
            .panic()
            .build()
            .unwrap();
            // Independent JVM views need independent JVM objects, without an
            // intermediate Rust clone even for an explicit borrowing view.
            assert!(!jni.rust.contains("Clone::clone"), "JNI {ty}");
            assert_eq!(c.rust.contains("Clone::clone"), copy, "C {ty}");
            let declaration = c
                .header
                .lines()
                .find(|line| line.contains("_make("))
                .unwrap();
            assert!(declaration.contains("_owned_"), "{declaration}");
        }
        for mixed in [false, true] {
            let expansion = match (bytes, mixed) {
                (false, false) => expand_return!(String).field_self().field_self(),
                (true, false) => expand_return!(Vec<u8>).field_self().field_self(),
                (false, true) => expand_return!(String).field_self().field_self_into(),
                (true, true) => expand_return!(Vec<u8>).field_self().field_self_into(),
            };
            let error = fail(
                JniGen::builder()
                    .flat(flat(&source))
                    .set_package_prefix("test")
                    .package(package!().fun(fun!(make).expand_return(expansion.clone()))),
            );
            assert!(error.contains("field_self_ref()"), "{error}");
            let c = prebindgen_c::Cbindgen::new(
                prebindgen_flat::Flat::parse(
                    syn::parse_file(&source)
                        .unwrap()
                        .items
                        .into_iter()
                        .map(|i| (i, Default::default())),
                )
                .unwrap(),
            )
            .function(fun!(make).expand_return(expansion))
            .panic()
            .build()
            .err()
            .unwrap();
            assert!(c.contains("field_self_ref()"), "{c}");
        }
        let source = format!(
            "pub fn make()->Option<{ty}>{{}}pub fn observe(value:&Option<{ty}>)->usize{{}}"
        );
        let expansion = if bytes {
            expand_return!(Option<Vec<u8>>)
                .field_self()
                .field(fun!(observe))
        } else {
            expand_return!(Option<String>)
                .field_self()
                .field(fun!(observe))
        };
        let jni = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(make).expand_return(expansion.clone())))
            .build()
            .unwrap();
        let c = prebindgen_c::Cbindgen::new(
            prebindgen_flat::Flat::parse(
                syn::parse_file(&source)
                    .unwrap()
                    .items
                    .into_iter()
                    .map(|i| (i, Default::default())),
            )
            .unwrap(),
        )
        .function(fun!(make).expand_return(expansion))
        .panic()
        .build()
        .unwrap();
        assert!(!jni.rust.contains("Clone::clone"));
        assert!(!c.rust.contains("Clone::clone"));
    }
}

#[test]
fn callback_return_handles_use_direct_locks_and_collections_keep_dynamic_locks() {
    for returned in ["Handle", "Option<Handle>", "Vec<Handle>"] {
        let source=format!("pub struct Handle {{pub n:i64}}pub fn call(callback:impl Fn()->{returned}+Send+Sync+'static)->i64{{}}");
        let generated = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().class(ptr_class!(Handle)).fun(fun!(call)))
            .build()
            .unwrap();
        let wrapper = &generated.kotlin["test/Functions.kt"];
        assert!(wrapper.contains("test.withHandleLocks("));
        if returned == "Vec<Handle>" {
            assert!(wrapper.contains(".handles.orEmpty()"));
            assert!(!wrapper.contains("kotlin.Triple("));
        } else {
            assert!(!wrapper.contains("listOfNotNull"), "{returned}: {wrapper}");
            assert!(!wrapper.contains("emptyList"), "{returned}: {wrapper}");
        }
    }
}

#[test]
fn buffer_outputs_prepare_jvm_objects_without_native_staging() {
    for ty in [
        "String",
        "Vec<u8>",
        "&'static str",
        "&'static String",
        "&'static [u8]",
        "&'static Vec<u8>",
    ] {
        for optional in [false, true] {
            let returned = if optional {
                format!("Option<Option<{ty}>>")
            } else {
                ty.into()
            };
            let generated = JniGen::builder()
                .flat(flat(&format!("pub fn make()->{returned}{{}}")))
                .set_package_prefix("test")
                .package(package!().fun(fun!(make)))
                .build()
                .unwrap();
            for staging in ["Clone::clone", ").to_string()", ").to_vec()"] {
                assert!(!generated.rust.contains(staging), "{returned}: {staging}");
            }
            // Ordinary small results must not gain a capacity JNI call.
            assert!(!generated.rust.contains("ensure_local_capacity"));
            syn::parse_file(&generated.rust).unwrap();
        }
    }
}

#[test]
fn wide_buffer_outputs_reserve_capacity_without_small_call_overhead() {
    for count in [1, 13, 14, 48] {
        let views = (0..count).fold(expand_return!(String), |views, _| views.field(fun!(view)));
        let generated = JniGen::builder()
            .flat(flat(
                "pub fn make()->String{}pub fn view(value:&String)->&str{}",
            ))
            .set_package_prefix("test")
            .package(package!().fun(fun!(make).expand_return(views)))
            .build()
            .unwrap();
        assert_eq!(generated.rust.contains("ensure_local_capacity"), count > 13);
        assert!(!generated.rust.contains("Clone::clone"));
        syn::parse_file(&generated.rust).unwrap();
    }
}

#[test]
fn reference_capacity_counts_row_carriers_and_domain_error_payloads() {
    for error in [false, true] {
        let source = if error {
            "pub type Owner=String;pub fn make()->Result<i64,Owner>{}pub fn text(owner:&Owner)->&str{}pub fn rows(owner:&Owner)->Vec<i64>{}"
        } else {
            "pub type Owner=String;pub fn make()->Owner{}pub fn text(owner:&Owner)->&str{}pub fn rows(owner:&Owner)->Vec<i64>{}"
        };
        let views = (0..14)
            .fold(expand_return!(Owner), |views, _| views.field(fun!(text)))
            .field(fun!(rows));
        let generated = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .expand(views)
            .package(package!().fun(fun!(make)))
            .build()
            .unwrap();
        // Fourteen buffers + row array + packet, boxing class and temporary.
        assert!(
            generated.rust.contains("ensure_local_capacity(18"),
            "error={error}"
        );
    }
}

#[test]
fn primitive_before_buffer_fields_has_capacity_for_boxing_class_and_value() {
    for count in [13, 14] {
        let views = (0..count).fold(expand_return!(Owner).field(fun!(number)), |views, _| {
            views.field(fun!(text))
        });
        let generated = JniGen::builder()
            .flat(flat("pub type Owner=String;pub fn make()->Owner{}pub fn number(owner:&Owner)->i64{}pub fn text(owner:&Owner)->&str{}"))
            .set_package_prefix("test")
            .package(package!().fun(fun!(make).expand_return(views)))
            .build().unwrap();
        assert_eq!(
            generated.rust.contains("ensure_local_capacity"),
            count == 14
        );
    }
}

#[test]
fn bulk_expansions_share_target_layouts_and_local_overrides_replace_defaults() {
    let generated=JniGen::builder().flat(flat("#[repr(C)] pub struct Row {pub tag:i32,pub number:i64} pub fn bulk(value:Vec<Row>)->Vec<Row>{} pub fn plain(value:Row)->Row{}"))
        .set_package_prefix("test")
        .expand(expand_param!(Row).variant_self().bulk())
        .expand(expand_return!(Row).field_self_into().bulk())
        .package(package!().class(data_class!(Row)).fun(fun!(bulk)).fun(fun!(plain).expand_param("value",expand_param!(Row).variant_self()).expand_return(expand_return!(Row).field_self_into())))
        .build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.rust.contains("offset_of!(__PbBulk_"));
    assert!(!generated.rust.contains("get_object_array_element"));
    assert!(!generated.rust.contains("new_object_array"));
    assert!(generated.kotlin["test/BulkLayouts.kt"].contains("JNINative.bulkLayouts()"));
    assert!(generated.kotlin["test/Functions.kt"].contains("List<test.Row>"));
    let native = &generated.kotlin["test/JNINative.kt"];
    let plain = native
        .lines()
        .find(|line| line.contains("call_plain_"))
        .unwrap();
    assert!(!plain.contains("ByteArray"), "{plain}");
    let bulk = native
        .lines()
        .find(|line| line.contains("call_bulk_"))
        .unwrap();
    assert!(bulk.contains("ByteArray"), "{bulk}");
}

#[test]
fn bulk_selector_and_empty_expansion_generate_valid_boundaries() {
    for source in [
        "pub fn identity(x:i64)->i64{} pub fn call(x:i64)->i64{}",
        "pub fn identity(){} pub fn call(){}",
    ] {
        let is_unit = source.contains("identity()");
        let mut builder = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test");
        let call = if is_unit {
            fun!(call).output(Output::auto().bulk())
        } else {
            fun!(call)
                .expand_param(
                    "x",
                    expand_param!(i64)
                        .variant(fun!(identity))
                        .variant_self()
                        .bulk(),
                )
                .expand_return(expand_return!(i64).field_self().bulk())
        };
        builder = builder.package(package!().fun(call));
        let generated = builder.build().unwrap();
        syn::parse_file(&generated.rust).unwrap();
        assert!(generated.kotlin.contains_key("test/BulkRuntime.kt"));
    }
}

#[test]
fn bulk_rejects_callback_objects_and_conflicting_collection_delivery() {
    let callback = JniGen::builder()
        .set_package_prefix("test")
        .flat(flat("pub fn call(callback:impl Fn(i64)){}"))
        .package(package!().fun(fun!(call).expand_param("callback", Input::auto().bulk())))
        .build()
        .err()
        .unwrap();
    assert!(callback.contains("callback"), "{callback}");
    let delivery = JniGen::builder()
        .set_package_prefix("test")
        .flat(flat("pub fn call()->Vec<i64>{}"))
        .package(
            package!().fun(
                fun!(call)
                    .expand_return(expand_return!(Vec<i64>).field_self().bulk())
                    .callback_return(),
            ),
        )
        .build()
        .err()
        .unwrap();
    assert!(
        delivery.contains("bulk cannot enclose callback delivery")
            && delivery.contains("callback element"),
        "{delivery}"
    );
}

#[test]
fn bulk_runtime_filename_is_reserved_for_all_class_kinds() {
    for class in [
        ClassDecl::from(data_class!(Row).name("BulkRuntime")),
        ClassDecl::from(ptr_class!(Row).name("BulkRuntime")),
        ClassDecl::from(enum_class!(Kind).name("BulkRuntime")),
        ClassDecl::from(callback!(impl Fn(i64)).name("BulkRuntime")),
    ] {
        let error = JniGen::builder()
            .flat(flat(
                "pub struct Row{pub number:i64} pub enum Kind{A,B} pub fn call(value:Row)->Row{} pub fn install(value:impl Fn(i64)){}",
            ))
            .set_package_prefix("test")
            .package(
                package!().class(class).fun(fun!(install)).fun(
                    fun!(call)
                        .expand_param("value", expand_param!(Row).variant_self().bulk())
                        .expand_return(expand_return!(Row).field_self_into().bulk()),
                ),
            )
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reserved generated class name"), "{error}");
    }
}

#[test]
fn bulk_collections_require_the_same_single_value_elements_as_ordinary_collections() {
    for bulk in [false, true] {
        for input in [false, true] {
            let mut model=String::from("pub struct Row{pub number:i64,pub text:String} pub fn from(number:i64,text:String)->Row{} pub fn number(value:&Row)->i64{} pub fn text(value:&Row)->&str{}");
            model.push_str(if input {
                "pub fn call(value:Vec<Row>){}"
            } else {
                "pub fn call()->Vec<Row>{}"
            });
            let mut builder = JniGen::builder()
                .flat(flat(&model))
                .set_package_prefix("test");
            if input {
                let mut expansion = expand_param!(Row).variant(fun!(from));
                if bulk {
                    expansion = expansion.bulk();
                }
                builder = builder.expand(expansion);
            } else {
                let mut expansion = expand_return!(Row).field(fun!(number)).field(fun!(text));
                if bulk {
                    expansion = expansion.bulk();
                }
                builder = builder.expand(expansion);
            }
            let error = builder
                .package(package!().fun(fun!(call)))
                .build()
                .err()
                .unwrap();
            assert!(
                error.contains("declare a data class"),
                "bulk={bulk}, input={input}: {error}"
            );
        }
    }
}

#[test]
fn type_derived_bulk_runtime_names_are_also_reserved() {
    for (source, class) in [
        (
            "pub struct BulkRuntime{pub value:i64}",
            ClassDecl::from(data_class!(BulkRuntime)),
        ),
        (
            "pub struct BulkRuntime{pub value:i64}",
            ClassDecl::from(ptr_class!(BulkRuntime)),
        ),
        (
            "pub enum BulkRuntime{A,B}",
            ClassDecl::from(enum_class!(BulkRuntime)),
        ),
    ] {
        let source = format!("{source} pub fn call(value:i64)->i64{{}}");
        let error = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(
                package!().class(class).fun(
                    fun!(call)
                        .expand_param("value", expand_param!(i64).variant_self().bulk())
                        .expand_return(expand_return!(i64).field_self().bulk()),
                ),
            )
            .build()
            .err()
            .unwrap();
        assert!(error.contains("reserved generated class name"), "{error}");
    }
}

fn bulk_result(model: &Bindings, name: &str) -> bool {
    let output = model.function(name).unwrap().output_conversion().unwrap();
    group::result_packet_fields(&output.output_outcomes().success)
        .any(|wire| matches!(wire.kind, JniWire::BulkBuffer { .. }))
}

#[test]
fn automatic_bulk_collection_outputs_keep_inputs_and_direct_buffers_ordinary() {
    let source = "pub struct Row { pub x:i64, pub y:f64 } pub struct TextRow { pub n:i64, pub text:String } pub struct BytesRow { pub n:i64, pub bytes:Vec<u8> } pub type Handle=String; pub fn numbers()->Vec<i64>{} pub fn rows()->Vec<Row>{} pub fn optional()->Option<Vec<Row>>{} pub fn array()->[Row;2]{} pub fn single()->Row{} pub fn bytes()->Vec<u8>{} pub fn texts()->Vec<TextRow>{} pub fn payloads()->Vec<BytesRow>{} pub fn handles()->Vec<Handle>{} pub fn echo(value:Vec<Row>)->Vec<Row>{}";
    let model = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!()
                .class(data_class!(Row))
                .class(data_class!(TextRow))
                .class(data_class!(BytesRow))
                .class(ptr_class!(Handle))
                .fun(fun!(numbers))
                .fun(fun!(rows))
                .fun(fun!(optional))
                .fun(fun!(array))
                .fun(fun!(single))
                .fun(fun!(bytes))
                .fun(fun!(texts))
                .fun(fun!(payloads))
                .fun(fun!(handles))
                .fun(fun!(echo)),
        )
        .prepare()
        .unwrap();
    for name in [
        "numbers", "rows", "optional", "array", "echo", "texts", "payloads", "handles",
    ] {
        assert!(bulk_result(&model, name), "{name}");
    }
    for name in ["single", "bytes"] {
        assert!(!bulk_result(&model, name), "{name}");
    }
    let input = model.function("echo").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap();
    assert!(matches!(
        input.input_node(),
        prebindgen_flat::WireNode::Chain {
            meta: JniChain::Sequence(JniSequence::Buffered),
            ..
        }
    ));
}

#[test]
fn bulk_array_default_respects_project_type_and_local_transport_choices() {
    let source = "pub struct Row { pub x:i64 } pub fn rows()->Vec<Row>{} pub fn plain()->Vec<Row>{} pub fn explicit()->Vec<Row>{}";
    let base = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
    };
    let functions = || {
        package!()
            .class(data_class!(Row))
            .fun(fun!(rows))
            .fun(fun!(plain).expand_return(expand_return!(Vec<Row>).field_self()))
            .fun(fun!(explicit).expand_return(expand_return!(Vec<Row>).field_self().bulk()))
    };
    let model = base().package(functions()).prepare().unwrap();
    assert!(bulk_result(&model, "rows"));
    assert!(!bulk_result(&model, "plain"));
    assert!(bulk_result(&model, "explicit"));
    let model = base()
        .set_bulk_collection_outputs(false)
        .package(functions())
        .prepare()
        .unwrap();
    assert!(!bulk_result(&model, "rows"));
    assert!(!bulk_result(&model, "plain"));
    assert!(bulk_result(&model, "explicit"));
    let model = base()
        .expand(expand_return!(Vec<Row>).field_self())
        .package(functions())
        .prepare()
        .unwrap();
    assert!(!bulk_result(&model, "rows"));
    assert!(bulk_result(&model, "explicit"));
}

#[test]
fn automatic_bulk_collections_preserve_fold_and_callback_local_overrides() {
    let generated=JniGen::builder().flat(flat("pub struct Row {pub x:i64} pub fn rows()->Vec<Row>{} pub fn only()->Vec<Row>{} pub fn receive(cb:impl Fn(Vec<Row>)){} pub fn receive_plain(cb:impl Fn(Vec<Row>)){}"))
        .set_package_prefix("test")
        .package(package!().class(data_class!(Row))
            .fun(fun!(rows).callback_return()).fun(fun!(only).callback_return_only())
            .fun(fun!(receive).expand_param("cb",callback!(impl Fn(Vec<Row>)).name("PackedRows")))
            .fun(fun!(receive_plain).expand_param("cb",callback!(impl Fn(Vec<Row>)).name("PlainRows")
                .expand_param(0,expand_return!(Vec<Row>).field_self()))))
        .build().unwrap();
    let kotlin = &generated.kotlin["test/Functions.kt"];
    assert!(kotlin.contains("rows("));
    assert!(kotlin.contains("rowsFold("));
    assert!(kotlin.contains("onlyFold("));
    assert!(!kotlin.contains("only("));
    assert!(kotlin.contains("fun run(raw0: ByteArray?)"));
    assert!(kotlin.contains("fun run(raw0: Array<Any?>?)"), "{kotlin}");
    assert!(kotlin.contains("SequenceCollector_rows_fold_"));
}

#[test]
fn automatic_bulk_uses_the_resolved_projection_instead_of_the_source_field_types() {
    let source="pub struct Row {pub x:i64} pub fn text(row:&Row)->String{} pub fn number(row:&Row)->i64{} pub fn rows()->Vec<Row>{}";
    let build = |projection: ExpandReturnDecl| {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .expand(projection)
            .package(package!().class(data_class!(Row)).fun(fun!(rows)))
            .prepare()
            .unwrap()
    };
    assert!(bulk_result(
        &build(expand_return!(Row).field(fun!(text))),
        "rows"
    ));
    assert!(bulk_result(
        &build(expand_return!(Row).field(fun!(number))),
        "rows"
    ));
    let local = JniGen::builder()
        .flat(flat(source))
        .set_package_prefix("test")
        .package(
            package!()
                .class(data_class!(Row))
                .fun(fun!(rows).expand_return(expand_return!(Row).field(fun!(text)))),
        )
        .prepare()
        .unwrap();
    assert!(bulk_result(&local, "rows"));
}

#[test]
fn automatic_bulk_canonicalizes_collection_spellings_and_preserves_explicit_defaults() {
    let source="pub struct Row {pub x:i64} pub fn first()->Vec<Row>{} pub fn second()->std::vec::Vec<crate::Row>{} pub fn borrowed()->&'static [Row]{}";
    let base = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
    };
    let functions = || {
        package!()
            .class(data_class!(Row))
            .fun(fun!(first))
            .fun(fun!(second))
            .fun(fun!(borrowed))
    };
    let model = base().package(functions()).prepare().unwrap();
    for name in ["first", "second", "borrowed"] {
        assert!(bulk_result(&model, name), "{name}");
    }
    let model = base()
        .expand(expand_return!(std::vec::Vec<crate::Row>).field_self())
        .package(functions())
        .prepare()
        .unwrap();
    assert!(!bulk_result(&model, "first"));
    assert!(!bulk_result(&model, "second"));
    assert!(bulk_result(&model, "borrowed"));
}

#[test]
fn default_nested_numeric_outputs_share_one_frame() {
    let generated = JniGen::builder()
        .flat(flat("pub fn nested()->Vec<Vec<i64>>{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(nested)))
        .build()
        .unwrap();
    assert_eq!(generated.rust.matches("byte_array_from_slice").count(), 1);
    assert_eq!(generated.rust.matches("bulk::Writer::new").count(), 1);
    assert!(!generated.rust.contains("set_object_array_element"));
    assert!(generated.kotlin["test/Functions.kt"].contains("List<kotlin.collections.List<Long>>"));
}

#[test]
fn expanded_result_uses_bulk_for_automatic_collection_without_packing_string_sibling() {
    let model=JniGen::builder()
        .flat(flat("pub struct Outer {pub rows:Vec<i64>,pub label:String} pub fn parts(value:Outer)->Outer{} pub fn make()->Outer{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(make).expand_return(expand_return!(Outer).fields_into(fields!(parts)))))
        .prepare().unwrap();
    let plan = model.function("make").unwrap().output_conversion().unwrap();
    let fields = group::result_packet_fields(&plan.output_outcomes().success).collect::<Vec<_>>();
    assert_eq!(fields.len(), 2);
    assert!(matches!(fields[0].kind, JniWire::BulkBuffer { .. }));
    assert!(matches!(fields[1].kind, JniWire::Text { .. }));
}

#[test]
fn nested_ordinary_overrides_do_not_inherit_implicit_inner_bulk_defaults() {
    let source="pub fn nested()->Vec<Vec<i64>>{} pub fn numbers()->Vec<i64>{} pub fn plain()->Vec<Vec<i64>>{} pub fn receive(cb:impl Fn(Vec<Vec<i64>>,Vec<Vec<i64>>)){}";
    let functions = || {
        package!()
            .fun(fun!(nested))
            .fun(fun!(numbers))
            .fun(fun!(plain).expand_return(expand_return!(Vec<Vec<i64>>).field_self()))
            .fun(
                fun!(receive).expand_param(
                    "cb",
                    callback!(impl Fn(Vec<Vec<i64>>, Vec<Vec<i64>>))
                        .expand_param(0, expand_return!(Vec<Vec<i64>>).field_self()),
                ),
            )
    };
    let base = || {
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
    };
    let model = base().package(functions()).prepare().unwrap();
    assert!(bulk_result(&model, "nested"));
    assert!(bulk_result(&model, "numbers"));
    assert!(!bulk_result(&model, "plain"));
    let kotlin = generate(&model).unwrap().kotlin;
    assert!(kotlin["test/Functions.kt"].contains("fun run(raw0: Array<Any?>?, raw1: ByteArray?)"));
    let model = base()
        .expand(expand_return!(Vec<Vec<i64>>).field_self())
        .package(functions())
        .prepare()
        .unwrap();
    assert!(!bulk_result(&model, "nested"));
    assert!(bulk_result(&model, "numbers"));
}

#[test]
fn nested_folds_remain_available_and_primitive_array_types_do_not_change() {
    for only in [false, true] {
        let declaration = if only {
            fun!(rows).callback_return_only()
        } else {
            fun!(rows).callback_return()
        };
        let generated = JniGen::builder()
            .flat(flat("pub fn rows()->Vec<Vec<i64>>{}"))
            .set_package_prefix("test")
            .package(package!().fun(declaration))
            .build()
            .unwrap();
        assert!(generated.kotlin["test/Functions.kt"].contains("rowsFold("));
    }
    // This Flat API maps fixed arrays to lists, including byte arrays. The
    // legacy generator's primitive-array representation is not used here.
    for (ty, kotlin) in [("[i64;4]", "Long"), ("[u8;16]", "Int")] {
        for enabled in [false, true] {
            let generated = JniGen::builder()
                .flat(flat(&format!("pub fn array()->{ty}{{}}")))
                .set_bulk_collection_outputs(enabled)
                .set_package_prefix("test")
                .package(package!().fun(fun!(array)))
                .build()
                .unwrap();
            assert!(generated.kotlin["test/Functions.kt"]
                .contains(&format!(": kotlin.collections.List<{kotlin}>")));
            assert_eq!(generated.rust.contains("bulk::Writer::new"), enabled);
        }
    }
}

#[test]
fn explicit_nested_outputs_have_no_hidden_bulk_frames_in_rows_or_callbacks() {
    let plain = JniGen::builder()
        .flat(flat("pub fn plain()->Vec<Vec<i64>>{}"))
        .set_package_prefix("test")
        .package(
            package!().fun(fun!(plain).expand_return(expand_return!(Vec<Vec<i64>>).field_self())),
        )
        .build()
        .unwrap();
    assert!(!plain.rust.contains("byte_array_from_slice"));
    assert!(!plain.rust.contains("bulk::Writer::new"));
    for (argument, tree) in [
        ("Vec<Vec<i64>>", expand_return!(Vec<Vec<i64>>).field_self()),
        ("&Vec<Vec<i64>>", expand_return!(Vec<Vec<i64>>).field_self()),
        ("&[Vec<i64>]", expand_return!(&[Vec<i64>]).field_self()),
    ] {
        let source = format!("pub fn receive(cb:impl Fn({argument})){{}}");
        let callback = syn::parse_str::<syn::Type>(&format!("impl Fn({argument})")).unwrap();
        let generated = JniGen::builder()
            .flat(flat(&source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(receive).expand_param(
                "cb",
                prebindgen_declarations::CallbackDecl::new(callback).expand_param(0, tree),
            )))
            .build()
            .unwrap();
        assert!(
            !generated.rust.contains("byte_array_from_slice"),
            "{argument}"
        );
        assert!(!generated.rust.contains("bulk::Writer::new"), "{argument}");
    }
    let fallible = JniGen::builder()
        .flat(flat("pub fn rows()->Result<Vec<Vec<i64>>,()>{}"))
        .set_package_prefix("test")
        .package(
            package!().fun(fun!(rows).expand_return(expand_return!(Vec<Vec<i64>>).field_self())),
        )
        .build()
        .unwrap();
    assert!(!fallible.rust.contains("bulk::Writer::new"));
}

#[test]
fn resource_free_callback_folds_omit_cleanup_bookkeeping() {
    for ty in ["i64", "String", "Row", "Option<Vec<Option<Row>>>"] {
        let generated = JniGen::builder()
            .flat(flat(&format!("pub struct Row {{pub id:i64,pub text:String}}pub fn values()->Option<Vec<{ty}>>{{}}")))
            .set_package_prefix("test")
            .package(package!().class(data_class!(Row)).fun(fun!(values).callback_return_only()))
            .build().unwrap();
        let text = &generated.kotlin["test/Functions.kt"];
        for bookkeeping in [
            "__pb_pending",
            "__pb_processed",
            "__pb_discard",
            "CollectionOwners",
            "cleanup: Throwable",
        ] {
            assert!(!text.contains(bookkeeping), "{ty}: {text}");
        }
        assert!(text.contains("current = fold("));
    }
}

#[test]
fn fixed_handle_callback_rows_use_lexical_cleanup_without_a_journal() {
    let generated = JniGen::builder()
        .flat(flat("pub type Owner=String;pub struct Row {pub first:Option<Owner>,pub second:Owner}pub fn values()->Vec<Row>{}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Owner)).class(data_class!(Row)).fun(fun!(values).callback_return_only()))
        .build().unwrap();
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(!text.contains("__pb_pending"));
    assert!(!text.contains("CollectionOwners"));
    assert!(text.contains("owner.close()"));
    assert!(text.contains("__pb_discard(failure, __pb_processed)"));
    assert!(text.contains("current = fold("));
}

#[test]
fn nested_callback_rows_reuse_one_owner_journal() {
    for bulk in [false, true] {
        let mut declaration = fun!(values).callback_return_only();
        if bulk {
            declaration = declaration
                .expand_return(expand_return!(Vec<Option<Owner>>).field_self_into().bulk());
        }
        let generated = JniGen::builder()
            .flat(flat(
                "pub type Owner=String;pub fn values()->Option<Vec<Option<Vec<Option<Owner>>>>>{}",
            ))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(false)
            .package(package!().class(ptr_class!(Owner)).fun(declaration))
            .build()
            .unwrap();
        let text = &generated.kotlin["test/Functions.kt"];
        assert_eq!(
            text.matches("= test.CollectionOwners()").count(),
            1,
            "{text}"
        );
        assert!(text.contains("__pb_foldOwners).own("), "{text}");
        assert!(text.contains("__pb_foldOwners!!.release()"));
        assert!(text.contains("__pb_foldOwners?.close(failure)"));
        assert!(!text.contains("__pb_pending"));
    }
}

#[test]
fn collection_output_owners_are_shared_without_row_cleanup_closures() {
    for bulk in [false, true] {
        let mut declaration = fun!(values);
        if bulk {
            declaration = declaration.expand_return(expand_return!(Owner).field_self_into().bulk());
        }
        let generated = JniGen::builder()
            .flat(flat(
                "pub type Owner=String;pub fn values()->Vec<Option<Vec<Option<Owner>>>>{}",
            ))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(false)
            .package(package!().class(ptr_class!(Owner)).fun(declaration))
            .build()
            .unwrap();
        let text = &generated.kotlin["test/Functions.kt"];
        assert_eq!(
            text.matches("= test.CollectionOwners()").count(),
            1,
            "{text}"
        );
        assert!(text.contains(".own("));
        assert!(!text.contains("cleanups"));
        assert!(!text.contains("bulkActions"));
        assert!(!text.contains("MutableList<("));
    }
}

#[test]
fn resource_free_collection_outputs_have_no_cleanup_bookkeeping() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub struct Row {pub number:i64,pub text:String}pub fn values()->Vec<Option<Row>>{}",
        ))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().class(data_class!(Row)).fun(fun!(values)))
        .build()
        .unwrap();
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(!text.contains("CollectionOwners"));
    assert!(!text.contains("cleanups"));
    assert!(!text.contains("cleanupError"));
    assert!(text.contains("kotlin.collections.ArrayList("));
}

#[test]
fn jni_collection_outputs_transfer_rows_without_native_commit_queues() {
    for callback in [false, true] {
        let function = if callback {
            fun!(values).callback_return_only()
        } else {
            fun!(values)
        };
        let generated = JniGen::builder()
            .flat(flat(
                "pub type Owner=String;pub fn values()->Vec<Option<Vec<Owner>>>{}",
            ))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(false)
            .package(package!().class(ptr_class!(Owner)).fun(function))
            .build()
            .unwrap();
        assert!(!generated.rust.contains("_commits"), "{}", generated.rust);
        assert!(generated.rust.contains("OutputRows::new"));
        assert!(generated.rust.contains("complete_row"));
        syn::parse_file(&generated.rust).unwrap();
    }
    let generated = JniGen::builder()
        .flat(flat("pub fn values()->Vec<i64>{}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().fun(fun!(values).callback_return()))
        .build()
        .unwrap();
    assert!(!generated.rust.contains("_commits"));
    assert!(!generated.rust.contains("OutputRows::new"));
}

#[test]
fn cleanup_helpers_share_resolved_shapes_across_functions_callbacks_and_nested_rows() {
    fn generate(reverse: bool) -> JniGen {
        let mut functions = vec![
            fun!(first),
            fun!(second),
            fun!(nested),
            fun!(callback),
            fun!(projected).expand_return(expand_return!(Row).field(fun!(row_owner))),
            fun!(swapped),
            fun!(bulk_first).expand_return(expand_return!(Row).field_self_into().bulk()),
            fun!(bulk_second).expand_return(expand_return!(Row).field_self_into().bulk()),
        ];
        if reverse {
            functions.reverse();
        }
        let mut package = package!()
            .class(ptr_class!(Owner))
            .class(ptr_class!(Other))
            .class(data_class!(Row))
            .class(data_class!(Swapped));
        for function in functions {
            package = package.fun(function);
        }
        JniGen::builder()
            .flat(flat(
                "
            pub type Owner=String; pub type Other=String;
            pub struct Row { pub owner:Owner, pub value:i64, pub other:Other }
            pub struct Swapped { pub other:Other, pub value:i64, pub owner:Owner }
            pub fn first()->Vec<Row>{} pub fn second()->Vec<Row>{}
            pub fn nested()->Vec<Option<Vec<Row>>>{}
            pub fn callback(callback:impl Fn(Vec<Row>)){}
            pub fn projected()->Vec<Row>{} pub fn row_owner(row:&Row)->&Owner{}
            pub fn swapped()->Vec<Swapped>{}
            pub fn bulk_first()->Vec<Row>{} pub fn bulk_second()->Vec<Row>{}
        ",
            ))
            .set_package_prefix("test")
            .expand(expand_return!(&Owner).field_self_ref())
            .set_bulk_collection_outputs(false)
            .package(package)
            .build()
            .unwrap()
    }
    fn helpers(generated: &JniGen) -> BTreeMap<String, String> {
        use quote::ToTokens;
        syn::parse_file(&generated.rust)
            .unwrap()
            .items
            .into_iter()
            .filter_map(|item| {
                let syn::Item::Fn(function) = item else {
                    return None;
                };
                let name = function.sig.ident.to_string();
                name.starts_with("__pb_cleanup_")
                    .then(|| (name, function.to_token_stream().to_string()))
            })
            .collect()
    }
    let generated = generate(false);
    let definitions = helpers(&generated);
    // Original Row, swapped handle types, a local single-owner projection and
    // the enclosing optional nested-array row each have one object helper.
    assert_eq!(
        definitions
            .keys()
            .filter(|name| name.starts_with("__pb_cleanup_typed_"))
            .count(),
        4
    );
    assert_eq!(
        definitions
            .keys()
            .filter(|name| name.starts_with("__pb_cleanup_native_"))
            .count(),
        1
    );
    assert_eq!(definitions, helpers(&generate(true)));
}

#[test]
fn input_collections_and_callback_results_do_not_emit_output_cleanup_helpers() {
    let generated=JniGen::builder()
        .flat(flat("pub type Owner=String;pub fn consume(values:Vec<Owner>)->i64{}pub fn run(callback:impl Fn()->Vec<Owner>)->i64{}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().class(ptr_class!(Owner)).fun(fun!(consume)).fun(fun!(run)))
        .build().unwrap();
    assert!(!generated.rust.contains("__pb_cleanup_"));
}

// A callback element is one boundary argument even when it expands into many
// Kotlin values or contains its own dynamic collection.
fn callback_element_is_bulk(model: &Bindings, function: &str) -> bool {
    let output = model
        .function(function)
        .unwrap()
        .output_conversion()
        .unwrap();
    let (_, element, delivery) =
        sequence::return_sequence(&output.output_outcomes().success).unwrap();
    assert!(matches!(delivery, JniSequence::Callback));
    let fields = group::callback_parameters(element).collect::<Vec<_>>();
    matches!(fields.as_slice(), [wire] if matches!(wire.kind, JniWire::BulkBuffer { .. }))
}

#[test]
fn bulk_row_expansions_compose_with_additional_and_only_callback_delivery() {
    for only in [false, true] {
        for global in [false, true] {
            let mut builder = JniGen::builder()
                .flat(flat(
                    "pub struct Row {pub n:i64,pub text:String}pub fn rows()->Vec<Row>{}",
                ))
                .set_package_prefix("test")
                .set_bulk_collection_outputs(false);
            let expansion = expand_return!(Row).field_self_into().bulk();
            let mut function = if only {
                fun!(rows).callback_return_only()
            } else {
                fun!(rows).callback_return()
            };
            if global {
                builder = builder.expand(expansion);
            } else {
                function = function.expand_return(expansion);
            }
            let model = builder
                .package(package!().class(data_class!(Row)).fun(function))
                .prepare()
                .unwrap();
            assert!(callback_element_is_bulk(&model, "rows_fold"));
            if !only {
                assert!(bulk_result(&model, "rows"));
            }
            let generated = generate(&model).unwrap();
            assert!(generated.kotlin["test/Functions.kt"].contains("fold: (A, test.Row) -> A"));
            assert!(generated.kotlin["test/JNINative.kt"].contains("fun run(v0: ByteArray?)"));
            syn::parse_file(&generated.rust).unwrap();
        }
    }
}

#[test]
fn bulk_callback_batches_and_optional_elements_keep_their_scopes() {
    for (source, expansion, signature) in [
        (
            "pub struct Row {pub n:i64}pub fn batches()->Vec<Vec<Row>>{}",
            expand_return!(Vec<Row>).field_self_into().bulk(),
            "fold: (A, kotlin.collections.List<test.Row>) -> A",
        ),
        (
            "pub struct Row {pub n:i64}pub fn batches()->Option<Vec<Option<Vec<Row>>>>{}",
            expand_return!(Vec<Row>).field_self_into().bulk(),
            "fold: (A, kotlin.collections.List<test.Row>?) -> A",
        ),
        (
            "pub struct Row {pub n:i64}pub fn batches()->&'static [Row]{}",
            expand_return!(Row).field_self_ref().bulk(),
            "fold: (A, test.Row) -> A",
        ),
    ] {
        let model = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!().class(data_class!(Row)).fun(
                    fun!(batches)
                        .expand_return(expansion)
                        .callback_return_only(),
                ),
            )
            .prepare()
            .unwrap();
        assert!(callback_element_is_bulk(&model, "batches_fold"));
        let generated = generate(&model).unwrap();
        assert!(
            generated.kotlin["test/Functions.kt"].contains(signature),
            "{}",
            generated.kotlin["test/Functions.kt"]
        );
        syn::parse_file(&generated.rust).unwrap();
    }
}

#[test]
fn local_callback_output_overrides_replace_bulk_defaults_and_order_is_irrelevant() {
    let source = "pub struct Row {pub n:i64}pub fn packed()->Vec<Row>{}pub fn plain()->Vec<Row>{}";
    let build = |reverse| {
        let local = expand_return!(Row).field_self_into();
        let plain = if reverse {
            fun!(plain).expand_return(local).callback_return_only()
        } else {
            fun!(plain).callback_return_only().expand_return(local)
        };
        JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(false)
            .expand(expand_return!(Row).field_self_into().bulk())
            .package(
                package!()
                    .class(data_class!(Row))
                    .fun(fun!(packed).callback_return())
                    .fun(plain),
            )
            .prepare()
            .unwrap()
    };
    let first = build(false);
    assert!(bulk_result(&first, "packed"));
    assert!(callback_element_is_bulk(&first, "packed_fold"));
    assert!(!callback_element_is_bulk(&first, "plain_fold"));
    let first = generate(&first).unwrap();
    let second = generate(&build(true)).unwrap();
    assert_eq!(first.rust, second.rust);
    assert_eq!(first.kotlin, second.kotlin);
}

#[test]
fn bulk_callback_elements_work_through_getters_and_consuming_field_expansions() {
    let model=JniGen::builder()
        .flat(flat("pub struct Row {pub n:i64,pub text:String}pub struct Batch {pub unused:i64}pub fn rows()->Vec<Row>{}pub fn batch()->Batch{}pub fn items(b:&Batch)->Vec<Row>{}pub fn parts(row:Row)->Row{}"))
        .set_package_prefix("test")
        .expand(expand_return!(Row).field_self_into().bulk())
        .package(package!().class(data_class!(Row))
            .fun(fun!(batch).expand_return(expand_return!(Batch).field(fun!(items))).callback_return())
            .fun(fun!(rows).callback_return_only().expand_return(expand_return!(Row).fields_into(prebindgen_declarations::fields!(parts)).bulk())))
        .prepare().unwrap();
    assert!(bulk_result(&model, "batch"));
    assert!(callback_element_is_bulk(&model, "batch_fold"));
    assert!(callback_element_is_bulk(&model, "rows_fold"));
    let generated = generate(&model).unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("fold: (A, Long, String) -> A"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn whole_collection_bulk_defaults_cannot_enclose_callback_delivery() {
    let error = JniGen::builder()
        .flat(flat("pub struct Row {pub n:i64}pub fn rows()->Vec<Row>{}"))
        .set_package_prefix("test")
        .expand(expand_return!(Vec<Row>).field_self_into().bulk())
        .package(
            package!()
                .class(data_class!(Row))
                .fun(fun!(rows).callback_return_only()),
        )
        .build()
        .err()
        .unwrap();
    assert!(
        error.contains("bulk cannot enclose callback delivery"),
        "{error}"
    );
}

#[test]
fn resolved_callback_elements_can_keep_optional_presence_outside_a_bulk_frame() {
    use prebindgen_declarations::{Output, SequenceDelivery};
    let mut model = JniGen::builder()
        .flat(flat(
            "pub type Owner=String;pub fn rows()->Vec<Option<Vec<Owner>>>{}",
        ))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Owner))
                .fun(fun!(rows).callback_return_only()),
        )
        .prepare()
        .unwrap();
    // Preserve support for direct request-free trees as well as the public
    // declaration spelling, which normalizes this to Bulk(Optional(...)).
    let recipe = Output::Sequence {
        delivery: SequenceDelivery::Callback,
        mode: None,
        element: Box::new(Output::Optional(Box::new(
            Output::Sequence {
                delivery: SequenceDelivery::Value,
                mode: None,
                element: Box::new(Output::Leaf),
            }
            .bulk(),
        ))),
    };
    let conversion =
        prebindgen_declarations::compile_output(&model, &JniCodec, "rows_fold", &recipe, &())
            .unwrap();
    let (_, element, _) = sequence::return_sequence(&conversion.output_outcomes().success).unwrap();
    let fields = group::callback_parameters(element).collect::<Vec<_>>();
    // The optional uses a null frame, so presence stays outside its payload
    // without adding a separate discriminator argument.
    assert!(
        matches!(element, prebindgen_flat::WireNode::Chain {meta:JniChain::Optional(_),child,..} if matches!(child.as_ref(),prebindgen_flat::WireNode::Chain {meta:JniChain::Bulk(_),..}))
    );
    assert_eq!(fields.len(), 1);
    assert_eq!(
        fields
            .iter()
            .filter(|w| matches!(w.kind, JniWire::BulkBuffer { .. }))
            .count(),
        1
    );
    model.function_mut("rows_fold").unwrap().ret.slot = Some(conversion);
    let generated = generate(&model).unwrap();
    assert!(generated.kotlin["test/Functions.kt"]
        .contains("fold: (A, kotlin.collections.List<test.Owner>?) -> A"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn primitive_callback_rows_omit_local_frames_including_handle_and_enum_wires() {
    let generated=JniGen::builder()
        .flat(flat("pub enum Code {Zero,Seven} pub struct Row {pub flag:bool,pub byte:i8,pub unsigned_byte:u8,pub short:i16,pub unsigned_short:u16,pub number:i32,pub unsigned_number:u32,pub long:i64,pub unsigned_long:u64,pub signed_size:isize,pub size:usize,pub float:f32,pub double:f64,pub character:char,pub code:Code}pub fn rows()->Vec<Row>{}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().class(data_class!(Row)).class(enum_class!(Code)).fun(fun!(rows).callback_return_only()))
        .build().unwrap();
    assert!(!generated.rust.contains("LocalFrame::new"));
    assert!(!generated.rust.contains("callback_frame"));
    assert!(generated.rust.contains("call_method_unchecked"));
    syn::parse_file(&generated.rust).unwrap();
    let generated=JniGen::builder()
        .flat(flat("pub type Owner=String;pub struct Row {pub first:Option<Owner>,pub second:Owner}pub fn rows()->Vec<Row>{}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().class(ptr_class!(Owner)).class(data_class!(Row)).fun(fun!(rows).callback_return_only()))
        .build().unwrap();
    assert!(!generated.rust.contains("LocalFrame::new"));
    assert!(generated.rust.contains("call_method_unchecked"));
    assert!(!generated.rust.contains("callback_frame"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn callback_rows_keep_local_frames_for_every_reference_encoding() {
    for ty in ["String", "Vec<u8>", "i128", "u128", "Vec<i64>", "Row"] {
        let generated = JniGen::builder()
            .flat(flat(&format!(
                "pub struct Row {{pub n:i64,pub text:Option<String>}}pub fn rows()->Vec<{ty}>{{}}"
            )))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(false)
            .package(
                package!()
                    .class(data_class!(Row))
                    .fun(fun!(rows).callback_return_only()),
            )
            .build()
            .unwrap();
        syn::parse_file(&generated.rust).unwrap();
        assert!(generated.rust.contains("let ret_w0_callback_frame"), "{ty}");
        assert!(
            generated.rust.contains("drop(ret_w0_callback_frame)"),
            "{ty}"
        );
    }
    // A Rust primitive packed into a bulk frame is a ByteArray on this boundary.
    let generated = JniGen::builder()
        .flat(flat("pub fn rows()->Vec<i64>{}"))
        .set_package_prefix("test")
        .package(
            package!().fun(
                fun!(rows)
                    .callback_return_only()
                    .expand_return(expand_return!(i64).field_self_into().bulk()),
            ),
        )
        .build()
        .unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.rust.contains("let ret_w0_callback_frame"));
    assert!(generated.rust.contains("drop(ret_w0_callback_frame)"));
}

#[test]
fn callback_capture_rows_share_jvm_shapes_without_boxing() {
    let generated=JniGen::builder().flat(flat("pub type Owner=String;pub fn signed()->Vec<i64>{}pub fn unsigned()->Vec<u64>{}pub fn owners()->Vec<Owner>{}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Owner)).fun(fun!(signed).callback_return_only()).fun(fun!(unsigned).callback_return_only()).fun(fun!(owners).callback_return_only()))
        .build().unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    assert_eq!(native.matches("internal class SequenceRow_").count(), 1);
    assert!(native.contains("(@JvmField val c0: Long)"));
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(text.contains("(__pb_fold_row.c0).toULong()"), "{text}");
    assert!(text.contains("drop_Owner_") && text.contains("__pb_fold_row.c0"));
    assert!(!text.contains("__pb_rows.add(kotlin.arrayOf"));
    assert!(!text.contains("__pb_fold_row["));
    assert!(!text.contains("rawRow"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn callback_capture_rows_use_all_jvm_carriers_and_keep_nested_cleanup() {
    let generated=JniGen::builder().flat(flat("pub type Owner=String;pub struct Row {pub b:bool,pub byte:i8,pub short:i16,pub number:i32,pub long:i64,pub float:f32,pub double:f64,pub text:Option<String>,pub bytes:Vec<u8>,pub wide:i128,pub owner:Option<Owner>}pub fn rows()->Option<Vec<Row>>{} pub fn bulk()->Vec<Row>{}pub fn nested()->Vec<Vec<i64>>{}"))
        .set_package_prefix("test").set_bulk_collection_outputs(false)
        .package(package!().class(ptr_class!(Owner)).class(data_class!(Row)).fun(fun!(rows).callback_return_only()).fun(fun!(nested).callback_return_only()).fun(fun!(bulk).callback_return_only().expand_return(expand_return!(Row).field_self_into().bulk())))
        .build().unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    let records = native
        .lines()
        .filter(|line| line.contains("internal class SequenceRow_"))
        .collect::<Vec<_>>()
        .join("\n");
    for ty in [
        "Boolean",
        "Byte",
        "Short",
        "Int",
        "Long",
        "Float",
        "Double",
        "String?",
        "ByteArray?",
        "Array<Any?>?",
    ] {
        assert!(records.contains(&format!(": {ty}")), "{ty}: {records}");
    }
    assert_eq!(native.matches("internal class SequenceRow_").count(), 4);
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(!text.contains("__pb_fold_row["));
    assert!(text.contains("__pb_discard(failure, __pb_processed)"));
    assert!(text.contains("for (__pb_rowIndex in 0 until __pb_rows.size)"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn typed_output_remapping_preserves_nullable_and_longer_cast_types() {
    let wire = prebindgen_flat::Wire {
        name: syn::parse_quote!(value),
        kind: JniWire::scalar(Primitive::I64),
    };
    let mut output = KotlinOutput {
        delivered: vec![
            "row[0] as Long".into(),
            "row[0] as Long?".into(),
            "row[0] as Long ?".into(),
            "row[0] as LongExtra".into(),
            "(row[0] as Long).toULong()".into(),
            "row[0] as Long\nval other = 0".into(),
        ],
        ..Default::default()
    };
    returns::remap(&mut output, "row", &[&wire], 0, &["column[index]".into()]);
    assert_eq!(
        output.delivered,
        [
            "column[index]",
            "column[index] as Long?",
            "column[index] as Long ?",
            "column[index] as LongExtra",
            "(column[index]).toULong()",
            "column[index]\nval other = 0"
        ]
    );
}

#[test]
fn ordinary_output_rows_share_capture_classes_and_use_unboxed_constructors() {
    let generated=JniGen::builder()
        .flat(flat("pub type Owner=String;pub fn signed()->Vec<i64>{}pub fn unsigned()->Vec<u64>{}pub fn owners()->Vec<Owner>{}pub fn fold()->Vec<i64>{}pub fn callback(cb:impl Fn(Vec<i64>)){}"))
        .set_package_prefix("test").set_bulk_collection_outputs(false)
        .package(package!().class(ptr_class!(Owner)).fun(fun!(signed)).fun(fun!(unsigned)).fun(fun!(owners)).fun(fun!(fold).callback_return_only()).fun(fun!(callback)))
        .build().unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    assert_eq!(native.matches("internal class SequenceRow_").count(), 1);
    assert_eq!(
        generated.proguard_rules(),
        "-keep class test.JNINative { *; }\n-keep class test.JNINative$SequenceRow_* { *; }\n-keep class test.JNINative$SequenceSink_* { *; }\n-keep class test.JNINative$SequenceCollector_* { *; }\n-keep interface test.JNINative$SequenceCapacity { *; }\n"
    );
    assert_eq!(generated.rust.matches("static __pb_row_type_").count(), 1);
    assert!(native.contains("::class.java"));
    assert!(generated.rust.contains(".construct("));
    assert!(!generated.rust.contains("handle_object"));
    assert!(!generated.rust.contains("valueOf"));
    assert!(!generated.rust.contains("LocalFrame::new"));
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(text.contains(".c0).toULong()"));
    assert!(generated.rust.contains("OutputRows::new_objects"));
    assert!(generated.rust.contains("get_field_unchecked"));
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn ordinary_output_rows_respect_jvm_constructor_slot_limit() {
    for (longs, ints, typed) in [
        (127, 0, true),
        (127, 1, false),
        (128, 0, false),
        (126, 2, true),
    ] {
        let fields = (0..longs)
            .map(|i| (format!("n{i}"), Primitive::I64))
            .chain((0..ints).map(|i| (format!("x{i}"), Primitive::I32)))
            .map(|(name, primitive)| prebindgen_flat::Wire {
                name: syn::parse_str(&name).unwrap(),
                kind: JniWire::scalar(primitive),
            })
            .collect::<Vec<_>>();
        let values = fields
            .iter()
            .map(|w| {
                let name = &w.name;
                (name.clone(), quote!(#name))
            })
            .collect::<Vec<_>>();
        let element = prebindgen_flat::WireNode::Leaf(fields);
        let output = sequence::output(
            syn::parse_quote!(rows),
            syn::parse_quote!(length),
            syn::parse_quote!(index),
            &element,
            &values,
        )
        .unwrap();
        assert!(
            matches!(output.metadata, JniChain::Sequence(actual) if actual == if typed { JniSequence::TypedBuffered } else { JniSequence::Buffered })
        );
        assert_eq!(output.store.to_string().contains("construct"), typed);
        assert_eq!(output.store.to_string().contains("valueOf"), !typed);
    }
}

#[test]
fn legacy_buffered_schema_keeps_array_decoder_and_input_only_skips_registration() {
    let node = prebindgen_flat::WireNode::Chain {
        controls: vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(rows),
            kind: JniWire::Rows {
                condition: "true".into(),
                null_niche_available: true,
            },
        }],
        child: Box::new(prebindgen_flat::WireNode::Leaf(vec![
            prebindgen_flat::Wire {
                name: syn::parse_quote!(value),
                kind: JniWire::scalar(Primitive::I64),
            },
        ])),
        meta: JniChain::Sequence(JniSequence::Buffered),
    };
    let context = KotlinContext {
        package: "test",
        classes: &BTreeMap::new(),
        result: "packet",
        failure: "failure",
        cleanup_failure: None,
        output_prefix: "result_",
    };
    let output = kotlin::output_kotlin_scoped(&node, &context, None).unwrap();
    assert!(output.materialize.contains("as kotlin.Array<*>"));
    assert!(output.materialize.contains("_row[0] as Long"));
    assert!(!output.materialize.contains("SequenceRow_"));
    let generated = JniGen::builder()
        .flat(flat("pub fn input(values:Vec<i64>)->i64{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(input)))
        .build()
        .unwrap();
    assert!(generated.proguard_rules().is_empty());
    assert!(!generated.rust.contains("OutputRowType"));
    assert!(!generated.kotlin["test/JNINative.kt"].contains("init_OutputRows_"));
}

#[test]
fn callback_collectors_reserve_once_before_rows_without_changing_row_signatures() {
    let generated = JniGen::builder()
        .flat(flat("pub fn rows()->Vec<i64>{}"))
        .set_package_prefix("test")
        .set_bulk_collection_outputs(false)
        .package(package!().fun(fun!(rows).callback_return_only()))
        .build()
        .unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    assert!(native.contains("interface SequenceCapacity { fun reserve(capacity: Int) }"));
    assert!(native.contains(": SequenceCapacity { fun run(v0: Long)"));
    assert!(native.contains("rows.ensureCapacity(capacity)"));
    assert!(native.contains("override fun run(v0: Long) { rows.add("));
    assert!(!native.contains("capacity: Int, v0"));
    let text = &generated.kotlin["test/Functions.kt"];
    assert!(text.contains("val __pb_rows = __pb_sink.rows"));
    assert!(!text.contains("ArrayList<"));
    let row = prebindgen_flat::WireNode::Leaf(vec![prebindgen_flat::Wire {
        name: syn::parse_quote!(value),
        kind: JniWire::scalar(Primitive::I64),
    }]);
    let transport = sequence_callback::transport(
        syn::parse_quote!(rows),
        syn::parse_quote!(length),
        &row,
        &[(syn::parse_quote!(value), quote!(value))],
    )
    .unwrap();
    assert!(transport.begin.to_string().contains("if length >= 64"));
    assert!(transport.begin.to_string().contains("reserve"));
    assert!(!transport.store.to_string().contains("reserve"));
    assert!(!transport.store.to_string().contains("length"));
    assert!(transport
        .begin
        .to_string()
        .contains("__PB_SEQUENCE_CAPACITY"));
    assert!(!transport.begin.to_string().contains(r#""reserve""#));
    assert_eq!(generated.rust.matches(r#""reserve""#).count(), 1);
    syn::parse_file(&generated.rust).unwrap();
}

#[test]
fn bulk_only_collections_need_no_collector_keep_rules() {
    let generated = JniGen::builder()
        .flat(flat("pub fn rows()->Vec<i64>{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(rows)))
        .build()
        .unwrap();
    assert!(!generated.proguard_rules().contains("SequenceSink_"));
    assert!(!generated.proguard_rules().contains("SequenceCollector_"));
    assert!(!generated.proguard_rules().contains("SequenceCapacity"));
    assert!(!generated.rust.contains("__PB_SEQUENCE_CAPACITY"));
    assert!(!generated.kotlin["test/JNINative.kt"].contains("SequenceCapacity"));
}

#[test]
fn collector_capacity_registration_is_shared_and_follows_the_library_loader() {
    let generated = JniGen::builder()
        .flat(flat(
            "pub fn rows()->Vec<i64>{}pub fn text()->Vec<String>{}",
        ))
        .set_package_prefix("test")
        .set_jni_native_init("System.loadLibrary(\"example\")")
        .package(
            package!()
                .fun(fun!(rows).callback_return_only())
                .fun(fun!(text).callback_return_only()),
        )
        .build()
        .unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    assert_eq!(
        native
            .matches("internal interface SequenceCapacity ")
            .count(),
        1
    );
    assert_eq!(native.matches("init { init_SequenceCapacity_").count(), 1);
    assert_eq!(
        generated
            .rust
            .matches("static __PB_SEQUENCE_CAPACITY")
            .count(),
        1
    );
    assert!(
        native.find("System.loadLibrary").unwrap()
            < native.find("init { init_SequenceCapacity_").unwrap()
    );
    assert!(generated
        .proguard_rules()
        .contains("JNINative$SequenceCapacity"));
}

#[test]
fn kotlin_identifiers_only_escape_keywords_and_nonidentifier_characters() {
    for name in ["packetId", "_packet2", "type", "数据", "é", "n١"] {
        assert_eq!(kotlin_identifier(name), name);
    }
    for name in [
        "when",
        "value",
        "field",
        "get",
        "set",
        "suspend",
        "context",
        "constructor",
        "init",
        "class",
        "in",
        "fun",
        "x\u{301}",
        "value-name",
        "2nd",
        "a²",
        "Ⅷ",
    ] {
        assert_eq!(kotlin_identifier(name), format!("`{name}`"));
    }
}

#[test]
fn required_inputs_are_nonnull_and_options_preserve_nullable_containers() {
    let source = "pub type Handle=String; pub enum Choice { A,B } pub fn required(h:&Handle,text:&str,bytes:&[u8],wide:i128,choice:Choice,callback:impl Fn()->String)->i64{} pub fn optional(h:Option<&Handle>,text:Option<&str>,bytes:Option<&[u8]>,wide:Option<i128>,choice:Option<Choice>)->i64{} pub fn rows(input:Vec<Handle>)->i64{} pub fn maybe_rows(input:Option<Vec<Handle>>)->i64{}";
    for bulk in [false, true] {
        let mut rows = fun!(rows);
        let mut maybe_rows = fun!(maybe_rows);
        if bulk {
            rows = rows.expand_param("input", expand_param!(Vec<Handle>).variant_self().bulk());
            maybe_rows = maybe_rows.expand_param(
                "input",
                expand_param!(Option<Vec<Handle>>).variant_self().bulk(),
            );
        }
        let g = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(ptr_class!(Handle))
                    .class(enum_class!(Choice))
                    .fun(fun!(required))
                    .fun(fun!(optional))
                    .fun(rows)
                    .fun(maybe_rows),
            )
            .build()
            .unwrap();
        let functions = &g.kotlin["test/Functions.kt"];
        let required = functions
            .lines()
            .find(|l| l.starts_with("public fun required("))
            .unwrap();
        for ty in [
            "h: test.Handle,",
            "text: String,",
            "bytes: ByteArray,",
            "wide: java.math.BigInteger,",
            "choice: test.Choice,",
            "callback: test.VoidToStringCallback,",
        ] {
            assert!(required.contains(ty), "{required}");
        }
        let optional = functions
            .lines()
            .find(|l| l.starts_with("public fun optional("))
            .unwrap();
        for ty in [
            "h: test.Handle?",
            "text: String?",
            "bytes: ByteArray?",
            "wide: java.math.BigInteger?",
            "choice: test.Choice?",
        ] {
            assert!(optional.contains(ty), "{optional}");
        }
        assert!(functions.contains("input: kotlin.collections.List<test.Handle>,"));
        assert!(functions.contains("input: kotlin.collections.List<test.Handle>?,"));
        assert!(!functions.contains("List<test.Handle?>"));
        assert!(g.kotlin["test/VoidToStringCallback.kt"].contains("run(): String\n"));
    }
}

#[test]
fn selector_reference_slots_are_nullable_for_plain_and_bulk_inputs() {
    let source="pub type Handle=String;pub enum Choice{A,B}pub struct Value{pub text:String}pub fn from_fields(text:String,bytes:Vec<u8>,wide:i128,choice:Choice,h:&Handle)->Value{}pub fn choose(value:Value)->i64{}";
    for bulk in [false, true] {
        let expansion = expand_param!(Value)
            .variant_self()
            .variant(fun!(from_fields));
        let expansion = if bulk { expansion.bulk() } else { expansion };
        let g = JniGen::builder()
            .flat(flat(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(ptr_class!(Handle))
                    .class(enum_class!(Choice))
                    .class(data_class!(Value))
                    .fun(fun!(choose).expand_param("value", expansion)),
            )
            .build()
            .unwrap();
        let signature = g.kotlin["test/Functions.kt"]
            .lines()
            .find(|l| l.starts_with("public fun choose("))
            .unwrap();
        for ty in [
            "valueVariant0: test.Value?",
            "valueVariant1Text: String?",
            "valueVariant1Bytes: ByteArray?",
            "valueVariant1Wide: java.math.BigInteger?",
            "valueVariant1Choice: test.Choice?",
            "valueVariant1H: test.Handle?",
        ] {
            assert!(signature.contains(ty), "{signature}");
        }
    }
}
