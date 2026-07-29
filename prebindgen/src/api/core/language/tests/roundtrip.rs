//! The round-trip property: an element's syntax slices are the source's own
//! tokens, sliced — never a reconstruction.
//!
//! Every test here would also pass against a model that rebuilt syntax from its
//! classification *for the easy cases*. The ones that matter are the cases where
//! a reconstruction loses: an empty tuple variant, a hex discriminant, a
//! lifetime, an aliased path, a doc comment. Those are the reason the slices
//! ride along at all.

use super::*;

/// Each parameter and the return type re-emit exactly what was written —
/// including a lifetime, which the classification does not model.
#[test]
fn function_parts_are_the_source_tokens() {
    let f = syn::parse_quote!(
        pub fn publish(
            key: &'a KeyExpr,
            payload: Vec<u8>,
            count: Option<i32>,
        ) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let element = parse_one(syn::Item::Fn(f));
    let func = as_fn(&element);

    assert_eq!(
        func.params
            .iter()
            .map(|p| tokens(&p.origin.syntax))
            .collect::<Vec<_>>(),
        vec![
            "key : & 'a KeyExpr",
            "payload : Vec < u8 >",
            "count : Option < i32 >",
        ]
    );
    // The lifetime is nowhere in the classification, and still survives.
    assert_eq!(tokens(&func.params[0].ty.origin.syntax), "& 'a KeyExpr");
    assert!(matches!(func.params[0].ty.kind, TypeKind::Ref { .. }));

    assert_eq!(tokens(&func.ret.origin.syntax), "Result < () , Error >");
}

/// A defaulted return and a written `-> ()` are the same function, and both
/// spell as `()`. The one thing that separates them — whether the source typed
/// an arrow — is in `Function::syntax.sig.output`, where the only consumer that
/// could care (one re-emitting the signature verbatim) already looks.
#[test]
fn a_defaulted_return_spells_as_the_unit() {
    for item in [
        syn::parse_quote!(
            pub fn a() {}
        ),
        syn::parse_quote!(
            pub fn b() -> () {}
        ),
    ] {
        let element = parse_one(item);
        let ret = &as_fn(&element).ret;
        assert!(matches!(ret.kind, TypeKind::Unit));
        assert_eq!(tokens(&ret.origin.syntax), "()");
    }

    let defaulted = parse_one(syn::parse_quote!(
        pub fn a() {}
    ));
    assert!(matches!(
        as_fn(&defaulted).origin.syntax.sig.output,
        syn::ReturnType::Default
    ));
}

/// A field's slice keeps its attributes and visibility, so an emitter can
/// re-state the field rather than rebuild it from name and type.
#[test]
fn struct_field_slices_keep_attributes() {
    let element = parse_one(syn::parse_quote!(
        pub struct Sample {
            /// The key it was published on.
            pub key: String,
            #[allow(dead_code)]
            pub(crate) seq: u64,
        }
    ));
    let fields = as_struct(&element).fields();
    assert_eq!(fields.len(), 2);
    assert!(tokens(&fields[0].origin.syntax).contains("The key it was published on."));
    assert_eq!(
        tokens(&fields[1].origin.syntax),
        "# [allow (dead_code)] pub (crate) seq : u64"
    );
}

/// One captured record is one item, so an item and every node lowered out of it
/// point at the **same** location — not equal copies, the same allocation.
///
/// That is the model, not an optimisation: a field has no location of its own,
/// and the honest answer to "where is this field" is "wherever its item is".
#[test]
fn an_item_and_its_components_share_one_location() {
    let element = parse_one(syn::parse_quote!(
        pub struct Sample {
            pub key: String,
            pub tags: Vec<u8>,
        }
    ));
    let s = as_struct(&element);
    let item = &s.origin.location;
    for field in s.fields() {
        assert!(Rc::ptr_eq(item, &field.origin.location), "field");
        assert!(Rc::ptr_eq(item, &field.ty.origin.location), "field type");
    }
    // And down through a nested type's arguments.
    let TypeKind::Sequence(elem) = &s.fields()[1].ty.kind else {
        panic!("a sequence");
    };
    assert!(Rc::ptr_eq(item, &elem.origin.location), "element type");

    // Variants, their fields, parameters and extents alike.
    let element = parse_one(syn::parse_quote!(
        pub enum E {
            A { x: [u8; 4] },
        }
    ));
    let e = as_enum(&element);
    let item = &e.origin.location;
    let v = &e.variants[0];
    assert!(Rc::ptr_eq(item, &v.origin.location), "variant");
    let f = &v.fields[0];
    assert!(Rc::ptr_eq(item, &f.origin.location), "variant field");
    let extent = f.ty.array_extent().expect("an extent");
    assert!(Rc::ptr_eq(item, &extent.origin.location), "extent");
    assert_eq!(tokens(&extent.origin.syntax), "4");

    let element = parse_one(syn::parse_quote!(
        pub fn f(a: u8) {}
    ));
    let func = as_fn(&element);
    let item = &func.origin.location;
    assert!(Rc::ptr_eq(item, &func.params[0].origin.location), "param");
    assert!(Rc::ptr_eq(item, &func.ret.origin.location), "elided return");
}

/// A component's diagnosis carries the item's location, which is the only one
/// there is — the record is per-item, so nothing finer was ever captured.
#[test]
fn a_component_diagnosis_carries_the_items_location() {
    let element = parse_one(syn::parse_quote!(
        pub struct Sample {
            pub bad: (u8, u8),
        }
    ));
    let Element::Unsupported(u) = &element else {
        panic!("a tuple field is outside the language");
    };
    assert!(matches!(*u.error, ItemError::FieldType { .. }));
    // The item's own location, reachable the same way as for any other element.
    assert!(std::ptr::eq(element.location(), &*u.origin.location));
}

/// The case that motivated the design. `B()` and `C {}` carry no payload and
/// are still not unit variants: Rust demands the delimiters wherever the variant
/// is named. The classification calls all three unit *groups*; `spell` keeps
/// them apart, off the syntax.
#[test]
fn empty_delimiters_survive_and_spell() {
    let element = parse_one(syn::parse_quote!(
        pub enum E {
            A,
            B(),
            C {},
            D(u32),
        }
    ));
    let e = as_enum(&element);

    // All four groups, and which of them are empty.
    assert_eq!(
        e.variants.iter().map(|v| v.is_unit()).collect::<Vec<_>>(),
        vec![true, true, true, false]
    );

    let spell = |v: &Variant| {
        let name = &v.name;
        v.spell(quote::quote!(E::#name), &[]).to_string()
    };
    assert_eq!(spell(&e.variants[0]), "E :: A");
    assert_eq!(spell(&e.variants[1]), "E :: B ()");
    assert_eq!(spell(&e.variants[2]), "E :: C { }");

    // And with payloads, in both addressing modes.
    let element = parse_one(syn::parse_quote!(
        pub enum Reading {
            Exact(i64),
            Range { low: i64, high: i64 },
        }
    ));
    let e = as_enum(&element);
    let bind = |v: &Variant| {
        let parts: Vec<_> = v
            .fields
            .iter()
            .map(|f| f.bind(&quote::format_ident!("__f{}", f.index)))
            .collect();
        let name = &v.name;
        v.spell(quote::quote!(Reading::#name), &parts).to_string()
    };
    assert_eq!(bind(&e.variants[0]), "Reading :: Exact (__f0)");
    assert_eq!(
        bind(&e.variants[1]),
        "Reading :: Range { low : __f0 , high : __f1 }"
    );
}

/// The same property for a struct, which is why it needs no modelled shape
/// either. `struct S;` and `struct S {}` hold zero fields alike and are still
/// spelled differently wherever Rust names them — one `spell` off the syntax
/// covers a struct and a variant, in either direction.
#[test]
fn struct_delimiters_survive_and_spell() {
    let spell = |item: syn::Item, parts: &[proc_macro2::TokenStream]| {
        let element = parse_one(item);
        let s = as_struct(&element);
        let name = &s.name;
        (
            s.fields.is_some(),
            s.fields().len(),
            s.spell(quote::quote!(#name), parts).to_string(),
        )
    };

    assert_eq!(
        spell(
            syn::parse_quote!(
                pub struct A;
            ),
            &[]
        ),
        (true, 0, "A".to_string())
    );
    assert_eq!(
        spell(
            syn::parse_quote!(
                pub struct B {}
            ),
            &[]
        ),
        (true, 0, "B { }".to_string())
    );
    assert_eq!(
        spell(
            syn::parse_quote!(
                pub struct C {
                    pub x: u8,
                }
            ),
            &[quote::quote!(x: __f0)]
        ),
        (true, 1, "C { x : __f0 }".to_string())
    );
    // Opaque: no modelled fields, and still spellable.
    assert_eq!(
        spell(
            syn::parse_quote!(
                pub struct D(Whatever<'_, dyn Trait>);
            ),
            &[quote::quote!(__f0)]
        ),
        (false, 0, "D (__f0)".to_string())
    );
}

/// A discriminant is two facts with two homes: the number is modelled, the
/// spelling stays in the variant's slice. `0x07` must reach a C header as
/// `0x07`, and no reconstruction from `7` can do that.
#[test]
fn discriminant_number_and_spelling_both_survive() {
    let element = parse_one(syn::parse_quote!(
        pub enum Priority {
            Low = 0x07,
            High,
        }
    ));
    let e = as_enum(&element);

    assert_eq!(
        e.discriminant_values().expect("literal discriminants"),
        vec![(&e.variants[0].name, 7), (&e.variants[1].name, 8)]
    );
    let (_, expr) = e.variants[0]
        .origin
        .syntax
        .discriminant
        .as_ref()
        .expect("an explicit discriminant");
    assert_eq!(tokens(expr), "0x07");
    assert!(e.variants[1].origin.syntax.discriminant.is_none());
}

/// A discriminant the frontend cannot evaluate breaks the *numeric* chain and
/// nothing else: the spelling is still there, so a consumer that re-emits it
/// carries on while one that needs the number is told which variant to blame.
#[test]
fn an_unevaluable_discriminant_keeps_its_spelling() {
    let element = parse_one(syn::parse_quote!(
        pub enum E {
            A = OTHER,
            B,
        }
    ));
    let e = as_enum(&element);
    assert!(e.variants.iter().all(|v| v.discriminant.is_none()));
    assert_eq!(e.discriminant_values().expect_err("no numbers"), "A");
    let (_, expr) = e.variants[0]
        .origin
        .syntax
        .discriminant
        .as_ref()
        .expect("explicit");
    assert_eq!(tokens(expr), "OTHER");
}

/// A discriminant at the top of the range is valid Rust, so running out of
/// `i64` ends the numeric chain the way an unevaluable spelling does — it does
/// not panic during ingest, which would take down every consumer including the
/// ones that only re-emit.
#[test]
fn a_discriminant_at_the_top_of_the_range_does_not_overflow() {
    let element = parse_one(syn::parse_quote!(
        #[repr(u64)]
        pub enum E {
            A = 9223372036854775807,
            B,
        }
    ));
    let e = as_enum(&element);
    assert_eq!(e.variants[0].discriminant, Some(i64::MAX));
    assert_eq!(e.variants[1].discriminant, None);

    // The last variant needs no successor, so it must not fail either.
    let element = parse_one(syn::parse_quote!(
        pub enum E {
            A = 9223372036854775807,
        }
    ));
    assert_eq!(
        as_enum(&element).discriminant_values().expect("a number")[0].1,
        i64::MAX
    );
}

/// The bottom of the range too. `i64::MIN` is a valid Rust discriminant, and its
/// magnitude is one past `i64::MAX` — so the sign has to be applied before the
/// range check, not after.
#[test]
fn a_discriminant_at_the_bottom_of_the_range_evaluates() {
    let element = parse_one(syn::parse_quote!(
        #[repr(i64)]
        pub enum E {
            A = -9223372036854775808,
            B,
        }
    ));
    let e = as_enum(&element);
    assert_eq!(e.variants[0].discriminant, Some(i64::MIN));
    assert_eq!(e.variants[1].discriminant, Some(i64::MIN + 1));
    // And the spelling is still the source's, as for any other discriminant.
    let (_, expr) = e.variants[0]
        .origin
        .syntax
        .discriminant
        .as_ref()
        .expect("explicit");
    assert_eq!(tokens(expr), "- 9223372036854775808");

    // One step further out is not a number, and ends the chain rather than
    // panicking — the same contract as an unevaluable spelling.
    let element = parse_one(syn::parse_quote!(
        #[repr(i128)]
        pub enum E {
            A = -9223372036854775809,
            B,
        }
    ));
    let e = as_enum(&element);
    assert_eq!(e.variants[0].discriminant, None);
    assert_eq!(e.variants[1].discriminant, None);
    assert_eq!(e.discriminant_values().expect_err("no numbers"), "A");
}

/// An array's extent is modelled as a number AND the const it named, while the
/// type's slice keeps the symbolic spelling — the three-way split that lets one
/// consumer emit `[u8; 4]` and another `uint8_t tag[TAG_LEN]`.
#[test]
fn array_extent_carries_number_const_and_spelling() {
    let elements = parse(vec![
        tag_len_const(),
        syn::parse_quote!(
            pub struct Marker {
                pub tag: [u8; TAG_LEN],
                pub pad: [u8; 2],
            }
        ),
    ]);
    let fields = as_struct(&elements[1]).fields();

    let named = fields[0].ty.array_extent().expect("an extent");
    assert_eq!(named.value, 4);
    assert_eq!(named.const_id().expect("a const").name, "TAG_LEN");
    assert_eq!(tokens(&fields[0].ty.origin.syntax), "[u8 ; TAG_LEN]");

    let literal = fields[1].ty.array_extent().expect("an extent");
    assert_eq!(literal.value, 2);
    assert!(literal.const_id().is_none());
}

/// The whole item is kept too, so anything the element model does not describe
/// — attributes, `cfg`, the function body — is still emittable.
#[test]
fn the_whole_item_survives() {
    let source: syn::ItemFn = syn::parse_quote!(
        /// Adds two numbers.
        #[inline]
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    );
    let element = parse_one(syn::Item::Fn(source.clone()));
    assert_eq!(tokens(&as_fn(&element).origin.syntax), tokens(&source));
    assert_eq!(tokens(&element.syntax()), tokens(&syn::Item::Fn(source)));
}

/// An item the language cannot express still keeps its tokens, so a diagnosis
/// can quote the source and nothing is lost by refusing it.
#[test]
fn an_unsupported_item_keeps_its_tokens() {
    let source: syn::Item = syn::parse_quote!(
        pub type Alias = u32;
    );
    let element = parse_one(source.clone());
    assert!(matches!(element, Element::Unsupported(_)));
    assert_eq!(tokens(&element.syntax()), tokens(&source));
}
