//! Declared-const emission: a `#[prebindgen] const` declared via
//! `package!().constant(constant!(X))` surfaces as a Rust nullary JNI
//! getter extern plus a Kotlin top-level lazily-initialized `val`
//! (`by lazy` — zero JNI calls at class-load, #58).

use super::*;

fn const_items() -> Vec<(syn::Item, crate::SourceLocation)> {
    let loc = myflat_loc();
    vec![
        (
            syn::Item::Const(syn::parse_quote!(
                pub const MAX_LEN: i64 = 42;
            )),
            loc.clone(),
        ),
        (
            syn::Item::Const(syn::parse_quote!(
                pub const GREETING: &str = "hi";
            )),
            loc.clone(),
        ),
    ]
}

/// End-to-end: both consts declared — the generated Rust contains the
/// verbatim consts and one getter extern each; the Kotlin package file
/// contains the two `val`s (typed `Int` / `String`) initialized through the
/// private helpers, and `JNINative` declares the matching `external fun`s.
#[test]
fn declared_consts_emit_getter_and_val() {
    let registry = Registry::<KotlinMeta>::from_items(const_items()).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("cfg")
            .constant(crate::constant!(MAX_LEN))
            .constant(crate::constant!(GREETING).name("HELLO")),
    );

    let dir = unique_test_dir("jnigen_consts_basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    // Path-alias const re-emission (the initializer tokens are never
    // copied — they may reference source-crate internals) + one extern
    // getter each.
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("pubconstMAX_LEN:i64=myflat::MAX_LEN;"),
        "{rust}"
    );
    assert!(
        rc.contains("pubconstGREETING:&str=myflat::GREETING;"),
        "{rust}"
    );
    assert!(
        !rc.contains("=42"),
        "initializer must not be copied: {rust}"
    );
    assert!(
        rust.contains("Java_io_test_jni_JNINative_constGetMaxLen"),
        "{rust}"
    );
    assert!(
        rust.contains("Java_io_test_jni_JNINative_constGetGreeting"),
        "{rust}"
    );
    // The getter's value comes from the const path, not a call.
    assert!(rust.contains("myflat::MAX_LEN"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let pkg = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/cfg.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("cfg package file");
    let pc: String = pkg.split_whitespace().collect();
    // Public lazily-initialized vals (`by lazy` — zero JNI calls at
    // class-load, #58), typed from the output converters; the `.name()`
    // override applies to the val, not the extern.
    assert!(
        pc.contains("valMAX_LEN:Longbylazy{constGetMaxLen("),
        "{pkg}"
    );
    assert!(
        pc.contains("valHELLO:Stringbylazy{constGetGreeting("),
        "{pkg}"
    );
    // No eager form anywhere in the package file.
    assert!(!pc.contains("=constGet"), "{pkg}");
    // The helpers are private wrapper functions.
    assert!(pc.contains("privatefunconstGetMaxLen("), "{pkg}");
    // JNINative declares the extern halves.
    let native = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("base package file");
    let nc: String = native.split_whitespace().collect();
    assert!(nc.contains("externalfunconstGetMaxLen("), "{native}");
    assert!(nc.contains("externalfunconstGetGreeting("), "{native}");
}

/// An undeclared const emits nothing (JniGen has a const declaration
/// mechanism, so const emission is declared-only); `ignore_const`
/// acknowledges it without emitting.
#[test]
fn undeclared_const_not_emitted() {
    let registry = Registry::<KotlinMeta>::from_items(const_items()).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("cfg").constant(crate::constant!(MAX_LEN)))
        .ignore(crate::constant!(GREETING));

    let dir = unique_test_dir("jnigen_consts_undeclared");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    assert!(rust.contains("pub const MAX_LEN"), "{rust}");
    // The ignored const neither re-emits nor gets a getter.
    assert!(!rust.contains("GREETING"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect();
    assert!(!all.contains("GREETING"), "{all}");
}

/// End-to-end for fn-sourced constants (`constant!(N).fun(…)`): a declared
/// nullary fn surfaces as a private helper + public lazily-initialized
/// `val` in the package file; the extern and the Rust wrapper are the
/// ordinary declared-function ones (`myflat::tag()` call).
#[test]
fn constant_fun_source_emits_val_over_ordinary_wrapper() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn tag() -> String {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("cfg").constant(crate::constant!(THE_TAG).fun(crate::fun!(tag))));

    let dir = unique_test_dir("jnigen_constant_fun_basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    // Ordinary declared-function wrapper: an extern calling `myflat::tag()`.
    assert!(rust.contains("Java_io_test_jni_JNINative_tag"), "{rust}");
    assert!(rust.contains("myflat::tag()"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let pkg = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/cfg.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("cfg package file");
    let pc: String = pkg.split_whitespace().collect();
    // Public lazily-initialized val named by `.name()`, over a PRIVATE
    // helper — no public callable fun.
    assert!(pc.contains("valTHE_TAG:Stringbylazy{tag("), "{pkg}");
    assert!(pc.contains("privatefuntag("), "{pkg}");
    assert!(!pc.contains("publicfuntag("), "{pkg}");
    // JNINative declares the ordinary extern.
    let native = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("base package file");
    let nc: String = native.split_whitespace().collect();
    assert!(nc.contains("externalfuntag("), "{native}");
}

/// A non-nullary fn cannot be a constant.
#[test]
#[should_panic(expected = "must be nullary")]
fn constant_fun_source_non_nullary_rejected() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn scaled(factor: i64) -> i64 {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("cfg").constant(crate::constant!(SCALED).fun(crate::fun!(scaled))),
    );
    let dir = unique_test_dir("jnigen_constant_fun_arity_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let _ = gen.write_kotlin(&dir.join("kotlin"));
}

/// A fn returning a declared opaque handle cannot be a constant — same
/// rejection (and guidance) as a handle-typed const.
#[test]
#[should_panic(expected = "declared opaque handle")]
fn constant_fun_source_handle_return_rejected() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn default_thing() -> ZThing {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("things")
            .class(crate::ptr_class!(ZThing))
            .constant(crate::constant!(DEFAULT_THING).fun(crate::fun!(default_thing))),
    );
    let dir = unique_test_dir("jnigen_constant_fun_handle_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let _ = gen.write_kotlin(&dir.join("kotlin"));
}

/// End-to-end for expression-sourced constants (`constant!(N).expr(…)`):
/// the binding-defined expression is evaluated inside a generated nullary
/// getter (with every source module glob-imported, composing source items
/// without the source crate exporting a dedicated accessor), and surfaces as
/// a private helper + public eagerly-initialized `val`.
#[test]
fn constant_expr_emits_getter_and_val() {
    let loc = myflat_loc();
    // Only `tag_of` exists in the source crate; the constant composes it.
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn tag_of(n: i64) -> String {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("cfg").fun(crate::fun!(tag_of)).constant(
            crate::constant!(DEFAULT_TAG).expr(crate::ty!(String), crate::expr!(tag_of(7))),
        ),
    );

    let dir = unique_test_dir("jnigen_constant_expr_basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The getter extern is seeded from the val name and evaluates the
    // expression with the source module's items in scope.
    assert!(
        rust.contains("Java_io_test_jni_JNINative_constGetDefaultTag"),
        "{rust}"
    );
    assert!(rc.contains("usemyflat::*;"), "{rust}");
    assert!(rc.contains("tag_of(7)"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let pkg = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/cfg.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("cfg package file");
    let pc: String = pkg.split_whitespace().collect();
    assert!(
        pc.contains("valDEFAULT_TAG:Stringbylazy{constGetDefaultTag("),
        "{pkg}"
    );
    assert!(pc.contains("privatefunconstGetDefaultTag("), "{pkg}");
    // JNINative declares the getter extern.
    let native = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("base package file");
    let nc: String = native.split_whitespace().collect();
    assert!(nc.contains("externalfunconstGetDefaultTag("), "{native}");
}

/// An expression constant whose declared type is an opaque handle is
/// rejected like every other constant kind.
#[test]
#[should_panic(expected = "declared opaque handle")]
fn constant_expr_handle_type_rejected() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_new() -> ZThing {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("things")
            .class(crate::ptr_class!(ZThing))
            .fun(crate::fun!(thing_new))
            .constant(
                crate::constant!(DEFAULT_THING).expr(crate::ty!(ZThing), crate::expr!(thing_new())),
            ),
    );
    let dir = unique_test_dir("jnigen_constant_expr_handle_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry
        .resolve(jni)
        .and_then(|gen| gen.write_rust(dir.join("gen.rs")));
}

/// A const whose type is a declared opaque handle is rejected with guidance
/// to use a factory function instead.
#[test]
#[should_panic(expected = "declared opaque handle")]
fn handle_const_rejected() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Const(syn::parse_quote!(
                pub const DEFAULT_THING: ZThing = ZThing { _p: 0 };
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_new() -> ZThing {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("things")
            .class(crate::ptr_class!(ZThing))
            .fun(crate::fun!(thing_new))
            .constant(crate::constant!(DEFAULT_THING)),
    );

    let dir = unique_test_dir("jnigen_consts_handle_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry
        .resolve(jni)
        .and_then(|gen| gen.write_rust(dir.join("gen.rs")));
}

/// `.with(ty!, path!)` — the binding-local nullary fn source, const analog
/// of `ConvertSourceDecl::with`: lowers to an expression getter that calls
/// the path verbatim (multi-segment paths bypass source-module
/// qualification, exactly like convert's `.with`).
#[test]
fn constant_with_source_calls_path_verbatim() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn unrelated() -> i64 {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("cfg").fun(crate::fun!(unrelated)).constant(
            crate::constant!(COVER_VERSION)
                .with(crate::ty!(String), crate::path!(crate::cover_version)),
        ),
    );
    let dir = unique_test_dir("jnigen_constant_with_basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rust.contains("Java_io_test_jni_JNINative_constGetCoverVersion"),
        "{rust}"
    );
    // The path is called verbatim — no `myflat::` qualification.
    assert!(rc.contains("crate::cover_version()"), "{rust}");
    assert!(!rc.contains("myflat::cover_version"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let pkg = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/cfg.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("cfg package file");
    let pc: String = pkg.split_whitespace().collect();
    assert!(
        pc.contains("valCOVER_VERSION:Stringbylazy{constGetCoverVersion("),
        "{pkg}"
    );
}

/// A constant has exactly ONE value source — a second modifier panics.
#[test]
#[should_panic(expected = "value source already set")]
fn constant_second_source_rejected() {
    let _ = crate::constant!(X)
        .expr(crate::ty!(i64), crate::expr!(1 + 1))
        .with(crate::ty!(i64), crate::path!(crate::f));
}

/// `ConstDecl::named(...)` is the runtime subject form for declaration
/// loops — equivalent to the `constant!` macro with the same ident.
#[test]
fn constant_named_runtime_form_matches_macro() {
    let a = crate::lang::ConstDecl::named(format!("ENCODING_{}", "ZENOH_BYTES"))
        .expr(crate::ty!(String), crate::expr!(enc()));
    let b = crate::constant!(ENCODING_ZENOH_BYTES).expr(crate::ty!(String), crate::expr!(enc()));
    assert_eq!(a.val_name(), b.val_name());
    assert_eq!(a.rust_ident, b.rust_ident);
}

/// A non-identifier runtime name is a decl-time panic (it seeds the extern
/// symbol).
#[test]
#[should_panic(expected = "not a valid identifier")]
fn constant_named_invalid_ident_rejected() {
    let _ = crate::lang::ConstDecl::named("NOT AN IDENT");
}
