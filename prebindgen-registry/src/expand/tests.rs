use quote::ToTokens;

use super::*;
use crate::test_util::SpellForTest;

/// A fixture declaration's identity. A declaration is a key, not a spelling —
/// see `ConstructorDecl::target`.
fn key(s: &str) -> crate::registry::TypeKey {
    crate::registry::TypeKey::parse(s).expect("a fixture type")
}

/// The children of a product node — a constructor's arguments, in parameter
/// order.
fn args(node: &InNode) -> &[InChild] {
    match &node.kind {
        TransformKind::Product { children, .. } => children,
        _ => panic!("a constructor product, not a dispatch"),
    }
}

/// Whether a constructor argument is itself built, rather than decoded from one
/// wire slot.
fn is_build(child: &InChild) -> bool {
    !matches!(child.node.kind, TransformKind::Leaf(_))
}

/// A reading for a fixture type — see the twin in `core/unfold/tests.rs`.
/// Plan leaves carry `TypeRef`s, and a fixture naming a type inline needs one.
fn tref(ty: syn::Type) -> prebindgen_flat::flat::TypeRef {
    prebindgen_flat::flat::Flat::builder()
        .build()
        .expect("an empty model")
        .classify(&ty)
        .expect("a fixture type the language accepts")
}
use crate::{registry::Registry, test_util::scanned_with as reg_with};

fn src_qualify(id: &syn::Ident) -> syn::Path {
    syn::parse_quote!(zenoh_flat::#id)
}

#[test]
fn single_constructor_plan_and_fold() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // Single build-from variant = one Ctor arm (no selector), declared
    // per-fn (`.expand_param`).
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_keyexpr_try_from"))]),
    });

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
    assert_eq!(plan.selector(), None);
    assert_eq!(plan.leaves.len(), 1);
    assert_eq!(plan.leaves[0].name.to_string(), "a");
    assert_eq!(plan.leaves[0].ty.spell().to_string(), "String");

    let locals = vec![ident("a")];
    let folded = emit_fold(plan, &locals, &src_qualify);
    let s = folded.to_token_stream().to_string();
    assert!(s.contains("z_keyexpr_try_from"), "fold calls ctor: {}", s);
    assert!(s.contains("map_err"), "fallible ctor mapped: {}", s);
}

#[test]
fn constructor_plan_and_fold() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ]),
    });
    // selector + try_from(String) + identity(ZKeyExpr) = 3 leaves
    // `&ZKeyExpr` consumer ⇒ borrowed identity leaf (clone-preserving).
    // Leaf types registered as required inputs (so the resolver builds
    // their converters).
    // `attachment: Option<ZZBytes>` with single `z_zbytes_from_vec(Vec<u8>)`.

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
    assert_eq!(plan.selector(), Some(0));
    // selector + try_from(String) + identity(ZKeyExpr) = 3 leaves
    assert_eq!(plan.leaves.len(), 3);
    assert_eq!(plan.leaves[0].ty.spell().to_string(), "i32");
    assert_eq!(plan.leaves[1].ty.spell().to_string(), "Option < String >");
    // `&ZKeyExpr` consumer ⇒ borrowed identity leaf (clone-preserving).
    assert_eq!(
        plan.leaves[2].ty.spell().to_string(),
        "Option < & ZKeyExpr >"
    );
    let arms = plan.tree.arms();
    assert_eq!(arms.len(), 2);
    assert!(arms[0].ctor().is_some());
    assert!(arms[1].ctor().is_none(), "identity arm");
    assert!(
        matches!(
            &arms[1].kind,
            TransformKind::Product {
                op: InProduct::Identity {
                    lift: Lift::CloneDeref
                },
                ..
            }
        ),
        "by-ref identity clones"
    );

    // Leaf types registered as required inputs (so the resolver builds
    // their converters).
    assert!(reg.input_types[&plan.leaves[1].ty.key()].root);

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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_zbytes_from_vec(bytes: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_session_delete(s: &ZSession, attachment: Option<ZZBytes>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_delete"),
        param: ident("attachment"),
        declared_target: Some(key("ZZBytes")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_zbytes_from_vec"))]),
    });
    // nullable leaf wrapping the ctor param
    // `encoding: Option<&ZEncoding>` with single, infallible
    // `z_encoding_from_string(String) -> ZEncoding`.

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
    assert!(matches!(plan.shape(), FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(!plan.by_ref);
    assert_eq!(plan.leaves.len(), 1);
    // nullable leaf wrapping the ctor param
    assert_eq!(
        plan.leaves[0].ty.spell().to_string(),
        "Option < Vec < u8 > >"
    );
    // The layer owns that one slot and binds its payload (`^`); the
    // constructor under it takes the binding, having no slot of its own.
    assert_eq!(
        plan.tree
            .lower(&mut RenderIn)
            .expect("lowering cannot fail"),
        "z_zbytes_from_vec(^)?^#0"
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_encoding_from_string(s: String) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_put"),
        param: ident("encoding"),
        declared_target: Some(key("ZEncoding")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_encoding_from_string"))]),
    });
    // `encoding: Option<&ZEncoding>` built from a TWO-arg, infallible
    // `z_encoding_from_id(i32, Option<String>) -> ZEncoding`: an explicit
    // `present: bool` flag + two plain (non-`Option`-wrapped) arg leaves.

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
    assert!(matches!(plan.shape(), FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
    assert_eq!(plan.leaves[0].ty.spell().to_string(), "Option < String >");
    assert_eq!(
        plan.target.spell().to_string(),
        "ZEncoding",
        "target peeled through Option<&_>"
    );
}

#[test]
fn optional_byref_multi_arg_ctor() {
    // `encoding: Option<&ZEncoding>` built from a TWO-arg, infallible
    // `z_encoding_from_id(i32, Option<String>) -> ZEncoding`: an explicit
    // `present: bool` flag + two plain (non-`Option`-wrapped) arg leaves.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_encoding_from_id(id: i32, schema: Option<String>) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_put"),
        param: ident("encoding"),
        declared_target: Some(key("ZEncoding")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_encoding_from_id"))]),
    });
    // leaf 0 = present:bool, leaf 1 = id:i32, leaf 2 = schema:Option<String>
    // `encoding: Option<&ZEncoding>` with TWO variants — build-from
    // `z_encoding_from_id(i32, Option<String>)` OR identity — composes the
    // selector dispatch under `Optional`: the selector leaf also encodes
    // absence (`-1` = `None`). The ctor's own `Option<String>` arg is a
    // PASSTHROUGH leaf (kept as `Option<String>`, not double-wrapped —
    // `None` is a legitimate value for the taken arm).

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
    assert!(matches!(plan.shape(), FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
    assert_eq!(plan.present(), Some(0), "explicit presence flag at leaf 0");
    // leaf 0 = present:bool, leaf 1 = id:i32, leaf 2 = schema:Option<String>
    assert_eq!(plan.leaves.len(), 3);
    assert_eq!(plan.leaves[0].name.to_string(), "encoding_present");
    assert_eq!(plan.leaves[0].ty.spell().to_string(), "bool");
    assert_eq!(plan.leaves[1].name.to_string(), "encoding_id");
    assert_eq!(plan.leaves[1].ty.spell().to_string(), "i32");
    assert_eq!(plan.leaves[2].name.to_string(), "encoding_schema");
    assert_eq!(plan.leaves[2].ty.spell().to_string(), "Option < String >");

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
fn optional_combined_selector_encodes_absence() {
    // `encoding: Option<&ZEncoding>` with TWO variants — build-from
    // `z_encoding_from_id(i32, Option<String>)` OR identity — composes the
    // selector dispatch under `Optional`: the selector leaf also encodes
    // absence (`-1` = `None`). The ctor's own `Option<String>` arg is a
    // PASSTHROUGH leaf (kept as `Option<String>`, not double-wrapped —
    // `None` is a legitimate value for the taken arm).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_encoding_from_id(id: i32, schema: Option<String>) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_put"),
        param: ident("encoding"),
        declared_target: Some(key("ZEncoding")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_encoding_from_id")),
            Variant::Identity,
        ]),
    });
    // leaves: sel:i32, id:Option<i32> (wrapped), schema:Option<String>
    // (passthrough), identity:Option<&ZEncoding>.
    // `Iterable(Construct)` is not yet produced by `apply` (no `Vec<_>`
    // param expansion is declared), but the fold is emit-ready: a hand-built
    // plan must produce the `into_iter().map(...).collect::<Result<Vec<_>,_>>()`
    // form, with the inner single-arg ctor applied per element.
    // A `.default()` ZKeyExpr constructor auto-`construct`s every matching
    // param of every declared fn — except where `.skip_default_construct`'d.

    apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply optional combined");
    let plan = reg
        .expansion_plans
        .get(&(ident("z_session_put"), ident("encoding")))
        .unwrap();
    assert!(matches!(plan.shape(), FoldShape::Optional((), _)));
    assert!(plan.produces_option());
    assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
    assert_eq!(plan.selector(), Some(0), "selector at leaf 0");
    assert_eq!(plan.present(), None, "absence rides the selector, no flag");
    // leaves: sel:i32, id:Option<i32> (wrapped), schema:Option<String>
    // (passthrough), identity:Option<&ZEncoding>.
    assert_eq!(plan.leaves.len(), 4);
    assert_eq!(plan.leaves[0].name.to_string(), "encoding_sel");
    assert_eq!(plan.leaves[0].ty.spell().to_string(), "i32");
    assert_eq!(plan.leaves[1].ty.spell().to_string(), "Option < i32 >");
    assert_eq!(
        plan.leaves[2].ty.spell().to_string(),
        "Option < String >",
        "already-Option ctor arg is NOT double-wrapped"
    );
    assert_eq!(
        plan.leaves[3].ty.spell().to_string(),
        "Option < & ZEncoding >"
    );
    let arms = plan.tree.arms();
    assert!(
        matches!(
            &args(arms[0])[1].node.kind,
            TransformKind::Leaf(InLeaf::Slot {
                slot: InSlot { slot: 2, .. },
                wrapped: false
            })
        ),
        "an already-Option ctor arg passes through unwrapped"
    );
    assert!(
        matches!(
            &arms[1].kind,
            TransformKind::Product {
                op: InProduct::Identity {
                    lift: Lift::CloneDeref
                },
                ..
            }
        ),
        "borrowed identity arm clones"
    );

    let locals = vec![ident("sel"), ident("id"), ident("schema"), ident("enc")];
    let s = emit_fold(plan, &locals, &src_qualify)
        .to_token_stream()
        .to_string();
    assert!(s.contains("if sel < 0"), "selector absence gate: {}", s);
    assert!(
        s.contains("z_encoding_from_id (__p0 , schema)"),
        "wrapped id unwrapped, passthrough schema passed directly: {}",
        s
    );
    assert!(
        s.contains("Clone :: clone"),
        "identity arm clones the borrow: {}",
        s
    );
    assert!(
        s.contains("Some") && s.contains("None"),
        "maps Option: {}",
        s
    );
}

#[test]
fn iterable_emit_shape() {
    // `Iterable(Construct)` is not yet produced by `apply` (no `Vec<_>`
    // param expansion is declared), but the fold is emit-ready: a hand-built
    // plan must produce the `into_iter().map(...).collect::<Result<Vec<_>,_>>()`
    // form, with the inner single-arg ctor applied per element.
    // The run owns the wire slot; the constructor under it takes the element
    // the layer bound.
    let core = InNode {
        ty: tref(syn::parse_quote!(ZKeyExpr)),
        kind: TransformKind::Product {
            op: InProduct::Ctor {
                func: ident("z_keyexpr_try_from"),
                fallible: true,
            },
            children: vec![InChild {
                link: InLink { by_ref: false },
                node: InNode {
                    ty: tref(syn::parse_quote!(String)),
                    kind: TransformKind::Leaf(InLeaf::Bound),
                },
            }],
        },
    };
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(core),
        },
    };
    let mut layout = crate::expand::SlotLayout::default();
    layout.claim(ident("kes"), crate::expand::SlotKind::Value);
    let plan = FoldPlan {
        target: tref(syn::parse_quote!(ZKeyExpr)),
        by_ref: false,
        leaves: wire_leaves(&tree),
        layout,
        selector: tree.selector(),
        present: tree.present(),
        tree,
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
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        "fn z_session_undeclare(s: &ZSession, k: ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.constructors.push(ConstructorDecl {
        target: key("ZKeyExpr"),
        variants: vec![Variant::Ctor(ident("z_keyexpr_try_from"))],
        default: true,
    });
    // Opt the undeclare's `k` out (must stay a handle).
    // Opt the undeclare's `k` out (must stay a handle).
    exp.skip_construct
        .insert((ident("z_session_undeclare"), ident("k")));
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
    let mut reg: Registry<()> = reg_with(&[
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
    exp.constructors.push(ConstructorDecl {
        target: key("ZKeyExpr"),
        variants: vec![Variant::Ctor(ident("z_keyexpr_try_from"))],
        default: true,
    });
    apply(&mut reg, &exp, &declared, &accessor, &Default::default()).expect("apply");
    assert!(reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_intersects"), ident("a"))));
    assert!(!reg
        .expansion_plans
        .contains_key(&(ident("z_keyexpr_clone"), ident("ke"))));

    // An explicit per-fn input flatten on an accessor is a build error.
    let mut reg2: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_clone(ke: &ZKeyExpr) -> ZKeyExpr { todo!() }",
    ]);
    let mut exp2 = Expansions::default();
    exp2.expands.push(ExpandDecl {
        func: ident("z_keyexpr_clone"),
        param: ident("ke"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_keyexpr_try_from"))]),
    });
    let err = apply(&mut reg2, &exp2, &declared, &accessor, &Default::default()).unwrap_err();
    assert!(matches!(err, ExpandError::ConstructOnAccessor { .. }));
}

#[test]
fn recursive_input_nests_param_constructors() {
    // z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample, consumed
    // by z_reply_sample(sample: ZSample). ZSample's default input expands
    // the `sample` param into z_sample_new's params, each of which (ZKeyExpr,
    // ZZBytes) recursively expands per ITS default input.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample { todo!() }",
        "fn z_keyexpr_try_from(s: String) -> ZKeyExpr { todo!() }",
        "fn z_zbytes_from_vec(b: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_reply_sample(sample: ZSample) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // Default inputs for ZSample (single), ZKeyExpr (combined: try_from|id),
    // ZZBytes (single).
    exp.constructors.push(ConstructorDecl {
        target: key("ZSample"),
        variants: vec![Variant::Ctor(ident("z_sample_new"))],
        default: true,
    });
    exp.constructors.push(ConstructorDecl {
        target: key("ZKeyExpr"),
        variants: vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ],
        default: true,
    });
    exp.constructors.push(ConstructorDecl {
        target: key("ZZBytes"),
        variants: vec![Variant::Ctor(ident("z_zbytes_from_vec"))],
        default: true,
    });
    // Top: single z_sample_new ctor, 2 args, both recursive Build.
    // key_expr's nested build is COMBINED (try_from | identity ⇒ selector).
    // payload's nested build is SINGLE (no selector).
    // Wire leaves: key-expr selector + try_from String + identity ZKeyExpr +
    // zbytes Vec<u8> — all flattened into the one signature.
    // A → B → A constructor cycle is a build error (not an infinite expansion).
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
    assert_eq!(plan.selector(), None);
    assert_eq!(plan.tree.arms().len(), 1);
    let args = args(plan.tree.core());
    assert_eq!(args.len(), 2);
    assert!(is_build(&args[0]), "key_expr is a nested build");
    assert!(is_build(&args[1]), "payload is a nested build");
    // key_expr's nested build is COMBINED (try_from | identity ⇒ selector).
    assert!(
        args[0].node.selector().is_some(),
        "ZKeyExpr default input is combined"
    );
    assert_eq!(args[0].node.arms().len(), 2);
    // payload's nested build is SINGLE (no selector).
    assert!(
        args[1].node.selector().is_none(),
        "ZZBytes default input is single"
    );
    // Wire leaves: key-expr selector + try_from String + identity ZKeyExpr +
    // zbytes Vec<u8> — all flattened into the one signature.
    let leaf_tys: Vec<String> = plan
        .leaves
        .iter()
        .map(|l| l.ty.spell().to_string())
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
    let mut reg: Registry<()> = reg_with(&[
        "fn make_a(b: B) -> A { todo!() }",
        "fn make_b(a: A) -> B { todo!() }",
        "fn consume_a(a: A) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.constructors.push(ConstructorDecl {
        target: key("A"),
        variants: vec![Variant::Ctor(ident("make_a"))],
        default: true,
    });
    exp.constructors.push(ConstructorDecl {
        target: key("B"),
        variants: vec![Variant::Ctor(ident("make_b"))],
        default: true,
    });
    // C5 validation map: a variant ctor ident that names no `#[prebindgen]`
    // fn is a hard `UnknownConstructor` at resolve time — a typo'd
    // `expand_param!(...).variant(fun!(…))` cannot silently vanish.
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
    // Mutual recursion terminates with a typed error that says WHERE: the
    // argument chain `a → b → a`, not just the function it started from.
    assert!(
        err.to_string().contains("input expansion at `a.b.a`:"),
        "the cycle names its argument chain: {err}"
    );
}

/// C5 validation map: a variant ctor ident that names no `#[prebindgen]`
/// fn is a hard `UnknownConstructor` at resolve time — a typo'd
/// `expand_param!(...).variant(fun!(…))` cannot silently vanish.
#[test]
fn unknown_constructor_errors() {
    use prebindgen_flat::types_util::ident;
    let mut reg: Registry<()> =
        reg_with(&["fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }"]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_keyexpr_try_from_typo"))]),
    });
    // C5 validation map: a variant ctor that exists but does not produce the
    // expanded type is a hard `TargetMismatch` — the ctor's return is
    // cross-checked against the parameter's type.
    let err = apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExpandError::UnknownConstructor(_)), "{err}");
}

/// C5 validation map: a variant ctor that exists but does not produce the
/// expanded type is a hard `TargetMismatch` — the ctor's return is
/// cross-checked against the parameter's type.
#[test]
fn constructor_target_mismatch_errors() {
    use prebindgen_flat::types_util::ident;
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_new(s: String) -> ZSample { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_sample_new"))]),
    });
    let err = apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExpandError::TargetMismatch { .. }), "{err}");
}

/// #96: structurally invalid declaration sets are rejected with COLLECTED
/// diagnostics — every offender reported in one error, before any plan
/// resolution.
#[test]
fn invalid_declarations_collected() {
    use prebindgen_flat::types_util::ident;
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_session_get(s: &ZSession, k: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // Duplicate constructor target (two records, same TypeKey).
    for _ in 0..2 {
        exp.constructors.push(ConstructorDecl {
            target: key("ZKeyExpr"),
            variants: vec![Variant::Ctor(ident("z_keyexpr_try_from"))],
            default: true,
        });
    }
    // Empty constructor variant list.
    exp.constructors.push(ConstructorDecl {
        target: key("ZEmpty"),
        variants: vec![],
        default: true,
    });
    // Duplicate per-fn expand for the same (fn, param), plus an empty subset.
    for sel in [
        ExpandSel::Subset(vec![Variant::Ctor(ident("z_keyexpr_try_from"))]),
        ExpandSel::Subset(vec![]),
    ] {
        exp.expands.push(ExpandDecl {
            func: ident("z_session_get"),
            param: ident("k"),
            declared_target: Some(key("ZKeyExpr")),
            sel,
        });
    }
    let err = apply(
        &mut reg,
        &exp,
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap_err();
    let ExpandError::InvalidDeclarations { entries } = &err else {
        panic!("expected InvalidDeclarations, got {err}");
    };
    // All four problems collected in one pass.
    assert_eq!(entries.len(), 4, "{err}");
    let text = err.to_string();
    assert!(
        text.contains("duplicate constructor declaration for `ZKeyExpr`"),
        "{text}"
    );
    assert!(
        text.contains("constructor for `ZEmpty` declares no variants"),
        "{text}"
    );
    assert!(
        text.contains("expand for parameter `k` of `z_session_get` declares no variants"),
        "{text}"
    );
    assert!(
        text.contains("duplicate expand declaration for parameter `k` of `z_session_get`"),
        "{text}"
    );
}

/// A `Vec<T>` parameter is **not** a `T` parameter, even when `T` has a
/// constructor.
///
/// Expansion builds one value: `FoldPlan`'s shape is `Base` or `Optional(Base)`,
/// with no iterable arm. So the layer peel here stops at the borrow — if it also
/// peeled the `Sequence`, a `Vec<T>` parameter would match a `T` constructor and
/// the wrapper would reconstruct a single `T` and hand it to a parameter
/// expecting the collection. Not a rejected plan: a wrong one, in generated code.
///
/// Missed once, and by everything: the whole suite passed either way, and
/// regen-check was byte-identical, because no example declares a `Vec<T>`
/// parameter whose element is constructible. That is what this fixture is.
#[test]
fn a_vec_param_does_not_match_its_elements_constructor() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_join_all(parts: Vec<ZKeyExpr>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    // The type-level default: every `ZKeyExpr` parameter may be built from a
    // `String`. `parts` is a `Vec<ZKeyExpr>`, so it is not one of them.
    exp.constructors.push(ConstructorDecl {
        target: key("ZKeyExpr"),
        variants: vec![Variant::Ctor(ident("z_keyexpr_try_from"))],
        default: true,
    });

    apply(
        &mut reg,
        &exp,
        &[ident("z_keyexpr_join_all")].into_iter().collect(),
        &Default::default(),
        &Default::default(),
    )
    .expect("apply");

    assert!(
        !reg.expansion_plans
            .contains_key(&(ident("z_keyexpr_join_all"), ident("parts"))),
        "a Vec<ZKeyExpr> parameter must not be expanded as one ZKeyExpr; \
         plans: {:?}",
        reg.expansion_plans.keys().collect::<Vec<_>>()
    );
}

/// A stand-in language adapter (#442), the input-direction twin of
/// `unfold::tests::Render`: it says what a leaf, a constructor and a dispatch
/// *render as* and nothing else. No recursion, no `TypeRef` walk, no leaf-index
/// arithmetic — the registry supplies the descent, the child order and the
/// links.
struct RenderIn;

impl crate::transform::TransformLowerer<IntoRust> for RenderIn {
    type Value = String;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, _node: &InNode, op: &InLeaf) -> Result<String, Self::Error> {
        Ok(match op {
            InLeaf::Slot { slot, wrapped } => {
                format!("#{}{}", slot.slot, if *wrapped { "?" } else { "" })
            }
            InLeaf::Bound => "^".to_string(),
        })
    }

    fn product(
        &mut self,
        _node: &InNode,
        op: &InProduct,
        children: crate::transform::Lowered<'_, IntoRust, String>,
    ) -> Result<String, Self::Error> {
        let args: Vec<String> = children
            .iter()
            .map(|(child, value)| format!("{}{value}", if child.link.by_ref { "&" } else { "" }))
            .collect();
        Ok(match op {
            InProduct::Ctor { func, fallible } => format!(
                "{func}{}({})",
                if *fallible { "!" } else { "" },
                args.join(", ")
            ),
            InProduct::Identity { lift } => {
                format!(
                    "self{}({})",
                    match lift {
                        Lift::Direct => "",
                        Lift::CloneDeref => ".clone",
                        Lift::MoveDeref => ".move",
                    },
                    args.join(", ")
                )
            }
        })
    }

    fn choice(
        &mut self,
        _node: &InNode,
        op: &InChoice,
        variants: crate::transform::Lowered<'_, IntoRust, String>,
    ) -> Result<String, Self::Error> {
        let arms: Vec<String> = variants.into_iter().map(|(_, v)| v).collect();
        Ok(format!("#{} ? {}", op.selector.slot, arms.join(" | ")))
    }

    fn optional(
        &mut self,
        _node: &InNode,
        op: &InPresence,
        _inner: &InNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(match op {
            InPresence::Selector => format!("{value}?sel"),
            InPresence::Flag(s) => format!("{value}?#{}", s.slot),
            InPresence::Payload { slot, .. } => format!("{value}?^#{}", slot.slot),
        })
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &InRun,
        _inner: &InNode,
        value: String,
    ) -> Result<String, Self::Error> {
        Ok(format!("[{value}]*#{}", op.slot.slot))
    }
}

#[test]
fn core_lowers_through_the_shared_visitor() {
    // `z_sample_new(&ZKeyExpr, ZZBytes)` where BOTH arguments have their own
    // default constructor: a product whose children are themselves built, one
    // of them a dispatch. That is every into-Rust node kind in one tree, with
    // a by-ref link over the nested one.
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_new(ke: &ZKeyExpr, payload: ZZBytes) -> ZSample { todo!() }",
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_zbytes_from_vec(v: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_reply_sample(sample: ZSample) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.constructors.push(ConstructorDecl {
        target: key("ZSample"),
        variants: vec![Variant::Ctor(ident("z_sample_new"))],
        default: true,
    });
    exp.constructors.push(ConstructorDecl {
        target: key("ZKeyExpr"),
        variants: vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ],
        default: true,
    });
    exp.constructors.push(ConstructorDecl {
        target: key("ZZBytes"),
        variants: vec![Variant::Ctor(ident("z_zbytes_from_vec"))],
        default: true,
    });
    exp.expands.push(ExpandDecl {
        func: ident("z_reply_sample"),
        param: ident("sample"),
        declared_target: Some(key("ZSample")),
        sel: ExpandSel::TopLevel,
    });

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
        .get(&(ident("z_reply_sample"), ident("sample")))
        .expect("plan");

    // The whole construction, read through the hooks alone: the key expression
    // is a dispatch behind a by-ref link, the payload a single constructor, and
    // every wire slot names its index in `plan.leaves`.
    let rendered = plan
        .tree
        .lower(&mut RenderIn)
        .expect("lowering cannot fail");
    assert_eq!(
        rendered,
        "z_sample_new(&#0 ? z_keyexpr_try_from!(#1?) | self.clone(#2?), z_zbytes_from_vec(#3))"
    );

    // The signature is COLLECTED from those same nodes: each slot's name and
    // type come from the node that uses it, in slot order.
    let leaves: Vec<(String, String)> = plan
        .leaves
        .iter()
        .map(|l| (l.name.to_string(), l.ty.spell().to_string()))
        .collect();
    assert_eq!(
        leaves,
        vec![
            ("sample_ke_sel".to_string(), "i32".to_string()),
            ("sample_ke_0".to_string(), "Option < String >".to_string()),
            (
                "sample_ke_1".to_string(),
                "Option < & ZKeyExpr >".to_string()
            ),
            ("sample_payload".to_string(), "Vec < u8 >".to_string()),
        ]
    );
}

/// #444 §1/§3: an adapter's selection on the input side is a **tree**, and
/// producing it is also a choice of layout — a claimed construction's
/// arguments, selector and presence slots collapse into the one value that
/// crosses instead, and the surviving slots close ranks.
#[test]
fn selecting_a_construction_replaces_its_slots() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ]),
    });
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
        .expect("plan");

    // Unselected: a selector plus one slot per arm.
    assert_eq!(plan.leaves().len(), 3);
    assert_eq!(plan.selector(), Some(0));

    // An adapter that builds a whole `ZKeyExpr` from one value of its own.
    let selected = crate::expand::select(plan.tree(), &mut |node, _link| {
        (node.ty.spell().to_string() == "ZKeyExpr").then(|| Claim::direct(node.ty.clone()))
    })
    .unwrap();
    let leaves = crate::expand::wire_leaves(&selected);
    assert_eq!(
        leaves.len(),
        1,
        "the dispatch and both arms collapse into the value that crosses"
    );
    assert_eq!(leaves[0].ty.spell().to_string(), "ZKeyExpr");
    assert_eq!(
        crate::expand::dependencies(&selected)
            .required
            .iter()
            .map(|t| t.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["ZKeyExpr"],
        "and the arms' own crossings are not demanded"
    );

    // Registration follows the same tree, so nothing roots what is gone.
    let rooted = |reg: &Registry<()>, ty: syn::Type| -> Option<bool> {
        reg.input_types
            .get(&TypeKey::from_type(&ty))
            .map(|cell| cell.root)
    };
    let mut claimed: Registry<()> = reg_with(&[]);
    crate::expand::register_dependencies(&mut claimed, &selected);
    assert_eq!(rooted(&claimed, syn::parse_quote!(ZKeyExpr)), Some(true));
    assert_ne!(
        rooted(&claimed, syn::parse_quote!(Option<String>)),
        Some(true)
    );
}

/// The surviving slots keep their order and close ranks, because the foreign
/// signature is a sequence: what a caller passes second must still be what the
/// wrapper reads second.
#[test]
fn selecting_renumbers_the_surviving_slots() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_sample_new(ke: &ZKeyExpr, payload: ZZBytes) -> ZSample { todo!() }",
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_zbytes_from_vec(v: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_reply_sample(sample: ZSample) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    for (target, variants) in [
        (key("ZSample"), vec![Variant::Ctor(ident("z_sample_new"))]),
        (
            key("ZKeyExpr"),
            vec![
                Variant::Ctor(ident("z_keyexpr_try_from")),
                Variant::Identity,
            ],
        ),
        (
            key("ZZBytes"),
            vec![Variant::Ctor(ident("z_zbytes_from_vec"))],
        ),
    ] {
        exp.constructors.push(ConstructorDecl {
            target,
            variants,
            default: true,
        });
    }
    exp.expands.push(ExpandDecl {
        func: ident("z_reply_sample"),
        param: ident("sample"),
        declared_target: Some(key("ZSample")),
        sel: ExpandSel::TopLevel,
    });
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
        .get(&(ident("z_reply_sample"), ident("sample")))
        .expect("plan");

    // Slots 0..2 build the key expression, slot 3 the payload.
    assert_eq!(plan.leaves().len(), 4);
    assert_eq!(plan.leaves()[3].name.to_string(), "sample_payload");

    // Claiming the key expression drops slots 0..2 and the payload closes up
    // into position 0 — keeping its name, because it is the same value.
    let selected = crate::expand::select(plan.tree(), &mut |node, _link| {
        (node.ty.spell().to_string() == "ZKeyExpr").then(|| Claim::clone_deref(node.ty.borrowed()))
    })
    .unwrap();
    let leaves = crate::expand::wire_leaves(&selected);
    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0].ty.spell().to_string(), "& ZKeyExpr");
    assert_eq!(leaves[1].name.to_string(), "sample_payload");

    // The signature is only half the contract: the claimed node's own type says
    // it produces an owned `ZKeyExpr`, so a borrowed reading has to be cloned
    // back up to it. Otherwise the consumer borrows a borrow.
    let locals = vec![ident("ke"), ident("payload")];
    let compact: String = crate::expand::emit_fold_tree(&selected, &locals, &src_qualify)
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();

    assert!(
        compact.contains("Clone::clone(&*ke)"),
        "a borrowed selected reading is cloned into the owned value the node declares: {compact}"
    );
    assert!(
        compact.contains("z_sample_new(&__a0,__a1)"),
        "…so the outer constructor takes one borrow, not two: {compact}"
    );
    assert!(
        !compact.contains("&&"),
        "no double borrow reaches the call site: {compact}"
    );
}

/// Claiming a subtree that occupies no wire slot of its own — the constructor
/// node inside an `InPresence::Payload` optional — is an error, not a panic.
///
/// In the `Payload` case the layer's single slot holds the `Option`-wrapped
/// argument, and the constructor under it receives the payload as
/// `InLeaf::Bound` (bound by the layer, not from a wire slot).  There is no
/// wire position for a converter to land on, so `select` returns
/// `SelectError::BoundOnlySubtree`.
#[test]
fn claiming_a_bound_only_subtree_is_an_error() {
    // `attachment: Option<ZZBytes>` with one single-arg constructor.
    // The resulting tree has Optional(Payload, inner=Product([Bound])).
    let mut reg: Registry<()> = reg_with(&[
        "fn z_zbytes_from_vec(bytes: Vec<u8>) -> ZZBytes { todo!() }",
        "fn z_session_delete(s: &ZSession, attachment: Option<ZZBytes>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_delete"),
        param: ident("attachment"),
        declared_target: Some(key("ZZBytes")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_zbytes_from_vec"))]),
    });
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
        .get(&(ident("z_session_delete"), ident("attachment")))
        .unwrap();

    // The outer Optional has the only wire slot; its inner Product's children
    // are all InLeaf::Bound.  Claiming the inner Product (skipping the Optional
    // by not claiming it) has no wire position to inherit.
    let result = crate::expand::select(plan.tree(), &mut |node, _link| {
        // Claim on the ZZBytes product, but NOT on the wrapping Optional.
        (node.ty.spell().to_string() == "ZZBytes"
            && !matches!(node.kind, TransformKind::Optional { .. }))
        .then(|| Claim::direct(node.ty.clone()))
    });
    let err = match result {
        // `InNode` has no `Debug`, so the refusal is taken by matching.
        Ok(_) => panic!("claiming a bound-only subtree must be refused"),
        Err(e) => e,
    };
    assert_eq!(
        err,
        crate::expand::SelectError::BoundOnlySubtree {
            claimed: "ZZBytes".to_string()
        },
        "the error says which construction"
    );
    assert!(err.to_string().contains("no wire slot"), "and why: {err}");
}

/// #444 §3: `Option`-wrapping by selector presence belongs to the POSITION, not
/// to the value in it. Claiming a subtree inside a live `Choice` arm leaves the
/// dispatch standing, so that position is still absent whenever another arm is
/// selected — the claimed value has to keep saying "not this one".
#[test]
fn a_claim_inside_a_live_choice_stays_wrapped() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ]),
    });
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
        .expect("plan");

    // A selector plus one `Option`-wrapped slot per arm.
    assert_eq!(plan.leaves().len(), 3);
    assert_eq!(plan.selector(), Some(0));

    // The adapter converts the constructor's own argument directly, and claims
    // ONLY that — the choice above it stands.
    // The arm's argument is offered WITH its selector-presence `Option` on, so
    // the adapter answers about the wrapped position.
    let selected = crate::expand::select(plan.tree(), &mut |node, _link| {
        (node.ty.spell().to_string() == "String").then(|| Claim::direct(node.ty.clone()))
    })
    .unwrap();

    assert_eq!(
        crate::expand::wire_leaves(&selected).len(),
        3,
        "the dispatch survives, so the selector and both arms keep their slots"
    );

    // `wrapped` is read straight off the tree by the construct emitter, which
    // uses it to decide whether a slot is unwrapped (missing ⇒ `Err`) before it
    // reaches the constructor — so the claimed leaf itself is what to check.
    fn claimed_wrapped(node: &crate::expand::InNode) -> Option<bool> {
        match &node.kind {
            TransformKind::Leaf(crate::expand::InLeaf::Slot { slot, wrapped }) => {
                (slot.name == "a_0").then_some(*wrapped)
            }
            TransformKind::Leaf(_) => None,
            TransformKind::Product { children, .. } => {
                children.iter().find_map(|c| claimed_wrapped(&c.node))
            }
            TransformKind::Choice { variants, .. } => {
                variants.iter().find_map(|v| claimed_wrapped(&v.node))
            }
            TransformKind::Optional { inner, .. } | TransformKind::Sequence { inner, .. } => {
                claimed_wrapped(inner)
            }
        }
    }
    assert_eq!(
        claimed_wrapped(&selected),
        Some(true),
        "a claim inside a live choice keeps the position's selector presence"
    );
}

/// #444 §3: claiming a **structural** node inside a live `Choice` — an arm's
/// constructor product rather than one of its argument leaves.
///
/// The distinction matters because the two are offered differently. An argument
/// leaf arrives with the position's `Option` already on it; the product above it
/// arrives as the bare constructed type. If the claim's reading were stored
/// unchanged, the leaf would declare a plain `ZKeyExpr` crossing while carrying
/// `wrapped = true`, and the construct emitter would pattern-match that value as
/// `Some(..)` — a tree that cannot generate compiling Rust.
#[test]
fn claiming_an_arm_product_keeps_the_position_optional() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ]),
    });
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
        .expect("plan");

    // Claim each arm's product — reached by a link, unlike the root choice — so
    // the dispatch above them survives.
    let selected = crate::expand::select(plan.tree(), &mut |node, link| {
        (link.is_some() && matches!(node.kind, TransformKind::Product { .. }))
            .then(|| Claim::direct(node.ty.clone()))
    })
    .unwrap();

    let leaves = crate::expand::wire_leaves(&selected);
    assert_eq!(
        leaves.len(),
        3,
        "the dispatch survives, so the selector and both arms keep their slots"
    );
    assert_eq!(
        leaves[1].ty.spell().to_string(),
        "Option < ZKeyExpr >",
        "the claimed arm still has to be able to say `not this one`"
    );

    // The signature being right is half of it: the fold expression owes the
    // same `Result<target, String>` the unselected tree owes, or the claimed
    // arm hands the dispatch an `Option` where its siblings give a `Result`.
    let locals = vec![ident("sel"), ident("v0"), ident("vid")];
    let folded = crate::expand::emit_fold_tree(&selected, &locals, &src_qualify);
    let compact: String = folded
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();

    assert!(compact.contains("matchsel"), "dispatch survives: {compact}");
    // The present path unwraps the arm's presence and lifts the value.
    assert!(
        compact.contains("Option::Some(__v)=>::core::result::Result::Ok(__v)"),
        "a claimed arm lifts its value into `Ok`: {compact}"
    );
    // …and the missing one is an error, not a silently absent value.
    assert!(
        compact.contains("identityvariantvaluemissing"),
        "a claimed arm rejects a missing slot: {compact}"
    );
}

/// #444 §3: claiming an **arity layer** — the `Option` shape of the parameter
/// itself, not a construction under it.
///
/// A layer maps its inner value over a shape, and a claim replaces the whole
/// layer, its presence slot included. Collapsing that to a bare identity hands
/// `Clone::clone(&*v)` an `Option<&T>`, which does not deref, and owes
/// `Option<T>` while producing `Option<&T>`. The layer has to survive the
/// claim, over a core that lifts the bound value.
#[test]
fn claiming_an_arity_layer_keeps_its_mapping() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_encoding_from_string(s: String) -> ZEncoding { todo!() }",
        "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_session_put"),
        param: ident("encoding"),
        declared_target: Some(key("ZEncoding")),
        sel: ExpandSel::Subset(vec![Variant::Ctor(ident("z_encoding_from_string"))]),
    });
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
        .get(&(ident("z_session_put"), ident("encoding")))
        .expect("plan");

    // The adapter decodes the whole optional parameter from one wire value,
    // through the reading the layer node itself names — and says how the
    // payload it binds becomes the owned target: `&ZEncoding` is a borrow of
    // the caller's value, so it is copied out of rather than consumed.
    let selected = crate::expand::select(plan.tree(), &mut |node, _link| {
        matches!(node.kind, TransformKind::Optional { .. })
            .then(|| Claim::clone_deref(node.ty.clone()))
    })
    .unwrap();

    let leaves = crate::expand::wire_leaves(&selected);
    assert_eq!(
        leaves.len(),
        1,
        "the construction's slots collapse into one"
    );
    assert_eq!(leaves[0].ty.spell().to_string(), "Option < & ZEncoding >");
    assert_eq!(
        crate::expand::dependencies(&selected)
            .required
            .iter()
            .map(|t| t.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["Option < & ZEncoding >"],
        "only the claimed crossing is required; the constructor's argument is gone"
    );

    // The shape survives and still targets the owned value: an adapter reading
    // the selected tree gets the same answers the plan gave.
    assert!(matches!(selected.shape(), FoldShape::Optional((), _)));
    assert_eq!(
        selected.core().ty.spell().to_string(),
        "ZEncoding",
        "the claim replaces the construction, not what it targets"
    );

    let locals = vec![ident("enc")];
    let compact: String = crate::expand::emit_fold_tree(&selected, &locals, &src_qualify)
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();

    // `Some`: the layer binds the payload, and the borrowed reading is cloned
    // up to the owned value the fold owes.
    assert!(
        compact.contains("Option::Some(__inner)=>{(::core::result::Result::Ok(::core::clone::Clone::clone(&*__inner))).map(::core::option::Option::Some)}"),
        "the present arm yields an owned value inside the option: {compact}"
    );
    // `None`: absent stays absent, and is not an error.
    assert!(
        compact
            .contains("Option::None=>{::core::result::Result::Ok(::core::option::Option::None)}"),
        "the absent arm yields `Ok(None)`: {compact}"
    );
    // The whole point: no `Clone::clone(&*enc)` on the option itself.
    assert!(
        !compact.contains("clone(&*enc)"),
        "the option is not dereferenced: {compact}"
    );
}

/// #444 §3: the run analogue of [`claiming_an_arity_layer_keeps_its_mapping`],
/// and the type a claimed run advertises for the value it binds.
///
/// A run's emitter iterates its slot, so a borrowed collection binds a borrowed
/// element: `&[T]` binds `&T`, not `&[T]`. Deriving that with `sequence_elem`
/// alone answers `None` for the borrowed spelling — a reference is not looked
/// through the way a transparent wrapper is — and taking the reading itself
/// would advertise the whole collection on a node whose expression is one
/// element. Nothing in the current emitter reads that type, so only an adapter
/// lowerer would see it, which is exactly who this tree is for.
#[test]
fn claiming_a_run_binds_one_borrowed_element() {
    // The run owns the wire slot; the constructor under it takes the element
    // the layer bound.
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZKeyExpr)),
                kind: TransformKind::Product {
                    op: InProduct::Ctor {
                        func: ident("z_keyexpr_try_from"),
                        fallible: true,
                    },
                    children: vec![InChild {
                        link: InLink { by_ref: false },
                        node: InNode {
                            ty: tref(syn::parse_quote!(String)),
                            kind: TransformKind::Leaf(InLeaf::Bound),
                        },
                    }],
                },
            }),
        },
    };

    // The adapter decodes the whole run from one borrowed slice.
    let selected = crate::expand::select(&tree, &mut |node, _link| {
        matches!(node.kind, TransformKind::Sequence { .. })
            .then(|| Claim::clone_deref(tref(syn::parse_quote!(&[ZKeyExpr]))))
    })
    .unwrap();

    // The layer survives, carrying the claimed reading on its slot.
    let leaves = crate::expand::wire_leaves(&selected);
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].ty.spell().to_string(), "& [ZKeyExpr]");

    // …and the node under it advertises what the layer actually binds.
    let TransformKind::Sequence { inner, .. } = &selected.kind else {
        panic!("the run survives the claim")
    };
    let TransformKind::Product { children, .. } = &inner.kind else {
        panic!("a claimed run stands over an identity core")
    };
    assert_eq!(
        children[0].node.ty.spell().to_string(),
        "& ZKeyExpr",
        "iterating a borrowed collection binds a borrowed element, not the collection"
    );

    let locals = vec![ident("kes")];
    let compact: String = crate::expand::emit_fold_tree(&selected, &locals, &src_qualify)
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();

    // Each borrowed element is cloned into the owned `Vec<ZKeyExpr>` the fold
    // owes, and the run still collects.
    assert!(
        compact.contains("kes.into_iter().map(|__inner|"),
        "the run still iterates its slot: {compact}"
    );
    assert!(
        compact.contains("Clone::clone(&*__inner)"),
        "the bound element is cloned to owned: {compact}"
    );
    assert!(
        compact.contains("collect::<::core::result::Result<::std::vec::Vec<_>,_>>()"),
        "…and collects into an owned Vec: {compact}"
    );
}

/// An arity layer claimed with a reading that has none of that layer's shape is
/// refused, rather than binding a value the layer cannot produce.
#[test]
fn claiming_a_layer_with_the_wrong_shape_is_an_error() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZKeyExpr)),
                kind: TransformKind::Leaf(InLeaf::Bound),
            }),
        },
    };
    // `ZKeyExpr` is one value, not a run of them: there is nothing to bind.
    let err = match crate::expand::select(&tree, &mut |node, _link| {
        matches!(node.kind, TransformKind::Sequence { .. })
            .then(|| Claim::direct(tref(syn::parse_quote!(ZKeyExpr))))
    }) {
        Ok(_) => panic!("a run claimed with a non-run reading must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::LayerReadingShape { layer, .. } if *layer == "a run"
        ),
        "got {err}"
    );
    assert!(
        err.to_string().contains("ZKeyExpr") && err.to_string().contains("layer's shape"),
        "the refusal names the reading and why: {err}"
    );
}

/// #444 §3: a `Cow<'_, [T]>` run binds `&T`, not `T`.
///
/// The model looks through `Cow` — a destination language sees a run of `T`
/// either way — but the emitter runs `into_iter()` on the type as written, and
/// a `Cow` iterates as its borrowed slice whatever it currently holds. Typing
/// the bound node from the model's element alone would promise `T` where the
/// expression yields `&T`, and the fold would collect `Vec<&T>` against a node
/// declaring `Vec<T>`.
#[test]
fn claiming_a_cow_run_binds_a_borrowed_element() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZKeyExpr)),
                kind: TransformKind::Leaf(InLeaf::Bound),
            }),
        },
    };

    let selected = crate::expand::select(&tree, &mut |node, _link| {
        matches!(node.kind, TransformKind::Sequence { .. })
            .then(|| Claim::clone_deref(tref(syn::parse_quote!(Cow<'static, [ZKeyExpr]>))))
    })
    .unwrap();

    let TransformKind::Sequence { inner, .. } = &selected.kind else {
        panic!("the run survives the claim")
    };
    let TransformKind::Product { children, .. } = &inner.kind else {
        panic!("a claimed run stands over an identity core")
    };
    assert_eq!(
        children[0].node.ty.spell().to_string(),
        "& ZKeyExpr",
        "a `Cow` run iterates as its borrowed slice, so the bound element is a borrow"
    );

    let locals = vec![ident("kes")];
    let compact: String = crate::expand::emit_fold_tree(&selected, &locals, &src_qualify)
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();
    assert!(
        compact.contains("Clone::clone(&*__inner)"),
        "…so each element is cloned into the owned Vec the node declares: {compact}"
    );
}

/// An optional claimed with a reading that only *denotes* an `Option` is
/// refused: the payload emitter matches the slot as written, and `match` does
/// not see through a `Box` the way the model's layer accessors do.
#[test]
fn claiming_an_optional_behind_a_wrapper_is_an_error() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(Option<ZEncoding>)),
        kind: TransformKind::Optional {
            op: InPresence::Payload {
                slot: InSlot {
                    slot: 0,
                    name: ident("enc"),
                },
                ty: tref(syn::parse_quote!(Option<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZEncoding)),
                kind: TransformKind::Leaf(InLeaf::Bound),
            }),
        },
    };
    let err = match crate::expand::select(&tree, &mut |node, _link| {
        matches!(node.kind, TransformKind::Optional { .. })
            .then(|| Claim::direct(tref(syn::parse_quote!(Box<Option<ZEncoding>>))))
    }) {
        Ok(_) => panic!("an optional behind a wrapper must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::LayerReadingShape { layer, .. } if *layer == "an optional"
        ),
        "got {err}"
    );
}

/// A structural claim states how its reading becomes the node's value, and the
/// fold emitter executes exactly that.
///
/// The registry cannot infer this: `Box<T>` owns its target and can be moved
/// out of, while a non-owning adapter handle dereferences to storage its
/// language still owns and can only be copied from. The two present the same
/// target and the same deref shape, so the adapter — which chose the converter
/// — states the operation.
#[test]
fn a_claim_states_how_its_reading_is_lifted() {
    let ctor = || InNode {
        ty: tref(syn::parse_quote!(ZKeyExpr)),
        kind: TransformKind::Product {
            op: InProduct::Ctor {
                func: ident("z_keyexpr_try_from"),
                fallible: true,
            },
            children: vec![InChild {
                link: InLink { by_ref: false },
                node: InNode {
                    ty: tref(syn::parse_quote!(String)),
                    kind: TransformKind::Leaf(InLeaf::Slot {
                        slot: InSlot {
                            slot: 0,
                            name: ident("s"),
                        },
                        wrapped: false,
                    }),
                },
            }],
        },
    };
    let folded = |claim: Claim| -> String {
        let selected = crate::expand::select(&ctor(), &mut |node, _l| {
            matches!(node.kind, TransformKind::Product { .. }).then(|| claim.clone())
        })
        .unwrap();
        crate::expand::emit_fold_tree(&selected, &[ident("v")], &src_qualify)
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect()
    };

    // Owned: the reading IS the value, so it moves.
    assert_eq!(
        folded(Claim::direct(tref(syn::parse_quote!(ZKeyExpr)))),
        "::core::result::Result::Ok(v)"
    );
    // A non-owning handle: dereference and copy out, leaving the caller's
    // value alone. Its Rust type is the adapter's, and the fold never names it.
    assert_eq!(
        folded(Claim::clone_deref(tref(syn::parse_quote!(
            OwnedObject<ZKeyExpr>
        )))),
        "::core::result::Result::Ok(::core::clone::Clone::clone(&*v))"
    );
    // An owning wrapper: dereference and move, which asks nothing of `ZKeyExpr`
    // — in particular not `Clone`, which is why this cannot be folded into the
    // case above.
    assert_eq!(
        folded(Claim::move_deref(tref(syn::parse_quote!(Box<ZKeyExpr>)))),
        "::core::result::Result::Ok(*v)"
    );
}

/// `Cow<'_, Vec<T>>` is a run to the model and not one to the emitter: a `Cow`
/// cannot be moved out of, so `into_iter()` on it does not compile at all
/// ("cannot move out of dereference"). `Cow<'_, [T]>` iterates its slice and
/// binds `&T`; the `Vec` spelling has no such fallback, so it is refused rather
/// than bound as something the layer cannot produce.
#[test]
fn a_cow_of_vec_run_is_refused() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZKeyExpr)),
                kind: TransformKind::Leaf(InLeaf::Bound),
            }),
        },
    };
    let claim_of = |reading: syn::Type| {
        crate::expand::select(&tree, &mut |node, _l| {
            matches!(node.kind, TransformKind::Sequence { .. })
                .then(|| Claim::clone_deref(tref(reading.clone())))
        })
    };

    let err = match claim_of(syn::parse_quote!(Cow<'static, Vec<ZKeyExpr>>)) {
        Ok(_) => panic!("a `Cow<'_, Vec<T>>` run must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::LayerReadingShape { layer, .. } if *layer == "a run"
        ),
        "got {err}"
    );

    // …while the slice spelling, which does iterate, still binds a borrow.
    let selected = claim_of(syn::parse_quote!(Cow<'static, [ZKeyExpr]>)).unwrap();
    let TransformKind::Sequence { inner, .. } = &selected.kind else {
        panic!("the run survives")
    };
    let TransformKind::Product { children, .. } = &inner.kind else {
        panic!("over an identity core")
    };
    assert_eq!(children[0].node.ty.spell().to_string(), "& ZKeyExpr");
}

/// A claim on a **leaf** carries a lift like any other, and the operation has
/// to reach the constructor that reads it.
///
/// A leaf is where converter selection lands most naturally, and a bare leaf
/// has nowhere to put a deref: the enclosing construction would pass the
/// reading itself. `Direct` still stays one plain slot, because there is
/// nothing to perform.
#[test]
fn a_leaf_claim_lifts_before_its_constructor_reads_it() {
    let tree = |target: syn::Type| InNode {
        ty: tref(syn::parse_quote!(ZKeyExpr)),
        kind: TransformKind::Product {
            op: InProduct::Ctor {
                func: ident("z_keyexpr_of"),
                fallible: false,
            },
            children: vec![InChild {
                link: InLink { by_ref: false },
                node: InNode {
                    ty: tref(target),
                    kind: TransformKind::Leaf(InLeaf::Slot {
                        slot: InSlot {
                            slot: 0,
                            name: ident("k"),
                        },
                        wrapped: false,
                    }),
                },
            }],
        },
    };
    let folded = |target: syn::Type, claim: Claim| -> String {
        let selected = crate::expand::select(&tree(target), &mut |node, _l| {
            matches!(node.kind, TransformKind::Leaf(_)).then(|| claim.clone())
        })
        .unwrap();
        crate::expand::emit_fold_tree(&selected, &[ident("k")], &src_qualify)
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect()
    };

    // A non-owning handle: the constructor must read the copy, not the handle.
    let cloned = folded(
        syn::parse_quote!(ZKeyExpr),
        Claim::clone_deref(tref(syn::parse_quote!(OwnedObject<ZKeyExpr>))),
    );
    assert!(
        cloned.contains("Ok(::core::clone::Clone::clone(&*k))"),
        "the lift happens: {cloned}"
    );
    assert!(
        cloned.contains("z_keyexpr_of(__a0)"),
        "…and the constructor reads its result, not the reading: {cloned}"
    );

    // An owning wrapper moves out instead, asking nothing of `ZKeyExpr`.
    let moved = folded(
        syn::parse_quote!(ZKeyExpr),
        Claim::move_deref(tref(syn::parse_quote!(Box<ZKeyExpr>))),
    );
    assert!(moved.contains("Ok(*k)"), "the lift happens: {moved}");
    assert!(
        moved.contains("z_keyexpr_of(__a0)"),
        "…and the constructor reads its result: {moved}"
    );

    // `Direct` has nothing to perform, so the slot is still passed straight in.
    assert_eq!(
        folded(
            syn::parse_quote!(ZKeyExpr),
            Claim::direct(tref(syn::parse_quote!(ZKeyExpr)))
        ),
        "::core::result::Result::Ok(zenoh_flat::z_keyexpr_of(k))"
    );
}

/// Both deref lifts produce an owned value, so a leaf whose position holds a
/// **borrow** is refused: turning an adapter handle into `&T` is a
/// borrow-through-deref, which no [`Lift`] states. Refusing beats lowering it
/// into a value of the wrong ownership.
#[test]
fn a_non_direct_lift_onto_a_borrowed_leaf_is_an_error() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(ZKeyExpr)),
        kind: TransformKind::Product {
            op: InProduct::Ctor {
                func: ident("z_keyexpr_of"),
                fallible: false,
            },
            children: vec![InChild {
                link: InLink { by_ref: false },
                node: InNode {
                    ty: tref(syn::parse_quote!(&ZKeyExpr)),
                    kind: TransformKind::Leaf(InLeaf::Slot {
                        slot: InSlot {
                            slot: 0,
                            name: ident("k"),
                        },
                        wrapped: false,
                    }),
                },
            }],
        },
    };
    let err = match crate::expand::select(&tree, &mut |node, _l| {
        matches!(node.kind, TransformKind::Leaf(_))
            .then(|| Claim::clone_deref(tref(syn::parse_quote!(OwnedObject<ZKeyExpr>))))
    }) {
        Ok(_) => panic!("an owned lift onto a borrowed position must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::LeafLiftTarget {
                lift: Lift::CloneDeref,
                ..
            }
        ),
        "got {err}"
    );
    assert!(
        err.to_string().contains("holds a borrow"),
        "the refusal says why: {err}"
    );
}

/// #444 §3: a claim on a **selector-wrapped** leaf, where the position's
/// `Option` belongs to the dispatch and not to the value.
///
/// The emitter unwraps presence and then lifts, so that `Option` is on neither
/// end of the operation. Reading the node's stored type instead hides the real
/// target: a `&ZKeyExpr` argument inside a live arm is stored as
/// `Option<&ZKeyExpr>`, whose `borrow_target()` is `None`, so an owned-producing
/// lift slipped past the refusal and the identity advertised the position's
/// `Option` while producing the payload.
#[test]
fn a_lift_on_a_wrapped_leaf_sees_through_the_presence() {
    let tree = |arg: syn::Type| InNode {
        ty: tref(syn::parse_quote!(ZKeyExpr)),
        kind: TransformKind::Choice {
            op: InChoice {
                selector: InSlot {
                    slot: 0,
                    name: ident("sel"),
                },
            },
            variants: vec![InChild {
                link: InLink { by_ref: false },
                node: InNode {
                    ty: tref(syn::parse_quote!(ZKeyExpr)),
                    kind: TransformKind::Product {
                        op: InProduct::Ctor {
                            func: ident("z_keyexpr_of"),
                            fallible: false,
                        },
                        children: vec![InChild {
                            link: InLink { by_ref: false },
                            node: InNode {
                                // Stored WITH the selector's `Option`, which is
                                // what a live arm's argument looks like.
                                ty: tref(arg),
                                kind: TransformKind::Leaf(InLeaf::Slot {
                                    slot: InSlot {
                                        slot: 1,
                                        name: ident("a_0"),
                                    },
                                    wrapped: true,
                                }),
                            },
                        }],
                    },
                },
            }],
        },
    };
    // Stored as the PAYLOAD; the selector's `Option` is the position's, added
    // wherever the wire is asked for.
    let select = |arg: syn::Type, claim: Claim| {
        crate::expand::select(&tree(arg), &mut |node, _l| {
            matches!(node.kind, TransformKind::Leaf(_)).then(|| claim.clone())
        })
    };

    // An OWNED argument: the lift is honoured, and the node it lands on
    // advertises the unwrapped target rather than the position's `Option`.
    let selected = select(
        syn::parse_quote!(ZKeyExpr),
        Claim::clone_deref(tref(syn::parse_quote!(OwnedObject<ZKeyExpr>))),
    )
    .unwrap();
    let TransformKind::Choice { variants, .. } = &selected.kind else {
        panic!("the dispatch survives")
    };
    let TransformKind::Product { children, .. } = &variants[0].node.kind else {
        panic!("the arm survives")
    };
    assert_eq!(
        children[0].node.ty.spell().to_string(),
        "ZKeyExpr",
        "the identity owes the payload, not the position's `Option`"
    );
    let compact: String =
        crate::expand::emit_fold_tree(&selected, &[ident("sel"), ident("a_0")], &src_qualify)
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect();
    assert!(
        compact.contains(
            "Option::Some(__v)=>::core::result::Result::Ok(::core::clone::Clone::clone(&*__v))"
        ),
        "presence is unwrapped and then the lift runs: {compact}"
    );

    // A BORROWED argument: the same claim now has an owned-producing lift onto
    // a borrowed target, which the presence used to hide.
    let err = match select(
        syn::parse_quote!(&ZKeyExpr),
        Claim::clone_deref(tref(syn::parse_quote!(OwnedObject<ZKeyExpr>))),
    ) {
        Ok(_) => panic!("an owned lift onto a borrowed argument must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::LeafLiftTarget {
                lift: Lift::CloneDeref,
                ..
            }
        ),
        "got {err}"
    );

    // A reading that owns an `Option` of its own does NOT thereby carry
    // selector presence: `match` sees an outer kind, not a semantic shape, so a
    // `Box<Option<T>>` slot marked `wrapped` would emit a match the `Box`
    // blocks. Presence is added around such a reading instead — and here the
    // claim is then contradictory, so it is refused rather than lowered.
    let err = match select(
        syn::parse_quote!(String),
        Claim::direct(tref(syn::parse_quote!(Box<Option<String>>))),
    ) {
        Ok(_) => panic!("a boxed optional reading is not selector presence"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::DirectLiftMismatch { bound, target, .. }
                if bound == "Box < Option < String > >" && target == "String"
        ),
        "presence was added around the reading, so the claim reads as what it is: {err}"
    );

    // …and a reading that needs presence added gets it explicitly, so the
    // emitted match is on an `Option` the slot really has.
    let selected = select(
        syn::parse_quote!(String),
        Claim::clone_deref(tref(syn::parse_quote!(OwnedObject<String>))),
    )
    .unwrap();
    assert_eq!(
        crate::expand::wire_leaves(&selected)[1]
            .ty
            .spell()
            .to_string(),
        "Option < OwnedObject < String > >",
        "selector presence is on the slot, outermost"
    );
    let compact: String =
        crate::expand::emit_fold_tree(&selected, &[ident("sel"), ident("a_0")], &src_qualify)
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect();
    assert!(
        compact.contains(
            "Option::Some(__v)=>::core::result::Result::Ok(::core::clone::Clone::clone(&*__v))"
        ),
        "the match is on the presence the slot carries: {compact}"
    );
}

/// `Direct` says the reading IS the value, so a claim where the two differ is a
/// contradiction — and `Ok(value)` offers no coercion to paper over it, the
/// `Result`'s parameter being inferred rather than a coercion site.
#[test]
fn a_direct_claim_that_is_not_the_value_is_an_error() {
    let tree = InNode {
        ty: tref(syn::parse_quote!(Vec<ZKeyExpr>)),
        kind: TransformKind::Sequence {
            op: InRun {
                slot: InSlot {
                    slot: 0,
                    name: ident("kes"),
                },
                ty: tref(syn::parse_quote!(Vec<String>)),
            },
            inner: Box::new(InNode {
                ty: tref(syn::parse_quote!(ZKeyExpr)),
                kind: TransformKind::Leaf(InLeaf::Bound),
            }),
        },
    };
    // A borrowed run binds `&ZKeyExpr`; claiming it `Direct` would collect
    // `Vec<&ZKeyExpr>` where the node declares `Vec<ZKeyExpr>`.
    let err = match crate::expand::select(&tree, &mut |node, _l| {
        matches!(node.kind, TransformKind::Sequence { .. })
            .then(|| Claim::direct(tref(syn::parse_quote!(&[ZKeyExpr]))))
    }) {
        Ok(_) => panic!("a direct claim that is not the value must be refused"),
        Err(e) => e,
    };
    assert!(
        matches!(
            &err,
            crate::expand::SelectError::DirectLiftMismatch { bound, target, .. }
                if bound == "& ZKeyExpr" && target == "ZKeyExpr"
        ),
        "got {err}"
    );
}

/// #447 §1: presence and the crossing type are one fact.
///
/// A slot leaf's `ty` is the value the position holds; whether that position is
/// gated by a live choice is `wrapped`; and the wire type is *derived* from the
/// pair. So the state this used to be able to hold — a plain `T` beside
/// `wrapped = true`, where the signature declares `T` while the emitter matches
/// `Some` — is no longer a state: there is nowhere to write the second, and
/// disagreeing answers cannot be given because only one is stored.
#[test]
fn a_gated_slots_wire_type_is_derived_from_its_payload() {
    let mut reg: Registry<()> = reg_with(&[
        "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
        "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
    ]);
    let mut exp = Expansions::default();
    exp.expands.push(ExpandDecl {
        func: ident("z_keyexpr_intersects"),
        param: ident("a"),
        declared_target: Some(key("ZKeyExpr")),
        sel: ExpandSel::Subset(vec![
            Variant::Ctor(ident("z_keyexpr_try_from")),
            Variant::Identity,
        ]),
    });
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
        .expect("plan");

    // The tree stores the payloads…
    fn payloads(node: &InNode, out: &mut Vec<(String, bool)>) {
        match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { wrapped, .. }) => {
                out.push((node.ty.spell().to_string(), *wrapped))
            }
            TransformKind::Leaf(_) => {}
            TransformKind::Product { children, .. } => {
                children.iter().for_each(|c| payloads(&c.node, out))
            }
            TransformKind::Choice { variants, .. } => {
                variants.iter().for_each(|v| payloads(&v.node, out))
            }
            TransformKind::Optional { inner, .. } | TransformKind::Sequence { inner, .. } => {
                payloads(inner, out)
            }
        }
    }
    let mut stored = Vec::new();
    payloads(plan.tree(), &mut stored);
    assert_eq!(
        stored,
        vec![
            ("String".to_string(), true),
            ("& ZKeyExpr".to_string(), true)
        ],
        "each arm's argument is stored as its payload, gated by the dispatch"
    );

    // …and every reading of the wire derives the same type from them, rather
    // than carrying a second copy that could drift.
    let leaves = crate::expand::wire_leaves(plan.tree());
    assert_eq!(
        leaves
            .iter()
            .map(|l| l.ty.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["i32", "Option < String >", "Option < & ZKeyExpr >"],
        "the selector, then each arm's gated slot"
    );
    assert_eq!(
        crate::expand::dependencies(plan.tree())
            .required
            .iter()
            .map(|t| t.spell().to_string())
            .collect::<Vec<_>>(),
        vec!["Option < String >", "Option < & ZKeyExpr >"],
        "and the crossings demanded are those same wire types"
    );
}

/// #447 §1: the slot numbers in the tree are **derivable** — a walk in
/// allocation order reproduces every one of them.
///
/// The check that has to hold before slot allocation can move out of the
/// canonical tree and into adapter-local planning. If a projection walking the
/// semantic structure yields today's numbering, the numbering is not a fact the
/// tree has to carry; if it does not, something about the layout is not
/// positional and the relocation would change generated signatures.
///
/// Allocation is pre-order: a layer takes its own slot before the construction
/// under it, and a dispatch its selector before its arms.
#[test]
fn slot_numbers_are_derivable_from_the_tree_walk() {
    /// The slot names the tree carries, in the same walk order.
    fn stored_names(node: &InNode, out: &mut Vec<String>) {
        match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { slot, .. }) => out.push(slot.name.to_string()),
            TransformKind::Leaf(InLeaf::Bound) => {}
            TransformKind::Product { children, .. } => {
                children.iter().for_each(|c| stored_names(&c.node, out))
            }
            TransformKind::Choice { op, variants } => {
                out.push(op.selector.name.to_string());
                variants.iter().for_each(|v| stored_names(&v.node, out));
            }
            TransformKind::Optional { op, inner } => {
                match op {
                    InPresence::Selector => {}
                    InPresence::Flag(s) => out.push(s.name.to_string()),
                    InPresence::Payload { slot, .. } => out.push(slot.name.to_string()),
                }
                stored_names(inner, out);
            }
            TransformKind::Sequence { op, inner } => {
                out.push(op.slot.name.to_string());
                stored_names(inner, out);
            }
        }
    }

    fn stored(node: &InNode, out: &mut Vec<usize>) {
        match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { slot, .. }) => out.push(slot.slot),
            TransformKind::Leaf(InLeaf::Bound) => {}
            TransformKind::Product { children, .. } => {
                children.iter().for_each(|c| stored(&c.node, out))
            }
            TransformKind::Choice { op, variants } => {
                out.push(op.selector.slot);
                variants.iter().for_each(|v| stored(&v.node, out));
            }
            TransformKind::Optional { op, inner } => {
                match op {
                    InPresence::Selector => {}
                    InPresence::Flag(s) => out.push(s.slot),
                    InPresence::Payload { slot, .. } => out.push(slot.slot),
                }
                stored(inner, out);
            }
            TransformKind::Sequence { op, inner } => {
                out.push(op.slot.slot);
                stored(inner, out);
            }
        }
    }

    // Three shapes that allocate differently: a dispatch (selector then arms),
    // a single-constructor optional (the layer's own payload slot), and a
    // multi-argument optional (an explicit flag, then one slot per argument).
    /// One shape to check: what to call it, the source it needs, the expanded
    /// parameter, and the variants it expands through.
    struct Case {
        label: &'static str,
        items: Vec<&'static str>,
        func: &'static str,
        param: &'static str,
        variants: Vec<Variant>,
    }

    let cases = vec![
        Case {
            label: "dispatch",
            items: vec![
                "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
                "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
            ],
            func: "z_keyexpr_intersects",
            param: "a",
            variants: vec![
                Variant::Ctor(ident("z_keyexpr_try_from")),
                Variant::Identity,
            ],
        },
        Case {
            label: "optional payload",
            items: vec![
                "fn z_encoding_from_string(s: String) -> ZEncoding { todo!() }",
                "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
            ],
            func: "z_session_put",
            param: "encoding",
            variants: vec![Variant::Ctor(ident("z_encoding_from_string"))],
        },
    ];

    for Case {
        label,
        items,
        func,
        param,
        variants,
    } in cases
    {
        let mut reg: Registry<()> = reg_with(&items);
        let mut exp = Expansions::default();
        exp.expands.push(ExpandDecl {
            func: ident(func),
            param: ident(param),
            declared_target: None,
            sel: ExpandSel::Subset(variants),
        });
        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .unwrap_or_else(|e| panic!("{label}: apply: {e}"));
        let plan = reg
            .expansion_plans
            .get(&(ident(func), ident(param)))
            .unwrap_or_else(|| panic!("{label}: plan"));

        let mut order = Vec::new();
        stored(plan.tree(), &mut order);
        assert_eq!(
            order,
            (0..order.len()).collect::<Vec<_>>(),
            "{label}: a pre-order walk meets the slots in numbering order, so the numbers \
             are the walk's and need not be stored"
        );
        assert_eq!(
            order.len(),
            plan.leaves().len(),
            "{label}: and it meets every slot the wire has"
        );

        // …and the layout beside the tree says the same thing the tree does,
        // which is what licenses the tree to stop saying it.
        assert_eq!(
            plan.layout().len(),
            plan.leaves().len(),
            "{label}: the layout has a position per wire value"
        );
        let mut names = Vec::new();
        stored_names(plan.tree(), &mut names);
        assert_eq!(
            names,
            (0..plan.layout().len())
                .map(|i| plan.layout().name(i).to_string())
                .collect::<Vec<_>>(),
            "{label}: and calls each position what the tree calls it"
        );
    }
}
