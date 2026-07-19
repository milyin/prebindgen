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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(crate::fun!(z_thing_name).name("name"))
                        .method(crate::fun!(z_thing_size).name("size")),
                )
                .fun(crate::fun!(z_make_a))
                // Per-fn inline fields: name + size + name again (different shape). The
                // third field reuses the `z_thing_name` accessor but must carry a
                // distinct (literal) leaf name — duplicate names are a hard error.
                .fun(
                    crate::fun!(z_make_b).expand_return(
                        crate::expand_return!(ZThing)
                            .field(crate::fun!(z_thing_name).name("name"))
                            .field(crate::fun!(z_thing_size).name("size"))
                            .field(crate::fun!(z_thing_name).name("name2")),
                    ),
                ),
        )
        // Default output: name + size (2 leaves ⇒ builder callback). The
        // `name` field inherits its Kotlin name from the class member; `size`
        // sets it explicitly — both paths resolve to the member-equal names.
        .expand(
            crate::expand_return!(ZThing)
                .field(crate::fun!(z_thing_name))
                .field(crate::fun!(z_thing_size).name("size")),
        );

    let dir = unique_test_dir("jnigen_inline_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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

/// Error decomposition is the OUTPUT decomposition with a fixed leading `je`:
/// the same record kinds work — an identity record (the error itself as an
/// owned handle), plain accessors, and accessors nested through `Option`
/// (spliced child decomposition, nullable leaves). The ze params are typed
/// exactly like a builder's; on a binding error the native side fills typed
/// defaults (closed handle, "", null for nullable).
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("errors")
                .class(crate::ptr_class!(ZDetail).method(crate::fun!(z_detail_code).name("code")))
                .class(
                    crate::ptr_class!(ZErr)
                        .method(crate::fun!(z_err_message).name("message"))
                        .method(crate::fun!(z_err_detail).name("detail")),
                )
                .fun(crate::fun!(z_fallible)),
        )
        .expand(crate::expand_return!(ZDetail).field(crate::fun!(z_detail_code)))
        // Canonical error decomposition: the owned error handle itself, its
        // message, and the Option-nested detail spliced to its code leaf.
        // Field names inherit from the class members.
        .expand(
            crate::expand_return!(ZErr)
                .field_self()
                .field(crate::fun!(z_err_message))
                .field(crate::fun!(z_err_detail)),
        );

    let dir = unique_test_dir("jnigen_err_universal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
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
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .method(crate::fun!(z_thing_name).name("name"))
                    // A method with extra params: `&ZThing` receiver + a `name: String` param.
                    .method(crate::fun!(z_thing_rename).name("rename"))
                    // A constructor: factory returning ZThing.
                    .constructor(crate::fun!(z_thing_make).name("make")),
            )
            // A free fn whose per-fn inline output decomposes to (handle, name).
            .fun(
                crate::fun!(z_get).expand_return(
                    crate::expand_return!(ZThing)
                        .field_self()
                        .field(crate::fun!(z_thing_name).name("name")),
                ),
            ),
    );

    let dir = unique_test_dir("jnigen_method_ctor");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(crate::fun!(z_fallible)))
        // No class declaration for ZErr anywhere — rust-side-only. The field
        // name is explicit (no class member to inherit from).
        .expand(crate::expand_return!(ZErr).field(crate::fun!(z_err_message).name("message")));

    let dir = unique_test_dir("jnigen_rust_side_only_err");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    // Handler in the BASE package with the decomposed message field; no ZErr
    // class anywhere.
    assert!(
        all.contains("funinterfaceZErrHandler<outR>{publicfunrun(je:String?,message:String):R"),
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(crate::fun!(z_run)))
        .expand(crate::expand_param!(ZOpts).variant(crate::fun!(z_opts_new)));

    let dir = unique_test_dir("jnigen_rust_side_only_in");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .package(crate::package!("ops").fun(crate::fun!(z_run)))
        .expand(crate::expand_param!(ZOpts).variant_self());
    let dir = unique_test_dir("jnigen_rso_self_in");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry
        .resolve(jni)
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .package(crate::package!("ops").fun(crate::fun!(z_make)))
        .expand(crate::expand_return!(ZThing).field_self());
    let dir = unique_test_dir("jnigen_rso_self_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry
        .resolve(jni)
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().package(
        crate::package!("ops")
            .class(crate::ptr_class!(ZThing).constructor(crate::fun!(z_thing_make)))
            .class(crate::ptr_class!(ZOther))
            // Wrong type: the param `t` is a ZThing, not a ZOther.
            .fun(crate::fun!(z_use).expand_param(
                "t",
                crate::expand_param!(ZOther).variant(crate::fun!(z_thing_make)),
            )),
    );
    let dir = unique_test_dir("jnigen_fn_param_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = registry.resolve(jni).expect_err("type mismatch must fail");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().package(
        crate::package!("ops")
            .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
            .class(crate::ptr_class!(ZOther))
            // Wrong type: z_make returns ZThing, not ZOther.
            .fun(crate::fun!(z_make).expand_return(crate::expand_return!(ZOther).field_self())),
    );
    let dir = unique_test_dir("jnigen_fn_return_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = registry.resolve(jni).expect_err("type mismatch must fail");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().package(
        crate::package!("ops")
            .class(crate::ptr_class!(ZThing).constructor(crate::fun!(z_thing_make)))
            .fun(crate::fun!(z_use).expand_param(
                "typo",
                crate::expand_param!(ZThing).variant(crate::fun!(z_thing_make)),
            )),
    );
    let dir = unique_test_dir("jnigen_fn_param_unknown");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = registry.resolve(jni).expect_err("unknown param must fail");
    assert!(format!("{err}").contains("typo"), "{err}");
}

/// Duplicate `.expand_return` on one function is a decl-time hard error —
/// the complete field set belongs in ONE decl.
#[test]
#[should_panic(expected = "already has a return expand override")]
fn fn_expand_return_duplicate_rejected() {
    let _ = crate::fun!(z_make)
        .expand_return(crate::expand_return!(ZThing).field_self())
        .expand_return(crate::expand_return!(ZThing).field_self());
}

/// A typo'd `fun!` inside a boundary decl is a HARD scan error (I7):
/// boundary-referenced fns ride the helper-function channel, and a declared
/// helper matching no `#[prebindgen]` item fails the scan — no silent
/// omission, no stale-ignore warning.
#[test]
fn typo_in_expand_decl_is_hard_error() {
    use crate::api::core::registry::{ScanError, WriteRustError};
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(crate::fun!(z_fallible)))
        // `z_err_mesage` (sic) exists nowhere among the indexed items.
        .expand(crate::expand_return!(ZErr).field(crate::fun!(z_err_mesage).name("message")));
    let dir = unique_test_dir("jnigen_expand_typo_hard_error");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let err = registry
        .resolve(jni)
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(crate::fun!(z_len)))
        .ignore(crate::matching(|name| name.starts_with("detail_const_")))
        // The previously-untested type-ignore path: acknowledge a type by key.
        .ignore(crate::ty!(ZUnusedThing));
    // The predicate flows through the Prebindgen hook…
    {
        use crate::api::core::prebindgen::Prebindgen;
        let preds = jni.ignored_name_predicates();
        assert_eq!(preds.len(), 1);
        assert!(preds[0]("detail_const_a") && !preds[0]("z_len"));
        assert!(jni
            .ignored_types()
            .contains(&TypeKey::parse("ZUnusedThing").expect("test type")));
    }
    // …and the full pipeline runs clean, emitting only the declared fn.
    let dir = unique_test_dir("jnigen_ignore_funs_where");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let _ = crate::lang::IgnoreDecl::from(crate::fun!(z_thing).name("thing"));
}

/// Same for constants: an ignore names a `#[prebindgen]` const, not a
/// value-sourced val.
#[test]
#[should_panic(expected = "value sources/.name() don't apply")]
fn ignore_const_with_source_rejected() {
    let _ = crate::lang::IgnoreDecl::from(
        crate::constant!(X).expr(crate::ty!(i64), crate::expr!(1 + 1)),
    );
}

/// A `.variant()` arm only names its constructor — a `.name()` decoration
/// has no surface to land on and is rejected at decl time (was a silent
/// discard).
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn expand_param_variant_with_name_rejected() {
    let _ = crate::expand_param!(ZThing).variant(crate::fun!(z_thing_new).name("thing"));
}

/// Same for expand overrides on a variant constructor.
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn expand_param_variant_with_expand_override_rejected() {
    let _ = crate::expand_param!(ZThing)
        .variant(crate::fun!(z_thing_new).expand_return(crate::expand_return!(ZName).field_self()));
}

/// A `.field()` accessor honors `.name()` but nothing else — expand
/// overrides are rejected at decl time (was a silent discard).
#[test]
#[should_panic(expected = "only .name() is honored")]
fn expand_return_field_with_expand_override_rejected() {
    let _ = crate::expand_return!(ZThing).field(
        crate::fun!(z_thing_name).expand_param("v", crate::expand_param!(ZName).variant_self()),
    );
}

/// Positive pin for the asymmetry: `.name()` on a `.field()` accessor is
/// the documented way to name the field — still accepted.
#[test]
fn expand_return_field_with_name_accepted() {
    let _ = crate::expand_return!(ZThing).field(crate::fun!(z_thing_name).name("label"));
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("t").class(
            crate::ptr_class!(ZThing)
                .method(crate::fun!(z_thing_free_standing))
                .constructor(crate::fun!(z_make)),
        ),
    );
    let err = registry.resolve(jni).expect_err("receiver-less member");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("t").class(
            crate::ptr_class!(ZThing)
                .method(crate::fun!(z_thing_len))
                .constructor(crate::fun!(z_make_number)),
        ),
    );
    let err = registry.resolve(jni).expect_err("wrong ctor return");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("enc")
                .class(crate::ptr_class!(ZEnc).method(crate::fun!(z_enc_get_id)))
                .fun(crate::fun!(z_enc_make)),
        )
        .expand(
            crate::expand_return!(ZEnc)
                .field(crate::fun!(z_enc_get_id))
                .field(
                    crate::fun!(crate::enc_if_custom)
                        .sig(crate::sig!((e: &ZEnc) -> Option<&ZEnc>))
                        .name("handle"),
                ),
        );
    let dir = unique_test_dir("jnigen_local_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let _ = crate::lang::FunctionDecl::new_local(syn::parse_quote!(enc_if_custom));
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("enc")
                .class(crate::ptr_class!(ZEnc))
                .fun(crate::fun!(z_enc_make)),
        )
        .expand(
            // `z_enc_get_id` names a real #[prebindgen] fn — a binding-local
            // field may not shadow it.
            crate::expand_return!(ZEnc).field(
                crate::fun!(crate::z_enc_get_id)
                    .sig(crate::sig!((e: &ZEnc) -> i32))
                    .name("id"),
            ),
        );
    let err = registry
        .resolve(jni)
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("msg")
                .class(crate::ptr_class!(ZEnc).method(crate::fun!(z_enc_get_id)))
                .class(crate::ptr_class!(ZMsg))
                .fun(crate::fun!(z_msg_make)),
        )
        .expand(
            crate::expand_return!(ZEnc)
                .field(crate::fun!(z_enc_get_id))
                .field(
                    crate::fun!(crate::enc_if_custom)
                        .sig(crate::sig!((e: &ZEnc) -> Option<&ZEnc>))
                        .name("handle"),
                ),
        )
        .expand(
            crate::expand_return!(ZMsg)
                .field(crate::fun!(z_msg_len).name("len"))
                .field(crate::fun!(z_msg_enc).name("enc")),
        );
    let dir = unique_test_dir("jnigen_local_field_splice");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("t")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(crate::fun!(z_thing_len))
                        // binding-local INSTANCE METHOD (receiver &Self first)
                        .method(
                            crate::fun!(crate::z_thing_ratio)
                                .sig(crate::sig!((t: &ZThing, scale: f64) -> f64)),
                        )
                        // binding-local COMPANION CONSTRUCTOR
                        .constructor(
                            crate::fun!(crate::z_thing_from_len)
                                .sig(crate::sig!((len: i64) -> ZThing)),
                        ),
                )
                // binding-local FREE FUNCTION, fallible (Result -> onError)
                .fun(
                    crate::fun!(crate::z_thing_describe)
                        .sig(crate::sig!((t: &ZThing, verbose: bool) -> Result<String, String>)),
                )
                .fun(crate::fun!(z_use)),
        )
        // The local constructor also serves as an expand_param! variant arm,
        // referenced by IDENT like any registry fn.
        .expand(
            crate::expand_param!(ZThing)
                .variant(crate::fun!(z_thing_from_len))
                .variant_self(),
        );
    let dir = unique_test_dir("jnigen_local_funs");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
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
                        .constructor(crate::fun!(z_thing_make))
                        // local METHOD, no .name(): hook sees `zThingRatio`.
                        .method(
                            crate::fun!(crate::sub::z_thing_ratio)
                                .sig(crate::sig!((t: &ZThing, scale: f64) -> f64)),
                        ),
                )
                // local FREE FN, no .name(): fun hook sees `zThingTag`.
                .fun(crate::fun!(crate::sub::z_thing_tag).sig(crate::sig!((t: &ZThing) -> i64)))
                .fun(crate::fun!(z_thing_query)),
        )
        // local FIELD, no .name(): defaults to camel(last segment). A second
        // field (the handle) keeps the decomposition on the builder path —
        // a single leaf would deliver by direct return, hiding the name.
        .expand(
            crate::expand_return!(ZThing)
                .field(crate::fun!(crate::sub::z_thing_len).sig(crate::sig!((t: &ZThing) -> i64)))
                .field_self(),
        );
    let raw = write_all(
        registry.resolve(jni).expect("resolve"),
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
    let _ = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("t").fun(crate::fun!(crate::z_no_sig)));
}

/// `.sig(…)` on an ident-built (registry) `fun!` is a hard error — the
/// signature is read from the registry.
#[test]
#[should_panic(expected = "read from the")]
fn sig_on_registry_fun_rejected() {
    let _ = crate::fun!(z_thing_len).sig(crate::sig!((t: &ZThing) -> i64));
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("t").class(crate::ptr_class!(ZThing)).fun(
            // shadows the #[prebindgen] fn of the same name
            crate::fun!(crate::z_thing_len).sig(crate::sig!((t: &ZThing) -> i64)),
        ),
    );
    let err = registry
        .resolve(jni)
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("t")
            .class(
                crate::ptr_class!(ZThing)
                    .gc_managed()
                    .constructor(crate::fun!(z_thing_new)),
            )
            .class(crate::ptr_class!(ZOther).constructor(crate::fun!(z_other_new)))
            .fun(crate::fun!(z_thing_use))
            .fun(crate::fun!(z_other_use)),
    );
    let raw = write_all(registry.resolve(jni).expect("resolve"), "jnigen_gc_managed");
    let all: String = raw.split_whitespace().collect();

    // Shared harness: cell-backed base, CAS helper, shared Cleaner, register fn.
    assert!(all.contains("abstractclassGcNativeHandle"), "{raw}");
    assert!(all.contains("internalfunreleaseCell"), "{raw}");
    assert!(all.contains("internalobjectNativeCleaner"), "{raw}");
    assert!(all.contains("internalfunregisterGcHandle"), "{raw}");

    // The gc class extends GcNativeHandle and self-registers via the cell.
    assert!(
        all.contains("classZThing(initialPtr:Long):GcNativeHandle(initialPtr)"),
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
            "valp=releaseCell(cell)__cleanable?.clean()returnZThing(if(p!=0L)pelsecell.get())"
        ),
        "{raw}"
    );

    // The plain class keeps the field-backed lifecycle.
    assert!(
        all.contains("classZOther(initialPtr:Long):NativeHandle(initialPtr)"),
        "{raw}"
    );
    assert!(all.contains("ptr=por1L"), "{raw}");
    assert!(
        !all.contains("classZOther(initialPtr:Long):GcNativeHandle"),
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
fn split_fixture(extra: &[&str]) -> Registry<KotlinMeta> {
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
    Registry::<KotlinMeta>::from_items(items).expect("index items")
}

fn write_all(gen: crate::api::core::Generation<JniGen>, tag: &str) -> String {
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(crate::fun!(z_store_expect).split_on_param("expected")),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(registry.resolve(jni).expect("resolve"), "jnigen_split_one");
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    crate::fun!(z_prefer)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(registry.resolve(jni).expect("resolve"), "jnigen_split_prod");
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    crate::fun!(z_summarize)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        )
        .expand(
            crate::expand_return!(ZSummary)
                .field(crate::fun!(z_summary_count))
                .field(crate::fun!(z_summary_total)),
        );
    let raw = write_all(
        registry.resolve(jni).expect("resolve"),
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops").class(crate::ptr_class!(ZThing)).fun(
                crate::fun!(z_combine)
                    .split_on_param("primary")
                    .split_on_param("fallback"),
            ),
        )
        .expand(
            crate::expand_param!(ZThing)
                .variant(crate::fun!(z_thing_one))
                .variant(crate::fun!(z_thing_two)),
        );
    let _ = write_all(
        registry.resolve(jni).expect("resolve"),
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(crate::fun!(z_use_name)), // NOT split — still errors
        )
        .expand(
            crate::expand_param!(ZName)
                .variant(crate::fun!(z_name_from_text))
                .variant(crate::fun!(z_name_from_label)),
        );
    let _ = write_all(registry.resolve(jni).expect("resolve"), "jnigen_split_decl");
}

/// #90: the validation boundary is cross-artifact — a colliding split
/// declaration (a Kotlin-side concern) fails `write_rust` too, as a clean
/// `Err` BEFORE the Rust file is written, so no half-written binding can
/// exist regardless of write order.
#[test]
fn split_declaration_collision_fails_write_rust_before_writing() {
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(crate::fun!(z_use_name)),
        )
        .expand(
            crate::expand_param!(ZName)
                .variant(crate::fun!(z_name_from_text))
                .variant(crate::fun!(z_name_from_label)),
        );
    let generation = registry.resolve(jni).expect("resolve");
    let dir = unique_test_dir("jnigen_split_decl_rust_err");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("gen.rs");
    let err = generation
        .write_rust(&out)
        .expect_err("colliding split declaration must fail write_rust");
    assert!(
        err.to_string().contains("same JVM signature"),
        "unexpected error: {err}"
    );
    assert!(
        !out.exists(),
        "write_rust must not write on validation failure"
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZName))
                .fun(crate::fun!(z_use_name)),
        )
        .expand(
            crate::expand_param!(ZName)
                .variant(crate::fun!(z_name_from_text))
                .variant(crate::fun!(z_name_from_label))
                .no_split(),
        );
    // No panic: the colliding variants are tolerated as selector-only.
    let raw = write_all(registry.resolve(jni).expect("resolve"), "jnigen_no_split");
    let all: String = raw.split_whitespace().collect();
    assert!(all.contains("nameSel:Int"), "{raw}"); // selector form emitted
}

/// #52: `.split_on_param` naming a parameter that does not exist on the
/// function is a hard error (typo guard).
#[test]
#[should_panic(expected = "no parameter named")]
fn split_on_unknown_param_rejected() {
    let registry = split_fixture(&[]);
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(crate::fun!(z_store_expect).split_on_param("nope")),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        );
    let _ = write_all(registry.resolve(jni).expect("resolve"), "jnigen_split_typo");
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(crate::fun!(z_maybe).split_on_param("opt")),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(registry.resolve(jni).expect("resolve"), "jnigen_split_opt");
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(crate::fun!(z_maybe).split_on_param("opt")),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant(crate::fun!(z_summary_scaled)),
        );
    let _ = write_all(
        registry.resolve(jni).expect("resolve"),
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZSummary))
                .fun(
                    crate::fun!(z_mixed)
                        .split_on_param("primary")
                        .split_on_param("fallback"),
                ),
        )
        .expand(
            crate::expand_param!(ZSummary)
                .variant(crate::fun!(z_summary_new))
                .variant_self(),
        );
    let raw = write_all(
        registry.resolve(jni).expect("resolve"),
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(crate::ptr_class!(ZEnc).constructor(crate::fun!(z_enc_from_id)))
                .fun(crate::fun!(z_put)),
        )
        .expand(
            crate::expand_param!(ZEnc)
                .variant(crate::fun!(z_enc_from_id))
                .variant_self(),
        );
    let dir = unique_test_dir("jnigen_opt_selector");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(
                    crate::ptr_class!(ZThing)
                        .constructor(crate::fun!(z_thing_make).name("make"))
                        .method(crate::fun!(z_thing_name).name("name")),
                )
                .fun(crate::fun!(z_thing_get)),
        )
        // Canonical output for ZThing: any ZThing-returning declared fn gets
        // callback delivery by default…
        .expand(
            crate::expand_return!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );
    let gen = registry.resolve(jni).expect("resolve");
    let registry = gen.registry();
    // …the free fn is decomposed…
    assert!(
        registry.unfold_plans.contains_key(&syn::Ident::new(
            "z_thing_get",
            proc_macro2::Span::call_site()
        )),
        "free fn gets the default output expansion"
    );
    // …but the constructor member is NOT (its return is the factory value).
    assert!(
        !registry.unfold_plans.contains_key(&syn::Ident::new(
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
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
            .fun(crate::fun!(z_thing_get)),
    );
    let dir = unique_test_dir("jnigen_q95");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("qualified spellings resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    let ac: String = all.split_whitespace().collect();
    // The typed handle class with its instance method, and the typed factory
    // wrapper returning the class — the full declaration↔signature chain.
    assert!(ac.contains("classZThing(initialPtr:Long)"), "{all}");
    assert!(ac.contains("funname(onError:"), "{all}");
    assert!(
        ac.contains("funzThingGet(onError:JniErrorHandler<ZThing>):ZThing"),
        "{all}"
    );
}
