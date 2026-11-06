use prebindgen_declarations::{expand_param, expand_return, fields, fun, Output, OutputRequest, SequenceDelivery};
fn rows(element: Output<OutputRequest>) -> Output<OutputRequest> {
    Output::Sequence { element: Box::new(element), mode: None, delivery: SequenceDelivery::Value }
}
fn main() {
    println!("cargo:rerun-if-env-changed=BENCH_ALIAS_CHECKS");
    let generation = prebindgen_c::Cbindgen::builder()
        .source(c_collection_flat::PREBINDGEN_OUT_DIR).prefix("bench")
        .set_emit_alias_checks(std::env::var("BENCH_ALIAS_CHECKS").as_deref()!=Ok("0"))
        .opaque_ptr("resource", syn::parse_quote!(Resource))
        .enum_type("input_tag", syn::parse_quote!(InputTag))
        .expand(expand_param!(TaggedInput).variant(fun!(tagged_new)))
        .expand(expand_param!(TaggedBuffersInput).variant(fun!(tagged_buffers_new)))
        .expand(expand_return!(Record).fields_into(fields!(record_into)))
        .expand(expand_param!(Record).variant(fun!(record_new)))
        .expand(expand_param!(WideInput).variant(fun!(wide_new)))
        .expand(expand_param!(TextInput).variant(fun!(text_new)))
        .expand(expand_param!(BytesInput).variant(fun!(bytes_new)))
        .expand(expand_param!(BuffersInput).variant(fun!(buffers_new)))
        .function(fun!(resource_new)).panic()
        .function(fun!(fixed_borrow)).panic()
        .function(fun!(fixed_move)).panic()
        .function(fun!(callback_fixed)).panic()
        .function(fun!(callback_many)).panic()
        .function(fun!(take_handles)).panic()
        .function(fun!(take_records)).panic()
        .function(fun!(take_wide)).panic()
        .function(fun!(take_text)).panic()
        .function(fun!(take_bytes)).panic()
        .function(fun!(take_buffers)).panic()
        .function(fun!(take_tagged)).panic()
        .function(fun!(take_tagged_buffers)).panic()
        .function(fun!(take_numbers)).panic()
        .function(fun!(control)).panic()
        .function(fun!(resource_value)).panic()
        .function(fun!(make_handles).output(rows(Output::auto()))).panic()
        .function(fun!(make_records).output(rows(Output::auto()))).panic()
        .function(fun!(make_nested).output(rows(rows(Output::auto())))).panic()
        .function(fun!(make_many).output(rows(rows(Output::auto())))).panic()
        .function(fun!(make_numbers).output(rows(Output::auto()))).panic()
        .build().expect("C collection benchmark bindings");
    let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    generation.write_rust(out.join("bindings.rs")).unwrap();
    generation.write_header(out.join("bindings.h")).unwrap();
}
