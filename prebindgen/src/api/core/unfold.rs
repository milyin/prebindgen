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

use std::collections::HashSet;

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
    /// Splice in another type's combined accessor via the accessor function
    /// `f(&T) -> &Child` (or `-> Option<&Child>`): the child type's
    /// combined-accessor leaves are flattened with the access path prefixed by
    /// `f` (and marked nullable when `f` returns `Option`).
    Nested(syn::Ident),
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

    /// `.combined_accessor_record_nested(func)` — splice another type's
    /// combined accessor via the accessor `func` (`f(&T) -> &Child` or
    /// `-> Option<&Child>`); `Child`'s combined-accessor leaves are flattened
    /// with the access path prefixed by `func`.
    pub fn add_combined_record_nested(&mut self, func: syn::Ident) {
        let i = self
            .cur_combined
            .expect(".combined_accessor_record_nested called without a current .combined_accessor");
        self.combined[i].records.push(AccRecord::Nested(func));
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
    /// skipped); `Some` ⇒ decompose the inner.
    Optional(Box<UnfoldShape>),
    /// `Vec<T>` return: deliver each element (whole, via its own output
    /// converter + projection — see [`UnfoldPlan::element`]) to a caller-supplied
    /// **fold** `(acc, element) -> acc`, threading the accumulator. The inner
    /// shape is `Decompose` (a degenerate single whole-element step; per-element
    /// combined-accessor decomposition is future work).
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
    /// Flattened output leaves, in builder-argument order. Populated for
    /// `Decompose`/`Optional` (combined-accessor decomposition); **empty** for
    /// `Iterable`, which delivers each element whole (see [`Self::element`]).
    pub leaves: Vec<UnfoldLeaf>,
    /// For an `Iterable` plan: the owned/ref element type, delivered to the fold
    /// via its own output converter + projection (not decomposed). `None` for
    /// `Decompose`/`Optional`.
    pub element: Option<syn::Type>,
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
    /// `true` when a nesting accessor on [`Self::path`] returns `Option` (M3):
    /// the reached value may be absent, so the leaf is `null` (Kotlin type gets
    /// a `?`; emit wraps the encode in a `match Some/None`).
    pub nullable: bool,
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
    /// A nested combined accessor recurses back into a type already on the
    /// nesting chain (`A → … → A`).
    Cycle {
        target: String,
    },
    /// A shape / record kind not yet implemented (staged for M4).
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
            UnfoldError::Cycle { target } => write!(
                f,
                "expand_output: nested combined accessors form a cycle through `{}`",
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

        // `Vec<T>` return (M4) → `Iterable`: each element is delivered WHOLE via
        // its own output converter + projection (no combined accessor, no
        // decomposition). The other shapes (`Option`/scalar) decompose via a
        // combined accessor (M1–M3). `Option<Vec>` / `Vec<Option>` combinations
        // are not supported.
        let plan = if let Some(inner) = vec_inner_type(&ret_ty) {
            if option_inner_type(&inner).is_some() {
                return Err(UnfoldError::Unsupported {
                    func: ed.func.clone(),
                    reason: "Vec<Option<…>> returns",
                });
            }
            // Keep the element type exactly as written (`T` or `&T`): its own
            // output converter is what `into_iter()` will feed (owned `T`, or
            // `&T` for a borrowed element). No decomposition / identity, so
            // `by_ref` is informational only.
            let by_ref = matches!(&inner, syn::Type::Reference(_));
            registry.require_output(&inner, &loc);
            UnfoldPlan {
                source: inner.clone(),
                by_ref,
                shape: UnfoldShape::Iterable(Box::new(UnfoldShape::Decompose)),
                leaves: vec![],
                element: Some(inner.clone()),
            }
        } else {
            let (optional, core_ty) = match option_inner_type(&ret_ty) {
                Some(inner) => (true, inner),
                None => (false, ret_ty.clone()),
            };
            let (by_ref, source) = match &core_ty {
                syn::Type::Reference(r) => (true, (*r.elem).clone()),
                other => (false, other.clone()),
            };
            let source_key = TypeKey::from_type(&source);
            let shape = if optional {
                UnfoldShape::Optional(Box::new(UnfoldShape::Decompose))
            } else {
                UnfoldShape::Decompose
            };
            let records = resolve_combined(acc, &source_key, ed)?;
            let plan = build_plan(acc, registry, ed, by_ref, &source, shape, &records)?;
            for leaf in &plan.leaves {
                registry.require_output(&leaf.out_ty, &loc);
            }
            plan
        };
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
        AccSel::TopLevel => find_combined_by_type(acc, source_key).map(<[AccRecord]>::to_vec).map_err(
            |candidates| match candidates {
                None => UnfoldError::NoCombinedAccessor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                },
                Some(candidates) => UnfoldError::AmbiguousCombinedAccessor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                    candidates,
                },
            },
        ),
    }
}

/// Find the unique combined accessor whose target is `type_key`. `Err(None)` =
/// none registered; `Err(Some(candidates))` = ambiguous (>1). Used for both the
/// top-level `.expand_output` and nested-record resolution.
fn find_combined_by_type<'a>(
    acc: &'a Accessors,
    type_key: &TypeKey,
) -> Result<&'a [AccRecord], Option<Vec<String>>> {
    let matches: Vec<&CombinedAccessorDecl> = acc
        .combined
        .iter()
        .filter(|c| TypeKey::from_type(&c.target) == *type_key)
        .collect();
    match matches.len() {
        1 => Ok(&matches[0].records),
        0 => Err(None),
        _ => Err(Some(
            matches
                .iter()
                .map(|c| c.name.clone().unwrap_or_else(|| "<combined>".to_string()))
                .collect(),
        )),
    }
}

/// Build the [`UnfoldPlan`] for a chosen combined accessor. `shape` is the outer
/// shape over the core decomposition (`Decompose` for `T`/`&T`,
/// `Optional(Decompose)` for `Option<T>`/`Option<&T>`). The records are
/// recursively flattened ([`flatten`]) — nested combined accessors contribute
/// their leaves with the access path prefixed.
fn build_plan<M>(
    acc: &Accessors,
    registry: &Registry<M>,
    ed: &ExpandOutputDecl,
    by_ref: bool,
    source: &syn::Type,
    shape: UnfoldShape,
    records: &[AccRecord],
) -> Result<UnfoldPlan, UnfoldError> {
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    let mut visited: HashSet<TypeKey> = HashSet::new();
    visited.insert(TypeKey::from_type(source));
    flatten(
        acc,
        registry,
        &ed.func,
        records,
        source,
        &[],
        false,
        &mut visited,
        &mut leaves,
    )?;

    Ok(UnfoldPlan {
        source: source.clone(),
        by_ref,
        shape,
        leaves,
        element: None,
    })
}

/// Recursively flatten a combined accessor's records into [`UnfoldLeaf`]s.
///
/// * `source` — the type whose combined accessor `records` belong to (the root
///   on the first call, a nested child type on recursion).
/// * `path_prefix` — accessor chain from the root value to `source` (empty at
///   the root; `[…, nesting_accessor]` when recursing into a nested child).
/// * `nullable` — `true` once any nesting accessor on the path returned
///   `Option` (the reached value may be absent ⇒ the leaf is `null`).
/// * `visited` — type keys on the current nesting chain (cycle guard; entries
///   are removed after each nested recursion so sibling records may reuse a type).
#[allow(clippy::too_many_arguments)]
fn flatten<M>(
    acc: &Accessors,
    registry: &Registry<M>,
    top_func: &syn::Ident,
    records: &[AccRecord],
    source: &syn::Type,
    path_prefix: &[syn::Ident],
    nullable: bool,
    visited: &mut HashSet<TypeKey>,
    leaves: &mut Vec<UnfoldLeaf>,
) -> Result<(), UnfoldError> {
    let source_key = TypeKey::from_type(source);
    // Identity uniqueness is per combined accessor (one move/clone of the value
    // at this level); nested levels each get their own identity budget.
    let mut seen_identity = false;

    for rec in records {
        match rec {
            AccRecord::Identity => {
                if seen_identity {
                    return Err(UnfoldError::MultipleIdentity {
                        target: source_key.to_string(),
                    });
                }
                seen_identity = true;
                // The value reached by `path_prefix` (the root itself when
                // empty). Its Kotlin type + projection come from the `&source`
                // output converter (borrowed-opaque → cloned handle); emit
                // special-cases the owned-`T` move at the root.
                leaves.push(UnfoldLeaf {
                    name: ident(&format!("__leaf{}", leaves.len())),
                    path: path_prefix.to_vec(),
                    out_ty: syn::parse_quote!(&#source),
                    identity: true,
                    nullable,
                });
            }
            AccRecord::Acc(func) => {
                let (takes, ret) = accessor_signature(registry, func)?;
                check_takes(func, &takes, source)?;
                let mut path = path_prefix.to_vec();
                path.push(func.clone());
                leaves.push(UnfoldLeaf {
                    name: ident(&format!("__leaf{}", leaves.len())),
                    path,
                    out_ty: ret,
                    identity: false,
                    nullable,
                });
            }
            AccRecord::Nested(func) => {
                let (takes, ret) = accessor_signature(registry, func)?;
                check_takes(func, &takes, source)?;
                // Peel an `Option` (nested value may be absent) then a leading
                // `&` to reach the nested child type.
                let (opt, core) = match option_inner_type(&ret) {
                    Some(inner) => (true, inner),
                    None => (false, ret),
                };
                let child_ty = match &core {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    other => other.clone(),
                };
                let child_key = TypeKey::from_type(&child_ty);
                if !visited.insert(child_key.clone()) {
                    return Err(UnfoldError::Cycle {
                        target: child_key.to_string(),
                    });
                }
                let child_records =
                    find_combined_by_type(acc, &child_key).map_err(|_| {
                        UnfoldError::NoCombinedAccessor {
                            func: top_func.clone(),
                            target: child_key.to_string(),
                        }
                    })?;
                let mut child_path = path_prefix.to_vec();
                child_path.push(func.clone());
                flatten(
                    acc,
                    registry,
                    top_func,
                    child_records,
                    &child_ty,
                    &child_path,
                    nullable || opt,
                    visited,
                    leaves,
                )?;
                // Chain-scoped: let a sibling record reuse the same child type.
                visited.remove(&child_key);
            }
        }
    }

    Ok(())
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
    fn combined_accessor_optional_primitive() {
        // M2: `z_sample_timestamp(&ZSample) -> Option<&ZTimestamp>` decomposed
        // into a single primitive leaf `z_timestamp_ntp64(&ZTimestamp) -> i64`
        // (no identity). Outer shape is `Optional(Decompose)`.
        let mut reg = reg_with(&[
            "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
            "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
        ]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZTimestamp));
        acc.add_combined_record(ident("z_timestamp_ntp64"));
        acc.add_expand_output(ident("z_sample_timestamp"));

        apply(&mut reg, &acc).expect("apply");

        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_timestamp"))
            .expect("plan");
        assert!(plan.by_ref, "inner was &ZTimestamp");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZTimestamp");
        assert!(
            matches!(&plan.shape, UnfoldShape::Optional(inner) if matches!(**inner, UnfoldShape::Decompose)),
            "outer shape is Optional(Decompose)"
        );
        assert_eq!(plan.leaves.len(), 1);
        assert!(!plan.leaves[0].identity);
        assert_eq!(plan.leaves[0].path[0].to_string(), "z_timestamp_ntp64");
        assert_eq!(plan.leaves[0].out_ty.to_token_stream().to_string(), "i64");
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(i64))));
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

    #[test]
    fn nested_combined_accessor_flatten() {
        // M3: `z_reply_sample -> Option<&ZSample>` whose ZSample combined
        // accessor nests ZKeyExpr (handle+string), ZZBytes (bytes), and a
        // nullable ZTimestamp (Option<&ZTimestamp> → ntp64), plus a direct enum
        // leaf. Verifies path prefixes + nullable propagation.
        let mut reg = reg_with(&[
            "fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_sample_payload(s: &ZSample) -> &ZZBytes { todo!() }",
            "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
            "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
            "fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> { todo!() }",
            "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
        ]);
        let mut acc = Accessors::default();
        // Child combined accessors (reused via nesting).
        acc.add_combined(syn::parse_quote!(ZKeyExpr));
        acc.add_combined_record_id();
        acc.add_combined_record(ident("z_keyexpr_as_str"));
        acc.add_combined(syn::parse_quote!(ZZBytes));
        acc.add_combined_record(ident("z_zbytes_to_bytes"));
        acc.add_combined(syn::parse_quote!(ZTimestamp));
        acc.add_combined_record(ident("z_timestamp_ntp64"));
        // Parent combined accessor with nested + direct records.
        acc.add_combined(syn::parse_quote!(ZSample));
        acc.add_combined_record_nested(ident("z_sample_key_expr"));
        acc.add_combined_record_nested(ident("z_sample_payload"));
        acc.add_combined_record(ident("z_sample_kind"));
        acc.add_combined_record_nested(ident("z_sample_timestamp"));
        acc.add_expand_output(ident("z_reply_sample"));

        apply(&mut reg, &acc).expect("apply");
        let plan = reg.unfold_plans.get(&ident("z_reply_sample")).expect("plan");
        assert!(plan.by_ref);
        assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
        assert!(matches!(&plan.shape, UnfoldShape::Optional(_)));

        let path = |l: &UnfoldLeaf| {
            l.path
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(".")
        };
        // keyexpr identity (path [z_sample_key_expr]) + string + payload bytes
        // + kind enum + nullable timestamp ntp64.
        assert_eq!(plan.leaves.len(), 5);
        assert!(plan.leaves[0].identity);
        assert_eq!(path(&plan.leaves[0]), "z_sample_key_expr");
        assert_eq!(path(&plan.leaves[1]), "z_sample_key_expr.z_keyexpr_as_str");
        assert_eq!(path(&plan.leaves[2]), "z_sample_payload.z_zbytes_to_bytes");
        assert_eq!(path(&plan.leaves[3]), "z_sample_kind");
        assert_eq!(plan.leaves[3].out_ty.to_token_stream().to_string(), "SampleKind");
        assert_eq!(path(&plan.leaves[4]), "z_sample_timestamp.z_timestamp_ntp64");
        // Only the timestamp leaf (Option nesting accessor) is nullable.
        assert!(!plan.leaves[1].nullable && !plan.leaves[2].nullable);
        assert!(plan.leaves[4].nullable);
    }

    #[test]
    fn nested_cycle_errors() {
        // A → B → A nesting is rejected.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZA { todo!() }",
            "fn a_to_b(a: &ZA) -> &ZB { todo!() }",
            "fn b_to_a(b: &ZB) -> &ZA { todo!() }",
        ]);
        let mut acc = Accessors::default();
        acc.add_combined(syn::parse_quote!(ZA));
        acc.add_combined_record_nested(ident("a_to_b"));
        acc.add_combined(syn::parse_quote!(ZB));
        acc.add_combined_record_nested(ident("b_to_a"));
        acc.add_expand_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::Cycle { .. }));
    }

    #[test]
    fn iterable_whole_element_plan() {
        // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
        // each element delivered WHOLE (no combined accessor, no leaves).
        let mut reg = reg_with(&[
            "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        ]);
        let mut acc = Accessors::default();
        acc.add_expand_output(ident("z_session_peers_zid"));

        apply(&mut reg, &acc).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(
            matches!(&plan.shape, UnfoldShape::Iterable(inner) if matches!(**inner, UnfoldShape::Decompose)),
            "outer shape is Iterable(Decompose)"
        );
        assert!(!plan.by_ref, "Vec<ZZenohId> owns its elements");
        assert!(plan.leaves.is_empty(), "whole-element: no decomposed leaves");
        assert_eq!(
            plan.element.as_ref().map(|t| t.to_token_stream().to_string()),
            Some("ZZenohId".to_string())
        );
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(ZZenohId))));
    }
}
