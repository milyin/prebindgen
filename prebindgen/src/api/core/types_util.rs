//! Shared `syn::Type` shape utilities — the Option/Vec/reference peelers and
//! short-name helpers every pipeline stage needs. One definition here
//! replaces the per-module copies that used to live in `core::unfold`,
//! `core::expand`, and the jnigen adapter.

use proc_macro2::Span;
use quote::ToTokens;

/// The single-segment path type for a bare item ident (`Foo` → `Foo`) —
/// direct construction, no string round trip, cannot fail.
pub fn type_from_ident(ident: &syn::Ident) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path::from(ident.clone()),
    })
}

/// Normalize a type to its canonical flat-namespace spelling (issue #95).
/// The COMPLETE equivalence rule set — any spelling not listed is preserved
/// verbatim:
///
/// 1. `Type::Group` / `Type::Paren` wrappers unwrap (`(Foo)` ≡ `Foo`).
/// 2. A multi-segment path headed by `crate` / `self` reduces to its final
///    segment, keeping that segment's generic arguments (`crate::a::Foo<T>`
///    ≡ `Foo<T>`). Sound because the flat namespace indexes at most one
///    item per bare ident, and a `crate::` path in a captured item can only
///    denote the source crate's own item.
/// 3. A multi-segment path headed by a name in `source_modules` (the
///    `#[prebindgen]` source crates chained into the registry,
///    hyphens-as-underscores) reduces the same way (`myflat::Foo` ≡ `Foo`).
///    Pure callers pass `&[]`.
/// 4. The std prelude whitelist reduces to its bare form — exactly
///    `std|core|alloc :: vec::Vec | option::Option | result::Result |
///    string::String | boxed::Box` (with or without a leading `::`).
///    Nothing else: `std::ffi::CString` stays qualified, and unknown crate
///    paths (`zenoh::KeyExpr`) are NEVER touched — the registry has no
///    index of a foreign namespace, so `a::KeyExpr` and `b::KeyExpr` may be
///    genuinely distinct types and their spelling is their identity.
/// 5. Lifetimes are NOT normalized (`&'a T` ≠ `&T`, `Foo<'static>` ≠ `Foo`)
///    — [`match_pattern`] treats lifetimes as fixed structure and
///    foreign-type declarations (`ptr_class!(ZKeyExpr<'static>)`) rely on
///    the verbatim spelling.
///
/// Idempotent; recurses through references, slices, tuples, pointers,
/// generic arguments, and `impl Trait` bounds. Paths with a qualified self
/// (`<T as Trait>::Assoc`) are left untouched.
pub fn normalize_type(ty: &mut syn::Type, source_modules: &[String]) {
    use syn::visit_mut::VisitMut;
    struct Normalizer<'a> {
        modules: &'a [String],
    }
    impl VisitMut for Normalizer<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            // Unwrap (possibly nested) group/paren wrappers in place.
            loop {
                match ty {
                    syn::Type::Group(g) => *ty = (*g.elem).clone(),
                    syn::Type::Paren(p) => *ty = (*p.elem).clone(),
                    _ => break,
                }
            }
            if let syn::Type::Path(tp) = ty {
                if tp.qself.is_none() {
                    reduce_flat_path(&mut tp.path, self.modules);
                }
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }
    }
    Normalizer {
        modules: source_modules,
    }
    .visit_type_mut(ty);
}

/// Apply [`normalize_type`] to every type position inside an item — fn
/// signatures, struct fields, enum variants, const types. The ingest-time
/// pass ([`crate::api::core::registry::Registry::from_items`]) that makes
/// captured spellings canonical before any key is formed, so every
/// downstream `TypeKey::from_type` sees the flat spelling.
pub fn normalize_item_types(item: &mut syn::Item, source_modules: &[String]) {
    use syn::visit_mut::VisitMut;
    struct ItemNormalizer<'a> {
        modules: &'a [String],
    }
    impl VisitMut for ItemNormalizer<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            // Normalizes the whole subtree; no further descent needed.
            normalize_type(ty, self.modules);
        }
    }
    ItemNormalizer {
        modules: source_modules,
    }
    .visit_item_mut(item);
}

/// The path-reduction step of [`normalize_type`]: collapse a reducible
/// multi-segment path to its final segment. See the rule list there.
fn reduce_flat_path(path: &mut syn::Path, source_modules: &[String]) {
    if path.segments.len() < 2 {
        return;
    }
    let head = path
        .segments
        .first()
        .expect("len checked")
        .ident
        .to_string();
    let reduce = match head.as_str() {
        "crate" | "self" => true,
        "std" | "core" | "alloc" => {
            let tail: Vec<String> = path
                .segments
                .iter()
                .skip(1)
                .map(|s| s.ident.to_string())
                .collect();
            matches!(
                tail.iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_slice(),
                ["vec", "Vec"]
                    | ["option", "Option"]
                    | ["result", "Result"]
                    | ["string", "String"]
                    | ["boxed", "Box"]
            )
        }
        other => source_modules.iter().any(|m| m == other),
    };
    if reduce {
        let last = path.segments.last().expect("len checked").clone();
        path.leading_colon = None;
        path.segments = std::iter::once(last).collect();
    }
}

/// Structurally match a concrete type `ty` against a wildcard `pattern` (a
/// `syn::Type` whose `_` placeholders are [`syn::Type::Infer`]). On success,
/// returns the subtrees of `ty` captured at each wildcard, in left-to-right
/// document order; `None` if the shapes don't unify.
///
/// This is the inverse of pattern substitution: `match_pattern(ty, pat)` finds
/// the args `a` such that substituting them into `pat` reproduces `ty`. It
/// replaces the rank resolver's combinatorial wildcard *enumeration* with a
/// direct unify — an adapter (or a user-registered wrapper table) keeps full
/// expressive power (any depth) without the framework enumerating every
/// placement. Handles the type shapes that appear as wildcard patterns
/// (`Path<…>`, `&`/`&mut`, `[_]`, `(…)`, `*const`/`*mut`); other leaves compare
/// by token equality.
pub fn match_pattern(ty: &syn::Type, pattern: &syn::Type) -> Option<Vec<syn::Type>> {
    let mut out = Vec::new();
    if unify(ty, pattern, &mut out) {
        Some(out)
    } else {
        None
    }
}

/// Count the wildcard (`_`) placeholders in a pattern — its "openness". Used to
/// order overlapping registered patterns most-specific-first (fewer wildcards
/// win, e.g. `Result<_, ConcreteErr>` over `Result<_, _>`).
pub fn wildcard_count(pattern: &syn::Type) -> usize {
    if matches!(pattern, syn::Type::Infer(_)) {
        return 1;
    }
    immediate_pattern_children(pattern)
        .iter()
        .map(wildcard_count)
        .sum()
}

/// Immediate substitutable child positions of a type (the generic type-args of
/// a path, the referent of a `&`/`*`, the element of a slice/array, the members
/// of a tuple). Mirrors the resolver's traversal so `match_pattern` /
/// `wildcard_count` descend the same positions wildcards can occupy.
fn immediate_pattern_children(ty: &syn::Type) -> Vec<syn::Type> {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .and_then(|seg| match &seg.arguments {
                syn::PathArguments::AngleBracketed(ab) => Some(
                    ab.args
                        .iter()
                        .filter_map(|a| match a {
                            syn::GenericArgument::Type(t) => Some(t.clone()),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default(),
        syn::Type::Reference(r) => vec![(*r.elem).clone()],
        syn::Type::Ptr(p) => vec![(*p.elem).clone()],
        syn::Type::Slice(s) => vec![(*s.elem).clone()],
        syn::Type::Array(a) => vec![(*a.elem).clone()],
        syn::Type::Tuple(t) => t.elems.iter().cloned().collect(),
        syn::Type::Group(g) => immediate_pattern_children(&g.elem),
        syn::Type::Paren(p) => immediate_pattern_children(&p.elem),
        _ => Vec::new(),
    }
}

fn unify(ty: &syn::Type, pat: &syn::Type, out: &mut Vec<syn::Type>) -> bool {
    if matches!(pat, syn::Type::Infer(_)) {
        out.push(ty.clone());
        return true;
    }
    match (ty, pat) {
        (syn::Type::Path(t), syn::Type::Path(p)) => {
            // Same path up to the last segment's generic args; unify those.
            if t.qself.is_some() || p.qself.is_some() {
                return token_eq(ty, pat);
            }
            let (ts, ps) = (&t.path.segments, &p.path.segments);
            if ts.len() != ps.len() {
                return false;
            }
            for (i, (tseg, pseg)) in ts.iter().zip(ps.iter()).enumerate() {
                if tseg.ident != pseg.ident {
                    return false;
                }
                let is_last = i + 1 == ts.len();
                // Non-last segments (and non-angle-bracketed last segments) must
                // match verbatim; the last segment's generic args unify.
                match (&tseg.arguments, &pseg.arguments) {
                    (
                        syn::PathArguments::AngleBracketed(ta),
                        syn::PathArguments::AngleBracketed(pa),
                    ) if is_last => {
                        // Compare ALL generic args positionally — lifetimes,
                        // const generics, and bindings are part of the fixed
                        // pattern structure and must match token-for-token; only
                        // a `_` in a type position captures. (Mirrors the old
                        // enumerator's exact `TypeKey` match, so e.g.
                        // `Foo<'static, _>` does NOT match `Foo<'a, T>`.)
                        if ta.args.len() != pa.args.len() {
                            return false;
                        }
                        for (a, b) in ta.args.iter().zip(pa.args.iter()) {
                            match (a, b) {
                                (
                                    syn::GenericArgument::Type(at),
                                    syn::GenericArgument::Type(bt),
                                ) => {
                                    if !unify(at, bt, out) {
                                        return false;
                                    }
                                }
                                (a, b) => {
                                    if a.to_token_stream().to_string()
                                        != b.to_token_stream().to_string()
                                    {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                    (a, b) => {
                        if a.to_token_stream().to_string() != b.to_token_stream().to_string() {
                            return false;
                        }
                    }
                }
            }
            true
        }
        (syn::Type::Reference(t), syn::Type::Reference(p)) => {
            // Mutability and lifetime are fixed structure — `&'static _` must not
            // match `&'a T`, and `&_` (no lifetime) must not match `&'a T`.
            t.mutability.is_some() == p.mutability.is_some()
                && lifetime_eq(&t.lifetime, &p.lifetime)
                && unify(&t.elem, &p.elem, out)
        }
        (syn::Type::Ptr(t), syn::Type::Ptr(p)) => {
            t.mutability.is_some() == p.mutability.is_some()
                && t.const_token.is_some() == p.const_token.is_some()
                && unify(&t.elem, &p.elem, out)
        }
        (syn::Type::Slice(t), syn::Type::Slice(p)) => unify(&t.elem, &p.elem, out),
        (syn::Type::Array(t), syn::Type::Array(p)) => {
            t.len.to_token_stream().to_string() == p.len.to_token_stream().to_string()
                && unify(&t.elem, &p.elem, out)
        }
        (syn::Type::Tuple(t), syn::Type::Tuple(p)) => {
            t.elems.len() == p.elems.len()
                && t.elems
                    .iter()
                    .zip(p.elems.iter())
                    .all(|(a, b)| unify(a, b, out))
        }
        (syn::Type::Group(t), _) => unify(&t.elem, pat, out),
        (_, syn::Type::Group(p)) => unify(ty, &p.elem, out),
        (syn::Type::Paren(t), _) => unify(&t.elem, pat, out),
        (_, syn::Type::Paren(p)) => unify(ty, &p.elem, out),
        _ => token_eq(ty, pat),
    }
}

/// Two optional reference lifetimes are equal iff both are absent or name the
/// same lifetime.
fn lifetime_eq(a: &Option<syn::Lifetime>, b: &Option<syn::Lifetime>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.ident == y.ident,
        _ => false,
    }
}

fn token_eq(a: &syn::Type, b: &syn::Type) -> bool {
    a.to_token_stream().to_string() == b.to_token_stream().to_string()
}

/// If `ty` is `Option<Inner>` (by last path segment), return `Inner`.
pub fn option_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    generic_inner(ty, "Option")
}

/// If `ty` is `Vec<Inner>` (by last path segment), return `Inner`.
pub fn vec_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    generic_inner(ty, "Vec")
}

fn generic_inner(ty: &syn::Type, wrapper: &str) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != wrapper {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    match ab.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    }
}

/// Last path-segment ident of a path type, **generics permitting**
/// (`Option<T>` → `Option`, `Vec<u8>` → `Vec`). Contrast with
/// [`bare_path_ident`], which is `None` for any generic/non-path shape.
pub fn path_tail_ident(ty: &syn::Type) -> Option<syn::Ident> {
    match ty {
        syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.clone()),
        _ => None,
    }
}

/// True when `ty`'s last path segment is `name` (`path_tail_is(ty, "Vec")`).
fn path_tail_is(ty: &syn::Type, name: &str) -> bool {
    path_tail_ident(ty).is_some_and(|i| i == name)
}

/// True when `ty` is `Option<…>` (by last path segment).
pub fn is_option_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Option")
}

/// True when `ty` is `Vec<…>` (by last path segment).
pub fn is_vec_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Vec")
}

/// True when `ty` is `Result<…>` (by last path segment).
pub fn is_result_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Result")
}

/// True when `ty` is the unit type `()`.
pub fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// If `ty` is `Result<T, E>` (by last path segment), return `(T, E)`.
pub fn result_parts(ty: &syn::Type) -> Option<(syn::Type, syn::Type)> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let mut args = ab.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    });
    let ok = args.next()?;
    let err = args.next()?;
    Some((ok, err))
}

/// If `ty` is `Result<T, E>`, return `T`.
pub fn result_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    result_parts(ty).map(|(ok, _)| ok)
}

/// If `ty` is `Result<T, E>`, return `E`.
pub fn result_err_type(ty: &syn::Type) -> Option<syn::Type> {
    result_parts(ty).map(|(_, err)| err)
}

/// First angle-bracketed **type** argument of a path type (`T` of `Option<T>`
/// / `Vec<T>` / `Result<T, _>`), skipping lifetime/const args. `None` when
/// there is no type argument.
pub fn first_type_arg(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    })
}

/// True when `ty` is `Option<&T>` / `Option<&mut T>`.
pub fn is_option_ref(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some_and(|inner| matches!(inner, syn::Type::Reference(_)))
}

/// The bare ident of a plain path type (`ZThing` → `ZThing`); `None` for
/// references, generics, or multi-shape types.
pub fn bare_path_ident(ty: &syn::Type) -> Option<syn::Ident> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    Some(seg.ident.clone())
}

/// Strip any nesting of `&` / `Option<…>` / `Vec<…>` layers down to the core
/// type (`Option<&Vec<ZThing>>` → `ZThing`).
pub fn peel_ref_option_vec(ty: &syn::Type) -> syn::Type {
    let mut t = ty.clone();
    loop {
        if let syn::Type::Reference(r) = &t {
            t = (*r.elem).clone();
            continue;
        }
        if let Some(inner) = option_inner_type(&t).or_else(|| vec_inner_type(&t)) {
            t = inner;
            continue;
        }
        return t;
    }
}

/// Build an identifier at call-site span.
pub(crate) fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, Span::call_site())
}

#[cfg(test)]
mod tests;
