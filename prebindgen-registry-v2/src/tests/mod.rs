//! What the engine does with a declaration set, independent of any adapter.

mod pipeline;

use prebindgen_flat::flat::FlatBuilder;

use crate::{
    decl::Declaration,
    outcome::EngineError,
    plan::generate,
    run::Generation,
    target::{
        BoundarySpec, ChildValue, ReprSpec, ResolvedShape, ResolvedValues, Selection,
        SelectionQuery, SiteDescriptor, SurfaceRequest, SurfaceSpec, Target, TargetAttempt,
        TargetSupport, Unsupported,
    },
};

/// A target that carries nothing: every value is refused at selection, so a
/// run over it exercises the accounting and nothing else.
///
/// What it was told an output is, standing in for a real target's choice.
///
/// The two halves are separate on purpose: the engine tells two declarations
/// of one entity apart by the whole choice, while the report names each row
/// by where the target says it is placed — and a target may place two
/// different choices identically.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Declared {
    id: u32,
    placement: String,
}

impl Declared {
    /// An output whose placement no test reads.
    fn any() -> Self {
        Declared {
            id: 0,
            placement: String::new(),
        }
    }

    /// One placed where the report will name it.
    fn placed(id: u32, placement: &str) -> Self {
        Declared {
            id,
            placement: placement.to_string(),
        }
    }
}

struct Nothing;

impl Target for Nothing {
    const NAME: &'static str = "test";

    type ConversionKey = Declared;
    type Payload = ();

    fn select(&self, query: &SelectionQuery<'_, Declared>) -> TargetSupport<Selection<Declared>> {
        Ok(TargetAttempt::Unsupported(Unsupported::new(
            "unsupported.nothing.carrier",
            format!("`{}` is carried by no target here", query.crossing.ty.key()),
        )))
    }

    fn represent(
        &self,
        _: &ResolvedShape<'_>,
        _: &[ChildValue<'_>],
        _: &Declared,
    ) -> TargetSupport<ReprSpec<()>> {
        unreachable!("nothing is selected")
    }

    fn boundary(
        &self,
        _: &SiteDescriptor<'_, Declared>,
        _: &ResolvedValues<'_, ()>,
    ) -> TargetSupport<BoundarySpec<()>> {
        unreachable!("nothing is selected")
    }

    fn surface(
        &self,
        _: &SurfaceRequest<'_, Declared>,
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
    declared: Vec<(Declaration, Declared)>,
}

/// Run the stated binding through the engine over [`sources`].
fn plan(stated: &Stated, sources: FlatBuilder) -> Result<Generation<()>, EngineError> {
    generate(
        sources.build()?,
        &Nothing,
        stated.declared.clone(),
        syn::parse_quote!(fixture),
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
            (captured_fn("handle_new"), Declared::any()),
            (declared_type("Handle"), Declared::any()),
        ],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");

    // Nothing crosses for this target, so both declarations come back as
    // skips — in the order the binding stated them.
    let skipped: Vec<String> = generation
        .skipped()
        .iter()
        .map(|(declaration, _)| declaration.to_string())
        .collect();
    assert_eq!(skipped, ["fn:handle_new", "type:Handle"]);
    assert!(generation.surfaces().is_empty());
    assert_eq!(generation.flat().captured().count(), 3);
}

#[test]
fn a_declaration_that_names_nothing_captured_is_an_error() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_neu"), Declared::any())],
    };
    let error = plan(&stated, sources()).expect_err("a typo is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(error.to_string().contains("handle_neu"), "{error}");
}

/// The binding may define a thing the source never captured — a callback
/// signature, a helper — and says so per declaration.
#[test]
fn a_declaration_the_binding_defines_itself_is_not_looked_up() {
    let stated = Stated {
        declared: vec![(
            Declaration::Callback("impl Fn(i64)".to_string()),
            Declared::any(),
        )],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");
    assert_eq!(generation.skipped().len(), 1);
}

#[test]
fn one_declaration_may_be_stated_once_per_choice() {
    let stated = Stated {
        declared: vec![
            (captured_fn("handle_new"), Declared::any()),
            (captured_fn("handle_new"), Declared::any()),
        ],
    };
    let error = plan(&stated, sources()).expect_err("a repeat is refused");
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
            Declaration::Const(syn::parse_str("handle_new").expect("an ident")),
            Declared::any(),
        )],
    };
    let error = plan(&stated, sources()).expect_err("wrong kind is refused");
    assert!(matches!(error, EngineError::DeclaredNotFound { .. }));
    assert!(
        error
            .to_string()
            .contains("no captured constant `handle_new`"),
        "the refusal names the kind it looked for: {error}"
    );
}

/// One missing capability takes down every declaration that needed it, and
/// each says so for itself: the code is what a build script groups on.
#[test]
fn every_skip_names_the_capability_that_stopped_it() {
    let stated = Stated {
        declared: vec![
            (captured_fn("handle_new"), Declared::any()),
            (captured_fn("handle_value"), Declared::any()),
            (declared_type("Handle"), Declared::any()),
        ],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");
    // One cause, three roots: each stops at the first value the target is
    // asked about.
    let codes: Vec<&str> = generation
        .skipped()
        .iter()
        .map(|(_, skip)| skip.capability.as_str())
        .collect();
    assert_eq!(codes, ["unsupported.nothing.carrier"; 3]);
}

/// One entity declared several times is several skips, in the order the
/// binding stated them.
#[test]
fn one_entity_declared_three_times_is_three_skips() {
    let stated = Stated {
        declared: vec![
            (captured_fn("handle_new"), Declared::placed(1, "x#2")),
            (captured_fn("handle_new"), Declared::placed(2, "x")),
            (captured_fn("handle_new"), Declared::placed(3, "x")),
        ],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");
    let skipped: Vec<String> = generation
        .skipped()
        .iter()
        .map(|(declaration, _)| declaration.to_string())
        .collect();
    assert_eq!(skipped, ["fn:handle_new"; 3]);
}
