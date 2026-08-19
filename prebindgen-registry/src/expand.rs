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
mod tree;

pub use self::{
    error::{ExpandDeclError, ExpandError},
    plan::{FoldLeaf, FoldPlan, FoldShape},
    tree::{
        dependencies, select, wire_leaves, Claim, Dependencies, InChild, InChoice, InLeaf, InLink,
        InNode, InPresence, InProduct, InRun, InSlot, IntoRust, Lift, SelectError,
    },
};
use crate::transform::{Lowered, TransformKind, TransformLowerer};

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
pub(crate) fn apply<M>(
    registry: &mut Registry<M>,
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
fn param_reading<M>(
    registry: &Registry<M>,
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
fn process_expand<M>(
    registry: &mut Registry<M>,
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
        &param_ty,
        optional,
        by_ref,
        &target,
        &variants,
        &mut visited,
    )?;

    register_dependencies(registry, &plan.tree);
    registry
        .expansion_plans
        .insert((ed.func.clone(), ed.param.clone()), plan);
    Ok(())
}

/// Pick the constructor (its variants) for one `.expand`/`.expand_with`
/// declaration. A constructor is keyed by its declared `target`; `TopLevel`
/// requires it to be unique for the parameter's target type.
fn resolve_constructor<M>(
    exp: &Expansions,
    _registry: &Registry<M>,
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
fn ctor_signature<M>(
    registry: &Registry<M>,
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

/// Root every input converter a construction needs, under one converter
/// selection.
///
/// Both halves of [`Dependencies`] are required for now: the current layout's
/// selector and presence flag cross as an `i32` and a `bool`, so their
/// converters must resolve. They are named apart because an adapter that picks
/// its own physical representation should not inherit this one's (#444 §1).
///
/// `claims` is taken rather than assumed because registration and lowering must
/// make the *same* decision: a subtree claimed in one and not the other roots
/// converters the binding never calls, and a root can only be gained, never
/// taken back.
fn register_dependencies<M>(registry: &mut Registry<M>, tree: &InNode) {
    let deps = dependencies(tree);
    for ty in deps.required.iter().chain(deps.intrinsic.iter()) {
        registry.require_input(ty);
    }
}

/// Build the [`FoldPlan`] for a chosen construction. A single `Ctor` variant
/// (no identity) is the plain/unconditional form (no selector); anything else is
/// selector-dispatched — so a "single" constructor and a 1-variant combined emit
/// identical code.
#[allow(clippy::too_many_arguments)]
fn build_plan<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    param_ty: &prebindgen_flat::flat::TypeRef,
    optional: bool,
    by_ref: bool,
    target: &prebindgen_flat::flat::TypeRef,
    variants: &[Variant],
    visited: &mut HashSet<TypeKey>,
) -> Result<FoldPlan, ExpandError> {
    let param = &ed.param;
    // Wire slots are handed out as the construction is built; the flat
    // signature is collected from the tree once it stands.
    let mut next = 0usize;

    // Optional (`Option<T>`/`Option<&T>`) param. No recursion under the layer.
    // The three ways the layer decides presence — see `InPresence`:
    //  * single single-arg ctor → the layer decodes its own `Option<arg>` slot
    //    and hands the payload to the constructor.
    //  * single multi-arg ctor  → an explicit leading `present: bool` flag +
    //    one plain (non-`Option`) slot per arg. The flag keeps nullable
    //    primitive args (e.g. an `Option<i32>` id) from boxing on the wire.
    //  * combined (≥2 variants) → the same selector dispatch as a non-optional
    //    param, with the selector ALSO encoding absence: `-1` = `None`,
    //    `0..n-1` = the taken arm (no separate present flag — not-taken arms'
    //    slots are null exactly as in the non-optional selector case).
    if optional {
        let presence_and_core = match variants {
            [Variant::Ctor(func)] => {
                let sig = ctor_signature(registry, func, &target.key())?;
                if sig.params.len() == 1 {
                    let (_pn, pty) = &sig.params[0];
                    // The layer's slot holds the ARGUMENT, optionally: its
                    // `Option` is whole-parameter presence, and what the
                    // constructor gets is the payload the layer unwrapped.
                    let payload = InSlot {
                        slot: next_slot(&mut next),
                        name: param.clone(),
                    };
                    let arg = InChild {
                        link: InLink { by_ref: false },
                        node: InNode {
                            ty: pty.clone(),
                            kind: TransformKind::Leaf(InLeaf::Bound),
                        },
                    };
                    (
                        InPresence::Payload {
                            slot: payload,
                            ty: pty.optional(),
                        },
                        ctor_node(target, func, sig.fallible, vec![arg]),
                    )
                } else {
                    // Multi-arg: presence flag first, then one plain slot per
                    // constructor argument.
                    let flag = InSlot {
                        slot: next_slot(&mut next),
                        name: ident(&format!("{}_present", param)),
                    };
                    let prefix = param.to_string();
                    let mut args = Vec::new();
                    for (pname, pty) in &sig.params {
                        let name = ident(&format!("{}_{}", prefix, pname));
                        let arg = build_arg(
                            exp,
                            registry,
                            ed,
                            pty,
                            name,
                            &format!("{}.{}", param, pname),
                            /*dispatched=*/ false,
                            &mut next,
                            visited,
                        )?;
                        if !matches!(arg.node.kind, TransformKind::Leaf(_)) {
                            return Err(ExpandError::UnsupportedOptional {
                                func: ed.func.clone(),
                                param: ed.param.clone(),
                                at: format!("{}.{}", prefix, pname),
                                reason: "nested-buildable constructor arguments cannot be optional",
                            });
                        }
                        args.push(arg);
                    }
                    (
                        InPresence::Flag(flag),
                        ctor_node(target, func, sig.fallible, args),
                    )
                }
            }
            _ => {
                // Combined-selector dispatch under the layer.
                visited.insert(target.key());
                let prefix = param.to_string();
                let core = build_core(
                    exp, registry, ed, target, variants, by_ref, &prefix, &prefix, &mut next,
                    visited,
                )?;
                visited.remove(&target.key());
                (InPresence::Selector, core)
            }
        };
        let (presence, core) = presence_and_core;
        let tree = InNode {
            ty: param_ty.clone(),
            kind: TransformKind::Optional {
                op: presence,
                inner: Box::new(core),
            },
        };
        return Ok(FoldPlan {
            target: target.clone(),
            by_ref,
            leaves: wire_leaves(&tree),
            tree,
        });
    }

    // Non-optional: build the (possibly recursive) construct core. The target is
    // on the cycle chain so a constructor parameter of the same type is rejected.
    visited.insert(target.key());
    let prefix = param.to_string();
    let tree = build_core(
        exp, registry, ed, target, variants, by_ref, &prefix, &prefix, &mut next, visited,
    )?;
    visited.remove(&target.key());
    Ok(FoldPlan {
        target: target.clone(),
        by_ref,
        leaves: wire_leaves(&tree),
        tree,
    })
}

/// A constructor call over `args`, producing `target`.
fn ctor_node(
    target: &prebindgen_flat::flat::TypeRef,
    func: &syn::Ident,
    fallible: bool,
    args: Vec<InChild>,
) -> InNode {
    InNode {
        ty: target.clone(),
        kind: TransformKind::Product {
            op: InProduct::Ctor {
                func: func.clone(),
                fallible,
            },
            children: args,
        },
    }
}

/// One wire slot used as an argument: `wrapped` when selector presence put an
/// `Option` around it.
fn leaf_child(
    ty: prebindgen_flat::flat::TypeRef,
    slot: usize,
    name: syn::Ident,
    wrapped: bool,
) -> InChild {
    InChild {
        link: InLink { by_ref: false },
        node: InNode {
            ty,
            kind: TransformKind::Leaf(InLeaf::Slot {
                slot: InSlot { slot, name },
                wrapped,
            }),
        },
    }
}

/// Hand out the next wire slot. Slots are numbered as the walk meets them,
/// which is the order the foreign signature takes them in.
fn next_slot(next: &mut usize) -> usize {
    let slot = *next;
    *next += 1;
    slot
}

/// Build a construct core for `target` from its `variants`: a single
/// constructor is one product, anything else is a selector choice over one
/// product per arm. Recursive: a constructor parameter whose type has its OWN
/// default constructor becomes a nested core in place of a leaf. Used by both
/// the top-level [`build_plan`] and each nested build.
///
/// `next` hands out wire slots — a node names the slot it uses and the
/// signature is [collected](wire_leaves) from the finished tree. `prefix`
/// disambiguates slot names across the tree, and `at` is the chain of
/// constructor parameter names a diagnostic reports the failing node by. The
/// two differ: a single-argument constructor keeps its parent's slot prefix
/// (the slot is named after the parameter), so `prefix` alone cannot say how
/// deep a chain of them a failure sits at.
#[allow(clippy::too_many_arguments)]
fn build_core<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    target: &prebindgen_flat::flat::TypeRef,
    variants: &[Variant],
    by_ref: bool,
    prefix: &str,
    at: &str,
    next: &mut usize,
    visited: &mut HashSet<TypeKey>,
) -> Result<InNode, ExpandError> {
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
                exp,
                registry,
                ed,
                pty,
                name,
                &format!("{at}.{pname}"),
                false,
                next,
                visited,
            )?);
        }
        return Ok(ctor_node(target, func, sig.fallible, args));
    }
    // Combined — selector slot, then `Option`-wrapped per-arm inputs.
    let selector = InSlot {
        slot: next_slot(next),
        name: ident(&format!("{}_sel", prefix)),
    };
    let mut arms: Vec<InChild> = Vec::new();
    for (vi, v) in variants.iter().enumerate() {
        let node = match v {
            Variant::Ctor(func) => {
                let sig = ctor_signature(registry, func, &target.key())?;
                let np = sig.params.len();
                let mut args = Vec::new();
                for (pi, (pname, pty)) in sig.params.iter().enumerate() {
                    let name = if np == 1 {
                        ident(&format!("{}_{}", prefix, vi))
                    } else {
                        ident(&format!("{}_{}_{}", prefix, vi, pi))
                    };
                    // `dispatched = true`: a combined arm's leaves are
                    // `Option`-wrapped (selector presence). Recursive nesting
                    // under a combined arm is rejected by `build_arg`.
                    args.push(build_arg(
                        exp,
                        registry,
                        ed,
                        pty,
                        name,
                        &format!("{at}.{pname}"),
                        true,
                        next,
                        visited,
                    )?);
                }
                ctor_node(target, func, sig.fallible, args)
            }
            Variant::Identity => {
                // The PAYLOAD: an identity arm's own value, borrowed when the
                // parameter is. Selector presence is derived from the leaf's
                // `wrapped` wherever the wire is asked for (#447 §1).
                let leaf_ty = if by_ref {
                    target.borrowed()
                } else {
                    target.clone()
                };
                let slot = next_slot(next);
                InNode {
                    ty: target.clone(),
                    kind: TransformKind::Product {
                        op: InProduct::Identity {
                            // A `&T` arm reaches the caller's handle, so the
                            // fold copies out of it rather than consuming it.
                            lift: if by_ref {
                                Lift::CloneDeref
                            } else {
                                Lift::Direct
                            },
                        },
                        children: vec![leaf_child(
                            leaf_ty,
                            slot,
                            ident(&format!("{}_{}", prefix, vi)),
                            true,
                        )],
                    },
                }
            }
        };
        arms.push(InChild {
            link: InLink { by_ref: false },
            node,
        });
    }
    Ok(InNode {
        ty: target.clone(),
        kind: TransformKind::Choice {
            op: InChoice { selector },
            variants: arms,
        },
    })
}

/// Build one constructor-parameter input. If the parameter's (peeled) type has
/// its own default constructor, recurse into a nested core (a recursive input);
/// otherwise it is a flat wire leaf.
#[allow(clippy::too_many_arguments)]
fn build_arg<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    pty: &prebindgen_flat::flat::TypeRef,
    name: syn::Ident,
    at: &str,
    dispatched: bool,
    next: &mut usize,
    visited: &mut HashSet<TypeKey>,
) -> Result<InChild, ExpandError> {
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
                at: at.to_string(),
                reason: "recursive input under a selector-dispatched constructor variant",
            });
        }
        if popt {
            return Err(ExpandError::UnsupportedRecursive {
                func: ed.func.clone(),
                at: at.to_string(),
                reason: "recursive input on an Option<…> parameter",
            });
        }
        if !visited.insert(key.clone()) {
            return Err(ExpandError::InputCycle {
                ty: key.to_string(),
                at: at.to_string(),
            });
        }
        let variants = c.variants.clone();
        let node = build_core(
            exp,
            registry,
            ed,
            &bare,
            &variants,
            pby_ref,
            &name.to_string(),
            at,
            next,
            visited,
        )?;
        visited.remove(&key);
        Ok(InChild {
            link: InLink { by_ref: pby_ref },
            node,
        })
    } else {
        // A dispatched (selector-presence) arm `Option`-wraps its slots — but
        // an argument that is itself `Option<…>` passes through with its own
        // type: `None` is a legitimate value for the taken arm, and the wire
        // cannot represent the double `Option` anyway. Such an argument is not
        // `wrapped`, so the emit side skips the selector-presence unwrap.
        let wrapped = dispatched && !popt;
        // The node carries the PAYLOAD; the `Option` selector presence adds is
        // derived from `wrapped` wherever the wire is asked for, so the two
        // cannot drift apart (#447 §1).
        Ok(leaf_child(pty.clone(), next_slot(next), name, wrapped))
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
/// `Err(String)` through its own error channel. One pass over the plan's tree:
/// the arity layers are nodes like the construction under them, so nothing here
/// walks a second structure.
pub fn emit_fold(
    plan: &FoldPlan,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    emit_fold_tree(plan.tree(), leaf_locals, qualify)
}

/// [`emit_fold`] over a construction tree that is not a plan's own — the tree
/// [`select`] produced for an adapter's converter
/// choices.
///
/// A selected tree owes the same contract as the tree it came from: the
/// expression has type `Result<<shaped> target, String>`. Without this an
/// adapter can choose its converters and then has nothing to emit from, since
/// a `FoldPlan` cannot be built outside this crate.
pub fn emit_fold_tree(
    tree: &InNode,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    tree.lower(&mut ConstructEmitter {
        leaf_locals,
        qualify,
    })
    .expect("emitting a built construct cannot fail")
}

/// The local an arity layer binds its unwrapped value to, and the local an
/// [`InLeaf::Bound`] argument reads. One name for both layers: a nested layer
/// shadows its parent's binding, which is what reaching the innermost value
/// means.
fn bound_local() -> syn::Ident {
    ident("__inner")
}

/// Lowers an into-Rust plan into the Rust expression that performs it.
///
/// Two kinds of value flow through it, told apart by the child's node kind
/// exactly as the constructor call must anyway: a **leaf** lowers to the local
/// holding its decoded value, while every other node lowers to an expression of
/// type `Result<_, String>`.
struct ConstructEmitter<'a> {
    /// The already-decoded Rust locals, 1:1 with `FoldPlan::leaves`.
    leaf_locals: &'a [syn::Ident],
    /// Maps a constructor ident to its call path (e.g. prefixing the source
    /// module).
    qualify: &'a dyn Fn(&syn::Ident) -> syn::Path,
}

impl ConstructEmitter<'_> {
    /// The `Option`-wrapped-by-selector-presence flag of a child that is a
    /// leaf; `None` for a child that is a nested construct.
    fn wrapped(child: &InChild) -> Option<bool> {
        match &child.node.kind {
            TransformKind::Leaf(InLeaf::Slot { wrapped, .. }) => Some(*wrapped),
            TransformKind::Leaf(InLeaf::Bound) => Some(false),
            _ => None,
        }
    }
}

impl TransformLowerer<IntoRust> for ConstructEmitter<'_> {
    type Value = syn::Expr;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, _node: &InNode, op: &InLeaf) -> Result<syn::Expr, Self::Error> {
        let local = match op {
            InLeaf::Slot { slot, .. } => self.leaf_locals[slot.slot].clone(),
            InLeaf::Bound => bound_local(),
        };
        Ok(syn::parse_quote!(#local))
    }

    fn product(
        &mut self,
        _node: &InNode,
        op: &InProduct,
        children: Lowered<'_, IntoRust, syn::Expr>,
    ) -> Result<syn::Expr, Self::Error> {
        let missing = quote!(::core::result::Result::Err(::std::string::String::from(
            "constructor variant input missing"
        )));
        match op {
            InProduct::Identity { lift } => {
                // The sole input is the target value, or something that reaches
                // it — the `lift` says which, and the adapter said the lift.
                //
                // Both deref forms go through `&*` / `*`, which see through
                // whatever the leaf decoded to: a plain `&T`, a `Box<T>`, or an
                // adapter smart-pointer like jnigen's `OwnedObject<T>`. The core
                // never has to know that type, only whether the value can be
                // moved out of it.
                let (child, value) = &children[0];
                let lifted = |v: &syn::Expr| -> syn::Expr {
                    match lift {
                        Lift::Direct => syn::parse_quote!(::core::result::Result::Ok(#v)),
                        Lift::CloneDeref => syn::parse_quote!(::core::result::Result::Ok(
                            ::core::clone::Clone::clone(&*#v)
                        )),
                        Lift::MoveDeref => syn::parse_quote!(::core::result::Result::Ok(*#v)),
                    }
                };
                if Self::wrapped(child) == Some(true) {
                    let some_val = lifted(&syn::parse_quote!(__v));
                    return Ok(syn::parse_quote!(match #value {
                        ::core::option::Option::Some(__v) => #some_val,
                        ::core::option::Option::None => ::core::result::Result::Err(
                            ::std::string::String::from("identity variant value missing")
                        ),
                    }));
                }
                Ok(lifted(value))
            }
            InProduct::Ctor { func, fallible } => {
                let path = (self.qualify)(func);
                if children.iter().all(|(c, _)| Self::wrapped(c).is_some()) {
                    // Flat arguments only. Those `Option`-wrapped by selector
                    // presence are unwrapped (missing ⇒ `Err`); the rest — a
                    // non-dispatched constructor's arguments, *passthrough* ones
                    // the constructor itself declares `Option<…>`, and the value
                    // an enclosing layer bound — are passed as decoded.
                    let mut wrapped_values: Vec<&syn::Expr> = Vec::new();
                    let mut wrapped_binds: Vec<syn::Ident> = Vec::new();
                    let mut call_args: Vec<syn::Expr> = Vec::new();
                    for (i, (child, value)) in children.iter().enumerate() {
                        if Self::wrapped(child) == Some(true) {
                            let b = ident(&format!("__p{}", i));
                            wrapped_values.push(value);
                            wrapped_binds.push(b.clone());
                            call_args.push(syn::parse_quote!(#b));
                        } else {
                            call_args.push(value.clone());
                        }
                    }
                    let call = ctor_call_result(&path, &call_args, *fallible);
                    return Ok(match wrapped_values.len() {
                        // Nothing to unwrap: call directly.
                        0 => call,
                        1 => {
                            // `match a { Some(p0) => <call>, None => Err }`
                            let value = wrapped_values[0];
                            let p0 = &wrapped_binds[0];
                            syn::parse_quote!(match #value {
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
                            syn::parse_quote!(match ( #(#wrapped_values),* ) {
                                ( #(#some_pats),* ) => #call,
                                _ => #missing,
                            })
                        }
                    });
                }
                // At least one argument is itself built. Bind every argument
                // (a leaf to its decoded value; a nested construct
                // `?`-unwrapped) in an IIFE that provides the `Result` context.
                // Selector-presence unwrapping never meets this case —
                // `build_arg` rejects a nested build under a dispatched arm.
                let mut stmts: Vec<TokenStream> = Vec::new();
                let mut args: Vec<TokenStream> = Vec::new();
                for (i, (child, value)) in children.iter().enumerate() {
                    let ai = ident(&format!("__a{}", i));
                    if Self::wrapped(child).is_some() {
                        stmts.push(quote!(let #ai = #value;));
                        args.push(quote!(#ai));
                    } else {
                        // Pin the nested build's error type to `String` so a
                        // non-fallible inner ctor's bare `Ok(..)` infers `E`.
                        stmts.push(quote!(
                            let #ai = {
                                let __r: ::core::result::Result<_, ::std::string::String> = #value;
                                __r?
                            };
                        ));
                        if child.link.by_ref {
                            args.push(quote!(&#ai));
                        } else {
                            args.push(quote!(#ai));
                        }
                    }
                }
                let call = ctor_call_result(&path, &args, *fallible);
                Ok(syn::parse_quote!({
                    (|| -> ::core::result::Result<_, ::std::string::String> {
                        #(#stmts)*
                        #call
                    })()
                }))
            }
        }
    }

    fn choice(
        &mut self,
        _node: &InNode,
        op: &InChoice,
        variants: Lowered<'_, IntoRust, syn::Expr>,
    ) -> Result<syn::Expr, Self::Error> {
        let sel = &self.leaf_locals[op.selector.slot];
        let arms: Vec<TokenStream> = variants
            .iter()
            .enumerate()
            .map(|(vi, (_, body))| {
                let lit = vi as i32;
                quote!(#lit => #body,)
            })
            .collect();
        Ok(syn::parse_quote!({
            match #sel {
                #(#arms)*
                __sel => ::core::result::Result::Err(::std::format!(
                    "invalid constructor selector: {}",
                    __sel
                )),
            }
        }))
    }

    fn optional(
        &mut self,
        _node: &InNode,
        op: &InPresence,
        _inner: &InNode,
        value: syn::Expr,
    ) -> Result<syn::Expr, Self::Error> {
        Ok(match op {
            // The dispatch's selector ALSO encodes absence — `-1` = `None`,
            // `0..n-1` = the taken arm (dispatched by the construction below;
            // an out-of-range selector still hits its `Err` default arm).
            InPresence::Selector => {
                let sel = &self.leaf_locals[_inner.selector().expect("a selector-decided layer")];
                syn::parse_quote!(if #sel < 0 {
                    ::core::result::Result::Ok(::core::option::Option::None)
                } else {
                    (#value).map(::core::option::Option::Some)
                })
            }
            // An explicit flag decides; the construction reads its plain
            // argument slots directly, and the flag slot is consumed only here.
            InPresence::Flag(flag) => {
                let present = &self.leaf_locals[flag.slot];
                syn::parse_quote!(if #present {
                    (#value).map(::core::option::Option::Some)
                } else {
                    ::core::result::Result::Ok(::core::option::Option::None)
                })
            }
            // Presence rides the layer's own `Option` slot: unwrap it, bind the
            // payload, and the single-argument construction below reads that
            // binding.
            InPresence::Payload { slot: payload, .. } => {
                let slot = &self.leaf_locals[payload.slot];
                let bound = bound_local();
                syn::parse_quote!(match #slot {
                    ::core::option::Option::Some(#bound) => {
                        (#value).map(::core::option::Option::Some)
                    }
                    ::core::option::Option::None => {
                        ::core::result::Result::Ok(::core::option::Option::None)
                    }
                })
            }
        })
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &InRun,
        _inner: &InNode,
        value: syn::Expr,
    ) -> Result<syn::Expr, Self::Error> {
        let slot = &self.leaf_locals[op.slot.slot];
        let bound = bound_local();
        Ok(syn::parse_quote!(
            #slot
                .into_iter()
                .map(|#bound| #value)
                .collect::<::core::result::Result<::std::vec::Vec<_>, _>>()
        ))
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
