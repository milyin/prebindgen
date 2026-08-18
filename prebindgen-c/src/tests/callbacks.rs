use super::*;

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
                "fn__cbg_in_z_closure_sample_t(c:z_closure_sample_t,)->implFn(zenoh_flat::ZSample)+Send+Sync+'static"
            ),
            "{src}"
        );
    assert!(
        compact.contains("Arc::new(__Ctx{context:c.context,drop:c.drop"),
        "{src}"
    );
    // Arg encoded via its OUTPUT converter, then passed (owned) with context.
    assert!(
        compact.contains("let__w0=__cbg_out_ZSample(__a0);"),
        "{src}"
    );
    assert!(compact.contains("__f(__w0,__ctx.context)"), "{src}");
    assert!(compact.contains("move|__a0:zenoh_flat::ZSample|"), "{src}");
    // Zero-arg trampoline.
    assert!(
        compact.contains(
            "fn__cbg_in_z_closure_drop_t(c:z_closure_drop_t,)->implFn()+Send+Sync+'static"
        ),
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
        compact.contains("letcallback=__cbg_in_z_closure_sample_t(callback);"),
        "{src}"
    );
    assert!(
        compact.contains("leton_close=__cbg_in_z_closure_drop_t(on_close);"),
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
            "fn__cbg_in_z_closure_value_t(c:z_closure_value_t,)->implFn(f64)+Send+Sync+'static"
        ),
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
/// `out_wrappers` gives `Option`/`Vec`/`Cow` a marker converter with a `()`
/// destination: it exists to resolve the entry and make the inner required,
/// while the real ABI is structural. The return path lowers those shapes in
/// `lower_shape`/`encode_value`; the callback-argument path had no case for
/// them, so it fell back to calling the entry's converter — the marker, which
/// takes no arguments (#428).
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
/// `&[u8]` is a plain borrow by every shape test there is, and in the converter
/// table it is a shared-slice **marker**: the `(ptr, len)` lowering for one is
/// structural in the callback path, so it has no element wire to be the element
/// of something else. `Vec<&'static [u8]>` therefore passed a check that
/// enumerated wrapper kinds, and the lowering mapped a zero-argument marker over
/// real slices (#428 review). Lowerability asks the table instead, so a marker
/// disqualifies its shape whatever put it there.
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

/// A `Result` callback argument is refused where it is declared, not emitted as
/// a call to the marker that stands in for it.
///
/// `out_wrappers` gives `Result<T, E>` the same `()` destination it gives
/// `Option`/`Vec`/`Cow`, and no arm of `lower_shape` lowers one. So a rule that
/// asked the *destination* whether a shape can be lowered structurally answered
/// yes for this one and produced the very `E0061` #428 is about (#428 review).
/// The question is the model's: which shapes does `lower_shape` decompose.
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
