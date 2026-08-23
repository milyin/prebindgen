use prebindgen_registry::{Conversions, RegistryBuilder};

use super::*;

/// Two fns returning the same type under different output decompositions:
/// the type-level `expand_return!` default and a per-fn `.return_expand(...)`
/// inline field list. Each gets its own builder interface.
#[test]
fn inline_output_gets_own_builder() {
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name"))
                        .method(prebindgen_registry::fun!(z_thing_size).name("size")),
                )
                .fun(prebindgen_registry::fun!(z_make_a))
                // Per-fn inline fields: name + size + name again (different shape). The
                // third field reuses the `z_thing_name` accessor but must carry a
                // distinct (literal) leaf name — duplicate names are a hard error.
                .fun(
                    prebindgen_registry::fun!(z_make_b).expand_return(
                        prebindgen_registry::expand_return!(ZThing)
                            .field(prebindgen_registry::fun!(z_thing_name).name("name"))
                            .field(prebindgen_registry::fun!(z_thing_size).name("size"))
                            .field(prebindgen_registry::fun!(z_thing_name).name("name2")),
                    ),
                ),
        )
        // Default output: name + size (2 leaves ⇒ builder callback). The
        // `name` field inherits its Kotlin name from the class member; `size`
        // sets it explicitly — both paths resolve to the member-equal names.
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field(prebindgen_registry::fun!(z_thing_name))
                .field(prebindgen_registry::fun!(z_thing_size).name("size")),
        );

    let dir = unique_test_dir("jnigen_inline_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
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
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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

/// Domain-error decomposition is the OUTPUT decomposition (issue #45 split off
/// the binding channel, so there is no leading `je`): the same record kinds
/// work — an identity record (the error itself as an owned handle), plain
/// accessors, and accessors nested through `Option` (spliced child
/// decomposition, nullable leaves). The ze params are typed exactly like a
/// builder's; a binding/system failure goes to the separate `onBindingError`
/// (`JniErrorHandler`) channel, so there are no fabricated defaults.
#[test]
fn error_unwrap_universal_records() {
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("errors")
                .class(
                    crate::ptr_class!(ZDetail)
                        .method(prebindgen_registry::fun!(z_detail_code).name("code")),
                )
                .class(
                    crate::ptr_class!(ZErr)
                        .method(prebindgen_registry::fun!(z_err_message).name("message"))
                        .method(prebindgen_registry::fun!(z_err_detail).name("detail")),
                )
                .fun(prebindgen_registry::fun!(z_fallible)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZDetail)
                .field(prebindgen_registry::fun!(z_detail_code)),
        )
        // Canonical error decomposition: the owned error handle itself, its
        // message, and the Option-nested detail spliced to its code leaf.
        // Field names inherit from the class members.
        .expand(
            prebindgen_registry::expand_return!(ZErr)
                .field_self()
                .field(prebindgen_registry::fun!(z_err_message))
                .field(prebindgen_registry::fun!(z_err_detail)),
        );

    let dir = unique_test_dir("jnigen_err_universal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Domain handler descriptor (`__DSINK_DESCR`): typed handle jlong, non-null
    // String, BOXED nullable Integer for the Option-nested code — exactly the
    // builder typing, with NO leading `je` String (that is the binding channel).
    assert!(
        rc.contains("\"(JLjava/lang/String;Ljava/lang/Integer;)Ljava/lang/Object;\""),
        "{rust}"
    );
    // Binding channel descriptor (`__SINK_DESCR`): the base `JniErrorHandler`.
    assert!(
        rc.contains("\"(Ljava/lang/String;)Ljava/lang/Object;\""),
        "{rust}"
    );
    // Domain-error arm: the SAME shared leaf encoder — owned identity moves
    // the error into a boxed handle, the nested Option accessor unwraps via
    // a match — delivered through `signal_domain_error` (no `je`, no defaults).
    assert!(rc.contains("std::boxed::Box::new(__de)"), "{rust}");
    assert!(rc.contains("matchmyflat::z_err_detail(&__de)"), "{rust}");
    assert!(rc.contains("signal_domain_error("), "{rust}");
    // No fabricated-defaults machinery — the binding channel carries only a
    // message string, so there is no `__ze_defaults` closure.
    assert!(!rc.contains("__ze_defaults"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Builder-typed DOMAIN handler interface — no leading `je` (the binding
    // channel is the separate `JniErrorHandler`).
    assert!(
        all.contains(
            "funinterfaceZErrHandler<outR>{publicfunrun(handle:ZErr,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    // Raw twin carries the jlong handle; the wrapper captures raw and wraps
    // on redispatch.
    assert!(
        all.contains(
            "funinterfaceZErrHandlerRaw<outR>{publicfunrun(handle:Long,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    // The fallible wrapper takes BOTH channels; the domain redispatch wraps the
    // captured leaves (no `je`), the binding one forwards its single message.
    assert!(
        all.contains("returnonError.run(ZErr.fromRawPtr(__dcap.ze0!!),__dcap.ze1!!,__dcap.ze2)"),
        "{all}"
    );
    assert!(
        all.contains("if(__bcap.failed)returnonBindingError.run(__bcap.ze0)"),
        "{all}"
    );
    // Zero-alloc thread-local capture holders for BOTH channels (no per-call SAM
    // lambda / Ref-boxed vars); the wrapper uses acquire() on each.
    assert!(
        all.contains("internalclassZErrHandlerRawCapture:ZErrHandlerRaw<Unit>"),
        "{all}"
    );
    assert!(
        all.contains("val__dcap=ZErrHandlerRawCapture.acquire()"),
        "{all}"
    );
    assert!(
        all.contains("val__bcap=JniErrorHandlerCapture.acquire()"),
        "{all}"
    );
    assert!(all.contains("ThreadLocal.withInitial"), "{all}");
    // Wrapper: nullable capture slots, `!!` redispatch for the non-null ze,
    // pass-through for the nullable one — NO `?:` default coalescing.
    assert!(!all.contains("?:\"\""), "{all}");
}

/// `.method(f)` binds the `&Class` receiver to `this` (dropped from the
/// signature, its handle locked) while keeping the non-receiver params; the
/// fn delegates to the same `JNINative` extern. `.constructor(f)` emits a
/// companion-object factory returning the class. Per-fn
/// `.expand_return(...field_self()...)` emits the handle leaf.
#[test]
fn method_constructor_and_inline_field_self() {
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name"))
                        // A method with extra params: `&ZThing` receiver + a `name: String` param.
                        .method(prebindgen_registry::fun!(z_thing_rename).name("rename"))
                        // A constructor: factory returning ZThing.
                        .constructor(prebindgen_registry::fun!(z_thing_make).name("make")),
                )
                // A free fn whose per-fn inline output decomposes to (handle, name).
                .fun(
                    prebindgen_registry::fun!(z_get).expand_return(
                        prebindgen_registry::expand_return!(ZThing)
                            .field_self()
                            .field(prebindgen_registry::fun!(z_thing_name).name("name")),
                    ),
                ),
        );

    let dir = unique_test_dir("jnigen_method_ctor");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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

/// A **rust-side-only** error type: `expand_return!` with NO class
/// declaration. The `Result<_, ZErr>` error channel decomposes the error into
/// its fields (here just the message), the `ZErrHandler` interface lands in
/// the BASE package (no type package exists), and no Kotlin class / `freePtr`
/// is emitted for `ZErr` — the value lives and dies in Rust.
#[test]
fn rust_side_only_error_type() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_err_message(e: &ZErr) -> String { unimplemented!() }",
        "pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_fallible)))
        // No class declaration for ZErr anywhere — rust-side-only. The field
        // name is explicit (no class member to inherit from).
        .expand(
            prebindgen_registry::expand_return!(ZErr)
                .field(prebindgen_registry::fun!(z_err_message).name("message")),
        );

    let dir = unique_test_dir("jnigen_rust_side_only_err");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // The error decomposition calls the accessor Rust-side...
    assert!(rc.contains("myflat::z_err_message(&__de)"), "{rust}");
    // ...and no freePtr destructor exists for ZErr (no opaque handle).
    assert!(!rc.contains("ZErr_1freePtr"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Domain handler in the BASE package with the decomposed message field (no
    // leading `je`); no ZErr class anywhere.
    assert!(
        all.contains("funinterfaceZErrHandler<outR>{publicfunrun(message:String):R"),
        "{all}"
    );
    assert!(!all.contains("classZErr("), "{all}");
    // The handler file belongs to the base package (io/test/jni.kt), not a
    // type package.
    let base_file: String = paths
        .iter()
        .filter(|p| p.ends_with("io/test/jni.kt"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    assert!(base_file.contains("funinterfaceZErrHandler"), "{all}");
}

/// A **rust-side-only** input type: `expand_param!` with NO class
/// declaration. Every param of the type is built from the ctor's ingredients
/// (no selector — single variant); the type never surfaces in Kotlin.
#[test]
fn rust_side_only_input_type() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_opts_new(retries: i32, verbose: bool) -> ZOpts { unimplemented!() }",
        "pub fn z_run(opts: ZOpts) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_run)))
        .expand(
            prebindgen_registry::expand_param!(ZOpts)
                .variant(prebindgen_registry::fun!(z_opts_new)),
        );

    let dir = unique_test_dir("jnigen_rust_side_only_in");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The wrapper folds the ctor Rust-side.
    assert!(rc.contains("myflat::z_opts_new("), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // The Kotlin wrapper takes the ctor's flattened ingredients (prefixed by
    // the param name), not a ZOpts object; no ZOpts class exists.
    assert!(
        all.contains("funzRun(optsRetries:Int,optsVerbose:Boolean"),
        "{all}"
    );
    assert!(!all.contains("classZOpts("), "{all}");
}

/// `variant_self()` on a type with no class declaration is structurally
/// impossible (no Kotlin object to pass) — hard error at write time.
#[test]
#[should_panic(expected = "has no class declaration")]
fn rust_side_only_variant_self_rejected() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_run(opts: ZOpts) -> i64 { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_run)))
        .expand(prebindgen_registry::expand_param!(ZOpts).variant_self());
    let dir = unique_test_dir("jnigen_rso_self_in");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = jni
        .build_with(registry)
        .and_then(|gen| gen.write_rust(dir.join("gen.rs")));
}

/// `field_self()` on a type with no class declaration is structurally
/// impossible (no Kotlin object to deliver) — hard error at write time.
#[test]
#[should_panic(expected = "has no class declaration")]
fn rust_side_only_field_self_rejected() {
    let loc = myflat_loc();
    let f: syn::ItemFn = syn::parse_str("pub fn z_make() -> ZThing { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_make)))
        .expand(prebindgen_registry::expand_return!(ZThing).field_self());
    let dir = unique_test_dir("jnigen_rso_self_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = jni
        .build_with(registry)
        .and_then(|gen| gen.write_rust(dir.join("gen.rs")));
}

/// Per-fn `.expand_param(name, expand_param!(T))`: the decl's `T` must match
/// the named parameter's peeled type — a typo'd type is a hard error naming
/// both types.
#[test]
fn fn_expand_param_type_mismatch_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_make(name: String) -> ZThing { unimplemented!() }",
        "pub fn z_use(t: ZThing) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new().package(
        crate::package!("ops")
            .class(crate::ptr_class!(ZThing).constructor(prebindgen_registry::fun!(z_thing_make)))
            .class(crate::ptr_class!(ZOther))
            // Wrong type: the param `t` is a ZThing, not a ZOther.
            .fun(
                prebindgen_registry::fun!(z_use).expand_param(
                    "t",
                    prebindgen_registry::expand_param!(ZOther)
                        .variant(prebindgen_registry::fun!(z_thing_make)),
                ),
            ),
    );
    let dir = unique_test_dir("jnigen_fn_param_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = jni
        .build_with(registry)
        .expect_err("type mismatch must fail");
    let msg = format!("{err}");
    assert!(msg.contains("ZOther") && msg.contains("ZThing"), "{msg}");
}

/// Per-fn `.expand_return(expand_return!(T))`: the decl's `T` must match the
/// function's peeled return type — a mismatch is a hard error.
#[test]
fn fn_expand_return_type_mismatch_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_make() -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new().package(
        crate::package!("ops")
            .class(
                crate::ptr_class!(ZThing)
                    .method(prebindgen_registry::fun!(z_thing_name).name("name")),
            )
            .class(crate::ptr_class!(ZOther))
            // Wrong type: z_make returns ZThing, not ZOther.
            .fun(
                prebindgen_registry::fun!(z_make)
                    .expand_return(prebindgen_registry::expand_return!(ZOther).field_self()),
            ),
    );
    let dir = unique_test_dir("jnigen_fn_return_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = jni
        .build_with(registry)
        .expect_err("type mismatch must fail");
    let msg = format!("{err}");
    assert!(msg.contains("ZOther") && msg.contains("ZThing"), "{msg}");
}

/// `.expand_param` on a parameter name the function doesn't have is a hard
/// error (`UnknownParam`) — the second typo guard.
#[test]
fn fn_expand_param_unknown_param_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_make(name: String) -> ZThing { unimplemented!() }",
        "pub fn z_use(t: ZThing) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new().package(
        crate::package!("ops")
            .class(crate::ptr_class!(ZThing).constructor(prebindgen_registry::fun!(z_thing_make)))
            .fun(
                prebindgen_registry::fun!(z_use).expand_param(
                    "typo",
                    prebindgen_registry::expand_param!(ZThing)
                        .variant(prebindgen_registry::fun!(z_thing_make)),
                ),
            ),
    );
    let dir = unique_test_dir("jnigen_fn_param_unknown");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = jni
        .build_with(registry)
        .expect_err("unknown param must fail");
    assert!(format!("{err}").contains("typo"), "{err}");
}

/// Duplicate `.expand_return` on one function is a decl-time hard error —
/// the complete field set belongs in ONE decl.
#[test]
#[should_panic(expected = "already has a return expand override")]
fn fn_expand_return_duplicate_rejected() {
    let _ = prebindgen_registry::fun!(z_make)
        .expand_return(prebindgen_registry::expand_return!(ZThing).field_self())
        .expand_return(prebindgen_registry::expand_return!(ZThing).field_self());
}

/// A typo'd `fun!` inside a boundary decl is a HARD scan error (I7):
/// boundary-referenced fns ride the helper-function channel, and a declared
/// helper matching no `#[prebindgen]` item fails the scan — no silent
/// omission, no stale-ignore warning.
#[test]
fn typo_in_expand_decl_is_hard_error() {
    use prebindgen_registry::{ScanError, WriteRustError};
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_fallible)))
        // `z_err_mesage` (sic) exists nowhere among the indexed items.
        .expand(
            prebindgen_registry::expand_return!(ZErr)
                .field(prebindgen_registry::fun!(z_err_mesage).name("message")),
        );
    let dir = unique_test_dir("jnigen_expand_typo_hard_error");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = jni
        .build_with(registry)
        .expect_err("typo'd expand accessor must fail the scan");
    match err {
        WriteRustError::Scan(ScanError::DeclaredNotFound { entries }) => {
            assert_eq!(
                entries,
                vec![("helper function", "z_err_mesage".to_string())]
            );
        }
        other => panic!("expected DeclaredNotFound, got {other:?}"),
    }
}

/// `.ignore(matching(…))` (C2/I4): one predicate acknowledges a whole
/// naming family — the matching undeclared items are skipped without
/// per-name lines, no extern is emitted for them, and the generation still
/// succeeds with only the declared surface. Also exercises the exact
/// type-ignore path (`.ignore(ty!(…))`).
#[test]
fn ignore_matching_acknowledges_naming_family() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_len(v: i64) -> i64 { unimplemented!() }",
        "pub fn detail_const_a() -> i64 { unimplemented!() }",
        "pub fn detail_const_b() -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_len)))
        .ignore(crate::matching(|name| name.starts_with("detail_const_")))
        // The previously-untested type-ignore path: acknowledge a type by key.
        .ignore(prebindgen_registry::ty!(ZUnusedThing));
    // The predicate flows through the Prebindgen hook…
    {
        let preds = jni.decls.ignored_name_predicates();
        assert_eq!(preds.len(), 1);
        assert!(preds[0]("detail_const_a") && !preds[0]("z_len"));
        assert!(jni
            .decls
            .ignored_types()
            .contains(&TypeKey::parse("ZUnusedThing").expect("test type")));
    }
    // …and the full pipeline runs clean, emitting only the declared fn.
    let dir = unique_test_dir("jnigen_ignore_funs_where");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    assert!(rust.contains("Java_io_test_jni_JNINative_zLen"), "{rust}");
    assert!(!rust.contains("detailConstA"), "{rust}");
}

/// An ignore names a bare item — surface overrides are meaningless and
/// rejected at decl time.
#[test]
#[should_panic(expected = "expand overrides don't apply")]
fn ignore_fun_with_overrides_rejected() {
    let _ = crate::IgnoreDecl::from(prebindgen_registry::fun!(z_thing).name("thing"));
}

/// Same for constants: an ignore names a `#[prebindgen]` const, not a
/// value-sourced val.
#[test]
#[should_panic(expected = "value sources/.name() don't apply")]
fn ignore_const_with_source_rejected() {
    let _ = crate::IgnoreDecl::from(crate::constant!(X).expr(
        prebindgen_registry::ty!(i64),
        prebindgen_registry::expr!(1 + 1),
    ));
}

/// A `.variant()` arm only names its constructor — a `.name()` decoration
/// has no surface to land on and is rejected at decl time (was a silent
/// discard).
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn expand_param_variant_with_name_rejected() {
    let _ = prebindgen_registry::expand_param!(ZThing)
        .variant(prebindgen_registry::fun!(z_thing_new).name("thing"));
}

/// Same for expand overrides on a variant constructor.
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn expand_param_variant_with_expand_override_rejected() {
    let _ = prebindgen_registry::expand_param!(ZThing).variant(
        prebindgen_registry::fun!(z_thing_new)
            .expand_return(prebindgen_registry::expand_return!(ZName).field_self()),
    );
}

/// A `.field()` accessor honors `.name()` but nothing else — expand
/// overrides are rejected at decl time (was a silent discard).
#[test]
#[should_panic(expected = "only .name() is honored")]
fn expand_return_field_with_expand_override_rejected() {
    let _ = prebindgen_registry::expand_return!(ZThing).field(
        prebindgen_registry::fun!(z_thing_name).expand_param(
            "v",
            prebindgen_registry::expand_param!(ZName).variant_self(),
        ),
    );
}

/// Positive pin for the asymmetry: `.name()` on a `.field()` accessor is
/// the documented way to name the field — still accepted.
#[test]
fn expand_return_field_with_name_accepted() {
    let _ = prebindgen_registry::expand_return!(ZThing)
        .field(prebindgen_registry::fun!(z_thing_name).name("label"));
}

/// N5: a `.method()` whose target has no parameter of the class type
/// is a hard `AdapterInvariant` error at resolve — previously it silently
/// emitted a method that ignored `this`.
#[test]
fn method_without_receiver_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_free_standing(v: i64) -> i64 { unimplemented!() }",
        "pub fn z_make() -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("t").class(
                crate::ptr_class!(ZThing)
                    .method(prebindgen_registry::fun!(z_thing_free_standing))
                    .constructor(prebindgen_registry::fun!(z_make)),
            ),
        );
    let err = jni.build_with(registry).expect_err("receiver-less member");
    let msg = format!("{err}");
    assert!(
        msg.contains("method `z_thing_free_standing`") && msg.contains("`ZThing`"),
        "{msg}"
    );
}

/// N5: a `.constructor()` member must return `Self` or `Result<Self, E>`.
#[test]
fn constructor_with_wrong_return_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_len(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_make_number() -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("t").class(
                crate::ptr_class!(ZThing)
                    .method(prebindgen_registry::fun!(z_thing_len))
                    .constructor(prebindgen_registry::fun!(z_make_number)),
            ),
        );
    let err = jni.build_with(registry).expect_err("wrong ctor return");
    let msg = format!("{err}");
    assert!(
        msg.contains("constructor `z_make_number`") && msg.contains("it returns `i64`"),
        "{msg}"
    );
}

/// Binding-local output field (`fun!(crate::…).sig(sig!(…)).name(…)`): the
/// accessor lives in the BINDING crate — the generated Rust calls it by its
/// declared path — and a self-typed `Option<&T>` return degrades to a
/// nullable typed handle leaf instead of a splice cycle: the
/// conditional-handle idiom ("deliver the handle only when the binding says
/// it's worth having").
#[test]
fn binding_local_field_conditional_handle() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_enc_get_id(e: &ZEnc) -> i32 { unimplemented!() }",
        "pub fn z_enc_make() -> ZEnc { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZEnc {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for src in fns {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("enc")
                .class(crate::ptr_class!(ZEnc).method(prebindgen_registry::fun!(z_enc_get_id)))
                .fun(prebindgen_registry::fun!(z_enc_make)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZEnc)
                .field(prebindgen_registry::fun!(z_enc_get_id))
                .field(
                    prebindgen_registry::fun!(crate::enc_if_custom)
                        .sig(prebindgen_registry::sig!((e: &ZEnc) -> Option<&ZEnc>))
                        .name("handle"),
                ),
        );
    let dir = unique_test_dir("jnigen_local_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The generated Rust calls the binding-local accessor by its DECLARED
    // path (the generated file compiles inside the binding crate).
    assert!(rc.contains("crate::enc_if_custom("), "{rust}");
    // The registry accessor stays source-qualified.
    assert!(rc.contains("myflat::z_enc_get_id("), "{rust}");
    // Wire shape: the conditional handle is an Option-unwrapped IDENTITY leaf
    // — present clones through the handle projection and BOXES the jlong
    // (matching the `Long?` slot of the raw interface), absent delivers JVM
    // null. A raw primitive `jvalue { j }` here would desync the descriptor.
    assert!(rc.contains("box_jlong"), "{rust}");
    assert!(
        rc.contains("Option::None=>jni::objects::JObject::null()"),
        "{rust}"
    );

    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let raw = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let all: String = raw.split_whitespace().collect();
    // Builder callback: the id leaf + the NULLABLE conditional handle leaf
    // (self-splice degraded to a plain converter leaf).
    assert!(all.contains("zEncGetId:Int,handle:ZEnc?"), "{raw}");
}

/// A binding-local callable must be crate-qualified: `fun!`'s ident arm
/// catches single segments (declaring a registry fn), and `new_local`
/// rejects a degenerate single-segment path outright.
#[test]
#[should_panic(expected = "crate::")]
fn binding_local_field_bare_path_rejected() {
    let _ = crate::FunctionDecl::new_local(syn::parse_quote!(enc_if_custom));
}

/// A binding-local fn name colliding with a `#[prebindgen]` item is a hard
/// error — the emitted call is `<prefix>::<name>`, so the name must denote
/// exactly the binding-local fn.
#[test]
fn binding_local_field_name_collision_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_enc_get_id(e: &ZEnc) -> i32 { unimplemented!() }",
        "pub fn z_enc_make() -> ZEnc { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZEnc {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for src in fns {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("enc")
                .class(crate::ptr_class!(ZEnc))
                .fun(prebindgen_registry::fun!(z_enc_make)),
        )
        .expand(
            // `z_enc_get_id` names a real #[prebindgen] fn — a binding-local
            // field may not shadow it.
            prebindgen_registry::expand_return!(ZEnc).field(
                prebindgen_registry::fun!(crate::z_enc_get_id)
                    .sig(prebindgen_registry::sig!((e: &ZEnc) -> i32))
                    .name("id"),
            ),
        );
    let err = jni
        .build_with(registry)
        .expect_err("collision must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("collides"), "{msg}");
}

/// A binding-local field spliced through a PARENT decomposition: the child's
/// conditional-handle leaf arrives prefixed (`enc__handle`) and nullable, and
/// the generated Rust composes the source accessor with the binding-local
/// one (`crate::enc_if_custom(myflat::z_msg_enc(&v))`).
#[test]
fn binding_local_field_splices_through_parent() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_enc_get_id(e: &ZEnc) -> i32 { unimplemented!() }",
        "pub fn z_msg_enc(m: &ZMsg) -> &ZEnc { unimplemented!() }",
        "pub fn z_msg_len(m: &ZMsg) -> i64 { unimplemented!() }",
        "pub fn z_msg_make() -> ZMsg { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZEnc {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZMsg {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
    ];
    for src in fns {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("msg")
                .class(crate::ptr_class!(ZEnc).method(prebindgen_registry::fun!(z_enc_get_id)))
                .class(crate::ptr_class!(ZMsg))
                .fun(prebindgen_registry::fun!(z_msg_make)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZEnc)
                .field(prebindgen_registry::fun!(z_enc_get_id))
                .field(
                    prebindgen_registry::fun!(crate::enc_if_custom)
                        .sig(prebindgen_registry::sig!((e: &ZEnc) -> Option<&ZEnc>))
                        .name("handle"),
                ),
        )
        .expand(
            prebindgen_registry::expand_return!(ZMsg)
                .field(prebindgen_registry::fun!(z_msg_len).name("len"))
                .field(prebindgen_registry::fun!(z_msg_enc).name("enc")),
        );
    let dir = unique_test_dir("jnigen_local_field_splice");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("crate::enc_if_custom(myflat::z_msg_enc("),
        "{rust}"
    );

    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let raw = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let all: String = raw.split_whitespace().collect();
    // Spliced child leaves: prefixed id + prefixed NULLABLE handle.
    assert!(all.contains("enc__zEncGetId:Int"), "{raw}");
    assert!(all.contains("enc__handle:ZEnc?"), "{raw}");
}

/// Binding-local FUNCTIONS (`fun!(crate::f).sig(sig!(…))`): a fn defined in
/// the binding crate exported through the full `FunctionDecl` surface — free
/// package fn, instance method, companion constructor (also referenced by
/// ident as an `expand_param!` variant arm). After synthesis it IS a registry
/// fn: converters, receiver rule, name mangling, expansion defaults all apply;
/// the generated Rust calls it by its declared path.
#[test]
fn binding_local_functions_all_positions() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_len(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_use(primary: ZThing) -> bool { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZThing {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for src in fns {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni =
        JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!("t")
                    .class(
                        crate::ptr_class!(ZThing)
                            .method(prebindgen_registry::fun!(z_thing_len))
                            // binding-local INSTANCE METHOD (receiver &Self first)
                            .method(
                                prebindgen_registry::fun!(crate::z_thing_ratio)
                                    .sig(prebindgen_registry::sig!((t: &ZThing, scale: f64) -> f64)),
                            )
                            // binding-local COMPANION CONSTRUCTOR
                            .constructor(
                                prebindgen_registry::fun!(crate::z_thing_from_len)
                                    .sig(prebindgen_registry::sig!((len: i64) -> ZThing)),
                            ),
                    )
                    // binding-local FREE FUNCTION, fallible (Result -> onError)
                    .fun(prebindgen_registry::fun!(crate::z_thing_describe).sig(
                        prebindgen_registry::sig!((t: &ZThing, verbose: bool) -> Result<String, String>),
                    ))
                    .fun(prebindgen_registry::fun!(z_use)),
            )
            // The local constructor also serves as an expand_param! variant arm,
            // referenced by IDENT like any registry fn.
            .expand(
                prebindgen_registry::expand_param!(ZThing)
                    .variant(prebindgen_registry::fun!(z_thing_from_len))
                    .variant_self(),
            );
    let dir = unique_test_dir("jnigen_local_funs");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // Every binding-local call is qualified by its declared path; registry
    // fns keep their source qualification.
    assert!(rc.contains("crate::z_thing_ratio("), "{rust}");
    assert!(rc.contains("crate::z_thing_from_len("), "{rust}");
    assert!(rc.contains("crate::z_thing_describe("), "{rust}");
    assert!(rc.contains("myflat::z_thing_len("), "{rust}");

    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let raw = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let all: String = raw.split_whitespace().collect();
    // Method on the class (receiver dropped, stated param names surface).
    assert!(all.contains("funzThingRatio(scale:Double,"), "{raw}");
    // Companion factory returning the class.
    assert!(all.contains("funzThingFromLen(len:Long,"), "{raw}");
    // Free fn with the Result error routed to onError; its ZThing param
    // picked up the TYPE-LEVEL expand default (selector form) — expansion
    // defaults apply to binding-local fns exactly as to registry fns.
    assert!(all.contains("funzThingDescribe("), "{raw}");
    assert!(
        all.contains("tSel:Int,t0:Long?,t1:ZThing?,verbose:Boolean,"),
        "{raw}"
    );
    // The variant arm built from the local ctor: selector slot named after
    // its single param.
    assert!(all.contains("primarySel:Int"), "{raw}");
}

/// Naming rule for binding-local fns: `.name()` is NEVER obligatory — the
/// default derivation feeds the manglers the camel-cased LAST PATH SEGMENT
/// (`crate::sub::z_thing_ratio` → hook sees `zThingRatio`), with the same
/// package/class context as a registry fn, and the hook's output names the
/// Kotlin member. A local field without `.name()` defaults the same way.
#[test]
fn binding_local_fn_names_flow_through_manglers() {
    let loc = myflat_loc();
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZThing {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for src in [
        "pub fn z_thing_make() -> ZThing { unimplemented!() }",
        // A PLAIN fn returning ZThing — the field decomposition applies here
        // (constructors are excluded from output decomposition by design).
        "pub fn z_thing_query() -> ZThing { unimplemented!() }",
    ] {
        items.push((syn::Item::Fn(syn::parse_str(src).unwrap()), loc.clone()));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        // Custom hooks: prefix every derived name — proof the hook RAN and
        // received the camel-cased last segment with its context.
        .set_fun_name_mangle(|pkg, name| {
            assert!(pkg.ends_with("t"), "fun hook package: {pkg}");
            format!("pkg_{name}")
        })
        .set_method_name_mangle(|_pkg, class, name| {
            if class == "ZThing" {
                format!("cls_{name}")
            } else {
                name.to_string()
            }
        })
        .package(
            crate::package!("t")
                .class(
                    crate::ptr_class!(ZThing)
                        .constructor(prebindgen_registry::fun!(z_thing_make))
                        // local METHOD, no .name(): hook sees `zThingRatio`.
                        .method(
                            prebindgen_registry::fun!(crate::sub::z_thing_ratio)
                                .sig(prebindgen_registry::sig!((t: &ZThing, scale: f64) -> f64)),
                        ),
                )
                // local FREE FN, no .name(): fun hook sees `zThingTag`.
                .fun(
                    prebindgen_registry::fun!(crate::sub::z_thing_tag)
                        .sig(prebindgen_registry::sig!((t: &ZThing) -> i64)),
                )
                .fun(prebindgen_registry::fun!(z_thing_query)),
        )
        // local FIELD, no .name(): defaults to camel(last segment). A second
        // field (the handle) keeps the decomposition on the builder path —
        // a single leaf would deliver by direct return, hiding the name.
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field(
                    prebindgen_registry::fun!(crate::sub::z_thing_len)
                        .sig(prebindgen_registry::sig!((t: &ZThing) -> i64)),
                )
                .field_self(),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_local_mangle",
    );
    let all: String = raw.split_whitespace().collect();
    // Method named by the class hook over the camel-cased last segment.
    assert!(all.contains("funcls_zThingRatio(scale:Double,"), "{raw}");
    // Free fn named by the package hook.
    assert!(all.contains("funpkg_zThingTag("), "{raw}");
    // Field leaf defaulted to camel(last segment) — builder param name.
    assert!(all.contains("zThingLen:Long"), "{raw}");
}

/// A path-built `fun!` without `.sig(…)` is a hard error at acceptance —
/// a path carries no signature to read.
#[test]
#[should_panic(expected = ".sig(sig!(")]
fn binding_local_fun_missing_sig_rejected() {
    let _ = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("t").fun(prebindgen_registry::fun!(crate::z_no_sig)));
}

/// `.sig(…)` on an ident-built (registry) `fun!` is a hard error — the
/// signature is read from the registry.
#[test]
#[should_panic(expected = "read from the")]
fn sig_on_registry_fun_rejected() {
    let _ =
        prebindgen_registry::fun!(z_thing_len).sig(prebindgen_registry::sig!((t: &ZThing) -> i64));
}

/// A binding-local fn name colliding with a `#[prebindgen]` item is a hard
/// resolve error — the emitted call would resolve the wrong fn.
#[test]
fn binding_local_fun_name_collision_rejected() {
    let loc = myflat_loc();
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZThing {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    items.push((
        syn::Item::Fn(
            syn::parse_str("pub fn z_thing_len(t: &ZThing) -> i64 { unimplemented!() }").unwrap(),
        ),
        loc.clone(),
    ));
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("t").class(crate::ptr_class!(ZThing)).fun(
                // shadows the #[prebindgen] fn of the same name
                prebindgen_registry::fun!(crate::z_thing_len)
                    .sig(prebindgen_registry::sig!((t: &ZThing) -> i64)),
            ),
        );
    let err = jni
        .build_with(registry)
        .expect_err("collision must be rejected");
    assert!(format!("{err}").contains("collides"), "{err}");
}

/// `.gc_managed()`: the typed handle extends `GcNativeHandle` (pointer in a
/// separate atomic cell), registers a Cleaner action capturing only the cell,
/// and every release path settles the once-only untagged→tagged CAS ticket —
/// `close()` frees eagerly, `take()` and by-value consumption void it, the GC
/// action frees only if it wins. A plain class keeps the field-backed
/// lifecycle; by-value consumption is routed through `markConsumed()` for
/// both.
#[test]
fn gc_managed_handle_lifecycle() {
    let loc = myflat_loc();
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOther {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
    ];
    let fns: &[&str] = &[
        "pub fn z_thing_new() -> ZThing { unimplemented!() }",
        "pub fn z_thing_use(t: ZThing) -> bool { unimplemented!() }",
        "pub fn z_other_new() -> ZOther { unimplemented!() }",
        "pub fn z_other_use(t: ZOther) -> bool { unimplemented!() }",
    ];
    for src in fns {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("t")
                .class(
                    crate::ptr_class!(ZThing)
                        .gc_managed()
                        .constructor(prebindgen_registry::fun!(z_thing_new)),
                )
                .class(
                    crate::ptr_class!(ZOther).constructor(prebindgen_registry::fun!(z_other_new)),
                )
                .fun(prebindgen_registry::fun!(z_thing_use))
                .fun(prebindgen_registry::fun!(z_other_use)),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_gc_managed",
    );
    let all: String = raw.split_whitespace().collect();

    // Shared harness: cell-backed base, CAS helper, shared Cleaner, register fn.
    assert!(all.contains("abstractclassGcNativeHandle"), "{raw}");
    assert!(all.contains("internalfunreleaseCell"), "{raw}");
    assert!(all.contains("internalobjectNativeCleaner"), "{raw}");
    assert!(all.contains("internalfunregisterGcHandle"), "{raw}");

    // The gc class extends GcNativeHandle and self-registers via the cell.
    assert!(
        all.contains("classZThingprivateconstructor(initialPtr:Long):GcNativeHandle(initialPtr)"),
        "{raw}"
    );
    assert!(
        all.contains("privateval__cleanable=registerGcHandle(this){freePtr(it)}"),
        "{raw}"
    );
    // close(): CAS ticket, eager free + eager deregistration.
    assert!(
        all.contains("valp=releaseCell(cell)if(p!=0L)freePtr(p)__cleanable?.clean()"),
        "{raw}"
    );
    // take(): ticket voided, ownership moves into the fresh wrapper.
    assert!(
        all.contains(
            "valp=releaseCell(cell)__cleanable?.clean()returnZThing.fromRawPtr(if(p!=0L)pelsecell.get())"
        ),
        "{raw}"
    );

    // The plain class keeps the field-backed lifecycle.
    assert!(
        all.contains("classZOtherprivateconstructor(initialPtr:Long):NativeHandle(initialPtr)"),
        "{raw}"
    );
    assert!(all.contains("ptr=por1L"), "{raw}");
    assert!(
        !all.contains("classZOtherprivateconstructor(initialPtr:Long):GcNativeHandle"),
        "{raw}"
    );

    // By-value consumption goes through markConsumed() for BOTH classes —
    // for the gc class that settles the ticket, for the plain one it is
    // exactly the old tag write.
    assert!(all.contains("t.markConsumed()"), "{raw}");
    assert!(!all.contains("t.ptr=t.ptror1L"), "{raw}");
}

/// #52 shared fixture: a `ZSummary` ptr class, its `(count, total)` builder, a
/// splittable 2-variant type-level `expand_param!`, and functions taking one or
/// two `ZSummary` params. `extra` fns are appended before indexing.
fn split_fixture(extra: &[&str]) -> RegistryBuilder {
    let loc = myflat_loc();
    let base: &[&str] = &[
        "pub fn z_summary_new(count: i64, total: f64) -> ZSummary { unimplemented!() }",
        "pub fn z_store_expect(expected: ZSummary) -> bool { unimplemented!() }",
        "pub fn z_prefer(primary: ZSummary, fallback: ZSummary) -> i64 { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZSummary {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for src in base.iter().chain(extra) {
        items.push((
            syn::Item::Fn(syn::parse_str(src).expect("parse fn")),
            loc.clone(),
        ));
    }
    crate::test_util::reg_from_items(declare_referenced(items)).expect("index items")
}

pub(super) fn write_all(gen: JniGen, tag: &str) -> String {
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// #52: `FunctionDecl::split_on_param` emits, alongside the retained selector
/// form, one idiomatic typed overload per variant — the build arm named after
/// the constructor's parameters, the `variant_self()` arm typed as the class —
/// each delegating to the selector wrapper.
#[test]
fn split_on_param_emits_typed_overloads() {
    let registry = split_fixture(&[]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(prebindgen_registry::fun!(z_store_expect).split_on_param("expected")),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_one",
    );
    let all: String = raw.split_whitespace().collect();
    assert!(all.contains("expectedSel:Int"), "{raw}"); // selector retained
    assert!(
        all.contains("funzStoreExpect(count:Long,total:Double,"),
        "{raw}"
    );
    assert!(all.contains("funzStoreExpect(expected:ZSummary,"), "{raw}");
    assert!(all.contains("zStoreExpect(0,count,total,null,"), "{raw}");
    assert!(all.contains("zStoreExpect(1,null,null,expected,"), "{raw}");
}

/// #52: two `.split_on_param` on one function emit the **cartesian product** of
/// the params' arms (2×2 = four overloads); build-arm params are prefixed with
/// their origin parameter name to stay unique.
#[test]
fn split_on_param_cartesian_product() {
    let registry = split_fixture(&[]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    prebindgen_registry::fun!(z_prefer)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_prod",
    );
    let all: String = raw.split_whitespace().collect();
    // build / build
    assert!(
        all.contains(
            "funzPrefer(primaryCount:Long,primaryTotal:Double,fallbackCount:Long,fallbackTotal:Double,"
        ),
        "{raw}"
    );
    // build / handle, handle / build, handle / handle
    assert!(
        all.contains("funzPrefer(primaryCount:Long,primaryTotal:Double,fallback:ZSummary,"),
        "{raw}"
    );
    assert!(
        all.contains("funzPrefer(primary:ZSummary,fallbackCount:Long,fallbackTotal:Double,"),
        "{raw}"
    );
    assert!(
        all.contains("funzPrefer(primary:ZSummary,fallback:ZSummary,"),
        "{raw}"
    );
    // A product delegation fills BOTH selector blocks.
    assert!(
        all.contains(
            "zPrefer(0,primaryCount,primaryTotal,null,0,fallbackCount,fallbackTotal,null,"
        ),
        "{raw}"
    );
}

/// #87: a split parameter on a function whose return is **builder-delivered**
/// (decomposed `expand_return!` fields ⇒ generic `<R>` wrapper) keeps the
/// wrapper's generic declaration on every overload — including the full
/// cartesian product — instead of referencing an undeclared `R`.
#[test]
fn split_on_param_preserves_wrapper_generics() {
    let registry = split_fixture(&[
        "pub fn z_summary_count(s: &ZSummary) -> i64 { unimplemented!() }",
        "pub fn z_summary_total(s: &ZSummary) -> f64 { unimplemented!() }",
        "pub fn z_summarize(primary: ZSummary, fallback: ZSummary) -> ZSummary { unimplemented!() }",
    ]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    prebindgen_registry::fun!(z_summarize)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        )
        .expand(
            prebindgen_registry::expand_return!(ZSummary)
                .field(prebindgen_registry::fun!(z_summary_count))
                .field(prebindgen_registry::fun!(z_summary_total)),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_generic",
    );
    let all: String = raw.split_whitespace().collect();
    // The selector wrapper is generic (builder-delivered return)…
    assert!(all.contains("fun<R>zSummarize(primarySel:Int"), "{raw}");
    // …and every cartesian overload re-declares `<R>`.
    assert!(
        all.contains(
            "fun<R>zSummarize(primaryCount:Long,primaryTotal:Double,fallbackCount:Long,fallbackTotal:Double,"
        ),
        "{raw}"
    );
    assert!(
        all.contains("fun<R>zSummarize(primaryCount:Long,primaryTotal:Double,fallback:ZSummary,"),
        "{raw}"
    );
    assert!(
        all.contains("fun<R>zSummarize(primary:ZSummary,fallbackCount:Long,fallbackTotal:Double,"),
        "{raw}"
    );
    assert!(
        all.contains("fun<R>zSummarize(primary:ZSummary,fallback:ZSummary,"),
        "{raw}"
    );
    // No wrapper form may reference `R` without declaring it (the only
    // non-generic `fun zSummarize` is the `external` JNINative extern).
    assert!(!all.contains("publicfunzSummarize("), "{raw}");
}

/// #52: a `.split_on_param` product whose two combinations erase to the same
/// JVM signature is a hard, per-function error. `from_one(Long)` /
/// `from_two(Long,Long)` on two params collide at (one,two) vs (two,one).
#[test]
#[should_panic(expected = "ambiguous")]
fn split_on_param_product_ambiguous_rejected() {
    let loc = myflat_loc();
    let srcs: &[&str] = &[
        "pub fn z_thing_one(a: i64) -> ZThing { unimplemented!() }",
        "pub fn z_thing_two(a: i64, b: i64) -> ZThing { unimplemented!() }",
        "pub fn z_combine(primary: ZThing, fallback: ZThing) -> bool { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZThing {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for s in srcs {
        items.push((syn::Item::Fn(syn::parse_str(s).unwrap()), loc.clone()));
    }
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).expect("index");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops").class(crate::ptr_class!(ZThing)).fun(
                prebindgen_registry::fun!(z_combine)
                    .split_on_param("primary")
                    .split_on_param("fallback"),
            ),
        )
        .expand(
            prebindgen_registry::expand_param!(ZThing)
                .variant(prebindgen_registry::fun!(z_thing_one))
                .variant(prebindgen_registry::fun!(z_thing_two)),
        );
    let _ = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_ambig",
    );
}

/// #52 proactive: a multi-variant `expand_param!` whose arms share a JVM
/// signature is a hard error at the DECLARATION — no function need split it.
#[test]
#[should_panic(expected = "same JVM signature")]
fn split_declaration_colliding_variants_rejected() {
    let loc = myflat_loc();
    let srcs: &[&str] = &[
        "pub fn z_name_from_text(text: String) -> ZName { unimplemented!() }",
        "pub fn z_name_from_label(label: String) -> ZName { unimplemented!() }",
        "pub fn z_use_name(name: ZName) -> bool { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZName {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for s in srcs {
        items.push((syn::Item::Fn(syn::parse_str(s).unwrap()), loc.clone()));
    }
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).expect("index");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(prebindgen_registry::fun!(z_use_name)), // NOT split — still errors
        )
        .expand(
            prebindgen_registry::expand_param!(ZName)
                .variant(prebindgen_registry::fun!(z_name_from_text))
                .variant(prebindgen_registry::fun!(z_name_from_label)),
        );
    let _ = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_decl",
    );
}

/// #90: the validation boundary is now in `resolve` — a colliding split
/// declaration (a Kotlin-side concern) fails `resolve` as a clean `Err`, so
/// no `JniGen` is produced and neither artifact can be written.
#[test]
fn split_declaration_collision_fails_resolve() {
    let loc = myflat_loc();
    let srcs: &[&str] = &[
        "pub fn z_name_from_text(text: String) -> ZName { unimplemented!() }",
        "pub fn z_name_from_label(label: String) -> ZName { unimplemented!() }",
        "pub fn z_use_name(name: ZName) -> bool { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZName {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for s in srcs {
        items.push((syn::Item::Fn(syn::parse_str(s).unwrap()), loc.clone()));
    }
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).expect("index");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(prebindgen_registry::fun!(z_use_name)),
        )
        .expand(
            prebindgen_registry::expand_param!(ZName)
                .variant(prebindgen_registry::fun!(z_name_from_text))
                .variant(prebindgen_registry::fun!(z_name_from_label)),
        );
    let err = jni
        .build_with(registry)
        .expect_err("colliding split declaration must fail resolve");
    assert!(
        err.to_string().contains("same JVM signature"),
        "unexpected error: {err}"
    );
}

/// #52: `.no_split()` suppresses the proactive splittability check for a
/// genuinely non-splittable variant set (used only as the selector form).
#[test]
fn split_no_split_suppresses_check() {
    let loc = myflat_loc();
    let srcs: &[&str] = &[
        "pub fn z_name_from_text(text: String) -> ZName { unimplemented!() }",
        "pub fn z_name_from_label(label: String) -> ZName { unimplemented!() }",
        "pub fn z_use_name(name: ZName) -> bool { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZName {
                _p: u8,
            }
        )),
        loc.clone(),
    )];
    for s in srcs {
        items.push((syn::Item::Fn(syn::parse_str(s).unwrap()), loc.clone()));
    }
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).expect("index");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(prebindgen_registry::fun!(z_use_name)),
        )
        .expand(
            prebindgen_registry::expand_param!(ZName)
                .variant(prebindgen_registry::fun!(z_name_from_text))
                .variant(prebindgen_registry::fun!(z_name_from_label))
                .no_split(),
        );
    // No panic: the colliding variants are tolerated as selector-only.
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_no_split",
    );
    let all: String = raw.split_whitespace().collect();
    assert!(all.contains("nameSel:Int"), "{raw}"); // selector form emitted
}

/// #52: `.split_on_param` naming a parameter that does not exist on the
/// function is a hard error (typo guard).
#[test]
#[should_panic(expected = "no parameter named")]
fn split_on_unknown_param_rejected() {
    let registry = split_fixture(&[]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(prebindgen_registry::fun!(z_store_expect).split_on_param("nope")),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        );
    let _ = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_typo",
    );
}

/// Nullable-arm rule: `.split_on_param` on an `Option<T>` parameter emits
/// overloads for its **single-leaf** arms only — here the `variant_self()`
/// arm, typed nullable (`ZSummary?`) with `null` = absent, delegating a
/// conditional selector (`-1` when null). The multi-leaf `(count, total)`
/// build arm stays selector-only.
#[test]
fn split_on_option_param_emits_nullable_arm() {
    let registry =
        split_fixture(&["pub fn z_maybe(opt: Option<ZSummary>) -> bool { unimplemented!() }"]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(prebindgen_registry::fun!(z_maybe).split_on_param("opt")),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_opt",
    );
    let all: String = raw.split_whitespace().collect();
    // Selector form retained; single nullable overload for the identity arm.
    assert!(all.contains("optSel:Int"), "{raw}");
    assert!(all.contains("funzMaybe(opt:ZSummary?,"), "{raw}");
    assert!(
        all.contains("zMaybe(if(opt!=null)1else-1,null,null,opt,"),
        "{raw}"
    );
    // No overload for the multi-leaf build arm.
    assert!(!all.contains("funzMaybe(count:"), "{raw}");
}

/// Nullable-arm rule: an `Option<T>` parameter whose expansion has **no**
/// single-leaf arm (two multi-arg build arms, no identity) cannot be split —
/// hard error, keep the selector form.
#[test]
#[should_panic(expected = "none of its arms is a single leaf")]
fn split_on_option_param_without_single_leaf_arm_rejected() {
    let registry = split_fixture(&[
        "pub fn z_summary_scaled(units: String, factor: f64) -> ZSummary { unimplemented!() }",
        "pub fn z_maybe(opt: Option<ZSummary>) -> bool { unimplemented!() }",
    ]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(prebindgen_registry::fun!(z_maybe).split_on_param("opt")),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant(prebindgen_registry::fun!(z_summary_scaled)),
        );
    let _ = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_opt_no_arm",
    );
}

/// Nullable-arm rule × cartesian product: a non-optional split param (all
/// arms) combines with an optional one (single-leaf arms only) — each combo
/// fills its own block, constant selector for the former, conditional for the
/// latter.
#[test]
fn split_on_param_optional_cartesian_with_plain() {
    let registry = split_fixture(&[
        "pub fn z_mixed(primary: ZSummary, fallback: Option<&ZSummary>) -> i64 { unimplemented!() }",
    ]);
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    prebindgen_registry::fun!(z_mixed)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            prebindgen_registry::expand_param!(ZSummary)
                .variant(prebindgen_registry::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(
        jni.build_with(registry).expect("resolve"),
        "jnigen_split_opt_prod",
    );
    let all: String = raw.split_whitespace().collect();
    // 2 (primary arms) × 1 (fallback single-leaf arm) overloads.
    assert!(
        all.contains("funzMixed(primaryCount:Long,primaryTotal:Double,fallback:ZSummary?,"),
        "{raw}"
    );
    assert!(
        all.contains("funzMixed(primary:ZSummary,fallback:ZSummary?,"),
        "{raw}"
    );
    // Constant selector for the plain block, conditional for the optional one.
    assert!(
        all.contains(
            "zMixed(0,primaryCount,primaryTotal,null,if(fallback!=null)1else-1,null,null,fallback,"
        ),
        "{raw}"
    );
}

/// Optional combined-selector expansion: an `Option<&T>` param with a
/// build-from arm AND an identity arm crosses as a selector tuple whose
/// selector also encodes absence (`-1` = `None`). The ctor's own
/// `Option<String>` arg passes through un-double-wrapped, and the identity
/// arm is a nullable typed handle.
#[test]
fn optional_selector_dispatch_end_to_end() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZEnc {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_enc_from_id(id: i32, schema: Option<String>) -> ZEnc {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_put(encoding: Option<&ZEnc>) -> bool {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(
                    crate::ptr_class!(ZEnc).constructor(prebindgen_registry::fun!(z_enc_from_id)),
                )
                .fun(prebindgen_registry::fun!(z_put)),
        )
        .expand(
            prebindgen_registry::expand_param!(ZEnc)
                .variant(prebindgen_registry::fun!(z_enc_from_id))
                .variant_self(),
        );
    let dir = unique_test_dir("jnigen_opt_selector");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // Rust side: the selector gates absence before the dispatch.
    assert!(rc.contains("<0"), "{rust}");
    assert!(rc.contains("Option::None"), "{rust}");
    assert!(rc.contains("z_enc_from_id"), "{rust}");

    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let raw = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let all: String = raw.split_whitespace().collect();
    // Selector Int + nullable build-arm leaves + nullable identity handle.
    assert!(all.contains("encodingSel:Int"), "{raw}");
    assert!(all.contains("encoding1:ZEnc?"), "{raw}");
    // The already-Option schema arg stays a single-level String?.
    assert!(all.contains("encoding01:String?"), "{raw}");
    assert!(!all.contains("String??"), "{raw}");
}

/// #96: a `.constructor()` member's return is a factory — it must be
/// excluded from the type-level `expand_return!` default auto-apply even
/// though its return type matches. Pins the `skip_output` derivation from
/// `class_members` (previously an eagerly-mutated accumulator).
#[test]
fn constructor_member_skips_default_output_expand() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_make() -> ZThing { unimplemented!() }",
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_thing_get(s: i64) -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(
                    crate::ptr_class!(ZThing)
                        .constructor(prebindgen_registry::fun!(z_thing_make).name("make"))
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                .fun(prebindgen_registry::fun!(z_thing_get)),
        )
        // Canonical output for ZThing: any ZThing-returning declared fn gets
        // callback delivery by default…
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field_self()
                .field(prebindgen_registry::fun!(z_thing_name)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    let registry = gen.registry();
    // …the free fn is decomposed…
    assert!(
        registry.unfold_plans().contains_key(&syn::Ident::new(
            "z_thing_get",
            proc_macro2::Span::call_site()
        )),
        "free fn gets the default output expansion"
    );
    // …but the constructor member is NOT (its return is the factory value).
    assert!(
        !registry.unfold_plans().contains_key(&syn::Ident::new(
            "z_thing_make",
            proc_macro2::Span::call_site()
        )),
        "constructor member must skip the default output expansion"
    );
}

// ── issue #95: qualified signature spellings + bare declarations ─────────

#[test]
fn qualified_signature_spelling_matches_bare_ptr_class() {
    // The source crate spells its own types with `myflat::`/`crate::` and a
    // std-prelude path; ingest normalizes them to the bare flat spelling,
    // so the bare `ptr_class!(ZThing)` declaration (and the whole
    // kotlin_fqn / leaf_key chain behind the wrapper) matches.
    let loc = myflat_loc();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_get() -> myflat::ZThing {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_name(this_: &crate::things::ZThing) -> std::string::String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                .fun(prebindgen_registry::fun!(z_thing_get)),
        );
    let dir = unique_test_dir("jnigen_q95");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni
        .build_with(registry)
        .expect("qualified spellings resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    let ac: String = all.split_whitespace().collect();
    // The typed handle class with its instance method, and the typed factory
    // wrapper returning the class — the full declaration↔signature chain.
    assert!(
        ac.contains("classZThingprivateconstructor(initialPtr:Long)"),
        "{all}"
    );
    assert!(ac.contains("funname(onError:"), "{all}");
    assert!(
        ac.contains("funzThingGet(onError:JniErrorHandler<ZThing?>):ZThing?"),
        "{all}"
    );
}

/// A `data_class` states a recipe saying what it is made of, and a field that is
/// itself one contributes its own wires rather than a single value.
///
/// The composition every binding compiles (see `JniGen::compile_crossing`),
/// asserted here on the shape that makes the recursion visible: `Holder` is a
/// scalar plus a nested `Summary`, so it crosses as three JNI values and not
/// as two.
#[test]
fn a_data_class_crosses_as_its_fields_and_a_nested_one_as_its_own() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Summary {
                    pub count: i64,
                    pub total: f64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Holder {
                    pub tag: i64,
                    pub summary: Summary,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn holder_tag(h: Holder) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Summary))
                .class(crate::data_class!(Holder))
                .fun(prebindgen_registry::fun!(holder_tag)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    let wires = gen
        .parts_wires_for_test("Holder")
        .expect("Holder states a parts recipe");
    let described: Vec<String> = wires
        .iter()
        .map(|w| {
            let ty = &w.ty;
            format!("{}: {}", w.path, quote::quote!(#ty))
        })
        .collect();
    assert_eq!(
        described,
        vec![
            "tag: jni :: sys :: jlong",
            "summary.count: jni :: sys :: jlong",
            "summary.total: jni :: sys :: jdouble",
        ],
        "a nested data class contributes its own wires, under its field's path"
    );
}

/// Two gates deep, the inner one supplies the absent value and the outer one
/// must not supply a second.
///
/// `Outer { mid: Option<Mid> }` where `Mid { inner: Option<Leaf> }`: reaching
/// `Leaf.id` passes through both. The value slot reads
/// `o.mid?.inner?.id ?: 0L` — one elvis, from the innermost gate — and the
/// presence flags read as plain comparisons, which are already non-null and
/// which Kotlin refuses to elvis at all.
#[test]
fn a_gate_inside_a_gate_supplies_one_absent_value() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Leaf {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Mid {
                    pub inner: Option<Leaf>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Outer {
                    pub mid: Option<Mid>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn outer_use(o: Option<Outer>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Leaf))
                .class(crate::data_class!(Mid))
                .class(crate::data_class!(Outer))
                .fun(prebindgen_registry::fun!(outer_use)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    assert_eq!(
        wire_lines(&gen, "Outer", "o"),
        vec![
            "oMidPresent: Boolean = o.mid != null",
            "oMidInnerPresent: Boolean = o.mid?.inner != null",
            "oMidInnerId: Long = o.mid?.inner?.id ?: 0L",
        ],
    );
    assert_eq!(
        wire_lines(&gen, "Mid", "m"),
        vec![
            "mInnerPresent: Boolean = m.inner != null",
            "mInnerId: Long = m.inner?.id ?: 0L",
        ],
    );
}

/// A nullable primitive keeps the allocation-free `(present, value)` pair
/// rather than boxing, at field depth as at parameter depth.
///
/// The value crosses through the **inner's** conversion — there is no boxed
/// `Option` on that wire to decode — and neither half names a field of its own:
/// both were decoupled from one, which is the field they answer with.
#[test]
fn a_nullable_primitive_field_crosses_as_a_pair() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Scal {
                    pub n: Option<i64>,
                    pub k: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn scal_use(s: Scal) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Scal))
                .fun(prebindgen_registry::fun!(scal_use)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    assert_eq!(
        wire_lines(&gen, "Scal", "s"),
        vec![
            "sNPresent: Boolean = s.n != null",
            "sNValue: Long = s.n ?: 0L",
            "sK: Long = s.k",
        ],
    );
    // Neither half of the pair names a field of its own; the field they were
    // decoupled from is what both answer with.
    let wires = gen.parts_wires_for_test("Scal").expect("recipe");
    assert_eq!(wires[0].field(), Some("n"));
    assert_eq!(wires[1].field(), Some("n"));
}

/// A `sealed_class` field crosses as a tag plus every alternative's slots, and
/// the recipe says so where the walk did.
///
/// The shape covertest carries: `Observation` holds a required `Reading` and an
/// optional one, whose alternatives between them cover a scalar payload, a
/// two-field payload, a string payload that rides a JVM `null`, and a Kotlin
/// enum payload read through `.value`. Both fields go through the same
/// composition, and the optional one takes the `null` arm and the presence flag
/// on top of it.
#[test]
fn a_sealed_class_field_crosses_as_a_tag_and_every_arm_s_slots() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    Exact(i64),
                    Range { low: i64, high: i64 },
                    Tagged(String, Priority),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Observation {
                    pub id: i64,
                    pub reading: Reading,
                    pub fallback: Option<Reading>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn observation_which(o: Observation) -> i32 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .class(crate::sealed_class!(Reading))
                .class(crate::data_class!(Observation))
                .fun(prebindgen_registry::fun!(observation_which)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    // The whole signature, in the order the three coordinated sites read it.
    // Between them the alternatives cover a scalar payload, a two-field
    // payload, a string payload that rides a JVM `null` rather than a literal,
    // and a Kotlin enum payload read through its discriminant.
    assert_eq!(
        wire_lines(&gen, "Observation", "o"),
        vec![
            "oId: Long = o.id",
            "oReadingTag: Int = when (o.reading) { is io.test.jni.Reading.Missing -> 0; is io.test.jni.Reading.Exact -> 1; is io.test.jni.Reading.Range -> 2; is io.test.jni.Reading.Tagged -> 3 }",
            "oReadingExactV0: Long = (o.reading as? io.test.jni.Reading.Exact)?.v0 ?: 0L",
            "oReadingRangeLow: Long = (o.reading as? io.test.jni.Reading.Range)?.low ?: 0L",
            "oReadingRangeHigh: Long = (o.reading as? io.test.jni.Reading.Range)?.high ?: 0L",
            "oReadingTaggedV0: String? = (o.reading as? io.test.jni.Reading.Tagged)?.v0",
            "oReadingTaggedV1: Int = (o.reading as? io.test.jni.Reading.Tagged)?.v1?.value ?: 0",
            // The optional field adds the gate, and its `when` adds the arm the
            // required one has no need of.
            "oFallbackPresent: Boolean = o.fallback != null",
            "oFallbackTag: Int = when (o.fallback) { null -> 0; is io.test.jni.Reading.Missing -> 0; is io.test.jni.Reading.Exact -> 1; is io.test.jni.Reading.Range -> 2; is io.test.jni.Reading.Tagged -> 3 }",
            "oFallbackExactV0: Long = (o.fallback as? io.test.jni.Reading.Exact)?.v0 ?: 0L",
            "oFallbackRangeLow: Long = (o.fallback as? io.test.jni.Reading.Range)?.low ?: 0L",
            "oFallbackRangeHigh: Long = (o.fallback as? io.test.jni.Reading.Range)?.high ?: 0L",
            "oFallbackTaggedV0: String? = (o.fallback as? io.test.jni.Reading.Tagged)?.v0",
            "oFallbackTaggedV1: Int = (o.fallback as? io.test.jni.Reading.Tagged)?.v1?.value ?: 0",
        ],
    );

    let dir = unique_test_dir("sealed_choice_input_chain");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write Rust"))
        .expect("read Rust");
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("reading:tuple5_to_Reading_")
            && rc.contains("fallback:tuple2_to_Option_Reading_")
            && rc.contains("::core::option::Option::Some(tuple5_to_Reading_"),
        "Product and Optional parents must delegate the sum walk to the Choice converter:\n{rust}"
    );
    assert!(
        !rc.contains("matcho_reading__tag"),
        "the exported wrapper must not reconstruct the sum inline:\n{rust}"
    );
}

/// Every spelling the walk flattens states the same recipe.
///
/// `build_flat_input_plan` accepts a `data_class` parameter through five
/// spellings — bare, `&`, `Option`, `Box`, and `Box<Option<…>>` — and all five
/// appear in `covertest-kotlin` or `perftest-kotlin`. A borrow and a transparent
/// wrapper find the class's own `parts` recipe, because a crossing is keyed by the
/// value that crosses; an optional has no such recipe and composes on the one the
/// registry derives for it, which is why `Declarations::bindings` binds its
/// part.
#[test]
fn every_spelling_the_walk_flattens_states_the_same_row() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Summary {
                    pub count: i64,
                    pub total: f64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Holder {
                    pub tag: i64,
                    pub summary: Summary,
                    pub note: Option<i64>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn bare(h: Holder) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn borrowed(h: &Holder) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn optional(h: Option<Holder>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn boxed(h: Box<Holder>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn boxed_optional(h: Box<Option<Holder>>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Summary))
                .class(crate::data_class!(Holder))
                .fun(prebindgen_registry::fun!(bare))
                .fun(prebindgen_registry::fun!(borrowed))
                .fun(prebindgen_registry::fun!(optional))
                .fun(prebindgen_registry::fun!(boxed))
                .fun(prebindgen_registry::fun!(boxed_optional)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    for spelling in [
        "Holder",
        "&Holder",
        "Option<Holder>",
        "Box<Holder>",
        "Box<Option<Holder>>",
    ] {
        let optional = spelling.contains("Option");
        let mut expected = vec![
            "hTag: Long = h.tag",
            "hSummaryCount: Long = h.summary.count",
            "hSummaryTotal: Double = h.summary.total",
            "hNotePresent: Boolean = h.note != null",
            "hNoteValue: Long = h.note ?: 0L",
        ];
        // The two optional spellings carry one wire more, and every read below
        // that gate is a safe call.
        let gated = vec![
            "hPresent: Boolean = h != null",
            "hTag: Long = h?.tag ?: 0L",
            "hSummaryCount: Long = h?.summary?.count ?: 0L",
            "hSummaryTotal: Double = h?.summary?.total ?: 0.0",
            "hNotePresent: Boolean = h?.note != null",
            "hNoteValue: Long = h?.note ?: 0L",
        ];
        if optional {
            expected = gated;
        }
        assert_eq!(wire_lines(&gen, spelling, "h"), expected, "{spelling}");
    }
}

/// Every field shape the walk reads specially, held to the recipe.
///
/// A `data_class` field is not always one property read: an `enum_class`
/// property holds the enum object where the wire holds its discriminant, an
/// unsigned representation's property is a `ULong` over a `Long` wire, an
/// optional primitive is decoupled into a presence flag and a raw slot rather
/// than boxed, an optional whose wire is a JVM object rides a `null`, and a
/// nested handle is locked through the object rather than the pointer it
/// carries. Each of those is a place the composition and the walk could differ
/// silently, so the fixture carries all of them at once — and again one layer
/// down, where a closed gate above turns every read into a safe call and
/// substitutes what a non-nullable slot carries meanwhile.
#[test]
fn every_field_shape_the_walk_reads_specially_states_the_same_row() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Bits {
                    pub pri: Priority,
                    pub maybe_pri: Option<Priority>,
                    pub big: u64,
                    pub maybe_big: Option<u64>,
                    pub name: String,
                    pub maybe_name: Option<String>,
                    pub flag: bool,
                    pub maybe_flag: Option<bool>,
                    pub bytes: [u8; 4],
                    pub opaque: Opaque,
                    pub maybe_opaque: Option<Opaque>,
                    pub nested: Nested,
                    pub maybe_nested: Option<Nested>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Opaque {
                    pub v: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Nested {
                    pub k: i64,
                    pub s: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn bits_use(b: Bits) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn bits_opt(b: Option<Bits>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .class(crate::ptr_class!(Opaque))
                .class(crate::data_class!(Nested))
                .class(crate::data_class!(Bits))
                .fun(prebindgen_registry::fun!(bits_use))
                .fun(prebindgen_registry::fun!(bits_opt)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    assert_eq!(
        wire_lines(&gen, "Bits", "b"),
        vec![
            "bPri: Int = b.pri.value",
            "bMaybePriPresent: Boolean = b.maybePri != null",
            "bMaybePriValue: Int = b.maybePri?.value ?: 0",
            "bBig: Long = b.big.toLong()",
            "bMaybeBigPresent: Boolean = b.maybeBig != null",
            "bMaybeBigValue: Long = b.maybeBig?.toLong() ?: 0L",
            "bName: String = b.name",
            "bMaybeName: String? = b.maybeName",
            "bFlag: Boolean = b.flag",
            "bMaybeFlagPresent: Boolean = b.maybeFlag != null",
            "bMaybeFlagValue: Boolean = b.maybeFlag ?: false",
            "bBytes: ByteArray = b.bytes",
            "bOpaque: Long = b.opaque",
            "bMaybeOpaque: Long = b.maybeOpaque",
            "bNestedK: Long = b.nested.k",
            "bNestedS: String = b.nested.s",
            "bMaybeNestedPresent: Boolean = b.maybeNested != null",
            "bMaybeNestedK: Long = b.maybeNested?.k ?: 0L",
            "bMaybeNestedS: String? = b.maybeNested?.s ?: \"\"",
        ],
    );
    // The same fields one layer down, where a gate above turns every read into
    // a safe call and states what a non-nullable slot carries meanwhile.
    assert_eq!(
        wire_lines(&gen, "Option<Bits>", "b"),
        vec![
            "bPresent: Boolean = b != null",
            "bPri: Int = b?.pri?.value ?: 0",
            "bMaybePriPresent: Boolean = b?.maybePri != null",
            "bMaybePriValue: Int = b?.maybePri?.value ?: 0",
            "bBig: Long = b?.big?.toLong() ?: 0L",
            "bMaybeBigPresent: Boolean = b?.maybeBig != null",
            "bMaybeBigValue: Long = b?.maybeBig?.toLong() ?: 0L",
            "bName: String? = b?.name ?: \"\"",
            "bMaybeName: String? = b?.maybeName",
            "bFlag: Boolean = b?.flag ?: false",
            "bMaybeFlagPresent: Boolean = b?.maybeFlag != null",
            "bMaybeFlagValue: Boolean = b?.maybeFlag ?: false",
            "bBytes: ByteArray? = b?.bytes ?: ByteArray(0)",
            "bOpaque: Long = b?.opaque",
            "bMaybeOpaque: Long = b?.maybeOpaque",
            "bNestedK: Long = b?.nested?.k ?: 0L",
            "bNestedS: String? = b?.nested?.s ?: \"\"",
            "bMaybeNestedPresent: Boolean = b?.maybeNested != null",
            "bMaybeNestedK: Long = b?.maybeNested?.k ?: 0L",
            "bMaybeNestedS: String? = b?.maybeNested?.s ?: \"\"",
        ],
    );
}

/// One line per JNI parameter a crossing occupies: the name a site gives it,
/// its Kotlin type, and the Kotlin expression that fills it.
///
/// What the three coordinated sites read, in the order they read it — so a
/// fixture states the whole signature rather than one fact about it.
fn wire_lines(gen: &crate::jni::JniGen, spelling: &str, param: &str) -> Vec<String> {
    gen.named_wires_for_test(spelling, param)
        .unwrap_or_else(|| panic!("{spelling} states no composition"))
        .into_iter()
        .map(|(name, kt_ty, access, ..)| format!("{name}: {kt_ty} = {access}"))
        .collect()
}
