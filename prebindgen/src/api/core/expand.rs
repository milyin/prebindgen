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
//! Two flavours:
//! * **Single** (`.constructor(f)`): the parameter becomes `f`'s parameters
//!   directly.
//! * **Combined** (`.combined_constructor(T)` + `.combined_variant(f)` /
//!   `.combined_variant_id()`): the parameter becomes a runtime selector
//!   (`i32`) plus one `Option`-wrapped input group per variant. The identity
//!   variant passes an already-built `T` straight through.
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

#[derive(Clone)]
struct SingleDecl {
    func: syn::Ident,
}

/// One arm of a combined constructor.
#[derive(Clone)]
enum Variant {
    /// Build the target by calling this constructor function.
    Ctor(syn::Ident),
    /// Pass an already-built target value straight through.
    Identity,
}

#[derive(Clone)]
struct CombinedDecl {
    name: Option<String>,
    target: syn::Type,
    variants: Vec<Variant>,
}

/// How an `.expand`/`.expand_with` chooses the constructor for a parameter.
#[derive(Clone)]
enum ExpandSel {
    /// Use the target's unique top-level constructor (error if ambiguous).
    TopLevel,
    /// Use the constructor named by this ident — a single constructor's
    /// function ident, or a combined constructor's `name`.
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
    singles: Vec<SingleDecl>,
    combined: Vec<CombinedDecl>,
    expands: Vec<ExpandDecl>,
    /// Cursor for the combined-constructor builder (`.combined_variant*`).
    cur_combined: Option<usize>,
}

impl Expansions {
    /// `.constructor(func)` — register a single constructor. Its target and
    /// fallibility are derived from `func`'s signature at [`apply`] time.
    pub fn add_constructor(&mut self, func: syn::Ident) {
        self.singles.push(SingleDecl { func });
        self.cur_combined = None;
    }

    /// `.combined_constructor(target)` — begin a combined constructor.
    pub fn add_combined(&mut self, target: syn::Type) {
        self.combined.push(CombinedDecl {
            name: None,
            target,
            variants: Vec::new(),
        });
        self.cur_combined = Some(self.combined.len() - 1);
    }

    /// `.combined_name(name)` — name the current combined constructor so it
    /// can be selected via `.expand_with`.
    pub fn set_combined_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_combined
            .expect(".combined_name called without a current .combined_constructor");
        self.combined[i].name = Some(name.into());
    }

    /// `.combined_variant(func)` — add a constructor-function arm.
    pub fn add_combined_variant(&mut self, func: syn::Ident) {
        let i = self
            .cur_combined
            .expect(".combined_variant called without a current .combined_constructor");
        self.combined[i].variants.push(Variant::Ctor(func));
    }

    /// `.combined_variant_id()` — add the identity arm (pass the target
    /// value straight through).
    pub fn add_combined_variant_id(&mut self) {
        let i = self
            .cur_combined
            .expect(".combined_variant_id called without a current .combined_constructor");
        self.combined[i].variants.push(Variant::Identity);
    }

    /// `.expand(param)` on the function `func` — expand `param` using the
    /// target's top-level constructor.
    pub fn add_expand(&mut self, func: syn::Ident, param: syn::Ident) {
        self.expands.push(ExpandDecl {
            func,
            param,
            sel: ExpandSel::TopLevel,
        });
        self.cur_combined = None;
    }

    /// `.expand_with(param, ctor)` — expand `param` using the explicitly
    /// named constructor (a single constructor's fn ident or a combined
    /// constructor's name).
    pub fn add_expand_with(&mut self, func: syn::Ident, param: syn::Ident, ctor: syn::Ident) {
        self.expands.push(ExpandDecl {
            func,
            param,
            sel: ExpandSel::Explicit(ctor),
        });
        self.cur_combined = None;
    }

    /// True iff nothing has been declared (lets `write_rust` skip [`apply`]).
    pub fn is_empty(&self) -> bool {
        self.expands.is_empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Resolved plan (stored on the registry, read at emission time)
// ──────────────────────────────────────────────────────────────────────

/// A resolved expansion for one `(function, parameter)`.
#[derive(Clone)]
pub struct FoldPlan {
    /// Owned type the fold produces — what the underlying call needs.
    pub target: syn::Type,
    /// True when the original parameter was `&T`: the call receives `&folded`.
    pub by_ref: bool,
    /// Flattened wire leaves, in foreign-signature order.
    pub leaves: Vec<FoldLeaf>,
    /// Index into [`Self::leaves`] of the selector leaf; `None` for a single
    /// constructor (the sole variant is applied unconditionally).
    pub selector: Option<usize>,
    /// Dispatch arms — one for a single constructor, selector order for a
    /// combined one.
    pub variants: Vec<FoldVariant>,
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
        }
    }
}

impl std::error::Error for ExpandError {}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// What an `.expand` resolved to.
enum Chosen {
    Single(syn::Ident),
    Combined(Vec<Variant>),
}

/// Resolve every `.expand` declaration into a [`FoldPlan`], register each
/// plan's leaf types as required inputs, and store the plans on the registry.
///
/// Runs inside `write_rust` after `scan_declared` and before `resolve`, so
/// leaf converters resolve through the normal rank machinery.
pub fn apply<M>(registry: &mut Registry<M>, exp: &Expansions) -> Result<(), ExpandError> {
    for ed in &exp.expands {
        let (item_fn, loc) = registry
            .functions
            .get(&ed.func)
            .cloned()
            .ok_or_else(|| ExpandError::UnknownFunction(ed.func.clone()))?;

        let param_ty = find_param_type(&item_fn, &ed.param)
            .ok_or_else(|| ExpandError::UnknownParam(ed.func.clone(), ed.param.clone()))?;

        let (by_ref, target) = match &param_ty {
            syn::Type::Reference(r) => (true, (*r.elem).clone()),
            other => (false, other.clone()),
        };
        let target_key = TypeKey::from_type(&target);

        let chosen = resolve_ctor(exp, registry, &target_key, ed)?;
        let plan = build_plan(registry, &ed.param, by_ref, &target, chosen)?;

        for leaf in &plan.leaves {
            registry.require_input(&leaf.ty, &loc);
        }
        registry
            .expansion_plans
            .insert((ed.func.clone(), ed.param.clone()), plan);
    }
    Ok(())
}

/// Pick the constructor for one `.expand`/`.expand_with` declaration.
fn resolve_ctor<M>(
    exp: &Expansions,
    registry: &Registry<M>,
    target_key: &TypeKey,
    ed: &ExpandDecl,
) -> Result<Chosen, ExpandError> {
    match &ed.sel {
        ExpandSel::Explicit(ident) => {
            // A single constructor by fn ident …
            if exp.singles.iter().any(|s| &s.func == ident) {
                return Ok(Chosen::Single(ident.clone()));
            }
            // … or a combined constructor by name.
            if let Some(c) = exp
                .combined
                .iter()
                .find(|c| c.name.as_deref() == Some(ident.to_string().as_str()))
            {
                return Ok(Chosen::Combined(c.variants.clone()));
            }
            Err(ExpandError::UnknownConstructor(ident.clone()))
        }
        ExpandSel::TopLevel => {
            // Combined constructors for this target are always roots.
            let combineds: Vec<&CombinedDecl> = exp
                .combined
                .iter()
                .filter(|c| TypeKey::from_type(&c.target) == *target_key)
                .collect();
            // Single constructors referenced as a combined variant are subsumed.
            let subsumed: HashSet<String> = combineds
                .iter()
                .flat_map(|c| c.variants.iter())
                .filter_map(|v| match v {
                    Variant::Ctor(f) => Some(f.to_string()),
                    Variant::Identity => None,
                })
                .collect();

            let mut roots: Vec<Chosen> = Vec::new();
            let mut names: Vec<String> = Vec::new();
            for c in &combineds {
                roots.push(Chosen::Combined(c.variants.clone()));
                names.push(c.name.clone().unwrap_or_else(|| "<combined>".to_string()));
            }
            for s in &exp.singles {
                let sig = ctor_signature(registry, &s.func)?;
                if TypeKey::from_type(&sig.target) == *target_key
                    && !subsumed.contains(&s.func.to_string())
                {
                    roots.push(Chosen::Single(s.func.clone()));
                    names.push(s.func.to_string());
                }
            }

            match roots.len() {
                1 => Ok(roots.into_iter().next().unwrap()),
                0 => Err(ExpandError::NoConstructor {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    target: target_key.to_string(),
                }),
                _ => Err(ExpandError::AmbiguousConstructor {
                    func: ed.func.clone(),
                    param: ed.param.clone(),
                    target: target_key.to_string(),
                    candidates: names,
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

/// Build the [`FoldPlan`] for a chosen construction.
fn build_plan<M>(
    registry: &Registry<M>,
    param: &syn::Ident,
    by_ref: bool,
    target: &syn::Type,
    chosen: Chosen,
) -> Result<FoldPlan, ExpandError> {
    let mut leaves: Vec<FoldLeaf> = Vec::new();

    match chosen {
        Chosen::Single(func) => {
            let sig = ctor_signature(registry, &func)?;
            check_target(&func, &sig.target, target)?;
            let n = sig.params.len();
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
                    ty: pty.clone(),
                });
            }
            Ok(FoldPlan {
                target: target.clone(),
                by_ref,
                leaves,
                selector: None,
                variants: vec![FoldVariant {
                    ctor: Some(func),
                    fallible: sig.fallible,
                    clone: false,
                    inputs,
                }],
            })
        }
        Chosen::Combined(decl_variants) => {
            // Selector leaf first.
            leaves.push(FoldLeaf {
                name: ident(&format!("{}_sel", param)),
                ty: syn::parse_quote!(i32),
            });
            let selector = Some(0usize);

            let mut variants: Vec<FoldVariant> = Vec::new();
            for (vi, v) in decl_variants.iter().enumerate() {
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
                        variants.push(FoldVariant {
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
                        variants.push(FoldVariant {
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
                leaves,
                selector,
                variants,
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
/// The returned expression has type `Result<plan.target, String>`. The
/// back-end routes its `Err(String)` through its own error channel.
pub fn emit_fold(
    plan: &FoldPlan,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
) -> syn::Expr {
    match plan.selector {
        None => variant_result_expr(&plan.variants[0], leaf_locals, qualify, /*optional=*/ false),
        Some(si) => {
            let sel = &leaf_locals[si];
            let arms: Vec<TokenStream> = plan
                .variants
                .iter()
                .enumerate()
                .map(|(vi, v)| {
                    let lit = vi as i32;
                    let body = variant_result_expr(v, leaf_locals, qualify, /*optional=*/ true);
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

/// Build a `Result<Target, String>` expression for one variant. When
/// `optional`, the variant's input leaves are `Option<_>` and are unwrapped
/// (a missing input yields `Err`); otherwise they are passed directly.
fn variant_result_expr(
    v: &FoldVariant,
    leaf_locals: &[syn::Ident],
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    optional: bool,
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
            if optional {
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
            if optional {
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
        exp.add_constructor(ident("z_keyexpr_try_from"));
        exp.add_expand(ident("z_keyexpr_intersects"), ident("a"));

        apply(&mut reg, &exp).expect("apply");

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
    fn combined_constructor_plan_and_fold() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.add_combined(syn::parse_quote!(ZKeyExpr));
        exp.add_combined_variant(ident("z_keyexpr_try_from"));
        exp.add_combined_variant_id();
        exp.add_expand(ident("z_keyexpr_intersects"), ident("a"));

        apply(&mut reg, &exp).expect("apply");

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
        exp.add_constructor(ident("z_keyexpr_try_from"));
        exp.add_constructor(ident("z_keyexpr_autocanonize"));
        exp.add_expand(ident("z_keyexpr_intersects"), ident("a"));

        match apply(&mut reg, &exp) {
            Err(ExpandError::AmbiguousConstructor { candidates, .. }) => {
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("expected AmbiguousConstructor, got {:?}", other.err()),
        }
    }

    #[test]
    fn combined_subsumes_single_so_unique_root() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        // Both a single and a combined that references it: the combined subsumes
        // the single, leaving exactly one root.
        exp.add_constructor(ident("z_keyexpr_try_from"));
        exp.add_combined(syn::parse_quote!(ZKeyExpr));
        exp.add_combined_variant(ident("z_keyexpr_try_from"));
        exp.add_combined_variant_id();
        exp.add_expand(ident("z_keyexpr_intersects"), ident("a"));

        apply(&mut reg, &exp).expect("subsumed single → unique root");
        let plan = reg
            .expansion_plans
            .get(&(ident("z_keyexpr_intersects"), ident("a")))
            .unwrap();
        assert_eq!(plan.selector, Some(0), "resolved to the combined root");
    }

    #[test]
    fn explicit_selection_picks_named_single() {
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_autocanonize(s: String) -> Result<ZKeyExpr, Error> { todo!() }",
            "fn z_keyexpr_intersects(a: &ZKeyExpr, b: &ZKeyExpr) -> bool { todo!() }",
        ]);
        let mut exp = Expansions::default();
        exp.add_constructor(ident("z_keyexpr_try_from"));
        exp.add_constructor(ident("z_keyexpr_autocanonize"));
        exp.add_expand_with(
            ident("z_keyexpr_intersects"),
            ident("a"),
            ident("z_keyexpr_autocanonize"),
        );

        apply(&mut reg, &exp).expect("explicit selection resolves");
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
}
