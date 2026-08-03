//! Shared `syn::Type` shape utilities — the Option/Vec/reference peelers and
//! short-name helpers every pipeline stage needs. One definition here
//! replaces the per-module copies that used to live in `core::unfold`,
//! `core::expand`, and the jnigen adapter.

use proc_macro2::Span;

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
#[cfg(feature = "unstable-cbindgen")]
pub fn is_vec_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Vec")
}

/// True when `ty` is `Result<…>` (by last path segment).
#[cfg(feature = "unstable-cbindgen")]
pub fn is_result_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Result")
}

/// True when `ty` is the unit type `()`.
#[cfg(feature = "unstable-cbindgen")]
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
#[cfg(feature = "unstable-cbindgen")]
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

// `is_option_ref` lived here — `option_inner_type(ty)` then a `Type::Reference`
// match — and decided how a handle parameter locks. Both halves read the
// spelling, so an optional borrow behind an erased wrapper answered `false`.
// The reading answers it instead: `TypeRef::optional_inner().borrow_target()`
// (#273, #275).

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

/// Convert a `PascalCase` / `camelCase` identifier to `snake_case`
/// (`ZKeyExpr` → `z_key_expr`). The single implementation behind the
/// public `prebindgen::lang::snake_case` re-export, and behind cbindgen's
/// type-name mangling.
pub fn pascal_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ── Enum shape: the one definition of "is this enum C-like" ────────────

/// How a captured `enum` can cross a language boundary — the single
/// classifier both adapters consult instead of each asserting on
/// `syn::Fields` itself.
///
/// The two shapes are not two mechanisms: a [`Unit`](EnumShape::Unit) enum
/// is the degenerate sum whose every variant group is empty, so a lowering
/// written for [`Sum`](EnumShape::Sum) collapses to "just a tag" for it.
/// The distinction exists because the *declarators* differ — `enum_class!`
/// / `.enum_type()` accept only the degenerate case, and handing them a
/// payload enum is an error naming the sum declarator rather than a shape
/// assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumShape {
    /// Every variant is fieldless: the value is exactly its discriminant.
    Unit,
    /// At least one variant carries a payload.
    Sum,
}

/// Classify an enum. See [`EnumShape`].
pub fn enum_shape(e: &syn::ItemEnum) -> EnumShape {
    if e.variants
        .iter()
        .all(|v| matches!(v.fields, syn::Fields::Unit))
    {
        EnumShape::Unit
    } else {
        EnumShape::Sum
    }
}

/// The first payload-carrying variant of an enum, if any — the offender an
/// adapter names when rejecting a [`Sum`](EnumShape::Sum) where only a
/// [`Unit`](EnumShape::Unit) enum is accepted.
pub fn first_payload_variant(e: &syn::ItemEnum) -> Option<&syn::Variant> {
    e.variants
        .iter()
        .find(|v| !matches!(v.fields, syn::Fields::Unit))
}

/// Resolve each enum variant to its discriminant value following Rust's own
/// assignment rule: an explicit `= N` sets the value, an implicit variant
/// takes the previous value plus one (starting at 0).
///
/// The single source of truth for every int↔variant mapping in the
/// pipeline — the Kotlin `value(N)` constants, the generated `jint →
/// variant` decode, and the `#[repr(C)]` mirror `CbindgenBuilder` emits — keeping
/// them from drifting and removing the need for a hand-written
/// `TryFrom<i32>` on the source enum. Non-literal discriminants are
/// rejected because prebindgen cannot reliably evaluate arbitrary
/// expressions at codegen time.
///
/// This describes the **unit** enum's wire numbering. A payload enum's
/// alternatives are identified by
/// [`Alternative::index`](crate::api::core::flat::Alternative) — declaration
/// order — never by a discriminant.
pub fn enum_discriminant_values(e: &syn::ItemEnum) -> Vec<(syn::Ident, i64)> {
    let mut out = Vec::with_capacity(e.variants.len());
    let mut next: i64 = 0;
    for variant in &e.variants {
        let value = match variant.discriminant.as_ref() {
            Some((_, expr)) => extract_int_literal(expr).unwrap_or_else(|| {
                panic!(
                    "enum `{}` variant `{}` has a non-literal discriminant; use a literal integer value (e.g. `= 1`) or an implicit discriminant",
                    e.ident,
                    variant.ident
                )
            }),
            None => next,
        };
        out.push((variant.ident.clone(), value));
        next = value + 1;
    }
    out
}

/// Pull a signed integer out of a `syn::Expr` literal (`5`, `-3`, `0x07`).
/// Returns `None` for anything else (constants, paths, arithmetic).
fn extract_int_literal(expr: &syn::Expr) -> Option<i64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.base10_parse::<i64>().ok(),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => extract_int_literal(expr).map(|v| -v),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
