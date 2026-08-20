use std::collections::BTreeMap;

use super::*;
use crate::{
    corpus::{Position, CALLS, POLICIES, SHAPES},
    guarantees, header,
    run::{declarations, kind_of, not_applicable, ClassKind, Target},
    runtime,
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

/// Every parameter of every call must be a type the model accepts, or the call
/// tests nothing and its `header` cell is measuring a fixture that never
/// contained the shape it names.
#[test]
fn every_call_parameter_classifies() {
    for call in CALLS {
        for param in call.params {
            let reading = syn::parse_str::<syn::Type>(param)
                .ok()
                .and_then(|ty| crate::corpus_model_classify(&ty));
            assert!(
                reading.is_some(),
                "the model does not accept `{param}` in call `{}`",
                call.id
            );
        }
    }
}

/// A policy case must name a shape that exists and must actually vary
/// something: a case whose override matches the canonical declaration is a row
/// duplicating the one beside it, proving nothing while looking like evidence.
#[test]
fn every_policy_varies_a_declaration() {
    for policy in POLICIES {
        let shape = SHAPES
            .iter()
            .find(|s| s.id == policy.shape)
            .unwrap_or_else(|| panic!("policy case names unknown shape `{}`", policy.shape));
        assert!(
            !shape.needs.is_empty(),
            "policy case `{}` re-declares a shape that declares no types",
            policy.shape
        );
        let canonical = crate::run::declarations(shape, policy.position);
        assert!(
            canonical
                .iter()
                .any(|d| !d.error && d.class != policy.class),
            "policy case `{}` in the `{}` position declares everything exactly as the \
             canonical run does, so its row repeats the one beside it",
            policy.shape,
            policy.position.as_str()
        );
    }
}

/// A runtime case must name a call the corpus defines, and no two cases may
/// share an id: the id is the test's name, and the harness's result line for
/// that name is the only thing that says the case holds.
#[test]
fn every_runtime_case_names_a_live_call() {
    let mut ids: Vec<&str> = Vec::new();
    for case in runtime::CASES {
        assert!(
            CALLS.iter().any(|c| c.id == case.call),
            "runtime case `{}` names unknown call `{}`",
            case.id,
            case.call
        );
        ids.push(case.id);
    }
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(before, ids.len(), "two runtime cases share an id");
}

/// A cell id is a file name, a receipt key and a guarantee key at once, so a
/// collision would silently merge two cells — one of them reporting the other's
/// compiler diagnostics and inheriting the other's floor.
#[test]
fn every_cell_id_is_unique() {
    let mut ids: Vec<String> = report::survey().iter().map(|c| c.id()).collect();
    let before = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(before, ids.len(), "two cells share an id");
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
                "shape `{}` is excused from the `{}` position; only a placement that \
                 must be written as Rust — a declaration's field or payload, or the \
                 argument of an `impl Fn` — may be",
                shape.id,
                position.as_str()
            );
        }
    }
}

/// A callback argument is excused for exactly one reason, and the excuse is
/// about **Rust**, not about the generators.
///
/// `impl Fn(impl Fn(..))` does not parse, so a callback shape has no cell in
/// that position. Every other shape does — an excuse that grew to cover a shape
/// a generator merely refuses would turn a rejection into a blank.
#[test]
fn a_callback_argument_is_excused_only_for_impl_trait() {
    for shape in SHAPES {
        let excused = not_applicable(shape, Position::Callback).is_some();
        assert_eq!(
            excused,
            shape.spelling.contains("impl Fn"),
            "shape `{}` is excused from the callback-argument position on the wrong \
             grounds — only a nested `impl Trait` is not writable there",
            shape.id,
        );
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
        let outcome = crate::run::run(shape, Position::Param, *target, "smoke__param__x");
        assert!(
            matches!(outcome.state, crate::run::State::PlanSupported),
            "a scalar parameter should cross for {}, got {:?}",
            target.as_str(),
            outcome.state
        );
        assert!(
            outcome.emitted.is_some(),
            "a cell that generated produced no Rust to check"
        );
    }
}

/// The compile check must attribute per cell, and must actually discriminate.
///
/// It would be easy to write a check that marks everything as compiling: no
/// diagnostic ever matches, every cell passes, and the column becomes
/// decoration. So this feeds it one unit that compiles and one that cannot, in
/// the same crate, and requires it to separate them — which also pins the
/// attribution path, since the only thing linking a diagnostic to a cell is the
/// file rustc names.
#[test]
fn the_compile_check_separates_good_from_bad() {
    let checked = check::check(
        "selftest",
        &[
            check::Unit {
                id: "selftest_good__param__jni".to_string(),
                fixture: "pub fn probe(v: u64) -> u64 { v }".to_string(),
                emitted: "pub fn wrapper(v: u64) -> u64 { flat::probe(v) }".to_string(),
            },
            check::Unit {
                id: "selftest_bad__param__jni".to_string(),
                fixture: "pub fn probe(v: u64) -> u64 { v }".to_string(),
                // A type error in emitted code, of the kind a generator makes: the
                // wrapper hands a string to a function taking an integer.
                emitted: "pub fn wrapper() -> u64 { flat::probe(\"not a u64\") }".to_string(),
            },
        ],
    )
    .expect("the check runs");

    assert!(
        checked.compiled.contains("selftest_good__param__jni"),
        "a unit that compiles was not recorded as compiling"
    );
    assert!(
        !checked.compiled.contains("selftest_bad__param__jni"),
        "a unit that does not compile was recorded as compiling — the check is \
         reporting a state it did not establish"
    );
    assert!(
        checked.failed.contains_key("selftest_bad__param__jni"),
        "the failing unit produced no attributed diagnostic, so nothing links a \
         compiler error to the cell it belongs to"
    );
}

/// The Kotlin stage must separate, and must attribute.
///
/// Same argument as the compile check's: a stage whose diagnostics never match
/// records every cell as passing, and the column becomes decoration. So this
/// feeds it one cell whose Kotlin is a program and one whose Kotlin is not, in
/// the same compiler pass, and requires both the verdicts and the attribution —
/// the only thing tying a diagnostic to a cell is the directory the sources were
/// written under, which is the cell id.
#[test]
fn the_kotlin_stage_separates_good_from_bad() {
    let unit = |id: &str, source: &str| crate::kotlin::Unit {
        id: id.to_string(),
        files: vec![crate::run::KotlinFile {
            path: "probe.kt".to_string(),
            source: source.to_string(),
        }],
    };
    let compiled = crate::kotlin::compile(
        "selftest",
        &[
            unit(
                "selftest_good__param__jni",
                "package selftest.good\npublic fun probe(v: Long): Long = v\n",
            ),
            unit(
                // A type error of the kind a generator makes: a nullable value
                // handed to something that requires a non-null one.
                "selftest_bad__param__jni",
                "package selftest.bad\npublic fun probe(v: Long?): Long = v\n",
            ),
        ],
    )
    .expect("the Kotlin check runs — is kotlinc on PATH?");

    assert!(
        compiled.ok.contains("selftest_good__param__jni"),
        "a cell whose Kotlin compiles was not recorded as compiling"
    );
    assert!(
        !compiled.ok.contains("selftest_bad__param__jni"),
        "a cell whose Kotlin does not compile was recorded as compiling — the \
         stage is reporting a state it did not establish"
    );
    assert!(
        compiled.failed.contains_key("selftest_bad__param__jni"),
        "the failing cell produced no attributed diagnostic, so nothing links a \
         Kotlin error to the cell it belongs to"
    );
}

/// The header stage must discriminate, and on the right thing.
///
/// "cbindgen returned `Ok`" would pass for a header declaring nothing at all,
/// so the receipt is that the wrapper is *declared*. The negative control is
/// the same Rust with the `extern "C"` removed: still valid Rust, still parsed
/// happily, and of no use whatsoever to a C caller.
#[test]
fn the_header_stage_requires_a_declaration() {
    let exported = r#"#[no_mangle] pub unsafe extern "C" fn probe(v: u64) -> u64 { v }"#;
    assert!(
        header::generate(exported, "probe").is_ok(),
        "an exported wrapper was not found in the header cbindgen produced"
    );

    let not_exported = "pub fn probe(v: u64) -> u64 { v }";
    assert!(
        !header::generate(not_exported, "probe").is_ok(),
        "a function C cannot call was reported as declared — the stage is \
         reporting a state it did not establish"
    );
}

/// The ratchet, against the floors this repository actually commits.
#[test]
fn no_cell_falls_below_its_guarantee() {
    let regressions = guarantees::regressions(report::survey());
    assert!(
        regressions.is_empty(),
        "cells got worse:\n{}\n\nIf this is deliberate — a shape given up on, \
         or a declaration that changed meaning — lower the floor in {} by hand \
         in the same commit, so the decision appears in the diff.",
        regressions
            .iter()
            .map(|r| format!("  {r}"))
            .collect::<Vec<_>>()
            .join("\n"),
        guarantees::PATH
    );
}

/// The ratchet must catch a fall, and must not catch a rise.
///
/// Otherwise it is a file nobody can tell is working: the committed floors are
/// all satisfied, so the passing gate above proves only that nothing regressed
/// *today*, not that a regression would be noticed.
#[test]
fn the_ratchet_catches_a_fall_and_ignores_a_rise() {
    use guarantees::Level;
    let floors = BTreeMap::from([
        ("fell__param__c".to_string(), Level::Header),
        ("rose__param__jni".to_string(), Level::Generates),
        ("held__param__jni".to_string(), Level::Compiles),
    ]);
    let observed = BTreeMap::from([
        ("fell__param__c".to_string(), Level::Compiles),
        ("rose__param__jni".to_string(), Level::Compiles),
        ("held__param__jni".to_string(), Level::Compiles),
    ]);

    let found = guarantees::regressions_of(&floors, &observed);
    let ids: Vec<&str> = found.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["fell__param__c"],
        "the ratchet must report exactly the cell that fell"
    );
}

/// Raising is automatic; lowering is not. A run that does worse than the
/// committed floor must leave that floor standing, or the ratchet would follow
/// the regression down and hold nothing.
#[test]
fn updating_never_lowers_a_floor() {
    use guarantees::Level;
    let floors = BTreeMap::from([("cell__param__c".to_string(), Level::Header)]);
    let observed = BTreeMap::from([("cell__param__c".to_string(), Level::Generates)]);

    let after = guarantees::raised(floors, &observed);
    assert_eq!(
        after.get("cell__param__c"),
        Some(&Level::Header),
        "a floor followed a cell downwards"
    );
}

/// Every floor must name a cell the matrix actually enumerates. A stale entry —
/// a renamed shape, a dropped position — would sit in the file being satisfied
/// by nothing at all.
#[test]
fn every_guarantee_names_a_live_cell() {
    let observed = guarantees::observed(report::survey());
    let committed = guarantees::committed();
    let stale: Vec<&String> = committed
        .keys()
        .filter(|id| !observed.contains_key(*id))
        .collect();
    assert!(
        stale.is_empty(),
        "these floors name cells that no longer exist: {stale:?}\n\
         Remove them, or restore the cell they were about."
    );
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
