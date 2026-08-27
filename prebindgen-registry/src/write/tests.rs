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

impl Prebindgen for IdentityExt {}

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
    /// Emits `a_fn`, and (when `call_converter`) calls the late converter
    /// plan from it. Prerequisites are the adapter's remaining output
    /// produced while the file is written, so this is where a caller that
    /// races the assembly's reachability filtering can still come from.
    fn prerequisites(&self, _registry: &Registry, emit: &crate::RustWriter) -> Vec<syn::Item> {
        if self.activate {
            self.reachable.set(true);
        }
        if self.call_converter {
            let converter = emit.operation_ident("test", &self.operation);
            vec![syn::Item::Fn(
                syn::parse_quote!(fn a_fn() { #converter(); }),
            )]
        } else {
            vec![syn::parse_quote!(
                fn a_fn() {}
            )]
        }
    }
}

#[test]
fn per_item_planning_precedes_late_converter_filtering() {
    let item: syn::ItemStruct = syn::parse_quote!(
        pub struct AStruct;
    );
    let registry = crate::test_util::reg_from_items(vec![(
        syn::Item::Struct(item),
        SourceLocation::default(),
    )])
    .expect("index")
    .export_type(crate::test_util::declared_origin(syn::parse_quote!(
        AStruct
    )))
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
    // Prerequisites are written before the assembly, so the caller precedes
    // the converter it activated. What the test pins is that the activation
    // was seen at all: a plan marked reachable while the file is assembled
    // survives the filter.
    assert!(source.find("fn a_fn").unwrap() < source.find("fn __test_in_convert").unwrap());
}

#[test]
fn a_call_to_a_filtered_converter_is_a_writer_error() {
    let item: syn::ItemStruct = syn::parse_quote!(
        pub struct AStruct;
    );
    let registry = crate::test_util::reg_from_items(vec![(
        syn::Item::Struct(item),
        SourceLocation::default(),
    )])
    .expect("index")
    .export_type(crate::test_util::declared_origin(syn::parse_quote!(
        AStruct
    )))
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
fn declared_items_reach_the_file_only_as_artifacts() {
    // Fed in a deliberately un-sorted order, as the sorting this replaced was
    // fed: every kind is declared, and none of them may reach the file from
    // the writer's own walk, because there is no such walk left.
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

    for declared in [
        "fn a_fn", "fn b_fn", "AStruct", "BStruct", "AEnum", "BEnum", "A_CONST", "B_CONST",
    ] {
        assert!(
            !content.contains(declared),
            "`{declared}` reached the file from an empty assembly:\n{content}"
        );
    }
}

/// What an adapter still hands the writer is typed Rust items, and the writer
/// never turns tokens back into items itself.
#[test]
fn adapter_emission_carries_typed_items_without_reparsing() {
    let contract = include_str!("../prebindgen.rs");
    assert_eq!(
        contract.matches("-> Vec<syn::Item>").count(),
        1,
        "`prerequisites` is the one method that still returns items to the writer"
    );

    let writer = include_str!("../write.rs");
    for removed in ["parse_items_from_tokens", "BadTokens", "syn::parse2"] {
        assert!(
            !writer.contains(removed),
            "typed emission must not restore `{removed}`"
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

    impl Prebindgen for ConstGatingExt {}

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
