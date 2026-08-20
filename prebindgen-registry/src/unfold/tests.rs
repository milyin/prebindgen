use quote::ToTokens;

use super::*;

/// A fixture declaration's identity. A declaration is a key, not a spelling —
/// see `ConstructorDecl::target`.
fn key(s: &str) -> crate::registry::TypeKey {
    crate::registry::TypeKey::parse(s).expect("a fixture type")
}
use prebindgen_flat::types_util::ident;

use crate::{
    registry::Registry,
    test_util::{scanned_with as reg_with, SpellForTest},
};

/// A reading for a fixture type, lowered by the model.
///
/// Plan leaves carry `TypeRef`s, and these fixtures assert on plan STRUCTURE —
/// so they need a reading for a type they name inline. Lowering it through an
/// empty `Flat` gives the same classification the pipeline would, without
/// standing up a source crate for it. Legitimate here and nowhere else: the
/// `classify` guard exempts tests precisely because a fixture composing its own
/// input is not a consumer reasoning from `origin`.
fn tref(ty: syn::Type) -> prebindgen_flat::flat::TypeRef {
    prebindgen_flat::flat::Flat::builder()
        .build()
        .expect("an empty model")
        .classify(&ty)
        .expect("a fixture type the language accepts")
}

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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZTimestamp"),
        records: vec![DeconRecord::Acc {
            func: ident("z_timestamp_ntp64"),
            name: "z_timestamp_ntp64".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // `z_sample_key_expr(&ZSample) -> &ZKeyExpr` decomposed into the keyexpr
    // handle (identity) + its string form (`z_keyexpr_as_str`).

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
    assert_eq!(plan.source.spell().to_string(), "ZTimestamp");
    assert!(
        matches!(&plan.shape, UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)),
        "outer shape is Optional(Decompose)"
    );
    assert_eq!(plan.leaves.len(), 1);
    assert!(!plan.leaves[0].identity);
    assert_eq!(
        plan.leaves[0].path[0].ident().to_string(),
        "z_timestamp_ntp64"
    );
    assert_eq!(plan.leaves[0].out_ty.spell().to_string(), "i64");
    assert!(
        reg.output_types[&TypeKey::from_type(&syn::parse_quote!(i64))].root,
        "the leaf type must be a root"
    );
}

#[test]
fn accessor_plan_byref() {
    // `z_sample_key_expr(&ZSample) -> &ZKeyExpr` decomposed into the keyexpr
    // handle (identity) + its string form (`z_keyexpr_as_str`).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Identity leaf: out_ty `&ZKeyExpr`, empty path, emitted last.
    // Accessor leaf: out_ty `&str`, path `[z_keyexpr_as_str]`.
    // Leaf out_tys registered as required outputs so the resolver builds
    // their converters.
    // Owned return: the root `.field_self()` MOVES the value, a nested
    // identity (spliced ZKeyExpr handle) borrows it — id-first is the
    // order that would generate non-compiling Rust, caught at apply time.

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
    assert_eq!(plan.source.spell().to_string(), "ZKeyExpr");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.leaves.len(), 2);

    // Identity leaf: out_ty `&ZKeyExpr`, empty path, emitted last.
    assert!(plan.leaves[0].identity);
    assert!(plan.leaves[0].path.is_empty());
    assert_eq!(plan.leaves[0].out_ty.spell().to_string(), "& ZKeyExpr");
    // Accessor leaf: out_ty `&str`, path `[z_keyexpr_as_str]`.
    assert!(!plan.leaves[1].identity);
    assert_eq!(plan.leaves[1].path.len(), 1);
    assert_eq!(
        plan.leaves[1].path[0].ident().to_string(),
        "z_keyexpr_as_str"
    );
    assert_eq!(plan.leaves[1].out_ty.spell().to_string(), "& str");

    // Leaf out_tys registered as required outputs so the resolver builds
    // their converters.
    assert!(reg.output_types[&TypeKey::from_type(&syn::parse_quote!(&str))].root);
}

#[test]
fn root_identity_before_nested_identity_errors() {
    // Owned return: the root `.field_self()` MOVES the value, a nested
    // identity (spliced ZKeyExpr handle) borrows it — id-first is the
    // order that would generate non-compiling Rust, caught at apply time.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
        "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
    ]);
    let accessors: std::collections::HashSet<syn::Ident> =
        ["z_query_key_expr"].iter().map(|s| ident(s)).collect();
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![DeconRecord::Identity],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZQuery"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_query_key_expr"),
                name: "key_expr".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // root identity FIRST — wrong
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_take_query")].into_iter().collect(),
        &accessors,
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::RootIdentityBeforeNested { .. }));

    // Root identity LAST (the zenoh `Query` shape) is accepted.
    let mut reg2: Registry<()> = reg_with(&[
        "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
        "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
    ]);
    let mut acc2 = Deconstructors::default();
    acc2.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![DeconRecord::Identity],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc2.deconstructors.push(DeconstructorDecl {
        target: key("ZQuery"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_query_key_expr"),
                name: "key_expr".into(),
            },
            DeconRecord::Identity,
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    }); // root identity last — ok
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_foo() -> ZKeyExpr { todo!() }",
        "fn wrong(x: &ZSample) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![DeconRecord::Acc {
            func: ident("wrong"),
            name: "wrong".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
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
    let mut reg: Registry<()> = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![DeconRecord::Identity, DeconRecord::Identity],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // A deconstructor record referencing a non-`.fun_accessor` fn errors.
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_foo(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Empty accessor set ⇒ z_keyexpr_as_str is not a fun_accessor ⇒ error.
    // With it declared as an accessor, the gate passes.
    // Two records of one deconstructor given the same literal name ⇒ hard
    // error (names are emitted verbatim; never auto-disambiguated).
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_foo() -> ZSample { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
        "fn z_sample_payload(s: &ZSample) -> Vec<u8> { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "field".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_payload"),
                name: "field".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // A record name containing the reserved `"__"` chain separator ⇒ error.
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_foo() -> ZSample { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![DeconRecord::Acc {
            func: ident("z_sample_key_expr"),
            name: "key__expr".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // M3: `z_reply_sample -> Option<&ZSample>` whose ZSample combined
    // accessor nests ZKeyExpr (handle+string), ZZBytes (bytes), and a
    // nullable ZTimestamp (Option<&ZTimestamp> → ntp64), plus a direct enum
    // leaf. Verifies path prefixes + nullable propagation.
    // Child accessors (reused via nesting).
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
    let mut reg: Registry<()> = reg_with(&[
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
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZBytes"),
        records: vec![DeconRecord::Acc {
            func: ident("z_zbytes_to_bytes"),
            name: "z_zbytes_to_bytes".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZTimestamp"),
        records: vec![DeconRecord::Acc {
            func: ident("z_timestamp_ntp64"),
            name: "z_timestamp_ntp64".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Parent accessor with nested + direct records.
    // Parent accessor with nested + direct records.
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "z_sample_key_expr".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_payload"),
                name: "z_sample_payload".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_kind"),
                name: "z_sample_kind".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_timestamp"),
                name: "z_sample_timestamp".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // keyexpr identity (path [z_sample_key_expr]) + string + payload bytes
    // + kind enum + nullable timestamp ntp64.
    // Only the timestamp leaf (Option nesting accessor) is nullable.
    // ZReply-shaped product (Result<Sample, ReplyError> decomposed in the
    // current product model): the root's records include two
    // `Option<&Child>` nesting accessors (`z_reply_sample`, `z_reply_err`)
    // whose children themselves contain `Option` nesting steps and a
    // nested identity — the double-unwrap case — plus an
    // `Option<ZZenohId>` Acc record with NO default child, which keeps
    // the full `Option<…>` as its leaf `out_ty` (its own `Option` is the
    // converter's business, not a nesting step ⇒ NOT nullable).

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
    assert_eq!(plan.source.spell().to_string(), "ZSample");
    assert!(matches!(&plan.shape, UnfoldShape::Optional((), _)));

    let path = |l: &UnfoldLeaf| {
        l.path
            .iter()
            .map(|i| i.ident().to_string())
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
    assert_eq!(plan.leaves[3].out_ty.spell().to_string(), "SampleKind");
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
    let mut reg: Registry<()> = reg_with(&[
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
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZTimestamp"),
        records: vec![DeconRecord::Acc {
            func: ident("z_timestamp_ntp64"),
            name: "z_timestamp_ntp64".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZBytes"),
        records: vec![DeconRecord::Acc {
            func: ident("z_zbytes_to_bytes"),
            name: "z_zbytes_to_bytes".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "z_sample_key_expr".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_timestamp"),
                name: "z_sample_timestamp".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZReplyError"),
        records: vec![DeconRecord::Acc {
            func: ident("z_reply_error_payload"),
            name: "z_reply_error_payload".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZReply"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_reply_replier_zid"),
                name: "z_reply_replier_zid".into(),
            },
            DeconRecord::Acc {
                func: ident("z_reply_is_ok"),
                name: "z_reply_is_ok".into(),
            },
            DeconRecord::Acc {
                func: ident("z_reply_sample"),
                name: "z_reply_sample".into(),
            },
            DeconRecord::Acc {
                func: ident("z_reply_err"),
                name: "z_reply_err".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Acc leaf keeping its full `Option<…>` return — not a nesting step.
    // Ok-arm leaves: spliced through the `Option`-returning
    // `z_reply_sample` ⇒ all nullable, incl. the nested keyexpr identity
    // and the doubly-`Option` timestamp path.
    // Err-arm leaf: spliced through `z_reply_err`.
    // A → B → A nesting is rejected.

    apply(
        &mut reg,
        &acc,
        &[ident("z_recv_reply")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg.unfold_plans.get(&ident("z_recv_reply")).expect("plan");
    assert!(!plan.by_ref, "owned ZReply return");
    assert_eq!(plan.source.spell().to_string(), "ZReply");
    assert!(matches!(&plan.shape, UnfoldShape::Base));
    assert!(matches!(plan.delivery, Delivery::Callback));

    let path = |l: &UnfoldLeaf| {
        l.path
            .iter()
            .map(|i| i.ident().to_string())
            .collect::<Vec<_>>()
            .join(".")
    };
    assert_eq!(plan.leaves.len(), 6);
    // Acc leaf keeping its full `Option<…>` return — not a nesting step.
    assert_eq!(path(&plan.leaves[0]), "z_reply_replier_zid");
    assert_eq!(
        plan.leaves[0].out_ty.spell().to_string(),
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_foo() -> ZA { todo!() }",
        "fn a_to_b(a: &ZA) -> &ZB { todo!() }",
        "fn b_to_a(b: &ZB) -> &ZA { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZA"),
        records: vec![DeconRecord::Acc {
            func: ident("a_to_b"),
            name: "a_to_b".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZB"),
        records: vec![DeconRecord::Acc {
            func: ident("b_to_a"),
            name: "b_to_a".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
    // each element delivered WHOLE (no accessor, no leaves): a per-fn
    // flatten with an empty record list on an element type that has no
    // deconstructor of its own.
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::Cycle { .. }));
    // The message says WHERE: `ZB` is reached through `a_to_b`, and it is the
    // deconstructor spliced there that closes the loop.
    assert_eq!(
        err.to_string(),
        "output expansion at `value.a_to_b()`: nested deconstructors form a cycle through `ZA`"
    );
}

#[test]
fn iterable_whole_element_plan() {
    // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
    // each element delivered WHOLE (no accessor, no leaves): a per-fn
    // flatten with an empty record list on an element type that has no
    // deconstructor of its own.
    let mut reg: Registry<()> =
        reg_with(&["fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.outputs.push(OutputDecl {
        func: ident("z_session_peers_zid"),
        sel: DeconSel::Inline(vec![]),
        target: DeconTarget::Output,
        delivery: Delivery::Callback,
        declared_source: Some(key("ZZenohId")),
    });
    // M5: `z_session_peers_zid -> Vec<ZZenohId>` with a ZZenohId combined
    // accessor → Iterable with per-element leaves: the string form + the
    // value itself via `record_id` (an identity leaf, owned at the
    // root since `Vec<ZZenohId>` owns its elements).

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
        plan.element.as_ref().map(|t| t.spell().to_string()),
        Some("ZZenohId".to_string())
    );
    assert!(reg.output_types[&TypeKey::from_type(&syn::parse_quote!(ZZenohId))].root);
}

#[test]
fn iterable_decomposed_plan() {
    // M5: `z_session_peers_zid -> Vec<ZZenohId>` with a ZZenohId combined
    // accessor → Iterable with per-element leaves: the string form + the
    // value itself via `record_id` (an identity leaf, owned at the
    // root since `Vec<ZZenohId>` owns its elements).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZenohId"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_zenoh_id_to_string"),
                name: "z_zenoh_id_to_string".into(),
            },
            DeconRecord::Identity,
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Identity leaf: owned value (`ZZenohId`, not `&ZZenohId`) since the Vec
    // owns its elements (by_ref = false).
    // `.converter(ZTimestamp, z_timestamp_ntp64)` + `.convert_output()` on
    // `z_sample_timestamp -> Option<&ZTimestamp>` ⇒ Return delivery, single
    // leaf, convert_out_ty = Option<i64>.

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
    assert_eq!(
        plan.leaves[0].path[0].ident().to_string(),
        "z_zenoh_id_to_string"
    );
    assert_eq!(plan.leaves[0].out_ty.spell().to_string(), "String");
    // Identity leaf: owned value (`ZZenohId`, not `&ZZenohId`) since the Vec
    // owns its elements (by_ref = false).
    assert!(plan.leaves[1].identity);
    assert!(plan.leaves[1].path.is_empty());
    assert_eq!(plan.leaves[1].out_ty.spell().to_string(), "ZZenohId");
}

#[test]
fn convert_output_single_value() {
    // `.converter(ZTimestamp, z_timestamp_ntp64)` + `.convert_output()` on
    // `z_sample_timestamp -> Option<&ZTimestamp>` ⇒ Return delivery, single
    // leaf, convert_out_ty = Option<i64>.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZTimestamp"),
        records: vec![DeconRecord::Acc {
            func: ident("z_timestamp_ntp64"),
            name: "z_timestamp_ntp64".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // The shaped convert type is registered as a required output.
    // A two-record deconstructor (handle + string) ⇒ Callback delivery (>1 leaf).

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
        plan.convert_out_ty.as_ref().map(|t| t.spell().to_string()),
        Some("Option < i64 >".to_string())
    );
    // The shaped convert type is registered as a required output.
    assert!(reg.output_types[&TypeKey::from_type(&syn::parse_quote!(Option<i64>))].root);
}

#[test]
fn multi_leaf_output_is_callback() {
    // A two-record deconstructor (handle + string) ⇒ Callback delivery (>1 leaf).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // A `Vec` return ⇒ Iterable + Callback (a fold), never a single Return.
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZenohId"),
        records: vec![DeconRecord::Acc {
            func: ident("z_zenoh_id_to_string"),
            name: "z_zenoh_id_to_string".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // A by-value `data_class` returned as `Option<Vec<T>>` (perftest's
    // `storage_get_vec` contract) synthesizes a FIXED-BUILDER fold wrapped in
    // an Optional layer: the field leaves cross raw per element and the
    // foreign folder rebuilds + appends them (no Java object is built on the
    // Rust side); `None` ⇒ a null list. Closes the data_class→Vec milestone.
    // An `impl Fn(&[data_class])` callback arg (perftest's
    // `storage_callback_vec`) synthesizes an Iterable fixed-folder
    // `callback_arg_plans` entry keyed by the `&[Payload]` arg: the
    // trampoline folds each element's field leaves into a foreign list, the
    // user callback still sees the whole `List<Payload>`.
    // A scalar `&Payload` callback arg must stay a Base fixed builder.
    // The ZError deconstructor (`z_error_message`) auto-applies to every fn
    // returning `Result<_, ZError>`, storing the plan in `error_plans`. Error
    // delivery is always Callback (its leaves are the `ze` callback args).
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
fn option_vec_output_is_optional_iterable_callback() {
    // An `Option<Vec<T>>` return with `T`'s default deconstructor composes as
    // a RECORD-BUILT `Optional(Iterable)` fold (issue #105): the auto-apply
    // peels the `Option` before probing the `Vec`, the elements decompose
    // into leaves (M5), and `None` skips the fold to deliver a null result.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_routers_zid(s: &ZSession) -> Option<Vec<ZZenohId>> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZenohId"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_zenoh_id_to_string"),
                name: "z_zenoh_id_to_string".into(),
            },
            DeconRecord::Identity,
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    apply(
        &mut reg,
        &acc,
        &[ident("z_routers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg.unfold_plans.get(&ident("z_routers_zid")).expect("plan");
    assert!(
        matches!(&plan.shape,
            UnfoldShape::Optional((), inner)
                if matches!(&**inner, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base))),
        "shape is Optional(Iterable(Base))"
    );
    assert!(!plan.fixed_builder, "record-built, not a fixed singleton");
    assert_eq!(plan.delivery, Delivery::Callback);
    assert!(plan.element.is_none(), "decomposed (M5): element not used");
    assert_eq!(plan.leaves.len(), 2);
    assert!(!plan.by_ref, "Option<Vec<ZZenohId>> owns its elements");
}

#[test]
fn option_vec_single_leaf_stays_callback() {
    // A SINGLE-leaf `Optional(Iterable)` fold must stay Callback: the
    // single-Return reclassification is gated on "no Iterable at any layer",
    // not just a top-level `Iterable` (an `Option<Vec<T>>` fold has no single
    // value to return through `convert_out_ty`).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_routers_zid(s: &ZSession) -> Option<Vec<ZZenohId>> { todo!() }",
        "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZZenohId"),
        records: vec![DeconRecord::Acc {
            func: ident("z_zenoh_id_to_string"),
            name: "z_zenoh_id_to_string".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    apply(
        &mut reg,
        &acc,
        &[ident("z_routers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg.unfold_plans.get(&ident("z_routers_zid")).expect("plan");
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 1);
    assert!(plan.convert_out_ty.is_none());
}

#[test]
fn option_vec_whole_element_plan() {
    // M4 dual of the decomposed case: an `Option<Vec<T>>` return with an
    // inline EMPTY record list delivers each element whole through its own
    // output converter, wrapped in the `Optional` layer.
    let mut reg: Registry<()> =
        reg_with(&["fn z_routers_zid(s: &ZSession) -> Option<Vec<ZZenohId>> { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.outputs.push(OutputDecl {
        func: ident("z_routers_zid"),
        sel: DeconSel::Inline(vec![]),
        target: DeconTarget::Output,
        delivery: Delivery::Callback,
        declared_source: Some(key("ZZenohId")),
    });
    apply(
        &mut reg,
        &acc,
        &[ident("z_routers_zid")].into_iter().collect(),
        &acc_set(),
    )
    .expect("apply");
    let plan = reg.unfold_plans.get(&ident("z_routers_zid")).expect("plan");
    assert!(
        matches!(&plan.shape,
            UnfoldShape::Optional((), inner) if matches!(&**inner, UnfoldShape::Iterable(_))),
        "shape is Optional(Iterable(Base))"
    );
    assert!(!plan.fixed_builder);
    assert!(
        plan.leaves.is_empty(),
        "whole-element: no decomposed leaves"
    );
    assert_eq!(
        plan.element.as_ref().map(|t| t.spell().to_string()),
        Some("ZZenohId".to_string())
    );
}

/// A by-value `data_class` decomposition: a product over field leaves, as the
/// JNI adapter declares one. `fields` are `(name, type)` in `fromParts` order.
fn value_struct_decon(source: syn::Type, fields: &[(&str, syn::Type)]) -> ValueDecon {
    use crate::transform::TransformKind;

    let source = tref(source);
    ValueDecon {
        key: source.key(),
        source: source.clone(),
        tree: OutNode {
            ty: source,
            kind: TransformKind::Product {
                op: OutProduct::Records,
                children: fields
                    .iter()
                    .map(|(name, ty)| OutChild {
                        link: OutLink {
                            steps: vec![PathStep::field(ident(name), false)],
                            name: vec![name.to_string()],
                        },
                        node: OutNode {
                            ty: tref(ty.clone()),
                            kind: TransformKind::Leaf(OutLeaf {
                                nullable: false,
                                identity: false,
                                reach: OutReach::Field,
                            }),
                        },
                    })
                    .collect(),
            },
        },
    }
}

#[test]
fn value_struct_vec_is_fixed_iterable_fold() {
    // A by-value `data_class` returned as `Option<Vec<T>>` (perftest's
    // `storage_get_vec` contract) synthesizes a FIXED-BUILDER fold wrapped in
    // an Optional layer: the field leaves cross raw per element and the
    // foreign folder rebuilds + appends them (no Java object is built on the
    // Rust side); `None` ⇒ a null list. Closes the data_class→Vec milestone.
    let mut reg: Registry<()> =
        reg_with(&["fn storage_get_vec(s: &Storage) -> Option<Vec<Payload>> { todo!() }"]);
    let vd = value_struct_decon(
        syn::parse_quote!(Payload),
        &[
            ("id", syn::parse_quote!(i64)),
            ("seq", syn::parse_quote!(i32)),
        ],
    );
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
    // …and the shape is that reading of the tree: an `Optional` node over a
    // `Sequence` node over the product the elements decompose into. The three
    // structural kinds nest in one plan, which is what the derived views are
    // read back off.
    {
        use crate::transform::TransformKind;
        let TransformKind::Optional { inner, .. } = &plan.tree.kind else {
            panic!("the return's `Option` is a node");
        };
        let TransformKind::Sequence { inner, .. } = &inner.kind else {
            panic!("the run is a node under it");
        };
        assert!(
            matches!(&inner.kind, TransformKind::Product { children, .. } if children.len() == 2),
            "each element decomposes into its fields"
        );
    }
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
    let mut reg: Registry<()> = reg_with(&[
        "fn storage_callback_vec(f: impl Fn(&[Payload]) + Send + Sync + 'static) { todo!() }",
    ]);
    let vd = value_struct_decon(
        syn::parse_quote!(Payload),
        &[
            ("id", syn::parse_quote!(i64)),
            ("seq", syn::parse_quote!(i32)),
        ],
    );
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
    let mut reg2: Registry<()> = reg_with(&[
        "fn storage_callback(f: impl Fn(&Payload) + Send + Sync + 'static) { todo!() }",
    ]);
    let vd2 = value_struct_decon(
        syn::parse_quote!(Payload),
        &[("id", syn::parse_quote!(i64))],
    );
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, ZError> { todo!() }",
        "fn z_error_message(e: &ZError) -> String { todo!() }",
        "fn z_infallible(s: &ZSample) -> bool { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZError"),
        records: vec![DeconRecord::Acc {
            func: ident("z_error_message"),
            name: "z_error_message".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // The infallible fn gets no error plan.
    // No output plans created (no ZKeyExpr return among the declared fns; the
    // ZError deconstructor only matches the Result error position).
    // Default-everywhere: the ZKeyExpr deconstructor auto-applies to BOTH a
    // `&ZKeyExpr` (borrow) and an owned `ZKeyExpr` return. (`Result<…>` returns
    // are excluded — they keep a handle — and `fun_accessor`s are skipped.)
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
    assert_eq!(plan.leaves[0].out_ty.spell().to_string(), "String");
    assert_eq!(plan.source.spell().to_string(), "ZError");
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_borrow_keyexpr(s: &ZSession) -> &ZKeyExpr { todo!() }",
        "fn z_make_keyexpr(s: &ZSession) -> ZKeyExpr { todo!() }",
        "fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Only the record fn is an accessor; the two return fns are plain.
    // An `impl Fn(ZSample)` parameter of a declared fn gets a type-level
    // plan from ZSample's default deconstructor — same leaves a return of
    // ZSample would produce, but owned (`by_ref = false`).
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_declare_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "z_sample_key_expr".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_kind"),
                name: "z_sample_kind".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // Nested keyexpr identity (borrowed: non-root) + string + direct enum.
    // Leaf out_tys registered so the resolver builds their converters.
    // No return-position plan was created for the declaring fn.
    // A BORROWED `impl Fn(&ZSample)` decomposes through the same default
    // deconstructor as the by-value case, but with `by_ref = true` (leaves
    // read through the reference) and keyed under the actual `&ZSample` arg
    // type — so `callback_input`/`callback_iface_spec` find it.
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_declare_sub"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

    let plan = reg
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(ZSample)))
        .expect("callback-arg plan for ZSample");
    assert!(!plan.by_ref, "the trampoline owns the callback arg");
    assert_eq!(plan.source.spell().to_string(), "ZSample");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 3);
    // Nested keyexpr identity (borrowed: non-root) + string + direct enum.
    assert!(plan.leaves[0].identity);
    assert_eq!(
        plan.leaves[0].path[0].ident().to_string(),
        "z_sample_key_expr"
    );
    assert_eq!(plan.leaves[0].out_ty.spell().to_string(), "& ZKeyExpr");
    assert_eq!(
        plan.leaves[1].path.last().unwrap().ident().to_string(),
        "z_keyexpr_as_str"
    );
    assert_eq!(plan.leaves[2].out_ty.spell().to_string(), "SampleKind");
    // Leaf out_tys registered so the resolver builds their converters.
    assert!(reg.output_types[&TypeKey::from_type(&syn::parse_quote!(&str))].root);
    assert!(reg.output_types[&TypeKey::from_type(&syn::parse_quote!(SampleKind))].root);
    // No return-position plan was created for the declaring fn.
    assert!(reg.unfold_plans.is_empty());
}

#[test]
fn callback_arg_borrowed_decomposed() {
    // A BORROWED `impl Fn(&ZSample)` decomposes through the same default
    // deconstructor as the by-value case, but with `by_ref = true` (leaves
    // read through the reference) and keyed under the actual `&ZSample` arg
    // type — so `callback_input`/`callback_iface_spec` find it.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_declare_sub(cb: impl Fn(&ZSample) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "z_keyexpr_as_str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "z_sample_key_expr".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_kind"),
                name: "z_sample_kind".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // No plan under the bare `ZSample` key — only under the borrowed arg type.
    // No deconstructor for ZQuery ⇒ no plan: the arg is delivered whole.
    // `impl Fn(Vec<ZSample>)`: the arg type key (`Vec<ZSample>`) matches no
    // deconstructor target ⇒ whole-value fallback, no plan.
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_declare_sub"].iter().map(|s| ident(s)).collect();
    apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

    // No plan under the bare `ZSample` key — only under the borrowed arg type.
    assert!(!reg
        .callback_arg_plans
        .contains_key(&TypeKey::from_type(&syn::parse_quote!(ZSample))));
    let plan = reg
        .callback_arg_plans
        .get(&TypeKey::from_type(&syn::parse_quote!(&ZSample)))
        .expect("callback-arg plan for &ZSample");
    assert!(plan.by_ref, "the callback only borrows the delivered value");
    assert_eq!(plan.source.spell().to_string(), "ZSample");
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.delivery, Delivery::Callback);
    assert_eq!(plan.leaves.len(), 3);
    assert!(plan.leaves[0].identity);
    assert_eq!(
        plan.leaves[0].path[0].ident().to_string(),
        "z_sample_key_expr"
    );
    assert_eq!(plan.leaves[2].out_ty.spell().to_string(), "SampleKind");
}

#[test]
fn callback_arg_identity_fallback() {
    // No deconstructor for ZQuery ⇒ no plan: the arg is delivered whole.
    let mut reg: Registry<()> = reg_with(&[
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
    let mut reg: Registry<()> =
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_batched(cb: impl Fn(Vec<ZSample>) + Send + Sync + 'static) { todo!() }",
        "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![DeconRecord::Acc {
            func: ident("z_sample_kind"),
            name: "z_sample_kind".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    // `Vec<String>` / `Option<Vec<ZenohId>>` returns and an `impl Fn(&[String])`
    // callback arg synthesize FIXED **whole-element** folds (no decon, element
    // set, no leaves) — the single-leaf dual of the `data_class` Vec fold.
    // `Vec<String>` return ⇒ Iterable(Base), whole element.
    // `Option<Vec<ZenohId>>` ⇒ Optional(Iterable(Base)).
    // `impl Fn(&[String])` callback arg ⇒ Iterable fold keyed by `&[String]`.
    // An un-nominated element is left on the ArrayList path (no plan); a fn
    // that already has a plan is never overwritten.
    // Pre-seed `strings` with a sentinel plan to prove it is preserved.
    // C5 validation map: a field accessor ident that names no `#[prebindgen]`
    // fn is a hard `UnknownAccessor` at resolve time — a typo'd
    // `expand_return!(...).field(fun!(…))` cannot silently vanish. (At the
    // jnigen level a registry-absent fn is caught even earlier, by the scan's
    // `DeclaredNotFound` on the helper channel; this is the core-tier guard
    // that stands on its own for any adapter.)
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
    let mut reg: Registry<()> = reg_with(&[
        "fn hello_get_locators(h: &Hello) -> Vec<String> { todo!() }",
        "fn session_peers(s: &Session) -> Option<Vec<ZenohId>> { todo!() }",
        "fn on_strings(f: impl Fn(&[String]) + Send + Sync + 'static) { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["hello_get_locators", "session_peers", "on_strings"]
            .iter()
            .map(|s| ident(s))
            .collect();
    let elements = vec![key("String"), key("ZenohId")];
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
        p.element.as_ref().map(|t| t.spell().to_string()),
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
        p2.element.as_ref().map(|t| t.spell().to_string()),
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
    let mut reg: Registry<()> = reg_with(&[
        "fn other(x: &X) -> Vec<NotNominated> { todo!() }",
        "fn strings() -> Vec<String> { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["other", "strings"].iter().map(|s| ident(s)).collect();
    // Pre-seed `strings` with a sentinel plan to prove it is preserved.
    let source = tref(syn::parse_quote!(String));
    let sentinel = UnfoldPlan {
        source: source.clone(),
        decon: None,
        by_ref: false,
        shape: UnfoldShape::Base,
        tree: std::rc::Rc::new(OutNode {
            ty: source.clone(),
            kind: crate::transform::TransformKind::Product {
                op: OutProduct::Records,
                children: Vec::new(),
            },
        }),
        leaves: vec![],
        element: None,
        delivery: Delivery::Return,
        convert_out_ty: None,
        fixed_builder: false,
        hoists: Vec::new(),
    };
    reg.unfold_plans.insert(ident("strings"), sentinel);
    apply_leaf_vec_folds(&mut reg, vec![key("String")], &declared).expect("apply_leaf_vec_folds");
    assert!(
        !reg.unfold_plans.contains_key(&ident("other")),
        "un-nominated `NotNominated` element ⇒ no fold plan"
    );
    assert_eq!(
        reg.unfold_plans.get(&ident("strings")).map(|p| p.delivery),
        Some(Delivery::Return),
        "pre-existing plan preserved (not overwritten)"
    );
}

/// C5 validation map: a field accessor ident that names no `#[prebindgen]`
/// fn is a hard `UnknownAccessor` at resolve time — a typo'd
/// `expand_return!(...).field(fun!(…))` cannot silently vanish. (At the
/// jnigen level a registry-absent fn is caught even earlier, by the scan's
/// `DeclaredNotFound` on the helper channel; this is the core-tier guard
/// that stands on its own for any adapter.)
#[test]
fn unknown_accessor_errors() {
    let mut reg: Registry<()> = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![DeconRecord::Acc {
            func: ident("z_keyexpr_as_str_typo"),
            name: "as_str".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_foo")].into_iter().collect(),
        &[ident("z_keyexpr_as_str_typo")].into_iter().collect(),
    )
    .unwrap_err();
    assert!(matches!(err, UnfoldError::UnknownAccessor(_)), "{err}");
}

/// #96: duplicate declaration records are rejected with COLLECTED
/// diagnostics — every offender reported in one error. (Empty record lists
/// are NOT diagnosed: an empty inline list is the valid whole-element
/// delivery form.)
#[test]
fn duplicate_declarations_collected() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        "fn z_session_key(s: &ZSession) -> ZKeyExpr { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    for _ in 0..2 {
        acc.deconstructors.push(DeconstructorDecl {
            target: key("ZKeyExpr"),
            records: vec![DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "asStr".into(),
            }],
            default: Some((DeconTarget::Output, Delivery::Callback)),
        });
        acc.outputs.push(OutputDecl {
            func: ident("z_session_key"),
            sel: DeconSel::Inline(vec![DeconRecord::Identity]),
            target: DeconTarget::Output,
            delivery: Delivery::Callback,
            declared_source: Some(key("ZKeyExpr")),
        });
    }
    let err = apply(
        &mut reg,
        &acc,
        &[ident("z_session_key")].into_iter().collect(),
        &acc_set(),
    )
    .unwrap_err();
    let UnfoldError::InvalidDeclarations { entries } = &err else {
        panic!("expected InvalidDeclarations, got {err}");
    };
    assert_eq!(entries.len(), 2, "{err}");
    let text = err.to_string();
    assert!(
        text.contains("duplicate deconstructor declaration for `ZKeyExpr`"),
        "{text}"
    );
    assert!(
        text.contains("duplicate output expansion for `z_session_key`"),
        "{text}"
    );
}

/// The decomposition a sum is made of, mirroring what the JNI adapter declares:
/// a choice over one arm per alternative, each a product of its payload
/// members. `Reading` has a unit `Missing` (tag 0) and an `Exact(i64)` (tag 1).
///
/// The tag leaf, its `out_ty` — **the sum**, which is how the emitter finds the
/// enum to `match` — and every leaf's group are DERIVED from this; the
/// declaration says only what the sum is.
fn reading_sum_decon() -> SumDecon {
    use crate::transform::TransformKind;

    let arm = |name: &str, tag: i32, fields: Vec<OutChild>| OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: Vec::new(),
        },
        node: OutNode {
            ty: tref(syn::parse_quote!(Reading)),
            kind: TransformKind::Product {
                op: OutProduct::Variant {
                    name: ident(name),
                    tag,
                },
                children: fields,
            },
        },
    };
    let payload = |name: &str, idx: u32, ty: syn::Type| OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: vec![name.to_string()],
        },
        node: OutNode {
            ty: tref(ty),
            kind: TransformKind::Leaf(OutLeaf {
                nullable: false,
                identity: false,
                reach: OutReach::VariantMember(syn::Member::Unnamed(syn::Index::from(
                    idx as usize,
                ))),
            }),
        },
    };
    SumDecon {
        key: TypeKey::from_type(&syn::parse_quote!(Reading)),
        source: tref(syn::parse_quote!(Reading)),
        tree: OutNode {
            ty: tref(syn::parse_quote!(Reading)),
            kind: TransformKind::Choice {
                op: OutChoice {
                    name: "tag".to_string(),
                },
                variants: vec![
                    // A unit variant contributes only its tag.
                    arm("Missing", 0, Vec::new()),
                    arm(
                        "Exact",
                        1,
                        vec![payload("exact_v0", 0, syn::parse_quote!(i64))],
                    ),
                ],
            },
        },
    }
}

/// A sum returned by a function decomposes into a **fixed-builder** plan over
/// its tag and groups — the same delivery a by-value `data_class` gets, with
/// the selector added. The declared return's own output requirement is dropped:
/// a sum has no whole-value converter by construction, so leaving it required
/// would fail the resolve on a converter that must not exist.
#[test]
fn sum_return_is_a_fixed_builder_plan() {
    let mut reg: Registry<()> = reg_with(&["fn read_one(which: i32) -> Reading { todo!() }"]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["read_one"].iter().map(|s| ident(s)).collect();
    apply_sum_returns(&mut reg, vec![reading_sum_decon()], &declared).expect("apply_sum_returns");

    let plan = reg.unfold_plans.get(&ident("read_one")).expect("plan");
    // The plan's tree IS the choice the declaration stated — a sum is not a
    // product, and the leaf list below is derived from that.
    assert!(
        matches!(
            &plan.tree.kind,
            crate::transform::TransformKind::Choice { variants, .. } if variants.len() == 2
        ),
        "a sum decomposes through a choice node, one arm per alternative"
    );
    assert!(plan.fixed_builder, "sum ⇒ fixed builder");
    assert_eq!(plan.delivery, Delivery::Callback);
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert!(!plan.by_ref);
    assert_eq!(plan.leaves.len(), 2, "the tag plus one group leaf");
    assert_eq!(plan.leaves[0].source, LeafSource::SumTag);
    assert_eq!(plan.leaves[0].group, None, "the selector joins no group");
    assert_eq!(
        plan.leaves[1].group,
        Some(1),
        "the group is its variant's tag"
    );
    assert!(matches!(
        &plan.leaves[1].source,
        LeafSource::VariantField { variant, .. } if variant == "Exact"
    ));
    // The selector is synthesized, so it must not drag an `i32` output
    // requirement into a binding that has no `i32` crossing of its own.
    assert!(!plan.leaves[0].has_converter());
    assert!(plan.leaves[1].has_converter());
    // #282's invariant, stated over the plan rather than over one type name:
    // EVERY leaf's `out_ty` is registered, and only a converter-bearing leaf is
    // a root. The assertion this replaced could state neither half — it read
    // `!...is_some_and(|c| c.root)` on `Reading`, which is also true when the
    // cell is ABSENT, and absent is what it was: this fixture's registry
    // declares nothing, so it passed for the wrong reason.
    for leaf in &plan.leaves {
        let cell = reg
            .output_types
            .get(&leaf.out_ty.key())
            .unwrap_or_else(|| panic!("leaf `{}` registers its out_ty", leaf.name));
        assert_eq!(
            cell.root,
            leaf.has_converter(),
            "leaf `{}`: a cell says the type entered the pipeline, a root says \
             the binding demands its converter — the selector makes only the \
             first, because a sum has no whole-value output converter",
            leaf.name
        );
    }
}

/// `Option` and `Vec` layers ride the existing shape fold — a sum needs
/// nothing new for them — and each layer's own scan-time output requirement is
/// dropped along with the bare type's.
#[test]
fn sum_return_layers_ride_the_shape_fold() {
    let mut reg: Registry<()> = reg_with(&[
        "fn read_maybe(w: i32) -> Option<Reading> { todo!() }",
        "fn read_all(n: i32) -> Vec<Reading> { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> = ["read_maybe", "read_all"]
        .iter()
        .map(|s| ident(s))
        .collect();
    apply_sum_returns(&mut reg, vec![reading_sum_decon()], &declared).expect("apply_sum_returns");

    let opt = reg.unfold_plans.get(&ident("read_maybe")).expect("plan");
    assert!(matches!(&opt.shape,
        UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)));
    let vec_plan = reg.unfold_plans.get(&ident("read_all")).expect("plan");
    assert!(matches!(&vec_plan.shape,
        UnfoldShape::Iterable(inner) if matches!(**inner, UnfoldShape::Base)));
    assert!(vec_plan.element.is_none(), "decomposed-leaf fold");
    for ty in ["Option<Reading>", "Vec<Reading>", "Reading"] {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        assert!(
            !reg.output_types
                .get(&TypeKey::from_type(&ty))
                .is_some_and(|c| c.root),
            "no layer of a sum return may require a whole-value converter: {}",
            ty.to_token_stream()
        );
    }
}

/// A `Vec<sum>` return **on its own** — no bare or `Option`-wrapped return of
/// the same sum anywhere in the binding. The test above cannot state this: its
/// `Option<Reading>` fixture unrequires the bare type through a *different*
/// layer, so it would keep passing if the `Vec` element were left required.
///
/// The bare requirement is **seeded explicitly**. A scan registers only the
/// top-level return, so `Reading` would not be in the set to begin with and the
/// assertion would hold whether or not `wire_fixed_returns` unrequired the
/// peeled element — passing while testing nothing. Seeding reproduces what an
/// adapter that does require the element leaves behind, which is the state the
/// unrequire exists for.
#[test]
fn a_vec_only_sum_return_drops_the_bare_requirement() {
    let mut reg: Registry<()> = reg_with(&["fn read_all(n: i32) -> Vec<Reading> { todo!() }"]);
    let bare: syn::Type = syn::parse_quote!(Reading);
    let bare_reading = reg
        .intern(crate::registry::Direction::Output, &bare, true)
        .expect("fixture type");
    reg.require_output(&bare_reading);
    assert!(
        reg.output_types[&TypeKey::from_type(&bare)].root,
        "fixture precondition: the bare element starts out required"
    );

    let declared: std::collections::HashSet<syn::Ident> =
        ["read_all"].iter().map(|s| ident(s)).collect();
    apply_sum_returns(&mut reg, vec![reading_sum_decon()], &declared).expect("apply_sum_returns");

    for ty in ["Vec<Reading>", "Reading"] {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        assert!(
            !reg.output_types
                .get(&TypeKey::from_type(&ty))
                .is_some_and(|c| c.root),
            "no layer of a sum return may require a whole-value converter: {}",
            ty.to_token_stream()
        );
    }
}

/// An `impl Fn(E)` callback argument decomposes the same way a return does —
/// the plan is keyed by the arg type, so the trampoline delivers the tag and
/// groups instead of a whole value built on the Rust side.
#[test]
fn sum_callback_arg_is_a_fixed_builder_plan() {
    let mut reg: Registry<()> = reg_with(&[
        "fn read_each(n: i32, f: impl Fn(Reading) + Send + Sync + 'static) { todo!() }",
    ]);
    let declared: std::collections::HashSet<syn::Ident> =
        ["read_each"].iter().map(|s| ident(s)).collect();
    apply_sum_returns(&mut reg, vec![reading_sum_decon()], &declared).expect("apply_sum_returns");

    let key = TypeKey::from_type(&syn::parse_quote!(Reading));
    let plan = reg
        .callback_arg_plans
        .get(&key)
        .expect("callback-arg plan keyed by the arg type");
    assert!(plan.fixed_builder);
    assert!(matches!(plan.shape, UnfoldShape::Base));
    assert_eq!(plan.leaves[0].source, LeafSource::SumTag);
}

/// A `Vec<Option<T>>` return does not match a `T` decomposition.
///
/// The boundary accepts `Option<Vec<T>>` — an optional list — and the layer stack
/// stops at any layer out of that order. So the optional inside a `Vec` belongs
/// to the **element**, the return's core is `Option<Payload>` rather than
/// `Payload`, and no fixed fold is installed.
///
/// This is the pairing that matters: the explicit decomposition path a few
/// hundred lines up refuses `Vec<Option<…>>` outright as unsupported, so a
/// silently-installed nested fold here would mean the two paths disagree about
/// the same return type. An unbounded layer peel makes exactly that happen —
/// verified by sabotage, not assumed.
#[test]
fn a_vec_of_optionals_installs_no_fixed_fold() {
    let mut reg: Registry<()> =
        reg_with(&["fn storage_get_vec(s: &Storage) -> Vec<Option<Payload>> { todo!() }"]);
    let vd = value_struct_decon(
        syn::parse_quote!(Payload),
        &[
            ("id", syn::parse_quote!(i64)),
            ("seq", syn::parse_quote!(i32)),
        ],
    );
    let declared: std::collections::HashSet<syn::Ident> =
        ["storage_get_vec"].iter().map(|s| ident(s)).collect();
    apply_value_structs(&mut reg, vec![vd], &declared).expect("apply_value_structs");

    assert!(
        !reg.unfold_plans.contains_key(&ident("storage_get_vec")),
        "a Vec<Option<Payload>> return must not fold as a Payload decomposition"
    );
}

/// A stand-in language adapter (#442): it says what a leaf and a product
/// *render as* and nothing else. There is no recursion here, no `TypeRef`
/// walk and no path arithmetic — the registry supplies the order, the links
/// and the descent, which is the whole claim the shared tree makes.
struct Render;

impl crate::transform::TransformLowerer<OutOfRust> for Render {
    type Value = String;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &OutNode, op: &OutLeaf) -> Result<String, Self::Error> {
        Ok(format!(
            "{}{}",
            node.ty.spell(),
            if op.identity { " (identity)" } else { "" }
        ))
    }

    fn product(
        &mut self,
        _node: &OutNode,
        _op: &OutProduct,
        children: crate::transform::Lowered<'_, OutOfRust, String>,
    ) -> Result<String, Self::Error> {
        let rendered: Vec<String> = children
            .into_iter()
            .map(|(child, value)| {
                let steps: Vec<String> = child
                    .link
                    .steps
                    .iter()
                    .map(|s| format!("{}{}", s.ident(), if s.is_optional() { "?" } else { "" }))
                    .collect();
                format!(
                    "{}<-{}: {value}",
                    child.link.name.join("__"),
                    steps.join(".")
                )
            })
            .collect();
        Ok(format!("[{}]", rendered.join(", ")))
    }

    fn choice(
        &mut self,
        _node: &OutNode,
        op: &OutChoice,
        variants: crate::transform::Lowered<'_, OutOfRust, String>,
    ) -> Result<String, Self::Error> {
        let arms: Vec<String> = variants.into_iter().map(|(_, v)| v).collect();
        Ok(format!("{}?{{{}}}", op.name, arms.join(" | ")))
    }

    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(format!("{value}?"))
    }

    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &OutRun,
        _inner: &OutNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(format!("[{value}]*"))
    }
}

/// `z_reply_sample -> Option<&ZSample>` whose ZSample splices ZKeyExpr
/// (identity + string) and a nullable ZTimestamp: an `Optional` layer over a
/// product with a nested product under each of its two children. Shared by the
/// traversal tests below.
fn reply_sample_registry() -> Registry<()> {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { todo!() }",
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZKeyExpr"),
        records: vec![
            DeconRecord::Identity,
            DeconRecord::Acc {
                func: ident("z_keyexpr_as_str"),
                name: "str".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZTimestamp"),
        records: vec![DeconRecord::Acc {
            func: ident("z_timestamp_ntp64"),
            name: "ntp64".into(),
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });
    acc.deconstructors.push(DeconstructorDecl {
        target: key("ZSample"),
        records: vec![
            DeconRecord::Acc {
                func: ident("z_sample_key_expr"),
                name: "ke".into(),
            },
            DeconRecord::Acc {
                func: ident("z_sample_timestamp"),
                name: "ts".into(),
            },
        ],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });

    apply(
        &mut reg,
        &acc,
        &[ident("z_reply_sample")].into_iter().collect(),
        &acc_set_without("z_reply_sample"),
    )
    .expect("apply");
    reg
}

#[test]
fn tree_lowers_through_the_shared_visitor() {
    // The tree is what the decomposition IS; the leaf list below is a derived
    // view of it, so both readings of the same plan must agree on names, order
    // and nullability.
    let reg = reply_sample_registry();

    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");

    // The whole plan, read through the hooks alone — the `Option<&ZSample>`
    // return is the trailing `?`, so the arity layer is a node like any other.
    let rendered = plan.tree.lower(&mut Render).expect("lowering cannot fail");
    assert_eq!(
        rendered,
        "[ke<-z_sample_key_expr: [<-: & ZKeyExpr (identity), str<-z_keyexpr_as_str: & str], \
         ts<-z_sample_timestamp?: [ntp64<-z_timestamp_ntp64: i64]]?"
    );
    // …and the shape the plan exposes is read back off those same nodes.
    assert!(
        matches!(&plan.shape, UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)),
        "the layer node IS the plan's shape"
    );
    assert!(
        plan.element.is_none(),
        "the value is taken apart, not delivered whole"
    );

    // …and the derived view of the same tree, which is what the plan exposes:
    // the identity takes the chain it sits at, the `Option` nesting step makes
    // everything under it nullable, and nothing else is.
    let names: Vec<(&str, bool)> = plan
        .leaves
        .iter()
        .map(|l| (l.name.as_str(), l.nullable))
        .collect();
    assert_eq!(
        names,
        vec![("ke", false), ("ke__str", false), ("ts__ntp64", true)]
    );
}

/// A stand-in adapter that has a **direct converter** for one type: it claims
/// that subtree whole and never looks inside it. Records every node it was
/// asked about, so a test can show what a claimed subtree costs — nothing.
struct Direct {
    /// Spelling of the type this adapter converts directly.
    whole: &'static str,
    /// Every node `descend` was asked about, in visit order.
    asked: Vec<String>,
}

impl crate::transform::TransformLowerer<OutOfRust> for Direct {
    type Value = String;
    type Error = std::convert::Infallible;

    fn descend(
        &mut self,
        node: &OutNode,
        _link: Option<&OutLink>,
    ) -> Result<crate::transform::Descend<String>, Self::Error> {
        let ty = node.ty.spell().to_string();
        self.asked.push(ty.clone());
        Ok(if ty == self.whole {
            crate::transform::Descend::Atomic(format!("<{ty}>"))
        } else {
            crate::transform::Descend::Recurse
        })
    }

    fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<String, Self::Error> {
        Ok(node.ty.spell().to_string())
    }

    fn product(
        &mut self,
        _node: &OutNode,
        _op: &OutProduct,
        children: crate::transform::Lowered<'_, OutOfRust, String>,
    ) -> Result<String, Self::Error> {
        let inner: Vec<String> = children.into_iter().map(|(_, v)| v).collect();
        Ok(format!("({})", inner.join(", ")))
    }

    fn choice(
        &mut self,
        _node: &OutNode,
        op: &OutChoice,
        variants: crate::transform::Lowered<'_, OutOfRust, String>,
    ) -> Result<String, Self::Error> {
        let arms: Vec<String> = variants.into_iter().map(|(_, v)| v).collect();
        Ok(format!("{}?{{{}}}", op.name, arms.join(" | ")))
    }

    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(format!("{value}?"))
    }

    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &OutRun,
        _inner: &OutNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(format!("[{value}]*"))
    }
}

/// #444: an adapter with a direct converter claims a subtree before anything
/// under it is visited, at a NESTED structural node — the `ZKeyExpr` product.
/// Its children are neither lowered nor even offered, so they can contribute no
/// slot, no converter dependency and no cleanup.
#[test]
fn a_direct_converter_ends_a_nested_subtree() {
    let reg = reply_sample_registry();
    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");

    let mut direct = Direct {
        whole: "ZKeyExpr",
        asked: Vec::new(),
    };
    let rendered = plan.tree.lower(&mut direct).expect("lowering cannot fail");
    assert_eq!(rendered, "(<ZKeyExpr>, (i64))?");

    // What the claimed subtree contained — the cloned handle and the string —
    // was never offered to the lowerer at all.
    assert!(
        !direct
            .asked
            .iter()
            .any(|t| t == "& ZKeyExpr" || t == "& str"),
        "a claimed subtree is not descended into: {:?}",
        direct.asked
    );
    // Its sibling still is.
    assert!(direct.asked.iter().any(|t| t == "i64"));
}

/// #444: the same decision at the OUTERMOST node — the `Option` layer — ends
/// the whole plan, so nothing below it is visited.
#[test]
fn a_direct_converter_ends_the_whole_tree() {
    let reg = reply_sample_registry();
    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");

    let mut direct = Direct {
        whole: "Option < & ZSample >",
        asked: Vec::new(),
    };
    let rendered = plan.tree.lower(&mut direct).expect("lowering cannot fail");
    assert_eq!(rendered, "<Option < & ZSample >>");
    assert_eq!(
        direct.asked,
        vec!["Option < & ZSample >".to_string()],
        "the root was claimed, so nothing under it was offered"
    );
}

/// #444: what a decomposition needs converters for is read off the tree, not
/// off a leaf list — and a sum's selector names its enum without demanding a
/// whole-value converter that cannot exist.
#[test]
fn dependencies_come_from_the_tree() {
    let deps = crate::unfold::dependencies(&reading_sum_decon().tree);
    let required: Vec<String> = deps.required.iter().map(|t| t.key().to_string()).collect();
    let referenced: Vec<String> = deps
        .referenced
        .iter()
        .map(|t| t.key().to_string())
        .collect();
    assert_eq!(required, vec!["i64".to_string()], "the one payload crosses");
    assert_eq!(
        referenced,
        vec!["Reading".to_string()],
        "the selector names the sum it chooses between"
    );
}

/// #444: a sum spliced into a value form must be **named** by the plan, not
/// **required** of it. The selector carries the enum so an emitter can `match`
/// it, but a sum has no whole-value output converter, so demanding one would
/// fail resolution for any binding that has not declared one anyway.
///
/// The three per-plan registration sites used to require every derived leaf's
/// type flatly, which reached the synthesized selector too; they go through the
/// same tree-derived split the fixed-decon paths use.
#[test]
fn a_spliced_sum_is_named_not_required() {
    let mut reg: Registry<()> = reg_with(&[
        "fn get_report() -> Report { todo!() }",
        "fn report_to_struct(r: &Report) -> ReportStruct { todo!() }",
    ]);
    let mut acc = Deconstructors::default();
    acc.deconstructors.push(DeconstructorDecl {
        target: key("Report"),
        records: vec![DeconRecord::Fields {
            func: ident("report_to_struct"),
            consuming: false,
            fields: vec![FieldRecord {
                members: vec![ident("reading")],
                name: "reading".into(),
                ty: tref(syn::parse_quote!(Reading)),
                decon: FieldDecon::Subtree(reading_sum_decon().tree),
            }],
        }],
        default: Some((DeconTarget::Output, Delivery::Callback)),
    });

    apply(
        &mut reg,
        &acc,
        &[ident("get_report")].into_iter().collect(),
        &[ident("report_to_struct")].into_iter().collect(),
    )
    .expect("apply");

    let plan = reg.unfold_plans.get(&ident("get_report")).expect("plan");
    assert!(
        plan.leaves().iter().any(|l| l.source == LeafSource::SumTag),
        "the spliced sum contributes its selector"
    );
    let sum = reg
        .output_types
        .get(&TypeKey::from_type(&syn::parse_quote!(Reading)))
        .expect("the selector registers the sum it chooses between");
    assert!(
        !sum.root,
        "a sum is named, not required — it has no whole-value output converter"
    );
    // Its payload, which does cross, still is.
    assert!(
        reg.output_types[&TypeKey::from_type(&syn::parse_quote!(i64))].root,
        "the live payload is a required crossing"
    );
}

/// #444: an adapter's converter selection is a **tree**, not a policy each
/// pass re-runs. `select` applies it once; everything after reads that value,
/// so there is nothing for two passes to disagree about.
///
/// The selected reading replaces the subtree, and it is the adapter's to state:
/// a structural node names the OWNED core, while the accessor that reached it
/// may borrow, so taking `node.ty` would root `T` where the plan calls the
/// converter for `&T`.
#[test]
fn a_selected_tree_replaces_what_it_claims() {
    let reg = reply_sample_registry();
    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");
    let names = |d: &[prebindgen_flat::flat::TypeRef]| -> Vec<String> {
        d.iter().map(|t| t.spell().to_string()).collect()
    };

    assert_eq!(
        names(&crate::unfold::dependencies(plan.tree()).required),
        vec!["& ZKeyExpr", "& str", "i64"],
        "with nothing selected, every crossing is required"
    );

    // An adapter that converts a whole key expression directly, through the
    // borrowed converter, because the accessor that reaches it borrows.
    let selected = crate::unfold::select(plan.tree(), &mut |node, link| {
        (node.ty.spell().to_string() == "ZKeyExpr").then(|| {
            if link.is_some_and(|l| l.steps.iter().any(|st| !st.yields_owned())) {
                node.ty.borrowed()
            } else {
                node.ty.clone()
            }
        })
    });
    assert_eq!(
        names(&crate::unfold::dependencies(&selected).required),
        vec!["& ZKeyExpr", "i64"],
        "the selected reading replaces the subtree; its children are gone"
    );

    // …and the same tree registers accordingly, which is what a binding
    // actually demands. `TypeCell::root` only ever gains, so a selection
    // honoured by one pass and not another could not be taken back.
    let rooted = |reg: &Registry<()>, ty: syn::Type| -> Option<bool> {
        reg.output_types
            .get(&TypeKey::from_type(&ty))
            .map(|cell| cell.root)
    };
    let mut claimed: Registry<()> = reg_with(&[]);
    crate::unfold::register_dependencies(&mut claimed, &selected);
    assert_eq!(rooted(&claimed, syn::parse_quote!(&ZKeyExpr)), Some(true));
    assert_ne!(
        rooted(&claimed, syn::parse_quote!(&str)),
        Some(true),
        "a child of the selected subtree is never demanded"
    );

    let mut plain: Registry<()> = reg_with(&[]);
    crate::unfold::register_dependencies(&mut plain, plan.tree());
    assert_eq!(rooted(&plain, syn::parse_quote!(&str)), Some(true));
}

/// #444 §2: a boundary use with no declared decomposition still has a semantic
/// plan — the value crosses whole under its arity layers. Without it an adapter
/// that declares nothing, as Cbindgen does, has no tree to lower and is back to
/// walking `TypeRef` itself.
#[test]
fn an_ordinary_boundary_use_has_a_plan() {
    use crate::transform::TransformKind;

    let plan = crate::unfold::ordinary(&tref(syn::parse_quote!(Option<Vec<ZSample>>)));
    let TransformKind::Optional { inner, .. } = &plan.kind else {
        panic!("the `Option` is a layer node");
    };
    let TransformKind::Sequence { inner, .. } = &inner.kind else {
        panic!("the `Vec` is a layer node under it");
    };
    assert!(
        matches!(inner.kind, TransformKind::Leaf(_)),
        "the element crosses whole"
    );
    assert_eq!(inner.ty.spell().to_string(), "ZSample");

    // The derived views agree: a whole element is delivered to the fold rather
    // than as a named slot, and it is the dependency.
    let (leaves, _) = crate::unfold::flat_view(&plan).expect("an ordinary plan projects");
    assert!(leaves.is_empty());
    assert_eq!(
        crate::unfold::element_of(&plan).map(|t| t.spell().to_string()),
        Some("ZSample".to_string())
    );
    assert_eq!(
        crate::unfold::dependencies(&plan)
            .required
            .iter()
            .map(|t| t.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["ZSample"]
    );

    // A plain value is a bare leaf, no layers.
    let scalar = crate::unfold::ordinary(&tref(syn::parse_quote!(i64)));
    assert!(matches!(scalar.kind, TransformKind::Leaf(_)));
}

/// #444 (review): the decision is taken for a value **in a position**. The same
/// `ZKeyExpr` reached by a borrowing accessor and by an owning one converts and
/// cleans up differently, so the pre-descent hook is handed the edge as well as
/// the node.
#[test]
fn the_cutoff_sees_the_edge_it_was_reached_by() {
    let reg = reply_sample_registry();
    let plan = reg
        .unfold_plans
        .get(&ident("z_reply_sample"))
        .expect("plan");

    let mut seen: Vec<(String, Option<String>)> = Vec::new();
    crate::unfold::select(plan.tree(), &mut |node, link| {
        seen.push((
            node.ty.spell().to_string(),
            link.map(|l| {
                l.steps
                    .iter()
                    .map(|s| s.ident().to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            }),
        ));
        None
    });
    assert!(
        seen.contains(&(
            "ZKeyExpr".to_string(),
            Some("z_sample_key_expr".to_string())
        )),
        "the nested product is offered with the accessor that reached it: {seen:?}"
    );
    assert!(
        seen.iter()
            .any(|(ty, link)| ty == "ZSample" && link.is_none()),
        "the node under the `Option` layer has no edge of its own: {seen:?}"
    );
}

/// #444 (review): a run delivering whole elements depends on the element's own
/// converter. The derived leaf list drops it — a whole element is not a named
/// wire slot — but "what is a slot" and "what needs a converter" are different
/// questions, and answering the second with the first left `Vec<T>`'s required
/// set empty for anyone reading the API rather than the construction sites.
#[test]
fn a_whole_element_run_requires_its_element() {
    use crate::transform::TransformKind;

    let elem = tref(syn::parse_quote!(&ZSample));
    let tree = OutNode {
        ty: tref(syn::parse_quote!(Vec<&ZSample>)),
        kind: TransformKind::Sequence {
            op: OutRun { borrowed: false },
            inner: Box::new(OutNode {
                ty: elem.clone(),
                kind: TransformKind::Leaf(OutLeaf {
                    nullable: false,
                    identity: false,
                    reach: OutReach::Accessor,
                }),
            }),
        },
    };
    let (leaves, _) = crate::unfold::flat_view(&tree).expect("a whole-element run projects");
    assert!(
        leaves.is_empty(),
        "a whole element is delivered to the fold, not as a named slot"
    );
    let deps = crate::unfold::dependencies(&tree);
    assert_eq!(
        deps.required
            .iter()
            .map(|t| t.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["& ZSample"],
        "…but it still crosses through its own converter"
    );
}

/// `flat_view`'s `Ok` side holds types without `Debug`, so a refusal is taken
/// by matching rather than by `unwrap_err`.
fn refused(r: Result<(Vec<UnfoldLeaf>, Vec<Hoist>), UnfoldError>) -> UnfoldError {
    match r {
        Ok(_) => panic!("expected the projection to refuse this shape"),
        Err(e) => e,
    }
}

/// #444 (review): a variant payload must be a leaf that BINDS A MEMBER.
///
/// Being a leaf is not enough — the binding an arm upgrades to
/// `LeafSource::VariantField` comes from the leaf's own reach, so a leaf
/// reached by field access or an accessor would be grouped under the arm while
/// claiming it was walked to. And a subtree of any kind loses the binding
/// outright, a nested sum losing its own tags on top, because `group` holds one
/// tag per leaf.
///
/// Reported as a typed planning error naming the sum and the arm, not an abort:
/// the projection is public, and an adapter handing over a shape it cannot have
/// should be told which one.
///
/// Nothing produces either shape today — a sum's payload members are leaves —
/// so the refusal is what makes that a stated limit instead of an accident.
#[test]
fn a_variant_payload_must_be_a_leaf_that_binds_a_member() {
    use crate::transform::TransformKind;

    let arm_payload = |payload: OutNode| -> OutNode {
        OutNode {
            ty: tref(syn::parse_quote!(Outer)),
            kind: TransformKind::Choice {
                op: OutChoice {
                    name: "tag".to_string(),
                },
                variants: vec![OutChild {
                    link: OutLink {
                        steps: Vec::new(),
                        name: Vec::new(),
                    },
                    node: OutNode {
                        ty: tref(syn::parse_quote!(Outer)),
                        kind: TransformKind::Product {
                            op: OutProduct::Variant {
                                name: ident("Wrapped"),
                                tag: 0,
                            },
                            children: vec![OutChild {
                                link: OutLink {
                                    steps: Vec::new(),
                                    name: vec!["inner".to_string()],
                                },
                                node: payload,
                            }],
                        },
                    },
                }],
            },
        }
    };
    let leaf = |reach: OutReach| OutNode {
        ty: tref(syn::parse_quote!(i64)),
        kind: TransformKind::Leaf(OutLeaf {
            nullable: false,
            identity: false,
            reach,
        }),
    };
    let member = || OutReach::VariantMember(syn::Member::Unnamed(syn::Index::from(0usize)));

    // A nested SUM — the case the previous check caught, now by structure
    // rather than by noticing its leaves already had a group.
    let err = refused(crate::unfold::flat_view(&arm_payload(
        reading_sum_decon().tree,
    )));
    assert!(
        matches!(
            &err,
            UnfoldError::UnsupportedVariantPayload { variant, found, .. }
                if variant == "Wrapped" && *found == "a choice"
        ),
        "got {err}"
    );

    // A nested PRODUCT — which has no group on its leaves, so it passed the
    // group-based check and was silently misprojected.
    let product = OutNode {
        ty: tref(syn::parse_quote!(InnerStruct)),
        kind: TransformKind::Product {
            op: OutProduct::Records,
            children: vec![OutChild {
                link: OutLink {
                    steps: vec![PathStep::field(ident("id"), false)],
                    name: vec!["id".to_string()],
                },
                node: leaf(OutReach::Field),
            }],
        },
    };
    let err = refused(crate::unfold::flat_view(&arm_payload(product)));
    assert!(
        matches!(&err, UnfoldError::UnsupportedVariantPayload { found, .. } if *found == "a product"),
        "got {err}"
    );
    assert!(
        err.to_string().contains("Outer") && err.to_string().contains("Wrapped"),
        "the error names the sum and the arm: {err}"
    );

    // …and a run, for the same reason.
    let run = OutNode {
        ty: tref(syn::parse_quote!(Vec<i64>)),
        kind: TransformKind::Sequence {
            op: OutRun { borrowed: false },
            inner: Box::new(leaf(OutReach::Field)),
        },
    };
    let err = refused(crate::unfold::flat_view(&arm_payload(run)));
    assert!(
        matches!(&err, UnfoldError::UnsupportedVariantPayload { found, .. } if *found == "a run"),
        "got {err}"
    );

    // A leaf is not enough on its own: the binding an arm upgrades comes from
    // the leaf's own reach, so one reached by field access or by an accessor
    // would be grouped under the arm while claiming it was walked to.
    for (reach, what) in [
        (OutReach::Field, "field access"),
        (OutReach::Accessor, "an accessor"),
    ] {
        let err = refused(crate::unfold::flat_view(&arm_payload(leaf(reach))));
        assert!(
            matches!(&err, UnfoldError::UnsupportedVariantPayload { found, .. } if found.contains(what)),
            "got {err}"
        );
    }

    // A payload leaf that BINDS A MEMBER projects, and comes out as the
    // variant field it is.
    let (leaves, _) = crate::unfold::flat_view(&arm_payload(leaf(member())))
        .expect("a payload that binds a member projects");
    assert!(
        leaves.iter().any(|l| matches!(
            &l.source,
            LeafSource::VariantField { variant, .. } if variant == "Wrapped"
        )),
        "the arm upgrades its payload's binding"
    );

    // The mirror: a member binding with no arm above it says which variant it
    // is matched out of, so nothing can complete it.
    let err = refused(crate::unfold::flat_view(&OutNode {
        ty: tref(syn::parse_quote!(Outer)),
        kind: TransformKind::Product {
            op: OutProduct::Records,
            children: vec![OutChild {
                link: OutLink {
                    steps: Vec::new(),
                    name: vec!["stray".to_string()],
                },
                node: leaf(member()),
            }],
        },
    }));
    assert!(
        matches!(&err, UnfoldError::VariantMemberOutsideArm { .. }),
        "got {err}"
    );
}

/// #444 (review): a `Choice` and a variant arm must be paired, both ways — the
/// same "bind the relationship in both directions" rule as the member/arm check
/// above, one level up.
///
/// A choice whose alternative is anything else flattens into a selector
/// followed by a leaf belonging to no alternative; an arm anywhere else groups
/// its leaves by a tag no selector chooses between. Both flattened cleanly
/// before this, so a plan could look valid and be uninterpretable.
#[test]
fn a_choice_and_a_variant_arm_must_be_paired() {
    use crate::transform::TransformKind;

    let payload = || OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: vec!["v0".to_string()],
        },
        node: OutNode {
            ty: tref(syn::parse_quote!(i64)),
            kind: TransformKind::Leaf(OutLeaf {
                nullable: false,
                identity: false,
                reach: OutReach::VariantMember(syn::Member::Unnamed(syn::Index::from(0usize))),
            }),
        },
    };
    let arm = || OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: Vec::new(),
        },
        node: OutNode {
            ty: tref(syn::parse_quote!(Outer)),
            kind: TransformKind::Product {
                op: OutProduct::Variant {
                    name: ident("Wrapped"),
                    tag: 0,
                },
                children: vec![payload()],
            },
        },
    };
    let choice_over = |variants: Vec<OutChild>| OutNode {
        ty: tref(syn::parse_quote!(Outer)),
        kind: TransformKind::Choice {
            op: OutChoice {
                name: "tag".to_string(),
            },
            variants,
        },
    };

    // 1. A choice whose alternative is a plain leaf.
    let err = refused(crate::unfold::flat_view(&choice_over(vec![OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: vec!["nope".to_string()],
        },
        node: OutNode {
            ty: tref(syn::parse_quote!(i64)),
            kind: TransformKind::Leaf(OutLeaf {
                nullable: false,
                identity: false,
                reach: OutReach::Field,
            }),
        },
    }])));
    assert!(
        matches!(
            &err,
            UnfoldError::ChoiceAlternativeNotAnArm { found, .. } if *found == "a leaf"
        ),
        "got {err}"
    );

    // 2. An arm under an ordinary product, and at the root.
    let err = refused(crate::unfold::flat_view(&OutNode {
        ty: tref(syn::parse_quote!(Holder)),
        kind: TransformKind::Product {
            op: OutProduct::Records,
            children: vec![arm()],
        },
    }));
    assert!(
        matches!(&err, UnfoldError::VariantArmOutsideChoice { variant } if variant == "Wrapped"),
        "got {err}"
    );
    let err = refused(crate::unfold::flat_view(&arm().node));
    assert!(
        matches!(&err, UnfoldError::VariantArmOutsideChoice { .. }),
        "got {err}"
    );

    // …and under EACH arity layer. Both, because "the parent positions a node
    // can have" is the whole argument for the check being exhaustive, and an
    // unchecked one is a shape that still flattens into grouped leaves with no
    // selector.
    for layer in [
        TransformKind::Optional {
            op: (),
            inner: Box::new(arm().node),
        },
        TransformKind::Sequence {
            op: OutRun { borrowed: false },
            inner: Box::new(arm().node),
        },
    ] {
        let err = refused(crate::unfold::flat_view(&OutNode {
            ty: tref(syn::parse_quote!(Wrapper)),
            kind: layer,
        }));
        assert!(
            matches!(&err, UnfoldError::VariantArmOutsideChoice { .. }),
            "got {err}"
        );
    }

    // 3. The paired shape still projects, selector first then the arm's leaf.
    let (leaves, _) =
        crate::unfold::flat_view(&choice_over(vec![arm()])).expect("a paired sum projects");
    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0].source, LeafSource::SumTag);
    assert_eq!(leaves[1].group, Some(0));
}

/// #444 §3: a claim says which converter carries a value, not what the position
/// means. Where the claimed node is already a leaf, its own semantics have to
/// survive the replacement — otherwise selecting over a decomposition tree
/// silently turns a nullable variant payload into a non-null accessor read.
#[test]
fn selecting_an_existing_leaf_keeps_its_semantics() {
    use crate::transform::TransformKind;

    let rich = OutLeaf {
        nullable: true,
        identity: true,
        reach: OutReach::VariantMember(syn::Member::Unnamed(syn::Index::from(0usize))),
    };
    let tree = OutNode {
        ty: tref(syn::parse_quote!(Outer)),
        kind: TransformKind::Choice {
            op: OutChoice {
                name: "tag".to_string(),
            },
            variants: vec![OutChild {
                link: OutLink {
                    steps: Vec::new(),
                    name: Vec::new(),
                },
                node: OutNode {
                    ty: tref(syn::parse_quote!(Outer)),
                    kind: TransformKind::Product {
                        op: OutProduct::Variant {
                            name: ident("Wrapped"),
                            tag: 0,
                        },
                        children: vec![OutChild {
                            link: OutLink {
                                steps: Vec::new(),
                                name: vec!["wrapped_v0".to_string()],
                            },
                            node: OutNode {
                                ty: tref(syn::parse_quote!(Payload)),
                                kind: TransformKind::Leaf(rich.clone()),
                            },
                        }],
                    },
                },
            }],
        },
    };

    // The adapter converts the payload directly, and through the BORROWED
    // reading — so the reading changes and the meaning of the position must not.
    let selected = crate::unfold::select(&tree, &mut |node, _link| {
        (node.ty.spell().to_string() == "Payload").then(|| node.ty.borrowed())
    });

    let TransformKind::Choice { variants, .. } = &selected.kind else {
        panic!("the choice survives selection")
    };
    let TransformKind::Product { children, .. } = &variants[0].node.kind else {
        panic!("the arm survives selection")
    };
    let TransformKind::Leaf(op) = &children[0].node.kind else {
        panic!("the claimed payload is a leaf")
    };
    assert_eq!(
        children[0].node.ty.spell().to_string(),
        "& Payload",
        "the claim replaces the reading"
    );
    assert!(op.nullable, "a nullable payload stays nullable");
    assert!(op.identity, "an identity leaf stays identity");
    assert!(
        matches!(&op.reach, OutReach::VariantMember(syn::Member::Unnamed(i)) if i.index == 0),
        "a variant member keeps naming its member, not a generic field read"
    );
}

/// #447 §5: one semantic tree, two adapters, deliberately different wire
/// layouts — and the same semantic child order and the same direct-converter
/// cutoff in both.
///
/// The point of the shared layer is that an adapter contributes *node-level
/// lowering* and nothing else: no second recursive walk over `TypeRef`, no
/// struct-field walker of its own. These two stand in for the shapes C and JNI
/// actually choose — one flat and positional, one object-shaped — and neither
/// needs a line of recursion to produce it.
#[test]
fn one_semantic_tree_lowers_into_two_wire_layouts() {
    use crate::transform::{Lowered, TransformKind, TransformLowerer};

    let field = |name: &str, node: OutNode| OutChild {
        link: OutLink {
            steps: vec![PathStep::field(ident(name), false)],
            name: vec![name.to_string()],
        },
        node,
    };
    let leaf = |ty: syn::Type| OutNode {
        ty: tref(ty),
        kind: TransformKind::Leaf(OutLeaf {
            nullable: false,
            identity: false,
            reach: OutReach::Field,
        }),
    };
    let arm = |name: &str, tag: i32, children: Vec<OutChild>| OutChild {
        link: OutLink {
            steps: Vec::new(),
            name: Vec::new(),
        },
        node: OutNode {
            ty: tref(syn::parse_quote!(Kind)),
            kind: TransformKind::Product {
                op: OutProduct::Variant {
                    name: ident(name),
                    tag,
                },
                children,
            },
        },
    };

    // All four node kinds, in one value: a record of a scalar, a run, an
    // optional, and a dispatch.
    let tree = OutNode {
        ty: tref(syn::parse_quote!(Reading)),
        kind: TransformKind::Product {
            op: OutProduct::Records,
            children: vec![
                field("id", leaf(syn::parse_quote!(i64))),
                field(
                    "tags",
                    OutNode {
                        ty: tref(syn::parse_quote!(Vec<String>)),
                        kind: TransformKind::Sequence {
                            op: OutRun { borrowed: false },
                            inner: Box::new(leaf(syn::parse_quote!(String))),
                        },
                    },
                ),
                field(
                    "note",
                    OutNode {
                        ty: tref(syn::parse_quote!(Option<String>)),
                        kind: TransformKind::Optional {
                            op: (),
                            inner: Box::new(leaf(syn::parse_quote!(String))),
                        },
                    },
                ),
                field(
                    "kind",
                    OutNode {
                        ty: tref(syn::parse_quote!(Kind)),
                        kind: TransformKind::Choice {
                            op: OutChoice {
                                name: "tag".to_string(),
                            },
                            variants: vec![
                                arm("Exact", 0, vec![field("v", leaf(syn::parse_quote!(i64)))]),
                                arm("Missing", 1, Vec::new()),
                            ],
                        },
                    },
                ),
            ],
        },
    };

    /// Records the order a lowering met its fields in, so two adapters can be
    /// compared on the one thing they must agree about.
    #[derive(Default)]
    struct Order(Vec<String>);

    impl Order {
        /// Named edges only: an arity layer reaches its inner node directly and
        /// a choice arm is not a field, so neither names a position.
        fn record(&mut self, link: Option<&OutLink>) {
            if let Some(name) = link.and_then(|l| l.name.first()) {
                self.0.push(name.clone());
            }
        }
    }

    /// A flat, positional layout — C's shape. Every value takes its own slot,
    /// presence is a leading flag, a run is a pointer and a length, and a
    /// dispatch is a tag followed by its arms' slots.
    struct FlatWire(Order);
    impl TransformLowerer<OutOfRust> for FlatWire {
        type Value = Vec<String>;
        type Error = std::convert::Infallible;

        fn descend(
            &mut self,
            _node: &OutNode,
            link: Option<&OutLink>,
        ) -> Result<crate::transform::Descend<Self::Value>, Self::Error> {
            self.0.record(link);
            Ok(crate::transform::Descend::Recurse)
        }

        fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<Self::Value, Self::Error> {
            Ok(vec![node.ty.spell().to_string()])
        }
        fn product(
            &mut self,
            _node: &OutNode,
            _op: &OutProduct,
            children: Lowered<'_, OutOfRust, Self::Value>,
        ) -> Result<Self::Value, Self::Error> {
            let mut out = Vec::new();
            for (child, slots) in children {
                if let Some(name) = child.link.name.first() {
                    out.extend(slots.into_iter().map(|s| format!("{name}:{s}")));
                } else {
                    out.extend(slots);
                }
            }
            Ok(out)
        }
        fn optional(
            &mut self,
            _node: &OutNode,
            _op: &(),
            _inner: &OutNode,
            value: Self::Value,
        ) -> Result<Self::Value, Self::Error> {
            let mut out = vec!["present".to_string()];
            out.extend(value);
            Ok(out)
        }
        fn sequence(
            &mut self,
            _node: &OutNode,
            _op: &OutRun,
            _inner: &OutNode,
            _value: Self::Value,
        ) -> Result<Self::Value, Self::Error> {
            Ok(vec!["ptr".to_string(), "len".to_string()])
        }
        fn choice(
            &mut self,
            _node: &OutNode,
            _op: &OutChoice,
            variants: Lowered<'_, OutOfRust, Self::Value>,
        ) -> Result<Self::Value, Self::Error> {
            let mut out = vec!["tag".to_string()];
            for (_, slots) in variants {
                out.extend(slots);
            }
            Ok(out)
        }
    }

    /// An object-shaped layout — the JVM's. One entry per field whatever its
    /// shape, with the arity written into the entry rather than spent on slots.
    struct ObjectWire(Order);
    impl TransformLowerer<OutOfRust> for ObjectWire {
        type Value = String;
        type Error = std::convert::Infallible;

        fn descend(
            &mut self,
            _node: &OutNode,
            link: Option<&OutLink>,
        ) -> Result<crate::transform::Descend<Self::Value>, Self::Error> {
            self.0.record(link);
            Ok(crate::transform::Descend::Recurse)
        }

        fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<Self::Value, Self::Error> {
            Ok(node.ty.spell().to_string())
        }
        fn product(
            &mut self,
            _node: &OutNode,
            _op: &OutProduct,
            children: Lowered<'_, OutOfRust, Self::Value>,
        ) -> Result<Self::Value, Self::Error> {
            let mut parts = Vec::new();
            for (child, value) in children {
                if let Some(name) = child.link.name.first() {
                    parts.push(format!("{name}: {value}"));
                } else {
                    parts.push(value);
                }
            }
            Ok(format!("{{{}}}", parts.join(", ")))
        }
        fn optional(
            &mut self,
            _node: &OutNode,
            _op: &(),
            _inner: &OutNode,
            value: Self::Value,
        ) -> Result<Self::Value, Self::Error> {
            Ok(format!("{value}?"))
        }
        fn sequence(
            &mut self,
            _node: &OutNode,
            _op: &OutRun,
            _inner: &OutNode,
            value: Self::Value,
        ) -> Result<Self::Value, Self::Error> {
            Ok(format!("List<{value}>"))
        }
        fn choice(
            &mut self,
            _node: &OutNode,
            _op: &OutChoice,
            variants: Lowered<'_, OutOfRust, Self::Value>,
        ) -> Result<Self::Value, Self::Error> {
            Ok(format!(
                "sealed({})",
                variants
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect::<Vec<_>>()
                    .join(" | ")
            ))
        }
    }

    let mut flat = FlatWire(Order::default());
    let flat_slots = tree.lower(&mut flat).unwrap();
    let mut object = ObjectWire(Order::default());
    let object_shape = tree.lower(&mut object).unwrap();

    // Deliberately different layouts: the same value, spent very differently.
    assert_eq!(
        flat_slots,
        vec![
            "id:i64",
            "tags:ptr",
            "tags:len",
            "note:present",
            "note:String",
            "kind:tag",
            "kind:v:i64",
        ]
    );
    assert_eq!(
        object_shape,
        "{id: i64, tags: List<String>, note: String?, kind: sealed({v: i64} | {})}"
    );

    // …and the one thing they must agree about: the semantic order of the
    // children, which is the tree's and not either adapter's.
    assert_eq!(flat.0 .0, ["id", "tags", "note", "kind", "v"]);
    assert_eq!(flat.0 .0, object.0 .0);

    // A claimed subtree is atomic for BOTH: neither lowering reaches the run's
    // element, because the cutoff is the tree's too.
    let selected = crate::unfold::select(&tree, &mut |node, _link| {
        (node.ty.spell().to_string() == "Vec < String >").then(|| node.ty.clone())
    });
    let mut flat = FlatWire(Order::default());
    let flat_slots = selected.lower(&mut flat).unwrap();
    let mut object = ObjectWire(Order::default());
    let object_shape = selected.lower(&mut object).unwrap();
    assert_eq!(
        flat_slots.iter().filter(|s| s.starts_with("tags")).count(),
        1,
        "the claimed run is one crossing, not a pointer and a length: {flat_slots:?}"
    );
    assert!(
        object_shape.contains("tags: Vec < String >"),
        "and not a List of its elements either: {object_shape}"
    );
}
