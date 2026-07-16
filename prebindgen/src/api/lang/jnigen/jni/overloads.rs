//! #52: idiomatic Kotlin overloads for expanded params.
//!
//! A multi-variant `expand_param!(T)` crosses the wire as a selector tuple
//! (`expectedSel: Int, expected00: Long?, …`); the raw call site passes magic
//! ints and null-padding. Two mechanisms turn that into idiomatic Kotlin:
//!
//! * **Proactive splittability check** ([`JniGen::validate_split_declarations`]):
//!   every multi-variant `expand_param!` declaration (type-level or per-fn) is
//!   verified up front to be *splittable* — its arms surface as pairwise-distinct
//!   JVM signatures — so a function can safely request overloads. A collision is
//!   a hard build error attributed to the declaration; `.no_split()` opts out.
//! * **Per-function emission** ([`render_param_overloads`], driven by
//!   [`FunctionDecl::split_on_param`](crate::fun)): for the named split params the
//!   generator emits, alongside the selector wrapper, the **cartesian product** of
//!   their arms as typed overloads, each delegating to the selector form. The
//!   concrete product must have no two combinations sharing a JVM signature.
//!
//! Only flat, non-optional arms are splittable; an explicit `.split_on_param` on
//! an `Option<T>` / recursively-built / single-variant / unknown parameter is a
//! hard error.

use super::*;
use crate::api::core::expand::{FoldArg, FoldPlan};

impl JniGen {
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

    /// Proactively verify every multi-variant `expand_param!` declaration is
    /// splittable (its arms have pairwise-distinct JVM-erased signatures), so
    /// [`FunctionDecl::split_on_param`](crate::fun) can emit unambiguous
    /// overloads. Runs regardless of whether any function actually splits the
    /// type, so authorship errors surface early. `.no_split()` opts a decl out.
    /// A collision panics with a message attributed to the declaration.
    pub(crate) fn validate_split_declarations(&self, registry: &Registry<KotlinMeta>) {
        let type_level = self
            .param_expand_decls
            .iter()
            .map(|d| (d.key.as_str().to_string(), d));
        let per_fn = self
            .fn_param_expands
            .iter()
            .map(|(func, param, d)| (format!("fun `{func}` param `{param}`"), d));
        for (site, decl) in type_level.chain(per_fn) {
            if decl.no_split || decl.variants.len() < 2 {
                continue;
            }
            let target = decl.key.to_type();
            let sigs: Vec<(String, Vec<String>)> = decl
                .variants
                .iter()
                .map(|v| {
                    let ctor = match v {
                        LocalVariant::Ctor(c) => Some(c),
                        LocalVariant::SelfIdentity => None,
                    };
                    (
                        ctor.map(|c| c.to_string())
                            .unwrap_or_else(|| "variant_self()".to_string()),
                        arm_erased_sig(self, registry, &target, ctor),
                    )
                })
                .collect();
            for i in 0..sigs.len() {
                for j in (i + 1)..sigs.len() {
                    if sigs[i].1 == sigs[j].1 {
                        panic!(
                            "expand_param!({t}) [{site}]: variants {a} and {b} both surface as \
                             `({sig})` — a split would emit two overloads with the same JVM \
                             signature; disambiguate the constructors or add .no_split()",
                            t = decl.key.as_str(),
                            a = sigs[i].0,
                            b = sigs[j].0,
                            sig = sigs[i].1.join(", "),
                        );
                    }
                }
            }
        }
    }
}

/// The JVM-erased type list of one arm: the constructor's parameter types
/// (build arm), or the single target type (`variant_self`, `ctor == None`).
fn arm_erased_sig(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    target: &syn::Type,
    ctor: Option<&syn::Ident>,
) -> Vec<String> {
    match ctor {
        Some(cf) => match registry.functions.get(cf) {
            Some((item_fn, _)) => item_fn
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    syn::FnArg::Typed(pt) => Some(rust_type_erased(ext, registry, &pt.ty)),
                    _ => None,
                })
                .collect(),
            None => Vec::new(),
        },
        None => vec![rust_type_erased(ext, registry, target)],
    }
}

/// The JVM-erased descriptor fragment a Rust type surfaces as: a declared
/// value-blob class erases to `ByteArray`, another declared class to its simple
/// Kotlin name, a primitive/`String` to its Kotlin builtin, everything else to
/// its token string (structural fallback). References are peeled first (a `&T`
/// handle erases like `T`).
fn rust_type_erased(ext: &JniGen, registry: &Registry<KotlinMeta>, ty: &syn::Type) -> String {
    let peeled = match ty {
        syn::Type::Reference(r) => &*r.elem,
        other => other,
    };
    let key = TypeKey::from_type(peeled);
    if let Some(cfg) = ext.types.get(&key) {
        if cfg.name_spec.is_some() {
            if cfg.value_blob {
                return "ByteArray".to_string();
            }
            if let Some(fqn) = ext.kotlin_fqn(key.as_str()) {
                return fqn.rsplit('.').next().unwrap_or(&fqn).to_string();
            }
        }
    }
    if let Some(kt) = registry
        .input_entry(peeled)
        .and_then(|e| e.metadata.kotlin_name.clone())
    {
        return erased(ext, &kt);
    }
    peeled.to_token_stream().to_string()
}

/// The JVM-erased descriptor fragment of an already-rendered [`kt::KtType`] —
/// value classes to `ByteArray`, primitives/`String` to themselves, other
/// classes to their simple name.
fn erased(ext: &JniGen, ty: &kt::KtType) -> String {
    let simple = ty.simple_name().unwrap_or("");
    if ext.is_value_blob_kotlin(simple) {
        return "ByteArray".to_string();
    }
    match simple {
        "Int" | "Long" | "Double" | "Float" | "Boolean" | "Byte" | "Short" | "Char" | "String"
        | "ByteArray" => simple.to_string(),
        _ => simple.to_string(),
    }
}

/// Whether `plan` is a multi-variant expansion that can be turned into
/// overloads: a selector (≥2 arms), a plain (non-`Option`) outer shape, and no
/// recursively-built arm.
fn plan_in_scope(plan: &FoldPlan) -> bool {
    plan.selector.is_some()
        && !plan.produces_option()
        && !plan
            .variants
            .iter()
            .any(|v| v.inputs.iter().any(|a| matches!(a, FoldArg::Build(_))))
}

/// One resolved split parameter of a function, positioned against the rendered
/// selector wrapper.
struct Split<'a> {
    /// The original Rust parameter ident (e.g. `expected`).
    param: syn::Ident,
    plan: &'a FoldPlan,
    /// Start of this param's contiguous leaf block in `sel_fun.params`.
    start: usize,
    /// Block length (`plan.leaves.len()`).
    len: usize,
    /// Selector-leaf index within the block.
    sel_idx: usize,
    /// Per-variant typed params: `(param, leaf-index-within-block)`.
    arms: Vec<Vec<(kt::KtParam, usize)>>,
}

/// Camel-cased Kotlin names of a `#[prebindgen]` constructor's parameters.
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

/// `origin` + Capitalized `name` (`primary` + `count` → `primaryCount`).
fn prefixed(origin: &str, name: &str) -> String {
    let mut c = name.chars();
    match c.next() {
        Some(first) => format!("{origin}{}{}", first.to_uppercase(), c.as_str()),
        None => origin.to_string(),
    }
}

/// Clear selector-dispatch nullability (`T?` → `T`). Genuinely optional
/// constructor parameters bypass this helper and retain their rendered `T?`.
fn non_null(mut ty: kt::KtType) -> kt::KtType {
    match &mut ty {
        kt::KtType::Named { nullable, .. } | kt::KtType::Function { nullable, .. } => {
            *nullable = false
        }
    }
    ty
}

fn is_option(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.last().is_some_and(|s| s.ident == "Option")
    )
}

/// The typed overload params of one variant arm, paired with the leaf index
/// each fills. `origin`/`multi` drive name disambiguation (build-arm params are
/// prefixed with the origin parameter name when the function splits more than
/// one parameter). Returns `None` if any input is not a flat leaf.
fn variant_typed_params(
    registry: &Registry<KotlinMeta>,
    variant: &crate::api::core::expand::FoldVariant,
    origin: &syn::Ident,
    block: &[kt::KtParam],
    multi: bool,
) -> Option<Vec<(kt::KtParam, usize)>> {
    let origin_kt = kt_param_name(&origin.to_string());
    let (names, optional): (Vec<String>, Vec<bool>) = match &variant.ctor {
        Some(cf) => {
            let (item_fn, _) = registry.functions.get(cf)?;
            let optional = item_fn
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    syn::FnArg::Typed(pt) => Some(is_option(&pt.ty)),
                    _ => None,
                })
                .collect();
            (ctor_param_names(item_fn), optional)
        }
        // Identity arm: one parameter, the value itself, named after the origin
        // parameter (already unique across split params — never prefixed).
        None => (vec![origin_kt.clone()], vec![false]),
    };
    let mut out = Vec::new();
    for (m, arg) in variant.inputs.iter().enumerate() {
        let FoldArg::Leaf(idx, _) = arg else {
            return None;
        };
        let slot = block.get(*idx)?;
        let base = names.get(m).cloned().unwrap_or_else(|| slot.name.clone());
        let name = if multi && variant.ctor.is_some() {
            prefixed(&origin_kt, &base)
        } else {
            base
        };
        let ty = if optional.get(m).copied().unwrap_or(false) {
            slot.ty.clone()
        } else {
            non_null(slot.ty.clone())
        };
        out.push((kt::KtParam::new(&name, ty), *idx));
    }
    Some(out)
}

/// Locate a param's contiguous leaf block in the selector wrapper's parameter
/// list, matching by leaf name in order.
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

/// Resolve one `.split_on_param` request against the rendered selector wrapper,
/// validating the parameter is expandable, multi-variant, and in-scope (all
/// hard errors — the user explicitly asked to split it).
fn resolve_split<'a>(
    registry: &'a Registry<KotlinMeta>,
    f: &syn::ItemFn,
    sel_fun: &kt::KtFun,
    param_name: &str,
    multi: bool,
) -> Split<'a> {
    let param = syn::Ident::new(param_name, Span::call_site());
    let plan = registry
        .expansion_plans
        .get(&(f.sig.ident.clone(), param.clone()))
        .unwrap_or_else(|| {
            panic!(
                "fun!({}).split_on_param(\"{param_name}\"): `{param_name}` is not an expandable \
                 parameter (it has no `expand_param!` variants)",
                f.sig.ident
            )
        });
    assert!(
        plan.selector.is_some(),
        "fun!({}).split_on_param(\"{param_name}\"): `{param_name}` has a single variant — there \
         is nothing to split (it already flattens to one signature)",
        f.sig.ident
    );
    assert!(
        !plan.produces_option(),
        "fun!({}).split_on_param(\"{param_name}\"): `{param_name}` is an `Option<_>` / `Vec<_>` \
         parameter — its overload has no clean single type; keep the selector form",
        f.sig.ident
    );
    assert!(
        plan_in_scope(plan),
        "fun!({}).split_on_param(\"{param_name}\"): `{param_name}` has a recursively-built arm — \
         it cannot be overloaded; keep the selector form",
        f.sig.ident
    );
    let leaf_names: Vec<String> = plan
        .leaves
        .iter()
        .map(|l| kt_param_name(&l.name.to_string()))
        .collect();
    let len = leaf_names.len();
    let start = find_block(&sel_fun.params, &leaf_names).unwrap_or_else(|| {
        panic!(
            "fun!({}).split_on_param(\"{param_name}\"): could not locate the parameter's leaf \
             block in the generated wrapper",
            f.sig.ident
        )
    });
    let block = &sel_fun.params[start..start + len];
    let sel_idx = plan.selector.expect("selector present");
    let arms: Vec<Vec<(kt::KtParam, usize)>> = plan
        .variants
        .iter()
        .map(|v| {
            variant_typed_params(registry, v, &param, block, multi).unwrap_or_else(|| {
                panic!(
                    "fun!({}).split_on_param(\"{param_name}\"): an arm has a non-flat input; \
                     it cannot be overloaded",
                    f.sig.ident
                )
            })
        })
        .collect();
    Split {
        param,
        plan,
        start,
        len,
        sel_idx,
        arms,
    }
}

/// The overloads for one function, delegating to its already-rendered selector
/// wrapper `sel_fun`. Empty unless the function has `.split_on_param` requests.
/// Emits the cartesian product of the named params' arms; panics (a build
/// error) if the product has two combinations with the same JVM signature.
pub(crate) fn render_param_overloads(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    sel_fun: &kt::KtFun,
) -> Vec<kt::KtFun> {
    // Requested split params for this function, in signature order.
    let requested: Vec<String> = {
        let want: std::collections::HashSet<&str> = ext
            .fn_split_params
            .iter()
            .filter(|(func, _)| func == &f.sig.ident)
            .map(|(_, p)| p.as_str())
            .collect();
        if want.is_empty() {
            return Vec::new();
        }
        f.sig
            .inputs
            .iter()
            .filter_map(|a| match a {
                syn::FnArg::Typed(pt) => match &*pt.pat {
                    syn::Pat::Ident(pid) if want.contains(pid.ident.to_string().as_str()) => {
                        Some(pid.ident.to_string())
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect()
    };
    // Any requested name that didn't match a real parameter is a typo — surface
    // it rather than silently dropping.
    for (func, p) in &ext.fn_split_params {
        if func == &f.sig.ident && !requested.iter().any(|r| r == p) {
            panic!(
                "fun!({}).split_on_param(\"{p}\"): no parameter named `{p}` on this function",
                f.sig.ident
            );
        }
    }

    let multi = requested.len() > 1;
    let splits: Vec<Split> = requested
        .iter()
        .map(|name| resolve_split(registry, f, sel_fun, name, multi))
        .collect();

    // Cartesian product of arm indices across all split params.
    let combos = cartesian(&splits.iter().map(|s| s.arms.len()).collect::<Vec<_>>());

    // Product-global JVM-signature collision check (fixed params are identical
    // across every overload, so only the split-arm lists can collide).
    let sigs: Vec<Vec<String>> = combos
        .iter()
        .map(|combo| {
            splits
                .iter()
                .zip(combo)
                .flat_map(|(s, &ai)| {
                    let ctor = s.plan.variants[ai].ctor.as_ref();
                    arm_erased_sig(ext, registry, &s.plan.target, ctor)
                })
                .collect()
        })
        .collect();
    for i in 0..sigs.len() {
        for j in (i + 1)..sigs.len() {
            if sigs[i] == sigs[j] {
                panic!(
                    "fun!({}): split_on_param product is ambiguous — combinations {} and {} both \
                     surface as `({})`; add .no_split() intent is not enough here, disambiguate \
                     the constructors or drop one .split_on_param",
                    f.sig.ident,
                    combo_label(&splits, &combos[i]),
                    combo_label(&splits, &combos[j]),
                    sigs[i].join(", "),
                );
            }
        }
    }

    // Emit one overload per product combination.
    let n = sel_fun.params.len();
    let mut out = Vec::with_capacity(combos.len());
    for combo in &combos {
        // Per-split delegation slots, keyed by block start.
        let mut params: Vec<kt::KtParam> = Vec::new();
        let mut call_args: Vec<String> = Vec::new();
        let mut pos = 0usize;
        while pos < n {
            if let Some((si, s)) = splits.iter().enumerate().find(|(_, s)| s.start == pos) {
                // Replace this param's whole leaf block with the chosen arm's
                // typed params; fill the block's delegation slots.
                let ai = combo[si];
                let typed = &s.arms[ai];
                let mut leaf_arg: Vec<String> = vec!["null".to_string(); s.len];
                leaf_arg[s.sel_idx] = ai.to_string();
                for (p, lidx) in typed {
                    params.push(p.clone());
                    leaf_arg[*lidx] = p.name.clone();
                }
                call_args.extend(leaf_arg);
                pos += s.len;
            } else {
                // A fixed (or non-split expanded) param — passes through.
                let p = &sel_fun.params[pos];
                params.push(p.clone());
                call_args.push(p.name.clone());
                pos += 1;
            }
        }

        // Guard against a param-name clash (e.g. an arm name colliding with a
        // fixed param) rather than emitting uncompilable Kotlin.
        let mut seen = std::collections::HashSet::new();
        for p in &params {
            assert!(
                seen.insert(p.name.clone()),
                "fun!({}): split overload has a duplicate parameter name `{}` — rename the \
                 constructor parameter",
                f.sig.ident,
                p.name
            );
        }

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

/// Cartesian product of index ranges `0..counts[k]`, as a list of index
/// tuples. `[2, 2]` → `[[0,0],[0,1],[1,0],[1,1]]`.
fn cartesian(counts: &[usize]) -> Vec<Vec<usize>> {
    let mut acc = vec![Vec::new()];
    for &c in counts {
        acc = acc
            .into_iter()
            .flat_map(|prefix| {
                (0..c).map(move |i| {
                    let mut next = prefix.clone();
                    next.push(i);
                    next
                })
            })
            .collect();
    }
    acc
}

/// A `param=variant` label for one product combination, for error messages.
fn combo_label(splits: &[Split], combo: &[usize]) -> String {
    let parts: Vec<String> = splits
        .iter()
        .zip(combo)
        .map(|(s, &ai)| {
            let v = match &s.plan.variants[ai].ctor {
                Some(c) => c.to_string(),
                None => "variant_self()".to_string(),
            };
            format!("{}={v}", s.param)
        })
        .collect();
    format!("({})", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;

    #[test]
    fn split_params_preserve_constructor_option_nullability() {
        let ctor: syn::ItemFn = syn::parse_quote! {
            pub fn z_summary_optional(count: Option<i64>, total: f64) -> ZSummary {
                unimplemented!()
            }
        };
        let registry = Registry::<KotlinMeta>::from_items(vec![(
            syn::Item::Fn(ctor),
            SourceLocation::default(),
        )])
        .expect("index constructor");
        let variant = crate::api::core::expand::FoldVariant {
            ctor: Some(syn::parse_quote!(z_summary_optional)),
            fallible: false,
            clone: false,
            inputs: vec![FoldArg::Leaf(0, false), FoldArg::Leaf(1, false)],
        };
        // Both slots are nullable in the selector wrapper. Only the first is
        // nullable in the constructor's actual signature.
        let block = vec![
            kt::KtParam::new("expected0", kt::KtType::long().nullable()),
            kt::KtParam::new("expected1", kt::KtType::cls("Double").nullable()),
        ];

        let params = variant_typed_params(
            &registry,
            &variant,
            &syn::parse_quote!(expected),
            &block,
            false,
        )
        .expect("flat arm");

        assert_eq!(params[0].0.ty.to_string(), "Long?");
        assert_eq!(params[1].0.ty.to_string(), "Double");
    }
}
