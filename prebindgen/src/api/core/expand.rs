//! Constructor expansion — fold a value's construction into the wire
//! signature of the function that consumes it, so the foreign side builds
//! the value and calls the function in a single FFI crossing.
//!
//! A *constructor* is any `#[prebindgen]` function `f(p0, …) -> T` (or
//! `-> Result<T, E>`) that builds a target type `T`. A type's input flatten
//! (`.flatten_input()` + `.variant`/`.variant_self`, or the per-fn
//! `.flatten_input_with(param)` override) replaces a parameter of that type —
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
//! converters. [`apply`] resolves declarations into [`FoldPlan`]s (stored on
//! the registry, keyed by `(fn, param)`) and registers each leaf type as a
//! required input so the resolver produces its converter. [`emit_fold`]
//! emits the dispatch expression at the parameter-emission site.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::api::core::{
    registry::{Registry, TypeKey},
    types_util::{ident, option_inner_type, result_ok_type},
};

mod error;
mod plan;

pub use self::{
    error::ExpandError,
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
enum Variant {
    /// Build the target by calling this constructor function.
    Ctor(syn::Ident),
    /// Pass an already-built target value straight through.
    Identity,
}

#[derive(Clone)]
struct ConstructorDecl {
    target: syn::Type,
    variants: Vec<Variant>,
    /// Auto-`construct` every matching param of every declared fn. Always
    /// `true` for `.flatten_input()`-created declarations.
    default: bool,
}

/// How a construct declaration chooses the variants for a parameter.
#[derive(Clone)]
enum ExpandSel {
    /// Use the target type's default constructor (error if none/ambiguous).
    TopLevel,
    /// Per-fn override (`.flatten_input_with`): use exactly these build-from
    /// variants (constructor fns and/or the identity/self arm).
    Subset(Vec<Variant>),
}

#[derive(Clone)]
struct ExpandDecl {
    func: syn::Ident,
    param: syn::Ident,
    sel: ExpandSel,
}

/// Constructor / expansion declarations gathered from a language builder.
/// Embedded in each adapter that supports expansion and handed to [`apply`]
/// via [`crate::api::core::prebindgen::Prebindgen::expansions`].
#[derive(Clone, Default)]
pub struct Expansions {
    constructors: Vec<ConstructorDecl>,
    expands: Vec<ExpandDecl>,
    /// Cursor for the constructor builder (`.constructor_variant*` / `.default`).
    cur_constructor: Option<usize>,
    /// Cursor for an in-progress per-fn input subset (`.flatten_input_with(...)`
    /// → `.variant`/`.variant_self`): index into [`Self::expands`].
    cur_expand: Option<usize>,
    /// `.skip_default_construct(param)` opt-outs: `(fn, param)` excluded from a
    /// constructor `.default()` auto-apply.
    skip_construct: std::collections::HashSet<(syn::Ident, syn::Ident)>,
}

impl Expansions {
    /// Find-or-create the default (always-`default`) constructor for `target`
    /// and set the cursor to it. Idempotent across a `.flatten_input()` chain —
    /// the first call creates it, subsequent ones reuse it so variants accumulate.
    pub fn ensure_default_constructor(&mut self, target: syn::Type) {
        let key = TypeKey::from_type(&target);
        if let Some(i) = self
            .constructors
            .iter()
            .position(|c| TypeKey::from_type(&c.target) == key)
        {
            self.cur_constructor = Some(i);
        } else {
            self.constructors.push(ConstructorDecl {
                target,
                variants: Vec::new(),
                default: true,
            });
            self.cur_constructor = Some(self.constructors.len() - 1);
        }
    }

    /// `.flatten_input_suppress(param)` on the current `.fun` — exclude
    /// `(func, param)` from constructor default auto-apply.
    pub fn add_skip_default_construct(&mut self, func: syn::Ident, param: syn::Ident) {
        self.skip_construct.insert((func, param));
    }

    /// `.variant(name)` inside `.flatten_input()` — add a constructor-function arm.
    pub fn add_constructor_variant(&mut self, func: syn::Ident) {
        let i = self
            .cur_constructor
            .expect(".variant called without a current .flatten_input");
        self.constructors[i].variants.push(Variant::Ctor(func));
    }

    /// `.variant_self()` inside `.flatten_input()` — add the identity arm
    /// (pass the target value straight through).
    pub fn add_constructor_variant_id(&mut self) {
        let i = self
            .cur_constructor
            .expect(".variant_self called without a current .flatten_input");
        self.constructors[i].variants.push(Variant::Identity);
    }

    /// Begin a per-fn input flatten (`.flatten_input_with(param)`): construct
    /// `param` from an explicit, incrementally-built variant list (constructor
    /// arms via [`Self::push_subset_variant`] and/or the identity/self arm via
    /// [`Self::push_subset_self`]). Recorded as an explicit decl so the
    /// auto-`default` skips it, and leaves the expand cursor on it.
    pub fn begin_subset(&mut self, func: syn::Ident, param: syn::Ident) {
        self.expands.push(ExpandDecl {
            func,
            param,
            sel: ExpandSel::Subset(Vec::new()),
        });
        self.cur_constructor = None;
        self.cur_expand = Some(self.expands.len() - 1);
    }

    /// `.variant(fn)` inside a `.flatten_input_with(...)` — append a build-from
    /// constructor arm to the current per-fn input subset.
    pub fn push_subset_variant(&mut self, func: syn::Ident) {
        let i = self
            .cur_expand
            .expect(".variant called without a current .flatten_input_with");
        if let ExpandSel::Subset(v) = &mut self.expands[i].sel {
            v.push(Variant::Ctor(func));
        }
    }

    /// `.variant_self()` inside a `.flatten_input_with(...)` — append the
    /// identity (pass-the-handle-through) arm to the current per-fn subset.
    pub fn push_subset_self(&mut self) {
        let i = self
            .cur_expand
            .expect(".variant_self called without a current .flatten_input_with");
        if let ExpandSel::Subset(v) = &mut self.expands[i].sel {
            v.push(Variant::Identity);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Resolve every `.construct` declaration (explicit + `.default()`
/// auto-applied) into a [`FoldPlan`], register each plan's leaf types as required
/// inputs, and store the plans on the registry. `declared_fns` is the adapter's
/// claimed `#[prebindgen]` fn set — the domain over which `.default()`
/// constructors auto-apply.
///
/// Runs inside `write_rust` after `scan_declared` and before `resolve`, so
/// leaf converters resolve through the normal rank machinery.
pub fn apply<M>(
    registry: &mut Registry<M>,
    exp: &Expansions,
    declared_fns: &std::collections::HashSet<syn::Ident>,
    accessor_fns: &std::collections::HashSet<syn::Ident>,
    method_receivers: &std::collections::HashMap<syn::Ident, TypeKey>,
) -> Result<(), ExpandError> {
    let mut done: HashSet<(String, String)> = HashSet::new();
    for ed in &exp.expands {
        // A `.fun_accessor` is never parameter-composed — an explicit
        // `.construct(param)` on one is a build error.
        if accessor_fns.contains(&ed.func) {
            return Err(ExpandError::ConstructOnAccessor {
                func: ed.func.clone(),
            });
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
        let ckey = TypeKey::from_type(&c.target);
        for func in declared_fns {
            // Read accessors are excluded from the composer.
            if accessor_fns.contains(func) {
                continue;
            }
            let Some((item_fn, _)) = registry.functions.get(func).cloned() else {
                continue;
            };
            // A method's receiver (first param of its class type) binds to `this`
            // and is never input-flattened; skip exactly that one param.
            let receiver_key = method_receivers.get(func);
            let mut receiver_skipped = false;
            for (pname, pty) in fn_params(&item_fn) {
                let core = option_inner_type(&pty).unwrap_or(pty);
                let bare = match &core {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    other => other.clone(),
                };
                let bare_key = TypeKey::from_type(&bare);
                if !receiver_skipped && receiver_key == Some(&bare_key) {
                    receiver_skipped = true;
                    continue;
                }
                if bare_key != ckey {
                    continue;
                }
                if exp.skip_construct.contains(&(func.clone(), pname.clone())) {
                    continue;
                }
                if !done.insert((func.to_string(), pname.to_string())) {
                    continue;
                }
                let ed = ExpandDecl {
                    func: func.clone(),
                    param: pname,
                    sel: ExpandSel::TopLevel,
                };
                process_expand(registry, exp, &ed)?;
            }
        }
    }
    Ok(())
}

/// `(name, type)` of each typed parameter.
fn fn_params(item_fn: &syn::ItemFn) -> Vec<(syn::Ident, syn::Type)> {
    item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            syn::FnArg::Typed(pt) => match &*pt.pat {
                syn::Pat::Ident(pi) => Some((pi.ident.clone(), (*pt.ty).clone())),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Build + store the fold plan for one `.construct` declaration.
fn process_expand<M>(
    registry: &mut Registry<M>,
    exp: &Expansions,
    ed: &ExpandDecl,
) -> Result<(), ExpandError> {
    let (item_fn, loc) = registry
        .functions
        .get(&ed.func)
        .cloned()
        .ok_or_else(|| ExpandError::UnknownFunction(ed.func.clone()))?;

    let param_ty = find_param_type(&item_fn, &ed.param)
        .ok_or_else(|| ExpandError::UnknownParam(ed.func.clone(), ed.param.clone()))?;

    // Peel `Option<…>` (whole param optional) then a leading `&` (borrow):
    // `Option<&T>` → optional + by_ref, `Option<T>` → optional, `&T` → by_ref.
    let (optional, inner) = match option_inner_type(&param_ty) {
        Some(i) => (true, i),
        None => (false, param_ty.clone()),
    };
    let (by_ref, target) = match &inner {
        syn::Type::Reference(r) => (true, (*r.elem).clone()),
        other => (false, other.clone()),
    };
    let target_key = TypeKey::from_type(&target);

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

    for leaf in &plan.leaves {
        registry.require_input(&leaf.ty, &loc);
    }
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
            .find(|c| TypeKey::from_type(&c.target) == *target_key)
            .map(|c| c.variants.clone())
            .ok_or_else(|| ExpandError::NoConstructor {
                func: ed.func.clone(),
                param: ed.param.clone(),
                target: target_key.to_string(),
            }),
    }
}

/// Constructor signature: parameter `(name, type)` pairs, the produced
/// (`Ok`) target type, and whether it is fallible (`-> Result<_, _>`).
fn ctor_signature<M>(registry: &Registry<M>, func: &syn::Ident) -> Result<CtorSig, ExpandError> {
    let (item_fn, _) = registry
        .functions
        .get(func)
        .ok_or_else(|| ExpandError::UnknownConstructor(func.clone()))?;

    let mut params: Vec<(syn::Ident, syn::Type)> = Vec::new();
    for input in &item_fn.sig.inputs {
        if let syn::FnArg::Typed(pt) = input {
            let name = match &*pt.pat {
                syn::Pat::Ident(pi) => pi.ident.clone(),
                _ => syn::Ident::new("arg", Span::call_site()),
            };
            params.push((name, (*pt.ty).clone()));
        }
    }
    let ret: syn::Type = match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => (**t).clone(),
    };
    let (target, fallible) = match result_ok_type(&ret) {
        Some(ok) => (ok, true),
        None => (ret, false),
    };
    Ok(CtorSig {
        params,
        target,
        fallible,
    })
}

struct CtorSig {
    params: Vec<(syn::Ident, syn::Type)>,
    target: syn::Type,
    fallible: bool,
}

/// Build the [`FoldPlan`] for a chosen construction. A single `Ctor` variant
/// (no identity) is the plain/unconditional form (no selector); anything else is
/// selector-dispatched — so a "single" constructor and a 1-variant combined emit
/// identical code.
fn build_plan<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    optional: bool,
    by_ref: bool,
    target: &syn::Type,
    variants: &[Variant],
    visited: &mut HashSet<TypeKey>,
) -> Result<FoldPlan, ExpandError> {
    let param = &ed.param;
    let mut leaves: Vec<FoldLeaf> = Vec::new();

    // Optional (`Option<T>`/`Option<&T>`) param: a single (never
    // selector-dispatched) constructor. No recursion under `Optional`.
    //  * single-arg ctor → one nullable leaf (`Option<arg>`) decides presence.
    //  * multi-arg ctor  → an explicit leading `present: bool` flag + one plain
    //    (non-`Option`) leaf per arg. The flag keeps nullable primitive args
    //    (e.g. an `Option<i32>` id) from boxing on the wire.
    if optional {
        let [Variant::Ctor(func)] = variants else {
            return Err(ExpandError::UnsupportedOptional {
                func: ed.func.clone(),
                param: ed.param.clone(),
                reason: "selector-dispatched constructors cannot be optional",
            });
        };
        let sig = ctor_signature(registry, func)?;
        check_target(func, &sig.target, target)?;
        if sig.params.len() == 1 {
            let (_pn, pty) = &sig.params[0];
            leaves.push(FoldLeaf {
                name: param.clone(),
                ty: opt(pty),
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
                    inputs: vec![FoldArg::Leaf(0)],
                }],
            });
        }
        // Multi-arg: presence flag (leaf 0) + one flat leaf per ctor arg.
        leaves.push(FoldLeaf {
            name: ident(&format!("{}_present", param)),
            ty: syn::parse_quote!(bool),
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
    visited.insert(TypeKey::from_type(target));
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
    visited.remove(&TypeKey::from_type(target));
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
fn build_core<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    target: &syn::Type,
    variants: &[Variant],
    by_ref: bool,
    prefix: &str,
    leaves: &mut Vec<FoldLeaf>,
    visited: &mut HashSet<TypeKey>,
) -> Result<(Option<usize>, Vec<FoldVariant>), ExpandError> {
    if let [Variant::Ctor(func)] = variants {
        // Single constructor — no selector; args passed directly (not Option-wrapped).
        let sig = ctor_signature(registry, func)?;
        check_target(func, &sig.target, target)?;
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
            ty: syn::parse_quote!(i32),
        });
        let mut fold_variants: Vec<FoldVariant> = Vec::new();
        for (vi, v) in variants.iter().enumerate() {
            match v {
                Variant::Ctor(func) => {
                    let sig = ctor_signature(registry, func)?;
                    check_target(func, &sig.target, target)?;
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
                        opt(&syn::parse_quote!(&#target))
                    } else {
                        opt(target)
                    };
                    leaves.push(FoldLeaf {
                        name: ident(&format!("{}_{}", prefix, vi)),
                        ty: leaf_ty,
                    });
                    fold_variants.push(FoldVariant {
                        ctor: None,
                        fallible: false,
                        clone: by_ref,
                        inputs: vec![FoldArg::Leaf(idx)],
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
fn build_arg<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    ed: &ExpandDecl,
    pty: &syn::Type,
    name: syn::Ident,
    dispatched: bool,
    leaves: &mut Vec<FoldLeaf>,
    visited: &mut HashSet<TypeKey>,
) -> Result<FoldArg, ExpandError> {
    // Peel `Option<…>` then a leading `&` to reach the parameter's core type.
    let (popt, core) = match option_inner_type(pty) {
        Some(i) => (true, i),
        None => (false, pty.clone()),
    };
    let (pby_ref, bare) = match &core {
        syn::Type::Reference(r) => (true, (*r.elem).clone()),
        other => (false, other.clone()),
    };
    let key = TypeKey::from_type(&bare);
    // A default constructor for the parameter's type ⇒ recursive nested build.
    let canon = exp
        .constructors
        .iter()
        .find(|c| TypeKey::from_type(&c.target) == key && !c.variants.is_empty());
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
            target: bare,
            by_ref: pby_ref,
            selector,
            variants: vars,
        })))
    } else {
        let idx = leaves.len();
        leaves.push(FoldLeaf {
            name,
            ty: if dispatched { opt(pty) } else { pty.clone() },
        });
        Ok(FoldArg::Leaf(idx))
    }
}

fn check_target(
    func: &syn::Ident,
    produces: &syn::Type,
    expected: &syn::Type,
) -> Result<(), ExpandError> {
    if TypeKey::from_type(produces) == TypeKey::from_type(expected) {
        Ok(())
    } else {
        Err(ExpandError::TargetMismatch {
            ctor: func.to_string(),
            produces: TypeKey::from_type(produces).to_string(),
            expected: TypeKey::from_type(expected).to_string(),
        })
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
/// top-down over the shared [core construct](`emit_core_construct`) — the value
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
            if let Some(pidx) = plan.present {
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
            FoldArg::Leaf(i) => &leaf_locals[*i],
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
                // Combined arm — Leaf-only inputs, `Option`-wrapped (selector
                // presence); unwrap or yield `Err`.
                let input_locals: Vec<&syn::Ident> = v.inputs.iter().map(&leaf).collect();
                let bind: Vec<syn::Ident> = (0..input_locals.len())
                    .map(|i| ident(&format!("__p{}", i)))
                    .collect();
                let call = ctor_call_result(&path, &bind, v.fallible);
                let missing = quote!(::core::result::Result::Err(::std::string::String::from(
                    "constructor variant input missing"
                )));
                if input_locals.len() == 1 {
                    // `match a { Some(p0) => <call>, None => Err }`
                    let loc = input_locals[0];
                    let p0 = &bind[0];
                    syn::parse_quote!(match #loc {
                        ::core::option::Option::Some(#p0) => #call,
                        ::core::option::Option::None => #missing,
                    })
                } else {
                    // `match (a, b, …) { (Some(p0), Some(p1), …) => <call>, _ => Err }`
                    let some_pats: Vec<TokenStream> = bind
                        .iter()
                        .map(|b| quote!(::core::option::Option::Some(#b)))
                        .collect();
                    syn::parse_quote!(match ( #(#input_locals),* ) {
                        ( #(#some_pats),* ) => #call,
                        _ => #missing,
                    })
                }
            } else if v.inputs.iter().all(|a| matches!(a, FoldArg::Leaf(_))) {
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
                        FoldArg::Leaf(li) => {
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

fn find_param_type(item_fn: &syn::ItemFn, param: &syn::Ident) -> Option<syn::Type> {
    for input in &item_fn.sig.inputs {
        if let syn::FnArg::Typed(pt) = input {
            if let syn::Pat::Ident(pi) = &*pt.pat {
                if &pi.ident == param {
                    return Some((*pt.ty).clone());
                }
            }
        }
    }
    None
}

fn opt(ty: &syn::Type) -> syn::Type {
    syn::parse_quote!(Option<#ty>)
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;
    use crate::api::core::registry::Registry;

    fn reg_with(fns: &[&str]) -> Registry<()> {
        let items = fns
            .iter()
            .map(|src| {
                let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
                (syn::Item::Fn(f), crate::SourceLocation::default())
            })
            .collect::<Vec<_>>();
        Registry::from_items(items).expect("index")
    }

    fn src_qualify(id: &syn::Ident) -> syn::Path {
        syn::parse_quote!(zenoh_flat::#id)
    }

    #[test]
    fn single_constructor_plan_and_fold() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        // Single build-from variant = one Ctor arm (no selector), declared
        // per-fn (`.flatten_input_with`).
        exp.begin_subset(ident("z_keyexpr_intersects"), ident("a"));
        exp.push_subset_variant(ident("z_keyexpr_try_from"));

        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .expect("apply");

        let plan = reg
            .expansion_plans
            .get(&(ident("z_keyexpr_intersects"), ident("a")))
            .expect("plan for a");
        assert!(plan.by_ref, "param was &ZKeyExpr");
        assert_eq!(plan.selector, None);
        assert_eq!(plan.leaves.len(), 1);
        assert_eq!(plan.leaves[0].name.to_string(), "a");
        assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "String");

        let locals = vec![ident("a")];
        let folded = emit_fold(plan, &locals, &src_qualify);
        let s = folded.to_token_stream().to_string();
        assert!(s.contains("z_keyexpr_try_from"), "fold calls ctor: {}", s);
        assert!(s.contains("map_err"), "fallible ctor mapped: {}", s);
    }

    #[test]
    fn constructor_plan_and_fold() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.begin_subset(ident("z_keyexpr_intersects"), ident("a"));
        exp.push_subset_variant(ident("z_keyexpr_try_from"));
        exp.push_subset_self();

        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .expect("apply");

        let plan = reg
            .expansion_plans
            .get(&(ident("z_keyexpr_intersects"), ident("a")))
            .unwrap();
        assert_eq!(plan.selector, Some(0));
        // selector + try_from(String) + identity(ZKeyExpr) = 3 leaves
        assert_eq!(plan.leaves.len(), 3);
        assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "i32");
        assert_eq!(
            plan.leaves[1].ty.to_token_stream().to_string(),
            "Option < String >"
        );
        // `&ZKeyExpr` consumer ⇒ borrowed identity leaf (clone-preserving).
        assert_eq!(
            plan.leaves[2].ty.to_token_stream().to_string(),
            "Option < & ZKeyExpr >"
        );
        assert_eq!(plan.variants.len(), 2);
        assert!(plan.variants[0].ctor.is_some());
        assert!(plan.variants[1].ctor.is_none(), "identity arm");
        assert!(plan.variants[1].clone, "by-ref identity clones");

        // Leaf types registered as required inputs (so the resolver builds
        // their converters).
        assert!(reg
            .required_inputs_scan
            .contains(&TypeKey::from_type(&plan.leaves[1].ty)));

        let locals = vec![ident("sel"), ident("v0"), ident("vid")];
        let folded = emit_fold(plan, &locals, &src_qualify);
        let s = folded.to_token_stream().to_string();
        assert!(s.contains("match sel"), "dispatch on selector: {}", s);
        assert!(s.contains("z_keyexpr_try_from"));
        assert!(s.contains("invalid constructor selector"));
    }

    #[test]
    fn optional_byvalue_single_ctor() {
        // `attachment: Option<ZZBytes>` with single `z_zbytes_from_vec(Vec<u8>)`.
        let mut reg = reg_with(&[
            "fn z_zbytes_from_vec(bytes: Vec<u8>) -> ZZBytes { todo!() }",
            "fn z_session_delete(s: &ZSession, attachment: Option<ZZBytes>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.begin_subset(ident("z_session_delete"), ident("attachment"));
        exp.push_subset_variant(ident("z_zbytes_from_vec"));

        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .expect("apply optional by-value");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_session_delete"), ident("attachment")))
            .unwrap();
        assert!(matches!(plan.shape, FoldShape::Optional((), _)));
        assert!(plan.produces_option());
        assert!(!plan.by_ref);
        assert_eq!(plan.leaves.len(), 1);
        // nullable leaf wrapping the ctor param
        assert_eq!(
            plan.leaves[0].ty.to_token_stream().to_string(),
            "Option < Vec < u8 > >"
        );

        let locals = vec![ident("att")];
        let s = emit_fold(plan, &locals, &src_qualify)
            .to_token_stream()
            .to_string();
        assert!(s.contains("z_zbytes_from_vec"), "fold calls ctor: {}", s);
        assert!(
            s.contains("Some") && s.contains("None"),
            "maps Option: {}",
            s
        );
    }

    #[test]
    fn optional_byref_single_ctor() {
        // `encoding: Option<&ZEncoding>` with single, infallible
        // `z_encoding_from_string(String) -> ZEncoding`.
        let mut reg = reg_with(&[
            "fn z_encoding_from_string(s: String) -> ZEncoding { todo!() }",
            "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.begin_subset(ident("z_session_put"), ident("encoding"));
        exp.push_subset_variant(ident("z_encoding_from_string"));

        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .expect("apply optional by-ref");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_session_put"), ident("encoding")))
            .unwrap();
        assert!(matches!(plan.shape, FoldShape::Optional((), _)));
        assert!(plan.produces_option());
        assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
        assert_eq!(
            plan.leaves[0].ty.to_token_stream().to_string(),
            "Option < String >"
        );
        assert_eq!(
            plan.target.to_token_stream().to_string(),
            "ZEncoding",
            "target peeled through Option<&_>"
        );
    }

    #[test]
    fn optional_byref_multi_arg_ctor() {
        // `encoding: Option<&ZEncoding>` built from a TWO-arg, infallible
        // `z_encoding_from_id(i32, Option<String>) -> ZEncoding`: an explicit
        // `present: bool` flag + two plain (non-`Option`-wrapped) arg leaves.
        let mut reg = reg_with(&[
            "fn z_encoding_from_id(id: i32, schema: Option<String>) -> ZEncoding { todo!() }",
            "fn z_session_put(s: &ZSession, encoding: Option<&ZEncoding>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.begin_subset(ident("z_session_put"), ident("encoding"));
        exp.push_subset_variant(ident("z_encoding_from_id"));

        apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .expect("apply optional multi-arg by-ref");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_session_put"), ident("encoding")))
            .unwrap();
        assert!(matches!(plan.shape, FoldShape::Optional((), _)));
        assert!(plan.produces_option());
        assert!(plan.by_ref, "Option<&T> ⇒ by_ref");
        assert_eq!(plan.present, Some(0), "explicit presence flag at leaf 0");
        // leaf 0 = present:bool, leaf 1 = id:i32, leaf 2 = schema:Option<String>
        assert_eq!(plan.leaves.len(), 3);
        assert_eq!(plan.leaves[0].name.to_string(), "encoding_present");
        assert_eq!(plan.leaves[0].ty.to_token_stream().to_string(), "bool");
        assert_eq!(plan.leaves[1].name.to_string(), "encoding_id");
        assert_eq!(plan.leaves[1].ty.to_token_stream().to_string(), "i32");
        assert_eq!(plan.leaves[2].name.to_string(), "encoding_schema");
        assert_eq!(
            plan.leaves[2].ty.to_token_stream().to_string(),
            "Option < String >"
        );

        let locals = vec![ident("pres"), ident("id"), ident("schema")];
        let s = emit_fold(plan, &locals, &src_qualify)
            .to_token_stream()
            .to_string();
        assert!(s.contains("if pres"), "presence-flag gated: {}", s);
        assert!(
            s.contains("z_encoding_from_id"),
            "fold calls multi-arg ctor: {}",
            s
        );
        assert!(
            s.contains("Some") && s.contains("None"),
            "maps Option: {}",
            s
        );
    }

    #[test]
    fn optional_combined_rejected() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_session_get(s: &ZSession, ke: Option<ZKeyExpr>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.begin_subset(ident("z_session_get"), ident("ke"));
        exp.push_subset_variant(ident("z_keyexpr_try_from"));
        exp.push_subset_self();

        match apply(
            &mut reg,
            &exp,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        ) {
            Err(ExpandError::UnsupportedOptional { .. }) => {}
            other => panic!("expected UnsupportedOptional, got {:?}", other.err()),
        }
    }

    #[test]
    fn iterable_emit_shape() {
        // `Iterable(Construct)` is not yet produced by `apply` (no `Vec<_>`
        // param expansion is declared), but the fold is emit-ready: a hand-built
        // plan must produce the `into_iter().map(...).collect::<Result<Vec<_>,_>>()`
        // form, with the inner single-arg ctor applied per element.
        let plan = FoldPlan {
            target: syn::parse_quote!(ZKeyExpr),
            by_ref: false,
            shape: FoldShape::Iterable(Box::new(FoldShape::Base)),
            leaves: vec![FoldLeaf {
                name: ident("kes"),
                ty: syn::parse_quote!(Vec<String>),
            }],
            selector: None,
            present: None,
            variants: vec![FoldVariant {
                ctor: Some(ident("z_keyexpr_try_from")),
                fallible: true,
                clone: false,
                inputs: vec![FoldArg::Leaf(0)],
            }],
        };
        let locals = vec![ident("kes")];
        let s = emit_fold(&plan, &locals, &src_qualify)
            .to_token_stream()
            .to_string();
        assert!(s.contains("into_iter"), "iterates: {}", s);
        assert!(s.contains("collect"), "collects: {}", s);
        assert!(
            s.contains("Vec") && s.contains("z_keyexpr_try_from"),
            "collects Result<Vec<_>> via per-elem ctor: {}",
            s
        );
        assert!(!plan.produces_option());
    }

    #[test]
    fn default_constructor_auto_applies_and_skips() {
        // A `.default()` ZKeyExpr constructor auto-`construct`s every matching
        // param of every declared fn — except where `.skip_default_construct`'d.
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
            "fn z_session_undeclare(s: &ZSession, k: ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        // Opt the undeclare's `k` out (must stay a handle).
        exp.add_skip_default_construct(ident("z_session_undeclare"), ident("k"));
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_intersects", "z_session_undeclare"]
                .iter()
                .map(|s| ident(s))
                .collect();
        apply(
            &mut reg,
            &exp,
            &declared,
            &Default::default(),
            &Default::default(),
        )
        .expect("apply");

        // Both `&ZKeyExpr` params of intersects are constructed.
        assert!(reg
            .expansion_plans
            .contains_key(&(ident("z_keyexpr_intersects"), ident("a"))));
        assert!(reg
            .expansion_plans
            .contains_key(&(ident("z_keyexpr_intersects"), ident("b"))));
        // The skipped param is NOT.
        assert!(!reg
            .expansion_plans
            .contains_key(&(ident("z_session_undeclare"), ident("k"))));
    }

    #[test]
    fn default_constructor_skips_accessor_and_explicit_construct_errors() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
            "fn z_keyexpr_clone(ke: &ZKeyExpr) -> ZKeyExpr { todo!() }",
        ]);
        let accessor: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_clone"].iter().map(|s| ident(s)).collect();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_intersects", "z_keyexpr_clone"]
                .iter()
                .map(|s| ident(s))
                .collect();

        // `.default()` skips the accessor's `ke`, constructs the consumer's a/b.
        let mut exp = Expansions::default();
        exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        apply(&mut reg, &exp, &declared, &accessor, &Default::default()).expect("apply");
        assert!(reg
            .expansion_plans
            .contains_key(&(ident("z_keyexpr_intersects"), ident("a"))));
        assert!(!reg
            .expansion_plans
            .contains_key(&(ident("z_keyexpr_clone"), ident("ke"))));

        // An explicit per-fn input flatten on an accessor is a build error.
        let mut reg2 = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_clone(ke: &ZKeyExpr) -> ZKeyExpr { todo!() }",
        ]);
        let mut exp2 = Expansions::default();
        exp2.begin_subset(ident("z_keyexpr_clone"), ident("ke"));
        exp2.push_subset_variant(ident("z_keyexpr_try_from"));
        let err = apply(&mut reg2, &exp2, &declared, &accessor, &Default::default()).unwrap_err();
        assert!(matches!(err, ExpandError::ConstructOnAccessor { .. }));
    }

    #[test]
    fn recursive_input_nests_param_constructors() {
        // z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample, consumed
        // by z_reply_sample(sample: ZSample). ZSample's default input expands
        // the `sample` param into z_sample_new's params, each of which (ZKeyExpr,
        // ZZBytes) recursively expands per ITS default input.
        let mut reg = reg_with(&[
            "fn z_sample_new(key_expr: ZKeyExpr, payload: ZZBytes) -> ZSample { todo!() }",
            "fn z_keyexpr_try_from(s: String) -> ZKeyExpr { todo!() }",
            "fn z_zbytes_from_vec(b: Vec<u8>) -> ZZBytes { todo!() }",
            "fn z_reply_sample(sample: ZSample) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        // Default inputs for ZSample (single), ZKeyExpr (combined: try_from|id),
        // ZZBytes (single).
        exp.ensure_default_constructor(syn::parse_quote!(ZSample));
        exp.add_constructor_variant(ident("z_sample_new"));
        exp.ensure_default_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.add_constructor_variant_id();
        exp.ensure_default_constructor(syn::parse_quote!(ZZBytes));
        exp.add_constructor_variant(ident("z_zbytes_from_vec"));
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_reply_sample"].iter().map(|s| ident(s)).collect();
        apply(
            &mut reg,
            &exp,
            &declared,
            &Default::default(),
            &Default::default(),
        )
        .expect("apply");

        let plan = reg
            .expansion_plans
            .get(&(ident("z_reply_sample"), ident("sample")))
            .expect("sample plan");
        // Top: single z_sample_new ctor, 2 args, both recursive Build.
        assert_eq!(plan.selector, None);
        assert_eq!(plan.variants.len(), 1);
        let args = &plan.variants[0].inputs;
        assert_eq!(args.len(), 2);
        assert!(
            matches!(args[0], FoldArg::Build(_)),
            "key_expr is a nested build"
        );
        assert!(
            matches!(args[1], FoldArg::Build(_)),
            "payload is a nested build"
        );
        // key_expr's nested build is COMBINED (try_from | identity ⇒ selector).
        if let FoldArg::Build(b) = &args[0] {
            assert!(b.selector.is_some(), "ZKeyExpr default input is combined");
            assert_eq!(b.variants.len(), 2);
        }
        // payload's nested build is SINGLE (no selector).
        if let FoldArg::Build(b) = &args[1] {
            assert!(b.selector.is_none(), "ZZBytes default input is single");
        }
        // Wire leaves: key-expr selector + try_from String + identity ZKeyExpr +
        // zbytes Vec<u8> — all flattened into the one signature.
        let leaf_tys: Vec<String> = plan
            .leaves
            .iter()
            .map(|l| l.ty.to_token_stream().to_string())
            .collect();
        assert!(
            leaf_tys.iter().any(|t| t.contains("i32")),
            "selector leaf: {leaf_tys:?}"
        );
        assert!(
            leaf_tys.iter().any(|t| t.contains("String")),
            "try_from arg: {leaf_tys:?}"
        );
    }

    #[test]
    fn recursive_input_cycle_errors() {
        // A → B → A constructor cycle is a build error (not an infinite expansion).
        let mut reg = reg_with(&[
            "fn make_a(b: B) -> A { todo!() }",
            "fn make_b(a: A) -> B { todo!() }",
            "fn consume_a(a: A) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.ensure_default_constructor(syn::parse_quote!(A));
        exp.add_constructor_variant(ident("make_a"));
        exp.ensure_default_constructor(syn::parse_quote!(B));
        exp.add_constructor_variant(ident("make_b"));
        let declared: std::collections::HashSet<syn::Ident> =
            ["consume_a"].iter().map(|s| ident(s)).collect();
        let err = apply(
            &mut reg,
            &exp,
            &declared,
            &Default::default(),
            &Default::default(),
        )
        .unwrap_err();
        assert!(matches!(err, ExpandError::InputCycle { .. }), "got {err:?}");
    }
}
