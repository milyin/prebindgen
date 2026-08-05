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
        names_a_type(t)
    }
    /// A `syn` mention in any type position, not just a return.
    fn names_a_type(t: &syn::Type) -> Option<String> {
        let toks: Vec<proc_macro2::TokenTree> =
            quote::ToTokens::to_token_stream(t).into_iter().collect();
        for (i, tt) in toks.iter().enumerate() {
            let proc_macro2::TokenTree::Ident(id) = tt else {
                continue;
            };
            if *id != "syn" {
                continue;
            }
            if let Some(proc_macro2::TokenTree::Ident(name)) = toks.get(i + 3) {
                let n = name.to_string();
                if !LEAF.contains(&n.as_str()) {
                    return Some(n);
                }
            }
        }
        None
    }
    /// `Origin<..>` is the seal itself — every element carries one, and its own
    /// accessors are what this file exists to keep sealed.
    fn is_origin(t: &syn::Type) -> bool {
        let toks = quote::ToTokens::to_token_stream(t).to_string();
        toks.starts_with("Origin ") || toks.starts_with("Origin<")
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
    // Every item kind that can put a `syn` type on a public surface. The list
    // is the census's, because each entry was there for a bypass someone found:
    // a type alias is transparent, an associated `type Target` hands the node to
    // `&*x` with no method to see, a public field is an accessor you cannot
    // remove, a trait method is as visible as its trait, and a nested module
    // hides all of the above one level down.
    fn walk(items: &[syn::Item], sealed: &[String], out: &mut Vec<String>) {
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
                // `pub type Node = syn::Type` — reported whatever its OWN
                // visibility is: an alias is transparent, so a private one still
                // lets a public signature hand the node out under its name.
                syn::Item::Type(t) => {
                    if let Some(n) = names_a_type(&t.ty) {
                        out.push(format!("type {} = syn::{n}", t.ident));
                    }
                }
                syn::Item::Impl(im) => {
                    let is_sealed_trait = im.trait_.as_ref().is_some_and(|(_, p, _)| {
                        p.segments
                            .last()
                            .is_some_and(|s| sealed.contains(&s.ident.to_string()))
                    });
                    if is_sealed_trait {
                        continue;
                    }
                    for it in &im.items {
                        match it {
                            syn::ImplItem::Fn(f) => {
                                // A trait impl's method is as public as the trait.
                                if (im.trait_.is_some() || is_public(&f.vis))
                                    && !is_transformer(&f.sig)
                                {
                                    if let Some(n) = names_a_node(&f.sig.output) {
                                        out.push(format!("{} -> syn::{n}", f.sig.ident));
                                    }
                                }
                            }
                            // `impl Deref for TypeRef { type Target = syn::Type; }`
                            // hands the node to `&*ty` with no method to look at.
                            syn::ImplItem::Type(t) => {
                                if let Some(n) = names_a_type(&t.ty) {
                                    out.push(format!("type {} = syn::{n}", t.ident));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                syn::Item::Trait(t) if is_public(&t.vis) => {
                    for it in &t.items {
                        match it {
                            syn::TraitItem::Fn(f) if !is_transformer(&f.sig) => {
                                if let Some(n) = names_a_node(&f.sig.output) {
                                    out.push(format!("trait fn {} -> syn::{n}", f.sig.ident));
                                }
                            }
                            syn::TraitItem::Type(ty) => {
                                if let Some((_, d)) = &ty.default {
                                    if let Some(n) = names_a_type(d) {
                                        out.push(format!("trait type {} = syn::{n}", ty.ident));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // A public field IS an accessor, and one you cannot deprecate.
                // A field whose type is an `Origin` is exempt: that wrapper is
                // the seal, and every element carries one.
                syn::Item::Struct(st) if is_public(&st.vis) => {
                    for f in &st.fields {
                        if is_public(&f.vis) && !is_origin(&f.ty) {
                            if let Some(n) = names_a_type(&f.ty) {
                                let name = f
                                    .ident
                                    .as_ref()
                                    .map_or_else(|| "_".to_string(), |i| i.to_string());
                                out.push(format!("field {} : syn::{n}", name));
                            }
                        }
                    }
                }
                // An enum variant's fields are public with the variant.
                syn::Item::Enum(en) if is_public(&en.vis) => {
                    for v in &en.variants {
                        for f in &v.fields {
                            if !is_origin(&f.ty) {
                                if let Some(n) = names_a_type(&f.ty) {
                                    out.push(format!("variant {} : syn::{n}", v.ident));
                                }
                            }
                        }
                    }
                }
                // A nested module hides every one of the above a level down.
                syn::Item::Mod(m) if is_public(&m.vis) => {
                    if let Some((_, items)) = &m.content {
                        walk(items, sealed, out);
                    }
                }
                _ => {}
            }
        }
    }

    walk(&file.items, &sealed_traits, &mut out);
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

/// The guard catches every item kind that can put a `syn` type on a public
/// surface.
///
/// Without this, the guard's coverage is an assumption — and the first version
/// of it silently missed five of these six, which review caught. Each case is
/// parsed as if it were a file in `flat`, so adding an item kind to [`doors`]
/// means adding it here too.
#[test]
fn the_guard_sees_every_item_kind() {
    let cases: &[(&str, &str)] = &[
        ("free fn", "pub fn leak() -> syn::Type { unimplemented!() }"),
        (
            "trait method",
            "pub trait T { fn leak(&self) -> syn::Type; }",
        ),
        ("type alias", "pub type Alias = syn::Type;"),
        ("public field", "pub struct S { pub node: syn::Type }"),
        (
            "associated type",
            "impl std::ops::Deref for S { type Target = syn::Type;              fn deref(&self) -> &syn::Type { unimplemented!() } }",
        ),
        (
            "nested module",
            "pub mod inner { pub fn leak() -> syn::Type { unimplemented!() } }",
        ),
        (
            "enum variant field",
            "pub enum E { V(syn::Type) }",
        ),
    ];
    let mut missed = Vec::new();
    for (what, src) in cases {
        let file: syn::File = syn::parse_str(src).expect("case parses");
        if doors(&file).is_empty() {
            missed.push(*what);
        }
    }
    assert!(
        missed.is_empty(),
        "the surface guard does not see: {missed:?}\n\n         Each of these can hand a `syn` node to a consumer, so each has to be a \
         door. Add the item kind to `doors` — and note that the reason this test \
         exists is that the first version of the guard missed five of them.\n"
    );
}

/// A **transformer** and a **leaf** are still exempt, and the `_decoy` trick
/// still fails — the two rules the census learned from real bypasses.
#[test]
fn the_exemptions_are_shape_checked() {
    let exempt: syn::File =
        syn::parse_str("pub fn canonical_type(ty: &syn::Type) -> syn::Type { unimplemented!() }")
            .unwrap();
    assert!(
        doors(&exempt).is_empty(),
        "a one-node transformer is exempt"
    );

    let leaf: syn::File =
        syn::parse_str("pub fn name(&self) -> syn::Ident { unimplemented!() }").unwrap();
    assert!(doors(&leaf).is_empty(), "a leaf is not a door");

    // Named like a transformer, shaped like a leak: the model is the source of
    // the returned node and the `syn` parameter is a decoy.
    let decoy: syn::File = syn::parse_str(
        "pub fn canonical_type(model: &TypeRef, _decoy: &syn::Type) -> syn::Type          { unimplemented!() }",
    )
    .unwrap();
    assert_eq!(
        doors(&decoy).len(),
        1,
        "a transformer is exempt by name AND shape — one parameter, itself a node"
    );
}
