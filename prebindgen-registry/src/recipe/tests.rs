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
