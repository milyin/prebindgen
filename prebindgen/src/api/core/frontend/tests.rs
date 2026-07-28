//! The frontend acceptance matrix.
//!
//! One table per closed subgrammar: source spelling → the lowered value, or the
//! precise reason it is refused. A new accepted or refused shape is a row, not a
//! test — which is the difference between this and the characterization tests it
//! replaces, each of which was written one defect at a time (issue #210).

use quote::ToTokens;

use super::{
    array_len::lower_array_len, resolve_array_lengths, ArrayLen, ArrayLenReason, ConstIndex,
};

/// The consts every row below resolves against, all marked in `myflat`:
///
/// * `MAX` — an ordinary literal-valued const;
/// * `SUFFIXED` — the same, written with a suffix;
/// * `COMPUTED` — marked, but its value is an expression `build.rs` cannot
///   evaluate;
/// * `OTHER_MAX` — marked in a DIFFERENT source crate, for the provenance rows.
fn consts() -> ConstIndex {
    ConstIndex::new([
        (
            "MAX".to_string(),
            syn::parse_quote!(4),
            Some("myflat".to_string()),
        ),
        (
            "SUFFIXED".to_string(),
            syn::parse_quote!(16usize),
            Some("myflat".to_string()),
        ),
        (
            "COMPUTED".to_string(),
            syn::parse_quote!(2 * 4),
            Some("myflat".to_string()),
        ),
        (
            "OTHER_MAX".to_string(),
            syn::parse_quote!(8),
            Some("helpers".to_string()),
        ),
    ])
}

/// Lower as if written inside an item from `myflat`.
fn lower(src: &str) -> Result<ArrayLen, ArrayLenReason> {
    let expr: syn::Expr = syn::parse_str(src).expect("test input must parse");
    lower_array_len(&expr, "[u8 ; …]", Some("myflat"), &consts()).map_err(|e| e.reason)
}

/// Render a lowered length the way it will be spelled in generated code.
fn spelled(len: &ArrayLen) -> String {
    len.to_expr().to_token_stream().to_string()
}

/// ACCEPTED shapes: what each lowers to, and how it is then spelled.
///
/// Every accepted length is a NUMBER — that is the contract, not an
/// implementation detail. A generator runs in `build.rs` and cannot evaluate
/// Rust, and a destination language that groups a small array into scalars needs
/// the count literally.
///
/// The spelling column is the half that used to be a separate walk. Pairing it
/// with acceptance in one table is the point: a row cannot say "accepted"
/// without also saying what it emits.
#[test]
fn accepted_array_lengths() {
    let cases: &[(&str, ArrayLen, &str)] = &[
        ("4", ArrayLen::Literal(4), "4"),
        ("0", ArrayLen::Literal(0), "0"),
        ("16usize", ArrayLen::Literal(16), "16"),
        // A marked const is evaluated here and emitted as its VALUE, so nothing
        // in generated code is a path and there is nothing to qualify.
        (
            "MAX",
            ArrayLen::Const {
                name: syn::parse_quote!(MAX),
                value: 4,
            },
            "4",
        ),
        (
            "SUFFIXED",
            ArrayLen::Const {
                name: syn::parse_quote!(SUFFIXED),
                value: 16,
            },
            "16",
        ),
    ];
    for (src, expect, spell) in cases {
        let got = lower(src).unwrap_or_else(|r| panic!("`{src}` was refused: {r:?}"));
        assert_eq!(&got, expect, "`{src}` lowered to the wrong value");
        assert_eq!(&spelled(&got), spell, "`{src}` is spelled wrong");
        assert_eq!(got.value(), expect.value(), "`{src}` has the wrong value");
    }
}

/// REFUSED shapes and why.
///
/// There is no separate list of accepted forms for this to drift from — a shape
/// absent from the table above is refused by construction.
#[test]
fn refused_array_lengths() {
    let cases: &[(&str, ArrayLenReason)] = &[
        // ── not a bare name ──────────────────────────────────────────────
        // Every one of these was a live defect or a live ambiguity while the
        // grammar still admitted paths. A `#[prebindgen]` const is addressed by
        // its bare name in one flat namespace, so a longer path either restates
        // that or reaches somewhere the frontend cannot follow.
        //
        // Issue #210 defect #8: accepted by the old whitelist, silently NOT
        // qualified by the old rewriter.
        ("<Holder>::N", ArrayLenReason::NotABareName),
        ("<Holder as Trait>::N", ArrayLenReason::NotABareName),
        // An associated const. prebindgen never captures `impl` blocks, so its
        // value is unknowable — supporting the spelling could only ever have
        // produced a path, never a number.
        ("Holder::N", ArrayLenReason::NotABareName),
        // A module path. Indistinguishable from an external crate path without
        // indexing modules, and redundant when the const IS marked.
        ("crate::limits::MAX", ArrayLenReason::NotABareName),
        ("myflat::limits::MAX", ArrayLenReason::NotABareName),
        ("limits::MAX", ArrayLenReason::NotABareName),
        ("crate::MAX", ArrayLenReason::NotABareName),
        // An external path. `usize::MAX` is not even a fixed number — it is
        // platform dependent, which is the whole hazard in miniature.
        ("usize::MAX", ArrayLenReason::NotABareName),
        ("other_crate::Holder::N", ArrayLenReason::NotABareName),
        ("::MAX", ArrayLenReason::NotABareName),
        // ── a bare name that is not a usable const ───────────────────────
        // The generated crate sees ONLY what the macro exposed, so an unmarked
        // const does not exist downstream. This used to be emitted verbatim and
        // failed later, at rustc; it is a frontend error now.
        ("UNMARKED", ArrayLenReason::NotAMarkedConst),
        // Marked, but `build.rs` cannot evaluate it.
        ("COMPUTED", ArrayLenReason::ConstIsNotALiteral),
        // ── not a literal or a name ──────────────────────────────────────
        ("MAX + 1", ArrayLenReason::NotLiteralOrName),
        ("-1", ArrayLenReason::NotLiteralOrName),
        ("MAX as usize", ArrayLenReason::NotLiteralOrName),
        ("(MAX)", ArrayLenReason::NotLiteralOrName),
        ("array_len()", ArrayLenReason::NotLiteralOrName),
        // Anything that can BIND a name.
        (
            "const { let array_len = 3; array_len }",
            ArrayLenReason::NotLiteralOrName,
        ),
        ("{ 3 }", ArrayLenReason::NotLiteralOrName),
        (
            "match 3 { array_len => array_len }",
            ArrayLenReason::NotLiteralOrName,
        ),
        (
            "if let array_len = 3 { array_len } else { 0 }",
            ArrayLenReason::NotLiteralOrName,
        ),
        ("|| 3", ArrayLenReason::NotLiteralOrName),
        // ── literals that are not lengths ────────────────────────────────
        ("\"4\"", ArrayLenReason::NotAnIntegerLiteral),
        ("true", ArrayLenReason::NotAnIntegerLiteral),
        (
            "99999999999999999999999999999999999",
            ArrayLenReason::IntegerOutOfRange,
        ),
    ];
    for (src, reason) in cases {
        match lower(src) {
            Ok(v) => panic!("`{src}` was accepted as {v:?}"),
            Err(got) => assert_eq!(&got, reason, "`{src}` refused for the wrong reason"),
        }
    }
}

/// A bare name must be a const from the item's OWN source crate.
///
/// Uniqueness holds across the **marked** namespace only. A source crate may
/// have an unmarked `MAX` of its own and mean that one, while another chained
/// source has a marked `MAX` — and the marked one is the only thing the frontend
/// can see. Resolving to it would silently change the length rather than fail.
///
/// The value is deliberately *available* for the foreign const, so this pins
/// provenance rather than incidental unresolvability.
#[test]
fn a_length_must_name_a_const_from_its_own_crate() {
    let expr: syn::Expr = syn::parse_quote!(OTHER_MAX);
    let err = lower_array_len(&expr, "[u8 ; OTHER_MAX]", Some("myflat"), &consts()).unwrap_err();
    assert_eq!(
        err.reason,
        ArrayLenReason::ForeignSourceConst {
            const_crate: "helpers".to_string(),
            item_crate: "myflat".to_string(),
        }
    );
    // ...and from `helpers` itself the same name is fine.
    let got = lower_array_len(&expr, "[u8 ; OTHER_MAX]", Some("helpers"), &consts()).unwrap();
    assert_eq!(got.value(), 8);
}

/// An origin-less stream is one anonymous crate, so provenance matches
/// trivially. Core supports hand-built item streams with no
/// `SourceLocation::crate_name`, and they must not be collateral of the
/// provenance rule.
#[test]
fn origin_less_streams_resolve_against_themselves() {
    let consts = ConstIndex::new([("MAX".to_string(), syn::parse_quote!(4), None)]);
    let expr: syn::Expr = syn::parse_quote!(MAX);
    let got = lower_array_len(&expr, "[u8 ; MAX]", None, &consts).unwrap();
    assert_eq!(got.value(), 4);
    // A stamped item may not reach an unstamped const either — same rule.
    let err = lower_array_len(&expr, "[u8 ; MAX]", Some("myflat"), &consts).unwrap_err();
    assert!(matches!(
        err.reason,
        ArrayLenReason::ForeignSourceConst { .. }
    ));
}

/// The diagnostic names the offending sub-expression, not just the array — a
/// caller with several arrays in one struct has to be told which one.
#[test]
fn rejection_names_the_offending_expression() {
    let expr: syn::Expr = syn::parse_quote!(MAX + 1);
    let err = lower_array_len(&expr, "[u8 ; MAX + 1]", Some("myflat"), &consts()).unwrap_err();
    assert_eq!(err.offending, "MAX + 1");
    assert_eq!(err.array, "[u8 ; MAX + 1]");
    let msg = err.to_string();
    assert!(msg.contains("MAX + 1"), "{msg}");
    assert!(msg.contains("integer literal"), "{msg}");
}

/// A refused length leaves the node untouched — no half-rewritten model reaches
/// a caller that ignores the error (invariant 7 of issue #211).
#[test]
fn rejection_is_transactional() {
    let mut item: syn::ItemStruct = syn::parse_quote!(
        pub struct Blob {
            pub good: [u8; MAX],
            pub bad: [u8; MAX + 1],
        }
    );
    let before = item.to_token_stream().to_string();
    let err = resolve_array_lengths(&mut item, &consts(), Some("myflat"), |r, s| {
        syn::visit_mut::VisitMut::visit_item_struct_mut(r, s)
    })
    .unwrap_err();
    assert_eq!(err.offending, "MAX + 1");
    assert_eq!(
        item.to_token_stream().to_string(),
        before,
        "the accepted field was rewritten despite the refusal"
    );
}

/// Nested arrays lower innermost-first, and both levels are recorded.
#[test]
fn nested_arrays_resolve_at_every_level() {
    let mut item: syn::ItemStruct = syn::parse_quote!(
        pub struct Grid {
            pub cells: [[u8; MAX]; SUFFIXED],
        }
    );
    let found = resolve_array_lengths(&mut item, &consts(), Some("myflat"), |r, s| {
        syn::visit_mut::VisitMut::visit_item_struct_mut(r, s)
    })
    .unwrap();
    let rendered = item.to_token_stream().to_string().replace(' ', "");
    assert!(rendered.contains("[[u8;4];16]"), "{rendered}");
    // Inner first, and the outer type is recorded in its REWRITTEN spelling —
    // a consumer keying on it never sees the captured one.
    let keys: Vec<String> = found
        .iter()
        .map(|(ty, _)| ty.to_token_stream().to_string().replace(' ', ""))
        .collect();
    assert_eq!(keys, vec!["[u8;4]".to_string(), "[[u8;4];16]".to_string()]);
}

/// A length is resolved wherever a type can appear, not only in a struct field.
#[test]
fn lengths_resolve_in_function_signatures() {
    let mut item: syn::ItemFn = syn::parse_quote!(
        pub fn echo(v: [u8; MAX]) -> [u8; SUFFIXED] {
            unimplemented!()
        }
    );
    resolve_array_lengths(&mut item, &consts(), Some("myflat"), |r, f| {
        syn::visit_mut::VisitMut::visit_item_fn_mut(r, f)
    })
    .unwrap();
    let rendered = item.to_token_stream().to_string().replace(' ', "");
    assert!(rendered.contains("v:[u8;4]"), "{rendered}");
    assert!(rendered.contains("->[u8;16]"), "{rendered}");
}

/// A const length and the same number written literally are ONE type, and
/// therefore one converter. They always were in Rust; evaluating the length
/// makes the frontend agree.
#[test]
fn a_const_length_and_its_literal_are_the_same_type() {
    let from_const: syn::Expr = syn::parse_quote!(MAX);
    let from_literal: syn::Expr = syn::parse_quote!(4);
    let a = lower_array_len(&from_const, "[u8 ; MAX]", Some("myflat"), &consts()).unwrap();
    let b = lower_array_len(&from_literal, "[u8 ; 4]", Some("myflat"), &consts()).unwrap();
    assert_ne!(a, b, "the spellings stay distinguishable in the model");
    assert_eq!(spelled(&a), spelled(&b), "but they emit one type");
}
