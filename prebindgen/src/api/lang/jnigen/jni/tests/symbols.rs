//! Whole-artifact Kotlin/JVM/native symbol validation (issue #89, stage 1):
//! the default mangler sanitizes Rust-derived names, while invalid `.name()`
//! / custom-hook output and genuine collisions are collected hard errors
//! surfaced before any file is written.

use super::*;

/// Run the pipeline to the `write_rust` boundary and return its result — the
/// point `validate_resolved` (and thus `validate_symbols`) runs. Also asserts
/// the Rust artifact is absent on error (nothing reaches disk).
fn write_result(tag: &str, registry: Registry<KotlinMeta>, jni: JniGen) -> Result<(), String> {
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let out = dir.join("gen.rs");
    match gen.write_rust(&out) {
        Ok(_) => Ok(()),
        Err(e) => {
            assert!(
                !out.exists(),
                "no artifact must be written on validation error"
            );
            Err(e.to_string())
        }
    }
}

fn one_fn(src: &str) -> Registry<KotlinMeta> {
    let f: syn::ItemFn = syn::parse_str(src).unwrap();
    Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), myflat_loc())]).expect("index")
}

/// A `.name()` override that isn't a legal Kotlin identifier is a hard error
/// naming the origin — the author can correct it in build.rs.
#[test]
fn invalid_name_override_is_error() {
    let registry = one_fn("pub fn z_do_thing() -> i64 { 0 }");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("thing").fun(crate::fun!(z_do_thing).name("when")));
    let err = write_result("jni_sym_name", registry, jni).expect_err("invalid .name()");
    assert!(err.contains("`when`"), "{err}");
    assert!(err.contains("not a valid Kotlin identifier"), "{err}");
}

/// A custom mangle hook that returns an illegal identifier is a hard error
/// (the hook is author code; the mangler was available to it).
#[test]
fn invalid_hook_output_is_error() {
    let registry = one_fn("pub fn z_do_thing() -> i64 { 0 }");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_fun_name_mangle(|_pkg, _name| "1bad".to_string())
        .package(crate::package!("thing").fun(crate::fun!(z_do_thing)));
    let err = write_result("jni_sym_hook", registry, jni).expect_err("invalid hook output");
    assert!(err.contains("`1bad`"), "{err}");
    assert!(err.contains("not a valid Kotlin identifier"), "{err}");
}

/// A default (Rust-derived) name that IS a valid Kotlin identifier passes
/// with no error — the common case, and the reason existing fixtures stay
/// byte-identical.
#[test]
fn valid_default_names_pass() {
    let registry = one_fn("pub fn z_do_thing() -> i64 { 0 }");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("thing").fun(crate::fun!(z_do_thing)));
    write_result("jni_sym_ok", registry, jni).expect("valid names must pass");
}

/// Two functions whose custom method hook collapses them onto one JNINative
/// method name produce a duplicate native symbol — a hard error naming both,
/// caught before the duplicate `#[no_mangle]` would fail Rust linking.
#[test]
fn duplicate_native_symbol_is_error() {
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_str("pub fn z_alpha() -> i64 { 0 }").unwrap()),
            myflat_loc(),
        ),
        (
            syn::Item::Fn(syn::parse_str("pub fn z_beta() -> i64 { 0 }").unwrap()),
            myflat_loc(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    // The JNINative extern method name (which the `Java_…` symbol derives
    // from) goes through the method hook; collapsing it onto one name for
    // every function forces two distinct fns to share a native symbol.
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_method_name_mangle(|_pkg, _class, _name| "collide".to_string())
        .package(
            crate::package!("thing")
                .fun(crate::fun!(z_alpha))
                .fun(crate::fun!(z_beta)),
        );
    let err = write_result("jni_sym_dupnative", registry, jni).expect_err("duplicate symbol");
    assert!(err.contains("duplicate native symbol"), "{err}");
    assert!(err.contains("z_alpha") && err.contains("z_beta"), "{err}");
}

/// A Rust struct field named like a Kotlin keyword is silently sanitized by
/// the default mangler (emitted as `object_`) — no error, and the surrounding
/// binding still resolves and writes.
#[test]
fn keyword_struct_field_is_sanitized_not_error() {
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Payload {
                    pub object: i64,
                    pub value: f64,
                }
            )),
            myflat_loc(),
        ),
        (
            syn::Item::Fn(syn::parse_str("pub fn make() -> Payload { todo!() }").unwrap()),
            myflat_loc(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(crate::data_class!(Payload))
            .fun(crate::fun!(make)),
    );
    // The keyword field is sanitized (mangle → `object_`), not rejected.
    let dir = unique_test_dir("jni_sym_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs"))
        .expect("keyword field sanitized, not an error");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    let ac: String = all.split_whitespace().collect();
    // The property surfaces under its sanitized name.
    assert!(
        ac.contains("valobject_:Long") || ac.contains("object_:Long"),
        "{all}"
    );
    assert!(
        !ac.contains("valobject:Long"),
        "unsanitized keyword leaked:\n{all}"
    );
}

/// The interface forced equal to its class name collides in the same package
/// — a collected top-level-name error (previously an emission-time panic).
#[test]
fn class_interface_collision_is_error() {
    let registry = one_fn("pub fn z_thing_new() -> ZThing { unimplemented!() }");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_interface_name_mangle(|_pkg, n| n.to_string()) // identity → iface == class
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).interface())
                .fun(crate::fun!(z_thing_new)),
        );
    let err = write_result("jni_sym_iface", registry, jni).expect_err("iface==class collision");
    assert!(
        err.contains("duplicate top-level Kotlin name `ZThing`"),
        "{err}"
    );
}
