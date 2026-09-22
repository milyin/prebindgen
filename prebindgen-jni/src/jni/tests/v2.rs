//! The v2 engine, driven through the ordinary JNI frontend.
//!
//! The declarations are the ones v1 takes — the same `package!`, the same
//! `ptr_class!`, the same `set_*` settings — and the engine is stated
//! explicitly, so a test never depends on the runner's `PREBINDGEN_PIPELINE`.

use prebindgen_registry::pipeline::Pipeline;
use prebindgen_registry_v2::Declaration;

use super::*;

/// A small but complete JNI binding: an opaque handle with a method and a
/// factory, a free package function, and an acknowledged ignore.
fn binding() -> JniGenBuilder {
    JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .set_fun_name_mangle(|_package, name| format!("do{}", capitalize(name)))
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .constructor(prebindgen_registry::fun!(z_thing_new))
                        .method(prebindgen_registry::fun!(z_thing_size)),
                )
                .fun(prebindgen_registry::fun!(z_thing_describe)),
        )
        .ignore(prebindgen_registry::fun!(z_thing_internal))
}

fn capitalize(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn fixture_items() -> Vec<(syn::Item, SourceLocation)> {
    let loc = myflat_loc();
    let sources: &[&str] = &[
        "pub fn z_thing_new() -> ZThing { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_thing_describe(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_thing_internal(t: &ZThing) -> i64 { unimplemented!() }",
    ];
    declare_referenced(
        sources
            .iter()
            .map(|src| (syn::Item::Fn(syn::parse_str(src).unwrap()), loc.clone()))
            .collect::<Vec<_>>(),
    )
}

/// The gate #719 §A names: the whole declaration set reaches v2, and every
/// requested declaration comes back either generated or skipped, with each
/// skip saying what the value waits on and where the walk stopped.
#[test]
fn every_declared_element_is_accounted_for() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");

    // The handle class is emitted — an address in a `Long`, with its release;
    // the members and the borrowed-handle function are not.
    let skipped: Vec<(String, &str, String)> = generated
        .skipped()
        .iter()
        .map(|(declaration, skip)| {
            (
                declaration.to_string(),
                skip.capability.as_str(),
                skip.path(),
            )
        })
        .collect();
    assert_eq!(
        skipped,
        [
            (
                "fn:z_thing_new".to_string(),
                "unsupported.jni.constructor",
                "fn:z_thing_new".to_string()
            ),
            (
                "fn:z_thing_size".to_string(),
                "unsupported.jni.carrier",
                "fn:z_thing_size -> param 0".to_string()
            ),
            (
                // A borrowed handle is a reference, which no v2 carrier holds
                // yet.
                "fn:z_thing_describe".to_string(),
                "unsupported.jni.carrier",
                "fn:z_thing_describe -> param 0".to_string()
            ),
        ]
    );

    // What is left is the handle class, placed where this adapter's package
    // prefix puts it rather than where v2 would guess.
    let kotlin = kotlin_of(&generated, "jnigen_v2_accounted");
    assert!(kotlin.contains("package io.test.jni.thing"), "{kotlin}");
    assert!(kotlin.contains("public class ZThing"), "{kotlin}");
}

/// The Kotlin this generation writes, as one string.
fn kotlin_of(generated: &JniGen, name: &str) -> String {
    let dir = unique_test_dir(name);
    let _ = std::fs::remove_dir_all(&dir);
    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    let kotlin: String = written
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    let _ = std::fs::remove_dir_all(&dir);
    kotlin
}

/// The implemented subset, through the ordinary frontend: a `data_class!` of
/// scalars and a `fun!` taking it by value. The Rust reads the object's
/// properties through the environment and exports the harness's symbol; the
/// Kotlin declares the class, the native method on the harness, and the
/// function a caller uses — under the package prefix and the function-name
/// hook this binding set.
#[test]
fn a_data_class_and_a_function_over_it_are_emitted() {
    let loc = myflat_loc();
    let sources: &[&str] = &[
        "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
        "pub fn stamp_sum(stamp: Stamp) -> i64 { unimplemented!() }",
        "pub fn stamp_new(secs: i64) -> Stamp { unimplemented!() }",
    ];
    let items = declare_referenced(
        sources
            .iter()
            .map(|src| (syn::parse_str::<syn::Item>(src).unwrap(), loc.clone()))
            .collect::<Vec<_>>(),
    );
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .set_jni_native_init("io.test.jni.Lib.load()")
        .items(items)
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp).name("Timestamp"))
                .fun(prebindgen_registry::fun!(stamp_sum))
                .fun(prebindgen_registry::fun!(stamp_new)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert_eq!(generated.skipped().len(), 1, "{:?}", generated.skipped());

    let dir = unique_test_dir("jnigen_v2_emitted");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    let compact: String = rust.split_whitespace().collect();
    assert!(
        compact.contains(
            "pubextern\"system\"fnJava_io_test_jni_JNINative_stampSum(mutenv:jni::JNIEnv<'_>,\
             _this:jni::objects::JObject<'_>,stamp:jni::objects::JObject<'_>,)->jni::sys::jlong{"
        ),
        "{rust}"
    );
    assert!(
        compact.contains("env.call_method(&stamp,\"getSecs\",\"()J\",&[])"),
        "{rust}"
    );
    assert!(
        compact.contains("myflat::Stamp{secs:v0,nanos:v1,}"),
        "{rust}"
    );
    assert!(
        compact.contains("report_jni_error(&mutenv,error)"),
        "{rust}"
    );
    assert!(
        !rust.contains("stampNew"),
        "a struct leaving Rust is a skip: {rust}"
    );

    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    assert_eq!(written.len(), 1, "{written:?}");
    let kotlin = std::fs::read_to_string(&written[0]).unwrap();
    for line in [
        "package io.test.jni",
        "public data class Timestamp(val secs: Long, val nanos: Long)",
        "public fun stampSum(stamp: Timestamp): Long = JNINative.stampSum(stamp)",
        "internal object JNINative {",
        "io.test.jni.Lib.load()",
        "external fun stampSum(stamp: Timestamp): Long",
    ] {
        assert!(
            kotlin.lines().any(|emitted| emitted.trim() == line),
            "missing `{line}`:\n{kotlin}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Class members are separately selected: the class and each of its methods
/// are declarations in their own right, so one can be skipped without the
/// others — and each waits on the declarator it was declared with, which is
/// what its capability code names.
#[test]
fn class_members_are_elements_of_their_own() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let capability = |id: &str| {
        generated
            .skipped()
            .iter()
            .find(|(declaration, _)| declaration.to_string() == id)
            .map(|(_, skip)| skip.capability.as_str().to_string())
            .unwrap_or_default()
    };
    // The class itself is emitted; each member waits on its own declarator.
    assert_eq!(capability("type:ZThing"), "");
    assert_eq!(capability("fn:z_thing_new"), "unsupported.jni.constructor");
    // Both take a borrowed handle, which no v2 carrier holds yet — the method
    // and the package function alike, each refused on its own.
    assert_eq!(capability("fn:z_thing_size"), "unsupported.jni.carrier");
    assert_eq!(capability("fn:z_thing_describe"), "unsupported.jni.carrier");
}

/// An ignore silences v1's undeclared-item warning and nothing else: under v2
/// the item is not declared, so nothing is generated for it and nothing
/// accounts for it, and it stays in the model, where a declared item may still
/// depend on it.
#[test]
fn an_ignore_does_not_reach_the_engine() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    assert!(
        !generated
            .skipped()
            .iter()
            .any(|(declaration, _)| declaration.to_string() == "fn:z_thing_internal"),
        "{:?}",
        generated.skipped()
    );
    assert!(
        !kotlin_of(&generated, "jnigen_v2_ignored").contains("zThingInternal"),
        "nothing is generated for it either"
    );
}

/// Both writers work under an engine that generated nothing. A zero-output plan
/// is still a plan, and a build script calls `write_rust` and `write_kotlin`
/// without asking which engine ran.
#[test]
fn the_ordinary_writers_run_under_v2() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let dir = unique_test_dir("jnigen_v2_writers");
    let _ = std::fs::remove_dir_all(&dir);

    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let contents = std::fs::read_to_string(&rust).unwrap();
    assert!(
        contents.starts_with("// Generated by prebindgen v2"),
        "the file names the engine that produced it: {contents}"
    );

    // The one thing lowered here is the handle class, in its package, with
    // the native method that frees one on the harness in the base package.
    let kotlin_root = dir.join("kotlin");
    let written = generated.write_kotlin(&kotlin_root).expect("write_kotlin");
    let kotlin: Vec<String> = written
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    let kotlin = kotlin.join("\n");
    assert!(
        kotlin.contains("public class ZThing internal constructor(ptr: Long) {"),
        "the handle class:\n{kotlin}"
    );
    assert!(
        kotlin.contains("external fun freeZThing(ptr: Long)"),
        "and its release on the harness:\n{kotlin}"
    );
    assert!(
        contents.contains("Java_io_test_jni_JNINative_freeZThing"),
        "with the wrapper the JVM binds it to:\n{contents}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// A binding-local fn is an entity the binding stated, and is declared like
/// any other — at a class member and at a package function. One with a
/// signature the engine can carry is generated, calling the helper where its
/// path says it is.
#[test]
fn a_binding_local_fn_is_an_entity_and_is_generated() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing).method(
                        prebindgen_registry::fun!(crate::local_size)
                            .sig(prebindgen_registry::sig!((t: &ZThing) -> i64)),
                    ),
                )
                .fun(
                    prebindgen_registry::fun!(crate::local_tag)
                        .sig(prebindgen_registry::sig!(() -> i64)),
                ),
        )
        .build_with(Pipeline::V2)
        .expect("a binding-local fn needs no captured item");
    // Once each: a helper is stated where it is bound, and declaring the
    // helper itself again would be the binding saying one thing twice.
    // `local_tag` takes nothing and returns an `i64`, so it is a wrapper the
    // engine can build; `local_size` takes a borrowed handle and is not.
    let skipped: Vec<String> = generated
        .skipped()
        .iter()
        .map(|(declaration, _)| declaration.to_string())
        .collect();
    assert_eq!(skipped, ["fn:local_size"]);

    let dir = unique_test_dir("jnigen_v2_local_fn");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    assert!(rust.contains("crate::local_tag()"), "{rust}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A class declaration need not name a captured type. `ptr_class!` over one
/// the source never exported is a handle; `data_class!` over one is a
/// reported skip, since there are no fields to read — and neither is an
/// error, which a function naming nothing captured is.
#[test]
fn a_class_over_a_type_the_source_never_exported_is_a_skip_not_an_error() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(String))
                .class(crate::data_class!(Absent)),
        )
        .build_with(Pipeline::V2)
        .expect("a declared class the source did not capture is not an error");
    // The handle is generated; the data class over a type nothing describes
    // has no fields to read.
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Absent");
    assert_eq!(skip.capability.as_str(), "unsupported.jni.not_a_struct");
}

/// A declared key may carry arguments the item does not, and the class is
/// planned over the type as declared: `ptr_class!(String<'static>)` — a
/// lifetime on a type the source never exported — names the item `String`,
/// and its handle and release are over `String<'static>`, which is where
/// the frontend recorded the choice.
#[test]
fn a_ptr_class_over_a_key_with_arguments_is_planned_as_declared() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(crate::package!("thing").class(crate::ptr_class!(Wrapper<'static>)))
        .build_with(Pipeline::V2)
        .expect("a parameterized key names its item");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("jnigen_v2_parameterized_key");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    assert!(rust.contains("as *mut crate::Wrapper<'static>"), "{rust}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// One function placed twice — as a package `fun` and as the `val` a
/// `constant!(X).fun(..)` reads through — is two outputs: two declarations of
/// one entity, each planned and accounted for on its own. Neither is lowered
/// yet, so each comes back as the capability its own declarator waits for.
#[test]
fn a_function_placed_twice_is_two_outputs() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing))
                .fun(prebindgen_registry::fun!(z_thing_new))
                .fun(prebindgen_registry::fun!(z_thing_describe))
                .constant(
                    crate::constant!(THE_SIZE).fun(prebindgen_registry::fun!(z_thing_describe)),
                ),
        )
        .build_with(Pipeline::V2)
        .expect("two placements of one function are two declarations");
    // `z_thing_describe` twice: the package function, whose parameter has no
    // carrier, and the `val` over the same call.
    let skipped: Vec<(String, &str)> = generated
        .skipped()
        .iter()
        .map(|(declaration, skip)| (declaration.to_string(), skip.capability.as_str()))
        .collect();
    assert_eq!(
        skipped,
        [
            ("fn:z_thing_describe".to_string(), "unsupported.jni.carrier"),
            ("fn:z_thing_describe".to_string(), "unsupported.jni.carrier"),
        ],
        "one function, two placements, two skips"
    );
}

/// `constant!(X).fun(fun!(f))` surfaces a Kotlin `val` backed by a nullary
/// function. The declaration is the function's — the `val` is what this target
/// shows the call as — so it resolves against the captured function and is
/// planned as one.
#[test]
fn a_function_backed_constant_is_the_functions_declaration() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing))
                .constant(
                    crate::constant!(THE_SIZE).fun(prebindgen_registry::fun!(z_thing_describe)),
                ),
        )
        .build_with(Pipeline::V2)
        .expect("a function-backed constant resolves");
    // The declaration is the function's; that the target shows the call as a
    // `val` is its own choice, recorded beside it.
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert!(matches!(
        declaration,
        Declaration::Function(ident) if ident == "z_thing_describe"
    ));
    // And it is planned from that function: what stops it is the value its
    // parameter crosses as, blamed on the parameter, rather than the engine
    // turning away everything declared as a constant.
    assert_eq!(skip.dependency_path, ["fn:z_thing_describe", "param 0"]);
}

/// The `Stamp` fixture the edge-case tests below build on: a struct of two
/// `i64`s, a function taking it, and whatever `extra` items a test adds.
fn stamp_items(extra: &[&str]) -> Vec<(syn::Item, SourceLocation)> {
    let loc = myflat_loc();
    let mut sources = vec![
        "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
        "pub fn stamp_sum(stamp: Stamp) -> i64 { unimplemented!() }",
    ];
    sources.extend_from_slice(extra);
    declare_referenced(
        sources
            .iter()
            .map(|src| (syn::parse_str::<syn::Item>(src).unwrap(), loc.clone()))
            .collect::<Vec<_>>(),
    )
}

/// The skip recorded for the declaration that prints as `id`, as
/// `(capability, path)`.
fn skip_of(generated: &JniGen, id: &str) -> (String, String) {
    generated
        .skipped()
        .iter()
        .find(|(declaration, _)| declaration.to_string() == id)
        .map(|(_, skip)| (skip.capability.as_str().to_string(), skip.path()))
        .unwrap_or_else(|| panic!("{id} is not skipped"))
}

/// A setting v2 does not lower — a per-function or a type-level boundary
/// declaration — refuses the function it applies to, rather than being dropped
/// and the default interface emitted as if it were the one asked for. A
/// type-level declaration takes effect at boundaries, so the class it names is
/// still emitted; and it applies whether or not its type has a class at all.
#[test]
fn an_unimplemented_setting_refuses_its_function_rather_than_being_dropped() {
    let items = || {
        stamp_items(&[
            "pub fn stamp_new(secs: i64) -> Stamp { unimplemented!() }",
            "pub fn from_parts(a: i32, b: i32) -> i64 { unimplemented!() }",
            "pub fn take_value(value: i64) -> i64 { unimplemented!() }",
            "pub fn give_value() -> i64 { unimplemented!() }",
        ])
    };
    // Per function.
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items())
        .package(
            crate::package!().class(crate::data_class!(Stamp)).fun(
                prebindgen_registry::fun!(stamp_sum).expand_param(
                    "stamp",
                    prebindgen_registry::expand_param!(Stamp)
                        .variant(prebindgen_registry::fun!(stamp_new)),
                ),
            ),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert_eq!(
        skip_of(&generated, "fn:stamp_sum").0,
        "unsupported.jni.expand_param"
    );
    assert_eq!(
        generated.skipped().len(),
        1,
        "the class alone is generated: {:?}",
        generated.skipped()
    );

    // For a declared class: the function taking it is refused, the class stays.
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items())
        .expand(
            prebindgen_registry::expand_param!(Stamp).variant(prebindgen_registry::fun!(stamp_new)),
        )
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp))
                .fun(prebindgen_registry::fun!(stamp_sum)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert_eq!(
        skip_of(&generated, "fn:stamp_sum"),
        (
            "unsupported.jni.expand_param".to_string(),
            "fn:stamp_sum".to_string()
        )
    );
    assert_eq!(
        generated.skipped().len(),
        1,
        "the class alone is generated: {:?}",
        generated.skipped()
    );

    // For a scalar with no class: a parameter of that type, and a result of it.
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items())
        .expand(
            prebindgen_registry::expand_param!(i64).variant(prebindgen_registry::fun!(from_parts)),
        )
        .expand(
            prebindgen_registry::expand_return!(i64).field(prebindgen_registry::fun!(take_value)),
        )
        .package(
            crate::package!()
                .fun(prebindgen_registry::fun!(take_value))
                .fun(prebindgen_registry::fun!(give_value)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert_eq!(
        skip_of(&generated, "fn:take_value").0,
        "unsupported.jni.expand_param"
    );
    assert_eq!(
        skip_of(&generated, "fn:give_value").0,
        "unsupported.jni.expand_return"
    );
    assert_eq!(
        generated.skipped().len(),
        2,
        "both functions are refused: {:?}",
        generated.skipped()
    );
}

/// Two packages may each export a function called `value`; the harness has one
/// namespace, so its native methods are named from the Rust identifier — as
/// v1 names them — and the public function's `.name()` does not reach it.
#[test]
fn a_public_name_does_not_name_the_native_method() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("example")
        .items(stamp_items(&[
            "pub fn first_value() -> i64 { unimplemented!() }",
            "pub fn second_value() -> i64 { unimplemented!() }",
        ]))
        .package(crate::package!("a").fun(prebindgen_registry::fun!(first_value).name("value")))
        .package(crate::package!("b").fun(prebindgen_registry::fun!(second_value).name("value")))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("jnigen_v2_two_values");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = std::fs::read_to_string(generated.write_rust(dir.join("b.rs")).unwrap()).unwrap();
    assert!(
        rust.contains("fn Java_example_JNINative_firstValue("),
        "{rust}"
    );
    assert!(
        rust.contains("fn Java_example_JNINative_secondValue("),
        "{rust}"
    );
    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("two packages, no clash");
    let kotlin: String = written
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    assert!(
        kotlin.contains("public fun value(): Long = example.JNINative.firstValue()"),
        "{kotlin}"
    );
    assert!(
        kotlin.contains("public fun value(): Long = example.JNINative.secondValue()"),
        "{kotlin}"
    );
    assert!(
        kotlin.contains("external fun firstValue(): Long"),
        "{kotlin}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A source parameter may be called `env` or `_this`; the two parameters the
/// JVM adds step aside rather than shadowing it.
#[test]
fn the_jvm_parameters_are_named_around_the_source_ones() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("example")
        .items(stamp_items(&[
            "pub fn stamp_env(env: Stamp, _this: i64) -> i64 { unimplemented!() }",
        ]))
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp))
                .fun(prebindgen_registry::fun!(stamp_env)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let dir = unique_test_dir("jnigen_v2_env_param");
    let rust = std::fs::read_to_string(generated.write_rust(dir.join("b.rs")).unwrap()).unwrap();
    let compact: String = rust.split_whitespace().collect();
    assert!(
        compact.contains(
            "(mutenv_:jni::JNIEnv<'_>,_this_:jni::objects::JObject<'_>,env:jni::objects::JObject<'_>,\
             _this:jni::sys::jlong,)"
        ),
        "{rust}"
    );
    assert!(
        compact.contains("env_.call_method(&env,\"getSecs\""),
        "{rust}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Kotlin's JVM accessor rule: a property starting with `is` followed by
/// anything but a lowercase letter keeps its name as the getter, whatever its
/// type. Reading `getIsReady` on such a class is a `NoSuchMethodError` at run
/// time, which no compiler catches.
#[test]
fn an_is_prefixed_property_is_read_through_its_own_name() {
    let loc = myflat_loc();
    let items = declare_referenced(vec![
        (
            syn::parse_str(
                "pub struct Flags { pub isReady: i64, pub is_set: i64, pub island: i64, \
                 pub is: i64, pub isé: i64, pub aé: i64 }",
            )
            .unwrap(),
            loc.clone(),
        ),
        (
            syn::parse_str("pub fn flags_sum(flags: Flags) -> i64 { unimplemented!() }").unwrap(),
            loc,
        ),
    ]);
    let generated = JniGenBuilder::new()
        .set_package_prefix("example")
        .items(items)
        .package(
            crate::package!()
                .class(crate::data_class!(Flags))
                .fun(prebindgen_registry::fun!(flags_sum)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let dir = unique_test_dir("jnigen_v2_is_getter");
    let rust = std::fs::read_to_string(generated.write_rust(dir.join("b.rs")).unwrap()).unwrap();
    // What `kotlinc` names each accessor, checked with `javap`: a bare `is`
    // gets the prefix, a non-ASCII suffix does not, and a non-ASCII name is
    // not an `is` at all.
    for getter in [
        "\"isReady\"",
        "\"is_set\"",
        "\"getIsland\"",
        "\"getIs\"",
        "\"isé\"",
        "\"getAé\"",
    ] {
        assert!(rust.contains(getter), "missing {getter}:\n{rust}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A Rust identifier that is a Kotlin keyword, or a raw identifier, is spelled
/// the way Kotlin can read it — in the declaration, in the delegating call and
/// in the getter name alike — rather than failing in the Kotlin writer after
/// planning reported the declaration emitted.
#[test]
fn keyword_and_raw_identifiers_are_spelled_for_kotlin() {
    let loc = myflat_loc();
    let items = declare_referenced(vec![
        (
            syn::parse_str("pub struct Kind { pub r#fun: i64 }").unwrap(),
            loc.clone(),
        ),
        (
            syn::parse_str("pub fn kind_of(when: Kind) -> i64 { unimplemented!() }").unwrap(),
            loc,
        ),
    ]);
    let generated = JniGenBuilder::new()
        .set_package_prefix("example")
        .items(items)
        .package(
            crate::package!()
                .class(crate::data_class!(Kind))
                .fun(prebindgen_registry::fun!(kind_of)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let dir = unique_test_dir("jnigen_v2_keywords");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = std::fs::read_to_string(generated.write_rust(dir.join("b.rs")).unwrap()).unwrap();
    assert!(rust.contains("\"getFun\""), "{rust}");
    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("valid Kotlin");
    let kotlin = std::fs::read_to_string(&written[0]).unwrap();
    assert!(
        kotlin.contains("data class Kind(val `fun`: Long)"),
        "{kotlin}"
    );
    assert!(
        kotlin.contains("fun kindOf(`when`: Kind): Long = JNINative.kindOf(`when`)"),
        "{kotlin}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// An `unsafe fn` is refused: the wrapper is a safe function and the call it
/// renders is a plain one, and hiding the contract in an `unsafe` block would
/// not establish it.
#[test]
fn an_unsafe_source_function_is_a_reported_skip() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("example")
        .items(stamp_items(&[
            "pub unsafe fn stamp_raw(stamp: Stamp) -> i64 { unimplemented!() }",
        ]))
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp))
                .fun(prebindgen_registry::fun!(stamp_raw)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert_eq!(
        skip_of(&generated, "fn:stamp_raw").0,
        "unsupported.fn.unsafe"
    );
}

/// Selecting v1 explicitly still runs v1: the same declarations, the whole
/// existing surface, and nothing left out.
#[test]
fn v1_is_unchanged_and_reachable_by_name() {
    let generated = binding().build_with(Pipeline::V1).expect("v1 resolves");
    assert_eq!(generated.pipeline(), Pipeline::V1);
    assert!(generated.skipped().is_empty());
    assert!(generated
        .registry()
        .flat()
        .function("z_thing_new")
        .is_some());
}

/// Two placements of one supported function are two wrappers. Each is its own
/// native method on the harness — the harness has one namespace, and two
/// definitions of one `Java_…` symbol would not compile — so a placement past
/// the first is named after where it is placed.
#[test]
fn two_placements_of_one_function_are_two_wrappers() {
    let loc = myflat_loc();
    let sources: &[&str] = &[
        "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
        "pub fn stamp_sum(stamp: Stamp) -> i64 { unimplemented!() }",
    ];
    let items = declare_referenced(
        sources
            .iter()
            .map(|src| (syn::parse_str::<syn::Item>(src).unwrap(), loc.clone()))
            .collect::<Vec<_>>(),
    );
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items)
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp))
                .fun(prebindgen_registry::fun!(stamp_sum)),
        )
        .package(crate::package!("other").fun(prebindgen_registry::fun!(stamp_sum)))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("jnigen_v2_two_placements");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    for symbol in [
        "Java_io_test_jni_JNINative_stampSum",
        "Java_io_test_jni_JNINative_otherStampSum",
    ] {
        assert_eq!(rust.matches(symbol).count(), 1, "{rust}");
    }

    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    let kotlin: String = written
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    for line in [
        "public fun stampSum(stamp: Stamp): Long = JNINative.stampSum(stamp)",
        "public fun stampSum(stamp: Stamp): Long = io.test.jni.JNINative.otherStampSum(stamp)",
        "external fun stampSum(stamp: Stamp): Long",
        "external fun otherStampSum(stamp: Stamp): Long",
    ] {
        assert!(
            kotlin.lines().any(|emitted| emitted.trim() == line),
            "missing `{line}`:\n{kotlin}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A captured constant exposed in two packages is two outputs, as a function
/// placed twice is. Both are `constant`, which v2 does not lower yet, so both
/// come back as capability skips — under ids of their own, rather than failing
/// the run as one declaration made twice.
#[test]
fn a_constant_exposed_twice_is_two_outputs() {
    let loc = myflat_loc();
    let items = declare_referenced(vec![(
        syn::parse_str::<syn::Item>("pub const THE_SIZE: i64 = 8;").unwrap(),
        loc,
    )]);
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items)
        .package(crate::package!("a").constant(crate::constant!(THE_SIZE)))
        .package(crate::package!("b").constant(crate::constant!(THE_SIZE)))
        .build_with(Pipeline::V2)
        .expect("two placements of one constant are two declarations");
    // One constant, two packages, two outputs — each waiting on the constant
    // lowering v2 does not have.
    let skipped: Vec<(String, &str)> = generated
        .skipped()
        .iter()
        .map(|(declaration, skip)| (declaration.to_string(), skip.capability.as_str()))
        .collect();
    assert_eq!(
        skipped,
        [
            (
                "const:THE_SIZE".to_string(),
                "unsupported.const.not_implemented"
            ),
            (
                "const:THE_SIZE".to_string(),
                "unsupported.const.not_implemented"
            ),
        ]
    );
}

/// A fieldless enum crosses as the number Rust assigns each of its values, and
/// Kotlin sees an `enum class` carrying the same numbers.
///
/// Out of Rust every value names a number, so that direction cannot fail. Into
/// Rust the carrier is an `Int`, which can hold a number no value names — a
/// caller passing one gets the binding failure the wrapper routes, not a
/// silently wrong value.
#[test]
fn a_fieldless_enum_crosses_as_its_number() {
    let loc = myflat_loc();
    let items = declare_referenced(
        [
            "pub enum Priority { Low, High = 7 }",
            "pub fn priority_raise(p: Priority) -> Priority { unimplemented!() }",
        ]
        .iter()
        .map(|src| (syn::parse_str::<syn::Item>(src).unwrap(), loc.clone()))
        .collect::<Vec<_>>(),
    );
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items)
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .fun(prebindgen_registry::fun!(priority_raise)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("jnigen_v2_enum");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    let compact: String = rust.split_whitespace().collect();

    // The wrapper takes and returns the number, and matches one value at a
    // time in both directions.
    assert!(
        compact.contains(
            "priorityRaise(mutenv:jni::JNIEnv<'_>,_this:jni::objects::JObject<'_>,\
                          p:::jni::sys::jint,)->::jni::sys::jint{"
        ),
        "{rust}"
    );
    assert!(
        compact.contains(
            "matchp{0=>::core::result::Result::Ok(myflat::Priority::Low),\
                          7=>::core::result::Result::Ok(myflat::Priority::High),"
        ),
        "{rust}"
    );
    // A number no value names is a binding failure, named for the class.
    assert!(rust.contains("has no value numbered"), "{rust}");
    assert!(
        compact.contains("matchv1{myflat::Priority::Low=>0,myflat::Priority::High=>7,}"),
        "{rust}"
    );

    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    let kotlin: String = written
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    let _ = std::fs::remove_dir_all(&dir);
    for line in [
        "public enum class Priority(public val value: Int) {",
        "LOW(0),",
        // The last entry closes the list, because a companion follows it.
        "HIGH(7);",
        "public fun fromInt(value: Int): Priority = entries.first { it.value == value }",
        // The public function speaks the enum; the harness speaks its number.
        "public fun priorityRaise(p: Priority): Priority = \
         Priority.fromInt(JNINative.priorityRaise(p.value))",
        "external fun priorityRaise(p: Int): Int",
    ] {
        assert!(
            kotlin.lines().any(|emitted| emitted.trim() == line),
            "missing `{line}`:\n{kotlin}"
        );
    }
}

/// A number a Kotlin `Int` cannot hold refuses the enum.
///
/// `#[repr(i64)] enum Priority { Low, High = 2147483648 }` is valid Rust, and
/// the model reads its numbers as the `i64` they are. Neither the `jint` the
/// wrapper matches on nor the `Int` the enum class carries can hold that one,
/// so it is refused — rather than emitted as a literal rustc rejects as
/// overflowing, and an enum entry the JVM would read as a different number.
#[test]
fn a_number_beyond_the_carrier_refuses_the_enum() {
    let loc = myflat_loc();
    let items = declare_referenced(vec![(
        syn::parse_str::<syn::Item>("#[repr(i64)] pub enum Priority { Low, High = 2147483648 }")
            .unwrap(),
        loc,
    )]);
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items)
        .package(crate::package!().class(crate::enum_class!(Priority)))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Priority");
    assert_eq!(skip.capability.as_str(), "unsupported.jni.enum_range");
}

/// A class declared with an interface is refused rather than emitted without
/// it.
///
/// v1 adds `.implements(..)` to the class's supertypes and emits the
/// `.interface()` it generates. v2 writes neither, so a class it emitted
/// would compile and be missing the supertype the binding asked for.
#[test]
fn a_class_with_an_interface_is_refused() {
    let loc = myflat_loc();
    let items = declare_referenced(vec![(
        syn::parse_str::<syn::Item>("pub enum Priority { Low, High }").unwrap(),
        loc,
    )]);
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(items)
        .package(crate::package!().class(crate::enum_class!(Priority).implements("io.test.Ranked")))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Priority");
    assert_eq!(skip.capability.as_str(), "unsupported.jni.interface");
}
