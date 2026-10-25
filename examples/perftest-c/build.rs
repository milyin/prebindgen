//! Migrated from main; unsupported declarations remain commented in place.
use prebindgen_declarations::{expand_param, expand_return, fields, fun};
use syn::parse_quote as pq;

fn main() {
    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name (e.g. `Payload` -> `payload_t`, `String` -> `string_t`).
    let mut cbindgen = prebindgen_c::Cbindgen::builder()
        .opaque_ptr("panic_output", pq!(CPanicOutput))
        .function(
            fun!(c_panic_output_new).expand_return(expand_return!(CPanicOutput).field_self_into()),
        )
        .function(fun!(c_plain_panic))
        .expand(expand_param!(Vec<i64>).variant_self())
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
        .function(
            fun!(api_record_new).expand_return(
                expand_return!(ApiRecord).fields_into(
                    fields!(api_record_into)
                        .field("omitted", expand_return!(OwnedResource))
                        .field("omitted_opt", expand_return!(OwnedResource))
                        .field("omitted_vec", expand_return!(OwnedResource)),
                ),
            ),
        )
        .opaque_error(syn::parse_quote!(ResultFailure), fun!(result_error_message))
        .opaque_error(syn::parse_quote!(StorageError), fun!(storage_error_message))
        .error("result_error", pq!(CResultError))
        .error("panic_error", pq!(CPanicError))
        .error("rich_error", pq!(CRichError))
        .error("drop_error", pq!(CDropError))
        .expand(expand_return!(CPanicOutput).field(fun!(c_panic_output_value)))
        .function(fun!(c_result_drop_panic))
        .function(fun!(c_result_error_drop))
        .function(fun!(c_result_panic_error))
        .function(fun!(c_result_rich_error))
        .function(fun!(c_result_optional_handle))
        .error("resource_error", pq!(CResourceError))
        .function(fun!(c_result_structured))
        .function(fun!(c_result_bool))
        .function(
            fun!(c_result_late_niche).expand_return(
                expand_return!(CResultPair).fields_into(fields!(c_result_pair_into)),
            ),
        )
        .function(fun!(c_result_resource_error))
        .function(fun!(collection_construct_all).expand_param(
            "values",
            expand_param!(CollectionItem).variant(fun!(collection_construct)),
        ))
        .panic()
        .function(fun!(collection_make_items).expand_return(
            expand_return!(CollectionItem).fields_into(fields!(collection_item_into)),
        ))
        .panic()
        .function(fun!(collection_bundle).expand_return(
            expand_return!(CollectionBundle).fields_into(fields!(collection_bundle_into)),
        ))
        .panic()
        .function(fun!(collection_optional_handles).callback_return())
        .panic()
        .function(fun!(collection_nested_handles).callback_return())
        .panic()
        .function(fun!(collection_numbers).expand_return(
            expand_return!(CollectionNumbers).fields(fields!(collection_numbers_view)),
        ))
        .panic()
        .function(fun!(collection_resource_ids).expand_param(
            "values",
            expand_param!(OwnedResource).variant(fun!(collection_resource_clone)),
        ))
        .panic()
        .function(fun!(collection_i64).callback_return())
        .panic()
        .function(
            fun!(collection_text)
                .expand_param("values", expand_param!(Vec<String>).variant_self())
                .callback_return(),
        )
        .panic()
        .function(fun!(collection_bytes).callback_return())
        .panic()
        .function(fun!(collection_nested).callback_return())
        .panic()
        .function(fun!(collection_options).callback_return())
        .panic()
        .function(fun!(collection_optional).callback_return())
        .panic()
        .function(fun!(collection_optional_items).callback_return())
        .panic()
        .function(fun!(collection_enum).callback_return())
        .panic()
        .function(fun!(collection_probe))
        .panic()
        .function(fun!(collection_borrow).callback_return())
        .panic()
        .function(fun!(collection_slice).callback_return())
        .panic()
        .function(fun!(collection_resource_new))
        .panic()
        .function(fun!(collection_handles).callback_return())
        .function(fun!(collection_alias))
        .panic()
        .function(fun!(collection_result).callback_return())
        .function(fun!(collection_try_construct_all).expand_param(
            "values",
            expand_param!(CollectionItem).variant(fun!(collection_construct)),
        ))
        .function(fun!(collection_try_make_items).expand_return(
            expand_return!(CollectionItem).fields_into(fields!(collection_item_into)),
        ))
        .error("collection_error", pq!(CollectionError))
        .function(fun!(collection_fail))
        .function(fun!(collection_result_optional).callback_return())
        .function(
            fun!(sequence_late_drop)
                .callback_return_only()
                .expand_return(expand_return!(Vec<CPanicOutput>).field(fun!(sequence_late_view))),
        )
        .function(fun!(sequence_scalars).callback_return_only().expand_return(
            expand_return!(SequenceScalars).fields_into(fields!(sequence_scalars_into)),
        ))
        .panic()
        .function(
            fun!(sequence_parts).callback_return_only().expand_return(
                expand_return!(SequencePart).fields_into(fields!(sequence_part_into)),
            ),
        )
        .function(
            fun!(collection_late_drop)
                .expand_return(expand_return!(CPanicOutput).field(fun!(collection_late_values))),
        )
        .function(fun!(storage_put_slice))
        .panic()
        .function(fun!(storage_get_vec).callback_return())
        .panic()
        .function(fun!(result_scalar))
        .function(fun!(result_unit))
        .function(fun!(result_optional))
        .function(fun!(result_text))
        .function(fun!(result_record))
        .function(fun!(result_owned))
        .function(
            fun!(result_other)
                .on_error(fun!(result_string_message))
                .expand_return(expand_return!(i64).field_self()),
        )
        .function(fun!(storage_try_with_label))
        .function(fun!(result_error_drops))
        .panic()
        .enum_type("priority", pq!(Priority))
        .opaque_ptr("enum_payload", pq!(PayloadHandle))
        .function(
            fun!(enum_payload_new).expand_return(expand_return!(PayloadHandle).field_self_into()),
        )
        .panic()
        .function(fun!(payload_priority))
        .panic()
        .enum_type("code", pq!(SparseCode))
        .enum_type("enum_names", pq!(EnumNames))
        .expand(expand_param!(EnumRecord).variant(fun!(enum_record_new)))
        .expand(expand_return!(EnumRecord).fields_into(fields!(enum_record_into)))
        .function(fun!(priority_weight))
        .panic()
        .function(fun!(priority_or))
        .panic()
        .function(fun!(sparse_echo))
        .panic()
        .function(fun!(sparse_optional))
        .panic()
        .function(fun!(sparse_nested))
        .panic()
        .function(fun!(sparse_ref))
        .panic()
        .function(fun!(sparse_borrowed))
        .panic()
        .function(fun!(enum_record_echo))
        .panic()
        .function(fun!(enum_names_echo))
        .panic()
        /* Unsupported: old C output naming or memory policy.
        .source_module(pq!(perftest_flat))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_type_name(|base| format!("{base}_t"))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_destructor(|base| format!("{base}_drop"))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_function(|n| n.to_string())
        */
                // The universal raw-memory freer for the malloc'd `(payload_t *, size_t)`
                // array returned by `storage_get_vec` (Vec<Payload>). The C side releases
                // each element's `label` with `payload_drop`, then frees the block with
                // this `perftest_free`. (Library-namespaced, like `sqlite3_free` /
                // `curl_free` — it MUST run inside this cdylib so the block is released on
                // the same heap it was allocated on, which a plain `free()` from the
                // consumer cannot guarantee across module/CRT boundaries.)
                /* Unsupported: old C output naming or memory policy.
        .free_memory_function("perftest_free")
        */
        .prefix("perftest");

    // `String` as an opaque handle: C holds it as `string_t *` (= `Box<String>`),
    // built by `string_new`, read via `string_len`, freed by `string_drop`. This is
    // what lets the FFI-safe `Payload` carry a heap string by opaque pointer.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.opaque_ptr(pq!(String));
    */

    // `Storage` as an opaque handle (`storage_t *` = `Box<Storage>`, `storage_drop`):
    // it owns the payload that the functions read/write.
    cbindgen = cbindgen.opaque_ptr("storage", pq!(Storage));

    // `PayloadHandler` as an opaque handle (`payload_handler_t *`, `payload_handler_drop`):
    // a prepared callback built once via `payload_handler_new` and fired by
    // `storage_callback` — the registered-subscriber pattern.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.opaque_ptr(pq!(PayloadHandler));
    */

    // `PayloadVecHandler` as an opaque handle: a prepared WHOLE-BATCH callback fired by
    // `storage_callback_vec` (its closure receives the slice by reference).
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.opaque_ptr(pq!(PayloadVecHandler));
    */

    // The zero-copy, `#[repr(C)]` value struct. Emits a visible-field `payload_t`
    // mirror (`Option<Box<String>>` -> `string_t *label`) + a `Transmute` and a
    // compile-time size/align assert proving the reinterpret sound. Owned-ness is
    // INFERRED from the fields: `Payload` has an opaque-pointer `label`, so a by-value
    // consume (`storage_put_by_take`) reads the value out through a `*mut payload_t` and
    // nulls the moved-out `label` in place, making the caller's later free a no-op (no
    // `.owned()` modifier, no `Default` requirement — the `label` is nullable).
    //
    // `.assume_c_field_validity()` acknowledges the one gap the mirror policy
    // cannot close: `Payload::flag` is a `bool`, and the whole-struct
    // reinterpret gives no hook to normalise a byte outside `{0, 1}` that a C
    // caller wrote (#170 instance 3). This benchmark's C side writes the field
    // through `_Bool`, so it is in-domain by construction; a real binding should
    // prefer a `data_struct` field, which normalises per field.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .repr_c_struct(pq!(Payload))
            .assume_c_field_validity();
    */

    // The `&Payload` callback signature -> a `closure_payload_t` closure struct
    // whose `call` takes a `const payload_t *` (zero-copy borrow). `.base_name`
    // gives it a clean name (the `&Payload` base would mangle to `___payload`).
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .callback(pq!(impl Fn(&Payload) + Send + Sync + 'static))
            .base_name("payload");
    */

    // The whole-batch `&[Payload]` callback signature -> a `closure_payload_vec_t` whose
    // `call` takes a `const payload_t *` + `size_t` — the slice delivered **by reference**
    // (zero-copy, no per-element materialization).
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .callback(pq!(impl Fn(&[Payload]) + Send + Sync + 'static))
            .base_name("payload_vec");
    */

    // Functions. `storage_new` returns a fresh handle (no fallible input). The others
    // take null-checked borrows / by-value consumes with no `Result`, so they `.panic()`
    // on a null pointer. The five `storage_put_*`/`storage_get_into_*` demonstrate the
    // distinct C parameter semantics (by-value consume, `const *` read, `*` read/write,
    // out-param-into-init, out-param-into-uninit).
    cbindgen = cbindgen.function(fun!(storage_new)).panic();
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_put_by_take)).panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_put_by_read)).panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .function(pq!(storage_put_by_read_and_update))
            .panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_get_into_init)).panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_get_into_uninit)).panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(payload_handler_new));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_callback)).panic();
    */
    cbindgen = cbindgen.function(fun!(string_new)).panic()/* C errors now return a status. */;
    cbindgen = cbindgen.function(fun!(string_len)).panic()/* C errors now return a status. */;

    // Array (slice / Vec) API. `storage_put_slice` takes `&[Payload]` — a
    // `repr_c_struct` slice — which lowers to `(const payload_t *, size_t)`
    // reinterpreted zero-copy (the slice analogue of the `&Payload` borrow).
    // `storage_get_vec` returns `Vec<Payload>` → a malloc'd `(payload_t *, size_t)`
    // array the C side frees per-element. Both have only null-checked borrow inputs
    // and no `Result`, so `.panic()`.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_put_slice)).panic();
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_get_vec)).panic();
    */

    // Whole-batch callback: prepare once, fire with the slice by reference.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(payload_vec_handler_new));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_callback_vec)).panic();
    */

    // Reduced flat records now allow constructor selectors and field projections.
    cbindgen = cbindgen
        .opaque_ptr("blob", pq!(BlobValue))
        .opaque_ptr("report", pq!(Report))
        .opaque_ptr("report_struct", pq!(ReportStruct))
        .expand(
            expand_param!(Report)
                .variant_self()
                .variant(fun!(report_new)),
        )
        .expand(
            expand_return!(Report)
                .fields(fields!(report_to_struct))
                .field_self(),
        )
        .function(fun!(blob_value_new))
        .panic()
        .function(fun!(blob_value_echo))
        .panic()
        .function(fun!(report_new))
        .panic()
        .function(fun!(report_into_struct))
        .panic();

    cbindgen = cbindgen
        .opaque_ptr("summary", pq!(Summary))
        .opaque_ptr("unsigned", pq!(Unsigned))
        .expand(
            expand_return!(Summary)
                .field(fun!(summary_count))
                .field(fun!(summary_total)),
        )
        .expand(
            expand_param!(Summary)
                .variant(fun!(summary_new))
                .variant_self(),
        )
        .expand(
            expand_return!(Stamp)
                .field(fun!(stamp_secs))
                .field(fun!(stamp_nanos)),
        )
        .expand(expand_return!(Payload).fields(fields!(payload_to_struct)))
        .expand(expand_param!(Payload).variant(fun!(payload_new)))
        .function(fun!(payload_new))
        .panic()
        .function(fun!(storage_with_payload))
        .panic()
        .function(fun!(summary_new))
        .panic()
        .function(fun!(summary_total_raw))
        .panic()
        .function(fun!(stamp_new))
        .panic()
        .function(fun!(unsigned_round_trip))
        .panic()
        .function(fun!(plain_note_echo))
        .panic()
        .function(fun!(unsigned_optional))
        .panic()
        .function(fun!(unsigned_data_maybe))
        .panic()
        .function(fun!(storage_get))
        .panic()
        .function(fun!(storage_put_opt))
        .panic()
        .opaque_ptr("archive", pq!(Archive))
        .function(fun!(archive_new))
        .panic()
        .function(fun!(archive_store))
        .panic()
        .function(fun!(archive_latest))
        .panic();
    cbindgen = cbindgen
        .function(fun!(report_label).expand_param(
            "report",
            expand_param!(Report).variant(fun!(report_from_summary)),
        ))
        .panic()
        .function(fun!(storage_summary_handle).expand_return(expand_return!(Summary).field_self()))
        .panic();
    for f in [
        fun!(storage_len),
        fun!(storage_contains),
        fun!(storage_total_len),
        fun!(storage_put_by_take),
        fun!(summary_merge),
        fun!(primitive_i8),
        fun!(primitive_i16),
        fun!(primitive_i32),
        fun!(primitive_i64),
        fun!(primitive_i128),
        fun!(primitive_isize),
        fun!(primitive_u8),
        fun!(primitive_u16),
        fun!(primitive_u32),
        fun!(primitive_u64),
        fun!(primitive_u128),
        fun!(primitive_usize),
        fun!(primitive_f32),
        fun!(primitive_f64),
        fun!(primitive_bool),
        fun!(primitive_char),
        fun!(primitive_unit),
    ] {
        cbindgen = cbindgen.function(f).panic();
    }
    cbindgen = cbindgen
        .opaque_ptr("owned_resource", pq!(OwnedResource))
        .expand(expand_param!(ResourceChoice).variant(fun!(resource_choice_new)))
        .opaque_ptr("niche_choice", pq!(NicheChoiceHandle))
        .expand(
            expand_param!(NicheChoiceHandle)
                .variant(fun!(niche_choice_new))
                .variant_self(),
        )
        .opaque_ptr("option_ctor", pq!(OptionCtor))
        .expand(
            expand_param!(OptionCtor)
                .variant(fun!(option_ctor_new))
                .variant_self(),
        )
        .opaque_ptr("option_choice", pq!(OptionChoice))
        .expand(
            expand_param!(OptionChoice)
                .variant(fun!(option_choice_new))
                .variant_self(),
        )
        .expand(expand_param!(OptionalFields).variant(fun!(optional_fields_new)))
        .expand(expand_return!(OptionalFields).fields_into(fields!(optional_fields_into)));
    for f in [
        fun!(resource_choice_id),
        fun!(option_ctor_new),
        fun!(option_ctor_id),
        fun!(option_nested_resource),
        fun!(option_nested_u32),
        fun!(niche_choice_new),
        fun!(niche_choice_id),
        fun!(option_str_echo),
        fun!(option_slice_echo),
        fun!(option_str_len),
        fun!(option_slice_len),
        fun!(option_string_len),
        fun!(option_i64_ref),
        fun!(option_choice_id),
        fun!(option_resource_pair),
        fun!(option_i64),
        fun!(option_char),
        fun!(option_u8),
        fun!(option_bool),
        fun!(option_text),
        fun!(option_bytes),
        fun!(option_unit),
        fun!(option_nested),
        fun!(option_nested_char),
        fun!(option_resource_new),
        fun!(option_resource_echo),
        fun!(option_resource_id),
        fun!(owned_resource_drops),
        fun!(optional_fields_echo),
    ] {
        cbindgen = cbindgen.function(f).panic();
    }
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generation = cbindgen.build().expect("build supported declarations");
    generation.write_rust(out.join("bindings.rs")).unwrap();
    generation.write_header(out.join("bindings.h")).unwrap();
}
