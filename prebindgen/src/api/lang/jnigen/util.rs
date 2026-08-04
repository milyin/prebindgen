//! Shared internal utilities used by multiple modules.

/// Convert a `snake_case` Rust identifier name to `camelCase`.
pub(crate) fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else if i == 0 {
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Convert a `CamelCase` Rust identifier to `SCREAMING_SNAKE_CASE`. Used to
/// project Rust enum variant idents into Kotlin enum constant names.
pub(crate) fn camel_to_screaming_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_uppercase());
    }
    out
}

/// True iff `ty` is the unit type `()`.
pub(crate) fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

#[cfg(test)]
mod tests;

/// A type's **head**: what the source spelled outermost, once `&`, `Option` and
/// `Vec` are peeled off.
///
/// The two naming helpers below are what jnigen calls a type in Kotlin — an
/// interface name, a lambda parameter — and both used to peel by matching
/// `syn::Type` and then read the last path segment. That was a classifier
/// outside the model, and it is only writable now because
/// [`TypeKind`](crate::api::core::flat::TypeKind) **is** the accepted syntax:
/// asking the kind for the head is asking the same question of the same
/// grammar, with the model answering instead of a `match` on `syn`.
///
/// Peels exactly what `types_util::peel_ref_option_vec` peeled — `&`, `Option`,
/// `Vec`, in any order, to a fixed point — and **not** a transparent wrapper:
/// `Box<Vec<T>>` stops at the `Box`, as it did before. The layer accessors
/// (`borrow_target`, `optional_inner`, `sequence_elem`) see through `Box`/`Cow`
/// by design, so this reads `kind()` directly rather than through them.
pub(crate) fn head_type(t: &crate::api::core::flat::TypeRef) -> &crate::api::core::flat::TypeRef {
    use crate::api::core::flat::TypeKind;
    match t.kind() {
        TypeKind::Ref { inner, .. } | TypeKind::Optional(inner) | TypeKind::Vec(inner) => {
            head_type(inner)
        }
        _ => t,
    }
}

/// The head's name as the source spelled it — the last path segment of what a
/// `syn::Type::Path` match would have found, and `None` for a form that is not
/// a path at all (a slice, an array, a borrow, a callback, the unit).
///
/// One arm per accepted form, and that is the point: a kind that is a path in
/// Rust has a name here, and one that is not, has none. The builtins answer
/// with the name Rust writes, because that is what the last segment of their
/// spelling was.
pub(crate) fn head_name(t: &crate::api::core::flat::TypeRef) -> Option<String> {
    use crate::api::core::flat::TypeKind;
    Some(match t.kind() {
        TypeKind::Named { id, .. } => id
            .name
            .rsplit("::")
            .next()
            .expect("a name has a last segment")
            .to_string(),
        TypeKind::Scalar(s) => s.as_str().to_string(),
        TypeKind::Str => "str".to_string(),
        TypeKind::String => "String".to_string(),
        TypeKind::Optional(_) => "Option".to_string(),
        TypeKind::Vec(_) => "Vec".to_string(),
        TypeKind::Boxed(_) => "Box".to_string(),
        TypeKind::Cow { .. } => "Cow".to_string(),
        TypeKind::Uninit(_) => "MaybeUninit".to_string(),
        TypeKind::Fallible { .. } => "Result".to_string(),
        TypeKind::Ref { .. }
        | TypeKind::Slice(_)
        | TypeKind::Array { .. }
        | TypeKind::Callback { .. }
        | TypeKind::Unit => return None,
    })
}
