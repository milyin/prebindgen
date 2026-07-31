//! Structural type-graph helpers, shared by the scan and the diagnostics BFS.

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

/// Immediate child type positions of `ty` (one level deep).
pub fn immediate_subtype_positions(ty: &syn::Type) -> Vec<syn::Type> {
    match ty {
        syn::Type::Path(p) => {
            if let Some(last) = p.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                    return ab
                        .args
                        .iter()
                        .filter_map(|a| {
                            if let syn::GenericArgument::Type(t) = a {
                                Some(t.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
            vec![]
        }
        syn::Type::Reference(r) => vec![(*r.elem).clone()],
        syn::Type::Tuple(t) => t.elems.iter().cloned().collect(),
        syn::Type::Array(a) => vec![(*a.elem).clone()],
        syn::Type::Slice(s) => vec![(*s.elem).clone()],
        syn::Type::Ptr(p) => vec![(*p.elem).clone()],
        syn::Type::Group(g) => immediate_subtype_positions(&g.elem),
        syn::Type::Paren(p) => immediate_subtype_positions(&p.elem),
        syn::Type::ImplTrait(_) => extract_fn_trait_args(ty).unwrap_or_default(),
        _ => vec![],
    }
}

/// The callback grammar, which the source language owns — re-exported here for
/// the existing call sites until they consume elements (stages L2–L4 of #229).
pub use crate::api::core::flat::extract_fn_trait_args;
