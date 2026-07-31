use super::*;
use crate::api::{core::registry::TypeEntry, test_util::cell};

/// Regression: when a required type is itself unresolved AND has fields
/// that are also unresolved, the diagnostic must list both. Previously
/// `propagate_required` could not cross an unresolved parent (no `subs`
/// edges exist past it), so a missing build.rs declaration for `ZKeyExpr`
/// — only referenced as a field of an unresolved `Outer` — went silent.
#[test]
fn final_invariant_reports_unresolved_field_of_unresolved_struct() {
    use crate::api::core::registry::{Registry, TypeKey};

    // A struct whose field type the build.rs forgot to declare. Registering
    // `Outer` as a root walks into the field through the model, so `ZKeyExpr`
    // gets a cell that is NOT a root — exactly the case the BFS is here to
    // catch. Driven through the real scan rather than simulated, so the state
    // under test is one the pipeline can actually produce.
    let mut reg: Registry<()> =
        crate::api::test_util::reg_with(&["pub struct Outer { pub inner: ZKeyExpr }"]);
    reg.require_input(&syn::parse_quote!(Outer));

    let zke_key = TypeKey::parse("ZKeyExpr").expect("test type");
    assert!(!reg.input_types[&zke_key].root, "the field is not a root");

    let err = check_complete(&reg).expect_err("must surface unresolved");
    let ResolveError::Unresolved { entries } = err;
    let reported: std::collections::HashSet<String> =
        entries.iter().map(|e| e.key.to_string()).collect();
    assert!(
        reported.contains("Outer"),
        "expected `Outer` in report, got {:?}",
        reported
    );
    assert!(
        reported.contains("ZKeyExpr"),
        "expected `ZKeyExpr` (transitively unresolved via Outer.inner) in report, got {:?}",
        reported
    );
}

/// Counterpart to the regression above: the BFS must NOT walk through
/// resolved nodes. `propagate_required` already covers their `subs`
/// edges, so re-walking them risks reporting deeper unresolved entries
/// that the resolved converter doesn't actually depend on.
#[test]
fn final_invariant_stops_at_resolved_nodes() {
    use crate::api::core::registry::{Direction, Registry, TypeEntry, TypeKey};

    // Through the real scan, so the state under test is one the pipeline can
    // actually produce: `Unrelated` is a field type nothing declares.
    let mut reg: Registry<()> = crate::api::test_util::reg_with(&[
        "pub struct Outer { pub inner: Inner }",
        "pub struct Inner { pub unused: Unrelated }",
    ]);

    // `Outer` required & unresolved; `Inner` RESOLVED (with a dummy
    // entry); `Unrelated` unresolved but only reachable through Inner.
    let outer_key = TypeKey::parse("Outer").expect("test type");
    let inner_key = TypeKey::parse("Inner").expect("test type");
    let unrelated_key = TypeKey::parse("Unrelated").expect("test type");

    reg.input_types
        .insert(outer_key.clone(), cell(&outer_key, true, None));

    reg.input_types.insert(
        inner_key.clone(),
        cell(
            &inner_key,
            false,
            Some(TypeEntry {
                destination: syn::parse_quote!(i64),
                function: syn::parse_quote!(
                    fn __dummy() {}
                ),
                pre_stages: vec![],
                subs: vec![],
                niches: crate::api::core::niches::Niches::empty(),
                metadata: (),
            }),
        ),
    );

    reg.input_types
        .insert(unrelated_key.clone(), cell(&unrelated_key, false, None));

    let err = check_complete(&reg).expect_err("must surface Outer");
    let ResolveError::Unresolved { entries } = err;
    let reported: std::collections::HashSet<String> =
        entries.iter().map(|e| e.key.to_string()).collect();
    assert!(reported.contains("Outer"));
    // Inner is resolved -> not reported.
    assert!(!reported.contains("Inner"));
    // Unrelated sits behind a resolved Inner -> must NOT be reported.
    assert!(
        !reported.contains("Unrelated"),
        "BFS must stop at resolved nodes, got report: {:?}",
        reported
    );
    let _ = Direction::Input; // keep import used
}

/// A type nothing declares directly, reached only through a resolved converter's
/// `subs`, must still fail the build when it has no converter of its own.
///
/// This is the half of the old `required` flag that is derived rather than
/// stored: `Mid` is not a root, and only `required_set`'s walk through `Outer`'s
/// `subs` makes it something a converter must exist for.
#[test]
fn a_type_reachable_only_through_subs_must_still_resolve() {
    use crate::api::{
        core::registry::{Registry, TypeKey},
        test_util::cell,
    };

    let mut reg: Registry<()> = Registry::empty();
    let outer = TypeKey::parse("Outer").expect("test type");
    let mid = TypeKey::parse("Mid").expect("test type");

    // `Outer` is a root AND resolved — so it is not itself reportable — but its
    // converter delegates to `Mid`.
    reg.input_types.insert(
        outer.clone(),
        cell(
            &outer,
            true,
            Some(TypeEntry {
                destination: syn::parse_quote!(i64),
                function: syn::parse_quote!(
                    fn __outer() {}
                ),
                pre_stages: vec![],
                subs: vec![mid.clone()],
                niches: crate::api::core::niches::Niches::empty(),
                metadata: (),
            }),
        ),
    );
    // `Mid` is present, unresolved, and NOT a root.
    reg.input_types.insert(mid.clone(), cell(&mid, false, None));

    let err = check_complete(&reg).expect_err("Mid must be reported");
    let ResolveError::Unresolved { entries } = err;
    let reported: std::collections::HashSet<String> =
        entries.iter().map(|e| e.key.to_string()).collect();
    assert_eq!(reported, ["Mid".to_string()].into_iter().collect());
}
