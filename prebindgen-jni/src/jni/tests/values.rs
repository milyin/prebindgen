use super::*;

#[test]
fn bounded_duration_option_uses_u64_niche_without_boxing() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "#[prebindgen] pub type Duration = std::time::Duration;",
        "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
        "pub fn duration_to_millis(v: &Duration) -> u64 { unimplemented!() }",
        "pub fn duration_echo(v: Option<Duration>) -> Option<Duration> { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        // `syn::Item`, not `ItemFn`: a fixture declares the types it names.
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .package(crate::package!("time").fun(prebindgen_registry::fun!(duration_echo)));
    let dir = unique_test_dir("jnigen_bounded_duration");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).unwrap();
    let rust_path = generation.write_rust(dir.join("gen.rs")).unwrap();
    let rust = std::fs::read_to_string(rust_path).unwrap();
    let paths = generation.write_kotlin(&dir.join("kotlin")).unwrap();
    let kotlin = paths
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let rc: String = rust.split_whitespace().collect();
    let kc: String = kotlin.split_whitespace().collect();

    assert!(rc.contains("outsideitsdeclareddomain"), "{rust}");
    assert!(
        rc.contains("None=>-1i64") || rc.contains("None=>-1"),
        "{rust}"
    );
    assert!(
        rc.contains("Some({let__inner_s0=jlong_to_u64_")
            && rc.contains("let__inner_s1=u64_to_Duration_"),
        "Option input must compose the raw u64 decoder with the Duration stage:\n{rust}"
    );
    assert!(
        rc.contains("Some(value)=>{let__inner_s0=")
            && rc.contains("Duration_to_u64_")
            && rc.contains("u64_to_jlong_"),
        "Option output must compose the Duration stage with the raw u64 encoder:\n{rust}"
    );
    assert!(!rc.contains("Optionbox:"), "{rust}");
    assert!(kc.contains("v:ULong?"), "{kotlin}");
    assert!(kc.contains("v?.toLong()?:-1L"), "{kotlin}");
    assert!(kc.contains("v:Long"), "{kotlin}");
}

#[test]
fn flattened_field_composes_bounded_conversion_stages() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Timed {
                    pub delay: Option<Duration>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_from_millis(v: u64) -> Duration {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_to_millis(v: &Duration) -> u64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn timed_use(value: &Timed) -> u64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn timed_echo(value: &Timed) -> Timed {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .package(
            crate::package!()
                .class(crate::data_class!(Timed))
                .fun(prebindgen_registry::fun!(timed_use))
                .fun(prebindgen_registry::fun!(timed_echo)),
        );
    let dir = unique_test_dir("jnigen_flat_staged_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(generation.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    let kotlin = generation
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let rc: String = rust.split_whitespace().collect();
    let kc: String = kotlin.split_whitespace().collect();

    assert!(kc.contains("valueDelay:Long"), "{kotlin}");
    assert!(kc.contains("value.delay?.toLong()?:-1L"), "{kotlin}");
    assert!(
        kc.contains("funfromParts(delay:Long):Timed=Timed(if(delay==-1L)nullelsedelay.toULong())"),
        "the struct factory must receive the niche as a primitive Long:\n{kotlin}"
    );
    assert!(
        kc.contains("TimedBuilderRaw<outR>{publicfunrun(delay:Long):R}"),
        "{kotlin}"
    );
    assert!(
        kc.contains("if(delay==-1L)nullelsedelay.toULong()"),
        "the raw builder adapter must restore the optional niche:\n{kotlin}"
    );
    assert!(rc.contains("jlong_to_u64"), "{rust}");
    assert!(rc.contains("u64_to_Duration"), "{rust}");
    assert!(
        rc.contains("jlong_to_Option_Duration") && rc.contains("env,&__delay_raw)?"),
        "whole-JObject input must invoke the complete optional Duration converter:\n{rust}"
    );
    assert!(
        rc.contains("let___delay:jni::sys::jlong=Option_Duration_to_jlong")
            && rc.contains("\"(J)Lio/test/jni/Timed;\""),
        "whole-struct output must pass the niche as primitive jlong:\n{rust}"
    );
    assert!(!rc.contains("let___delay:jni::objects::JObject"), "{rust}");
    assert!(
        rc.contains("myflat::Timed{delay:__flat_value_delay"),
        "{rust}"
    );
}

/// The four ways a **bounded** `convert!` leaf can meet optionality, in one
/// fixture — the matrix #142 is about.
///
/// Two independent facts decide the wrap: whether the leaf carries a niche of
/// its own (`Option<Duration>` ⇒ the sentinel IS its `None`), and whether an
/// **ancestor** can be absent (a conditional value form here, which makes every
/// leaf below it nullable). They compose, and the sentinel belongs to the first
/// fact alone:
///
/// | ancestor optional | leaf's own type | wire | wrap |
/// |---|---|---|---|
/// | no  | `Duration`         | `Long`  | `x.toULong()` |
/// | no  | `Option<Duration>` | `Long`  | `if (x == -1L) null else x.toULong()` |
/// | yes | `Duration`         | `Long?` | `x?.toULong()` — **no sentinel** |
/// | yes | `Option<Duration>` | `Long?` | `x?.let { if (it == -1L) null else it.toULong() }` |
///
/// Row 3 is the one that was wrong: `projection_leaf_sentinel` answers off the
/// declared domain, which `attach_domain_sentinels` puts on the **bare** type's
/// converter too, so a leaf with no niche encoding at all got a sentinel test
/// spliced into its wrap. `-1` is outside the declared range so nothing
/// mis-decoded in practice, but the emitted expression tested for a value its
/// own encoder can never produce.
///
/// Row 4 is the one #142 predicted would be wrong and is not: the two absences
/// are independent and both collapse to the `ULong?` the typed view declares.
/// The Rust side boxes any nullable leaf (`leaf_is_prim`), so the nullable wire
/// is real, and the inner `None` still rides the sentinel inside it.
#[test]
fn a_bounded_leaf_takes_its_sentinel_from_its_own_type_not_its_ancestor() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct SpanStruct {
                    pub required: Duration,
                    pub delay: Option<Duration>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_from_millis(v: u64) -> Duration {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_to_millis(v: &Duration) -> u64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn span_to_struct(s: &Span) -> SpanStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        // The CONDITIONAL hoist: `Span`'s value form is reached through an
        // `Option`, so every leaf below it is nullable — rows 3 and 4.
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn holder_span(h: &Holder) -> Option<&Span> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn holder_each(cb: impl Fn(Holder) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        // …and the same two leaves NOT under an optional ancestor — rows 1 and 2.
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn span_each(cb: impl Fn(Span) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .package(
            crate::package!()
                .class(crate::ptr_class!(Span))
                .class(crate::ptr_class!(Holder))
                .fun(prebindgen_registry::fun!(span_each))
                .fun(prebindgen_registry::fun!(holder_each)),
        )
        .expand(
            prebindgen_registry::expand_return!(Span)
                .fields(prebindgen_registry::fields!(span_to_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(Holder)
                .field(prebindgen_registry::fun!(holder_span)),
        );
    let dir = unique_test_dir("jnigen_niche_matrix");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let kotlin = generation
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Rows 1 and 2 — no optional ancestor. Both wires stay primitive `Long`,
    // and only the leaf that HAS a niche tests for it.
    assert!(
        kc.contains("funrun(required:Long,delay:Long)"),
        "with no optional ancestor both leaves keep the primitive wire:\n{kotlin}"
    );
    assert!(
        kc.contains("required.toULong()"),
        "row 1: a bare bounded leaf just converts:\n{kotlin}"
    );
    assert!(
        kc.contains("if(delay==-1L)nullelsedelay.toULong()"),
        "row 2: the leaf's own niche is tested, unguarded:\n{kotlin}"
    );

    // Rows 3 and 4 — under the conditional hoist. Both wires widen, because the
    // Rust side boxes any nullable leaf.
    assert!(
        kc.contains("funrun(holderSpan__required:Long?,holderSpan__delay:Long?)"),
        "under an optional ancestor both leaves box:\n{kotlin}"
    );
    // Row 3: the ancestor's `?` is the WHOLE of this leaf's absence. A sentinel
    // test here would ask about a value the encoder cannot emit.
    assert!(
        kc.contains("holderSpan__required?.toULong()"),
        "row 3: a bare bounded leaf under an optional ancestor takes no \
         sentinel:\n{kotlin}"
    );
    assert!(
        !kc.contains("holderSpan__required?.let{if(it==-1L)"),
        "row 3: …and specifically not the doubly-optional shape:\n{kotlin}"
    );
    // Row 4: both absences are live and independent — the ancestor's null and
    // the leaf's own `None` — and both collapse to the declared `ULong?`.
    assert!(
        kc.contains("holderSpan__delay?.let{if(it==-1L)nullelseit.toULong()}"),
        "row 4: an ancestor's `?` does not erase the leaf's own niche:\n{kotlin}"
    );
}

#[test]
fn duration_requires_an_explicit_conversion() {
    let alias: syn::Item =
        syn::parse_str("#[prebindgen] pub type Duration = std::time::Duration;").unwrap();
    let function: syn::ItemFn =
        syn::parse_str("pub fn duration_echo(v: Duration) -> Duration { unimplemented!() }")
            .unwrap();
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (alias, myflat_loc()),
        (syn::Item::Fn(function), myflat_loc()),
    ]))
    .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("time").fun(prebindgen_registry::fun!(duration_echo)));

    let error = jni
        .build_with(registry)
        .expect_err("Duration must not have an implicit unchecked converter")
        .to_string();
    assert!(error.contains("Duration"), "{error}");
}

#[test]
#[should_panic(expected = "domain type i64 does not match input representation u64")]
fn conversion_domain_must_match_the_representation() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
        "pub fn duration_use(v: Duration) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        // `syn::Item`, not `ItemFn`: a fixture declares the types it names.
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let jni = JniGenBuilder::new()
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .valid_range(0i64..=1_000i64),
        )
        .package(crate::package!("time").fun(prebindgen_registry::fun!(duration_use)));

    let _ = jni.build_with(registry);
}

/// Phase 4: a bare `Option<primitive>` / `Option<enum>` **input** parameter
/// crosses as a decoupled `(present: Boolean, value: <prim>)` pair instead of a
/// boxed `java.lang.*` `JObject`. The Rust side reassembles the `Option` from
/// two raw scalars (`if <p>_present != 0u8 { Some(..) } else { None }`) with no
/// reflective `intValue()`/`longValue()` unbox. The public Kotlin signature
/// keeps `T?`; the call site passes `<name> != null` and `<name> ?: <zero>`
/// (`<name>?.value ?: 0` for an enum).
#[test]
fn option_scalar_param_crosses_as_present_value_pair() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Mode {
                    A,
                    B,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_set_timeout(ms: Option<i64>, count: Option<i32>, mode: Option<Mode>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::enum_class!(Mode)))
        .package(crate::package!("cfg").fun(prebindgen_registry::fun!(z_set_timeout)));

    let dir = unique_test_dir("jnigen_optscalar");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Public wrapper signature keeps the nullable typed params.
    assert!(kc.contains("ms:Long?"), "{kotlin}");
    assert!(kc.contains("count:Int?"), "{kotlin}");
    assert!(kc.contains("mode:Mode?"), "{kotlin}");

    // Extern declares the decomposed `(present, value)` pairs, never a boxed
    // `Long?`/`Int?` value wire.
    assert!(kc.contains("msPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("msValue:Long"), "{kotlin}");
    assert!(kc.contains("countPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("countValue:Int"), "{kotlin}");
    assert!(kc.contains("modePresent:Boolean"), "{kotlin}");
    assert!(kc.contains("modeValue:Int"), "{kotlin}");

    // Call site splits each param into present-flag + value-or-zero (enum reads
    // `?.value`).
    assert!(kc.contains("ms!=null"), "{kotlin}");
    assert!(kc.contains("ms?:0L"), "{kotlin}");
    assert!(kc.contains("count?:0"), "{kotlin}");
    assert!(kc.contains("mode?.value?:0"), "{kotlin}");

    // Rust native wrapper takes the two raw scalars and rebuilds the `Option`
    // with no boxed-object unbox, then passes the rebuilt values to the source
    // fn. (The `Option<i64>`/`Option<i32>`/`Option<Mode>` boxed converters are
    // still emitted but are now dead `#[allow(dead_code)]` — the param path no
    // longer references them, exactly like the Phase-1 dead Vec converters.)
    assert!(rc.contains("ms_present:jni::sys::jboolean"), "{rust}");
    assert!(rc.contains("ms_value:jni::sys::jlong"), "{rust}");
    assert!(rc.contains("count_value:jni::sys::jint"), "{rust}");
    assert!(rc.contains("mode_value:jni::sys::jint"), "{rust}");
    assert!(rc.contains("ifms_present!=0u8"), "{rust}");
    // The live path feeds the three rebuilt `Option`s straight to the source
    // call — no boxed `JObject` param anywhere in the wrapper.
    assert!(
        rc.contains("myflat::z_set_timeout(ms,count,mode)"),
        "{rust}"
    );
}

/// Phase 2: a `Vec<opaque-handle>` / `Option<Vec<handle>>` **return** crosses as
/// a Kotlin-side leaf fold — each element's raw `jlong` pointer crosses and the
/// generated `<Handle>Folder` singleton wraps it into the typed handle class and
/// appends to an `ArrayList`. No Rust-side `java.util.ArrayList` of handle
/// objects is built (the `reject_vec_of_handle` guard is lifted for outputs).
#[test]
fn vec_of_handle_output_folds_kotlin_side() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_list() -> Vec<ZThing> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_list_opt() -> Option<Vec<ZThing>> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing))
                .fun(prebindgen_registry::fun!(thing_list))
                .fun(prebindgen_registry::fun!(thing_list_opt)),
        );

    let dir = unique_test_dir("jnigen_vec_handle_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // A `ZThingFolderRaw<A>` interface is generated, and the wrapper returns a
    // typed list, allocating the `ArrayList<ZThing>` accumulator on the Kotlin
    // side. The fold is FIXED (the hoisted singleton below is its only
    // implementation), so no typed twin or `asRaw` proxy is emitted — that
    // surface would be dead public API (#160).
    assert!(kc.contains("interfaceZThingFolderRaw<A>"), "{kotlin}");
    assert!(!kc.contains("interfaceZThingFolder<A>"), "{kotlin}");
    assert!(!kc.contains("ZThingFolder<A>.asRaw"), "{kotlin}");
    assert!(kc.contains("List<ZThing>"), "{kotlin}");
    assert!(kc.contains("ArrayList<ZThing>()"), "{kotlin}");
    // The folder singleton wraps each raw `jlong` element into the typed handle
    // class and appends it — no Rust object construction.
    assert!(
        kc.contains("ZThing.fromRawPtr(element)") || kc.contains("acc.add(ZThing.fromRawPtr("),
        "{kotlin}"
    );
    // `Option<Vec<…>>` surfaces as a nullable list.
    assert!(kc.contains("List<ZThing>?"), "{kotlin}");

    // Rust: each element's pointer is delivered as a raw `jvalue { j: … }` to the
    // folder's `run`, NOT wrapped into a Java object; no Rust-side `ArrayList` is
    // built for the handle vec.
    assert!(rc.contains("jvalue{j:__enc}"), "{rust}");
    assert!(
        !rc.contains(r#"new_object("java/util/ArrayList""#),
        "no Rust-side ArrayList for Vec<handle>: {rust}"
    );
}

/// Phase 5: a `data_class` **input** param carrying an `Option<primitive>` /
/// `Option<enum>` field — which used to decline field-flattening and box the
/// whole struct into a `JObject` (Rust `env.get_field(...)`) — now flattens, the
/// `Option` field crossing as a `(<field>Present: Boolean, <field>Value: <prim>)`
/// leaf pair the Rust side rebuilds with no reflective unbox.
#[test]
fn option_scalar_struct_field_flattens() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Opts {
                    pub id: i64,
                    pub ttl: Option<i64>,
                    pub flag: Option<bool>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn opts_put(o: &Opts) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Opts))
                .fun(prebindgen_registry::fun!(opts_put)),
        );

    let dir = unique_test_dir("jnigen_optfield");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The public wrapper keeps the typed `Opts` param; the extern crosses the
    // option fields as decomposed `(present, value)` pairs (the plain `id` field
    // stays a single leaf).
    assert!(kc.contains("oTtlPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("oTtlValue:Long"), "{kotlin}");
    assert!(kc.contains("oFlagPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("oFlagValue:Boolean"), "{kotlin}");
    // Call site destructures the typed object: present-flag + value-or-zero.
    assert!(kc.contains("o.ttl!=null"), "{kotlin}");
    assert!(kc.contains("o.ttl?:0L"), "{kotlin}");
    assert!(kc.contains("o.flag?:false"), "{kotlin}");

    // Rust rebuilds each field's `Option` from the raw scalars (gated on present)
    // and reconstructs the struct inline from the flat leaves, passing it to the
    // source fn. (The whole-struct `JObject_to_Opts` `get_field` converter is
    // still emitted but is now dead `#[allow(dead_code)]`, like Phase 4's boxed
    // converters — the live param path no longer references it.)
    assert!(rc.contains("o_ttl_present:jni::sys::jboolean"), "{rust}");
    assert!(rc.contains("o_ttl_value:jni::sys::jlong"), "{rust}");
    assert!(rc.contains("ifo_ttl_present!=0u8"), "{rust}");
    assert!(
        rc.contains("myflat::Opts{id:__flat_o_id,ttl:__flat_o_ttl,flag:__flat_o_flag"),
        "{rust}"
    );
    assert!(rc.contains("myflat::opts_put(&o)"), "{rust}");
}

/// A `data_class` with a NESTED data-class field plus enum / `Option<prim>` /
/// `Option<enum>` fields. Output recursively uses `fromParts`; input now
/// recursively flattens the same graph into primitive leaves, without passing
/// either `Job` or `Inner` as a `JObject`.
///  * output `fromParts` descriptor: an `Option`-boxed primitive slot is the
///    BOX class (`Ljava/lang/Long;` / `Ljava/lang/Integer;`), not the bare
///    primitive — and the Kotlin factory takes `Int?` for `Option<enum>`,
///    rebuilding via `?.let { E.fromInt(it) }`;
///  * input `get_field` descriptors are the slots' EXACT static types (nested
///    class FQN, box class, enum class + `getValue()I` decode), not the erased
///    `Ljava/lang/Object;`;
///  * a bare `Option<enum>` RETURN wires as `Int?` (the boxed discriminant),
///    mapped back in the wrapper — previously the extern claimed the enum
///    class while the native side returned a boxed `Integer`.
#[test]
fn recursive_data_class_input_flattens_nested_and_optional_fields() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Level {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Inner {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Job {
                    pub inner: Inner,
                    pub level: Level,
                    pub ttl: Option<i64>,
                    pub mode: Option<Level>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn job_make(tag: i64) -> Job {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn job_mode(j: &Job) -> Option<Level> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("model")
                .class(crate::enum_class!(Level))
                .class(crate::data_class!(Inner))
                .class(crate::data_class!(Job)),
        )
        .package(
            crate::package!("job")
                .fun(prebindgen_registry::fun!(job_make))
                .fun(prebindgen_registry::fun!(job_mode)),
        );

    let dir = unique_test_dir("jnigen_fromparts_optbox");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // OUTPUT (`job_make` → `fromParts`): the nested `inner` inlines to its `J`
    // leaf, the bare enum stays a raw `I`, and the two `Option` fields occupy
    // their BOX-class slots.
    assert!(
        rc.contains(r#""(JILjava/lang/Long;Ljava/lang/Integer;)Lio/test/jni/model/Job;""#),
        "{rust}"
    );
    // Kotlin factory: `Long?` / `Int?` params, enum rebuilt nullably; nested
    // child reassembled via its own factory.
    assert!(kc.contains("ttl:Long?"), "{kotlin}");
    assert!(kc.contains("mode:Int?"), "{kotlin}");
    assert!(kc.contains("mode?.let{Level.fromInt(it)}"), "{kotlin}");
    assert!(kc.contains("Inner.fromParts(inner_id)"), "{kotlin}");

    // INPUT (`job_mode`): the native method receives the recursively flattened
    // leaves and Rust reconstructs `Inner` before `Job`. No live wrapper-side
    // `get_field` decode is needed.
    assert!(kc.contains("jInnerId:Long"), "{kotlin}");
    assert!(kc.contains("jLevel:Int"), "{kotlin}");
    assert!(kc.contains("jTtlPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("jModeValue:Int"), "{kotlin}");
    assert!(kc.contains("j.inner.id"), "{kotlin}");
    assert!(rc.contains("myflat::Inner{id:__flat_j_inner_id"), "{rust}");
    assert!(
        rc.contains("myflat::Job{inner:__flat_j_inner,level:__flat_j_level"),
        "{rust}"
    );

    // RETURN (`job_mode` → `Option<Level>`): the extern wires `Int?`; the
    // wrapper maps the boxed discriminant back to the nullable enum.
    assert!(kc.contains("jModeValue:Int"), "{kotlin}");
    assert!(kc.contains("errorSink:Any"), "{kotlin}");
    assert!(kc.contains("):Int?"), "{kotlin}");
    assert!(kc.contains("?.let{Level.fromInt(it)}"), "{kotlin}");
}

#[test]
fn jobject_input_is_an_explicit_hybrid_leaf_escape_hatch() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct FlatChild {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ObjectChild {
                    pub name: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Hybrid {
                    pub flat: FlatChild,
                    pub maybe: Option<FlatChild>,
                    pub object: ObjectChild,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn hybrid_use(h: Hybrid) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn hybrid_optional(h: Option<Hybrid>) -> i64 {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(FlatChild))
                .class(crate::data_class!(ObjectChild).jobject_input())
                .class(crate::data_class!(Hybrid))
                .fun(prebindgen_registry::fun!(hybrid_use))
                .fun(prebindgen_registry::fun!(hybrid_optional)),
        );
    let dir = unique_test_dir("jnigen_hybrid_jobject_input");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(generation.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    let kotlin = generation
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let rc: String = rust.split_whitespace().collect();
    let kc: String = kotlin.split_whitespace().collect();

    assert!(kc.contains("hFlatId:Long"), "{kotlin}");
    assert!(kc.contains("hObject:ObjectChild"), "{kotlin}");
    assert!(kc.contains("h.flat.id"), "{kotlin}");
    assert!(kc.contains("hMaybePresent:Boolean"), "{kotlin}");
    assert!(kc.contains("h.maybe?.id?:0L"), "{kotlin}");
    assert!(kc.contains("h.object_"), "{kotlin}");
    assert!(kc.contains("hPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("hObject:io.test.jni.ObjectChild?"), "{kotlin}");
    assert!(kc.contains("h?.object_"), "{kotlin}");
    assert!(rc.contains("JObject_to_ObjectChild"), "{rust}");
    assert!(
        rc.contains(
            "myflat::Hybrid{flat:__flat_h_flat,maybe:__flat_h_maybe,object:__flat_h_object"
        ),
        "{rust}"
    );
    assert!(generation.report().contains("input `JObject` opt-in"));
}

/// A payload-less struct keeps its own delimiters through the `.jobject_input()`
/// decoder.
///
/// The decoder walks `flat::Struct::fields`, and the element does not record
/// whether the fields were named — that is spelling. So the braced initializer
/// this used to hard-code emitted `myflat::Unit {}` for `struct Unit;`, which is
/// not Rust. The `syn::Fields::Named` guard the walk replaced happened to refuse
/// it by returning `None`; the per-field name check cannot, because an empty
/// struct has no field to refuse.
///
/// A tuple struct is absent because it cannot reach here at all: the model
/// reads one as an `Extern`, so `Flat::struct_type` answers `None` for it.
///
/// `Struct::spell` is the one place those delimiters are chosen — the dual of
/// the `Alternative::spell` the sum decoder uses for `E::B()`.
#[test]
fn empty_structs_keep_their_own_constructor_delimiters() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Unit;
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct EmptyNamed {}
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn take_empties(a: Unit, c: EmptyNamed) -> i64 {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Unit).jobject_input())
                .class(crate::data_class!(EmptyNamed).jobject_input())
                .fun(prebindgen_registry::fun!(take_empties)),
        );
    let dir = unique_test_dir("jnigen_empty_struct_delimiters");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(generation.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Each shape gets the delimiters Rust demands for it, and none gets braces
    // it cannot have.
    assert!(
        rc.contains("myflat::Unit)") || rc.contains("myflat::Unit}"),
        "unit struct must be constructed bare, got:\n{rust}"
    );
    assert!(
        !rc.contains("myflat::Unit{}"),
        "unit struct must not take braces:\n{rust}"
    );
    assert!(
        rc.contains("myflat::EmptyNamed{}"),
        "empty named struct keeps its braces:\n{rust}"
    );
}

#[test]
fn recursive_flattened_owned_handles_join_lock_and_consume_scaffold() {
    let loc = myflat_loc();
    let items = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Token {
                    pub value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Envelope {
                    pub token: Token,
                    pub spare: Option<Token>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn envelope_use(e: Envelope) -> i64 {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Token))
                .class(crate::data_class!(Envelope))
                .fun(prebindgen_registry::fun!(envelope_use)),
        );
    let dir = unique_test_dir("jnigen_recursive_handles");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(generation.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    let kotlin = generation
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let rc: String = rust.split_whitespace().collect();
    let kc: String = kotlin.split_whitespace().collect();

    assert!(kc.contains("withSortedHandleLocks(__locks)"), "{kotlin}");
    assert!(kc.contains("__locks.add(e.token)"), "{kotlin}");
    assert!(kc.contains("e.spare?.let{__locks.add(it)}"), "{kotlin}");
    assert!(kc.contains("e.spare?.isClosed()==true"), "{kotlin}");
    assert!(kc.contains("valeToken_ptr=e.token.ptr"), "{kotlin}");
    assert!(kc.contains("valeSpare_ptr=e.spare?.ptr?:0L"), "{kotlin}");
    assert!(kc.contains("e.token.markConsumed()"), "{kotlin}");
    assert!(kc.contains("e.spare?.markConsumed()"), "{kotlin}");
    assert!(
        rc.contains("Box::from_raw(e_tokenas*mutmyflat::Token)"),
        "{rust}"
    );
    assert!(
        rc.contains(
            "Option::Some(unsafe{*::std::boxed::Box::from_raw(e_spareas*mutmyflat::Token)})"
        ),
        "{rust}"
    );
}

#[test]
fn recursive_flattening_rejects_jvm_parameter_slot_overflow() {
    let fields = (0..127)
        .map(|index| format!("pub f{index}: i64"))
        .collect::<Vec<_>>()
        .join(",");
    let wide: syn::ItemStruct =
        syn::parse_str(&format!("pub struct Wide {{ {fields} }}")).expect("parse wide struct");
    let use_wide: syn::ItemFn = syn::parse_quote!(
        pub fn use_wide(value: Wide) -> i64 {
            unimplemented!()
        }
    );
    let loc = myflat_loc();
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(wide.clone()), loc.clone()),
        (syn::Item::Fn(use_wide.clone()), loc.clone()),
    ]))
    .expect("index items");
    let jni = JniGenBuilder::new().package(
        crate::package!()
            .class(crate::data_class!(Wide))
            .fun(prebindgen_registry::fun!(use_wide)),
    );
    let error = jni
        .build_with(registry)
        .expect_err("256 JVM slots must fail")
        .to_string();
    assert!(error.contains("uses 256 JVM parameter slots"), "{error}");
    assert!(error.contains("jobject_input"), "{error}");

    // The explicit object boundary keeps the same public Kotlin data class,
    // but the native method receives it in one slot and performs the legacy
    // whole-object field decode instead of producing an illegal signature.
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(wide), loc.clone()),
        (syn::Item::Fn(use_wide), loc),
    ]))
    .expect("index marked items");
    let jni = JniGenBuilder::new().package(
        crate::package!()
            .class(crate::data_class!(Wide).jobject_input())
            .fun(prebindgen_registry::fun!(use_wide)),
    );
    let generation = jni
        .build_with(registry)
        .expect("JObject boundary must bypass the flattened slot limit");
    assert!(generation.report().contains("input `JObject` opt-in"));
}

/// An output-only `convert!` type must resolve with only its `.output()`
/// conversion declared: conversions are required per USAGE direction, unlike
/// the four class declarators (always both). The conversion is an ordinary
/// `#[prebindgen]` fn — its signature supplies the continue type (`i64` ⇒
/// jlong wire, Kotlin `Long`), no verbatim strings, no injected expressions.
#[test]
fn output_only_convert_resolves_without_input_twin() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn len_of(s: &String) -> Len { unimplemented!() }",
        "pub fn len_value(l: &Len) -> i64 { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(prebindgen_registry::convert!(Len).output(prebindgen_registry::fun!(len_value)))
        .package(crate::package!("len").fun(prebindgen_registry::fun!(len_of)));
    let dir = unique_test_dir("jnigen_outonly_convert");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni
        .build_with(registry)
        .expect("an output-only convert type must not require an input twin");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The return crosses through the conversion fn, composed with i64's own
    // converter chain (jlong wire).
    assert!(rc.contains("myflat::len_value(&v)"), "{rust}");
    assert!(rc.contains("myflat::len_of(&s)"), "{rust}");
}

/// Multi-source qualification: a fn with a recorded origin crate is called
/// with that crate's module prefix, while origin-less fns keep the
/// registry's default module — the helper-crate model behind `convert!`.
#[test]
fn convert_fn_qualifies_with_origin_crate() {
    // Two chained streams: the flat crate provides `len_of`, a helper crate
    // provides the conversion fn — each item's origin rides its
    // `SourceLocation` stamp, exactly as `Source` streams deliver it.
    let loc = |krate: &str| SourceLocation {
        crate_name: Some(krate.to_string()),
        ..SourceLocation::default()
    };
    let item = |src: &str, krate: &str| -> (syn::Item, SourceLocation) {
        let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
        (syn::Item::Fn(f), loc(krate))
    };
    let flat = vec![item(
        "pub fn len_of(s: &String) -> Len { unimplemented!() }",
        "myflat",
    )];
    let helpers = vec![item(
        "pub fn len_value(l: &Len) -> i64 { unimplemented!() }",
        "my-helpers",
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(flat.into_iter().chain(helpers)))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(prebindgen_registry::convert!(Len).output(prebindgen_registry::fun!(len_value)))
        .package(crate::package!("len").fun(prebindgen_registry::fun!(len_of)));
    let dir = unique_test_dir("jnigen_convert_origin");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The conversion fn call carries the origin-crate module (dashes →
    // underscores); the exported fn keeps the default source module.
    assert!(rc.contains("my_helpers::len_value(&v)"), "{rust}");
    assert!(rc.contains("myflat::len_of(&s)"), "{rust}");
}

/// `convert!` input fn must produce the declared type — a mismatch is a
/// hard error naming both.
#[test]
#[should_panic(expected = "produces `Other`, not `Len`")]
fn convert_input_target_mismatch_rejected() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn from_long(v: i64) -> Other { unimplemented!() }",
        "pub fn use_len(l: Len) { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .convert(prebindgen_registry::convert!(Len).input(prebindgen_registry::fun!(from_long)))
        .package(crate::package!("len").fun(prebindgen_registry::fun!(use_len)));
    let dir = unique_test_dir("jnigen_convert_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = jni
        .build_with(registry)
        .and_then(|gen| gen.write_rust(dir.join("gen.rs")));
}

/// `convert!` via `core::convert` trait impls: `.input(from!(i32))` /
/// `.output(into!(i32))` generate fully-qualified `Into` calls; the wire
/// and Kotlin surface derive from the stated repr's converter chain.
#[test]
fn convert_via_trait_impls() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn temp_double(c: Celsius) -> Celsius { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Celsius)
                .input(prebindgen_registry::from!(i32))
                .output(prebindgen_registry::into!(i32)),
        )
        .package(crate::package!("m").fun(prebindgen_registry::fun!(temp_double)));
    let dir = unique_test_dir("jnigen_convert_trait");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("<i32as::core::convert::Into<myflat::Celsius>>::into(v)"),
        "{rust}"
    );
    assert!(
        rc.contains("<myflat::Celsiusas::core::convert::Into<i32>>::into(v)"),
        "{rust}"
    );
}

/// `.input(try_from!(i32))`: the generated converter is fallible with the
/// impl's associated `Error` as its error type; the body is the qualified
/// `try_into` call.
#[test]
fn convert_via_try_from_is_fallible() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn pct_use(p: Percent) -> i32 { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(prebindgen_registry::convert!(Percent).input(prebindgen_registry::try_from!(i32)))
        .package(crate::package!("m").fun(prebindgen_registry::fun!(pct_use)));
    let dir = unique_test_dir("jnigen_convert_tryfrom");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("<i32as::core::convert::TryInto<myflat::Percent>>::try_into(v)"),
        "{rust}"
    );
    // The converter's Result error type is the impl's associated Error.
    assert!(
        rc.contains("<i32as::core::convert::TryInto<myflat::Percent>>::Error"),
        "{rust}"
    );
}

/// Structural `Option<T>` converters return `__JniErr`, while a custom
/// conversion stage may retain its raw `E`. Both input and output composition
/// must normalize that `E` before using `?`.
#[test]
fn option_composition_normalizes_fallible_stage_errors() {
    let loc = myflat_loc();
    let f: syn::ItemFn = syn::parse_str(
        "pub fn pct_optional(p: Option<Percent>) -> Option<Percent> { unimplemented!() }",
    )
    .unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Percent)
                .input(prebindgen_registry::try_from!(i32))
                .output(
                    prebindgen_registry::fun!(crate::conv::pct_out)
                        .sig(prebindgen_registry::sig!((p: Percent) -> Result<i32, String>)),
                ),
        )
        .package(crate::package!("m").fun(prebindgen_registry::fun!(pct_optional)));
    let dir = unique_test_dir("jnigen_option_fallible_stages");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    assert!(
        rc.matches("__e.to_string()").count() >= 2,
        "input and output stages must both normalize their raw errors:\n{rust}"
    );
    assert!(rc.contains("JObject_to_Option_Percent"), "{rust}");
    assert!(rc.contains("Option_Percent_to_JObject"), "{rust}");
}

/// Binding-local conversion fns via the ONE vocabulary —
/// `fun!(crate::…).sig(sig!(…))`: synthesized into the registry, lowered
/// through the ordinary `#[prebindgen]`-fn path, called by the declared path.
#[test]
fn convert_via_local_fns() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn label_id(l: Label) -> Label { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Label)
                .input(
                    prebindgen_registry::fun!(crate::conv::label_in)
                        .sig(prebindgen_registry::sig!((s: String) -> Label)),
                )
                .output(
                    prebindgen_registry::fun!(crate::conv::label_out)
                        .sig(prebindgen_registry::sig!((l: Label) -> String)),
                ),
        )
        .package(crate::package!("m").fun(prebindgen_registry::fun!(label_id)));
    let dir = unique_test_dir("jnigen_convert_local");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(rc.contains("crate::conv::label_in("), "{rust}");
    assert!(rc.contains("crate::conv::label_out("), "{rust}");
}

/// Two input conversions on one decl are contradictory — decl-time panic.
#[test]
#[should_panic(expected = "input conversion is already declared")]
fn convert_duplicate_input_rejected() {
    let _ = prebindgen_registry::convert!(Widget)
        .input(prebindgen_registry::from!(i32))
        .input(
            prebindgen_registry::fun!(crate::widget_in)
                .sig(prebindgen_registry::sig!((v: String) -> Widget)),
        );
}

/// The source macros state their direction; the acceptor cross-checks it.
#[test]
#[should_panic(expected = "an input conversion is built with from!/try_from!")]
fn convert_input_into_direction_rejected() {
    let _ = prebindgen_registry::convert!(Widget).input(prebindgen_registry::into!(i32));
}

#[test]
#[should_panic(expected = "an output conversion is built with into!/try_into!")]
fn convert_output_from_direction_rejected() {
    let _ = prebindgen_registry::convert!(Widget).output(prebindgen_registry::from!(i32));
}

/// A binding-local conversion source must state its signature — a path
/// carries nothing to read (the sig's `Result<_, E>` is the error channel,
/// replacing the former `.error(ty!)`).
#[test]
#[should_panic(expected = ".sig(sig!(")]
fn convert_local_source_missing_sig_rejected() {
    let _ =
        prebindgen_registry::convert!(Widget).input(prebindgen_registry::fun!(crate::widget_in));
}

/// A `fun!` conversion source is never surfaced in Kotlin — decorations are
/// rejected at the source seam (same policy as ignore/variant/field).
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn convert_source_fun_with_decorations_rejected() {
    let _ = prebindgen_registry::convert!(Widget)
        .input(prebindgen_registry::fun!(widget_in).name("widgetIn"));
}

/// The fallible binding-local form: the sig's `Result<_, E>` IS the error
/// channel — `E` lands in the converter signature and `Err` routes to the
/// caller's error handler, exactly like a `#[prebindgen]` conversion fn's.
#[test]
fn convert_via_local_try_fn_is_fallible() {
    let loc = myflat_loc();
    let f: syn::ItemFn =
        syn::parse_str("pub fn label_id(l: Label) -> Label { unimplemented!() }").unwrap();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(vec![(syn::Item::Fn(f), loc)]))
            .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Label)
                .input(
                    prebindgen_registry::fun!(crate::conv::label_in)
                        .sig(prebindgen_registry::sig!((s: String) -> Result<Label, String>)),
                )
                .output(
                    prebindgen_registry::fun!(crate::conv::label_out)
                        .sig(prebindgen_registry::sig!((l: Label) -> String)),
                ),
        )
        .package(crate::package!("m").fun(prebindgen_registry::fun!(label_id)));
    let dir = unique_test_dir("jnigen_convert_local_try");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(rc.contains("crate::conv::label_in("), "{rust}");
    assert!(rc.contains("Result<myflat::Label,String>"), "{rust}");
}

/// I5: data-class members — the receiver re-enters Rust as `this`'s field
/// leaves (the data-class param destructuring rebased to `this`); a
/// constructor member joins the `fromParts` companion. Extern signatures
/// are identical to the free-fn form.
#[test]
fn data_class_members_reenter_as_field_leaves() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Point {
                    pub x: i64,
                    pub y: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn point_norm(p: &Point) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn point_origin() -> Point {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!().class(
                crate::data_class!(Point)
                    .method(prebindgen_registry::fun!(point_norm).name("norm"))
                    .constructor(prebindgen_registry::fun!(point_origin).name("origin")),
            ),
        );
    let dir = unique_test_dir("jnigen_data_members");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let ac: String = all.split_whitespace().collect();
    // The instance method lives INSIDE the data class and destructures
    // `this` into the flattened leaf args.
    assert!(ac.contains("dataclassPoint("), "{all}");
    assert!(ac.contains("funnorm("), "{all}");
    assert!(ac.contains("this.x,this.y"), "{all}");
    // The factory joined the fromParts companion: within the Point class
    // block there is exactly ONE companion object holding both.
    let point_block = all
        .split("data class Point")
        .nth(1)
        .and_then(|rest| rest.split("fun interface").next())
        .expect("Point class block");
    assert_eq!(point_block.matches("companion object").count(), 1, "{all}");
    let pb: String = point_block.split_whitespace().collect();
    assert!(pb.contains("funorigin("), "{all}");
    assert!(pb.contains("funfromParts("), "{all}");
}

/// #108: fixed-width unsigned scalars use lossless signed JNI carriers. The
/// public Kotlin surface widens `u8/u16/u32` and projects `u64` to `ULong`,
/// while the harness stays primitive (`Int`/`Long`) and nullable `u64` keeps
/// a raw `Long?` twin.
#[test]
fn unsigned_scalars_use_lossless_kotlin_surface_and_raw_jni_wires() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Unsigned {
                    pub byte: u8,
                    pub short: u16,
                    pub int: u32,
                    pub long: u64,
                    pub maybe_long: Option<u64>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn unsigned_round_trip(
                    byte: u8,
                    short: u16,
                    int: u32,
                    long: u64,
                    maybe_long: Option<u64>,
                ) -> Unsigned {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn unsigned_callback(f: impl Fn(u64) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn unsigned_data_maybe(value: &Unsigned) -> Option<u64> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn unsigned_result(value: u64) -> Result<u64, String> {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Unsigned))
                .fun(prebindgen_registry::fun!(unsigned_round_trip))
                .fun(prebindgen_registry::fun!(unsigned_data_maybe))
                .fun(prebindgen_registry::fun!(unsigned_callback))
                .fun(prebindgen_registry::fun!(unsigned_result)),
        );
    let dir = unique_test_dir("jnigen_unsigned");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let rust_path = generation
        .write_rust(dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    assert!(rc.contains("u8::try_from(*v)"), "{rust}");
    assert!(rc.contains("u16::try_from(*v)"), "{rust}");
    assert!(rc.contains("u32::try_from(*v)"), "{rust}");
    assert!(rc.contains("*vas::core::primitive::u64"), "{rust}");

    let paths = generation
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    let kotlin = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Typed wrapper + data-class surface.
    assert!(kc.contains("byte:Int"), "{kotlin}");
    assert!(kc.contains("short:Int"), "{kotlin}");
    assert!(kc.contains("int:Long"), "{kotlin}");
    assert!(kc.contains("long:ULong"), "{kotlin}");
    assert!(kc.contains("maybeLong:ULong?"), "{kotlin}");

    // Raw harness and wrapper bridges retain stable JNI primitives.
    assert!(kc.contains("externalfununsignedRoundTrip("), "{kotlin}");
    assert!(kc.contains("long:Long"), "{kotlin}");
    assert!(kc.contains("maybeLong:Long?"), "{kotlin}");
    assert!(kc.contains("long.toLong()"), "{kotlin}");
    assert!(kc.contains("maybeLong?.toLong()"), "{kotlin}");
    assert!(kc.contains(".toULong()"), "{kotlin}");
    assert!(kc.contains("valueMaybeLongPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("valueMaybeLongValue:Long"), "{kotlin}");
    assert!(kc.contains("value.maybeLong!=null"), "{kotlin}");
    assert!(
        rc.contains("value_maybe_long_present:jni::sys::jboolean"),
        "{rust}"
    );

    // Callback gets a typed interface plus a raw Long twin and adapter.
    assert!(kc.contains("funrun(u64:ULong)"), "{kotlin}");
    assert!(kc.contains("funrun(u64:Long)"), "{kotlin}");
    assert!(kc.contains("u64.toULong()"), "{kotlin}");

    // The projection composes through Result's success arm while retaining
    // the ordinary typed domain-error callback.
    assert!(kc.contains("fununsignedResult(value:ULong"), "{kotlin}");
    assert!(kc.contains("):ULong"), "{kotlin}");
}

/// A data class's constructor property types come from the SAME
/// [`build_struct_plan`] its `fromParts` factory walks, so a property and its
/// own factory parameter cannot disagree (#156). Pinned across the field kinds
/// the plan distinguishes: a handle projection, a nested data class, a bare
/// enum, and an `Option` leaf over an object-shaped wire.
#[test]
fn data_class_properties_match_their_from_parts_params() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Child {
                    pub n: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Level {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Handle {
                    pub v: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Bag {
                    pub handle: Handle,
                    pub child: Child,
                    pub level: Level,
                    pub note: Option<String>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn bag_make() -> Bag {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn bag_take(b: Bag) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Handle))
                .class(crate::data_class!(Child))
                .class(crate::enum_class!(Level))
                .class(crate::data_class!(Bag))
                .fun(prebindgen_registry::fun!(bag_make))
                .fun(prebindgen_registry::fun!(bag_take)),
        );

    let dir = unique_test_dir("jnigen_data_class_props");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).expect("resolve");
    let kotlin = generation
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The declaration: each property takes the plan's type for its kind.
    assert!(kc.contains("valhandle:Handle"), "{kotlin}");
    assert!(kc.contains("valchild:Child"), "{kotlin}");
    assert!(kc.contains("vallevel:Level"), "{kotlin}");
    assert!(kc.contains("valnote:String?"), "{kotlin}");
    // An owned handle property makes the class closeable.
    assert!(kc.contains("AutoCloseable"), "{kotlin}");
    assert!(kc.contains("funclose()"), "{kotlin}");
    // …and the factory reassembles into exactly those properties: the nested
    // child is inlined as its own leaves, the handle arrives as a raw pointer
    // and the enum as its discriminant, then each is rebuilt.
    assert!(kc.contains("Bag(Handle.fromRawPtr(handle)"), "{kotlin}");
    assert!(kc.contains("Child.fromParts(child_n)"), "{kotlin}");
    assert!(kc.contains("Level.fromInt(level)"), "{kotlin}");

    // …and the raw-pointer guard follows that same plan, factory by factory
    // (#37). `Bag` takes a handle leaf as a bare `Long`, so its factory can
    // forge one and carries both guards; `Child` takes an `i64` and carries
    // neither — marking it would remove a safe factory from Java and make
    // consumers opt into a contract it does not have. The negative half of
    // this rule is pinned again in `snapshots::raw_pointer_entry_points_are_guarded`.
    assert!(
        kc.contains(
            "@JvmSynthetic@io.test.jni.UnsafeNativeApi@JvmStaticpublicfunfromParts(handle:Long"
        ),
        "the handle-bearing factory is guarded:\n{kotlin}"
    );
    assert!(
        kc.contains("@JvmStaticpublicfunfromParts(n:Long):Child"),
        "the pointer-free factory is not:\n{kotlin}"
    );
}

/// Every shape an array length can take — a FREE const, an ASSOCIATED const,
/// and a `const fn` CALL — is qualified against its origin module, and that
/// rewrite reaches ONLY the length, never a converter body's locals.
///
/// None of the three owners is declared to JniGenBuilder: each is a compile-time
/// namespace, not a boundary type, so qualification must not depend on a
/// Kotlin class existing for it.
///
/// The const here is deliberately named `env`, which is also the name of the
/// `JNIEnv` local every generated converter uses. A source crate may legally
/// declare it (`#[allow(non_upper_case_globals)] pub const env`), so a
/// whole-item expression pass would rewrite `env.get_java_vm()` to
/// `myflat::env.get_java_vm()` even when restricted to registered const idents
/// — thousands of `no method named get_java_vm found for type usize`. Scoping
/// the pass to `TypeArray::len` is what makes the two cases distinguishable.
#[test]
fn array_length_const_is_qualified_without_touching_locals() {
    // Stamped stream: names qualify with the origin crate's module.
    check_array_length_qualification(myflat_loc(), "myflat");
}

/// The same contract for an ORIGIN-LESS stream. Core supports hand-built item
/// streams with no `SourceLocation::crate_name` and documents `crate` as their
/// module, so the name set must not be derived from the origin map — those
/// items are absent from it entirely, and deriving from it silently emitted
/// every length bare.
#[test]
fn array_length_qualification_falls_back_to_crate_without_an_origin() {
    check_array_length_qualification(SourceLocation::default(), "crate");
}

fn check_array_length_qualification(loc: SourceLocation, module: &str) {
    let mut items: Vec<(syn::Item, SourceLocation)> = Vec::new();
    items.push((
        syn::Item::Const(syn::parse_quote!(
            #[allow(non_upper_case_globals)]
            pub const env: usize = 4;
        )),
        loc.clone(),
    ));
    items.push((
        syn::Item::Struct(syn::parse_quote!(
            pub struct Blob {
                pub bytes: [u8; env],
            }
        )),
        loc.clone(),
    ));
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn blob_echo(b: Blob) -> Blob {
                unimplemented!()
            }
        )),
        loc.clone(),
    ));
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("blob")
                .class(crate::data_class!(Blob))
                .fun(prebindgen_registry::fun!(blob_echo)),
        );
    let dir = unique_test_dir(&format!("jnigen_array_len_const_{module}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_with(registry).unwrap();
    let rust_path = generation.write_rust(dir.join("gen.rs")).unwrap();
    let rust = std::fs::read_to_string(rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // The length IS qualified — otherwise the generated file names a const
    // that is not in scope.
    assert!(rc.contains(&format!("[u8;{module}::env]")), "{rust}");
    // ...and the identically-named LOCAL is untouched. These two assertions
    // fail in opposite directions, so neither alone pins the behavior: the
    // `env` here is the `&mut JNIEnv` every converter body threads through.
    assert!(rc.contains("env.byte_array_from_slice"), "{rust}");
    assert!(
        !rc.contains(&format!("{module}::env.byte_array_from_slice")),
        "{rust}"
    );
    assert!(!rc.contains(&format!("{module}::env,")), "{rust}");
    assert!(!rc.contains(&format!("&mut{module}::env")), "{rust}");
}

/// A borrowed **transparent wrapper** around a sequence is refused, not decoded as
/// a bare `Vec`.
///
/// `Box<Vec<T>>` and `Cow<'_, [T]>` read as a run of values — correctly: no
/// destination language can tell them from `Vec<T>`, which is why
/// `sequence_elem` answers through the wrapper. But the generated glue **is** a
/// destination artifact, and it is the one consumer that can tell: `&Vec<i32>`
/// is not `&Box<Vec<i32>>`, which is why the wrapper is still standing in
/// `kind` and in `TypeRef::origin` alike.
///
/// So the selector asks two questions, and only the first is `kind`'s: that this
/// is a run of values makes the `Vec` shortcut a *candidate*, and the **spelling**
/// says whether a decoded `Vec<T>` could actually be handed to the function.
/// `&[T]` deref-coerces and `&Vec<T>` is the thing itself; a wrapper is neither.
///
/// **What this pins is the refusal.** A miscompile is not reachable today even
/// without the spelling guard, because the scan requires every nested position and
/// nothing converts `Box<Vec<i32>>` either — so the binding is rejected before
/// anything is emitted, which is the right failure. The guard keeps the selector's
/// reasoning honest for the day that over-approximation is relaxed; this test
/// pins the behaviour a user sees now, and it is deliberately an assertion about
/// *refusing*, not about generated text that no longer exists.
#[test]
fn a_borrowed_transparent_sequence_wrapper_is_not_decoded_as_a_vec() {
    let loc = prebindgen::SourceLocation::default();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_take_boxed(v: &Box<Vec<i32>>) -> i64 {
                v.len() as i64
            }
        )),
        loc.clone(),
    )];
    let decls = crate::package!("ops").fun(crate::FunctionDecl::new(
        syn::parse_str("z_take_boxed").unwrap(),
    ));
    let registry = crate::test_util::reg_from_items(items).expect("index");
    let err = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(decls)
        .build_with(registry)
        .expect_err("a borrowed transparent sequence wrapper has no conversion");

    let msg = err.to_string();
    assert!(
        msg.contains("Box < Vec < i32 > >"),
        "the refusal must name the wrapper spelling the binding cannot convert:\n{msg}"
    );
}

/// The enum probe answers about the **type**, not about the wrapper around it.
///
/// `is_kotlin_enum_reading` peels with [`enum_probe`], which walks the model's
/// own layers, and then keys on `TypeKind::Named` — so every spelling of "a
/// `Priority`, held some way" reaches the `enum_class!(Priority)` declaration.
/// The spelling-keyed `is_kotlin_enum` cannot: `Box<Priority>` canonicalizes to
/// `Box < Priority >`, which no declaration ever registered, and the answer is
/// `false` about a type that IS a Kotlin enum. Both are asserted here, because
/// the difference between them is the reason the reading-taking one exists.
///
/// A **run is not peeled**, and that is the half a `layer_stack`-based probe
/// would get wrong: `Vec<Priority>` is a `List<Priority>` on the Kotlin side, so
/// treating it as an enum would wire a list to a `.value` discriminant.
///
/// `Probe` is deliberately undeclared — jnigen is opt-in, so it emits nothing,
/// and it exists only to give the model a field per spelling to classify.
#[test]
fn the_enum_probe_sees_through_wrappers_a_spelling_key_misses() {
    use prebindgen_registry::flat;

    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 1,
                    High = 2,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Probe {
                    pub plain: Priority,
                    pub borrowed: Box<Priority>,
                    pub optional: Option<Priority>,
                    pub boxed_optional: Box<Option<Priority>>,
                    pub optional_borrow: Option<Box<Priority>>,
                    pub run: Vec<Priority>,
                    pub unrelated: i64,
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::enum_class!(Priority)))
        .build_with(registry)
        .expect("resolve");
    let (ext, registry) = (gen.declarations(), gen.registry());

    let flat::Type::Struct(probe) = registry.flat().declared_type("Probe").expect("indexed") else {
        panic!("Probe is a struct");
    };
    let field = |name: &str| {
        &probe
            .fields
            .iter()
            .find(|f| f.name.as_ref().is_some_and(|n| n == name))
            .unwrap_or_else(|| panic!("field `{name}`"))
            .ty
    };

    for name in [
        "plain",
        "borrowed",
        "optional",
        "boxed_optional",
        "optional_borrow",
    ] {
        let reading = field(name);
        assert!(
            ext.is_kotlin_enum_reading(reading),
            "`{name}` holds a declared Kotlin enum, however it is wrapped — the \
             probe peels the model's layers, so it must say so"
        );
    }

    for name in ["run", "unrelated"] {
        assert!(
            !ext.is_kotlin_enum_reading(field(name)),
            "`{name}` is not an enum value: a run of enums is a `List`, and the \
             probe must not peel the sequence layer to reach the element"
        );
    }

    // The difference from the spelling-keyed probe, pinned: a transparent
    // wrapper is invisible to the model and decisive for a canonical key.
    assert!(
        !ext.is_kotlin_enum_key(&field("borrowed").key()),
        "if the spelling key ever started seeing through `Box`, this test would \
         stop distinguishing the two probes"
    );
    assert!(ext.is_kotlin_enum_key(&field("plain").key()));
}

/// A **transparently-wrapped** parameter takes the same specialized lowering as
/// its bare twin, and the emitter puts the wrapper back.
///
/// The model erases `Box`/`Cow` ([`TRANSPARENT_WRAPPERS`]), so
/// `Box<Option<Mode>>` classifies as `Optional` exactly as `Option<Mode>` does.
/// But `build_option_scalar_input_plan` does not *decode* the parameter, it
/// **rebuilds** it: the emitter writes a literal `Option::Some(v)` /
/// `Option::None` and hands that to the source function, and handing a bare
/// `Option<Mode>` to a parameter spelled `Box<Option<Mode>>` is an `E0308`.
///
/// #290 closed that by **declining** the wrapped spelling. #292 item 3 replaced
/// the refusal with the rebuild — `Box::new(..)` is exactly what the syntax
/// asks for — so what this pins flipped: the wrapped parameter must now reach
/// the decoupled `(present, value)` wire, *and* the Rust side must re-wrap.
///
/// Asserted on the **pair** in both artifacts so it cannot pass vacuously: the
/// bare twin must take the same Kotlin surface (or the wrapped one proves
/// nothing) and must **not** get a `Box::new` (or the wrap assertion would hold
/// for an emitter that wrapped everything).
#[test]
fn a_transparently_wrapped_option_takes_the_present_value_pair_and_is_rebuilt() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Mode {
                    A,
                    B,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_bare(mode: Option<Mode>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_boxed(mode: Box<Option<Mode>>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::enum_class!(Mode)))
        .package(
            crate::package!("cfg")
                .fun(prebindgen_registry::fun!(z_bare))
                .fun(prebindgen_registry::fun!(z_boxed)),
        );

    let dir = unique_test_dir("jnigen_wrapped_optscalar");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // The wrapped spelling may legitimately fail to resolve a converter of its
    // own — that is a refusal too, and equally not an `E0308`. Only a build that
    // SUCCEEDS can be asked what it emitted.
    let Ok(gen) = jni.build_with(registry) else {
        return;
    };
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The control: the bare twin still takes the decoupled pair, so a refusal
    // below is about the wrapper and not about the fixture failing to reach the
    // specialized path at all.
    assert!(
        kc.contains("zBare(modePresent:Boolean,modeValue:Int"),
        "the bare `Option<Mode>` must still cross as (present, value) — \
         otherwise this test proves nothing about the wrapped one:\n{kotlin}"
    );

    // …and so does the wrapped one. The two spellings are one type to Kotlin,
    // so an identical surface is the whole claim — a wrapper must not cost a
    // parameter its lowering.
    assert!(
        kc.contains("zBoxed(modePresent:Boolean,modeValue:Int"),
        "`Box<Option<Mode>>` must take the same present/value lowering as its \
         bare twin — the model erases the `Box`, and the emitter puts it back \
         rather than declining the shape:\n{kotlin}"
    );

    // The Rust half, which is what makes taking that lowering legal: the
    // rebuilt `Option` is wrapped back up before it reaches the source fn.
    // Without this the extern hands an `Option<Mode>` to a parameter spelled
    // `Box<Option<Mode>>` — `E0308`, and no Kotlin assertion could see it.
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
        .expect("read rust");
    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("letmode=::std::boxed::Box::new(if"),
        "the rebuilt `Option` must be re-wrapped for the spelling:\n{rust}"
    );
    // The control on the Rust side too: exactly ONE of the two externs wraps,
    // so the assertion above is about the spelling and not an unconditional
    // `Box` the emitter adds to everything.
    assert_eq!(
        rc.matches("::std::boxed::Box::new(if").count(),
        1,
        "only the wrapped spelling gets a `Box::new`; the bare twin builds the \
         `Option` and passes it as is:\n{rust}"
    );
}

/// The transparent-wrapper guard runs **before** the model's layers are
/// interpreted, not after.
///
/// A wrapper sits *outside* the layer it wraps, so `Box<&Vec<Foo>>` **reads** as
/// a borrow — `unwrapped` peels the `Box` to answer, and it is the reading, not
/// the kind, that the layers come off. A guard that takes that reading first
/// and forgets the wrapper replaces the argument with the
/// inner sequence reading, whose own spelling is a clean `Vec<Foo>`, and the
/// outer wrapper is never seen: the Vec-build plan is selected, its emitter
/// hands the source fn a `&[Foo]` built from the transient Rust-side `Vec`, and
/// the parameter still spells `Box<&Vec<Foo>>`. That is the same `E0308` class
/// [`a_transparently_wrapped_option_takes_the_present_value_pair_and_is_rebuilt`]
/// covers, reached by peeling in the wrong order.
///
/// So this pins the **ordering**, which the shape-by-shape tests cannot: every
/// layer is checked on the way down, and the outermost is checked first.
#[test]
fn an_outer_wrapper_around_a_reference_is_seen_before_the_layers_are_read() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Foo {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_bare(v: &[Foo]) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_wrapped(v: Box<&Vec<Foo>>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("foo")
                .class(crate::data_class!(Foo))
                .fun(prebindgen_registry::fun!(put_bare))
                .fun(prebindgen_registry::fun!(put_wrapped)),
        );

    let dir = unique_test_dir("jnigen_outer_wrapper_ref");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // The wrapped spelling may legitimately resolve no converter of its own —
    // that is a refusal too, and equally not an `E0308`.
    let Ok(gen) = jni.build_with(registry) else {
        return;
    };
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The control: the bare `&[Foo]` twin still takes the Vec-build handle path,
    // so a refusal below is about the wrapper and not about the fixture failing
    // to reach the specialized path at all.
    assert!(
        kc.contains("fooVecNew"),
        "the bare `&[Foo]` must still take the Vec-build path — otherwise this \
         test proves nothing about the wrapped one:\n{kotlin}"
    );
    assert!(
        kc.contains("val__vec_v=JNINative.fooVecNew(v.size)"),
        "{kotlin}"
    );

    // The finding: `Box<&Vec<Foo>>` must not reach the Vec-build call site. Its
    // wrapper is invisible to `kind` (which says `Ref`), so only a check made
    // before the layers are read can catch it.
    // Split on the WRAPPER, not on `externalfunputWrapped(` in `JNINative` —
    // the extern block is followed by the shared `fooVecNew` declarations, so a
    // looser split would read them as this function's body and pass falsely.
    let wrapped_body = kc
        .split("publicfunputWrapped(")
        .nth(1)
        .map(|s| s.split("publicfun").next().unwrap_or(s).to_string())
        .unwrap_or_default();
    assert!(
        !wrapped_body.contains("fooVecNew"),
        "`Box<&Vec<Foo>>` took the Vec-build path, which hands the source fn a \
         `&[Foo]` built from a transient Vec while the parameter spells \
         `Box<&Vec<Foo>>` — an E0308 in the generated crate:\n{kotlin}"
    );
}

/// The two spellings of a **borrowed run** get the same local, and it is the
/// borrow of the `Vec` rather than a slice of it (#384).
///
/// `sequence_elem` answers for `&[T]` and `&Vec<T>` alike — they are one type to
/// the model — so both reach the Vec-build path. The emitter was not symmetric
/// with that: it ascribed `&[#elem]` to the local, which coerced
/// `&*(.. as *const Vec<T>)` at the `let` and could therefore produce only one
/// of the two. A `&Vec<T>` parameter was then handed a `&[T]`, which does not
/// coerce back — `E0308` in the generated crate.
///
/// Unascribed, the coercion moves to the call site and serves both. What this
/// test can pin is the **shape**; that it compiles is `covertest-kotlin`'s
/// `ref_vec_id_sum`/`slice_id_sum` pair, since a lib test emits tokens and never
/// builds them.
#[test]
fn both_spellings_of_a_borrowed_run_get_the_vec_borrow() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Foo {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_slice(v: &[Foo]) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_ref_vec(v: &Vec<Foo>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("foo")
                .class(crate::data_class!(Foo))
                .fun(prebindgen_registry::fun!(put_slice))
                .fun(prebindgen_registry::fun!(put_ref_vec)),
        );

    let dir = unique_test_dir("jnigen_borrowed_run_spellings");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let gen = jni.build_with(registry).expect("resolve");
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).expect("read rust");
    let rc: String = rust.split_whitespace().collect();

    // Both spellings reach the Vec-build path — the premise. `&Vec<Foo>` did
    // too before the fix, which is exactly why it broke rather than falling
    // back to the general converter.
    for f in ["publicfunputSlice(", "publicfunputRefVec("] {
        let body = kc
            .split(f)
            .nth(1)
            .map(|s| s.split("publicfun").next().unwrap_or(s).to_string())
            .unwrap_or_default();
        assert!(
            body.contains("fooVecNew"),
            "`{f}` must take the Vec-build path — both spellings are one type to \
             the model:\n{kotlin}"
        );
    }

    // The finding: ONE local, and it is the `Vec` borrow. Counted rather than
    // merely found, so a per-spelling form cannot pass.
    assert_eq!(
        rc.matches("letv=unsafe{&*(v_handleas*constVec<myflat::Foo>)};")
            .count(),
        2,
        "both borrowed runs must get the same unascribed `&Vec<Foo>` local:\n{rust}"
    );
    // …and no slice ascription survives, which is the thing that broke.
    assert!(
        !rc.contains("letv:&[myflat::Foo]="),
        "ascribing `&[Foo]` coerces at the `let`, so a `&Vec<Foo>` parameter \
         gets a `&[Foo]` and the generated crate does not build (E0308):\n{rust}"
    );
}

/// A `Box` on the ELEMENT keeps the push-helper fast path, and the two spellings
/// share **one** helper trio (#296).
///
/// The trio stores a `Vec<#elem>` and takes its name from the element's Kotlin
/// class, so keying it on the spelling would make `Vec<Foo>` and `Vec<Box<Foo>>`
/// two storages wanting the one name `fooVec`. That collision is what #294 read
/// as forcing the refusal. Keying on the CANONICAL element dissolves it: one
/// storage, one name, and the `Box` goes back on per element where the Vec is
/// consumed.
///
/// The cost of the old refusal was never correctness — the general converter
/// serves the shape — it was that the crossing silently fell back to a
/// per-element `JObject` plus a field read per field, which is the entire thing
/// this path exists to remove. So the assertion is about **which path**, and the
/// bare twin is carried alongside as the control.
///
/// `&[Box<Foo>]` is asserted absent rather than left untested: it is a
/// deliberate refusal (the by-ref arm borrows the Kotlin-owned Vec, and a
/// `Vec<Box<Foo>>` is a different allocation rather than a different view), and
/// an unpinned refusal decays into an accident.
#[test]
fn a_wrapped_vec_element_keeps_the_push_path_and_shares_one_trio() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Foo {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_bare(v: Vec<Foo>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_boxed_elem(v: Vec<Box<Foo>>) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_boxed_elem_slice(v: &[Box<Foo>]) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("foo")
                .class(crate::data_class!(Foo))
                .fun(prebindgen_registry::fun!(put_bare))
                .fun(prebindgen_registry::fun!(put_boxed_elem))
                .fun(prebindgen_registry::fun!(put_boxed_elem_slice)),
        );

    let dir = unique_test_dir("jnigen_wrapped_vec_elem");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let gen = jni.build_with(registry).expect("resolve");
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).expect("read rust");
    let rc: String = rust.split_whitespace().collect();

    // The control: the bare `Vec<Foo>` twin takes the Vec-build path, so a
    // finding below is about the wrapper and not about the fixture failing to
    // reach the specialized path at all.
    assert!(
        kc.contains("val__vec_v=JNINative.fooVecNew(v.size)"),
        "the bare `Vec<Foo>` must take the Vec-build path — otherwise this test \
         proves nothing about the wrapped one:\n{kotlin}"
    );
    // The finding: the wrapped element takes it too. Split on the wrapper so the
    // shared `fooVecNew` declarations in `JNINative` cannot be read as this
    // function's body (the trap the sibling test above documents).
    let boxed_body = kc
        .split("publicfunputBoxedElem(")
        .nth(1)
        .map(|s| s.split("publicfun").next().unwrap_or(s).to_string())
        .unwrap_or_default();
    assert!(
        boxed_body.contains("fooVecNew"),
        "`Vec<Box<Foo>>` fell back to the general `JObject` converter — a `Box` \
         the model erases must not cost the push-helper path (#296):\n{kotlin}"
    );

    // One trio, not two: the declaration is emitted once for the two spellings.
    // This is the collision claim, measured rather than argued.
    assert_eq!(
        kc.matches("externalfunfooVecNew(cap:Int):Long").count(),
        1,
        "the two spellings must share ONE helper trio — a per-spelling trio \
         would emit `fooVecNew` twice and collide:\n{kotlin}"
    );
    // …and the storage is the canonical element on the Rust side, which is what
    // makes one trio serve both.
    assert!(
        rc.contains("Vec::<myflat::Foo>::with_capacity"),
        "the trio must store the CANONICAL element:\n{rust}"
    );
    assert!(
        rc.contains(".map(|__e|::std::boxed::Box::new(__e))"),
        "the element wrapper must go back on where the Vec is consumed:\n{rust}"
    );
    // The bare twin must NOT gain that pass — the wrap is per-spelling, not
    // something the emitter adds to every Vec-build param.
    assert_eq!(
        rc.matches(".map(|__e|::std::boxed::Box::new(__e))").count(),
        1,
        "only the wrapped spelling maps its elements:\n{rust}"
    );

    // The stated refusal: the by-ref arm borrows the Kotlin-owned `Vec<Foo>`,
    // and there is no `Vec<Box<Foo>>` to borrow without building one.
    let slice_body = kc
        .split("publicfunputBoxedElemSlice(")
        .nth(1)
        .map(|s| s.split("publicfun").next().unwrap_or(s).to_string())
        .unwrap_or_default();
    assert!(
        !slice_body.contains("fooVecNew"),
        "`&[Box<Foo>]` must keep the general converter path — serving it would \
         mean consuming the Vec the arm exists to borrow:\n{kotlin}"
    );
}
