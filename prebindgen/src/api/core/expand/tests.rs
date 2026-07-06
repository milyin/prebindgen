use quote::ToTokens;

use super::*;
use crate::api::test_util::reg_with;

fn src_qualify(id: &syn::Ident) -> syn::Path {
    syn::parse_quote!(zenoh_flat::#id)
}

#[test]
fn single_constructor_plan_and_fold() {
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // Single build-from variant = one Ctor arm (no selector), declared
    // per-fn (`.flatten_input_with`).
    exp.begin_subset(ident("z_keyexpr_intersects"), ident("a"));
    exp.push_subset_variant(ident("z_keyexpr_try_from"));

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply");

    let plan = reg
        .expansion_plans
        .get(&(ident("z_keyexpr_intersects"), ident("a")))
        .expect("plan for a");
    assert!(plan.by_ref, "param was &ZKeyExpr");
    assert_eq!(plan.selector, None);
    assert_eq!(plan.leaves.len(), 1);
    assert_eq!(plan.leaves[0].name.to_string(), "a");
    assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "String");

    let locals = vec![ident("a")];
    let folded = emit_fold(plan, &locals, &src_qualify);
    let s = folded.to_token_stream().to_string();
    assert!(s.contains("z_keyexpr_try_from"), "fold calls ctor: {}", s);
    assert!(s.contains("map_err"), "fallible ctor mapped: {}", s);
}

#[test]
fn constructor_plan_and_fold() {
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.begin_subset(ident("z_keyexpr_intersects"), ident("a"));
    exp.push_subset_variant(ident("z_keyexpr_try_from"));
    exp.push_subset_self();

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply");

    let plan = reg
        .expansion_plans
        .get(&(ident("z_keyexpr_intersects"), ident("a")))
        .unwrap();
    assert_eq!(plan.selector, Some(0));
    // selector + try_from(String) + identity(ZKeyExpr) = 3 leaves
    assert_eq!(plan.leaves.len(), 3);
    assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "i32");
    assert_eq!(
        plan.leaves[1].ty.to_token_stream().to_string(),
        "Option < String >"
    );
    // `&ZKeyExpr` consumer ⇒ borrowed identity leaf (clone-preserving).
    assert_eq!(
        plan.leaves[2].ty.to_token_stream().to_string(),
        "Option < & ZKeyExpr >"
    );
    assert_eq!(plan.variants.len(), 2);
    assert!(plan.variants[0].ctor.is_some());
    assert!(plan.variants[1].ctor.is_none(), "identity arm");
    assert!(plan.variants[1].clone, "by-ref identity clones");

    // Leaf types registered as required inputs (so the resolver builds
    // their converters).
    assert!(reg
        .required_inputs_scan
        .contains(&TypeKey::from_type(&plan.leaves[1].ty)));

    let locals = vec![ident("sel"), ident("v0"), ident("vid")];
    let folded = emit_fold(plan, &locals, &src_qualify);
    let s = folded.to_token_stream().to_string();
    assert!(s.contains("match sel"), "dispatch on selector: {}", s);
    assert!(s.contains("z_keyexpr_try_from"));
    assert!(s.contains("invalid constructor selector"));
}

#[test]
fn optional_byvalue_single_ctor() {
    // `attachment: Option<ZZBytes>` with single `z_zbytes_from_vec(Vec<u8>)`.
    let mut reg = reg_with(&[
        "fn z_zbytes_from_vec(bytes: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_session_delete(s: &ZSession, attachment: Option<ZZBytes>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.begin_subset(ident("z_session_delete"), ident("attachment"));
    exp.push_subset_variant(ident("z_zbytes_from_vec"));

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply optional by-value");
    let plan = reg
        .expansion_plans
        .get(&(ident("z_session_delete"), ident("attachment")))
        .unwrap();
    assert!(matches!(plan.shape, FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(!plan.by_ref);
    assert_eq!(plan.leaves.len(), 1);
    // nullable leaf wrapping the ctor param
    assert_eq!(
        plan.leaves[0].ty.to_token_stream().to_string(),
        "Option < Vec < u8 > >"
    );

    let locals = vec![ident("att")];
    let s = emit_fold(plan, &locals, &src_qualify)
        .to_token_stream()
        .to_string();
    assert!(s.contains("z_zbytes_from_vec"), "fold calls ctor: {}", s);
    assert!(
        s.contains("Some") && s.contains("None"),
        "maps Option: {}",
        s
    );
}

#[test]
fn optional_byref_single_ctor() {
    // `encoding: Option<&ZEncoding>` with single, infallible
    // `z_encoding_from_string(String) -> ZEncoding`.
    let mut reg = reg_with(&[
        "fn z_encoding_from_string(s: String) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.begin_subset(ident("z_session_put"), ident("encoding"));
    exp.push_subset_variant(ident("z_encoding_from_string"));

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply optional by-ref");
    let plan = reg
        .expansion_plans
        .get(&(ident("z_session_put"), ident("encoding")))
        .unwrap();
    assert!(matches!(plan.shape, FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
    assert_eq!(
        plan.leaves[0].ty.to_token_stream().to_string(),
        "Option < String >"
    );
    assert_eq!(
        plan.target.to_token_stream().to_string(),
        "ZEncoding",
        "target peeled through Option<&_>"
    );
}

#[test]
fn optional_byref_multi_arg_ctor() {
    // `encoding: Option<&ZEncoding>` built from a TWO-arg, infallible
    // `z_encoding_from_id(i32, Option<String>) -> ZEncoding`: an explicit
    // `present: bool` flag + two plain (non-`Option`-wrapped) arg leaves.
    let mut reg = reg_with(&[
        "fn z_encoding_from_id(id: i32, schema: Option<String>) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.begin_subset(ident("z_session_put"), ident("encoding"));
    exp.push_subset_variant(ident("z_encoding_from_id"));

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply optional multi-arg by-ref");
    let plan = reg
        .expansion_plans
        .get(&(ident("z_session_put"), ident("encoding")))
        .unwrap();
    assert!(matches!(plan.shape, FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
    assert_eq!(plan.present, Some(0), "explicit presence flag at leaf 0");
    // leaf 0 = present:bool, leaf 1 = id:i32, leaf 2 = schema:Option<String>
    assert_eq!(plan.leaves.len(), 3);
    assert_eq!(plan.leaves[0].name.to_string(), "encoding_present");
    assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "bool");
    assert_eq!(plan.leaves[1].name.to_string(), "encoding_id");
    assert_eq!(plan.leaves[1].ty.to_token_stream().to_string(), "i32");
    assert_eq!(plan.leaves[2].name.to_string(), "encoding_schema");
    assert_eq!(
        plan.leaves[2].ty.to_token_stream().to_string(),
        "Option < String >"
    );

    let locals = vec![ident("pres"), ident("id"), ident("schema")];
    let s = emit_fold(plan, &locals, &src_qualify)
        .to_token_stream()
        .to_string();
    assert!(s.contains("if pres"), "presence-flag gated: {}", s);
    assert!(
        s.contains("z_encoding_from_id"),
        "fold calls multi-arg ctor: {}",
        s
    );
    assert!(
        s.contains("Some") && s.contains("None"),
        "maps Option: {}",
        s
    );
}

#[test]
fn optional_combined_rejected() {
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_session_get(s: &ZSession, ke: Option<ZKeyExpr>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.begin_subset(ident("z_session_get"), ident("ke"));
    exp.push_subset_variant(ident("z_keyexpr_try_from"));
    exp.push_subset_self();

    match apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    ) {
        Err(ExpandError::UnsupportedOptional { .. }) => {}
        other => panic!("expected UnsupportedOptional, got {:?}", other.err()),
    }
}

#[test]
fn iterable_emit_shape() {
    // `Iterable(Construct)` is not yet produced by `apply` (no `Vec<_>`
    // param expansion is declared), but the fold is emit-ready: a hand-built
    // plan must produce the `into_iter().map(...).collect::<Result<Vec<_>,_>>()`
    // form, with the inner single-arg ctor applied per element.
    let plan = FoldPlan {
        target: syn::parse_quote!(ZKeyExpr),
        by_ref: false,
        shape: FoldShape::Iterable(Box::new(FoldShape::Base)),
        leaves: vec![FoldLeaf {
            name: ident("kes"),
            ty: syn::parse_quote!(Vec<String>),
        }],
        selector: None,
        present: None,
        variants: vec![FoldVariant {
            ctor: Some(ident("z_keyexpr_try_from")),
            fallible: true,
            clone: false,
            inputs: vec![FoldArg::Leaf(0)],
        }],
    };
    let locals = vec![ident("kes")];
    let s = emit_fold(&plan, &locals, &src_qualify)
        .to_token_stream()
        .to_string();
    assert!(s.contains("into_iter"), "iterates: {}", s);
    assert!(s.contains("collect"), "collects: {}", s);
    assert!(
        s.contains("Vec") && s.contains("z_keyexpr_try_from"),
        "collects Result<Vec<_>> via per-elem ctor: {}",
        s
    );
    assert!(!plan.produces_option());
}

#[test]
fn default_constructor_auto_applies_and_skips() {
    // A `.default()` ZKeyExpr constructor auto-`construct`s every matching
    // param of every declared fn — except where `.skip_default_construct`'d.
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        "fn z_session_undeclare(s: &ZSession, k: ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
    exp.add_constructor_variant(ident("z_keyexpr_try_from"));
    // Opt the undeclare's `k` out (must stay a handle).
    exp.add_skip_default_construct(ident("z_session_undeclare"), ident("k"));
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_keyexpr_intersects", "z_session_undeclare"]
            .iter()
            .map(|s| ident(s))
            .collect();
    apply(
        &mut reg,
        &exp,
        &declared,
        &Default::default(),
        &Default::default(),
    )
    .expect("apply");

    // Both `&ZKeyExpr` params of intersects are constructed.
    assert!(reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_intersects"), ident("a"))));
    assert!(reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_intersects"), ident("b"))));
    // The skipped param is NOT.
    assert!(!reg
        .expansion_plans
        .contains_key(&(ident("z_session_undeclare"), ident("k"))));
}

#[test]
fn default_constructor_skips_accessor_and_explicit_construct_errors() {
    let mut reg = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        "fn z_keyexpr_clone(ke: &ZKeyExpr) -> ZKeyExpr { todo!() }",
    ]);
    let accessor: std::collections::HashSet<syn::Ident> =
        ["z_keyexpr_clone"].iter().map(|s| ident(s)).collect();
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_keyexpr_intersects", "z_keyexpr_clone"]
            .iter()
            .map(|s| ident(s))
            .collect();

    // `.default()` skips the accessor's `ke`, constructs the consumer's a/b.
    let mut exp = Expansions::default();
    exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
    exp.add_constructor_variant(ident("z_keyexpr_try_from"));
    apply(&mut reg, &exp, &declared, &accessor, &Default::default()).expect("apply");
    assert!(reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_intersects"), ident("a"))));
    assert!(!reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_clone"), ident("ke"))));

    // An explicit per-fn input flatten on an accessor is a build error.
    let mut reg2 = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_clone(ke: &ZKeyExpr) -> ZKeyExpr { todo!() }",
    ]);
    let mut exp2 = Expansions::default();
    exp2.begin_subset(ident("z_keyexpr_clone"), ident("ke"));
    exp2.push_subset_variant(ident("z_keyexpr_try_from"));
    let err = apply(&mut reg2, &exp2, &declared, &accessor, &Default::default()).unwrap_err();
    assert!(matches!(err, ExpandError::ConstructOnAccessor { .. }));
}

#[test]
fn recursive_input_nests_param_constructors() {
    // z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample, consumed
    // by z_reply_sample(sample: ZSample). ZSample's default input expands
    // the `sample` param into z_sample_new's params, each of which (ZKeyExpr,
    // ZZBytes) recursively expands per ITS default input.
    let mut reg = reg_with(&[
        "fn z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample { todo!() }",
        "fn z_keyexpr_try_from(s: String) -> ZKeyExpr { todo!() }",
        "fn z_zbytes_from_vec(b: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_reply_sample(sample: ZSample) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // Default inputs for ZSample (single), ZKeyExpr (combined: try_from|id),
    // ZZBytes (single).
    exp.ensure_default_constructor(syn::parse_quote!(ZSample));
    exp.add_constructor_variant(ident("z_sample_new"));
    exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
    exp.add_constructor_variant(ident("z_keyexpr_try_from"));
    exp.add_constructor_variant_id();
    exp.ensure_default_constructor(syn::parse_quote!(ZZBytes));
    exp.add_constructor_variant(ident("z_zbytes_from_vec"));
    let declared: std::collections::HashSet<syn::Ident> =
        ["z_reply_sample"].iter().map(|s| ident(s)).collect();
    apply(
        &mut reg,
        &exp,
        &declared,
        &Default::default(),
        &Default::default(),
    )
    .expect("apply");

    let plan = reg
        .expansion_plans
        .get(&(ident("z_reply_sample"), ident("sample")))
        .expect("sample plan");
    // Top: single z_sample_new ctor, 2 args, both recursive Build.
    assert_eq!(plan.selector, None);
    assert_eq!(plan.variants.len(), 1);
    let args = &plan.variants[0].inputs;
    assert_eq!(args.len(), 2);
    assert!(
        matches!(args[0], FoldArg::Build(_)),
        "key_expr is a nested build"
    );
    assert!(
        matches!(args[1], FoldArg::Build(_)),
        "payload is a nested build"
    );
    // key_expr's nested build is COMBINED (try_from | identity ⇒ selector).
    if let FoldArg::Build(b) = &args[0] {
        assert!(b.selector.is_some(), "ZKeyExpr default input is combined");
        assert_eq!(b.variants.len(), 2);
    }
    // payload's nested build is SINGLE (no selector).
    if let FoldArg::Build(b) = &args[1] {
        assert!(b.selector.is_none(), "ZZBytes default input is single");
    }
    // Wire leaves: key-expr selector + try_from String + identity ZKeyExpr +
    // zbytes Vec<u8> — all flattened into the one signature.
    let leaf_tys: Vec<String> = plan
        .leaves
        .iter()
        .map(|l| l.ty.to_token_stream().to_string())
        .collect();
    assert!(
        leaf_tys.iter().any(|t| t.contains("i32")),
        "selector leaf: {leaf_tys:?}"
    );
    assert!(
        leaf_tys.iter().any(|t| t.contains("String")),
        "try_from arg: {leaf_tys:?}"
    );
}

#[test]
fn recursive_input_cycle_errors() {
    // A → B → A constructor cycle is a build error (not an infinite expansion).
    let mut reg = reg_with(&[
        "fn make_a(b: B) -> A { todo!() }",
        "fn make_b(a: A) -> B { todo!() }",
        "fn consume_a(a: A) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.ensure_default_constructor(syn::parse_quote!(A));
    exp.add_constructor_variant(ident("make_a"));
    exp.ensure_default_constructor(syn::parse_quote!(B));
    exp.add_constructor_variant(ident("make_b"));
    let declared: std::collections::HashSet<syn::Ident> =
        ["consume_a"].iter().map(|s| ident(s)).collect();
    let err = apply(
        &mut reg,
        &exp,
        &declared,
        &Default::default(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExpandError::InputCycle { .. }), "got {err:?}");
}
