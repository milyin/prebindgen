//! Declared-const emission: a `#[prebindgen] const` declared via
//! `package!().constant(constant!(X))` surfaces as a Rust nullary JNI
//! getter extern plus a Kotlin top-level eagerly-initialized `val`.

use super::*;

fn const_items() -> Vec<(syn::Item, crate::SourceLocation)> {
    let loc = crate::SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(const_items()).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("cfg")
                .constant(crate::constant!(MAX_LEN))
                .constant(crate::constant!(GREETING).name("HELLO")),
        );

    let dir = unique_test_dir("jnigen_consts_basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    // Verbatim const re-emission + one extern getter each.
    assert!(rust.contains("pub const MAX_LEN"), "{rust}");
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
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let pkg = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/cfg.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("cfg package file");
    let pc: String = pkg.split_whitespace().collect();
    // Public eagerly-initialized vals, typed from the output converters;
    // the `.name()` override applies to the val, not the extern.
    assert!(pc.contains("valMAX_LEN:Long=constGetMaxLen("), "{pkg}");
    assert!(pc.contains("valHELLO:String=constGetGreeting("), "{pkg}");
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
    let mut registry = Registry::<KotlinMeta>::from_items(const_items()).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(crate::package!("cfg").constant(crate::constant!(MAX_LEN)))
        .ignore_const(crate::constant!(GREETING));

    let dir = unique_test_dir("jnigen_consts_undeclared");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    assert!(rust.contains("pub const MAX_LEN"), "{rust}");
    // The ignored const neither re-emits nor gets a getter.
    assert!(!rust.contains("GREETING"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect();
    assert!(!all.contains("GREETING"), "{all}");
}

/// A const whose type is a declared opaque handle is rejected with guidance
/// to use a factory function instead.
#[test]
#[should_panic(expected = "declared opaque handle")]
fn handle_const_rejected() {
    let loc = crate::SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("things")
                .class(crate::ptr_class!(ZThing))
                .fun(crate::fun!(thing_new))
                .constant(crate::constant!(DEFAULT_THING)),
        );

    let dir = unique_test_dir("jnigen_consts_handle_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry.write_rust(&jni, dir.join("gen.rs"));
}
