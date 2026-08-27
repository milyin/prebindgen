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

/// An artifact that renders `a_fn`, calling the late converter plan.
///
/// Its own reachability is fixed; what varies is whether the plan it calls is
/// reachable by the time the file is written.
#[derive(Clone)]
struct CallerPlan {
    operation: crate::generation::OperationId,
}

impl RustArtifact for CallerPlan {
    fn key(&self) -> ArtifactKey {
        ArtifactKey::Artifact(
            crate::generation::ArtifactId::new("test", "caller").expect("identity"),
        )
    }

    fn render(&self, emit: &crate::RustWriter) -> Vec<syn::Item> {
        let converter = emit.operation_ident("test", &self.operation);
        vec![syn::Item::Fn(
            syn::parse_quote!(fn a_fn() { #converter(); }),
        )]
    }
}

/// Either artifact of this test's assembly: a converter plan whose
/// reachability can flip after the assembly is frozen, or the caller.
#[derive(Clone)]
enum LateOrCaller {
    Converter(LatePlan),
    Caller(CallerPlan),
}

impl RustArtifact for LateOrCaller {
    fn key(&self) -> ArtifactKey {
        match self {
            Self::Converter(plan) => plan.key(),
            Self::Caller(plan) => plan.key(),
        }
    }

    fn reachable(&self) -> bool {
        match self {
            Self::Converter(plan) => plan.reachable(),
            Self::Caller(plan) => plan.reachable(),
        }
    }

    fn render(&self, emit: &crate::RustWriter) -> Vec<syn::Item> {
        match self {
            Self::Converter(plan) => plan.render(emit),
            Self::Caller(plan) => plan.render(emit),
        }
    }
}

/// A converter plan that is dormant when the assembly is frozen and reachable
/// by the time the file is written is emitted.
///
/// What this pins is that reachability is read when the file is written and
/// not snapshotted while the assembly is frozen. It does not pin *when* during
/// writing: nothing marks an artifact reachable while another renders, so
/// reading every artifact's reachability just before the render loop would
/// satisfy this too.
///
/// The property is not hypothetical. JniGen shares one reachability cell
/// between every clone of a plan and sets it when a parent that calls the plan
/// is compiled, which can happen after the plan has been added to the builder.
#[test]
fn an_artifact_reached_after_freezing_is_emitted() {
    let registry = late_test_registry();
    let reachable = Rc::new(Cell::new(false));
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "late-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let assembly = assembly_of([LateOrCaller::Converter(LatePlan {
        operation,
        reachable: reachable.clone(),
    })]);
    let dir = crate::test_util::unique_test_dir("write_late_plan");
    std::fs::create_dir_all(&dir).unwrap();

    // Frozen dormant, reached afterwards — as a parent compiled later does.
    reachable.set(true);
    let path =
        write_rust(&registry, &IdentityExt, &assembly, dir.join("gen.rs")).expect("write_rust");
    let source = std::fs::read_to_string(path).expect("read generated file");

    assert!(
        source.contains("fn __test_in_convert_wire_to_test_late_converter_"),
        "{source}"
    );
}

#[test]
fn a_call_to_a_filtered_converter_is_a_writer_error() {
    let registry = late_test_registry();
    let reachable = Rc::new(Cell::new(false));
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "late-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let assembly = assembly_of([
        LateOrCaller::Converter(LatePlan {
            operation: operation.clone(),
            reachable,
        }),
        LateOrCaller::Caller(CallerPlan {
            operation: operation.clone(),
        }),
    ]);
    let dir = crate::test_util::unique_test_dir("write_missing_converter");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("gen.rs");

    let err = write_rust(&registry, &IdentityExt, &assembly, &path)
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

/// A registry with one declared type, which is all these two tests need of it.
fn late_test_registry() -> Registry {
    crate::test_util::reg_from_items(vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct AStruct;
        )),
        SourceLocation::default(),
    )])
    .expect("index")
    .export_type(crate::test_util::declared_origin(syn::parse_quote!(
        AStruct
    )))
    .scanned()
    .expect("scan")
}

/// Two reachable artifacts under one identity are a planning error. A
/// converter is exempt: one registry operation is legitimately reached from
/// several sites, which is what the de-duplication above is for.
#[test]
#[should_panic(expected = "two reachable artifacts share the identity")]
fn two_reachable_artifacts_may_not_share_an_identity() {
    let operation = crate::generation::OperationId::shared(
        crate::generation::ArtifactId::new("test", "shared-converter").expect("identity"),
        crate::recipe::Direction::Construct,
    );
    let caller = || {
        LateOrCaller::Caller(CallerPlan {
            operation: operation.clone(),
        })
    };
    let _ = assembly_of([caller(), caller()]);
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
        operation: operation.clone(),
        reachable: true,
        renders: renders.clone(),
        rendered_reachable: rendered_reachable.clone(),
    };
    // One registry operation reached from a second site: legal, and the
    // exemption the duplicate-identity guard states.
    let reached_again = OperationPlan {
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
        &assembly_of([dormant, reachable, reached_again]),
        dir.join("gen.rs"),
    )
    .expect("write Rust");

    assert_eq!(
        renders.get(),
        1,
        "a shared registry operation must be rendered exactly once, however \
         many sites reached it"
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

/// The adapter hands the writer no Rust items at all — everything it emits is
/// an artifact of its assembly — and the writer never turns tokens back into
/// items itself.
#[test]
fn the_adapter_emits_no_items_and_the_writer_reparses_none() {
    let contract = include_str!("../prebindgen.rs");
    assert_eq!(
        contract.matches("-> Vec<syn::Item>").count(),
        0,
        "no method of the trait may return items for the writer to place"
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
