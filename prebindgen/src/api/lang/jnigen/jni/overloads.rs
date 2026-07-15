//! #52: opt-in idiomatic Kotlin overloads for `.split()` expanded params.
//!
//! A multi-variant `expand_param!(T)` crosses the wire as a selector tuple
//! (`expectedSel: Int, expected00: Long?, …`). With `.split()` the generator
//! emits, alongside that selector form, one **typed overload per variant**
//! delegating to it — turning `storageExpectSummary(s, 0, count, total, null,
//! onError)` into `storageExpectSummary(s, count, total, onError)`.
//!
//! Validation is folded into emission (in-scope-aware, so it never flags a
//! case that would not emit an overload):
//! * **≤1 split parameter per function** — overloads are the product of every
//!   split param's arms; capping at one keeps the surface finite and reduces
//!   collision detection to the sound, complete type-level check.
//! * **distinct JVM-erased arm signatures** — two variants that surface as the
//!   same erased parameter list are a platform-declaration clash; a hard build
//!   error names the offending class's variants.
//!
//! Only flat, non-optional single-level arms are overloaded; `Option<T>` /
//! `Vec<T>` params and recursively-built arms keep the selector form for that
//! function (documented fallback — there is no clean single type to offer).

use super::*;
use crate::api::core::expand::{FoldArg, FoldPlan};

impl JniGen {
    /// Whether the expanded parameter `(func, param)` producing `target` is
    /// marked `.split()`. A per-fn `.expand_param(param, …)` override for this
    /// exact `(func, param)` wins over the type-level default — the same
    /// replace semantics [`Self::build_expansions`] applies.
    pub(crate) fn is_split_param(
        &self,
        func: &syn::Ident,
        param: &syn::Ident,
        target: &syn::Type,
    ) -> bool {
        let param_s = param.to_string();
        if let Some((_, _, decl)) = self
            .fn_param_expands
            .iter()
            .find(|(f, p, _)| f == func && *p == param_s)
        {
            return decl.split;
        }
        let key = TypeKey::from_type(target);
        self.param_expand_decls
            .iter()
            .any(|d| d.key == key && d.split)
    }

    /// `true` if `simple` is the Kotlin simple name of a `value_blob`
    /// (`@JvmInline value class`) type — which erases to `ByteArray` on the
    /// JVM, so two distinct such classes share one method descriptor.
    fn is_value_blob_kotlin(&self, simple: &str) -> bool {
        self.types.values().any(|c| {
            c.value_blob
                && c.name_spec
                    .as_ref()
                    .map(|s| self.fqn_of(s))
                    .and_then(|fqn| fqn.rsplit('.').next().map(str::to_string))
                    .as_deref()
                    == Some(simple)
        })
    }
}

/// One resolved split parameter of a function.
struct SplitParam<'a> {
    /// The original Rust parameter ident (e.g. `expected`).
    param: syn::Ident,
    /// Its resolved expansion plan (multi-variant, non-optional, flat).
    plan: &'a FoldPlan,
}

/// Whether `plan` is a multi-variant expansion this feature can turn into
/// overloads: it has a selector (≥2 arms), a plain (non-`Option`) outer shape,
/// and no recursively-built arm.
fn plan_in_scope(plan: &FoldPlan) -> bool {
    plan.selector.is_some()
        && !plan.produces_option()
        && !plan
            .variants
            .iter()
            .any(|v| v.inputs.iter().any(|a| matches!(a, FoldArg::Build(_))))
}

/// Every in-scope split parameter of `f`, in signature order. Used both to
/// pick the one to overload and to enforce the ≤1-per-function rule.
fn split_params<'a>(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &'a Registry<KotlinMeta>,
) -> Vec<SplitParam<'a>> {
    let mut out = Vec::new();
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let syn::Pat::Ident(pid) = &*pt.pat else {
            continue;
        };
        let Some(plan) = registry
            .expansion_plans
            .get(&(f.sig.ident.clone(), pid.ident.clone()))
        else {
            continue;
        };
        if !ext.is_split_param(&f.sig.ident, &pid.ident, &plan.target) {
            continue;
        }
        if !plan_in_scope(plan) {
            continue;
        }
        out.push(SplitParam {
            param: pid.ident.clone(),
            plan,
        });
    }
    out
}

/// Camel-cased Kotlin names of a `#[prebindgen]` constructor's parameters, in
/// order — the idiomatic overload parameter names for a build arm.
fn ctor_param_names(f: &syn::ItemFn) -> Vec<String> {
    f.sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            syn::FnArg::Typed(pt) => match &*pt.pat {
                syn::Pat::Ident(pid) => Some(kt_param_name(&pid.ident.to_string())),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// A named type with its nullability cleared (`T?` → `T`) — a split overload
/// takes each arm's real, non-nullable constructor parameter type, unlike the
/// selector form whose per-arm slots are all nullable.
fn non_null(mut ty: kt::KtType) -> kt::KtType {
    match &mut ty {
        kt::KtType::Named { nullable, .. } | kt::KtType::Function { nullable, .. } => {
            *nullable = false
        }
    }
    ty
}

/// The JVM method descriptor fragment one overload parameter erases to —
/// value classes to `ByteArray`, primitives/`String` to themselves, other
/// classes to their FQN. Two arms collide iff their fragment lists match.
fn erased(ext: &JniGen, ty: &kt::KtType) -> String {
    let simple = ty.simple_name().unwrap_or("");
    if ext.is_value_blob_kotlin(simple) {
        return "ByteArray".to_string();
    }
    match simple {
        "Int" | "Long" | "Double" | "Float" | "Boolean" | "Byte" | "Short" | "Char" | "String"
        | "ByteArray" => simple.to_string(),
        _ => ty.leaf_name().unwrap_or(simple).to_string(),
    }
}

/// The typed overload parameters of one variant arm, paired with the flat leaf
/// index each fills in the selector form. `block` is the selector wrapper's
/// contiguous run of leaf params (index `k` = `plan.leaves[k]`). Returns
/// `None` if any input is not a flat leaf (out of scope).
fn variant_typed_params(
    registry: &Registry<KotlinMeta>,
    variant: &crate::api::core::expand::FoldVariant,
    base_param: &syn::Ident,
    block: &[kt::KtParam],
) -> Option<Vec<(kt::KtParam, usize)>> {
    let names: Vec<String> = match &variant.ctor {
        Some(cf) => {
            let (item_fn, _) = registry.functions.get(cf)?;
            ctor_param_names(item_fn)
        }
        // Identity arm: a single parameter, the target value itself, named
        // after the original Rust parameter.
        None => vec![kt_param_name(&base_param.to_string())],
    };
    let mut out = Vec::new();
    for (m, arg) in variant.inputs.iter().enumerate() {
        let FoldArg::Leaf(idx) = arg else {
            return None;
        };
        let slot = block.get(*idx)?;
        let name = names.get(m).cloned().unwrap_or_else(|| slot.name.clone());
        out.push((kt::KtParam::new(&name, non_null(slot.ty.clone())), *idx));
    }
    Some(out)
}

/// Locate the split param's contiguous leaf block in the already-rendered
/// selector wrapper's parameter list, matching by leaf name in order.
fn find_block(params: &[kt::KtParam], leaf_names: &[String]) -> Option<usize> {
    if leaf_names.is_empty() || params.len() < leaf_names.len() {
        return None;
    }
    (0..=params.len() - leaf_names.len()).find(|&s| {
        params[s..s + leaf_names.len()]
            .iter()
            .zip(leaf_names)
            .all(|(p, n)| &p.name == n)
    })
}

/// The overloads for one function, delegating to its already-rendered selector
/// wrapper `sel_fun` (same Kotlin name). Empty unless the function has one
/// in-scope split parameter. Panics (a build error) on the ≤1-per-function and
/// distinct-signature rules.
pub(crate) fn render_param_overloads(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    sel_fun: &kt::KtFun,
) -> Vec<kt::KtFun> {
    let splits = split_params(ext, f, registry);
    if splits.len() > 1 {
        let names: Vec<String> = splits.iter().map(|s| s.param.to_string()).collect();
        panic!(
            "fn `{}` has {} `.split()` parameters ({}); a function may split at most one — \
             un-split the others with a per-fn `.expand_param(name, …)` override that omits \
             `.split()`",
            f.sig.ident,
            names.len(),
            names.join(", ")
        );
    }
    let Some(sp) = splits.first() else {
        return Vec::new();
    };
    let plan = sp.plan;

    let leaf_names: Vec<String> = plan
        .leaves
        .iter()
        .map(|l| kt_param_name(&l.name.to_string()))
        .collect();
    let len = leaf_names.len();
    // Degrade to selector-only if the leaf block can't be located (a shape the
    // offset assumption doesn't cover) — never emit a malformed overload.
    let Some(start) = find_block(&sel_fun.params, &leaf_names) else {
        return Vec::new();
    };
    let block = &sel_fun.params[start..start + len];
    let sel_idx = plan.selector.expect("in-scope ⇒ selector present");

    // Build every arm's typed params first, so the distinct-signature check
    // runs before any emission.
    let mut arms: Vec<Vec<(kt::KtParam, usize)>> = Vec::with_capacity(plan.variants.len());
    for variant in &plan.variants {
        let Some(tp) = variant_typed_params(registry, variant, &sp.param, block) else {
            return Vec::new();
        };
        arms.push(tp);
    }
    // Distinct-signature (JVM erasure) check — class-attributed hard error.
    for i in 0..arms.len() {
        for j in (i + 1)..arms.len() {
            let di: Vec<String> = arms[i].iter().map(|(p, _)| erased(ext, &p.ty)).collect();
            let dj: Vec<String> = arms[j].iter().map(|(p, _)| erased(ext, &p.ty)).collect();
            if di == dj {
                panic!(
                    "expand_param!({}).split(): variants {} and {} both surface as `({})` — \
                     two overloads with the same JVM signature are a platform-declaration \
                     clash; disambiguate the constructors or drop `.split()`",
                    plan.target.to_token_stream(),
                    variant_label(&plan.variants[i]),
                    variant_label(&plan.variants[j]),
                    di.join(", "),
                );
            }
        }
    }

    let mut out = Vec::with_capacity(arms.len());
    for (i, typed) in arms.iter().enumerate() {
        // Overload signature: fixed params, the arm's typed params in place of
        // the leaf block, then the trailing fixed/onError params.
        let mut params: Vec<kt::KtParam> = Vec::new();
        params.extend_from_slice(&sel_fun.params[..start]);
        params.extend(typed.iter().map(|(p, _)| p.clone()));
        params.extend_from_slice(&sel_fun.params[start + len..]);

        // Delegation args: leaf slots get the selector value, this arm's typed
        // params, or `null`; every other param passes through by name.
        let mut leaf_arg: Vec<String> = vec!["null".to_string(); len];
        leaf_arg[sel_idx] = i.to_string();
        for (p, lidx) in typed {
            leaf_arg[*lidx] = p.name.clone();
        }
        let call_args: Vec<String> = sel_fun
            .params
            .iter()
            .enumerate()
            .map(|(j, p)| {
                if j >= start && j < start + len {
                    leaf_arg[j - start].clone()
                } else {
                    p.name.clone()
                }
            })
            .collect();

        let mut ov = kt::KtFun::new(&sel_fun.name).vis(kt::Vis::Public);
        for p in params {
            ov = ov.param(p);
        }
        if let Some(rt) = &sel_fun.ret {
            ov = ov.returns(rt.clone());
        }
        ov = ov.expr_body(kt::Code::new().line(format!(
            "{}({})",
            sel_fun.name,
            call_args.join(", ")
        )));
        out.push(ov);
    }
    out
}

/// A short label for a variant in error messages: the constructor ident, or
/// `variant_self()` for the identity arm.
fn variant_label(v: &crate::api::core::expand::FoldVariant) -> String {
    match &v.ctor {
        Some(c) => c.to_string(),
        None => "variant_self()".to_string(),
    }
}
