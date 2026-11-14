use prebindgen_declarations::convert;
use prebindgen_jni::prelude::*;
use prebindgen_jni::{callback, expand_param, expand_return, fun, param_fields};
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
        let handle_values = fun!(handle_values_sum).expand_param(
            "values",
            if bulk {
                expand_param!(i64).variant(fun!(input_handle_value)).bulk()
            } else {
                expand_param!(i64).variant(fun!(input_handle_value))
            },
        );
        let record_input = expand_param!(Vec<HandleRecord>).element(
            expand_param!(HandleRecord).variant_record(param_fields!().field(
                "handle",
                expand_param!(InputHandle).variant(fun!(input_handle_copy)),
            )),
        );
        let handle_records = fun!(handle_records_sum).expand_param(
            "values",
            if bulk {
                record_input.bulk()
            } else {
                record_input
            },
        );
        let owned_handles = if bulk {
            fun!(owned_handles_echo)
                .expand_param("values", expand_param!(InputHandle).variant_self().bulk())
                .expand_return(expand_return!(InputHandle).field_self_into().bulk())
        } else {
            fun!(owned_handles_echo)
        };
        let owned_records_input = expand_param!(Vec<HandleRecord>)
            .element(expand_param!(HandleRecord).variant_record(param_fields!()));
        let owned_records = fun!(owned_handle_records_echo).expand_param(
            "values",
            if bulk {
                owned_records_input.bulk()
            } else {
                owned_records_input
            },
        );
        let owned_records = if bulk {
            owned_records.expand_return(expand_return!(HandleRecord).field_self_into().bulk())
        } else {
            owned_records
        };
        let nested_owners = if bulk {
            fun!(nested_owned_handles_echo)
                .expand_param("values", expand_param!(InputHandle).variant_self().bulk())
                .expand_return(expand_return!(InputHandle).field_self_into().bulk())
        } else {
            fun!(nested_owned_handles_echo)
        };
        let (owned_handles, nested_owners) = if bulk {
            (owned_handles, nested_owners)
        } else {
            (
                owned_handles.callback_return(),
                nested_owners.callback_return(),
            )
        };
        let nested_callback_input = expand_param!(Vec<Vec<InputHandle>>).element(
            expand_param!(Vec<InputHandle>).element(if bulk {
                expand_param!(InputHandle).variant_self().bulk()
            } else {
                expand_param!(InputHandle).variant_self()
            }),
        );
        let nested_callback_input = if bulk {
            nested_callback_input.bulk()
        } else {
            nested_callback_input
        };
        let nested_callback = fun!(callback_nested_owned_handles).expand_param(
            "callback",
            callback!(impl Fn() -> Vec<Vec<InputHandle>>)
                .name("OwnedNestedCallback")
                .expand_return(nested_callback_input),
        );
        let nested_callback = if bulk {
            nested_callback.expand_return(expand_return!(InputHandle).field_self_into().bulk())
        } else {
            nested_callback.callback_return()
        };
        let batch_input = expand_param!(Vec<HandleBatch>).element(
            expand_param!(HandleBatch).variant_record(
                param_fields!().field(
                    "rows",
                    expand_param!(Vec<HandleRecord>)
                        .element(expand_param!(HandleRecord).variant_record(param_fields!())),
                ),
            ),
        );
        let mixed_batch_input = expand_param!(Vec<HandleBatch>).element(
            expand_param!(HandleBatch).variant_record(
                param_fields!().field(
                    "rows",
                    expand_param!(Vec<HandleRecord>)
                        .element(expand_param!(HandleRecord).variant_record(param_fields!()))
                        .bulk(),
                ),
            ),
        );
        let batches = fun!(callback_batches_owned_handles)
            .expand_param(
                "callback",
                callback!(impl Fn() -> Vec<HandleBatch>)
                    .name("OwnedBatchesCallback")
                    .expand_return(if bulk {
                        batch_input.bulk()
                    } else {
                        batch_input
                    }),
            )
            .expand_return(expand_return!(Vec<HandleBatch>).field_self_into().bulk());
        let mixed_batches = fun!(callback_mixed_batches_owned_handles)
            .expand_param(
                "callback",
                callback!(impl Fn() -> Vec<HandleBatch>)
                    .name("OwnedMixedBatchesCallback")
                    .expand_return(mixed_batch_input),
            )
            .expand_return(expand_return!(Vec<HandleBatch>).field_self_into().bulk());
        let callback_input = expand_param!(Vec<HandleRecord>)
            .element(expand_param!(HandleRecord).variant_record(param_fields!()));
        let callback_input = if bulk {
            callback_input.bulk()
        } else {
            callback_input
        };
        let callback_records = fun!(callback_owned_records).expand_param(
            "callback",
            callback!(impl Fn() -> Vec<HandleRecord>)
                .name("OwnedRecordsCallback")
                .expand_return(callback_input),
        );
        let callback_records = if bulk {
            callback_records.expand_return(expand_return!(HandleRecord).field_self_into().bulk())
        } else {
            callback_records
        };
        let record_input = expand_param!(HandleRecord).variant_record(param_fields!());
        let record_input = if bulk {
            record_input.bulk()
        } else {
            record_input
        };
        let callback_record = fun!(callback_owned_record).expand_param(
            "callback",
            callback!(impl Fn() -> HandleRecord)
                .name("OwnedRecordCallback")
                .expand_return(record_input),
        );
        let callback_record = if bulk {
            callback_record.expand_return(expand_return!(HandleRecord).field_self_into().bulk())
        } else {
            callback_record
        };
        let buffer_input = expand_param!(Vec<BufferRecord>)
            .element(expand_param!(BufferRecord).variant_record(param_fields!()));
        let buffers = fun!(owned_buffer_records_echo).expand_param(
            "values",
            if bulk {
                buffer_input.bulk()
            } else {
                buffer_input
            },
        );
        let buffers = if bulk {
            buffers.expand_return(expand_return!(BufferRecord).field_self_into().bulk())
        } else {
            buffers
        };
        let value_input = || expand_param!(BenchValue).variant(fun!(bench_value_from));
        let value_output = || expand_return!(BenchValue).field(fun!(bench_value_to));
        let canonical_value = fun!(canonical_value);
        let expanded_value = fun!(expanded_value)
            .expand_param("value", value_input())
            .expand_return(value_output());
        let canonical_values = fun!(canonical_values);
        let expanded_values = fun!(expanded_values);
        let (canonical_values, expanded_values) = if bulk {
            (
                canonical_values
                    .expand_param(
                        "values",
                        expand_param!(Vec<BenchValue>).variant_self().bulk(),
                    )
                    .expand_return(expand_return!(Vec<BenchValue>).field_self_into().bulk()),
                expanded_values
                    .expand_param("values", value_input().bulk())
                    .expand_return(value_output().bulk()),
            )
        } else {
            (
                canonical_values,
                expanded_values
                    .expand_param("values", value_input())
                    .expand_return(value_output()),
            )
        };
        let canonical_callback = fun!(canonical_value_callback);
        let expanded_callback = fun!(expanded_value_callback)
            .expand_param("value", value_input())
            .expand_return(value_output())
            .expand_param(
                "callback",
                callback!(impl Fn(BenchValue) -> BenchValue)
                    .name("ExpandedValueCallback")
                    .expand_param(0, value_output())
                    .expand_return(value_input()),
            );
        let builder = JniGen::builder()
            .convert(
                convert!(BenchValue)
                    .input(fun!(bench_value_from))
                    .output(fun!(bench_value_to)),
            )
            // Freeze both baselines: this fixture compares explicit transport choices.
            .set_bulk_collection_outputs(false)
            .source(bench_flat::PREBINDGEN_OUT_DIR)
            .set_package_prefix(format!("bench.{mode}"))
            .set_jni_native_init(r#"System.loadLibrary("bulk_bench")"#)
            .package(
                package!()
                    .fun(canonical_value)
                    .fun(expanded_value)
                    .fun(canonical_values)
                    .fun(expanded_values)
                    .fun(canonical_callback)
                    .fun(expanded_callback)
                    .class(ptr_class!(InputHandle))
                    .class(data_class!(HandleRecord))
                    .class(data_class!(HandleBatch))
                    .fun(batches)
                    .fun(mixed_batches)
                    .class(data_class!(BufferRecord))
                    .fun(fun!(input_handle_new))
                    .fun(fun!(input_handle_value))
                    .fun(fun!(buffer_sizes))
                    .fun(
                        fun!(selected_handle).expand_param(
                            "value",
                            expand_param!(i64)
                                .variant_self()
                                .variant(fun!(input_handle_value)),
                        ),
                    )
                    .fun(handle_values)
                    .fun(handle_records)
                    .fun(owned_handles)
                    .fun(owned_records)
                    .fun(callback_records)
                    .fun(callback_record)
                    .fun(buffers)
                    .fun(nested_owners)
                    .fun(nested_callback)
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
            );
        let generated = builder.build().expect("generate benchmark JNI");
        generated
            .write_rust(out.join(format!("{mode}.rs")))
            .unwrap();
        generated.write_kotlin(&out.join("kotlin")).unwrap();
    }
}
