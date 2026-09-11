//! What the engine does with a declaration set, independent of any adapter.

mod pipeline;

use prebindgen_flat::flat::FlatBuilder;

use crate::{
    decl::{DeclaredElement, ElementKind, SourceKind},
    outcome::{EngineError, Outcome},
    plan::{generate, BindingRequests},
    run::Generation,
    target::{
        BoundarySpec, ChildValue, RelationId, ReprSpec, ResolvedShape, ResolvedValues,
        SelectionQuery, SiteDescriptor, SurfaceRequest, SurfaceSpec, Target, TargetAttempt,
        TargetSupport, Unsupported,
    },
};

/// A target that carries nothing: every value is refused at selection, so a
/// run over it exercises the accounting and nothing else.
struct Nothing;

impl Target for Nothing {
    type Policy = ();
    type Payload = ();

    fn select(&self, query: &SelectionQuery<'_, ()>) -> TargetSupport<RelationId> {
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.nothing.carrier",
            format!("`{}` is carried by no target here", query.crossing.ty.key()),
        )))
    }

    fn represent(
        &self,
        _: &ResolvedShape<'_>,
        _: &[ChildValue<'_>],
        _: &(),
    ) -> TargetSupport<ReprSpec<()>> {
        unreachable!("nothing is selected")
    }

    fn boundary(
        &self,
        _: &SiteDescriptor<'_>,
        _: &ResolvedValues<'_, ()>,
        _: &(),
    ) -> TargetSupport<BoundarySpec<()>> {
        unreachable!("nothing is selected")
    }

    fn surface(
        &self,
        _: &SurfaceRequest<'_, ()>,
        _: &ResolvedValues<'_, ()>,
    ) -> TargetSupport<SurfaceSpec<()>> {
        unreachable!("nothing is selected")
    }

    fn render_operation(&self, _: &(), _: &[syn::Ident]) -> proc_macro2::TokenStream {
        unreachable!("nothing is selected")
    }
}

/// A binding stated directly, standing in for a facade's own storage.
struct Stated {
    declared: Vec<DeclaredElement>,
    ignored: Vec<DeclaredElement>,
}

/// Run the stated binding through the engine over [`sources`].
fn plan(
    stated: &Stated,
    sources: FlatBuilder,
    crate_name: &str,
) -> Result<Generation<()>, EngineError> {
    let mut requests = BindingRequests::new("test", syn::parse_quote!(fixture), ());
    for element in &stated.declared {
        requests.output(element.clone(), requests.default_policy);
    }
    requests.ignored = stated.ignored.clone();
    generate(sources.build()?, &Nothing, requests, crate_name)
}

/// Two captured functions and a captured struct, in the shape a source crate
/// hands over.
fn sources() -> FlatBuilder {
    let location = prebindgen::SourceLocation {
        crate_name: Some("fixture".to_string()),
        ..Default::default()
    };
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
        (
            syn::parse_quote!(
                pub struct Handle {
                    value: i64,
                }
            ),
            location.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn handle_new() -> Handle {
                    unimplemented!()
                }
            ),
            location.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn handle_value(h: &Handle) -> i64 {
                    unimplemented!()
                }
            ),
            location,
        ),
    ];
    prebindgen_flat::Flat::builder().items(items)
}

fn element(kind: ElementKind, origin: &str) -> DeclaredElement {
    DeclaredElement::new(kind, origin, format!("c_{origin}"), "declared")
}

#[test]
fn every_declared_element_is_skipped_and_every_ignore_is_counted_apart() {
    let stated = Stated {
        declared: vec![
            element(ElementKind::Function, "handle_new"),
            element(ElementKind::Type, "Handle").local(),
        ],
        ignored: vec![element(ElementKind::Function, "handle_value").local()],
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    let report = generation.report();

    let counts = report.counts();
    assert_eq!((counts.emitted, counts.skipped, counts.ignored), (0, 2, 1));
    assert_eq!(report.source_identity.captured_items, 3);
    assert_eq!(report.source_identity.declaring_crate, "fixture-crate");

    // Types sort before functions, and the ignore is an outcome like any other.
    let ids: Vec<&str> = report
        .elements
        .iter()
        .map(|entry| entry.element.id.as_str())
        .collect();
    assert_eq!(ids, ["type:Handle", "fn:handle_new", "fn:handle_value"]);
    assert_eq!(report.elements[2].outcome, Outcome::Ignored);
}

#[test]
fn a_declaration_that_names_nothing_captured_is_an_error() {
    let stated = Stated {
        declared: vec![element(ElementKind::Function, "handle_neu")],
        ignored: Vec::new(),
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("a typo is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(error.to_string().contains("handle_neu"), "{error}");
}

/// The binding may define a thing the source never captured — a callback
/// signature, a helper — and says so per element.
#[test]
fn an_element_the_binding_defines_itself_is_not_looked_up() {
    let stated = Stated {
        declared: vec![element(ElementKind::Callback, "impl Fn(i64)").local()],
        ignored: Vec::new(),
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    assert_eq!(generation.report().counts().skipped, 1);
}

#[test]
fn one_id_may_name_only_one_element() {
    let stated = Stated {
        declared: vec![
            element(ElementKind::Function, "handle_new"),
            element(ElementKind::Function, "handle_new"),
        ],
        ignored: Vec::new(),
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("a repeat is refused");
    assert!(matches!(error, EngineError::DuplicateElement { .. }));
    assert!(error.to_string().contains("fn:handle_new"), "{error}");

    // The same name under two target kinds is two elements, and legal: the
    // captured function may back both a callable and a `val`.
    let stated = Stated {
        declared: vec![
            element(ElementKind::Function, "handle_new"),
            element(ElementKind::Const, "handle_new").sourced_as(SourceKind::Function),
        ],
        ignored: Vec::new(),
    };
    plan(&stated, sources(), "fixture-crate").expect("two kinds, two ids");
}

/// A declaration names one of the three captured kinds, and naming the wrong
/// one is a mistake rather than a shape v2 has yet to implement.
#[test]
fn a_declaration_must_name_the_kind_it_says_it_does() {
    // `handle_new` is a captured function, so declaring it as a constant is as
    // wrong as declaring a name nothing captured.
    let stated = Stated {
        declared: vec![element(ElementKind::Const, "handle_new")],
        ignored: Vec::new(),
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("wrong kind is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(
        error
            .to_string()
            .contains("no captured constant `handle_new`"),
        "the refusal names the kind it looked for: {error}"
    );

    // And a constant-shaped output backed by a captured function resolves,
    // because the element says which kind to look for.
    let stated = Stated {
        declared: vec![element(ElementKind::Const, "handle_new").sourced_as(SourceKind::Function)],
        ignored: Vec::new(),
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("a function-backed constant");
    assert_eq!(generation.report().counts().skipped, 1);
}

/// The manifest groups by cause, so one missing capability is stated once with
/// every element it took down.
#[test]
fn skips_are_grouped_by_capability_code() {
    let stated = Stated {
        declared: vec![
            element(ElementKind::Function, "handle_new"),
            element(ElementKind::Function, "handle_value"),
            element(ElementKind::Type, "Handle").local(),
        ],
        ignored: Vec::new(),
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    let groups = generation.report().skips_by_capability();
    // One cause, three roots: each stops at the first value the target is
    // asked about.
    assert_eq!(groups["unsupported.nothing.carrier"].len(), 3);
    assert_eq!(groups.len(), 1);
}
