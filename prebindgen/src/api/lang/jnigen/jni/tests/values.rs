use super::*;

#[test]
fn bounded_duration_option_uses_u64_niche_without_boxing() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn duration_from_millis(v: u64) -> std::time::Duration { unimplemented!() }",
        "pub fn duration_to_millis(v: &std::time::Duration) -> u64 { unimplemented!() }",
        "pub fn duration_echo(v: Option<std::time::Duration>) -> Option<std::time::Duration> { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        let function: syn::ItemFn = syn::parse_str(source).unwrap();
        (syn::Item::Fn(function), loc.clone())
    })
    .collect();
    let registry = Registry::<KotlinMeta>::from_items(items).unwrap();
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(std::time::Duration)
                .input(crate::fun!(duration_from_millis))
                .output(crate::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .package(crate::package!("time").fun(crate::fun!(duration_echo)));
    let dir = unique_test_dir("jnigen_bounded_duration");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = registry.resolve(jni).unwrap();
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
                    pub delay: Option<std::time::Duration>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_from_millis(v: u64) -> std::time::Duration {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn duration_to_millis(v: &std::time::Duration) -> u64 {
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
            loc,
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(std::time::Duration)
                .input(crate::fun!(duration_from_millis))
                .output(crate::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .package(
            crate::package!()
                .class(crate::data_class!(Timed))
                .fun(crate::fun!(timed_use)),
        );
    let dir = unique_test_dir("jnigen_flat_staged_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = registry.resolve(jni).expect("resolve");
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
    assert!(rc.contains("jlong_to_u64"), "{rust}");
    assert!(rc.contains("u64_to_Duration"), "{rust}");
    assert!(
        rc.contains("myflat::Timed{delay:__flat_value_delay"),
        "{rust}"
    );
}

#[test]
fn duration_requires_an_explicit_conversion() {
    let function: syn::ItemFn = syn::parse_str(
        "pub fn duration_echo(v: std::time::Duration) -> std::time::Duration { unimplemented!() }",
    )
    .unwrap();
    let registry = Registry::<KotlinMeta>::from_items([(syn::Item::Fn(function), myflat_loc())])
        .expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("time").fun(crate::fun!(duration_echo)));

    let error = registry
        .resolve(jni)
        .expect_err("Duration must not have an implicit unchecked converter")
        .to_string();
    assert!(error.contains("Duration"), "{error}");
}

#[test]
#[should_panic(expected = "domain type i64 does not match input representation u64")]
fn conversion_domain_must_match_the_representation() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn duration_from_millis(v: u64) -> std::time::Duration { unimplemented!() }",
        "pub fn duration_use(v: std::time::Duration) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        let function: syn::ItemFn = syn::parse_str(source).unwrap();
        (syn::Item::Fn(function), loc.clone())
    })
    .collect();
    let registry = Registry::<KotlinMeta>::from_items(items).unwrap();
    let jni = JniGen::new()
        .convert(
            crate::convert!(std::time::Duration)
                .input(crate::fun!(duration_from_millis))
                .valid_range(0i64..=1_000i64),
        )
        .package(crate::package!("time").fun(crate::fun!(duration_use)));

    let _ = registry.resolve(jni);
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::enum_class!(Mode)))
        .package(crate::package!("cfg").fun(crate::fun!(z_set_timeout)));

    let dir = unique_test_dir("jnigen_optscalar");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!("thing")
            .class(crate::ptr_class!(ZThing))
            .fun(crate::fun!(thing_list))
            .fun(crate::fun!(thing_list_opt)),
    );

    let dir = unique_test_dir("jnigen_vec_handle_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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

    // A `ZThingFolder<A>` interface is generated, and the wrapper returns a typed
    // list, allocating the `ArrayList<ZThing>` accumulator on the Kotlin side.
    assert!(kc.contains("interfaceZThingFolder<A>"), "{kotlin}");
    assert!(kc.contains("List<ZThing>"), "{kotlin}");
    assert!(kc.contains("ArrayList<ZThing>()"), "{kotlin}");
    // The folder singleton wraps each raw `jlong` element into the typed handle
    // class and appends it — no Rust object construction.
    assert!(
        kc.contains("ZThing(element)") || kc.contains("acc.add(ZThing("),
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::data_class!(Opts))
            .fun(crate::fun!(opts_put)),
    );

    let dir = unique_test_dir("jnigen_optfield");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("model")
                .class(crate::enum_class!(Level))
                .class(crate::data_class!(Inner))
                .class(crate::data_class!(Job)),
        )
        .package(
            crate::package!("job")
                .fun(crate::fun!(job_make))
                .fun(crate::fun!(job_mode)),
        );

    let dir = unique_test_dir("jnigen_fromparts_optbox");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::data_class!(FlatChild))
            .class(crate::data_class!(ObjectChild).jobject_input())
            .class(crate::data_class!(Hybrid))
            .fun(crate::fun!(hybrid_use))
            .fun(crate::fun!(hybrid_optional)),
    );
    let dir = unique_test_dir("jnigen_hybrid_jobject_input");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::ptr_class!(Token))
            .class(crate::data_class!(Envelope))
            .fun(crate::fun!(envelope_use)),
    );
    let dir = unique_test_dir("jnigen_recursive_handles");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items([
        (syn::Item::Struct(wide.clone()), loc.clone()),
        (syn::Item::Fn(use_wide.clone()), loc.clone()),
    ])
    .expect("index items");
    let jni = JniGen::new().package(
        crate::package!()
            .class(crate::data_class!(Wide))
            .fun(crate::fun!(use_wide)),
    );
    let error = registry
        .resolve(jni)
        .expect_err("256 JVM slots must fail")
        .to_string();
    assert!(error.contains("uses 256 JVM parameter slots"), "{error}");
    assert!(error.contains("jobject_input"), "{error}");

    // The explicit object boundary keeps the same public Kotlin data class,
    // but the native method receives it in one slot and performs the legacy
    // whole-object field decode instead of producing an illegal signature.
    let registry = Registry::<KotlinMeta>::from_items([
        (syn::Item::Struct(wide), loc.clone()),
        (syn::Item::Fn(use_wide), loc),
    ])
    .expect("index marked items");
    let jni = JniGen::new().package(
        crate::package!()
            .class(crate::data_class!(Wide).jobject_input())
            .fun(crate::fun!(use_wide)),
    );
    let generation = registry
        .resolve(jni)
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(crate::convert!(Len).output(crate::fun!(len_value)))
        .package(crate::package!("len").fun(crate::fun!(len_of)));
    let dir = unique_test_dir("jnigen_outonly_convert");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry
        .resolve(jni)
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
        Registry::<KotlinMeta>::from_items(flat.into_iter().chain(helpers)).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(crate::convert!(Len).output(crate::fun!(len_value)))
        .package(crate::package!("len").fun(crate::fun!(len_of)));
    let dir = unique_test_dir("jnigen_convert_origin");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .convert(crate::convert!(Len).input(crate::fun!(from_long)))
        .package(crate::package!("len").fun(crate::fun!(use_len)));
    let dir = unique_test_dir("jnigen_convert_mismatch");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = registry
        .resolve(jni)
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(Celsius)
                .input(crate::from!(i32))
                .output(crate::into!(i32)),
        )
        .package(crate::package!("m").fun(crate::fun!(temp_double)));
    let dir = unique_test_dir("jnigen_convert_trait");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(crate::convert!(Percent).input(crate::try_from!(i32)))
        .package(crate::package!("m").fun(crate::fun!(pct_use)));
    let dir = unique_test_dir("jnigen_convert_tryfrom");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(Percent)
                .input(crate::try_from!(i32))
                .output(
                    crate::fun!(crate::conv::pct_out)
                        .sig(crate::sig!((p: Percent) -> Result<i32, String>)),
                ),
        )
        .package(crate::package!("m").fun(crate::fun!(pct_optional)));
    let dir = unique_test_dir("jnigen_option_fallible_stages");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(Label)
                .input(crate::fun!(crate::conv::label_in).sig(crate::sig!((s: String) -> Label)))
                .output(crate::fun!(crate::conv::label_out).sig(crate::sig!((l: Label) -> String))),
        )
        .package(crate::package!("m").fun(crate::fun!(label_id)));
    let dir = unique_test_dir("jnigen_convert_local");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let _ = crate::convert!(Widget)
        .input(crate::from!(i32))
        .input(crate::fun!(crate::widget_in).sig(crate::sig!((v: String) -> Widget)));
}

/// The source macros state their direction; the acceptor cross-checks it.
#[test]
#[should_panic(expected = "an input conversion is built with from!/try_from!")]
fn convert_input_into_direction_rejected() {
    let _ = crate::convert!(Widget).input(crate::into!(i32));
}

#[test]
#[should_panic(expected = "an output conversion is built with into!/try_into!")]
fn convert_output_from_direction_rejected() {
    let _ = crate::convert!(Widget).output(crate::from!(i32));
}

/// A binding-local conversion source must state its signature — a path
/// carries nothing to read (the sig's `Result<_, E>` is the error channel,
/// replacing the former `.error(ty!)`).
#[test]
#[should_panic(expected = ".sig(sig!(")]
fn convert_local_source_missing_sig_rejected() {
    let _ = crate::convert!(Widget).input(crate::fun!(crate::widget_in));
}

/// A `fun!` conversion source is never surfaced in Kotlin — decorations are
/// rejected at the source seam (same policy as ignore/variant/field).
#[test]
#[should_panic(expected = ".name()/expand overrides don't apply")]
fn convert_source_fun_with_decorations_rejected() {
    let _ = crate::convert!(Widget).input(crate::fun!(widget_in).name("widgetIn"));
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
        Registry::<KotlinMeta>::from_items(vec![(syn::Item::Fn(f), loc)]).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .convert(
            crate::convert!(Label)
                .input(
                    crate::fun!(crate::conv::label_in)
                        .sig(crate::sig!((s: String) -> Result<Label, String>)),
                )
                .output(crate::fun!(crate::conv::label_out).sig(crate::sig!((l: Label) -> String))),
        )
        .package(crate::package!("m").fun(crate::fun!(label_id)));
    let dir = unique_test_dir("jnigen_convert_local_try");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!().class(
            crate::data_class!(Point)
                .method(crate::fun!(point_norm).name("norm"))
                .constructor(crate::fun!(point_origin).name("origin")),
        ),
    );
    let dir = unique_test_dir("jnigen_data_members");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
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
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::data_class!(Unsigned))
            .fun(crate::fun!(unsigned_round_trip))
            .fun(crate::fun!(unsigned_data_maybe))
            .fun(crate::fun!(unsigned_callback))
            .fun(crate::fun!(unsigned_result)),
    );
    let dir = unique_test_dir("jnigen_unsigned");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = registry.resolve(jni).expect("resolve");
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
