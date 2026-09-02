use super::*;

/// Regression: when a required type is itself unresolved AND has fields
/// that are also unresolved, the diagnostic must list both. Previously
/// `propagate_required` could not cross an unresolved parent (no `subs`
/// edges exist past it), so a missing build.rs declaration for `ZKeyExpr`
/// — only referenced as a field of an unresolved `Outer` — went silent.
#[test]
fn final_invariant_reports_unresolved_field_of_unresolved_struct() {
    use crate::registry::{Registry, TypeKey};

    // A struct whose field type the build.rs forgot to declare. Registering
    // `Outer` as a root walks into the field through the model, so `ZKeyExpr`
    // gets a cell that is NOT a root — exactly the case the BFS is here to
    // catch. Driven through the real scan rather than simulated, so the state
    // under test is one the pipeline can actually produce.
    let mut reg: Registry =
        crate::test_util::scanned_with(&["pub struct Outer { pub inner: ZKeyExpr }"]);
    let outer = reg
        .intern(
            crate::registry::Direction::Construct,
            &syn::parse_quote!(Outer),
            true,
        )
        .expect("fixture type");
    reg.require_input(&outer);

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
    use crate::registry::{Direction, Registry};

    // Through the real scan, so the state under test is one the pipeline can
    // actually produce: `Unrelated` is a field type nothing declares.
    let mut reg: Registry = crate::test_util::scanned_with(&[
        "pub struct Outer { pub inner: Inner }",
        "pub struct Inner { pub unused: Unrelated }",
    ]);

    // `Outer` required & unresolved; `Inner` RESOLVED (with a dummy
    // entry); `Unrelated` unresolved but only reachable through Inner.
    let outer_ty: syn::Type = syn::parse_quote!(Outer);
    let inner_ty: syn::Type = syn::parse_quote!(Inner);
    let unrelated_ty: syn::Type = syn::parse_quote!(Unrelated);

    reg.insert_crossing(Direction::Construct, &outer_ty, true, None);

    reg.insert_crossing(Direction::Construct, &inner_ty, false, Some(vec![]));

    reg.insert_crossing(Direction::Construct, &unrelated_ty, false, None);

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
    let _ = Direction::Construct; // keep import used
}

/// A type nothing declares directly, reached only through a resolved converter's
/// `subs`, must still fail the build when it has no converter of its own.
///
/// This is the half of the old `required` flag that is derived rather than
/// stored: `Mid` is not a root, and only `required_set`'s walk through `Outer`'s
/// `subs` makes it something a converter must exist for.
#[test]
fn a_type_reachable_only_through_subs_must_still_resolve() {
    use crate::registry::{Registry, TypeKey};

    let mut reg: Registry = Registry::empty();
    let outer: syn::Type = syn::parse_quote!(Outer);
    let mid: syn::Type = syn::parse_quote!(Mid);

    // `Outer` is a root AND resolved — so it is not itself reportable — but its
    // converter delegates to `Mid`.
    reg.insert_crossing(
        Direction::Construct,
        &outer,
        true,
        Some(vec![TypeKey::from_type(&mid)]),
    );
    // `Mid` is present, unresolved, and NOT a root.
    reg.insert_crossing(Direction::Construct, &mid, false, None);

    let err = check_complete(&reg).expect_err("Mid must be reported");
    let ResolveError::Unresolved { entries } = err;
    let reported: std::collections::HashSet<String> =
        entries.iter().map(|e| e.key.to_string()).collect();
    assert_eq!(reported, ["Mid".to_string()].into_iter().collect());
}
