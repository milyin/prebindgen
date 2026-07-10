use super::*;

/// Order-sensitive global config (`package_prefix`, the mangle closures) is
/// baked into FQNs at declaration time — configuring it after the first
/// declaration is a hard builder error, not a silent mis-naming.
#[test]
#[should_panic(expected = "package_prefix must be configured before")]
fn late_package_prefix_panics() {
    let _ = JniGen::new()
        .ptr_class(syn::parse_quote!(ZThing))
        .package_prefix("io.test.jni");
}

/// Same guard for the per-kind name-mangle closures.
#[test]
#[should_panic(expected = "kotlin_fun_name_mangle must be configured before")]
fn late_mangle_closure_panics() {
    let _ = JniGen::new()
        .package("thing")
        .fun(syn::parse_quote!(thing_get))
        .kotlin_fun_name_mangle(|n| n.to_string());
}

/// A rank-0 wrapper on a Rust builtin generates a converter qualified with the
/// `source_module` (`myflat::usize`) — invalid Rust — so the registration is
/// rejected up front.
#[test]
#[should_panic(expected = "wrapper on builtin `usize`")]
fn builtin_wrapper_pattern_panics() {
    let _ = JniGen::new().output_wrapper(
        syn::parse_quote!(usize),
        |_r: &Registry<KotlinMeta>| -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
            Some((
                syn::parse_quote!(jni::sys::jlong),
                None,
                syn::parse_quote!(v as jni::sys::jlong),
            ))
        },
    );
}

/// Per-declaration class rename (`.name()`, the type-level dual of the per-fn
/// override) and base-package functions (`.fun` with no active subpackage —
/// mirroring class declarations, which could always live in the base package).
#[test]
fn per_class_name_and_base_package_fun() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn ping() -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        // Rename the handle class; the mangle closures do NOT apply to it.
        .kotlin_ptr_class_name_mangle(|n| format!("JNI{n}"))
        .ptr_class(syn::parse_quote!(ZThing))
        .name("Gadget")
        // Base-package functions: no `package(...)` context active.
        .fun(syn::parse_quote!(ping))
        .fun(syn::parse_quote!(thing_new));

    let dir = unique_test_dir("jnigen_class_name_base_fun");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The literal rename wins over both the default short name and the mangle.
    assert!(kc.contains("classGadget("), "{kotlin}");
    assert!(!kc.contains("ZThing("), "{kotlin}");
    assert!(!kc.contains("JNIZThing"), "{kotlin}");
    // Wrappers reference the renamed class.
    assert!(
        kc.contains("funthingNew(onError:JniErrorHandler<Gadget>):Gadget"),
        "{kotlin}"
    );
    // Base-package functions land in the base package file (which also hosts
    // NativeHandle), not in any subpackage.
    let base = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("base package file");
    assert!(base.contains("fun ping("), "{base}");
    assert!(base.contains("fun thingNew("), "{base}");
}
