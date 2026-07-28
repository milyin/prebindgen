use quote::ToTokens;

use super::*;

fn ty(s: &str) -> syn::Type {
    syn::parse_str(s).unwrap()
}
fn caps(v: Option<Vec<syn::Type>>) -> Option<Vec<String>> {
    v.map(|a| a.iter().map(|t| t.to_token_stream().to_string()).collect())
}

#[test]
fn match_pattern_outermost_and_deep() {
    // Outermost single wildcard.
    assert_eq!(
        caps(match_pattern(&ty("Option<u64>"), &ty("Option<_>"))),
        Some(vec!["u64".to_string()])
    );
    // Two wildcards (Result).
    assert_eq!(
        caps(match_pattern(
            &ty("Result<ZKeyExpr, ZError>"),
            &ty("Result<_, _>")
        )),
        Some(vec!["ZKeyExpr".to_string(), "ZError".to_string()])
    );
    // Deep single wildcard, intermediate level concrete (`Option<&_>`).
    assert_eq!(
        caps(match_pattern(&ty("Option<&ZKeyExpr>"), &ty("Option<&_>"))),
        Some(vec!["ZKeyExpr".to_string()])
    );
    // The shallow pattern also matches, capturing the reference whole.
    assert_eq!(
        caps(match_pattern(&ty("Option<&ZKeyExpr>"), &ty("Option<_>"))),
        Some(vec!["& ZKeyExpr".to_string()])
    );
    // `&mut _` vs `&_` mutability must agree.
    assert!(match_pattern(&ty("&mut Foo"), &ty("&_")).is_none());
    assert_eq!(
        caps(match_pattern(&ty("&mut Foo"), &ty("&mut _"))),
        Some(vec!["Foo".to_string()])
    );
    // Slice element.
    assert_eq!(
        caps(match_pattern(&ty("&[u8]"), &ty("&[_]"))),
        Some(vec!["u8".to_string()])
    );
    // Arbitrary depth (the framework never enumerated this, but a user
    // pattern can name it).
    assert_eq!(
        caps(match_pattern(
            &ty("Vec<Option<u64>>"),
            &ty("Vec<Option<_>>")
        )),
        Some(vec!["u64".to_string()])
    );
    // Head mismatch.
    assert!(match_pattern(&ty("Vec<u64>"), &ty("Option<_>")).is_none());
    // Concrete non-wildcard pattern: matches only itself, no captures.
    assert_eq!(
        caps(match_pattern(&ty("MyType"), &ty("MyType"))),
        Some(vec![])
    );
    assert!(match_pattern(&ty("Other"), &ty("MyType")).is_none());
}

/// Lifetimes and const-generic args are fixed pattern structure — they must
/// match token-for-token, not be silently dropped (restores the old
/// enumerator's exact `TypeKey` semantics).
#[test]
fn match_pattern_respects_lifetimes_and_const_generics() {
    // Reference lifetimes must match exactly.
    assert_eq!(
        caps(match_pattern(&ty("&'static Foo"), &ty("&'static _"))),
        Some(vec!["Foo".to_string()])
    );
    assert!(match_pattern(&ty("&'a Foo"), &ty("&'static _")).is_none());
    // A no-lifetime pattern must not match a borrow that names a lifetime.
    assert!(match_pattern(&ty("&'a Foo"), &ty("&_")).is_none());
    assert_eq!(
        caps(match_pattern(&ty("&Foo"), &ty("&_"))),
        Some(vec!["Foo".to_string()])
    );
    // A lifetime generic arg in a path is fixed structure.
    assert_eq!(
        caps(match_pattern(
            &ty("Cow<'static, _>"),
            &ty("Cow<'static, _>")
        )),
        Some(vec!["_".to_string()])
    );
    assert!(match_pattern(&ty("Cow<'a, str>"), &ty("Cow<'static, _>")).is_none());
    // Const-generic arg is fixed structure: arity must match exactly.
    assert_eq!(
        caps(match_pattern(&ty("Arr<u8, 4>"), &ty("Arr<_, 4>"))),
        Some(vec!["u8".to_string()])
    );
    assert!(match_pattern(&ty("Arr<u8, 8>"), &ty("Arr<_, 4>")).is_none());
    // Array length is fixed structure.
    assert!(match_pattern(&ty("[u8; 8]"), &ty("[_; 4]")).is_none());
    assert_eq!(
        caps(match_pattern(&ty("[u8; 4]"), &ty("[_; 4]"))),
        Some(vec!["u8".to_string()])
    );
}

#[test]
fn wildcard_count_specificity() {
    assert_eq!(wildcard_count(&ty("Result<_, _>")), 2);
    assert_eq!(wildcard_count(&ty("Result<_, ConcreteErr>")), 1);
    assert_eq!(wildcard_count(&ty("Option<&_>")), 1);
    assert_eq!(wildcard_count(&ty("ZKeyExpr")), 0);
}

// ── Enum shape / sum model ─────────────────────────────────────────────

#[test]
fn enum_shape_classifies_unit_and_payload() {
    let unit: syn::ItemEnum = syn::parse_quote! { enum E { A, B = 7, C } };
    assert_eq!(enum_shape(&unit), EnumShape::Unit);
    assert!(first_payload_variant(&unit).is_none());
    // An empty enum has no payload variant, so it is the degenerate unit
    // case — not a sum.
    let empty: syn::ItemEnum = syn::parse_quote! { enum E {} };
    assert_eq!(enum_shape(&empty), EnumShape::Unit);

    // One payload variant is enough, whatever the shape of the others.
    for src in [
        quote::quote! { enum E { A(u32), B } },
        quote::quote! { enum E { A, B { x: u32 } } },
        // An empty tuple/brace variant still carries fields syntactically
        // (`Fields::Unnamed`), so it classifies as a sum — the declarator
        // that accepts it is the sum one.
        quote::quote! { enum E { A, B() } },
    ] {
        let e: syn::ItemEnum = syn::parse_quote!(#src);
        assert_eq!(enum_shape(&e), EnumShape::Sum, "for `{src}`");
        assert!(first_payload_variant(&e).is_some(), "for `{src}`");
    }
}

/// The offending variant an adapter names in its rejection message is the
/// **first** payload one, in declaration order.
#[test]
fn first_payload_variant_is_the_first_in_declaration_order() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A, B(u32), C { x: u8 } } };
    assert_eq!(
        first_payload_variant(&e).expect("payload variant").ident,
        "B"
    );
}

/// Tags are declaration order `0..N-1` and never an explicit discriminant —
/// a payload enum carries no `repr`, so a discriminant is a wire detail the
/// neutral tier must not name.
#[test]
fn sum_spec_tags_are_declaration_order() {
    let e: syn::ItemEnum = syn::parse_quote! {
        enum E { A(u32), B, C { x: u8 } }
    };
    let spec = SumSpec::from_item_enum(&e);
    assert_eq!(spec.source, "E");
    assert_eq!(spec.key.as_str(), "E");
    assert_eq!(
        spec.variants
            .iter()
            .map(|v| (v.ident.to_string(), v.tag))
            .collect::<Vec<_>>(),
        vec![
            ("A".to_string(), 0),
            ("B".to_string(), 1),
            ("C".to_string(), 2)
        ]
    );
    // A unit variant is the empty group.
    assert!(spec.variants[1].is_unit());
    assert!(!spec.variants[0].is_unit());
}

/// Explicit discriminants on a payload enum do not move the tags — those
/// are two independent numberings, and only `enum_discriminant_values`
/// reads the discriminant.
#[test]
fn sum_spec_tags_ignore_explicit_discriminants() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A = 5, B = 9 } };
    let spec = SumSpec::from_item_enum(&e);
    assert_eq!(
        spec.variants.iter().map(|v| v.tag).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        enum_discriminant_values(&e)
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>(),
        vec![5, 9]
    );
}

/// Leaf names follow the existing nested-prefix convention:
/// `<variant_snake>_<field>`, tuple fields `<variant_snake>_<i>`.
#[test]
fn sum_spec_leaf_names_and_members() {
    let e: syn::ItemEnum = syn::parse_quote! {
        enum RecoveryMode {
            PeriodicQueries(Duration),
            Heartbeat,
            Windowed { size: u32, ratio: f64 },
            Pair(u8, u8),
        }
    };
    let spec = SumSpec::from_item_enum(&e);

    let names: Vec<Vec<String>> = spec
        .variants
        .iter()
        .map(|v| v.fields.iter().map(|f| f.name.clone()).collect())
        .collect();
    assert_eq!(
        names,
        vec![
            vec!["periodic_queries_0".to_string()],
            vec![],
            vec!["windowed_size".to_string(), "windowed_ratio".to_string()],
            vec!["pair_0".to_string(), "pair_1".to_string()],
        ]
    );

    // Members address the field in a pattern: named by ident, tuple by index.
    let named = &spec.variants[2].fields[0].member;
    assert!(matches!(named, syn::Member::Named(id) if id == "size"));
    let unnamed = &spec.variants[3].fields[1].member;
    assert!(matches!(unnamed, syn::Member::Unnamed(i) if i.index == 1));

    // Payload types survive verbatim.
    assert_eq!(
        spec.variants[0].fields[0].ty.to_token_stream().to_string(),
        "Duration"
    );
}

/// A unit-only enum is the degenerate sum: every group is empty, so the
/// lowering collapses to "a tag". That is why existing enums are
/// unaffected by the sum machinery.
#[test]
fn sum_spec_of_unit_enum_is_all_empty_groups() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A, B, C } };
    let spec = SumSpec::from_item_enum(&e);
    assert_eq!(spec.variants.len(), 3);
    assert!(spec.variants.iter().all(|v| v.is_unit()));
}

#[test]
fn sum_spec_single_variant() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { Only(String) } };
    let spec = SumSpec::from_item_enum(&e);
    assert_eq!(spec.variants.len(), 1);
    assert_eq!(spec.variants[0].tag, 0);
    assert_eq!(spec.variants[0].fields[0].name, "only_0");
}

#[test]
fn pascal_to_snake_basics() {
    assert_eq!(pascal_to_snake("ZKeyExpr"), "z_key_expr");
    assert_eq!(pascal_to_snake("PeriodicQueries"), "periodic_queries");
    assert_eq!(pascal_to_snake("already_snake"), "already_snake");
    assert_eq!(pascal_to_snake("A"), "a");
    assert_eq!(pascal_to_snake(""), "");
}

// ── Discriminants (moved here from the jnigen adapter) ─────────────────

fn discriminants(e: syn::ItemEnum) -> Vec<(String, i64)> {
    enum_discriminant_values(&e)
        .into_iter()
        .map(|(ident, value)| (ident.to_string(), value))
        .collect()
}

#[test]
fn discriminants_no_explicit_values() {
    // Implicit C-like enum: 0, 1, 2 — matches Rust's default repr,
    // which is also what the `as jint` output cast produces.
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A, B, C } };
    assert_eq!(
        discriminants(e),
        vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)]
    );
}

#[test]
fn discriminants_all_explicit() {
    let e: syn::ItemEnum = syn::parse_quote! {
        enum E { A = 1, B = 2, C = 7 }
    };
    assert_eq!(
        discriminants(e),
        vec![("A".into(), 1), ("B".into(), 2), ("C".into(), 7)]
    );
}

#[test]
fn discriminants_mixed_follow_rust_rule() {
    // Explicit sets the value; the next implicit variant is prev + 1.
    let e: syn::ItemEnum = syn::parse_quote! {
        enum E { A = 5, B, C = 1, D }
    };
    assert_eq!(
        discriminants(e),
        vec![
            ("A".into(), 5),
            ("B".into(), 6),
            ("C".into(), 1),
            ("D".into(), 2),
        ]
    );
}

#[test]
fn discriminants_negative_literal() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A = -1, B } };
    assert_eq!(discriminants(e), vec![("A".into(), -1), ("B".into(), 0)]);
}

#[test]
#[should_panic(expected = "non-literal discriminant")]
fn discriminants_non_literal_rejected() {
    let e: syn::ItemEnum = syn::parse_quote! {
        enum E { A = OTHER, B }
    };
    let _ = discriminants(e);
}

/// Turbofish and a trailing generic comma are punctuation, not identity, so
/// they collapse before any `TypeKey` is formed.
///
/// This has to happen HERE rather than in a consumer: a key's identity is the
/// normalized token string, so otherwise one type has two keys — and during the
/// frontend migration they split by POSITION, a modeled struct field getting
/// one and a function signature the other.
#[test]
fn normalize_drops_turbofish_and_trailing_comma() {
    let canon = |s: &str| {
        let mut t = ty(s);
        normalize_type(&mut t, &[]);
        t.to_token_stream().to_string()
    };
    let want = canon("Wrapper<u8>");
    assert_eq!(canon("Wrapper::<u8>"), want);
    assert_eq!(canon("Wrapper<u8,>"), want);
    assert_eq!(canon("Wrapper::<u8,>"), want);
    // Nested, and through the peelers a field type actually takes.
    assert_eq!(
        canon("Option<Wrapper::<u8,>>"),
        canon("Option<Wrapper<u8>>")
    );
    assert_eq!(canon("Vec::<u8,>"), canon("Vec<u8>"));
    // A lifetime argument is identity and survives — only punctuation goes.
    assert_eq!(canon("Foo::<'static,>"), canon("Foo<'static>"));
    assert_ne!(canon("Foo<'static>"), canon("Foo"));

    // `Fn(..)` is a second punctuation list with the same problem, plus a
    // return that may spell unit explicitly and bounds whose order is
    // irrelevant. One callback, four spellings, one key.
    let cb = canon("impl Fn(u8) + Send + Sync + 'static");
    assert_eq!(canon("impl Fn(u8,) + Send + Sync + 'static"), cb);
    assert_eq!(canon("impl Fn(u8) -> () + Send + Sync + 'static"), cb);
    assert_eq!(canon("impl Fn(u8) + Sync + Send + 'static"), cb);
    assert_eq!(canon("impl Fn(u8,) -> () + Sync + Send + 'static"), cb);
    // A lifetime bound sorts last, so the canonical form stays valid Rust.
    assert!(cb.starts_with("impl Fn"), "{cb}");
    assert!(cb.ends_with("'static"), "{cb}");
    // A non-unit return is NOT punctuation and must survive to be refused.
    assert_ne!(canon("impl Fn(u8) -> u16 + Send + Sync + 'static"), cb);
    // A parenthesized unit return reaches the canonical form in ONE pass: the
    // recursion that unwraps `Paren` has to run BEFORE the unit test, or this
    // needs a second pass and the key depends on how many it has had.
    assert_eq!(canon("impl Fn(u8) -> (()) + Send + Sync + 'static"), cb);
}

/// Normalization is idempotent: a canonical form is its own canonical form.
///
/// Asserted over every spelling the suite exercises rather than for one case,
/// because the way this breaks is a rule that reads a node the recursion has
/// not reached yet, and that mistake is not specific to any one rule. It
/// matters because items are normalized at ingest and `TypeKey::from_type`
/// normalizes again, while a directly built key normalizes once.
#[test]
fn normalize_is_idempotent() {
    for src in [
        "Wrapper::<u8>",
        "Wrapper<u8,>",
        "Option<Wrapper::<u8,>>",
        "Foo::<'static,>",
        "impl Fn(u8,) -> () + Sync + Send + 'static",
        "impl Fn(u8) -> (()) + Send + Sync + 'static",
        "impl Fn(u8) -> u16 + Send + Sync + 'static",
        "crate::a::Foo<T>",
        "std::option::Option<u8>",
        "&'static [u8]",
        "[u8; 4]",
    ] {
        let mut once = ty(src);
        normalize_type(&mut once, &[]);
        let mut twice = once.clone();
        normalize_type(&mut twice, &[]);
        assert_eq!(
            once.to_token_stream().to_string(),
            twice.to_token_stream().to_string(),
            "`{src}` is not idempotent"
        );
    }
}
