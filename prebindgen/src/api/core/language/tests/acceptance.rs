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

/// A callback returns nothing, and that is **checked**. `TypeKind::Callback` has
/// no slot for a return, so accepting `impl Fn() -> u8` would drop a fact a
/// destination language needs — silently, which is worse than the refusal.
#[test]
fn a_callback_must_return_nothing() {
    // Written out, `-> ()` is the same callback.
    let TypeKind::Callback { args } =
        kind(quote::quote!(impl Fn(u32) -> () + Send + Sync + 'static))
    else {
        panic!("a callback");
    };
    assert_eq!(args.len(), 1);

    // Anything else is not the accepted `impl Trait` form.
    for spelling in [
        quote::quote!(impl Fn() -> u8 + Send + Sync + 'static),
        quote::quote!(impl Fn(u32) -> Sample + Send + Sync + 'static),
        quote::quote!(impl Fn() -> Option<u8> + Send + Sync + 'static),
    ] {
        assert_eq!(
            reason(spelling),
            UnsupportedTypeReason::DisallowedImplTrait,
            "a returning callback is refused, not silently truncated"
        );
    }
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

/// An extent carries three facts for three questions, and they come apart. No
/// blanket equality could serve all three, which is why the type provides none:
/// a consumer projects the one it needs.
#[test]
fn the_three_extent_projections_are_independent() {
    // `A` and `TAG_LEN` are both 4; `0x04` and `4` are the same literal value
    // spelled differently.
    let elements = parse(vec![
        syn::parse_quote!(
            pub struct Marker {
                pub by_const: [u8; TAG_LEN],
                pub by_other_const: [u8; ALSO_FOUR],
                pub by_literal: [u8; 4],
                pub by_hex_literal: [u8; 0x04],
                pub longer: [u8; 8],
            }
        ),
        tag_len_const(),
        syn::parse_quote!(
            pub const ALSO_FOUR: usize = 4;
        ),
    ]);
    let fields = as_struct(&elements[0]).fields();
    let at = |i: usize| fields[i].ty.array_extent().expect("an extent");
    let (by_const, by_other_const, by_literal, by_hex, longer) =
        (at(0), at(1), at(2), at(3), at(4));

    // Type identity is the evaluated value: all four fours are ONE type and one
    // converter, however they were addressed or spelled.
    for e in [by_const, by_other_const, by_literal, by_hex] {
        assert_eq!(e.value, 4);
    }
    assert_ne!(longer.value, by_literal.value);

    // Declaration spelling is per occurrence, and distinguishes cases the value
    // cannot: `4` is not `0x04`, and neither is `TAG_LEN`.
    assert_eq!(tokens(&by_literal.origin.syntax), "4");
    assert_eq!(tokens(&by_hex.origin.syntax), "0x04");
    assert_eq!(tokens(&by_const.origin.syntax), "TAG_LEN");

    // Header dependency is the named const, and distinguishes cases the
    // spelling groups together and the value does not see at all.
    assert_eq!(
        by_const.const_id().expect("a const dependency").name,
        "TAG_LEN"
    );
    assert_eq!(
        by_other_const.const_id().expect("a const dependency").name,
        "ALSO_FOUR"
    );
    assert!(by_literal.const_id().is_none());
    assert!(by_hex.const_id().is_none());

    // The three really are orthogonal: each pair below agrees on one projection
    // and differs on another.
    assert!(by_const.value == by_literal.value && by_const.const_id() != by_literal.const_id());
    assert!(
        by_literal.value == by_hex.value
            && tokens(&by_literal.origin.syntax) != tokens(&by_hex.origin.syntax)
    );
    assert!(
        by_const.const_id() != by_other_const.const_id() && by_const.value == by_other_const.value
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

/// A variant's index is its declaration order and is never its discriminant:
/// one is where the source *put* it, the other is the value Rust *assigns* it.
/// The two numberings are independent, and this is the pair that proves it.
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
        e.values.iter().map(|v| v.index).collect::<Vec<_>>(),
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

/// The two enum shapes are two entities, and the classification is decided once:
/// any alternative with a field makes it a sum.
///
/// They are not one model with a dead field each. A sum has no discriminant slot,
/// because its alternatives are identified by position — the mirror an adapter
/// builds numbers its own arms — and a fieldless enum's identity is exactly the
/// value Rust assigns.
#[test]
fn the_two_enum_shapes_are_two_entities() {
    let fieldless = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B = 7,
        }
    ));
    let e = as_enum(&fieldless);
    assert_eq!(e.values.len(), 2);
    assert_eq!(e.values[1].discriminant, Some(7));

    // Empty delimiters are still fieldless — the group question, not the syntax
    // one — so this is an enum, and `spell` keeps the delimiters.
    let empty_groups = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B(),
            C {},
        }
    ));
    assert_eq!(as_enum(&empty_groups).values.len(), 3);

    // One field anywhere makes it a sum.
    let sum = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B(u32),
            C { x: u8 },
        }
    ));
    let v = as_variant(&sum);
    assert_eq!(v.alternatives.len(), 3);
    assert!(v.alternatives[0].is_empty(), "a sum may mix");
    assert_eq!(v.alternatives[1].fields.len(), 1);
    assert_eq!(
        v.alternatives.iter().map(|a| a.index).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );

    // No alternatives at all: nothing carries a payload, so it is the degenerate
    // enum rather than an empty sum.
    let empty = parse_one(syn::parse_quote!(
        pub enum E {}
    ));
    assert!(as_enum(&empty).values.is_empty());
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
    let v = as_variant(&element);
    assert!(matches!(
        v.alternatives[0].fields[1].member(),
        syn::Member::Unnamed(i) if i.index == 1
    ));
    assert!(matches!(
        v.alternatives[1].fields[0].member(),
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

/// Function shapes `Function` has no slot for, and would therefore drop in
/// silence.
///
/// `async` is the one that bites: the future would be dropped and the export
/// would be a function whose body never runs.
#[test]
fn function_shapes_outside_the_language() {
    let element = parse_one(syn::parse_quote!(
        pub async fn ping() {}
    ));
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnsupportedAsync
    ));
    // Named, so nothing else can claim the address while it sits inert.
    assert_eq!(element.name().expect("named"), "ping");

    let element = parse_one(syn::parse_quote!(
        pub unsafe extern "C" fn log(fmt: u8, ...) {}
    ));
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnsupportedVariadic
    ));
}

/// A generic binder is refused on every item kind. The elements have no binder,
/// so a `T` would lower as an ordinary nominal reference into the flat namespace
/// — indistinguishable from a real item named `T`.
#[test]
fn a_generic_parameter_is_outside_the_language() {
    let cases: Vec<(syn::Item, &str, &str)> = vec![
        (
            syn::parse_quote!(
                pub struct Wrapper<T> {
                    pub value: T,
                }
            ),
            "T",
            "a type parameter",
        ),
        (
            syn::parse_quote!(
                pub fn first<T>(items: Vec<T>) -> T {
                    unimplemented!()
                }
            ),
            "T",
            "a type parameter",
        ),
        (
            syn::parse_quote!(
                pub enum Either<L, R> {
                    Left(L),
                    Right(R),
                }
            ),
            "L",
            "a type parameter",
        ),
        (
            // Unused, and still a binder.
            syn::parse_quote!(
                pub struct Padded<const N: usize> {
                    pub value: u8,
                }
            ),
            "N",
            "a const generic parameter",
        ),
    ];
    for (item, expected_param, expected_kind) in cases {
        let element = parse_one(item);
        let ItemError::UnsupportedGenericParam { param, kind } = as_unsupported(&element) else {
            panic!(
                "expected a generic-parameter diagnosis, got {}",
                describe(&element)
            );
        };
        assert_eq!(param, expected_param);
        assert_eq!(*kind, expected_kind);
    }
}

/// A **lifetime** binder is not a generic parameter for this purpose: lifetimes
/// say nothing a destination language can act on, and the spelling that needs
/// them is already in the syntax — the same call made for a lifetime argument.
#[test]
fn a_lifetime_binder_is_accepted() {
    let element = parse_one(syn::parse_quote!(
        pub struct Borrowed<'a> {
            pub key: &'a str,
        }
    ));
    let s = as_struct(&element);
    assert_eq!(s.fields().len(), 1);
    assert_eq!(tokens(&s.fields()[0].ty.origin.syntax), "& 'a str");
}

/// `impl Trait` in argument position is an anonymous type parameter in Rust, but
/// `syn` does not desugar it into the binder list — so the callback form, which
/// every callback-taking source function uses, is untouched by the generic
/// refusal. This is the test that says so.
#[test]
fn a_callback_parameter_is_not_a_generic_binder() {
    let element = parse_one(syn::parse_quote!(
        pub fn for_each(f: impl Fn(u64) + Send + Sync + 'static) {}
    ));
    let func = as_fn(&element);
    assert!(matches!(func.params[0].ty.kind, TypeKind::Callback { .. }));
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

/// The feeders accumulate, and the whole-stream rules span them.
///
/// This is why inputs are collected before anything is classified rather than
/// parsed one at a time: a duplicate name is only visible with every input in
/// hand, and so is a const that an array length in another input reaches for.
#[test]
fn the_feeders_accumulate_and_whole_stream_rules_span_them() {
    let marker: syn::Item = syn::parse_quote!(
        pub struct Marker {
            pub tag: [u8; TAG_LEN],
        }
    );

    // A length in the first feeder naming a const from the second.
    let elements = Language::new()
        .items(vec![(marker.clone(), loc())])
        .items(vec![(tag_len_const(), loc())])
        .parse()
        .expect("the const is found across feeders");
    assert_eq!(elements.len(), 2);
    assert_eq!(
        as_struct(&elements[0]).fields()[0]
            .ty
            .array_extent()
            .expect("an extent")
            .value,
        4
    );

    // And a name colliding across feeders is still the one hard failure.
    let err = Language::new()
        .items(vec![(marker.clone(), loc())])
        .items(vec![(marker, loc())])
        .parse()
        .expect_err("a duplicate across feeders is still a duplicate");
    let ParseError::DuplicateName(d) = err;
    assert_eq!(d.name, "Marker");
}

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
