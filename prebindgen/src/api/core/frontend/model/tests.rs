//! The type-lowering matrix.
//!
//! Source spelling → the [`SourceType`] it means, or the precise reason it is
//! refused. A new accepted or refused form is a row.

use quote::ToTokens;

use super::{
    lower_type, ArrayExtent, ConstId, ExtentSource, NamedArg, ScalarKind, SourceType,
    UnsupportedTypeReason,
};
use crate::api::core::{frontend::ConstIndex, TypeKey};

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
        path: tp,
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
                path: syn::parse_quote!(Wrapper),
                args: vec![NamedArg::Type(SourceType::Scalar(ScalarKind::U8))],
            },
        ),
        // A lifetime is kept VERBATIM as an argument: part of the spelling, not
        // modeled structure — and `Foo<'static>` is not `Foo`.
        (
            "KeyExpr<'static>",
            SourceType::Named {
                path: syn::parse_quote!(KeyExpr),
                args: vec![NamedArg::Lifetime(syn::parse_quote!('static))],
            },
        ),
        (
            "&u8",
            SourceType::Ref {
                lifetime: None,
                mutable: false,
                inner: Box::new(SourceType::Scalar(ScalarKind::U8)),
            },
        ),
        (
            "&mut Foo",
            SourceType::Ref {
                lifetime: None,
                mutable: true,
                inner: Box::new(named("Foo")),
            },
        ),
        (
            "&[u8]",
            SourceType::Ref {
                lifetime: None,
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
        // Only `()` is in the language. No adapter has ever lowered a tuple —
        // every `Type::Tuple` site in both of them is the unit case or a
        // generic walk — so accepting one only deferred the failure to a late
        // "unresolved type".
        ("(u8, String)", UnsupportedTypeReason::UnsupportedTuple),
        ("(Foo,)", UnsupportedTypeReason::UnsupportedTuple),
        // An associated type: `#[prebindgen]` never captures `impl` blocks, so
        // what this resolves to is unknowable here.
        ("<T as Trait>::Assoc", UnsupportedTypeReason::AssociatedType),
        ("<Holder>::N", UnsupportedTypeReason::AssociatedType),
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

/// **The projection preserves type IDENTITY.**
///
/// For every accepted form, the `TypeKey` of the normalized original equals the
/// `TypeKey` of its projection — so writing `field.ty = ty.to_syn()` in pass 3
/// cannot change what a type *is*.
///
/// This compares against the **normalized original**, which is what makes it
/// detect loss. Its predecessor projected an already-projected type and so only
/// proved a lossy function idempotent; it passed while `&'static Foo` became
/// `&Foo`, `Foo<'static>` became `Foo`, and `foreign::Option<u8>` became the
/// prelude `Option<u8>`.
///
/// Arrays are the one deliberate exception — the extent becomes its number —
/// so they are asserted against the expected numeric form instead.
#[test]
fn the_projection_preserves_type_identity() {
    let identity = [
        "u8",
        "bool",
        "String",
        "()",
        "Option<Vec<u8>>",
        "Box<Foo>",
        "Result<u8, String>",
        "Foo",
        "Wrapper<u8>",
        // The five the review found. Each was silently rewritten before.
        "& 'static Foo",
        "&'a mut Foo",
        "Foo<'static>",
        "Foo<'a, u8>",
        "foreign::Option<u8>",
        "a::b::Foo<u8>",
        "::root::Foo",
        // Neighbours of those, to pin that the fix did not overshoot.
        "&Foo",
        "&mut Foo",
        "&[u8]",
        "*mut u8",
        "impl Fn(u8) + Send + Sync + 'static",
    ];
    for src in identity {
        let ty: syn::Type = syn::parse_str(src).unwrap_or_else(|e| panic!("`{src}`: {e}"));
        let lowered = lower(src).unwrap_or_else(|r| panic!("`{src}` was refused: {r:?}"));
        assert_eq!(
            TypeKey::from_type(&ty),
            TypeKey::from_type(&lowered.to_syn()),
            "`{src}` lost identity: projected as `{}`",
            lowered.to_syn().to_token_stream()
        );
    }

    // An extent is deliberately projected as its value, and nothing else moves.
    let arr = lower("[u8; N]").unwrap();
    assert_eq!(
        TypeKey::from_type(&arr.to_syn()),
        TypeKey::from_type(&syn::parse_quote!([u8; 4]))
    );
    let nested = lower("Option<[[u8; N]; M]>").unwrap();
    assert_eq!(
        TypeKey::from_type(&nested.to_syn()),
        TypeKey::from_type(&syn::parse_quote!(Option<[[u8; 4]; 2]>))
    );
}

/// A builtin must be spelled BARE. `foreign::Option` merely shares a name with
/// the prelude type and is a different type; collapsing it would silently
/// retype the field and pick the wrong converter.
///
/// `normalize_type` is what makes this safe to demand: it reduces the real std
/// paths to bare form at ingest and deliberately leaves unknown crate paths
/// alone.
#[test]
fn a_builtin_must_be_spelled_bare() {
    assert!(matches!(lower("Option<u8>"), Ok(SourceType::Optional(_))));
    assert!(matches!(lower("Vec<u8>"), Ok(SourceType::Sequence(_))));
    assert!(matches!(lower("String"), Ok(SourceType::Str)));

    for foreign in ["foreign::Option<u8>", "foreign::Vec<u8>", "foreign::String"] {
        assert!(
            matches!(lower(foreign), Ok(SourceType::Named { .. })),
            "`{foreign}` must stay a foreign named type"
        );
    }
    // A user type ending in a scalar's name is not that scalar either.
    assert!(matches!(lower("mycrate::u8"), Ok(SourceType::Named { .. })));
}
