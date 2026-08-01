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

/// Build the fixture through `JniGenBuilder`, letting the caller adjust the
/// `ZSample` boundary decl. Returns the generated Rust + the joined Kotlin.
fn value_form_gen(tag: &str, decl: crate::lang::ExpandReturnDecl) -> (String, String) {
    let registry = crate::api::test_util::reg_from_items(declare_referenced(value_form_items()))
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
                .fun(crate::fun!(z_sample_sub)),
        )
        // A KeyExpr crosses as its string, never as a handle — the rule a
        // `.fields()` expansion has to keep honouring for the `key_expr` field.
        .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
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
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(all)).expect("index items");
        let jni = JniGenBuilder::new()
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
        let gen = jni.build_with(registry).expect("resolve");
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

/// The complete-set rule taken to its limit: an override with NO records is an
/// empty leaf set, so the field does not cross at all. That is how a binding
/// drops a field its source's value form carries but its surface has no use
/// for — without it, adopting a value form is all-or-nothing.
#[test]
fn an_empty_per_field_override_drops_the_field() {
    let (_, kotlin) = value_form_gen(
        "jnigen_vf_drop",
        crate::expand_return!(ZSample).fields(
            crate::fields!(z_sample_to_struct).field("key_expr", crate::expand_return!(ZKeyExpr)),
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
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
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
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
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
        let _ = jni
            .build_with(registry)
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
            crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZSample))
                    .class(crate::ptr_class!(ZKeyExpr))
                    .fun(crate::fun!(z_sample_sub)),
            )
            .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
            .expand(crate::expand_return!(ZSample).fields(crate::fields!(z_sample_to_struct)));
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

    // The wrapper changes the Rust spelling and NOTHING that crosses. Compared
    // after normalizing the one legitimate difference — the field's own type is
    // spelled in the generated converter signatures.
    assert_eq!(
        plain.replace("Box<Option<ZKeyExpr>>", "Option<ZKeyExpr>"),
        boxed.replace("Box<Option<ZKeyExpr>>", "Option<ZKeyExpr>"),
        "a transparent wrapper must not change what crosses the boundary"
    );
}

/// Naming a field the value form does not have is the very drift this
/// declarator exists to catch, so it is an error rather than a silent no-op.
#[test]
fn an_adjustment_naming_an_unknown_field_is_an_error() {
    let build = |decl: crate::lang::FieldsDecl| {
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(value_form_items()))
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
                    .fun(crate::fun!(z_sample_sub)),
            )
            .expand(crate::expand_return!(ZKeyExpr).field(crate::fun!(z_keyexpr_as_str)))
            .expand(crate::expand_return!(ZSample).fields(decl));
        let dir = unique_test_dir("jnigen_vf_unknown");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = jni
            .build_with(registry)
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
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .fun(crate::fun!(z_one_make)),
        )
        .expand(crate::expand_return!(ZOne).fields_self_into(crate::fields!(z_one_into_struct)));
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(crate::fun!(z_envelope_sub)),
        )
        .expand(crate::expand_return!(ZChild).field_self())
        .expand(
            crate::expand_return!(ZEnvelope)
                .fields_self_into(crate::fields!(z_envelope_into_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZSingleEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(crate::fun!(z_single_envelope_make)),
        )
        .expand(crate::expand_return!(ZChild).field_self())
        .expand(
            crate::expand_return!(ZSingleEnvelope)
                .fields_self_into(crate::fields!(z_single_envelope_into_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOptionalEnvelope))
                .class(crate::ptr_class!(ZChild))
                .fun(crate::fun!(z_optional_envelope_sub)),
        )
        .expand(crate::expand_return!(ZChild).field_self())
        .expand(
            crate::expand_return!(ZOptionalEnvelope)
                .fields_self_into(crate::fields!(z_optional_envelope_into_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOptionalSingle))
                .class(crate::ptr_class!(ZChild))
                .fun(crate::fun!(z_optional_single_make)),
        )
        .expand(crate::expand_return!(ZChild).field_self())
        .expand(
            crate::expand_return!(ZOptionalSingle)
                .fields_self_into(crate::fields!(z_optional_single_into_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZChild))
                .fun(crate::fun!(z_root_child_make))
                .fun(crate::fun!(z_root_child_maybe)),
        )
        .expand(crate::expand_return!(ZChild).field_self());
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
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(value_form_items()))
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
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
            crate::expand_return!(ZOuter).fields_self_into(crate::fields!(z_outer_into_struct)),
        ),
    ] {
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(items(outer_by_value)))
                .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZOuter))
                    .class(crate::ptr_class!(ZInner))
                    .fun(crate::fun!(z_outer_sub)),
            )
            .expand(
                crate::expand_return!(ZInner).fields_self_into(crate::fields!(z_inner_into_struct)),
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

fn nested_review_jni(outer: crate::lang::ExpandReturnDecl) -> JniGenBuilder {
    JniGenBuilder::new()
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

/// A value form below an `Option` is a CONDITIONAL hoist — supported at one
/// level (see [`a_value_form_under_an_optional_accessor_is_hoisted_conditionally`]),
/// but it cannot NEST: the inner form would have to be bound inside the outer
/// one's `Some` arm, and the binder has no arm to put a second local in.
/// Reject it during planning rather than emit a hoist that reaches through an
/// `Option` it cannot unwrap.
#[test]
fn an_optional_nested_value_form_is_rejected_before_emission() {
    let registry = crate::api::test_util::reg_from_items(declare_referenced(nested_review_items()))
        .expect("index items");
    let jni = nested_review_jni(
        crate::expand_return!(ZReviewOuter).fields(crate::fields!(z_review_outer_to_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)))
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_get_carrier)));
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
fn conditional_owned_gen(tag: &str, decl: crate::lang::ExpandReturnDecl) -> String {
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(decl)
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_take_carrier)));
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZChild))
                .class(crate::ptr_class!(ZHolder))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields(crate::fields!(zc_to_struct)))
        .expand(crate::expand_return!(ZChild).field(crate::fun!(zchild_carrier)))
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_child)));
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZWrapper))
                .class(crate::ptr_class!(ZOuter))
                .fun(crate::fun!(zo_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)))
        .expand(crate::expand_return!(ZWrapper).field(crate::fun!(zw_carrier)))
        .expand(crate::expand_return!(ZOuter).fields_self_into(crate::fields!(zo_into_struct)));
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)))
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_carrier)));
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZMiddle))
                .class(crate::ptr_class!(ZChild))
                .class(crate::ptr_class!(ZHolder))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields(crate::fields!(zc_to_struct)))
        .expand(crate::expand_return!(ZMiddle).field(crate::fun!(zmiddle_carrier)))
        .expand(crate::expand_return!(ZChild).field(crate::fun!(zchild_middle)))
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_child)));
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
        crate::expand_return!(ZCarrier).fields(crate::fields!(zc_to_struct)),
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
        crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)),
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZHolder))
                .class(crate::sealed_class!(ZOutcome))
                .fun(crate::fun!(zh_sub)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)))
        .expand(crate::expand_return!(ZHolder).field(crate::fun!(zh_get_carrier)));
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
        let registry =
            crate::api::test_util::reg_from_items(declare_referenced(nested_review_items()))
                .expect("index items");
        let jni = nested_review_jni(
            crate::expand_return!(ZReviewOuter).fields(
                crate::fields!(z_review_outer_to_struct)
                    .field("items", crate::expand_return!(ZReviewInner).field_self()),
            ),
        );
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
    let registry = crate::api::test_util::reg_from_items(declare_referenced(consuming_items()))
        .expect("index items");
    let jni = JniGenBuilder::new()
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
        crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)),
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
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZCarrier))
                .class(crate::ptr_class!(ZVault))
                .fun(crate::fun!(zc_sub))
                .fun(crate::fun!(zc_borrowed)),
        )
        .expand(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_into_struct)));
    let dir = unique_test_dir("jnigen_vf_consume_ref");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
    let _ = crate::expand_return!(ZCarrier)
        .fields_self_into(crate::fields!(zc_into_struct))
        .field_self();
}

/// And the other way round — the decl is a builder, so both orders must be
/// caught or the rule holds only for the order someone happened to write.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_preceding_sibling() {
    let _ = crate::expand_return!(ZCarrier)
        .field_self()
        .fields_self_into(crate::fields!(zc_into_struct));
}

/// Any sibling record, not just the identity one.
#[test]
#[should_panic(expected = "only record")]
fn a_consuming_value_form_rejects_a_plain_field_sibling() {
    let _ = crate::expand_return!(ZCarrier)
        .fields_self_into(crate::fields!(zc_into_struct))
        .field(crate::fun!(zc_to_struct));
}

/// The declarator states whether the value is given away and the accessor's
/// signature has to agree — otherwise the emitted call would not compile in the
/// consumer's crate, and a boundary would silently stop being the one declared.
/// Both directions are errors; the fixture has one accessor of each kind.
#[test]
fn the_declarator_and_the_accessor_s_receiver_must_agree() {
    let build = |decl: crate::lang::ExpandReturnDecl| -> String {
        let registry = crate::api::test_util::reg_from_items(declare_referenced(consuming_items()))
            .expect("index");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZCarrier))
                    .fun(crate::fun!(zc_sub)),
            )
            .expand(decl);
        match jni.build_with(registry) {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        }
    };

    let msg = build(crate::expand_return!(ZCarrier).fields_self_into(crate::fields!(zc_to_struct)));
    assert!(
        msg.contains("CONSUMING") && msg.contains("zc_to_struct"),
        "`.fields_self_into` on a borrowing accessor must be refused, naming it: {msg:?}"
    );

    let msg = build(crate::expand_return!(ZCarrier).fields(crate::fields!(zc_into_struct)));
    assert!(
        msg.contains("BORROWING") && msg.contains("zc_into_struct"),
        "`.fields` on a by-value accessor must be refused, naming it: {msg:?}"
    );
}
