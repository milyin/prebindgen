//! The Kotlin-side alias preflight (#189) — the JNI counterpart of the C one
//! in `lang::cbindgen::tests::aliasing`.
//!
//! Two typed handles a caller passes to one wrapper can be the same object, or
//! two objects over one native allocation. `zCombine(x, x)` then hands that
//! allocation to two consuming converters. Rejecting the declaration is not an
//! option — it is a supported shape today — so the wrapper compares `ptr`
//! before the lock and before any conversion, and routes the rejection through
//! the same binding-error channel a closed handle uses.

use super::*;

/// One `ptr_class` handle (plus a second, unrelated one) and whichever
/// functions are under test.
fn build(fns: &[&str], tag: &str) -> String {
    let loc = myflat_loc();
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZOther {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
    ];
    let mut decls = crate::package!("ops")
        .class(crate::ptr_class!(ZThing))
        .class(crate::ptr_class!(ZOther));
    for src in fns {
        let f: syn::ItemFn = syn::parse_str(src).expect("test fn");
        let id = f.sig.ident.clone();
        decls = decls.fun(crate::lang::FunctionDecl::new(id));
        items.push((syn::Item::Fn(f), loc.clone()));
    }
    let registry = crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(decls);
    super::flatten::write_all(registry.resolve(jni).expect("resolve"), tag)
}

/// The generation predicate on the Kotlin side. JniGen has no exclusive-borrow
/// mode of its own — `&T` and `&mut T` both reach Kotlin as a locked borrow —
/// so the rule reduces to "at least one consumed handle, and any other handle
/// in the same domain".
#[test]
fn preflight_is_emitted_exactly_for_the_predicate() {
    for (src, want, tag) in [
        // consume × consume
        (
            "pub fn z_op(a: ZThing, b: ZThing) -> i64 { unimplemented!() }",
            true,
            "jni_alias_cc",
        ),
        // consume × borrow
        (
            "pub fn z_op(a: ZThing, b: &ZThing) -> i64 { unimplemented!() }",
            true,
            "jni_alias_cb",
        ),
        // borrow × borrow — legal, no preflight.
        (
            "pub fn z_op(a: &ZThing, b: &ZThing) -> i64 { unimplemented!() }",
            false,
            "jni_alias_bb",
        ),
        // one handle only.
        (
            "pub fn z_op(a: ZThing, b: i64) -> i64 { unimplemented!() }",
            false,
            "jni_alias_one",
        ),
        // two consumes of DIFFERENT classes — distinct allocations, so their
        // pointers can never be equal and the check would be dead code.
        (
            "pub fn z_op(a: ZThing, b: ZOther) -> i64 { unimplemented!() }",
            false,
            "jni_alias_xtype",
        ),
    ] {
        let out = build(&[src], tag);
        assert_eq!(
            out.contains("Aliasing arguments"),
            want,
            "preflight emission wrong for `{src}`:\n{out}"
        );
    }
}

/// `T` and `Option<T>` are one resource domain. The comparison is on `ptr` —
/// the resource's own address — precisely so the two spellings do not have to
/// be enumerated, and so a caller passing the same handle as `a` and as
/// `Some(a)` is caught.
#[test]
fn option_and_bare_share_a_resource_domain() {
    let out = build(
        &["pub fn z_op(a: ZThing, b: Option<ZThing>) -> i64 { unimplemented!() }"],
        "jni_alias_option",
    );
    let all: String = out.split_whitespace().collect();
    assert!(out.contains("Aliasing arguments"), "{out}");
    assert!(all.contains("a.ptr!=0L&&a.ptr==(b?.ptr?:0L)"), "{out}");
}

/// The preflight runs **before** the lock, before the closed-handle guard's
/// neighbours, and before the native call — i.e. before anything has consumed
/// or converted an argument.
#[test]
fn preflight_precedes_the_lock_and_the_call() {
    let all = build(
        &["pub fn z_op(a: ZThing, b: ZThing) -> i64 { unimplemented!() }"],
        "jni_alias_order",
    );
    // `write_all` concatenates every emitted file, and the runtime support file
    // mentions the lock helper too — so the ordering is read inside the wrapper.
    let start = all.find("public fun zOp(").expect("wrapper emitted");
    let out = &all[start..];
    let alias_at = out.find("Aliasing arguments").expect("preflight emitted");
    let lock_at = out
        .find("withSortedHandleLocks")
        .expect("lock scaffold emitted");
    assert!(
        alias_at < lock_at,
        "preflight must precede the lock:\n{out}"
    );
    let call_at = out.find("markConsumed").expect("consume emitted");
    assert!(
        alias_at < call_at,
        "preflight must precede the consume:\n{out}"
    );
}
