//! Constructor expansion — fold a value's construction into the wire
//! signature of the function that consumes it, so the foreign side builds
//! the value and calls the function in a single FFI crossing.
//!
//! A *constructor* is any `#[prebindgen]` function `f(p0, …) -> T` (or
//! `-> Result<T, E>`) that builds a target type `T`. A type's input flatten
//! (a type-level `expand_param!` `.variant*` list, or the per-fn
//! `.expand_param(param, …)` override) replaces a parameter of that type —
//! in the generated foreign signature only — with the constructor's inputs,
//! flattened. The generated wrapper decodes those inputs, runs the
//! constructor Rust-side (the **fold**), and passes the built value to the
//! underlying call.
//!
//! * **One `Ctor` variant** (no identity): the parameter becomes `f`'s
//!   parameters directly (no selector) — the plain "single" form.
//! * **Two or more variants** (or an identity arm): the parameter becomes a
//!   runtime selector (`i32`) plus one `Option`-wrapped input group per variant.
//!   The identity variant passes an already-built `T` straight through.
//!
//! Everything here is **language-agnostic**: the fold is pure Rust and the
//! per-leaf wire encode/decode is delegated to the adapter's existing
//! converters. Resolution turns the declarations into [`FoldPlan`]s (stored on
//! the registry, keyed by `(fn, param)`) and registers each leaf type as a
//! required input so the resolver produces its converter. [`emit_fold`]
//! emits the dispatch expression at the parameter-emission site.

use std::collections::HashSet;

use prebindgen_flat::types_util::ident;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    declared_target::check_declared_target,
    registry::{Registry, TypeKey},
};

mod error;
mod plan;

pub use self::{
    error::{ExpandDeclError, ExpandError},
    plan::{FoldArg, FoldBuild, FoldLeaf, FoldPlan, FoldShape, FoldVariant},
};

// ──────────────────────────────────────────────────────────────────────
// Declarations (populated by the language builder)
// ──────────────────────────────────────────────────────────────────────

/// One variant of a constructor — a selector-dispatched alternative for the
/// expanded parameter. A constructor with a single `Ctor` variant (and no
/// `Identity`) is the degenerate "single" form: applied unconditionally with no
/// selector. Two or more variants (or an `Identity` arm) get a runtime selector.
#[derive(Clone)]
pub enum Variant {
    /// Build the target by calling this constructor function.
    Ctor(syn::Ident),
    /// Pass an already-built target value straight through.
    Identity,
}

/// A type-level constructor declaration (`expand_param!(T).variant*`): the
/// complete, ordered variant list for building `target` from flattened
/// leaves. An immutable record — the variant order is the declaration order
/// of the `variants` vector.
#[derive(Clone)]
pub struct ConstructorDecl {
    /// The type being built, as an **identity**. Every use keyed it; none
    /// spelled it.
    pub target: TypeKey,
    pub variants: Vec<Variant>,
    /// Auto-`construct` every matching param of every declared fn. Always
    /// `true` for type-level default (`expand_param!` `.variant*`) declarations.
    pub default: bool,
}

/// How a construct declaration chooses the variants for a parameter.
#[derive(Clone)]
pub enum ExpandSel {
    /// Use the target type's default constructor (error if none/ambiguous).
    TopLevel,
    /// Per-fn override (`.expand_param`): use exactly these build-from
    /// variants (constructor fns and/or the identity/self arm).
    Subset(Vec<Variant>),
}

/// A per-fn input expansion (`.expand_param(param, expand_param!(T)…)`) —
/// construct `param` of `func` from the explicit variant list. Recorded as
/// an explicit decl so the auto-`default` skips it; an identity-only list
/// lowers to the skip-default plain form at resolution. Not related to the
/// jnigen declaration-DSL type of the same name — this is the lowered core
/// record.
#[derive(Clone)]
pub struct ExpandDecl {
    pub func: syn::Ident,
    pub param: syn::Ident,
    /// The type the per-fn decl was declared for (`expand_param!(T)`) —
    /// cross-checked against the named param's peeled type at resolution.
    /// `None` for the internal `TopLevel` form (the type comes from the
    /// param itself).
    pub declared_target: Option<TypeKey>,
    pub sel: ExpandSel,
}

/// Constructor / expansion declarations gathered from a language builder —
/// an immutable record set: complete values, no build protocol. Declaration
/// order is the vector order. Handed to the registry as
/// [`Decompositions::expansions`](crate::Decompositions::expansions); empty
/// or duplicate declarations are diagnosed at resolution (collected), not at
/// construction.
#[derive(Clone, Default)]
pub struct Expansions {
    pub constructors: Vec<ConstructorDecl>,
    pub expands: Vec<ExpandDecl>,
    /// `.skip_default_construct(param)` opt-outs: `(fn, param)` excluded from a
    /// constructor `.default()` auto-apply — the lowered form of an
    /// identity-only per-fn variant set (the plain handle, no selector).
    pub skip_construct: std::collections::HashSet<(syn::Ident, syn::Ident)>,
}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Structural validation of the declaration records — empty variant lists
/// and duplicate targets. Collects EVERY offender before failing, so a
/// build surfaces all declaration problems at once.
fn validate_declarations(exp: &Expansions) -> Result<(), ExpandError> {
    let mut entries: Vec<ExpandDeclError> = Vec::new();
    let mut ctor_targets: HashSet<String> = HashSet::new();
    for c in &exp.constructors {
        let target = c.target.as_str().to_string();
        if c.variants.is_empty() {
            entries.push(ExpandDeclError::EmptyConstructor {
                target: target.clone(),
            });
        }
        if !ctor_targets.insert(target.clone()) {
            entries.push(ExpandDeclError::DuplicateConstructor { target });
        }
    }
    let mut expand_keys: HashSet<(String, String)> = HashSet::new();
    for ed in &exp.expands {
        if let ExpandSel::Subset(v) = &ed.sel {
            if v.is_empty() {
                entries.push(ExpandDeclError::EmptySubset {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                });
            }
        }
        if !expand_keys.insert((ed.func.to_string(), ed.param.to_string())) {
            entries.push(ExpandDeclError::DuplicateExpand {
                func: ed.func.clone(),
                param: ed.param.clone(),
            });
        }
    }
    if entries.is_empty() {
        Ok(())
    } else {
        Err(ExpandError::InvalidDeclarations { entries })
    }
}

/// Resolve every `.construct` declaration (explicit + `.default()`
/// auto-applied) into a [`FoldPlan`], register each plan's leaf types as required
/// inputs, and store the plans on the registry. `declared_fns` is the adapter's
/// claimed `#[prebindgen]` fn set — the domain over which `.default()`
/// constructors auto-apply.
///
/// Runs inside the builder's scan, before any conversion is built, so
/// leaf converters resolve through the normal rank machinery.
pub(crate) fn apply(
    registry: &mut Registry,
    exp: &Expansions,
    declared_fns: &std::collections::HashSet<syn::Ident>,
    accessor_fns: &std::collections::HashSet<syn::Ident>,
    method_receivers: &std::collections::HashMap<syn::Ident, TypeKey>,
) -> Result<(), ExpandError> {
    validate_declarations(exp)?;
    let mut done: HashSet<(String, String)> = HashSet::new();
    let mut skip_construct = exp.skip_construct.clone();
    for ed in &exp.expands {
        // A `.fun_accessor` is never parameter-composed — an explicit
        // `.construct(param)` on one is a build error.
        if accessor_fns.contains(&ed.func) {
            return Err(ExpandError::ConstructOnAccessor {
                func: ed.func.clone(),
            });
        }
        // Per-fn decl cross-check: the named param must exist and its peeled
        // (`Option`/`&`) type must equal the decl's declared type — the
        // typo guard for both coordinates of `.expand_param(name, decl)`.
        if let Some(declared) = &ed.declared_target {
            let param_ty = param_reading(registry, &ed.func, &ed.param)?;
            let bare = constructed_value(&param_ty).key();
            if bare != *declared {
                return Err(ExpandError::ParamTypeMismatch {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    declared: declared.as_str().to_string(),
                    actual: bare.as_str().to_string(),
                });
            }
        }
        // Identity-only variant set = the plain form: no selector, the param
        // crosses as the bare value — lowered to the skip-default opt-out
        // (the complete-set rule: "the set is {self}").
        if let ExpandSel::Subset(v) = &ed.sel {
            if matches!(v.as_slice(), [Variant::Identity]) {
                skip_construct.insert((ed.func.clone(), ed.param.clone()));
                done.insert((ed.func.to_string(), ed.param.to_string()));
                continue;
            }
        }
        process_expand(registry, exp, ed)?;
        done.insert((ed.func.to_string(), ed.param.to_string()));
    }

    // `.default()` auto-apply: `construct` every matching param of every declared
    // fn whose type peeled of `Option`/`&` equals a defaulted constructor target.
    for c in &exp.constructors {
        if !c.default {
            continue;
        }
        let ckey = c.target.clone();
        for func in declared_fns {
            // Read accessors are excluded from the composer.
            if accessor_fns.contains(func) {
                continue;
            }
            let Some(params) = registry.flat().function(&func).map(|f| f.params.clone()) else {
                continue;
            };
            // A method's receiver (first param of its class type) binds to `this`
            // and is never input-flattened; skip exactly that one param.
            let receiver_key = method_receivers.get(func);
            let mut receiver_skipped = false;
            for (pname, pty) in params.iter().map(|p| (p.name.clone(), p.ty.clone())) {
                let bare_key = constructed_value(&pty).key();
                if !receiver_skipped && receiver_key == Some(&bare_key) {
                    receiver_skipped = true;
                    continue;
                }
                if bare_key != ckey {
                    continue;
                }
                if skip_construct.contains(&(func.clone(), pname.clone())) {
                    continue;
                }
                if !done.insert((func.to_string(), pname.to_string())) {
                    continue;
                }
                let ed = ExpandDecl {
                    func: func.clone(),
                    param: pname,
                    declared_target: None,
                    sel: ExpandSel::TopLevel,
                };
                process_expand(registry, exp, &ed)?;
            }
        }
    }
    Ok(())
}

/// `(name, type)` of each typed parameter.
/// The **reading** of a declared function's parameter.
///
/// `Param::ty` is a `TypeRef` computed at parse time. Reaching into the item's
/// `spell()` and digging the parameter out of `sig.inputs` — what these
/// three sites used to do — re-derives a fact the model was already handing over,
/// which is `origin` used for reasoning rather than for emission.
fn param_reading(
    registry: &Registry,
    func: &syn::Ident,
    param: &syn::Ident,
) -> Result<prebindgen_flat::flat::TypeRef, ExpandError> {
    registry
        .flat()
        .function(&func)
        .ok_or_else(|| ExpandError::UnknownFunction(func.clone()))?
        .params
        .iter()
        .find(|p| &p.name == param)
        .map(|p| p.ty.clone())
        .ok_or_else(|| ExpandError::UnknownParam(func.clone(), param.clone()))
}

/// Build + store the fold plan for one `.construct` declaration.
fn process_expand(
    registry: &mut Registry,
    exp: &Expansions,
    ed: &ExpandDecl,
) -> Result<(), ExpandError> {
    let param_ty = param_reading(registry, &ed.func, &ed.param)?;

    // The boundary layers: `Option<&T>` → optional + by_ref, `Option<T>` →
    // optional, `&T` → by_ref, and `target` is what is left under them.
    let (optional, by_ref, target) = constructed_value_layers(&param_ty);
    let target_key = target.key();

    let variants = resolve_constructor(exp, registry, &target_key, ed)?;
    let mut visited: HashSet<TypeKey> = HashSet::new();
    let plan = build_plan(
        exp,
        registry,
        ed,
        optional,
        by_ref,
        &target,
        &variants,
        &mut visited,
    )?;

    // One callback per expanded parameter. A delivered value's site is named by
    // the SOURCE parameter it arrived on, not by the leaf, so a second callback
    // here would give two distinct positions one identity. Refused where the
    // expansion is declared rather than left to collide when the sites are
    // planned (#687 review).
    if plan
        .leaves
        .iter()
        .filter(|leaf| leaf.ty.callback_args().is_some())
        .count()
        > 1
    {
        return Err(ExpandError::TwoCallbacksInOneParam {
            func: ed.func.clone(),
            param: ed.param.clone(),
        });
    }
    for leaf in &plan.leaves {
        registry.require_input(&leaf.ty);
    }
    registry
        .expansion_plans
        .insert((ed.func.clone(), ed.param.clone()), plan);
    Ok(())
}

/// Pick the constructor (its variants) for one `.expand`/`.expand_with`
/// declaration. A constructor is keyed by its declared `target`; `TopLevel`
/// requires it to be unique for the parameter's target type.
fn resolve_constructor(
    exp: &Expansions,
    _registry: &Registry,
    target_key: &TypeKey,
    ed: &ExpandDecl,
) -> Result<Vec<Variant>, ExpandError> {
    match &ed.sel {
        ExpandSel::Subset(variants) => Ok(variants.clone()),
        // Unique per target: `ensure_default_constructor` dedups by type key.
        ExpandSel::TopLevel => exp
            .constructors
            .iter()
            .find(|c| c.target == *target_key)
            .map(|c| c.variants.clone())
            .ok_or_else(|| ExpandError::NoConstructor {
                func: ed.func.clone(),
                param: ed.param.clone(),
                target: target_key.to_string(),
            }),
    }
}

/// Constructor signature: parameter `(name, type)` pairs and whether it is
/// fallible (`-> Result<_, _>`). The produced (`Ok`) target type is *checked*
/// here rather than returned — see below.
///
/// `expected` is the type the declaration is *for*, and the returned signature
/// is one already proven to produce it. Taking it as a parameter rather than
/// leaving the caller to check afterwards is the point: a declarator cannot
/// reach a constructor's signature without saying what that constructor is
/// supposed to build, so the check cannot be the thing a new declarator forgets
/// (#223). The comparison is [`check_declared_target`], shared with the output
/// side's accessor lookup.
fn ctor_signature(
    registry: &Registry,
    func: &syn::Ident,
    expected: &TypeKey,
) -> Result<CtorSig, ExpandError> {
    // Read off the element rather than re-walked from the signature: `params`
    // and `ret` are the same facts, already decided once — including that an
    // elided return and a written `-> ()` are one thing.
    let f = registry
        .flat()
        .function(&func)
        .ok_or_else(|| ExpandError::UnknownConstructor(func.clone()))?;

    let params: Vec<(syn::Ident, prebindgen_flat::flat::TypeRef)> = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    // The model already read this return; `fallible_parts` is that reading, not a
    // second look at the spelling.
    let (target, fallible) = match f.ret.fallible_parts() {
        Some((ok, _)) => (ok.key(), true),
        None => (f.ret.key(), false),
    };
    check_declared_target(func, &target, expected)?;
    Ok(CtorSig { params, fallible })
}

struct CtorSig {
    /// Readings, not spellings: they come off `Function::params`, and a consumer
    /// that needs the spelling takes it at the point it stores one.
    params: Vec<(syn::Ident, prebindgen_flat::flat::TypeRef)>,
    fallible: bool,
}

/// Build the [`FoldPlan`] for a chosen construction. A single `Ctor` variant
/// (no identity) is the plain/unconditional form (no selector); anything else is
/// selector-dispatched — so a "single" constructor and a 1-variant combined emit
/// identical code.
#[allow(clippy::too_many_arguments)]
fn build_plan(
    exp: &Expansions,
    registry: &Registry,
    ed: &ExpandDecl,
    optional: bool,
    by_ref: bool,
    target: &prebindgen_flat::flat::TypeRef,
    variants: &[Variant],
    visited: &mut HashSet<TypeKey>,
) -> Result<FoldPlan, ExpandError> {
    let param = &ed.param;
    let mut leaves: Vec<FoldLeaf> = Vec::new();

    // Optional (`Option<T>`/`Option<&T>`) param. No recursion under `Optional`.
    //  * single single-arg ctor → one nullable leaf (`Option<arg>`) decides
    //    presence.
    //  * single multi-arg ctor  → an explicit leading `present: bool` flag +
    //    one plain (non-`Option`) leaf per arg. The flag keeps nullable
    //    primitive args (e.g. an `Option<i32>` id) from boxing on the wire.
    //  * combined (≥2 variants) → the same selector dispatch as a non-optional
    //    param, with the selector ALSO encoding absence: `-1` = `None`,
    //    `0..n-1` = the taken arm (no separate present flag — not-taken arms'
    //    leaves are null exactly as in the non-optional selector case).
    if optional {
        let [Variant::Ctor(func)] = variants else {
            // Combined-selector dispatch under `Optional`.
            visited.insert(target.key());
            let prefix = param.to_string();
            let (selector, fold_variants) = build_core(
                exp,
                registry,
                ed,
                target,
                variants,
                by_ref,
                &prefix,
                &mut leaves,
                visited,
            )?;
            visited.remove(&target.key());
            return Ok(FoldPlan {
                target: target.clone(),
                by_ref,
                shape: FoldShape::Optional((), Box::new(FoldShape::Base)),
                leaves,
                selector,
                present: None,
                variants: fold_variants,
            });
        };
        let sig = ctor_signature(registry, func, &target.key())?;
        if sig.params.len() == 1 {
            let (_pn, pty) = &sig.params[0];
            leaves.push(FoldLeaf {
                name: param.clone(),
                ty: pty.optional(),
            });
            return Ok(FoldPlan {
                target: target.clone(),
                by_ref,
                shape: FoldShape::Optional((), Box::new(FoldShape::Base)),
                leaves,
                selector: None,
                present: None,
                variants: vec![FoldVariant {
                    ctor: Some(func.clone()),
                    fallible: sig.fallible,
                    clone: false,
                    inputs: vec![FoldArg::Leaf(0, false)],
                }],
            });
        }
        // Multi-arg: presence flag (leaf 0) + one flat leaf per ctor arg.
        leaves.push(FoldLeaf {
            name: ident(&format!("{}_present", param)),
            // A presence flag no source wrote — placeless by construction.
            ty: prebindgen_flat::flat::TypeRef::scalar(prebindgen_flat::flat::ScalarKind::Bool),
        });
        let prefix = param.to_string();
        let mut inputs = Vec::new();
        for (pname, pty) in &sig.params {
            let name = ident(&format!("{}_{}", prefix, pname));
            let arg = build_arg(
                exp,
                registry,
                ed,
                pty,
                name,
                /*dispatched=*/ false,
                &mut leaves,
                visited,
            )?;
            if matches!(arg, FoldArg::Build(_)) {
                return Err(ExpandError::UnsupportedOptional {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    reason: "nested-buildable constructor arguments cannot be optional",
                });
            }
            inputs.push(arg);
        }
        return Ok(FoldPlan {
            target: target.clone(),
            by_ref,
            shape: FoldShape::Optional((), Box::new(FoldShape::Base)),
            leaves,
            selector: None,
            present: Some(0),
            variants: vec![FoldVariant {
                ctor: Some(func.clone()),
                fallible: sig.fallible,
                clone: false,
                inputs,
            }],
        });
    }

    // Non-optional: build the (possibly recursive) construct core. The target is
    // on the cycle chain so a constructor parameter of the same type is rejected.
    visited.insert(target.key());
    let prefix = param.to_string();
    let (selector, fold_variants) = build_core(
        exp,
        registry,
        ed,
        target,
        variants,
        by_ref,
        &prefix,
        &mut leaves,
        visited,
    )?;
    visited.remove(&target.key());
    Ok(FoldPlan {
        target: target.clone(),
        by_ref,
        shape: FoldShape::Base,
        leaves,
        selector,
        present: None,
        variants: fold_variants,
    })
}

/// Build a construct core (selector + dispatch arms) for `target` from its
/// `variants`, appending wire leaves to `leaves`. Recursive: a constructor
/// parameter whose type has its OWN default constructor is built as a nested
/// [`FoldArg::Build`] (recursive input). Used by both the top-level [`build_plan`]
/// and each nested build. `prefix` disambiguates leaf names across the tree.
#[allow(clippy::too_many_arguments)]
fn build_core(
    exp: &Expansions,
    registry: &Registry,
    ed: &ExpandDecl,
    target: &prebindgen_flat::flat::TypeRef,
    variants: &[Variant],
    by_ref: bool,
    prefix: &str,
    leaves: &mut Vec<FoldLeaf>,
    visited: &mut HashSet<TypeKey>,
) -> Result<(Option<usize>, Vec<FoldVariant>), ExpandError> {
    if let [Variant::Ctor(func)] = variants {
        // Single constructor — no selector; args passed directly (not Option-wrapped).
        let sig = ctor_signature(registry, func, &target.key())?;
        let np = sig.params.len();
        let mut args = Vec::new();
        for (pname, pty) in &sig.params {
            let name = if np == 1 {
                ident(prefix)
            } else {
                ident(&format!("{}_{}", prefix, pname))
            };
            args.push(build_arg(
                exp, registry, ed, pty, name, false, leaves, visited,
            )?);
        }
        Ok((
            None,
            vec![FoldVariant {
                ctor: Some(func.clone()),
                fallible: sig.fallible,
                clone: false,
                inputs: args,
            }],
        ))
    } else {
        // Combined — selector leaf, then `Option`-wrapped per-arm inputs.
        let sel_idx = leaves.len();
        leaves.push(FoldLeaf {
            name: ident(&format!("{}_sel", prefix)),
            // The selector, likewise composed and placeless.
            ty: prebindgen_flat::flat::TypeRef::scalar(prebindgen_flat::flat::ScalarKind::I32),
        });
        let mut fold_variants: Vec<FoldVariant> = Vec::new();
        for (vi, v) in variants.iter().enumerate() {
            match v {
                Variant::Ctor(func) => {
                    let sig = ctor_signature(registry, func, &target.key())?;
                    let np = sig.params.len();
                    let mut args = Vec::new();
                    for (pi, (_pname, pty)) in sig.params.iter().enumerate() {
                        let name = if np == 1 {
                            ident(&format!("{}_{}", prefix, vi))
                        } else {
                            ident(&format!("{}_{}_{}", prefix, vi, pi))
                        };
                        // `dispatched = true`: a combined arm's leaves are
                        // `Option`-wrapped (selector presence). Recursive nesting
                        // under a combined arm is rejected by `build_arg`.
                        args.push(build_arg(
                            exp, registry, ed, pty, name, true, leaves, visited,
                        )?);
                    }
                    fold_variants.push(FoldVariant {
                        ctor: Some(func.clone()),
                        fallible: sig.fallible,
                        clone: false,
                        inputs: args,
                    });
                }
                Variant::Identity => {
                    let idx = leaves.len();
                    let leaf_ty = if by_ref {
                        target.borrowed().optional()
                    } else {
                        target.optional()
                    };
                    leaves.push(FoldLeaf {
                        name: ident(&format!("{}_{}", prefix, vi)),
                        ty: leaf_ty,
                    });
                    fold_variants.push(FoldVariant {
                        ctor: None,
                        fallible: false,
                        clone: by_ref,
                        inputs: vec![FoldArg::Leaf(idx, false)],
                    });
                }
            }
        }
        Ok((Some(sel_idx), fold_variants))
    }
}

/// Build one constructor-parameter input. If the parameter's (peeled) type has
/// its own default constructor, recurse into a nested [`FoldArg::Build`]
/// (recursive input); otherwise it is a flat wire [`FoldArg::Leaf`].
#[allow(clippy::too_many_arguments)]
fn build_arg(
    exp: &Expansions,
    registry: &Registry,
    ed: &ExpandDecl,
    pty: &prebindgen_flat::flat::TypeRef,
    name: syn::Ident,
    dispatched: bool,
    leaves: &mut Vec<FoldLeaf>,
    visited: &mut HashSet<TypeKey>,
) -> Result<FoldArg, ExpandError> {
    // The boundary layers down to the parameter's core type.
    let (popt, pby_ref, bare) = constructed_value_layers(pty);
    let key = bare.key();
    // A default constructor for the parameter's type ⇒ recursive nested build.
    let canon = exp
        .constructors
        .iter()
        .find(|c| c.target == key && !c.variants.is_empty());
    if let Some(c) = canon {
        if dispatched {
            return Err(ExpandError::UnsupportedRecursive {
                func: ed.func.clone(),
                reason: "recursive input under a selector-dispatched constructor variant",
            });
        }
        if popt {
            return Err(ExpandError::UnsupportedRecursive {
                func: ed.func.clone(),
                reason: "recursive input on an Option<…> parameter",
            });
        }
        if !visited.insert(key.clone()) {
            return Err(ExpandError::InputCycle {
                ty: key.to_string(),
            });
        }
        let variants = c.variants.clone();
        let (selector, vars) = build_core(
            exp,
            registry,
            ed,
            &bare,
            &variants,
            pby_ref,
            &name.to_string(),
            leaves,
            visited,
        )?;
        visited.remove(&key);
        Ok(FoldArg::Build(Box::new(FoldBuild {
            target: bare.clone(),
            by_ref: pby_ref,
            selector,
            variants: vars,
        })))
    } else {
        let idx = leaves.len();
        // A dispatched (selector-presence) arm `Option`-wraps its leaves — but
        // an argument that is itself `Option<…>` passes through with its own
        // type: `None` is a legitimate value for the taken arm, and the wire
        // cannot represent the double `Option` anyway. Marked `passthrough` so
        // the emit side skips the selector-presence unwrap.
        let passthrough = dispatched && popt;
        leaves.push(FoldLeaf {
            name,
            ty: if dispatched && !passthrough {
                pty.optional()
            } else {
                pty.clone()
            },
        });
        Ok(FoldArg::Leaf(idx, passthrough))
    }
}

/// The shared mismatch, in this direction's vocabulary: an input constructor is
/// declared to **produce** the parameter's target.
impl From<crate::declared_target::TargetMismatch> for ExpandError {
    fn from(m: crate::declared_target::TargetMismatch) -> Self {
        ExpandError::TargetMismatch {
            ctor: m.func,
            produces: m.actual,
            expected: m.expected,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// emit_fold
// ──────────────────────────────────────────────────────────────────────

/// Emit the fold expression for an expanded parameter. `leaf_locals` are the
/// already-decoded Rust locals (1:1 with `plan.leaves`); `qualify` maps a
/// constructor ident to its call path (e.g. prefixing the source module).
///
/// The returned expression has type `Result<<shaped> plan.target, String>`
/// (`Result<Target>`, `Result<Option<Target>>`, …). The adapter routes its
/// `Err(String)` through its own error channel. Folds the [`FoldShape`] layers
/// top-down over one shared core construct — the value
/// analog of how `Option<_>`/`Vec<_>` wrappers compose at the wire.
pub fn emit_fold(
    plan: &FoldPlan,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    fold_shape(&plan.shape, plan, leaf_locals, None, qualify)
}

/// Recurse over one [`FoldShape`] layer. `bound` is `Some(var)` when an
/// enclosing `Optional`/`Iterable` layer has unwrapped the structured leaf and
/// bound its element to `var` — the inner construct then builds from `var`
/// instead of reading `leaf_locals`.
fn fold_shape(
    shape: &FoldShape,
    plan: &FoldPlan,
    leaf_locals: &[syn::Ident],
    bound: Option<&syn::Ident>,
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    match shape {
        FoldShape::Base => emit_core_construct(plan, leaf_locals, bound, qualify),
        FoldShape::Optional((), inner) => {
            if let Some(sidx) = plan.selector {
                // Combined-selector dispatch under `Optional`: the selector
                // ALSO encodes absence — `-1` = `None`, `0..n-1` = the taken
                // arm (dispatched by the shared construct core; an out-of-range
                // selector still hits its `Err` default arm).
                let sel_local = &leaf_locals[sidx];
                let inner_expr = emit_core_construct(plan, leaf_locals, None, qualify);
                syn::parse_quote!(if #sel_local < 0 {
                    ::core::result::Result::Ok(::core::option::Option::None)
                } else {
                    (#inner_expr).map(::core::option::Option::Some)
                })
            } else if let Some(pidx) = plan.present {
                // Multi-arg: an explicit `present: bool` flag decides presence;
                // the construct reads its plain arg leaves directly (`bound =
                // None`), the flag leaf is consumed only by this `if`.
                let present_local = &leaf_locals[pidx];
                let inner_expr = emit_core_construct(plan, leaf_locals, None, qualify);
                syn::parse_quote!(if #present_local {
                    (#inner_expr).map(::core::option::Option::Some)
                } else {
                    ::core::result::Result::Ok(::core::option::Option::None)
                })
            } else {
                // Single-arg: presence rides the sole shaped leaf's `Option`.
                // The structured value is the enclosing bound var, or — at the
                // top — that leaf's decoded local (`leaf_locals[0]`).
                let value = bound.unwrap_or(&leaf_locals[0]);
                let inner_ident = ident("__inner");
                let inner_expr = fold_shape(inner, plan, leaf_locals, Some(&inner_ident), qualify);
                syn::parse_quote!(match #value {
                    ::core::option::Option::Some(#inner_ident) => {
                        (#inner_expr).map(::core::option::Option::Some)
                    }
                    ::core::option::Option::None => {
                        ::core::result::Result::Ok(::core::option::Option::None)
                    }
                })
            }
        }
        FoldShape::Iterable(inner) => {
            let value = bound.unwrap_or(&leaf_locals[0]);
            let elem_ident = ident("__elem");
            let inner_expr = fold_shape(inner, plan, leaf_locals, Some(&elem_ident), qualify);
            syn::parse_quote!(
                #value
                    .into_iter()
                    .map(|#elem_ident| #inner_expr)
                    .collect::<::core::result::Result<::std::vec::Vec<_>, _>>()
            )
        }
    }
}

/// Emit the innermost construct → `Result<Target, String>`. With `bound =
/// Some(v)` (under an `Optional`/`Iterable` layer ⇒ single, single-arg ctor)
/// the ctor is applied to `v`; with `bound = None` (top level) it reads the
/// leaves — a single constructor (any arity) or a combined-selector dispatch.
fn emit_core_construct(
    plan: &FoldPlan,
    leaf_locals: &[syn::Ident],
    bound: Option<&syn::Ident>,
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    if let Some(v) = bound {
        // Shaped construct: a single, single-arg constructor applied to the
        // unwrapped element. (`apply` guarantees this shape — never identity,
        // never combined, never multi-arg under a shape layer.)
        let var = &plan.variants[0];
        let func = var
            .ctor
            .as_ref()
            .expect("shaped expansion is single-constructor (never identity)");
        return ctor_call_result(&qualify(func), std::slice::from_ref(v), var.fallible);
    }
    emit_dispatch(plan.selector, &plan.variants, leaf_locals, qualify)
}

/// Emit a construct dispatch → `Result<Target, String>`: a single variant
/// applied directly (no selector), or a `match` over the selector leaf. Shared
/// by the top-level [`emit_core_construct`] and each nested [`emit_build`].
fn emit_dispatch(
    selector: Option<usize>,
    variants: &[FoldVariant],
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    match selector {
        None => variant_result_expr(
            &variants[0],
            leaf_locals,
            qualify,
            /*dispatched=*/ false,
        ),
        Some(si) => {
            let sel = &leaf_locals[si];
            let arms: Vec<TokenStream> = variants
                .iter()
                .enumerate()
                .map(|(vi, v)| {
                    let lit = vi as i32;
                    let body =
                        variant_result_expr(v, leaf_locals, qualify, /*dispatched=*/ true);
                    quote!(#lit => #body,)
                })
                .collect();
            syn::parse_quote!({
                match #sel {
                    #(#arms)*
                    __sel => ::core::result::Result::Err(::std::format!(
                        "invalid constructor selector: {}",
                        __sel
                    )),
                }
            })
        }
    }
}

/// Emit a nested recursive-input build → `Result<SubTarget, String>` (the dual
/// of [`emit_core_construct`] for a [`FoldArg::Build`] parameter).
fn emit_build(
    b: &FoldBuild,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    emit_dispatch(b.selector, &b.variants, leaf_locals, qualify)
}

/// Build a `Result<Target, String>` expression for one core variant. When
/// `dispatched` (a combined-selector arm), the variant's input leaves are
/// `Option<_>` — only the selected arm's inputs are present — so they are
/// unwrapped (a missing input yields `Err`); otherwise they are passed
/// directly. (This `Option`-ness is *selector presence*, distinct from
/// [`FoldShape::Optional`], which is whole-param presence handled by the
/// enclosing fold.)
fn variant_result_expr(
    v: &FoldVariant,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    dispatched: bool,
) -> syn::Expr {
    // A Leaf arg's decoded local. Identity arms and combined-dispatched arms are
    // Leaf-only (recursive `Build` args appear only in a non-dispatched single
    // constructor — `build_arg` rejects nesting under a dispatched variant).
    let leaf = |a: &FoldArg| -> &syn::Ident {
        match a {
            FoldArg::Leaf(i, _) => &leaf_locals[*i],
            FoldArg::Build(_) => {
                unreachable!("recursive Build arg only in a non-dispatched single constructor")
            }
        }
    };

    match &v.ctor {
        None => {
            // Identity: the sole input is the target value (or a borrow of it
            // that we clone, for `&T` consumers — preserving the caller's handle).
            let loc = leaf(&v.inputs[0]);
            // `&*__v` derefs through whatever the borrow leaf decoded to (a
            // plain `&T`, or an adapter smart-pointer like jnigen's
            // `OwnedObject<T>`) down to `T`, then clones — keeping the caller's
            // handle alive without the core knowing the adapter's borrow type.
            let some_val: syn::Expr = if v.clone {
                syn::parse_quote!(::core::result::Result::Ok(::core::clone::Clone::clone(
                    &*__v
                )))
            } else {
                syn::parse_quote!(::core::result::Result::Ok(__v))
            };
            if dispatched {
                syn::parse_quote!(match #loc {
                    ::core::option::Option::Some(__v) => #some_val,
                    ::core::option::Option::None => ::core::result::Result::Err(
                        ::std::string::String::from("identity variant value missing")
                    ),
                })
            } else if v.clone {
                syn::parse_quote!(::core::result::Result::Ok(::core::clone::Clone::clone(&*#loc)))
            } else {
                syn::parse_quote!(::core::result::Result::Ok(#loc))
            }
        }
        Some(func) => {
            let path = qualify(func);
            if dispatched {
                // Combined arm — Leaf-only inputs. Selector-presence-wrapped
                // inputs are unwrapped (missing ⇒ `Err`); **passthrough**
                // inputs (constructor args that are themselves `Option<…>`)
                // pass their decoded local directly — `None` is a legitimate
                // value for the taken arm.
                let mut wrapped_locals: Vec<&syn::Ident> = Vec::new();
                let mut wrapped_binds: Vec<syn::Ident> = Vec::new();
                let mut call_args: Vec<syn::Expr> = Vec::new();
                for (i, a) in v.inputs.iter().enumerate() {
                    let loc = leaf(a);
                    if matches!(a, FoldArg::Leaf(_, true)) {
                        call_args.push(syn::parse_quote!(#loc));
                    } else {
                        let b = ident(&format!("__p{}", i));
                        wrapped_locals.push(loc);
                        wrapped_binds.push(b.clone());
                        call_args.push(syn::parse_quote!(#b));
                    }
                }
                let call = ctor_call_result(&path, &call_args, v.fallible);
                let missing = quote!(::core::result::Result::Err(::std::string::String::from(
                    "constructor variant input missing"
                )));
                match wrapped_locals.len() {
                    // All-passthrough arm: the selector alone decides; call directly.
                    0 => call,
                    1 => {
                        // `match a { Some(p0) => <call>, None => Err }`
                        let loc = wrapped_locals[0];
                        let p0 = &wrapped_binds[0];
                        syn::parse_quote!(match #loc {
                            ::core::option::Option::Some(#p0) => #call,
                            ::core::option::Option::None => #missing,
                        })
                    }
                    _ => {
                        // `match (a, b, …) { (Some(p0), Some(p1), …) => <call>, _ => Err }`
                        let some_pats: Vec<TokenStream> = wrapped_binds
                            .iter()
                            .map(|b| quote!(::core::option::Option::Some(#b)))
                            .collect();
                        syn::parse_quote!(match ( #(#wrapped_locals),* ) {
                            ( #(#some_pats),* ) => #call,
                            _ => #missing,
                        })
                    }
                }
            } else if v.inputs.iter().all(|a| matches!(a, FoldArg::Leaf(..))) {
                // Non-dispatched, flat (no recursion): call directly — identical
                // to the pre-recursion form.
                let args: Vec<&syn::Ident> = v.inputs.iter().map(&leaf).collect();
                ctor_call_result(&path, &args, v.fallible)
            } else {
                // Non-dispatched with ≥1 recursive `Build` arg: bind each arg
                // (Leaf = the decoded value; Build = the nested construct,
                // `?`-unwrapped) in an IIFE that provides the `Result` context.
                let mut stmts: Vec<TokenStream> = Vec::new();
                let mut args: Vec<TokenStream> = Vec::new();
                for (i, a) in v.inputs.iter().enumerate() {
                    let ai = ident(&format!("__a{}", i));
                    match a {
                        FoldArg::Leaf(li, _) => {
                            let loc = &leaf_locals[*li];
                            stmts.push(quote!(let #ai = #loc;));
                            args.push(quote!(#ai));
                        }
                        FoldArg::Build(b) => {
                            // Pin the nested build's error type to `String` so a
                            // non-fallible inner ctor's bare `Ok(..)` infers `E`.
                            let be = emit_build(b, leaf_locals, qualify);
                            stmts.push(quote!(
                                let #ai = {
                                    let __r: ::core::result::Result<_, ::std::string::String> = #be;
                                    __r?
                                };
                            ));
                            if b.by_ref {
                                args.push(quote!(&#ai));
                            } else {
                                args.push(quote!(#ai));
                            }
                        }
                    }
                }
                let call = ctor_call_result(&path, &args, v.fallible);
                syn::parse_quote!({
                    (|| -> ::core::result::Result<_, ::std::string::String> {
                        #(#stmts)*
                        #call
                    })()
                })
            }
        }
    }
}

/// `path(args…)` lifted to `Result<Target, String>` (mapping a fallible
/// constructor's error via `Display`).
fn ctor_call_result<I: quote::ToTokens>(path: &syn::Path, args: &[I], fallible: bool) -> syn::Expr {
    if fallible {
        syn::parse_quote!(#path( #(#args),* ).map_err(|__e| ::std::format!("{}", __e)))
    } else {
        syn::parse_quote!(::core::result::Result::Ok(#path( #(#args),* )))
    }
}

// ──────────────────────────────────────────────────────────────────────
// Small helpers
// ──────────────────────────────────────────────────────────────────────

/// The value a constructor builds: `Option` off, then the borrow, and **nothing
/// else** — read off the model's classification rather than by taking the
/// spelling apart.
///
/// `Option<&T>`, `&T` and `T` all answer `T`, which is what every caller here
/// wants: they are matching a declared target, and a declaration names the type,
/// not the way a particular parameter happens to wrap it.
///
/// **`Vec<T>` answers `Vec<T>`, deliberately.** Expansion builds one value —
/// `FoldPlan`'s shape is `Base` or `Optional(Base)`, with no iterable arm — so
/// peeling a `Sequence` here would let a `Vec<T>` parameter match a `T`
/// constructor and emit a wrapper that reconstructs a single `T` and hands it to
/// a parameter expecting the collection. Leaving the `Sequence` on the core is
/// what makes that a non-match instead of a miscompile, and it is the reason this
/// is not [`TypeRef::layers`], which peels all three.
///
/// A type the grammar cannot express answers itself — the identity, not a
/// fallback classifier. Nothing reaching here can be one: every signature in play
/// was accepted by the frontend before the scan registered it.
fn constructed_value(reading: &prebindgen_flat::flat::TypeRef) -> &prebindgen_flat::flat::TypeRef {
    let after_opt = reading.optional_inner().unwrap_or(reading);
    after_opt.borrow_target().unwrap_or(after_opt)
}

/// [`constructed_value`], plus which of the two layers were there.
fn constructed_value_layers(
    reading: &prebindgen_flat::flat::TypeRef,
) -> (bool, bool, prebindgen_flat::flat::TypeRef) {
    let optional = reading.optional_inner().is_some();
    let after_opt = reading.optional_inner().unwrap_or(reading);
    let by_ref = after_opt.borrow_target().is_some();
    let core = after_opt.borrow_target().unwrap_or(after_opt);
    // The core READING, not its spelling: the plan composes `Option<&T>` over
    // it, and composing from a reading keeps the kind and the syntax paired.
    (optional, by_ref, core.clone())
}

// `opt` lived here — `parse_quote!(Option<#ty>)` — and built a spelling with no
// classification beside it, so every consumer had to hand it back to the
// registry to learn it was an optional. `TypeRef::optional` composes both at
// once (#275).

#[cfg(test)]
mod tests;
