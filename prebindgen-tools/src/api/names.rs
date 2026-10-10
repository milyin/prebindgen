use prebindgen_flat::flat::{TypeKind, TypeRef};
use proc_macro2::Span;

/// An identifier from a name, accepting raw spellings (`r#type`) and
/// escaping a Rust keyword into its raw form, so a name taken from any
/// source can always be turned into an ident.
pub fn ident(name: &str) -> syn::Ident {
    if let Some(raw) = name.strip_prefix("r#") {
        return syn::Ident::new_raw(raw, Span::call_site());
    }
    syn::parse_str::<syn::Ident>(name)
        .unwrap_or_else(|_| syn::Ident::new_raw(name, Span::call_site()))
}

/// The bare name of an identifier: `r#type` → `type`.
pub fn bare(id: &syn::Ident) -> String {
    let s = id.to_string();
    s.strip_prefix("r#").map(str::to_string).unwrap_or(s)
}

/// `base` joined to `suffix` with an underscore: `p` + `id` → `p_id`.
pub fn join(base: &syn::Ident, suffix: &str) -> syn::Ident {
    ident(&format!("{}_{suffix}", bare(base)))
}

/// `PascalCase` or `camelCase` → `snake_case`.
pub fn snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower =
                i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            let prev_upper = i > 0 && chars[i - 1].is_uppercase();
            if i > 0 && (prev_lower || (prev_upper && next_lower)) && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `snake_case` → `lowerCamelCase`. A name without underscores is returned
/// with its first letter lowered.
pub fn camel(s: &str) -> String {
    let p = pascal(s);
    let mut chars = p.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `snake_case` → `PascalCase`. Leading/trailing/double underscores are
/// dropped; an already-Pascal name is returned unchanged.
pub fn pascal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for part in s.split('_').filter(|p| !p.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.extend(chars);
        }
    }
    out
}

/// A readable, identifier-safe name for a type, for naming generated helpers:
/// `Option<Vec<Payload>>` → `Option_Vec_Payload`, `&mut Calculator` →
/// `mut_Calculator`, `&str` → `ref_str`, `[u8; 4]` → `array_u8_4`.
///
/// Distinct types give distinct names; lifetimes do not participate, so two
/// spellings that differ only by a lifetime share a helper.
pub fn mangle(ty: &TypeRef) -> String {
    let mut out = String::new();
    mangle_into(ty, &mut out);
    out
}

fn mangle_into(ty: &TypeRef, out: &mut String) {
    let push = |out: &mut String, s: &str| {
        if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
        out.push_str(s);
    };
    match ty.kind() {
        TypeKind::Scalar(k) => push(out, k.as_str()),
        TypeKind::Str => push(out, "str"),
        TypeKind::String => push(out, "String"),
        TypeKind::Unit => push(out, "unit"),
        TypeKind::Optional(t) => {
            push(out, "Option");
            mangle_into(t, out)
        }
        TypeKind::Vec(t) => {
            push(out, "Vec");
            mangle_into(t, out)
        }
        TypeKind::Slice(t) => {
            push(out, "slice");
            mangle_into(t, out)
        }
        TypeKind::Boxed(t) => {
            push(out, "Box");
            mangle_into(t, out)
        }
        TypeKind::Cow { inner, .. } => {
            push(out, "Cow");
            mangle_into(inner, out)
        }
        TypeKind::Out(t) => {
            push(out, "mut_Uninit");
            mangle_into(t, out)
        }
        TypeKind::Fallible { ok, err } => {
            push(out, "Result");
            mangle_into(ok, out);
            mangle_into(err, out)
        }
        TypeKind::Ref { mutable, inner, .. } => {
            push(out, if *mutable { "mut" } else { "ref" });
            mangle_into(inner, out)
        }
        TypeKind::Array { elem, extent } => {
            push(out, "array");
            mangle_into(elem, out);
            push(out, &extent.value.to_string())
        }
        TypeKind::Named { id, args } => {
            push(out, &id.name.replace("::", "_"));
            for a in args {
                mangle_into(a, out)
            }
        }
        TypeKind::Callback { args } => {
            push(out, "Fn");
            for a in args {
                mangle_into(a, out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cases() {
        assert_eq!(snake("PayloadVecHandler"), "payload_vec_handler");
        assert_eq!(snake("HTTPServer"), "http_server");
        assert_eq!(snake("already_snake"), "already_snake");
        assert_eq!(camel("storage_put_by_take"), "storagePutByTake");
        assert_eq!(camel("Storage"), "storage");
        assert_eq!(pascal("payload_vec"), "PayloadVec");
        assert_eq!(pascal("Payload"), "Payload");
        assert_eq!(ident("type").to_string(), "r#type");
        assert_eq!(bare(&ident("r#type")), "type");
        assert_eq!(join(&ident("p"), "id").to_string(), "p_id");
    }
}
