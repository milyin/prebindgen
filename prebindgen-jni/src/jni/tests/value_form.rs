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
fn value_form_items() -> Vec<(syn::Item, prebindgen::SourceLocation)> {
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

/// Build the fixture through `JniGenBuilder`, letting the caller adjust the
/// `ZSample` boundary decl. Returns the generated Rust + the joined Kotlin.
fn value_form_gen(tag: &str, decl: crate::ExpandReturnDecl) -> (String, String) {
    let registry = crate::test_util::reg_from_items(declare_referenced(value_form_items()))
        .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZSample))
                .class(crate::ptr_class!(ZKeyExpr))
                .class(crate::ptr_class!(ZBytes))
                .class(crate::data_class!(ZStamp))
                .class(crate::data_class!(ZOrigin))
                .fun(prebindgen_registry::fun!(z_sample_sub)),
        )
        // A KeyExpr crosses as its string, never as a handle — the rule a
        // `.fields()` expansion has to keep honouring for the `key_expr` field.
        .expand(
            prebindgen_registry::expand_return!(ZKeyExpr)
                .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
        )
        .expand(decl);

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
        prebindgen_registry::expand_return!(ZSample)
            .fields(prebindgen_registry::fields!(z_sample_to_struct)),
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
        prebindgen_registry::expand_return!(ZSample)
            .fields(prebindgen_registry::fields!(z_sample_to_struct)),
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
        prebindgen_registry::expand_return!(ZSample)
            .fields(prebindgen_registry::fields!(z_sample_to_struct)),
    );
    for field in ["stamp", "attachment"] {
        assert!(
            rust.contains(&format!(".{field}).clone()")),
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
///
/// BLOCKED by the prebindgen-jni crate split: reads `Registry::callback_arg_plans`,
/// a `pub(crate)` field of `prebindgen_registry::Registry` — reachable when this
/// test lived inside the `prebindgen` crate, not from the separate
/// `prebindgen-jni` crate it moved to. Left in place, not deleted, pending a
/// `prebindgen` accessor for this field (see the carve-prebindgen-jni report).
#[test]
fn deriving_matches_the_equivalent_hand_written_list() {
    let items = value_form_items();
    let accessors: Vec<(syn::Item, prebindgen::SourceLocation)> = {
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

    let leaves_of = |decl: crate::ExpandReturnDecl,
                     extra: Vec<(syn::Item, prebindgen::SourceLocation)>|
     -> Vec<(String, String)> {
        let mut all = items.clone();
        all.extend(extra);
        let registry =
            crate::test_util::reg_from_items(declare_referenced(all)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .class(crate::ptr_class!(ZBytes))
                    .class(crate::data_class!(ZStamp))
                    .class(crate::data_class!(ZOrigin))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZKeyExpr)
                    .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
            )
            .expand(decl);
        let gen = jni.build_with(registry).expect("resolve");
        gen.declarations()
            .unfolded()
            .callback_arg_plans
            .values()
            .flat_map(|p| p.leaves.iter())
            .map(|l| (l.name.clone(), l.out_ty.to_string()))
            .collect()
    };

    // The two fields the hand-written list can state with real accessors.
    let derived = leaves_of(
        prebindgen_registry::expand_return!(ZSample).fields(
            prebindgen_registry::fields!(z_sample_to_struct)
                .name("key_expr", "keyExpr")
                .name("express", "express"),
        ),
        vec![],
    );
    let by_hand = leaves_of(
        prebindgen_registry::expand_return!(ZSample)
            .field(prebindgen_registry::fun!(z_sample_key_expr).name("keyExpr"))
            .field(prebindgen_registry::fun!(z_sample_express).name("express")),
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
        prebindgen_registry::expand_return!(ZSample).fields(
            prebindgen_registry::fields!(z_sample_to_struct).field(
                "key_expr",
                prebindgen_registry::expand_return!(ZKeyExpr).field_self(),
            ),
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

/// The complete-set rule taken to its limit: an override with NO records is an
/// empty leaf set, so the field does not cross at all. That is how a binding
/// drops a field its source's value form carries but its surface has no use
/// for — without it, adopting a value form is all-or-nothing.
#[test]
fn an_empty_per_field_override_drops_the_field() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_drop",
        prebindgen_registry::expand_return!(ZSample).fields(
            prebindgen_registry::fields!(z_sample_to_struct)
                .field("key_expr", prebindgen_registry::expand_return!(ZKeyExpr)),
        ),
    );
    assert!(
        !kotlin.contains("keyExpr"),
        "a field whose override states no leaves contributes none:\n{kotlin}"
    );
    assert!(
        kotlin.contains("express: Boolean"),
        "and its siblings are untouched:\n{kotlin}"
    );
}

/// A rename keys on the Rust field ident and reaches an inlined nested field
/// through its dotted path.
#[test]
fn a_field_can_be_renamed_including_a_nested_one() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_rename",
        prebindgen_registry::expand_return!(ZSample).fields(
            prebindgen_registry::fields!(z_sample_to_struct)
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
        prebindgen_registry::expand_return!(ZSample)
            .fields(prebindgen_registry::fields!(z_sample_to_struct))
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZReply))
                .class(crate::ptr_class!(ZBytes))
                .class(crate::sealed_class!(ZOutcome))
                .fun(prebindgen_registry::fun!(z_reply_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZReply)
                .fields(prebindgen_registry::fields!(z_reply_to_struct)),
        );

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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

/// A sum's slots sit at a FIXED position in the leaf list, so the shape that
/// would repeat them is refused by name rather than mis-emitted: `Vec<sum>` has
/// variable arity and no fixed layout to lay out.
///
/// `Option<sum>` used to be refused beside it; it no longer is (#220) — see
/// [`an_optional_sum_field_gates_its_whole_segment`].
#[test]
fn a_vec_sum_field_is_rejected_by_name() {
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
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZReply))
                    .class(crate::sealed_class!(ZOutcome))
                    .fun(prebindgen_registry::fun!(z_reply_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZReply)
                    .fields(prebindgen_registry::fields!(z_reply_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_sum_reject");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = jni
            .build_with(registry)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };

    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        build(syn::parse_quote!(Vec<ZOutcome>))
    }))
    .expect_err("a Vec of sums must be rejected");
    let msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        msg.contains("variable arity"),
        "expected the reason in: {msg}"
    );
    assert!(
        msg.contains("ZReplyStruct.result"),
        "the message names the offending field: {msg}"
    );
}

/// `Option<sum>` as a value-form field (#220): the whole segment gates together.
///
/// A sum's leaves are not independent — only one group is live per value — so
/// absence cannot be a per-leaf `null` the way it is for an ordinary optional
/// field. The segment binds as ONE tuple whose `None` arm carries every slot's
/// wire default, which is the shape a conditional value form's hoist already
/// emits, applied to an optional step inside the segment's own path.
///
/// The selector is the one slot that must NOT stay a raw `jint`: zero is a real
/// variant, so an absent sum needs a representation the tag's own domain does
/// not provide. It boxes, and JVM null means "no value here" — the same rule
/// that already holds for a sum under a conditional form, and the reason this
/// needs no present flag beside the tag.
#[test]
fn an_optional_sum_field_gates_its_whole_segment() {
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
                        pub seq: i64,
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
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZReply))
                    .class(crate::sealed_class!(ZOutcome))
                    .fun(prebindgen_registry::fun!(z_reply_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZReply)
                    .fields(prebindgen_registry::fields!(z_reply_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_opt_sum");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
        let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
            .expect("read rust");
        let kotlin = gen
            .write_kotlin(&dir.join("kotlin"))
            .expect("write_kotlin")
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        (rust, kotlin)
    };

    let (rust, kotlin) = build(syn::parse_quote!(Option<ZOutcome>));
    let rc: String = rust.split_whitespace().collect();

    // The segment is one tuple bind over the field's `Option`, reached through a
    // coercion site so the destructure does not care how the source spelled it.
    assert!(
        rc.contains(":&::core::option::Option<_>=&(&__vf0).result;"),
        "the optional step is a coercion site over the field:\n{rust}"
    );
    assert!(
        rc.contains("::core::option::Option::None=>{(jni::objects::JObject::null(),jni::objects::JObject::null(),)}"),
        "the absent arm yields the whole segment's defaults as one tuple:\n{rust}"
    );
    // The selector boxes: `0` is `ZOutcome::Empty`, so a raw `jint` has no
    // spelling left for "absent".
    assert!(
        rc.contains("jni::objects::JObject::null()"),
        "an absent segment defaults its slots, the tag's to JVM null:\n{rust}"
    );
    // The live groups still convert exactly as an ungated sum's do.
    assert!(
        rust.contains("ZOutcome::Failed") && rust.contains("ZOutcome::Empty"),
        "every alternative still gets its arm:\n{rust}"
    );
    // Kotlin reads absence off the selector, ahead of the real tags.
    assert!(
        kotlin.contains("null -> null"),
        "the reassembly answers `null` for an absent sum, before tag 0:\n{kotlin}"
    );

    // The plain sibling leaf is unaffected — gating is the segment's, not the
    // whole form's.
    assert!(
        rust.contains("seq"),
        "a non-sum field beside it still crosses normally:\n{rust}"
    );
}

/// The dual: a BARE sum field takes no gate at all, so the optional path is not
/// entered when there is nothing to gate — the selector stays a raw `jint` and
/// the segment stays a plain `match` with no tuple bind.
#[test]
fn a_bare_sum_field_takes_no_gate() {
    let loc = myflat_loc();
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
                    pub result: ZOutcome,
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZReply))
                .class(crate::sealed_class!(ZOutcome))
                .fun(prebindgen_registry::fun!(z_reply_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZReply)
                .fields(prebindgen_registry::fields!(z_reply_to_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_bare_sum");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
        .expect("read rust");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // Whitespace-stripped, as the sibling test asserts: `prettyplease` breaks a
    // tuple-valued arm after the `{`, so `None => (` on one line never appears
    // in EITHER emission — the assertion would hold whether or not the gate was
    // emitted, and pin nothing.
    assert!(
        !rust
            .split_whitespace()
            .collect::<String>()
            .contains("::core::option::Option::None=>{("),
        "nothing to gate, so no tuple bind:\n{rust}"
    );
    assert!(
        !kotlin.contains("null -> null"),
        "the selector carries no absent case:\n{kotlin}"
    );
    assert!(
        rust.contains("ZOutcome::Failed"),
        "the segment still emits its arms:\n{rust}"
    );
}

/// `Vec<data class>` crosses in BOTH positions — as a return and as a value-form
/// **field** — with its elements folded from raw leaves either way (#217).
///
/// #217 reported the field position as a hard panic ("variable arity"), on the
/// reasoning that the `fromParts` bridge is fixed-layout: the encode, the
/// factory and `build_data_class` occupy identical slots in identical order,
/// and a runtime length breaks that. That was true,
/// and the resolution was not the array codegen the issue anticipated — it is
/// that the field never needs to enter the fixed layout at all. It stays **one**
/// slot whose own converter is the element's leaf-vec fold, so the slot count is
/// still fixed and the elements still cross as raw leaves.
///
/// So the guard the issue names is now unreachable: it sits inside the
/// decomposition's `TypeKind::DataStruct` branch, and `type_kind` answers
/// `DataStruct` only for a key that is a single identifier — which a `Vec<_>`
/// key never is. The `Vec<sum>` guard beside it is a different question and
/// stays live (see
/// [`a_sum_field_behind_option_or_vec_is_rejected_by_name`]).
///
/// **The raw-leaf fold is asserted separately, and that is the point.** #217's
/// standing warning is that silently degrading the field to a whole-object
/// crossing would reintroduce the per-value JVM object the bridge exists to
/// avoid. Such a degradation would still surface as `List<Rec>` and still build
/// through `fromParts` — it would pass every other assertion here. Only the
/// folder assertion would catch it.
#[test]
fn a_vec_of_data_classes_crosses_as_a_return_and_as_a_field() {
    let loc = myflat_loc();
    // `field` = the value form's field type. `None` builds only the control
    // (the `Vec<Rec>` return), so the two claims do not share a failure mode.
    let build = |field: Option<syn::Type>| {
        let mut items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct Rec {
                        pub id: i64,
                    }
                )),
                loc.clone(),
            ),
            // The CONTROL: the same `Vec<Rec>`, in the position that works.
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn stack_records(n: i64) -> Vec<Rec> {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let has_field = field.is_some();
        if let Some(field) = field {
            items.push((
                syn::Item::Struct(syn::parse_quote!(
                    pub struct StackStruct {
                        pub records: #field,
                    }
                )),
                loc.clone(),
            ));
            // Returning the struct by value is what BUILDS the flattened
            // bridge (`wire_fixed_returns` → `build_struct_plan`). Declaring
            // the data class alone only renders it, with a whole-object
            // `fromParts`, and never asks `classify_field` anything.
            items.push((
                syn::Item::Fn(syn::parse_quote!(
                    pub fn stack_struct_of(n: i64) -> StackStruct {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ));
        }
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let mut jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::data_class!(Rec))
                    .fun(prebindgen_registry::fun!(stack_records)),
            );
        if has_field {
            jni = jni.package(
                crate::package!()
                    .class(crate::data_class!(StackStruct))
                    .fun(prebindgen_registry::fun!(stack_struct_of)),
            );
        }
        let dir = unique_test_dir("jnigen_vec_dataclass_field");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
        let kdir = dir.join("kotlin");
        let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
        gen.write_rust(dir.join("g.rs")).expect("write_rust");
        paths
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    };

    // 1. The control. Without this the refusal below says nothing about
    //    POSITION — it would be satisfied by `Vec<Rec>` failing everywhere.
    let kotlin = build(None);
    let kc: String = kotlin.split_whitespace().collect();
    assert!(
        kc.contains("stackRecords") && kc.contains("List<Rec>"),
        "`Vec<Rec>` must cross as `List<Rec>` in the RETURN position — that is \
         the half of the asymmetry that works:\n{kotlin}"
    );

    // 2. The same element type as a FIELD of a value form. This is the
    //    position #217 reported as a hard panic; it crosses.
    let kotlin = build(Some(syn::parse_quote!(Vec<Rec>)));
    let kc: String = kotlin.split_whitespace().collect();
    assert!(
        kc.contains("publicdataclassStackStruct(valrecords:List<Rec>)"),
        "the `Vec<Rec>` FIELD must surface as `List<Rec>`:\n{kotlin}"
    );
    assert!(
        kc.contains("StackStructBuilder{records->StackStruct.fromParts(records)}"),
        "the field is delivered as one builder slot:\n{kotlin}"
    );
    // The property the guard existed to protect: elements are rebuilt Kotlin-side
    // from RAW leaves, so no per-element JVM object crosses the boundary. A
    // whole-object degradation would satisfy the two assertions above and fail
    // this one, which is why it is asserted separately.
    assert!(
        kc.contains("RecFolderRaw{acc,id->acc.add(Rec.fromParts(id));acc}"),
        "each element must be folded from its raw leaves, not crossed as an \
         object — that is what the fixed bridge exists to avoid:\n{kotlin}"
    );
}

/// A field's optional-ness is read off `kind`; **how Rust spells it is the
/// source's business**, and the emitter must accept any of the spellings.
///
/// `Box<T>` *is* `T` in the model, deliberately: the flat model states the
/// destination-language invariant, and no target language can tell an
/// `Option<T>` field from a `Box<Option<T>>` one. The emitter used to classify
/// off `kind` correctly ("optional ⇒ needs a `Some`/`None` split") and then
/// *spell* off `kind` too, matching `Option`'s patterns against a place still
/// typed `Box<Option<Child>>` — `E0308` (#268).
///
/// So the destructuring goes through a coercion site. Deref coercion is
/// transitive and a no-op when the types already match, which is why one shape
/// serves every representation and the plain spelling is unchanged.
///
/// Both spellings are asserted to reach the SAME leaf surface: if the wrapper
/// changed what crosses, the model would be leaking a Rust detail it exists to
/// hide. (What this cannot yet assert is that the result compiles — nothing in
/// this suite compiles generated Rust, which is exactly how the bug survived.
/// #269 owns that, and names this test.)
#[test]
fn an_optional_field_crosses_the_same_however_rust_spells_it() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| -> String {
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZSampleStruct {
                        pub kex: #field_ty,
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
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZKeyExpr)
                    .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZSample)
                    .fields(prebindgen_registry::fields!(z_sample_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_boxed_opt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
        std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
            .expect("read rust")
    };

    let plain = build(syn::parse_quote!(Option<ZKeyExpr>));
    let boxed = build(syn::parse_quote!(Box<Option<ZKeyExpr>>));

    for (label, rust) in [("Option<T>", &plain), ("Box<Option<T>>", &boxed)] {
        let rc: String = rust.split_whitespace().collect();
        // The destructuring is coerced, never applied to the raw place — the
        // one thing that makes it representation-agnostic.
        assert!(
            rc.contains("let__o0:&::core::option::Option<_>=&"),
            "{label}: the optional field is reached through a coercion site:\n{rust}"
        );
        assert!(
            !rc.contains("match&(&__vf0).kex"),
            "{label}: the raw place is never destructured directly:\n{rust}"
        );
        // …and the field still crosses as the child's own expansion.
        assert!(
            rc.contains("myflat::z_keyexpr_as_str(__n0)"),
            "{label}: the child's boundary still applies:\n{rust}"
        );
    }

    // Rust-only wrapper converters may differ internally; the observable JNI
    // callback contract and its leaf conversion remain identical.
    for rust in [&plain, &boxed] {
        assert!(rust.contains("(Ljava/lang/String;)V"), "{rust}");
    }
}

/// Naming a field the value form does not have is the very drift this
/// declarator exists to catch, so it is an error rather than a silent no-op.
#[test]
fn an_adjustment_naming_an_unknown_field_is_an_error() {
    let build = |decl: crate::FieldsDecl| {
        let registry = crate::test_util::reg_from_items(declare_referenced(value_form_items()))
            .expect("index");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .class(crate::ptr_class!(ZBytes))
                    .class(crate::data_class!(ZStamp))
                    .class(crate::data_class!(ZOrigin))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZKeyExpr)
                    .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
            )
            .expand(prebindgen_registry::expand_return!(ZSample).fields(decl));
        let dir = unique_test_dir("jnigen_vf_unknown");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = jni
            .build_with(registry)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };

    for decl in [
        prebindgen_registry::fields!(z_sample_to_struct).name("kex", "kex"),
        prebindgen_registry::fields!(z_sample_to_struct).field(
            "kex",
            prebindgen_registry::expand_return!(ZKeyExpr).field_self(),
        ),
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
    let _ = prebindgen_registry::expand_return!(ZSample)
        .fields(prebindgen_registry::fields!(z_sample_to_struct))
        .fields(prebindgen_registry::fields!(z_sample_to_struct));
}

/// Repeating an adjustment for one field is a declaration bug — the complete
/// set rule, same as `.expand_param` / `.field`.
#[test]
#[should_panic(expected = "already has an override")]
fn a_repeated_override_is_an_error() {
    let _ = prebindgen_registry::fields!(z_sample_to_struct)
        .field(
            "key_expr",
            prebindgen_registry::expand_return!(ZKeyExpr).field_self(),
        )
        .field(
            "key_expr",
            prebindgen_registry::expand_return!(ZKeyExpr).field_self(),
        );
}

/// `"__"` is the reserved chain separator, so an author-supplied rename may
/// not smuggle one in and forge a nesting that isn't there.
#[test]
#[should_panic(expected = "reserved")]
fn a_rename_may_not_contain_the_chain_separator() {
    let _ = prebindgen_registry::fields!(z_sample_to_struct).name("express", "a__b");
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .fun(prebindgen_registry::fun!(z_one_make)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZOne)
                .fields(prebindgen_registry::fields!(z_one_to_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_single");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains(".label).clone()"),
        "the single leaf is CLONED out of the value form, matching the owned \
         `String` its converter takes — composing it as a borrow would feed \
         `&String` to a `String` converter:\n{rust}"
    );
}

/// The same shortcut with a CONSUMING form. It composed its reach straight off
/// the raw value and never looked at the plan's hoists, so it emitted
/// `z_one_into_struct(&__cvsrc)` against a by-value receiver — Rust that does
/// not compile in the consumer's crate — and then cloned a field it owns. Both
/// paths now bind hoists with the same `bind_hoists`, so neither can drift from
/// the other again.
#[test]
fn a_single_leaf_consuming_value_form_moves_its_field() {
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
                pub fn z_one_into_struct(o: ZOne) -> ZOneStruct {
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .fun(prebindgen_registry::fun!(z_one_make)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZOne)
                .fields_self_into(prebindgen_registry::fields!(z_one_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_single_consume");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("z_one_into_struct(__cvsrc)") && !rust.contains("z_one_into_struct(&"),
        "the by-value accessor is handed the value, not a borrow of it:\n{rust}"
    );
    assert!(
        rust.contains(".label") && !rust.contains(".label).clone()"),
        "and the field it owns is MOVED out, not cloned:\n{rust}"
    );
}

/// A binding that states which decompositions the row differential does not
/// compare is held to that set, and an unstated one is held to comparing
/// everything.
///
/// The check is what keeps the differential honest, so it must not be possible
/// to disable by omission: a binding that says nothing gets the empty set, and
/// a decomposition leaving the comparison fails the build. Here `ZChild` is a
/// handle field of a CONSUMING value form, which the decomposition hands over
/// states parts of its own — a `.field_self()` declaration no binding lowers
/// yet — so it is skipped, and an empty expectation refuses it.
#[test]
fn an_unstated_parity_expectation_refuses_a_skipped_decomposition() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZEnvelopeStruct {
                    pub child: ZChild,
                    pub tag: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_envelope_into_struct(e: ZEnvelope) -> ZEnvelopeStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_envelope_sub(cb: impl Fn(ZEnvelope) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let error = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        // What a production build gets by default: nothing skipped.
        .expect_parity_skips::<[&str; 0], &str>([])
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_envelope_sub)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self())
        .expand(
            prebindgen_registry::expand_return!(ZEnvelope)
                .fields_self_into(prebindgen_registry::fields!(z_envelope_into_struct)),
        )
        .build_with(registry)
        .expect_err("a skipped decomposition against an empty expectation");
    let message = error.to_string();
    assert!(
        message.contains("NOT compared against their rows have changed")
            && message.contains("value-form-field-with-parts"),
        "the refusal names the decomposition and why it was skipped: {message}"
    );
}

/// A handle field of a consuming value form is the value form's field like any
/// other: the form gave its value away, so the handle **moves** into its Box
/// rather than being cloned through the borrowed-opaque converter — which also
/// stops `.fields_self_into(..)` from silently requiring a `Clone` the handle type
/// need not have.
///
/// The identity branch computed `consuming` and then returned before using it,
/// so every reached handle took the clone arm; only a handle at the owned ROOT
/// (empty path) moved.
#[test]
fn a_handle_field_of_a_consuming_value_form_moves() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZEnvelopeStruct {
                    pub child: ZChild,
                    pub tag: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_envelope_into_struct(e: ZEnvelope) -> ZEnvelopeStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_envelope_sub(cb: impl Fn(ZEnvelope) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_envelope_sub)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self())
        .expand(
            prebindgen_registry::expand_return!(ZEnvelope)
                .fields_self_into(prebindgen_registry::fields!(z_envelope_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_handle_field_consume");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("Box::new(__vf0.child)"),
        "the handle field is MOVED into its Box:\n{rust}"
    );
    assert!(
        !rust.contains("&__vf0.child"),
        "and is not handed to the borrowed-opaque converter, which would clone \
         it:\n{rust}"
    );
}

/// The cross-product of the two above: a value form whose SOLE field is a
/// handle. That takes the single-leaf `Delivery::Return` shortcut with an
/// *identity* leaf, so neither the multi-leaf handle test (which is a callback
/// plan) nor the single-leaf `String` test (a `Field` leaf) covered it, and the
/// shortcut handed the borrowed converter `&__vf0.child`.
///
/// The fix is in the PLAN, not in the shortcut: an identity leaf under a
/// consuming form resolves its `out_ty` to the OWNED type, which both selects
/// the owning converter and tells every emitter it may move.
#[test]
fn a_sole_handle_field_of_a_consuming_value_form_moves() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZSingleEnvelopeStruct {
                    pub child: ZChild,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_single_envelope_into_struct(e: ZSingleEnvelope) -> ZSingleEnvelopeStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_single_envelope_make() -> ZSingleEnvelope {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZSingleEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_single_envelope_make)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self())
        .expand(
            prebindgen_registry::expand_return!(ZSingleEnvelope)
                .fields_self_into(prebindgen_registry::fields!(z_single_envelope_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_sole_handle_consume");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("z_single_envelope_into_struct(__cvsrc)"),
        "the by-value accessor is handed the value:\n{rust}"
    );
    assert!(
        rust.contains("__vf0.child") && !rust.contains("&__vf0.child"),
        "and the sole handle field is MOVED out, not borrowed into the \
         cloning converter:\n{rust}"
    );
}

/// An `Option<Handle>` field is the same claim behind an `Option` — and the
/// commonest shape there is (`SampleStruct.attachment`). The consuming form
/// owns the whole `Option`, so it is matched BY VALUE and the present handle
/// moves into its Box; only the *reach* differs from the non-optional case, not
/// the ownership.
#[test]
fn an_optional_handle_field_of_a_consuming_value_form_moves() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOptionalEnvelopeStruct {
                    pub child: Option<ZChild>,
                    pub tag: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_optional_envelope_into_struct(
                    e: ZOptionalEnvelope,
                ) -> ZOptionalEnvelopeStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_optional_envelope_sub(
                    cb: impl Fn(ZOptionalEnvelope) + Send + Sync + 'static,
                ) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOptionalEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_optional_envelope_sub)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self())
        .expand(
            prebindgen_registry::expand_return!(ZOptionalEnvelope).fields_self_into(
                prebindgen_registry::fields!(z_optional_envelope_into_struct),
            ),
        );
    let dir = unique_test_dir("jnigen_vf_optional_handle_consume");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("match __vf0.child") && !rust.contains("&__vf0.child"),
        "the `Option` is matched BY VALUE, not borrowed:\n{rust}"
    );
    assert!(
        rust.contains("Box::new(__n)"),
        "and the present handle is MOVED into its Box rather than cloned \
         through the borrowed converter:\n{rust}"
    );
}

/// The cross-product of the two above: a value form whose SOLE field is an
/// `Option<Handle>`. Delivery was chosen on leaf COUNT alone, so this landed on
/// the flat `Delivery::Return` path — which has no `None` arm, and whose
/// `convert_out_ty` names the leaf's own type rather than an optional of it, so
/// it composed `&(&__vf0).child` into a converter typed for `ZChild`.
///
/// A nullable leaf now goes to callback delivery, which has that arm already.
/// Absence is a delivery question, not an ownership one — making `out_ty` owned
/// says who frees the handle, not whether there is one.
#[test]
fn a_sole_optional_handle_field_takes_callback_delivery() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOptionalSingleStruct {
                    pub child: Option<ZChild>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_optional_single_into_struct(e: ZOptionalSingle) -> ZOptionalSingleStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_optional_single_make() -> ZOptionalSingle {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOptionalSingle))
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_optional_single_make)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self())
        .expand(
            prebindgen_registry::expand_return!(ZOptionalSingle)
                .fields_self_into(prebindgen_registry::fields!(z_optional_single_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_sole_optional_handle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        !rust.contains("&(&__vf0).child") && !rust.contains("&__vf0.child"),
        "the optional field is never composed as a borrow — that is what the \
         flat return path did, handing `&Option<ZChild>` to a `ZChild` \
         converter:\n{rust}"
    );
    assert!(
        rust.contains("match __vf0.child") && rust.contains("Box::new(__n)"),
        "it takes callback delivery, whose `None` arm exists, and the present \
         handle still moves:\n{rust}"
    );
}

/// A ROOT identity leaf owns its value with no value form in sight — a plain
/// `-> ZChild` return under the type-level `expand_return!(ZChild).field_self()`
/// that exists so the same boundary can be spliced as a value-form field. The
/// flat return path tied its move to the rebased hoist's `consuming` flag,
/// which is `false` when there is no hoist, so it emitted `&__cvsrc` into the
/// owning `ZChild`-to-jlong converter.
///
/// Ownership is not "a consuming form gave it to me" — that is one of its two
/// sources. For an identity leaf the plan already states it in `out_ty`, so the
/// emitter reads it there rather than re-deriving it from the hoist. The
/// `Option` return is the same value inside a `map` closure.
#[test]
fn an_owned_root_identity_moves_without_any_value_form() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_root_child_make() -> ZChild {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_root_child_maybe() -> Option<ZChild> {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZChild))
                .fun(prebindgen_registry::fun!(z_root_child_make))
                .fun(prebindgen_registry::fun!(z_root_child_maybe)),
        )
        .expand(prebindgen_registry::expand_return!(ZChild).field_self());
    let dir = unique_test_dir("jnigen_vf_root_identity");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        !rust.contains("&__cvsrc") && !rust.contains("&__inner"),
        "an owned root is MOVED into its converter, not borrowed — the owning \
         converter takes `ZChild`, not `&ZChild`:\n{rust}"
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
        let registry = crate::test_util::reg_from_items(declare_referenced(value_form_items()))
            .expect("index");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .class(crate::ptr_class!(ZBytes))
                    .class(crate::data_class!(ZStamp))
                    .class(crate::data_class!(ZOrigin))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZKeyExpr)
                    .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
            )
            .expand(prebindgen_registry::expand_return!(ZSample).fields(
                // `key_expr` is a `ZKeyExpr`, not a `ZBytes`.
                prebindgen_registry::fields!(z_sample_to_struct).field(
                    "key_expr",
                    prebindgen_registry::expand_return!(ZBytes).field_self(),
                ),
            ));
        let dir = unique_test_dir("jnigen_vf_ovr_ty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = jni
            .build_with(registry)
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOuter))
                .class(crate::ptr_class!(ZInner))
                .fun(prebindgen_registry::fun!(z_outer_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZInner)
                .fields(prebindgen_registry::fields!(z_inner_to_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZOuter)
                .fields(prebindgen_registry::fields!(z_outer_to_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_nested");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
    let items = |outer_by_value: bool| -> Vec<(syn::Item, prebindgen::SourceLocation)> {
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
            prebindgen_registry::expand_return!(ZOuter)
                .fields(prebindgen_registry::fields!(z_outer_to_struct)),
        ),
        (
            "consume",
            true,
            prebindgen_registry::expand_return!(ZOuter)
                .fields_self_into(prebindgen_registry::fields!(z_outer_into_struct)),
        ),
    ] {
        let registry = crate::test_util::reg_from_items(declare_referenced(items(outer_by_value)))
            .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZOuter))
                    .class(crate::ptr_class!(ZInner))
                    .fun(prebindgen_registry::fun!(z_outer_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZInner)
                    .fields_self_into(prebindgen_registry::fields!(z_inner_into_struct)),
            )
            .expand(outer);
        let dir = unique_test_dir(&format!("jnigen_vf_nested_consume_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
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
fn nested_review_items() -> Vec<(syn::Item, prebindgen::SourceLocation)> {
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

fn nested_review_jni(outer: crate::ExpandReturnDecl) -> JniGenBuilder {
    JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZReviewOuter))
                .class(crate::ptr_class!(ZReviewInner))
                .fun(prebindgen_registry::fun!(z_review_outer_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZReviewInner)
                .fields(prebindgen_registry::fields!(z_review_inner_to_struct)),
        )
        .expand(outer)
}

/// A value form below an `Option` is a CONDITIONAL hoist — supported at one
/// level (see [`a_value_form_under_an_optional_accessor_is_hoisted_conditionally`]),
/// but it cannot NEST: the inner form would have to be bound inside the outer
/// one's `Some` arm, and the binder has no arm to put a second local in.
/// Reject it during planning rather than emit a hoist that reaches through an
/// `Option` it cannot unwrap.
#[test]
fn an_optional_nested_value_form_is_rejected_before_emission() {
    let registry = crate::test_util::reg_from_items(declare_referenced(nested_review_items()))
        .expect("index items");
    let jni = nested_review_jni(
        prebindgen_registry::expand_return!(ZReviewOuter)
            .fields(prebindgen_registry::fields!(z_review_outer_to_struct)),
    );
    let err = match jni.build_with(registry) {
        Ok(_) => panic!("an optional nested value form must be rejected"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("z_review_inner_to_struct") && msg.contains("Option"),
        "the error names the unsupported conditional hoist: {msg}"
    );
}

/// A value form reached through an `Option`-returning accessor — the shape a
/// reply's optional sample has. The form runs only where the value is present,
/// so it binds an `Option<TStruct>` local; every leaf under it then lives in
/// ONE `match` on that local, whose absent arm fills each slot with the same
/// wire default an inert sum group carries. Without this the whole containing
/// decomposition was refused.
#[test]
fn a_value_form_under_an_optional_accessor_is_hoisted_conditionally() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_get_carrier(h: &ZHolder) -> Option<&ZCarrier> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZHolder)
                .field(prebindgen_registry::fun!(zh_get_carrier)),
        );
    let dir = unique_test_dir("jnigen_vf_conditional");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni
        .build_with(registry)
        .expect("a conditional hoist resolves");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert_eq!(
        rust.matches("zc_into_struct").count(),
        1,
        "the value form runs ONCE, not once per leaf:\n{rust}"
    );
    assert!(
        rust.contains("zh_get_carrier(&__cb_arg0)") && rust.contains(".map(|__hb0|"),
        "the hoist is built only where the optional step has a value — as a \
         `map`, since the equivalent `match` trips `clippy::manual_map` in the \
         consumer:\n{rust}"
    );
    assert!(
        rust.contains("zc_into_struct((__hb0).clone())"),
        "what the arm binds is a borrow, so a consuming accessor gets a clone \
         of it — the same trade a borrowed root makes:\n{rust}"
    );
    assert!(
        rust.contains("match __vf0 {") && rust.contains("Some(__u0)"),
        "and the leaves share ONE match on the local, taken by value so the \
         struct's fields still move out:\n{rust}"
    );
    assert!(
        rust.contains("__u0.label") && !rust.contains("__u0.label.clone()"),
        "each leaf reads its own field off the arm binding, moved not cloned:\n{rust}"
    );
    assert!(
        rust.contains("JObject::null()"),
        "the absent arm fills every slot with the wire default:\n{rust}"
    );
}

/// The optional step may hand over an OWNED payload (`Option<T>`) as readily
/// as a borrowed one (`Option<&T>`), and the two need opposite treatment at the
/// value-form call: an owned payload is borrowed for a `&Self` accessor and
/// MOVED into a by-value one, where a borrowed payload is passed straight
/// through and cloned. Getting this from the accessor's own signature is what
/// keeps a by-value accessor from demanding a `Clone` its type need not have.
fn conditional_owned_gen(tag: &str, decl: crate::ExpandReturnDecl) -> String {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_take_carrier(h: &ZHolder) -> Option<ZCarrier> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(decl)
        .expand(
            prebindgen_registry::expand_return!(ZHolder)
                .field(prebindgen_registry::fun!(zh_take_carrier)),
        );
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust")
}

/// An owned payload may have ORDINARY steps between it and the value form, and
/// those compose onto whatever the arm bound. Marking every `Some` binding as
/// already-borrowed handed the bare `T` to the next accessor, which is typed
/// for `&T` — recording ownership only at the value form cannot repair a call
/// that sits before it.
#[test]
fn an_owned_optional_payload_is_borrowed_for_the_steps_after_it() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_child(h: &ZHolder) -> Option<ZChild> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zchild_carrier(c: &ZChild) -> &ZCarrier {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZChild))
                .class(crate::ptr_class!(ZHolder))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields(prebindgen_registry::fields!(zc_to_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZChild)
                .field(prebindgen_registry::fun!(zchild_carrier)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZHolder).field(prebindgen_registry::fun!(zh_child)),
        );
    let dir = unique_test_dir("jnigen_vf_cond_owned_chain");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("zchild_carrier(&__hb0)"),
        "the step after an owned payload BORROWS it — passing the bare value \
         would hand `T` to an accessor typed for `&T`:\n{rust}"
    );
}

/// Sibling hoists under ONE consuming parent: the first moves a field out of
/// it, so the second may not borrow the parent as a whole. Its leading field
/// run has to be projected directly (`&__vf0.wrapper`) — a disjoint borrow that
/// survives the move — rather than reached through the parent
/// (`&(&__vf0).wrapper`), which is a borrow of a partially moved value.
#[test]
fn a_rebased_hoist_projects_its_leading_fields_past_a_sibling_move() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOuterStruct {
                    pub direct: ZCarrier,
                    pub wrapper: ZWrapper,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zo_into_struct(o: ZOuter) -> ZOuterStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zw_carrier(w: &ZWrapper) -> ZCarrier {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zo_sub(cb: impl Fn(ZOuter) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZWrapper))
                .class(crate::ptr_class!(ZOuter))
                .fun(prebindgen_registry::fun!(zo_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZWrapper)
                .field(prebindgen_registry::fun!(zw_carrier)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZOuter)
                .fields_self_into(prebindgen_registry::fields!(zo_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_sibling_move");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("zc_into_struct(__vf0.direct)"),
        "the first sibling still MOVES its field out of the parent:\n{rust}"
    );
    assert!(
        rust.contains("zw_carrier(&__vf0.wrapper)"),
        "and the second projects its own field directly — a disjoint borrow \
         that survives that move:\n{rust}"
    );
    assert!(
        !rust.contains("&(&__vf0)"),
        "borrowing the partially moved parent as a whole is what E0382 rejects:\n{rust}"
    );
}

/// A consuming value form reached through ORDINARY accessors: the chain in
/// front of it folds by the borrowing rule, and the form itself still takes its
/// receiver by value. Both boundaries are in one expression, and neither may be
/// decided by the other — `zh_carrier` wants `&ZHolder`, `zc_into_struct` wants
/// the `ZCarrier` it returns, moved.
#[test]
fn a_consuming_value_form_keeps_its_by_value_boundary_behind_accessors() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_carrier(h: &ZHolder) -> ZCarrier {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZHolder)
                .field(prebindgen_registry::fun!(zh_carrier)),
        );
    let dir = unique_test_dir("jnigen_vf_consume_behind_acc");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("zc_into_struct(myflat::zh_carrier(&__cb_arg0))"),
        "the accessor in front borrows its receiver, and its owned result MOVES \
         into the by-value value form — neither boundary decides the other:\n{rust}"
    );
}

/// Ownership has to survive EVERY call on the chain, not just the optional
/// binding and the value form. An ordinary accessor returning an owned value
/// leaves that value in hand, and the next accessor takes its receiver by
/// reference — so the fold has to borrow between them wherever it happens.
#[test]
fn an_owned_intermediate_result_is_borrowed_for_the_next_step() {
    let loc = myflat_loc();
    let mut items = consuming_items();
    items.extend([
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_child(h: &ZHolder) -> Option<ZChild> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zchild_middle(c: &ZChild) -> ZMiddle {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zmiddle_carrier(m: &ZMiddle) -> &ZCarrier {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ]);
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZMiddle))
                .class(crate::ptr_class!(ZChild))
                .class(crate::ptr_class!(ZHolder))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields(prebindgen_registry::fields!(zc_to_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZMiddle)
                .field(prebindgen_registry::fun!(zmiddle_carrier)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZChild)
                .field(prebindgen_registry::fun!(zchild_middle)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZHolder).field(prebindgen_registry::fun!(zh_child)),
        );
    let dir = unique_test_dir("jnigen_vf_cond_owned_middle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    assert!(
        rust.contains("zmiddle_carrier(&myflat::zchild_middle(&__hb0))"),
        "an owned INTERMEDIATE result is borrowed for the call that follows \
         it, exactly as the optional payload is:\n{rust}"
    );
}

#[test]
fn an_owned_optional_payload_is_borrowed_for_a_borrowing_value_form() {
    let rust = conditional_owned_gen(
        "jnigen_vf_cond_owned_borrow",
        prebindgen_registry::expand_return!(ZCarrier)
            .fields(prebindgen_registry::fields!(zc_to_struct)),
    );
    assert!(
        rust.contains("zc_to_struct(&__hb0)"),
        "an owned `Option<T>` payload is BORROWED for a `&Self` accessor — \
         passing it through would supply `T` where `&T` is required:\n{rust}"
    );
}

#[test]
fn an_owned_optional_payload_is_moved_into_a_consuming_value_form() {
    let rust = conditional_owned_gen(
        "jnigen_vf_cond_owned_consume",
        prebindgen_registry::expand_return!(ZCarrier)
            .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
    );
    assert!(
        rust.contains("zc_into_struct(__hb0)"),
        "an owned payload MOVES into a by-value accessor:\n{rust}"
    );
    assert!(
        !rust.contains("zc_into_struct((__hb0).clone())"),
        "cloning it would demand a `Clone` the type need not have, on a value \
         that was already ours:\n{rust}"
    );
}

/// A conditional value form may carry a SUM field, and a sum's segment is
/// emitted as one `match` of its own rather than as per-leaf statements. That
/// segment belongs INSIDE the conditional arm like every other leaf under the
/// form: emitted before it, it would reach a binding that does not exist yet
/// (`cannot find value __u0 in this scope`), and the sum's leaves — which are
/// held out of the per-leaf ordering — would never be routed into the arm at
/// all.
#[test]
fn a_sum_field_of_a_conditional_value_form_stays_inside_the_arm() {
    let loc = myflat_loc();
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
                pub struct ZCarrierStruct {
                    pub outcome: ZOutcome,
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
                pub fn zh_get_carrier(h: &ZHolder) -> Option<&ZCarrier> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn zh_sub(cb: impl Fn(ZHolder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .class(crate::sealed_class!(ZOutcome))
                .fun(prebindgen_registry::fun!(zh_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZHolder)
                .field(prebindgen_registry::fun!(zh_get_carrier)),
        );
    let dir = unique_test_dir("jnigen_vf_conditional_sum");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");

    let arm = rust
        .split_once("Some(__u0)")
        .expect("the conditional arm is emitted")
        .1;
    assert!(
        arm.contains("__u0.outcome"),
        "the sum's `match` is emitted INSIDE the arm that binds `__u0`, after \
         it exists:\n{rust}"
    );
    assert!(
        !rust
            .split_once("Some(__u0)")
            .expect("the conditional arm is emitted")
            .0
            .contains("__u0."),
        "and nothing reaches the binding before the arm introduces it:\n{rust}"
    );
    assert!(
        arm.contains("ZOutcome::Failed"),
        "the variant arms are the sum's own, unchanged by being nested:\n{rust}"
    );
    assert!(
        rust.contains("__u0.count"),
        "the ordinary sibling leaf still rides the same arm:\n{rust}"
    );
}

/// Override records are applied to a `Vec<T>` field as a whole; a fixed leaf
/// list cannot apply `T`'s deconstructor once per element. The declaration
/// check must compare against `Vec<T>`, not peel it to `T`.
#[test]
fn a_vec_field_override_must_name_the_whole_vec_type() {
    let build = || {
        let registry = crate::test_util::reg_from_items(declare_referenced(nested_review_items()))
            .expect("index items");
        let jni = nested_review_jni(prebindgen_registry::expand_return!(ZReviewOuter).fields(
            prebindgen_registry::fields!(z_review_outer_to_struct).field(
                "items",
                prebindgen_registry::expand_return!(ZReviewInner).field_self(),
            ),
        ));
        let _ = jni.build_with(registry);
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
fn consuming_items() -> Vec<(syn::Item, prebindgen::SourceLocation)> {
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

fn consuming_gen(tag: &str, decl: crate::ExpandReturnDecl) -> String {
    let registry = crate::test_util::reg_from_items(declare_referenced(consuming_items()))
        .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .fun(prebindgen_registry::fun!(zc_sub)),
        )
        .expand(decl);
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
        prebindgen_registry::expand_return!(ZCarrier)
            .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
    );
    assert!(
        rust.contains("zc_into_struct(__cb_arg0)"),
        "the value is passed BY MOVE, not borrowed:\n{rust}"
    );
    assert!(
        rust.contains("__vf0.label") && rust.contains("__vf0.count"),
        "each field is read off the one hoisted local:\n{rust}"
    );
    // Spelled the way a clone reaches a field it does not own — off the
    // borrow, `(&__vf0.label).clone()` — since that is what would appear here
    // if the form stopped consuming.
    assert!(
        !rust.contains(".label).clone()") && !rust.contains(".count).clone()"),
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
        prebindgen_registry::expand_return!(ZCarrier)
            .fields(prebindgen_registry::fields!(zc_to_struct)),
    );
    assert!(
        rust.contains("zc_to_struct(&__cb_arg0)"),
        "a `&T` accessor is still handed a borrow:\n{rust}"
    );
    assert!(
        rust.contains("(&__vf0.label).clone()"),
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZVault))
                .fun(prebindgen_registry::fun!(zc_sub))
                .fun(prebindgen_registry::fun!(zc_borrowed)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZCarrier)
                .fields_self_into(prebindgen_registry::fields!(zc_into_struct)),
        );
    let dir = unique_test_dir("jnigen_vf_consume_ref");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let (composed_only, ident, _) = gen
        .parts_plan_for_test(
            syn::parse_quote!(Option<&ZCarrier>),
            prebindgen_registry::recipe::Direction::Deconstruct,
        )
        .expect("Option<&ZCarrier> parts row");
    assert!(
        composed_only && ident.starts_with("__jni_out_convert_"),
        "the value-form decomposition must retain a non-rendering parts row"
    );
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");
    assert!(
        // Parenthesized because the clone applies to whatever the fold holds,
        // which is not always a bare name — `&__out.clone()` would parse as
        // `&(__out.clone())` and clone the wrong thing.
        rust.contains("zc_into_struct((__inner).clone())"),
        "a borrowed plan clones the value, then consumes the clone:\n{rust}"
    );
}

/// `.fields_self_into(..)` gives the value away, so anything else reading it is
/// broken by construction. Refused **where it is declared** — the collision is
/// visible in the decl itself, so it does not need a resolve to be found.
///
/// `.field_self()` beside it would deliver the handle the form just consumed.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_following_sibling() {
    let _ = prebindgen_registry::expand_return!(ZCarrier)
        .fields_self_into(prebindgen_registry::fields!(zc_into_struct))
        .field_self();
}

/// And the other way round — the decl is a builder, so both orders must be
/// caught or the rule holds only for the order someone happened to write.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_preceding_sibling() {
    let _ = prebindgen_registry::expand_return!(ZCarrier)
        .field_self()
        .fields_self_into(prebindgen_registry::fields!(zc_into_struct));
}

/// Any sibling record, not just the identity one.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_plain_field_sibling() {
    let _ = prebindgen_registry::expand_return!(ZCarrier)
        .fields_self_into(prebindgen_registry::fields!(zc_into_struct))
        .field(prebindgen_registry::fun!(zc_to_struct));
}

/// The declarator states whether the value is given away and the accessor's
/// signature has to agree — otherwise the emitted call would not compile in the
/// consumer's crate, and a boundary would silently stop being the one declared.
/// Both directions are errors; the fixture has one accessor of each kind.
#[test]
fn the_declarator_and_the_accessor_s_receiver_must_agree() {
    let build = |decl: crate::ExpandReturnDecl| -> String {
        let registry =
            crate::test_util::reg_from_items(declare_referenced(consuming_items())).expect("index");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZCarrier))
                    .fun(prebindgen_registry::fun!(zc_sub)),
            )
            .expand(decl);
        match jni.build_with(registry) {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        }
    };

    let msg = build(
        prebindgen_registry::expand_return!(ZCarrier)
            .fields_self_into(prebindgen_registry::fields!(zc_to_struct)),
    );
    assert!(
        msg.contains("CONSUMING") && msg.contains("zc_to_struct"),
        "`.fields_self_into` on a borrowing accessor must be refused, naming it: {msg:?}"
    );

    let msg = build(
        prebindgen_registry::expand_return!(ZCarrier)
            .fields(prebindgen_registry::fields!(zc_into_struct)),
    );
    assert!(
        msg.contains("BORROWING") && msg.contains("zc_into_struct"),
        "`.fields` on a by-value accessor must be refused, naming it: {msg:?}"
    );
}

/// A field's optional-ness is `kind`'s; how Rust spells it is the source's,
/// and a **whole-value crossing** must not care either — the half #268 could
/// not reach.
///
/// #268 fixed the *access path* for a decomposed child. A field with no
/// deconstructor needs its own converter, and converter selection dispatched by
/// rebuilding a pattern from source syntax: `with_first_arg(Box<Option<T>>)`
/// yielded `Box<_>`, which matched no handler, so the crossing got no converter
/// at all and failed resolution by name (#270).
///
/// Both model structures now resolve, and Flat emits the converter input from
/// that structure. Declaring `Option<T>` for a modeled `Box<Option<T>>` value
/// would mismatch its own call site.
#[test]
fn a_whole_value_crossing_uses_only_its_flat_structure() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| -> String {
        let items = vec![
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
                    pub struct ZSampleStruct {
                        pub stamp: #field_ty,
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
                    pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::data_class!(ZStamp))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZSample)
                    .fields(prebindgen_registry::fields!(z_sample_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_whole_boxed");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Resolving at all is half the assertion: this is what #270 reported as
        // `Unresolved { key: "Box < Option < ZStamp > >" }`.
        let gen = jni.build_with(registry).expect("resolve");
        std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
            .expect("read rust")
    };

    let plain = build(syn::parse_quote!(Option<ZStamp>));
    let boxed = build(syn::parse_quote!(Box<Option<ZStamp>>));

    let bc: String = boxed.split_whitespace().collect();
    // Flat emits the complete modeled wrapper structure.
    assert!(
        bc.contains("v:::std::boxed::Box<::core::option::Option<myflat::ZStamp>>",),
        "the converter takes the model-emitted type:\n{boxed}"
    );
    // ...and matches the canonical shape directly before destructuring.
    assert!(
        bc.contains("match*v"),
        "the modeled wrapper is used as the canonical shape:\n{boxed}"
    );
    // The Kotlin surface is the wrapper's business only in Rust: both
    // spellings deliver the same nullable data class.
    for (label, rust) in [("Option<T>", &plain), ("Box<Option<T>>", &boxed)] {
        assert!(
            rust.contains("__jni_out_convert_"),
            "{label}: the field still crosses as its own converter:\n{rust}"
        );
    }
}

/// An owned string crosses the same however Rust spells it, with no
/// per-spelling arm behind it.
///
/// `Box<String>` used to work only because two `TypeKey == "Box < String >"`
/// matches were written by hand — one per direction. That is what a
/// spelling-keyed converter table costs: a hardcoded case for every
/// representation someone happens to write. Both are deleted; `kind == Str`
/// dispatches, the signature comes from the spelling, and `.into()` constructs
/// it.
#[test]
fn an_owned_string_crosses_the_same_however_rust_spells_it() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| -> String {
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZSampleStruct {
                        pub label: #field_ty,
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
                    pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZSample)
                    .fields(prebindgen_registry::fields!(z_sample_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_boxed_string");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
        let (rust, kotlin) = (
            std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
                .expect("read rust"),
            gen.write_kotlin(&dir.join("kotlin"))
                .expect("write_kotlin")
                .iter()
                .map(|p| std::fs::read_to_string(p).unwrap())
                .collect::<Vec<_>>()
                .join("\n"),
        );
        format!("{rust}\n// ---KOTLIN---\n{kotlin}")
    };

    for spelling in [
        syn::parse_quote!(String),
        syn::parse_quote!(Box<String>),
        syn::parse_quote!(Cow<'static, str>),
    ] {
        let out = build(spelling);
        assert!(
            out.contains("String"),
            "every owned-string spelling crosses as a Kotlin String:\n{out}"
        );
    }
}

/// A transparent wrapper is bridged only where generated Rust **can** bridge
/// it, and an unsupported representation is refused at selection rather than
/// emitted as code that will not compile.
///
/// The model erases more than `Box`: `Cow<'_, T>` *is* `T` too. But a converter
/// that MOVES its payload has to undo the exact wrapper the source wrote, and
/// there is no trait for that — `Box<T> → T` is `*b`, `Cow<'_, T> → T::Owned`
/// is `into_owned()`, and a `Cow` cannot be moved through at all (`E0507`).
/// Layers are counted, too: `Box<Box<Option<T>>>` is `Optional`, and one
/// dereference leaves `Box<Option<T>>`.
///
/// Both shapes used to RESOLVE and emit `let v: Option<String> = *v;` — the
/// worst outcome available, because resolution succeeding is what tells the
/// binding its type is supported. Failing to resolve names the type; emitting
/// unbuildable Rust names nothing (#270 review).
#[test]
fn a_transparent_wrapper_is_bridged_only_where_it_can_be() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| -> Result<String, String> {
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZSampleStruct {
                        pub f: #field_ty,
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
                    pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .fun(prebindgen_registry::fun!(z_sample_sub)),
            )
            .expand(
                prebindgen_registry::expand_return!(ZSample)
                    .fields(prebindgen_registry::fields!(z_sample_to_struct)),
            );
        let dir = unique_test_dir("jnigen_vf_bridge");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        match jni.build_with(registry) {
            Ok(g) => Ok(std::fs::read_to_string(
                g.write_rust(dir.join("g.rs")).expect("write_rust"),
            )
            .expect("read rust")),
            Err(e) => Err(format!("{e}")),
        }
    };

    // One box: bridged with one dereference in the direct shape match.
    // Generated code still runs through the consumer's own lints (#292).
    let one = build(syn::parse_quote!(Box<Option<String>>)).expect("a single box is bridgeable");
    let oc: String = one.split_whitespace().collect();
    assert!(oc.contains("match*v"), "one layer, one dereference:\n{one}");

    // Two boxes: bridged with TWO. This is the case a single deref got wrong,
    // silently — one `*` on `Box<Box<_>>` still leaves a `Box<_>`.
    let two =
        build(syn::parse_quote!(Box<Box<Option<String>>>)).expect("nested boxes are bridgeable");
    let tc: String = two.split_whitespace().collect();
    assert!(
        tc.contains("match**v"),
        "two layers, two dereferences:\n{two}"
    );

    // `Cow` is erased by the model and CANNOT be moved through, so the crossing
    // must not resolve. The diagnosis names the type.
    let cow = build(syn::parse_quote!(Cow<'static, Option<String>>))
        .expect_err("a Cow payload cannot be moved out, so it must not resolve");
    assert!(
        cow.contains("could not be resolved") && cow.contains("Cow"),
        "the refusal names the unsupported representation: {cow}"
    );
}

/// An erased wrapper over a **terminal** crosses in BOTH directions, and one
/// selector arm serves every terminal kind (#309).
///
/// The layer arms bridge a wrapper as part of handling their own layer, so
/// `Box<Option<T>>` and `Box<Vec<T>>` resolved all along — which is what made
/// the gap hard to see. A wrapper over a plain `TypeKind::Named` has no arm, and
/// the terminal lookup keys on the SPELLING: no config sits under
/// `Box < Priority >`. Inbound that was a last resort added by #294; outbound it
/// was nothing at all, so the same field was a parameter this binding could take
/// and a return it could not give.
///
/// The three terminal kinds are asserted together because they miss the terminal
/// lookup the same way, and one arm therefore covers them all — a claim worth
/// showing rather than arguing.
#[test]
fn an_erased_wrapper_over_a_terminal_crosses_both_ways() {
    use prebindgen_registry::Conversions as _;

    let loc = myflat_loc();
    // The enum is carried wrapped AND bare, so the Kotlin assertion below can
    // say "these present alike" rather than merely "the wrapped one compiles".
    // The handle and the data class are wrapped only: what they are here to
    // show is that ONE arm serves every terminal kind, and a bare twin of each
    // would test the terminal lookup rather than the bridge.
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Leaf {
                    pub v: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Wrapped {
                    pub boxed_enum: Box<Priority>,
                    pub plain_enum: Priority,
                    pub boxed_handle: Box<ZSample>,
                    pub boxed_data: Box<Leaf>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_sample_to_wrapped(s: &ZSample) -> Wrapped {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_wrapped_take(w: Wrapped) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZSample))
                .class(crate::enum_class!(Priority))
                .class(crate::data_class!(Leaf))
                .class(crate::data_class!(Wrapped))
                .fun(prebindgen_registry::fun!(z_wrapped_take)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZSample)
                .fields(prebindgen_registry::fields!(z_sample_to_wrapped)),
        );
    let dir = unique_test_dir("jnigen_terminal_bridge");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Resolving at all is the claim: every one of these reached `None` outbound
    // before, and the build failed naming the wrapped spelling.
    let gen = jni
        .build_with(registry)
        .expect("an erased wrapper over a terminal resolves in both directions");
    for ty in [
        syn::parse_quote!(Box<Priority>),
        syn::parse_quote!(Box<ZSample>),
        syn::parse_quote!(Box<Leaf>),
    ] {
        let reading = gen.registry.reading_of(&ty).expect("wrapped reading");
        assert!(
            gen.decls
                .in_frag(&reading)
                .expect("wrapped input")
                .is_transparent_plan(),
            "the input bridge must retain a frozen transparent-wrapper plan"
        );
        assert!(
            gen.decls
                .out_frag(&reading)
                .expect("wrapped output")
                .is_transparent_plan(),
            "the output bridge must retain a frozen transparent-wrapper plan"
        );
    }
    let rust =
        std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust")).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Outbound: the wrapper comes off, then the inner converter runs. Inbound
    // is the mirror — the inner converter runs, then the wrapper goes back on.
    assert!(rc.contains("Box::new(__inner)"), "{rust}");
    assert!(rc.contains("let__inner=*v"), "{rust}");
    assert!(
        rc.contains("Box::into_raw"),
        "the reached Box<ZSample> output bridge must reach its opaque child:\n{rust}"
    );
    // The wrapper is invisible to Kotlin, so the bare and wrapped enum fields
    // present as the same type — the point of the model erasing it.
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();
    assert!(
        kc.contains("valboxedEnum:Priority") && kc.contains("valplainEnum:Priority"),
        "a wrapped enum presents as the enum class, exactly as the bare one does:\n{kotlin}"
    );
}

/// A borrowed data class crosses through the same registry-owned Optional
/// composition as its by-value twin. Its converter owns `Option<T>` long
/// enough for the final wrapper to lend `Option<&T>` to the source function.
/// Fixed-layout declarations stay flattened; whole-object input remains an
/// explicit opt-in.
#[test]
fn an_optional_data_class_borrow_uses_an_owned_registry_carrier() {
    let build = |jobject_input: bool| -> (String, String) {
        let loc = myflat_loc();
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZData {
                        pub value: i64,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_take(t: Option<&ZData>) -> i64 {
                        unimplemented!()
                    }
                )),
                loc,
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let class = if jobject_input {
            crate::data_class!(ZData).jobject_input()
        } else {
            crate::data_class!(ZData)
        };
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(class)
                    .fun(prebindgen_registry::fun!(z_take)),
            );
        let dir = unique_test_dir(if jobject_input {
            "jnigen_optional_borrowed_jobject_data"
        } else {
            "jnigen_optional_borrowed_flat_data"
        });
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni
            .build_with(registry)
            .expect("an optional borrow resolves");
        let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
            .expect("read rust");
        let kotlin = gen
            .write_kotlin(&dir.join("kotlin"))
            .expect("write_kotlin")
            .iter()
            .map(|path| std::fs::read_to_string(path).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        (rust, kotlin)
    };

    let (flat_rust, flat_kotlin) = build(false);
    assert!(
        flat_rust.contains("myflat::z_take(t.as_ref())"),
        "the flattened Optional carrier is borrowed only at the source call:\n{flat_rust}"
    );
    assert!(
        flat_rust.contains("t_present: jni::sys::jboolean")
            && flat_rust.contains("t_value: jni::sys::jlong"),
        "the borrowed data class keeps the allocation-free flattened ABI:\n{flat_rust}"
    );
    assert!(
        flat_kotlin.contains("t: ZData?"),
        "the public Kotlin parameter stays a nullable data class:\n{flat_kotlin}"
    );
    assert!(
        !flat_rust.contains(".call_method"),
        "the flattened site must not retain an unused reflective decoder:\n{flat_rust}"
    );

    let (object_rust, object_kotlin) = build(true);
    assert!(
        object_rust.contains("myflat::z_take(t.as_ref())"),
        "the whole-object Optional carrier is borrowed only at the source call:\n{object_rust}"
    );
    assert!(
        object_rust.contains("t: jni::objects::JObject"),
        "an explicit whole-object declaration keeps its JObject ABI:\n{object_rust}"
    );
    assert!(
        object_kotlin.contains("t: ZData?"),
        "the whole-object public parameter is still nullable:\n{object_kotlin}"
    );
}

#[test]
fn an_unbuildable_optional_borrow_is_explicitly_refused() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZData {
                    pub value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_take(t: Box<Option<&ZData>>) -> i64 {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(ZData))
                .fun(prebindgen_registry::fun!(z_take)),
        );
    let error = match jni.build_with(registry) {
        Ok(_) => panic!("an outer wrapper cannot contain converter-local borrows"),
        Err(error) => error.to_string(),
    };
    assert!(
        error.contains("could not be resolved") && error.contains("Box < Option < & ZData > >"),
        "the unsupported outer wrapper must be refused and name its crossing: {error}"
    );

    // The finished-registry diagnosis intentionally reports completeness,
    // because scanned but unreachable crossings may refuse without making the
    // binding invalid. Compile the same crossing directly to pin the adapter's
    // exact capability boundary as well.
    let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZData {
                pub value: i64,
            }
        )),
        myflat_loc(),
    )]))
    .expect("index the seam type");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::data_class!(ZData)))
        .build_with(registry)
        .expect("build the supported neighbouring crossings");
    let refusal = gen
        .crossing_plan_for_test(
            syn::parse_quote!(Box<Option<&ZData>>),
            prebindgen_registry::recipe::Direction::Construct,
        )
        .expect_err("the Optional composer must reject a wrapped borrowed carrier");
    assert!(
        refusal.contains("no registry-composed JNI representation for this optional"),
        "the seam must name the exact Optional composition boundary: {refusal}"
    );
}

/// A wrapper cannot be bridged where the converter does not produce the spelled
/// type at all — the **borrow** shapes — so those refuse rather than resolve.
///
/// `&T` and `Option<&T>` are served by handing back the inner type's own
/// converter (or an owned transient) and letting the call site add the final
/// borrow. There is no value in hand to unwrap a representation from, so
/// `Box<&T>` would pass an owned `T` where `Box<&T>` is expected, and
/// `Box<Option<&T>>` would have to construct references owned by a local
/// converter. Both are refused.
///
/// The canonical spellings are asserted alongside, because a guard that also
/// refused those would be worse than no guard.
#[test]
fn a_wrapped_borrow_has_nothing_to_bridge_and_refuses() {
    let loc = myflat_loc();
    let build = |param_ty: syn::Type| -> Result<(String, bool), String> {
        let spelling = param_ty.to_token_stream().to_string();
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZThing {
                        pub v: i64,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_take(t: #param_ty) -> i64 {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZThing))
                    .fun(prebindgen_registry::fun!(z_take)),
            );
        let dir = unique_test_dir("jnigen_wrapped_borrow");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        match jni.build_with(registry) {
            Ok(g) => {
                let planned = g
                    .borrowed_optional_handle_plan_for_test(&spelling)
                    .expect("compiled input crossing");
                Ok((
                    std::fs::read_to_string(g.write_rust(dir.join("g.rs")).expect("write_rust"))
                        .expect("read rust"),
                    planned,
                ))
            }
            Err(e) => Err(format!("{e}")),
        }
    };

    // The canonical borrows still resolve, and still adapt at the call site.
    let (borrowed, borrowed_is_plan) =
        build(syn::parse_quote!(&ZThing)).expect("a plain borrow resolves");
    assert!(!borrowed_is_plan, "a plain borrow is not the Optional plan");
    assert!(
        borrowed.contains("myflat::z_take(&t)"),
        "the call site adds the borrow:\n{borrowed}"
    );
    let (opt, opt_is_plan) =
        build(syn::parse_quote!(Option<&ZThing>)).expect("an optional borrow resolves");
    assert!(
        opt_is_plan,
        "the optional borrow must retain an unrendered borrowed-handle plan"
    );
    assert!(
        opt.contains("myflat::z_take(t.as_deref())"),
        "the call site derefs the OwnedObject:\n{opt}"
    );
    assert!(
        opt.contains("OwnedObject::from_raw"),
        "the plan renders the non-owning carrier:\n{opt}"
    );

    let (mutable, mutable_is_plan) =
        build(syn::parse_quote!(Option<&mut ZThing>)).expect("an optional mutable borrow resolves");
    assert!(
        mutable_is_plan,
        "the optional mutable borrow must retain the same late plan"
    );
    assert!(
        mutable.contains("myflat::z_take(t.as_deref_mut())"),
        "the call site mutably derefs the OwnedObject:\n{mutable}"
    );

    // Wrapped, they have nothing to bridge and must not resolve.
    for spelling in [
        syn::parse_quote!(Box<&ZThing>),
        syn::parse_quote!(Box<Option<&ZThing>>),
    ] {
        let err = build(spelling).expect_err("a wrapped borrow must not resolve");
        assert!(
            err.contains("could not be resolved"),
            "the refusal names the type: {err}"
        );
    }
}

/// Nullability is the **model's** answer, so an optional behind an erased
/// wrapper renders exactly as the bare one does — everywhere, not only in the
/// positions a compiled fixture reaches.
///
/// `is_option_type` asked the spelling whether its last path segment read
/// `Option`. The model erases `Box` and `Cow`, so `Box<Option<T>>` answered
/// "not optional" and Kotlin lost its `?`. That is a wrong contract rather than
/// a cosmetic slip: a non-null parameter for an optional value makes the absent
/// case unexpressible (#273).
///
/// covertest's `boxed_note_echo` covers the parameter and return positions and
/// compiles them; this covers the **data-class field** and **callback** ones,
/// which it does not reach.
#[test]
fn nullability_ignores_how_rust_spells_the_optional() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| -> String {
        let items = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZRec {
                        pub note: #field_ty,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_rec_emit(cb: impl Fn(ZRec) + Send + Sync + 'static) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            ),
        ];
        let registry =
            crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::data_class!(ZRec))
                    .fun(prebindgen_registry::fun!(z_rec_emit)),
            );
        let dir = unique_test_dir("jnigen_nullability");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
        let _ = gen.write_rust(dir.join("g.rs")).expect("write_rust");
        gen.write_kotlin(&dir.join("kotlin"))
            .expect("write_kotlin")
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    };

    let plain = build(syn::parse_quote!(Option<String>));
    let boxed = build(syn::parse_quote!(Box<Option<String>>));

    for (label, kotlin) in [("Option<T>", &plain), ("Box<Option<T>>", &boxed)] {
        assert!(
            kotlin.contains("note: String?"),
            "{label}: an optional field is nullable in Kotlin:\n{kotlin}"
        );
    }
    // The wrapper is a Rust spelling and nothing else: the Kotlin surface of
    // the two is identical, character for character.
    assert_eq!(
        plain, boxed,
        "a transparent wrapper must not change the Kotlin surface"
    );
}

/// A value form states a recipe saying what it hands out, and it says what the
/// expansion plan it will replace says.
///
/// `expand_return!(Vault).fields(fields!(vault_to_struct))` calls the accessor
/// once and hands out the fields of the struct it returns, named as the
/// declaration names them — a `.name(..)` rename included, which is why the recipe
/// asks the declaration rather than deriving a Kotlin property a second time.
#[test]
fn a_value_form_states_what_it_hands_out() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Vault {
                    pub inner: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct VaultStruct {
                    pub seq: i64,
                    pub label: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn vault_to_struct(v: &Vault) -> VaultStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn vault_new() -> Vault {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .expand(
            prebindgen_registry::expand_return!(Vault)
                .fields(prebindgen_registry::fields!(vault_to_struct).name("label", "tag")),
        )
        .package(
            crate::package!()
                .class(crate::ptr_class!(Vault))
                .fun(prebindgen_registry::fun!(vault_new)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    assert_eq!(
        gen.out_lines_for_test("Vault")
            .expect("Vault states a value-form recipe"),
        vec!["seq: i64 <- seq @[]", "tag: String <- label @[]"],
        "the recipe hands out the value form's fields, named as declared",
    );
    assert_eq!(
        gen.out_lines_for_test("Vault"),
        gen.plan_lines_for_test("vault_new"),
        "the recipe and the expansion plan disagree",
    );
}

/// A fallible function whose error type expands binds its return exactly once.
///
/// `build_output` accepts an output plan and an error plan together and gives
/// the output one precedence, so the binding table makes that same single
/// choice in one pass — two passes would answer one site twice and be refused
/// as a rebind. This covers the error-plan half of that precedence.
///
/// It does not cover a function holding **both** plans, which I could not
/// construct: an output plan is stored only when the decomposed type matches
/// the return peeled of `&`, `Option` and `Vec`, and `Result` is not peeled
/// there, so a fallible return takes no output expansion — a type-level one
/// does not auto-apply, and a per-function one is refused with "the function's
/// return type is `Result<T, E>`, not `T`". An assertion that no function holds
/// both held across every fixture and every test (#684 review).
///
/// Nor does it claim the site plan is what the emitter reads: that is
/// `a_decomposed_return_shares_one_wire_list_with_its_site`, which asserts the
/// two hold the same allocation. It holds only where the source has a `parts`
/// row; where it has none the leaves are still compiled privately.
#[test]
fn a_fallible_return_whose_error_expands_binds_once() {
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
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZFailStruct {
                    pub reason: String,
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
                pub fn z_fail_to_struct(e: &ZFail) -> ZFailStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        // The one function with both: its `Ok` type expands on the output side
        // and its `Err` type expands on the error side.
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_one_try(n: i64) -> Result<ZOne, ZFail> {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .class(crate::ptr_class!(ZFail))
                .fun(prebindgen_registry::fun!(z_one_try)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZOne)
                .fields(prebindgen_registry::fields!(z_one_to_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZFail)
                .fields(prebindgen_registry::fields!(z_fail_to_struct)),
        )
        .build_with(registry)
        .expect("a fallible function whose error expands resolves");

    // The site the two passes would have collided on is planned, once.
    let returns = gen
        .declarations()
        .site_plans
        .borrow()
        .iter()
        .filter(|plan| {
            plan.id().site().owner == "z_one_try"
                && matches!(
                    plan.id().site().role,
                    prebindgen_registry::recipe::Role::Return
                )
        })
        .count();
    assert_eq!(returns, 1, "one return site for a fallible expanded return");
}

/// The walk enumerates the error arm of a fallible return, because a `Result`
/// has two arms whatever reads it. JniGen throws the error rather than handing
/// it back, so nothing crosses there — and it says so through `plans_site`,
/// which is the binding's own answer.
///
/// The distinction this pins is between declining and refusing. Left to be
/// attempted, the site reaches `JCompile::plan`, which has no `Role::Error` arm
/// and returns a refusal; `build_with` drops every refusal, so the position
/// would look handled while nothing had decided anything (#685 review).
#[test]
fn the_error_arm_of_a_fallible_return_is_declined_not_refused() {
    use prebindgen_registry::recipe::{Compile, Crossing, Direction, Role, Site};

    let loc = myflat_loc();
    let items = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_one_try(n: i64) -> Result<ZOne, ZFail> {
                unimplemented!()
            }
        )),
        loc,
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .class(crate::ptr_class!(ZFail))
                .fun(prebindgen_registry::fun!(z_one_try)),
        )
        .build_with(registry)
        .expect("a fallible return resolves");

    let decls = gen.declarations();
    // No plan for the position…
    assert!(
        decls
            .site_plans
            .borrow()
            .iter()
            .all(|plan| !matches!(plan.id().site().role, Role::Error)),
        "JniGen planned an error arm"
    );

    // …and the reason is the decline, not a refusal that was thrown away. The
    // return of the same function is planned, so this is not the walk skipping
    // the function altogether.
    let adapter = crate::jni::compile::JCompile {
        decls,
        declared_return: None,
    };
    let owner: syn::Ident = syn::parse_quote!(z_one_try);
    let error_ty = gen
        .registry()
        .flat()
        .function(&owner)
        .and_then(|f| f.ret.fallible_parts().map(|(_, e)| e.clone()))
        .expect("the fixture's return is fallible");
    assert!(
        !adapter.plans_site(
            &Site {
                owner: owner.clone(),
                role: Role::Error,
            },
            &Crossing::new(error_ty, Direction::Deconstruct),
        ),
        "the error arm must be declined, so the walk never asks `plan` for it"
    );
}
