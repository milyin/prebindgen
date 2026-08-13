use super::*;
use crate::corpus::{Position, SHAPES};
use crate::run::{not_applicable, Target};

/// The gate this crate exists to hold.
///
/// [`tag::tag_of`] makes a new accepted type form a **compile** error here; this
/// makes it a **test** failure until the form also has a fixture. Without the
/// second half the first is cosmetic — a form could be tagged and never
/// enumerated, which is exactly how the hand-written grammar this replaces came
/// to be missing 7 of the 15 forms.
#[test]
fn every_type_form_is_covered() {
    let classified = classify_corpus();
    let missing: Vec<&str> = TypeTag::ALL
        .iter()
        .filter(|tag| !classified.iter().any(|(_, tags)| tags.contains(tag)))
        .map(|tag| tag.as_str())
        .collect();

    assert!(
        missing.is_empty(),
        "no fixture writes these accepted type forms: {missing:?}\n\
         Add a shape to `corpus::SHAPES` that spells one."
    );
}

/// Every corpus entry must actually be the shape it claims to be — a typo in a
/// spelling would otherwise silently contribute nothing and leave the coverage
/// test to be satisfied by something else.
#[test]
fn every_shape_classifies() {
    for (shape, tags) in classify_corpus() {
        assert!(
            !tags.is_empty(),
            "the model does not accept `{}` (shape `{}`), so this fixture tests nothing",
            shape.spelling,
            shape.id
        );
    }
}

/// Shape ids are the receipt key a cell is reported under, so they have to be
/// unique.
#[test]
fn shape_ids_are_unique() {
    let mut ids: Vec<&str> = SHAPES.iter().map(|s| s.id).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate shape id in `corpus::SHAPES`");
}

/// The not-applicable rule is a statement about Rust, not a way to excuse a
/// cell: it may only fire for a declaration position.
#[test]
fn not_applicable_only_in_declarations() {
    for shape in SHAPES {
        for position in [Position::Param, Position::Return] {
            assert!(
                not_applicable(shape, position).is_none(),
                "shape `{}` is excused from the `{}` position; only field and payload \
                 placements may be",
                shape.id,
                position.as_str()
            );
        }
    }
}

/// A cell must always produce an answer — the run must never hang or escape a
/// panic. Cheap smoke over one representative per target.
#[test]
fn a_cell_always_answers() {
    let shape = SHAPES
        .iter()
        .find(|s| s.id == "scalar")
        .expect("scalar shape");
    for target in Target::ALL {
        let state = crate::run::run(shape, Position::Param, *target);
        assert!(
            matches!(state, crate::run::State::PlanSupported),
            "a scalar parameter should cross for {}, got {state:?}",
            target.as_str()
        );
    }
}

/// The committed report is the regression gate; a stale one gates nothing.
#[test]
fn report_is_current() {
    let committed =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(REPORT_PATH))
            .expect("REPORT.md is committed");

    assert_eq!(
        committed,
        report::render(),
        "REPORT.md is out of date — run `cargo run -p shape-matrix`.\n\
         If a cell changed, that is the diff to review: an answer moved."
    );
}
