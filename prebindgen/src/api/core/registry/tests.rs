use std::collections::HashSet;

use proc_macro2::TokenStream;

use super::*;
use crate::api::core::{
    niches::Niches,
    prebindgen::{ConverterImpl, Prebindgen},
};

/// Minimal `Prebindgen` for scan-pipeline tests. Carries the
/// declared sets the test wants and stubs every emission/converter
/// hook into something inert.
#[derive(Default)]
struct StubExt {
    functions: HashSet<syn::Ident>,
    ignored_functions: HashSet<syn::Ident>,
    ignored_name_predicates: Vec<crate::api::core::prebindgen::NamePredicate>,
    helper_functions: HashSet<syn::Ident>,
    consts: Option<HashSet<syn::Ident>>,
    types: HashSet<TypeKey>,
    ignored_types: HashSet<TypeKey>,
    local_fns: Vec<(syn::ItemFn, String)>,
}

impl Prebindgen for StubExt {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.clone()
    }
    fn ignored_functions(&self) -> HashSet<syn::Ident> {
        self.ignored_functions.clone()
    }
    fn ignored_name_predicates(&self) -> Vec<crate::api::core::prebindgen::NamePredicate> {
        self.ignored_name_predicates.clone()
    }
    fn helper_functions(&self) -> HashSet<syn::Ident> {
        self.helper_functions.clone()
    }
    fn declared_consts(&self) -> Option<HashSet<syn::Ident>> {
        self.consts.clone()
    }
    fn declared_types(&self) -> HashSet<TypeKey> {
        self.types.clone()
    }
    fn ignored_types(&self) -> HashSet<TypeKey> {
        self.ignored_types.clone()
    }
    fn local_functions(&self) -> Vec<(syn::ItemFn, String)> {
        self.local_fns.clone()
    }

    fn on_function(&self, _f: &syn::ItemFn, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }
    fn on_struct(&self, _s: &syn::ItemStruct, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }
    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }
    fn on_input_type(
        &self,
        _ty: &syn::Type,
        _registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }
    fn on_output_type(
        &self,
        _ty: &syn::Type,
        _registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }
}

// suppress unused warning on Niches — kept available for richer tests
#[allow(dead_code)]
fn _force_niches_use() -> Niches {
    Niches::empty()
}

fn fn_item(src: &str) -> (syn::Item, SourceLocation) {
    let item: syn::ItemFn = syn::parse_str(src).expect("test fn parse");
    (syn::Item::Fn(item), SourceLocation::default())
}

#[test]
fn scan_declared_empty_ext_marks_nothing_required() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let ext = StubExt::default();
    reg.scan_declared(&ext).expect("empty ext = no scan");
    assert!(!reg.input_types.values().any(|c| c.root));
    assert!(!reg.output_types.values().any(|c| c.root));
}

#[test]
fn scan_declared_marks_types_required_only_for_declared_fns() {
    let items = vec![
        fn_item("fn a(x: u64) -> u64 { x }"),
        fn_item("fn b(x: u32) -> u32 { x }"),
    ];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("a").unwrap());
    reg.scan_declared(&ext).unwrap();
    let is_root = |t: &HashMap<TypeKey, TypeCell<()>>, k: &str| {
        t.get(&TypeKey::parse(k).expect("test type"))
            .is_some_and(|c| c.root)
    };
    assert!(is_root(&reg.input_types, "u64"));
    assert!(is_root(&reg.output_types, "u64"));
    assert!(!is_root(&reg.input_types, "u32"));
    assert!(!is_root(&reg.output_types, "u32"));
}

#[test]
fn scan_declared_rejects_function_declared_and_ignored_overlap() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let ident: syn::Ident = syn::parse_str("good").unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(ident.clone());
    ext.ignored_functions.insert(ident.clone());

    match reg.scan_declared(&ext) {
        Err(ScanError::ConflictingFunctionIntent { name }) if name == ident => (),
        other => panic!("expected ConflictingFunctionIntent, got {:?}", other),
    }
}

#[test]
fn scan_declared_rejects_type_declared_and_ignored_overlap() {
    let item: syn::ItemStruct = syn::parse_str("struct Thing { value: u64 }").unwrap();
    let items = vec![(syn::Item::Struct(item), SourceLocation::default())];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let key = TypeKey::parse("Thing").expect("test type");
    let mut ext = StubExt::default();
    ext.types.insert(key.clone());
    ext.ignored_types.insert(key.clone());

    match reg.scan_declared(&ext) {
        Err(ScanError::ConflictingTypeIntent { key: actual }) if actual == key => (),
        other => panic!("expected ConflictingTypeIntent, got {:?}", other),
    }
}

/// A declared function that matches no indexed item is a hard error, not a
/// warning — explicit intent gone wrong (I7).
#[test]
fn scan_declared_missing_function_is_hard_error() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("good").unwrap());
    ext.functions.insert(syn::parse_str("typo_fn").unwrap());
    match reg.scan_declared(&ext) {
        Err(ScanError::DeclaredNotFound { entries }) => {
            assert_eq!(entries, vec![("function", "typo_fn".to_string())]);
        }
        other => panic!("expected DeclaredNotFound, got {:?}", other),
    }
}

/// All missing declared items (fn, helper fn, const) are collected into ONE
/// error, sorted, so a broken build.rs is fixed in a single pass.
#[test]
fn scan_declared_collects_all_missing_kinds_in_one_error() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("typo_fn").unwrap());
    ext.helper_functions
        .insert(syn::parse_str("typo_helper").unwrap());
    ext.consts = Some(HashSet::from([syn::parse_str("TYPO_CONST").unwrap()]));
    match reg.scan_declared(&ext) {
        Err(ScanError::DeclaredNotFound { entries }) => {
            assert_eq!(
                entries,
                vec![
                    ("constant", "TYPO_CONST".to_string()),
                    ("function", "typo_fn".to_string()),
                    ("helper function", "typo_helper".to_string()),
                ]
            );
            // The message lists every entry.
            let msg = ScanError::DeclaredNotFound { entries }.to_string();
            assert!(msg.contains("typo_fn") && msg.contains("TYPO_CONST"));
        }
        other => panic!("expected DeclaredNotFound, got {:?}", other),
    }
}

/// A stale *ignore* entry stays a warning: the scan succeeds.
#[test]
fn scan_declared_missing_ignore_is_not_an_error() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.ignored_functions
        .insert(syn::parse_str("gone_fn").unwrap());
    reg.scan_declared(&ext)
        .expect("stale ignore must only warn");
}

/// An ignore predicate acknowledges matching undeclared items of EVERY
/// kind — fn, struct/enum, const (one flat namespace, so a name filter
/// needs no kind) — and is silent when it matches nothing: a filter, not a
/// claim.
#[test]
fn scan_declared_accepts_ignore_predicates() {
    let s: syn::ItemStruct = syn::parse_str("struct HelperThing { v: u64 }").unwrap();
    let c: syn::ItemConst = syn::parse_str("const HELPER_MAX: u64 = 1;").unwrap();
    let items = vec![
        fn_item("fn helper_a(x: u64) -> u64 { x }"),
        fn_item("fn helper_b(x: u64) -> u64 { x }"),
        (syn::Item::Struct(s), SourceLocation::default()),
        (syn::Item::Const(c), SourceLocation::default()),
    ];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    // Const skip-warnings only run for adapters WITH a const mechanism.
    let mut ext = StubExt {
        consts: Some(HashSet::new()),
        ..StubExt::default()
    };
    ext.ignored_name_predicates
        .push(std::sync::Arc::new(|n: &str| {
            let l = n.to_lowercase();
            l.starts_with("helper")
        }));
    // A second, zero-match predicate is fine too.
    ext.ignored_name_predicates
        .push(std::sync::Arc::new(|n: &str| n.starts_with("nothing_")));
    reg.scan_declared(&ext).expect("predicates must scan clean");
    // Nothing was declared, so nothing became a root.
    assert!(!reg.input_types.values().any(|c| c.root));
}

#[test]
fn type_entry_helpers_expose_converter_chain_contract() {
    let entry = TypeEntry {
        destination: syn::parse_quote!(jni::sys::jlong),
        function: syn::parse_quote!(
            fn __wire(v: Owned) -> jni::sys::jlong {
                0
            }
        ),
        pre_stages: vec![
            Stage {
                function: syn::parse_quote!(
                    fn __stage_rust(v: Rust) -> Result<Mid, Err> {
                        todo!()
                    }
                ),
                metadata: (),
            },
            Stage {
                function: syn::parse_quote!(
                    fn __stage_wire(v: Mid) -> Result<Owned, Err> {
                        todo!()
                    }
                ),
                metadata: (),
            },
        ],
        subs: vec![
            TypeKey::parse("Rust").expect("test type"),
            TypeKey::parse("Mid").expect("test type"),
        ],
        niches: Niches::empty(),
        metadata: (),
    };

    assert_eq!(entry.converter_ident(), "__wire");
    assert_eq!(
        TypeKey::from_type(entry.wire_type()),
        TypeKey::parse("jni::sys::jlong").expect("test type")
    );
    assert_eq!(
        entry
            .output_stage_order()
            .map(|(_, s)| s.function.sig.ident.to_string())
            .collect::<Vec<_>>(),
        vec!["__stage_rust", "__stage_wire"]
    );
    assert_eq!(
        entry
            .input_stage_order()
            .map(|(_, s)| s.function.sig.ident.to_string())
            .collect::<Vec<_>>(),
        vec!["__stage_wire", "__stage_rust"]
    );
    assert_eq!(
        entry
            .dependency_keys()
            .iter()
            .map(TypeKey::as_str)
            .collect::<Vec<_>>(),
        vec!["Rust", "Mid"]
    );
}

/// A name collision across two chained source streams fails registry
/// construction with an error that names BOTH origin crates — the
/// `SourceLocation` file paths are crate-relative (both may read
/// `src/lib.rs`), so the crates (stamped into each stream item's location
/// by `Source`) are the only unambiguous coordinates.
/// Ingestion checks that the flat API is expressible, and reports **every**
/// offender at once — a source crate that needs migrating should see one list,
/// not one rebuild per item.
///
/// This replaces two tests that asserted the opposite (that `from_items` was
/// index-only and diagnosed at declaration time). The frontend owns that
/// judgement now, and its diagnosis is richer: it names the parameter.
#[test]
fn from_items_rejects_what_the_language_cannot_express() {
    let err = match Registry::<()>::from_items(vec![
        fn_item("fn bogus(x: u64) -> impl std::fmt::Debug { 0u64 }"),
        fn_item("fn worse(self) -> u64 { 0 }"),
    ]) {
        Ok(_) => panic!("neither item is expressible"),
        Err(e) => e,
    };

    let ScanError::NotExpressible { entries } = &err else {
        panic!("expected a NotExpressible report, got {err}");
    };
    assert_eq!(entries.len(), 2, "all offenders at once");

    let msg = err.to_string();
    assert!(msg.contains("bogus") && msg.contains("impl Trait"), "{msg}");
    assert!(msg.contains("worse") && msg.contains("self"), "{msg}");
}

#[test]
fn duplicate_name_across_sources_names_both_crates() {
    use crate::{
        api::record::{Record, RecordKind},
        SourceLocation,
    };

    let make_source = |crate_name: &str| -> crate::Source {
        let dir = crate::api::test_util::unique_test_dir(&format!("dup_src_{crate_name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("crate_name.txt"), crate_name).unwrap();
        let record = Record::new(
            RecordKind::Function,
            "same_name".to_string(),
            "pub fn same_name() -> i32 { 1 }".to_string(),
            SourceLocation {
                file: "src/lib.rs".to_string(),
                line: 1,
                column: 1,
                crate_name: None,
            },
            None,
        );
        crate::api::utils::jsonl::write_to_jsonl_file(dir.join("default_1.jsonl"), &[&record])
            .unwrap();
        crate::Source::new(&dir)
    };

    let a = make_source("first-crate");
    let b = make_source("second-crate");
    let msg = match Registry::<()>::from_items(a.items_all().chain(b.items_all())) {
        Ok(_) => panic!("collision must fail"),
        Err(e) => e.to_string(),
    };
    assert!(msg.contains("same_name"), "{msg}");
    assert!(msg.contains("first-crate"), "{msg}");
    assert!(msg.contains("second-crate"), "{msg}");
}

/// Chained streams from two sources feed ONE `from_items` call: per-item
/// origins come from the `SourceLocation` stamps, and the first item's
/// origin becomes the default module.
#[test]
fn from_items_records_origins_from_location_stamps() {
    let loc = |krate: &str| SourceLocation {
        file: "src/lib.rs".to_string(),
        line: 1,
        column: 1,
        crate_name: Some(krate.to_string()),
    };
    let f_a: syn::ItemFn = syn::parse_str("fn from_flat(x: u64) -> u64 { x }").unwrap();
    let f_b: syn::ItemFn = syn::parse_str("fn from_helper(x: u64) -> u64 { x }").unwrap();
    let a = vec![(syn::Item::Fn(f_a), loc("flat-crate"))];
    let b = vec![(syn::Item::Fn(f_b), loc("helper-crate"))];
    let reg: Registry<()> = Registry::from_items(a.into_iter().chain(b)).unwrap();

    let path = |p: syn::Path| p.to_token_stream().to_string();
    assert_eq!(
        reg.origin_module(&syn::parse_str("from_flat").unwrap())
            .map(path),
        Some("flat_crate".to_string())
    );
    assert_eq!(
        reg.origin_module(&syn::parse_str("from_helper").unwrap())
            .map(path),
        Some("helper_crate".to_string())
    );
    // First origin seen = default module; both modules listed in order.
    assert_eq!(
        reg.default_module().map(path),
        Some("flat_crate".to_string())
    );
    assert_eq!(
        reg.all_source_modules()
            .into_iter()
            .map(path)
            .collect::<Vec<_>>(),
        vec!["flat_crate".to_string(), "helper_crate".to_string()]
    );
}

/// N5: `Prebindgen::validate` runs during `resolve` after the scan; an
/// adapter-invariant failure surfaces as `ScanError::AdapterInvariant`
/// with the adapter's message verbatim.
#[test]
fn resolve_surfaces_adapter_invariant_errors() {
    struct FailingExt(StubExt);
    impl Prebindgen for FailingExt {
        type Metadata = ();
        fn validate(&self, _registry: &Registry<()>) -> Result<(), String> {
            Err("member fun `f` has no receiver".to_string())
        }
        fn on_function(&self, f: &syn::ItemFn, r: &Registry<()>) -> TokenStream {
            self.0.on_function(f, r)
        }
        fn on_struct(&self, s: &syn::ItemStruct, r: &Registry<()>) -> TokenStream {
            self.0.on_struct(s, r)
        }
        fn on_enum(&self, e: &syn::ItemEnum, r: &Registry<()>) -> TokenStream {
            self.0.on_enum(e, r)
        }
        fn on_input_type(&self, t: &syn::Type, r: &Registry<()>) -> Option<ConverterImpl<()>> {
            self.0.on_input_type(t, r)
        }
        fn on_output_type(&self, t: &syn::Type, r: &Registry<()>) -> Option<ConverterImpl<()>> {
            self.0.on_output_type(t, r)
        }
    }
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let reg: Registry<()> = Registry::from_items(items).unwrap();
    let err = reg
        .resolve(FailingExt(StubExt::default()))
        .expect_err("validate Err must abort resolve");
    let msg = format!("{err}");
    assert!(msg.contains("member fun `f` has no receiver"), "{msg}");
}

// ── issue #95: semantic type identity ───────────────────────────────────

/// A source-crate-stamped location, the way `Source` stamps parsed records.
fn crate_loc(name: &str) -> SourceLocation {
    SourceLocation {
        crate_name: Some(name.to_string()),
        ..Default::default()
    }
}

#[test]
fn typekey_equivalence_rules() {
    let k = |s: &str| TypeKey::parse(s).expect("test type");
    // Group/paren unwrap + whitespace.
    assert_eq!(k("Foo"), k("(Foo)"));
    assert_eq!(k("Vec<u8>"), k("Vec < u8 >"));
    // `crate::` / `self::` reduce to the bare flat name, at any depth and
    // in nested positions.
    assert_eq!(k("Foo"), k("crate::Foo"));
    assert_eq!(k("Foo"), k("crate::a::b::Foo"));
    assert_eq!(k("Foo"), k("self::Foo"));
    assert_eq!(k("Option<Foo>"), k("Option<crate::a::Foo>"));
    assert_eq!(k("&Foo"), k("&crate::Foo"));
    // The std prelude whitelist.
    assert_eq!(k("Vec<Foo>"), k("std::vec::Vec<crate::Foo>"));
    assert_eq!(k("Option<i32>"), k("core::option::Option<i32>"));
    assert_eq!(k("Result<Foo, Bar>"), k("std::result::Result<Foo, Bar>"));
    assert_eq!(k("String"), k("std::string::String"));
    assert_eq!(k("Box<Foo>"), k("alloc::boxed::Box<Foo>"));
    // Distinctness: unknown crate heads and non-whitelisted std paths keep
    // their spelling; lifetimes are structure, not spelling.
    assert_ne!(k("a::Foo"), k("b::Foo"));
    assert_ne!(k("a::Foo"), k("Foo"));
    assert_ne!(k("std::ffi::CString"), k("CString"));
    assert_ne!(k("&Foo"), k("&'a Foo"));
    assert_ne!(k("Foo<'static>"), k("Foo"));
    // Idempotence: re-keying a key's own type or string is the identity.
    let once = k("std::vec::Vec<crate::m::Foo>");
    assert_eq!(once, TypeKey::from_type(&once.to_type()));
    assert_eq!(once, k(once.as_str()));
    assert_eq!(once.as_str(), "Vec < Foo >");
}

#[test]
fn typekey_parse_returns_structured_error() {
    let err = TypeKey::parse("not a type !!").expect_err("must fail");
    assert_eq!(err.input, "not a type !!");
    assert!(err.to_string().contains("invalid type"), "{err}");
}

#[test]
fn qualified_signature_matches_bare_declaration() {
    // A captured signature may spell an indexed item with the source
    // crate's own name or `crate::`; ingest normalizes both to the bare
    // flat spelling, so bare-declared types and bare sub-positions match.
    let f: syn::ItemFn =
        syn::parse_str("fn get(x: &myflat::Thing) -> std::vec::Vec<crate::Thing> { todo!() }")
            .unwrap();
    let s: syn::ItemStruct = syn::parse_str("pub struct Thing { pub v: u64 }").unwrap();
    let items = vec![
        (syn::Item::Struct(s), crate_loc("myflat")),
        (syn::Item::Fn(f), crate_loc("myflat")),
    ];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("get").unwrap());
    ext.types
        .insert(TypeKey::parse("Thing").expect("test type"));
    reg.scan_declared(&ext).unwrap();
    assert!(reg.input_types[&TypeKey::parse("&Thing").expect("test type")].root);
    assert!(reg.output_types[&TypeKey::parse("Vec<Thing>").expect("test type")].root);
    // No spelling-variant duplicate cells survive anywhere.
    let no_paths =
        |t: &HashMap<TypeKey, TypeCell<()>>| !t.keys().any(|k| k.as_str().contains("::"));
    assert!(no_paths(&reg.input_types));
    assert!(no_paths(&reg.output_types));
}

#[test]
fn multi_source_rename_cross_reference_normalizes() {
    // Source B (a renamed dependency: crate `cov-helpers` = module
    // `cov_helpers`) references source A's type by A's crate name. B's
    // items are chained FIRST, so this also proves pass 1 gathers every
    // module name before pass 2 normalizes (chain-order independence).
    let b_fn: syn::ItemFn =
        syn::parse_str("fn use_a(x: &srca::TypeA) -> cov_helpers::TypeB { todo!() }").unwrap();
    let b_ty: syn::ItemStruct = syn::parse_str("pub struct TypeB { pub v: u64 }").unwrap();
    let a_ty: syn::ItemStruct = syn::parse_str("pub struct TypeA { pub v: u64 }").unwrap();
    let items = vec![
        (syn::Item::Fn(b_fn), crate_loc("cov-helpers")),
        (syn::Item::Struct(b_ty), crate_loc("cov-helpers")),
        (syn::Item::Struct(a_ty), crate_loc("srca")),
    ];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("use_a").unwrap());
    ext.types
        .insert(TypeKey::parse("TypeA").expect("test type"));
    ext.types
        .insert(TypeKey::parse("TypeB").expect("test type"));
    reg.scan_declared(&ext).unwrap();
    assert!(reg.input_types[&TypeKey::parse("&TypeA").expect("test type")].root);
    assert!(reg.output_types[&TypeKey::parse("TypeB").expect("test type")].root);
}

#[test]
fn qualified_declared_type_is_hard_error() {
    // `ptr_class!(myflat::Thing)`-shaped declaration: the head names a
    // chained source crate, so the key can never match the flat namespace —
    // a collected hard error with the bare fix-it, not a silent miss.
    let s: syn::ItemStruct = syn::parse_str("pub struct Thing { pub v: u64 }").unwrap();
    let items = vec![(syn::Item::Struct(s), crate_loc("myflat"))];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    ext.types
        .insert(TypeKey::parse("myflat::Thing").expect("test type"));
    match reg.scan_declared(&ext) {
        Err(ScanError::QualifiedDeclaredTypes { entries }) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "myflat :: Thing");
            assert_eq!(entries[0].1, "Thing");
            let msg = ScanError::QualifiedDeclaredTypes { entries }.to_string();
            assert!(msg.contains("declare it as `Thing`"), "{msg}");
        }
        other => panic!("expected QualifiedDeclaredTypes, got {:?}", other),
    }
}

#[test]
fn foreign_qualified_declared_type_stays_supported() {
    // `ptr_class!(zenoh::KeyExpr<'static>)`-style: the head is NOT a source
    // module, so the declaration passes through verbatim and is marked
    // required under its own spelling (the no-indexed-body arm).
    let items = vec![fn_item("fn touch(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let mut ext = StubExt::default();
    let foreign = TypeKey::parse("zenoh::KeyExpr<'static>").expect("test type");
    ext.types.insert(foreign.clone());
    reg.scan_declared(&ext)
        .expect("foreign qualified declaration is supported");
    assert!(reg.input_types[&foreign].root);
    assert!(reg.output_types[&foreign].root);
}

// ── The directory-reading builder ──────────────────────────────────────

/// Write a real prebindgen output directory holding one marked fn, stamped with
/// `crate_name` at capture time — the same fixture shape
/// `duplicate_name_across_sources_names_both_crates` builds.
fn write_source_dir(tag: &str, crate_name: &str, fn_name: &str) -> std::path::PathBuf {
    use crate::{
        api::record::{Record, RecordKind},
        SourceLocation,
    };

    let dir = crate::api::test_util::unique_test_dir(&format!("builder_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("crate_name.txt"), crate_name).unwrap();
    let record = Record::new(
        RecordKind::Function,
        fn_name.to_string(),
        format!("pub fn {fn_name}() -> i32 {{ 1 }}"),
        SourceLocation {
            file: "src/lib.rs".to_string(),
            line: 1,
            column: 1,
            crate_name: None,
        },
        None,
    );
    crate::api::utils::jsonl::write_to_jsonl_file(dir.join("default_1.jsonl"), &[&record]).unwrap();
    dir
}

/// Render a module path the way the other origin tests in this file do.
fn module(p: syn::Path) -> String {
    p.to_token_stream().to_string()
}

fn fn_ident(name: &str) -> syn::Ident {
    syn::parse_str(name).unwrap()
}

/// A build script names a directory and gets a registry — no `Source` in between.
/// The captured crate becomes the default module, exactly as it does through a
/// hand-built stream.
#[test]
fn builder_reads_a_source_directory() {
    let dir = write_source_dir("plain", "flat-crate", "marked_fn");
    let registry: Registry<()> = Registry::builder().source(&dir).build().expect("indexes");

    assert!(registry
        .flat()
        .function(&fn_ident("marked_fn").to_string())
        .is_some());
    // Dashes normalize to underscores, as they must to be a Rust module path.
    assert_eq!(
        registry.default_module().map(module),
        Some("flat_crate".to_string())
    );
    assert_eq!(
        registry.origin_module(&fn_ident("marked_fn")).map(module),
        Some("flat_crate".to_string())
    );
}

/// `source_named` overrides the capture-time stamp, which is what a dependency
/// renamed in `Cargo.toml` needs: the recorded package name would not resolve
/// from the crate that refers to it by another name.
#[test]
fn builder_source_named_overrides_the_captured_crate() {
    let dir = write_source_dir("renamed", "real-package-name", "helper_fn");

    // Without the override, the registry believes the package name.
    let plain: Registry<()> = Registry::builder().source(&dir).build().expect("indexes");
    assert_eq!(
        plain.origin_module(&fn_ident("helper_fn")).map(module),
        Some("real_package_name".to_string())
    );

    let renamed: Registry<()> = Registry::builder()
        .source_named(&dir, "as_renamed")
        .build()
        .expect("indexes");
    assert_eq!(
        renamed.origin_module(&fn_ident("helper_fn")).map(module),
        Some("as_renamed".to_string())
    );
    assert_eq!(
        renamed.default_module().map(module),
        Some("as_renamed".to_string())
    );
}

/// Feeders accumulate, and the override stays **per directory** — a
/// registry-level one could only fix a single module, which is the whole reason
/// it lives on the source.
#[test]
fn builder_composes_directories_and_streams() {
    let flat = write_source_dir("multi_flat", "flat-crate", "flat_fn");
    let helper = write_source_dir("multi_helper", "real-helper-name", "helper_fn");

    let registry: Registry<()> = Registry::builder()
        .source(&flat)
        .source_named(&helper, "renamed_helper")
        .items(vec![(
            syn::Item::Fn(syn::parse_quote!(
                pub fn synthetic() -> i32 {
                    2
                }
            )),
            crate::SourceLocation::default(),
        )])
        .build()
        .expect("indexes");

    for name in ["flat_fn", "helper_fn", "synthetic"] {
        assert!(
            registry
                .flat()
                .function(&fn_ident(name).to_string())
                .is_some(),
            "{name}"
        );
    }
    // Each directory keeps its own origin; the stream item has none to keep.
    assert_eq!(
        registry.origin_module(&fn_ident("flat_fn")).map(module),
        Some("flat_crate".to_string())
    );
    assert_eq!(
        registry.origin_module(&fn_ident("helper_fn")).map(module),
        Some("renamed_helper".to_string())
    );
    assert_eq!(
        registry.origin_module(&fn_ident("synthetic")).map(module),
        None
    );

    // First-seen order, which is what makes the first entry the default module.
    assert_eq!(
        registry
            .all_source_modules()
            .into_iter()
            .map(module)
            .collect::<Vec<_>>(),
        vec!["flat_crate".to_string(), "renamed_helper".to_string()]
    );
}

/// The builder is sugar: it reaches the same indexing as the primitive, so the
/// whole-stream rules cannot differ between the two entry points.
#[test]
fn builder_and_from_items_agree() {
    let dir = write_source_dir("agree", "flat-crate", "marked_fn");

    let built: Registry<()> = Registry::builder().source(&dir).build().expect("indexes");
    let streamed: Registry<()> =
        Registry::from_items(crate::Source::new(&dir).items_all()).expect("indexes");

    assert_eq!(
        built.flat().functions().count(),
        streamed.flat().functions().count()
    );
    assert_eq!(
        built.default_module().map(module),
        streamed.default_module().map(module)
    );
    assert_eq!(
        built.flat().guards().count(),
        streamed.flat().guards().count()
    );
}

// ── What a table cell knows about its type ─────────────────────────────

/// A cell for a type the source wrote carries the **frontend's own** reading:
/// the same classification the element holds, and the item's location. Not a
/// re-derivation — the registry looks the model up rather than lowering twice.
#[test]
fn a_source_type_cell_carries_the_models_typeref() {
    use crate::api::core::flat::TypeKind;

    let loc = SourceLocation {
        file: "src/lib.rs".into(),
        line: 42,
        column: 7,
        crate_name: Some("myflat".into()),
    };
    let item: syn::Item = syn::parse_str("pub fn f(v: Option<u64>) -> u64 { v.unwrap() }").unwrap();
    let mut reg: Registry<()> = Registry::from_items([(item, loc.clone())]).unwrap();

    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("f").unwrap());
    reg.scan_declared(&ext).unwrap();

    let key = TypeKey::parse("Option<u64>").expect("test type");
    let cell = &reg.input_types[&key];
    assert!(cell.root, "a top-level parameter is a root");
    assert!(
        matches!(cell.subject.kind(), Some(TypeKind::Optional(_))),
        "the frontend classified it, so the cell has that classification"
    );
    // One location per cell, and it is the model's — not a copy the scan made.
    assert_eq!(cell.subject.location(), Some(&loc));

    // The nested position is in the model too, and is not a root.
    let inner = &reg.input_types[&TypeKey::parse("u64").expect("test type")];
    assert!(!inner.root);
    assert!(matches!(inner.subject.kind(), Some(TypeKind::Scalar(_))));
}

/// A type only the binding authored has **no** reading and no location — a fact
/// about it, not information that went missing. Declaring a type the source
/// never mentions is the ordinary way to reach this state.
#[test]
fn an_adapter_authored_type_cell_has_no_source_reading() {
    let items = vec![fn_item("fn f(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();

    let mut ext = StubExt::default();
    ext.types
        .insert(TypeKey::parse("Foreign").expect("test type"));
    reg.scan_declared(&ext).unwrap();

    let cell = &reg.input_types[&TypeKey::parse("Foreign").expect("test type")];
    assert!(cell.root, "the binding asked for it directly");
    assert!(matches!(cell.subject, TypeSubject::Adapter(_)));
    assert!(cell.subject.kind().is_none());
    assert_eq!(cell.subject.location(), None);
}

// ── The projection itself ──────────────────────────────────────────────

/// Every element kind lands where the projection says, the model is **kept**,
/// and a name's origin crate survives the trip.
///
/// The seam's only direct test: everything else reaches `from_flat` through
/// `from_items`, which cannot distinguish "the projection is right" from "the
/// parser and the projection are wrong in matching ways".
#[test]
fn from_flat_projects_each_element_kind() {
    let at = |krate: &str| SourceLocation {
        file: "src/lib.rs".into(),
        crate_name: Some(krate.to_string()),
        ..SourceLocation::default()
    };
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                pub fn f(v: u64) -> u64 {
                    v
                }
            ),
            at("myflat"),
        ),
        (
            syn::parse_quote!(
                pub struct S {
                    pub a: u64,
                }
            ),
            at("myflat"),
        ),
        // A sum and a C-style enum are different elements, one map.
        (
            syn::parse_quote!(
                pub enum Sum {
                    A(u64),
                    B,
                }
            ),
            at("myflat"),
        ),
        (
            syn::parse_quote!(
                pub enum Flags {
                    X = 1,
                    Y = 2,
                }
            ),
            at("myflat"),
        ),
        (
            syn::parse_quote!(
                pub const K: u64 = 7;
            ),
            at("myflat"),
        ),
        // Each source's injected feature guard: no address, so several coexist.
        (
            syn::parse_quote!(
                const _: () = ();
            ),
            at("myflat"),
        ),
        (
            syn::parse_quote!(
                const _: () = ();
            ),
            at("helpers"),
        ),
        // An alias declared by a SECONDARY source. It lands in no map — an
        // `Extern` states a name exists, which the registry has never had a
        // place for — but it DOES record an origin, so a reference to it
        // qualifies against the crate that declared it rather than falling back
        // to the default module.
        (
            syn::parse_quote!(
                pub type Handle = helpers::Inner;
            ),
            at("helpers"),
        ),
    ];
    let flat = crate::api::core::flat::Flat::builder()
        .items(items)
        .build()
        .expect("parse");
    let reg: Registry<()> = Registry::from_flat(flat).expect("project");

    let id = |n: &str| syn::parse_str::<syn::Ident>(n).unwrap();
    assert!(reg.flat().function(&id("f").to_string()).is_some());
    assert!(reg.flat().struct_type(&id("S").to_string()).is_some());
    assert!(
        reg.flat().enum_item(&id("Sum").to_string()).is_some(),
        "a sum is an enum here"
    );
    assert!(reg.flat().enum_item(&id("Flags").to_string()).is_some());
    assert!(reg.flat().constant(&id("K").to_string()).is_some());
    assert_eq!(
        reg.flat().guards().count(),
        2,
        "both anonymous consts, in stream order"
    );
    assert!(
        reg.flat().struct_type(&id("Handle").to_string()).is_none()
            && reg.flat().enum_item(&id("Handle").to_string()).is_none(),
        "an Extern names a type; it declares no body to index"
    );

    // The model is held, not discarded — this is what makes the registry a
    // projection rather than a second reading.
    assert!(reg.flat().element("f").is_some());
    assert!(
        reg.flat().declared_type("Handle").is_some(),
        "the alias is reachable through the model even though no map holds it"
    );

    // Origins, including the alias's — a behaviour change from the old
    // `syn::Item::Type` no-op, which recorded none.
    assert_eq!(reg.origin_module(&id("f")), Some(syn::parse_quote!(myflat)));
    assert_eq!(
        reg.origin_module(&id("Handle")),
        Some(syn::parse_quote!(helpers)),
        "an alias declared by a helper crate qualifies against that crate"
    );
    // First-seen source order, which is what makes the first entry the default.
    assert_eq!(reg.default_module(), Some(syn::parse_quote!(myflat)));
}

/// The inexpressible report names the **crate**, not just the location.
///
/// A captured path is crate-relative, so two offenders from different sources
/// both read `src/lib.rs:0:0` and the location alone cannot say which crate to
/// fix. Same reason the duplicate-name diagnostic carries it.
#[test]
fn not_expressible_report_names_the_crate_of_each_offender() {
    let at = |krate: &str| SourceLocation {
        file: "src/lib.rs".into(),
        crate_name: Some(krate.to_string()),
        ..SourceLocation::default()
    };
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                pub async fn a() {}
            ),
            at("myflat"),
        ),
        (
            syn::parse_quote!(
                pub fn b<T>(v: T) -> T {
                    v
                }
            ),
            at("helpers"),
        ),
    ];
    let Err(err) = Registry::<()>::from_items(items) else {
        panic!("both items are inexpressible")
    };
    let msg = err.to_string();

    assert!(msg.contains("2 `#[prebindgen]` item(s)"), "{msg}");
    assert!(msg.contains("in crate `myflat`"), "{msg}");
    assert!(msg.contains("in crate `helpers`"), "{msg}");
    // Both share a file path, so the crate is the only thing telling them apart.
    assert_eq!(msg.matches("src/lib.rs").count(), 2, "{msg}");
    // No trailing newline: the message is embedded in `expect`/`panic` output.
    assert!(!msg.ends_with('\n'), "{msg:?}");
}

/// A binding-local fn is checked against the **same grammar** as a captured one.
///
/// `sig!(..)` is written by hand in a build script and inserted straight into the
/// registry, so it is the one input that never passes through `Flat`. Without a
/// check here a `self` receiver or a pattern parameter is silently dropped and
/// the user meets it as an arity mismatch out of rustc on generated code — the
/// wrong end of the pipeline to learn about a build.rs typo.
#[test]
fn a_binding_local_fn_is_checked_against_the_grammar() {
    for (src, expected) in [
        ("fn takes_self(&self, x: u32) -> u32 { x }", "receiver"),
        (
            "fn takes_pattern((a, b): (u32, u32)) -> u32 { a }",
            "pattern",
        ),
        ("fn takes_impl(x: impl std::fmt::Debug) {}", "impl Trait"),
        ("async fn is_async() {}", "async"),
    ] {
        let reg: Registry<()> =
            Registry::from_items(vec![fn_item("fn good(x: u64) -> u64 { x }")]).unwrap();
        let ext = StubExt {
            local_fns: vec![(syn::parse_str(src).expect("parse local fn"), "b".into())],
            ..Default::default()
        };
        let err = reg
            .resolve(ext)
            .expect_err(&format!("`{src}` must be refused"));
        let msg = err.to_string();
        assert!(
            msg.contains("binding-local fn"),
            "must say which input is at fault: {msg}"
        );
        assert!(
            msg.to_lowercase().contains(&expected.to_lowercase()),
            "`{src}` should be diagnosed as {expected}, got: {msg}"
        );
    }
}

/// The same check accepts what the grammar allows, so it is a grammar check and
/// not a blanket refusal — and it does **not** demand that a local fn's types be
/// declared, which a binding-local fn legitimately may not be.
#[test]
fn a_well_formed_binding_local_fn_passes() {
    let reg: Registry<()> =
        Registry::from_items(vec![fn_item("fn good(x: u64) -> u64 { x }")]).unwrap();
    let ext = StubExt {
        local_fns: vec![(
            syn::parse_str("fn helper(s: &Undeclared) -> u64 { 0 }").expect("parse"),
            "b".into(),
        )],
        ..Default::default()
    };
    reg.resolve(ext)
        .expect("a grammatical local fn passes, undeclared types and all");
}

/// A guard is not a const, structurally — so nothing that consumes the const
/// surface has to remember it exists.
///
/// The three `c.ident == "_"` sentinel checks this replaced had gone **dead**
/// without anyone noticing: once ingestion routed unnamed consts away from
/// `consts`, they guarded a state the pipeline could no longer produce. This is
/// the assertion that would have caught that, and that keeps a future
/// reclassification honest.
#[test]
fn a_guard_never_reaches_the_const_surface() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                const _: () = ();
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const REAL: u64 = 7;
            ),
            loc.clone(),
        ),
        // A second source's guard: several coexist, having no address to collide on.
        (
            syn::parse_quote!(
                const _: () = ();
            ),
            loc.clone(),
        ),
    ];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();

    assert_eq!(reg.flat().guards().count(), 2);
    assert_eq!(reg.flat().constants().count(), 1);
    assert!(reg.flat().constant("REAL").is_some());

    // An adapter WITH a const mechanism that declares nothing warns about `REAL`
    // only: a guard is not undeclared API, it is not API.
    let ext = StubExt {
        consts: Some(HashSet::new()),
        ..Default::default()
    };
    reg.scan_declared(&ext).expect("guards are not declarable");
}

// ── One index: what the deleted maps used to guarantee ─────────────────

/// `named_item_idents` must **not** name an alias.
///
/// It used to derive from the four maps, and an `Extern` was in none of them.
/// Derived from the model it would include alias names unless filtered — and its
/// caller uses it to decide which names generated Rust qualifies, so including
/// one would move generated output. This is the assertion that keeps the filter.
#[test]
fn named_item_idents_omits_aliases() {
    let reg: Registry<()> = crate::api::test_util::reg_with(&[
        "pub fn f(x: u64) -> u64 { x }",
        "pub struct S { pub a: u64 }",
        "pub enum E { A }",
        "pub const K: u64 = 7;",
        "pub type Handle = other::Inner;",
    ]);
    let names: HashSet<String> = reg.named_item_idents().map(|i| i.to_string()).collect();
    assert_eq!(
        names,
        ["f", "S", "E", "K"].map(String::from).into_iter().collect(),
        "an alias names a type but declares no body; it must not be qualified"
    );
}

/// A binding-local fn joins the one index, carries its adapter-supplied origin
/// crate — and does **not** join the source-module list.
///
/// The last part is the subtle one: `source_modules` decides `default_module`,
/// which is what an unqualified reference resolves against. If a fn a build
/// script invented could extend it, adding one would silently change how
/// captured items are qualified.
#[test]
fn a_binding_local_fn_joins_the_index_but_not_the_source_modules() {
    let at = SourceLocation {
        crate_name: Some("myflat".into()),
        ..SourceLocation::default()
    };
    let reg: Registry<()> = Registry::from_items(vec![(
        syn::parse_quote!(
            pub fn captured(x: u64) -> u64 {
                x
            }
        ),
        at,
    )])
    .unwrap();
    let before_default = reg.default_module();
    let before_all = reg.all_source_modules();

    let ext = StubExt {
        local_fns: vec![(
            syn::parse_str("fn helper(v: u64) -> u64 { v }").unwrap(),
            "my-helpers".into(),
        )],
        ..Default::default()
    };
    let gen = reg.resolve(ext).expect("resolve");
    let reg = gen.registry();

    // In the one index, reachable exactly like a captured fn.
    assert!(reg.flat().function("helper").is_some());
    // With its own origin crate, which is what qualifies its generated call.
    assert_eq!(
        reg.origin_module(&syn::parse_str::<syn::Ident>("helper").unwrap()),
        Some(syn::parse_quote!(my_helpers))
    );
    // But the module list is untouched.
    assert_eq!(reg.default_module(), before_default);
    assert_eq!(reg.all_source_modules(), before_all);
    assert!(!reg
        .flat()
        .source_modules()
        .contains(&"my_helpers".to_string()));
}

/// Both enum shapes answer to `enum_item` — the merge the old `enums` map made,
/// which 30 adapter reads depend on.
#[test]
fn enum_item_answers_for_both_shapes() {
    let reg: Registry<()> = crate::api::test_util::reg_with(&[
        "pub enum Sum { A(u64), B }",
        "pub enum Flags { X = 1, Y = 2 }",
        "pub struct S { pub a: u64 }",
    ]);
    assert!(reg.flat().enum_item("Sum").is_some(), "a sum");
    assert!(reg.flat().enum_item("Flags").is_some(), "a C-style enum");
    assert!(reg.flat().enum_item("S").is_none(), "not a struct");
    assert!(reg.flat().struct_type("S").is_some());
    assert!(reg.flat().struct_type("Sum").is_none());
}

// ── An alias is a declaration of its name ──────────────────────────────

/// The predicate both type diagnostics gate on counts **every** declared type,
/// alias included.
///
/// An alias was excluded because the pre-`Flat` code asked the `structs`/`enums`
/// maps, which never held one. That was an artefact of where the answer came
/// from, not a decision: `#[prebindgen] pub type Handle = ..` declares the name
/// `Handle`, an adapter may declare it bare, and a diagnostic that says "no such
/// captured item" about it is simply false.
#[test]
fn every_declared_type_counts_including_an_alias() {
    let reg: Registry<()> = crate::api::test_util::reg_with(&[
        "pub struct S { pub a: u64 }",
        "pub enum Sum { A(u64), B }",
        "pub enum Flags { X = 1 }",
        "pub type Handle = other::Inner;",
        "pub fn f(x: u64) -> u64 { x }",
        "pub const K: u64 = 7;",
    ]);
    let id = |n: &str| syn::parse_str::<syn::Ident>(n).unwrap();

    for name in ["S", "Sum", "Flags", "Handle"] {
        assert!(
            reg.declares_type(&id(name)),
            "`{name}` is a declared type and must count"
        );
    }
    // Not types: a fn and a const share the flat namespace but declare no type.
    for name in ["f", "K", "Absent"] {
        assert!(!reg.declares_type(&id(name)), "`{name}` declares no type");
    }

    // The sibling that must NOT change: it feeds a "skipping undeclared
    // struct/enum" warning, so an alias — which is neither — stays out.
    let bodies: HashSet<String> = reg.struct_enum_idents().map(|i| i.to_string()).collect();
    assert_eq!(
        bodies,
        ["S", "Sum", "Flags"]
            .map(String::from)
            .into_iter()
            .collect(),
        "struct_enum_idents feeds a struct/enum message and must exclude aliases"
    );
}

/// Both diagnostic sites reach the predicate for an alias, and neither errors.
///
/// `scan_declared` is the entry point for both: a path-qualified declared type
/// whose tail names an alias (the "did you mean the bare name?" heuristic) and
/// an ignored type that names one (the "not found among #[prebindgen] items"
/// check). The messages themselves are `cargo:warning=` on stdout and are not
/// captured here — what this pins is that an alias flows through the same path a
/// struct does, without the `QualifiedDeclaredTypes` hard error.
#[test]
fn an_alias_flows_through_both_type_diagnostics() {
    let build = |declare_qualified: bool| {
        let reg: Registry<()> = crate::api::test_util::reg_with(&[
            "pub type Handle = other::Inner;",
            "pub fn f(x: u64) -> u64 { x }",
        ]);
        let mut ext = StubExt::default();
        if declare_qualified {
            // Head is NOT a source module, so this is the warn-and-pass-through
            // branch rather than the hard error.
            ext.types
                .insert(TypeKey::parse("foreign::Handle").expect("test type"));
        } else {
            ext.ignored_types
                .insert(TypeKey::parse("Handle").expect("test type"));
        }
        (reg, ext)
    };

    for qualified in [true, false] {
        let (mut reg, ext) = build(qualified);
        reg.scan_declared(&ext)
            .expect("an alias is a captured item; neither site may fail");
    }
}

/// A type only a **binding-local** fn writes still has a frontend reading, and
/// its cell must say so.
///
/// The ordering that made this wrong: the type index used to be built in
/// `from_flat`, while local fns are inserted later by `resolve`. So their
/// parameter types missed the index and their cells came out `Adapter` — "no
/// reading" — even though `lower_signature` had produced `TypeRef`s for them.
/// `Flat` owns the index now and `add_local_function` feeds it.
#[test]
fn a_type_only_a_local_fn_writes_still_has_a_reading() {
    use crate::api::core::flat::TypeKind;

    /// Resolves anything to itself, so declaring the local fn does not also
    /// require an adapter that can convert its types.
    struct AnyConverterExt(StubExt);
    impl AnyConverterExt {
        fn converter(ty: &syn::Type) -> Option<ConverterImpl<()>> {
            Some(ConverterImpl {
                destination: ty.clone(),
                function: syn::parse_quote!(
                    fn __id() {}
                ),
                pre_stages: vec![],
                subs: vec![],
                niches: Niches::empty(),
                metadata: (),
            })
        }
    }
    impl Prebindgen for AnyConverterExt {
        type Metadata = ();
        fn declared_functions(&self) -> HashSet<syn::Ident> {
            self.0.declared_functions()
        }
        fn local_functions(&self) -> Vec<(syn::ItemFn, String)> {
            self.0.local_functions()
        }
        fn on_function(&self, f: &syn::ItemFn, r: &Registry<()>) -> TokenStream {
            self.0.on_function(f, r)
        }
        fn on_struct(&self, st: &syn::ItemStruct, r: &Registry<()>) -> TokenStream {
            self.0.on_struct(st, r)
        }
        fn on_enum(&self, e: &syn::ItemEnum, r: &Registry<()>) -> TokenStream {
            self.0.on_enum(e, r)
        }
        fn on_input_type(&self, t: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
            Self::converter(t)
        }
        fn on_output_type(&self, t: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
            Self::converter(t)
        }
    }

    // `Option<u64>` appears nowhere in the captured stream.
    let reg: Registry<()> =
        Registry::from_items(vec![fn_item("fn captured(x: u64) -> u64 { x }")]).unwrap();
    assert!(
        reg.flat()
            .type_ref(&syn::parse_quote!(Option<u64>))
            .is_none(),
        "fixture precondition: the captured stream never writes this type"
    );

    let ext = AnyConverterExt(StubExt {
        local_fns: vec![(
            syn::parse_str("fn helper(v: Option<u64>) -> u64 { 0 }").unwrap(),
            "helpers".into(),
        )],
        functions: ["helper"]
            .iter()
            .map(|s| syn::parse_str(s).unwrap())
            .collect(),
        ..Default::default()
    });
    let gen = reg.resolve(ext).expect("resolve");
    let reg = gen.registry();

    // The model now holds the reading …
    let read = reg
        .flat()
        .type_ref(&syn::parse_quote!(Option<u64>))
        .expect("a local fn's parameter type is in the model");
    assert!(matches!(read.kind, TypeKind::Optional(_)));

    // … and the cell scanned from that parameter carries it, rather than
    // claiming the type is one the binding invented.
    let cell = &reg.input_types[&TypeKey::parse("Option<u64>").expect("test type")];
    assert!(
        matches!(cell.subject, TypeSubject::Source(_)),
        "the frontend read this type; the cell must not call it adapter-authored"
    );
    assert!(matches!(cell.subject.kind(), Some(TypeKind::Optional(_))));
}

/// A type with no source position must not get an invented one.
///
/// Three facts have to stay apart: a type can have a **frontend reading**
/// (`TypeSubject::Source`), a **reportable position**, or neither. A
/// binding-local fn's parameter types have the first and not the second —
/// `lower_signature` lowers them against `SourceLocation::default()`, since
/// `Origin` needs a location and a `sig!(..)` has no file.
///
/// Indexing those types (this PR) flipped their cells from `Adapter` to
/// `Source`, and `location()` returned the default unconditionally, so the
/// diagnostic read `:0:0: error:` — a position that looks real. The same fault
/// already showed for any hand-built stream, whose captured items also carry
/// default locations; both are fixed by asking whether the location has a
/// position at all.
#[test]
fn an_unresolved_type_without_a_position_reports_none() {
    let reg: Registry<()> =
        Registry::from_items(vec![fn_item("fn captured(x: u64) -> u64 { x }")]).unwrap();
    let ext = StubExt {
        local_fns: vec![(
            syn::parse_str("fn helper(v: Option<u64>) -> u64 { 0 }").unwrap(),
            "helpers".into(),
        )],
        functions: ["helper"]
            .iter()
            .map(|s| syn::parse_str(s).unwrap())
            .collect(),
        ..Default::default()
    };
    // `StubExt` supplies no converters, so every scanned type is unresolved:
    // `Option<u64>` reached only through the local fn, `u64` through both.
    let err = reg.resolve(ext).expect_err("StubExt resolves nothing");
    let msg = err.to_string();

    assert!(
        !msg.contains(":0:0:"),
        "no file and no line means no position to print:\n{msg}"
    );
    assert!(
        msg.contains("error: unresolved prebindgen input type `Option < u64 >`"),
        "the local-only type is still reported, just without a position:\n{msg}"
    );

    // A captured item that DOES have a position still reports it.
    let located: Registry<()> = Registry::from_items(vec![(
        syn::parse_quote!(
            pub fn f(x: u64) -> u64 {
                x
            }
        ),
        SourceLocation {
            file: "src/lib.rs".into(),
            line: 12,
            column: 3,
            crate_name: Some("myflat".into()),
        },
    )])
    .unwrap();
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("f").unwrap());
    let err = located.resolve(ext).expect_err("StubExt resolves nothing");
    assert!(
        err.to_string().contains("src/lib.rs:12:3: error:"),
        "a real position must still be reported:\n{}",
        err
    );
}
