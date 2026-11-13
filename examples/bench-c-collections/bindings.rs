use prebindgen_c::prelude::*;
use prebindgen_c::COutputUnary;
use prebindgen_c::{
    expand_param, expand_return, fields, fun, CInputRequest, COutput, COutputRequest,
};
use prebindgen_declarations::SequenceDelivery;
fn rows(element: COutput<CInputRequest, COutputRequest>) -> COutput<CInputRequest, COutputRequest> {
    COutput::Unary {
        operation: COutputUnary::Sequence {
            delivery: SequenceDelivery::Value,
            mode: None,
        },
        child: Box::new(element),
    }
}
fn main() {
    let builder = prebindgen_c::Cbindgen::builder()
        .source(c_collection_flat::PREBINDGEN_OUT_DIR)
        .prefix("bench")
        .opaque_ptr("borrowed_numbers", syn::parse_quote!(BorrowedNumbers))
        .function(fun!(borrowed_numbers_new))
        .panic()
        .function(fun!(borrowed_values))
        .panic()
        .function(fun!(borrowed_emit))
        .panic()
        .function(fun!(shared_input_inspect))
        .panic()
        .function(fun!(shared_input_sum))
        .panic()
        .function(fun!(shared_input_array))
        .panic()
        .function(fun!(shared_input_text))
        .panic()
        .function(fun!(shared_input_owned))
        .panic()
        .opaque_ptr("resource", syn::parse_quote!(Resource))
        .opaque_owned_struct(
            "native_packet",
            syn::parse_quote!(NativePacket),
            prebindgen_c::OpaqueLayout::auto::<c_collection_flat::NativePacket>(),
        )
        .expand(expand_param!(NativeMixed).variant(fun!(native_mixed_new)))
        .function(fun!(native_packet_new))
        .panic()
        .function(fun!(take_native))
        .panic()
        .function(fun!(take_native_mixed))
        .panic()
        .enum_type("input_tag", syn::parse_quote!(InputTag))
        .expand(expand_param!(TaggedInput).variant(fun!(tagged_new)))
        .expand(expand_param!(TaggedBuffersInput).variant(fun!(tagged_buffers_new)))
        .expand(expand_return!(Record).fields_into(fields!(record_into)))
        .expand(expand_param!(Record).variant(fun!(record_new)))
        .expand(expand_param!(WideInput).variant(fun!(wide_new)))
        .expand(expand_param!(TextInput).variant(fun!(text_new)))
        .expand(expand_param!(BytesInput).variant(fun!(bytes_new)))
        .expand(expand_param!(BuffersInput).variant(fun!(buffers_new)))
        .function(fun!(resource_new))
        .panic()
        .function(fun!(fixed_borrow))
        .panic()
        .function(fun!(fixed_move))
        .panic()
        .function(fun!(callback_fixed))
        .panic()
        .function(fun!(callback_many))
        .panic()
        .function(fun!(callback_native))
        .panic()
        .function(fun!(callback_native_mixed))
        .panic()
        .function(fun!(take_handles))
        .panic()
        .function(fun!(take_records))
        .panic()
        .function(fun!(take_wide))
        .panic()
        .function(fun!(take_text))
        .panic()
        .function(fun!(take_bytes))
        .panic()
        .function(fun!(take_buffers))
        .panic()
        .function(fun!(take_tagged))
        .panic()
        .function(fun!(take_tagged_buffers))
        .panic()
        .function(fun!(take_numbers))
        .panic()
        .function(fun!(control))
        .panic()
        .function(fun!(resource_value))
        .panic()
        .function(fun!(make_handles).output(rows(COutput::auto())))
        .panic()
        .function(fun!(make_records).output(rows(COutput::auto())))
        .panic()
        .function(fun!(make_nested).output(rows(rows(COutput::auto()))))
        .panic()
        .function(fun!(make_many).output(rows(rows(COutput::auto()))))
        .panic()
        .function(fun!(make_numbers).output(rows(COutput::auto())))
        .panic();
    let generation = builder.build().expect("C collection benchmark bindings");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    generation.write_rust(out.join("bindings.rs")).unwrap();
    generation.write_header(out.join("bindings.h")).unwrap();
}
