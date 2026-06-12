//! Shared `syn::Type` shape utilities — the Option/Vec/reference peelers and
//! short-name helpers every pipeline stage needs. One definition here
//! replaces the per-module copies that used to live in `core::unfold`,
//! `core::expand`, and the jnigen back-end.

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

/// True when `ty` is `Option<…>` (by last path segment).
pub fn is_option_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(tp)
        if tp.path.segments.last().is_some_and(|s| s.ident == "Option"))
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

/// Short (last-segment) name of a type, peeled of a leading `&` —
/// `&zenoh_flat::ZSample` → `"ZSample"`. Empty for non-path shapes.
pub fn short_type_name(ty: &syn::Type) -> String {
    let bare = match ty {
        syn::Type::Reference(r) => &*r.elem,
        other => other,
    };
    if let syn::Type::Path(tp) = bare {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident.to_string();
        }
    }
    String::new()
}
