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
