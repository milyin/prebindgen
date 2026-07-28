//! The frontend acceptance matrix.
//!
//! One table per closed subgrammar: source spelling → the lowered value, or the
//! precise reason it is refused. A new accepted or refused shape is a row, not a
//! test — which is the difference between this and the characterization tests it
//! replaces, each of which was written one defect at a time (issue #210).

use std::collections::HashMap;

use quote::ToTokens;

use super::{lower_array_len, resolve_array_lengths, ArrayLen, ArrayLenReason, NameIndex};

/// The namespace every row below resolves against: a free const `MAX`, a type
/// `Holder` owning an associated const, and a `const fn` `array_len` — the three
/// item kinds a length could plausibly name. All three live in `myflat`.
fn names() -> NameIndex {
    let module: syn::Path = syn::parse_quote!(myflat);
    let names: HashMap<String, syn::Path> = ["MAX", "Holder", "array_len"]
        .into_iter()
        .map(|n| (n.to_string(), module.clone()))
        .collect();
    NameIndex::new(names, &["myflat".to_string()])
}

fn lower(src: &str) -> Result<ArrayLen, ArrayLenReason> {
    let expr: syn::Expr = syn::parse_str(src).expect("test input must parse");
    lower_array_len(&expr, "[u8 ; …]", &names()).map_err(|e| e.reason)
}

/// Render a lowered length the way it will be spelled in generated code.
fn spelled(len: &ArrayLen) -> String {
    len.to_expr().to_token_stream().to_string()
}

/// ACCEPTED shapes: what each lowers to, and how it is then spelled.
///
/// The spelling column is the half that used to be a separate walk. Pairing it
/// with acceptance in one table is the point: a row cannot say "accepted"
/// without also saying what it resolves to.
#[test]
fn accepted_array_lengths() {
    let cases: &[(&str, ArrayLen, &str)] = &[
        // Integer literals, including a suffixed one.
        ("4", ArrayLen::Literal(4), "4"),
        ("0", ArrayLen::Literal(0), "0"),
        ("16usize", ArrayLen::Literal(16), "16"),
        // A free const: the whole path is the name.
        (
            "MAX",
            ArrayLen::SourceConst {
                path: syn::parse_quote!(myflat::MAX),
            },
            "myflat :: MAX",
        ),
        // An associated const: only the OWNER is qualified — `::N` is relative
        // to it, so `myflat::Holder::myflat::N` must not be constructible.
        (
            "Holder::N",
            ArrayLen::SourceConst {
                path: syn::parse_quote!(myflat::Holder::N),
            },
            "myflat :: Holder :: N",
        ),
        // Already-qualified captured spellings resolve to the same value as the
        // bare ones, so a source crate that writes `crate::MAX` is not speaking
        // a different language from one that writes `MAX`.
        (
            "crate::MAX",
            ArrayLen::SourceConst {
                path: syn::parse_quote!(myflat::MAX),
            },
            "myflat :: MAX",
        ),
        (
            "myflat::MAX",
            ArrayLen::SourceConst {
                path: syn::parse_quote!(myflat::MAX),
            },
            "myflat :: MAX",
        ),
        // Stripping the source head keeps the OWNER, unlike type-path
        // reduction, which would collapse this to `N`.
        (
            "myflat::Holder::N",
            ArrayLen::SourceConst {
                path: syn::parse_quote!(myflat::Holder::N),
            },
            "myflat :: Holder :: N",
        ),
        // Not a source item: emitted verbatim. Prefixing an origin module here
        // would be a guess, and `usize` has none.
        (
            "usize::MAX",
            ArrayLen::ExternalConst {
                path: syn::parse_quote!(usize::MAX),
            },
            "usize :: MAX",
        ),
        // A const the source crate did not mark `#[prebindgen]` is, to the
        // registry, indistinguishable from a foreign one — it lands here and
        // will not resolve in the generated crate. Marking it is the fix.
        (
            "UNMARKED",
            ArrayLen::ExternalConst {
                path: syn::parse_quote!(UNMARKED),
            },
            "UNMARKED",
        ),
    ];
    for (src, expect, spell) in cases {
        let got = lower(src).unwrap_or_else(|r| panic!("`{src}` was refused: {r:?}"));
        assert_eq!(&got, expect, "`{src}` lowered to the wrong value");
        assert_eq!(&spelled(&got), spell, "`{src}` is spelled wrong");
    }
}

/// REFUSED shapes and why.
///
/// Everything that is not a literal or a plain path, plus the two path forms
/// that carry no resolvable leading segment. There is no separate list of
/// accepted forms for this to drift from — a shape absent from the table above
/// is refused by construction.
#[test]
fn refused_array_lengths() {
    let cases: &[(&str, ArrayLenReason)] = &[
        // Issue #210, defect #8: a qualified self was ACCEPTED by the old
        // whitelist and silently declined by the old rewriter, emitting
        // `<Holder>::N` into a crate where `Holder` is not in scope. One walk
        // makes accepted-but-unqualified unrepresentable.
        ("<Holder>::N", ArrayLenReason::QualifiedSelf),
        ("<Holder as Trait>::N", ArrayLenReason::QualifiedSelf),
        // Rooted at the crate root: names a dependency of the GENERATED crate,
        // which the frontend cannot see.
        ("::MAX", ArrayLenReason::CrateRootPath),
        // Const arithmetic and casts — deliberately out of the language.
        ("MAX + 1", ArrayLenReason::NotLiteralOrPath),
        ("-1", ArrayLenReason::NotLiteralOrPath),
        ("MAX as usize", ArrayLenReason::NotLiteralOrPath),
        ("(MAX)", ArrayLenReason::NotLiteralOrPath),
        // A `const fn` call. Its callee is an indexed item, but the call is
        // structure the flat grammar does not have.
        ("array_len()", ArrayLenReason::NotLiteralOrPath),
        // Anything that can BIND a name. A local shadowing a source item would
        // otherwise be rewritten into it — `array_len` the local becoming
        // `myflat::array_len` the function.
        (
            "const { let array_len = 3; array_len }",
            ArrayLenReason::NotLiteralOrPath,
        ),
        ("{ 3 }", ArrayLenReason::NotLiteralOrPath),
        (
            "match 3 { array_len => array_len }",
            ArrayLenReason::NotLiteralOrPath,
        ),
        (
            "if let array_len = 3 { array_len } else { 0 }",
            ArrayLenReason::NotLiteralOrPath,
        ),
        ("|| 3", ArrayLenReason::NotLiteralOrPath),
        // Literals that are not lengths.
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

/// The diagnostic names the offending sub-expression, not just the array — a
/// caller with several arrays in one struct has to be told which one.
#[test]
fn rejection_names_the_offending_expression() {
    let expr: syn::Expr = syn::parse_quote!(MAX + 1);
    let err = lower_array_len(&expr, "[u8 ; MAX + 1]", &names()).unwrap_err();
    assert_eq!(err.offending, "MAX + 1");
    assert_eq!(err.array, "[u8 ; MAX + 1]");
    let msg = err.to_string();
    assert!(msg.contains("MAX + 1"), "{msg}");
    assert!(msg.contains("hoist"), "{msg}");
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
    let err = resolve_array_lengths(&mut item, &names(), |r, s| {
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
            pub cells: [[u8; MAX]; Holder::N],
        }
    );
    let found = resolve_array_lengths(&mut item, &names(), |r, s| {
        syn::visit_mut::VisitMut::visit_item_struct_mut(r, s)
    })
    .unwrap();
    let rendered = item.to_token_stream().to_string().replace(' ', "");
    assert!(
        rendered.contains("[[u8;myflat::MAX];myflat::Holder::N]"),
        "{rendered}"
    );
    // Inner first, and the outer type is recorded in its REWRITTEN spelling —
    // a consumer keying on it never sees the captured one.
    let keys: Vec<String> = found
        .iter()
        .map(|(ty, _)| ty.to_token_stream().to_string().replace(' ', ""))
        .collect();
    assert_eq!(
        keys,
        vec![
            "[u8;myflat::MAX]".to_string(),
            "[[u8;myflat::MAX];myflat::Holder::N]".to_string(),
        ]
    );
}

/// A length is resolved wherever a type can appear, not only in a struct field.
#[test]
fn lengths_resolve_in_function_signatures() {
    let mut item: syn::ItemFn = syn::parse_quote!(
        pub fn echo(v: [u8; MAX]) -> [u8; Holder::N] {
            unimplemented!()
        }
    );
    resolve_array_lengths(&mut item, &names(), |r, f| {
        syn::visit_mut::VisitMut::visit_item_fn_mut(r, f)
    })
    .unwrap();
    let rendered = item.to_token_stream().to_string().replace(' ', "");
    assert!(rendered.contains("v:[u8;myflat::MAX]"), "{rendered}");
    assert!(rendered.contains("->[u8;myflat::Holder::N]"), "{rendered}");
}

/// An origin-less stream documents `crate` as its module, so the same source
/// spelling resolves to `crate::MAX` there. Both are pinned because deriving the
/// name set from the origin map — which holds only stamped items — silently
/// emitted every length bare for these streams.
#[test]
fn origin_less_streams_resolve_against_the_crate_root() {
    let names = NameIndex::new(
        [("MAX".to_string(), syn::parse_quote!(crate))]
            .into_iter()
            .collect(),
        &[],
    );
    let expr: syn::Expr = syn::parse_quote!(MAX);
    let got = lower_array_len(&expr, "[u8 ; MAX]", &names).unwrap();
    assert_eq!(spelled(&got), "crate :: MAX");
    // `crate::MAX` in the source is the same length, not a different one.
    let expr: syn::Expr = syn::parse_quote!(crate::MAX);
    let got = lower_array_len(&expr, "[u8 ; crate::MAX]", &names).unwrap();
    assert_eq!(spelled(&got), "crate :: MAX");
}
