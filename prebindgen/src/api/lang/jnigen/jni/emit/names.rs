//! Converter/symbol naming, path qualification of emitted types, and
//! small `syn` type probes.

use super::*;

/// Last-segment ident of a `TypeKey` — e.g. `"Publisher<'static>"` →
/// `"Publisher"`, `"AdvancedSubscriber<()>"` → `"AdvancedSubscriber"`. Used by
/// the structured builders ([`JniGen::ptr_class`],
/// [`JniGen::data_class`]) to derive a default Kotlin class name from
/// the Rust type-key. Panics for non-path types (e.g. closures, references) —
/// the per-kind `kotlin_*_name_mangle` closures see only path-shaped
/// shorts. For verbatim Kotlin expressions on non-path types, chain
/// [`JniGen::kotlin_type`] after the structured builder.
pub(crate) fn rust_short_name(key: &TypeKey) -> String {
    rust_short_name_opt(key).unwrap_or_else(|| {
        panic!(
            "rust_short_name: cannot derive Kotlin name from type-key `{}` — \
             only path-shaped types are supported here; use \
             `kotlin_type(\"<verbatim>\")` to set the name explicitly",
            key.as_str()
        )
    })
}

/// Fallible variant of [`rust_short_name`] — returns `None` for
/// non-path types instead of panicking. Used by
/// [`JniGen::note_wrapper_registration`] which is called for rank-0
/// wrapper patterns including non-path shapes like `()` where there
/// is no Kotlin short name to derive.
pub(crate) fn rust_short_name_opt(key: &TypeKey) -> Option<String> {
    let ty = key.to_type();
    if let syn::Type::Path(tp) = &ty {
        if let Some(last) = tp.path.segments.last() {
            return Some(last.ident.to_string());
        }
    }
    None
}

/// `VisitMut` that prefixes every bare single-segment `Type::Path` whose
/// ident lives in `source_names` with `source_module`. Walks the full
/// AST — function signatures, generic args, type ascriptions, casts,
/// turbofish — so any emitted item passes through one universal pass
/// instead of each emit site having to remember to qualify.
pub(crate) struct QualifyEmittedTypes<'a> {
    pub(crate) source_module: &'a syn::Path,
    pub(crate) source_names: &'a std::collections::HashSet<String>,
}

impl syn::visit_mut::VisitMut for QualifyEmittedTypes<'_> {
    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        if tp.qself.is_none() && tp.path.leading_colon.is_none() && tp.path.segments.len() == 1 {
            let ident = tp.path.segments[0].ident.to_string();
            if self.source_names.contains(&ident) {
                let mut qualified = self.source_module.clone();
                qualified.segments.push(tp.path.segments[0].clone());
                tp.path = qualified;
            }
        }
        syn::visit_mut::visit_type_path_mut(self, tp);
    }
}

pub(crate) fn mangle_jni_name(ext: &JniGen, ident: &syn::Ident) -> syn::Ident {
    let camel = snake_to_camel(&ident.to_string());
    let mangled = ext.mangle_fun(&camel);
    let mut name = ext.jni_class_path.clone();
    name.push('_');
    name.push_str(&mangled);
    syn::Ident::new(&name, Span::call_site())
}

/// If `ty` is a `&T` borrow with no explicit lifetime, splice in `'<life>`.
/// Otherwise return `ty` unchanged.
pub(crate) fn annotate_borrow_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Reference(r) = ty {
        if r.lifetime.is_none() {
            let mut new = r.clone();
            new.lifetime = Some(syn::Lifetime::new(
                &format!("'{}", life),
                proc_macro2::Span::call_site(),
            ));
            return syn::Type::Reference(new);
        }
    }
    ty.clone()
}

/// If `ty` is `JObject` / `JString` / `JByteArray` (no explicit angle args),
/// splice in `<'<life>>`. Otherwise return `ty` unchanged.
pub(crate) fn annotate_jobject_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            let name = last.ident.to_string();
            if matches!(
                name.as_str(),
                "JObject" | "JString" | "JByteArray" | "JClass"
            ) && matches!(last.arguments, syn::PathArguments::None)
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

pub(crate) fn pat_match(ty: &syn::Type, pat: &str) -> bool {
    ty.to_token_stream().to_string() == pat
}

/// `true` if `ty` is a path whose final segment is `name` (e.g. `Vec<_>` for
/// `name = "Vec"`, `Option<&T>` for `name = "Option"`). Ignores generic args.
pub(crate) fn pat_match_top(ty: &syn::Type, name: &str) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return last.ident == name;
        }
    }
    false
}

/// If `ty` is `Option<&T>` or `Option<&mut T>`, return `Some(is_mut)`.
/// Returns `None` for any other shape. Used by `emit_jni_function_wrapper`
/// to decide whether the call site needs `.as_deref()` / `.as_deref_mut()`
/// when the input converter produced `Option<OwnedObject<T>>`.
pub(crate) fn option_inner_ref_mutability(ty: &syn::Type) -> Option<bool> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(inner) = ab.args.first()? else {
        return None;
    };
    let syn::Type::Reference(r) = inner else {
        return None;
    };
    Some(r.mutability.is_some())
}

/// Inline-class field name for a value projection identified by its folded
/// [`Projection::leaf_key`] (e.g. `"ZZenohId"`) rather than by a raw param type.
/// Used for `Option<value-blob>` params where the written type isn't the bare
/// value class but the projection still resolves the leaf — so the wrapper
/// knows which inline field to unwrap (`<name>.bytes`).
pub(crate) fn value_projection_field_for_leaf(
    ext: &JniGen,
    leaf_key: &str,
) -> Option<String> {
    let key = TypeKey::parse(leaf_key);
    let cfg = ext.types.get(&key)?;
    if cfg.value_blob {
        return Some("bytes".to_string());
    }
    None
}

/// INPUT: wire → rust. Format `<wire_id>_to_<rust_id>_<hash>` (including
/// `impl Fn(...)` lambda converters — the legacy
/// `process_kotlin_<Name>_callback` naming is gone with the fun-interface
/// subsystem).
pub(crate) fn input_name(rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_token_stream().to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", wire_id, rust_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

/// OUTPUT: rust → wire. Format `<rust_id>_to_<wire_id>_<hash>`.
pub(crate) fn output_name(rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_token_stream().to_string());
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

pub(crate) fn hash_pair(rust: &syn::Type, wire: &syn::Type) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let mut h = DefaultHasher::new();
    rust.to_token_stream().to_string().hash(&mut h);
    "::".hash(&mut h);
    wire.to_token_stream().to_string().hash(&mut h);
    h.finish()
}
