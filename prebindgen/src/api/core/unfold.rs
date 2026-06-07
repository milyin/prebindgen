//! Output (data) expansion — the dual of constructor expansion
//! (`api/core/expand.rs`). A function returning a rich type is *decomposed*
//! by **accessors** into a set of leaf values, all delivered to a
//! foreign-supplied **builder** in a single FFI crossing (the back-end picks
//! the delivery mechanism — jnigen passes a Kotlin function-type lambda).
//!
//! An *accessor* is a `#[prebindgen]` function `f(&T) -> &F` (a reference
//! return where possible, for zero-copy). A **combined accessor**
//! (`.combined_accessor(T)` + `.combined_accessor_record(f)` /
//! `.combined_accessor_record_id()`) is a **deterministic product**: every
//! record always runs and contributes its leaf — there is no selector (unlike
//! a combined *constructor*, whose selector picks one variant). Marking a
//! function `.expand_output()` replaces its return — in the generated foreign
//! signature only — with the builder plus the combined accessor's flattened
//! leaves.
//!
//! Resolution is language-agnostic: [`apply`] turns declarations into
//! [`UnfoldPlan`]s (stored on the registry, keyed by function ident) and
//! registers every leaf's `out_ty` as a required **output** so the resolver
//! produces its converter (and projection). The jnigen back-end reads the
//! plan at the return-emission site.
//!
//! Scope is staged (see the approved plan): M1 implements the flat
//! [`UnfoldShape::Decompose`] case (identity + single-accessor records, with
//! `&T` and owned `T` returns). [`UnfoldShape::Optional`] / [`Iterable`] and
//! nested records are reserved here and rejected by [`apply`] until M2–M4.
//!
//! [`Iterable`]: UnfoldShape::Iterable

use proc_macro2::Span;

use crate::api::core::registry::{Registry, TypeKey};

// ──────────────────────────────────────────────────────────────────────
// Declarations (populated by the language builder)
// ──────────────────────────────────────────────────────────────────────

/// One record (field) of a combined accessor. A combined accessor is a
/// product: every record contributes a leaf.
#[derive(Clone)]
enum AccRecord {
    /// Read this field by calling the accessor function `f(&T) -> &F`.
    Acc(syn::Ident),
    /// The value itself — the handle/identity leaf (cloned for a `&T` return,
    /// moved for an owned `T`). At most one per combined accessor.
    Identity,
    /// Splice in another combined accessor's leaves (recursive flatten).
    /// Reserved for M3; rejected by [`apply`] for now.
    #[allow(dead_code)]
    Nested(syn::Type),
}

#[derive(Clone)]
struct CombinedAccessorDecl {
    name: Option<String>,
    target: syn::Type,
    records: Vec<AccRecord>,
}

/// How an `.expand_output`/`.expand_output_with` chooses the combined accessor
/// for a function's return type.
#[derive(Clone)]
enum AccSel {
    /// Use the return type's unique combined accessor (error if ambiguous).
    TopLevel,
    /// Use the combined accessor named by this string.
    Explicit(String),
}

#[derive(Clone)]
struct ExpandOutputDecl {
    func: syn::Ident,
    sel: AccSel,
}

/// Accessor / output-expansion declarations gathered from a language builder.
/// Embedded in each adapter that supports output expansion and handed to
/// [`apply`] via
/// [`crate::api::core::prebindgen::Prebindgen::accessors`].
#[derive(Clone, Default)]
pub struct Accessors {
    combined: Vec<CombinedAccessorDecl>,
    expands: Vec<ExpandOutputDecl>,
    /// Cursor for the combined-accessor builder (`.combined_accessor_record*`).
    cur_combined: Option<usize>,
}

impl Accessors {
    /// `.combined_accessor(target)` — begin a combined accessor for `target`.
    pub fn add_combined(&mut self, target: syn::Type) {
        self.combined.push(CombinedAccessorDecl {
            name: None,
            target,
            records: Vec::new(),
        });
        self.cur_combined = Some(self.combined.len() - 1);
    }

    /// `.combined_accessor_name(name)` — name the current combined accessor so
    /// it can be selected via `.expand_output_with`.
    pub fn set_combined_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_combined
            .expect(".combined_accessor_name called without a current .combined_accessor");
        self.combined[i].name = Some(name.into());
    }

    /// `.combined_accessor_record(func)` — add an accessor-function record.
    pub fn add_combined_record(&mut self, func: syn::Ident) {
        let i = self
            .cur_combined
            .expect(".combined_accessor_record called without a current .combined_accessor");
        self.combined[i].records.push(AccRecord::Acc(func));
    }

    /// `.combined_accessor_record_id()` — add the identity/handle record (the
    /// value itself).
    pub fn add_combined_record_id(&mut self) {
        let i = self
            .cur_combined
            .expect(".combined_accessor_record_id called without a current .combined_accessor");
        self.combined[i].records.push(AccRecord::Identity);
    }

    /// `.combined_accessor_record_nested(target)` — splice another combined
    /// accessor's leaves (M3).
    pub fn add_combined_record_nested(&mut self, target: syn::Type) {
        let i = self
            .cur_combined
            .expect(".combined_accessor_record_nested called without a current .combined_accessor");
        self.combined[i].records.push(AccRecord::Nested(target));
    }

    /// `.expand_output()` on the function `func` — decompose its return value
    /// using the return type's unique combined accessor.
    pub fn add_expand_output(&mut self, func: syn::Ident) {
        self.expands.push(ExpandOutputDecl {
            func,
            sel: AccSel::TopLevel,
        });
        self.cur_combined = None;
    }

    /// `.expand_output_with(name)` — decompose using the named combined
    /// accessor.
    pub fn add_expand_output_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.expands.push(ExpandOutputDecl {
            func,
            sel: AccSel::Explicit(name.into()),
        });
        self.cur_combined = None;
    }

    /// True iff no `.expand_output` was declared (lets `write_rust` skip
    /// [`apply`]).
    pub fn is_empty(&self) -> bool {
        self.expands.is_empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Resolved plan (stored on the registry, read at emission time)
// ──────────────────────────────────────────────────────────────────────

/// Outer shape wrapping the [core decomposition](`UnfoldShape::Decompose`).
/// The output-side analog of [`crate::api::core::expand::FoldShape`].
#[derive(Clone)]
pub enum UnfoldShape {
    /// Innermost: run the combined accessor's records on the value, producing
    /// all [leaves](`UnfoldPlan::leaves`) and invoking the builder once.
    Decompose,
    /// `Option<T>` / `Option<&T>` return: `None` ⇒ a null result (builder
    /// skipped); `Some` ⇒ decompose the inner. Reserved for M2.
    #[allow(dead_code)]
    Optional(Box<UnfoldShape>),
    /// `Vec<T>` return: map the builder over each element ⇒ `List<R>`.
    /// Reserved for M4.
    #[allow(dead_code)]
    Iterable(Box<UnfoldShape>),
}

/// A resolved output expansion for one function.
#[derive(Clone)]
pub struct UnfoldPlan {
    /// Owned core type the records decompose — the function's return after
    /// peeling `&` / `Option` / `Vec`.
    pub source: syn::Type,
    /// True when the return was `&T` / `Option<&T>`: the identity leaf clones
    /// the borrow; otherwise it moves the owned value.
    pub by_ref: bool,
    /// Outer shape over the core decomposition (`Decompose` for a plain
    /// `T`/`&T` return).
    pub shape: UnfoldShape,
    /// Flattened output leaves, in builder-argument order.
    pub leaves: Vec<UnfoldLeaf>,
}

/// One flattened output leaf of a decomposed return value.
#[derive(Clone)]
pub struct UnfoldLeaf {
    /// Internal Rust local name for the encoded leaf.
    pub name: syn::Ident,
    /// Accessor-call chain from the root value (`[]` = the identity/root
    /// itself; `[f]` = `f(&root)`; longer = nested records, M3).
    pub path: Vec<syn::Ident>,
    /// Type whose resolved **output** converter encodes this leaf — a
    /// reference type for accessors (`&str`, `&F`), `&Source` for the identity
    /// leaf (so the borrowed-opaque clone converter / projection is reused).
    pub out_ty: syn::Type,
    /// `true` for the move/clone-the-value handle leaf, emitted **last** (after
    /// every reference leaf's JVM conversion has ended its borrow).
    pub identity: bool,
}

// ──────────────────────────────────────────────────────────────────────
// Errors
// ──────────────────────────────────────────────────────────────────────

/// Errors surfaced while resolving [`Accessors`] in [`apply`].
#[derive(Debug)]
pub enum UnfoldError {
    UnknownFunction(syn::Ident),
    UnknownAccessor(syn::Ident),
    NoCombinedAccessor {
        func: syn::Ident,
        target: String,
    },
    AmbiguousCombinedAccessor {
        func: syn::Ident,
        target: String,
        candidates: Vec<String>,
    },
    UnknownCombinedAccessor {
        func: syn::Ident,
        name: String,
    },
    AccessorTargetMismatch {
        accessor: String,
        takes: String,
        expected: String,
    },
    MultipleIdentity {
        target: String,
    },
    /// A shape / record kind not yet implemented (staged for M2–M4).
    Unsupported {
        func: syn::Ident,
        reason: &'static str,
    },
}

impl std::fmt::Display for UnfoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnfoldError::UnknownFunction(name) => write!(
                f,
                "expand_output: function `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::UnknownAccessor(name) => write!(
                f,
                "expand_output: accessor `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::NoCombinedAccessor { func, target } => write!(
                f,
                "expand_output: no combined accessor registered for `{}` (return of `{}`)",
                target, func
            ),
            UnfoldError::AmbiguousCombinedAccessor {
                func,
                target,
                candidates,
            } => write!(
                f,
                "expand_output: multiple combined accessors for `{}` (return of `{}`): {} — disambiguate with `.expand_output_with`",
                target,
                func,
                candidates.join(", ")
            ),
            UnfoldError::UnknownCombinedAccessor { func, name } => write!(
                f,
                "expand_output: no combined accessor named `{}` (for `{}`)",
                name, func
            ),
            UnfoldError::AccessorTargetMismatch {
                accessor,
                takes,
                expected,
            } => write!(
                f,
                "expand_output: accessor `{}` takes `{}` but the combined accessor decomposes `{}`",
                accessor, takes, expected
            ),
            UnfoldError::MultipleIdentity { target } => write!(
                f,
                "expand_output: combined accessor for `{}` has more than one identity record",
                target
            ),
            UnfoldError::Unsupported { func, reason } => write!(
                f,
                "expand_output: `{}` not yet supported: {}",
                func, reason
            ),
        }
    }
}

impl std::error::Error for UnfoldError {}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Resolve every `.expand_output` declaration into an [`UnfoldPlan`], register
/// each leaf's `out_ty` as a required output, and store the plans on the
/// registry.
///
/// Runs inside `write_rust` after `expand::apply` and before `resolve`, so leaf
/// converters resolve through the normal rank machinery.
pub fn apply<M>(registry: &mut Registry<M>, acc: &Accessors) -> Result<(), UnfoldError> {
    for ed in &acc.expands {
        let (item_fn, loc) = registry
            .functions
            .get(&ed.func)
            .cloned()
            .ok_or_else(|| UnfoldError::UnknownFunction(ed.func.clone()))?;

        let ret_ty: syn::Type = match &item_fn.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, t) => (**t).clone(),
        };

        // Peel `Option<…>` / `Vec<…>` (reserved) then a leading `&` (borrow).
        // M1 supports only `T` / `&T`.
        if option_inner_type(&ret_ty).is_some() {
            return Err(UnfoldError::Unsupported {
                func: ed.func.clone(),
                reason: "Option<…> returns (M2)",
            });
        }
        if vec_inner_type(&ret_ty).is_some() {
            return Err(UnfoldError::Unsupported {
                func: ed.func.clone(),
                reason: "Vec<…> returns (M4)",
            });
        }
        let (by_ref, source) = match &ret_ty {
            syn::Type::Reference(r) => (true, (*r.elem).clone()),
            other => (false, other.clone()),
        };
        let source_key = TypeKey::from_type(&source);

        let records = resolve_combined(acc, &source_key, ed)?;
        let plan = build_plan(registry, ed, by_ref, &source, &records)?;

        for leaf in &plan.leaves {
            registry.require_output(&leaf.out_ty, &loc);
        }
        registry.unfold_plans.insert(ed.func.clone(), plan);
    }
    Ok(())
}

/// Pick the combined accessor (its records) for one `.expand_output`.
fn resolve_combined(
    acc: &Accessors,
    source_key: &TypeKey,
    ed: &ExpandOutputDecl,
) -> Result<Vec<AccRecord>, UnfoldError> {
    match &ed.sel {
        AccSel::Explicit(name) => acc
            .combined
            .iter()
            .find(|c| c.name.as_deref() == Some(name.as_str()))
            .map(|c| c.records.clone())
            .ok_or_else(|| UnfoldError::UnknownCombinedAccessor {
                func: ed.func.clone(),
                name: name.clone(),
            }),
        AccSel::TopLevel => {
            let matches: Vec<&CombinedAccessorDecl> = acc
                .combined
                .iter()
                .filter(|c| TypeKey::from_type(&c.target) == *source_key)
                .collect();
            match matches.len() {
                1 => Ok(matches[0].records.clone()),
                0 => Err(UnfoldError::NoCombinedAccessor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                }),
                _ => Err(UnfoldError::AmbiguousCombinedAccessor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                    candidates: matches
                        .iter()
                        .map(|c| c.name.clone().unwrap_or_else(|| "<combined>".to_string()))
                        .collect(),
                }),
            }
        }
    }
}

/// Build the [`UnfoldPlan`] for a chosen combined accessor (M1: flat).
fn build_plan<M>(
    registry: &Registry<M>,
    ed: &ExpandOutputDecl,
    by_ref: bool,
    source: &syn::Type,
    records: &[AccRecord],
) -> Result<UnfoldPlan, UnfoldError> {
    let source_key = TypeKey::from_type(source);
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    let mut seen_identity = false;

    for (i, rec) in records.iter().enumerate() {
        match rec {
            AccRecord::Identity => {
                if seen_identity {
                    return Err(UnfoldError::MultipleIdentity {
                        target: source_key.to_string(),
                    });
                }
                seen_identity = true;
                // The identity leaf's Kotlin type + projection come from the
                // `&Source` output converter (borrowed-opaque → cloned handle);
                // emit special-cases the owned-`T` move.
                leaves.push(UnfoldLeaf {
                    name: ident(&format!("__leaf{}", i)),
                    path: vec![],
                    out_ty: syn::parse_quote!(&#source),
                    identity: true,
                });
            }
            AccRecord::Acc(func) => {
                let (takes, ret) = accessor_signature(registry, func)?;
                check_takes(func, &takes, source)?;
                leaves.push(UnfoldLeaf {
                    name: ident(&format!("__leaf{}", i)),
                    path: vec![func.clone()],
                    out_ty: ret,
                    identity: false,
                });
            }
            AccRecord::Nested(_) => {
                return Err(UnfoldError::Unsupported {
                    func: ed.func.clone(),
                    reason: "nested combined accessors (M3)",
                });
            }
        }
    }

    Ok(UnfoldPlan {
        source: source.clone(),
        by_ref,
        shape: UnfoldShape::Decompose,
        leaves,
    })
}

/// An accessor `f(&T) -> R`: returns the (peeled) `T` it takes and its return
/// type `R` as written (a reference where possible).
fn accessor_signature<M>(
    registry: &Registry<M>,
    func: &syn::Ident,
) -> Result<(syn::Type, syn::Type), UnfoldError> {
    let (item_fn, _) = registry
        .functions
        .get(func)
        .ok_or_else(|| UnfoldError::UnknownAccessor(func.clone()))?;

    // First parameter is the receiver `&T`; peel the borrow to get `T`.
    let takes = item_fn
        .sig
        .inputs
        .iter()
        .find_map(|input| match input {
            syn::FnArg::Typed(pt) => Some((*pt.ty).clone()),
            _ => None,
        })
        .ok_or_else(|| UnfoldError::UnknownAccessor(func.clone()))?;
    let takes = match takes {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other,
    };
    let ret: syn::Type = match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => (**t).clone(),
    };
    Ok((takes, ret))
}

fn check_takes(
    func: &syn::Ident,
    takes: &syn::Type,
    expected: &syn::Type,
) -> Result<(), UnfoldError> {
    if TypeKey::from_type(takes) == TypeKey::from_type(expected) {
        Ok(())
    } else {
        Err(UnfoldError::AccessorTargetMismatch {
            accessor: func.to_string(),
            takes: TypeKey::from_type(takes).to_string(),
            expected: TypeKey::from_type(expected).to_string(),
        })
    }
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

/// If `ty` is `Option<Inner>` (by last path segment), return `Inner`.
fn option_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    angle_inner(ty, "Option")
}

/// If `ty` is `Vec<Inner>` (by last path segment), return `Inner`.
fn vec_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    angle_inner(ty, "Vec")
}

fn angle_inner(ty: &syn::Type, wrapper: &str) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let last = tp.path.segments.last()?;
    if last.ident != wrapper {
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

    #[test]
    fn combined_accessor_plan_byref() {
        // `z_sample_key_expr(&ZSample) -> &ZKeyExpr` decomposed into the keyexpr
        // handle (identity) + its string form (`z_keyexpr_as_str`).
        let mut reg = reg_with(&[
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record_id();
        acc.add_combined_record(ident("z_keyexpr_as_str"));
        acc.add_expand_output(ident("z_sample_key_expr"));

        apply(&mut reg, &acc).expect("apply");

        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_key_expr"))
            .expect("plan");
        assert!(plan.by_ref, "return was &ZKeyExpr");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZKeyExpr");
        assert!(matches!(plan.shape, UnfoldShape::Decompose));
        assert_eq!(plan.leaves.len(), 2);

        // Identity leaf: out_ty `&ZKeyExpr`, empty path, emitted last.
        assert!(plan.leaves[0].identity);
        assert!(plan.leaves[0].path.is_empty());
        assert_eq!(
            plan.leaves[0].out_ty.to_token_stream().to_string(),
            "& ZKeyExpr"
        );
        // Accessor leaf: out_ty `&str`, path `[z_keyexpr_as_str]`.
        assert!(!plan.leaves[1].identity);
        assert_eq!(plan.leaves[1].path.len(), 1);
        assert_eq!(plan.leaves[1].path[0].to_string(), "z_keyexpr_as_str");
        assert_eq!(
            plan.leaves[1].out_ty.to_token_stream().to_string(),
            "& str"
        );

        // Leaf out_tys registered as required outputs so the resolver builds
        // their converters.
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(&str))));
    }

    #[test]
    fn ambiguous_combined_accessor_errors() {
        let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record_id();
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record_id();
        acc.add_expand_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::AmbiguousCombinedAccessor { .. }));
    }

    #[test]
    fn accessor_target_mismatch_errors() {
        // Accessor takes a different type than the combined accessor's target.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZKeyExpr { todo!() }",
            "fn wrong(x: &ZSample) -> &str { todo!() }",
        ]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record(ident("wrong"));
        acc.add_expand_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::AccessorTargetMismatch { .. }));
    }

    #[test]
    fn multiple_identity_errors() {
        let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record_id();
        acc.add_combined_record_id();
        acc.add_expand_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::MultipleIdentity { .. }));
    }
}
