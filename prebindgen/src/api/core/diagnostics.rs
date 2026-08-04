//! What a binding did **not** claim, reported to the build log.
//!
//! Which items a binding skipped says nothing about which conversions it needs,
//! so none of this belongs to [`Registry`](crate::api::core::registry::Registry)
//! — that derives a crossing set from what **is** declared, and an ignore has no
//! effect on that set. What ignores are for is suppressing a report: telling
//! "you meant to skip this" apart from "you forgot this".
//!
//! So the ignores live here, with the reporting, and a generator calls
//! [`warn_unclaimed`] itself.

use std::collections::HashSet;

use crate::api::core::{flat::Flat, prebindgen::NamePredicate, registry::TypeKey};

/// What a binding claimed, so everything else can be reported.
///
/// The two populations are separate on purpose. A **declared** item is claimed
/// and emitted; an **ignored** one is claimed and deliberately dropped. Both
/// silence the skip report, but only an ignore that matches nothing is itself
/// worth a warning — a declaration that matches nothing is a hard error the
/// registry raises long before this runs.
#[derive(Default)]
pub struct Claimed {
    /// Functions the binding emits, plus the helpers it only references.
    pub functions: HashSet<syn::Ident>,
    /// Types the binding emits, plus the ones that cross only through a plan.
    pub types: HashSet<TypeKey>,
    /// Consts the binding emits, or `None` when it has no const mechanism at
    /// all — then every const is re-emitted verbatim, so none is ever skipped
    /// and reporting one would be a lie.
    pub consts: Option<HashSet<syn::Ident>>,
    pub ignored_functions: HashSet<syn::Ident>,
    pub ignored_types: HashSet<TypeKey>,
    pub ignored_consts: HashSet<syn::Ident>,
    /// Bulk ignores keyed on a naming family rather than an exact ident.
    /// Kind-agnostic: prebindgen names live in one flat namespace.
    pub ignored_name_predicates: Vec<NamePredicate>,
}

impl Claimed {
    /// Whether a bulk-ignore predicate covers this name.
    ///
    /// A predicate matching nothing is silent by design — it is a filter, not a
    /// claim, and its match count varies across feature configurations.
    fn predicate_ignored(&self, name: &str) -> bool {
        !self.ignored_name_predicates.is_empty()
            && self.ignored_name_predicates.iter().any(|p| p(name))
    }
}

/// Print one `cargo:warning=` line per captured item this binding never
/// claimed, and per ignore entry that matches nothing.
pub fn warn_unclaimed(flat: &Flat, claimed: &Claimed) {
    for line in unclaimed_report(flat, claimed) {
        println!("cargo:warning={line}");
    }
}

/// The report itself, as lines — so it can be asserted on rather than scraped
/// off stdout. Sorted within each group, so a build says the same thing twice.
pub(crate) fn unclaimed_report(flat: &Flat, claimed: &Claimed) -> Vec<String> {
    let mut out = Vec::new();

    // Stale ignores: an entry naming nothing is a build.rs that has drifted
    // from its source crate.
    for ident in sorted(claimed.ignored_functions.iter().map(|i| i.to_string())) {
        if flat.function(&ident_of(&ident)).is_none() {
            out.push(format!(
                "prebindgen: ignored function `{ident}` not found among #[prebindgen] items"
            ));
        }
    }
    for key in sorted(claimed.ignored_types.iter().map(|k| k.as_str().to_owned())) {
        let named = TypeKey::parse(&key)
            .ok()
            .and_then(|k| k.ident())
            .is_some_and(|ident| flat.declared_type(&ident).is_some());
        if !named {
            out.push(format!(
                "prebindgen: ignored type `{key}` not found among #[prebindgen] items"
            ));
        }
    }
    if claimed.consts.is_some() {
        for ident in sorted(claimed.ignored_consts.iter().map(|i| i.to_string())) {
            if flat.constant(&ident_of(&ident)).is_none() {
                out.push(format!(
                    "prebindgen: ignored const `{ident}` not found among #[prebindgen] items"
                ));
            }
        }
    }

    for name in sorted(
        flat.functions()
            .map(|f| &f.name)
            .filter(|k| !claimed.functions.contains(*k) && !claimed.ignored_functions.contains(*k))
            .map(|k| k.to_string())
            .filter(|n| !claimed.predicate_ignored(n)),
    ) {
        out.push(format!(
            "prebindgen: skipping undeclared #[prebindgen] fn `{name}`"
        ));
    }

    // Struct/enum only — an alias is deliberately absent, because the message
    // names a kind an alias is not.
    for name in sorted(
        struct_enum_idents(flat)
            .filter(|i| {
                let key = TypeKey::from_ident(i);
                !claimed.types.contains(&key) && !claimed.ignored_types.contains(&key)
            })
            .map(|i| i.to_string())
            .filter(|n| !claimed.predicate_ignored(n)),
    ) {
        out.push(format!(
            "prebindgen: skipping undeclared #[prebindgen] struct/enum `{name}`"
        ));
    }

    if let Some(declared) = &claimed.consts {
        for name in sorted(
            flat.constants()
                .map(|c| &c.name)
                .filter(|k| !declared.contains(*k) && !claimed.ignored_consts.contains(*k))
                .map(|k| k.to_string())
                .filter(|n| !claimed.predicate_ignored(n)),
        ) {
            out.push(format!(
                "prebindgen: skipping undeclared #[prebindgen] const `{name}`"
            ));
        }
    }

    out
}

/// Every **struct or enum** name — either enum shape, never an alias.
fn struct_enum_idents(flat: &Flat) -> impl Iterator<Item = &syn::Ident> {
    use crate::api::core::flat::Type;
    flat.types().filter_map(|t| match t {
        Type::Struct(_) | Type::Variant(_) | Type::Enum(_) => Some(t.name()),
        Type::Extern(_) => None,
    })
}

fn sorted(it: impl Iterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = it.collect();
    v.sort();
    v
}

fn ident_of(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

#[cfg(test)]
mod tests;
