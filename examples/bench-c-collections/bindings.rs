use prebindgen_declarations::{expand_return, fields, fun, Output, OutputRequest, SequenceDelivery};
fn rows(element: Output<OutputRequest>) -> Output<OutputRequest> {
    Output::Sequence { element: Box::new(element), mode: None, delivery: SequenceDelivery::Value }
}
fn main() {
    let generation = prebindgen_c::Cbindgen::builder()
        .source(c_collection_flat::PREBINDGEN_OUT_DIR).prefix("bench")
        .opaque_ptr("resource", syn::parse_quote!(Resource))
        .expand(expand_return!(Record).fields_into(fields!(record_into)))
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
