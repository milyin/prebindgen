//! Migrated from main; unsupported declarations remain commented in place.
use prebindgen_declarations::fun;
use syn::parse_quote as pq;

fn main() {
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    let unstable = std::env::var("CARGO_FEATURE_UNSTABLE").is_ok();
    */

    // The C / cbindgen adapter. Name-mangling rules turn each Rust type/function
    // into its C name, so no per-item `.name(...)` is needed.
    let mut cbindgen = prebindgen_c::Cbindgen::builder()
        .source(example_flat::PREBINDGEN_OUT_DIR)
        /* Unsupported: old C output naming or memory policy.
        .source_module(pq!(example_flat))
        */
                // Single universal freer for the `char*` data the layer hands out
                // (the `String` returns). Opaque handles keep their typed `*_drop`.
                /* Unsupported: old C output naming or memory policy.
        .free_memory_function("example_free")
        */
                // `Calculator` -> `calculator_t` / `calculator_drop`; the base defaults to
                // the snake_case of the Rust short name (no `mangle_rust_type` override).
                /* Unsupported: old C output naming or memory policy.
        .mangle_type_name(|base| format!("{base}_t"))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_destructor(|base| format!("{base}_drop"))
        */
                /* Unsupported: old C output naming or memory policy.
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        */
                // Keep the Rust function names verbatim as the exported C symbols.
                /* Unsupported: old C output naming or memory policy.
        .mangle_function(|n| n.to_string())
        */
        .prefix("example");

    // The opaque `Calculator` handle (Box-owned; `calculator_drop` generated).
    cbindgen = cbindgen.opaque_ptr("calculator", pq!(Calculator));

    // `Error` is opaque (a boxed std error), marshalled to C as a `char*` message
    // via `error_get_message(&e)`; each fallible wrapper gains a `char **e` out-param.
    // `error_get_message` is consumed only as that message fn, so it is not also
    // declared as an exported `.function` — mark it ignored to silence the
    // "skipping undeclared" notice.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .opaque_error(pq!(Error), pq!(error_get_message))
            .ignore_function(pq!(error_get_message));
    */

    // The primitive-repr `Operation` enum -> a C enum.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.enum_type(pq!(Operation));
    */

    // The data-carrying `Shape` enum -> a `#[repr(C)]` enum with payload
    // variants, which cbindgen renders as a C tag + `union`. Its `Labeled` arm
    // owns a `char *`, so a typed `shape_drop` is generated to free the active
    // arm. `Drawing` carries one as a by-value field.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.tagged_union(pq!(Shape));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.data_struct(pq!(Drawing));
    */

    // #158 part 2: payload wires come from the resolved CONVERTER DESTINATION,
    // not the layout-preserving mirror policy. `Note` exercises the two shapes
    // that admits — a nested `data_struct` by value (`Caption`, owning a
    // `char *` the union's drop must reach through and free) and a converted
    // leaf (`Millis` → `u64`).
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.data_struct(pq!(Caption));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.convert(
            prebindgen_declarations::convert!(Millis)
                .input(prebindgen_declarations::fun!(millis_from_raw))
                .output(prebindgen_declarations::fun!(millis_to_raw)),
        );
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .ignore_function(pq!(millis_from_raw))
            .ignore_function(pq!(millis_to_raw));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.tagged_union(pq!(Note));
    */

    // Multi-target cfg demonstration: `InsideFoo` (a fieldless enum whose
    // discriminants vary by `target_arch`) and `Foo` (a by-value data struct whose
    // field set varies by `target_arch` + feature). Declared unconditionally —
    // exactly one `InsideFoo` and one field-shape of `Foo` survive the target's
    // cfg filtering, so the generated `inside_foo_t` / `foo_t` differ per target.
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.enum_type(pq!(InsideFoo));
    */
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen.data_struct(pq!(Foo));
    */

    // The history-replay callback signature -> a `closure_value_t` closure struct
    // (`.base_name` gives the otherwise `f64`-derived struct a descriptive name).
    /* Unsupported: this declaration requires unsupported conversions or builder options.
    cbindgen = cbindgen
            .callback(pq!(impl Fn(f64) + Send + Sync + 'static))
            .base_name("value");
    */

    // Constructors / `Result`-returning ops (fallible inputs route through the
    // error out-param), plus the infallible by-value `Foo` accessors and the
    // `InsideFoo` producer — none need `.panic()`. `calculator_apply` takes an
    // `Operation` by value, so its `char **e` also carries an invalid-discriminant
    // rejection (see `inside_foo_value` below).
    for function in [
        fun!(calculator_new),
        // Caption retains only its supported text field.
        fun!(caption_new),
        // Unsupported signature: pq!(calculator_new_from_str),
        // Unsupported signature: pq!(calculator_apply),
        // Two consumed handles of one type, and one consumed beside one
        // borrowed: the two shapes the alias preflight covers. Both return
        // `Result`, so the rejection reaches the caller through `char **e`
        // rather than aborting.
        // Unsupported signature: pq!(calculator_merge),
        // Unsupported signature: pq!(calculator_absorb),
        fun!(foo_new),
        fun!(foo_get_id),
        // Unsupported signature: pq!(inside_foo_default),
        // The tagged union crossing OUT: constructed and returned. Nothing to
        // validate on this side — Rust always writes a live arm.
        // Unsupported signature: pq!(shape_new_empty),
        // Unsupported signature: pq!(shape_new_circle),
        // Unsupported signature: pq!(shape_new_rect),
        // The union crossing IN on a fallible function: an out-of-range tag
        // reports through the same `char **e` as the domain error.
        // Unsupported signature: pq!(shape_try_area),
        // `Note`'s constructors: the nested-struct, converted-leaf and `bool`
        // payloads crossing OUT.
        // Unsupported signature: pq!(note_new_silent),
        // Unsupported signature: pq!(note_new_after),
        // Unsupported signature: pq!(note_new_flagged),
    ] {
        cbindgen = cbindgen.function(function).panic();
    }
    /* Unsupported: these signatures require unsupported conversions.
    if unstable {
            // `calculator_reset` mirrors an `#[unstable]` slice of the API; only present
            // in the captured source when the feature is enabled. Its `&mut` borrow is
            // fallible (null-checked) with no `Result`, so `.panic()`.
            cbindgen = cbindgen.function(pq!(calculator_reset)).panic();
        }
    */

    // Borrow-only accessors / predicates / the callback driver: they have fallible
    // (null-checked) borrow inputs but no `Result` channel, so `.panic()` lets the
    // wrapper abort on a null handle. `inside_foo_value` joins them for the same
    // reason with a different fallible input: a C enum is an `int` at the ABI, so
    // its discriminant is validated on the way in (never materialised unchecked),
    // and with no `char **e` to report an out-of-range one it aborts. The three
    // union-consuming accessors are the same case one level up — the tag a C
    // caller supplies is validated before the Rust enum exists, here abortively.
    /* Unsupported: these signatures require unsupported conversions.
    for function in [
            pq!(inside_foo_value),
            pq!(shape_area),
            pq!(shape_get_label),
            pq!(drawing_new),
            pq!(drawing_get_shape),
            // `Note` crossing IN (tag validated, so fallible without a `Result`),
            // and its `&str`-taking constructor.
            pq!(note_value),
            pq!(note_emphatic),
            pq!(note_new_titled),
            pq!(note_new_sketched),
            pq!(caption_new),
            pq!(calculator_new_clone),
            pq!(calculator_get_value),
            pq!(calculator_get_count),
            pq!(calculator_is),
            pq!(calculator_to_string),
            pq!(calculator_get_history),
            pq!(calculator_for_each),
            // `&str` label input is fallible (null-checked) with no `Result`.
            pq!(shape_new_labeled),
        ] {
            cbindgen = cbindgen.function(function).panic();
        }
    */

    cbindgen = cbindgen
        .opaque_ptr("local_counter", pq!(LocalCounter))
        .function(fun!(local_counter_new))
        .panic()
        .function(fun!(local_counter_value))
        .panic()
        .function(fun!(local_counter_label))
        .panic()
        .opaque_ptr("caption", pq!(Caption))
        .opaque_ptr("foo", pq!(Foo));

    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // Exercise explicit target selection independently from Cargo's build target.
    println!("cargo:rerun-if-env-changed=PREBINDGEN_BINDING_TARGET");
    if let Ok(target) = std::env::var("PREBINDGEN_BINDING_TARGET") {
        cbindgen = cbindgen.target(target);
    }
    let generation = cbindgen.build().expect("build supported declarations");
    generation.write_rust(out.join("bindings.rs")).unwrap();
    generation.write_header(out.join("bindings.h")).unwrap();
}
