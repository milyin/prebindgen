//! The type-lowering matrix.
//!
//! Source spelling → the [`SourceType`] it means, or the precise reason it is
//! refused. A new accepted or refused form is a row.

use quote::ToTokens;

use super::{
    lower_type, ArrayExtent, ConstId, ExtentSource, ScalarKind, SourceType, UnsupportedTypeReason,
};
use crate::api::core::frontend::ConstIndex;

fn consts() -> ConstIndex {
    ConstIndex::new([
        (
            "N".to_string(),
            syn::parse_quote!(4),
            Some("myflat".to_string()),
        ),
        (
            "M".to_string(),
            syn::parse_quote!(2),
            Some("myflat".to_string()),
        ),
    ])
}

fn lower(src: &str) -> Result<SourceType, UnsupportedTypeReason> {
    let ty: syn::Type = syn::parse_str(src).expect("test input must parse");
    lower_type(&ty, &consts(), Some("myflat")).map_err(|e| e.reason)
}

fn named(src: &str) -> SourceType {
    let ty: syn::Type = syn::parse_str(src).expect("parse");
    let syn::Type::Path(tp) = ty else {
        panic!("not a path")
    };
    SourceType::Named {
        name: tp.path,
        args: Vec::new(),
    }
}

/// ACCEPTED forms, and the model each means.
///
/// The model *is* the classification: `is_scalar`, `is_string`, `is_vec`,
/// `is_option` and `box_inner` are these variants, not questions an adapter
/// re-asks `syn`. That is what stages F5/F6 will delete.
#[test]
fn accepted_types() {
    let cases: &[(&str, SourceType)] = &[
        ("u8", SourceType::Scalar(ScalarKind::U8)),
        ("bool", SourceType::Scalar(ScalarKind::Bool)),
        ("f64", SourceType::Scalar(ScalarKind::F64)),
        ("usize", SourceType::Scalar(ScalarKind::Usize)),
        ("String", SourceType::Str),
        ("()", SourceType::Unit),
        (
            "Option<u8>",
            SourceType::Optional(Box::new(SourceType::Scalar(ScalarKind::U8))),
        ),
        (
            "Vec<Vec<u8>>",
            SourceType::Sequence(Box::new(SourceType::Sequence(Box::new(
                SourceType::Scalar(ScalarKind::U8),
            )))),
        ),
        ("Box<String>", SourceType::Boxed(Box::new(SourceType::Str))),
        (
            "Result<u8, String>",
            SourceType::Fallible {
                ok: Box::new(SourceType::Scalar(ScalarKind::U8)),
                err: Box::new(SourceType::Str),
            },
        ),
        // A user type, and a user type with arguments — the fallback, which is
        // where an indexed struct/enum and a foreign path both land.
        ("Foo", named("Foo")),
        (
            "Wrapper<u8>",
            SourceType::Named {
                name: syn::parse_quote!(Wrapper),
                args: vec![SourceType::Scalar(ScalarKind::U8)],
            },
        ),
        // A lifetime is part of the spelling, not modeled structure.
        ("KeyExpr<'static>", named("KeyExpr")),
        (
            "&u8",
            SourceType::Ref {
                mutable: false,
                inner: Box::new(SourceType::Scalar(ScalarKind::U8)),
            },
        ),
        (
            "&mut Foo",
            SourceType::Ref {
                mutable: true,
                inner: Box::new(named("Foo")),
            },
        ),
        (
            "&[u8]",
            SourceType::Ref {
                mutable: false,
                inner: Box::new(SourceType::Slice(Box::new(SourceType::Scalar(
                    ScalarKind::U8,
                )))),
            },
        ),
        (
            "*const u8",
            SourceType::Ptr {
                mutable: false,
                inner: Box::new(SourceType::Scalar(ScalarKind::U8)),
            },
        ),
        (
            "(u8, String)",
            SourceType::Tuple(vec![SourceType::Scalar(ScalarKind::U8), SourceType::Str]),
        ),
        (
            "impl Fn(u8) + Send + Sync + 'static",
            SourceType::Callback {
                args: vec![SourceType::Scalar(ScalarKind::U8)],
            },
        ),
    ];
    for (src, expect) in cases {
        let got = lower(src).unwrap_or_else(|r| panic!("`{src}` was refused: {r:?}"));
        assert_eq!(&got, expect, "`{src}` lowered to the wrong model");
    }
}

/// An array carries BOTH halves of its extent: the value, and which const —
/// if any — the source named. That pairing is the whole reason this model
/// exists rather than a bare `usize`.
#[test]
fn array_extents_carry_value_and_spelling() {
    let literal = lower("[u8; 4]").unwrap();
    assert_eq!(
        literal,
        SourceType::Array {
            elem: Box::new(SourceType::Scalar(ScalarKind::U8)),
            extent: ArrayExtent {
                value: 4,
                source: ExtentSource::Literal,
            },
        }
    );

    let by_const = lower("[u8; N]").unwrap();
    assert_eq!(
        by_const,
        SourceType::Array {
            elem: Box::new(SourceType::Scalar(ScalarKind::U8)),
            extent: ArrayExtent {
                value: 4,
                source: ExtentSource::Const(ConstId {
                    name: "N".to_string(),
                    origin: Some("myflat".to_string()),
                }),
            },
        }
    );

    // Same semantic type, distinguishable spelling — the pair a type-keyed
    // table cannot hold and a use-site model can.
    assert_eq!(
        literal.to_syn().to_token_stream().to_string(),
        by_const.to_syn().to_token_stream().to_string()
    );
    assert_ne!(literal, by_const);
}

/// Nested extents are collected outermost-first, which is what tells an emitter
/// which consts a spelled type will name.
#[test]
fn nested_extents_are_collected() {
    let ty = lower("[[u8; N]; M]").unwrap();
    let names: Vec<Option<&str>> = ty
        .extents()
        .iter()
        .map(|e| e.const_id().map(|c| c.name.as_str()))
        .collect();
    assert_eq!(names, vec![Some("M"), Some("N")]);
    // And through a wrapper, since a field may be `Option<[u8; N]>`.
    let wrapped = lower("Option<[u8; N]>").unwrap();
    assert_eq!(wrapped.extents().len(), 1);
    assert_eq!(wrapped.extents()[0].value, 4);
}

/// REFUSED forms. Acceptance is a consequence of lowering, so a form absent
/// from the accepted table is refused by construction.
#[test]
fn refused_types() {
    let cases: &[(&str, UnsupportedTypeReason)] = &[
        (
            "impl Iterator<Item = u8>",
            UnsupportedTypeReason::DisallowedImplTrait,
        ),
        ("dyn Fn(u8)", UnsupportedTypeReason::UnsupportedForm),
        ("fn(u8) -> u8", UnsupportedTypeReason::UnsupportedForm),
        ("_", UnsupportedTypeReason::UnsupportedForm),
        ("!", UnsupportedTypeReason::UnsupportedForm),
        (
            "Option<u8, u8>",
            UnsupportedTypeReason::WrongGenericArity { expected: 1 },
        ),
        (
            "Result<u8>",
            UnsupportedTypeReason::WrongGenericArity { expected: 2 },
        ),
    ];
    for (src, reason) in cases {
        match lower(src) {
            Ok(v) => panic!("`{src}` was accepted as {v:?}"),
            Err(got) => assert_eq!(&got, reason, "`{src}` refused for the wrong reason"),
        }
    }
    // A bad extent is reported as such, not flattened into "unsupported form" —
    // the length subgrammar's own diagnostic survives being nested in a type.
    let err = lower("[u8; MISSING]").unwrap_err();
    assert!(
        matches!(err, UnsupportedTypeReason::BadArrayExtent(_)),
        "{err:?}"
    );
}

/// `to_syn` round-trips: the projection of a lowered type re-lowers to itself.
///
/// This is what makes the `syn` field types stored on an item a projection
/// rather than a second source of truth — they cannot say something the model
/// does not.
#[test]
fn to_syn_round_trips() {
    for src in [
        "u8",
        "String",
        "()",
        "Option<Vec<u8>>",
        "Box<Foo>",
        "Result<u8, String>",
        "Wrapper<u8>",
        "&mut Foo",
        "&[u8]",
        "*mut u8",
        "(u8, String)",
        "[u8; 4]",
        "[[u8; N]; M]",
        "impl Fn(u8) + Send + Sync + 'static",
    ] {
        let once = lower(src).unwrap_or_else(|r| panic!("`{src}` refused: {r:?}"));
        let projected = once.to_syn();
        let twice = lower_type(&projected, &consts(), Some("myflat")).unwrap_or_else(|e| {
            panic!(
                "`{src}` projection `{}` refused: {e}",
                projected.to_token_stream()
            )
        });
        // The extent's SPELLING is deliberately not in the projection, so
        // compare the semantic form.
        assert_eq!(
            twice.to_syn().to_token_stream().to_string(),
            projected.to_token_stream().to_string(),
            "`{src}` did not round-trip"
        );
    }
}
