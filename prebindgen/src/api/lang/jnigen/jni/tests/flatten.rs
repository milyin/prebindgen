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
                        .fun(crate::fun!(z_thing_name).name("name"))
                        .fun(crate::fun!(z_thing_size).name("size")),
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
                .class(crate::ptr_class!(ZDetail).fun(crate::fun!(z_detail_code).name("code")))
                .class(
                    crate::ptr_class!(ZErr)
                        .fun(crate::fun!(z_err_message).name("message"))
                        .fun(crate::fun!(z_err_detail).name("detail")),
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

/// `.fun(f)` binds the `&Class` receiver to `this` (dropped from the
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
                    .fun(crate::fun!(z_thing_name).name("name"))
                    // A fun with extra params: `&ZThing` receiver + a `name: String` param.
                    .fun(crate::fun!(z_thing_rename).name("rename"))
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
            .class(crate::ptr_class!(ZThing).fun(crate::fun!(z_thing_name).name("name")))
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

/// `ignore_funs_where` (C2): one predicate acknowledges a whole naming
/// family — the matching undeclared fns are skipped without per-name
/// `ignore_fun` lines, no extern is emitted for them, and the generation
/// still succeeds with only the declared surface.
#[test]
fn ignore_funs_where_acknowledges_matching_family() {
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
        .ignore_funs_where(|name| name.starts_with("detail_const_"));
    // The predicate flows through the Prebindgen hook…
    {
        use crate::api::core::prebindgen::Prebindgen;
        let preds = jni.ignored_function_predicates();
        assert_eq!(preds.len(), 1);
        assert!(preds[0]("detail_const_a") && !preds[0]("z_len"));
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
