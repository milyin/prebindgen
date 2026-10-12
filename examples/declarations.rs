use prebindgen_registry::{expand_param, expand_return, fields, fun, ExpandDecl, FunctionDecl};

// The same existing build.rs declarations configure both adapters; no slot access.
fn expansions() -> Vec<ExpandDecl> {
    vec![
        expand_param!(KeyExpr)
            .variant_self()
            .variant(fun!(keyexpr_from_string))
            .into(),
        expand_param!(Vec<u8>).variant_self().into(),
        expand_param!(Payload)
            .variant_self()
            .variant(fun!(payload_from_fields))
            .variant(fun!(payload_from_string))
            .variant(fun!(payload_from_slice))
            .into(),
        expand_return!(KeyExpr)
            .field(fun!(keyexpr_to_string))
            .into(),
        expand_return!(Payload)
            .field_self()
            .fields(
                fields!(payload_parts)
                    .field(
                        "key",
                        expand_return!(KeyExpr)
                            .field_self()
                            .field(fun!(keyexpr_to_string)),
                    )
                    .field("data", expand_return!(Vec<u8>).field_self()),
            )
            .into(),
    ]
}
fn functions() -> Vec<FunctionDecl> {
    vec![
        fun!(foo).expand_param("y", expand_param!(Payload).variant_self()),
        fun!(keyexpr_from_string).expand_return(expand_return!(KeyExpr).field_self()),
        fun!(keyexpr_text).expand_param("k", expand_param!(KeyExpr).variant_self()),
        fun!(bytes_from_slice).expand_return(expand_return!(Vec<u8>).field_self()),
        fun!(bytes_read),
        fun!(payload_from_string).expand_return(expand_return!(Payload).field_self()),
        fun!(payload_from_slice).expand_return(expand_return!(Payload).field_self()),
        fun!(payload_text).expand_param("p", expand_param!(Payload).variant_self()),
        fun!(noop),
    ]
}
