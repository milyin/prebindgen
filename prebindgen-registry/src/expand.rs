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

use prebindgen_flat::types_util::ident;
use proc_macro2::TokenStream;
use quote::quote;

mod plan;

pub use self::plan::{FoldArg, FoldBuild, FoldLeaf, FoldPlan, FoldShape, FoldVariant};

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
