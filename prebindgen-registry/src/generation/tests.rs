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
    FragmentId::new(&crossing, crossing.row(RecipeName::new("whole"))).unwrap()
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
    let error = FragmentId::new(&pair, leaf.row(RecipeName::new("whole"))).unwrap_err();
    assert!(matches!(error, IdentityError::RecipeCrossing { .. }));
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
