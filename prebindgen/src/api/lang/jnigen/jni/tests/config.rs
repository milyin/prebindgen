use super::*;

/// A `convert!` on a Rust builtin is rejected up front, at construction
/// time — builtins already have converters, and generated calls would try
/// to crate-qualify the builtin.
#[test]
#[should_panic(expected = "convert!(usize): builtins already have converters")]
fn builtin_convert_type_panics() {
    let _ = crate::convert!(usize);
}

/// #54: `.implements(...)` — the ptr_class integration hatch. Declared
/// interfaces join the generated class's supertype list after the lifecycle
/// base; dotted FQNs are imported and shortened; the class body (close/take/
/// freePtr) is unchanged.
#[test]
fn ptr_class_implements_adds_interface_supertypes() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn z_thing_new() -> ZThing { unimplemented!() }").unwrap();
    let registry =
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(
                crate::ptr_class!(ZThing)
                    .implements("io.other.Resource")
                    .implements("LocalIface"),
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
    // The lifecycle members are untouched by the hatch.
    assert!(thing.contains("override fun close()"), "{thing}");
    assert!(thing.contains("fun take(): ZThing"), "{thing}");
}

/// A duplicate interface on one decl is a decl-time hard error.
#[test]
#[should_panic(expected = "the interface is already declared")]
fn ptr_class_duplicate_implements_rejected() {
    let _ = crate::ptr_class!(ZThing)
        .implements("io.other.Resource")
        .implements("io.other.Resource");
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
        .set_ptr_class_name_mangle(|n| n.strip_prefix('Z').unwrap_or(n).to_string());

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

/// C1: the default member name is namespace-relative — the class's Rust-name
/// prefix is stripped from the fn ident (underscore-insensitively), the rest
/// camel-cases. `.name()` stays verbatim; a no-prefix member falls back to
/// the full camelCase ident.
#[test]
fn member_names_default_to_namespace_relative() {
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
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("ke").class(
            crate::ptr_class!(KeyExpr)
                // Underscore-insensitive strip: `key_expr_` and `keyexpr_`
                // both spell the class namespace.
                .fun(crate::fun!(key_expr_get_str))
                .fun(crate::fun!(keyexpr_intersects))
                // No class prefix → full camelCase fallback.
                .fun(crate::fun!(shared_helper))
                // `.name()` is verbatim and wins over the derivation.
                .fun(crate::fun!(keyexpr_to_string).name("toStr")),
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

/// C1: `set_member_name_mangle` — the sixth hook — receives the
/// namespace-stripped camelCase default, skips `.name()`d members, and is
/// order-independent (registered AFTER the `.package(...)` declaration).
#[test]
fn member_name_mangle_hook_applies_order_independently() {
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
                    .fun(crate::fun!(thing_size))
                    .fun(crate::fun!(thing_label).name("label")),
            ),
        )
        // Registered AFTER the declaration — must still apply (settings are
        // order-independent by construction).
        .set_member_name_mangle(|n| format!("{n}Native"));
    let dir = unique_test_dir("jnigen_member_mangle");
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
    // Hook over the stripped default (`thing_size` → `size` → `sizeNative`);
    // note the strip matched the RUST short name ZThing? No — `thing_size`
    // has no `zthing` prefix, so the fallback full ident applies first:
    // `thingSize` → `thingSizeNative`.
    assert!(ac.contains("funthingSizeNative("), "{all}");
    // `.name("label")` bypasses the hook entirely.
    assert!(ac.contains("funlabel("), "{all}");
    assert!(!ac.contains("funlabelNative("), "{all}");
}

/// #56: the harness hook follows the same contract as the other five —
/// it receives the DERIVED DEFAULT for its tier (`"JNINative"`, an explicit
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

/// C4: the Kotlin root is generator-owned — `write_kotlin` deletes and
/// recreates it on every run, so stale files from a previous generation
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
    // A stale file from a "previous run" at a path this generation won't
    // write — wiped with the rest of the owned root.
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
                        .fun(crate::fun!(summary_count))
                        .fun(crate::fun!(summary_total)),
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
    assert!(
        report.contains("build: io.test.jni.ops.SummaryBuilder<R>"),
        "{report}"
    );
    assert!(
        report.contains("return `Summary` decomposed → [count, total] (Callback delivery)"),
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
    // Class members appear under the class heading with C1-derived names.
    assert!(
        report.contains("## class `io.test.jni.ops.Summary` (ptr_class, Rust `Summary`)"),
        "{report}"
    );
    assert!(report.contains("fun count("), "{report}");
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
                        .fun(crate::fun!(summary_count))
                        .fun(crate::fun!(summary_total)),
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
    assert!(all.contains("(`count`, `total`)"), "{all}");
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
