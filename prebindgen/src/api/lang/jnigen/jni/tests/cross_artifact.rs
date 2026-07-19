//! Cross-artifact golden tests (issue #90): parse the generated Rust extern
//! signatures and the generated Kotlin `external fun` declarations back out
//! of a full pipeline run and assert they agree — symbol, arity, and
//! per-position wire types. The per-side snapshot tests check each artifact
//! against expectations; these check the two artifacts against EACH OTHER,
//! so a lowering change that drifts one side without the other fails even
//! if both sides look individually plausible.

use std::collections::BTreeMap;

use super::*;

/// One parsed extern signature: parameter wire types in order, and the
/// return type (`None` = unit).
#[derive(Debug)]
struct ExternSig {
    params: Vec<String>,
    ret: Option<String>,
}

/// Parse every `#[no_mangle] extern "C"` function out of the generated Rust
/// file: JNI export symbol → signature. The fixed leading `env`/`_class`
/// params are dropped (their Kotlin side is implicit in the JNI calling
/// convention); the rest are the wire params the Kotlin `external fun`
/// declares, reduced to their last path segment (`jni::sys::jlong` →
/// `jlong`, `jni::objects::JObject<'a>` → `JObject`).
fn rust_externs(rust_src: &str) -> BTreeMap<String, ExternSig> {
    fn last_segment(ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(tp) => tp
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            syn::Type::Tuple(t) if t.elems.is_empty() => "()".to_string(),
            other => other.to_token_stream().to_string(),
        }
    }
    let file = syn::parse_file(rust_src).expect("generated Rust parses");
    let mut out = BTreeMap::new();
    for item in &file.items {
        let syn::Item::Fn(f) = item else { continue };
        let is_extern_c = matches!(&f.sig.abi, Some(abi)
            if abi.name.as_ref().map(|n| n.value()) == Some("C".to_string()));
        let no_mangle = f.attrs.iter().any(|a| a.path().is_ident("no_mangle"));
        if !is_extern_c || !no_mangle {
            continue;
        }
        let params: Vec<String> = f
            .sig
            .inputs
            .iter()
            .skip(2) // env + _class
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pt) => Some(last_segment(&pt.ty)),
                syn::FnArg::Receiver(_) => None,
            })
            .collect();
        let ret = match &f.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => {
                let s = last_segment(ty);
                (s != "()").then_some(s)
            }
        };
        out.insert(f.sig.ident.to_string(), ExternSig { params, ret });
    }
    out
}

/// Byte offsets at which a named `class` / `object` / `interface`
/// declaration starts, in order. A generated file bundles a whole package
/// (several classes plus the `JNINative` object can share one file), so an
/// extern's owning class is the nearest preceding named declaration.
/// `companion object {` is unnamed and skipped — a `@JvmStatic` extern in a
/// companion resolves against the enclosing class, matching the JNI symbol.
fn kotlin_class_starts(src: &str) -> Vec<(usize, String)> {
    let mut owners = Vec::new();
    let mut off = 0;
    for line in src.lines() {
        let t = line.trim_start();
        for kw in ["object ", "class ", "interface "] {
            if let Some(idx) = t.find(kw) {
                let after = &t[idx + kw.len()..];
                let name: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    owners.push((off, name));
                    break;
                }
            }
        }
        off += line.len() + 1;
    }
    owners
}

/// Parse every `external fun name(params): Ret` declaration out of one
/// generated Kotlin file (both the single-line and the wrapped
/// one-param-per-line forms), attributed to its owning class/object.
fn kotlin_externs(src: &str) -> Vec<(String, String, ExternSig)> {
    let owners = kotlin_class_starts(src);
    let owner_at = |off: usize| -> String {
        owners
            .iter()
            .take_while(|(o, _)| *o <= off)
            .last()
            .map(|(_, n)| n.clone())
            .expect("an external fun has an enclosing class/object")
    };
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(pos) = rest.find("external fun ") {
        let abs = src.len() - rest.len() + pos;
        rest = &rest[pos + "external fun ".len()..];
        let open = rest.find('(').expect("external fun has a param list");
        let name = rest[..open].trim().to_string();
        // Wire types are non-generic, so the matching ')' is the first one.
        let close = rest.find(')').expect("param list closes");
        let params: Vec<String> = rest[open + 1..close]
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(|p| {
                p.split_once(':')
                    .map(|(_, ty)| ty.trim().to_string())
                    .unwrap_or_else(|| panic!("unparsable extern param `{p}` in `{name}`"))
            })
            .collect();
        let after = &rest[close + 1..];
        let line_end = after.find('\n').unwrap_or(after.len());
        let ret_part = after[..line_end].trim();
        let ret = ret_part
            .strip_prefix(':')
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty() && r != "Unit");
        out.push((owner_at(abs), name, ExternSig { params, ret }));
        rest = after;
    }
    out
}

/// The Kotlin `package …;` header of a generated file.
fn kotlin_package(src: &str) -> String {
    src.lines()
        .find_map(|l| l.trim().strip_prefix("package "))
        .expect("generated Kotlin declares a package")
        .trim()
        .to_string()
}

/// Wire-type compatibility: a Rust extern param/return and the Kotlin
/// `external fun` type at the same position. Primitives must match exactly
/// (and stay non-null — JNI primitives can't carry null); object wires must
/// face a non-primitive (or nullable-boxed) Kotlin type.
fn wire_compatible(rust_wire: &str, kt: &str) -> bool {
    let kt_base = kt.trim_end_matches('?');
    let nullable = kt.ends_with('?');
    let kt_is_primitive = matches!(
        kt_base,
        "Boolean" | "Byte" | "Char" | "Short" | "Int" | "Long" | "Float" | "Double"
    );
    match rust_wire {
        "jboolean" => kt == "Boolean",
        "jbyte" => kt == "Byte",
        "jchar" => kt == "Char",
        "jshort" => kt == "Short",
        "jint" => kt == "Int",
        "jlong" => kt == "Long",
        "jfloat" => kt == "Float",
        "jdouble" => kt == "Double",
        "JString" | "jstring" => kt_base == "String",
        "JByteArray" | "jbyteArray" => kt_base == "ByteArray",
        // A JObject wire carries any reference: erased Any/Any?, a boxed
        // primitive (`Int?`), a String?, a List, …
        "JObject" | "JClass" | "jobject" => !kt_is_primitive || nullable,
        _ => false,
    }
}

/// The cross-artifact assertion: every Kotlin `external fun` in every
/// generated file must have a Rust `#[no_mangle] extern "C"` twin under the
/// spec-mangled symbol derived from (file package, file class, method name),
/// with the same arity and position-wise compatible wire types — and every
/// Rust extern must be claimed by exactly one Kotlin declaration (no
/// orphaned exports).
fn assert_cross_artifact(rust_src: &str, kotlin: &BTreeMap<String, String>) {
    let rust = rust_externs(rust_src);
    assert!(!rust.is_empty(), "fixture emits at least one Rust extern");
    let mut claimed: BTreeMap<String, String> = BTreeMap::new();

    for src in kotlin.values() {
        let externs = kotlin_externs(src);
        if externs.is_empty() {
            continue;
        }
        let package = kotlin_package(src);
        for (class, method, kt_sig) in externs {
            let class = class.as_str();
            let symbol = super::super::symbol::native_symbol(&package, class, &method);
            let rust_sig = rust.get(&symbol).unwrap_or_else(|| {
                panic!(
                    "Kotlin `{package}.{class}.{method}` expects Rust extern `{symbol}` — \
                     not found among: {:?}",
                    rust.keys().collect::<Vec<_>>()
                )
            });
            assert_eq!(
                rust_sig.params.len(),
                kt_sig.params.len(),
                "arity mismatch for `{package}.{class}.{method}` / `{symbol}`: \
                 Rust {:?} vs Kotlin {:?}",
                rust_sig.params,
                kt_sig.params,
            );
            for (i, (rw, kt)) in rust_sig.params.iter().zip(&kt_sig.params).enumerate() {
                assert!(
                    wire_compatible(rw, kt),
                    "param {i} of `{package}.{class}.{method}` / `{symbol}`: \
                     Rust wire `{rw}` incompatible with Kotlin `{kt}`",
                );
            }
            match (&rust_sig.ret, &kt_sig.ret) {
                (None, None) => {}
                (Some(rw), Some(kt)) => assert!(
                    wire_compatible(rw, kt),
                    "return of `{package}.{class}.{method}` / `{symbol}`: \
                     Rust wire `{rw}` incompatible with Kotlin `{kt}`",
                ),
                (r, k) => panic!(
                    "return presence mismatch for `{package}.{class}.{method}` / \
                     `{symbol}`: Rust {r:?} vs Kotlin {k:?}"
                ),
            }
            if let Some(prev) =
                claimed.insert(symbol.clone(), format!("{package}.{class}.{method}"))
            {
                panic!(
                    "symbol `{symbol}` claimed twice: `{prev}` and `{package}.{class}.{method}`"
                );
            }
        }
    }

    let orphans: Vec<&String> = rust
        .keys()
        .filter(|sym| !claimed.contains_key(*sym))
        .collect();
    assert!(
        orphans.is_empty(),
        "Rust externs with no Kotlin declaration: {orphans:?}"
    );
}

/// Run a full pipeline and return both artifacts.
fn run_pipeline(
    tag: &str,
    items: Vec<(syn::Item, crate::SourceLocation)>,
    jni: JniGen,
) -> (String, BTreeMap<String, String>) {
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let mut kotlin = BTreeMap::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        kotlin.insert(name, std::fs::read_to_string(p).unwrap());
    }
    (rust, kotlin)
}

/// Handles, fallible constructor, enum params/returns, `Option<&T>` borrow,
/// by-value consume, `Option<primitive>` scalar pair, and a declared const —
/// the wire shapes of the representative snapshot fixture, checked
/// cross-artifact.
#[test]
fn cross_artifact_representative_shapes_agree() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Error {
                    pub message: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Color {
                    Red,
                    Green = 5,
                    Blue,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Const(syn::parse_quote!(
                pub const MAX_LEN: i32 = 128;
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_new() -> Result<ZThing, Error> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
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
                pub fn z_thing_consume(t: ZThing) -> bool {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_peek(t: Option<&ZThing>, budget: Option<i32>) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_paint(c: Color) -> Color {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Error))
                .class(crate::ptr_class!(ZThing))
                .class(crate::enum_class!(Color)),
        )
        .package(
            crate::package!("thing")
                .fun(crate::fun!(z_thing_new))
                .fun(crate::fun!(z_thing_name))
                .fun(crate::fun!(z_thing_consume))
                .fun(crate::fun!(z_thing_peek))
                .fun(crate::fun!(z_paint))
                .constant(crate::constant!(MAX_LEN)),
        );
    let (rust, kotlin) = run_pipeline("jnigen_xart_repr", items, jni);
    assert_cross_artifact(&rust, &kotlin);
}

/// Flattenable data-class inputs, `&[T]` vec-build helper externs,
/// `impl Fn(...)` callbacks, and a builder-delivered (`expand_return`)
/// return — the multi-param / synthetic-extern shapes, checked
/// cross-artifact.
#[test]
fn cross_artifact_flatten_vec_callback_builder_agree() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Payload {
                    pub id: i64,
                    pub name: String,
                }
            )),
            loc.clone(),
        ),
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
                pub fn z_thing_get() -> ZThing {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn take_payload(p: Payload, maybe: Option<Payload>) -> i32 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn take_many(ps: &[Payload]) -> i32 {
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
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::data_class!(Payload))
                .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
                .fun(crate::fun!(z_thing_get))
                .fun(crate::fun!(take_payload))
                .fun(crate::fun!(take_many))
                .fun(crate::fun!(z_thing_sub)),
        )
        // Canonical output: handle (identity) + its string form — a return /
        // callback arg of ZThing decomposes into these 2 leaves (builder
        // delivery for `z_thing_get`).
        .expand(
            crate::expand_return!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );
    let (rust, kotlin) = run_pipeline("jnigen_xart_shapes", items, jni);
    assert_cross_artifact(&rust, &kotlin);
}

/// Record-built `Iterable` folds, bare AND `Optional`-wrapped (issue #105):
/// a `Vec<ZThing>` and an `Option<Vec<ZThing>>` return, both decomposed via
/// the same `expand_return!`. The extern must take the fold pair
/// (`__acc`/`__fold` ↔ `acc: Any?`/`fold: Any`) for BOTH shapes, and the
/// wrapper surface must be the generic fold — `<A>(…, acc: A, fold:
/// ZThingFolder<A>)` returning `A` (bare) / `A?` (`None` ⇒ null, the fold
/// never invoked).
#[test]
fn cross_artifact_optional_iterable_fold_agrees() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, crate::SourceLocation)> = vec![
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
                pub fn z_things_maybe() -> Option<Vec<ZThing>> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing).method(crate::fun!(z_thing_name).name("name")))
                .fun(crate::fun!(z_things_all))
                .fun(crate::fun!(z_things_maybe)),
        )
        .expand(
            crate::expand_return!(ZThing)
                .field_self()
                .field(crate::fun!(z_thing_name)),
        );
    let (rust, kotlin) = run_pipeline("jnigen_xart_opt_fold", items, jni);
    assert_cross_artifact(&rust, &kotlin);

    // Both externs take the fold pair on the Rust side…
    let rc: String = rust.chars().filter(|c| !c.is_whitespace()).collect();
    for sym in ["zThingsAll", "zThingsMaybe"] {
        assert!(
            rc.contains(&format!(
                "{sym}<'a>(mutenv:jni::JNIEnv<'a>,_class:jni::objects::JClass<'a>,\
                 __acc:jni::objects::JObject<'a>,__fold:jni::objects::JObject<'a>,"
            )),
            "extern `{sym}` takes the (__acc, __fold) pair:\n{rust}"
        );
    }
    // …and the Kotlin wrapper surface is the generic fold on both, returning
    // `A` for the bare shape and `A?` for the `Optional`-wrapped one.
    let wrappers = kotlin
        .values()
        .find(|src| src.contains("fun <A> zThingsAll"))
        .expect("a generated file declares the fold wrappers");
    let kc: String = wrappers.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        kc.contains("fun<A>zThingsAll(acc:A,onError:JniErrorHandler<A>,fold:ZThingFolder<A>):A{"),
        "bare fold wrapper surface:\n{wrappers}"
    );
    assert!(
        kc.contains(
            "fun<A>zThingsMaybe(acc:A,onError:JniErrorHandler<A?>,fold:ZThingFolder<A>):A?{"
        ),
        "optional fold wrapper surface (returns A?, null = None):\n{wrappers}"
    );
    assert!(
        kc.contains("JNINative.zThingsMaybe(acc,fold.asRaw(),__cap)asA?"),
        "optional fold call site casts the erased result to A?:\n{wrappers}"
    );
}
