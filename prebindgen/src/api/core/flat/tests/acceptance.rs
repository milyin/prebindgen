//! The acceptance matrix: source spelling → element, or a diagnosis naming the
//! item and the component that could not be expressed.
//!
//! Ported from #212/#226 and extended to functions, consts and item kinds.

use super::*;

/// Lower one type by putting it in a struct field, and report what the language
/// made of it. The field path is used because a field is the position every
/// consumer already agrees is a boundary surface.
fn lower(ty: proc_macro2::TokenStream) -> Result<TypeRef, UnsupportedType> {
    let item: syn::Item = syn::parse_quote!(
        pub struct S {
            pub f: #ty,
        }
    );
    // The fixture types stand in for a declared type wherever the grammar needs
    // a nominal one, so references resolve and the test is about the grammar.
    let mut items = fixture_types();
    items.push(tag_len_const());
    items.push(opaque("Sample"));
    let n = items.len();
    items.push(item);
    match parse(items).remove(n) {
        Element::Type(Type::Struct(s)) => Ok(s.fields[0].ty.clone()),
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
        let TypeKind::Ref { mode, inner } = kind(spelling) else {
            panic!("a borrow");
        };
        assert_eq!(mode, RefMode::Shared);
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
/// The prelude: every name the language pre-declares reaches the same kind however
/// it is spelled. This is the drift guard — adding a builtin arm to `lower_path`
/// and forgetting its prelude entry fails here, which is exactly how `MaybeUninit`
/// slipped through and worked only when the source happened to `use` it.
#[test]
fn the_prelude_reaches_every_builtin_by_either_spelling() {
    use crate::api::core::types_util::Normalization;

    // Each entry, bare against fully qualified. `MaybeUninit` needs a `&mut` to
    // mean anything, so it is checked separately below.
    for (path, name) in Normalization::PRELUDE {
        if *name == "MaybeUninit" {
            continue;
        }
        let bare: proc_macro2::TokenStream = match *name {
            "Result" => quote::quote!(Result<u8, Error>),
            "String" => quote::quote!(String),
            // `Cow` needs a lifetime and an unsized target to be valid Rust.
            "Cow" => quote::quote!(Cow<'_, [u8]>),
            _ => {
                let n = quote::format_ident!("{name}");
                quote::quote!(#n<u8>)
            }
        };
        let qualified: proc_macro2::TokenStream = {
            let p: syn::Path = syn::parse_str(path).expect("a prelude path");
            match *name {
                "Result" => quote::quote!(#p<u8, Error>),
                "String" => quote::quote!(#p),
                "Cow" => quote::quote!(#p<'_, [u8]>),
                _ => quote::quote!(#p<u8>),
            }
        };
        assert_eq!(
            format!("{:?}", kind(bare)),
            format!("{:?}", kind(qualified)),
            "`{name}` must classify the same as `{path}`"
        );
    }

    // `core` and `alloc` are re-exports of the same items, so either root works.
    assert!(matches!(
        kind(quote::quote!(core::option::Option<u8>)),
        TypeKind::Optional(_)
    ));
    assert!(matches!(
        kind(quote::quote!(alloc::string::String)),
        TypeKind::Str
    ));

    // The bug: qualified `MaybeUninit` used to fall through to an unresolvable
    // nominal type, so an out-parameter worked only if the source `use`d it.
    let TypeKind::Ref { mode, .. } = kind(quote::quote!(&mut std::mem::MaybeUninit<Sample>)) else {
        panic!("a borrow");
    };
    assert_eq!(mode, RefMode::Out);
}

/// A `#[prebindgen] pub type` is a **one-way road**: it brings a foreign type into
/// the flat API under a name, and that name is thereafter the only way to spell it.
///
/// It is a *declaration*, not an equivalence. Treating it as a reduction rule broke
/// the contract normalization actually has — choose among spellings of one type,
/// never change what a type is — because `type Bytes = Vec<u8>` would make `Vec<u8>`
/// an extern. So a qualified spelling stays refused even when an alias names exactly
/// that type.
#[test]
fn an_alias_is_a_declaration_not_an_equivalence() {
    let items: Vec<syn::Item> = vec![
        syn::parse_quote!(
            pub type Session = zenoh::Session;
        ),
        syn::parse_quote!(
            pub fn by_name(s: &Session) {}
        ),
        syn::parse_quote!(
            pub fn by_path(s: &zenoh::Session) {}
        ),
    ];
    let flat = Flat::builder()
        .items(items.into_iter().map(|i| (i, loc())))
        .build()
        .expect("a refusal is deferred, not fatal");

    // The declared name works.
    let f = flat.function("by_name").expect("declared");
    let TypeKind::Ref { inner, .. } = &f.params[0].ty.kind else {
        panic!("a borrow");
    };
    let TypeKind::Named { id } = &inner.kind else {
        panic!("a nominal type");
    };
    assert_eq!(id.name, "Session");

    // The path it aliases does NOT, and the diagnosis says to use the name.
    assert!(flat.function("by_path").is_none());
    let u = flat.unsupported().next().expect("one refusal");
    assert!(matches!(
        &*u.error,
        ItemError::UnresolvedType { name } if name == "zenoh::Session"
    ));
    assert!(
        u.error.to_string().contains("refer to that"),
        "the diagnosis must point at the declared name: {}",
        u.error
    );

    // And the declaration itself still records what it points at.
    let Type::Extern(e) = flat.declared_type("Session").expect("declared") else {
        panic!("an extern");
    };
    assert_eq!(e.target.as_deref(), Some("zenoh :: Session"));
}

/// An alias cannot retype anything, whatever its target's arguments — the property
/// that made key-shape bugs possible in the first place, now unreachable because an
/// alias is not an equivalence at all.
///
/// Covers the reported cases: a concrete generic target (`Vec<u8>`, which shadowed
/// the prelude), and a const-generic pair (`Wrap<4>` / `Wrap<8>`, which collided
/// because a key kept only type arguments).
#[test]
fn an_alias_never_retypes_a_spelling() {
    let flat = Flat::builder()
        .items(
            vec![
                syn::parse_quote!(
                    pub type Bytes = std::vec::Vec<u8>;
                ),
                syn::parse_quote!(
                    pub type Small = zenoh::Wrap<4>;
                ),
                syn::parse_quote!(
                    pub type Big = zenoh::Wrap<8>;
                ),
                syn::parse_quote!(
                    pub fn strings(xs: std::vec::Vec<String>) {}
                ),
                syn::parse_quote!(
                    pub fn bytes(xs: std::vec::Vec<u8>) {}
                ),
                syn::parse_quote!(
                    pub fn by_name(b: Bytes) {}
                ),
                syn::parse_quote!(
                    pub fn small(w: Small) {}
                ),
                syn::parse_quote!(
                    pub fn big(w: Big) {}
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("parses");

    let param = |name: &str| {
        flat.function(name)
            .unwrap_or_else(|| panic!("{name} survives"))
            .params[0]
            .ty
            .kind
            .clone()
    };

    // A prelude type keeps its grammar meaning at every instantiation, whatever an
    // unrelated alias happens to target.
    for f in ["strings", "bytes"] {
        assert!(
            matches!(param(f), TypeKind::Sequence(_)),
            "`{f}`: the grammar's spelling stays canonical"
        );
    }

    // Each alias is usable by its own name, and they cannot collide: a bare path is
    // never reduced, so the name IS the identity.
    for (f, expected) in [("by_name", "Bytes"), ("small", "Small"), ("big", "Big")] {
        let TypeKind::Named { id } = param(f) else {
            panic!("{f}: a nominal type");
        };
        assert_eq!(id.name, expected, "{f}");
        assert!(matches!(
            flat.declared_type(expected).expect(expected),
            Type::Extern(_)
        ));
    }
}

#[test]
fn a_qualified_builtin_is_a_named_type() {
    assert!(matches!(
        kind(quote::quote!(std::option::Option<u8>)),
        TypeKind::Optional(_)
    ));

    // Not an `Option` — and, being path-qualified, it cannot name a flat-API item
    // either, so it is refused as unresolved rather than silently retyped.
    let element = {
        let mut items = fixture_types();
        let n = items.len();
        items.push(syn::parse_quote!(
            pub struct S {
                pub f: foreign::Option<u8>,
            }
        ));
        parse(items).remove(n)
    };
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnresolvedType { name } if name == "foreign::Option"
    ));
}

#[test]
fn references() {
    assert!(matches!(
        kind(quote::quote!(&Sample)),
        TypeKind::Ref {
            mode: RefMode::Shared,
            ..
        }
    ));
    assert!(matches!(
        kind(quote::quote!(&mut Sample)),
        TypeKind::Ref {
            mode: RefMode::Exclusive,
            ..
        }
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

/// `Cow<'_, T>` **is** `T`, the same treatment `Box<T>` gets: borrowed or owned, and
/// no destination language can tell.
///
/// Both adapters already behave that way — cbindgen lowers `Cow<'_, [T]>` "just like
/// `Vec<T>` outputs", and jnigen's converter is `byte_array_from_slice(&v)`, which
/// works by deref and is identical to the `Vec<u8>` one — so this classification
/// predicts their behaviour rather than leaving it a special case.
#[test]
fn a_cow_is_what_it_borrows() {
    // The property the whole treatment rests on: indistinguishable from the owned
    // spelling of the same thing.
    assert_eq!(
        format!("{:?}", kind(quote::quote!(Cow<'_, [u8]>))),
        format!("{:?}", kind(quote::quote!(Vec<u8>))),
        "a byte Cow classifies exactly as a byte Vec"
    );
    assert!(matches!(kind(quote::quote!(Cow<'_, str>)), TypeKind::Str));

    // The `Cow` survives where codegen reads it: a generated signature must spell
    // `Cow<'_, [u8]>`, which is not interchangeable with `Vec<u8>` in Rust.
    let ty = lower(quote::quote!(Cow<'_, [u8]>)).expect("in the language");
    assert_eq!(tokens(&ty.origin.syntax), "Cow < '_ , [u8] >");

    // Transparent for any target, as `Box` is: whether it can actually cross is the
    // adapter's call, and both already restrict which elements they accept.
    let TypeKind::Sequence(elem) = kind(quote::quote!(Cow<'_, [Sample]>)) else {
        panic!("a sequence");
    };
    assert!(matches!(elem.kind, TypeKind::Named { .. }));

    // A lifetime argument is expected on `Cow` alone. On any other builtin it is
    // still not a shape the language has, so the exception is exactly one name wide:
    // `Vec<'a, u8>` is a nominal `Vec` nobody declared, and the item is refused.
    let element = {
        let mut items = fixture_types();
        let n = items.len();
        items.push(syn::parse_quote!(
            pub struct S {
                pub f: Vec<'a, u8>,
            }
        ));
        parse(items).remove(n)
    };
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnresolvedType { name } if name == "Vec"
    ));
}

/// The signature that motivated this: zenoh-flat's `zbytes_to_bytes`. It was refused
/// under the closed API because a lifetime argument sent `Cow` to an undeclared
/// nominal type.
#[test]
fn a_cow_returning_accessor_resolves() {
    let flat = Flat::builder()
        .items(
            vec![
                syn::parse_quote!(
                    pub type ZBytes = zenoh::bytes::ZBytes;
                ),
                syn::parse_quote!(
                    pub fn zbytes_to_bytes(z: &ZBytes) -> Cow<'_, [u8]> {}
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("parses");

    assert_eq!(flat.unsupported().count(), 0, "no longer refused");
    let f = flat.function("zbytes_to_bytes").expect("survives");
    assert!(matches!(f.ret.kind, TypeKind::Sequence(_)));
    // And the return still spells its `Cow`, so an adapter can emit the signature.
    assert_eq!(tokens(&f.ret.origin.syntax), "Cow < '_ , [u8] >");
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

/// Generic arguments are accepted and not modelled — a reference is a *name*, and
/// the spelling keeps the rest.
///
/// Nothing could read retained arguments: a surviving reference resolves to a
/// declared type, and no declaration takes type parameters. They are still lowered,
/// so a bad type inside one is diagnosed.
#[test]
fn generic_arguments_are_spelling_only() {
    let ty = lower(quote::quote!(Foo<'a, u8>)).expect("in the language");
    let TypeKind::Named { id } = &ty.kind else {
        panic!("a named type");
    };
    assert_eq!(id.name, "Foo");
    assert_eq!(tokens(&ty.origin.syntax), "Foo < 'a , u8 >");

    // Lowered, so still checked: a tuple inside a generic argument is refused.
    assert_eq!(
        reason(quote::quote!(Foo<(u8, u8)>)),
        UnsupportedTypeReason::UnsupportedTuple
    );
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
    // Expression forms that BIND a local, which is the dangerous family: a length
    // is qualified against its source module, so a local shadowing a marked item
    // would be rewritten into it. Scope tracking is the general answer; none of
    // these has a place in a boundary type, so the whole family is refused.
    // (Moved here from the jnigen suite: the subgrammar is the frontend's.)
    assert_eq!(
        extent_reason(quote::quote!(
            [u8; const {
                let n = 3;
                n
            }]
        )),
        ArrayLenReason::NotLiteralOrName
    );
    assert_eq!(
        extent_reason(quote::quote!(
            [u8; match 3 {
                n => n,
            }]
        )),
        ArrayLenReason::NotLiteralOrName
    );
    assert_eq!(
        extent_reason(quote::quote!([u8; if let n = 3 { n } else { 0 }])),
        ArrayLenReason::NotLiteralOrName
    );
    // A CALL is not a name either, however const the callee.
    assert_eq!(
        extent_reason(quote::quote!([u8; array_len()])),
        ArrayLenReason::NotLiteralOrName
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
        as_struct(&elements[0]).fields[0]
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
    let fields = &as_struct(&elements[0]).fields;
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
/// today into errors. A unit struct is the empty product, not a handle — the
/// delimiters are spelling, and `spell` reads them off the syntax.
#[test]
fn struct_shapes() {
    let named = parse_one(syn::parse_quote!(
        pub struct A {
            pub x: u8,
        }
    ));
    assert_eq!(as_struct(&named).fields.len(), 1);

    // A tuple struct is a handle, so its fields are never lowered — which is why
    // a field type outside the grammar is not an error here.
    let tuple = parse_one(syn::parse_quote!(
        pub struct B(SomethingUnexpressible<'_, dyn Trait>);
    ));
    assert_eq!(as_extern(&tuple).name, "B");

    let unit = parse_one(syn::parse_quote!(
        pub struct C;
    ));
    assert!(
        as_struct(&unit).fields.is_empty(),
        "an empty product, not a handle"
    );
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
    assert!(elements.iter().all(|e| matches!(e, Element::Constant(_))));
    assert!(elements.iter().all(|e| e.name().is_none()));
}

/// An item kind the language does not model is diagnosed, not carried: a
/// `#[prebindgen]` crate marks what crosses the boundary and leaves the code
/// around it to the consumer. The proc-macro refuses to mark a `use` at all, and
/// a type alias now *declares* an opaque handle — so a `union` is the only kind
/// left that reaches here.
#[test]
fn an_unmodelled_item_kind_is_diagnosed() {
    let element = parse_one(syn::parse_quote!(
        pub union U {
            a: u8,
        }
    ));
    assert!(element.name().is_some(), "keeps its address");
    assert!(matches!(
        as_unsupported(&element),
        ItemError::UnsupportedItemKind { kind } if *kind == "a union"
    ));
}

/// A marked type alias DECLARES an opaque handle — the way a foreign or
/// crate-private type gets a name in the flat API. That is what lets references
/// be required to resolve, so it is the keystone of the whole model.
#[test]
fn a_marked_alias_declares_an_extern() {
    let element = parse_one(syn::parse_quote!(
        pub type Session = zenoh::Session;
    ));
    let e = as_extern(&element);
    assert_eq!(e.name, "Session");
    // What it points at is a modelled fact, so an adapter can recognise a target
    // without taking the syntax apart. Not classified: a std type may hide behind a
    // foreign alias, as `Error = zenoh::Error` does.
    assert_eq!(e.target.as_deref(), Some("zenoh :: Session"));
    // The whole item survives, so a consumer can still read what it aliased.
    assert_eq!(
        tokens(&e.origin.syntax),
        "pub type Session = zenoh :: Session ;"
    );

    // A std target is recorded the same way — nothing here decides it is special.
    let element = parse_one(syn::parse_quote!(
        pub type Duration = std::time::Duration;
    ));
    assert_eq!(
        as_extern(&element).target.as_deref(),
        Some("std :: time :: Duration")
    );

    // A tuple struct points at nothing: it IS the definition.
    let element = parse_one(syn::parse_quote!(
        pub struct Handle(Whatever);
    ));
    let e = as_extern(&element);
    assert_eq!(e.name, "Handle");
    assert_eq!(e.target, None);

    // And it satisfies a reference, which is the point.
    let mut items = fixture_types();
    items.push(syn::parse_quote!(
        pub type Session = zenoh::Session;
    ));
    let n = items.len();
    items.push(syn::parse_quote!(
        pub fn session_close(s: Session) {}
    ));
    let elements = parse(items);
    assert!(matches!(elements[n], Element::Function(_)));
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
    assert_eq!(s.fields.len(), 1);
    assert_eq!(tokens(&s.fields[0].ty.origin.syntax), "& 'a str");
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

// ── Resolution and access ──────────────────────────────────────────────

/// The model answers by name, which is what every later stage needs. An
/// unsupported item is still reachable — it holds its slot in the namespace —
/// but it is not a type.
#[test]
fn the_model_is_addressed_by_name() {
    let flat = Flat::builder()
        .items(
            vec![
                syn::parse_quote!(
                    pub type Session = zenoh::Session;
                ),
                syn::parse_quote!(
                    pub const LIMIT: usize = 4;
                ),
                syn::parse_quote!(
                    pub fn session_close(s: Session) {}
                ),
                syn::parse_quote!(
                    pub union U {
                        a: u8,
                    }
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("parses");

    assert!(flat.function("session_close").is_some());
    assert!(flat.declared_type("Session").is_some());
    assert!(flat.constant("LIMIT").is_some());
    assert_eq!(flat.functions().count(), 1);
    assert_eq!(flat.types().count(), 1);
    assert_eq!(flat.constants().count(), 1);

    // Reachable, but not a type — it holds its name so nothing else can claim it.
    assert!(flat.element("U").is_some());
    assert!(flat.declared_type("U").is_none());
    assert_eq!(flat.unsupported().count(), 1);

    // A name nobody declared is simply absent.
    assert!(flat.element("nope").is_none());
}

/// A reference leads to the declaration it names. Resolving here is the point of
/// #211: a dangling name used to surface much later, as an unresolved converter
/// from whichever adapter looked first.
#[test]
fn a_reference_resolves_to_its_declaration() {
    let flat = Flat::builder()
        .items(
            vec![
                syn::parse_quote!(
                    pub type Session = zenoh::Session;
                ),
                syn::parse_quote!(
                    pub fn session_close(s: Session) {}
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("parses");

    let f = flat.function("session_close").expect("declared");
    let TypeKind::Named { id, .. } = &f.params[0].ty.kind else {
        panic!("a nominal type");
    };
    let target = flat.resolve(id).expect("resolves");
    assert!(matches!(target, Type::Extern(_)));
    assert_eq!(target.name(), "Session");
}

/// Resolution runs once every declaration is in hand, so a reference may point
/// forward, or into another source entirely — which is how a helper crate names
/// types it cannot mark itself.
#[test]
fn resolution_spans_feeders_and_declaration_order() {
    // Forward reference within one feeder.
    let flat = Flat::builder()
        .items(
            vec![
                syn::parse_quote!(
                    pub fn session_close(s: Session) {}
                ),
                syn::parse_quote!(
                    pub type Session = zenoh::Session;
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("a forward reference resolves");
    assert!(flat.function("session_close").is_some());

    // And across feeders: the declaration arrives in the second stream.
    let flat = Flat::builder()
        .items(vec![(
            syn::parse_quote!(
                pub fn session_close(s: Session) {}
            ),
            loc(),
        )])
        .items(vec![(
            syn::parse_quote!(
                pub type Session = zenoh::Session;
            ),
            loc(),
        )])
        .build()
        .expect("a cross-feeder reference resolves");
    assert!(flat.function("session_close").is_some());
}

/// A name the flat API does not declare makes the *referencing* item
/// unsupported, inert until an adapter declares it — the same deferral every
/// other refusal uses, so an item no binding touches stays harmless.
#[test]
fn an_undeclared_reference_refuses_the_referencing_item() {
    let flat = Flat::builder()
        .items(vec![(
            syn::parse_quote!(
                pub fn session_close(s: Session) {}
            ),
            loc(),
        )])
        .build()
        .expect("a refusal is deferred, not fatal");

    assert!(flat.function("session_close").is_none());
    let u = flat.unsupported().next().expect("one refusal");
    assert!(matches!(
        &*u.error,
        ItemError::UnresolvedType { name } if name == "Session"
    ));
    // It still holds its name, so nothing else can claim it.
    assert_eq!(u.name.as_ref().expect("named"), "session_close");

    // Reachable through every layer a reference can nest in.
    for ty in [
        quote::quote!(Option<Session>),
        quote::quote!(Vec<Session>),
        quote::quote!(&Session),
        quote::quote!(Result<Session, Error>),
        quote::quote!(impl Fn(Session) + Send + Sync + 'static),
        quote::quote!([Session; 4]),
        // NOT `Wrapper<Session>`: a declared type takes no type parameters, so a
        // source writing that would not compile. Generic arguments are lowered and
        // discarded — `generic_arguments_are_spelling_only` covers that they are
        // still checked.
    ] {
        let flat = Flat::builder()
            .items(vec![
                (opaque("Error"), loc()),
                (
                    syn::parse_quote!(
                        pub fn f(s: #ty) {}
                    ),
                    loc(),
                ),
            ])
            .build()
            .expect("deferred");
        assert_eq!(
            flat.unsupported().count(),
            1,
            "`Session` must be found inside {}",
            ty
        );
    }
}

/// An out-parameter is a **mode of borrowing**, not a type. `&mut MaybeUninit<T>`
/// says the caller supplies the slot and the callee fills it; the `MaybeUninit` is
/// absorbed into the mode, so `inner` is the value's own type.
///
/// Uninitialized storage anywhere else promises nothing a destination language can
/// use, so the combinations that mean nothing cannot be written down.
#[test]
fn an_out_parameter_is_a_borrow_mode() {
    let TypeKind::Ref { mode, inner } = kind(quote::quote!(&mut MaybeUninit<Sample>)) else {
        panic!("a borrow");
    };
    assert_eq!(mode, RefMode::Out);
    // The `MaybeUninit` is gone from the type: it described the borrow.
    let TypeKind::Named { id, .. } = &inner.kind else {
        panic!("the value's own type");
    };
    assert_eq!(id.name, "Sample");

    // The three modes are one axis.
    assert_eq!(
        [
            quote::quote!(&Sample),
            quote::quote!(&mut Sample),
            quote::quote!(&mut MaybeUninit<Sample>),
        ]
        .map(|t| match kind(t) {
            TypeKind::Ref { mode, .. } => mode,
            other => panic!("a borrow, got {other:?}"),
        }),
        [RefMode::Shared, RefMode::Exclusive, RefMode::Out]
    );

    // Owned, or shared-borrowed, it means nothing.
    assert_eq!(
        reason(quote::quote!(MaybeUninit<Sample>)),
        UnsupportedTypeReason::OwnedUninit
    );
    assert_eq!(
        reason(quote::quote!(&MaybeUninit<Sample>)),
        UnsupportedTypeReason::SharedUninit
    );

    // And it still needs no declaration, unlike every other generic-bearing name.
    let flat = Flat::builder()
        .items(vec![
            (opaque("Sample"), loc()),
            (
                syn::parse_quote!(
                    pub fn get(out: &mut MaybeUninit<Sample>) -> bool {}
                ),
                loc(),
            ),
        ])
        .build()
        .expect("parses");
    assert!(flat.function("get").is_some());
}

/// Refusing a type removes a *declaration*, so its dependents must be refused
/// too — otherwise a surviving element would hold a reference that resolves to
/// nothing, which is the one invariant the model promises.
///
/// Checked in both declaration orders, because a single pass against a snapshot
/// of the initial declarations keeps the dependent whichever way round it is.
#[test]
fn refusal_is_transitive() {
    let broken: syn::Item = syn::parse_quote!(
        pub struct Broken {
            pub field: Missing,
        }
    );
    let user: syn::Item = syn::parse_quote!(
        pub fn use_broken(value: Broken) {}
    );

    for (label, items) in [
        ("declaration first", vec![broken.clone(), user.clone()]),
        ("dependent first", vec![user, broken]),
    ] {
        let flat = Flat::builder()
            .items(items.into_iter().map(|i| (i, loc())))
            .build()
            .expect("deferred, not fatal");

        assert!(
            flat.declared_type("Broken").is_none(),
            "{label}: `Missing` is undeclared"
        );
        assert!(
            flat.function("use_broken").is_none(),
            "{label}: `Broken` is no longer a declaration either"
        );
        assert_eq!(flat.unsupported().count(), 2, "{label}");
        // Both still hold their names against the namespace.
        assert!(flat.element("Broken").is_some(), "{label}");
        assert!(flat.element("use_broken").is_some(), "{label}");
    }
}

/// And through a chain of any length, in either direction — the fixed point only
/// ever shrinks the declared set, so it terminates and misses no hop.
#[test]
fn refusal_is_transitive_through_a_chain() {
    let chain: Vec<syn::Item> = vec![
        syn::parse_quote!(
            pub struct A {
                pub field: Missing,
            }
        ),
        syn::parse_quote!(
            pub struct B {
                pub field: A,
            }
        ),
        syn::parse_quote!(
            pub struct C {
                pub field: B,
            }
        ),
        syn::parse_quote!(
            pub fn takes_c(value: C) {}
        ),
    ];

    for (label, items) in [
        ("forward", chain.clone()),
        ("reversed", chain.into_iter().rev().collect()),
    ] {
        let flat = Flat::builder()
            .items(items.into_iter().map(|i| (i, loc())))
            .build()
            .expect("deferred");
        assert_eq!(
            flat.types().count(),
            0,
            "{label}: the whole chain collapses"
        );
        assert_eq!(flat.functions().count(), 0, "{label}");
        assert_eq!(flat.unsupported().count(), 4, "{label}");
    }

    // A sound chain is untouched, so the fixed point is not just refusing
    // everything reachable.
    let flat = Flat::builder()
        .items(
            vec![
                opaque("Missing"),
                syn::parse_quote!(
                    pub struct A {
                        pub field: Missing,
                    }
                ),
                syn::parse_quote!(
                    pub fn takes_a(value: A) {}
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("parses");
    assert_eq!(flat.unsupported().count(), 0);
    assert!(flat.function("takes_a").is_some());
}

/// Every reference reachable from a *surviving* element resolves. That is what
/// the transitive pass buys, and what `resolve` relies on.
#[test]
fn every_surviving_reference_resolves() {
    let flat = Flat::builder()
        .items(
            vec![
                opaque("Missing"),
                syn::parse_quote!(
                    pub struct Held {
                        pub field: Missing,
                    }
                ),
                syn::parse_quote!(
                    pub fn takes_held(value: Held) -> Held {}
                ),
                // ... alongside a chain that does collapse.
                syn::parse_quote!(
                    pub struct Broken {
                        pub field: Absent,
                    }
                ),
                syn::parse_quote!(
                    pub fn takes_broken(value: Broken) {}
                ),
            ]
            .into_iter()
            .map(|i: syn::Item| (i, loc())),
        )
        .build()
        .expect("deferred");

    for f in flat.functions() {
        for r in f.params.iter().map(|p| &p.ty).chain([&f.ret]) {
            if let TypeKind::Named { id, .. } = &r.kind {
                assert!(
                    flat.resolve(id).is_some(),
                    "`{}` must resolve from a surviving function",
                    id.name
                );
            }
        }
    }
    assert!(flat.function("takes_held").is_some());
    assert!(flat.function("takes_broken").is_none());
}

/// A generic alias is a generic binder like any other item's, and `Extern` has no
/// binder or arity — so accepting one would let `Handle<u8>` resolve against a
/// declaration that says nothing about its parameter. It is also why
/// `MaybeUninit` needed grammar support rather than an alias.
#[test]
fn a_generic_alias_is_refused() {
    for (item, param, kind_str) in [
        (
            syn::parse_quote!(
                pub type Handle<T> = hidden::Handle<T>;
            ),
            "T",
            "a type parameter",
        ),
        (
            syn::parse_quote!(
                pub type Padded<const N: usize> = hidden::Padded<N>;
            ),
            "N",
            "a const generic parameter",
        ),
    ] {
        let element = parse_one(item);
        let ItemError::UnsupportedGenericParam { param: got, kind } = as_unsupported(&element)
        else {
            panic!(
                "expected a generic-parameter diagnosis, got {}",
                describe(&element)
            );
        };
        assert_eq!(got, param);
        assert_eq!(*kind, kind_str);
    }

    // A lifetime binder stays accepted, as it is on every other item kind:
    // lifetimes are spelling and the spelling already travels.
    let element = parse_one(syn::parse_quote!(
        pub type Borrowed<'a> = hidden::Borrowed<'a>;
    ));
    assert_eq!(as_extern(&element).name, "Borrowed");
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
    let flat = Flat::builder()
        .items(vec![(marker.clone(), loc())])
        .items(vec![(tag_len_const(), loc())])
        .build()
        .expect("the const is found across feeders");
    let elements: Vec<Element> = flat.elements().cloned().collect();
    assert_eq!(elements.len(), 2);
    assert_eq!(
        as_struct(&elements[0]).fields[0]
            .ty
            .array_extent()
            .expect("an extent")
            .value,
        4
    );

    // And a name colliding across feeders is still the one hard failure.
    let err = Flat::builder()
        .items(vec![(marker.clone(), loc())])
        .items(vec![(marker, loc())])
        .build()
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
