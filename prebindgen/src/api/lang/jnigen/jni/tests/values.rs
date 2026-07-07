use super::*;

/// Phase 4: a bare `Option<primitive>` / `Option<enum>` **input** parameter
/// crosses as a decoupled `(present: Boolean, value: <prim>)` pair instead of a
/// boxed `java.lang.*` `JObject`. The Rust side reassembles the `Option` from
/// two raw scalars (`if <p>_present != 0u8 { Some(..) } else { None }`) with no
/// reflective `intValue()`/`longValue()` unbox. The public Kotlin signature
/// keeps `T?`; the call site passes `<name> != null` and `<name> ?: <zero>`
/// (`<name>?.value ?: 0` for an enum).
#[test]
fn option_scalar_param_crosses_as_present_value_pair() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::enum_class!(Mode)))
        .package(crate::package!("cfg").fun(crate::fun!(z_set_timeout)));

    let dir = unique_test_dir("jnigen_optscalar");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
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
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing))
                .fun(crate::fun!(thing_list))
                .fun(crate::fun!(thing_list_opt)),
        );

    let dir = unique_test_dir("jnigen_vec_handle_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
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
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Opts))
                .fun(crate::fun!(opts_put)),
        );

    let dir = unique_test_dir("jnigen_optfield");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
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
        rc.contains("myflat::Opts{id:__o_id,ttl:__o_ttl,flag:__o_flag"),
        "{rust}"
    );
    assert!(rc.contains("myflat::opts_put(&o)"), "{rust}");
}

/// A `data_class` with a NESTED data-class field plus enum / `Option<prim>` /
/// `Option<enum>` fields — the shape that declines BOTH the fixed-builder
/// output synthesis and the input leaf-flatten, so it round-trips through the
/// whole-value `fromParts` / `get_field` converters. Pins three fixes those
/// paths needed (each surfaced at runtime by `examples/covertest-kotlin`):
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
fn fromparts_fallback_boxes_option_fields() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
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
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
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

    // INPUT (`job_mode`'s whole-`Job` param): every `get_field` names the
    // slot's exact static type; enum-typed slots decode via `getValue()I`.
    assert!(
        rc.contains(r#"get_field(v,"inner","Lio/test/jni/model/Inner;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"level","Lio/test/jni/model/Level;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"ttl","Ljava/lang/Long;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"mode","Lio/test/jni/model/Level;")"#),
        "{rust}"
    );
    assert!(rc.contains(r#""getValue","()I""#), "{rust}");
    assert!(!rc.contains("Ljava/lang/Object;\")"), "{rust}");

    // RETURN (`job_mode` → `Option<Level>`): the extern wires `Int?`; the
    // wrapper maps the boxed discriminant back to the nullable enum.
    assert!(
        kc.contains("funjobMode(j:Job,errorSink:Any):Int?"),
        "{kotlin}"
    );
    assert!(kc.contains("?.let{Level.fromInt(it)}"), "{kotlin}");
}

/// An output-only wrapper type must resolve with only its `.output()`
/// registered: wrapper registrations are required per USAGE direction, unlike
/// the four class declarators (always both). Regression: registering the
/// wrapper used to add the type to `declared_types`, and the scan blanket-
/// required both directions.
#[test]
fn output_only_wrapper_resolves_without_input_twin() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn len_of(s: &String) -> Len {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .scalar_type_wrapper(
            crate::scalar_type_wrapper!(Len, jni::sys::jlong, "Long")
                .on_return(|v| syn::parse_quote!(#v.0 as jni::sys::jlong)),
        )
        .package(crate::package!("len").fun(crate::fun!(len_of)));
    let dir = unique_test_dir("jnigen_outonly_wrapper");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("an output-only wrapper type must not require an input twin");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();
    // The return crosses through the registered output wrapper (jlong wire).
    assert!(rc.contains("Len_to_jlong"), "{rust}");
    assert!(rc.contains("myflat::len_of(&s)"), "{rust}");
}
