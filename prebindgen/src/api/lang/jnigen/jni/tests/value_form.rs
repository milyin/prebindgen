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
            rust.contains(&format!(".{field}.clone()")),
            "`{field}` must be cloned whole, not matched open:\n{rust}"
        );
        assert!(
            !rust.contains(&format!(".{field} {{")),
            "`{field}` has nothing decomposed below it, so it is not a nesting \
             step — no `match` on it:\n{rust}"
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

// ── Review findings on #221 ──────────────────────────────────────────────────

/// A **single-leaf** value form takes the `Delivery::Return` shortcut, whose
/// reach is composed separately from the multi-leaf encoder
/// (`emit/wrapper.rs`'s `is_convert` path). That path must still give a plain
/// field leaf the owned value its converter is typed for: composing the field
/// as a borrow feeds `&i64` to the `i64` converter, and for a non-`Copy` field
/// borrows out of the temporary the value-form call returned.
#[test]
fn a_single_leaf_value_form_delivers_an_owned_field() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOneStruct {
                    pub label: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_one_to_struct(o: &ZOne) -> ZOneStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_one_make(n: i64) -> ZOne {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .fun(crate::fun!(z_one_make)),
        )
        .expand(crate::expand_return!(ZOne).fields(crate::fields!(z_one_to_struct)));
    let dir = unique_test_dir("jnigen_vf_single");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains(".label).clone()"),
        "the single leaf is CLONED out of the value form, matching the owned \
         `String` its converter takes — composing it as a borrow would feed \
         `&String` to a `String` converter:\n{rust}"
    );
}

/// A per-field `.field(name, expand_return!(T))` override states the field's
/// type, so it has to be checked against the field. Otherwise the override
/// silently survives an upstream field-type change — which is exactly the drift
/// this declarator exists to catch — and two same-shaped handle types are
/// interchangeable by accident.
#[test]
fn a_per_field_override_must_name_the_field_s_own_type() {
    let build = || {
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
            .expand(
                crate::expand_return!(ZSample).fields(
                    // `key_expr` is a `ZKeyExpr`, not a `ZBytes`.
                    crate::fields!(z_sample_to_struct)
                        .field("key_expr", crate::expand_return!(ZBytes).field_self()),
                ),
            );
        let dir = unique_test_dir("jnigen_vf_ovr_ty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = registry
            .resolve(jni)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(build))
        .expect_err("a mistyped override must be rejected");
    let msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        msg.contains("key_expr"),
        "the message names the field: {msg}"
    );
    assert!(
        msg.contains("ZBytes") && msg.contains("ZKeyExpr"),
        "the message names the declared type and the real one: {msg}"
    );
}

/// The "called once per delivery" contract has to survive **composition**: when
/// a value form's field splices a child type whose own boundary is also derived
/// from a value form, the child accessor is a second hoist, not a call repeated
/// once per child leaf.
#[test]
fn a_nested_value_form_is_hoisted_too() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZInnerStruct {
                    pub a: i64,
                    pub b: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOuterStruct {
                    pub inner: ZInner,
                    pub tag: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_inner_to_struct(i: &ZInner) -> ZInnerStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_outer_to_struct(o: &ZOuter) -> ZOuterStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_outer_sub(cb: impl Fn(ZOuter) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOuter))
                .class(crate::ptr_class!(ZInner))
                .fun(crate::fun!(z_outer_sub)),
        )
        .expand(crate::expand_return!(ZInner).fields(crate::fields!(z_inner_to_struct)))
        .expand(crate::expand_return!(ZOuter).fields(crate::fields!(z_outer_to_struct)));
    let dir = unique_test_dir("jnigen_vf_nested");
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

    assert!(
        kotlin.contains("inner__a: Long") && kotlin.contains("inner__b: Long"),
        "the child value form's fields splice in, prefixed:\n{kotlin}"
    );
    for f in ["z_outer_to_struct", "z_inner_to_struct"] {
        let calls = rust.matches(f).count();
        assert_eq!(
            calls, 1,
            "`{f}` is bound to one local and every leaf below it reaches off \
             that local; found {calls} calls in:\n{rust}"
        );
    }
}

/// A consuming value form reached **through another one** is handed the
/// parent's field by MOVE. A hoisted value form is an owned struct and its
/// fields are disjoint, so giving one field away leaves every sibling leaf
/// readable — which is why this shape needs no clone and is not refused.
///
/// Checked under both parents: what makes the move legal is that the hoist
/// local is owned, and a borrowing form's returned struct is owned just as much
/// as a consuming one's.
#[test]
fn a_nested_consuming_value_form_moves_the_parent_s_field() {
    let loc = myflat_loc();
    let items = |outer_by_value: bool| -> Vec<(syn::Item, crate::SourceLocation)> {
        let outer: syn::Item = if outer_by_value {
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_outer_into_struct(o: ZOuter) -> ZOuterStruct {
                    unimplemented!()
                }
            ))
        } else {
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_outer_to_struct(o: &ZOuter) -> ZOuterStruct {
                    unimplemented!()
                }
            ))
        };
        vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZInnerStruct {
                        pub a: i64,
                        pub b: i64,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZOuterStruct {
                        pub inner: ZInner,
                        pub tag: i64,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_inner_into_struct(i: ZInner) -> ZInnerStruct {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
            (outer, loc.clone()),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_outer_sub(cb: impl Fn(ZOuter) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ]
    };

    for (tag, outer_by_value, outer) in [
        (
            "borrow",
            false,
            crate::expand_return!(ZOuter).fields(crate::fields!(z_outer_to_struct)),
        ),
        (
            "consume",
            true,
            crate::expand_return!(ZOuter).fields_into(crate::fields!(z_outer_into_struct)),
        ),
    ] {
        let registry =
            Registry::<KotlinMeta>::from_items(items(outer_by_value)).expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZOuter))
                    .class(crate::ptr_class!(ZInner))
                    .fun(crate::fun!(z_outer_sub)),
            )
            .expand(crate::expand_return!(ZInner).fields_into(crate::fields!(z_inner_into_struct)))
            .expand(outer);
        let dir = unique_test_dir(&format!("jnigen_vf_nested_consume_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = registry.resolve(jni).expect("resolve");
        let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
            .expect("read rust");

        assert!(
            rust.contains("z_inner_into_struct(__vf0.inner)"),
            "[{tag}] the parent's field is MOVED into the nested form, not borrowed \
             or cloned:\n{rust}"
        );
        assert!(
            rust.contains("__vf0.tag") && !rust.contains("(&__vf0)"),
            "[{tag}] and a sibling leaf still reads its own field off the parent local, \
             projected directly — borrowing the partially-moved local as a whole would \
             not compile:\n{rust}"
        );
        assert!(
            !rust.contains("__vf1.a.clone()"),
            "[{tag}] the nested form's own fields move out too:\n{rust}"
        );
    }
}

// A compact fixture shared by the two follow-up review regressions.
fn nested_review_items() -> Vec<(syn::Item, crate::SourceLocation)> {
    let loc = myflat_loc();
    vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZReviewInnerStruct {
                    pub value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZReviewOuterStruct {
                    pub optional: Option<ZReviewInner>,
                    pub items: Vec<ZReviewInner>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_review_inner_to_struct(i: &ZReviewInner) -> ZReviewInnerStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_review_outer_to_struct(o: &ZReviewOuter) -> ZReviewOuterStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_review_outer_sub(cb: impl Fn(ZReviewOuter) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]
}

fn nested_review_jni(outer: crate::lang::ExpandReturnDecl) -> JniGen {
    JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZReviewOuter))
                .class(crate::ptr_class!(ZReviewInner))
                .fun(crate::fun!(z_review_outer_sub)),
        )
        .expand(
            crate::expand_return!(ZReviewInner).fields(crate::fields!(z_review_inner_to_struct)),
        )
        .expand(outer)
}

/// A nested value form below an `Option` cannot be emitted as an
/// unconditional hoist: its accessor takes `&Inner`, not `&Option<Inner>`.
/// Reject it during planning until conditional hoists can share one `Some`
/// scope across every descendant leaf.
#[test]
fn an_optional_nested_value_form_is_rejected_before_emission() {
    let registry = Registry::<KotlinMeta>::from_items(nested_review_items()).expect("index items");
    let jni = nested_review_jni(
        crate::expand_return!(ZReviewOuter).fields(crate::fields!(z_review_outer_to_struct)),
    );
    let err = match registry.resolve(jni) {
        Ok(_) => panic!("an optional nested value form must be rejected"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("z_review_inner_to_struct") && msg.contains("Option"),
        "the error names the unsupported conditional hoist: {msg}"
    );
}

/// Override records are applied to a `Vec<T>` field as a whole; a fixed leaf
/// list cannot apply `T`'s deconstructor once per element. The declaration
/// check must compare against `Vec<T>`, not peel it to `T`.
#[test]
fn a_vec_field_override_must_name_the_whole_vec_type() {
    let build = || {
        let registry =
            Registry::<KotlinMeta>::from_items(nested_review_items()).expect("index items");
        let jni = nested_review_jni(
            crate::expand_return!(ZReviewOuter).fields(
                crate::fields!(z_review_outer_to_struct)
                    .field("items", crate::expand_return!(ZReviewInner).field_self()),
            ),
        );
        let _ = registry.resolve(jni);
    };

    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(build))
        .expect_err("an element-typed override on a Vec field must be rejected");
    let msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        msg.contains("items") && msg.contains("Vec") && msg.contains("ZReviewInner"),
        "the error names the field, its whole Vec type, and the declared element type: {msg}"
    );
}

// ── Consuming value forms ────────────────────────────────────────────────────

/// A value form whose accessor takes its receiver **by value** destroys the
/// object into its parts, so nothing needs cloning. Fixture mirrors the
/// borrowing one; `zc_owned` / `zc_borrowed` give an owned and a `&T` plan of
/// the same type, since one declaration serves both.
fn consuming_items() -> Vec<(syn::Item, crate::SourceLocation)> {
    let loc = myflat_loc();
    vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZCarrierStruct {
                    pub label: String,
                    pub count: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zc_into_struct(c: ZCarrier) -> ZCarrierStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zc_to_struct(c: &ZCarrier) -> ZCarrierStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zc_sub(cb: impl Fn(ZCarrier) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]
}

fn consuming_gen(tag: &str, decl: crate::lang::ExpandReturnDecl) -> String {
    let registry = Registry::<KotlinMeta>::from_items(consuming_items()).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .fun(crate::fun!(zc_sub)),
        )
        .expand(decl);
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust")
}

/// The headline: a consuming value form is handed the value itself, and each
/// field is **moved** into its leaf. The clones the borrowing form pays — one
/// per field, on a value it is about to drop — simply are not emitted.
#[test]
fn a_consuming_value_form_moves_its_fields() {
    let rust = consuming_gen(
        "jnigen_vf_consume",
        crate::expand_return!(ZCarrier).fields_into(crate::fields!(zc_into_struct)),
    );
    assert!(
        rust.contains("zc_into_struct(__cb_arg0)"),
        "the value is passed BY MOVE, not borrowed:\n{rust}"
    );
    assert!(
        rust.contains("__vf0.label") && rust.contains("__vf0.count"),
        "each field is read off the one hoisted local:\n{rust}"
    );
    assert!(
        !rust.contains("__vf0.label.clone()") && !rust.contains("__vf0.count.clone()"),
        "and MOVED out of it — a consuming form exists precisely to drop these \
         clones:\n{rust}"
    );
}

/// The borrowing form is untouched: same declaration shape, still borrows, still
/// clones. Consuming-ness is inferred per accessor, so one does not disturb the
/// other.
#[test]
fn the_borrowing_value_form_still_clones() {
    let rust = consuming_gen(
        "jnigen_vf_borrow",
        crate::expand_return!(ZCarrier).fields(crate::fields!(zc_to_struct)),
    );
    assert!(
        rust.contains("zc_to_struct(&__cb_arg0)"),
        "a `&T` accessor is still handed a borrow:\n{rust}"
    );
    assert!(
        rust.contains("__vf0.label.clone()"),
        "and its fields are still cloned out:\n{rust}"
    );
}

/// One declaration is reached by BOTH owned and borrowed plans of the same type
/// (records are type-level, `by_ref` is per-function). A borrowed plan has no
/// value to give up, so it clones once up front rather than being rejected —
/// the same cost the borrowing form of the accessor would have paid.
#[test]
fn a_borrowed_plan_clones_before_consuming() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn zc_borrowed(v: &ZVault) -> Option<&ZCarrier> {
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
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZVault))
                .fun(crate::fun!(zc_sub))
                .fun(crate::fun!(zc_borrowed)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_into(crate::fields!(zc_into_struct)));
    let dir = unique_test_dir("jnigen_vf_consume_ref");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");
    assert!(
        rust.contains("zc_into_struct(__inner.clone())"),
        "a borrowed plan clones the value, then consumes the clone:\n{rust}"
    );
}

/// `.fields_into(..)` gives the value away, so anything else reading it is
/// broken by construction. Refused **where it is declared** — the collision is
/// visible in the decl itself, so it does not need a resolve to be found.
///
/// `.field_self()` beside it would deliver the handle the form just consumed.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_following_sibling() {
    let _ = crate::expand_return!(ZCarrier)
        .fields_into(crate::fields!(zc_into_struct))
        .field_self();
}

/// And the other way round — the decl is a builder, so both orders must be
/// caught or the rule holds only for the order someone happened to write.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_preceding_sibling() {
    let _ = crate::expand_return!(ZCarrier)
        .field_self()
        .fields_into(crate::fields!(zc_into_struct));
}

/// Any sibling record, not just the identity one.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_plain_field_sibling() {
    let _ = crate::expand_return!(ZCarrier)
        .fields_into(crate::fields!(zc_into_struct))
        .field(crate::fun!(zc_to_struct));
}

/// The declarator states whether the value is given away and the accessor's
/// signature has to agree — otherwise the emitted call would not compile in the
/// consumer's crate, and a boundary would silently stop being the one declared.
/// Both directions are errors; the fixture has one accessor of each kind.
#[test]
fn the_declarator_and_the_accessor_s_receiver_must_agree() {
    let build = |decl: crate::lang::ExpandReturnDecl| -> String {
        let registry = Registry::<KotlinMeta>::from_items(consuming_items()).expect("index");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZCarrier))
                    .fun(crate::fun!(zc_sub)),
            )
            .expand(decl);
        match registry.resolve(jni) {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        }
    };

    let msg = build(crate::expand_return!(ZCarrier).fields_into(crate::fields!(zc_to_struct)));
    assert!(
        msg.contains("CONSUMING") && msg.contains("zc_to_struct"),
        "`.fields_into` on a borrowing accessor must be refused, naming it: {msg:?}"
    );

    let msg = build(crate::expand_return!(ZCarrier).fields(crate::fields!(zc_into_struct)));
    assert!(
        msg.contains("BORROWING") && msg.contains("zc_into_struct"),
        "`.fields` on a by-value accessor must be refused, naming it: {msg:?}"
    );
}
