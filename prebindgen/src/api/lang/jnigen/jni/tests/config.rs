use super::*;

/// A `convert!` on a Rust builtin is rejected up front, at construction
/// time — builtins already have converters, and generated calls would try
/// to crate-qualify the builtin.
#[test]
#[should_panic(expected = "convert!(usize): builtins already have converters")]
fn builtin_convert_type_panics() {
    let _ = crate::convert!(usize);
}

/// #54: `.implements(...)` WITHOUT `.interface()` — declared interfaces join
/// the class's supertype list (nominal); dotted FQNs are imported and
/// shortened; the class body is untouched and NO generated interface is
/// emitted (members stay non-`override`).
#[test]
fn ptr_class_implements_adds_interface_supertypes() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_new() -> ZThing { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| (syn::Item::Fn(syn::parse_str(src).unwrap()), loc.clone()))
        .collect();
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .implements("io.other.Resource")
                    .implements("LocalIface")
                    .method(crate::fun!(z_thing_size)),
            )
            .fun(crate::fun!(z_thing_new)),
    );
    let dir = unique_test_dir("jnigen_implements");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let thing = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/thing.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("thing package file");
    assert!(
        thing.contains(
            "class ZThing(initialPtr: Long) : NativeHandle(initialPtr), Resource, LocalIface {"
        ),
        "{thing}"
    );
    assert!(thing.contains("import io.other.Resource"), "{thing}");
    // No generated interface, no `override` on the declared member.
    assert!(!thing.contains("interface ZThingApi"), "{thing}");
    assert!(thing.contains("public fun zThingSize("), "{thing}");
    assert!(!thing.contains("override fun zThingSize("), "{thing}");
}

/// #54: `.interface()` — the generated `<Name>Api` interface mirrors the
/// class's public surface, the class implements it with `override` on
/// class-body members (peek/isClosed inherited from NativeHandle need none),
/// and a user `.implements` interface can extend it.
#[test]
fn ptr_class_interface_emits_generated_api() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_thing_new() -> ZThing { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| (syn::Item::Fn(syn::parse_str(src).unwrap()), loc.clone()))
        .collect();
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .interface()
                    .implements("io.other.Resource")
                    .method(crate::fun!(z_thing_size)),
            )
            .fun(crate::fun!(z_thing_new)),
    );
    let dir = unique_test_dir("jnigen_interface");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let thing = paths
        .iter()
        .find(|p| p.ends_with("io/test/jni/thing.kt"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .expect("thing package file");
    let tc: String = thing.split_whitespace().collect();
    // Generated interface with the inherited-surface abstracts + the member,
    // extending AutoCloseable.
    assert!(tc.contains("interfaceZThingApi:AutoCloseable{"), "{thing}");
    assert!(tc.contains("funpeek():Long"), "{thing}");
    assert!(tc.contains("funisClosed():Boolean"), "{thing}");
    assert!(tc.contains("funzThingSize("), "{thing}");
    // Class implements the generated interface + the user one; members override.
    assert!(
        tc.contains("classZThing(initialPtr:Long):NativeHandle(initialPtr),ZThingApi,Resource{"),
        "{thing}"
    );
    assert!(tc.contains("publicoverridefuntake():ZThing"), "{thing}");
    assert!(tc.contains("publicoverridefunzThingSize("), "{thing}");
}

/// A duplicate interface on one decl is a decl-time hard error.
#[test]
#[should_panic(expected = "the interface is already declared")]
fn ptr_class_duplicate_implements_rejected() {
    let _ = crate::ptr_class!(ZThing)
        .implements("io.other.Resource")
        .implements("io.other.Resource");
}

/// The `set_interface_name_mangle` hook receives the class package and final
/// name and its result must differ from the class name. An identity hook
/// makes the interface collide with the class in the same package — now a
/// COLLECTED error from the whole-artifact symbol pass (issue #89), surfaced
/// by `write_rust`/`write_kotlin` before any file is written, rather than an
/// emission-time panic.
#[test]
fn interface_name_mangle_identity_rejected() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_thing_new() -> ZThing { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_interface_name_mangle(|package, n| {
            assert_eq!(package, "io.test.jni.thing");
            n.to_string()
        })
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).interface())
                .fun(crate::fun!(z_thing_new)),
        );
    let dir = unique_test_dir("jnigen_iface_identity");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let err = gen
        .write_rust(dir.join("gen.rs"))
        .expect_err("interface==class must be a collected error");
    let msg = err.to_string();
    assert!(
        msg.contains("duplicate top-level Kotlin name `ZThing`"),
        "{msg}"
    );
    // The invalid binding must not have written a Rust artifact.
    assert!(!dir.join("gen.rs").exists());
}

/// A per-decl `.interface_name(...)` pins the interface name (and implies
/// `.interface()`); `set_interface_name_mangle` receives package + class name.
#[test]
fn interface_name_override_and_hook() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Mode {
                    A = 0,
                    B = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn flip(m: Mode) -> Mode {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_interface_name_mangle(|package, n| {
            assert_eq!(package, "io.test.jni.m", "hook receives the target package");
            assert_eq!(n, "Mode", "hook receives the class name");
            format!("{n}Contract")
        })
        .package(
            crate::package!("m")
                .class(crate::enum_class!(Mode).interface_name("ModeIface"))
                .fun(crate::fun!(flip)),
        );
    let dir = unique_test_dir("jnigen_iface_name");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let ac: String = all.split_whitespace().collect();
    // The per-decl override wins over the hook.
    assert!(ac.contains("interfaceModeIface{"), "{all}");
    assert!(ac.contains("valvalue:Int"), "{all}");
    assert!(
        ac.contains("enumclassMode(overridepublicvalvalue:Int):ModeIface{")
            || ac.contains("enumclassMode(publicoverridevalvalue:Int):ModeIface{"),
        "{all}"
    );
}

/// #54: `.interface()` on a `value_class!` — the generated `<Name>Api`
/// interface carries the `bytes` property and the accessors, and the
/// `@JvmInline value class` implements it with `override val bytes`
/// (the ctor-prop path, shared with data/enum classes).
#[test]
fn value_class_interface_emits_generated_api() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                #[derive(Clone, Copy)]
                pub struct ZStamp {
                    pub secs: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_stamp_secs(s: &ZStamp) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("t").class(
            crate::value_class!(ZStamp)
                .interface()
                .method(crate::fun!(z_stamp_secs).name("secs")),
        ),
    );
    let dir = unique_test_dir("jnigen_value_iface");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let ac: String = all.split_whitespace().collect();
    assert!(ac.contains("interfaceZStampApi{"), "{all}");
    assert!(ac.contains("valbytes:ByteArray"), "{all}");
    assert!(ac.contains("funsecs("), "{all}");
    assert!(
        ac.contains("valueclassZStamp(overridepublicvalbytes:ByteArray):ZStampApi{")
            || ac.contains("valueclassZStamp(publicoverridevalbytes:ByteArray):ZStampApi{"),
        "{all}"
    );
    assert!(ac.contains("publicoverridefunsecs("), "{all}");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        // Rename the handle class; the mangle closures do NOT apply to it.
        .set_ptr_class_name_mangle(|package, n| {
            assert_eq!(package, "io.test.jni");
            format!("JNI{n}")
        })
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZThing).name("Gadget"))
                .fun(crate::fun!(ping))
                .fun(crate::fun!(thing_new)),
        );

    let dir = unique_test_dir("jnigen_class_name_base_fun");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    // Declarations first, settings last.
    let jni = JniGen::new()
        .package(
            crate::package!("things")
                .class(crate::ptr_class!(ZThing))
                .fun(crate::fun!(thing_new)),
        )
        .set_package_prefix("io.late.jni")
        .set_ptr_class_name_mangle(|package, n| {
            assert_eq!(package, "io.late.jni.things");
            n.strip_prefix('Z').unwrap_or(n).to_string()
        });

    let dir = unique_test_dir("jnigen_setters_after_decls");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");

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

/// The I3 contract: after `Registry::resolve`, `write_kotlin` and
/// `write_rust` are pure reads on one receiver — calling Kotlin FIRST
/// produces byte-identical output to the usual order.
#[test]
fn generation_writes_are_order_free() {
    let build = || {
        let loc = myflat_loc();
        let f: syn::ItemFn =
            syn::parse_str("pub fn z_ping(v: i64) -> i64 { unimplemented!() }").unwrap();
        let registry =
            Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!("thing").fun(crate::fun!(z_ping)));
        registry.resolve(jni).expect("resolve")
    };
    let read_all = |dir: &std::path::Path, paths: &[std::path::PathBuf]| -> String {
        let mut out = String::new();
        let mut sorted: Vec<_> = paths.to_vec();
        sorted.sort();
        for p in sorted {
            out.push_str(&format!("== {}\n", p.strip_prefix(dir).unwrap().display()));
            out.push_str(&std::fs::read_to_string(&p).unwrap());
        }
        out
    };

    // Usual order: rust, then kotlin.
    let d1 = unique_test_dir("jnigen_orderfree_a");
    let _ = std::fs::remove_dir_all(&d1);
    std::fs::create_dir_all(&d1).unwrap();
    let gen = build();
    let rust1 = std::fs::read_to_string(gen.write_rust(d1.join("gen.rs")).unwrap()).unwrap();
    let kt1 = read_all(&d1, &gen.write_kotlin(&d1.join("kotlin")).unwrap());

    // Flipped order: kotlin FIRST.
    let d2 = unique_test_dir("jnigen_orderfree_b");
    let _ = std::fs::remove_dir_all(&d2);
    std::fs::create_dir_all(&d2).unwrap();
    let gen = build();
    let kt2 = read_all(&d2, &gen.write_kotlin(&d2.join("kotlin")).unwrap());
    let rust2 = std::fs::read_to_string(gen.write_rust(d2.join("gen.rs")).unwrap()).unwrap();

    assert_eq!(rust1, rust2);
    assert_eq!(kt1, kt2);
}

/// The method hook can use package + class context to strip a flat API's
/// encoded class namespace. `.name()` stays verbatim; an unmatched method
/// keeps its full camelCase ident.
#[test]
fn method_hook_can_strip_flat_class_prefix() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn key_expr_get_str(k: &KeyExpr) -> String { unimplemented!() }",
        "pub fn keyexpr_intersects(k: &KeyExpr, s: String) -> bool { unimplemented!() }",
        "pub fn shared_helper(k: &KeyExpr) -> i64 { unimplemented!() }",
        "pub fn keyexpr_to_string(k: &KeyExpr) -> String { unimplemented!() }",
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
        .set_method_name_mangle(|package, class, name| {
            if class == "JNINative" {
                return name.to_string();
            }
            assert_eq!(package, "io.test.jni.ke");
            assert_eq!(class, "KeyExpr");
            if name
                .get(..class.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(class))
            {
                let rest = &name[class.len()..];
                let mut chars = rest.chars();
                if let Some(first) = chars.next() {
                    return first.to_lowercase().chain(chars).collect();
                }
            }
            name.to_string()
        })
        .package(
            crate::package!("ke").class(
                crate::ptr_class!(KeyExpr)
                    .method(crate::fun!(key_expr_get_str))
                    .method(crate::fun!(keyexpr_intersects))
                    .method(crate::fun!(shared_helper))
                    // `.name()` is verbatim and wins over the hook.
                    .method(crate::fun!(keyexpr_to_string).name("toStr")),
            ),
        );
    let dir = unique_test_dir("jnigen_member_names");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let ac: String = all.split_whitespace().collect();
    assert!(ac.contains("fungetStr("), "{all}");
    assert!(ac.contains("funintersects("), "{all}");
    assert!(ac.contains("funsharedHelper("), "{all}");
    assert!(ac.contains("funtoStr("), "{all}");
    // The wrapper tier lost the namespace; the extern tier (JNINative)
    // keeps the full ident — that's the JNI symbol, not a member name.
    let ke_file: String = paths
        .iter()
        .filter(|p| p.ends_with("ke.kt"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("")
        .split_whitespace()
        .collect();
    assert!(!ke_file.contains("funkeyExprGetStr("), "{all}");
    assert!(ac.contains("externalfunkeyExprGetStr("), "{all}");
}

/// The method hook receives package, final class, and the full camelCase Rust
/// name, skips `.name()`d methods, and is
/// order-independent (registered AFTER the `.package(...)` declaration).
#[test]
fn method_name_mangle_hook_applies_order_independently() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn thing_size(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn thing_label(t: &ZThing) -> String { unimplemented!() }",
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
            crate::package!("t").class(
                crate::ptr_class!(ZThing)
                    .name("Thing")
                    .method(crate::fun!(thing_size))
                    .method(crate::fun!(thing_label).name("label")),
            ),
        )
        // Registered AFTER the declaration — must still apply (settings are
        // order-independent by construction).
        .set_method_name_mangle(|package, class, n| {
            if class == "JNINative" {
                return n.to_string();
            }
            assert_eq!(package, "io.test.jni.t");
            assert_eq!(class, "Thing");
            format!("{n}Native")
        });
    let dir = unique_test_dir("jnigen_method_mangle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let ac: String = all.split_whitespace().collect();
    // The hook receives the full default (`thing_size` → `thingSize`).
    assert!(ac.contains("funthingSizeNative("), "{all}");
    // `.name("label")` bypasses the hook entirely.
    assert!(ac.contains("funlabel("), "{all}");
    assert!(!ac.contains("funlabelNative("), "{all}");
}

/// #56: the harness hook receives the DERIVED DEFAULT for its tier
/// (`"JNINative"`, an explicit
/// default value, not a hidden `JNI`-prepend) and replaces it wholesale;
/// the unset default is identity.
#[test]
fn harness_hook_receives_derived_default() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_ping(v: i64) -> i64 { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_harness_name_mangle(|n| {
            assert_eq!(n, "JNINative", "hook must receive the derived default");
            "MyNative".to_string()
        })
        .package(crate::package!("thing").fun(crate::fun!(z_ping)));
    let dir = unique_test_dir("jnigen_harness_mangle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    // The extern symbol path carries the replaced harness name.
    assert!(rust.contains("Java_io_test_jni_MyNative_zPing"), "{rust}");
    assert!(!rust.contains("JNINative"), "{rust}");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("object MyNative"), "{all}");
    assert!(!all.contains("JNINative"), "{all}");
}

/// Top-level functions and class methods are separate naming tiers. The
/// former sees its destination subpackage; the JNI extern sees the base
/// package plus its generated harness class.
#[test]
fn function_and_native_method_hooks_receive_placement() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_session_ping(v: i64) -> i64 { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .set_fun_name_mangle(|package, name| {
            assert_eq!(package, "io.test.jni.session");
            assert_eq!(name, "zSessionPing");
            "ping".to_string()
        })
        .set_method_name_mangle(|package, class, name| {
            assert_eq!(package, "io.test.jni");
            assert_eq!(class, "JNINative");
            assert_eq!(name, "zSessionPing");
            format!("{name}Native")
        })
        .package(crate::package!("session").fun(crate::fun!(z_session_ping)));
    let dir = unique_test_dir("jnigen_placement_mangles");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("public fun ping("), "{all}");
    assert!(all.contains("external fun zSessionPingNative("), "{all}");
    assert!(
        rust.contains("Java_io_test_jni_JNINative_zSessionPingNative"),
        "{rust}"
    );
}

/// C4: after `write_kotlin` creates its ownership marker, later runs replace
/// the entire generated tree, so stale files from a previous generation
/// (renamed package, removed declaration) never linger and consumers need
/// no cleanup boilerplate. Repeat runs are idempotent.
#[test]
fn write_kotlin_owns_and_resets_the_root() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_ping(v: i64) -> i64 { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("thing").fun(crate::fun!(z_ping)));
    let gen = registry.resolve(jni).expect("resolve");

    let dir = unique_test_dir("jnigen_owned_root");
    let _ = std::fs::remove_dir_all(&dir);
    let root = dir.join("kotlin");
    // Bootstrap the generator-owned root, then leave a stale file from a
    // previous generation at a path this generation won't write.
    gen.write_kotlin(&root).expect("initial write_kotlin");
    let stale_dir = root.join("io/test/jni/old");
    std::fs::create_dir_all(&stale_dir).unwrap();
    let stale = stale_dir.join("stale.kt");
    std::fs::write(&stale, "package io.test.jni.old\n").unwrap();

    let paths = gen.write_kotlin(&root).expect("write_kotlin");
    assert!(!stale.exists(), "stale file must be wiped with the root");
    assert!(!stale_dir.exists());
    assert!(paths.iter().all(|p| p.exists()));

    // Idempotent: a second run rewrites the same set.
    let paths2 = gen.write_kotlin(&root).expect("write_kotlin again");
    assert_eq!(paths, paths2);
    assert!(paths2.iter().all(|p| p.exists()));
}

/// C7: `Generation::report()` — the explain mode. The report carries the
/// FINAL Kotlin signature of each fn (same render path as the emitters),
/// the plans that shaped it, an unshaped fn with no `shaped by:` lines,
/// and the type table. Deterministic across calls.
#[test]
fn report_explains_the_resolved_surface() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn summary_count(s: &Summary) -> i64 { unimplemented!() }",
        "pub fn summary_total(s: &Summary) -> i64 { unimplemented!() }",
        "pub fn storage_summary(v: i64) -> Summary { unimplemented!() }",
        "pub fn z_plain(v: i64) -> i64 { unimplemented!() }",
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
                    crate::ptr_class!(Summary)
                        .method(crate::fun!(summary_count))
                        .method(crate::fun!(summary_total)),
                )
                .fun(crate::fun!(storage_summary))
                .fun(crate::fun!(z_plain)),
        )
        .expand(
            crate::expand_return!(Summary)
                .field(crate::fun!(summary_count))
                .field(crate::fun!(summary_total)),
        );
    let gen = registry.resolve(jni).expect("resolve");
    let report = gen.report();

    // The reshaped fn: exact signature (builder callback form) + provenance.
    assert!(report.contains("`storage_summary`"), "{report}");
    // The report shortens surface types uniformly (issue #89 follow-up:
    // rendered through an `ImportSet`), so the builder param shows the short
    // interface name.
    assert!(report.contains("build: SummaryBuilder<R>"), "{report}");
    assert!(
        report.contains(
            "return `Summary` decomposed → [summaryCount, summaryTotal] (Callback delivery)"
        ),
        "{report}"
    );
    // The plain fn appears with no `shaped by:` after it.
    let plain_at = report.find("`z_plain`").expect("plain fn listed");
    let after_plain = &report[plain_at..];
    let next_entry = after_plain[1..]
        .find("- `")
        .map(|i| i + 1)
        .unwrap_or(after_plain.len());
    assert!(
        !after_plain[..next_entry].contains("shaped by:"),
        "{report}"
    );
    // Class methods appear under the class heading with their effective names.
    assert!(
        report.contains("## class `io.test.jni.ops.Summary` (ptr_class, Rust `Summary`)"),
        "{report}"
    );
    assert!(report.contains("fun summaryCount("), "{report}");
    // Types table with the jlong wire.
    assert!(
        report.contains("- `Summary`: ptr_class → `io.test.jni.ops.Summary`"),
        "{report}"
    );
    // Deterministic.
    assert_eq!(report, gen.report());
}

/// N1: `///` docs become KDoc, and shaped positions get generated notes
/// documenting the REAL prototype after expansions.
#[test]
fn docs_become_kdoc_with_shape_notes() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                /// A summary of stored payloads.
                pub struct Summary {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                /// Count of aggregated entries.
                pub fn summary_count(s: &Summary) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn summary_total(s: &Summary) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                /// Produces the storage summary.
                pub fn storage_summary(v: i64) -> Summary {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_plain(v: i64) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Const(syn::parse_quote!(
                /// The magic number.
                pub const MAGIC: i64 = 7;
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("ops")
                .class(
                    crate::ptr_class!(Summary)
                        .method(crate::fun!(summary_count))
                        .method(crate::fun!(summary_total)),
                )
                .fun(crate::fun!(storage_summary))
                .fun(crate::fun!(z_plain))
                .constant(crate::constant!(MAGIC)),
        )
        .expand(
            crate::expand_return!(Summary)
                .field(crate::fun!(summary_count))
                .field(crate::fun!(summary_total)),
        );
    let gen = registry.resolve(jni).expect("resolve");
    let dir = unique_test_dir("jnigen_kdoc");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");

    // Author prose on the wrapper + shape notes for the decomposed return.
    assert!(all.contains("Produces the storage summary."), "{all}");
    assert!(
        all.contains("The Rust `Summary` result is delivered decomposed"),
        "{all}"
    );
    assert!(all.contains("(`summaryCount`, `summaryTotal`)"), "{all}");
    // Member method carries the fn's doc (name derived by C1).
    assert!(all.contains("Count of aggregated entries."), "{all}");
    // Class kdoc: author prose FIRST, framework line after.
    let class_doc_at = all
        .find("A summary of stored payloads.")
        .expect("struct doc present");
    let framework_at = all
        .find("Typed handle for a native Zenoh `Summary`.")
        .expect("framework line kept");
    assert!(class_doc_at < framework_at, "{all}");
    // Const doc + framework line.
    assert!(all.contains("The magic number."), "{all}");
    assert!(
        all.contains("Mirrors the Rust `#[prebindgen]` const `MAGIC`"),
        "{all}"
    );
    // Undocumented, unshaped fn: no kdoc block directly above it.
    let plain_at = all.find("fun zPlain(").expect("plain fn present");
    let before = &all[..plain_at];
    let tail = &before[before.len().saturating_sub(120)..];
    assert!(!tail.contains("*/"), "unexpected kdoc above zPlain: {tail}");
}
