//! Output (data) expansion — the dual of constructor expansion
//! (`api/core/expand.rs`). A function returning a rich type is *decomposed* by a
//! **deconstructor** into a set of leaf values.
//!
//! A **deconstructor** (`.deconstructor(T)` + `.deconstructor_record(f)` /
//! `.deconstructor_record_id()` / `.deconstructor_record_nested(g)`) is a
//! **deterministic product**: every record always runs and contributes its leaf
//! — there is no selector (unlike a *constructor*, whose selector picks one
//! variant). A record's accessor is a `#[prebindgen]` function `f(&T) -> &F` (a
//! reference return where possible, for zero-copy). A **converter**
//! (`.converter(T, f)`) is the single-record special case.
//!
//! Two **deliveries** (see [`Delivery`]):
//! * `.deconstruct_output()` — replaces the return with a foreign **callback**
//!   receiving all the leaves (any leaf count).
//! * `.convert_output()` — **returns** the single leaf value directly (no
//!   callback); requires a single-value deconstructor.
//!
//! Resolution is language-agnostic: [`apply`] turns declarations into
//! [`UnfoldPlan`]s (stored on the registry, keyed by function ident) and
//! registers every leaf's `out_ty` as a required **output** so the resolver
//! produces its converter (and projection). The jnigen back-end reads the
//! plan at the return-emission site.
//!
//! [`Iterable`]: UnfoldShape::Iterable

use std::collections::HashSet;

use proc_macro2::Span;

use crate::api::core::registry::{Registry, TypeKey};

// ──────────────────────────────────────────────────────────────────────
// Declarations (populated by the language builder)
// ──────────────────────────────────────────────────────────────────────

/// One record (field) of a deconstructor. A deconstructor is a product: every
/// record contributes a leaf.
#[derive(Clone)]
enum DeconRecord {
    /// Read this field by calling the accessor function `f(&T) -> &F`.
    Acc(syn::Ident),
    /// The value itself — the handle/identity leaf (cloned for a `&T` return,
    /// moved for an owned `T`, copied for a `Copy` value_blob). At most one per
    /// deconstructor.
    Identity,
    /// Splice in another type's deconstructor via the accessor function
    /// `f(&T) -> &Child` (or `-> Option<&Child>`): the child type's records are
    /// flattened with the access path prefixed by `f` (and marked nullable when
    /// `f` returns `Option`).
    Nested(syn::Ident),
}

#[derive(Clone)]
struct DeconstructorDecl {
    name: Option<String>,
    target: syn::Type,
    records: Vec<DeconRecord>,
}

/// How an output-expansion `.deconstruct_output`/`.convert_output` (and their
/// `_with` variants) chooses the deconstructor for a function's return type.
#[derive(Clone)]
enum DeconSel {
    /// Use the return type's unique deconstructor (error if ambiguous).
    TopLevel,
    /// Use the deconstructor named by this string.
    Explicit(String),
}

/// How the decomposed value(s) are delivered to the foreign side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    /// `.deconstruct_output()` — deliver the leaves to a foreign **callback**
    /// (builder / fold). Any leaf count.
    Callback,
    /// `.convert_output()` — **return** the single decomposed value directly
    /// (no callback). Requires the plan to flatten to exactly one leaf and a
    /// non-`Iterable` shape.
    Return,
}

#[derive(Clone)]
struct OutputDecl {
    func: syn::Ident,
    sel: DeconSel,
    delivery: Delivery,
}

/// Deconstructor / converter / output-expansion declarations gathered from a
/// language builder. Embedded in each adapter that supports output expansion
/// and handed to [`apply`] via
/// [`crate::api::core::prebindgen::Prebindgen::deconstructors`].
#[derive(Clone, Default)]
pub struct Deconstructors {
    deconstructors: Vec<DeconstructorDecl>,
    outputs: Vec<OutputDecl>,
    /// Cursor for the deconstructor builder (`.deconstructor_record*`).
    cur_deconstructor: Option<usize>,
}

impl Deconstructors {
    /// `.deconstructor(target)` — begin a deconstructor for `target`.
    pub fn add_deconstructor(&mut self, target: syn::Type) {
        self.deconstructors.push(DeconstructorDecl {
            name: None,
            target,
            records: Vec::new(),
        });
        self.cur_deconstructor = Some(self.deconstructors.len() - 1);
    }

    /// `.converter(target, func)` — a single-value deconstructor: one accessor
    /// record `func` (`f(&target) -> F`). Sugar for `.deconstructor(target)` +
    /// `.deconstructor_record(func)`; usable via `.convert_output` (return) and
    /// as a nested record source.
    pub fn add_converter(&mut self, target: syn::Type, func: syn::Ident) {
        self.add_deconstructor(target);
        self.add_deconstructor_record(func);
        self.cur_deconstructor = None;
    }

    /// `.deconstructor_name(name)` — name the current deconstructor so it can be
    /// selected via `.deconstruct_output_with` / `.convert_output_with`.
    pub fn set_deconstructor_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_name called without a current .deconstructor");
        self.deconstructors[i].name = Some(name.into());
    }

    /// `.deconstructor_record(func)` — add an accessor-function record.
    pub fn add_deconstructor_record(&mut self, func: syn::Ident) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Acc(func));
    }

    /// `.deconstructor_record_id()` — add the identity record (the value
    /// itself).
    pub fn add_deconstructor_record_id(&mut self) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record_id called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Identity);
    }

    /// `.deconstructor_record_nested(func)` — splice another type's
    /// deconstructor via the accessor `func` (`f(&T) -> &Child` or
    /// `-> Option<&Child>`); `Child`'s records are flattened with the access
    /// path prefixed by `func`.
    pub fn add_deconstructor_record_nested(&mut self, func: syn::Ident) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record_nested called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Nested(func));
    }

    /// `.deconstruct_output()` on the function `func` — decompose its return
    /// value via the return type's unique deconstructor and deliver the leaves
    /// to a foreign **callback**.
    pub fn add_deconstruct_output(&mut self, func: syn::Ident) {
        self.outputs.push(OutputDecl {
            func,
            sel: DeconSel::TopLevel,
            delivery: Delivery::Callback,
        });
        self.cur_deconstructor = None;
    }

    /// `.deconstruct_output_with(name)` — like [`Self::add_deconstruct_output`]
    /// but selects the deconstructor by name.
    pub fn add_deconstruct_output_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.outputs.push(OutputDecl {
            func,
            sel: DeconSel::Explicit(name.into()),
            delivery: Delivery::Callback,
        });
        self.cur_deconstructor = None;
    }

    /// `.convert_output()` on the function `func` — decompose its return value
    /// via a single-value deconstructor (converter) and **return** the value
    /// directly (no callback). Errors at [`apply`] if the plan is not single-leaf.
    pub fn add_convert_output(&mut self, func: syn::Ident) {
        self.outputs.push(OutputDecl {
            func,
            sel: DeconSel::TopLevel,
            delivery: Delivery::Return,
        });
        self.cur_deconstructor = None;
    }

    /// `.convert_output_with(name)` — like [`Self::add_convert_output`] but
    /// selects the deconstructor by name.
    pub fn add_convert_output_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.outputs.push(OutputDecl {
            func,
            sel: DeconSel::Explicit(name.into()),
            delivery: Delivery::Return,
        });
        self.cur_deconstructor = None;
    }

    /// True iff no output expansion was declared (lets `write_rust` skip
    /// [`apply`]).
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Resolved plan (stored on the registry, read at emission time)
// ──────────────────────────────────────────────────────────────────────

/// Outer shape wrapping the [core decomposition](`UnfoldShape::Decompose`).
/// The output-side analog of [`crate::api::core::expand::FoldShape`].
#[derive(Clone)]
pub enum UnfoldShape {
    /// Innermost: run the accessor's records on the value, producing
    /// all [leaves](`UnfoldPlan::leaves`) and invoking the builder once.
    Decompose,
    /// `Option<T>` / `Option<&T>` return: `None` ⇒ a null result (builder
    /// skipped); `Some` ⇒ decompose the inner.
    Optional(Box<UnfoldShape>),
    /// `Vec<T>` return: deliver each element (whole, via its own output
    /// converter + projection — see [`UnfoldPlan::element`]) to a caller-supplied
    /// **fold** `(acc, element) -> acc`, threading the accumulator. The inner
    /// shape is `Decompose` (a degenerate single whole-element step; per-element
    /// accessor decomposition is future work).
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
    /// `Decompose`/`Optional` (accessor decomposition); **empty** for
    /// `Iterable`, which delivers each element whole (see [`Self::element`]).
    pub leaves: Vec<UnfoldLeaf>,
    /// For an `Iterable` plan: the owned/ref element type, delivered to the fold
    /// via its own output converter + projection (not decomposed). `None` for
    /// `Decompose`/`Optional`.
    pub element: Option<syn::Type>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<syn::Type>,
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

/// Errors surfaced while resolving [`Deconstructors`] in [`apply`].
#[derive(Debug)]
pub enum UnfoldError {
    UnknownFunction(syn::Ident),
    UnknownAccessor(syn::Ident),
    NoDeconstructor {
        func: syn::Ident,
        target: String,
    },
    AmbiguousDeconstructor {
        func: syn::Ident,
        target: String,
        candidates: Vec<String>,
    },
    UnknownDeconstructor {
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
    /// A nested deconstructor recurses back into a type already on the nesting
    /// chain (`A → … → A`).
    Cycle {
        target: String,
    },
    /// `.convert_output()` on a deconstructor that does not flatten to exactly
    /// one leaf, or whose shape is `Iterable` (use `.deconstruct_output()`).
    ConvertNotSingle {
        func: syn::Ident,
        reason: &'static str,
    },
    /// A shape / record kind not yet implemented.
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
                "output expansion: function `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::UnknownAccessor(name) => write!(
                f,
                "output expansion: accessor `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::NoDeconstructor { func, target } => write!(
                f,
                "output expansion: no deconstructor registered for `{}` (return of `{}`)",
                target, func
            ),
            UnfoldError::AmbiguousDeconstructor {
                func,
                target,
                candidates,
            } => write!(
                f,
                "output expansion: multiple deconstructors for `{}` (return of `{}`): {} — disambiguate with `.deconstruct_output_with` / `.convert_output_with`",
                target,
                func,
                candidates.join(", ")
            ),
            UnfoldError::UnknownDeconstructor { func, name } => write!(
                f,
                "output expansion: no deconstructor named `{}` (for `{}`)",
                name, func
            ),
            UnfoldError::AccessorTargetMismatch {
                accessor,
                takes,
                expected,
            } => write!(
                f,
                "output expansion: accessor `{}` takes `{}` but the deconstructor decomposes `{}`",
                accessor, takes, expected
            ),
            UnfoldError::MultipleIdentity { target } => write!(
                f,
                "output expansion: deconstructor for `{}` has more than one identity record",
                target
            ),
            UnfoldError::Cycle { target } => write!(
                f,
                "output expansion: nested deconstructors form a cycle through `{}`",
                target
            ),
            UnfoldError::ConvertNotSingle { func, reason } => write!(
                f,
                "convert_output: `{}` is not a single-value deconstructor: {}",
                func, reason
            ),
            UnfoldError::Unsupported { func, reason } => write!(
                f,
                "output expansion: `{}` not yet supported: {}",
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
pub fn apply<M>(registry: &mut Registry<M>, acc: &Deconstructors) -> Result<(), UnfoldError> {
    for ed in &acc.outputs {
        let (item_fn, loc) = registry
            .functions
            .get(&ed.func)
            .cloned()
            .ok_or_else(|| UnfoldError::UnknownFunction(ed.func.clone()))?;

        let ret_ty: syn::Type = match &item_fn.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, t) => (**t).clone(),
        };

        // `Vec<T>` return → `Iterable`. Two element-delivery modes:
        //   * **decomposed** (M5): the element type has an accessor →
        //     flatten it into leaves, fold `(acc, leaf0, …) -> acc`.
        //   * **whole** (M4): no accessor → deliver each element whole
        //     via its own output converter + projection, fold `(acc, T) -> acc`.
        // The other shapes (`Option`/scalar) decompose via an accessor
        // (M1–M3). `Vec<Option<…>>` is not supported.
        let plan = if let Some(inner) = vec_inner_type(&ret_ty) {
            if option_inner_type(&inner).is_some() {
                return Err(UnfoldError::Unsupported {
                    func: ed.func.clone(),
                    reason: "Vec<Option<…>> returns",
                });
            }
            let shape = UnfoldShape::Iterable(Box::new(UnfoldShape::Decompose));
            // Element type peeled of a leading `&` (accessors take `&Element`).
            let (by_ref, element) = match &inner {
                syn::Type::Reference(r) => (true, (*r.elem).clone()),
                other => (false, other.clone()),
            };
            let ekey = TypeKey::from_type(&element);
            if find_deconstructor_by_type(acc, &ekey).is_ok() {
                // Decomposed: reuse the shared flatten (M3 nesting composes).
                let records = find_deconstructor_by_type(acc, &ekey)
                    .expect("checked is_ok")
                    .to_vec();
                let plan = build_plan(acc, registry, ed, by_ref, &element, shape, &records)?;
                for leaf in &plan.leaves {
                    registry.require_output(&leaf.out_ty, &loc);
                }
                plan
            } else {
                // Whole element: keep the type exactly as written so the
                // element's own output converter matches `into_iter()`'s yield.
                let by_ref = matches!(&inner, syn::Type::Reference(_));
                registry.require_output(&inner, &loc);
                UnfoldPlan {
                    source: inner.clone(),
                    by_ref,
                    shape,
                    leaves: vec![],
                    element: Some(inner.clone()),
                    delivery: ed.delivery,
                    convert_out_ty: None,
                }
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
            let records = resolve_deconstructor(acc, &source_key, ed)?;
            let plan = build_plan(acc, registry, ed, by_ref, &source, shape, &records)?;
            for leaf in &plan.leaves {
                registry.require_output(&leaf.out_ty, &loc);
            }
            plan
        };
        // `convert_output` (return delivery): require a single leaf + non-Iterable
        // shape, then resolve the leaf's `out_ty` lifted through the shape as a
        // required output so the wrapper can return it via its ordinary converter.
        let plan = if ed.delivery == Delivery::Return {
            if matches!(plan.shape, UnfoldShape::Iterable(_)) {
                return Err(UnfoldError::ConvertNotSingle {
                    func: ed.func.clone(),
                    reason: "Vec<…> returns must use .deconstruct_output() (a fold callback)",
                });
            }
            if plan.leaves.len() != 1 {
                return Err(UnfoldError::ConvertNotSingle {
                    func: ed.func.clone(),
                    reason: "a converter must flatten to exactly one leaf",
                });
            }
            let leaf_ty = plan.leaves[0].out_ty.clone();
            let cv_ty: syn::Type = if matches!(plan.shape, UnfoldShape::Optional(_)) {
                syn::parse_quote!(Option<#leaf_ty>)
            } else {
                leaf_ty
            };
            registry.require_output(&cv_ty, &loc);
            UnfoldPlan {
                convert_out_ty: Some(cv_ty),
                ..plan
            }
        } else {
            plan
        };
        registry.unfold_plans.insert(ed.func.clone(), plan);
    }
    Ok(())
}

/// Pick the deconstructor (its records) for one output expansion.
fn resolve_deconstructor(
    acc: &Deconstructors,
    source_key: &TypeKey,
    ed: &OutputDecl,
) -> Result<Vec<DeconRecord>, UnfoldError> {
    match &ed.sel {
        DeconSel::Explicit(name) => acc
            .deconstructors
            .iter()
            .find(|c| c.name.as_deref() == Some(name.as_str()))
            .map(|c| c.records.clone())
            .ok_or_else(|| UnfoldError::UnknownDeconstructor {
                func: ed.func.clone(),
                name: name.clone(),
            }),
        DeconSel::TopLevel => find_deconstructor_by_type(acc, source_key).map(<[DeconRecord]>::to_vec).map_err(
            |candidates| match candidates {
                None => UnfoldError::NoDeconstructor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                },
                Some(candidates) => UnfoldError::AmbiguousDeconstructor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                    candidates,
                },
            },
        ),
    }
}

/// Find the unique deconstructor whose target is `type_key`. `Err(None)` =
/// none registered; `Err(Some(candidates))` = ambiguous (>1). Used for both the
/// top-level output expansion and nested-record resolution.
fn find_deconstructor_by_type<'a>(
    acc: &'a Deconstructors,
    type_key: &TypeKey,
) -> Result<&'a [DeconRecord], Option<Vec<String>>> {
    let matches: Vec<&DeconstructorDecl> = acc
        .deconstructors
        .iter()
        .filter(|c| TypeKey::from_type(&c.target) == *type_key)
        .collect();
    match matches.len() {
        1 => Ok(&matches[0].records),
        0 => Err(None),
        _ => Err(Some(
            matches
                .iter()
                .map(|c| c.name.clone().unwrap_or_else(|| "<deconstructor>".to_string()))
                .collect(),
        )),
    }
}

/// Build the [`UnfoldPlan`] for a chosen accessor. `shape` is the outer
/// shape over the core decomposition (`Decompose` for `T`/`&T`,
/// `Optional(Decompose)` for `Option<T>`/`Option<&T>`). The records are
/// recursively flattened ([`flatten`]) — nested accessors contribute
/// their leaves with the access path prefixed.
fn build_plan<M>(
    acc: &Deconstructors,
    registry: &Registry<M>,
    ed: &OutputDecl,
    by_ref: bool,
    source: &syn::Type,
    shape: UnfoldShape,
    records: &[DeconRecord],
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
        by_ref,
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
        delivery: ed.delivery,
        convert_out_ty: None,
    })
}

/// Recursively flatten an accessor's records into [`UnfoldLeaf`]s.
///
/// * `source` — the type whose accessor `records` belong to (the root
///   on the first call, a nested child type on recursion).
/// * `path_prefix` — accessor chain from the root value to `source` (empty at
///   the root; `[…, nesting_accessor]` when recursing into a nested child).
/// * `by_ref` — the top-level return/element borrow-ness. The identity leaf is
///   **owned** (`source`) only at the root of an owned value (`path_prefix`
///   empty && `!by_ref`) — a `value_blob` (`Copy`) delivers itself by copy and a
///   `ptr_class` moves; everywhere else it is **borrowed** (`&source`, cloned).
/// * `nullable` — `true` once any nesting accessor on the path returned
///   `Option` (the reached value may be absent ⇒ the leaf is `null`).
/// * `visited` — type keys on the current nesting chain (cycle guard; entries
///   are removed after each nested recursion so sibling records may reuse a type).
#[allow(clippy::too_many_arguments)]
fn flatten<M>(
    acc: &Deconstructors,
    registry: &Registry<M>,
    top_func: &syn::Ident,
    records: &[DeconRecord],
    source: &syn::Type,
    path_prefix: &[syn::Ident],
    by_ref: bool,
    nullable: bool,
    visited: &mut HashSet<TypeKey>,
    leaves: &mut Vec<UnfoldLeaf>,
) -> Result<(), UnfoldError> {
    let source_key = TypeKey::from_type(source);
    // Identity uniqueness is per accessor (one move/clone of the value
    // at this level); nested levels each get their own identity budget.
    let mut seen_identity = false;

    for rec in records {
        match rec {
            DeconRecord::Identity => {
                if seen_identity {
                    return Err(UnfoldError::MultipleIdentity {
                        target: source_key.to_string(),
                    });
                }
                seen_identity = true;
                // Owned at the root of an owned value (`value_blob` copies /
                // `ptr_class` moves); borrowed (clone) otherwise. The Kotlin type
                // + projection come from this `out_ty`'s output converter.
                let out_ty: syn::Type = if path_prefix.is_empty() && !by_ref {
                    source.clone()
                } else {
                    syn::parse_quote!(&#source)
                };
                leaves.push(UnfoldLeaf {
                    name: ident(&format!("__leaf{}", leaves.len())),
                    path: path_prefix.to_vec(),
                    out_ty,
                    identity: true,
                    nullable,
                });
            }
            DeconRecord::Acc(func) => {
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
            DeconRecord::Nested(func) => {
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
                    find_deconstructor_by_type(acc, &child_key).map_err(|_| {
                        UnfoldError::NoDeconstructor {
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
                    by_ref,
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
    fn accessor_optional_primitive() {
        // M2: `z_sample_timestamp(&ZSample) -> Option<&ZTimestamp>` decomposed
        // into a single primitive leaf `z_timestamp_ntp64(&ZTimestamp) -> i64`
        // (no identity). Outer shape is `Optional(Decompose)`.
        let mut reg = reg_with(&[
            "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
            "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"));
        acc.add_deconstruct_output(ident("z_sample_timestamp"));

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
    fn accessor_plan_byref() {
        // `z_sample_key_expr(&ZSample) -> &ZKeyExpr` decomposed into the keyexpr
        // handle (identity) + its string form (`z_keyexpr_as_str`).
        let mut reg = reg_with(&[
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"));
        acc.add_deconstruct_output(ident("z_sample_key_expr"));

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
    fn ambiguous_accessor_errors() {
        let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::AmbiguousDeconstructor { .. }));
    }

    #[test]
    fn accessor_target_mismatch_errors() {
        // Accessor takes a different type than the accessor's target.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZKeyExpr { todo!() }",
            "fn wrong(x: &ZSample) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record(ident("wrong"));
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::AccessorTargetMismatch { .. }));
    }

    #[test]
    fn multiple_identity_errors() {
        let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record_id();
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::MultipleIdentity { .. }));
    }

    #[test]
    fn nested_accessor_flatten() {
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
        let mut acc = Deconstructors::default();
        // Child accessors (reused via nesting).
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"));
        acc.add_deconstructor(syn::parse_quote!(ZZBytes));
        acc.add_deconstructor_record(ident("z_zbytes_to_bytes"));
        acc.add_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"));
        // Parent accessor with nested + direct records.
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record_nested(ident("z_sample_key_expr"));
        acc.add_deconstructor_record_nested(ident("z_sample_payload"));
        acc.add_deconstructor_record(ident("z_sample_kind"));
        acc.add_deconstructor_record_nested(ident("z_sample_timestamp"));
        acc.add_deconstruct_output(ident("z_reply_sample"));

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
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZA));
        acc.add_deconstructor_record_nested(ident("a_to_b"));
        acc.add_deconstructor(syn::parse_quote!(ZB));
        acc.add_deconstructor_record_nested(ident("b_to_a"));
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::Cycle { .. }));
    }

    #[test]
    fn iterable_whole_element_plan() {
        // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
        // each element delivered WHOLE (no accessor, no leaves).
        let mut reg = reg_with(&[
            "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstruct_output(ident("z_session_peers_zid"));

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

    #[test]
    fn iterable_decomposed_plan() {
        // M5: `z_session_peers_zid -> Vec<ZZenohId>` with a ZZenohId combined
        // accessor → Iterable with per-element leaves: the string form + the
        // value itself via `record_id` (a `value_blob` identity, owned at the
        // root since `Vec<ZZenohId>` owns its elements).
        let mut reg = reg_with(&[
            "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
            "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZZenohId));
        acc.add_deconstructor_record(ident("z_zenoh_id_to_string"));
        acc.add_deconstructor_record_id();
        acc.add_deconstruct_output(ident("z_session_peers_zid"));

        apply(&mut reg, &acc).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
        assert!(plan.element.is_none(), "decomposed: element not used");
        assert_eq!(plan.leaves.len(), 2);
        assert_eq!(plan.leaves[0].path[0].to_string(), "z_zenoh_id_to_string");
        assert_eq!(plan.leaves[0].out_ty.to_token_stream().to_string(), "String");
        // Identity leaf: owned value (`ZZenohId`, not `&ZZenohId`) since the Vec
        // owns its elements (by_ref = false).
        assert!(plan.leaves[1].identity);
        assert!(plan.leaves[1].path.is_empty());
        assert_eq!(plan.leaves[1].out_ty.to_token_stream().to_string(), "ZZenohId");
    }

    #[test]
    fn convert_output_single_value() {
        // `.converter(ZTimestamp, z_timestamp_ntp64)` + `.convert_output()` on
        // `z_sample_timestamp -> Option<&ZTimestamp>` ⇒ Return delivery, single
        // leaf, convert_out_ty = Option<i64>.
        let mut reg = reg_with(&[
            "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
            "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_converter(syn::parse_quote!(ZTimestamp), ident("z_timestamp_ntp64"));
        acc.add_convert_output(ident("z_sample_timestamp"));

        apply(&mut reg, &acc).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_timestamp"))
            .expect("plan");
        assert_eq!(plan.delivery, Delivery::Return);
        assert!(matches!(&plan.shape, UnfoldShape::Optional(_)));
        assert_eq!(plan.leaves.len(), 1);
        assert_eq!(
            plan.convert_out_ty.as_ref().map(|t| t.to_token_stream().to_string()),
            Some("Option < i64 >".to_string())
        );
        // The shaped convert type is registered as a required output.
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(Option<i64>))));
    }

    #[test]
    fn convert_output_multi_leaf_rejected() {
        // A two-record deconstructor (handle + string) cannot be `convert_output`.
        let mut reg = reg_with(&[
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"));
        acc.add_convert_output(ident("z_sample_key_expr"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::ConvertNotSingle { .. }));
    }

    #[test]
    fn convert_output_vec_rejected() {
        // `convert_output` on a `Vec` return is rejected (use deconstruct_output).
        let mut reg = reg_with(&[
            "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
            "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_converter(syn::parse_quote!(ZZenohId), ident("z_zenoh_id_to_string"));
        acc.add_convert_output(ident("z_session_peers_zid"));
        let err = apply(&mut reg, &acc).unwrap_err();
        assert!(matches!(err, UnfoldError::ConvertNotSingle { .. }));
    }
}
