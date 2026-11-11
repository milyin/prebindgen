use prebindgen_c::Cbindgen;
use prebindgen_declarations::{
    convert, expand_param, from, fun, into, try_from, try_into, Output, SequenceDelivery,
};
pub fn configure(c: Cbindgen) -> Cbindgen {
    c.convert(
        convert!(CanonMillis)
            .input(fun!(canonical_millis_new))
            .output(fun!(canonical_millis_value)),
    )
    .convert(convert!(CanonCelsius).input(from!(i32)).output(into!(i32)))
    .convert(
        convert!(CanonPercent)
            .input(try_from!(i32))
            .output(try_into!(i32)),
    )
    .convert(
        convert!(CanonOwned)
            .input(fun!(canonical_owned_new))
            .output(fun!(canonical_owned_token)),
    )
    .opaque_ptr("canonical_token", syn::parse_quote!(CanonToken))
    .error("canonical_error", syn::parse_quote!(CanonError))
    .function(
        fun!(canonical_millis_borrow)
            .expand_param("value", expand_param!(&CanonMillis).reconstruct()),
    )
    .panic()
    .function(fun!(canonical_millis_from_token).expand_param(
        "value",
        convert!(CanonMillis).input(fun!(canonical_from_borrowed_token)),
    ))
    .panic()
    .function(fun!(canonical_millis_echo))
    .panic()
    .function(fun!(canonical_millis_optional))
    .panic()
    .function(fun!(canonical_millis_rows).expand_return(Output::Sequence {
        element: Box::new(Output::value()),
        mode: None,
        delivery: SequenceDelivery::Value,
    }))
    .panic()
    .function(fun!(canonical_millis_callback))
    .panic()
    .function(fun!(canonical_millis_notify))
    .panic()
    .function(fun!(canonical_millis_static))
    .panic()
    .function(fun!(canonical_celsius))
    .panic()
    .function(
        fun!(canonical_millis_local)
            .expand_param("value", convert!(CanonMillis).input(from!(i32)))
            .expand_return(convert!(CanonMillis).output(into!(i32))),
    )
    .panic()
    .function(fun!(canonical_domain_error))
    .function(fun!(canonical_token_new))
    .panic()
    .function(fun!(canonical_token_value))
    .panic()
    .function(fun!(canonical_live))
    .panic()
    .function(fun!(canonical_consume))
    .function(fun!(canonical_owned_callback))
    .panic()
}
