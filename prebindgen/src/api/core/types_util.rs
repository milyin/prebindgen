//! Shared `syn::Type` shape utilities — the Option/Vec/reference peelers and
//! short-name helpers every pipeline stage needs. One definition here
//! replaces the per-module copies that used to live in `core::unfold`,
//! `core::expand`, and the jnigen adapter.

use proc_macro2::Span;
use quote::ToTokens;

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
mod tests {
    use super::*;
    use quote::ToTokens;

    fn ty(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }
    fn caps(v: Option<Vec<syn::Type>>) -> Option<Vec<String>> {
        v.map(|a| a.iter().map(|t| t.to_token_stream().to_string()).collect())
    }

    #[test]
    fn match_pattern_outermost_and_deep() {
        // Outermost single wildcard.
        assert_eq!(
            caps(match_pattern(&ty("Option<u64>"), &ty("Option<_>"))),
            Some(vec!["u64".to_string()])
        );
        // Two wildcards (Result).
        assert_eq!(
            caps(match_pattern(
                &ty("Result<ZKeyExpr, ZError>"),
                &ty("Result<_, _>")
            )),
            Some(vec!["ZKeyExpr".to_string(), "ZError".to_string()])
        );
        // Deep single wildcard, intermediate level concrete (`Option<&_>`).
        assert_eq!(
            caps(match_pattern(&ty("Option<&ZKeyExpr>"), &ty("Option<&_>"))),
            Some(vec!["ZKeyExpr".to_string()])
        );
        // The shallow pattern also matches, capturing the reference whole.
        assert_eq!(
            caps(match_pattern(&ty("Option<&ZKeyExpr>"), &ty("Option<_>"))),
            Some(vec!["& ZKeyExpr".to_string()])
        );
        // `&mut _` vs `&_` mutability must agree.
        assert!(match_pattern(&ty("&mut Foo"), &ty("&_")).is_none());
        assert_eq!(
            caps(match_pattern(&ty("&mut Foo"), &ty("&mut _"))),
            Some(vec!["Foo".to_string()])
        );
        // Slice element.
        assert_eq!(
            caps(match_pattern(&ty("&[u8]"), &ty("&[_]"))),
            Some(vec!["u8".to_string()])
        );
        // Arbitrary depth (the framework never enumerated this, but a user
        // pattern can name it).
        assert_eq!(
            caps(match_pattern(
                &ty("Vec<Option<u64>>"),
                &ty("Vec<Option<_>>")
            )),
            Some(vec!["u64".to_string()])
        );
        // Head mismatch.
        assert!(match_pattern(&ty("Vec<u64>"), &ty("Option<_>")).is_none());
        // Concrete non-wildcard pattern: matches only itself, no captures.
        assert_eq!(
            caps(match_pattern(&ty("MyType"), &ty("MyType"))),
            Some(vec![])
        );
        assert!(match_pattern(&ty("Other"), &ty("MyType")).is_none());
    }

    /// Lifetimes and const-generic args are fixed pattern structure — they must
    /// match token-for-token, not be silently dropped (restores the old
    /// enumerator's exact `TypeKey` semantics).
    #[test]
    fn match_pattern_respects_lifetimes_and_const_generics() {
        // Reference lifetimes must match exactly.
        assert_eq!(
            caps(match_pattern(&ty("&'static Foo"), &ty("&'static _"))),
            Some(vec!["Foo".to_string()])
        );
        assert!(match_pattern(&ty("&'a Foo"), &ty("&'static _")).is_none());
        // A no-lifetime pattern must not match a borrow that names a lifetime.
        assert!(match_pattern(&ty("&'a Foo"), &ty("&_")).is_none());
        assert_eq!(
            caps(match_pattern(&ty("&Foo"), &ty("&_"))),
            Some(vec!["Foo".to_string()])
        );
        // A lifetime generic arg in a path is fixed structure.
        assert_eq!(
            caps(match_pattern(
                &ty("Cow<'static, _>"),
                &ty("Cow<'static, _>")
            )),
            Some(vec!["_".to_string()])
        );
        assert!(match_pattern(&ty("Cow<'a, str>"), &ty("Cow<'static, _>")).is_none());
        // Const-generic arg is fixed structure: arity must match exactly.
        assert_eq!(
            caps(match_pattern(&ty("Arr<u8, 4>"), &ty("Arr<_, 4>"))),
            Some(vec!["u8".to_string()])
        );
        assert!(match_pattern(&ty("Arr<u8, 8>"), &ty("Arr<_, 4>")).is_none());
        // Array length is fixed structure.
        assert!(match_pattern(&ty("[u8; 8]"), &ty("[_; 4]")).is_none());
        assert_eq!(
            caps(match_pattern(&ty("[u8; 4]"), &ty("[_; 4]"))),
            Some(vec!["u8".to_string()])
        );
    }

    #[test]
    fn wildcard_count_specificity() {
        assert_eq!(wildcard_count(&ty("Result<_, _>")), 2);
        assert_eq!(wildcard_count(&ty("Result<_, ConcreteErr>")), 1);
        assert_eq!(wildcard_count(&ty("Option<&_>")), 1);
        assert_eq!(wildcard_count(&ty("ZKeyExpr")), 0);
    }
}
