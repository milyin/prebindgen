use super::*;

/// Regression: when a required type is itself unresolved AND has fields
/// that are also unresolved, the diagnostic must list both. Previously
/// `propagate_required` could not cross an unresolved parent (no `subs`
/// edges exist past it), so a missing build.rs declaration for `ZKeyExpr`
/// — only referenced as a field of an unresolved `Outer` — went silent.
#[test]
fn final_invariant_reports_unresolved_field_of_unresolved_struct() {
    use crate::api::core::registry::{Registry, TypeKey};

    let mut reg: Registry<()> = Registry::default();

    // Index a struct `Outer { inner: ZKeyExpr }` so the BFS can walk
    // into its field. `ZKeyExpr` itself stays *unindexed* (the user's
    // build.rs forgot to declare it), but it does appear in the type
    // tables because scan-recursion would have registered it as a field
    // of `Outer`. Simulate the post-scan registry state directly.
    let outer_struct: syn::ItemStruct = syn::parse_str("struct Outer { inner: ZKeyExpr }").unwrap();
    reg.structs.insert(
        outer_struct.ident.clone(),
        (outer_struct, SourceLocation::default()),
    );

    // `Outer` is a required INPUT, unresolved (slot stays `None`).
    let outer_key = TypeKey::parse("Outer").expect("test type");
    reg.input_types.insert(outer_key.clone(), None);
    reg.required_inputs_scan.insert(outer_key.clone());

    // `ZKeyExpr` is also in the type table (scan recursed into the
    // field) but unresolved and NOT marked required at scan time —
    // exactly the case the BFS is here to catch.
    let zke_key = TypeKey::parse("ZKeyExpr").expect("test type");
    reg.input_types.insert(zke_key.clone(), None);

    let err = final_invariant_check(&reg).expect_err("must surface unresolved");
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
    use crate::{
        api::core::registry::{Direction, Registry, TypeEntry, TypeKey},
        SourceLocation as Loc,
    };

    let mut reg: Registry<()> = Registry::default();

    let outer_struct: syn::ItemStruct = syn::parse_str("struct Outer { inner: Inner }").unwrap();
    let inner_struct: syn::ItemStruct =
        syn::parse_str("struct Inner { unused: Unrelated }").unwrap();
    reg.structs
        .insert(outer_struct.ident.clone(), (outer_struct, Loc::default()));
    reg.structs
        .insert(inner_struct.ident.clone(), (inner_struct, Loc::default()));

    // `Outer` required & unresolved; `Inner` RESOLVED (with a dummy
    // entry); `Unrelated` unresolved but only reachable through Inner.
    let outer_key = TypeKey::parse("Outer").expect("test type");
    let inner_key = TypeKey::parse("Inner").expect("test type");
    let unrelated_key = TypeKey::parse("Unrelated").expect("test type");

    reg.input_types.insert(outer_key.clone(), None);
    reg.required_inputs_scan.insert(outer_key.clone());

    reg.input_types.insert(
        inner_key.clone(),
        Some(TypeEntry {
            destination: syn::parse_quote!(i64),
            function: syn::parse_quote!(
                fn __dummy() {}
            ),
            pre_stages: vec![],
            subs: vec![],
            required: false,
            niches: crate::api::core::niches::Niches::empty(),
            metadata: (),
        }),
    );

    reg.input_types.insert(unrelated_key.clone(), None);

    let err = final_invariant_check(&reg).expect_err("must surface Outer");
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
