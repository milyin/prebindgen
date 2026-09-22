//! What the engine does with a declaration set, independent of any adapter.

mod pipeline;

use prebindgen_flat::flat::FlatBuilder;

use crate::{
    decl::Declaration,
    outcome::{EngineError, Outcome},
    plan::generate,
    run::Generation,
    target::{
        BoundarySpec, ChildValue, Described, ReprSpec, ResolvedShape, ResolvedValues, Selection,
        SelectionQuery, SiteDescriptor, SurfaceRequest, SurfaceSpec, Target, TargetAttempt,
        TargetSupport, Unsupported,
    },
};

/// A target that carries nothing: every value is refused at selection, so a
/// run over it exercises the accounting and nothing else.
struct Nothing;

impl Target for Nothing {
    const NAME: &'static str = "test";

    type ConversionKey = ();
    type Payload = ();

    fn select(&self, query: &SelectionQuery<'_, ()>) -> TargetSupport<Selection<()>> {
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
        _: &SiteDescriptor<'_, ()>,
        _: &ResolvedValues<'_, ()>,
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

    fn describe(&self, _: &Declaration, _: &()) -> Described {
        Described::new("declared", "c_")
    }
}

/// A binding stated directly, standing in for a facade's own storage.
struct Stated {
    declared: Vec<(Declaration, ())>,
}

/// Run the stated binding through the engine over [`sources`].
fn plan(
    stated: &Stated,
    sources: FlatBuilder,
    crate_name: &str,
) -> Result<Generation<()>, EngineError> {
    generate(
        sources.build()?,
        &Nothing,
        stated.declared.clone(),
        syn::parse_quote!(fixture),
        crate_name,
    )
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

/// A function, by name.
fn captured_fn(name: &str) -> Declaration {
    Declaration::Function(syn::parse_str(name).expect("a test names an ident"))
}

/// A type, by name.
fn declared_type(name: &str) -> Declaration {
    Declaration::Type(prebindgen_flat::TypeKey::parse(name).expect("a test names a type"))
}

#[test]
fn every_declaration_is_skipped_and_accounted_for() {
    let stated = Stated {
        declared: vec![
            ((captured_fn("handle_new")), ()),
            ((declared_type("Handle")), ()),
        ],
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    let report = generation.report();

    let counts = report.counts();
    assert_eq!((counts.emitted, counts.skipped), (0, 2));
    assert_eq!(report.source_identity.captured_items, 3);
    assert_eq!(report.source_identity.declaring_crate, "fixture-crate");

    // Types sort before functions.
    let ids: Vec<String> = report
        .declarations
        .iter()
        .map(|entry| entry.id().to_string())
        .collect();
    assert_eq!(ids, ["fn:handle_new", "type:Handle"]);
}

#[test]
fn a_declaration_that_names_nothing_captured_is_an_error() {
    let stated = Stated {
        declared: vec![((captured_fn("handle_neu")), ())],
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("a typo is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(error.to_string().contains("handle_neu"), "{error}");
}

/// The binding may define a thing the source never captured — a callback
/// signature, a helper — and says so per declaration.
#[test]
fn a_declaration_the_binding_defines_itself_is_not_looked_up() {
    let stated = Stated {
        declared: vec![((Declaration::Callback("impl Fn(i64)".to_string())), ())],
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    assert_eq!(generation.report().counts().skipped, 1);
}

#[test]
fn one_id_may_name_only_one_declaration() {
    let stated = Stated {
        declared: vec![
            ((captured_fn("handle_new")), ()),
            ((captured_fn("handle_new")), ()),
        ],
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("a repeat is refused");
    assert!(matches!(error, EngineError::DuplicateDeclaration { .. }));
    assert!(error.to_string().contains("fn:handle_new"), "{error}");
}

/// A declaration names one of the three captured kinds, and naming the wrong
/// one fails the run instead of being reported as a skipped capability.
#[test]
fn a_declaration_must_name_the_kind_it_says_it_does() {
    // `handle_new` is a captured function, so declaring it as a constant is as
    // wrong as declaring a name nothing captured.
    let stated = Stated {
        declared: vec![(
            (Declaration::Const(syn::parse_str("handle_new").expect("an ident"))),
            (),
        )],
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("wrong kind is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(
        error
            .to_string()
            .contains("no captured constant `handle_new`"),
        "the refusal names the kind it looked for: {error}"
    );
}

/// The report groups by cause, so one missing capability is stated once with
/// every declaration it took down.
#[test]
fn skips_are_grouped_by_capability_code() {
    let stated = Stated {
        declared: vec![
            ((captured_fn("handle_new")), ()),
            ((captured_fn("handle_value")), ()),
            ((declared_type("Handle")), ()),
        ],
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    let groups = generation.report().skips_by_capability();
    // One cause, three roots: each stops at the first value the target is
    // asked about.
    assert_eq!(groups["unsupported.nothing.carrier"].len(), 3);
    assert_eq!(groups.len(), 1);
}

/// The report's declaration columns are a published schema: the id as the
/// origin prints, what the target described, and the outcome's fields at the
/// same level.
#[test]
fn an_entry_serializes_flat() {
    let entry = crate::report::Entry {
        id: "fn:z_thing_describe".to_string(),
        declaration: captured_fn("z_thing_describe"),
        described: Described::new("constant_fun", "example.DESCRIBE"),
        outcome: Outcome::Emitted,
    };
    assert_eq!(
        serde_json::to_string(&entry).expect("an entry is plain data"),
        r#"{"id":"fn:z_thing_describe","placement":"example.DESCRIBE","representation":"constant_fun","outcome":"emitted"}"#
    );
}
