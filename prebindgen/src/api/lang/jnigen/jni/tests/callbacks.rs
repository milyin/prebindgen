use super::*;

fn callback_snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).fun(crate::fun!(z_thing_name).name("name")))
                // ZOther: plain ptr_class, no canonical output ⇒ whole-handle fallback.
                .class(crate::ptr_class!(ZOther))
                .fun(crate::fun!(z_thing_sub))
                .fun(crate::fun!(z_other_sub)),
        )
        // Canonical output: handle (identity) + its string form — a callback
        // arg of ZThing decomposes into these 2 leaves.
        .expand(
            crate::return_expand!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );

    let dir = unique_test_dir("jnigen_cb_snap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
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
/// (`return_expand!` `.field_self()`) must emit the root MOVE after every borrow of
/// the owned value — otherwise the nested child clone (which borrows the root)
/// follows `Box::into_raw(Box::new(value))` and fails to compile with "use of
/// moved value". Declaring `.field_self()` LAST guarantees the
/// correct order (the emitter emits identity leaves in declaration order, after
/// all non-identity leaves). This mirrors the zenoh-flat `ZQuery` queryable
/// callback (handle + decomposed fields, nested `ZKeyExpr` identity).
#[test]
fn callback_root_identity_moved_after_nested_borrow() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZChild).fun(crate::fun!(z_child_name).name("name")))
                .class(crate::ptr_class!(ZParent).fun(crate::fun!(z_parent_child).name("child")))
                .fun(crate::fun!(z_parent_sub)),
        )
        // Child handle: canonical output = identity (clone) + its name string.
        .expand(
            crate::return_expand!(ZChild)
                .field_self()
                .field(crate::fun!(z_child_name)),
        )
        // Parent: a nested child-handle record, then its OWN root identity LAST.
        .expand(
            crate::return_expand!(ZParent)
                .field(crate::fun!(z_parent_child))
                .field_self(),
        );

    let dir = unique_test_dir("jnigen_root_id_order");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
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
/// (`z_reply_zid -> Option<ZId>`, a value class with no canonical child).
/// Every `Option` nesting step must become its own `match` (`None` ⇒ null
/// leaf) — never a blind accessor compose through an `Option`.
#[test]
fn callback_double_option_unwrap_pipeline() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
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
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_get(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    ));
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .set_source_module(syn::parse_quote!(myflat))
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("query")
                .class(crate::value_class!(ZId))
                .class(crate::ptr_class!(ZKeyExpr).fun(crate::fun!(z_keyexpr_as_str).name("asStr")))
                .class(crate::ptr_class!(ZTs).fun(crate::fun!(z_ts_ntp64).name("ntp64")))
                .class(
                    crate::ptr_class!(ZSample)
                        .fun(crate::fun!(z_sample_key_expr).name("keyExpr"))
                        .fun(crate::fun!(z_sample_timestamp).name("timestamp")),
                )
                .class(crate::ptr_class!(ZErr).fun(crate::fun!(z_err_payload).name("payload")))
                .class(
                    crate::ptr_class!(ZReply)
                        .fun(crate::fun!(z_reply_zid).name("zid"))
                        .fun(crate::fun!(z_reply_is_ok).name("isOk"))
                        .fun(crate::fun!(z_reply_sample).name("sample"))
                        .fun(crate::fun!(z_reply_err).name("err")),
                )
                .fun(crate::fun!(z_get)),
        )
        .expand(
            crate::return_expand!(ZKeyExpr)
                .field_self()
                .field(crate::fun!(z_keyexpr_as_str)),
        )
        .expand(crate::return_expand!(ZTs).field(crate::fun!(z_ts_ntp64)))
        .expand(
            crate::return_expand!(ZSample)
                .field(crate::fun!(z_sample_key_expr))
                .field(crate::fun!(z_sample_timestamp)),
        )
        .expand(crate::return_expand!(ZErr).field(crate::fun!(z_err_payload)))
        .expand(
            crate::return_expand!(ZReply)
                .field(crate::fun!(z_reply_zid))
                .field(crate::fun!(z_reply_is_ok))
                .field(crate::fun!(z_reply_sample))
                .field(crate::fun!(z_reply_err)),
        );

    let dir = unique_test_dir("jnigen_double_opt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
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
    // 6 leaves ⇒ typed `run` descriptor: nullable value-blob `[B`, raw `Z`
    // for the non-null bool discriminator, typed handle class (full FQN),
    // nullable String, BOXED Long for the nullable timestamp, nullable `[B`.
    assert!(
        rc.contains("\"([BZLjava/lang/Long;Ljava/lang/String;Ljava/lang/Long;[B)V\""),
        "{rust}"
    );
    // The non-null bool crosses as a raw typed jvalue — never boxed.
    assert!(rc.contains("jni::sys::jvalue{z:"), "{rust}");

    // Kotlin tier: the generated callback `fun interface` carries the typed
    // params — ok-arm and err-arm leaves nullable (the value may be absent),
    // the discriminator non-null; the value-blob leaf surfaces as its raw
    // (nullable) ByteArray wire, NOT the value class — the SDK wraps.
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let iface_file = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("fun interface ZReplyCallback"))
        .unwrap_or_default();
    // Scope to the interface block — the merged package file also holds the
    // ZId value class and other decls.
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
    // The wrapper takes the typed interface and forwards it bare (no
    // value-blob rebuilding adapter exists anymore).
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
// (`.return_expand`). Interface identity follows the DECLARATION, so
// differently-decomposed functions get distinct interfaces instead of
// colliding on one type-keyed name.
// ────────────────────────────────────────────────────────────────────────
