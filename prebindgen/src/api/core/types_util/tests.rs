use super::*;

/// Lifetimes and const-generic args are fixed pattern structure — they must
/// match token-for-token, not be silently dropped (restores the old
/// enumerator's exact `TypeKey` semantics).
// ── Enum shape / sum model ─────────────────────────────────────────────

#[test]
fn enum_shape_classifies_unit_and_payload() {
    let unit: syn::ItemEnum = syn::parse_quote! { enum E { A, B = 7, C } };
    assert_eq!(enum_shape(&unit), EnumShape::Unit);
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
    }
}

#[test]
fn pascal_to_snake_basics() {
    assert_eq!(pascal_to_snake("ZKeyExpr"), "z_key_expr");
    assert_eq!(pascal_to_snake("PeriodicQueries"), "periodic_queries");
    assert_eq!(pascal_to_snake("already_snake"), "already_snake");
    assert_eq!(pascal_to_snake("A"), "a");
    assert_eq!(pascal_to_snake(""), "");
}
