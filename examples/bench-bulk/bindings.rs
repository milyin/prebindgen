use prebindgen_declarations::{callback, expand_param, expand_return, fun};
use prebindgen_jni::{data_class, package, ptr_class, JniGen};
fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    for (mode, bulk) in [("plain", false), ("bulk", true)] {
        let scalar = fun!(scalar);
        let row = fun!(row_sum);
        let numbers = fun!(numbers_sum);
        let rows = fun!(rows_sum);
        let make = fun!(rows_make);
        let echo = fun!(rows_echo);
        let handler = fun!(handler_new);
        let reduce = if bulk {
            fun!(rows_for_reduce).expand_return(expand_return!(Vec<Row>).field_self_into().bulk())
        } else {
            fun!(rows_for_reduce).callback_return_only()
        };
        let packet = if bulk {
            fun!(packet_sum).expand_param("packet", expand_param!(Packet).variant_self().bulk())
        } else {
            fun!(packet_sum)
        };
        let (scalar, row, numbers, rows, make, echo, handler) = if bulk {
            (
                scalar
                    .expand_param("value", expand_param!(i64).variant_self().bulk())
                    .expand_return(expand_return!(i64).field_self().bulk()),
                row.expand_param("value", expand_param!(Row).variant_self().bulk()),
                numbers.expand_param("values", expand_param!(Vec<i64>).variant_self().bulk()),
                rows.expand_param("values", expand_param!(&[Row]).variant_self().bulk()),
                make.expand_return(expand_return!(Vec<Row>).field_self_into().bulk()),
                echo.expand_param("values", expand_param!(Vec<Row>).variant_self().bulk())
                    .expand_return(expand_return!(Vec<Row>).field_self_into().bulk()),
                handler.expand_param(
                    "callback",
                    callback!(impl Fn(Vec<Row>) -> i64 + Send + Sync + 'static)
                        .name("RowsCallback")
                        .expand_param(0, expand_return!(Vec<Row>).field_self_into().bulk()),
                ),
            )
        } else {
            (
                scalar,
                row,
                numbers,
                rows,
                make,
                echo,
                handler.expand_param(
                    "callback",
                    callback!(impl Fn(Vec<Row>) -> i64 + Send + Sync + 'static)
                        .name("RowsCallback"),
                ),
            )
        };
        let row_make = fun!(row_make);
        let wide_make = fun!(wide_row_make);
        let wide_sum = fun!(wide_row_sum);
        let row_callback =
            callback!(impl Fn(i64) -> Row + Send + Sync + 'static).name("RowReturnCallback");
        let wide_callback = callback!(impl Fn(i64) -> WideRow + Send + Sync + 'static)
            .name("WideRowReturnCallback");
        let (row_make, wide_make, wide_sum, row_callback, wide_callback) = if bulk {
            (
                row_make.expand_return(expand_return!(Row).field_self_into().bulk()),
                wide_make.expand_return(expand_return!(WideRow).field_self_into().bulk()),
                wide_sum.expand_param("value", expand_param!(WideRow).variant_self().bulk()),
                row_callback.expand_return(expand_param!(Row).variant_self().bulk()),
                wide_callback.expand_return(expand_param!(WideRow).variant_self().bulk()),
            )
        } else {
            (row_make, wide_make, wide_sum, row_callback, wide_callback)
        };
        let generated = JniGen::builder()
            .source(bench_flat::PREBINDGEN_OUT_DIR)
            .set_package_prefix(format!("bench.{mode}"))
            .set_jni_native_init(r#"System.loadLibrary("bulk_bench")"#)
            .package(
                package!()
                    .class(data_class!(Row))
                    .class(data_class!(WideRow))
                    .class(ptr_class!(RowReturnHandler))
                    .class(ptr_class!(WideRowReturnHandler))
                    .fun(row_make)
                    .fun(wide_make)
                    .fun(wide_sum)
                    .fun(fun!(row_return_handler_new).expand_param("callback", row_callback))
                    .fun(fun!(wide_row_return_handler_new).expand_param("callback", wide_callback))
                    .fun(fun!(row_return_fire))
                    .fun(fun!(wide_row_return_fire))
                    .class(data_class!(Packet))
                    .class(ptr_class!(Handler))
                    .fun(fun!(control))
                    .fun(scalar)
                    .fun(row)
                    .fun(numbers)
                    .fun(rows)
                    .fun(make)
                    .fun(reduce)
                    .fun(echo)
                    .fun(packet)
                    .fun(handler)
                    .fun(fun!(fire)),
            )
            .build()
            .expect("generate benchmark JNI");
        generated
            .write_rust(out.join(format!("{mode}.rs")))
            .unwrap();
        generated.write_kotlin(&out.join("kotlin")).unwrap();
    }
}
