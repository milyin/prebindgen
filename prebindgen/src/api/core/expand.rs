//! Constructor expansion — fold a value's construction into the wire
//! signature of the function that consumes it, so the foreign side builds
//! the value and calls the function in a single FFI crossing.
//!
//! A *constructor* is any `#[prebindgen]` function `f(p0, …) -> T` (or
//! `-> Result<T, E>`) that builds a target type `T`. Marking a function's
//! parameter with `.expand` replaces that parameter — in the generated
//! foreign signature only — with the constructor's inputs, flattened. The
//! generated wrapper decodes those inputs, runs the constructor Rust-side
//! (the **fold**), and passes the built value to the underlying call.
//!
//! A constructor is declared `.constructor(T)` + one or more
//! `.constructor_variant(f)` / `.constructor_variant_id()`:
//! * **One `Ctor` variant** (no identity): the parameter becomes `f`'s
//!   parameters directly (no selector) — the plain "single" form.
//! * **Two or more variants** (or an identity arm): the parameter becomes a
//!   runtime selector (`i32`) plus one `Option`-wrapped input group per variant.
//!   The identity variant passes an already-built `T` straight through.
//!
//! Everything here is **language-agnostic**: the fold is pure Rust and the
//! per-leaf wire encode/decode is delegated to the back-end's existing
//! converters. [`apply`] resolves declarations into [`FoldPlan`]s (stored on
//! the registry, keyed by `(fn, param)`) and registers each leaf type as a
//! required input so the resolver produces its converter. [`emit_fold`]
//! emits the dispatch expression at the parameter-emission site.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::api::core::registry::{Registry, TypeKey};

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
    name: Option<String>,
    target: syn::Type,
    variants: Vec<Variant>,
    /// `.default()` — auto-`construct` every matching param of every declared fn.
    default: bool,
}

/// How an `.expand`/`.expand_with` chooses the constructor for a parameter.
#[derive(Clone)]
enum ExpandSel {
    /// Use the target's unique top-level constructor (error if ambiguous).
    TopLevel,
    /// Use the constructor named (via `.constructor_name`) by this ident.
    Explicit(syn::Ident),
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
    /// `.skip_default_construct(param)` opt-outs: `(fn, param)` excluded from a
    /// constructor `.default()` auto-apply.
    skip_construct: std::collections::HashSet<(syn::Ident, syn::Ident)>,
}

impl Expansions {
    /// `.constructor(target)` — begin a constructor for `target`. A single
    /// `.constructor_variant` makes it a plain (unconditional) constructor; add
    /// more variants / `.constructor_variant_id` for a selector-dispatched one.
    pub fn add_constructor(&mut self, target: syn::Type) {
        self.constructors.push(ConstructorDecl {
            name: None,
            target,
            variants: Vec::new(),
            default: false,
        });
        self.cur_constructor = Some(self.constructors.len() - 1);
    }

    /// `.default()` — auto-`construct` the current constructor's target on every
    /// matching param (type peeled of `Option`/`&` == target) of every declared
    /// fn, unless `.skip_default_construct`'d. Panics without a current
    /// `.constructor`.
    pub fn set_default(&mut self) {
        let i = self
            .cur_constructor
            .expect(".default called without a current .constructor");
        self.constructors[i].default = true;
    }

    /// `.skip_default_construct(param)` on the current `.package_fun` — exclude
    /// `(func, param)` from constructor `.default()` auto-apply.
    pub fn add_skip_default_construct(&mut self, func: syn::Ident, param: syn::Ident) {
        self.skip_construct.insert((func, param));
    }

    /// `.constructor_name(name)` — name the current constructor so it can be
    /// selected via `.expand_with`.
    pub fn set_constructor_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_constructor
            .expect(".constructor_name called without a current .constructor");
        self.constructors[i].name = Some(name.into());
    }

    /// `.constructor_variant(func)` — add a constructor-function arm.
    pub fn add_constructor_variant(&mut self, func: syn::Ident) {
        let i = self
            .cur_constructor
            .expect(".constructor_variant called without a current .constructor");
        self.constructors[i].variants.push(Variant::Ctor(func));
    }

    /// `.constructor_variant_id()` — add the identity arm (pass the target
    /// value straight through).
    pub fn add_constructor_variant_id(&mut self) {
        let i = self
            .cur_constructor
            .expect(".constructor_variant_id called without a current .constructor");
        self.constructors[i].variants.push(Variant::Identity);
    }

    /// `.construct(param)` on the function `func` — construct `param` from the
    /// target's top-level constructor.
    pub fn add_construct(&mut self, func: syn::Ident, param: syn::Ident) {
        self.expands.push(ExpandDecl {
            func,
            param,
            sel: ExpandSel::TopLevel,
        });
        self.cur_constructor = None;
    }

    /// `.construct_with(param, ctor)` — construct `param` from the constructor
    /// named `ctor` (via `.constructor_name`).
    pub fn add_construct_with(&mut self, func: syn::Ident, param: syn::Ident, ctor: syn::Ident) {
        self.expands.push(ExpandDecl {
            func,
            param,
            sel: ExpandSel::Explicit(ctor),
        });
        self.cur_constructor = None;
    }

    /// True iff nothing has been declared (lets `write_rust` skip [`apply`]). A
    /// `.default()` constructor counts (it synthesizes `construct`s).
    pub fn is_empty(&self) -> bool {
        self.expands.is_empty() && !self.constructors.iter().any(|c| c.default)
    }
}

// ──────────────────────────────────────────────────────────────────────
// Resolved plan (stored on the registry, read at emission time)
// ──────────────────────────────────────────────────────────────────────

/// Outer shape wrapping the [core construct](`FoldShape::Construct`). The
/// value-side analog of how the rank-1 `Option<_>` / `Vec<_>` handlers compose
/// converters at the wire — mirrors jnigen's `FoldStrategy` (`jni/fold.rs`).
/// The innermost `Construct` builds the target from the decoded leaves; each
/// wrapping layer lifts that construction over `Option` / `Vec`.
#[derive(Clone)]
pub enum FoldShape {
    /// Innermost: build the target directly from the leaves — a single
    /// constructor (any arity) or a combined-selector dispatch.
    Construct,
    /// `Option<T>` / `Option<&T>` param: the single leaf is `Option<ctor_arg>`;
    /// `Some` ⇒ run the inner shape on the unwrapped value and re-wrap `Some`,
    /// `None` ⇒ `None`. Inner is always [`Self::Construct`] today (single,
    /// single-arg constructor; combined-under-`Optional` is rejected in `apply`).
    Optional(Box<FoldShape>),
    /// `Vec<T>` param: map the inner shape over each element and collect.
    /// Emit-ready and unit-tested, but not yet produced by `apply` (no `Vec<_>`
    /// param expansion is declared) — kept so the fold composes for future Vec.
    Iterable(Box<FoldShape>),
}

/// A resolved expansion for one `(function, parameter)`.
#[derive(Clone)]
pub struct FoldPlan {
    /// Owned type the core construct produces — what the underlying call needs
    /// (before any [`Self::shape`] wrapping).
    pub target: syn::Type,
    /// True when the original parameter was `&T` / `Option<&T>`: the call
    /// receives `&folded` (or `folded.as_ref()` when also optional). A
    /// call-site concern (the resolver's `&_` handler shares the inner
    /// converter the same way), not part of the fold.
    pub by_ref: bool,
    /// Outer shape over the core construct (`Construct` for a plain `T`/`&T`
    /// param; `Optional(Construct)` for `Option<T>`/`Option<&T>`).
    pub shape: FoldShape,
    /// Flattened wire leaves, in foreign-signature order.
    pub leaves: Vec<FoldLeaf>,
    /// Index into [`Self::leaves`] of the selector leaf; `None` for a single
    /// constructor (the sole variant is applied unconditionally).
    pub selector: Option<usize>,
    /// Dispatch arms — one for a single constructor, selector order for a
    /// combined one.
    pub variants: Vec<FoldVariant>,
}

impl FoldPlan {
    /// True when the fold produces an `Option<_>` (outermost shape layer is
    /// `Optional`) — drives the by-ref call-site form (`folded.as_ref()`).
    pub fn produces_option(&self) -> bool {
        matches!(self.shape, FoldShape::Optional(_))
    }
}

/// One flattened wire leaf of an expanded parameter.
#[derive(Clone)]
pub struct FoldLeaf {
    /// Foreign-side parameter name.
    pub name: syn::Ident,
    /// Rust type whose resolved **input** converter decodes this leaf. For a
    /// single constructor these are the raw constructor parameter types; for a
    /// combined one the selector (`i32`) and `Option`-wrapped variant inputs.
    pub ty: syn::Type,
}

/// One dispatch arm of a [`FoldPlan`].
#[derive(Clone)]
pub struct FoldVariant {
    /// `None` => identity (pass the decoded target value through). `Some` =>
    /// call this constructor function.
    pub ctor: Option<syn::Ident>,
    /// Whether the constructor returns `Result` (its `Err` is routed through
    /// the back-end's error channel). Always `false` for identity.
    pub fallible: bool,
    /// `true` for a borrowed identity arm (`&T` parameter): the input leaf is
    /// `Option<&T>` and the fold clones it (`T: Clone`) so the caller's handle
    /// is preserved rather than consumed. `false` otherwise.
    pub clone: bool,
    /// Indices into [`FoldPlan::leaves`] for this variant's inputs, in
    /// constructor-parameter order.
    pub inputs: Vec<usize>,
}

// ──────────────────────────────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────────────────────────────

/// Errors surfaced while resolving [`Expansions`] in [`apply`].
#[derive(Debug)]
pub enum ExpandError {
    UnknownFunction(syn::Ident),
    UnknownParam(syn::Ident, syn::Ident),
    UnknownConstructor(syn::Ident),
    NoConstructor {
        func: syn::Ident,
        param: syn::Ident,
        target: String,
    },
    AmbiguousConstructor {
        func: syn::Ident,
        param: syn::Ident,
        target: String,
        candidates: Vec<String>,
    },
    TargetMismatch {
        ctor: String,
        produces: String,
        expected: String,
    },
    UnsupportedOptional {
        func: syn::Ident,
        param: syn::Ident,
        reason: &'static str,
    },
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::UnknownFunction(name) => write!(
                f,
                "expand: function `{}` is not a #[prebindgen] item",
                name
            ),
            ExpandError::UnknownParam(func, param) => write!(
                f,
                "expand: function `{}` has no parameter named `{}`",
                func, param
            ),
            ExpandError::UnknownConstructor(name) => write!(
                f,
                "expand: constructor `{}` is not a #[prebindgen] item",
                name
            ),
            ExpandError::NoConstructor {
                func,
                param,
                target,
            } => write!(
                f,
                "expand: no constructor registered for `{}` (parameter `{}` of `{}`)",
                target, param, func
            ),
            ExpandError::AmbiguousConstructor {
                func,
                param,
                target,
                candidates,
            } => write!(
                f,
                "expand: multiple independent constructors for `{}` (parameter `{}` of `{}`): {} — disambiguate with `.expand_with`",
                target,
                param,
                func,
                candidates.join(", ")
            ),
            ExpandError::TargetMismatch {
                ctor,
                produces,
                expected,
            } => write!(
                f,
                "expand: constructor `{}` produces `{}` but the parameter expects `{}`",
                ctor, produces, expected
            ),
            ExpandError::UnsupportedOptional {
                func,
                param,
                reason,
            } => write!(
                f,
                "expand: optional parameter `{}` of `{}` is not supported: {}",
                param, func, reason
            ),
        }
    }
}

impl std::error::Error for ExpandError {}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Resolve every `.construct` declaration (explicit + `.default()`
/// auto-applied) into a [`FoldPlan`], register each plan's leaf types as required
/// inputs, and store the plans on the registry. `declared_fns` is the back-end's
/// claimed `#[prebindgen]` fn set — the domain over which `.default()`
/// constructors auto-apply.
///
/// Runs inside `write_rust` after `scan_declared` and before `resolve`, so
/// leaf converters resolve through the normal rank machinery.
pub fn apply<M>(
    registry: &mut Registry<M>,
    exp: &Expansions,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), ExpandError> {
    let mut done: HashSet<(String, String)> = HashSet::new();
    for ed in &exp.expands {
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
            let Some((item_fn, _)) = registry.functions.get(func).cloned() else {
                continue;
            };
            for (pname, pty) in fn_params(&item_fn) {
                let core = option_inner_type(&pty).unwrap_or(pty);
                let bare = match &core {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    other => other.clone(),
                };
                if TypeKey::from_type(&bare) != ckey {
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
    let plan = build_plan(registry, ed, optional, by_ref, &target, &variants)?;

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
        ExpandSel::Explicit(ident) => exp
            .constructors
            .iter()
            .find(|c| c.name.as_deref() == Some(ident.to_string().as_str()))
            .map(|c| c.variants.clone())
            .ok_or_else(|| ExpandError::UnknownConstructor(ident.clone())),
        ExpandSel::TopLevel => {
            let matches: Vec<&ConstructorDecl> = exp
                .constructors
                .iter()
                .filter(|c| TypeKey::from_type(&c.target) == *target_key)
                .collect();
            match matches.len() {
                1 => Ok(matches[0].variants.clone()),
                0 => Err(ExpandError::NoConstructor {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    target: target_key.to_string(),
                }),
                _ => Err(ExpandError::AmbiguousConstructor {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    target: target_key.to_string(),
                    candidates: matches
                        .iter()
                        .map(|c| c.name.clone().unwrap_or_else(|| "<constructor>".to_string()))
                        .collect(),
                }),
            }
        }
    }
}

/// Constructor signature: parameter `(name, type)` pairs, the produced
/// (`Ok`) target type, and whether it is fallible (`-> Result<_, _>`).
fn ctor_signature<M>(
    registry: &Registry<M>,
    func: &syn::Ident,
) -> Result<CtorSig, ExpandError> {
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
    registry: &Registry<M>,
    ed: &ExpandDecl,
    optional: bool,
    by_ref: bool,
    target: &syn::Type,
    variants: &[Variant],
) -> Result<FoldPlan, ExpandError> {
    let param = &ed.param;
    let mut leaves: Vec<FoldLeaf> = Vec::new();

    if let [Variant::Ctor(func)] = variants {
        {
            let sig = ctor_signature(registry, func)?;
            check_target(func, &sig.target, target)?;
            let n = sig.params.len();
            // Optional (`Option<T>`/`Option<&T>`) is only well-defined for a
            // single-argument constructor: one nullable leaf decides presence.
            if optional && n != 1 {
                return Err(ExpandError::UnsupportedOptional {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    reason: "constructor must take exactly one argument",
                });
            }
            let mut inputs = Vec::new();
            for (pname, pty) in &sig.params {
                let name = if n == 1 {
                    param.clone()
                } else {
                    ident(&format!("{}_{}", param, pname))
                };
                inputs.push(leaves.len());
                leaves.push(FoldLeaf {
                    name,
                    // Optional param ⇒ the (single) leaf is nullable so the
                    // foreign side encodes `None` as a null wire value.
                    ty: if optional { opt(pty) } else { pty.clone() },
                });
            }
            Ok(FoldPlan {
                target: target.clone(),
                by_ref,
                shape: if optional {
                    FoldShape::Optional(Box::new(FoldShape::Construct))
                } else {
                    FoldShape::Construct
                },
                leaves,
                selector: None,
                variants: vec![FoldVariant {
                    ctor: Some(func.clone()),
                    fallible: sig.fallible,
                    clone: false,
                    inputs,
                }],
            })
        }
    } else {
        {
            if optional {
                return Err(ExpandError::UnsupportedOptional {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    reason: "selector-dispatched constructors cannot be optional",
                });
            }
            // Selector leaf first.
            leaves.push(FoldLeaf {
                name: ident(&format!("{}_sel", param)),
                ty: syn::parse_quote!(i32),
            });
            let selector = Some(0usize);

            let mut fold_variants: Vec<FoldVariant> = Vec::new();
            for (vi, v) in variants.iter().enumerate() {
                match v {
                    Variant::Ctor(func) => {
                        let sig = ctor_signature(registry, func)?;
                        check_target(func, &sig.target, target)?;
                        let np = sig.params.len();
                        let mut inputs = Vec::new();
                        for (pi, (_pname, pty)) in sig.params.iter().enumerate() {
                            let name = if np == 1 {
                                ident(&format!("{}_{}", param, vi))
                            } else {
                                ident(&format!("{}_{}_{}", param, vi, pi))
                            };
                            inputs.push(leaves.len());
                            leaves.push(FoldLeaf {
                                name,
                                ty: opt(pty),
                            });
                        }
                        fold_variants.push(FoldVariant {
                            ctor: Some(func.clone()),
                            fallible: sig.fallible,
                            clone: false,
                            inputs,
                        });
                    }
                    Variant::Identity => {
                        let idx = leaves.len();
                        // A `&T` consumer borrows: take `Option<&T>` and clone
                        // so the caller's handle survives. A by-value consumer
                        // takes ownership: `Option<T>` (consumed).
                        let leaf_ty = if by_ref {
                            opt(&syn::parse_quote!(&#target))
                        } else {
                            opt(target)
                        };
                        leaves.push(FoldLeaf {
                            name: ident(&format!("{}_{}", param, vi)),
                            ty: leaf_ty,
                        });
                        fold_variants.push(FoldVariant {
                            ctor: None,
                            fallible: false,
                            clone: by_ref,
                            inputs: vec![idx],
                        });
                    }
                }
            }

            Ok(FoldPlan {
                target: target.clone(),
                by_ref,
                shape: FoldShape::Construct,
                leaves,
                selector,
                variants: fold_variants,
            })
        }
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
/// (`Result<Target>`, `Result<Option<Target>>`, …). The back-end routes its
/// `Err(String)` through its own error channel. Folds the [`FoldShape`] layers
/// top-down over the shared [core construct](`emit_core_construct`) — the value
/// analog of how the rank-1 `Option<_>`/`Vec<_>` handlers compose at the wire.
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
        FoldShape::Construct => emit_core_construct(plan, leaf_locals, bound, qualify),
        FoldShape::Optional(inner) => {
            // The structured value is the enclosing bound var, or — at the top —
            // the single shaped leaf's decoded local (`leaf_locals[0]`).
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
    match plan.selector {
        None => variant_result_expr(&plan.variants[0], leaf_locals, qualify, /*dispatched=*/ false),
        Some(si) => {
            let sel = &leaf_locals[si];
            let arms: Vec<TokenStream> = plan
                .variants
                .iter()
                .enumerate()
                .map(|(vi, v)| {
                    let lit = vi as i32;
                    let body = variant_result_expr(v, leaf_locals, qualify, /*dispatched=*/ true);
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
    let input_locals: Vec<&syn::Ident> = v.inputs.iter().map(|&i| &leaf_locals[i]).collect();

    match &v.ctor {
        None => {
            // Identity: the sole input is the target value (or a borrow of it
            // that we clone, for `&T` consumers — preserving the caller's handle).
            let loc = input_locals[0];
            // `&*__v` derefs through whatever the borrow leaf decoded to (a
            // plain `&T`, or a back-end smart-pointer like jnigen's
            // `OwnedObject<T>`) down to `T`, then clones — keeping the caller's
            // handle alive without the core knowing the back-end's borrow type.
            let some_val: syn::Expr = if v.clone {
                syn::parse_quote!(::core::result::Result::Ok(::core::clone::Clone::clone(&*__v)))
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
                let bind: Vec<syn::Ident> = (0..input_locals.len())
                    .map(|i| ident(&format!("__p{}", i)))
                    .collect();
                let call = ctor_call_result(&path, &bind, v.fallible);
                let missing = quote!(::core::result::Result::Err(
                    ::std::string::String::from("constructor variant input missing")
                ));
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
            } else {
                let args: Vec<&syn::Ident> = input_locals;
                ctor_call_result(&path, &args, v.fallible)
            }
        }
    }
}

/// `path(args…)` lifted to `Result<Target, String>` (mapping a fallible
/// constructor's error via `Display`).
fn ctor_call_result<I: quote::ToTokens>(
    path: &syn::Path,
    args: &[I],
    fallible: bool,
) -> syn::Expr {
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

/// If `ty` is `Option<Inner>` (by last path segment), return `Inner`.
fn option_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let last = tp.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &last.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    })
}

/// If `ty` is `Result<Ok, Err>` (by last path segment), return `Ok`.
fn result_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let last = tp.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &last.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    })
}

fn opt(ty: &syn::Type) -> syn::Type {
    syn::parse_quote!(Option<#ty>)
}

fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::core::registry::Registry;
    use quote::ToTokens;

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
        // Single-method constructor = one Ctor variant (no selector).
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.add_construct(ident("z_keyexpr_intersects"), ident("a"));

        apply(&mut reg, &exp, &Default::default()).expect("apply");

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
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.add_constructor_variant_id();
        exp.add_construct(ident("z_keyexpr_intersects"), ident("a"));

        apply(&mut reg, &exp, &Default::default()).expect("apply");

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
    fn ambiguous_top_level_errors() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_autocanonize(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        // Two constructors for the same target ⇒ TopLevel is ambiguous.
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_autocanonize"));
        exp.add_construct(ident("z_keyexpr_intersects"), ident("a"));

        match apply(&mut reg, &exp, &Default::default()) {
            Err(ExpandError::AmbiguousConstructor { candidates, .. }) => {
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("expected AmbiguousConstructor, got {:?}", other.err()),
        }
    }

    #[test]
    fn explicit_selection_picks_named_constructor() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_autocanonize(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        // Two constructors for ZKeyExpr, disambiguated by name via `.expand_with`.
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.set_constructor_name("tryfrom");
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_autocanonize"));
        exp.set_constructor_name("autocanon");
        exp.add_construct_with(
            ident("z_keyexpr_intersects"),
            ident("a"),
            ident("autocanon"),
        );

        apply(&mut reg, &exp, &Default::default()).expect("explicit selection resolves");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_keyexpr_intersects"), ident("a")))
            .unwrap();
        assert_eq!(plan.selector, None);
        assert_eq!(
            plan.variants[0].ctor.as_ref().unwrap().to_string(),
            "z_keyexpr_autocanonize"
        );
    }

    #[test]
    fn optional_byvalue_single_ctor() {
        // `attachment: Option<ZZBytes>` with single `z_zbytes_from_vec(Vec<u8>)`.
        let mut reg = reg_with(&[
            "fn z_zbytes_from_vec(bytes: Vec<u8>) -> ZZBytes { todo!() }",
            "fn z_session_delete(s: &ZSession, attachment: Option<ZZBytes>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.add_constructor(syn::parse_quote!(ZZBytes));
        exp.add_constructor_variant(ident("z_zbytes_from_vec"));
        exp.add_construct(ident("z_session_delete"), ident("attachment"));

        apply(&mut reg, &exp, &Default::default()).expect("apply optional by-value");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_session_delete"), ident("attachment")))
            .unwrap();
        assert!(matches!(plan.shape, FoldShape::Optional(_)));
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
        assert!(s.contains("Some") && s.contains("None"), "maps Option: {}", s);
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
        exp.add_constructor(syn::parse_quote!(ZEncoding));
        exp.add_constructor_variant(ident("z_encoding_from_string"));
        exp.add_construct(ident("z_session_put"), ident("encoding"));

        apply(&mut reg, &exp, &Default::default()).expect("apply optional by-ref");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_session_put"), ident("encoding")))
            .unwrap();
        assert!(matches!(plan.shape, FoldShape::Optional(_)));
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
    fn optional_combined_rejected() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_session_get(s: &ZSession, ke: Option<ZKeyExpr>) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.add_constructor_variant_id();
        exp.add_construct(ident("z_session_get"), ident("ke"));

        match apply(&mut reg, &exp, &Default::default()) {
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
            shape: FoldShape::Iterable(Box::new(FoldShape::Construct)),
            leaves: vec![FoldLeaf {
                name: ident("kes"),
                ty: syn::parse_quote!(Vec<String>),
            }],
            selector: None,
            variants: vec![FoldVariant {
                ctor: Some(ident("z_keyexpr_try_from")),
                fallible: true,
                clone: false,
                inputs: vec![0],
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
        exp.add_constructor(syn::parse_quote!(ZKeyExpr));
        exp.add_constructor_variant(ident("z_keyexpr_try_from"));
        exp.set_default();
        // Opt the undeclare's `k` out (must stay a handle).
        exp.add_skip_default_construct(ident("z_session_undeclare"), ident("k"));
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_intersects", "z_session_undeclare"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &exp, &declared).expect("apply");

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
}
