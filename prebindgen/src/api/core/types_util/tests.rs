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

#[test]
fn pascal_to_snake_basics() {
    assert_eq!(pascal_to_snake("ZKeyExpr"), "z_key_expr");
    assert_eq!(pascal_to_snake("PeriodicQueries"), "periodic_queries");
    assert_eq!(pascal_to_snake("already_snake"), "already_snake");
    assert_eq!(pascal_to_snake("A"), "a");
    assert_eq!(pascal_to_snake(""), "");
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
