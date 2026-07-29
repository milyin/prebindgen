//! The acceptance matrix: source spelling → element, or a diagnosis naming the
//! item and the component that could not be expressed.
//!
//! Ported from #212/#226 and extended to functions, consts and item kinds.

use super::*;

/// Lower one type by putting it in a struct field, and report what the language
/// made of it. The field path is used because a field is the position every
/// consumer already agrees is a boundary surface.
fn lower(ty: proc_macro2::TokenStream) -> Result<Type, UnsupportedType> {
    let item: syn::Item = syn::parse_quote!(
        pub struct S {
            pub f: #ty,
        }
    );
    match parse(vec![tag_len_const(), item]).remove(1) {
        Element::Struct(s) => Ok(s.fields()[0].ty.clone()),
        Element::Unsupported(u) => match *u.error {
            ItemError::FieldType { source, .. } => Err(source),
            other => panic!("expected a field-type diagnosis, got {other}"),
        },
        other => panic!("expected a struct, got {}", describe(&other)),
    }
}

fn kind(ty: proc_macro2::TokenStream) -> TypeKind {
    lower(ty).expect("in the language").kind
}

fn reason(ty: proc_macro2::TokenStream) -> UnsupportedTypeReason {
    lower(ty).expect_err("outside the language").reason
}

// ── Types ──────────────────────────────────────────────────────────────

#[test]
fn scalars_and_strings() {
    assert!(matches!(
        kind(quote::quote!(u8)),
        TypeKind::Scalar(ScalarKind::U8)
    ));
    assert!(matches!(
        kind(quote::quote!(bool)),
        TypeKind::Scalar(ScalarKind::Bool)
    ));
    assert!(matches!(
        kind(quote::quote!(f64)),
        TypeKind::Scalar(ScalarKind::F64)
    ));
    assert!(matches!(kind(quote::quote!(String)), TypeKind::Str));
    assert!(matches!(kind(quote::quote!(())), TypeKind::Unit));
}

/// `String` and `str` are one concept, and the borrow is the `Ref` layer's
/// fact. Every adapter already treats `&str` as a borrowed string by hand;
/// classifying `str` as a nominal type would send them all looking for an item
/// named `str` to resolve.
#[test]
fn a_string_is_a_string_however_it_is_spelled() {
    assert!(matches!(kind(quote::quote!(str)), TypeKind::Str));
    for spelling in [quote::quote!(&str), quote::quote!(&String)] {
        let TypeKind::Ref { mutable, inner } = kind(spelling) else {
            panic!("a borrow");
        };
        assert!(!mutable);
        assert!(matches!(inner.kind, TypeKind::Str));
    }
}

#[test]
fn the_builtin_generics() {
    assert!(matches!(
        kind(quote::quote!(Option<u8>)),
        TypeKind::Optional(_)
    ));
    assert!(matches!(
        kind(quote::quote!(Vec<u8>)),
        TypeKind::Sequence(_)
    ));
    assert!(matches!(
        kind(quote::quote!(Result<u8, Error>)),
        TypeKind::Fallible { .. }
    ));
}

/// `Box<T>` **is** `T` — an owned value either way, and no destination language
/// can tell them apart, so it carries no kind of its own. The `Box` survives
/// where it matters: in the syntax generated Rust spells.
#[test]
fn a_box_classifies_as_what_it_wraps() {
    let ty = lower(quote::quote!(Box<String>)).expect("in the language");
    assert!(matches!(ty.kind, TypeKind::Str));
    assert_eq!(tokens(&ty.origin.syntax), "Box < String >");

    // And it composes: the nullable heap string of a `#[repr(C)]` struct field
    // is an optional string, spelled with its `Box`.
    let ty = lower(quote::quote!(Option<Box<String>>)).expect("in the language");
    let TypeKind::Optional(inner) = &ty.kind else {
        panic!("an option");
    };
    assert!(matches!(inner.kind, TypeKind::Str));
    assert_eq!(tokens(&inner.origin.syntax), "Box < String >");
}

/// A builtin must be spelled BARE **after normalization**: the real std path
/// reduces and classifies, while a path-qualified lookalike is a foreign type
/// that merely shares the name, and collapsing it would silently retype the
/// field.
#[test]
fn a_qualified_builtin_is_a_named_type() {
    assert!(matches!(
        kind(quote::quote!(std::option::Option<u8>)),
        TypeKind::Optional(_)
    ));
    let TypeKind::Named { id, .. } = kind(quote::quote!(foreign::Option<u8>)) else {
        panic!("a named type");
    };
    assert_eq!(id.name, "foreign::Option");
}

#[test]
fn references() {
    assert!(matches!(
        kind(quote::quote!(&Sample)),
        TypeKind::Ref { mutable: false, .. }
    ));
    assert!(matches!(
        kind(quote::quote!(&mut Sample)),
        TypeKind::Ref { mutable: true, .. }
    ));
}

/// `Vec<T>` and `[T]` are one concept — a run of `T` — and ownership is the
/// `Ref` layer's fact, not a second variant. That is already how the pipeline
/// behaves: one `Shape::Iterable` covers both, and jnigen rewrites a `&[T]`
/// input into the `Vec<_>` pattern outright.
#[test]
fn a_sequence_is_a_sequence_borrowed_or_owned() {
    assert!(matches!(
        kind(quote::quote!(Vec<u8>)),
        TypeKind::Sequence(_)
    ));
    // Bare, as a callback argument is written: `impl Fn([T])`.
    assert!(matches!(kind(quote::quote!([u8])), TypeKind::Sequence(_)));
    let TypeKind::Ref { inner, .. } = kind(quote::quote!(&[u8])) else {
        panic!("a reference");
    };
    assert!(matches!(inner.kind, TypeKind::Sequence(_)));
}

/// A raw pointer is not in the language. A `#[prebindgen]` crate is idiomatic
/// Rust and the adapter owns the lowering to pointers — no adapter has a
/// selection arm for one, so accepting it would only defer the failure to a
/// late "unresolved type".
#[test]
fn a_raw_pointer_is_not_in_the_language() {
    assert_eq!(
        reason(quote::quote!(*const u8)),
        UnsupportedTypeReason::UnsupportedForm
    );
    assert_eq!(
        reason(quote::quote!(*mut Sample)),
        UnsupportedTypeReason::UnsupportedForm
    );
}

/// A lifetime argument is accepted and not modelled — `Foo<'a, T>` classifies
/// as `Foo` with one type argument, and the spelling keeps the rest.
#[test]
fn a_lifetime_argument_is_spelling_only() {
    let ty = lower(quote::quote!(Foo<'a, u8>)).expect("in the language");
    let TypeKind::Named { id, args } = &ty.kind else {
        panic!("a named type");
    };
    assert_eq!(id.name, "Foo");
    assert_eq!(args.len(), 1);
    assert_eq!(tokens(&ty.origin.syntax), "Foo < 'a , u8 >");
}

#[test]
fn the_callback_form() {
    let TypeKind::Callback { args } =
        kind(quote::quote!(impl Fn(&Sample, u32) + Send + Sync + 'static))
    else {
        panic!("a callback");
    };
    assert_eq!(args.len(), 2);
}

#[test]
fn types_outside_the_language() {
    assert_eq!(
        reason(quote::quote!((u8, u8))),
        UnsupportedTypeReason::UnsupportedTuple
    );
    assert_eq!(
        reason(quote::quote!(<Holder as Trait>::Assoc)),
        UnsupportedTypeReason::AssociatedType
    );
    assert_eq!(
        reason(quote::quote!(Option<u8, u16>)),
        UnsupportedTypeReason::WrongGenericArity { expected: 1 }
    );
    assert_eq!(
        reason(quote::quote!(Result<u8>)),
        UnsupportedTypeReason::WrongGenericArity { expected: 2 }
    );
    assert_eq!(
        reason(quote::quote!(impl Iterator<Item = u8>)),
        UnsupportedTypeReason::DisallowedImplTrait
    );
    assert_eq!(
        reason(quote::quote!(dyn Fn(u8))),
        UnsupportedTypeReason::UnsupportedForm
    );
    assert_eq!(
        reason(quote::quote!(!)),
        UnsupportedTypeReason::UnsupportedForm
    );
}

// ── Array extents ──────────────────────────────────────────────────────

fn extent_reason(ty: proc_macro2::TokenStream) -> ArrayLenReason {
    match reason(ty) {
        UnsupportedTypeReason::BadArrayExtent(e) => e.reason,
        other => panic!("expected an extent diagnosis, got {other:?}"),
    }
}

#[test]
fn extents_outside_the_subgrammar() {
    assert_eq!(
        extent_reason(quote::quote!([u8; TAG_LEN + 1])),
        ArrayLenReason::NotLiteralOrName
    );
    assert_eq!(
        extent_reason(quote::quote!([u8; crate::limits::MAX])),
        ArrayLenReason::NotABareName
    );
    assert_eq!(
        extent_reason(quote::quote!([u8; UNMARKED])),
        ArrayLenReason::NotAMarkedConst
    );
    assert_eq!(
        extent_reason(quote::quote!([u8; 'c'])),
        ArrayLenReason::NotAnIntegerLiteral
    );
}

/// A const may be declared after the item that uses it: the const index is
/// built before anything is lowered.
#[test]
fn an_extent_may_name_a_const_declared_later() {
    let elements = parse(vec![
        syn::parse_quote!(
            pub struct Marker {
                pub tag: [u8; TAG_LEN],
            }
        ),
        tag_len_const(),
    ]);
    assert_eq!(
        as_struct(&elements[0]).fields()[0]
            .ty
            .array_extent()
            .expect("an extent")
            .value,
        4
    );
}

/// A const whose own initializer is not a literal cannot be a length —
/// `build.rs` cannot evaluate it — but it is still a perfectly good const.
#[test]
fn a_computed_const_is_indexed_but_is_not_a_length() {
    let elements = parse(vec![
        syn::parse_quote!(
            pub const COMPUTED: usize = 2 * 2;
        ),
        syn::parse_quote!(
            pub struct Marker {
                pub tag: [u8; COMPUTED],
            }
        ),
    ]);
    assert_eq!(as_const(&elements[0]).name, "COMPUTED");
    match as_unsupported(&elements[1]) {
        ItemError::FieldType {
            source:
                UnsupportedType {
                    reason: UnsupportedTypeReason::BadArrayExtent(e),
                    ..
                },
            ..
        } => assert_eq!(e.reason, ArrayLenReason::ConstIsNotALiteral),
        other => panic!("expected an extent diagnosis, got {other}"),
    }
}

// ── Item kinds ─────────────────────────────────────────────────────────

/// A struct is a product of fields, or opaque. A tuple struct is the opaque
/// one: usable as a handle, its fields deliberately not lowered, because no
/// adapter has ever crossed them and lowering would turn types that are ignored
/// today into errors. A unit struct is the empty product, not a third shape —
/// the delimiters are spelling, and `spell` reads them off the syntax.
#[test]
fn struct_shapes() {
    let named = parse_one(syn::parse_quote!(
        pub struct A {
            pub x: u8,
        }
    ));
    assert_eq!(as_struct(&named).fields().len(), 1);

    let tuple = parse_one(syn::parse_quote!(
        pub struct B(SomethingUnexpressible<'_, dyn Trait>);
    ));
    assert!(as_struct(&tuple).fields.is_none(), "opaque");
    assert!(as_struct(&tuple).fields().is_empty());

    let unit = parse_one(syn::parse_quote!(
        pub struct C;
    ));
    assert!(as_struct(&unit).fields.is_some(), "empty, not opaque");
    assert!(as_struct(&unit).fields().is_empty());
}

/// Tags are declaration order and are never the discriminant. The two
/// numberings are independent, and this is the pair that proves it.
#[test]
fn tags_are_declaration_order() {
    let element = parse_one(syn::parse_quote!(
        pub enum E {
            A = 5,
            B = 9,
        }
    ));
    let e = as_enum(&element);
    assert_eq!(
        e.variants.iter().map(|v| v.tag).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        e.discriminant_values()
            .expect("literals")
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<_>>(),
        vec![5, 9]
    );
}

/// A fieldless enum is the degenerate sum, and `is_unit` is the question the
/// declarators ask.
#[test]
fn unit_and_payload_enums() {
    let unit = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B = 7,
        }
    ));
    assert!(as_enum(&unit).is_unit());
    assert!(as_enum(&unit).first_payload_variant().is_none());

    let sum = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B(u32),
            C { x: u8 },
        }
    ));
    assert!(!as_enum(&sum).is_unit());
    assert_eq!(
        as_enum(&sum)
            .first_payload_variant()
            .expect("a payload")
            .name,
        "B"
    );
}

/// A field is addressed by name or by position, and the model says which
/// without anyone reading `syn::Fields`.
#[test]
fn field_members_follow_the_addressing() {
    let element = parse_one(syn::parse_quote!(
        pub enum Reading {
            Exact(i64, i64),
            Range { low: i64 },
        }
    ));
    let e = as_enum(&element);
    assert!(matches!(
        e.variants[0].fields[1].member(),
        syn::Member::Unnamed(i) if i.index == 1
    ));
    assert!(matches!(
        e.variants[1].fields[0].member(),
        syn::Member::Named(id) if id == "low"
    ));
}

#[test]
fn consts_carry_their_type_and_value() {
    let element = parse_one(tag_len_const());
    let c = as_const(&element);
    assert_eq!(c.name, "TAG_LEN");
    assert!(matches!(c.ty.kind, TypeKind::Scalar(ScalarKind::Usize)));
    assert_eq!(tokens(&c.origin.syntax.expr), "4");
}

/// An unnamed `const _` — each source's injected feature guard — is a const
/// like any other, and simply has no address, so several sources may each carry
/// one without colliding in the flat namespace.
#[test]
fn an_unnamed_const_is_a_const_without_an_address() {
    let elements = parse(vec![
        syn::parse_quote!(
            const _: () = ();
        ),
        syn::parse_quote!(
            const _: () = ();
        ),
    ]);
    assert!(elements.iter().all(|e| matches!(e, Element::Const(_))));
    assert!(elements.iter().all(|e| e.name().is_none()));
}

/// An item kind the language does not model is diagnosed, not carried: a
/// `#[prebindgen]` crate marks what crosses the boundary and leaves the code
/// around it to the consumer. The proc-macro refuses to mark a `use` at all, so
/// only a `union` or a type alias can reach here — and both keep their name, so
/// nothing else can claim it.
#[test]
fn an_unmodelled_item_kind_is_diagnosed() {
    for (item, expected) in [
        (
            syn::parse_quote!(
                pub union U {
                    a: u8,
                }
            ),
            "a union",
        ),
        (
            syn::parse_quote!(
                pub type Alias = u32;
            ),
            "a type alias",
        ),
    ] {
        let element = parse_one(item);
        assert!(element.name().is_some(), "keeps its address");
        assert!(matches!(
            as_unsupported(&element),
            ItemError::UnsupportedItemKind { kind } if *kind == expected
        ));
    }
}

// ── Functions ──────────────────────────────────────────────────────────

#[test]
fn function_signatures() {
    let element = parse_one(syn::parse_quote!(
        pub fn put(key: &KeyExpr, payload: Vec<u8>) -> Result<(), Error> {
            unimplemented!()
        }
    ));
    let f = as_fn(&element);
    assert_eq!(f.name, "put");
    assert_eq!(
        f.params
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>(),
        vec!["key", "payload"]
    );
    assert!(matches!(f.ret.kind, TypeKind::Fallible { .. }));
}

/// An elided return and a written `-> ()` are the same function. Nothing in the
/// pipeline distinguishes them — every consumer normalizes one to the other on
/// the spot — so the model does it once instead.
#[test]
fn an_elided_return_is_the_unit() {
    for sig in [
        quote::quote!(
            pub fn f() {}
        ),
        quote::quote!(
            pub fn f() -> () {}
        ),
    ] {
        let element = parse_one(syn::parse_quote!(#sig));
        assert!(matches!(as_fn(&element).ret.kind, TypeKind::Unit));
    }
}

#[test]
fn a_receiver_is_not_a_free_function() {
    let element = parse_one(syn::parse_quote!(
        pub fn get(self) -> u8 {
            unimplemented!()
        }
    ));
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnsupportedReceiver
    ));
}

#[test]
fn a_parameter_must_be_bound_to_one_name() {
    let element = parse_one(syn::parse_quote!(
        pub fn f((a, b): (u8, u8)) {}
    ));
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnsupportedParamPattern { .. }
    ));
}

/// The diagnosis names the component, not just the item — the whole point of
/// lowering each part separately.
#[test]
fn a_diagnosis_names_the_component() {
    let element = parse_one(syn::parse_quote!(
        pub fn f(ok: u8, bad: (u8, u8)) {}
    ));
    match as_unsupported(&element) {
        ItemError::ParamType { param, source } => {
            assert_eq!(param, "bad");
            assert_eq!(source.reason, UnsupportedTypeReason::UnsupportedTuple);
        }
        other => panic!("expected a parameter diagnosis, got {other}"),
    }

    let element = parse_one(syn::parse_quote!(
        pub fn f() -> (u8, u8) {
            unimplemented!()
        }
    ));
    assert!(matches!(
        as_unsupported(&element),
        ItemError::ReturnType { .. }
    ));

    let element = parse_one(syn::parse_quote!(
        pub enum E {
            V { bad: (u8, u8) },
        }
    ));
    match as_unsupported(&element) {
        ItemError::VariantFieldType { variant, field, .. } => {
            assert_eq!(variant, "V");
            assert_eq!(field, "bad");
        }
        other => panic!("expected a variant diagnosis, got {other}"),
    }
}

/// An item the language cannot express is inert, not fatal: a source crate may
/// mark items no binding uses, and those have never had to be expressible. It
/// keeps its name (so nothing else can claim it) and its syntax.
#[test]
fn an_unsupported_item_is_indexed_not_refused() {
    let elements = parse(vec![
        syn::parse_quote!(
            pub fn unusable(pair: (u8, u8)) {}
        ),
        syn::parse_quote!(
            pub fn usable(x: u8) {}
        ),
    ]);
    assert_eq!(elements[0].name().expect("named"), "unusable");
    assert!(matches!(elements[0], Element::Unsupported(_)));
    assert!(matches!(elements[1], Element::Function(_)));
}

// ── The flat namespace ─────────────────────────────────────────────────

/// Two marked items with one name are ambiguous however the crates are
/// arranged, so this is the one thing a parse refuses outright.
#[test]
fn duplicate_names_are_a_hard_error() {
    let err = try_parse(vec![
        syn::parse_quote!(
            pub struct Sample {
                pub x: u8,
            }
        ),
        syn::parse_quote!(
            pub fn Sample() {}
        ),
    ])
    .expect_err("a duplicate");
    let ParseError::DuplicateName(d) = err;
    assert_eq!(d.name, "Sample");
}

/// Even an item the language could not express holds its name against the
/// namespace: it is still a marked item, and a second one would still be
/// ambiguous.
#[test]
fn an_unsupported_item_still_holds_its_name() {
    assert!(try_parse(vec![
        syn::parse_quote!(
            pub fn Thing(pair: (u8, u8)) {}
        ),
        syn::parse_quote!(
            pub struct Thing {
                pub x: u8,
            }
        ),
    ])
    .is_err());
}
