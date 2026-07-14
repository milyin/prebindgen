use super::*;

/// A `convert!` on a Rust builtin is rejected up front, at construction
/// time — builtins already have converters, and generated calls would try
/// to crate-qualify the builtin.
#[test]
#[should_panic(expected = "convert!(usize): builtins already have converters")]
fn builtin_convert_type_panics() {
    let _ = crate::convert!(usize);
}

/// Per-declaration class rename (`.name()`, the type-level dual of the per-fn
/// override) and base-package functions (`.fun` with the empty subpackage —
/// mirroring class declarations, which could always live in the base package).
#[test]
fn per_class_name_and_base_package_fun() {
    use crate::SourceLocation;
    let loc = myflat_loc();
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
        .set_package_prefix("io.test.jni")
        // Rename the handle class; the mangle closures do NOT apply to it.
        .set_ptr_class_name_mangle(|n| format!("JNI{n}"))
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZThing).name("Gadget"))
                .fun(crate::fun!(ping))
                .fun(crate::fun!(thing_new)),
        );

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

/// Setters are order-insensitive: declaring the package FIRST and applying
/// `set_package_prefix` + a class mangle AFTER must produce the same output
/// as the conventional settings-first order — the setter re-derives every
/// declared class's FQN from the retained raw declaration inputs.
#[test]
fn setters_after_declarations_apply() {
    use crate::SourceLocation;
    let loc = myflat_loc();
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
    ];
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    // Declarations first, settings last.
    let jni = JniGen::new()
        .package(
            crate::package!("things")
                .class(crate::ptr_class!(ZThing))
                .fun(crate::fun!(thing_new)),
        )
        .set_package_prefix("io.late.jni")
        .set_ptr_class_name_mangle(|n| n.strip_prefix('Z').unwrap_or(n).to_string());

    let dir = unique_test_dir("jnigen_setters_after_decls");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");

    // The late-set package prefix drives the file layout...
    let things = paths
        .iter()
        .find(|p| p.ends_with("io/late/jni/things.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("subpackage file under the late-set prefix");
    // ...and the late-set mangle drives the class name (`ZThing` → `Thing`).
    let tc: String = things.split_whitespace().collect();
    assert!(tc.contains("classThing("), "{things}");
    assert!(!tc.contains("classZThing("), "{things}");
    assert!(
        tc.contains("funthingNew(onError:JniErrorHandler<Thing>):Thing"),
        "{things}"
    );
}
