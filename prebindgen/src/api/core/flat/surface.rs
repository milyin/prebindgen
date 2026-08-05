//! One test: **nothing public in `flat` hands out captured syntax.**
//!
//! The census had a 300-line `escape_surface_is_closed` doing this, and C6
//! deleted it on the argument that visibility makes it unnecessary. That
//! argument was wrong, and this file exists because a reviewer proved it:
//! `TypeRef::as_syn` — the single door this whole umbrella is about — was
//! still `pub` after C5 claimed the type half was sealed, along with the three
//! `spell(head, parts)` shape methods.
//!
//! Visibility enforces the boundary only for the methods someone remembered to
//! seal. What it cannot do is *notice the one you missed*, and at C5 I checked
//! the three methods I had changed rather than the property I had claimed.
//!
//! So this reads the module's own surface, as the census did, and fails on
//! anything public returning a `syn` type. It is ~60 lines rather than ~300
//! because it no longer has to bucket, count or attribute anything: with the
//! capability in place the answer is simply *none*, and a new door is a test
//! failure rather than a number to justify.

use std::{fs, path::Path};

/// `syn` types small enough that handing one out is not handing out structure.
///
/// The allowlist is the exception side on purpose — a `syn` type nobody
/// considered is a door until someone argues it is a leaf, and that argument is
/// a diff on this line. (Inverting it is what the census learned the hard way:
/// a method returning `&[syn::Attribute]` was invisible because `Attribute` was
/// not on the opposite list.)
const LEAF: &[&str] = &["Ident", "Lifetime", "Member", "Index"];

/// A **transformer** takes a node and gives one back, so it hands out nothing a
/// caller did not already hold. Exempt by name *and* shape — exactly one
/// parameter, itself a node — because a name alone is defeated by
/// `fn leak(model: &TypeRef, _decoy: &syn::Type) -> &syn::Type`, which the
/// census learned before this file existed.
const TRANSFORMERS: &[&str] = &[
    "canonical_type",
    "normalize_type",
    "normalize_item_types",
    "extract_fn_trait_args",
];

fn flat_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api/core/flat")
}

/// Public items in `flat` whose return type names a non-leaf `syn` type.
fn doors(file: &syn::File) -> Vec<String> {
    fn is_public(v: &syn::Visibility) -> bool {
        matches!(v, syn::Visibility::Public(_))
    }
    fn names_a_node(ty: &syn::ReturnType) -> Option<String> {
        let syn::ReturnType::Type(_, t) = ty else {
            return None;
        };
        let toks: Vec<proc_macro2::TokenTree> =
            quote::ToTokens::to_token_stream(t).into_iter().collect();
        for (i, tt) in toks.iter().enumerate() {
            let proc_macro2::TokenTree::Ident(id) = tt else {
                continue;
            };
            if *id != "syn" {
                continue;
            }
            // `syn :: <Name>` — two `:` puncts then the name.
            if let Some(proc_macro2::TokenTree::Ident(name)) = toks.get(i + 3) {
                let n = name.to_string();
                if !LEAF.contains(&n.as_str()) {
                    return Some(n);
                }
            }
        }
        None
    }
    fn is_transformer(sig: &syn::Signature) -> bool {
        TRANSFORMERS.contains(&sig.ident.to_string().as_str())
            && sig.inputs.len() == 1
            && matches!(sig.inputs.first(), Some(syn::FnArg::Typed(pt))
                if quote::ToTokens::to_token_stream(&pt.ty).to_string().contains("syn"))
    }
    // A trait impl's methods are as visible as the trait. `Shaped` is
    // `pub(in crate::api::core)`, so its impls are not a public surface — the
    // trait's own declaration is what this checks, not the impl block.
    let sealed_traits: Vec<String> = file
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Trait(t) if !is_public(&t.vis) => Some(t.ident.to_string()),
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    let walk = |items: &[syn::Item], out: &mut Vec<String>| {
        for item in items {
            match item {
                syn::Item::Fn(f) if is_public(&f.vis) => {
                    if is_transformer(&f.sig) {
                        continue;
                    }
                    if let Some(n) = names_a_node(&f.sig.output) {
                        out.push(format!("fn {} -> syn::{n}", f.sig.ident));
                    }
                }
                syn::Item::Impl(im) => {
                    let sealed = im.trait_.as_ref().is_some_and(|(_, p, _)| {
                        p.segments
                            .last()
                            .is_some_and(|s| sealed_traits.contains(&s.ident.to_string()))
                    });
                    if sealed {
                        continue;
                    }
                    for it in &im.items {
                        // A trait impl's methods are as public as the trait.
                        let syn::ImplItem::Fn(f) = it else { continue };
                        if im.trait_.is_some() || is_public(&f.vis) {
                            if let Some(n) = names_a_node(&f.sig.output) {
                                out.push(format!("{} -> syn::{n}", f.sig.ident));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    };
    walk(&file.items, &mut out);
    out
}

#[test]
fn nothing_public_in_flat_hands_out_syntax() {
    let mut found: Vec<String> = Vec::new();
    for entry in fs::read_dir(flat_dir()).expect("flat is a directory") {
        let path = entry.expect("readable").path();
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("tests.rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        // This file, and `surface.rs` itself, are test-only.
        if name == "surface.rs" {
            continue;
        }
        let text = fs::read_to_string(&path).expect("utf-8");
        let file: syn::File = syn::parse_str(&text).expect("flat source parses");
        for d in doors(&file) {
            found.push(format!("  {name}: {d}"));
        }
    }
    found.sort();
    assert!(
        found.is_empty(),
        "PUBLIC SYNTAX DOOR IN `flat`\n{}\n\n\
         Captured syntax leaves the model through `core::emit::Emit` and nothing \
         else — that is what lets an adapter's *decisions* be checked by the \
         compiler instead of by a census. A `pub` method here returning a `syn` \
         node reopens the boundary for every consumer, including out-of-crate \
         ones no test in this repo compiles.\n\n\
         Seal it (`pub(in crate::api::core)`) and expose it on `Emit` if an \
         emitter needs it; if it is genuinely a leaf, add it to `LEAF` here and \
         say why.\n",
        found.join("\n")
    );
}
