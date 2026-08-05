use super::*;
use crate::api::core::registry::Conversions;

fn callback_snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use crate::SourceLocation;
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
                // ZOther: plain ptr_class, no canonical output ⇒ whole-handle fallback.
                .class(crate::ptr_class!(ZOther))
                .fun(crate::fun!(z_thing_sub))
                .fun(crate::fun!(z_other_sub)),
        )
        // Canonical output: handle (identity) + its string form — a callback
        // arg of ZThing decomposes into these 2 leaves.
        .expand(
            crate::expand_return!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );

    let dir = unique_test_dir("jnigen_cb_snap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let gen = jni.build_with(registry).expect("resolve");
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
        all.contains("funZThingCallback.asRaw():ZThingCallbackRaw=ZThingCallbackRaw{handle,name->run(ZThing(handle),name)}"),
        "{all}"
    );
    // Plan-less ZOther arg (Phase 3): a raw twin `run(zOther: Long)` + an `asRaw`
    // proxy that wraps the pointer into the handle class AND `close()`s it in a
    // `finally` (close-unless-taken) — the Rust side delivers only the raw jlong.
    assert!(
        all.contains("funinterfaceZOtherCallbackRaw{publicfunrun(zOther:Long)"),
        "{all}"
    );
    assert!(all.contains("val__own0=ZOther(zOther)"), "{all}");
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
    use crate::SourceLocation;
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZChild).method(crate::fun!(z_child_name).name("name")))
                .class(crate::ptr_class!(ZParent).method(crate::fun!(z_parent_child).name("child")))
                .fun(crate::fun!(z_parent_sub)),
        )
        // Child handle: canonical output = identity (clone) + its name string.
        .expand(
            crate::expand_return!(ZChild)
                .field_self()
                .field(crate::fun!(z_child_name)),
        )
        // Parent: a nested child-handle record, then its OWN root identity LAST.
        .expand(
            crate::expand_return!(ZParent)
                .field(crate::fun!(z_parent_child))
                .field_self(),
        );

    let dir = unique_test_dir("jnigen_root_id_order");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
    use crate::SourceLocation;
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("query")
                .class(crate::data_class!(ZId))
                .class(
                    crate::ptr_class!(ZKeyExpr).method(crate::fun!(z_keyexpr_as_str).name("asStr")),
                )
                .class(crate::ptr_class!(ZTs).method(crate::fun!(z_ts_ntp64).name("ntp64")))
                .class(
                    crate::ptr_class!(ZSample)
                        .method(crate::fun!(z_sample_key_expr).name("keyExpr"))
                        .method(crate::fun!(z_sample_timestamp).name("timestamp")),
                )
                .class(crate::ptr_class!(ZErr).method(crate::fun!(z_err_payload).name("payload")))
                .class(
                    crate::ptr_class!(ZReply)
                        .method(crate::fun!(z_reply_zid).name("zid"))
                        .method(crate::fun!(z_reply_is_ok).name("isOk"))
                        .method(crate::fun!(z_reply_sample).name("sample"))
                        .method(crate::fun!(z_reply_err).name("err")),
                )
                .fun(crate::fun!(z_get)),
        )
        .expand(
            crate::expand_return!(ZKeyExpr)
                .field_self()
                .field(crate::fun!(z_keyexpr_as_str)),
        )
        .expand(crate::expand_return!(ZTs).field(crate::fun!(z_ts_ntp64)))
        .expand(
            crate::expand_return!(ZSample)
                .field(crate::fun!(z_sample_key_expr))
                .field(crate::fun!(z_sample_timestamp)),
        )
        .expand(crate::expand_return!(ZErr).field(crate::fun!(z_err_payload)))
        .expand(
            crate::expand_return!(ZReply)
                .field(crate::fun!(z_reply_zid))
                .field(crate::fun!(z_reply_is_ok))
                .field(crate::fun!(z_reply_sample))
                .field(crate::fun!(z_reply_err)),
        );

    let dir = unique_test_dir("jnigen_double_opt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
    use crate::SourceLocation;
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
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
                .fun(crate::fun!(z_things_all))
                .fun(crate::fun!(z_thing_sub)),
        )
        .expand(
            crate::expand_return!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );
    let gen = jni.build_with(registry).expect("resolve");
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
    let plan = registry
        .unfold_plans()
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
// Function-plan memo (the #90 "build once, store" capstone): validation
// and every emitter share ONE `JniFunctionPlan` per function ident.
// ────────────────────────────────────────────────────────────────────────

#[test]
fn fn_plan_memo_shares_one_derivation() {
    use crate::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_do_thing(x: i64) -> i64 {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("thing").fun(crate::fun!(z_do_thing)));
    // resolve runs validation, which builds and stores every function's plan.
    let gen = jni.build_with(registry).expect("resolve");
    let (ext, registry) = (gen.declarations(), gen.registry());
    let f = registry.flat().function("z_do_thing").expect("indexed");

    // The plan is already in the memo (populated at resolve by validation) —
    // repeated lookups return the SAME allocation, and it equals what a fresh
    // `JniFunctionPlan::build` derives.
    let a = ext.fn_plan(registry, f).expect("plan");
    let b = ext.fn_plan(registry, f).expect("plan");
    assert!(std::rc::Rc::ptr_eq(&a, &b), "one plan per function ident");
    assert_eq!(a.native_symbol, b.native_symbol);
    let fresh = JniFunctionPlan::build(ext, registry, f).expect("fresh build");
    assert_eq!(a.native_symbol, fresh.native_symbol);
    assert_eq!(a.jni_method, fresh.jni_method);
}

/// A callback identity is the same whether its args come from the **reading**
/// or from the signature's syntax.
///
/// `SpecKey::Callback` is a memo key, so it holds `TypeKey`s — a `TypeRef`
/// could not go in it (`Ord` is required, and an `Origin` carries a
/// `SourceLocation`, so two identical readings from different files would
/// compare unequal). That means the args reach `SpecKey::callback` as
/// spellings, and #275's last part changes *which* spelling: from
/// `extract_fn_trait_args(&pt.ty)` to each arg `TypeRef`'s `spell()`.
///
/// If those two disagreed, the memo would split one interface identity into
/// two — an extra generated `fun interface` and a descriptor mismatch. Nothing
/// else would fail: not a panic, not an unresolved type, just a duplicate. So
/// it is pinned here rather than trusted.
#[test]
fn a_callback_identity_is_the_same_from_the_reading_or_the_syntax() {
    use crate::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_sub(cb: impl Fn(ZThing) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZThing))
                .fun(crate::fun!(z_sub)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    let registry = gen.registry();

    let f = registry.flat().function("z_sub").expect("the declared fn");
    let cb = f
        .params
        .iter()
        .find_map(|p| match p.ty.kind() {
            crate::api::core::flat::TypeKind::Callback { args } => Some((p, args)),
            _ => None,
        })
        .expect("z_sub takes a callback");
    let (param, arg_readings) = cb;

    // The two routes to the same identity. The syntax route builds the key's
    // own `Vec<TypeKey>` directly, because `SpecKey::callback` now takes
    // readings — which is exactly the claim being pinned: the readings and the
    // signature's own `impl Fn` bounds agree on every arg's identity.
    let from_reading = SpecKey::callback(arg_readings);
    let from_syntax = SpecKey::Callback(
        crate::api::core::registry::extract_fn_trait_args(
            &crate::api::core::emit::Emit::for_test().spell_ty(&param.ty),
        )
        .expect("the param is an impl Fn")
        .iter()
        .map(crate::api::core::registry::TypeKey::from_type)
        .collect(),
    );

    assert_eq!(
        from_reading, from_syntax,
        "a callback keyed off its readings must be the SAME memo identity as one \
         keyed off the signature's syntax — otherwise the memo silently emits two \
         interfaces for one callback"
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
    use crate::SourceLocation;
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
            crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::ptr_class!(ZThing))
                    .fun(crate::fun!(z_sub)),
            );
        let dir = unique_test_dir("jnigen_wrapped_cb");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        match jni.build_with(registry) {
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
