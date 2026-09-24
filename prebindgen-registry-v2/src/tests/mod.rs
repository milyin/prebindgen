//! What the engine does with a declaration set, independent of any adapter.

mod pipeline;

use prebindgen_flat::flat::FlatBuilder;

use crate::{
    binding::{Accepts, Binding, FunctionForm, OutputForm, Representation, Scope, WireClass},
    decl::Declaration,
    outcome::EngineError,
    plan::generate,
    run::Generation,
    target::{CarrierFeed, OperationFeed, Target, Unsupported, Written},
};

/// A target that carries nothing: every value it is asked about is refused, so
/// a run over it exercises the accounting and nothing else.
struct Nothing;

/// The classes of a target that carries nothing: there are none.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum NoClass {}

impl WireClass for NoClass {
    fn all() -> Vec<Self> {
        Vec::new()
    }

    fn name(&self) -> &'static str {
        match *self {}
    }

    fn rust(&self) -> syn::Type {
        match *self {}
    }
}

impl Target for Nothing {
    const NAME: &'static str = "test";

    type WireClass = NoClass;
    type CarrierMeta = ();
    type Op = ();
    /// Which output this is, standing in for a real target's placement: the
    /// engine tells two declarations of one entity apart by their whole form.
    type OutputMeta = u32;

    fn write_operation(&self, _: &(), _: &OperationFeed<'_, Self>) -> Written {
        unreachable!("nothing is planned")
    }

    fn write_carrier(&self, _: &CarrierFeed<'_, Self>) -> Vec<proc_macro2::TokenStream> {
        unreachable!("nothing is planned")
    }
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

/// A binding stated directly, standing in for a frontend: every value of
/// `Handle`, owned or borrowed, is refused, and each output is placed at `id`.
struct Stated {
    declared: Vec<(Declaration, u32)>,
}

/// Run the stated binding through the engine over [`sources`].
fn plan(stated: &Stated, sources: FlatBuilder) -> Result<Generation<Nothing>, EngineError> {
    let mut binding: Binding<Nothing> = Binding::new();
    let refused = binding.representation(Representation::Unsupported(Unsupported::new(
        "unsupported.nothing.carrier",
        "this target carries nothing",
    )));
    for key in ["Handle", "&Handle"] {
        binding.rule(
            Scope::Type(prebindgen_flat::TypeKey::parse(key).expect("a type key")),
            refused,
        );
    }
    for (declaration, id) in &stated.declared {
        let form = match declaration {
            Declaration::Type(_) | Declaration::Callback(_) => OutputForm::Type {
                representation: refused,
                release: None,
                meta: *id,
            },
            _ => OutputForm::Function {
                form: FunctionForm {
                    abi: "C".to_string(),
                    symbol: format!("f{id}"),
                    context: Vec::new(),
                    inputs: Vec::new(),
                    routes: Vec::new(),
                    attrs: Vec::new(),
                    unsafety: false,
                    params: Accepts::of([]),
                    ret: Accepts::of([]),
                },
                meta: *id,
            },
        };
        binding.output(declaration.clone(), form);
    }
    generate(
        sources.build()?,
        &Nothing,
        binding,
        syn::parse_quote!(fixture),
    )
}

#[test]
fn every_declaration_is_skipped_and_accounted_for() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_new"), 0), (declared_type("Handle"), 0)],
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
    assert!(generation.retained().is_empty());
    assert_eq!(generation.flat().captured().count(), 3);
}

#[test]
fn a_declaration_that_names_nothing_captured_is_an_error() {
    let stated = Stated {
        declared: vec![(captured_fn("handle_neu"), 0)],
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
            Declaration::Callback(
                prebindgen_flat::TypeKey::parse("impl Fn(i64) + Send + Sync + 'static")
                    .expect("a callback type"),
            ),
            0,
        )],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");
    assert_eq!(generation.skipped().len(), 1);
}

#[test]
fn one_declaration_may_be_stated_once_per_form() {
    let stated = Stated {
        declared: vec![
            (captured_fn("handle_new"), 0),
            (captured_fn("handle_new"), 0),
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
            0,
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
            (captured_fn("handle_new"), 0),
            (captured_fn("handle_value"), 1),
            (declared_type("Handle"), 0),
        ],
    };
    let generation = plan(&stated, sources()).expect("v2 plans");
    // One cause, three roots: each stops at the first value it needs.
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
            (captured_fn("handle_new"), 1),
            (captured_fn("handle_new"), 2),
            (captured_fn("handle_new"), 3),
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
