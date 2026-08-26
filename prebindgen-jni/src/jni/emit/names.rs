//! Converter/symbol naming and small `syn` probes over adapter-owned types.

use super::*;

/// Last-segment ident of a `TypeKey` — e.g. `"Publisher<'static>"` →
/// `"Publisher"`, `"AdvancedSubscriber<()>"` → `"AdvancedSubscriber"`. Used by
/// the structured builders ([`Declarations::ptr_class`],
/// [`Declarations::data_class`]) to derive a default Kotlin class name from
/// the Rust type-key. Panics for non-path types (e.g. closures, references) —
/// the per-kind `*_name_mangle` closures see only path-shaped
/// shorts. For verbatim Kotlin expressions on non-path types, use a
/// scalar / generic type wrapper.
pub(crate) fn rust_short_name(key: &TypeKey) -> String {
    rust_short_name_opt(key).unwrap_or_else(|| {
        panic!(
            "rust_short_name: cannot derive Kotlin name from type-key `{}` — \
             only path-shaped types are supported here",
            key.as_str()
        )
    })
}

/// Fallible variant of [`rust_short_name`] — returns `None` for
/// non-path types instead of panicking. Used by
/// [`Declarations::note_wrapper_registration`] which is called for rank-0
/// wrapper patterns including non-path shapes like `()` where there
/// is no Kotlin short name to derive.
pub(crate) fn rust_short_name_opt(key: &TypeKey) -> Option<String> {
    key.short_name()
}

/// Add the generated function lifetime to every JNI reference in an
/// intermediate type, including references nested in registry-owned tuples.
pub(crate) fn annotate_jobject_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Tuple(tuple) = ty {
        let mut tuple = tuple.clone();
        for elem in &mut tuple.elems {
            *elem = annotate_jobject_with_lifetime(elem, life);
        }
        return syn::Type::Tuple(tuple);
    }
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if crate::jni::wire_access::is_jni_reference_wire(ty)
                && matches!(last.arguments, syn::PathArguments::None)
            {
                let mut new = tp.clone();
                if let Some(last) = new.path.segments.last_mut() {
                    let lt =
                        syn::Lifetime::new(&format!("'{}", life), proc_macro2::Span::call_site());
                    last.arguments =
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: syn::token::Lt::default(),
                            args: syn::punctuated::Punctuated::from_iter(std::iter::once(
                                syn::GenericArgument::Lifetime(lt),
                            )),
                            gt_token: syn::token::Gt::default(),
                        });
                }
                return syn::Type::Path(new);
            }
        }
    }
    ty.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

// `pat_match` lived here — `ty.to_token_stream().to_string() == pat` — and was
// how the converter selector decided what a type WAS: rebuild a wildcard
// pattern from the spelling, render it to a string, compare. That made the
// answer depend on how Rust happened to spell the type, so `Box<Option<T>>`
// reconstructed as `Box<_>`, matched nothing, and got no converter at all
// (#270). Dispatch reads `TypeKind` now; nothing needs it.

/// INPUT: wire → rust. Format `<wire_id>_to_<rust_id>_<hash>` (including
/// `impl Fn(...)` lambda converters — the legacy
/// `process_kotlin_<Name>_callback` naming is gone with the fun-interface
/// subsystem).
pub(crate) fn input_name(rust: &TokenStream, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", wire_id, rust_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

/// OUTPUT: rust → wire. Format `<rust_id>_to_<wire_id>_<hash>`.
pub(crate) fn output_name(rust: &TokenStream, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", rust_id, wire_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

pub(crate) fn sanitize_for_ident(s: &str) -> String {
    // Special-case the empty tuple — the all-punctuation token stream
    // would sanitize to a meaningless fallback. `unit` is recognisable.
    if s.trim() == "()" {
        return "unit".to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("ty");
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn wire_short(wire: &syn::Type) -> String {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return sanitize_for_ident(&last.ident.to_string());
        }
    }
    sanitize_for_ident(&wire.to_token_stream().to_string())
}

pub(crate) fn hash_pair(rust: &TokenStream, wire: &syn::Type) -> u64 {
    hash_name_pair(&rust.to_string(), wire)
}

/// Hash the canonical source name and intermediate type used in a converter
/// identifier. Late plans pass a `TypeKey` string so naming never requires
/// access to Rust syntax; legacy rendered converters pass their token spelling.
pub(crate) fn hash_name_pair(source: &str, wire: &syn::Type) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let mut h = DefaultHasher::new();
    source.hash(&mut h);
    "::".hash(&mut h);
    wire.to_token_stream().to_string().hash(&mut h);
    h.finish()
}
