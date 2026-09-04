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

fn recipe_name(value: &str) -> RecipeName {
    RecipeName::new(value)
}

fn row(crossing: &Crossing, name: &str) -> RecipeKey {
    crossing.row(recipe_name(name))
}

fn row_at(crossing: &CrossingKey, name: &str) -> RecipeKey {
    crossing.row(recipe_name(name))
}

fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

/// The struct every fixture that needs parts is built over.
const SAMPLE: &str = "pub struct Sample { pub key: u32, pub payload: u64 }";

fn shape_of(recipe: &Recipe) -> &str {
    match recipe {
        Recipe::Constructing(s) => name_of(s),
        Recipe::Deconstructing(s) => name_of(s),
    }
}

fn name_of<OP>(shape: &Shape<OP>) -> &'static str {
    match shape {
        Shape::Atomic => "atomic",
        Shape::Optional => "optional",
        Shape::Sequence => "sequence",
        Shape::Invoke => "invoke",
        Shape::Product(_) => "product",
        Shape::Choice { .. } => "choice",
    }
}

fn derived_shape(model: &Flat, spelling: &str) -> String {
    let table = Recipes::default();
    let crossing = Crossing::new(ty(model, spelling), Direction::Deconstruct);
    let (key, recipe) = table.recipe(&crossing);
    format!("{}:{}", key.name(), shape_of(&recipe))
}

#[test]
fn an_undeclared_crossing_gets_its_arity_row_from_the_kind() {
    let model = model(&[SAMPLE]);
    assert_eq!(derived_shape(&model, "Sample"), "derived:atomic");
    assert_eq!(derived_shape(&model, "Option<Sample>"), "derived:optional");
    assert_eq!(derived_shape(&model, "Vec<Sample>"), "derived:sequence");
    assert_eq!(derived_shape(&model, "&[Sample]"), "derived:sequence");
    assert_eq!(derived_shape(&model, "[u8; 4]"), "derived:sequence");
    // One layer per recipe: the inner `Option` is the inner crossing's recipe, not
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
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("whole"),
        Deconstructing::Atomic,
    );
    let table = builder.build(&model).expect("table");

    for spelling in ["Sample", "&Sample", "&mut Sample", "Box<Sample>"] {
        let crossing = Crossing::new(ty(&model, spelling), Direction::Deconstruct);
        assert_eq!(
            table.recipe(&crossing).0,
            row(&crossing, "whole"),
            "{spelling} did not find the declared recipe"
        );
    }
}

#[test]
fn a_crossing_reports_the_mode_the_site_spelled() {
    let model = model(&[SAMPLE]);
    let mode = |spelling: &str| Crossing::new(ty(&model, spelling), Direction::Construct).mode();
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
            recipe_name("fields"),
            Constructing::Product(Construct::Call(ident("sample_new"))),
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    let table = builder.build(&model).expect("table");

    let sample = ty(&model, "Sample");
    for direction in [Direction::Construct, Direction::Deconstruct] {
        let key = Crossing::new(sample.clone(), direction).key();
        assert_eq!(key.direction, direction);
        assert_eq!(table.names_of(&key), vec![&recipe_name("fields")]);
    }
}

#[test]
fn one_row_is_the_default_and_several_have_to_say_which() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let key = Crossing::new(sample.clone(), Direction::Deconstruct).key();

    let mut one = Recipes::builder();
    one.declare(sample.clone(), recipe_name("whole"), Deconstructing::Atomic);
    let table = one.build(&model).expect("one recipe is its own default");
    assert_eq!(table.default_of(&key), Some(&row_at(&key, "whole")));

    let mut undecided = Recipes::builder();
    undecided
        .declare(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(
            sample.clone(),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let errors = undecided.build(&model).expect_err("no default");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NoDefault { defaults, .. }] if defaults.is_empty()),
        "{errors:?}"
    );

    let mut decided = Recipes::builder();
    decided
        .declare_default(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(
            sample,
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let table = decided.build(&model).expect("one default");
    assert_eq!(table.default_of(&key), Some(&row_at(&key, "whole")));
    assert_eq!(table.names_of(&key).len(), 2);
}

#[test]
fn one_name_can_label_several_globally_distinct_rows() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let scalar = ty(&model, "u32");
    let mut builder = Recipes::builder();
    builder
        .declare(sample.clone(), recipe_name("whole"), Constructing::Atomic)
        .declare(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(scalar, recipe_name("whole"), Deconstructing::Atomic);
    let table = builder.build(&model).expect("table");

    let sample_in = Crossing::new(sample.clone(), Direction::Construct).key();
    let sample_out = Crossing::new(sample, Direction::Deconstruct).key();
    let scalar_out = Crossing::new(ty(&model, "u32"), Direction::Deconstruct).key();
    let sample_in = table.key_of(&sample_in, &recipe_name("whole")).unwrap();
    let sample_out = table.key_of(&sample_out, &recipe_name("whole")).unwrap();
    let scalar_out = table.key_of(&scalar_out, &recipe_name("whole")).unwrap();

    assert_eq!(sample_in.name(), sample_out.name());
    assert_eq!(sample_out.name(), scalar_out.name());
    assert_ne!(sample_in, sample_out);
    assert_ne!(sample_out, scalar_out);
    assert_ne!(sample_in, scalar_out);
    assert!(table.get(sample_in).is_some());
    assert!(table.get(sample_out).is_some());
    assert!(table.get(scalar_out).is_some());
}

#[test]
fn two_defaults_are_as_wrong_as_none() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let mut builder = Recipes::builder();
    builder
        .declare_default(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare_default(
            sample,
            recipe_name("fields"),
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
    let duplicate = Crossing::new(sample.clone(), Direction::Deconstruct).row(recipe_name("whole"));
    let mut builder = Recipes::builder();
    builder
        .declare(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(sample, recipe_name("whole"), Deconstructing::Atomic);
    let errors = builder.build(&model).expect_err("declared twice");
    assert!(
        matches!(errors.as_slice(), [RecipeError::Duplicate { recipe, .. }] if recipe == &duplicate),
        "{errors:?}"
    );
}

#[test]
fn a_recipe_naming_a_function_the_model_lacks_is_refused() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("fields"),
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
        recipe_name("fields"),
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
        recipe_name("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Accessor(ident(
            "sample_key",
        ))])),
    );
    builder.build(&model).expect("table");
}

#[test]
fn an_identity_reach_needs_no_accessor_and_yields_the_value_itself() {
    // The form `DeconRecord::Identity` states: a handle leaf is the whole
    // value, with no field to index and no accessor to call. A table that
    // could not spell it is why #622's callback-argument rows are placeholders
    // (#613 step 10).
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("handle"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity])),
    );
    builder.build(&model).expect("an identity reach validates");
}

#[test]
fn a_field_index_past_the_end_is_refused() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("fields"),
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
        recipe_name("fields"),
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
        recipe_name("variants"),
        Deconstructing::Choice {
            arms: vec![
                Arm {
                    alternative: Some(0),
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
                Arm {
                    alternative: Some(1),
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
            ],
        },
    );
    good.build(&model).expect("both arms hold field 0");

    let mut past_the_end = Recipes::builder();
    past_the_end.declare(
        reply.clone(),
        recipe_name("variants"),
        Deconstructing::Choice {
            arms: vec![Arm {
                alternative: Some(0),
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
        recipe_name("variants"),
        Deconstructing::Choice {
            arms: vec![Arm {
                alternative: Some(4),
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
        recipe_name("read"),
        Deconstructing::Product(Deconstruct::ValueForm {
            func: ident("handle_read"),
            // `Sample`'s two fields, not `Handle`'s none.
            parts: vec![Reach::Field(0), Reach::Field(1)],
        }),
    );
    builder.build(&model).expect("table");
}

#[test]
fn a_callback_takes_invoke_and_nothing_else() {
    let model = model(&["pub fn listen(on: impl Fn(u32) + Send + Sync + 'static) {}"]);
    let listen = model.function("listen").expect("listen");
    let callback = listen.params[0].ty.clone();

    // Any other shape is refused: converting the arguments is what makes the
    // callable callable, so there is no second answer to choose between.
    let mut builder = Recipes::builder();
    builder.declare(callback.clone(), recipe_name("whole"), Constructing::Atomic);
    let errors = builder
        .build(&model)
        .expect_err("only `Invoke` fits a callback");
    assert!(
        matches!(errors.as_slice(), [RecipeError::CallbackShape { .. }]),
        "{errors:?}"
    );

    // `Invoke` is declarable like any other shape, and states the same thing
    // the table would have derived.
    let mut builder = Recipes::builder();
    builder.declare(callback, recipe_name("invoke"), Constructing::Invoke);
    builder
        .build(&model)
        .expect("`Invoke` is a callback's shape");
}

#[test]
fn invoke_on_a_type_that_is_not_a_callback_is_refused() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    let mut builder = Recipes::builder();
    builder.declare(sample, recipe_name("invoke"), Constructing::Invoke);
    let errors = builder.build(&model).expect_err("`Sample` is not callable");
    assert!(
        matches!(
            errors.as_slice(),
            [RecipeError::WrongShape {
                shape: "Invoke",
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn a_callback_derives_the_row_that_takes_it_apart() {
    let model = model(&["pub fn listen(on: impl Fn(u32) + Send + Sync + 'static) {}"]);
    let listen = model.function("listen").expect("listen");
    let table = Recipes::default();
    for direction in [Direction::Construct, Direction::Deconstruct] {
        let crossing = Crossing::new(listen.params[0].ty.clone(), direction);
        let (key, recipe) = table.recipe(&crossing);
        assert_eq!(key, row(&crossing, "derived"));
        assert_eq!(shape_of(&recipe), "invoke", "{recipe:?}");
        // The recipe's own direction is the crossing's; its arguments take the other.
        assert!(recipe.is_invoke());
        assert_eq!(recipe.direction(), direction);
    }
}

#[test]
fn a_row_reaching_its_own_crossing_is_refused() {
    let model = model(&[SAMPLE, "pub fn sample_clone(s: Sample) -> Sample {}"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("clone"),
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
            recipe_name("fields"),
            Constructing::Product(Construct::Call(ident("a_new"))),
        )
        .declare(
            ty(&model, "B"),
            recipe_name("fields"),
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
    // is the same type in the same direction, which is a cycle; the same recipe
    // filed under the other direction is not, because nothing links the two.
    let model = model(&[
        SAMPLE,
        "pub fn sample_new(key: u32, payload: u64) -> Sample {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Constructing::Product(Construct::Call(ident("sample_new"))),
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    builder
        .build(&model)
        .expect("the two directions do not meet");
}

#[test]
fn every_problem_is_reported_at_once() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![
                Reach::Field(9),
                Reach::Accessor(ident("nowhere")),
            ])),
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("whole"),
            Deconstructing::Atomic,
        );
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
    assert_eq!(Direction::Construct.swap(), Direction::Deconstruct);
    assert_eq!(Direction::Deconstruct.swap().swap(), Direction::Deconstruct);
}

#[test]
fn a_part_is_accepted_where_the_edge_can_consume_it() {
    // Owning it is enough for anything: a value handed over can be consumed,
    // lent, or lent mutably.
    assert!(Mode::Owned.satisfies(Mode::Owned));
    assert!(Mode::Owned.satisfies(Mode::Shared));
    assert!(Mode::Owned.satisfies(Mode::Exclusive));
    // A borrow serves only its own kind.
    assert!(!Mode::Shared.satisfies(Mode::Owned));
    assert!(Mode::Shared.satisfies(Mode::Shared));
    assert!(!Mode::Shared.satisfies(Mode::Exclusive));
    assert!(!Mode::Exclusive.satisfies(Mode::Owned));
    assert!(!Mode::Exclusive.satisfies(Mode::Shared));
    assert!(Mode::Exclusive.satisfies(Mode::Exclusive));
}

// ── Sites and recipe selection ───────────────────────────────────────────────

fn site(owner: &str, index: usize) -> Site {
    Site {
        owner: ident(owner),
        role: Role::Param { index },
    }
}

/// A table where `Sample` has two deconstructing recipes, `whole` being the
/// default — the shape every selection test below needs.
fn two_recipes(model: &Flat) -> Recipes {
    let mut builder = Recipes::builder();
    builder
        .declare_default(
            ty(model, "Sample"),
            recipe_name("whole"),
            Deconstructing::Atomic,
        )
        .declare(
            ty(model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    builder.build(model).expect("table")
}

#[test]
fn a_site_nobody_bound_takes_the_default_row() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("a site nobody bound still crosses");
    assert_eq!(bound.recipe, row(&crossing, "whole"));
    assert_eq!(bound.origin, Origin::Adapter);
    assert!(!bindings.is_declared(&site("z_put", 0)));
}

#[test]
fn an_undeclared_crossing_resolves_to_its_derived_row() {
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Vec<Sample>"), Direction::Deconstruct);

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("the derived recipe");
    assert_eq!(bound.recipe, row(&crossing, "derived"));
}

#[test]
fn a_site_takes_the_row_it_names() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing.clone(),
        Ask::Recipe(recipe_name("fields")),
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");

    let bound = bindings
        .resolve(&site("z_put", 0), &crossing, &recipes)
        .expect("bound");
    assert_eq!(bound.recipe, row(&crossing, "fields"));
    assert_eq!(bound.origin, Origin::Function);
    // Every other site of the same crossing still takes the default.
    let other = bindings
        .resolve(&site("z_get", 0), &crossing, &recipes)
        .expect("bound");
    assert_eq!(other.recipe, row(&crossing, "whole"));
}

/// An adapter picks a declared row for one site, where the binding table cannot
/// say which: the choice follows from something already compiled.
#[test]
fn an_adapter_can_select_one_declared_row_for_one_site() {
    /// Answers `fields` for parameter 0 and leaves every other site alone.
    #[derive(Default)]
    struct Selective(Recorder);

    impl Compile for Selective {
        type Fragment = Note;
        type Plan = String;
        type Error = String;

        fn site_recipe(&mut self, _cx: &mut Ctx<'_, Self>, bound: &Bound) -> Option<RecipeName> {
            matches!(bound.site.role, Role::Param { index: 0 }).then(|| recipe_name("fields"))
        }
        fn atomic(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>) -> Frag<Self> {
            self.0.atomic(cx, at)
        }
        fn optional(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>, inner: &Note) -> Frag<Self> {
            self.0.optional(cx, at, inner)
        }
        fn sequence(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            elements: Mode,
            inner: &Note,
        ) -> Frag<Self> {
            self.0.sequence(cx, at, elements, inner)
        }
        fn construct(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            func: &Function,
            args: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.construct(cx, at, func, args)
        }
        fn fields(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            parts: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.fields(cx, at, parts)
        }
        fn value_form(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            func: &Function,
            parts: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.value_form(cx, at, func, parts)
        }
        fn choice(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            arms: &[(Option<&Alternative>, &Note)],
        ) -> Frag<Self> {
            self.0.choice(cx, at, arms)
        }
        fn callback(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            at: At<'_>,
            args: &[&Note],
            result: Option<&Note>,
        ) -> Frag<Self> {
            self.0.callback(cx, at, args, result)
        }
        fn plan(
            &mut self,
            cx: &mut Ctx<'_, Self>,
            bound: &Bound,
            root: &Note,
        ) -> Result<String, String> {
            self.0.plan(cx, bound, root)
        }
    }

    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
    let mut adapter = Selective::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let selected = compiler
        .site(&mut adapter, site("z_put", 0), crossing.clone())
        .expect("declared row")
        .expect("site plan");
    assert!(selected.contains("recipe `fields`"), "{selected}");

    // Every other site takes the binding's answer, which is the default row.
    let default = compiler
        .site(&mut adapter, site("z_get", 1), crossing)
        .expect("default row")
        .expect("site plan");
    assert!(default.contains("recipe `whole`"), "{default}");
}

#[test]
fn the_higher_precedence_declaration_wins_whichever_was_written_first() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);

    for order in [
        [Origin::Type, Origin::Function],
        [Origin::Function, Origin::Type],
    ] {
        let mut builder = Bindings::builder();
        for origin in order {
            let ask = match origin {
                Origin::Function => Ask::Recipe(recipe_name("fields")),
                _ => Ask::Recipe(recipe_name("whole")),
            };
            builder.bind(site("z_put", 0), crossing.clone(), ask, origin);
        }
        let bindings = builder.build(&recipes).expect("bindings");
        let bound = bindings
            .resolve(&site("z_put", 0), &crossing, &recipes)
            .expect("bound");
        assert_eq!(bound.recipe, row(&crossing, "fields"), "written {order:?}");
        assert_eq!(bound.origin, Origin::Function, "written {order:?}");
    }
}

#[test]
fn two_declarations_of_equal_precedence_may_agree_and_may_not_disagree() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);

    let mut agreeing = Bindings::builder();
    agreeing
        .bind(
            site("z_put", 0),
            crossing.clone(),
            Ask::Recipe(recipe_name("fields")),
            Origin::Function,
        )
        .bind(
            site("z_put", 0),
            crossing.clone(),
            Ask::Recipe(recipe_name("fields")),
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
            Ask::Recipe(recipe_name("fields")),
            Origin::Function,
        )
        .bind(
            site("z_put", 0),
            crossing,
            Ask::Recipe(recipe_name("whole")),
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
    let recipes = two_recipes(&model);
    let mut builder = Bindings::builder();
    builder
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
            Ask::Default,
            Origin::Function,
        )
        // Same ask, different type: still two answers for one place.
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "u32"), Direction::Deconstruct),
            Ask::Default,
            Origin::Function,
        )
        // A third disagreement over the same site is still one report.
        .bind(
            site("z_put", 0),
            Crossing::new(ty(&model, "u64"), Direction::Deconstruct),
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
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
    let missing = crossing.row(recipe_name("jobject"));
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing,
        Ask::Recipe(recipe_name("jobject")),
        Origin::Function,
    );
    let errors = builder.build(&recipes).expect_err("no such recipe");
    assert!(
        matches!(errors.as_slice(), [RecipeError::UnknownRecipe { recipe, .. }] if recipe == &missing),
        "{errors:?}"
    );
}

#[test]
fn an_omitted_site_contributes_nothing() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
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
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
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
    assert_eq!(bound.recipe, row(&crossing, "whole"));
    assert_eq!(bound.origin, Origin::Type);
}

#[test]
fn one_site_is_one_role_of_one_owner() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
    let ret = Site {
        owner: ident("z_put"),
        role: Role::Return,
    };
    let mut builder = Bindings::builder();
    builder.bind(
        ret.clone(),
        crossing.clone(),
        Ask::Recipe(recipe_name("fields")),
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");

    assert_eq!(
        bindings.resolve(&ret, &crossing, &recipes).unwrap().recipe,
        row(&crossing, "fields")
    );
    // The same function's parameter 0 is a different site.
    assert_eq!(
        bindings
            .resolve(&site("z_put", 0), &crossing, &recipes)
            .unwrap()
            .recipe,
        row(&crossing, "whole")
    );
}

// ── Compiling recipes into fragments ─────────────────────────────────────────

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

    fn composed(&mut self, _shape: crate::generation::ShapePlan) {}
}

/// The smallest adapter that exercises every hook: it emits nothing and only
/// writes down what it was asked.
#[derive(Default)]
struct Recorder {
    /// One line per hook call, in call order.
    calls: Vec<String>,
    /// Crossings whose fragment claims to be reached through a shared borrow
    /// where the crossing itself is not — an adapter that lends where the
    /// spelling hands over.
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
                // The crossing's own mode, as both real adapters report it;
                // `shared` is an override for the tests that need a fragment to
                // disagree with its spelling.
                mode: if self.shared.contains(&name) {
                    Mode::Shared
                } else {
                    at.crossing.mode()
                },
                validity: if self.borrowed.contains(&name) {
                    Validity::Borrowed
                } else {
                    Validity::SelfSufficient
                },
            },
        }
    }

    fn hook(&mut self, at: At<'_>, hook: &str, detail: String) -> Frag<Self> {
        let ty = at.crossing.value().stripped_key();
        Ok(self.note(
            at,
            format!("{hook} {ty} {}: {detail}", at.crossing.direction()),
        ))
    }
}

fn part_names<C: Compile>(parts: Parts<'_, C>) -> String {
    parts
        .iter()
        .map(|(p, _)| {
            let source = match &p.from {
                PartSource::Argument { index } => format!("arg{index}"),
                PartSource::Field { index, .. } => format!("field{index}"),
                PartSource::Accessor { func } => format!("via {}", func.name),
                PartSource::Identity => "self".to_string(),
                PartSource::Path { indices, .. } => format!("path{indices:?}"),
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

    fn atomic(&mut self, _cx: &mut Cx<'_, Note>, at: At<'_>) -> Frag<Self> {
        self.hook(at, "atomic", "model".to_owned())
    }

    fn optional(&mut self, _cx: &mut Cx<'_, Note>, at: At<'_>, inner: &Note) -> Frag<Self> {
        let detail = inner.text.clone();
        self.hook(at, "optional", detail)
    }

    fn sequence(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        at: At<'_>,
        elements: Mode,
        inner: &Note,
    ) -> Frag<Self> {
        let detail = format!("elements {elements} of {}", inner.text);
        self.hook(at, "sequence", detail)
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        at: At<'_>,
        func: &Function,
        args: Parts<'_, Self>,
    ) -> Frag<Self> {
        let detail = format!("{}({})", func.name, part_names::<Self>(args));
        self.hook(at, "construct", detail)
    }

    fn fields(&mut self, _cx: &mut Cx<'_, Note>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        let detail = part_names::<Self>(parts);
        self.hook(at, "fields", detail)
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        at: At<'_>,
        func: &Function,
        parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        let detail = format!("{} -> {}", func.name, part_names::<Self>(parts));
        self.hook(at, "value_form", detail)
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        at: At<'_>,
        arms: &[(Option<&Alternative>, &Note)],
    ) -> Frag<Self> {
        let detail = arms
            .iter()
            .map(|(a, f)| match a {
                Some(a) => format!("{}#{} [{}]", a.name, a.index, f.text),
                None => format!("_ [{}]", f.text),
            })
            .collect::<Vec<_>>()
            .join(" | ");
        self.hook(at, "choice", detail)
    }

    fn callback(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        at: At<'_>,
        args: &[&Note],
        result: Option<&Note>,
    ) -> Frag<Self> {
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

    fn plan(
        &mut self,
        _cx: &mut Cx<'_, Note>,
        bound: &Bound,
        root: &Note,
    ) -> Result<String, String> {
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
        CompileError::Adapter(a) => panic!("the adapter refused: {a:?}"),
    }
}

fn compile_one(
    model: &Flat,
    recipes: &Recipes,
    adapter: &mut Recorder,
    site: Site,
    spelling: &str,
    direction: Direction,
) -> String {
    let bindings = Bindings::default();
    let mut compiler = Compiler::new(model, recipes, &bindings);
    compiler
        .site(adapter, site, Crossing::new(ty(model, spelling), direction))
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
        recipe_name("fields"),
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
        Direction::Construct,
    );
    assert!(
        plan.contains("sample_new(key=arg0/owned, payload=arg1/owned)"),
        "{plan}"
    );
    // Each part is a crossing of its own, compiled before the whole.
    assert_eq!(
        adapter.calls,
        vec![
            "atomic u32 construct: model",
            "atomic u64 construct: model",
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
        recipe_name("fields"),
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
        Direction::Deconstruct,
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
        recipe_name("fields"),
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
        Direction::Deconstruct,
    );
    // Declaration order is the recipe's, not the struct's.
    assert!(
        plan.contains("payload=field1/owned, key=field0/owned"),
        "{plan}"
    );
}

#[test]
fn a_product_reached_through_a_borrow_lends_each_field() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("fields"),
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
        "&Sample",
        Direction::Deconstruct,
    );
    assert!(plan.contains("payload=field1/&, key=field0/&"), "{plan}");
}

/// A declared identity row is refused, not an abort.
///
/// The part IS its receiver, so its crossing key equals the row's own and the
/// compiler used to re-enter it until the stack ran out (#635). It resolves
/// through the crossing's default row now, which for a type whose only row is
/// the identity one is that row again — a cycle the declaration really wrote,
/// and reported as one.
///
/// A handle leaf whose converter is terminal rather than another row is what
/// makes such a declaration useful; that needs parts that carry no child
/// fragment, which the adapter trait cannot express yet.
#[test]
fn a_self_defaulting_identity_row_is_refused_as_a_cycle() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare_default(
        ty(&model, "Sample"),
        recipe_name("handle"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity])),
    );
    let recipes = builder.build(&model).expect("table");
    let mut adapter = Recorder::default();
    let bindings = Bindings::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let err = compiler
        .site(
            &mut adapter,
            Site {
                owner: ident("z_get"),
                role: Role::Return,
            },
            Crossing::new(ty(&model, "&Sample"), Direction::Deconstruct),
        )
        .expect_err("an identity row that defaults to itself is a cycle");
    assert!(format!("{err:?}").contains("Cycle"), "{err:?}");
}

/// A borrowed identity part is lent, not owned — the coverage #635's review
/// asked for, now that an identity row beside a default one compiles.
///
/// `Crossing::value()` strips `&`/`&mut`, so deriving the mode from it would
/// record a `&Sample` identity row as owned, losing the clone-for-borrow /
/// move-for-owned distinction the form exists to carry.
#[test]
fn an_identity_part_through_a_borrow_is_lent_not_owned() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    let sample = ty(&model, "Sample");
    builder
        .declare_default(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(
            sample.clone(),
            recipe_name("handle"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity])),
        );
    let recipes = builder.build(&model).expect("table");

    let crossing = Crossing::new(ty(&model, "&Sample"), Direction::Deconstruct);
    let mut bind = Bindings::builder();
    bind.bind(
        site("z_get", 0),
        crossing.clone(),
        Ask::Recipe(recipe_name("handle")),
        Origin::Function,
    );
    let bindings = bind.build(&recipes).expect("bindings");

    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let plan = compiler
        .site(&mut adapter, site("z_get", 0), crossing)
        .expect("compile")
        .expect("not omitted");
    assert!(plan.contains("self=self/&"), "{plan}");
}

/// The owned half: the same row reached without a borrow moves its value.
#[test]
fn an_owned_identity_part_is_moved() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    let sample = ty(&model, "Sample");
    builder
        .declare_default(sample.clone(), recipe_name("whole"), Deconstructing::Atomic)
        .declare(
            sample.clone(),
            recipe_name("handle"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Identity])),
        );
    let recipes = builder.build(&model).expect("table");

    let crossing = Crossing::new(sample, Direction::Deconstruct);
    let mut bind = Bindings::builder();
    bind.bind(
        site("z_get", 0),
        crossing.clone(),
        Ask::Recipe(recipe_name("handle")),
        Origin::Function,
    );
    let bindings = bind.build(&recipes).expect("bindings");

    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let plan = compiler
        .site(&mut adapter, site("z_get", 0), crossing)
        .expect("compile")
        .expect("not omitted");
    assert!(plan.contains("self=self/owned"), "{plan}");
}

/// A path reaches a field of a field — what an inlined nested class needs.
///
/// `FieldRecord::members` is exactly this chain, and before `Reach::Path` a
/// `parts` row for a value form that inlines a nested declared class could not
/// be spelled: `value_form_of` declined any record with several members, which
/// is why two of #638's three rows still fall back to `Atomic` (#613 step 10).
#[test]
fn a_path_reach_resolves_a_field_of_a_field() {
    let model = model(&[
        SAMPLE,
        "pub struct Outer { pub inner: Sample, pub tag: u8 }",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Outer"),
        recipe_name("fields"),
        // `inner.payload`, then `tag` — a chain beside a plain field.
        Deconstructing::Product(Deconstruct::Fields(vec![
            Reach::Path(vec![0, 1]),
            Reach::Field(1),
        ])),
    );
    let recipes = builder.build(&model).expect("a path over declared structs");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Outer",
        Direction::Deconstruct,
    );
    assert!(
        plan.contains("payload="),
        "the path reaches the nested field:\n{plan}"
    );
    assert!(
        plan.contains("tag="),
        "the plain field is unaffected:\n{plan}"
    );
}

/// A path whose hop is past the end is refused, like a one-hop `Field`.
#[test]
fn a_path_reach_past_the_end_is_refused() {
    let model = model(&[
        SAMPLE,
        "pub struct Outer { pub inner: Sample, pub tag: u8 }",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Outer"),
        recipe_name("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Path(vec![0, 7])])),
    );
    let errors = builder
        .build(&model)
        .expect_err("the second hop is out of range");
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

/// A field taken apart HERE, by a shape the row carries, rather than by the
/// row its own type has.
///
/// The form a sum-typed field needs: a `sealed_class` has no deconstructing
/// whole-value crossing, so such a field cannot be a whole part at all, and
/// `Fields(Vec<Reach>)` cannot hold the `Choice` its leaves need (#613 step 10).
#[test]
fn a_nested_shape_takes_a_field_apart_in_place() {
    let model = model(&[
        SAMPLE,
        "pub struct Outer { pub inner: Sample, pub tag: u8 }",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Outer"),
        recipe_name("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![
            Reach::Nested {
                index: 0,
                shape: Box::new(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
            },
            Reach::Field(1),
        ])),
    );
    let recipes = builder.build(&model).expect("a nested shape validates");
    let mut adapter = Recorder::default();

    let plan = compile_one(
        &model,
        &recipes,
        &mut adapter,
        Site {
            owner: ident("z_get"),
            role: Role::Return,
        },
        "Outer",
        Direction::Deconstruct,
    );
    // The nested shape contributes ITS parts, not one part for the field.
    assert!(
        plan.contains("key="),
        "the nested shape's first part:\n{plan}"
    );
    assert!(
        plan.contains("payload="),
        "the nested shape's second part:\n{plan}"
    );
    assert!(
        plan.contains("tag="),
        "the plain field is unaffected:\n{plan}"
    );
}

/// A nested shape over a SUM-typed field is refused, not silently empty.
///
/// `Deconstruct` reads parts off a product, and a variant has no fields to
/// read, so this reach would contribute nothing at all — dropping the sum's
/// leaves without a word. Reaching those leaves needs a `Choice`, which lives
/// on `Shape` and not on `Deconstruct` (#658 review).
#[test]
fn a_nested_shape_over_a_sum_field_is_refused() {
    let model = model(&[
        "pub enum Pick { A { x: u32 }, B { y: u64 } }",
        "pub struct Holder { pub pick: Pick, pub tag: u8 }",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Holder"),
        recipe_name("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Nested {
            index: 0,
            shape: Box::new(Deconstruct::Fields(vec![Reach::Field(0)])),
        }])),
    );
    let errors = builder
        .build(&model)
        .expect_err("a sum field cannot be read as a product");
    assert!(
        matches!(errors.as_slice(), [RecipeError::NotAProduct { .. }]),
        "{errors:?}"
    );
}

/// A nested shape over a field index past the end is refused.
#[test]
fn a_nested_shape_past_the_end_is_refused() {
    let model = model(&[
        SAMPLE,
        "pub struct Outer { pub inner: Sample, pub tag: u8 }",
    ]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Outer"),
        recipe_name("fields"),
        Deconstructing::Product(Deconstruct::Fields(vec![Reach::Nested {
            index: 9,
            shape: Box::new(Deconstruct::Fields(vec![Reach::Field(0)])),
        }])),
    );
    let errors = builder.build(&model).expect_err("index 9 is past the end");
    assert!(
        matches!(
            errors.as_slice(),
            [RecipeError::OutOfRange {
                index: 9,
                len: 2,
                ..
            }]
        ),
        "{errors:?}"
    );
}

#[test]
fn an_omitted_reach_contributes_no_part() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("fields"),
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
        Direction::Deconstruct,
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
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("whole"),
        Deconstructing::Atomic,
    );
    let recipes = builder.build(&model).expect("table");
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    for spelling in ["Sample", "&Sample", "Box<Sample>"] {
        let crossing = Crossing::new(ty(&model, spelling), Direction::Deconstruct);
        // All three find the one declared recipe …
        assert_eq!(recipes.recipe(&crossing).0, row(&crossing, "whole"));
        compiler.crossing(&mut adapter, &crossing).expect("compile");
    }
    // … and each still gets its own Rust, because taking a value out of a
    // pointer, borrowing through one and rebuilding a Box are three things.
    assert_eq!(compiler.compiled_fragments(), 3);
    assert_eq!(
        adapter.calls,
        vec![
            "atomic Sample deconstruct: model",
            "atomic Sample deconstruct: model",
            "atomic Sample deconstruct: model",
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

    let (first, _) = compiler
        .crossing(
            &mut adapter,
            &Crossing::new(ty(&model, "Option<Sample>"), Direction::Deconstruct),
        )
        .expect("compile");
    assert!(first.text.starts_with("optional"), "{}", first.text);
    // Asking twice is the same fragment, not a second compilation.
    compiler
        .crossing(
            &mut adapter,
            &Crossing::new(ty(&model, "Option<Sample>"), Direction::Deconstruct),
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
        recipe_name("fields"),
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
                Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
            Direction::Deconstruct,
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
fn a_mode_reached_through_a_container_composes_both_layers() {
    use Mode::{Exclusive, Owned, Shared};
    // An owned container gives its contents up, so the value's own mode stands.
    assert_eq!(Owned.through(Owned), Owned);
    assert_eq!(Shared.through(Owned), Shared);
    assert_eq!(Exclusive.through(Owned), Exclusive);
    // A shared view yields nothing stronger, whatever it holds.
    assert_eq!(Owned.through(Shared), Shared);
    assert_eq!(Shared.through(Shared), Shared);
    assert_eq!(Exclusive.through(Shared), Shared);
    // An exclusive view of a shared reference is still shared; of anything
    // else it is exclusive.
    assert_eq!(Shared.through(Exclusive), Shared);
    assert_eq!(Owned.through(Exclusive), Exclusive);
    assert_eq!(Exclusive.through(Exclusive), Exclusive);
}

#[test]
fn an_optionals_value_is_held_through_the_optional() {
    // Reading through a shared `&Option<T>` can only lend its value, so the
    // shared fragment is the correct one and demanding an owned `T` refuses it.
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();

    let mut adapter = Recorder::default();
    adapter.shared.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&Option<Sample>"), Direction::Construct),
        )
        .expect("a shared optional lends its value");

    // The same fragment cannot serve an owned `Option<T>`, which hands it over.
    let mut adapter = Recorder::default();
    adapter.shared.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Option<Sample>"), Direction::Construct),
        )
        .expect_err("an owned optional hands its value over");
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
}

#[test]
fn an_exclusive_optional_lends_its_value_exclusively() {
    // `&mut Option<T>` lends `&mut T`, so an owned fragment satisfies it and a
    // merely shared one does not — the opposite direction from `&Option<T>`.
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();

    // The positive half, and the one that matters: the *ordinary* fragment for
    // a bare `T` reports owned, and it has to serve this edge. Asserting only
    // the refusal below would pass with no usable fragment at all.
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&mut Option<Sample>"), Direction::Construct),
        )
        .expect("an owned fragment serves an exclusive optional");

    let mut adapter = Recorder::default();
    adapter.shared.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&mut Option<Sample>"), Direction::Construct),
        )
        .expect_err("a shared fragment cannot serve an exclusive optional");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::Composition {
                wanted: Mode::Exclusive,
                got: Mode::Shared,
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn a_shared_run_of_exclusive_references_caps_at_shared() {
    // `&[&mut T]` yields `&&mut T`, so it cannot hand over the `&mut T` its
    // element is spelled as. Reading the element alone asks `Exclusive` and
    // accepts the exclusive fragment, which the sequence hook could not
    // legally be fed; composing caps the ask at `Shared` and refuses it.
    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let error = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&[&mut Sample]"), Direction::Construct),
        )
        .expect_err("a shared slice cannot lend its elements exclusively");
    assert!(
        matches!(
            recipe_error(&error),
            RecipeError::Composition {
                wanted: Mode::Shared,
                got: Mode::Exclusive,
                ..
            }
        ),
        "{error:?}"
    );

    // An exclusive run of exclusive references does hand them over.
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "&mut [&mut Sample]"), Direction::Construct),
        )
        .expect("an exclusive slice lends its elements exclusively");
    assert!(
        adapter.calls.iter().any(|c| c.contains("elements &mut")),
        "{:?}",
        adapter.calls
    );
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
        Direction::Deconstruct,
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
        recipe_name("variants"),
        Deconstructing::Choice {
            arms: vec![
                Arm {
                    alternative: Some(0),
                    op: Deconstruct::Fields(vec![Reach::Field(0)]),
                },
                Arm {
                    alternative: Some(1),
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
        Direction::Deconstruct,
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
            Crossing::new(listen.params[0].ty.clone(), Direction::Construct),
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
}

#[test]
fn a_callback_argument_is_a_part_of_the_callback_row_that_names_it() {
    // The argument takes the *other* direction — Rust holds the value and pushes it
    // out — but the part still belongs to the callback recipe, so a binding
    // written against that recipe applies. Keying the site by the swapped
    // crossing instead made every such binding silently miss.
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(Sample) + Send + Sync + 'static) {}",
    ]);
    let listen = model.function("listen").expect("listen");
    let callback = listen.params[0].ty.clone();
    let mut builder = Recipes::builder();
    builder
        .declare_default(
            ty(&model, "Sample"),
            recipe_name("whole"),
            Deconstructing::Atomic,
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0)])),
        );
    let recipes = builder.build(&model).expect("table");

    // The recipe this part belongs to: the callback, constructed.
    let recipe = Crossing::new(callback.clone(), Direction::Construct);
    // Built the same way the driver builds it, which is the point of the
    // helper: a per-part binding is found by this exact key or not at all.
    let recipe_key = row(&recipe, "derived");
    let part = Site::part(&recipe_key, 0);
    let mut bound = Bindings::builder();
    bound.bind(
        part,
        // The part's own crossing carries the swap; the site does not.
        Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
        Ask::Recipe(recipe_name("fields")),
        Origin::Part,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    compiler
        .site(&mut adapter, site("listen", 0), recipe)
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
    // A callback recipe is shared by every function whose callback has the same
    // signature, so a per-function answer cannot apply to it. `Role::CallbackArg`
    // is a root role: an adapter compiles that position itself.
    let model = model(&[
        SAMPLE,
        "pub fn listen(on: impl Fn(Sample) + Send + Sync + 'static) {}",
    ]);
    let mut builder = Recipes::builder();
    builder
        .declare_default(
            ty(&model, "Sample"),
            recipe_name("whole"),
            Deconstructing::Atomic,
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
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
        Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
        Ask::Recipe(recipe_name("fields")),
        Origin::Function,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let plan = compiler
        .site(
            &mut adapter,
            arg,
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
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
        recipe_name("fields"),
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
            Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
        recipe_name("fields"),
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
            Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
            Crossing::new(ty(model, spelling), Direction::Construct),
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
            Crossing::new(listen.params[0].ty.clone(), Direction::Construct),
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
            Crossing::new(ty(&model, "Vec<u64>"), Direction::Construct),
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
            Crossing::new(ty(&model, "&[u64]"), Direction::Construct),
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
        recipe_name("fields"),
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
            Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
        recipe_name("fields"),
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
        recipe_name("fields"),
        Constructing::Product(Construct::Call(ident("sample_try"))),
    );
    good.build(&model)
        .expect("Result<Sample, _> builds a Sample");

    let mut bad = Recipes::builder();
    bad.declare(
        ty(&model, "Sample"),
        recipe_name("fields"),
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
        recipe_name("fields"),
        Constructing::Product(Construct::Call(ident("sample_boxed"))),
    );
    builder
        .build(&model)
        .expect("a Box<Sample> builds a Sample");
}

#[test]
fn what_a_role_tolerates_is_the_adapters_own_answer() {
    // The strict default refuses a borrowed return; an adapter whose target
    // hands out non-owning pointers deliberately says so and is not refused.
    // C is that adapter: a zero-copy accessor crosses as `*const T`, and C's
    // contract is that the caller neither frees it nor outlives it.
    struct Lenient(Recorder);
    impl Compile for Lenient {
        type Fragment = Note;
        type Plan = String;
        type Error = String;
        fn tolerates(&self, _role: &Role) -> Validity {
            Validity::Borrowed
        }
        fn atomic(&mut self, cx: &mut Cx<'_, Note>, at: At<'_>) -> Frag<Self> {
            self.0.atomic(cx, at)
        }
        fn optional(&mut self, cx: &mut Cx<'_, Note>, at: At<'_>, inner: &Note) -> Frag<Self> {
            self.0.optional(cx, at, inner)
        }
        fn sequence(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            elements: Mode,
            inner: &Note,
        ) -> Frag<Self> {
            self.0.sequence(cx, at, elements, inner)
        }
        fn construct(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            func: &Function,
            args: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.construct(cx, at, func, args)
        }
        fn fields(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            parts: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.fields(cx, at, parts)
        }
        fn value_form(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            func: &Function,
            parts: Parts<'_, Self>,
        ) -> Frag<Self> {
            self.0.value_form(cx, at, func, parts)
        }
        fn choice(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            arms: &[(Option<&Alternative>, &Note)],
        ) -> Frag<Self> {
            self.0.choice(cx, at, arms)
        }
        fn callback(
            &mut self,
            cx: &mut Cx<'_, Note>,
            at: At<'_>,
            args: &[&Note],
            result: Option<&Note>,
        ) -> Frag<Self> {
            self.0.callback(cx, at, args, result)
        }
        fn plan(
            &mut self,
            cx: &mut Cx<'_, Note>,
            bound: &Bound,
            root: &Note,
        ) -> Result<String, String> {
            self.0.plan(cx, bound, root)
        }
    }

    let model = model(&[SAMPLE]);
    let recipes = Recipes::default();
    let bindings = Bindings::default();
    let ret = Site {
        owner: ident("z_sample_payload"),
        role: Role::Return,
    };
    let crossing = || Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);

    // Strict: refused.
    let mut strict = Recorder::default();
    strict.borrowed.insert("Sample".to_owned());
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    let error = compiler
        .site(&mut strict, ret.clone(), crossing())
        .expect_err("the default refuses a borrowed return");
    assert!(
        matches!(recipe_error(&error), RecipeError::Validity { .. }),
        "{error:?}"
    );

    // Lenient: the same fragment, accepted, because the target says so.
    let mut inner = Recorder::default();
    inner.borrowed.insert("Sample".to_owned());
    let mut lenient = Lenient(inner);
    let mut compiler = Compiler::new(&model, &recipes, &bindings);
    compiler
        .site(&mut lenient, ret, crossing())
        .expect("an adapter that hands out non-owning pointers is not refused");
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
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
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
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
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
        Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
            Crossing::new(ty(&model, "Sample"), Direction::Construct),
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
        .declare_default(
            ty(&model, "Sample"),
            recipe_name("whole"),
            Deconstructing::Atomic,
        )
        .declare(
            ty(&model, "Sample"),
            recipe_name("fields"),
            Deconstructing::Product(Deconstruct::Fields(vec![Reach::Field(0), Reach::Field(1)])),
        );
    let recipes = builder.build(&model).expect("table");
    let mut bound = Bindings::builder();
    bound.bind(
        site("z_put", 0),
        Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
        Ask::Recipe(recipe_name("fields")),
        Origin::Function,
    );
    let bindings = bound.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    let overridden = compiler
        .site(
            &mut adapter,
            site("z_put", 0),
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
        )
        .expect("compile")
        .expect("not omitted");
    let plain = compiler
        .site(
            &mut adapter,
            site("z_get", 0),
            Crossing::new(ty(&model, "Sample"), Direction::Deconstruct),
        )
        .expect("compile")
        .expect("not omitted");

    assert!(overridden.contains("fields Sample"), "{overridden}");
    assert!(plain.contains("atomic Sample"), "{plain}");
    // Two recipes of one crossing, so two fragments — plus the two field types.
    assert_eq!(compiler.compiled_fragments(), 4);
}

#[test]
fn a_struct_with_no_constructor_is_built_from_its_own_fields() {
    let model = model(&[SAMPLE]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "Sample"),
        recipe_name("literal"),
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
        Direction::Construct,
    );
    // Every field contributes, in the model's order, and the same `fields` hook
    // serves both directions.
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
        recipe_name("variants"),
        Constructing::Choice {
            arms: vec![
                Arm {
                    alternative: Some(0),
                    op: Construct::Fields,
                },
                Arm {
                    alternative: Some(1),
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
        Direction::Construct,
    );
    assert!(
        plan.contains("Ok#0 [fields Reply construct: 0=field0/owned]"),
        "{plan}"
    );
    assert!(
        plan.contains("Err#1 [fields Reply construct: 0=field0/owned]"),
        "{plan}"
    );
    // The two arms' payloads are different types, so they are different recipes.
    assert!(adapter.calls.iter().any(|c| c.starts_with("atomic u32")));
    assert!(adapter.calls.iter().any(|c| c.starts_with("atomic u64")));
}

#[test]
fn building_a_type_the_model_gives_no_fields_is_refused() {
    let model = model(&["pub struct Handle;"]);
    let mut builder = Recipes::builder();
    builder.declare(
        ty(&model, "u32"),
        recipe_name("literal"),
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
        recipe_name("read"),
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
        Direction::Deconstruct,
    );
    assert!(
        plan.contains("handle_read -> key=field0/owned, payload=field1/owned"),
        "{plan}"
    );
}

#[test]
fn an_emitter_asking_for_a_crossing_gets_the_row_the_crossing_defaults_to() {
    let model = model(&[SAMPLE]);
    let sample = ty(&model, "Sample");
    // `fields` sorts before `whole`, so a lookup that ranked the recipes by name
    // rather than asking which one the crossing defaults to would answer with
    // the wrong one.
    let recipes = two_recipes(&model);
    let crossing = Crossing::new(sample.clone(), Direction::Deconstruct);
    let mut builder = Bindings::builder();
    builder.bind(
        site("z_put", 0),
        crossing.clone(),
        Ask::Recipe(recipe_name("fields")),
        Origin::Function,
    );
    let bindings = builder.build(&recipes).expect("bindings");
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    // One site takes `fields`; the crossing on its own takes its default.
    compiler
        .site(&mut adapter, site("z_put", 0), crossing.clone())
        .expect("site");
    compiler.crossing(&mut adapter, &crossing).expect("whole");
    let compiled = compiler.finish();

    let key = sample.key();
    // An emitter asks by crossing and gets the default …
    assert_eq!(
        compiled
            .fragment(&key, Direction::Deconstruct)
            .map(|n| n.text.clone()),
        Some("atomic Sample deconstruct: model".to_string()),
    );
    // … and a caller holding a recipe still reaches that recipe.
    assert!(compiled
        .recipe_fragment(&key, &row(&crossing, "fields"))
        .is_some());
    // The other direction was never crossed, so there is no answer to give.
    assert!(compiled.fragment(&key, Direction::Construct).is_none());
}

/// A row is compiled by its **key**, which only the table can hand out, so a
/// name the crossing does not have cannot be compiled at all.
///
/// The two callers that reach [`Compiler::recipe`] with a name they got *from*
/// the table — a crossing's default, and a site's binding — rely on its
/// fallback to the derived recipe, and it is right for them. A caller's own
/// claim must not reach that fallback: it would compile the default and file it
/// under the asked-for name, leaving [`Compiled::recipe_fragment`] to answer for
/// a recipe nobody declared. Taking a key rather than a name is what makes that
/// unwritable.
///
/// The shape that hits it is a conditionally-declared recipe: an adapter that
/// declares `fields` only for a type that has some, then asks every declared
/// type for `fields` anyway.
#[test]
fn a_row_is_compiled_by_a_key_only_the_table_hands_out() {
    let model = model(&[SAMPLE]);
    let recipes = two_recipes(&model);
    let bindings = Bindings::default();
    let crossing = Crossing::new(ty(&model, "Sample"), Direction::Deconstruct);
    let mut adapter = Recorder::default();
    let mut compiler = Compiler::new(&model, &recipes, &bindings);

    assert!(
        recipes
            .key_of(&crossing.key(), &recipe_name("nonesuch"))
            .is_none(),
        "`Sample` declares `whole` and `fields`, and nothing else"
    );

    // The two recipes that DO exist compile through the key the table gives.
    for name in ["whole", "fields"] {
        let key = recipes
            .key_of(&crossing.key(), &recipe_name(name))
            .unwrap_or_else(|| panic!("`{name}` is declared"))
            .clone();
        compiler
            .row(&mut adapter, &crossing, &key)
            .unwrap_or_else(|e| panic!("`{name}` is declared: {e}"));
    }

    // And nothing was filed under the name nobody declared.
    let compiled = compiler.finish();
    let missing = row(&crossing, "nonesuch");
    assert!(compiled
        .recipe_fragment(&ty(&model, "Sample").key(), &missing)
        .is_none());
}
