use prebindgen::SourceLocation;

use super::*;
use crate::{
    flat::Flat,
    recipe::{Origin, RecipeName, Role, Validity},
    test_util::declare_referenced,
};

struct Fake;

impl Representation for Fake {
    type Intermediate = &'static str;
    type Step = &'static str;
    type TerminalCodec = &'static str;
    type ProductBridge = &'static str;
    type OptionalBridge = &'static str;
    type SequenceBridge = &'static str;
    type ChoiceBridge = &'static str;
    type CallbackBridge = &'static str;
    type Niche = u8;
    type Cleanup = &'static str;
    type FailureRoute = &'static str;
    type AbiLayout = &'static str;
    type Artifact = &'static str;
}

fn model() -> Flat {
    let items = [
        "pub struct Leaf;",
        "pub struct Pair { pub left: Leaf, pub right: Leaf }",
    ]
    .into_iter()
    .map(|source| {
        (
            syn::parse_str::<syn::Item>(source).unwrap(),
            SourceLocation::default(),
        )
    });
    Flat::builder()
        .items(declare_referenced(items))
        .build()
        .unwrap()
}

fn ty(model: &Flat, spelling: &str) -> TypeRef {
    model.classify(&syn::parse_str(spelling).unwrap()).unwrap()
}

fn fragment_id(model: &Flat, spelling: &str, direction: Direction) -> FragmentId {
    let crossing = Crossing::new(ty(model, spelling), direction);
    FragmentId::new(
        crossing.spelled().key(),
        crossing.row(RecipeName::new("whole")),
    )
}

fn yield_of(id: &FragmentId, mode: Mode, validity: Validity) -> Yield {
    Yield {
        ty: id.recipe().crossing().ty.clone(),
        mode,
        validity,
    }
}

fn atomic(model: &Flat, id: FragmentId, failure: Failure, mode: Mode) -> FragmentPlan<Fake> {
    FragmentPlan::new(
        id.clone(),
        ty(model, id.spelling().as_str()),
        "intermediate",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            failure,
            Cleanup::None,
        ),
        yield_of(&id, mode, Validity::SelfSufficient),
    )
}

fn site(
    model: &Flat,
    fragment: &FragmentId,
    failure_route: Option<&'static str>,
    slots: usize,
) -> SitePlan<Fake> {
    let site = Site {
        owner: syn::parse_str("make_pair").unwrap(),
        role: Role::Return,
    };
    let crossing = Crossing::new(
        ty(model, fragment.spelling().as_str()),
        fragment.direction(),
    );
    let bound = Bound {
        site: site.clone(),
        crossing,
        recipe: fragment.recipe().clone(),
        origin: Origin::Adapter,
    };
    SitePlan::new(
        SiteId::new(site),
        bound,
        fragment.clone(),
        yield_of(fragment, Mode::Owned, Validity::SelfSufficient),
        AbiLayout::new(slots, "layout"),
        failure_route,
        Cleanup::None,
    )
}

fn artifact(
    kind: &str,
    name: &str,
    prerequisites: Vec<ArtifactId>,
    inputs: Vec<ArtifactInput>,
) -> ArtifactPlan<Fake> {
    ArtifactPlan::new(
        ArtifactId::new(kind, name).unwrap(),
        prerequisites,
        inputs,
        "artifact",
    )
}

fn errors(result: Result<GenerationPlan<Fake>, PlanErrors>) -> PlanErrors {
    result.expect_err("plan should be invalid")
}

fn has(errors: &PlanErrors, predicate: impl Fn(&PlanError) -> bool) {
    assert!(errors.errors().iter().any(predicate), "{errors}");
}

#[test]
fn semantic_identities_refuse_accidental_aliases() {
    let model = model();
    let leaf = Crossing::new(ty(&model, "Leaf"), Direction::Construct);
    let pair = Crossing::new(ty(&model, "Pair"), Direction::Construct);
    let recipe = leaf.row(RecipeName::new("whole"));
    assert_ne!(
        FragmentId::new(leaf.spelled().key(), recipe.clone()),
        FragmentId::new(pair.spelled().key(), recipe),
    );
    assert!(ArtifactId::new("", "wrapper").is_err());
}

#[test]
fn freeze_prunes_unreached_fragments_and_orders_dependencies_first() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let pair = fragment_id(&model, "Pair", Direction::Construct);
    let unused = fragment_id(&model, "&Leaf", Direction::Construct);
    let requirement = yield_of(&leaf, Mode::Owned, Validity::Borrowed);
    let pair_plan = FragmentPlan::new(
        pair.clone(),
        ty(&model, "Pair"),
        "pair intermediate",
        ConverterPlan::new(
            ShapePlan::Product {
                bridge: FixedArity::new(2, "tuple"),
                parts: vec![
                    FragmentUse::new(leaf.clone(), requirement.clone()),
                    FragmentUse::new(leaf.clone(), requirement),
                ],
            },
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&pair, Mode::Owned, Validity::SelfSufficient),
    );
    let site_plan = site(&model, &pair, None, 2);
    let site_id = site_plan.id().clone();
    let helper_id = ArtifactId::new("converter", "pair").unwrap();
    let wrapper_id = ArtifactId::new("wrapper", "make_pair").unwrap();

    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder
        .fragment(pair_plan)
        .fragment(atomic(
            &model,
            unused.clone(),
            Failure::Infallible,
            Mode::Owned,
        ))
        .fragment(atomic(
            &model,
            leaf.clone(),
            Failure::Infallible,
            Mode::Owned,
        ))
        .site(site_plan)
        .artifact(artifact(
            "wrapper",
            "make_pair",
            vec![helper_id.clone()],
            vec![ArtifactInput::Site {
                site: site_id,
                slots: 2,
            }],
        ))
        .artifact(artifact(
            "converter",
            "pair",
            vec![],
            vec![ArtifactInput::Fragment(pair.clone())],
        ));
    let plan = builder.build().unwrap();

    let fragments: Vec<_> = plan.fragments().map(FragmentPlan::id).collect();
    assert_eq!(fragments, vec![&leaf, &pair]);
    assert!(plan.fragment(&unused).is_none());
    let artifacts: Vec<_> = plan.artifacts().map(ArtifactPlan::id).collect();
    assert_eq!(artifacts, vec![&helper_id, &wrapper_id]);
}

#[test]
fn freeze_reports_arity_niche_ownership_and_validity_errors() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let pair = fragment_id(&model, "Pair", Direction::Construct);
    let bad = FragmentPlan::new(
        pair.clone(),
        ty(&model, "Pair"),
        "pair",
        ConverterPlan::new(
            ShapePlan::Product {
                bridge: FixedArity::new(2, "tuple"),
                parts: vec![FragmentUse::new(
                    leaf.clone(),
                    yield_of(&leaf, Mode::Exclusive, Validity::SelfSufficient),
                )],
            },
            NichePlan::new(2, vec![1], vec![1]),
            Failure::Infallible,
            Cleanup::UnlessTransferred("drop"),
        ),
        yield_of(&pair, Mode::Shared, Validity::Borrowed),
    );
    let leaf_plan = FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "leaf",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&leaf, Mode::Shared, Validity::Borrowed),
    );
    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder
        .fragment(leaf_plan)
        .fragment(bad)
        .site(site(&model, &pair, None, 1));
    let errors = errors(builder.build());

    has(&errors, |e| matches!(e, PlanError::Arity(_)));
    has(&errors, |e| matches!(e, PlanError::InsufficientNiches(_)));
    has(&errors, |e| matches!(e, PlanError::OverlappingNiches(_)));
    has(&errors, |e| matches!(e, PlanError::TransferOfBorrowed(_)));
    has(&errors, |e| matches!(e, PlanError::ContractMode(_)));
    has(&errors, |e| matches!(e, PlanError::ContractValidity(_)));
}

#[test]
fn invoke_children_must_use_the_opposite_direction() {
    let model = model();
    let callable = fragment_id(&model, "Pair", Direction::Construct);
    let argument = fragment_id(&model, "Leaf", Direction::Construct);
    let invoke = FragmentPlan::new(
        callable.clone(),
        ty(&model, "Pair"),
        "callable",
        ConverterPlan::new(
            ShapePlan::Invoke {
                bridge: FixedArity::new(1, "invoke"),
                arguments: vec![FragmentUse::new(
                    argument.clone(),
                    yield_of(&argument, Mode::Owned, Validity::SelfSufficient),
                )],
            },
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&callable, Mode::Owned, Validity::SelfSufficient),
    );
    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder
        .fragment(atomic(&model, argument, Failure::Infallible, Mode::Owned))
        .fragment(invoke)
        .site(site(&model, &callable, None, 1));
    has(&errors(builder.build()), |e| {
        matches!(e, PlanError::ChildDirection(_))
    });
}

#[test]
fn site_failure_and_abi_contracts_are_checked() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let mut missing_route = GenerationPlanBuilder::<Fake>::new();
    missing_route
        .fragment(atomic(&model, leaf.clone(), Failure::Fallible, Mode::Owned))
        .site(site(&model, &leaf, None, 2));
    has(&errors(missing_route.build()), |e| {
        matches!(e, PlanError::MissingFailureRoute(_))
    });

    let site_plan = site(&model, &leaf, Some("throw"), 2);
    let site_id = site_plan.id().clone();
    let mut bad_arity = GenerationPlanBuilder::<Fake>::new();
    bad_arity
        .fragment(atomic(&model, leaf, Failure::Fallible, Mode::Owned))
        .site(site_plan)
        .artifact(artifact(
            "wrapper",
            "leaf",
            vec![],
            vec![ArtifactInput::Site {
                site: site_id,
                slots: 1,
            }],
        ));
    has(&errors(bad_arity.build()), |e| {
        matches!(e, PlanError::AbiArity(_, _))
    });
}

#[test]
fn duplicate_and_cyclic_artifact_identities_are_rejected() {
    let a = ArtifactId::new("helper", "a").unwrap();
    let b = ArtifactId::new("helper", "b").unwrap();
    let mut duplicate = GenerationPlanBuilder::<Fake>::new();
    duplicate
        .artifact(artifact("helper", "a", vec![], vec![]))
        .artifact(artifact("helper", "a", vec![], vec![]));
    has(&errors(duplicate.build()), |e| {
        matches!(e, PlanError::DuplicateArtifact(_))
    });

    let mut cyclic = GenerationPlanBuilder::<Fake>::new();
    cyclic
        .artifact(artifact("helper", "a", vec![b.clone()], vec![]))
        .artifact(artifact("helper", "b", vec![a], vec![]));
    has(&errors(cyclic.build()), |e| {
        matches!(e, PlanError::ArtifactCycle(_))
    });
}

#[test]
fn staged_conversion_is_one_ordered_fragment_chain() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let fragment = FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "jint",
        ConverterPlan::with_chain(
            ShapePlan::Atomic("jni scalar"),
            ConversionChain::Steps(vec![
                ConverterStep::new(
                    ChainValue::Intermediate("jint"),
                    ChainValue::Intermediate("i32"),
                    "normalize jint",
                    Failure::Infallible,
                    Cleanup::None,
                ),
                ConverterStep::new(
                    ChainValue::Intermediate("i32"),
                    ChainValue::Source,
                    "construct Percent",
                    Failure::Fallible,
                    Cleanup::OnFailure("drop intermediate"),
                ),
            ]),
            NichePlan::none(),
            Failure::Fallible,
            Cleanup::None,
        ),
        yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
    );
    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder
        .fragment(fragment)
        .site(site(&model, &leaf, Some("throw"), 1));
    let plan = builder.build().unwrap();

    let chain = plan.fragment(&leaf).unwrap().converter().chain();
    assert_eq!(plan.fragment(&leaf).unwrap().intermediate(), &"jint");
    assert_eq!(chain.steps().len(), 2);
    assert_eq!(*chain.steps()[0].operation(), "normalize jint");
    assert_eq!(*chain.steps()[1].operation(), "construct Percent");
}

#[test]
fn malformed_conversion_chains_are_rejected() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);

    let mut empty = GenerationPlanBuilder::<Fake>::new();
    empty.fragment(FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "Leaf",
        ConverterPlan::with_chain(
            ShapePlan::Atomic("codec"),
            ConversionChain::Steps(vec![]),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
    ));
    has(&errors(empty.build()), |e| {
        matches!(e, PlanError::EmptyConversionChain(_))
    });

    let mut broken = GenerationPlanBuilder::<Fake>::new();
    broken.fragment(FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "Leaf",
        ConverterPlan::with_chain(
            ShapePlan::Atomic("codec"),
            ConversionChain::Steps(vec![ConverterStep::new(
                ChainValue::Intermediate("not jint"),
                ChainValue::Intermediate("i32"),
                "convert",
                Failure::Fallible,
                Cleanup::None,
            )]),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
    ));
    let errors = errors(broken.build());
    has(&errors, |e| {
        matches!(e, PlanError::BrokenConversionChain(_))
    });
    has(&errors, |e| {
        matches!(e, PlanError::UnreportedStepFailure(_))
    });
}

#[test]
fn fragment_identity_duplicates_and_yield_type_are_checked() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);

    let mut duplicate = GenerationPlanBuilder::<Fake>::new();
    duplicate
        .fragment(atomic(
            &model,
            leaf.clone(),
            Failure::Infallible,
            Mode::Owned,
        ))
        .fragment(atomic(
            &model,
            leaf.clone(),
            Failure::Infallible,
            Mode::Owned,
        ));
    has(&errors(duplicate.build()), |e| {
        matches!(e, PlanError::DuplicateFragment(_))
    });

    let leaf_crossing = Crossing::new(ty(&model, "Leaf"), Direction::Construct);
    let leaf_recipe = leaf_crossing.row(RecipeName::new("whole"));
    let pair_spelling = ty(&model, "Pair").key();
    let bad_spelling = FragmentId::new(pair_spelling.clone(), leaf_recipe.clone());
    let mut spelling = GenerationPlanBuilder::<Fake>::new();
    spelling.fragment(FragmentPlan::new(
        bad_spelling.clone(),
        ty(&model, "Leaf"),
        "Leaf",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&bad_spelling, Mode::Owned, Validity::SelfSufficient),
    ));
    has(&errors(spelling.build()), |e| {
        matches!(e, PlanError::FragmentSpelling(_))
    });

    let bad_crossing = FragmentId::new(pair_spelling, leaf_recipe);
    let mut crossing = GenerationPlanBuilder::<Fake>::new();
    crossing.fragment(FragmentPlan::new(
        bad_crossing.clone(),
        ty(&model, "Pair"),
        "Pair",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&bad_crossing, Mode::Owned, Validity::SelfSufficient),
    ));
    has(&errors(crossing.build()), |e| {
        matches!(e, PlanError::FragmentCrossing(_))
    });

    let mut wrong_yield = GenerationPlanBuilder::<Fake>::new();
    wrong_yield.fragment(FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "Leaf",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        Yield {
            ty: ty(&model, "Pair").key(),
            mode: Mode::Owned,
            validity: Validity::SelfSufficient,
        },
    ));
    has(&errors(wrong_yield.build()), |e| {
        matches!(e, PlanError::YieldType(_))
    });
}

#[test]
fn unknown_fragment_edges_and_fragment_cycles_are_rejected() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let pair = fragment_id(&model, "Pair", Direction::Construct);

    let mut unknown = GenerationPlanBuilder::<Fake>::new();
    unknown.fragment(FragmentPlan::new(
        pair.clone(),
        ty(&model, "Pair"),
        "Pair",
        ConverterPlan::new(
            ShapePlan::Product {
                bridge: FixedArity::new(1, "tuple"),
                parts: vec![FragmentUse::new(
                    leaf.clone(),
                    yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
                )],
            },
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&pair, Mode::Owned, Validity::SelfSufficient),
    ));
    has(&errors(unknown.build()), |e| {
        matches!(e, PlanError::UnknownChild { .. })
    });

    let pair_part = FragmentUse::new(
        leaf.clone(),
        yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
    );
    let leaf_part = FragmentUse::new(
        pair.clone(),
        yield_of(&pair, Mode::Owned, Validity::SelfSufficient),
    );
    let mut cyclic = GenerationPlanBuilder::<Fake>::new();
    cyclic
        .fragment(FragmentPlan::new(
            pair.clone(),
            ty(&model, "Pair"),
            "Pair",
            ConverterPlan::new(
                ShapePlan::Product {
                    bridge: FixedArity::new(1, "tuple"),
                    parts: vec![pair_part],
                },
                NichePlan::none(),
                Failure::Infallible,
                Cleanup::None,
            ),
            yield_of(&pair, Mode::Owned, Validity::SelfSufficient),
        ))
        .fragment(FragmentPlan::new(
            leaf.clone(),
            ty(&model, "Leaf"),
            "Leaf",
            ConverterPlan::new(
                ShapePlan::Product {
                    bridge: FixedArity::new(1, "tuple"),
                    parts: vec![leaf_part],
                },
                NichePlan::none(),
                Failure::Infallible,
                Cleanup::None,
            ),
            yield_of(&leaf, Mode::Owned, Validity::SelfSufficient),
        ));
    has(&errors(cyclic.build()), |e| {
        matches!(e, PlanError::FragmentCycle(_))
    });
}

#[test]
fn site_identity_recipe_contract_and_cleanup_are_checked() {
    let model = model();
    let leaf = fragment_id(&model, "Leaf", Direction::Construct);
    let stored_site = Site {
        owner: syn::parse_str("make_pair").unwrap(),
        role: Role::Return,
    };
    let bound_site = Site {
        owner: syn::parse_str("make_pair").unwrap(),
        role: Role::Param { index: 0 },
    };
    let alternate =
        Crossing::new(ty(&model, "Leaf"), Direction::Construct).row(RecipeName::new("alternate"));
    let bad_site = SitePlan::new(
        SiteId::new(stored_site),
        Bound {
            site: bound_site,
            crossing: Crossing::new(ty(&model, "Pair"), Direction::Deconstruct),
            recipe: alternate,
            origin: Origin::Adapter,
        },
        leaf.clone(),
        Yield {
            ty: ty(&model, "Pair").key(),
            mode: Mode::Owned,
            validity: Validity::SelfSufficient,
        },
        AbiLayout::new(1, "layout"),
        Some("throw"),
        Cleanup::UnlessTransferred("drop"),
    );
    let borrowed_leaf = FragmentPlan::new(
        leaf.clone(),
        ty(&model, "Leaf"),
        "Leaf",
        ConverterPlan::new(
            ShapePlan::Atomic("codec"),
            NichePlan::none(),
            Failure::Infallible,
            Cleanup::None,
        ),
        yield_of(&leaf, Mode::Shared, Validity::Borrowed),
    );
    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder.fragment(borrowed_leaf).site(bad_site);
    let site_errors = errors(builder.build());

    has(&site_errors, |e| matches!(e, PlanError::SiteIdentity(_)));
    has(&site_errors, |e| matches!(e, PlanError::SiteDirection(_)));
    has(&site_errors, |e| matches!(e, PlanError::SiteSpelling(_)));
    has(&site_errors, |e| matches!(e, PlanError::SiteRecipe(_)));
    has(&site_errors, |e| matches!(e, PlanError::ContractType(_)));
    has(&site_errors, |e| matches!(e, PlanError::ContractMode(_)));
    has(&site_errors, |e| {
        matches!(e, PlanError::ContractValidity(_))
    });
    has(&site_errors, |e| {
        matches!(e, PlanError::UnexpectedFailureRoute(_))
    });
    has(&site_errors, |e| {
        matches!(e, PlanError::SiteTransferOfBorrowed(_))
    });

    let duplicate_site = site(&model, &leaf, None, 1);
    let mut duplicate = GenerationPlanBuilder::<Fake>::new();
    duplicate
        .fragment(atomic(
            &model,
            leaf.clone(),
            Failure::Infallible,
            Mode::Owned,
        ))
        .site(duplicate_site)
        .site(site(&model, &leaf, None, 1));
    has(&errors(duplicate.build()), |e| {
        matches!(e, PlanError::DuplicateSite(_))
    });

    let mut unknown = GenerationPlanBuilder::<Fake>::new();
    unknown.site(site(&model, &leaf, None, 1));
    has(&errors(unknown.build()), |e| {
        matches!(e, PlanError::UnknownSiteFragment(_))
    });
}

#[test]
fn unknown_artifact_dependencies_and_inputs_are_rejected() {
    let model = model();
    let unknown_fragment = fragment_id(&model, "Leaf", Direction::Construct);
    let unknown_site = SiteId::new(Site {
        owner: syn::parse_str("missing").unwrap(),
        role: Role::Return,
    });
    let prerequisite = ArtifactId::new("helper", "missing").unwrap();
    let mut builder = GenerationPlanBuilder::<Fake>::new();
    builder.artifact(artifact(
        "wrapper",
        "broken",
        vec![prerequisite],
        vec![
            ArtifactInput::Fragment(unknown_fragment),
            ArtifactInput::Site {
                site: unknown_site,
                slots: 1,
            },
        ],
    ));
    let errors = errors(builder.build());

    has(&errors, |e| {
        matches!(e, PlanError::UnknownPrerequisite { .. })
    });
    has(&errors, |e| {
        matches!(e, PlanError::UnknownArtifactFragment(_))
    });
    has(&errors, |e| matches!(e, PlanError::UnknownArtifactSite(_)));
}
