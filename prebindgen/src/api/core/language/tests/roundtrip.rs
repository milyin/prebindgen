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
            .map(|p| tokens(&p.syntax))
            .collect::<Vec<_>>(),
        vec![
            "key : & 'a KeyExpr",
            "payload : Vec < u8 >",
            "count : Option < i32 >",
        ]
    );
    // The lifetime is nowhere in the classification, and still survives.
    assert_eq!(tokens(&func.params[0].ty.syntax), "& 'a KeyExpr");
    assert!(matches!(func.params[0].ty.kind, TypeKind::Ref { .. }));

    assert_eq!(
        tokens(&func.ret.as_ref().expect("a return type").syntax),
        "Result < () , Error >"
    );
}

/// A defaulted return is distinguishable from a written `-> ()`: the first has
/// no tokens to keep, the second is a modelled unit whose slice is `()`.
#[test]
fn a_defaulted_return_is_not_a_written_unit() {
    let defaulted = parse_one(syn::parse_quote!(
        pub fn a() {}
    ));
    assert!(as_fn(&defaulted).ret.is_none());

    let written = parse_one(syn::parse_quote!(
        pub fn b() -> () {}
    ));
    let ret = as_fn(&written).ret.as_ref().expect("a written return");
    assert!(matches!(ret.kind, TypeKind::Unit));
    assert_eq!(tokens(&ret.syntax), "()");
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
    let fields = as_struct(&element).fields.named();
    assert_eq!(fields.len(), 2);
    assert!(tokens(&fields[0].syntax).contains("The key it was published on."));
    assert_eq!(
        tokens(&fields[1].syntax),
        "# [allow (dead_code)] pub (crate) seq : u64"
    );
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
        .syntax
        .discriminant
        .as_ref()
        .expect("an explicit discriminant");
    assert_eq!(tokens(expr), "0x07");
    assert!(e.variants[1].syntax.discriminant.is_none());
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
    let fields = as_struct(&elements[1]).fields.named();

    let named = fields[0].ty.array_extent().expect("an extent");
    assert_eq!(named.value, 4);
    assert_eq!(named.const_id().expect("a const").name, "TAG_LEN");
    assert_eq!(tokens(&fields[0].ty.syntax), "[u8 ; TAG_LEN]");

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
    assert_eq!(tokens(&as_fn(&element).syntax), tokens(&source));
    assert_eq!(tokens(&element.syntax()), tokens(&syn::Item::Fn(source)));
}

/// An item the language does not interpret is carried verbatim and classified
/// as nothing at all.
#[test]
fn passthrough_is_verbatim() {
    let source: syn::Item = syn::parse_quote!(
        pub type Alias = u32;
    );
    let element = parse_one(source.clone());
    assert!(matches!(element, Element::Passthrough(_)));
    assert_eq!(tokens(&element.syntax()), tokens(&source));
    assert!(element.name().is_none());
}
