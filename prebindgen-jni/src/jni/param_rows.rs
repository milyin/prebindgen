//! The parameter side, stated as rows and read back as leaves.
//!
//! An `expand_param!` declaration says a value may be built by calling one of
//! several constructors, or handed over already built. That is a row: a product
//! for one constructor, a choice for several. This module writes those rows,
//! and answers the questions a row deliberately does not — what the leaves are
//! called, what a selector and a presence flag are — so
//! [`Folding::fold`](prebindgen_registry::fold::Folding::fold) can read a
//! `FoldPlan` back off them.
//!
//! While `expand::apply` still builds the plans the emitters use, every plan
//! read off a row is compared against the one it built and a build fails on any
//! difference (#701 step 2). The comparison goes with the older path, once
//! every declaration this binding makes has been through it.

use prebindgen_registry::{
    expand::{FoldArg, FoldLeaf, FoldPlan},
    flat::{Flat, ScalarKind, TypeRef},
    fold::{FoldPolicy, Folding},
    recipe::{Arm, Bindings, Construct, Constructing, RecipeName, Recipes, Shape},
    LocalVariant, TypeKey,
};

use super::Declarations;

/// The row every `expand_param!` for a type declares.
///
/// One name for all of them: a nested build looks up the argument's own type
/// under this name, and a type is built one way wherever it is reached.
pub(crate) fn type_row() -> RecipeName {
    RecipeName::new("expand-param")
}

/// The row one function's `.expand_param(...)` declares for one parameter.
///
/// A name of its own, because it sits on the same crossing as the type-level
/// row and says something different. A site takes this one; a nested build
/// never does.
pub(crate) fn site_row(func: &syn::Ident, param: &str) -> RecipeName {
    RecipeName::new(format!("expand-param@{func}#{param}"))
}

/// What the JVM calls the values a parameter comes apart into.
///
/// Every answer here is one `expand::apply` writes into a plan today, and the
/// comparison below is what holds them to it.
pub(crate) struct JniFold;

fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

impl FoldPolicy for JniFold {
    fn selector(&self, prefix: &str) -> FoldLeaf {
        FoldLeaf {
            name: ident(&format!("{prefix}_sel")),
            ty: TypeRef::scalar(ScalarKind::I32),
        }
    }

    fn presence(&self, prefix: &str) -> FoldLeaf {
        FoldLeaf {
            name: ident(&format!("{prefix}_present")),
            ty: TypeRef::scalar(ScalarKind::Bool),
        }
    }

    fn sole(&self, prefix: &str) -> syn::Ident {
        ident(prefix)
    }

    fn part(&self, prefix: &str, name: &str) -> syn::Ident {
        ident(&format!("{prefix}_{name}"))
    }

    fn arm_sole(&self, prefix: &str, arm: usize) -> syn::Ident {
        ident(&format!("{prefix}_{arm}"))
    }

    fn arm_part(&self, prefix: &str, arm: usize, index: usize) -> syn::Ident {
        ident(&format!("{prefix}_{arm}_{index}"))
    }

    fn arm_leaf_ty(&self, ty: &TypeRef) -> (TypeRef, bool) {
        // An argument already optional keeps its own type. The wire has no
        // second absence to spend, and `None` is a legitimate value for the arm
        // the selector picked.
        if ty.optional_inner().is_some() {
            (ty.clone(), true)
        } else {
            (ty.optional(), false)
        }
    }
}

/// One `expand_param!` variant list, as a row.
///
/// A single constructor is a product — there is nothing to choose between. Two
/// or more is a choice whose arms name no alternative: they are ways to obtain
/// the value, not alternatives of a sum.
fn row_of(variants: &[LocalVariant]) -> Constructing {
    if let [LocalVariant::Ctor(func)] = variants {
        return Shape::Product(Construct::Call(func.clone()));
    }
    Shape::Choice {
        arms: variants
            .iter()
            .map(|variant| Arm {
                alternative: None,
                op: match variant {
                    LocalVariant::Ctor(func) => Construct::Call(func.clone()),
                    LocalVariant::SelfIdentity => Construct::Identity,
                },
            })
            .collect(),
    }
}

/// An identity-only list declares the plain form — the value crosses as itself,
/// which is what happens with no declaration at all. It states no row.
fn states_a_row(variants: &[LocalVariant]) -> bool {
    !matches!(variants, [] | [LocalVariant::SelfIdentity])
}

impl Declarations {
    /// Every row an `expand_param!` declaration states, type-level and per
    /// function.
    ///
    /// Read off the declarations and the model, so this can be stated before
    /// there is a registry to ask.
    pub(crate) fn expansion_rows(&self, model: &Flat) -> Vec<(TypeRef, RecipeName, Constructing)> {
        let reading = |key: &TypeKey| {
            let ty: syn::Type = syn::parse_str(key.as_str()).ok()?;
            model.classify(&ty).ok()
        };
        let mut rows = Vec::new();
        for decl in &self.param_expand_decls {
            if !states_a_row(decl.variants()) {
                continue;
            }
            if let Some(ty) = reading(decl.key()) {
                rows.push((ty, type_row(), row_of(decl.variants())));
            }
        }
        for (func, param, decl) in &self.fn_param_expands {
            if !states_a_row(decl.variants()) {
                continue;
            }
            if let Some(ty) = reading(decl.key()) {
                rows.push((ty, site_row(func, param), row_of(decl.variants())));
            }
        }
        rows
    }

    /// The row one parameter's construction is stated under.
    ///
    /// A function that overrode this parameter states its own; everything else
    /// takes the type's.
    fn row_for(&self, func: &syn::Ident, param: &syn::Ident) -> RecipeName {
        let overridden = self.fn_param_expands.iter().any(|(f, p, decl)| {
            f == func && p == &param.to_string() && states_a_row(decl.variants())
        });
        if overridden {
            site_row(func, &param.to_string())
        } else {
            type_row()
        }
    }

    /// Read every expanded parameter's plan back off its row, and hold the
    /// older path to it.
    ///
    /// Temporary, and deleted with `expand::apply`: while both exist, every
    /// declaration this binding makes is a comparison, which is far more of the
    /// surface than a fixture written by hand reaches.
    pub(crate) fn check_expansion_parity(
        &self,
        model: &Flat,
        recipes: &Recipes,
        bindings: &Bindings,
        plans: &std::collections::HashMap<(syn::Ident, syn::Ident), FoldPlan>,
    ) -> Result<(), String> {
        let folding = Folding::new(recipes, bindings, model);
        // Sorted, so a binding with two differences reports the same one every
        // time rather than whichever the hash order reached first.
        let mut positions: Vec<_> = plans.keys().collect();
        positions.sort_by_key(|(func, param)| (func.to_string(), param.to_string()));
        for (func, param) in positions {
            let expected = &plans[&(func.clone(), param.clone())];
            let Some(reading) = model
                .function(func)
                .and_then(|f| f.params.iter().find(|p| &p.name == param))
                .map(|p| p.ty.clone())
            else {
                continue;
            };
            let row = self.row_for(func, param);
            let actual = folding
                .fold(&JniFold, &param.to_string(), &reading, &row, &type_row())
                .map_err(|e| format!("`{func}`'s `{param}` does not fold from its row: {e}"))?;
            if describe(&actual) != describe(expected) {
                return Err(format!(
                    "`{func}`'s `{param}` reads back differently from its row.\n\
                     from the row:\n{}\nfrom the declarations:\n{}",
                    describe(&actual),
                    describe(expected)
                ));
            }
        }
        Ok(())
    }
}

/// One plan, flattened far enough that two can be compared by equality.
fn describe(plan: &FoldPlan) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "target={} by_ref={} optional={} selector={:?} present={:?}",
        plan.target.key(),
        plan.by_ref,
        plan.produces_option(),
        plan.selector,
        plan.present
    );
    for (index, leaf) in plan.leaves.iter().enumerate() {
        let _ = writeln!(out, "  leaf {index}: {} : {}", leaf.name, leaf.ty.key());
    }
    for variant in &plan.variants {
        let _ = writeln!(
            out,
            "  variant ctor={:?} fallible={} clone={}",
            variant.ctor.as_ref().map(|c| c.to_string()),
            variant.fallible,
            variant.clone
        );
        for arg in &variant.inputs {
            let _ = writeln!(out, "    {}", describe_arg(arg));
        }
    }
    out
}

fn describe_arg(arg: &FoldArg) -> String {
    match arg {
        FoldArg::Leaf(index, passthrough) => format!("leaf {index} passthrough={passthrough}"),
        FoldArg::Build(build) => {
            let mut out = format!(
                "build {} by_ref={} selector={:?}",
                build.target.key(),
                build.by_ref,
                build.selector
            );
            for variant in &build.variants {
                out.push_str(&format!(
                    "\n      variant ctor={:?}",
                    variant.ctor.as_ref().map(|c| c.to_string())
                ));
                for arg in &variant.inputs {
                    out.push_str(&format!("\n        {}", describe_arg(arg)));
                }
            }
            out
        }
    }
}
