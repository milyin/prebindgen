use super::*;

fn model(source: &str) -> CBindings {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap()
}

#[test]
fn cow_bytes_need_an_explicit_c_representation() {
    for (source, name) in [
        ("pub fn bytes()->Cow<'static,[u8]>{}", "bytes"),
        ("pub fn read(value:Cow<'_,[u8]>){}", "read"),
    ] {
        let result = Cbindgen::new(model(source))
            .function(DeclCFunction::new(syn::parse_str(name).unwrap()))
            .panic()
            .build();
        let error = result.err().unwrap();
        assert!(error.contains("C Cow byte"), "{error}");
        assert!(error.contains("explicit native owner"), "{error}");
    }
    Cbindgen::new(model(
        "pub type ByteCow=std::borrow::Cow<'static,[u8]>;pub fn bytes()->ByteCow{}",
    ))
    .opaque_ptr("cow", syn::parse_quote!(ByteCow))
    .function(fun!(bytes))
    .panic()
    .build()
    .unwrap();
    Cbindgen::new(model(
        "pub fn from_bytes(value:&[u8])->Cow<'_,[u8]>{}pub fn read(value:Cow<'_,[u8]>)->usize{}",
    ))
    .function(fun!(read).expand_param(
        "value",
        expand_param!(std::borrow::Cow<'_, [u8]>).variant(fun!(from_bytes)),
    ))
    .panic()
    .build()
    .unwrap();
}

#[test]
fn placeholder_reference_spelling_preserves_direct_c_loans() {
    let prepared = Cbindgen::new(model(
        "pub type Owner=String;pub fn view(owner:&'_ Owner)->&'_ Owner{}",
    ))
    .opaque_ptr("owner", syn::parse_quote!(Owner))
    .function(fun!(view))
    .panic()
    .prepare()
    .unwrap();
    let output = prepared
        .function("view")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap();
    assert!(
        crate::group::result_packet_fields(&output.outcomes.success).all(
            |wire| matches!(&wire.kind,CWire::Pointer{pointer,..} if pointer.access==Access::Shared)
        )
    );
    assert!(!output.statements.to_string().contains("Clone"));
}

#[test]
fn cow_constructor_temporaries_cannot_back_escaping_c_loans() {
    Cbindgen::new(model(
        "pub fn static_cow()->Cow<'static,[u8]>{}pub fn loan(value:Cow<'static,[u8]>)->&[u8]{}",
    ))
    .function(fun!(loan).expand_param(
        "value",
        expand_param!(std::borrow::Cow<'static, [u8]>).variant(fun!(static_cow)),
    ))
    .panic()
    .build()
    .unwrap();
    let source="pub fn borrow(value:&Vec<u8>)->Cow<'_,[u8]>{}pub fn loan(value:Cow<'_,[u8]>)->&[u8]{}pub fn fixed(value:Cow<'_,[u8]>)->&'static [u8]{}";
    let input = || {
        expand_param!(std::borrow::Cow<'_, [u8]>)
            .variant(fun!(borrow).expand_param("value", expand_param!(&Vec<u8>).reconstruct()))
    };
    let result = Cbindgen::new(model(source))
        .function(fun!(loan).expand_param("value", input()))
        .panic()
        .build();
    let error = result.err().unwrap();
    assert!(
        error.contains("Cow input `value`") && error.contains("backing storage may expire"),
        "{error}"
    );
    Cbindgen::new(model(source))
        .function(fun!(fixed).expand_param("value", input()))
        .panic()
        .build()
        .unwrap();
    Cbindgen::new(model(source))
        .function(
            fun!(loan)
                .expand_param("value", input())
                .expand_return(expand_return!(&[u8]).field_self_cloned()),
        )
        .panic()
        .build()
        .unwrap();
}
