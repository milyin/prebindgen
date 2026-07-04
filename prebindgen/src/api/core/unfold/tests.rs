use quote::ToTokens;

use super::*;
use crate::api::{core::types_util::ident, test_util::reg_with};

/// A generous `.fun_accessor` set covering every function used as a
/// deconstructor record across these tests (a superset is fine — `apply`
/// only checks records are members). The `nested_record_*` tests that
/// exercise the gate's *rejection* pass an explicit smaller set instead.
fn acc_set() -> std::collections::HashSet<syn::Ident> {
    [
        "a_to_b",
        "b_to_a",
        "wrong",
        "z_error_message",
        "z_keyexpr_as_str",
        "z_reply_replier_zid",
        "z_reply_is_ok",
        "z_reply_sample",
        "z_reply_err",
        "z_reply_error_payload",
        "z_sample_key_expr",
        "z_sample_payload",
        "z_sample_encoding",
        "z_sample_kind",
        "z_sample_timestamp",
        "z_sample_express",
        "z_sample_priority",
        "z_sample_congestion_control",
        "z_sample_attachment",
        "z_timestamp_ntp64",
        "z_zbytes_to_bytes",
        "z_zenoh_id_to_string",
        "z_encoding_to_string",
    ]
    .iter()
    .map(|s| ident(s))
    .collect()
}

/// [`acc_set`] minus the decomposed fn: the default auto-apply skips
/// accessor fns, and some tests decompose a fn that doubles as a record
/// accessor elsewhere in the shared set.
fn acc_set_without(f: &str) -> std::collections::HashSet<syn::Ident> {
    let mut s = acc_set();
    s.remove(&ident(f));
    s
}

#[test]
fn accessor_optional_primitive() {
    // M2: `z_sample_timestamp(&ZSample) -> Option<&ZTimestamp>` decomposed
    // into a single primitive leaf `z_timestamp_ntp64(&ZTimestamp) -> i64`
    // (no identity). Outer shape is `Optional(Decompose)`.
    let mut reg = reg_with(&[
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
    acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");

    apply(
        &mut reg,
        &acc,
        &[ident("z_sample_timestamp")].into_iter().collect(),
        &acc_set_without("z_sample_timestamp"),
    )
    .expect("apply");

    let plan = reg
        .unfold_plans
        .get(&ident("z_sample_timestamp"))
        .expect("plan");
    assert!(plan.by_ref, "inner was &ZTimestamp");
    assert_eq!(plan.source.to_token_stream().to_string(), "ZTimestamp");
    assert!(
        matches!(&plan.shape, UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)),
        "outer shape is Optional(Decompose)"
    );
    assert_eq!(plan.leaves.len(), 1);
    assert!(!plan.leaves[0].identity);
    assert_eq!(plan.leaves[0].path[0].to_string(), "z_timestamp_ntp64");
    assert_eq!(plan.leaves[0].out_ty.to_token_stream().to_string(), "i64");
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(i64))));
}

#[test]
fn accessor_plan_byref() {
    // `z_sample_key_expr(&ZSample) -> &ZKeyExpr` decomposed into the keyexpr
    // handle (identity) + its string form (`z_keyexpr_as_str`).
    let mut reg = reg_with(&[
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");

    apply(
        &mut reg,
        &acc,
        &[ident("z_sample_key_expr")].into_iter().collect(),
        &acc_set_without("z_sample_key_expr"),
    )
    .expect("apply");

    let plan = reg
        .unfold_plans
        .get(&ident("z_sample_key_expr"))
        .expect("plan");
    assert!(plan.by_ref, "return was &ZKeyExpr");
    assert_eq!(plan.source.to_token_stream().to_string(), "ZKeyExpr");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.leaves.len(), 2);

    // Identity leaf: out_ty `&ZKeyExpr`, empty path, emitted last.
    assert!(plan.leaves[0].identity);
    assert!(plan.leaves[0].path.is_empty());
    assert_eq!(
        plan.leaves[0].out_ty.to_token_stream().to_string(),
        "& ZKeyExpr"
    );
    // Accessor leaf: out_ty `&str`, path `[z_keyexpr_as_str]`.
    assert!(!plan.leaves[1].identity);
    assert_eq!(plan.leaves[1].path.len(), 1);
    assert_eq!(plan.leaves[1].path[0].to_string(), "z_keyexpr_as_str");
    assert_eq!(plan.leaves[1].out_ty.to_token_stream().to_string(), "& str");

    // Leaf out_tys registered as required outputs so the resolver builds
    // their converters.
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(&str))));
}

#[test]
fn root_identity_before_nested_identity_errors() {
    // Owned return: the root `.field_self()` MOVES the value, a nested
    // identity (spliced ZKeyExpr handle) borrows it — id-first is the
    // order that would generate non-compiling Rust, caught at apply time.
    let mut reg = reg_with(&[
        "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
        "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
    ]);
    let accessors: std::collections::HashSet<syn::Ident> =
        ["z_query_key_expr"].iter().map(|s| ident(s)).collect();
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZQuery));
    acc.add_deconstructor_record_id(); // root identity FIRST — wrong
    acc.add_deconstructor_record(ident("z_query_key_expr"), "key_expr");
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_take_query")].into_iter().collect(),
        &accessors,
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::RootIdentityBeforeNested { .. }));

    // Root identity LAST (the zenoh `Query` shape) is accepted.
    let mut reg2 = reg_with(&[
        "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
        "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
    ]);
    let mut acc2 = Deconstructors::default();
    acc2.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc2.add_deconstructor_record_id();
    acc2.ensure_default_deconstructor(syn::parse_quote!(ZQuery));
    acc2.add_deconstructor_record(ident("z_query_key_expr"), "key_expr");
    acc2.add_deconstructor_record_id(); // root identity last — ok
    apply(
        &mut reg2,
        &acc2,
        &[ident("z_take_query")].into_iter().collect(),
        &accessors,
    )
    .expect("root identity last is the supported order");
}

#[test]
fn accessor_target_mismatch_errors() {
    // Accessor takes a different type than the accessor's target.
    let mut reg = reg_with(&[
        "fn z_foo() -> ZKeyExpr { todo!() }",
        "fn wrong(x: &ZSample) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record(ident("wrong"), "wrong");
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::AccessorTargetMismatch { .. }));
}

#[test]
fn multiple_identity_errors() {
    let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record_id();
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::MultipleIdentity { .. }));
}

#[test]
fn record_must_be_fun_accessor() {
    // A deconstructor record referencing a non-`.fun_accessor` fn errors.
    let mut reg = reg_with(&[
        "fn z_foo(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    // Empty accessor set ⇒ z_keyexpr_as_str is not a fun_accessor ⇒ error.
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::RecordNotAccessor { .. }));
    // With it declared as an accessor, the gate passes.
    let accset: std::collections::HashSet<syn::Ident> =
        ["z_keyexpr_as_str"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &Default::default(), &accset).expect("gate passes");
}

#[test]
fn duplicate_leaf_name_errors() {
    // Two records of one deconstructor given the same literal name ⇒ hard
    // error (names are emitted verbatim; never auto-disambiguated).
    let mut reg = reg_with(&[
        "fn z_foo() -> ZSample { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
        "fn z_sample_payload(s: &ZSample) -> Vec<u8> { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "field");
    acc.add_deconstructor_record(ident("z_sample_payload"), "field");
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(
        matches!(err, UnfoldError::DuplicateLeafName { .. }),
        "{err:?}"
    );
}

#[test]
fn reserved_separator_in_name_errors() {
    // A record name containing the reserved `"__"` chain separator ⇒ error.
    let mut reg = reg_with(&[
        "fn z_foo() -> ZSample { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "key__expr");
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(
        matches!(err, UnfoldError::ReservedSeparator { .. }),
        "{err:?}"
    );
}

#[test]
fn nested_accessor_flatten() {
    // M3: `z_reply_sample -> Option<&ZSample>` whose ZSample combined
    // accessor nests ZKeyExpr (handle+string), ZZBytes (bytes), and a
    // nullable ZTimestamp (Option<&ZTimestamp> → ntp64), plus a direct enum
    // leaf. Verifies path prefixes + nullable propagation.
    let mut reg = reg_with(&[
        "fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_payload(s: &ZSample) -> &ZZBytes { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        "fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    // Child accessors (reused via nesting).
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZZBytes));
    acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
    acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
    // Parent accessor with nested + direct records.
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
    acc.add_deconstructor_record(ident("z_sample_payload"), "z_sample_payload");
    acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
    acc.add_deconstructor_record(ident("z_sample_timestamp"), "z_sample_timestamp");

    apply(
        &mut reg,
        &acc,
        &[ident("z_reply_sample")].into_iter().collect(),
        &acc_set_without("z_reply_sample"),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");
    assert!(plan.by_ref);
    assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
    assert!(matches!(&plan.shape, UnfoldShape::Optional((), _)));

    let path = |l: &UnfoldLeaf| {
        l.path
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
    };
    // keyexpr identity (path [z_sample_key_expr]) + string + payload bytes
    // + kind enum + nullable timestamp ntp64.
    assert_eq!(plan.leaves.len(), 5);
    assert!(plan.leaves[0].identity);
    assert_eq!(path(&plan.leaves[0]), "z_sample_key_expr");
    assert_eq!(path(&plan.leaves[1]), "z_sample_key_expr.z_keyexpr_as_str");
    assert_eq!(path(&plan.leaves[2]), "z_sample_payload.z_zbytes_to_bytes");
    assert_eq!(path(&plan.leaves[3]), "z_sample_kind");
    assert_eq!(
        plan.leaves[3].out_ty.to_token_stream().to_string(),
        "SampleKind"
    );
    assert_eq!(
        path(&plan.leaves[4]),
        "z_sample_timestamp.z_timestamp_ntp64"
    );
    // Only the timestamp leaf (Option nesting accessor) is nullable.
    assert!(!plan.leaves[1].nullable && !plan.leaves[2].nullable);
    assert!(plan.leaves[4].nullable);
}

#[test]
fn reply_product_double_option_flatten() {
    // ZReply-shaped product (Result<Sample, ReplyError> decomposed in the
    // current product model): the root's records include two
    // `Option<&Child>` nesting accessors (`z_reply_sample`, `z_reply_err`)
    // whose children themselves contain `Option` nesting steps and a
    // nested identity — the double-unwrap case — plus an
    // `Option<ZZenohId>` Acc record with NO default child, which keeps
    // the full `Option<…>` as its leaf `out_ty` (its own `Option` is the
    // converter's business, not a nesting step ⇒ NOT nullable).
    let mut reg = reg_with(&[
        "fn z_recv_reply(q: &ZQuery) -> ZReply { todo!() }",
        "fn z_reply_replier_zid(r: &ZReply) -> Option<ZZenohId> { todo!() }",
        "fn z_reply_is_ok(r: &ZReply) -> bool { todo!() }",
        "fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { todo!() }",
        "fn z_reply_err(r: &ZReply) -> Option<&ZReplyError> { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
        "fn z_reply_error_payload(e: &ZReplyError) -> &ZZBytes { todo!() }",
        "fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
    acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZZBytes));
    acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
    acc.add_deconstructor_record(ident("z_sample_timestamp"), "z_sample_timestamp");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZReplyError));
    acc.add_deconstructor_record(ident("z_reply_error_payload"), "z_reply_error_payload");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZReply));
    acc.add_deconstructor_record(ident("z_reply_replier_zid"), "z_reply_replier_zid");
    acc.add_deconstructor_record(ident("z_reply_is_ok"), "z_reply_is_ok");
    acc.add_deconstructor_record(ident("z_reply_sample"), "z_reply_sample");
    acc.add_deconstructor_record(ident("z_reply_err"), "z_reply_err");

    apply(
        &mut reg,
        &acc,
        &[ident("z_recv_reply")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg.unfold_plans.get(&ident("z_recv_reply")).expect("plan");
    assert!(!plan.by_ref, "owned ZReply return");
    assert_eq!(plan.source.to_token_stream().to_string(), "ZReply");
    assert!(matches!(&plan.shape, UnfoldShape::Base));
    assert!(matches!(plan.delivery, Delivery::Callback));

    let path = |l: &UnfoldLeaf| {
        l.path
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
    };
    assert_eq!(plan.leaves.len(), 6);
    // Acc leaf keeping its full `Option<…>` return — not a nesting step.
    assert_eq!(path(&plan.leaves[0]), "z_reply_replier_zid");
    assert_eq!(
        plan.leaves[0].out_ty.to_token_stream().to_string(),
        "Option < ZZenohId >"
    );
    assert!(!plan.leaves[0].nullable && !plan.leaves[0].identity);
    assert_eq!(path(&plan.leaves[1]), "z_reply_is_ok");
    assert!(!plan.leaves[1].nullable);
    // Ok-arm leaves: spliced through the `Option`-returning
    // `z_reply_sample` ⇒ all nullable, incl. the nested keyexpr identity
    // and the doubly-`Option` timestamp path.
    assert!(plan.leaves[2].identity);
    assert_eq!(path(&plan.leaves[2]), "z_reply_sample.z_sample_key_expr");
    assert!(plan.leaves[2].nullable);
    assert_eq!(
        path(&plan.leaves[3]),
        "z_reply_sample.z_sample_key_expr.z_keyexpr_as_str"
    );
    assert!(plan.leaves[3].nullable);
    assert_eq!(
        path(&plan.leaves[4]),
        "z_reply_sample.z_sample_timestamp.z_timestamp_ntp64"
    );
    assert!(plan.leaves[4].nullable);
    // Err-arm leaf: spliced through `z_reply_err`.
    assert_eq!(
        path(&plan.leaves[5]),
        "z_reply_err.z_reply_error_payload.z_zbytes_to_bytes"
    );
    assert!(plan.leaves[5].nullable);
}

#[test]
fn nested_cycle_errors() {
    // A → B → A nesting is rejected.
    let mut reg = reg_with(&[
        "fn z_foo() -> ZA { todo!() }",
        "fn a_to_b(a: &ZA) -> &ZB { todo!() }",
        "fn b_to_a(b: &ZB) -> &ZA { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZA));
    acc.add_deconstructor_record(ident("a_to_b"), "a_to_b");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZB));
    acc.add_deconstructor_record(ident("b_to_a"), "b_to_a");
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::Cycle { .. }));
}

#[test]
fn iterable_whole_element_plan() {
    // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
    // each element delivered WHOLE (no accessor, no leaves): a per-fn
    // flatten with an empty record list on an element type that has no
    // deconstructor of its own.
    let mut reg = reg_with(&["fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.begin_inline_output(ident("z_session_peers_zid"));

    apply(
        &mut reg,
        &acc,
        &[ident("z_session_peers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_session_peers_zid"))
        .expect("plan");
    assert!(
        matches!(&plan.shape, UnfoldShape::Iterable(inner) if matches!(**inner, UnfoldShape::Base)),
        "outer shape is Iterable(Decompose)"
    );
    assert!(!plan.by_ref, "Vec<ZZenohId> owns its elements");
    assert!(
        plan.leaves.is_empty(),
        "whole-element: no decomposed leaves"
    );
    assert_eq!(
        plan.element
            .as_ref()
            .map(|t| t.to_token_stream().to_string()),
        Some("ZZenohId".to_string())
    );
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(ZZenohId))));
}

#[test]
fn iterable_decomposed_plan() {
    // M5: `z_session_peers_zid -> Vec<ZZenohId>` with a ZZenohId combined
    // accessor → Iterable with per-element leaves: the string form + the
    // value itself via `record_id` (a `value_blob` identity, owned at the
    // root since `Vec<ZZenohId>` owns its elements).
    let mut reg = reg_with(&[
        "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZZenohId));
    acc.add_deconstructor_record(ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
    acc.add_deconstructor_record_id();

    apply(
        &mut reg,
        &acc,
        &[ident("z_session_peers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_session_peers_zid"))
        .expect("plan");
    assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
    assert!(plan.element.is_none(), "decomposed: element not used");
    assert_eq!(plan.leaves.len(), 2);
    assert_eq!(plan.leaves[0].path[0].to_string(), "z_zenoh_id_to_string");
    assert_eq!(
        plan.leaves[0].out_ty.to_token_stream().to_string(),
        "String"
    );
    // Identity leaf: owned value (`ZZenohId`, not `&ZZenohId`) since the Vec
    // owns its elements (by_ref = false).
    assert!(plan.leaves[1].identity);
    assert!(plan.leaves[1].path.is_empty());
    assert_eq!(
        plan.leaves[1].out_ty.to_token_stream().to_string(),
        "ZZenohId"
    );
}

#[test]
fn convert_output_single_value() {
    // `.converter(ZTimestamp, z_timestamp_ntp64)` + `.convert_output()` on
    // `z_sample_timestamp -> Option<&ZTimestamp>` ⇒ Return delivery, single
    // leaf, convert_out_ty = Option<i64>.
    let mut reg = reg_with(&[
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
    acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");

    apply(
        &mut reg,
        &acc,
        &[ident("z_sample_timestamp")].into_iter().collect(),
        &acc_set_without("z_sample_timestamp"),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_sample_timestamp"))
        .expect("plan");
    assert_eq!(plan.delivery, Delivery::Return);
    assert!(matches!(&plan.shape, UnfoldShape::Optional((), _)));
    assert_eq!(plan.leaves.len(), 1);
    assert_eq!(
        plan.convert_out_ty
            .as_ref()
            .map(|t| t.to_token_stream().to_string()),
        Some("Option < i64 >".to_string())
    );
    // The shaped convert type is registered as a required output.
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(Option<i64>))));
}

#[test]
fn multi_leaf_output_is_callback() {
    // A two-record deconstructor (handle + string) ⇒ Callback delivery (>1 leaf).
    let mut reg = reg_with(&[
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    apply(
        &mut reg,
        &acc,
        &[ident("z_sample_key_expr")].into_iter().collect(),
        &acc_set_without("z_sample_key_expr"),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_sample_key_expr"))
        .expect("plan");
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 2);
    assert!(plan.convert_out_ty.is_none());
}

#[test]
fn vec_output_is_iterable_callback() {
    // A `Vec` return ⇒ Iterable + Callback (a fold), never a single Return.
    let mut reg = reg_with(&[
        "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZZenohId));
    acc.add_deconstructor_record(ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
    apply(
        &mut reg,
        &acc,
        &[ident("z_session_peers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg
        .unfold_plans
        .get(&ident("z_session_peers_zid"))
        .expect("plan");
    assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
    assert_eq!(plan.delivery, Delivery::Callback);
}

#[test]
fn value_struct_vec_is_fixed_iterable_fold() {
    // A by-value `data_class` returned as `Option<Vec<T>>` (perftest's
    // `storage_get_vec` contract) synthesizes a FIXED-BUILDER fold wrapped in
    // an Optional layer: the field leaves cross raw per element and the
    // foreign folder rebuilds + appends them (no Java object is built on the
    // Rust side); `None` ⇒ a null list. Closes the data_class→Vec milestone.
    let mut reg =
        reg_with(&["fn storage_get_vec(s: &Storage) -> Option<Vec<Payload>> { todo!() }"]);
    let leaf = |name: &str, ty: syn::Type| UnfoldLeaf {
        name: name.to_string(),
        path: vec![ident(name)],
        out_ty: ty,
        identity: false,
        nullable: false,
        source: LeafSource::Field,
    };
    let vd = ValueDecon {
        key: TypeKey::from_type(&syn::parse_quote!(Payload)),
        source: syn::parse_quote!(Payload),
        leaves: vec![
            leaf("id", syn::parse_quote!(i64)),
            leaf("seq", syn::parse_quote!(i32)),
        ],
    };
    let declared: std::collections::HashSet<syn::Ident> =
        ["storage_get_vec"].iter().map(|s| ident(s)).collect();
    apply_value_structs(&mut reg, vec![vd], &declared).expect("apply_value_structs");

    let plan = reg
        .unfold_plans
        .get(&ident("storage_get_vec"))
        .expect("fixed-builder fold plan");
    assert!(plan.fixed_builder, "Vec<data_class> ⇒ fixed builder");
    assert!(
        matches!(&plan.shape,
                UnfoldShape::Optional((), inner)
                    if matches!(&**inner, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base))),
        "Option<Vec<T>> ⇒ Optional(Iterable(Base))"
    );
    assert_eq!(plan.delivery, Delivery::Callback);
    assert!(plan.decon.is_some(), "carries the field decon");
    assert!(
        plan.element.is_none(),
        "decomposed-leaf fold, not whole-element"
    );
    assert_eq!(plan.leaves.len(), 2, "field leaves cross raw per element");
    assert!(plan.leaves.iter().all(|l| l.source == LeafSource::Field));
    assert!(!plan.by_ref, "owned Vec<Payload> elements");
}

#[test]
fn value_struct_slice_callback_is_fixed_iterable_fold() {
    // An `impl Fn(&[data_class])` callback arg (perftest's
    // `storage_callback_vec`) synthesizes an Iterable fixed-folder
    // `callback_arg_plans` entry keyed by the `&[Payload]` arg: the
    // trampoline folds each element's field leaves into a foreign list, the
    // user callback still sees the whole `List<Payload>`.
    let mut reg = reg_with(&[
        "fn storage_callback_vec(f: impl Fn(&[Payload]) + Send + Sync + 'static) { todo!() }",
    ]);
    let leaf = |name: &str, ty: syn::Type| UnfoldLeaf {
        name: name.to_string(),
        path: vec![ident(name)],
        out_ty: ty,
        identity: false,
        nullable: false,
        source: LeafSource::Field,
    };
    let vd = ValueDecon {
        key: TypeKey::from_type(&syn::parse_quote!(Payload)),
        source: syn::parse_quote!(Payload),
        leaves: vec![
            leaf("id", syn::parse_quote!(i64)),
            leaf("seq", syn::parse_quote!(i32)),
        ],
    };
    let declared: std::collections::HashSet<syn::Ident> =
        ["storage_callback_vec"].iter().map(|s| ident(s)).collect();
    apply_value_structs(&mut reg, vec![vd], &declared).expect("apply_value_structs");

    let key = TypeKey::from_type(&syn::parse_quote!(&[Payload]));
    let plan = reg
        .callback_arg_plans
        .get(&key)
        .expect("slice callback-arg fold plan");
    assert!(plan.fixed_builder, "&[data_class] ⇒ fixed folder");
    assert!(
        matches!(&plan.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)),
        "&[T] ⇒ Iterable(Base)"
    );
    assert_eq!(plan.delivery, Delivery::Callback);
    assert!(plan.decon.is_some(), "carries the field decon");
    assert!(plan.element.is_none(), "decomposed-leaf fold");
    assert_eq!(plan.leaves.len(), 2);
    assert!(plan.leaves.iter().all(|l| l.source == LeafSource::Field));
    // A scalar `&Payload` callback arg must stay a Base fixed builder.
    let mut reg2 = reg_with(&[
        "fn storage_callback(f: impl Fn(&Payload) + Send + Sync + 'static) { todo!() }",
    ]);
    let vd2 = ValueDecon {
        key: TypeKey::from_type(&syn::parse_quote!(Payload)),
        source: syn::parse_quote!(Payload),
        leaves: vec![leaf("id", syn::parse_quote!(i64))],
    };
    let declared2: std::collections::HashSet<syn::Ident> =
        ["storage_callback"].iter().map(|s| ident(s)).collect();
    apply_value_structs(&mut reg2, vec![vd2], &declared2).expect("apply_value_structs");
    let scalar = reg2
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(&Payload)))
        .expect("scalar callback-arg plan");
    assert!(matches!(scalar.shape, UnfoldShape::Base), "&T ⇒ Base");
}

#[test]
fn convert_error_decomposes_result_e() {
    // The ZError deconstructor (`z_error_message`) auto-applies to every fn
    // returning `Result<_, ZError>`, storing the plan in `error_plans`. Error
    // delivery is always Callback (its leaves are the `ze` callback args).
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, ZError> { todo!() }",
        "fn z_error_message(e: &ZError) -> String { todo!() }",
        "fn z_infallible(s: &ZSample) -> bool { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZError));
    acc.add_deconstructor_record(ident("z_error_message"), "z_error_message");
    let declared: std::collections::HashSet<syn::Ident> = ["z_keyexpr_try_from", "z_infallible"]
        .iter()
        .map(|s| ident(s))
        .collect();
    let accset: std::collections::HashSet<syn::Ident> =
        ["z_error_message"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &accset).expect("apply");

    let plan = reg
        .error_plans
        .get(&ident("z_keyexpr_try_from"))
        .expect("error plan for the fallible fn");
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 1);
    assert_eq!(
        plan.leaves[0].out_ty.to_token_stream().to_string(),
        "String"
    );
    assert_eq!(plan.source.to_token_stream().to_string(), "ZError");
    // The infallible fn gets no error plan.
    assert!(!reg.error_plans.contains_key(&ident("z_infallible")));
    // No output plans created (no ZKeyExpr return among the declared fns; the
    // ZError deconstructor only matches the Result error position).
    assert!(reg.unfold_plans.is_empty());
}

#[test]
fn default_output_applies_to_owned_and_borrow_returns() {
    // Default-everywhere: the ZKeyExpr deconstructor auto-applies to BOTH a
    // `&ZKeyExpr` (borrow) and an owned `ZKeyExpr` return. (`Result<…>` returns
    // are excluded — they keep a handle — and `fun_accessor`s are skipped.)
    let mut reg = reg_with(&[
        "fn z_borrow_keyexpr(s: &ZSession) -> &ZKeyExpr { todo!() }",
        "fn z_make_keyexpr(s: &ZSession) -> ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    // Only the record fn is an accessor; the two return fns are plain.
    let accset: std::collections::HashSet<syn::Ident> =
        ["z_keyexpr_as_str"].iter().map(|s| ident(s)).collect();
    let declared: std::collections::HashSet<syn::Ident> = ["z_borrow_keyexpr", "z_make_keyexpr"]
        .iter()
        .map(|s| ident(s))
        .collect();
    apply(&mut reg, &acc, &declared, &accset).expect("apply");

    assert!(
        reg.unfold_plans.contains_key(&ident("z_borrow_keyexpr")),
        "borrow return"
    );
    assert!(
        reg.unfold_plans.contains_key(&ident("z_make_keyexpr")),
        "owned return"
    );
}

#[test]
fn callback_arg_plan_derived() {
    // An `impl Fn(ZSample)` parameter of a declared fn gets a type-level
    // plan from ZSample's default deconstructor — same leaves a return of
    // ZSample would produce, but owned (`by_ref = false`).
    let mut reg = reg_with(&[
        "fn z_declare_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
    acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_declare_sub"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

    let plan = reg
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(ZSample)))
        .expect("callback-arg plan for ZSample");
    assert!(!plan.by_ref, "the trampoline owns the callback arg");
    assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 3);
    // Nested keyexpr identity (borrowed: non-root) + string + direct enum.
    assert!(plan.leaves[0].identity);
    assert_eq!(plan.leaves[0].path[0].to_string(), "z_sample_key_expr");
    assert_eq!(
        plan.leaves[0].out_ty.to_token_stream().to_string(),
        "& ZKeyExpr"
    );
    assert_eq!(
        plan.leaves[1].path.last().unwrap().to_string(),
        "z_keyexpr_as_str"
    );
    assert_eq!(
        plan.leaves[2].out_ty.to_token_stream().to_string(),
        "SampleKind"
    );
    // Leaf out_tys registered so the resolver builds their converters.
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(&str))));
    assert!(reg
        .required_outputs_scan
        .contains(&TypeKey::from_type(&syn::parse_quote!(SampleKind))));
    // No return-position plan was created for the declaring fn.
    assert!(reg.unfold_plans.is_empty());
}

#[test]
fn callback_arg_borrowed_decomposed() {
    // A BORROWED `impl Fn(&ZSample)` decomposes through the same default
    // deconstructor as the by-value case, but with `by_ref = true` (leaves
    // read through the reference) and keyed under the actual `&ZSample` arg
    // type — so `callback_input`/`callback_iface_spec` find it.
    let mut reg = reg_with(&[
        "fn z_declare_sub(cb: impl Fn(&ZSample) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
    acc.add_deconstructor_record_id();
    acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
    acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_declare_sub"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

    // No plan under the bare `ZSample` key — only under the borrowed arg type.
    assert!(reg
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(ZSample)))
        .is_none());
    let plan = reg
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(&ZSample)))
        .expect("callback-arg plan for &ZSample");
    assert!(plan.by_ref, "the callback only borrows the delivered value");
    assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 3);
    assert!(plan.leaves[0].identity);
    assert_eq!(plan.leaves[0].path[0].to_string(), "z_sample_key_expr");
    assert_eq!(
        plan.leaves[2].out_ty.to_token_stream().to_string(),
        "SampleKind"
    );
}

#[test]
fn callback_arg_identity_fallback() {
    // No deconstructor for ZQuery ⇒ no plan: the arg is delivered whole.
    let mut reg = reg_with(&[
        "fn z_declare_queryable(cb: impl Fn(ZQuery) + Send + Sync + 'static) { todo!() }",
    ]);
    let acc = Deconstructors::default();
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_declare_queryable"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
    assert!(reg.callback_arg_plans.is_empty());
}

#[test]
fn callback_zero_arg_no_plan() {
    let mut reg =
        reg_with(&["fn z_with_close(on_close: impl Fn() + Send + Sync + 'static) { todo!() }"]);
    let acc = Deconstructors::default();
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_with_close"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
    assert!(reg.callback_arg_plans.is_empty());
}

#[test]
fn callback_arg_nonbare_skipped() {
    // `impl Fn(Vec<ZSample>)`: the arg type key (`Vec<ZSample>`) matches no
    // deconstructor target ⇒ whole-value fallback, no plan.
    let mut reg = reg_with(&[
        "fn z_batched(cb: impl Fn(Vec<ZSample>) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
    acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_batched"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
    assert!(reg.callback_arg_plans.is_empty());
}

#[test]
fn leaf_vec_fold_synthesizes_whole_element_plans() {
    // `Vec<String>` / `Option<Vec<ZenohId>>` returns and an `impl Fn(&[String])`
    // callback arg synthesize FIXED **whole-element** folds (no decon, element
    // set, no leaves) — the single-leaf dual of the `data_class` Vec fold.
    let mut reg = reg_with(&[
        "fn hello_get_locators(h: &Hello) -> Vec<String> { todo!() }",
        "fn session_peers(s: &Session) -> Option<Vec<ZenohId>> { todo!() }",
        "fn on_strings(f: impl Fn(&[String]) + Send + Sync + 'static) { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["hello_get_locators", "session_peers", "on_strings"]
            .iter()
            .map(|s| ident(s))
            .collect();
    let elements = vec![syn::parse_quote!(String), syn::parse_quote!(ZenohId)];
    apply_leaf_vec_folds(&mut reg, elements, &declared).expect("apply_leaf_vec_folds");

    // `Vec<String>` return ⇒ Iterable(Base), whole element.
    let p = reg
        .unfold_plans
        .get(&ident("hello_get_locators"))
        .expect("Vec<String> plan");
    assert!(p.fixed_builder, "synthesized leaf fold is fixed");
    assert!(matches!(&p.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)));
    assert_eq!(p.delivery, Delivery::Callback);
    assert!(p.decon.is_none(), "whole-element fold carries no decon");
    assert!(p.leaves.is_empty(), "no decomposed leaves");
    assert_eq!(
        p.element.as_ref().map(|t| t.to_token_stream().to_string()),
        Some("String".to_string())
    );

    // `Option<Vec<ZenohId>>` ⇒ Optional(Iterable(Base)).
    let p2 = reg
        .unfold_plans
        .get(&ident("session_peers"))
        .expect("Option<Vec<ZenohId>> plan");
    assert!(p2.fixed_builder);
    assert!(matches!(&p2.shape,
            UnfoldShape::Optional((), inner)
                if matches!(&**inner, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base))));
    assert_eq!(
        p2.element.as_ref().map(|t| t.to_token_stream().to_string()),
        Some("ZenohId".to_string())
    );

    // `impl Fn(&[String])` callback arg ⇒ Iterable fold keyed by `&[String]`.
    let key = TypeKey::from_type(&syn::parse_quote!(&[String]));
    let cb = reg
        .callback_arg_plans
        .get(&key)
        .expect("slice callback fold plan");
    assert!(cb.fixed_builder);
    assert!(matches!(&cb.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)));
    assert!(cb.element.is_some());
    assert!(cb.decon.is_none());
}

#[test]
fn leaf_vec_fold_skips_unnominated_and_preexisting() {
    // An un-nominated element is left on the ArrayList path (no plan); a fn
    // that already has a plan is never overwritten.
    let mut reg = reg_with(&[
        "fn other(x: &X) -> Vec<NotNominated> { todo!() }",
        "fn strings() -> Vec<String> { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["other", "strings"].iter().map(|s| ident(s)).collect();
    // Pre-seed `strings` with a sentinel plan to prove it is preserved.
    let sentinel = UnfoldPlan {
        source: syn::parse_quote!(String),
        decon: None,
        by_ref: false,
        shape: UnfoldShape::Base,
        leaves: vec![],
        element: None,
        delivery: Delivery::Return,
        convert_out_ty: None,
        fixed_builder: false,
    };
    reg.unfold_plans.insert(ident("strings"), sentinel);
    apply_leaf_vec_folds(&mut reg, vec![syn::parse_quote!(String)], &declared)
        .expect("apply_leaf_vec_folds");
    assert!(
        reg.unfold_plans.get(&ident("other")).is_none(),
        "un-nominated `NotNominated` element ⇒ no fold plan"
    );
    assert_eq!(
        reg.unfold_plans.get(&ident("strings")).map(|p| p.delivery),
        Some(Delivery::Return),
        "pre-existing plan preserved (not overwritten)"
    );
}
