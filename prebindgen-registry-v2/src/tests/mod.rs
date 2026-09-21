//! What the engine does with a declaration set, independent of any adapter.

mod pipeline;

use prebindgen_flat::flat::FlatBuilder;

use crate::{
    decl::Declaration,
    outcome::{EngineError, Outcome},
    plan::{generate, BindingRequests},
    run::Generation,
    target::{
        BoundarySpec, ChildValue, Described, RelationId, ReprSpec, ResolvedShape, ResolvedValues,
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

    fn describe(&self, _: &()) -> Described {
        Described::new("declared", "c_")
    }
}

/// A binding stated directly, standing in for a facade's own storage.
struct Stated {
    declared: Vec<Declaration>,
    ignored: Vec<Declaration>,
}

/// Run the stated binding through the engine over [`sources`].
fn plan(
    stated: &Stated,
    sources: FlatBuilder,
    crate_name: &str,
) -> Result<Generation<()>, EngineError> {
    let mut requests = BindingRequests::new("test", syn::parse_quote!(fixture), ());
    for declaration in &stated.declared {
        requests.output(declaration.clone(), requests.default_policy);
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

/// A captured function's origin, by name.
fn captured_fn(name: &str) -> Declaration {
    Declaration::Function(syn::parse_str(name).expect("a test names an ident"))
}

/// A Kotlin `val` read through that captured function.
fn constant_fn(name: &str) -> Declaration {
    Declaration::ConstFromFunction(syn::parse_str(name).expect("a test names an ident"))
}

/// A type the binding represents, whether or not the source captured it.
fn local_type(name: &str) -> Declaration {
    Declaration::LocalType(prebindgen_flat::TypeKey::parse(name).expect("a test names a type"))
}

#[test]
fn every_declaration_is_skipped_and_every_ignore_is_counted_apart() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_new")), (local_type("Handle"))],
        ignored: vec![(Declaration::LocalFunction("handle_value".to_string()))],
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    let report = generation.report();

    let counts = report.counts();
    assert_eq!((counts.emitted, counts.skipped, counts.ignored), (0, 2, 1));
    assert_eq!(report.source_identity.captured_items, 3);
    assert_eq!(report.source_identity.declaring_crate, "fixture-crate");

    // Types sort before functions, and the ignore is an outcome like any other.
    let ids: Vec<String> = report
        .declarations
        .iter()
        .map(|entry| entry.declaration.to_string())
        .collect();
    assert_eq!(ids, ["fn:handle_new", "fn:handle_value", "type:Handle"]);
    assert_eq!(report.declarations[1].outcome, Outcome::Ignored);
}

#[test]
fn a_declaration_that_names_nothing_captured_is_an_error() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_neu"))],
        ignored: Vec::new(),
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
        declared: vec![(Declaration::Callback("impl Fn(i64)".to_string()))],
        ignored: Vec::new(),
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("v2 plans");
    assert_eq!(generation.report().counts().skipped, 1);
}

#[test]
fn one_id_may_name_only_one_declaration() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_new")), (captured_fn("handle_new"))],
        ignored: Vec::new(),
    };
    let error = plan(&stated, sources(), "fixture-crate").expect_err("a repeat is refused");
    assert!(matches!(error, EngineError::DuplicateDeclaration { .. }));
    assert!(error.to_string().contains("fn:handle_new"), "{error}");

    // The same name under two target kinds is two declarations, and legal: the
    // captured function may back both a callable and a `val`.
    let stated = Stated {
        declared: vec![(captured_fn("handle_new")), (constant_fn("handle_new"))],
        ignored: Vec::new(),
    };
    plan(&stated, sources(), "fixture-crate").expect("two kinds, two ids");
}

/// A declaration names one of the three captured kinds, and naming the wrong
/// one fails the run instead of being reported as a skipped capability.
#[test]
fn a_declaration_must_name_the_kind_it_says_it_does() {
    // `handle_new` is a captured function, so declaring it as a constant is as
    // wrong as declaring a name nothing captured.
    let stated = Stated {
        declared: vec![(Declaration::Const(syn::parse_str("handle_new").expect("an ident")))],
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
    // because the origin says which kind to look for.
    let stated = Stated {
        declared: vec![(constant_fn("handle_new"))],
        ignored: Vec::new(),
    };
    let generation = plan(&stated, sources(), "fixture-crate").expect("a function-backed constant");
    assert_eq!(generation.report().counts().skipped, 1);
}

/// The report groups by cause, so one missing capability is stated once with
/// every declaration it took down.
#[test]
fn skips_are_grouped_by_capability_code() {
    let stated = Stated {
        declared: vec![
            (captured_fn("handle_new")),
            (captured_fn("handle_value")),
            (local_type("Handle")),
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

/// The report's declaration columns are a published schema: the id as the
/// origin prints, what the target described, and the outcome's fields at the
/// same level.
#[test]
fn an_entry_serializes_flat() {
    let entry = crate::report::Entry {
        declaration: constant_fn("z_thing_describe"),
        described: Described::new("constant_fun", "example.DESCRIBE"),
        outcome: Outcome::Emitted,
    };
    assert_eq!(
        serde_json::to_string(&entry).expect("an entry is plain data"),
        r#"{"id":"const:z_thing_describe","placement":"example.DESCRIBE","representation":"constant_fun","outcome":"emitted"}"#
    );
}
