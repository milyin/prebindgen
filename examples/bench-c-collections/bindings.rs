use prebindgen_declarations::{expand_param, expand_return, fields, fun, Output, OutputRequest, SequenceDelivery};
fn rows(element: Output<OutputRequest>) -> Output<OutputRequest> {
    Output::Sequence { element: Box::new(element), mode: None, delivery: SequenceDelivery::Value }
}
fn main() {
    let generation = prebindgen_c::Cbindgen::builder()
        .source(c_collection_flat::PREBINDGEN_OUT_DIR).prefix("bench")
        .opaque_ptr("resource", syn::parse_quote!(Resource))
        .expand(expand_return!(Record).fields_into(fields!(record_into)))
        .expand(expand_param!(Record).variant(fun!(record_new)))
        .expand(expand_param!(WideInput).variant(fun!(wide_new)))
        .expand(expand_param!(TextInput).variant(fun!(text_new)))
        .function(fun!(resource_new)).panic()
        .function(fun!(take_handles)).panic()
        .function(fun!(take_records)).panic()
        .function(fun!(take_wide)).panic()
        .function(fun!(take_text)).panic()
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
