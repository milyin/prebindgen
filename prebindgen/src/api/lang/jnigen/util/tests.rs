use super::camel_to_screaming_snake;

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
