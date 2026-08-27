use std::{
    cell::Cell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use prebindgen::SourceLocation;

use super::*;
use crate::registry::RegistryBuilder;

/// Freeze a test assembly from artifacts stated in emission order.
fn assembly_of<A: RustArtifact, I: IntoIterator<Item = A>>(artifacts: I) -> Assembly<A> {
    artifacts.into_iter().collect()
}

struct IdentityExt;

impl IdentityExt {
    fn declare_into(&self, mut reg: RegistryBuilder) -> RegistryBuilder {
        for f in [syn::parse_quote!(a_fn), syn::parse_quote!(b_fn)] {
            reg = reg.export(&f);
        }
        for t in ["AEnum", "AStruct", "BEnum", "BStruct"] {
            reg = reg.export_type(crate::test_util::declared_origin(
                syn::parse_str(t).expect("test type"),
            ));
        }
        reg
    }
}

impl Prebindgen for IdentityExt {
    fn on_function(
        &self,
        f: &prebindgen_flat::flat::Function,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &f.name;
        vec![syn::parse_quote!(fn #ident() {})]
    }

    fn on_struct(
        &self,
        s: &prebindgen_flat::flat::Struct,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &s.name;
        vec![syn::parse_quote!(pub struct #ident;)]
    }

    fn on_variant(
        &self,
        v: &prebindgen_flat::flat::Variant,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &v.name;
        vec![syn::parse_quote!(pub enum #ident {})]
    }

    fn on_enum(
        &self,
        e: &prebindgen_flat::flat::Enum,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &e.name;
        vec![syn::parse_quote!(pub enum #ident {})]
    }

    fn on_const(
        &self,
        c: &prebindgen_flat::flat::Constant,
        _registry: &Registry,
        emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &c.name;
        let ty = emit.emit_source_type(&c.ty);
        vec![syn::parse_quote!(pub const #ident: #ty = 0;)]
    }
}

#[derive(Clone)]
struct LatePlan {
    operation: crate::generation::OperationId,
    reachable: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct OperationPlan {
    operation: crate::generation::OperationId,
    reachable: bool,
    renders: Rc<Cell<usize>>,
    rendered_reachable: Rc<Cell<bool>>,
}

impl RustArtifact for OperationPlan {
    fn key(&self) -> ArtifactKey {
        ArtifactKey::Operation(self.operation.clone())
    }

    fn reachable(&self) -> bool {
        self.reachable
    }

    fn render(&self, emit: &crate::RustWriter) -> Vec<syn::Item> {
        self.renders.set(self.renders.get() + 1);
        self.rendered_reachable.set(self.reachable);
        let ident = emit.operation_ident("test", &self.operation);
        vec![syn::parse_quote!(fn #ident() {})]
    }
}

impl RustArtifact for LatePlan {
    fn key(&self) -> ArtifactKey {
        ArtifactKey::Operation(self.operation.clone())
    }

    fn reachable(&self) -> bool {
        self.reachable.get()
    }

    fn render(&self, emit: &crate::RustWriter) -> Vec<syn::Item> {
        let ident = emit.operation_ident("test", &self.operation);
        vec![syn::parse_quote!(
            fn #ident() {}
        )]
    }
}

struct LateExt {
    reachable: Rc<Cell<bool>>,
    activate: bool,
    call_converter: bool,
    operation: crate::generation::OperationId,
}

impl Prebindgen for LateExt {
    fn on_function(
        &self,
        f: &prebindgen_flat::flat::Function,
        _registry: &Registry,
        emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        if self.activate {
            self.reachable.set(true);
        }
        if self.call_converter {
            let ident = &f.name;
            let converter = emit.operation_ident("test", &self.operation);
            vec![syn::Item::Fn(
                syn::parse_quote!(fn #ident() { #converter(); }),
            )]
        } else {
            let ident = &f.name;
            vec![syn::parse_quote!(fn #ident() {})]
        }
    }

    fn on_struct(
        &self,
        s: &prebindgen_flat::flat::Struct,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &s.name;
        vec![syn::parse_quote!(pub struct #ident;)]
    }

    fn on_variant(
        &self,
        v: &prebindgen_flat::flat::Variant,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &v.name;
        vec![syn::parse_quote!(pub enum #ident {})]
    }

    fn on_enum(
        &self,
        e: &prebindgen_flat::flat::Enum,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        let ident = &e.name;
        vec![syn::parse_quote!(pub enum #ident {})]
    }

    fn on_const(
        &self,
        _c: &prebindgen_flat::flat::Constant,
        _registry: &Registry,
        _emit: &crate::RustWriter,
    ) -> Vec<syn::Item> {
        Vec::new()
    }
}

#[test]
fn per_item_planning_precedes_late_converter_filtering() {
    let item: syn::ItemFn = syn::parse_quote!(
        fn a_fn() {}
    );
    let ident: syn::Ident = syn::parse_quote!(a_fn);
    let registry =
        crate::test_util::reg_from_items(vec![(syn::Item::Fn(item), SourceLocation::default())])
            .expect("index")
            .export(&ident)
            .scanned()
            .expect("scan");
    let reachable = Rc::new(Cell::new(false));
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "late-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let ext = LateExt {
        reachable: reachable.clone(),
        activate: true,
        call_converter: false,
        operation: operation.clone(),
    };
    let plan = LatePlan {
        operation,
        reachable,
    };
    let dir = crate::test_util::unique_test_dir("write_late_plan");
    std::fs::create_dir_all(&dir).unwrap();

    let path =
        write_rust(&registry, &ext, &assembly_of([plan]), dir.join("gen.rs")).expect("write_rust");
    let source = std::fs::read_to_string(path).expect("read generated file");

    assert!(
        source.contains("fn __test_in_convert_wire_to_test_late_converter_"),
        "{source}"
    );
    assert!(source.find("fn __test_in_convert").unwrap() < source.find("fn a_fn").unwrap());
}

#[test]
fn a_call_to_a_filtered_converter_is_a_writer_error() {
    let item: syn::ItemFn = syn::parse_quote!(
        fn a_fn() {}
    );
    let ident: syn::Ident = syn::parse_quote!(a_fn);
    let registry =
        crate::test_util::reg_from_items(vec![(syn::Item::Fn(item), SourceLocation::default())])
            .expect("index")
            .export(&ident)
            .scanned()
            .expect("scan");
    let reachable = Rc::new(Cell::new(false));
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "late-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let ext = LateExt {
        reachable: reachable.clone(),
        activate: false,
        call_converter: true,
        operation: operation.clone(),
    };
    let plan = LatePlan {
        operation: operation.clone(),
        reachable,
    };
    let dir = crate::test_util::unique_test_dir("write_missing_converter");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("gen.rs");

    let err = write_rust(&registry, &ext, &assembly_of([plan]), &path)
        .expect_err("a call to a filtered private converter must fail in the writer");

    match err {
        WriteError::UnrenderedConverterCalls { calls } => {
            let missing = crate::RustWriter::for_test()
                .operation_ident("test", &operation)
                .to_string();
            assert_eq!(calls, vec![("a_fn".to_string(), missing)]);
        }
    }
    assert!(
        !path.exists(),
        "an invalid generated file must not reach the destination"
    );
}

#[test]
fn registry_operations_are_deduplicated_before_rendering() {
    let item: syn::ItemFn = syn::parse_quote!(
        fn a_fn() {}
    );
    let ident: syn::Ident = syn::parse_quote!(a_fn);
    let registry =
        crate::test_util::reg_from_items(vec![(syn::Item::Fn(item), SourceLocation::default())])
            .expect("index")
            .export(&ident)
            .scanned()
            .expect("scan");
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "shared-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let renders = Rc::new(Cell::new(0));
    let rendered_reachable = Rc::new(Cell::new(false));
    let dormant = OperationPlan {
        operation: operation.clone(),
        reachable: false,
        renders: renders.clone(),
        rendered_reachable: rendered_reachable.clone(),
    };
    let reachable = OperationPlan {
        operation,
        reachable: true,
        renders: renders.clone(),
        rendered_reachable: rendered_reachable.clone(),
    };
    let dir = crate::test_util::unique_test_dir("write_operation_dedup");
    std::fs::create_dir_all(&dir).unwrap();

    write_rust(
        &registry,
        &IdentityExt,
        &assembly_of([dormant, reachable]),
        dir.join("gen.rs"),
    )
    .expect("write Rust");

    assert_eq!(
        renders.get(),
        1,
        "a shared registry operation must be rendered exactly once"
    );
    assert!(
        rendered_reachable.get(),
        "a reachable representative must replace an earlier dormant twin"
    );
}

#[test]
fn write_rust_sorts_declared_items_by_ident() {
    // Fed in a deliberately un-sorted order: the assertion below is that
    // emission sorts by name, and the model preserves stream order.
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                fn b_fn() {}
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                fn a_fn() {}
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub struct BStruct;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub struct AStruct;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub enum BEnum {
                    B,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub enum AEnum {
                    A,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const B_CONST: u32 = 2;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const A_CONST: u32 = 1;
            ),
            loc,
        ),
    ];
    let reg: Registry = IdentityExt
        .declare_into(crate::test_util::reg_from_items(items).expect("index"))
        .scanned()
        .expect("scan");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("prebindgen-write-rust-{unique}.rs"));
    let written = write_rust(
        &reg,
        &IdentityExt,
        &assembly_of([] as [OperationPlan; 0]),
        &path,
    )
    .expect("write_rust");
    let content = std::fs::read_to_string(&written).expect("read generated file");
    let _ = std::fs::remove_file(&written);

    assert!(
        content.find("pub const A_CONST").unwrap() < content.find("pub const B_CONST").unwrap()
    );
    assert!(content.find("pub enum AEnum").unwrap() < content.find("pub enum BEnum").unwrap());
    assert!(
        content.find("pub struct AStruct").unwrap() < content.find("pub struct BStruct").unwrap()
    );
    assert!(content.find("fn a_fn").unwrap() < content.find("fn b_fn").unwrap());
}

#[test]
fn per_item_emission_carries_typed_items_without_reparsing() {
    let contract = include_str!("../prebindgen.rs");
    let item_methods = contract
        .split_once("// ── Item methods")
        .expect("item methods")
        .1;
    assert_eq!(item_methods.matches("-> Vec<syn::Item>").count(), 5);

    let writer = include_str!("../write.rs");
    for removed in ["parse_items_from_tokens", "BadTokens", "syn::parse2"] {
        assert!(
            !writer.contains(removed),
            "typed per-item emission must not restore `{removed}`"
        );
    }
}

/// An adapter with a const mechanism gates **named** consts and cannot gate
/// guards — pinned at the emission site, not just in the registry.
///
/// `a_guard_never_reaches_the_const_surface` proves the maps are separate, but it
/// never calls `write_rust`. This is what would catch a change that keeps
/// `Registry::guards` populated and then forgets to emit them, or re-gates them
/// on the way out.
#[test]
fn guards_emit_ungated_and_in_stream_order() {
    /// Declares a const mechanism and declares nothing through it, so
    /// `KEPT_OUT` must not emit.
    struct ConstGatingExt;

    trait ResolveGating {
        fn resolve_gating(self, ext: ConstGatingExt) -> Result<Registry, crate::WriteRustError>;
    }
    impl ResolveGating for RegistryBuilder {
        fn resolve_gating(self, ext: ConstGatingExt) -> Result<Registry, crate::WriteRustError> {
            let registry = self.declares_consts().build()?;
            let _ = &ext;
            Ok(registry)
        }
    }

    impl Prebindgen for ConstGatingExt {
        fn on_function(
            &self,
            _f: &prebindgen_flat::flat::Function,
            _r: &Registry,
            _emit: &crate::RustWriter,
        ) -> Vec<syn::Item> {
            Vec::new()
        }
        fn on_struct(
            &self,
            _s: &prebindgen_flat::flat::Struct,
            _r: &Registry,
            _emit: &crate::RustWriter,
        ) -> Vec<syn::Item> {
            Vec::new()
        }
        fn on_variant(
            &self,
            _v: &prebindgen_flat::flat::Variant,
            _r: &Registry,
            _emit: &crate::RustWriter,
        ) -> Vec<syn::Item> {
            Vec::new()
        }
        fn on_enum(
            &self,
            _e: &prebindgen_flat::flat::Enum,
            _r: &Registry,
            _emit: &crate::RustWriter,
        ) -> Vec<syn::Item> {
            Vec::new()
        }
        fn on_const(
            &self,
            _c: &prebindgen_flat::flat::Constant,
            _r: &Registry,
            _emit: &crate::RustWriter,
        ) -> Vec<syn::Item> {
            Vec::new()
        }
    }

    let loc = SourceLocation::default();
    // Two distinguishable guards, straddling the named const, so the assertion
    // below pins order rather than merely presence.
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                const _: () = {
                    first_check();
                };
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const KEPT_OUT: u64 = 7;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                const _: () = {
                    second_check();
                };
            ),
            loc.clone(),
        ),
    ];
    let registry: RegistryBuilder = crate::test_util::reg_from_items(items).expect("index");
    assert_eq!(registry.flat().guards().count(), 2);

    let dir = crate::test_util::unique_test_dir("write_guards");
    std::fs::create_dir_all(&dir).unwrap();
    let registry = registry.resolve_gating(ConstGatingExt).expect("resolve");
    let path = crate::write::write_rust(
        &registry,
        &ConstGatingExt,
        &assembly_of([] as [OperationPlan; 0]),
        dir.join("gen.rs"),
    )
    .expect("write_rust");
    let src = std::fs::read_to_string(&path).unwrap();

    // The named const is gated out; both guards emit regardless.
    assert!(
        !src.contains("KEPT_OUT"),
        "declared_consts is empty:\n{src}"
    );
    let first = src
        .find("first_check")
        .unwrap_or_else(|| panic!("guard 1 missing:\n{src}"));
    let second = src
        .find("second_check")
        .unwrap_or_else(|| panic!("guard 2 missing:\n{src}"));
    assert!(first < second, "guards must keep stream order:\n{src}");
}
