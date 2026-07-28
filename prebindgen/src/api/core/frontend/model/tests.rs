//! The type-lowering matrix.
//!
//! Source spelling → the [`SourceType`] it means, or the precise reason it is
//! refused. A new accepted or refused form is a row.

use quote::ToTokens;

use super::{
    lower_enum, lower_type, ArrayExtent, ConstId, DiscriminantSource, ExtentSource, NamedArg,
    ScalarKind, SourceEnum, SourceType, UnsupportedTypeReason, VariantShape,
};
use crate::api::core::{frontend::ConstIndex, registry::CallbackReject, TypeKey};

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
            UnsupportedTypeReason::DisallowedImplTrait(CallbackReject::NotCallbackShape),
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
        // RESERVED: the language intends these; no wire carries them yet.
        // Previously the return was not read at all, so it was accepted and
        // then silently dropped.
        (
            "impl Fn(u8) -> u16 + Send + Sync + 'static",
            UnsupportedTypeReason::DisallowedImplTrait(CallbackReject::NonUnitReturn),
        ),
        (
            "impl Fn(u8) -> impl Fn(u8) + Send + Sync + 'static + Send + Sync + 'static",
            UnsupportedTypeReason::DisallowedImplTrait(CallbackReject::ReturnsCallback),
        ),
        // RESERVED: the elided `impl Fn(&u8)` IS higher-ranked, so this is the
        // last two-spellings-one-type case rather than an impossibility — it
        // waits on the exact elision rule in #222, not on any wire.
        (
            "impl for<'a> Fn(&'a u8) + Send + Sync + 'static",
            UnsupportedTypeReason::DisallowedImplTrait(CallbackReject::HigherRankedBinder),
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
        // Punctuation, not identity: `normalize_type` collapses both before a
        // key is formed, so the projection agrees with every other position.
        "Wrapper::<u8>",
        "Wrapper<u8,>",
        // Neighbours of those, to pin that the fix did not overshoot.
        "&Foo",
        "&mut Foo",
        "&[u8]",
        "*mut u8",
        "impl Fn(u8) + Send + Sync + 'static",
        // One callback, four spellings. `normalize_type` collapses the
        // punctuation and the bound order, so all four are one key.
        "impl Fn(u8,) + Send + Sync + 'static",
        "impl Fn(u8) -> () + Send + Sync + 'static",
        "impl Fn(u8) + Sync + Send + 'static",
        "impl Fn(u8,) -> () + Sync + Send + 'static",
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

// ── Enums ──────────────────────────────────────────────────────────────

fn lower_e(src: proc_macro2::TokenStream) -> SourceEnum {
    let e: syn::ItemEnum = syn::parse_quote!(#src);
    lower_enum(&e, &consts(), Some("myflat")).expect("lowers")
}

/// The shape question every adapter used to ask `syn::Fields` itself.
#[test]
fn shape_classifies_unit_and_payload() {
    assert!(lower_e(quote::quote! { enum E { A, B = 7, C } }).is_unit());
    // An empty enum has no payload variant, so it is the degenerate unit
    // case — not a sum.
    assert!(lower_e(quote::quote! { enum E {} }).is_unit());

    // One payload variant is enough, whatever the shape of the others.
    for src in [
        quote::quote! { enum E { A(u32), B } },
        quote::quote! { enum E { A, B { x: u32 } } },
        // An empty tuple/brace variant carries no field, so it is a unit
        // group like any other — `Fields::Unnamed` with nothing in it says
        // nothing the model needs.
        quote::quote! { enum E { A, B(u32), C() } },
    ] {
        let e = lower_e(src.clone());
        assert!(!e.is_unit(), "for `{src}`");
        assert!(e.first_payload_variant().is_some(), "for `{src}`");
    }
}

/// The offending variant an adapter names in its rejection message is the
/// **first** payload one, in declaration order.
#[test]
fn first_payload_variant_is_the_first_in_declaration_order() {
    let e = lower_e(quote::quote! { enum E { A, B(u32), C { x: u8 } } });
    assert_eq!(
        e.first_payload_variant().expect("payload variant").name,
        "B"
    );
}

/// Tags are declaration order `0..N-1` and never the discriminant. The two
/// numberings are independent, and this is the pair that proves it.
#[test]
fn tags_are_declaration_order_and_ignore_discriminants() {
    let e = lower_e(quote::quote! { enum E { A(u32), B, C { x: u8 } } });
    assert_eq!(e.name, "E");
    assert_eq!(
        e.variants
            .iter()
            .map(|v| (v.name.to_string(), v.tag))
            .collect::<Vec<_>>(),
        vec![
            ("A".to_string(), 0),
            ("B".to_string(), 1),
            ("C".to_string(), 2)
        ]
    );
    assert!(e.variants[1].is_unit());
    assert!(!e.variants[0].is_unit());

    let e = lower_e(quote::quote! { enum E { A = 5, B = 9 } });
    assert_eq!(
        e.variants.iter().map(|v| v.tag).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        e.discriminant_values()
            .expect("literal discriminants")
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>(),
        vec![5, 9]
    );
}

/// Leaf names follow the nested-prefix convention: `<variant_snake>_<field>`,
/// tuple fields `<variant_snake>_<i>`. Members address the field in a pattern.
#[test]
fn leaf_names_members_and_shape() {
    let e = lower_e(quote::quote! {
        enum RecoveryMode {
            PeriodicQueries(Duration),
            Heartbeat,
            Windowed { size: u32, ratio: f64 },
            Pair(u8, u8),
        }
    });

    let names: Vec<Vec<String>> = e
        .variants
        .iter()
        .map(|v| v.fields.iter().map(|f| f.leaf_name.clone()).collect())
        .collect();
    assert_eq!(
        names,
        vec![
            vec!["periodic_queries_0".to_string()],
            vec![],
            vec!["windowed_size".to_string(), "windowed_ratio".to_string()],
            vec!["pair_0".to_string(), "pair_1".to_string()],
        ]
    );

    assert_eq!(
        e.variants.iter().map(|v| v.shape()).collect::<Vec<_>>(),
        vec![
            VariantShape::Tuple,
            VariantShape::Unit,
            VariantShape::Named,
            VariantShape::Tuple
        ]
    );

    let named = &e.variants[2].fields[0].member;
    assert!(matches!(named, syn::Member::Named(id) if id == "size"));
    let unnamed = &e.variants[3].fields[1].member;
    assert!(matches!(unnamed, syn::Member::Unnamed(i) if i.index == 1));

    // A payload type is LOWERED, not carried verbatim — that is what lets an
    // adapter ask the model instead of re-reading the field's syntax.
    assert!(matches!(
        &e.variants[0].fields[0].ty,
        SourceType::Named { .. }
    ));
    assert!(matches!(
        e.variants[2].fields[0].ty,
        SourceType::Scalar(ScalarKind::U32)
    ));
}

/// A unit-only enum is the degenerate sum: every group is empty, so a lowering
/// written for the general case collapses to "just a tag".
#[test]
fn unit_enum_is_all_empty_groups() {
    let e = lower_e(quote::quote! { enum E { A, B, C } });
    assert_eq!(e.variants.len(), 3);
    assert!(e.variants.iter().all(|v| v.is_unit()));

    let e = lower_e(quote::quote! { enum E { Only(String) } });
    assert_eq!(e.variants.len(), 1);
    assert_eq!(e.variants[0].tag, 0);
    assert_eq!(e.variants[0].fields[0].leaf_name, "only_0");
}

/// A payload the type grammar refuses is refused HERE, naming the variant and
/// field, rather than reaching an adapter.
#[test]
fn a_refused_payload_type_names_its_variant_and_field() {
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A, B { bad: (u8, u8) } } };
    let (variant, field, err) = lower_enum(&e, &consts(), Some("myflat")).expect_err("refused");
    assert_eq!(variant, "B");
    assert_eq!(field.expect("named field"), "bad");
    assert!(matches!(
        err.reason,
        UnsupportedTypeReason::UnsupportedTuple
    ));

    // A tuple variant has no field name to report, only a position.
    let e: syn::ItemEnum = syn::parse_quote! { enum E { A((u8, u8)) } };
    let (variant, field, _) = lower_enum(&e, &consts(), Some("myflat")).expect_err("refused");
    assert_eq!(variant, "A");
    assert!(field.is_none());
}

// ── Discriminants ──────────────────────────────────────────────────────

fn discriminants(src: proc_macro2::TokenStream) -> Vec<(String, i64)> {
    lower_e(src)
        .discriminant_values()
        .expect("literal discriminants")
        .into_iter()
        .map(|(ident, value)| (ident.to_string(), value))
        .collect()
}

#[test]
fn discriminants_follow_rusts_own_rule() {
    // Implicit C-like enum: 0, 1, 2.
    assert_eq!(
        discriminants(quote::quote! { enum E { A, B, C } }),
        vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)]
    );
    assert_eq!(
        discriminants(quote::quote! { enum E { A = 1, B = 2, C = 7 } }),
        vec![("A".into(), 1), ("B".into(), 2), ("C".into(), 7)]
    );
    // Explicit sets the value; the next implicit variant is prev + 1.
    assert_eq!(
        discriminants(quote::quote! { enum E { A = 5, B, C = 1, D } }),
        vec![
            ("A".into(), 5),
            ("B".into(), 6),
            ("C".into(), 1),
            ("D".into(), 2),
        ]
    );
    assert_eq!(
        discriminants(quote::quote! { enum E { A = -1, B } }),
        vec![("A".into(), -1), ("B".into(), 0)]
    );
}

/// An unevaluable discriminant is **not** a lowering failure: a C mirror
/// re-emits the spelling and never needs the number. It surfaces only when a
/// consumer asks for values, and then it names the offender.
#[test]
fn an_unevaluable_discriminant_is_carried_not_refused() {
    let e = lower_e(quote::quote! { enum E { A = OTHER, B } });
    assert!(matches!(
        e.variants[0].discriminant.source,
        DiscriminantSource::Explicit(_)
    ));
    assert!(e.variants[0].discriminant.value.is_none());
    // The chain is broken from there: B has no computable value either.
    assert!(e.variants[1].discriminant.value.is_none());
    assert_eq!(e.discriminant_values().expect_err("no values"), "A");
}

/// The spelling survives exactly, which is what lets a C header keep `0x07`
/// rather than re-rendering it as `7`.
#[test]
fn an_explicit_discriminant_keeps_its_spelling() {
    let e = lower_e(quote::quote! { enum E { A = 0x07 } });
    let DiscriminantSource::Explicit(expr) = &e.variants[0].discriminant.source else {
        panic!("explicit");
    };
    assert_eq!(expr.to_token_stream().to_string(), "0x07");
    assert_eq!(e.variants[0].discriminant.value, Some(7));
}
