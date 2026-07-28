//! `expand_return!(T).fields(fields!(t_to_struct))` — deriving a type's output
//! boundary from its **value form** (the struct gathering its own accessors)
//! instead of restating the field list, which is how the two drift apart.
//!
//! The contract each test below pins down: `.fields()` is `.field()` applied to
//! every struct field, so a field still crosses by **its own type's** default
//! output boundary.

use super::*;

/// The value-form fixture: a `ZSample` handle whose fields cover the shapes
/// that decide behaviour — a handle field with its own `expand_return!`
/// (decomposed, not handed over raw), a scalar, an `Option<data class>`, a
/// non-optional nested data class (inlined), and an `Option<handle>`.
fn value_form_items() -> Vec<(syn::Item, crate::SourceLocation)> {
    let loc = myflat_loc();
    vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZStamp {
                    pub secs: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOrigin {
                    pub node: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZSampleStruct {
                    pub key_expr: ZKeyExpr,
                    pub payload: ZBytes,
                    pub express: bool,
                    pub stamp: Option<ZStamp>,
                    pub origin: ZOrigin,
                    pub attachment: Option<ZBytes>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_sample_to_struct(s: &ZSample) -> ZSampleStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]
}

/// Build the fixture through `JniGen`, letting the caller adjust the
/// `ZSample` boundary decl. Returns the generated Rust + the joined Kotlin.
fn value_form_gen(tag: &str, decl: crate::lang::ExpandReturnDecl) -> (String, String) {
    let registry = Registry::<KotlinMeta>::from_items(value_form_items()).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZSample))
                .class(crate::ptr_class!(ZKeyExpr))
                .class(crate::ptr_class!(ZBytes))
                .class(crate::data_class!(ZStamp))
                .class(crate::data_class!(ZOrigin))
                .fun(crate::fun!(z_sample_sub)),
        )
        // A KeyExpr crosses as its string, never as a handle — the rule a
        // `.fields()` expansion has to keep honouring for the `key_expr` field.
        .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
        .expand(decl);

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    (rust, kotlin)
}

/// The headline: `.fields(fields!(f))` produces the leaves the fields' own
/// boundaries dictate, NOT one leaf per field.
///
/// Read the expected signature field by field — each is a different rule:
/// `key_expr` splices `ZKeyExpr`'s own decl (a String, not a handle);
/// `express` is a scalar; `stamp` behind `Option` stays one data-class leaf;
/// `origin` is a non-optional data class and INLINES (`origin__node`);
/// `payload` / `attachment` have no decl of their own, so they stay handles.
#[test]
fn fields_expand_by_each_field_s_own_boundary() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_basic",
        crate::expand_return!(ZSample).fields(crate::fields!(z_sample_to_struct)),
    );
    assert!(
        kotlin.contains("keyExpr__zKeyexprAsStr: String"),
        "a field whose type has its own expand_return! is decomposed by it, \
         not handed over as a handle:\n{kotlin}"
    );
    assert!(
        kotlin.contains("express: Boolean"),
        "a scalar field is one leaf:\n{kotlin}"
    );
    assert!(
        kotlin.contains("stamp: ZStamp?"),
        "an Option<data class> field stays ONE leaf (its converter builds it):\n{kotlin}"
    );
    assert!(
        kotlin.contains("origin__node: Long"),
        "a non-optional nested data class INLINES into its own fields:\n{kotlin}"
    );
    assert!(
        kotlin.contains("payload: ZBytes") && kotlin.contains("attachment: ZBytes?"),
        "a handle field with no boundary decl stays a handle, nullable under Option:\n{kotlin}"
    );
}

/// The value form is called ONCE per delivery. Without the hoist each field
/// would rebuild the whole struct — cloning every field once per leaf — which
/// is exactly the per-message cost `expand_return` exists to avoid.
#[test]
fn the_value_form_accessor_is_called_once() {
    let (rust, _) = value_form_gen(
        "jnigen_vf_hoist",
        crate::expand_return!(ZSample).fields(crate::fields!(z_sample_to_struct)),
    );
    let calls = rust.matches("z_sample_to_struct").count();
    assert_eq!(
        calls, 1,
        "the value form is bound to one local and every leaf reaches off it; \
         found {calls} calls in:\n{rust}"
    );
}

/// An `Option` field with nothing decomposed below it crosses **whole** — its
/// own converter takes the `Option`. Unwrapping it to reach the inner value
/// would hand `ZStamp` to a converter typed `Option<ZStamp>`: a mismatch the
/// Kotlin signature cannot show, because it reads `ZStamp?` either way.
#[test]
fn an_optional_field_reaches_its_converter_whole() {
    let (rust, _) = value_form_gen(
        "jnigen_vf_opt",
        crate::expand_return!(ZSample).fields(crate::fields!(z_sample_to_struct)),
    );
    for field in ["stamp", "attachment"] {
        assert!(
            rust.contains(&format!("__vf.{field}.clone()")),
            "`{field}` must be cloned whole, not matched open:\n{rust}"
        );
        assert!(
            !rust.contains(&format!("match &(&__vf).{field}")),
            "`{field}` has nothing decomposed below it, so it is not a nesting \
             step:\n{rust}"
        );
    }
}

/// A hand-written field list and the derived one are the SAME leaves — the
/// property that makes adopting `.fields()` a no-op on the wire, and the whole
/// reason it can be trusted to replace a list that has drifted.
#[test]
fn deriving_matches_the_equivalent_hand_written_list() {
    let items = value_form_items();
    let accessors: Vec<(syn::Item, crate::SourceLocation)> = {
        let loc = myflat_loc();
        vec![
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_sample_express(s: &ZSample) -> bool {
                        unimplemented!()
                    }
                )),
                loc,
            ),
        ]
    };

    let leaves_of = |decl: crate::lang::ExpandReturnDecl,
                     extra: Vec<(syn::Item, crate::SourceLocation)>|
     -> Vec<(String, String)> {
        let mut all = items.clone();
        all.extend(extra);
        let registry = Registry::<KotlinMeta>::from_items(all).expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .class(crate::ptr_class!(ZBytes))
                    .class(crate::data_class!(ZStamp))
                    .class(crate::data_class!(ZOrigin))
                    .fun(crate::fun!(z_sample_sub)),
            )
            .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
            .expand(decl);
        let gen = registry.resolve(jni).expect("resolve");
        gen.registry()
            .callback_arg_plans
            .values()
            .flat_map(|p| p.leaves.iter())
            .map(|l| (l.name.clone(), l.out_ty.to_token_stream().to_string()))
            .collect()
    };

    // The two fields the hand-written list can state with real accessors.
    let derived = leaves_of(
        crate::expand_return!(ZSample).fields(
            crate::fields!(z_sample_to_struct)
                .name("key_expr", "keyExpr")
                .name("express", "express"),
        ),
        vec![],
    );
    let by_hand = leaves_of(
        crate::expand_return!(ZSample)
            .field(crate::fun!(z_sample_key_expr).name("keyExpr"))
            .field(crate::fun!(z_sample_express).name("express")),
        accessors,
    );

    let take = |v: &[(String, String)], n: &str| -> Option<(String, String)> {
        v.iter().find(|(name, _)| name.starts_with(n)).cloned()
    };
    for prefix in ["keyExpr", "express"] {
        assert_eq!(
            take(&derived, prefix).map(|(n, _)| n),
            take(&by_hand, prefix).map(|(n, _)| n),
            "derived and hand-written leaves must agree on `{prefix}`\n\
             derived: {derived:?}\nby hand: {by_hand:?}"
        );
    }
}

/// A per-field override replaces that field's type default wholesale — here
/// keeping the raw `ZKeyExpr` handle instead of its declared string form.
#[test]
fn a_per_field_override_replaces_the_type_default() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_override",
        crate::expand_return!(ZSample).fields(
            crate::fields!(z_sample_to_struct)
                .field("key_expr", crate::expand_return!(ZKeyExpr).field_self()),
        ),
    );
    assert!(
        kotlin.contains("keyExpr: ZKeyExpr"),
        "the override wins over ZKeyExpr's type-level decl:\n{kotlin}"
    );
    assert!(
        !kotlin.contains("keyExpr__zKeyexprAsStr"),
        "the overridden field must NOT also carry the type default:\n{kotlin}"
    );
}

/// A rename keys on the Rust field ident and reaches an inlined nested field
/// through its dotted path.
#[test]
fn a_field_can_be_renamed_including_a_nested_one() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_rename",
        crate::expand_return!(ZSample).fields(
            crate::fields!(z_sample_to_struct)
                .name("express", "fast")
                .name("origin.node", "nodeId"),
        ),
    );
    assert!(
        kotlin.contains("fast: Boolean"),
        "a renamed field uses the literal name:\n{kotlin}"
    );
    assert!(
        kotlin.contains("origin__nodeId: Long"),
        "a nested field is renamed through its dotted path, keeping the prefix:\n{kotlin}"
    );
}

/// `.fields()` mixes with the other declarators — the value form's fields
/// *and* the live handle, which is the `Query`-style shape.
#[test]
fn fields_mixes_with_field_self() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_mixed",
        crate::expand_return!(ZSample)
            .fields(crate::fields!(z_sample_to_struct))
            .field_self(),
    );
    assert!(
        kotlin.contains("express: Boolean") && kotlin.contains("handle: ZSample"),
        "the derived fields and the identity leaf are delivered together:\n{kotlin}"
    );
}

/// A **sum** field decomposes into its selector plus one group per
/// alternative, right there among its sibling fields — a sum has no
/// whole-value converter, so this is the only way it can cross at all. The
/// `ReplyStruct { result: ReplyResult, .. }` shape.
fn sum_field_gen(tag: &str) -> (String, String) {
    let loc = myflat_loc();
    let mut items = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum ZOutcome {
                    Empty,
                    Ok(ZBytes),
                    Failed(String),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZReplyStruct {
                    pub result: ZOutcome,
                    pub seq: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_reply_to_struct(r: &ZReply) -> ZReplyStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_reply_sub(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc,
    ));
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZReply))
                .class(crate::ptr_class!(ZBytes))
                .class(crate::sealed_class!(ZOutcome))
                .fun(crate::fun!(z_reply_sub)),
        )
        .expand(crate::expand_return!(ZReply).fields(crate::fields!(z_reply_to_struct)));

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    (rust, kotlin)
}

#[test]
fn a_sum_field_crosses_as_its_selector_and_groups() {
    let (rust, kotlin) = sum_field_gen("jnigen_vf_sum");
    assert!(
        kotlin.contains("result__tag: Int"),
        "the sum field contributes its selector, prefixed by the field:\n{kotlin}"
    );
    assert!(
        kotlin.contains("result__ok_v0: Long") && kotlin.contains("result__failed_v0: String?"),
        "one group slot per alternative payload, object slots nullable \
         (an inert group arrives as null):\n{kotlin}"
    );
    assert!(
        kotlin.contains("seq: Long"),
        "a sibling field is unaffected:\n{kotlin}"
    );
    assert!(
        kotlin.contains("ZOutcome.Ok(") && kotlin.contains("ZOutcome.Failed("),
        "the receiver rebuilds the live alternative from the tag:\n{kotlin}"
    );
    assert!(
        rust.contains("myflat::ZOutcome::Ok") && rust.contains("myflat::ZOutcome::Failed"),
        "Rust matches the sum once, filling every group's slots:\n{rust}"
    );
}

/// A sum's slots sit at a FIXED position in the leaf list, so the two shapes
/// that would move or repeat them are refused by name rather than mis-emitted:
/// `Vec<sum>` has variable arity, and `Option<sum>` would need a present flag
/// beside the tag that an output leaf list cannot carry.
#[test]
fn a_sum_field_behind_option_or_vec_is_rejected_by_name() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| {
        let items = vec![
            (
                syn::Item::Enum(syn::parse_quote!(
                    pub enum ZOutcome {
                        Empty,
                        Failed(String),
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZReplyStruct {
                        pub result: #field_ty,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_reply_to_struct(r: &ZReply) -> ZReplyStruct {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_reply_sub(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZReply))
                    .class(crate::sealed_class!(ZOutcome))
                    .fun(crate::fun!(z_reply_sub)),
            )
            .expand(crate::expand_return!(ZReply).fields(crate::fields!(z_reply_to_struct)));
        let dir = unique_test_dir("jnigen_vf_sum_reject");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = registry
            .resolve(jni)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };

    for (ty, want) in [
        (syn::parse_quote!(Vec<ZOutcome>), "variable arity"),
        (syn::parse_quote!(Option<ZOutcome>), "present flag"),
    ] {
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(ty)))
            .expect_err("a sum behind Option/Vec must be rejected");
        let msg = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains(want), "expected `{want}` in: {msg}");
        assert!(
            msg.contains("ZReplyStruct.result"),
            "the message names the offending field: {msg}"
        );
    }
}

/// Naming a field the value form does not have is the very drift this
/// declarator exists to catch, so it is an error rather than a silent no-op.
#[test]
fn an_adjustment_naming_an_unknown_field_is_an_error() {
    let build = |decl: crate::lang::FieldsDecl| {
        let registry = Registry::<KotlinMeta>::from_items(value_form_items()).expect("index");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .class(crate::ptr_class!(ZBytes))
                    .class(crate::data_class!(ZStamp))
                    .class(crate::data_class!(ZOrigin))
                    .fun(crate::fun!(z_sample_sub)),
            )
            .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
            .expand(crate::expand_return!(ZSample).fields(decl));
        let dir = unique_test_dir("jnigen_vf_unknown");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = registry
            .resolve(jni)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };

    for decl in [
        crate::fields!(z_sample_to_struct).name("kex", "kex"),
        crate::fields!(z_sample_to_struct)
            .field("kex", crate::expand_return!(ZKeyExpr).field_self()),
    ] {
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(decl)))
            .expect_err("an unknown field name must be rejected");
        let msg = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains("kex"), "the message names the field: {msg}");
        assert!(
            msg.contains("ZSampleStruct"),
            "the message names the value form: {msg}"
        );
    }
}

/// One value form states the whole field set: a second `.fields()` would make
/// the leaf order depend on declaration order for no gain.
#[test]
#[should_panic(expected = "already expands a value form")]
fn a_second_value_form_is_an_error() {
    let _ = crate::expand_return!(ZSample)
        .fields(crate::fields!(z_sample_to_struct))
        .fields(crate::fields!(z_sample_to_struct));
}

/// Repeating an adjustment for one field is a declaration bug — the complete
/// set rule, same as `.expand_param` / `.field`.
#[test]
#[should_panic(expected = "already has an override")]
fn a_repeated_override_is_an_error() {
    let _ = crate::fields!(z_sample_to_struct)
        .field("key_expr", crate::expand_return!(ZKeyExpr).field_self())
        .field("key_expr", crate::expand_return!(ZKeyExpr).field_self());
}

/// `"__"` is the reserved chain separator, so an author-supplied rename may
/// not smuggle one in and forge a nesting that isn't there.
#[test]
#[should_panic(expected = "reserved")]
fn a_rename_may_not_contain_the_chain_separator() {
    let _ = crate::fields!(z_sample_to_struct).name("express", "a__b");
}
