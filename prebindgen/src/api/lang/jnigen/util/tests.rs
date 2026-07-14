use super::{camel_to_screaming_snake, enum_discriminant_values};

#[test]
fn camel_to_screaming_snake_basics() {
    assert_eq!(camel_to_screaming_snake("RealTime"), "REAL_TIME");
    assert_eq!(
        camel_to_screaming_snake("InteractiveHigh"),
        "INTERACTIVE_HIGH"
    );
    assert_eq!(camel_to_screaming_snake("Data"), "DATA");
    assert_eq!(camel_to_screaming_snake("Background"), "BACKGROUND");
}

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
#[should_panic(expected = "non-literal discriminant")]
fn discriminants_non_literal_rejected() {
    let e: syn::ItemEnum = syn::parse_quote! {
        enum E { A = OTHER, B }
    };
    let _ = discriminants(e);
}

#[test]
fn strip_type_prefix_basics() {
    use super::strip_type_prefix;
    // Plain one-segment class namespace.
    assert_eq!(strip_type_prefix("storage_len", "Storage"), Some("len"));
    assert_eq!(
        strip_type_prefix("payload_label_len", "Payload"),
        Some("label_len")
    );
    // Underscore-insensitive on both sides: class KeyExpr vs `keyexpr_` and
    // `key_expr_` idents.
    assert_eq!(
        strip_type_prefix("keyexpr_get_str", "KeyExpr"),
        Some("get_str")
    );
    assert_eq!(
        strip_type_prefix("key_expr_get_str", "KeyExpr"),
        Some("get_str")
    );
    assert_eq!(
        strip_type_prefix("zbytes_as_bytes", "ZBytes"),
        Some("as_bytes")
    );
    // No namespace prefix → None (fallback keeps the full ident).
    assert_eq!(strip_type_prefix("millis_add", "Storage"), None);
    // Partial overlap that never completes the class name → None.
    assert_eq!(strip_type_prefix("key_something", "KeyExpr"), None);
    // The ident IS the class name alone → None (empty remainder).
    assert_eq!(strip_type_prefix("storage", "Storage"), None);
    assert_eq!(strip_type_prefix("storage_", "Storage"), None);
}

#[test]
fn doc_string_extracts_and_sanitizes() {
    use super::doc_string;
    let f: syn::ItemFn = syn::parse_quote! {
        /// Puts a payload.
        ///
        /// Second paragraph with */ inside.
        fn f() {}
    };
    let doc = doc_string(&f.attrs).expect("docs present");
    assert_eq!(
        doc,
        "Puts a payload.\n\nSecond paragraph with *\u{200B}/ inside."
    );
    let bare: syn::ItemFn = syn::parse_quote!(
        fn g() {}
    );
    assert_eq!(doc_string(&bare.attrs), None);
}
