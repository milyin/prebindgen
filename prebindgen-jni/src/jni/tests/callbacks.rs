use prebindgen_registry::Conversions;

use super::*;

fn callback_snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_name(this_: &ZThing) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_sub(
                    cb: impl Fn(ZThing) + Send + Sync + 'static,
                    on_close: impl Fn() + Send + Sync + 'static,
                ) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_other_sub(cb: impl Fn(ZOther) + Send + Sync + 'static) {
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
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                // ZOther: plain ptr_class, no canonical output ⇒ whole-handle fallback.
                .class(crate::ptr_class!(ZOther))
                .fun(prebindgen_registry::fun!(z_thing_sub))
                .fun(prebindgen_registry::fun!(z_other_sub)),
        )
        // Canonical output: handle (identity) + its string form — a callback
        // arg of ZThing decomposes into these 2 leaves.
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field_self()
                .field(prebindgen_registry::fun!(z_thing_name)),
        );

    let dir = unique_test_dir("jnigen_cb_snap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let gen = jni.build_over(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let mut kotlin = std::collections::BTreeMap::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        kotlin.insert(name, std::fs::read_to_string(p).unwrap());
    }
    (rust, kotlin)
}

#[test]
fn callback_snapshot_rust_side() {
    let (rust, _) = callback_snapshot_pipeline();
    let rc: String = rust.split_whitespace().collect();
    // The trampoline invokes the typed callback interface's `run` — never the
    // erased `FunctionN.invoke`.
    assert!(rc.contains(r#""run""#), "{rust}");
    assert!(!rc.contains(r#""invoke""#), "{rust}");
    // Decomposed ZThing arg: 2 typed leaves (raw jlong handle + String),
    // void return; on_close ⇒ zero-arg `()V`.
    assert!(rc.contains(r#""(JLjava/lang/String;)V""#), "{rust}");
    assert!(rc.contains(r#""()V""#), "{rust}");
    // Plan-less ZOther arg (Phase 3): crosses as a raw `jlong` (`(J)V`), NOT a
    // boxed handle object — so the Rust trampoline neither `new_object`s the
    // typed class nor `close()`s it (the Kotlin `asRaw` proxy wraps + closes).
    assert!(rc.contains(r#""(J)V""#), "{rust}");
    assert!(!rc.contains(r#""close""#), "{rust}");
    assert!(!rc.contains("io/test/jni/thing/ZOther"), "{rust}");
    // Daemon-thread attachment + local-frame bracketing kept from the old
    // trampoline.
    assert!(rc.contains("attach_current_thread_as_daemon"), "{rust}");
    assert!(rc.contains("push_local_frame"), "{rust}");
    assert!(rc.contains("pop_local_frame"), "{rust}");
    // Identity leaf of the decomposed arg: moved into a fresh box and crosses
    // as a RAW jlong jvalue — no native `new_object` of the typed class.
    assert!(rc.contains("jni::sys::jvalue{j:"), "{rust}");
    assert!(!rc.contains("io/test/jni/thing/ZThing"), "{rust}");
    // The decomposed leaf encode calls the accessor off the owned root.
    assert!(rc.contains("myflat::z_thing_name"), "{rust}");
}

#[test]
fn callback_snapshot_kotlin_side() {
    let (_, kotlin) = callback_snapshot_pipeline();
    let names: Vec<&String> = kotlin.keys().collect();

    // Extern tier: callbacks erased to `Any`, like the errorSink.
    let native: String = kotlin
        .values()
        .find(|v| v.contains("object JNINative"))
        .map(|v| v.split_whitespace().collect())
        .unwrap_or_else(|| {
            panic!("no generated file contains `object JNINative`; files: {names:?}")
        });
    assert!(native.contains("cb:Any"), "{native}");
    assert!(native.contains("onClose:Any"), "{native}");

    // Typed callback `fun interface`s with NAMED parameters — decomposed
    // ZThing's identity leaf is `handle`, its accessor leaf carries the literal
    // author-supplied name (`z_thing_name` declared as `"name"`); `Fn()` ⇒
    // the shared zero-arg `VoidCallback` (root package); the plan-less
    // fallback arg is the decapped type short (`zOther`).
    let all: String = kotlin
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    assert!(
        all.contains("funinterfaceZThingCallback{publicfunrun(handle:ZThing,name:String)"),
        "{all}"
    );
    assert!(
        all.contains("funinterfaceZOtherCallback{publicfunrun(zOther:ZOther)"),
        "{all}"
    );
    assert!(
        all.contains("funinterfaceVoidCallback{publicfunrun()"),
        "{all}"
    );
    // Raw twin + proxy adapter for the decomposed-arg callback (raw jlong
    // handle at the wire); the all-passthrough interfaces get no twin.
    assert!(
        all.contains("funinterfaceZThingCallbackRaw{publicfunrun(handle:Long,name:String)"),
        "{all}"
    );
    assert!(
        all.contains("funZThingCallback.asRaw():ZThingCallbackRaw=ZThingCallbackRaw{handle,name->run(ZThing.fromRawPtr(handle),name)}"),
        "{all}"
    );
    // Plan-less ZOther arg (Phase 3): a raw twin `run(zOther: Long)` + an `asRaw`
    // proxy that wraps the pointer into the handle class AND `close()`s it in a
    // `finally` (close-unless-taken) — the Rust side delivers only the raw jlong.
    assert!(
        all.contains("funinterfaceZOtherCallbackRaw{publicfunrun(zOther:Long)"),
        "{all}"
    );
    assert!(all.contains("val__own0=ZOther.fromRawPtr(zOther)"), "{all}");
    assert!(all.contains("finally{__own0.close()}"), "{all}");
    assert!(!all.contains("VoidCallbackRaw"), "{all}");

    // Wrapper tier: the params are the typed interfaces, forwarded bare.
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingSub"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZThingCallback"), "{pkg}");
    assert!(pc.contains("cb.asRaw()"), "{pkg}");
    assert!(pc.contains("onClose:VoidCallback"), "{pkg}");
    assert!(pc.contains("cb:ZOtherCallback"), "{pkg}");
}
#[test]

fn declared_optional_conversion_callback_falls_back_to_whole_value() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct Payload { pub id: i64 }",
        "pub fn optional_from_wire(v: i64) -> Option<Payload> { unimplemented!() }",
        "pub fn optional_to_wire(v: &Option<Payload>) -> i64 { unimplemented!() }",
        "pub fn optional_sub(cb: impl Fn(Option<Payload>) + Send + Sync + 'static) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
    .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .convert(
            prebindgen_registry::convert!(Option<Payload>)
                .input(prebindgen_registry::fun!(optional_from_wire))
                .output(prebindgen_registry::fun!(optional_to_wire)),
        )
        .package(
            crate::package!()
                .class(crate::data_class!(Payload))
                .fun(prebindgen_registry::fun!(optional_sub)),
        );

    let dir = unique_test_dir("jnigen_declared_optional_callback");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_over(registry).expect("resolve");
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

    assert!(
        rc.contains("optional_to_wire"),
        "the callback must use the declared whole Optional conversion:\n{rust}"
    );
    assert!(
        !rc.contains("Option_Payload_to_tuple"),
        "the suppressed parts row must not be synthesized:\n{rust}"
    );
    assert!(
        kc.contains("funinterfacePayloadCallback"),
        "whole-value fallback keeps the established callback identity:\n{kotlin}"
    );
    assert!(!kc.contains("PayloadOptionalCallback"), "{kotlin}");
    assert!(!kc.contains("payloadPresent:Boolean"), "{kotlin}");
}

/// `Ledger` has an output expansion but is deliberately not a declared Kotlin
/// class. Its fields reach a consuming `Report` value form through two
/// conditional (`Option`) accessors. The registry's deconstruction plan is its
/// only JNI representation, and Invoke must retain that plan without a
/// callback-specific compatibility row.
#[test]
fn undeclared_expanded_callback_retains_its_registry_invoke_plan() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct ReportStruct { pub label: String }",
        "pub fn report_into_struct(r: Report) -> ReportStruct { unimplemented!() }",
        "pub fn ledger_filed(l: &Ledger) -> Option<&Report> { unimplemented!() }",
        "pub fn ledger_archived(l: &Ledger) -> Option<Report> { unimplemented!() }",
        "pub fn ledger_each(sink: impl Fn(Ledger) + Send + Sync + 'static) { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
    .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Report))
                .fun(prebindgen_registry::fun!(ledger_each)),
        )
        .expand(
            prebindgen_registry::expand_return!(Report)
                .fields_self_into(prebindgen_registry::fields!(report_into_struct)),
        )
        .expand(
            prebindgen_registry::expand_return!(Ledger)
                .field(prebindgen_registry::fun!(ledger_filed))
                .field(prebindgen_registry::fun!(ledger_archived)),
        );

    let dir = unique_test_dir("jnigen_ledger_callback_compatibility");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let generation = jni.build_over(registry).expect("resolve");
    let callback = "impl Fn(Ledger) + Send + Sync + 'static";
    let (converter, is_late_invoke) = generation
        .callback_invoke_for_test(callback)
        .expect("Ledger callback is compiled through its ordinary crossing");
    assert!(
        is_late_invoke,
        "the ordinary callback fragment must retain an unrendered Invoke plan"
    );

    let rust = std::fs::read_to_string(generation.write_rust(dir.join("gen.rs")).unwrap()).unwrap();
    assert_eq!(
        rust.matches(&format!("fn {converter}")).count(),
        1,
        "the retained callback plan must be emitted:\n{rust}"
    );
    assert!(rust.contains("ledger_filed(&__cb_arg0)"), "{rust}");
    assert!(rust.contains("ledger_archived(&__cb_arg0)"), "{rust}");
}

/// Planning an Invoke may freeze Flat/model and JNI ABI facts, but only the
/// final `RustFunction::render` boundary may receive a `RustWriter`. Pin both sides of
/// that seam: the recipe hook does not ask its compiler context for spelling,
/// and the adapter planner cannot accept a `RustWriter` parameter later.
#[test]
fn callback_planning_has_no_source_spelling_access() {
    // These delimiters deliberately follow rustfmt's stable spelling of both
    // private signatures. If either signature changes, update this fence's
    // boundary before deciding whether the new capability is still forbidden.
    let compile = include_str!("../compile.rs");
    let callback_hook = compile
        .split_once("    fn callback(\n")
        .expect("callback recipe hook")
        .1
        .split_once("    /// One site:")
        .expect("end of callback recipe hook")
        .0;
    assert!(!callback_hook.contains(".emit()"), "{callback_hook}");

    let callback = include_str!("../emit/callback.rs");
    let planner_signature = callback
        .split_once("pub(crate) fn callback_input(\n")
        .expect("callback planner")
        .1
        .split_once(") -> Option<(syn::Type, JInvokePlan)>")
        .expect("callback planner signature")
        .0;
    assert!(
        !planner_signature.contains("RustWriter"),
        "{planner_signature}"
    );
}

/// Regression: a callback-delivered type that has BOTH a nested handle identity
/// (a child `ptr_class` reached by an accessor) AND its own root identity
/// (`expand_return!` `.field_self()`) must emit the root MOVE after every borrow of
/// the owned value — otherwise the nested child clone (which borrows the root)
/// follows `Box::into_raw(Box::new(value))` and fails to compile with "use of
/// moved value". Declaring `.field_self()` LAST guarantees the
/// correct order (the emitter emits identity leaves in declaration order, after
/// all non-identity leaves). This mirrors the zenoh-flat `ZQuery` queryable
/// callback (handle + decomposed fields, nested `ZKeyExpr` identity).
#[test]
fn callback_root_identity_moved_after_nested_borrow() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_parent_child(this_: &ZParent) -> &ZChild {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_child_name(this_: &ZChild) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_parent_sub(
                    cb: impl Fn(ZParent) + Send + Sync + 'static,
                    on_close: impl Fn() + Send + Sync + 'static,
                ) {
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
                .class(
                    crate::ptr_class!(ZChild)
                        .method(prebindgen_registry::fun!(z_child_name).name("name")),
                )
                .class(
                    crate::ptr_class!(ZParent)
                        .method(prebindgen_registry::fun!(z_parent_child).name("child")),
                )
                .fun(prebindgen_registry::fun!(z_parent_sub)),
        )
        // Child handle: canonical output = identity (clone) + its name string.
        .expand(
            prebindgen_registry::expand_return!(ZChild)
                .field_self()
                .field(prebindgen_registry::fun!(z_child_name)),
        )
        // Parent: a nested child-handle record, then its OWN root identity LAST.
        .expand(
            prebindgen_registry::expand_return!(ZParent)
                .field(prebindgen_registry::fun!(z_parent_child))
                .field_self(),
        );

    let dir = unique_test_dir("jnigen_root_id_order");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_over(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // The root `ZParent` identity is a move (`Box::new(__cb_arg0)`); the nested
    // `ZChild` identity (and its `z_child_name` leaf) borrow the same owned arg
    // via `z_parent_child(&__cb_arg0)`. Every borrow must precede the move.
    let move_pos = rc
        .find("Box::new(__cb_arg0")
        .unwrap_or_else(|| panic!("root identity move not found in:\n{rust}"));
    let last_borrow = rc
        .rfind("z_parent_child(&__cb_arg0")
        .unwrap_or_else(|| panic!("nested child borrow not found in:\n{rust}"));
    assert!(
        last_borrow < move_pos,
        "root identity move must follow every borrow of the owned arg\n{rust}"
    );
}

/// ZReply-shaped product decomposition: the callback arg's plan contains leaf
/// paths with MULTIPLE `Option`-returning nesting steps (`z_reply_sample` →
/// `z_sample_timestamp`), a nested handle identity reached *through* an
/// `Option` step (`z_reply_sample` → `z_sample_key_expr`), and an Acc leaf
/// whose own return keeps its full `Option<…>` as the converter input
/// (`z_reply_zid -> Option<ZId>`, a data class with no canonical child).
/// Every `Option` nesting step must become its own `match` (`None` ⇒ null
/// leaf) — never a blind accessor compose through an `Option`.
#[test]
fn callback_double_option_unwrap_pipeline() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_reply_zid(r: &ZReply) -> Option<ZId> { unimplemented!() }",
        "pub fn z_reply_is_ok(r: &ZReply) -> bool { unimplemented!() }",
        "pub fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { unimplemented!() }",
        "pub fn z_reply_err(r: &ZReply) -> Option<&ZErr> { unimplemented!() }",
        "pub fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { unimplemented!() }",
        "pub fn z_sample_timestamp(s: &ZSample) -> Option<&ZTs> { unimplemented!() }",
        "pub fn z_ts_ntp64(t: &ZTs) -> i64 { unimplemented!() }",
        "pub fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { unimplemented!() }",
        "pub fn z_err_payload(e: &ZErr) -> Vec<u8> { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    // `ZId` is the value leaf of the outer `Option`; it needs a real struct so
    // it can be a `data_class!`.
    items.push((
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZId {
                pub hi: i64,
                pub lo: i64,
            }
        )),
        loc.clone(),
    ));
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_get(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    ));
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("query")
                .class(crate::data_class!(ZId))
                .class(
                    crate::ptr_class!(ZKeyExpr)
                        .method(prebindgen_registry::fun!(z_keyexpr_as_str).name("asStr")),
                )
                .class(
                    crate::ptr_class!(ZTs)
                        .method(prebindgen_registry::fun!(z_ts_ntp64).name("ntp64")),
                )
                .class(
                    crate::ptr_class!(ZSample)
                        .method(prebindgen_registry::fun!(z_sample_key_expr).name("keyExpr"))
                        .method(prebindgen_registry::fun!(z_sample_timestamp).name("timestamp")),
                )
                .class(
                    crate::ptr_class!(ZErr)
                        .method(prebindgen_registry::fun!(z_err_payload).name("payload")),
                )
                .class(
                    crate::ptr_class!(ZReply)
                        .method(prebindgen_registry::fun!(z_reply_zid).name("zid"))
                        .method(prebindgen_registry::fun!(z_reply_is_ok).name("isOk"))
                        .method(prebindgen_registry::fun!(z_reply_sample).name("sample"))
                        .method(prebindgen_registry::fun!(z_reply_err).name("err")),
                )
                .fun(prebindgen_registry::fun!(z_get)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZKeyExpr)
                .field_self()
                .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZTs).field(prebindgen_registry::fun!(z_ts_ntp64)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZSample)
                .field(prebindgen_registry::fun!(z_sample_key_expr))
                .field(prebindgen_registry::fun!(z_sample_timestamp)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZErr)
                .field(prebindgen_registry::fun!(z_err_payload)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZReply)
                .field(prebindgen_registry::fun!(z_reply_zid))
                .field(prebindgen_registry::fun!(z_reply_is_ok))
                .field(prebindgen_registry::fun!(z_reply_sample))
                .field(prebindgen_registry::fun!(z_reply_err)),
        );

    let dir = unique_test_dir("jnigen_double_opt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_over(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Both Option nesting steps of the timestamp leaf get their own match;
    // the innermost accessor composes off the second unwrap binding.
    assert!(rc.contains("matchmyflat::z_reply_sample("), "{rust}");
    assert!(rc.contains("matchmyflat::z_sample_timestamp("), "{rust}");
    assert!(rc.contains("myflat::z_ts_ntp64(__n1)"), "{rust}");
    // Never a blind compose through an `Option`-returning accessor.
    assert!(
        !rc.contains("myflat::z_ts_ntp64(myflat::z_sample_timestamp("),
        "{rust}"
    );
    assert!(
        !rc.contains("myflat::z_sample_key_expr(myflat::z_reply_sample("),
        "{rust}"
    );
    // The nested keyexpr identity is reached through the `Option` unwrap and
    // has a null arm.
    assert!(rc.contains("myflat::z_sample_key_expr(__n0)"), "{rust}");
    assert!(rc.contains("jni::objects::JObject::null()"), "{rust}");
    // The `Option<ZId>` Acc leaf composes its full return directly into the
    // converter — no unwrap of the leaf's own `Option`.
    assert!(rc.contains("myflat::z_reply_zid(&__cb_arg0)"), "{rust}");
    assert!(!rc.contains("matchmyflat::z_reply_zid("), "{rust}");
    // 6 leaves ⇒ typed `run` descriptor: nullable `ZId` data class, raw `Z`
    // for the non-null bool discriminator, typed handle class (full FQN),
    // nullable String, BOXED Long for the nullable timestamp, nullable `[B`.
    assert!(
        rc.contains(
            "\"(Lio/test/jni/query/ZId;ZLjava/lang/Long;Ljava/lang/String;Ljava/lang/Long;[B)V\""
        ),
        "{rust}"
    );
    // The non-null bool crosses as a raw typed jvalue — never boxed.
    assert!(rc.contains("jni::sys::jvalue{z:"), "{rust}");

    // Kotlin tier: the generated callback `fun interface` carries the typed
    // params — ok-arm and err-arm leaves nullable (the value may be absent),
    // the discriminator non-null; the nested `ZId` data class surfaces as its
    // typed (nullable) Kotlin class.
    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let iface_file = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("fun interface ZReplyCallback"))
        .unwrap_or_default();
    // Scope to the interface block — the merged package file also holds the
    // ZId data class and other decls.
    let iface = iface_file
        .split("fun interface ZReplyCallback")
        .nth(1)
        .and_then(|s| s.split_once('}').map(|(b, _)| b.to_string()))
        .unwrap_or_default();
    let ic: String = iface.split_whitespace().collect();
    assert!(ic.contains("isOk:Boolean"), "{iface}");
    assert!(ic.contains("sample__keyExpr:ZKeyExpr?"), "{iface}");
    assert!(ic.contains(":Long?"), "{iface}");
    assert!(ic.contains(":ZId?"), "{iface}");
    // The wrapper takes the typed interface and forwards it bare.
    let pkg = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("public fun zGet"))
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZReplyCallback"), "{pkg}");
    // The call site forwards the generated raw-proxy adapter — the typed
    // interface is the user surface, the extern receives the raw twin.
    assert!(pc.contains("JNINative.zGet(cb.asRaw(),"), "{pkg}");
}

// ────────────────────────────────────────────────────────────────────────
// Declaration-keyed interfaces: a type may have several decompositions —
// the default (unnamed) deconstructor and per-fn inline records
// (`.expand_return`). Interface identity follows the DECLARATION, so
// differently-decomposed functions get distinct interfaces instead of
// colliding on one type-keyed name.
// ────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────
// Spec memo (issue #107): every consumer — resolve-time trampoline,
// per-function plan, declaration emitter — reads ONE derivation per
// interface identity through `JniGenBuilder::iface_spec`.
// ────────────────────────────────────────────────────────────────────────

#[test]
fn iface_spec_memo_shares_one_derivation() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_name(this_: &ZThing) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_things_all() -> Vec<ZThing> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_sub(cb: impl Fn(ZThing) + Send + Sync + 'static) {
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
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                .fun(prebindgen_registry::fun!(z_things_all))
                .fun(prebindgen_registry::fun!(z_thing_sub)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field_self()
                .field(prebindgen_registry::fun!(z_thing_name)),
        );
    let gen = jni.build_over(registry).expect("resolve");
    let (ext, registry) = (gen.declarations(), gen.registry());

    // Same key twice ⇒ the same allocation (resolve already populated the
    // memo through the trampoline — a hit also exercises the debug-build
    // re-derivation assert).
    let a = ext
        .iface_spec(registry, &SpecKey::JniErrorHandler)
        .expect("global handler spec");
    let b = ext
        .iface_spec(registry, &SpecKey::JniErrorHandler)
        .expect("global handler spec");
    assert!(Arc::ptr_eq(&a, &b), "one derivation per identity");

    // The plan-facing dispatcher and a direct key lookup share one
    // allocation — the wrapper surface and the interface declaration cannot
    // diverge from the fold upcall's descriptor.
    let plan = ext
        .unfolded()
        .unfold_plans
        .get(&syn::parse_str::<syn::Ident>("z_things_all").unwrap())
        .expect("fold plan");
    let via_plan = folder_iface_for_plan(ext, registry, plan).expect("folder spec");
    let decon = plan.decon.clone().expect("record-built fold");
    let direct = ext
        .iface_spec(registry, &SpecKey::Folder(decon))
        .expect("folder spec");
    assert!(Arc::ptr_eq(&via_plan, &direct), "folder identity shared");

    // The impl-Fn identity: the trampoline (resolve time) and the wrapper
    // surface key on the same canonical arg types.
    let args = vec![registry
        .reading_of(&syn::parse_quote!(ZThing))
        .expect("ZThing is interned")];
    let cb1 = ext
        .iface_spec(registry, &SpecKey::callback(&args))
        .expect("callback spec");
    let cb2 = ext
        .iface_spec(registry, &SpecKey::callback(&args))
        .expect("callback spec");
    assert!(Arc::ptr_eq(&cb1, &cb2), "callback identity shared");
    assert_eq!(cb1.descr, "(JLjava/lang/String;)V");
}

// ────────────────────────────────────────────────────────────────────────
// Frozen generation-plan boundary: resolution drains every mutable planning
// memo, and every emitter shares the immutable allocations thereafter.
// ────────────────────────────────────────────────────────────────────────

#[test]
fn generation_plan_freezes_and_drains_derivations() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Record {
                    pub value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Outcome {
                    Empty,
                    Value { record: Record },
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_do_thing(records: Vec<Record>, outcome: Outcome) -> Outcome {
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
                .class(crate::data_class!(Record))
                .class(crate::sealed_class!(Outcome))
                .fun(prebindgen_registry::fun!(z_do_thing)),
        );
    // Resolve runs validation, builds every function plan, then freezes: the
    // derived memos are drained into the generation plan, and the recipe
    // compiler's own store is left where it is as the single fragment lookup.
    let gen = jni.build_over(registry).expect("resolve");
    let (ext, registry) = (gen.declarations(), gen.registry());
    let f = registry.flat().function("z_do_thing").expect("indexed");

    assert!(ext.fn_plans.borrow().is_empty());
    assert!(ext.iface_specs.borrow().is_empty());
    assert!(ext.struct_plans.borrow().is_empty());
    assert!(ext.sum_plans.borrow().is_empty());
    assert!(ext.vec_build_plans.borrow().is_empty());
    // The recipe-compiler store is NOT drained, and that is the point of 5a:
    // draining it is what forced every fragment lookup to decide first whether
    // the plan had been frozen, and two stores are what let the two answers
    // differ. One store now serves both phases.
    assert!(
        !ext.compiled.borrow().fragments().is_empty(),
        "one store serves both phases, so freeze leaves it where the compiler did"
    );
    let (fragments, functions, interfaces, structs, sums, vec_builds) =
        gen.generation_plan().counts();
    assert!(
        fragments >= 1,
        "the canonical plan holds the fragments its sites reach"
    );
    assert_eq!(functions, 1);
    assert!(interfaces >= 1, "the binding error interface is frozen");
    assert_eq!(structs, 1);
    assert_eq!(sums, 1);
    assert_eq!(vec_builds, 1);

    // Repeated writer-facing lookups return the same frozen allocation. A
    // post-freeze fallback derivation would repopulate a memo and fail the
    // final assertions below.
    let a = ext.fn_plan(registry, f).expect("plan");
    let b = ext.fn_plan(registry, f).expect("plan");
    assert!(std::rc::Rc::ptr_eq(&a, &b), "one plan per function ident");
    assert_eq!(a.native_symbol, b.native_symbol);
    let records = a.leaves().next().expect("records parameter leaf");
    assert!(
        matches!(&records.kotlin, KotlinParamOp::VecBuild { .. }),
        "the Kotlin operation remains independently frozen"
    );
    let RustParamOp::Pipeline { wire_ident } = &records.rust else {
        panic!("Vec-build Rust operation must execute its registry pipeline");
    };
    assert_eq!(wire_ident, "records_handle");
    assert_eq!(records.native.len(), 1);
    assert_eq!(records.native[0].rust_ident, "records_handle");
    assert_eq!(
        records.native[0].rust_wire.to_string(),
        "jni :: sys :: jlong"
    );
    assert_eq!(records.native[0].kt_name, "records");
    assert_eq!(records.native[0].jvm_slots, 2);

    // Exercise both artifact writers. None of the recursively used struct,
    // sum, interface, function, or Vec-helper lookups may resume planning.
    let dir = unique_test_dir("jnigen_frozen_generation");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create generation directory");
    gen.write_rust(dir.join("generated.rs"))
        .expect("write frozen Rust plan");
    gen.write_kotlin(&dir.join("kotlin"))
        .expect("write frozen Kotlin plan");

    assert!(ext.fn_plans.borrow().is_empty());
    assert!(ext.iface_specs.borrow().is_empty());
    assert!(ext.struct_plans.borrow().is_empty());
    assert!(ext.sum_plans.borrow().is_empty());
    assert!(ext.vec_build_plans.borrow().is_empty());
}

#[test]
fn generation_has_no_parallel_converter_function_cache() {
    fn production_sources(dir: &std::path::Path, sources: &mut String) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("read JNI source directory")
            .map(|entry| entry.expect("read JNI source entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "tests") {
                    continue;
                }
                production_sources(&path, sources);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push_str(&std::fs::read_to_string(path).expect("read JNI source file"));
                sources.push('\n');
            }
        }
    }

    let mut sources = String::new();
    production_sources(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/jni"),
        &mut sources,
    );
    assert!(
        !sources.contains("compiled_fns"),
        "JNI converter emission must derive from frozen registry fragments"
    );
}

/// A callback argument spelled as a **wrapped** borrow — `Box<&T>` — is refused,
/// rather than generating a trampoline the consumer cannot compile.
///
/// `Box` is transparent, so `Box<&T>`'s kind is `Ref` exactly as `&T`'s is: the
/// model answers "borrow" for both, and that answer is correct. What differs is
/// the *spelling*, and the generated trampoline is written in the spelling. Neutralise
/// the guard — pass the canonical `&T` as `produced` at `selector.rs`'s borrow arm
/// instead of the crossing's own `syntax` — and this test fails on emitted code that
/// hands a `Box<&ZThing>` to a `ZThing_to_jlong(v: &myflat::ZThing)`:
///
/// ```ignore
/// Box::new(move |__cb_arg0: Box<&myflat::ZThing>| {
///     let __cb0_enc = ZThing_to_jlong_11822692(&mut env, __cb_arg0)?;
/// ```
///
/// The guard that holds is [`Declarations::output_wrapper_shape`]'s borrowed-opaque
/// arm, which matches `syn::Type::Reference` on `produced` structurally: `Box<&T>`
/// is a `Type::Path`, so it gets no whole-value output converter, and a callback
/// arg's is a required type. Same `kind`-classifies / spelling-decides split as
/// #272's `decoded_vec_satisfies` and `is_unsized_spelling`.
///
/// A local re-check inside `emit/callback.rs` was tried and dropped (#279
/// review): it changed no output, and added a spelling classification that
/// never fires. This test is the protection instead.
#[test]
fn a_wrapped_borrow_callback_arg_declines() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let build = |argty: syn::Type| -> Result<String, String> {
        let items: Vec<(syn::Item, SourceLocation)> = vec![
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct ZThing {
                        pub v: i64,
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Fn(syn::parse_quote!(
                    pub fn z_sub(cb: impl Fn(#argty) + Send + Sync + 'static) {
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
                    .class(crate::ptr_class!(ZThing))
                    .fun(prebindgen_registry::fun!(z_sub)),
            );
        let dir = unique_test_dir("jnigen_wrapped_cb");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        match jni.build_over(registry) {
            Ok(g) => Ok(std::fs::read_to_string(
                g.write_rust(dir.join("g.rs")).expect("write_rust"),
            )
            .expect("read rust")),
            Err(e) => Err(format!("{e}")),
        }
    };

    // The canonical borrow still resolves and still clones through the core.
    let plain = build(syn::parse_quote!(&ZThing)).expect("a plain borrow resolves");
    assert!(
        plain.contains("__cb_arg0: &myflat::ZThing"),
        "the trampoline takes the borrow as written:\n{plain}"
    );

    // Wrapped, the clone is inexpressible, so nothing claims it.
    let err = build(syn::parse_quote!(Box<&ZThing>))
        .expect_err("a wrapped borrow callback arg must not resolve");
    assert!(
        err.contains("could not be resolved"),
        "the refusal names the type: {err}"
    );
}

/// A callback argument's layers are peeled by the same walk a sum payload's
/// are.
///
/// #429 was the sum builder applying the leaf's conversion to the whole wire
/// value; #432 fixed it by walking the layers. The `asRaw` proxy — the other
/// place a value is converted for delivery — kept its own one-shot wrap, so the
/// same defect survived in the callback direction until the shape matrix gained
/// that position (#438).
///
/// Both callers run `carry_layers` now, each with its own receiver, which is
/// why this test and `a_payload_carries_its_option_and_collection_layers`
/// assert the same expression from two directions.
#[test]
fn a_callback_argument_carries_its_layers() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn probe(cb: impl Fn(Vec<Option<u64>>) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().fun(prebindgen_registry::fun!(probe)));

    let dir = unique_test_dir("jnigen_cb_arg_layers");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let kotlin = jni
        .build_over(registry)
        .expect("resolve")
        .write_kotlin(&dir.join("kotlin"))
        .unwrap()
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // The typed view is a list of optional `ULong`s and the raw twin a list of
    // boxed `Long`s, so the conversion runs per element — not on the list.
    assert!(
        kotlin.contains("public fun run(vec: List<ULong?>)"),
        "the typed view keeps its layers:\n{kotlin}"
    );
    assert!(
        kotlin.contains("vec.map { it?.toULong() }"),
        "the proxy converts element by element:\n{kotlin}"
    );
}

/// A callback-valued parameter is a site, and says so canonically.
///
/// It never reaches `Compiler::site` — `classify_leaf` answers it whole,
/// because a callback ARGUMENT does not always have a conversion of its own —
/// so nothing else would state it. Leaving it out made "JNI describes its
/// sites" false for a live path (#622 review), and the collected
/// `GenerationPlan` could not have noticed: a site that is never frozen is not
/// a duplicate or an invalid one, it is simply absent.
#[test]
fn a_callback_parameter_is_a_site_in_the_canonical_plan() {
    use prebindgen_registry::recipe::Role;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(
            syn::parse_str("pub fn z_each(n: i64, sink: impl Fn(i64) + Send + Sync + 'static) { unimplemented!() }")
                .expect("parse fn"),
        ),
        loc.clone(),
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_each)))
        .build_over(registry)
        .expect("resolve");

    let sites: Vec<String> = gen
        .declarations()
        .site_plans
        .borrow()
        .iter()
        .filter(|plan| plan.id().site().owner == "z_each")
        .map(|plan| match plan.id().site().role {
            Role::Param { index } => format!("Param({index})"),
            ref other => format!("{other}"),
        })
        .collect();
    assert!(
        sites.contains(&"Param(1)".to_string()),
        "the callback parameter states its own site: {sites:?}"
    );
}

/// An **expanded** parameter's callback leaf is still a leaf of that
/// parameter.
///
/// The callback shortcut in `classify_leaf` runs for expansion leaves too, and
/// naming it `Role::Param` there would undo the identity rule the ordinary
/// path states — an expanded callback leaf would claim a source position the
/// function does not have (#622 review). One role selection answers for both.
#[test]
fn an_expanded_callback_leaf_keeps_its_expansion_identity() {
    use prebindgen_registry::recipe::Role;
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_opts_new(retries: i32, on_tick: impl Fn(i64) + Send + Sync + 'static) -> ZOpts { unimplemented!() }",
        "pub fn z_go(opts: ZOpts, tail: i64) -> i64 { unimplemented!() }",
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
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_go)))
        .expand(
            prebindgen_registry::expand_param!(ZOpts)
                .variant(prebindgen_registry::fun!(z_opts_new)),
        )
        .build_over(registry)
        .expect("resolve");

    let roles: Vec<String> = gen
        .declarations()
        .site_plans
        .borrow()
        .iter()
        .filter(|plan| plan.id().site().owner == "z_go")
        .map(|plan| match plan.id().site().role {
            Role::Param { index } => format!("Param({index})"),
            Role::ExpansionLeaf { param, leaf } => format!("ExpansionLeaf({param},{leaf})"),
            ref other => format!("{other}"),
        })
        .collect();
    assert_eq!(
        roles,
        [
            "return value".to_string(),
            "ExpansionLeaf(0,0)".to_string(),
            "ExpansionLeaf(0,1)".to_string(),
            // The value that leaf's callback delivers: a place in `z_go`, and
            // `param` is the SOURCE parameter the callback came out of — the
            // same position `ExpansionLeaf` names.
            "argument 0 of the callback in parameter 0".to_string(),
            "Param(1)".to_string(),
        ],
        "the callback leaf is the expansion's second leaf, not a parameter: {roles:?}"
    );
    // And it is the REGISTRY's answer, not one the adapter pushed beside it:
    // `planned_sites` is what `Registry::compile_sites` returned. Asserting on
    // `site_plans` alone would pass either way (#687 review).
    assert!(
        gen.declarations()
            .planned_sites
            .borrow()
            .keys()
            .any(|site| site.owner == "z_go"
                && matches!(
                    site.role,
                    prebindgen_registry::recipe::Role::CallbackArg { param: 0, arg: 0 }
                )),
        "the expanded callback's delivered value is a site the registry planned"
    );
}

/// A value a callback delivers by taking it apart states its own site, and
/// that site names the row the delivery took.
///
/// `ZReply` is the case #622's first two attempts dead-ended on: its
/// decomposition is a deconstructor declaration, not a value form, so before
/// this the crossing had no `parts` row at all and a site could only be
/// fabricated — asking for `parts` answered `NoSuchRecipe` and asking for the
/// default answered a whole-value conversion the boundary does not have.
///
/// What the site says is not derived here. The ABI is the same allocation the
/// trampoline's `JInvokePart` holds — asserted by identity rather than by
/// comparing two lists, because a copy that happens to match today is the
/// second answer this umbrella exists to delete.
#[test]
fn a_delivered_argument_names_the_row_it_crossed_on() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_reply_zid(r: &ZReply) -> Option<ZId> { unimplemented!() }",
        "pub fn z_reply_is_ok(r: &ZReply) -> bool { unimplemented!() }",
        "pub fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { unimplemented!() }",
        "pub fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { unimplemented!() }",
        "pub fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    items.push((
        syn::Item::Struct(syn::parse_quote!(
            pub struct ZId {
                pub hi: i64,
                pub lo: i64,
            }
        )),
        loc.clone(),
    ));
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_get(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    ));
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("query")
                .class(crate::data_class!(ZId))
                .class(
                    crate::ptr_class!(ZKeyExpr)
                        .method(prebindgen_registry::fun!(z_keyexpr_as_str).name("asStr")),
                )
                .class(
                    crate::ptr_class!(ZSample)
                        .method(prebindgen_registry::fun!(z_sample_key_expr).name("keyExpr")),
                )
                .class(
                    crate::ptr_class!(ZReply)
                        .method(prebindgen_registry::fun!(z_reply_zid).name("zid"))
                        .method(prebindgen_registry::fun!(z_reply_is_ok).name("isOk"))
                        .method(prebindgen_registry::fun!(z_reply_sample).name("sample")),
                )
                .fun(prebindgen_registry::fun!(z_get)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZKeyExpr)
                .field_self()
                .field(prebindgen_registry::fun!(z_keyexpr_as_str)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZSample)
                .field(prebindgen_registry::fun!(z_sample_key_expr)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZReply)
                .field(prebindgen_registry::fun!(z_reply_zid))
                .field(prebindgen_registry::fun!(z_reply_is_ok))
                .field(prebindgen_registry::fun!(z_reply_sample)),
        )
        .build_over(registry)
        .expect("resolve");

    let decls = gen.declarations();
    let plans = decls.site_plans.borrow();
    let site = plans
        .iter()
        .find(|plan| {
            plan.id().site().owner == "z_get"
                && matches!(
                    plan.id().site().role,
                    prebindgen_registry::recipe::Role::CallbackArg { param: 0, arg: 0 }
                )
        })
        .expect("the delivered ZReply states a site");

    // That site is the REGISTRY's answer. `planned_sites` is exactly what
    // `Registry::compile_sites` returned, so asserting on it tells a site the
    // walk planned apart from one the adapter pushed beside it — which is what
    // `site_plans` alone cannot distinguish (#687 review).
    assert!(
        decls
            .planned_sites
            .borrow()
            .keys()
            .any(|s| s.owner == "z_get"
                && matches!(
                    s.role,
                    prebindgen_registry::recipe::Role::CallbackArg { param: 0, arg: 0 }
                )),
        "a plain callback parameter's delivered value is a site the registry planned"
    );

    // The row is `site_bindings`' answer, and it is the decomposing one — not
    // the whole-value default, which for this crossing is a conversion that
    // does not exist.
    assert_eq!(
        site.bound().recipe.name(),
        &crate::jni::recipes::parts(),
        "a delivered ZReply crosses on its `parts` row"
    );
    assert_eq!(
        site.bound().crossing.direction(),
        prebindgen_registry::recipe::Direction::Deconstruct
    );

    // The ABI is several leaves, and it is the trampoline's own list rather
    // than a second one derived beside it.
    let crate::jni::compile::JAbiLeaves::Decomposed(stated) = site.abi().payload() else {
        panic!("a decomposed delivery does not occupy one whole leaf");
    };
    assert!(stated.len() > 1, "ZReply is delivered as several leaves");
    assert_eq!(site.abi().slots(), stated.len());

    let callback = decls
        .in_frag(&param_reading(&gen, "z_get", 0))
        .expect("the callback parameter compiled");
    let crate::jni::compile::JAbiLeaves::Decomposed(delivered) = callback
        .rust
        .invoke_plan()
        .expect("the callback conversion is an Invoke")
        .arg_abi(0)
        .expect("its first argument has a delivery")
    else {
        panic!("the trampoline delivers ZReply whole");
    };
    assert!(
        std::rc::Rc::ptr_eq(stated, &delivered),
        "the site's ABI is a copy of the delivery's rather than the same leaves"
    );
}

/// A value a callback delivers **whole** states its site over the delivery too.
///
/// The other half of the row above: a scalar argument has no decomposition, so
/// it names no `parts` row and takes its crossing's default — and its site's
/// ABI is still the trampoline's own, not a second one derived beside it.
#[test]
fn a_whole_delivered_argument_shares_the_trampolines_abi() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let fns: &[&str] =
        &["pub fn z_watch(on_tick: impl Fn(i64) + Send + Sync + 'static) { unimplemented!() }"];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_watch)))
        .build_over(registry)
        .expect("resolve");

    let decls = gen.declarations();
    let plans = decls.site_plans.borrow();
    let site = plans
        .iter()
        .find(|plan| {
            plan.id().site().owner == "z_watch"
                && matches!(
                    plan.id().site().role,
                    prebindgen_registry::recipe::Role::CallbackArg { param: 0, arg: 0 }
                )
        })
        .expect("the delivered i64 states a site");

    // No decomposition and no declaration, so there is no row to name at all:
    // the site takes the recipe the registry derives from the type's kind,
    // attributed to the adapter because nothing bound it.
    assert_eq!(site.bound().recipe.name().as_str(), "derived");
    assert_eq!(
        site.bound().origin,
        prebindgen_registry::recipe::Origin::Adapter
    );
    assert_eq!(site.abi().slots(), 1);

    let crate::jni::compile::JAbiLeaves::Invoked(stated) = site.abi().payload() else {
        panic!("a whole delivery is not several leaves");
    };
    let crate::jni::compile::JAbiLeaves::Invoked(delivered) = decls
        .in_frag(&param_reading(&gen, "z_watch", 0))
        .expect("the callback parameter compiled")
        .rust
        .invoke_plan()
        .expect("the callback conversion is an Invoke")
        .arg_abi(0)
        .expect("its first argument has a delivery")
    else {
        panic!("the trampoline decomposes an i64");
    };
    assert!(
        std::rc::Rc::ptr_eq(stated, &delivered),
        "the site's ABI is a copy of the delivery's rather than the same plan"
    );
}

/// A whole-element fold earns no decomposition row, and a scalar of the same
/// element type is not dragged onto one.
///
/// `apply_leaf_vec_folds` files a plan for `impl Fn(&[T])` keyed under `&[T]`
/// whose `source` is the ELEMENT and whose `decon` is `None`: nothing is taken
/// apart, each element crosses whole through its own converter. A row declared
/// off that plan would state a decomposition of `T` that does not exist — and
/// the `T` argument beside it would bind to that row and name a fragment
/// compiled under a different recipe (#623 review).
///
/// Both arguments in one callback, which is the shape that makes the collision
/// observable rather than theoretical.
#[test]
fn a_whole_element_fold_states_no_decomposition_row() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_handle_id(h: &ZHandle) -> i64 { unimplemented!() }",
        "pub fn z_stream(cb: impl Fn(&[ZHandle], ZHandle) + Send + Sync + 'static) { unimplemented!() }",
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
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("stream")
                .class(
                    crate::ptr_class!(ZHandle)
                        .method(prebindgen_registry::fun!(z_handle_id).name("id")),
                )
                .fun(prebindgen_registry::fun!(z_stream)),
        )
        .build_over(registry)
        .expect("resolve");

    let decls = gen.declarations();
    let element = param_reading(&gen, "z_handle_id", 0);
    let crossing = prebindgen_registry::recipe::Crossing::new(
        element.borrow_target().unwrap_or(&element).clone(),
        prebindgen_registry::recipe::Direction::Deconstruct,
    );
    assert!(
        decls
            .recipe_table()
            .key_of(&crossing.key(), &crate::jni::recipes::parts())
            .is_none(),
        "a whole-element fold declared a decomposition row for its element: {:?}",
        decls.recipe_table().names_of(&crossing.key())
    );

    let plans = decls.site_plans.borrow();
    let role_of = |arg: usize| {
        plans
            .iter()
            .find(|plan| {
                plan.id().site().owner == "z_stream"
                    && plan.id().site().role
                        == prebindgen_registry::recipe::Role::CallbackArg { param: 0, arg }
            })
            .map(|plan| plan.bound().recipe.name().as_str().to_string())
    };
    // Both delivered values state a site, and neither took a decomposition row:
    // the slice is folded element by element, the handle crosses whole.
    assert!(role_of(0).is_some(), "the folded slice states no site");
    assert_ne!(role_of(0).as_deref(), Some("parts"));
    assert_eq!(role_of(1).as_deref(), Some("whole"));
}

/// One captured function's parameter reading, by name and position.
#[cfg(test)]
fn param_reading(
    gen: &crate::JniGen,
    func: &str,
    index: usize,
) -> prebindgen_registry::flat::TypeRef {
    gen.registry()
        .flat()
        .function(&syn::Ident::new(func, proc_macro2::Span::call_site()))
        .unwrap_or_else(|| panic!("{func} is captured"))
        .params[index]
        .ty
        .clone()
}

/// Two callbacks under one expanded parameter are refused where the expansion
/// is declared.
///
/// A delivered value's site is named by the SOURCE parameter it arrived on —
/// `Role::CallbackArg { param, arg }` — not by the leaf the callback came in
/// on, because a parameter delivers through at most one callback. Nothing had
/// enforced that: a constructor taking two callbacks gave two distinct
/// positions one identity, and with different signatures the second site would
/// have read the first callback's edge (#687 review).
#[test]
fn two_callbacks_in_one_expanded_parameter_are_refused() {
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_opts_new(on_tick: impl Fn(i64) + Send + Sync + 'static, on_text: impl Fn(bool) + Send + Sync + 'static) -> ZOpts { unimplemented!() }",
        "pub fn z_go(opts: ZOpts) -> i64 { unimplemented!() }",
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

    let error = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_go)))
        .expand(
            prebindgen_registry::expand_param!(ZOpts)
                .variant(prebindgen_registry::fun!(z_opts_new)),
        )
        .build_over(registry)
        .expect_err("two callbacks under one parameter cannot both be named");
    let message = error.to_string();
    assert!(
        message.contains("more than one callback"),
        "the refusal names the shape and why: {message}"
    );
}

/// A site the registry refuses fails the build, naming the parameter.
///
/// `fn_plan` re-diagnoses a missing site when it reads one, but only for the
/// sites it reads: a `Role::CallbackArg` plan is frozen inside `JCompile::plan`
/// and no later lookup consumes it, so a refusal there left a site that never
/// existed rather than a diagnostic (#690 review).
///
/// No shape refuses at a site today, which is why discarding the refusals looked
/// safe — a tuple, a nested `Vec`, a raw pointer, a handle and a borrow all
/// plan. So the refusal is made, through a test-only seam, and what is under
/// test is the path that carries it out.
#[test]
fn a_refused_callback_argument_fails_the_build_and_names_its_parameter() {
    let loc = myflat_loc();
    let items = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_watch(on_one: impl Fn(ZOne) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc,
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let builder = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZOne))
                .fun(prebindgen_registry::fun!(z_watch)),
        );
    *builder.decls.refuse_role.borrow_mut() =
        Some("argument 0 of the callback in parameter 0".to_string());

    let Err(error) = builder.build_over(registry) else {
        panic!("a refused site fails the build rather than vanishing")
    };
    let message = error.to_string();
    assert!(
        message.contains("could not be planned"),
        "the refusal is reported: {message}"
    );
    assert!(
        message.contains("on_one"),
        "and names the parameter the author wrote, not only the position: {message}"
    );
}
