use super::*;

#[test]
fn callback_artifact_consumes_frozen_argument_sites() {
    use prebindgen_registry::{generation::ArtifactInput, recipe::Role};

    let loc = SourceLocation::default();
    let function: syn::ItemFn = syn::parse_quote!(
        pub fn on_value(cb: impl Fn(u64) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(function), loc)]))
            .expect("index items");
    let binding = CbindgenBuilder::new()
        .callback(syn::parse_quote!(impl Fn(u64) + Send + Sync + 'static))
        .function(syn::parse_quote!(on_value))
        .build_with(registry)
        .expect("resolve");
    let generation = binding
        .gen
        .generation
        .as_ref()
        .expect("frozen generation plan");
    let callback_sites: Vec<_> = generation
        .sites()
        .filter(|site| matches!(site.id().site().role, Role::CallbackArg { .. }))
        .collect();
    assert_eq!(callback_sites.len(), 1);
    // The plan holds every artifact of the file now, not only the ones compiled
    // from a crossing, so this asks for the callback's rather than for the
    // plan's only one.
    let callback = generation
        .artifacts()
        .find(|artifact| artifact.id().kind() == "c-callback")
        .expect("the callback is an artifact of the plan");
    assert!(matches!(
        callback.inputs(),
        [ArtifactInput::Site { site, slots: 1 }] if site == callback_sites[0].id()
    ));
}

/// A `.takeable_param(idx)` callback arg is delivered as `*mut z_x_t`: the
/// closure `call` takes a pointer, the trampoline drops it after the call, and
/// a public `z_x_take(dst, src)` move function is emitted.
#[test]
fn takeable_callback_param() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Sample {
            pub _0: u64,
        }
    );
    // A function declaring a subscriber-like callback by value.
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Sample) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_owned_struct(syn::parse_quote!(Sample), syn::parse_quote!(z_sample_t))
        .callback(syn::parse_quote!(impl Fn(Sample) + Send + Sync + 'static))
        .base_name("z_closure_sample_t")
        .takeable_param(0)
        .function(syn::parse_quote!(z_declare_sub));

    let src = write(cbindgen, registry, "takeable");
    let compact: String = src.split_whitespace().collect();

    // Closure `call` receives the sample as an owned pointer.
    assert!(
        compact.contains("call:::core::option::Option<unsafeextern\"C\"fn(*mutz_sample_t,*mut"),
        "{src}"
    );
    // Trampoline passes `&mut __w0` and drops it after the call.
    assert!(compact.contains("&mut__w0as*mutz_sample_t"), "{src}");
    assert!(
        compact.contains("<z_sample_tas::prebindgen_c_runtime::Transmute>::into_rust(__w0)"),
        "{src}"
    );
    // Public take (move) function emitted (no name mangler in this test ⇒
    // `sample_take`; a real adapter mangles to `z_sample_take`).
    assert!(
        compact
            .contains("pubunsafeextern\"C\"fnsample_take(dst:*mutz_sample_t,src:*mutz_sample_t)"),
        "{src}"
    );
    assert!(
        compact.contains(
            "ptr::write(src,<z_sample_tas::prebindgen_c_runtime::Gravestone>::gravestone(),)"
        ),
        "{src}"
    );
}

/// A subscriber-shaped fn with an `impl Fn(ZSample)` callback and a zero-arg
/// `impl Fn()` on-close: each declared callback emits a by-value `#[repr(C)]`
/// closure struct (`context`/`call`/`drop`), `call` taking the arg's **owned**
/// output wire (`z_sample_t *`) plus the `void *context`. The trampoline
/// rebuilds a Rust closure that encodes args via their output converters and
/// invokes the C `call` through an `Arc<Ctx>` that runs `drop(context)` on
/// release.
#[test]
fn callback_subscriber_emits_closure_structs() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sub(
            session: &ZSession,
            callback: impl Fn(ZSample) + Send + Sync + 'static,
            on_close: impl Fn() + Send + Sync + 'static,
        ) -> Result<ZSubscriber, Error> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZSession))
        .base_name("z_session_t")
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZSubscriber))
        .base_name("z_subscriber_t")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
        .base_name("z_closure_sample_t")
        .callback(syn::parse_quote!(impl Fn() + Send + Sync + 'static))
        .base_name("z_closure_drop_t")
        .function(syn::parse_quote!(z_sub));

    let src = write(cbindgen, registry, "cb_sub");
    let compact: String = src.split_whitespace().collect();

    // Closure structs: sample carries the owned handle wire; drop is zero-arg.
    assert!(compact.contains("structz_closure_sample_t"), "{src}");
    assert!(
            compact.contains(
                "pubcall:::core::option::Option<unsafeextern\"C\"fn(*mutz_sample_t,*mut::core::ffi::c_void),>"
            ),
            "{src}"
        );
    assert!(compact.contains("structz_closure_drop_t"), "{src}");

    // Trampoline: by-value struct in, `impl Fn(<src arg>)` out; Arc-held ctx.
    assert!(
        compact.contains(
            "fn__c_in_convert_wire_to_impl_Fn_ZSample_Send_Sync_static_c_invoke_callback_capture_"
        ) && compact
            .contains("(c:z_closure_sample_t,)->implFn(zenoh_flat::ZSample)+Send+Sync+'static"),
        "{src}"
    );
    assert!(
        compact.contains("Arc::new(__Ctx{context:c.context,drop:c.drop"),
        "{src}"
    );
    // Arg encoded via its OUTPUT converter, then passed (owned) with context.
    assert!(
        operation_call(&compact, "__c_out_convert_ZSample_", "__a0"),
        "{src}"
    );
    assert!(compact.contains("__f(__w0,__ctx.context)"), "{src}");
    assert!(compact.contains("move|__a0:zenoh_flat::ZSample|"), "{src}");
    // Zero-arg trampoline.
    assert!(
        compact.contains(
            "fn__c_in_convert_wire_to_impl_Fn_Send_Sync_static_c_invoke_callback_capture_"
        ) && compact.contains("(c:z_closure_drop_t,)->implFn()+Send+Sync+'static"),
        "{src}"
    );
    assert!(compact.contains("move||{"), "{src}");
    assert!(compact.contains("__f(__ctx.context)"), "{src}");
    // Drop runs the C `drop(context)` on release.
    assert!(compact.contains("Some(__d)=self.drop"), "{src}");
    assert!(compact.contains("__d(self.context)"), "{src}");

    // Wrapper takes both closures by value and decodes them.
    assert!(compact.contains("callback:z_closure_sample_t"), "{src}");
    assert!(compact.contains("on_close:z_closure_drop_t"), "{src}");
    assert!(
        operation_call(
            &compact,
            "__c_in_convert_wire_to_impl_Fn_ZSample_Send_Sync_static_c_invoke_callback_capture_",
            "callback",
        ),
        "{src}"
    );
    assert!(
        operation_call(
            &compact,
            "__c_in_convert_wire_to_impl_Fn_Send_Sync_static_c_invoke_callback_capture_",
            "on_close",
        ),
        "{src}"
    );
    // Result of an opaque handle rides the return (NULL = Err); `e` out-param.
    assert!(compact.contains("->*mutz_subscriber_t"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
}

/// A callback with a built-in scalar argument (`impl Fn(f64)`) must NOT have its
/// argument module-qualified — `f64` lives in no source module, so emitting
/// `zenoh_flat::f64` would be invalid Rust. Regression for the primitive
/// callback-arg qualification bug.
#[test]
fn callback_scalar_arg_not_module_qualified() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_on_value(callback: impl Fn(f64) + Send + Sync + 'static) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .callback(syn::parse_quote!(impl Fn(f64) + Send + Sync + 'static))
        .base_name("z_closure_value_t")
        .function(syn::parse_quote!(z_on_value));

    let src = write(cbindgen, registry, "cb_scalar");
    let compact: String = src.split_whitespace().collect();

    // The bug was `f64` qualified to `zenoh_flat::f64`.
    assert!(!compact.contains("zenoh_flat::f64"), "{src}");
    // Closure param + `impl Fn` return keep `f64` bare.
    assert!(compact.contains("move|__a0:f64|"), "{src}");
    assert!(
        compact.contains(
            "fn__c_in_convert_wire_to_impl_Fn_f64_Send_Sync_static_c_invoke_callback_capture_"
        ) && compact.contains("(c:z_closure_value_t,)->implFn(f64)+Send+Sync+'static"),
        "{src}"
    );
}

/// Without a `.name(...)` override the closure-struct C name is composed
/// generically from the args' configured C type names (`closure_<argCname>`)
/// — `lang::CbindgenBuilder` invents no target-language convention of its own.
#[test]
fn callback_struct_name_defaults_generically() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sub2(
            session: &ZSession,
            callback: impl Fn(ZSample) + Send + Sync + 'static,
        ) -> Result<ZSubscriber, Error> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZSession))
        .base_name("z_session_t")
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZSubscriber))
        .base_name("z_subscriber_t")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        // No `.name(...)` on the callback ⇒ generic default.
        .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
        .function(syn::parse_quote!(z_sub2));

    let src = write(cbindgen, registry, "cb_default");
    let compact: String = src.split_whitespace().collect();

    // Composed from the arg's configured C name `z_sample_t`.
    assert!(compact.contains("structclosure_z_sample_t"), "{src}");
    assert!(compact.contains("callback:closure_z_sample_t"), "{src}");
}

/// A source type under any accepted shape is qualified, not just an outermost
/// one: a callback argument spelled `Option<&Handle>` names the language's
/// `Option` and the source crate's `Handle`, each against its own root (#414).
///
/// The closure the trampoline builds is where a bare name shows: both the
/// returned `impl Fn(..)` and the closure's own parameter ascription are
/// written into the generated file, and neither resolves in a consumer crate
/// that has not imported `Handle`.
#[test]
fn callback_arg_qualifies_a_source_type_under_a_wrapper() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Handle {
            pub _0: u64,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Option<&Handle>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(Handle))
        .callback(syn::parse_quote!(
            impl Fn(Option<&Handle>) + Send + Sync + 'static
        ))
        .base_name("z_closure_handle_t")
        .function(syn::parse_quote!(z_declare_sub));

    let src = write(cbindgen, registry, "cb_optref_qualified");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("implFn(::core::option::Option<&zenoh_flat::Handle>)"),
        "{src}"
    );
    assert!(
        compact.contains("move|__a0:::core::option::Option<&zenoh_flat::Handle>|"),
        "{src}"
    );
}

/// An `Option<T>` callback argument is lowered like any other composite, not
/// handed to the marker that stands in for one.
///
/// `out_wrappers` retains a `()` marker only as a legacy converter carrier. The
/// registry-composed `CValue` owns the real ABI and encoder, and both return and
/// callback sites consume that same frozen payload (#428).
#[test]
fn callback_arg_lowers_an_optional_structurally() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Handle {
            pub _0: u64,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Option<&Handle>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(Handle))
        .callback(syn::parse_quote!(
            impl Fn(Option<&Handle>) + Send + Sync + 'static
        ))
        .base_name("z_closure_handle_t")
        .function(syn::parse_quote!(z_declare_sub));

    let src = write(cbindgen, registry, "cb_optional_arg");
    let compact: String = src.split_whitespace().collect();

    // The marker is never applied to anything: it takes no arguments.
    assert!(
        !compact.contains("__cbg_outmark_option___Handle(__a0)"),
        "the composite marker is called as if it were a converter:\n{src}"
    );
    // `Option<&T>` over an opaque handle carries its absence in the pointer, so
    // the C `call` takes one `*const handle` and NULL is `None`. The slot is
    // `MaybeUninit`, which is `#[repr(transparent)]` and so is neither an ABI
    // nor a header change — it is what lets an absent value leave its slot
    // unwritten instead of being filled with a fabricated one.
    assert!(
        compact.contains(
            "call:::core::option::Option<unsafeextern\"C\"fn(::core::mem::MaybeUninit<*consthandle>,"
        ),
        "the closure `call` takes the borrowed pointer:\n{src}"
    );
}

/// …and so is a `Result` **under** an `Option`, which is the same defect one
/// layer in.
///
/// `Option<Result<T, E>>` is an optional, so a test that looks only at the
/// outermost layer admits it — and then the lowering reaches the `Result` as a
/// base field and calls *its* marker (#428 review). Lowerability is a property
/// of the whole shape, so the check recurses.
#[test]
fn a_result_under_an_option_is_refused_too() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Handle {
            pub _0: u64,
        }
    );
    let err: syn::ItemStruct = syn::parse_quote!(
        pub struct Error {
            pub _0: u64,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Option<Result<Handle, Error>>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Struct(err), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(Handle))
        .opaque_error(syn::parse_quote!(Error), syn::parse_quote!(error_message))
        .callback(syn::parse_quote!(
            impl Fn(Option<Result<Handle, Error>>) + Send + Sync + 'static
        ))
        .base_name("z_closure_opt_result_t")
        .function(syn::parse_quote!(z_declare_sub));

    let message = catch_msg(|| {
        let _ = write(cbindgen, registry, "cb_opt_result_arg");
    });
    assert!(
        message.contains("has no C ABI"),
        "a shape whose inner layer cannot be lowered is refused whole: {message}"
    );
}

/// A run whose ELEMENT has no wire of its own is refused too — the case a list
/// of shapes cannot catch.
///
/// A shared slice itself occupies a pointer-plus-length site and therefore has
/// no single element wire that a surrounding `Vec` can allocate. Recursive
/// `CValue::has_abi` validation rejects that terminal marker before rendering.
#[test]
fn a_run_of_markers_is_refused() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Vec<&'static [u8]>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .callback(syn::parse_quote!(
            impl Fn(Vec<&'static [u8]>) + Send + Sync + 'static
        ))
        .base_name("z_closure_slices_t")
        .function(syn::parse_quote!(z_declare_sub));

    let message = catch_msg(|| {
        let _ = write(cbindgen, registry, "cb_vec_slice_arg");
    });
    assert!(
        message.contains("has no C ABI"),
        "a run whose element has no wire of its own is refused whole: {message}"
    );
}

/// A composite with a `convert!` of its own keeps that ABI: it is not
/// decomposed just because its *shape* is one the lowering knows.
///
/// `select_output_type` tries `out_custom` before `out_wrappers`, so a declared
/// conversion gives `Option<T>` a real wire and no marker. The struct emitter
/// reads the destination and emits that one wire; a dispatcher that decided from
/// the model shape alone would pass two arguments to a `call` declared with one
/// (#428 review). Both halves ask the same question now.
#[test]
fn a_converted_optional_callback_arg_keeps_its_declared_wire() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "#[prebindgen] pub type Duration = std::time::Duration;",
        "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
        "pub fn duration_to_millis(v: &Duration) -> u64 { unimplemented!() }",
        "pub fn maybe_from_millis(v: i64) -> Option<Duration> { unimplemented!() }",
        "pub fn maybe_to_millis(v: &Option<Duration>) -> i64 { unimplemented!() }",
        "pub fn duration_each(cb: impl Fn(Option<Duration>) + Send + Sync + 'static) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        // Both are declared, which is what makes this a real test of the gate:
        // the inner type has a converter, so the shape IS lowerable, and only
        // the marker test can tell that it must not be lowered.
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis)),
        )
        .convert(
            prebindgen_registry::convert!(Option<Duration>)
                .input(prebindgen_registry::fun!(maybe_from_millis))
                .output(prebindgen_registry::fun!(maybe_to_millis)),
        )
        .callback(syn::parse_quote!(
            impl Fn(Option<Duration>) + Send + Sync + 'static
        ))
        .base_name("z_closure_maybe_duration_t")
        .function(syn::parse_quote!(duration_each));

    let src = write(cbindgen, registry, "cb_converted_optional");
    let compact: String = src.split_whitespace().collect();

    // One parameter, the declared representation — not a decomposition.
    assert!(
        compact.contains("unsafeextern\"C\"fn(i64,*mut::core::ffi::c_void)")
            && operation_call(&compact, "__c_out_convert_Option_Duration_", "__a0",),
        "the declared wire survives in the closure struct:\n{src}"
    );
    // …and the closure calls that converter rather than filling lowered slots.
    assert!(
        !compact.contains("MaybeUninit::<i64>::zeroed()"),
        "a converted composite must not be decomposed:\n{src}"
    );
}

/// …and a declared conversion still beats the shape when it is **nested**.
///
/// `Vec<Option<Duration>>` over a declared `Option<Duration>` is a run of a
/// composite that has a wire of its own, so it lowers to that wire's pointer and
/// length. The registry-composed Sequence payload must stop at that declared
/// converter instead of reopening the Optional shape underneath it (#428 review).
#[test]
fn a_run_of_converted_optionals_uses_the_declared_wire() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "#[prebindgen] pub type Duration = std::time::Duration;",
        "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
        "pub fn duration_to_millis(v: &Duration) -> u64 { unimplemented!() }",
        "pub fn maybe_from_millis(v: i64) -> Option<Duration> { unimplemented!() }",
        "pub fn maybe_to_millis(v: &Option<Duration>) -> i64 { unimplemented!() }",
        "pub fn duration_each(cb: impl Fn(Vec<Option<Duration>>) + Send + Sync + 'static) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis)),
        )
        .convert(
            prebindgen_registry::convert!(Option<Duration>)
                .input(prebindgen_registry::fun!(maybe_from_millis))
                .output(prebindgen_registry::fun!(maybe_to_millis)),
        )
        .callback(syn::parse_quote!(
            impl Fn(Vec<Option<Duration>>) + Send + Sync + 'static
        ))
        .base_name("z_closure_durations_t")
        // The run hands C a malloc'd block, so the freer that releases it has
        // to be declared — the same requirement a `Vec` return carries (#437).
        .free_memory_function("z_free")
        .function(syn::parse_quote!(duration_each));

    let src = write(cbindgen, registry, "cb_vec_converted_optional");
    let compact: String = src.split_whitespace().collect();

    // #437: the run is built by the array builder, and this binding returns no
    // `Vec` at all — the helper is emitted because the callback's own encode
    // declares it, not because a return type was scanned for one.
    assert!(compact.contains("__cbg_alloc_array"), "{src}");
    assert!(
        compact.contains("fn__cbg_alloc_array<W>"),
        "the array builder must be emitted for a callback that delivers a run:\n{src}"
    );

    // The run lowers to the element's DECLARED wire, pointer and length.
    assert!(
        compact.contains("unsafeextern\"C\"fn(::core::mem::MaybeUninit<*muti64>,::core::mem::MaybeUninit<usize>,"),
        "the array carries the declared element wire:\n{src}"
    );
    // …and each element goes through that converter, not through a decomposition.
    assert!(
        compact.contains("__c_out_convert_Option_Duration_"),
        "the declared element converter is what fills the array:\n{src}"
    );
}

/// A `Vec<()>` is refused for the same reason as a run of slices: the unit has
/// no storage to be an element, so there is nothing for the `(ptr, len)` pair
/// to point at (#428 review).
#[test]
fn a_run_of_units_is_refused() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Vec<()>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .callback(syn::parse_quote!(impl Fn(Vec<()>) + Send + Sync + 'static))
        .base_name("z_closure_units_t")
        .function(syn::parse_quote!(z_declare_sub));

    let message = catch_msg(|| {
        let _ = write(cbindgen, registry, "cb_vec_unit_arg");
    });
    assert!(
        message.contains("has no C ABI"),
        "a run whose element has no storage is refused whole: {message}"
    );
}

/// A `Result` callback argument is refused where it is declared, not emitted as
/// a call to the marker that stands in for it.
///
/// A `Result<T, E>` marker has no frozen C ABI payload. Callback-site validation
/// rejects it instead of treating every marker as a composable shape and later
/// calling a zero-argument converter with a value (#428 review).
#[test]
fn a_result_callback_arg_is_refused_at_its_declaration() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Handle {
            pub _0: u64,
        }
    );
    let err: syn::ItemStruct = syn::parse_quote!(
        pub struct Error {
            pub _0: u64,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Result<Handle, Error>) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Struct(err), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(Handle))
        .opaque_error(syn::parse_quote!(Error), syn::parse_quote!(error_message))
        .callback(syn::parse_quote!(
            impl Fn(Result<Handle, Error>) + Send + Sync + 'static
        ))
        .base_name("z_closure_result_t")
        .function(syn::parse_quote!(z_declare_sub));

    let message = catch_msg(|| {
        let _ = write(cbindgen, registry, "cb_result_arg");
    });
    assert!(
        message.contains("has no C ABI") && message.contains("Result"),
        "the refusal names the shape and why: {message}"
    );
}
