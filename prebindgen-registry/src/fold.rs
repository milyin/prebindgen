//! Leaves, derived from rows rather than stored.
//!
//! A row says what a value is made of: a shape, and for a product an operation
//! naming what assembles the parts. It does not say how many values that costs
//! on the foreign signature, what those values are called, or how absence and
//! arm choice are encoded. Those are the target's answers, and this module
//! takes them as a [`FoldPolicy`] the adapter supplies.
//!
//! What comes out is a [`FoldPlan`](crate::expand::FoldPlan) — the same plan
//! `expand::apply` builds from the older decomposition declarations, with the
//! same leaf names, order and types, so the two can be compared value for
//! value while both exist.
//!
//! Only the constructing direction lives here. The deconstructing view is the
//! other half of the same walk and arrives with the return side (#701 step 3).

use prebindgen_flat::flat::{Flat, TypeRef};

use crate::{
    expand::{FoldArg, FoldBuild, FoldLeaf, FoldPlan, FoldShape, FoldVariant},
    recipe::{
        Bindings, Construct, Crossing, Direction, Recipe, RecipeKey, RecipeName, Recipes, Shape,
    },
};

/// What a target decides when a row tree is flattened into leaves.
///
/// Every method names a leaf the model never wrote, or names one the model did
/// write. Nothing here follows from the row: a target free to spend two leaves
/// on an optional value and one on another is making its own choice, and the
/// registry's job is to walk the tree the same way whichever it makes.
pub trait FoldPolicy {
    /// The leaf saying which arm of a choice is live.
    ///
    /// Under an optional value it also carries absence, which is why no
    /// presence flag joins it there.
    fn selector(&self, prefix: &str) -> FoldLeaf;

    /// The leaf saying whether an optional value is present.
    fn presence(&self, prefix: &str) -> FoldLeaf;

    /// Name for the one part of a single-part product.
    fn sole(&self, prefix: &str) -> syn::Ident;

    /// Name for one part of a product with more than one.
    fn part(&self, prefix: &str, name: &str) -> syn::Ident;

    /// Name for the one part of a single-part arm.
    fn arm_sole(&self, prefix: &str, arm: usize) -> syn::Ident;

    /// Name for one part of an arm with more than one.
    fn arm_part(&self, prefix: &str, arm: usize, index: usize) -> syn::Ident;

    /// The type a leaf carries inside an arm, given the part's own type.
    ///
    /// An arm's parts are only live when the selector picks that arm, so a
    /// target that has no other way to say "not this arm" wraps them. One that
    /// is handed a part already optional leaves it alone — the wrapping would
    /// have nowhere to go, and absence is a legitimate value for the arm the
    /// selector did pick.
    fn arm_leaf_ty(&self, ty: &TypeRef) -> (TypeRef, bool);
}

/// Why a row tree could not be flattened.
#[derive(Debug)]
pub enum FoldError {
    /// A row named a constructor the model does not have.
    UnknownConstructor(syn::Ident),
    /// A row's shape has no leaves to give at this position.
    NotConstructible {
        /// The crossing whose row was walked.
        crossing: String,
        /// What the row said instead.
        shape: &'static str,
    },
    /// A part's row reaches the value being built.
    Cycle {
        /// The type the walk came back to.
        ty: String,
    },
}

impl std::fmt::Display for FoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoldError::UnknownConstructor(name) => {
                write!(f, "no `#[prebindgen]` function named `{name}`")
            }
            FoldError::NotConstructible { crossing, shape } => write!(
                f,
                "`{crossing}`'s row is {shape}, which builds no value from leaves"
            ),
            FoldError::Cycle { ty } => {
                write!(f, "`{ty}` is built from a part that is built from it")
            }
        }
    }
}

/// What one walk carries from step to step: the target's answers, the row name
/// every nested build is looked up under, the leaves so far, and the chain of
/// types being built, which is what catches a value built from itself.
struct Walk<'w> {
    policy: &'w dyn FoldPolicy,
    row: &'w RecipeName,
    leaves: Vec<FoldLeaf>,
    building: Vec<String>,
}

/// The tables a walk reads: the rows, the bindings that say which row a part
/// takes, and the model every signature comes from.
pub struct Folding<'a> {
    recipes: &'a Recipes,
    bindings: &'a Bindings,
    model: &'a Flat,
}

impl<'a> Folding<'a> {
    /// Read the leaves off these tables.
    pub fn new(recipes: &'a Recipes, bindings: &'a Bindings, model: &'a Flat) -> Self {
        Self {
            recipes,
            bindings,
            model,
        }
    }

    /// The leaves one parameter crosses as, and how they rebuild its value.
    ///
    /// `reading` is the parameter's own type, layers included: `Option<&T>`
    /// says both that the value may be absent and that the call borrows it.
    /// The row walked is the one declared for `T`, since the layers are the
    /// model's answer and not something a row restates.
    pub fn fold(
        &self,
        policy: &dyn FoldPolicy,
        prefix: &str,
        reading: &TypeRef,
        row: &RecipeName,
    ) -> Result<FoldPlan, FoldError> {
        let (optional, by_ref, target) = layers(reading);
        let mut walk = Walk {
            policy,
            row,
            leaves: Vec::new(),
            building: vec![target.key().to_string()],
        };

        let shape = self
            .constructing(&target, row)
            .ok_or_else(|| FoldError::NotConstructible {
                crossing: target.to_string(),
                shape: "not a row that builds a value from parts",
            })?
            .clone();
        // An optional value whose row is a single constructor spends its leaves
        // differently from one whose row is a choice: the choice's selector
        // already has a value for absence, a single constructor's leaves do
        // not. That is the policy's business, and the two ask it different
        // questions.
        if optional {
            if let Shape::Product(Construct::Call(func)) = &shape {
                return self.optional_call(&mut walk, prefix, &target, by_ref, func);
            }
        }
        let (selector, variants) = self.core(&mut walk, prefix, &target, by_ref, &shape)?;
        Ok(FoldPlan {
            target,
            by_ref,
            shape: if optional {
                FoldShape::Optional((), Box::new(FoldShape::Base))
            } else {
                FoldShape::Base
            },
            leaves: walk.leaves,
            selector,
            present: None,
            variants,
        })
    }

    /// An optional value built by one constructor.
    ///
    /// One part and the part carries absence itself; more than one and the
    /// parts stay plain, with a flag in front deciding. A target that boxed a
    /// nullable primitive for every part would pay for the second case in the
    /// common one.
    fn optional_call(
        &self,
        walk: &mut Walk<'_>,
        prefix: &str,
        target: &TypeRef,
        by_ref: bool,
        func: &syn::Ident,
    ) -> Result<FoldPlan, FoldError> {
        let params = self.params(func)?;
        let fallible = self.fallible(func)?;
        let plan = |leaves: Vec<FoldLeaf>, present, inputs| FoldPlan {
            target: target.clone(),
            by_ref,
            shape: FoldShape::Optional((), Box::new(FoldShape::Base)),
            leaves,
            selector: None,
            present,
            variants: vec![FoldVariant {
                ctor: Some(func.clone()),
                fallible,
                clone: false,
                inputs,
            }],
        };
        if let [(_, ty)] = params.as_slice() {
            walk.leaves.push(FoldLeaf {
                name: walk.policy.sole(prefix),
                ty: ty.optional(),
            });
            return Ok(plan(
                std::mem::take(&mut walk.leaves),
                None,
                vec![FoldArg::Leaf(0, false)],
            ));
        }
        walk.leaves.push(walk.policy.presence(prefix));
        let mut inputs = Vec::new();
        for (name, ty) in &params {
            let index = walk.leaves.len();
            walk.leaves.push(FoldLeaf {
                name: walk.policy.part(prefix, &name.to_string()),
                ty: ty.clone(),
            });
            inputs.push(FoldArg::Leaf(index, false));
        }
        Ok(plan(std::mem::take(&mut walk.leaves), Some(0), inputs))
    }

    /// The dispatch a row states: one unconditional constructor, or a selector
    /// and one arm per way of obtaining the value.
    fn core(
        &self,
        walk: &mut Walk<'_>,
        prefix: &str,
        target: &TypeRef,
        by_ref: bool,
        shape: &Shape<Construct>,
    ) -> Result<(Option<usize>, Vec<FoldVariant>), FoldError> {
        match shape {
            Shape::Product(Construct::Call(func)) => {
                let inputs = self.arguments(walk, prefix, func, None)?;
                Ok((
                    None,
                    vec![FoldVariant {
                        ctor: Some(func.clone()),
                        fallible: self.fallible(func)?,
                        clone: false,
                        inputs,
                    }],
                ))
            }
            Shape::Choice { arms } => {
                let selector = walk.leaves.len();
                walk.leaves.push(walk.policy.selector(prefix));
                let mut variants = Vec::new();
                for (arm, entry) in arms.iter().enumerate() {
                    variants.push(match &entry.op {
                        Construct::Call(func) => FoldVariant {
                            ctor: Some(func.clone()),
                            fallible: self.fallible(func)?,
                            clone: false,
                            inputs: self.arguments(walk, prefix, func, Some(arm))?,
                        },
                        Construct::Identity => {
                            let index = walk.leaves.len();
                            // The value itself, as one leaf. A borrowed
                            // crossing lends it, so the arm clones and the leaf
                            // carries the borrow; an owned one gives it away.
                            let ty = if by_ref {
                                target.borrowed().optional()
                            } else {
                                target.optional()
                            };
                            walk.leaves.push(FoldLeaf {
                                name: walk.policy.arm_sole(prefix, arm),
                                ty,
                            });
                            FoldVariant {
                                ctor: None,
                                fallible: false,
                                clone: by_ref,
                                inputs: vec![FoldArg::Leaf(index, false)],
                            }
                        }
                        Construct::Fields => {
                            return Err(FoldError::NotConstructible {
                                crossing: target.to_string(),
                                shape: "an arm written from the value's own fields",
                            })
                        }
                    });
                }
                Ok((Some(selector), variants))
            }
            other => Err(FoldError::NotConstructible {
                crossing: target.to_string(),
                shape: name_of(other),
            }),
        }
    }

    /// One constructor's parameters, each a leaf or a value built from leaves
    /// of its own.
    fn arguments(
        &self,
        walk: &mut Walk<'_>,
        prefix: &str,
        func: &syn::Ident,
        arm: Option<usize>,
    ) -> Result<Vec<FoldArg>, FoldError> {
        let params = self.params(func)?;
        let sole = params.len() == 1;
        let mut inputs = Vec::new();
        for (index, (name, ty)) in params.iter().enumerate() {
            let leaf_name = match (arm, sole) {
                (Some(arm), true) => walk.policy.arm_sole(prefix, arm),
                (Some(arm), false) => walk.policy.arm_part(prefix, arm, index),
                (None, true) => walk.policy.sole(prefix),
                (None, false) => walk.policy.part(prefix, &name.to_string()),
            };
            inputs.push(self.argument(walk, ty, leaf_name, arm.is_some())?);
        }
        Ok(inputs)
    }

    /// One constructor parameter.
    ///
    /// A parameter whose own type states a row that builds it from leaves is
    /// built the same way, recursively, and contributes that row's leaves in
    /// place of one of its own.
    fn argument(
        &self,
        walk: &mut Walk<'_>,
        ty: &TypeRef,
        name: syn::Ident,
        dispatched: bool,
    ) -> Result<FoldArg, FoldError> {
        let (optional, by_ref, core) = layers(ty);
        // Only outside an arm and outside an `Option`: a nested build under a
        // selector would need leaves that are live only when two things hold at
        // once, and one under an `Option` would need a second absence.
        if !dispatched && !optional {
            if let Some(shape) = self.constructing(&core, walk.row).cloned() {
                if builds_from_leaves(&shape) {
                    let key = core.key().to_string();
                    if walk.building.contains(&key) {
                        return Err(FoldError::Cycle { ty: key });
                    }
                    walk.building.push(key);
                    let (selector, variants) =
                        self.core(walk, &name.to_string(), &core, by_ref, &shape)?;
                    walk.building.pop();
                    return Ok(FoldArg::Build(Box::new(FoldBuild {
                        target: core,
                        by_ref,
                        selector,
                        variants,
                    })));
                }
            }
        }
        let index = walk.leaves.len();
        let (leaf_ty, passthrough) = if dispatched {
            walk.policy.arm_leaf_ty(ty)
        } else {
            (ty.clone(), false)
        };
        walk.leaves.push(FoldLeaf { name, ty: leaf_ty });
        Ok(FoldArg::Leaf(index, passthrough))
    }

    /// This crossing's row under `row`, if it states one.
    ///
    /// By NAME rather than by default, and that is the whole reason an identity
    /// arm works: the arm's part is the value itself, which takes the
    /// crossing's default row — its own conversion. Were the constructor row
    /// the default, the arm would be a part of the row it belongs to, which is
    /// the cycle `Recipes::build` reports.
    fn constructing(&self, ty: &TypeRef, row: &RecipeName) -> Option<&Shape<Construct>> {
        let crossing = Crossing::new(ty.clone(), Direction::Construct).key();
        let key = self.recipes.key_of(&crossing, row)?;
        match self.recipes.get(key)? {
            Recipe::Constructing(shape) => Some(shape),
            Recipe::Deconstructing(_) => None,
        }
    }

    fn params(&self, func: &syn::Ident) -> Result<Vec<(syn::Ident, TypeRef)>, FoldError> {
        let f = self
            .model
            .function(func)
            .ok_or_else(|| FoldError::UnknownConstructor(func.clone()))?;
        Ok(f.params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect())
    }

    fn fallible(&self, func: &syn::Ident) -> Result<bool, FoldError> {
        let f = self
            .model
            .function(func)
            .ok_or_else(|| FoldError::UnknownConstructor(func.clone()))?;
        Ok(f.ret.fallible_parts().is_some())
    }

    /// Kept so a later step can resolve a part through its binding rather than
    /// through the crossing's default row (#701 step 3).
    #[allow(dead_code)]
    fn bindings(&self) -> &Bindings {
        self.bindings
    }

    /// Likewise, for a row named rather than defaulted.
    #[allow(dead_code)]
    fn row(&self, key: &RecipeKey) -> Option<&Recipe> {
        self.recipes.get(key)
    }
}

/// Whether a row builds its value out of leaves rather than crossing whole.
fn builds_from_leaves(shape: &Shape<Construct>) -> bool {
    matches!(
        shape,
        Shape::Product(Construct::Call(_)) | Shape::Choice { .. }
    )
}

/// The boundary layers down to the value a row builds, and which were there.
fn layers(reading: &TypeRef) -> (bool, bool, TypeRef) {
    let optional = reading.optional_inner().is_some();
    let after_opt = reading.optional_inner().unwrap_or(reading);
    let by_ref = after_opt.borrow_target().is_some();
    let core = after_opt.borrow_target().unwrap_or(after_opt);
    (optional, by_ref, core.clone())
}

fn name_of(shape: &Shape<Construct>) -> &'static str {
    match shape {
        Shape::Atomic => "atomic",
        Shape::Optional => "optional",
        Shape::Sequence => "a sequence",
        Shape::Invoke => "a callable",
        Shape::Product(_) => "a product written from fields",
        Shape::Choice { .. } => "a choice",
    }
}

#[cfg(test)]
mod tests;
