use super::*;

/// Two fns returning the same type under different output decompositions:
/// the default one and a per-fn `.default_return_field(...)` inline field list.
/// Each gets its own builder interface.
#[test]
fn inline_output_gets_own_builder() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_make_a() -> ZThing { unimplemented!() }",
        "pub fn z_make_b() -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new(
        JniGenConfig::new()
            .source_module(syn::parse_quote!(myflat))
            .package_prefix("io.test.jni"),
    )
    .package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .fun(crate::fun!(z_thing_name).name("name"))
                    .fun(crate::fun!(z_thing_size).name("size"))
                    // Default output: name + size (2 leaves ⇒ builder callback).
                    .default_return_field(crate::fun!(z_thing_name).name("name"))
                    .default_return_field(crate::fun!(z_thing_size).name("size")),
            )
            .fun(crate::fun!(z_make_a))
            // Per-fn inline fields: name + size + name again (different shape). The
            // third field reuses the `z_thing_name` accessor but must carry a
            // distinct (literal) leaf name — duplicate names are a hard error.
            .fun(
                crate::fun!(z_make_b)
                    .return_field(crate::fun!(z_thing_name).name("name"))
                    .return_field(crate::fun!(z_thing_size).name("size"))
                    .return_field(crate::fun!(z_thing_name).name("name2")),
            ),
    );

    let dir = unique_test_dir("jnigen_inline_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Each extern names its own builder interface: the canonical
    // `ZThingBuilder` for z_make_a, the per-fn `ZThingZMakeBBuilder`.
    assert!(rc.contains("io/test/jni/thing/ZThingBuilder"), "{rust}");
    assert!(
        rc.contains("io/test/jni/thing/ZThingZMakeBBuilder"),
        "{rust}"
    );

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Canonical builder: (name, size); inline builder: (name, size, name2).
    assert!(
        all.contains("funinterfaceZThingBuilder<outR>{publicfunrun(name:String,size:Long):R"),
        "{all}"
    );
    assert!(
        all.contains(
            "funinterfaceZThingZMakeBBuilder<outR>{publicfunrun(name:String,size:Long,name2:String):R"
        ),
        "{all}"
    );
    // Wrappers take their own builder types.
    assert!(all.contains("build:ZThingBuilder<R>"), "{all}");
    assert!(all.contains("build:ZThingZMakeBBuilder<R>"), "{all}");
}

/// Error decomposition is the OUTPUT decomposition with a fixed leading `je`:
/// the same record kinds work — an identity record (the error itself as an
/// owned handle), plain accessors, and accessors nested through `Option`
/// (spliced child decomposition, nullable leaves). The ze params are typed
/// exactly like a builder's; on a binding error the native side fills typed
/// defaults (closed handle, "", null for nullable).
#[test]
fn error_unwrap_universal_records() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_err_message(e: &ZErr) -> String { unimplemented!() }",
        "pub fn z_err_detail(e: &ZErr) -> Option<&ZDetail> { unimplemented!() }",
        "pub fn z_detail_code(d: &ZDetail) -> i32 { unimplemented!() }",
        "pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new(
        JniGenConfig::new()
            .source_module(syn::parse_quote!(myflat))
            .package_prefix("io.test.jni"),
    )
    .package(
        crate::package!("errors")
            .class(
                crate::ptr_class!(ZDetail)
                    .fun(crate::fun!(z_detail_code).name("code"))
                    .default_return_field(crate::fun!(z_detail_code).name("code")),
            )
            .class(
                crate::ptr_class!(ZErr)
                    .fun(crate::fun!(z_err_message).name("message"))
                    .fun(crate::fun!(z_err_detail).name("detail"))
                    // Canonical error decomposition: the owned error handle itself,
                    // its message, and the Option-nested detail spliced to its code leaf.
                    .default_return_field_self()
                    .default_return_field(crate::fun!(z_err_message).name("message"))
                    .default_return_field(crate::fun!(z_err_detail).name("detail")),
            )
            .fun(crate::fun!(z_fallible)),
    );

    let dir = unique_test_dir("jnigen_err_universal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Handler descriptor: typed handle class, non-null String, BOXED nullable
    // Integer for the Option-nested code — exactly the builder typing.
    assert!(
        rc.contains(
            "\"(Ljava/lang/String;JLjava/lang/String;Ljava/lang/Integer;)Ljava/lang/Object;\""
        ),
        "{rust}"
    );
    // Domain-error arm: the SAME shared leaf encoder — owned identity moves
    // the error into a boxed handle, the nested Option accessor unwraps via
    // a match.
    assert!(rc.contains("std::boxed::Box::new(__de)"), "{rust}");
    assert!(rc.contains("matchmyflat::z_err_detail(&__de)"), "{rust}");
    // Binding-error defaults: zeroed jlong for the handle (no native
    // construction), empty string, null for the nullable leaf — built lazily
    // in the __ze_defaults closure.
    assert!(
        !rc.contains("env.new_object(\"io/test/jni/errors/ZErr\""),
        "{rust}"
    );
    assert!(rc.contains("env.new_string(\"\")"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Builder-typed handler interface.
    assert!(
        all.contains(
            "funinterfaceZErrHandler<outR>{publicfunrun(je:String?,handle:ZErr,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    // Raw twin carries the jlong handle; the wrapper captures raw and wraps
    // on redispatch.
    assert!(
        all.contains(
            "funinterfaceZErrHandlerRaw<outR>{publicfunrun(je:String?,handle:Long,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    assert!(
        all.contains("returnonError.run(__cap.je,ZErr(__cap.ze0!!),__cap.ze1!!,__cap.ze2)"),
        "{all}"
    );
    // Zero-alloc thread-local capture holder generated for the error handler
    // (no per-call SAM lambda / Ref-boxed vars); the wrapper uses acquire().
    assert!(
        all.contains("internalclassZErrHandlerRawCapture:ZErrHandlerRaw<Unit>"),
        "{all}"
    );
    assert!(
        all.contains("val__cap=ZErrHandlerRawCapture.acquire()"),
        "{all}"
    );
    assert!(all.contains("ThreadLocal.withInitial"), "{all}");
    // Wrapper: nullable capture slots, `!!` redispatch for the non-null ze,
    // pass-through for the nullable one — NO `?:` default coalescing.
    assert!(!all.contains("?:\"\""), "{all}");
}

/// `.fun(f)` binds the `&Class` receiver to `this` (dropped from the
/// signature, its handle locked) while keeping the non-receiver params; the
/// fn delegates to the same `JNINative` extern. `.constructor(f)` emits a
/// companion-object factory returning the class. Per-fn
/// `.default_return_field_self()` emits the handle leaf.
#[test]
fn method_constructor_and_inline_field_self() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_thing_rename(t: &ZThing, name: String) -> bool { unimplemented!() }",
        "pub fn z_thing_make(name: String) -> ZThing { unimplemented!() }",
        "pub fn z_get() -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new(
        JniGenConfig::new()
            .source_module(syn::parse_quote!(myflat))
            .package_prefix("io.test.jni"),
    )
    .package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .fun(crate::fun!(z_thing_name).name("name"))
                    // A fun with extra params: `&ZThing` receiver + a `name: String` param.
                    .fun(crate::fun!(z_thing_rename).name("rename"))
                    // A constructor: factory returning ZThing.
                    .constructor(crate::fun!(z_thing_make).name("make")),
            )
            // A free fn whose per-fn inline output decomposes to (handle, name).
            .fun(
                crate::fun!(z_get)
                    .return_field_self()
                    .return_field(crate::fun!(z_thing_name).name("name")),
            ),
    );

    let dir = unique_test_dir("jnigen_method_ctor");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let flat: String = all.split_whitespace().collect();

    // The method binds `this` and keeps the non-receiver `name` param (no `t`).
    assert!(flat.contains("publicfunrename(name:String"), "{all}");
    // The receiver is locked under `this`.
    assert!(all.contains("withSortedHandleLocks(this)"), "{all}");
    // The constructor is a companion-object factory returning ZThing.
    assert!(flat.contains("publiccompanionobject"), "{all}");
    assert!(flat.contains("publicfunmake(name:String"), "{all}");
    // Per-fn inline output: `z_get` decomposes to (handle, name) — a 2-leaf
    // builder (`handle: ZThing, name: String`) from the inline field list.
    assert!(
        flat.contains("publicfunrun(handle:ZThing,name:String)"),
        "{all}"
    );
}
