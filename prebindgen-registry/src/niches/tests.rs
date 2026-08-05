use quote::ToTokens;

use super::*;

fn slot_strs(s: &NicheSlot) -> (String, String) {
    (
        s.value.to_token_stream().to_string(),
        s.matches.to_token_stream().to_string(),
    )
}

#[test]
fn empty_is_empty() {
    let n = Niches::empty();
    assert!(n.is_empty());
    assert_eq!(n.len(), 0);
    assert!(n.carve().is_none());
}

#[test]
fn one_constructs_single_slot() {
    let n = Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0));
    assert_eq!(n.len(), 1);
    let (slot, rest) = n.carve().unwrap();
    let (val, pred) = slot_strs(&slot);
    assert_eq!(val, "0i64");
    assert_eq!(pred, "* v == 0");
    assert!(rest.is_empty());
}

#[test]
fn from_slots_preserves_order() {
    let n = Niches::from_slots([
        NicheSlot {
            value: syn::parse_quote!(0i32),
            matches: syn::parse_quote!(*v == 0),
        },
        NicheSlot {
            value: syn::parse_quote!(-1i32),
            matches: syn::parse_quote!(*v == -1),
        },
        NicheSlot {
            value: syn::parse_quote!(99i32),
            matches: syn::parse_quote!(*v == 99),
        },
    ]);
    assert_eq!(n.len(), 3);
    let (s0, n) = n.carve().unwrap();
    assert_eq!(slot_strs(&s0).0, "0i32");
    let (s1, n) = n.carve().unwrap();
    assert_eq!(slot_strs(&s1).0, "- 1i32");
    let (s2, n) = n.carve().unwrap();
    assert_eq!(slot_strs(&s2).0, "99i32");
    assert!(n.is_empty());
}

/// Carving propagates the remainder, allowing wrappers to stack.
/// This mirrors `Option<Option<TypeWithTwoNiches>>` collapsing to
/// the same wire as the inner type.
#[test]
fn cascading_carve() {
    let n = Niches::from_slots([
        NicheSlot {
            value: syn::parse_quote!(i32::MIN),
            matches: syn::parse_quote!(*v == i32::MIN),
        },
        NicheSlot {
            value: syn::parse_quote!(i32::MAX),
            matches: syn::parse_quote!(*v == i32::MAX),
        },
    ]);

    // Outer wrapper takes the first niche.
    let (outer, rest1) = n.carve().unwrap();
    assert_eq!(slot_strs(&outer).0, "i32 :: MIN");
    assert_eq!(rest1.len(), 1);

    // Inner wrapper (carving from `rest1`) takes the second.
    let (inner, rest2) = rest1.carve().unwrap();
    assert_eq!(slot_strs(&inner).0, "i32 :: MAX");
    assert!(rest2.is_empty());
}

/// `Niches::default()` equivalence to `empty()`.
#[test]
fn default_is_empty() {
    let n = Niches::default();
    assert!(n.is_empty());
}

/// Cloning produces independent ownership; carving the clone
/// doesn't disturb the original (each carve consumes by value).
#[test]
fn clone_independence() {
    let original = Niches::one(syn::parse_quote!(0i32), syn::parse_quote!(*v == 0));
    let cloned = original.clone();
    let (_slot, rest) = cloned.carve().unwrap();
    assert!(rest.is_empty());
    assert_eq!(original.len(), 1, "original unaffected by clone's carve");
}
