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
fn from_items_does_not_scan_signatures() {
    // A `#[prebindgen]`-marked fn whose return is a bare `impl Foo`
    // would have failed `from_items` under the old code path
    // (ScanError::DisallowedImplTrait). Now `from_items` is index-
    // only and accepts it without complaint.
    let items = vec![fn_item("fn bogus(x: u64) -> impl std::fmt::Debug { 0u64 }")];
    let reg: Registry<()> = Registry::from_items(items).expect("from_items must succeed");
    assert!(reg.required_inputs_scan.is_empty());
    assert!(reg.required_outputs_scan.is_empty());
    // The fn is indexed but no types are pre-required.
    assert!(reg
        .functions
        .contains_key(&syn::parse_str("bogus").unwrap()));
}

#[test]
fn scan_declared_empty_ext_marks_nothing_required() {
    let items = vec![fn_item("fn good(x: u64) -> u64 { x }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();
    let ext = StubExt::default();
    reg.scan_declared(&ext).expect("empty ext = no scan");
    assert!(reg.required_inputs_scan.is_empty());
    assert!(reg.required_outputs_scan.is_empty());
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
    assert!(reg
        .required_inputs_scan
        .contains(&TypeKey::parse("u64").expect("test type")));
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::parse("u64").expect("test type")));
    assert!(!reg
        .required_inputs_scan
        .contains(&TypeKey::parse("u32").expect("test type")));
    assert!(!reg
        .required_outputs_scan
        .contains(&TypeKey::parse("u32").expect("test type")));
}

#[test]
fn scan_declared_fails_disallowed_impl_trait_only_when_fn_declared() {
    let items = vec![fn_item("fn bogus(x: u64) -> impl std::fmt::Debug { 0u64 }")];
    let mut reg: Registry<()> = Registry::from_items(items).unwrap();

    // Empty ext: the bogus fn is not scanned, so no error.
    let empty = StubExt::default();
    assert!(reg.scan_declared(&empty).is_ok());

    // Declare the fn: scan now fires the disallowed-impl-Trait error.
    let mut ext = StubExt::default();
    ext.functions.insert(syn::parse_str("bogus").unwrap());
    match reg.scan_declared(&ext) {
        Err(ScanError::DisallowedImplTrait { .. }) => (),
        other => panic!("expected DisallowedImplTrait, got {:?}", other),
    }
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
    // Nothing was declared, so nothing became required.
    assert!(reg.required_inputs_scan.is_empty());
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
        required: true,
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
    assert!(reg
        .required_inputs_scan
        .contains(&TypeKey::parse("&Thing").expect("test type")));
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::parse("Vec<Thing>").expect("test type")));
    // No spelling-variant duplicate cells survive anywhere.
    let no_paths = |set: &HashSet<TypeKey>| !set.iter().any(|k| k.as_str().contains("::"));
    assert!(no_paths(&reg.required_inputs_scan));
    assert!(no_paths(&reg.required_outputs_scan));
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
    assert!(reg
        .required_inputs_scan
        .contains(&TypeKey::parse("&TypeA").expect("test type")));
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::parse("TypeB").expect("test type")));
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
    assert!(reg.required_inputs_scan.contains(&foreign));
    assert!(reg.required_outputs_scan.contains(&foreign));
}
