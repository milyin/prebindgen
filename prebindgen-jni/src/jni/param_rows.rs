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
//! This is where the parameter side is decided. The plans the emitters read
//! come from here, built before there is a registry, and the readings they
//! deliver are handed over beside them (#701 step 2).

use prebindgen_registry::{
    expand::{FoldLeaf, FoldPlan},
    flat::{Flat, ScalarKind, TypeRef},
    fold::{FoldPolicy, Folding},
    recipe::{Arm, Construct, Constructing, Direction, RecipeName, Recipes, Shape},
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
/// A row states none of this: what a selector and a presence flag are called
/// and typed, how a leaf is named under a product, an arm or a nested build,
/// and what a part's type becomes inside an arm are all the target's answers.
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

    fn presence_leaf(&self, parts: usize) -> bool {
        // One part carries absence itself. Past one it cannot, and a flag in
        // front is cheaper than boxing a nullable primitive per part — an
        // `Option<i32>` argument would arrive as an `Integer?`.
        parts > 1
    }

    fn identity_leaf_ty(&self, ty: &TypeRef, borrowed: bool) -> TypeRef {
        // Optional because the selector decides whether this arm is live; a
        // borrowed crossing lends the value, so the leaf carries the borrow and
        // the arm clones out of it.
        if borrowed {
            ty.borrowed().optional()
        } else {
            ty.optional()
        }
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
            assert!(
                self.is_class_declared(decl.key())
                    || !decl
                        .variants()
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "expand_param!({k}).variant_self(): `{k}` has no class declaration, so there is \
                 no Kotlin object to pass — drop .variant_self() (the type is rust-side-only) \
                 or declare the type in a package",
                k = decl.key().as_str()
            );
            if !states_a_row(decl.variants()) {
                continue;
            }
            if let Some(ty) = reading(decl.key()) {
                rows.push((ty, type_row(), row_of(decl.variants())));
            }
        }
        for (func, param, decl) in &self.fn_param_expands {
            assert!(
                self.is_class_declared(decl.key())
                    || !decl
                        .variants()
                        .iter()
                        .any(|v| matches!(v, LocalVariant::SelfIdentity)),
                "fun!({func}).expand_param(\"{param}\", expand_param!({k}).variant_self()): `{k}` \
                 has no class declaration, so there is no Kotlin object to pass — drop \
                 .variant_self() (the type is rust-side-only) or declare the type in a package",
                k = decl.key().as_str()
            );
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

    /// Which parameters are built from leaves, derived from the declarations
    /// alone.
    ///
    /// This is the half of the parameter side a row cannot state: a row says
    /// how a type is built, and this says where that happens. A per-function `.expand_param(...)`
    /// names its own position. A type-level `expand_param!` applies to every
    /// parameter of that type in every exported function — except an accessor,
    /// which is not a composer; except the receiver a method is called on,
    /// which binds to `this`; and except a position whose own declaration
    /// asked for the plain value instead.
    pub(crate) fn expanded_positions(
        &self,
        model: &Flat,
        exports: &std::collections::HashSet<syn::Ident>,
        accessors: &std::collections::HashSet<syn::Ident>,
    ) -> std::collections::BTreeSet<(String, String)> {
        // The value a declaration is about, under the layers a parameter may
        // wear: `Option<&T>` is a way of passing a `T`.
        let core = |ty: &TypeRef| {
            let after_opt = ty.optional_inner().unwrap_or(ty);
            after_opt.borrow_target().unwrap_or(after_opt).key()
        };
        let mut positions = std::collections::BTreeSet::new();
        // A per-function declaration that asked for the plain value is not an
        // expansion, and it also stops the type-level one from applying there.
        let mut plain = std::collections::BTreeSet::new();
        for (func, param, decl) in &self.fn_param_expands {
            let position = (func.to_string(), param.clone());
            if states_a_row(decl.variants()) {
                positions.insert(position);
            } else {
                plain.insert(position);
            }
        }
        let receivers = self.method_receivers();
        for decl in &self.param_expand_decls {
            if !states_a_row(decl.variants()) {
                continue;
            }
            let target = decl.key().clone();
            let mut names: Vec<&syn::Ident> = exports.iter().collect();
            names.sort_by_key(|name| name.to_string());
            for func in names {
                if accessors.contains(func) {
                    continue;
                }
                let Some(function) = model.function(func) else {
                    continue;
                };
                let receiver = receivers.get(func);
                let mut took_receiver = false;
                for param in &function.params {
                    let bare = core(&param.ty);
                    if !took_receiver && receiver == Some(&bare) {
                        took_receiver = true;
                        continue;
                    }
                    if bare != target {
                        continue;
                    }
                    let position = (func.to_string(), param.name.to_string());
                    if plain.contains(&position) {
                        continue;
                    }
                    positions.insert(position);
                }
            }
        }
        positions
    }

    /// What a per-function `.expand_param(...)` says about a position, checked
    /// against the position itself.
    ///
    /// Neither check belongs to a row: a row says how a TYPE is built, and
    /// these are about the parameter the declaration was written for. Naming a
    /// parameter the function does not have, or a type it does not take, is a
    /// declaration that would otherwise do nothing at all — the row would sit
    /// on a crossing nothing reaches.
    fn check_param_expands(
        &self,
        model: &Flat,
        accessors: &std::collections::HashSet<syn::Ident>,
    ) -> Result<(), String> {
        for (func, param, decl) in &self.fn_param_expands {
            // An accessor reads a value out; it never builds one. Refused for
            // every such declaration, including one asking for the plain
            // value: the answer is the same either way, and a declaration that
            // cannot mean anything is worth saying so about.
            if accessors.contains(func) {
                return Err(format!(
                    "`{func}` is an accessor, so its `{param}` is not built from leaves — an \
                     accessor reads a value out rather than composing one"
                ));
            }
            let Some(function) = model.function(func) else {
                return Err(format!(
                    "`{func}` has a parameter expansion, and no `#[prebindgen]` function of \
                     that name"
                ));
            };
            let Some(found) = function.params.iter().find(|p| p.name == ident(param)) else {
                let names = function
                    .params
                    .iter()
                    .map(|p| format!("`{}`", p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "`{func}` has no parameter `{param}` to expand; it takes {names}"
                ));
            };
            let after_opt = found.ty.optional_inner().unwrap_or(&found.ty);
            let bare = after_opt.borrow_target().unwrap_or(after_opt).key();
            if bare != *decl.key() {
                return Err(format!(
                    "`{func}`'s `{param}` is a `{bare}`, and its expansion is declared for \
                     `{declared}`",
                    declared = decl.key().as_str()
                ));
            }
        }
        Ok(())
    }

    /// Every expanded parameter's plan, read off its row, and the readings
    /// those plans deliver.
    ///
    /// Read from the declarations and the model, so this can be answered before
    /// there is a registry — which is what lets the leaves be handed over as a
    /// fact rather than asked for mid-resolve. The rows go in a table of their
    /// own for the same reason: the binding's own table is built later, from a
    /// registry this runs before.
    pub(crate) fn expansion_plans(
        &self,
        model: &Flat,
        exports: &std::collections::HashSet<syn::Ident>,
        accessors: &std::collections::HashSet<syn::Ident>,
    ) -> Result<(ExpansionPlans, Vec<TypeRef>), String> {
        self.check_param_expands(model, accessors)?;
        let mut builder = Recipes::builder();
        let mut seen = std::collections::HashSet::new();
        for (ty, name, row) in self.expansion_rows(model) {
            // Nothing else declares a row here, so the value's own crossing is
            // the derived one — see `Declarations::recipes` for the same
            // arrangement beside the rows a class declares.
            if seen.insert(ty.key()) {
                builder.declare_derived_default(ty.clone(), Direction::Construct);
            }
            builder.declare(ty, name, row);
        }
        let recipes = builder.build(model).map_err(|errors| {
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let folding = Folding::new(&recipes, model);

        let mut plans = ExpansionPlans::new();
        let mut leaves = Vec::new();
        for (func, param) in self.expanded_positions(model, exports, accessors) {
            let (func, param) = (ident(&func), ident(&param));
            let reading = model
                .function(&func)
                .and_then(|f| f.params.iter().find(|p| p.name == param))
                .map(|p| p.ty.clone())
                .ok_or_else(|| format!("`{func}` has no parameter `{param}` to expand"))?;
            let row = self.row_for(&func, &param);
            let plan = folding
                .fold(&JniFold, &param.to_string(), &reading, &row, &type_row())
                .map_err(|e| format!("`{func}`'s `{param}` does not fold from its row: {e}"))?;
            // One callback per expanded parameter. A delivered value's site is
            // named by the parameter it arrived on rather than by the leaf, so
            // two callbacks here would give two positions one identity.
            if plan
                .leaves
                .iter()
                .filter(|leaf| leaf.ty.callback_args().is_some())
                .count()
                > 1
            {
                return Err(format!(
                    "`{func}`'s `{param}` is built from more than one callback, and one \
                     parameter has no way to name them apart"
                ));
            }
            leaves.extend(plan.leaves.iter().map(|leaf| leaf.ty.clone()));
            plans.insert((func, param), plan);
        }
        Ok((plans, leaves))
    }
}

/// Every expanded parameter's plan, keyed by the position it belongs to.
pub(crate) type ExpansionPlans = std::collections::HashMap<(syn::Ident, syn::Ident), FoldPlan>;
