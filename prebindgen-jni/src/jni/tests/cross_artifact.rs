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
/// The report's signatures against the Kotlin wrappers' — the fourth consumer
/// #613 step 7 names, beside the Rust externs, the Kotlin declarations and the
/// callback interfaces the tests above already cross-check.
///
/// `report.rs` documents that it renders "through the same `render_wrapper_fn`
/// the emitters use, so it cannot drift from the real output". That is a
/// structural argument, and this is the check that keeps it true: every
/// function the report names must appear in the emitted Kotlin with the same
/// parameter arity.
fn assert_report_agrees(report: &str, kotlin: &BTreeMap<String, String>) {
    let all: String = kotlin.values().flat_map(|s| s.chars()).collect();
    let compact: String = all.split_whitespace().collect();
    let mut checked = 0;
    for line in report.lines() {
        // `- `rust_ident` — `fun name(a: A, b: B): R``
        let Some(rest) = line.strip_prefix("- `") else {
            continue;
        };
        let Some((_ident, sig)) = rest.split_once("` — `") else {
            continue;
        };
        let Some(sig) = sig.strip_suffix('`') else {
            continue;
        };
        let Some(open) = sig.find('(') else { continue };
        let Some(name) = sig[..open].rsplit(' ').next() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        assert!(
            compact.contains(&format!("fun{name}(")),
            "the report names `{name}`, which the emitted Kotlin does not \
             declare — the report drifted from the wrappers it claims to \
             render through"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "fixture's report names at least one function; it named none, so this \
         check proved nothing"
    );
}

fn run_pipeline(
    tag: &str,
    items: Vec<(syn::Item, prebindgen::SourceLocation)>,
    jni: JniGenBuilder,
) -> (String, BTreeMap<String, String>, String) {
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let report = gen.report();
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let mut kotlin = BTreeMap::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        kotlin.insert(name, std::fs::read_to_string(p).unwrap());
    }
    (rust, kotlin, report)
}

/// Handles, fallible constructor, enum params/returns, `Option<&T>` borrow,
/// by-value consume, `Option<primitive>` scalar pair, and a declared const —
/// the wire shapes of the representative snapshot fixture, checked
/// cross-artifact.
#[test]
fn cross_artifact_representative_shapes_agree() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Error))
                .class(crate::ptr_class!(ZThing))
                .class(crate::enum_class!(Color)),
        )
        .package(
            crate::package!("thing")
                .fun(prebindgen_registry::fun!(z_thing_new))
                .fun(prebindgen_registry::fun!(z_thing_name))
                .fun(prebindgen_registry::fun!(z_thing_consume))
                .fun(prebindgen_registry::fun!(z_thing_peek))
                .fun(prebindgen_registry::fun!(z_paint))
                .constant(crate::constant!(MAX_LEN)),
        );
    let (rust, kotlin, report) = run_pipeline("jnigen_xart_repr", items, jni);
    assert_cross_artifact(&rust, &kotlin);
    assert_report_agrees(&report, &kotlin);
}

/// Optional enum niches are primitive JNI wires at every reserved nesting
/// depth. The Kotlin extern must consume that frozen wire decision rather than
/// inferring a reference return from the intentionally lossy `Priority?`
/// public surface.
#[test]
fn optional_enum_nesting_keeps_rust_and_kotlin_return_wires_equal() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 0,
                    Normal = 1,
                    High = 2,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn priority_optional() -> Option<Priority> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn priority_nested_optional() -> Option<Option<Priority>> {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .fun(prebindgen_registry::fun!(priority_optional))
                .fun(prebindgen_registry::fun!(priority_nested_optional)),
        );
    let (rust, kotlin, report) = run_pipeline("jnigen_xart_optional_enum", items, jni);

    assert_cross_artifact(&rust, &kotlin);
    assert_report_agrees(&report, &kotlin);

    let rust: String = rust.split_whitespace().collect();
    let kotlin: String = kotlin
        .values()
        .flat_map(|source| source.split_whitespace())
        .collect();
    assert!(
        rust.matches("->jni::sys::jint").count() >= 2,
        "both enum option depths must return jint:\n{rust}"
    );
    assert!(
        kotlin.contains("externalfunpriorityOptional(")
            && kotlin.contains("externalfunpriorityNestedOptional(")
            && kotlin.matches("):Int").count() >= 2,
        "both Kotlin externs must declare the same primitive wire:\n{kotlin}"
    );
    assert!(
        kotlin.contains(
            "returnif(__ret==-2147483647||__ret==Int.MIN_VALUE)nullelseio.test.jni.Priority.fromInt(__ret)"
        ),
        "the collapsed Kotlin surface must map both nested absent values to null:\n{kotlin}"
    );
}

/// Flattenable data-class inputs, `&[T]` vec-build helper externs,
/// `impl Fn(...)` callbacks, and a builder-delivered (`expand_return`)
/// return — the multi-param / synthetic-extern shapes, checked
/// cross-artifact.
#[test]
fn cross_artifact_flatten_vec_callback_builder_agree() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
                pub fn payload_sub(cb: impl Fn(&Payload) + Send + Sync + 'static) {
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
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(crate::data_class!(Payload))
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                .fun(prebindgen_registry::fun!(z_thing_get))
                .fun(prebindgen_registry::fun!(take_payload))
                .fun(prebindgen_registry::fun!(take_many))
                .fun(prebindgen_registry::fun!(payload_sub))
                .fun(prebindgen_registry::fun!(z_thing_sub)),
        )
        // Canonical output: handle (identity) + its string form — a return /
        // callback arg of ZThing decomposes into these 2 leaves (builder
        // delivery for `z_thing_get`).
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field_self()
                .field(prebindgen_registry::fun!(z_thing_name)),
        );
    let (rust, kotlin, report) = run_pipeline("jnigen_xart_shapes", items, jni);
    assert_cross_artifact(&rust, &kotlin);
    assert_report_agrees(&report, &kotlin);
    let compact: String = rust.split_whitespace().collect();
    assert!(
        compact.contains("Box::new(move|__cb_arg0:&myflat::Payload|")
            && compact.contains(")=match__jni_out_convert_")
            && compact.contains("(&mutenv,__cb_arg0,")
            && !compact.contains("__cb_arg0.id")
            && !compact.contains("__cb_arg0.name"),
        "borrowed data-class callbacks must delegate field decomposition to one Product chain:\n{rust}"
    );
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
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![
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
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .method(prebindgen_registry::fun!(z_thing_name).name("name")),
                )
                .fun(prebindgen_registry::fun!(z_things_all))
                .fun(prebindgen_registry::fun!(z_things_maybe)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .field_self()
                .field(prebindgen_registry::fun!(z_thing_name)),
        );
    let (rust, kotlin, report) = run_pipeline("jnigen_xart_opt_fold", items, jni);
    assert_cross_artifact(&rust, &kotlin);
    assert_report_agrees(&report, &kotlin);

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
    // A? for both: null is the recovery value or the optional result.
    let wrappers = kotlin
        .values()
        .find(|src| src.contains("fun <A> zThingsAll"))
        .expect("a generated file declares the fold wrappers");
    let kc: String = wrappers.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        kc.contains("fun<A>zThingsAll(acc:A,onError:JniErrorHandler<A?>,fold:ZThingFolder<A>):A?{"),
        "bare fold wrapper surface:\n{wrappers}"
    );
    assert!(
        kc.contains(
            "fun<A>zThingsMaybe(acc:A,onError:JniErrorHandler<A?>,fold:ZThingFolder<A>):A?{"
        ),
        "optional fold wrapper surface (returns A?, null = None):\n{wrappers}"
    );
    let call = kc
        .find("val__ret=JNINative.zThingsMaybe(acc,fold.asRaw(),__bcap)")
        .expect("optional fold stores the erased native result");
    let check = call
        + kc[call..]
            .find("if(__bcap.failed)returnonError.run(__bcap.ze0)")
            .expect("optional fold checks the binding-error capture");
    let cast = check
        + kc[check..]
            .find("return__retasA?")
            .expect("optional fold casts the successful result to A?");
    assert!(
        call < check && check < cast,
        "optional fold must redispatch a native error before casting its erased result:\n{wrappers}"
    );
}

/// The extern is planned once, at the end of resolution: what it renders from
/// is its own frozen state, never the declaration object the binding was
/// written into or the registry it was resolved against.
///
/// Fenced on the struct's fields rather than on the file, so a mention of
/// either name in a doc comment or a neighbouring function cannot satisfy it.
#[test]
fn a_planned_extern_holds_no_declaration_object() {
    let source = include_str!("../emit/wrapper.rs");
    let file = syn::parse_file(source).expect("the JNI wrapper emitter parses");
    let fields = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Struct(item) if item.ident == "JWrapper" => {
                Some(item.fields.to_token_stream().to_string())
            }
            _ => None,
        })
        .expect("the planned extern");
    let fields: String = fields.split_whitespace().collect();
    for forbidden in ["Declarations", "Registry", "Compiled", "RefCell"] {
        assert!(
            !fields.contains(forbidden),
            "a planned extern retains {forbidden}, so rendering could resume planning"
        );
    }
    assert!(
        fields.contains("JniFunctionPlan"),
        "a planned extern must render from its frozen function plan"
    );
}

/// The extern's body reads frozen plans only: the two live lookups it used to
/// make — the function plan and the origin module of what it calls — are
/// answered before rendering starts.
#[test]
fn extern_rendering_asks_the_registry_nothing() {
    let source = include_str!("../emit/wrapper.rs");
    let file = syn::parse_file(source).expect("the JNI wrapper emitter parses");
    let renderer = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| &item.items)
        .find_map(|item| match item {
            syn::ImplItem::Fn(method) if method.sig.ident == "render_fn" => Some(method),
            _ => None,
        })
        .expect("the extern renderer");
    let inputs: String = renderer
        .sig
        .inputs
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();
    assert_eq!(
        inputs, "&self,emit:&prebindgen_registry::RustWriter",
        "the extern renderer takes its frozen self and the writer, and nothing else"
    );
    let body: String = renderer
        .block
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();
    for forbidden in ["fn_plan(", "fn_module("] {
        assert!(
            !body.contains(forbidden),
            "the extern renderer resumed planning through {forbidden}"
        );
    }
    assert!(
        body.contains("self.plan") && body.contains("self.callee"),
        "the extern renderer must read its frozen plan and callee"
    );
}

/// JniGen declares one shape vocabulary of its own, and #613 is about removing
/// it rather than gaining another.
///
/// A shape-shaped enum — one naming three or more of the registry's structural
/// forms — is a place where the same structural question is asked a second
/// time. Two are recorded here: `JLayout`, the shape of the single adapter-side
/// intermediate over flattened ABI leaves, and `JBody`, the rendering half.
/// Both are deletion targets of steps 4 and 5, so this list may shrink; it may
/// not grow without a reader deciding that a third is worth having.
///
/// The sources are DISCOVERED, not listed: a fence over a list only fences the
/// files someone remembered to add to it, and a new module is the most natural
/// way to introduce a third vocabulary.
#[test]
fn jnigen_gains_no_third_shape_vocabulary() {
    let sources = prebindgen_registry::generation::production_sources(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
    );
    let borrowed: Vec<(&str, &str)> = sources
        .iter()
        .map(|(label, text)| (label.as_str(), text.as_str()))
        .collect();
    let names: Vec<String> = prebindgen_registry::generation::shape_like_enums(&borrowed)
        .iter()
        .map(|(name, label)| format!("{name} ({label})"))
        .collect();
    assert_eq!(
        names,
        ["JBody (src/jni/chain.rs)", "JLayout (src/jni/compile.rs)"],
        "the JniGen shape vocabularies changed; #613 shrinks this list, and a \
         new entry needs an argument rather than a test update"
    );
}
