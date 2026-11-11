use prebindgen_declarations::{
    callback, convert, expand_param, expand_return, from, fun, into, try_from, try_into, Input,
};
use prebindgen_jni::{data_class, package, ptr_class, JniGenBuilder};
pub fn configure(jni: JniGenBuilder) -> JniGenBuilder {
    jni.convert(
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
        convert!(CanonLabel)
            .input(fun!(canonical_label_new))
            .output(fun!(canonical_label_value)),
    )
    .convert(
        convert!(CanonOwned)
            .input(fun!(canonical_owned_new))
            .output(fun!(canonical_owned_token)),
    )
    .expand(expand_return!(CanonError).field(fun!(canonical_error_code)))
    .package(
        package!("canonical")
            .class(data_class!(CanonSettings))
            .class(data_class!(CanonView))
            .class(ptr_class!(CanonToken))
            .fun(
                fun!(canonical_millis_borrow)
                    .expand_param("value", expand_param!(&CanonMillis).reconstruct()),
            )
            .fun(fun!(canonical_millis_from_token).expand_param(
                "value",
                convert!(CanonMillis).input(fun!(canonical_from_borrowed_token)),
            ))
            .fun(fun!(canonical_millis_echo))
            .fun(fun!(canonical_millis_optional))
            .fun(fun!(canonical_millis_rows))
            .fun(fun!(canonical_millis_callback))
            .fun(fun!(canonical_millis_notify))
            .fun(fun!(canonical_millis_static))
            .fun(fun!(canonical_celsius))
            .fun(fun!(canonical_percent))
            .fun(fun!(canonical_percent_invalid))
            .fun(fun!(canonical_label_echo))
            .fun(fun!(canonical_settings))
            .fun(
                fun!(canonical_settings_rows)
                    .expand_param("value", Input::Sequence(Box::new(Input::value())).bulk()),
            )
            .fun(fun!(canonical_settings_callback))
            .fun(fun!(canonical_view_echo))
            .fun(fun!(canonical_domain_error))
            .fun(fun!(canonical_token_new))
            .fun(fun!(canonical_token_value))
            .fun(fun!(canonical_live))
            .fun(fun!(canonical_consume))
            .fun(fun!(canonical_owned_callback))
            .fun(
                fun!(canonical_millis_callback_local).expand_param(
                    "callback",
                    callback!(impl Fn(CanonMillis) -> CanonMillis)
                        .name("CanonicalIntCallback")
                        .expand_param(0, convert!(CanonMillis).output(into!(i32)))
                        .expand_return(convert!(CanonMillis).input(from!(i32))),
                ),
            )
            .fun(
                fun!(canonical_millis_local)
                    .expand_param("value", convert!(CanonMillis).input(from!(i32)))
                    .expand_return(convert!(CanonMillis).output(into!(i32))),
            ),
    )
}
