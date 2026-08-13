use super::*;
use crate::{
    corpus::{Position, SHAPES},
    run::{declarations, kind_of, not_applicable, ClassKind, Target},
};

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

/// The same gate, one axis over: this crate's declaration vocabulary **is** the
/// adapter's, not a copy of it.
///
/// `kind_of` is exhaustive over `prebindgen_jni::ClassDecl`, so a fifth class
/// kind is a compile error; the round trip here is what stops the two drifting
/// in the other direction, by proving each kind builds the declaration it
/// claims to. The C side is deliberately not gated this way — its build-script
/// API has no closed kind vocabulary to match on, and is due to be reworked in
/// the JNI style (#192), so `to_c` translates rather than defines.
#[test]
fn class_kind_vocabulary_matches_the_adapter() {
    let probe: syn::Type = syn::parse_quote!(Probe);
    for kind in ClassKind::ALL {
        assert_eq!(
            *kind,
            kind_of(&kind.decl(probe.clone())),
            "`{}` does not build the declaration it names",
            kind.as_str()
        );
    }
}

/// A vocabulary nothing exercises is not coverage. The compile-time half is
/// `kind_of`; this is the half that makes a newly added kind fail until some
/// cell actually declares a type as one.
#[test]
fn every_class_kind_is_exercised() {
    let missing: Vec<&str> = ClassKind::ALL
        .iter()
        .filter(|kind| {
            !SHAPES.iter().any(|shape| {
                Position::ALL.iter().any(|position| {
                    declarations(shape, *position)
                        .iter()
                        .any(|d| d.class == **kind)
                })
            })
        })
        .map(|kind| kind.as_str())
        .collect();

    assert!(
        missing.is_empty(),
        "no cell declares a type as: {missing:?}\n\
         Add a `Need` to `corpus` that declares one."
    );
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
