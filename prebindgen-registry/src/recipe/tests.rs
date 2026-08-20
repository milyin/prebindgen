use prebindgen::SourceLocation;

use super::*;
use crate::{flat::Flat, test_util::declare_referenced};

fn model(sources: &[&str]) -> Flat {
    let items = sources
        .iter()
        .map(|src| {
            let item: syn::Item = syn::parse_str(src).expect("parse item");
            (item, SourceLocation::default())
        })
        .collect::<Vec<_>>();
    Flat::builder()
        .items(declare_referenced(items))
        .build()
        .expect("parse")
}

fn ty(model: &Flat, spelling: &str) -> TypeRef {
    model
        .classify(&syn::parse_str(spelling).expect("parse type"))
        .expect("classify")
}

fn id(name: &str) -> RecipeId {
    RecipeId::new(name)
}

fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

/// The struct every fixture that needs parts is built over.
const SAMPLE: &str = "pub struct Sample { pub key: u32, pub payload: u64 }";

fn shape_of(row: &Row) -> &str {
    let shape = match row {
        Row::Callback(_) => return "callback",
        Row::Constructing(s) => {
            return match s {
                Shape::Atomic => "atomic",
                Shape::Optional { .. } => "optional",
                Shape::Sequence { .. } => "sequence",
                Shape::Product(_) => "product",
                Shape::Choice { .. } => "choice",
            }
        }
        Row::Deconstructing(s) => s,
    };
    match shape {
        Shape::Atomic => "atomic",
        Shape::Optional { .. } => "optional",
        Shape::Sequence { .. } => "sequence",
        Shape::Product(_) => "product",
        Shape::Choice { .. } => "choice",
    }
}

fn derived_shape(model: &Flat, spelling: &str) -> String {
    let table = Recipes::default();
    let crossing = Crossing::new(ty(model, spelling), Assembly::Deconstruct);
    let (id, row) = table.row(&crossing);
    format!("{id}:{}", shape_of(&row))
}

#[test]
fn an_undeclared_crossing_gets_its_arity_row_from_the_kind() {
    let model = model(&[SAMPLE]);
    assert_eq!(derived_shape(&model, "Sample"), "derived:atomic");
    assert_eq!(derived_shape(&model, "Option<Sample>"), "derived:optional");
    assert_eq!(derived_shape(&model, "Vec<Sample>"), "derived:sequence");
    assert_eq!(derived_shape(&model, "&[Sample]"), "derived:sequence");
    assert_eq!(derived_shape(&model, "[u8; 4]"), "derived:sequence");
    // One layer per row: the inner `Option` is the inner crossing's row, not
    // this one's.
    assert_eq!(
        derived_shape(&model, "Option<Option<Sample>>"),
        "derived:optional"
    );
}

#[test]
fn a_borrow_and_a_wrapper_find_the_row_the_bare_type_declared() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic);
    let table = builder.build(&model).expect("table");

    for spelling in ["Sample", "&Sample", "&mut Sample", "Box<Sample>"] {
        let crossing = Crossing::new(ty(&model, spelling), Assembly::Deconstruct);
        assert_eq!(
            table.row(&crossing).0,
            id("whole"),
            "{spelling} did not find the declared row"
        );
    }
}

#[test]
fn a_crossing_reports_the_mode_the_site_spelled() {
    let model = model(&[SAMPLE]);
    let mode = |spelling: &str| Crossing::new(ty(&model, spelling), Assembly::Construct).mode();
    assert_eq!(mode("Sample"), Mode::Owned);
    assert_eq!(mode("&Sample"), Mode::Shared);
    assert_eq!(mode("&mut Sample"), Mode::Exclusive);
    assert_eq!(mode("Box<Sample>"), Mode::Owned);
}

#[test]
fn the_shape_files_the_row_under_its_own_job() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Constructing::Product(Construct::Call(ident("sample_new"))),
        )
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    let table = builder.build(&model).expect("table");

    let sample = ty(&model, "Sample");
    for assembly in [Assembly::Construct, Assembly::Deconstruct] {
        let key = Crossing::new(sample.clone(), assembly).key();
        assert_eq!(key.assembly, assembly);
        assert_eq!(table.rows(&key), vec![&id("fields")]);
    }
}

#[test]
fn one_row_is_the_default_and_several_have_to_say_which() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let key = Crossing::new(sample.clone(), Assembly::Deconstruct).key();

    let mut one = Recipes::builder();
    one.declare(sample.clone(), id("whole"), Deconstructing::Atomic);
    let table = one.build(&model).expect("one row is its own default");
    assert_eq!(table.default_of(&key), Some(&id("whole")));

    let mut undecided = Recipes::builder();
    undecided
        .declare(sample.clone(), id("whole"), Deconstructing::Atomic)
        .declare(
            sample.clone(),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let errors = undecided.build(&model).expect_err("no default");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NoDefault { defaults, .. }] if defaults.is_empty()),
        "{errors:?}"
    );

    let mut decided = Recipes::builder();
    decided
        .declare_default(sample.clone(), id("whole"), Deconstructing::Atomic)
        .declare(
            sample,
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let table = decided.build(&model).expect("one default");
    assert_eq!(table.default_of(&key), Some(&id("whole")));
    assert_eq!(table.rows(&key).len(), 2);
}

#[test]
fn two_defaults_are_as_wrong_as_none() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let mut builder = Recipes::builder();
    builder
        .declare_default(sample.clone(), id("whole"), Deconstructing::Atomic)
        .declare_default(
            sample,
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let errors = builder.build(&model).expect_err("two defaults");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NoDefault { defaults, .. }] if defaults.len() == 2),
        "{errors:?}"
    );
}

#[test]
fn one_name_cannot_be_declared_twice_for_one_crossing() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let mut builder = Recipes::builder();
    builder
        .declare(sample.clone(), id("whole"), Deconstructing::Atomic)
        .declare(sample, id("whole"), Deconstructing::Atomic);
    let errors = builder.build(&model).expect_err("declared twice");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Duplicate { recipe, .. }] if recipe == &id("whole")),
        "{errors:?}"
    );
}

#[test]
fn a_recipe_naming_a_function_the_model_lacks_is_refused() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_new"))),
    );
    let errors = builder.build(&model).expect_err("no such function");
    assert!(
        matches!(errors.as_slice(), [RecipeError::UnknownFunction { func, .. }] if func == "sample_new"),
        "{errors:?}"
    );
}

#[test]
fn an_accessor_whose_first_parameter_is_another_type_is_refused() {
    let model = model(&[SAMPLE, "pub fn other_key(other: &u64) -> u32 {}"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
            "other_key",
        ))])),
    );
    let errors = builder.build(&model).expect_err("not an accessor");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAnAccessor { func, .. }] if func == "other_key"),
        "{errors:?}"
    );
}

#[test]
fn an_accessor_reached_through_a_borrow_is_accepted() {
    let model = model(&[SAMPLE, "pub fn sample_key(s: &Sample) -> u32 {}"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
            "sample_key",
        ))])),
    );
    builder.build(&model).expect("table");
}

#[test]
fn a_field_index_past_the_end_is_refused() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(7)])),
    );
    let errors = builder.build(&model).expect_err("no field 7");
    assert!(
        matches!(
            errors.as_slice(),
            [RecipeError::OutOfRange {
                index: 7,
                len: 2,
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn a_field_of_a_type_with_no_fields_is_refused() {
    let model = model(&["pub struct Handle;"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "u32"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
    );
    let errors = builder.build(&model).expect_err("a scalar has no fields");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAProduct { .. }]),
        "{errors:?}"
    );
}

#[test]
fn an_arms_payload_supplies_the_field_indices() {
    let model = model(&["pub enum Reply { Ok(u32), Err(u64) }"]);
    let reply = ty(&model, "Reply");
    let mut good = Recipes::builder();
    good.declare(
        reply.clone(),
        id("variants"),
        Deconstructing::Choice {
            arms: vec![
                Arm {
                    alternative: 0,
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
                Arm {
                    alternative: 1,
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
            ],
        },
    );
    good.build(&model).expect("both arms hold field 0");

    let mut past_the_end = Recipes::builder();
    past_the_end.declare(
        reply.clone(),
        id("variants"),
        Deconstructing::Choice {
            arms: vec![Arm {
                alternative: 0,
                op: Deconstruct::Fields(vec![Reach::Field(1)]),
            }],
        },
    );
    let errors = past_the_end
        .build(&model)
        .expect_err("arm 0 holds one field");
    assert!(
        matches!(
            errors.as_slice(),
            [RecipeError::OutOfRange {
                index: 1,
                len: 1,
                ..
            }]
        ),
        "{errors:?}"
    );

    let mut no_such_arm = Recipes::builder();
    no_such_arm.declare(
        reply,
        id("variants"),
        Deconstructing::Choice {
            arms: vec![Arm {
                alternative: 4,
                op: Deconstruct::Fields(vec![]),
            }],
        },
    );
    let errors = no_such_arm.build(&model).expect_err("no alternative 4");
    assert!(
        matches!(
            errors.as_slice(),
            [RecipeError::OutOfRange {
                index: 4,
                len: 2,
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn a_value_form_reads_its_parts_off_what_the_accessor_returns() {
    let model = model(&[
        "pub struct Handle;",
        SAMPLE,
        "pub fn handle_read(h: &Handle) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Handle"),
        id("read"),
        Deconstructing::Product(Deconstruct::ValueForm {
            func: ident("handle_read"),
            // `Sample`'s two fields, not `Handle`'s none.
            parts: vec![Reach::Field(0), Reach::Field(1)],
        }),
    );
    builder.build(&model).expect("table");
}

#[test]
fn a_callback_has_no_row_to_declare() {
    let model = model(&["pub fn listen(on: impl Fn(u32) + Send + Sync + 'static) {}"]);
    let listen = model.function("listen").expect("listen");
    let callback = listen.params[0].ty.clone();
    let mut builder = Recipes::builder();
    builder.declare(callback, id("whole"), Constructing::Atomic);
    let errors = builder.build(&model).expect_err("a callback has no row");
    assert!(
        matches!(errors.as_slice(), [RecipeError::CallbackDeclared { .. }]),
        "{errors:?}"
    );
}

#[test]
fn a_callback_derives_the_row_that_takes_it_apart() {
    let model = model(&["pub fn listen(on: impl Fn(u32) + Send + Sync + 'static) {}"]);
    let listen = model.function("listen").expect("listen");
    let table = Recipes::default();
    for assembly in [Assembly::Construct, Assembly::Deconstruct] {
        let crossing = Crossing::new(listen.params[0].ty.clone(), assembly);
        let (id, row) = table.row(&crossing);
        assert_eq!(id, RecipeId::derived());
        assert!(matches!(*row, Row::Callback(a) if a == assembly), "{row:?}");
        // The row's own job is the crossing's; its arguments do the other one.
        assert_eq!(row.assembly(), assembly);
    }
}

#[test]
fn a_row_reaching_its_own_crossing_is_refused() {
    let model = model(&[SAMPLE, "pub fn sample_clone(s: Sample) -> Sample {}"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("clone"),
        Constructing::Product(Construct::Call(ident("sample_clone"))),
    );
    let errors = builder.build(&model).expect_err("a cycle of one");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Cycle { path }] if path.len() == 2),
        "{errors:?}"
    );
}

#[test]
fn a_cycle_through_two_crossings_is_refused() {
    let model = model(&[
        "pub struct A { pub b: B }",
        "pub struct B { pub a: A }",
        "pub fn a_new(b: B) -> A {}",
        "pub fn b_new(a: A) -> B {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "A"),
            id("fields"),
            Constructing::Product(Construct::Call(ident("a_new"))),
        )
        .declare(
            ty(&model, "B"),
            id("fields"),
            Constructing::Product(Construct::Call(ident("b_new"))),
        );
    let errors = builder.build(&model).expect_err("A reaches A through B");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Cycle { .. }]),
        "{errors:?}"
    );
}

#[test]
fn a_row_that_only_reaches_a_different_job_is_not_a_cycle() {
    // `Sample` deconstructs through an accessor returning a `Sample`. The part
    // is the same type doing the same job, which is a cycle; the same recipe
    // filed under the other job is not, because nothing links the two.
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Constructing::Product(Construct::Call(ident("sample_new"))),
        )
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    builder.build(&model).expect("the two jobs do not meet");
}

#[test]
fn every_problem_is_reported_at_once() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![
                Reach::Field(9),
                Reach::Accessor(ident("nowhere")),
            ])),
        )
        .declare(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic);
    let errors = builder.build(&model).expect_err("three problems");
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors
        .iter()
        .any(|e| matches!(e, RecipeError::OutOfRange { .. })));
    assert!(errors
        .iter()
        .any(|e| matches!(e, RecipeError::UnknownFunction { .. })));
    assert!(errors
        .iter()
        .any(|e| matches!(e, RecipeError::NoDefault { .. })));
}

#[test]
fn the_two_jobs_swap_only_once() {
    assert_eq!(Assembly::Construct.swap(), Assembly::Deconstruct);
    assert_eq!(Assembly::Deconstruct.swap().swap(), Assembly::Deconstruct);
}

#[test]
fn a_part_is_accepted_where_the_edge_can_consume_it() {
    assert!(Mode::Owned.satisfies(Mode::Owned));
    assert!(Mode::Owned.satisfies(Mode::Shared));
    assert!(!Mode::Shared.satisfies(Mode::Owned));
    assert!(Mode::Shared.satisfies(Mode::Shared));
    assert!(!Mode::Shared.satisfies(Mode::Exclusive));
    assert!(Mode::Exclusive.satisfies(Mode::Exclusive));
    assert!(!Mode::Exclusive.satisfies(Mode::Shared));
}

// ── Sites and row selection ───────────────────────────────────────────────

fn site(owner: &str, index: usize) -> Site {
    Site {
        owner: ident(owner),
        role: Role::Param { index },
    }
}

/// A table where `Sample` has two deconstructing rows, `whole` being the
/// default — the shape every selection test below needs.
fn two_rows(model: &Flat) -> Recipes {
    let mut builder = Recipes::builder();
    builder
        .declare_default(ty(model, "Sample"), id("whole"), Deconstructing::Atomic)
        .declare(
            ty(model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    builder.build(model).expect("table")
}

#[test]
fn a_site_nobody_bound_takes_the_default_row() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("a site nobody bound still crosses");
    assert_eq!(bound.recipe, id("whole"));
    assert_eq!(bound.origin, Origin::Adapter);
    assert!(!bindings.is_declared(&site("z_put", 0)));
}

#[test]
fn an_undeclared_crossing_resolves_to_its_derived_row() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Vec<Sample>"), Assembly::Deconstruct);

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("the derived row");
    assert_eq!(bound.recipe, RecipeId::derived());
}

#[test]
fn a_site_takes_the_row_it_names() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing.clone(),
        Ask::Recipe(id("fields")),
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("bound");
    assert_eq!(bound.recipe, id("fields"));
    assert_eq!(bound.origin, Origin::Function);
    // Every other site of the same crossing still takes the default.
    let other = bindings
        .resolve(&site("z_get", 0), &crossing, &recipes)
        .expect("bound");
    assert_eq!(other.recipe, id("whole"));
}

#[test]
fn the_higher_precedence_declaration_wins_whichever_was_written_first() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);

    for order in [
        [Origin::Type, Origin::Function],
        [Origin::Function, Origin::Type],
    ] {
        let mut builder = Bindings::builder();
        for origin in order {
            let ask = match origin {
                Origin::Function => Ask::Recipe(id("fields")),
                _ => Ask::Recipe(id("whole")),
            };
            builder.bind(site("z_put", 0), crossing.clone(), ask, origin);
        }
        let bindings = builder.build(&recipes).expect("bindings");
        let bound = bindings
            .resolve(&site("z_put", 0), &crossing, &recipes)
            .expect("bound");
        assert_eq!(bound.recipe, id("fields"), "written {order:?}");
        assert_eq!(bound.origin, Origin::Function, "written {order:?}");
    }
}

#[test]
fn two_declarations_of_equal_precedence_may_agree_and_may_not_disagree() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);

    let mut agreeing = Bindings::builder();
    agreeing
        .bind(
            site("z_put", 0),
            crossing.clone(),
            Ask::Recipe(id("fields")),
            Origin::Function,
        )
        .bind(
            site("z_put", 0),
            crossing.clone(),
            Ask::Recipe(id("fields")),
            Origin::Function,
        );
    agreeing
        .build(&recipes)
        .expect("saying it twice is not a conflict");

    let mut disagreeing = Bindings::builder();
    disagreeing
        .bind(
            site("z_put", 0),
            crossing.clone(),
            Ask::Recipe(id("fields")),
            Origin::Function,
        )
        .bind(
            site("z_put", 0),
            crossing,
            Ask::Recipe(id("whole")),
            Origin::Function,
        );
    let errors = disagreeing.build(&recipes).expect_err("two answers");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Rebound { origin, .. }] if *origin == Origin::Function),
        "{errors:?}"
    );
}

#[test]
fn two_declarations_of_equal_precedence_naming_different_crossings_disagree() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let mut builder = Bindings::builder();
    builder
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
            Ask::Default,
            Origin::Function,
        )
        // Same ask, different type: still two answers for one place.
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "u32"), Assembly::Deconstruct),
            Ask::Default,
            Origin::Function,
        )
        // A third disagreement over the same site is still one report.
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "u64"), Assembly::Deconstruct),
            Ask::Default,
            Origin::Function,
        );
    let errors = builder.build(&recipes).expect_err("two crossings");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Rebound { .. }]),
        "{errors:?}"
    );
}

#[test]
fn a_site_naming_a_row_the_crossing_lacks_is_refused() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing,
        Ask::Recipe(id("jobject")),
        Origin::Function,
    );
    let errors = builder.build(&recipes).expect_err("no such row");
    assert!(
        matches!(errors.as_slice(), [RecipeError::UnknownRow { recipe, .. }] if recipe == &id("jobject")),
        "{errors:?}"
    );
}

#[test]
fn an_omitted_site_contributes_nothing() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(site("z_put", 0), crossing.clone(), Ask::Omit, Origin::Part);
    let bindings = builder.build(&recipes).expect("bindings");

    assert!(bindings.is_declared(&site("z_put", 0)));
    assert!(bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .is_none());
}

#[test]
fn asking_for_the_default_records_which_row_that_was() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing.clone(),
        Ask::Default,
        Origin::Type,
    );
    let bindings = builder.build(&recipes).expect("bindings");

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("bound");
    assert_eq!(bound.recipe, id("whole"));
    assert_eq!(bound.origin, Origin::Type);
}

#[test]
fn one_site_is_one_role_of_one_owner() {
    let model = model(&[SAMPLE]);
    let recipes = two_rows(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct);
    let ret = Site {
        owner: ident("z_put"),
        role: Role::Return,
    };
    let mut builder = Bindings::builder();
    builder.bind(
        ret.clone(),
        crossing.clone(),
        Ask::Recipe(id("fields")),
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");

    assert_eq!(
        bindings.resolve(&ret, &crossing, &recipes).unwrap().recipe,
        id("fields")
    );
    // The same function's parameter 0 is a different site.
    assert_eq!(
        bindings
            .resolve(&site("z_put", 0), &crossing, &recipes)
            .unwrap()
            .recipe,
        id("whole")
    );
}

// ── Compiling rows into fragments ─────────────────────────────────────────

use std::collections::{HashMap, HashSet};

use crate::flat::Alternative;

/// A fragment that records the shape it was built from, so a test can assert on
/// the tree an adapter is handed rather than on generated code.
#[derive(Clone, Debug)]
struct Note {
    text: String,
    yields: Yield,
}

impl Carrier for Note {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

/// The smallest adapter that exercises every hook: it emits nothing and only
/// writes down what it was asked.
#[derive(Default)]
struct Recorder {
    /// One line per hook call, in call order.
    calls: Vec<String>,
    /// Crossings whose fragment is reached through a borrow.
    shared: HashSet<String>,
    /// Crossings whose fragment is valid only while its source is alive.
    borrowed: HashSet<String>,
    /// Crossings whose fragment claims to produce a different Rust type — an
    /// adapter answering for the wrong value, which nothing in the shape of a
    /// recipe can prevent.
    mistyped: HashMap<String, TypeKey>,
}

impl Recorder {
    fn note(&mut self, at: At<'_>, text: String) -> Note {
        self.calls.push(text.clone());
        let name = at.crossing.value().stripped_key().to_string();
        Note {
            text,
            yields: Yield {
                ty: self
                    .mistyped
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| at.crossing.value().stripped_key()),
                mode: if self.shared.contains(&name) {
                    Mode::Shared
                } else {
                    Mode::Owned
                },
                validity: if self.borrowed.contains(&name) {
                    Validity::Borrowed
                } else {
                    Validity::SelfSufficient
                },
            },
        }
    }

    fn hook(&mut self, at: At<'_>, hook: &str, detail: String) -> Result<Note, String> {
        let ty = at.crossing.value().stripped_key();
        Ok(self.note(
            at,
            format!("{hook} {ty} {}: {detail}", at.crossing.assembly()),
        ))
    }
}

fn part_names<C: Compile>(parts: Parts<'_, C>) -> String {
    parts
        .iter()
        .map(|(p, _)| {
            let source = match p.from {
                PartSource::Argument { index } => format!("arg{index}"),
                PartSource::Field { index, .. } => format!("field{index}"),
                PartSource::Accessor { func } => format!("via {}", func.name),
            };
            format!("{}={source}/{}", p.name, p.mode)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

impl Compile for Recorder {
    type Fragment = Note;
    type Plan = String;
    type Error = String;

    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        // A fragment is generated Rust, so a hook is an emission callback and
        // can spell a model type without naming the flat protocol itself.
        let spelled = cx.emit().spell(at.crossing.spelled()).to_string();
        self.hook(at, "atomic", spelled)
    }

    fn optional(&mut self, _cx: &mut Cx<'_>, at: At<'_>, inner: &Note) -> Frag<Self> {
        let detail = inner.text.clone();
        self.hook(at, "optional", detail)
    }

    fn sequence(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        elements: Mode,
        inner: &Note,
    ) -> Frag<Self> {
        let detail = format!("elements {elements} of {}", inner.text);
        self.hook(at, "sequence", detail)
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        args: Parts<'_, Self>,
    ) -> Frag<Self> {
        let detail = format!("{}({})", func.name, part_names::<Self>(args));
        self.hook(at, "construct", detail)
    }

    fn identity(&mut self, _cx: &mut Cx<'_>, at: At<'_>, inner: &Note) -> Frag<Self> {
        let detail = inner.text.clone();
        self.hook(at, "identity", detail)
    }

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        let detail = part_names::<Self>(parts);
        self.hook(at, "fields", detail)
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        let detail = format!("{} -> {}", func.name, part_names::<Self>(parts));
        self.hook(at, "value_form", detail)
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &Note)],
    ) -> Frag<Self> {
        let detail = arms
            .iter()
            .map(|(a, f)| format!("{}#{} [{}]", a.name, a.index, f.text))
            .collect::<Vec<_>>()
            .join(" | ");
        self.hook(at, "choice", detail)
    }

    fn callback(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        args: &[&Note],
        result: Option<&Note>,
    ) -> Frag<Self> {
        cx.require(RequirementId::new("callback-shim"));
        let detail = format!(
            "({}) -> {:?}",
            args.iter()
                .map(|a| a.text.clone())
                .collect::<Vec<_>>()
                .join(", "),
            result.map(|r| r.text.clone())
        );
        self.hook(at, "callback", detail)
    }

    fn plan(&mut self, _cx: &mut Cx<'_>, bound: &Bound, root: &Note) -> Result<String, String> {
        Ok(format!(
            "{} <- {} [{}]",
            bound.site, bound.recipe, root.text
        ))
    }
}

/// The registry's half of a compile failure, or a panic naming the adapter's.
fn recipe_error(error: &CompileError<String>) -> &RecipeError {
    match error {
        CompileError::Recipe(e) => e,
        CompileError::Adapter(a) => panic!("the adapter refused: {a}"),
    }
}

fn compile_one(
    model: &Flat,
    recipes: &Recipes,
    adapter: &mut Recorder,
    site: Site,
    spelling: &str,
    assembly: Assembly,
) -> String {
    let bindings = Bindings::default();
    let mut compiler = Compiler::new(model, recipes, &bindings);
    compiler
        .site(adapter, site, Crossing::new(ty(model, spelling), assembly))
        .expect("compile")
        .expect("not omitted")
}

#[test]
fn a_constructors_parameters_are_the_parts() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_new"))),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        site("z_put", 0),
        "Sample",
        Assembly::Construct,
    );
    assert!(
        plan.contains("sample_new(key=arg0/owned, payload=arg1/owned)"),
        "{plan}"
    );
    // Each part is a crossing of its own, compiled before the whole.
    assert_eq!(
        adapter.calls,
        vec![
            "atomic u32 construct: u32",
            "atomic u64 construct: u64",
            "construct Sample construct: sample_new(key=arg0/owned, payload=arg1/owned)",
        ]
    );
}

#[test]
fn an_accessors_return_is_the_parts_type_and_its_borrowing() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_key(s: &Sample) -> u32 {}",
        "pub fn sample_payload(s: &Sample) -> &u64 {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![
            Reach::Accessor(ident("sample_key")),
            Reach::Accessor(ident("sample_payload")),
        ])),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Sample",
        Assembly::Deconstruct,
    );
    assert!(
        plan.contains("sample_key=via sample_key/owned, sample_payload=via sample_payload/&"),
        "{plan}"
    );
}

#[test]
fn a_field_reach_reads_the_models_own_field() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(1), Reach::Field(0)])),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Sample",
        Assembly::Deconstruct,
    );
    // Declaration order is the recipe's, not the struct's.
    assert!(
        plan.contains("payload=field1/owned, key=field0/owned"),
        "{plan}"
    );
}

#[test]
fn an_omitted_reach_contributes_no_part() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Omit])),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Sample",
        Assembly::Deconstruct,
    );
    assert!(
        plan.contains("fields Sample deconstruct: key=field0/owned"),
        "{plan}"
    );
}

#[test]
fn one_row_answering_three_spellings_still_builds_three_fragments() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic);
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    for spelling in ["Sample", "&Sample", "Box<Sample>"] {
        let crossing = Crossing::new(ty(&model, spelling), Assembly::Deconstruct);
        // All three find the one declared row …
        assert_eq!(recipes.row(&crossing).0, id("whole"));
        compiler.crossing(&mut adapter, &crossing).expect("compile");
    }
    // … and each still gets its own Rust, because taking a value out of a
    // pointer, borrowing through one and rebuilding a Box are three things.
    assert_eq!(compiler.compiled_fragments(), 3);
    assert_eq!(
        adapter.calls,
        vec![
            "atomic Sample deconstruct: Sample",
            "atomic Sample deconstruct: & Sample",
            "atomic Sample deconstruct: Box < Sample >",
        ]
    );
}

#[test]
fn a_crossing_can_be_compiled_without_a_site() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let first = compiler
        .crossing(
            &mut adapter,
            &Crossing::new(ty(&model, "Option<Sample>"), Assembly::Deconstruct),
        )
        .expect("compile");
    assert!(first.text.starts_with("optional"), "{}", first.text);
    // Asking twice is the same fragment, not a second compilation.
    compiler
        .crossing(
            &mut adapter,
            &Crossing::new(ty(&model, "Option<Sample>"), Assembly::Deconstruct),
        )
        .expect("compile");
    assert_eq!(adapter.calls.len(), 2);
}

#[test]
fn a_row_is_compiled_once_however_many_sites_take_it() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_new"))),
    );
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    for owner in ["z_put", "z_get", "z_reply"] {
        compiler
            .site(
                &mut adapter,
                site(owner, 0),
                Crossing::new(ty(&model, "Sample"), Assembly::Construct),
            )
            .expect("compile");
    }
    // Three sites, three plans — and one fragment per crossing: Sample, u32,
    // u64.
    assert_eq!(compiler.compiled_fragments(), 3);
    assert_eq!(adapter.calls.len(), 3);
}

#[test]
fn a_sequence_reads_its_element_mode_off_the_collection() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let modes = |spelling: &str| {
        let mut adapter = Recorder::default();
        compile_one(
            &model,
            &recipes,
            &mut adapter,
            Site {
                owner: ident("z_get"),
                role: Role::Return,
            },
            spelling,
            Assembly::Deconstruct,
        )
    };
    assert!(
        modes("Vec<u32>").contains("elements owned"),
        "{}",
        modes("Vec<u32>")
    );
    assert!(modes("&[u32]").contains("elements &"));
    assert!(modes("&Vec<u32>").contains("elements &"));
    assert!(modes("[u32; 4]").contains("elements owned"));
}

#[test]
fn a_nested_layer_is_a_row_of_its_own() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let mut adapter = Recorder::default();
    compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Option<Vec<u32>>",
        Assembly::Deconstruct,
    );
    assert_eq!(adapter.calls.len(), 3, "{:?}", adapter.calls);
    assert!(adapter.calls[0].starts_with("atomic u32"));
    assert!(adapter.calls[1].starts_with("sequence"));
    assert!(adapter.calls[2].starts_with("optional"));
}

#[test]
fn every_arm_of_a_choice_reaches_the_hook_already_composed() {
    let model = model(&["pub enum Reply { Ok(u32), Err(u64) }"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Reply"),
        id("variants"),
        Deconstructing::Choice {
            arms: vec![
                Arm {
                    alternative: 0,
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
                Arm {
                    alternative: 1,
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
            ],
        },
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Reply",
        Assembly::Deconstruct,
    );
    assert!(plan.contains("Ok#0 ["), "{plan}");
    assert!(plan.contains("Err#1 ["), "{plan}");
    // Each arm's payload supplies its own field 0, so the two differ.
    assert!(plan.contains("0=field0/owned"), "{plan}");
    assert_eq!(
        adapter
            .calls
            .iter()
            .filter(|c| c.starts_with("choice"))
            .count(),
        1
    );
}

#[test]
fn a_callbacks_arguments_do_the_other_job() {
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(Sample) + Send + Sync + 'static) {}",
    ]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let listen = model.function("listen").expect("listen");

    compiler
        .site(
            &mut adapter,
            site("listen", 0),
            Crossing::new(listen.params[0].ty.clone(), Assembly::Construct),
        )
        .expect("compile");

    // The callback itself is constructed; the `Sample` it carries is
    // deconstructed, because Rust holds it and pushes it out through the call.
    assert!(
        adapter.calls[0].starts_with("atomic Sample deconstruct"),
        "{:?}",
        adapter.calls
    );
    assert!(adapter.calls[1].contains("callback"), "{:?}", adapter.calls);
    assert_eq!(
        compiler
            .required()
            .map(|r| r.to_string())
            .collect::<Vec<_>>(),
        vec!["callback-shim".to_string()]
    );
}

#[test]
fn a_callback_argument_is_a_part_of_the_callback_row_that_names_it() {
    // The argument does the *other* job — Rust holds the value and pushes it
    // out — but the part still belongs to the callback row, so a binding
    // written against that row applies. Keying the site by the swapped
    // crossing instead made every such binding silently miss.
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(Sample) + Send + Sync + 'static) {}",
    ]);
    let listen = model.function("listen").expect("listen");
    let callback = listen.params[0].ty.clone();
    let mut builder = Recipes::builder();
    builder
        .declare_default(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic)
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let recipes = builder.build(&model).expect("table");

    // The row this part belongs to: the callback, constructed.
    let row = Crossing::new(callback.clone(), Assembly::Construct);
    // Built the same way the driver builds it, which is the point of the
    // helper: a per-part binding is found by this exact key or not at all.
    let part = Site::part(&row, &RecipeId::derived(), 0);
    let mut bound = Bindings::builder();
    bound.bind(
        part,
        // The part's own crossing carries the swap; the site does not.
        Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        Ask::Recipe(id("fields")),
        Origin::Part,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    compiler
        .site(&mut adapter, site("listen", 0), row)
        .expect("compile");
    assert!(
        adapter
            .calls
            .iter()
            .any(|c| c.starts_with("fields Sample deconstruct")),
        "the per-part binding did not reach the callback argument: {:?}",
        adapter.calls
    );
}

#[test]
fn a_callback_argument_is_overridden_by_compiling_it_as_its_own_site() {
    // A callback row is shared by every function whose callback has the same
    // signature, so a per-function answer cannot apply to it. `Role::CallbackArg`
    // is a root role: an adapter compiles that position itself.
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(Sample) + Send + Sync + 'static) {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare_default(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic)
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let recipes = builder.build(&model).expect("table");

    let arg = Site {
        owner: ident("listen"),
        role: Role::CallbackArg { param: 0, arg: 0 },
    };
    let mut bound = Bindings::builder();
    bound.bind(
        arg.clone(),
        Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        Ask::Recipe(id("fields")),
        Origin::Function,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let plan = compiler
        .site(
            &mut adapter,
            arg,
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        )
        .expect("compile")
        .expect("not omitted");
    assert!(plan.contains("fields Sample deconstruct"), "{plan}");
}

#[test]
fn a_part_that_only_lends_cannot_feed_an_edge_that_consumes() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_new"))),
    );
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    adapter.shared.insert("u64".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Construct),
        )
        .expect_err("payload is taken by value");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::Composition {
                part: 1,
                wanted: Mode::Owned,
                got: Mode::Shared,
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn a_part_producing_the_wrong_rust_type_is_refused() {
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_new"))),
    );
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    // `payload` is a `u64`, and the adapter's fragment answers with a `u32`.
    // Nothing about the recipe's shape can catch that: the recipe names the
    // constructor and the model supplies the part types, so the only place the
    // two can disagree is what the adapter says it produces.
    adapter
        .mistyped
        .insert("u64".to_owned(), ty(&model, "u32").stripped_key());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Construct),
        )
        .expect_err("the part produces the wrong type");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::ComposedType { part: 1, wanted, got, .. }
                if wanted.as_str() == "u64" && got.as_str() == "u32"
        ),
        "{error:?}"
    );
}

/// Compile one crossing with `mistyped` in force, and hand back the refusal.
fn mistyped_refusal(
    model: &Flat,
    spelling: &str,
    lie_about: &str,
    lie: &str,
) -> CompileError<String> {
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    adapter
        .mistyped
        .insert(lie_about.to_owned(), ty(model, lie).stripped_key());
    let mut compiler = Compiler::new(model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(model, spelling), Assembly::Construct),
        )
        .expect_err("the inner fragment produces the wrong type")
}

#[test]
fn an_optionals_value_is_a_part_and_is_checked_like_one() {
    let model = model(&[SAMPLE]);
    let error = mistyped_refusal(&model, "Option<u64>", "u64", "u32");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::ComposedType { wanted, got, .. }
                if wanted.as_str() == "u64" && got.as_str() == "u32"
        ),
        "{error:?}"
    );
}

#[test]
fn a_runs_element_is_a_part_and_is_checked_like_one() {
    let model = model(&[SAMPLE]);
    for spelling in ["Vec<u64>", "&[u64]", "[u64; 4]"] {
        let error = mistyped_refusal(&model, spelling, "u64", "u32");
        assert!(
            matches!(
                recipe_error(&error),
                RecipeError::ComposedType { wanted, got, .. }
                    if wanted.as_str() == "u64" && got.as_str() == "u32"
            ),
            "{spelling}: {error:?}"
        );
    }
}

#[test]
fn a_callback_argument_is_a_part_and_is_checked_like_one() {
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(u64) + Send + Sync + 'static) {}",
    ]);
    let listen = model.function("listen").expect("listen");
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    adapter
        .mistyped
        .insert("u64".to_owned(), ty(&model, "u32").stripped_key());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let error = compiler
        .site(
            &mut adapter,
            site("listen", 0),
            Crossing::new(listen.params[0].ty.clone(), Assembly::Construct),
        )
        .expect_err("the argument's fragment produces the wrong type");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::ComposedType { wanted, got, .. }
                if wanted.as_str() == "u64" && got.as_str() == "u32"
        ),
        "{error:?}"
    );
}

#[test]
fn a_runs_element_must_be_held_the_way_the_collection_lends_it() {
    // A `Vec<T>` gives its elements up, so an element fragment that only lends
    // cannot serve one — the mode half of the contract, on a non-product edge.
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    adapter.shared.insert("u64".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Vec<u64>"), Assembly::Construct),
        )
        .expect_err("a Vec hands its elements over");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::Composition {
                wanted: Mode::Owned,
                got: Mode::Shared,
                ..
            }
        ),
        "{error:?}"
    );

    // `&[T]` lends them, so the same fragment is fine there.
    let mut adapter = Recorder::default();
    adapter.shared.insert("u64".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&[u64]"), Assembly::Construct),
        )
        .expect("a slice lends its elements");
}

#[test]
fn a_part_answering_through_a_borrow_or_a_box_still_matches_its_type() {
    // The type check normalizes the way a crossing is keyed, so a fragment
    // yielding `T` answers a part spelled `&T` — whether it may be *held* that
    // way is the mode check, which is separate and runs second.
    let model = model(&[
        SAMPLE,
        "pub fn sample_of(key: &u32, payload: Box<u64>) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_of"))),
    );
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Construct),
        )
        .expect("a borrow and a Box are answered by the bare type's fragment");
}

#[test]
fn a_constructor_that_builds_another_type_is_refused() {
    let model = model(&[
        SAMPLE,
        "pub struct Other { pub n: u32 }",
        "pub fn make_other(key: u32) -> Other {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("make_other"))),
    );
    let errors = builder
        .build(&model)
        .expect_err("make_other builds an Other");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAConstructor { func, .. }] if func == "make_other"),
        "{errors:?}"
    );
}

#[test]
fn a_fallible_constructor_builds_its_success_arm() {
    // Where a construction's fallibility is read from is the return type, so
    // `Result<Sample, E>` builds a `Sample` and nothing states it twice.
    let model = model(&[
        SAMPLE,
        "pub struct Error { pub code: u32 }",
        "pub fn sample_try(key: u32) -> Result<Sample, Error> {}",
        "pub fn other_try(key: u32) -> Result<u32, Error> {}",
    ]);
    let mut good = Recipes::builder();
    good.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_try"))),
    );
    good.build(&model)
        .expect("Result<Sample, _> builds a Sample");

    let mut bad = Recipes::builder();
    bad.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("other_try"))),
    );
    let errors = bad.build(&model).expect_err("Result<u32, _> does not");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAConstructor { .. }]),
        "{errors:?}"
    );
}

#[test]
fn a_constructor_reached_through_a_borrow_or_a_box_is_accepted() {
    let model = model(&[SAMPLE, "pub fn sample_boxed(key: u32) -> Box<Sample> {}"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("fields"),
        Constructing::Product(Construct::Call(ident("sample_boxed"))),
    );
    builder
        .build(&model)
        .expect("a Box<Sample> builds a Sample");
}

#[test]
fn a_returned_value_the_foreign_side_keeps_cannot_be_borrowed() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    adapter.borrowed.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let ret = Site {
        owner: ident("z_get"),
        role: Role::Return,
    };
    let error = compiler
        .site(
            &mut adapter,
            ret,
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        )
        .expect_err("a return outlives the call");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::Validity {
                needed: Validity::SelfSufficient,
                got: Validity::Borrowed,
                ..
            }
        ),
        "{error:?}"
    );

    // The same borrowed fragment is fine where the value lives only for the
    // call.
    let mut adapter = Recorder::default();
    adapter.borrowed.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        )
        .expect("a parameter tolerates a borrow");
}

#[test]
fn an_omitted_site_compiles_to_no_plan() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        Crossing::new(ty(&model, "Sample"), Assembly::Construct),
        Ask::Omit,
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    assert!(compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Construct),
        )
        .expect("compile")
        .is_none());
    assert!(adapter.calls.is_empty());
}

#[test]
fn a_site_takes_the_row_the_binding_names_and_others_take_the_default() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder
        .declare_default(ty(&model, "Sample"), id("whole"), Deconstructing::Atomic)
        .declare(
            ty(&model, "Sample"),
            id("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    let recipes = builder.build(&model).expect("table");
    let mut bound = Bindings::builder();
    bound.bind(
        site("z_put", 0),
        Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        Ask::Recipe(id("fields")),
        Origin::Function,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let overridden = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        )
        .expect("compile")
        .expect("not omitted");
    let plain = compiler
        .site(
            &mut adapter,
            site("z_get", 0),
            Crossing::new(ty(&model, "Sample"), Assembly::Deconstruct),
        )
        .expect("compile")
        .expect("not omitted");

    assert!(overridden.contains("fields Sample"), "{overridden}");
    assert!(plain.contains("atomic Sample"), "{plain}");
    // Two rows of one crossing, so two fragments — plus the two field types.
    assert_eq!(compiler.compiled_fragments(), 4);
}

#[test]
fn a_struct_with_no_constructor_is_built_from_its_own_fields() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        id("literal"),
        Constructing::Product(Construct::Fields),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        site("z_put", 0),
        "Sample",
        Assembly::Construct,
    );
    // Every field contributes, in the model's order, and the same `fields` hook
    // serves both jobs.
    assert!(
        plan.contains("fields Sample construct: key=field0/owned, payload=field1/owned"),
        "{plan}"
    );
}

#[test]
fn an_arm_is_built_from_its_own_payload_fields() {
    let model = model(&["pub enum Reply { Ok(u32), Err(u64) }"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Reply"),
        id("variants"),
        Constructing::Choice {
            arms: vec![
                Arm {
                    alternative: 0,
                    op: Construct::Fields,
                },
                Arm {
                    alternative: 1,
                    op: Construct::Fields,
                },
            ],
        },
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        site("z_put", 0),
        "Reply",
        Assembly::Construct,
    );
    assert!(
        plan.contains("Ok#0 [fields Reply construct: 0=field0/owned]"),
        "{plan}"
    );
    assert!(
        plan.contains("Err#1 [fields Reply construct: 0=field0/owned]"),
        "{plan}"
    );
    // The two arms' payloads are different types, so they are different rows.
    assert!(adapter.calls.iter().any(|c| c.starts_with("atomic u32")));
    assert!(adapter.calls.iter().any(|c| c.starts_with("atomic u64")));
}

#[test]
fn building_a_type_the_model_gives_no_fields_is_refused() {
    let model = model(&["pub struct Handle;"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "u32"),
        id("literal"),
        Constructing::Product(Construct::Fields),
    );
    let errors = builder.build(&model).expect_err("a scalar has no fields");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAProduct { .. }]),
        "{errors:?}"
    );
}

#[test]
fn a_value_form_binds_the_accessors_result_and_reads_its_fields() {
    let model = model(&[
        "pub struct Handle;",
        SAMPLE,
        "pub fn handle_read(h: &Handle) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Handle"),
        id("read"),
        Deconstructing::Product(Deconstruct::ValueForm {
            func: ident("handle_read"),
            parts: vec![Reach::Field(0), Reach::Field(1)],
        }),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Handle",
        Assembly::Deconstruct,
    );
    assert!(
        plan.contains("handle_read -> key=field0/owned, payload=field1/owned"),
        "{plan}"
    );
}
