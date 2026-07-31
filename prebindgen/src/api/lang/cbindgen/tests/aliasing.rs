//! The runtime alias preflight (#189): which calls get one, which do not, and
//! what it compares.
//!
//! Two arguments of one call can name the same resource — `z_combine(x, x)`
//! reconstructs one allocation twice. Rejecting the *declaration* would remove
//! shapes that ship today, so the generator emits a pointer-identity check
//! instead, ahead of every conversion. The runtime behaviour is pinned by
//! `example-cbindgen`'s `boundary_tests`; these tests pin **when** the check is
//! emitted, which is the part a refactor can silently change.

use super::*;

/// The fixture: one opaque handle plus whichever function under test.
fn build(fns: &[&str]) -> String {
    let loc = SourceLocation::default();
    let mut items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Thing {
                    pub v: u64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Other {
                    pub v: u64,
                }
            )),
            loc.clone(),
        ),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ];
    let mut idents: Vec<syn::Ident> = Vec::new();
    for src in fns {
        let f: syn::ItemFn = syn::parse_str(src).expect("test fn");
        idents.push(f.sig.ident.clone());
        items.push((syn::Item::Fn(f), loc.clone()));
    }
    let registry =
        crate::api::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let mut cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .free_memory_function("my_free")
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_function(|n| n.to_string())
        .opaque_ptr(syn::parse_quote!(Thing))
        .opaque_ptr(syn::parse_quote!(Other))
        .data_struct(syn::parse_quote!(Error))
        .error();
    for id in idents {
        cbindgen = cbindgen.function(syn::parse_quote!(#id)).panic();
    }
    write(cbindgen, registry, "cbindgen_aliasing")
}

/// The generation predicate, cell by cell. Emitted when the call has **at least
/// one `Consume` or `ExclusiveBorrow`** *and* **any other active access in the
/// same resource domain** — which is deliberately wider than "two or more
/// consumed parameters", the reading that would have skipped the two mixed
/// rows below.
#[test]
fn preflight_is_emitted_exactly_for_the_predicate() {
    for (src, want) in [
        // consume × consume
        (
            "pub fn f(a: Thing, b: Thing) -> Result<u64, Error> { unimplemented!() }",
            true,
        ),
        // consume × shared borrow — the borrow dangles when the consume takes
        // ownership.
        (
            "pub fn f(a: Thing, b: &Thing) -> Result<u64, Error> { unimplemented!() }",
            true,
        ),
        // exclusive borrow × shared borrow — the `&mut` is not exclusive at all.
        (
            "pub fn f(a: &mut Thing, b: &Thing) -> Result<u64, Error> { unimplemented!() }",
            true,
        ),
        // exclusive × exclusive
        (
            "pub fn f(a: &mut Thing, b: &mut Thing) -> Result<u64, Error> { unimplemented!() }",
            true,
        ),
        // shared × shared — legal Rust, legal C, no preflight.
        (
            "pub fn f(a: &Thing, b: &Thing) -> Result<u64, Error> { unimplemented!() }",
            false,
        ),
        // one resource only.
        (
            "pub fn f(a: Thing, b: u64) -> Result<u64, Error> { unimplemented!() }",
            false,
        ),
        // two consumes of DIFFERENT domains — distinct allocations, nothing to
        // compare.
        (
            "pub fn f(a: Thing, b: Other) -> Result<u64, Error> { unimplemented!() }",
            false,
        ),
    ] {
        let src_gen = build(&[src]);
        assert_eq!(
            src_gen.contains("aliasing arguments"),
            want,
            "preflight emission wrong for `{src}`:\n{src_gen}"
        );
    }
}

/// `T` and `Option<T>` are the same resource domain: both arrive as the same
/// handle pointer. Comparing the syntactic parameter type instead would let
/// `f(x, Some(x))` through — the exact miss #187's review called out.
#[test]
fn option_and_bare_share_a_resource_domain() {
    let src =
        build(&["pub fn f(a: Thing, b: Option<Thing>) -> Result<u64, Error> { unimplemented!() }"]);
    assert!(src.contains("aliasing arguments"), "{src}");
    let compact: String = src.split_whitespace().collect();
    assert!(
        compact.contains("if!(aas*const()).is_null()&&(aas*const())==(bas*const())"),
        "{src}"
    );
}

/// The preflight runs **before** the first conversion — not inside one, and not
/// between them. By the time a converter has run, one of the aliased arguments
/// has already been consumed and there is nothing left to reject.
#[test]
fn preflight_precedes_every_conversion() {
    let src = build(&["pub fn f(a: Thing, b: Thing) -> Result<u64, Error> { unimplemented!() }"]);
    let alias_at = src.find("aliasing arguments").expect("preflight emitted");
    let first_conv = src
        .find("__cbg_in_Thing(a)")
        .expect("input conversion emitted");
    assert!(
        alias_at < first_conv,
        "preflight must precede the first conversion:\n{src}"
    );
}

/// Three resources of one domain produce all three pairwise checks — the rule
/// is over pairs, not over "the first two handles".
#[test]
fn every_qualifying_pair_is_checked() {
    let src = build(&[
        "pub fn f(a: Thing, b: &Thing, c: &Thing) -> Result<u64, Error> { unimplemented!() }",
    ]);
    let compact: String = src.split_whitespace().collect();
    // a×b and a×c qualify (a consumes); b×c is borrow/borrow and does not.
    assert!(compact.contains("(aas*const())==(bas*const())"), "{src}");
    assert!(compact.contains("(aas*const())==(cas*const())"), "{src}");
    assert!(!compact.contains("(bas*const())==(cas*const())"), "{src}");
}
