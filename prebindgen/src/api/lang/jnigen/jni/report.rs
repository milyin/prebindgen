//! `Generation::<JniGen>::report()` — the resolved binding surface,
//! explained.
//!
//! Declarations act at a distance: one `expand_return!` / `convert!` /
//! error decl reshapes every function touching its type, and the effect
//! set is computed inside `resolve`. The report is the missing *explain*
//! mode: for each declared function its FINAL Kotlin signature (rendered
//! through the same [`render_wrapper_fn`] the emitters use, so it cannot
//! drift from the real output) plus the plans that shaped it, and for
//! each type its kind / Kotlin FQN / wire. Consumers write it next to the
//! committed regen so a decl's effect is reviewable in a PR without
//! reading generated Kotlin.
//!
//! The report is deliberately an inherent method of `Generation<JniGen>`
//! (the `write_kotlin` seam): the *pattern* — describe your resolved
//! surface — is adapter-universal, but the *content* is intrinsically in
//! the destination language's vocabulary, so each adapter implements its
//! own.

use super::*;

impl crate::api::core::Generation<JniGen> {
    /// Render the resolved binding surface as a deterministic markdown
    /// report: per package / class the final Kotlin signature of every
    /// wrapper (exactly as generated) with the expand/error plans that
    /// shaped it, then the type table (kind, Kotlin FQN, wire, conversion
    /// sources). Pure read over the resolved registry.
    pub fn report(&self) -> String {
        let ext = self.adapter();
        let registry = self.registry();
        let mut out = String::new();
        out.push_str("# JniGen binding report\n\n");
        out.push_str(&format!(
            "Base package: `{}`\n",
            if ext.package.is_empty() {
                "(none)"
            } else {
                &ext.package
            }
        ));

        // ── Packages: free functions + constants ─────────────────────────
        for (subpackage, pkg_cfg) in &ext.packages {
            let has_consts = !pkg_cfg.constants.is_empty()
                || !pkg_cfg.constant_functions.is_empty()
                || !pkg_cfg.constant_exprs.is_empty();
            if pkg_cfg.functions.is_empty() && !has_consts {
                continue;
            }
            let full = ext.package_name(subpackage);
            out.push_str(&format!("\n## package `{full}`\n\n"));
            let mut entries: Vec<&FunctionEntry> = pkg_cfg.functions.iter().collect();
            entries.sort_by_key(|m| m.rust_ident.to_string());
            for m in entries {
                let name = ext.effective_function_name(subpackage, m);
                self.report_fn(&mut out, &m.rust_ident, Some(&name), None);
            }
            let mut consts: Vec<String> = pkg_cfg
                .constants
                .iter()
                .map(|c| {
                    format!(
                        "- `val {}` — `#[prebindgen]` const `{}`\n",
                        c.kotlin_name_override
                            .clone()
                            .unwrap_or_else(|| c.rust_ident.to_string()),
                        c.rust_ident
                    )
                })
                .chain(pkg_cfg.constant_functions.iter().map(|c| {
                    format!(
                        "- `val {}` — nullary `#[prebindgen]` fn `{}`\n",
                        c.kotlin_name_override
                            .clone()
                            .unwrap_or_else(|| c.rust_ident.to_string()),
                        c.rust_ident
                    )
                }))
                .chain(pkg_cfg.constant_exprs.iter().map(|e| {
                    format!(
                        "- `val {}: {}` — binding expression\n",
                        e.kotlin_name,
                        e.ty.to_token_stream()
                    )
                }))
                .collect();
            consts.sort();
            for c in consts {
                out.push_str(&c);
            }
        }

        // ── Classes: members ──────────────────────────────────────────────
        let mut class_keys: Vec<&TypeKey> = ext.class_members.keys().collect();
        class_keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in class_keys {
            let members = &ext.class_members[key];
            if members.is_empty() {
                continue;
            }
            let fqn = ext
                .kotlin_fqn(key)
                .unwrap_or_else(|| key.as_str().to_string());
            out.push_str(&format!(
                "\n## class `{fqn}` ({}, Rust `{}`)\n\n",
                ext.class_kind_name(key),
                key.as_str()
            ));
            let mut ms: Vec<&ClassMember> = members.iter().collect();
            ms.sort_by_key(|m| m.rust_ident.to_string());
            for m in ms {
                let name = ext.effective_method_name(key, m);
                let receiver = match m.kind {
                    MemberKind::Method => Some(key),
                    MemberKind::Constructor => None,
                };
                self.report_fn(&mut out, &m.rust_ident, Some(&name), receiver);
            }
        }

        // ── Types ────────────────────────────────────────────────────────
        out.push_str("\n## types\n\n");
        let mut type_keys: Vec<&TypeKey> = ext.types.keys().collect();
        type_keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in type_keys {
            let cfg = &ext.types[key];
            if cfg.name_spec.is_none() {
                continue;
            }
            let fqn = ext
                .kotlin_fqn(key)
                .unwrap_or_else(|| key.as_str().to_string());
            let wire = registry
                .output_entry(&key.to_type())
                .map(|e| e.wire_type().to_token_stream().to_string())
                .unwrap_or_else(|| "?".to_string());
            out.push_str(&format!(
                "- `{}`: {} → `{fqn}` (wire `{wire}`)\n",
                key.as_str(),
                ext.class_kind_name(key),
            ));
        }
        // Conversion sources (`convert!` decls) — the canonical single-value
        // aspect, reported once per type rather than per function.
        let mut convs: Vec<String> = ext
            .convert_decls
            .iter()
            .map(|d| format!("- `convert!({})`{}\n", d.key.as_str(), d.describe_sources()))
            .collect();
        convs.sort();
        if !convs.is_empty() {
            out.push_str("\n## conversions\n\n");
            for c in convs {
                out.push_str(&c);
            }
        }
        // Rust-side-only boundary types (expand decls on undeclared types).
        let mut boundary: Vec<String> = crate::api::core::Prebindgen::boundary_only_types(ext)
            .into_iter()
            .map(|k| format!("- `{}` (never materializes in Kotlin)\n", k.as_str()))
            .collect();
        boundary.sort();
        if !boundary.is_empty() {
            out.push_str("\n## rust-side-only types\n\n");
            for b in boundary {
                out.push_str(&b);
            }
        }
        out
    }

    /// One function entry: the final Kotlin signature (same render path as
    /// the emitters) + the plans that shaped it.
    fn report_fn(
        &self,
        out: &mut String,
        rust_ident: &syn::Ident,
        kotlin_name: Option<&str>,
        receiver_key: Option<&TypeKey>,
    ) {
        let ext = self.adapter();
        let registry = self.registry();
        let Some((item_fn, _)) = registry.functions.get(rust_ident) else {
            return;
        };
        let Some(f) = render_wrapper_fn(ext, item_fn, registry, kotlin_name, receiver_key) else {
            return;
        };
        out.push_str(&format!("- `{}` — `{}`\n", rust_ident, signature(&f)));

        // Param expansions.
        let mut shaped: Vec<String> = Vec::new();
        let mut plans: Vec<(&syn::Ident, &crate::api::core::expand::FoldPlan)> = registry
            .expansion_plans
            .iter()
            .filter(|((func, _), _)| func == rust_ident)
            .map(|((_, param), plan)| (param, plan))
            .collect();
        plans.sort_by_key(|(p, _)| p.to_string());
        for (param, plan) in plans {
            let variants: Vec<String> = plan
                .variants
                .iter()
                .map(|v| match &v.ctor {
                    Some(c) => c.to_string(),
                    None => "self".to_string(),
                })
                .collect();
            shaped.push(format!(
                "param `{param}` expanded from `{}` — variants [{}]",
                plan.target.to_token_stream(),
                variants.join(", ")
            ));
        }
        if let Some(plan) = registry.unfold_plans.get(rust_ident) {
            let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
            shaped.push(format!(
                "return `{}` decomposed → [{}] ({:?} delivery)",
                plan.source.to_token_stream(),
                leaves.join(", "),
                plan.delivery
            ));
        }
        if let Some(plan) = registry.error_plans.get(rust_ident) {
            let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
            shaped.push(format!(
                "domain error `{}` decomposed → onError [{}] (binding failures → onBindingError)",
                plan.source.to_token_stream(),
                leaves.join(", ")
            ));
        }
        for s in shaped {
            out.push_str(&format!("  - shaped by: {s}\n"));
        }
    }
}

impl JniGen {
    /// Human-readable class-kind name of a declared type (report use).
    pub(crate) fn class_kind_name(&self, key: &TypeKey) -> &'static str {
        let Some(cfg) = self.types.get(key) else {
            return "undeclared";
        };
        if cfg.opaque.is_some() {
            "ptr_class"
        } else if cfg.enum_cfg.is_some() {
            "enum_class"
        } else if cfg.value_blob {
            "value_class"
        } else if cfg.class_decl {
            "data_class"
        } else {
            "wrapper"
        }
    }
}

/// `fun <generics> name(params): ret` off the public [`kt::KtFun`] fields —
/// the signature only, no body/annotations (they are emission detail).
fn signature(f: &kt::KtFun) -> String {
    // The surface carries full-FQN types; render them through a throwaway
    // `ImportSet` so the report shows short names (the imports are discarded).
    let mut imports = kt::ImportSet::new("");
    let generics = if f.generics.is_empty() {
        String::new()
    } else {
        format!("<{}> ", f.generics.join(", "))
    };
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.ty.render(&mut imports)))
        .collect();
    let ret = match &f.ret {
        Some(t) => format!(": {}", t.render(&mut imports)),
        None => String::new(),
    };
    format!("fun {generics}{}({}){ret}", f.name, params.join(", "))
}
