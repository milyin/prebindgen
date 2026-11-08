fn c_bindings(out: &std::path::Path) {
    for checked in [true,false] {
        let generated=prebindgen_c::Cbindgen::builder()
            .source(type_group_flat::PREBINDGEN_OUT_DIR).prefix("buckets")
            .set_emit_alias_checks(checked)
            .opaque_ptr("a",syn::parse_quote!(A)).function(fun!(a_new)).panic()
            .opaque_ptr("b",syn::parse_quote!(B)).function(fun!(b_new)).panic()
            .opaque_ptr("c",syn::parse_quote!(C)).function(fun!(c_new)).panic()
            .opaque_ptr("d",syn::parse_quote!(D)).function(fun!(d_new)).panic()
            .opaque_ptr("token",syn::parse_quote!(Token)).function(fun!(token_new)).panic()
            .opaque_ptr("owner",syn::parse_quote!(Owner)).function(fun!(owner_new)).panic()
            .opaque_ptr("child",syn::parse_quote!(Child)).function(fun!(owner_child)).panic()
            .function(fun!(overlap)).panic()
            .function(fun!(control)).panic().function(fun!(live)).panic()
            .function(fun!(mut_one).expand_param("values",expand_param!(i64).variant(fun!(a_mut)))).panic()
            .function(fun!(mixed_mut)
                .expand_param("a",expand_param!(i64).variant(fun!(a_mut)))
                .expand_param("b",expand_param!(i64).variant(fun!(b_mut)))
                .expand_param("c",expand_param!(i64).variant(fun!(c_mut)))
                .expand_param("d",expand_param!(i64).variant(fun!(d_mut)))).panic()
            .function(fun!(mixed_shared)
                .expand_param("a",expand_param!(i64).variant(fun!(a_read)))
                .expand_param("b",expand_param!(i64).variant(fun!(b_read)))
                .expand_param("c",expand_param!(i64).variant(fun!(c_read)))
                .expand_param("d",expand_param!(i64).variant(fun!(d_read)))).panic()
            .build().unwrap();
        generated.write_rust(out.join(if checked {"c_checked.rs"} else {"c_unchecked.rs"})).unwrap();
        if checked { generated.write_header(out.join("bindings.h")).unwrap(); }
    }
}
