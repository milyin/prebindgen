use super::*;

/// Lifetimes and const-generic args are fixed pattern structure — they must
/// match token-for-token, not be silently dropped (restores the old
/// enumerator's exact `TypeKey` semantics).
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

#[test]
fn pascal_to_snake_basics() {
    assert_eq!(pascal_to_snake("ZKeyExpr"), "z_key_expr");
    assert_eq!(pascal_to_snake("PeriodicQueries"), "periodic_queries");
    assert_eq!(pascal_to_snake("already_snake"), "already_snake");
    assert_eq!(pascal_to_snake("A"), "a");
    assert_eq!(pascal_to_snake(""), "");
}
