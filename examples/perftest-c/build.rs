//! Migrated from main; unsupported declarations remain commented in place.
use prebindgen_registry::fun;
use syn::parse_quote as pq;

fn main() {
    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name (e.g. `Payload` -> `payload_t`, `String` -> `string_t`).
    let mut cbindgen = prebindgen_c::Cbindgen::builder()
        .source(perftest_flat::PREBINDGEN_OUT_DIR)
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
    cbindgen = cbindgen.handle("storage", pq!(Storage));

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
    cbindgen = cbindgen.function(fun!(storage_new));
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(storage_get)).panic();
    */
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
    cbindgen = cbindgen.function(fun!(string_new))/* C errors now return a status. */;
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.function(pq!(string_len)).panic();
    */

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

    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generation = cbindgen.build().expect("build supported declarations");
    generation.write_rust(out.join("bindings.rs")).unwrap();
    generation.write_header(out.join("bindings.h")).unwrap();
}
