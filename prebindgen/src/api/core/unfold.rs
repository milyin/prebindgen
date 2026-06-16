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
//! produces its converter (and projection). The jnigen adapter reads the
//! plan at the return-emission site.
//!
//! [`Iterable`]: UnfoldShape::Iterable

use std::collections::HashSet;

use crate::api::core::{
    registry::{Registry, TypeKey},
    types_util::{option_inner_type, result_err_type, vec_inner_type},
};

mod error;
mod plan;

pub use self::{
    error::UnfoldError,
    plan::{DeconId, DeconSpec, UnfoldLeaf, UnfoldPlan, UnfoldShape},
};

// ──────────────────────────────────────────────────────────────────────
// Declarations (populated by the language builder)
// ──────────────────────────────────────────────────────────────────────

/// One record (field) of a deconstructor. A deconstructor is a product: every
/// record contributes a leaf.
#[derive(Clone)]
enum DeconRecord {
    /// Read this field by calling the accessor function `f(&T) -> &F`. `name`
    /// is the author-supplied leaf name, used **literally** (no casing /
    /// stripping); it may not contain the reserved `"__"` chain separator.
    Acc { func: syn::Ident, name: String },
    /// The value itself — the handle/identity leaf (cloned for a `&T` return,
    /// moved for an owned `T`, copied for a `Copy` value_blob). At most one per
    /// deconstructor.
    Identity,
    /// Splice in another type's deconstructor via the accessor function
    /// `f(&T) -> &Child` (or `-> Option<&Child>`): the child type's records are
    /// flattened with the access path prefixed by `f` (and marked nullable when
    /// `f` returns `Option`). `name` is the author-supplied segment prefix for
    /// the spliced child leaves, joined with `"__"`; it may not contain `"__"`.
    Nested { func: syn::Ident, name: String },
}

#[derive(Clone)]
struct DeconstructorDecl {
    name: Option<String>,
    target: syn::Type,
    records: Vec<DeconRecord>,
    /// `.default()` — auto-apply this deconstructor to every matching declared
    /// fn (`Some` carries the inferred `(target-position, delivery)` to use).
    default: Option<(DeconTarget, Delivery)>,
}

/// How an output-expansion `.deconstruct_output`/`.convert_output` (and their
/// `_with` variants) chooses the deconstructor for a function's return type.
#[derive(Clone)]
enum DeconSel {
    /// Use the return type's unique deconstructor (error if ambiguous).
    TopLevel,
    /// Use the deconstructor named by this string.
    Explicit(String),
    /// Per-fn override (`.fun_output`): use exactly these accessor-fn records.
    Inline(Vec<DeconRecord>),
}

/// Which value of a function the deconstructor decomposes: its success return
/// (`Output`) or its `Result<_, E>` domain error (`Error`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeconTarget {
    Output,
    Error,
}

/// How the decomposed value(s) are delivered to the foreign side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    /// `.deconstruct_output()` / `.deconstruct_error()` — deliver the leaves to a
    /// foreign **callback** (builder / fold). Any leaf count.
    Callback,
    /// `.convert_output()` / `.convert_error()` — **return**/deliver the single
    /// decomposed value (no builder). Requires exactly one leaf and a
    /// non-`Iterable` shape.
    Return,
}

#[derive(Clone)]
struct OutputDecl {
    func: syn::Ident,
    sel: DeconSel,
    target: DeconTarget,
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
    /// Cursor for the deconstructor builder (`.deconstructor_record*` /
    /// `.default`).
    cur_deconstructor: Option<usize>,
    /// `.skip_default_*` opt-outs: fns excluded from a `.default()` auto-apply.
    skip_output: std::collections::HashSet<syn::Ident>,
    skip_error: std::collections::HashSet<syn::Ident>,
}

impl Deconstructors {
    /// `.deconstructor(target)` — begin a deconstructor for `target`.
    pub fn add_deconstructor(&mut self, target: syn::Type) {
        self.deconstructors.push(DeconstructorDecl {
            name: None,
            target,
            records: Vec::new(),
            default: None,
        });
        self.cur_deconstructor = Some(self.deconstructors.len() - 1);
    }

    /// `.deconstructor_name(name)` — name the cursor declaration so per-fn
    /// `_with(name)` selectors can pick it. A named declaration is an
    /// *alternative* decomposition of its target type; the type's bare
    /// (unnamed) declaration stays the canonical one. Panics without a live
    /// deconstructor cursor.
    pub fn add_deconstructor_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect("deconstructor_name must be chained after `.deconstructor(...)`");
        self.deconstructors[i].name = Some(name.into());
    }

    /// True when a `.deconstructor` / `.converter` is the live cursor (so a
    /// chained `.default()` routes here rather than to a constructor).
    pub fn has_current(&self) -> bool {
        self.cur_deconstructor.is_some()
    }

    /// Clear the deconstructor cursor so a following `.default()` doesn't route
    /// here. Called when a *constructor* declaration starts (the two cursors are
    /// mutually exclusive — `.default()` targets the most recent decl).
    pub fn clear_cursor(&mut self) {
        self.cur_deconstructor = None;
    }

    /// `.default()` — auto-apply the current deconstructor to every declared fn
    /// whose output (`Output`) or `Result` error (`Error`) matches `target`,
    /// unless skipped. The `(target-position, delivery)` is inferred at [`apply`]:
    /// a single-`Acc`-record deconstructor (a converter) ⇒ `Return`; otherwise
    /// `Callback`. The position (`Output` vs `Error`) is chosen by `apply` per
    /// fn (a fn whose return is a borrow of `target` ⇒ Output; whose `Result`
    /// error is `target` ⇒ Error). Panics without a current `.deconstructor`.
    pub fn set_default(&mut self) {
        let i = self
            .cur_deconstructor
            .expect(".default called without a current .deconstructor / .converter");
        // Placeholder delivery; `apply` recomputes per use. Stored just to mark
        // "is a default".
        self.deconstructors[i].default = Some((DeconTarget::Output, Delivery::Callback));
    }

    /// Find-or-create the canonical (always-`default`) deconstructor for `target`
    /// and set the cursor to it. Idempotent across a `.ptr_class_output*` chain.
    /// Delivery is derived from leaf count at emit time (1 ⇒ return, N ⇒ callback),
    /// so the stored `Delivery` is just a marker.
    pub fn ensure_canonical_deconstructor(&mut self, target: syn::Type) {
        let key = TypeKey::from_type(&target);
        // Already building a deconstructor of this type — canonical OR a
        // named alternative begun via `add_deconstructor` +
        // `add_deconstructor_name` — keep the cursor so records append to it.
        if let Some(i) = self.cur_deconstructor {
            if TypeKey::from_type(&self.deconstructors[i].target) == key {
                return;
            }
        }
        // Only the UNNAMED declaration is the canonical one; named
        // alternatives of the same type must not receive canonical records.
        if let Some(i) = self
            .deconstructors
            .iter()
            .position(|d| d.name.is_none() && TypeKey::from_type(&d.target) == key)
        {
            self.cur_deconstructor = Some(i);
        } else {
            self.deconstructors.push(DeconstructorDecl {
                name: None,
                target,
                records: Vec::new(),
                default: Some((DeconTarget::Output, Delivery::Callback)),
            });
            self.cur_deconstructor = Some(self.deconstructors.len() - 1);
        }
    }

    /// `.skip_default_deconstruct_output()` / `.skip_default_convert_output()` —
    /// exclude `func` from output-position `.default()` auto-apply.
    pub fn add_skip_default_output(&mut self, func: syn::Ident) {
        self.skip_output.insert(func);
    }

    /// `.skip_default_convert_error()` — exclude `func` from error-position
    /// `.default()` auto-apply.
    pub fn add_skip_default_error(&mut self, func: syn::Ident) {
        self.skip_error.insert(func);
    }

    /// `.converter(target, func, name)` — a single-value deconstructor: one
    /// accessor record `func` (`f(&target) -> F`) named `name`. Sugar for
    /// `.deconstructor(target)` + `.deconstructor_record(func, name)`; usable via
    /// `.convert_output`/`.convert_error` and as a nested record source. Leaves
    /// the cursor on it so `.default()` can chain.
    pub fn add_converter(&mut self, target: syn::Type, func: syn::Ident, name: impl Into<String>) {
        self.add_deconstructor(target);
        self.add_deconstructor_record(func, name);
    }

    /// `.deconstructor_name(name)` — name the current deconstructor so it can be
    /// selected via `.deconstruct_output_with` / `.convert_output_with`.
    pub fn set_deconstructor_name(&mut self, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_name called without a current .deconstructor");
        self.deconstructors[i].name = Some(name.into());
    }

    /// `.deconstructor_record(func, name)` — add an accessor-function record
    /// with the author-supplied (literal) leaf `name`.
    pub fn add_deconstructor_record(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Acc {
            func,
            name: name.into(),
        });
    }

    /// `.deconstructor_record_id()` — add the identity record (the value
    /// itself).
    pub fn add_deconstructor_record_id(&mut self) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record_id called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Identity);
    }

    /// `.deconstructor_record_nested(func, name)` — splice another type's
    /// deconstructor via the accessor `func` (`f(&T) -> &Child` or
    /// `-> Option<&Child>`); `Child`'s records are flattened with the access
    /// path prefixed by `func` and the leaf names prefixed by `name__`.
    pub fn add_deconstructor_record_nested(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".deconstructor_record_nested called without a current .deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Nested {
            func,
            name: name.into(),
        });
    }

    fn push_output(
        &mut self,
        func: syn::Ident,
        sel: DeconSel,
        target: DeconTarget,
        delivery: Delivery,
    ) {
        self.outputs.push(OutputDecl {
            func,
            sel,
            target,
            delivery,
        });
        self.cur_deconstructor = None;
    }

    /// `.deconstruct_output()` — decompose the fn's return value and deliver the
    /// leaves to a foreign **callback**.
    pub fn add_deconstruct_output(&mut self, func: syn::Ident) {
        self.push_output(
            func,
            DeconSel::TopLevel,
            DeconTarget::Output,
            Delivery::Callback,
        );
    }

    /// `.fun_output(records)` — per-fn override: decompose the return via exactly
    /// these `(accessor-fn, leaf-name)` records (each unwrapped per its return
    /// type's canonical output). Recorded as an explicit decl so the
    /// auto-`default` skips it.
    pub fn add_output_inline(&mut self, func: syn::Ident, funcs: Vec<(syn::Ident, String)>) {
        let records = funcs
            .into_iter()
            .map(|(func, name)| DeconRecord::Acc { func, name })
            .collect();
        self.push_output(
            func,
            DeconSel::Inline(records),
            DeconTarget::Output,
            Delivery::Callback,
        );
    }

    /// `.deconstruct_output_with(name)` — by named deconstructor.
    pub fn add_deconstruct_output_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.push_output(
            func,
            DeconSel::Explicit(name.into()),
            DeconTarget::Output,
            Delivery::Callback,
        );
    }

    /// `.convert_output()` — decompose the fn's return via a single-value
    /// deconstructor and **return** the value directly (no callback).
    pub fn add_convert_output(&mut self, func: syn::Ident) {
        self.push_output(
            func,
            DeconSel::TopLevel,
            DeconTarget::Output,
            Delivery::Return,
        );
    }

    /// `.convert_output_with(name)` — by named deconstructor.
    pub fn add_convert_output_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.push_output(
            func,
            DeconSel::Explicit(name.into()),
            DeconTarget::Output,
            Delivery::Return,
        );
    }

    /// `.deconstruct_error()` — decompose the fn's `Result<_, E>` domain error
    /// and deliver its leaves to the foreign error callback (after the fixed
    /// `je: String?` binding param).
    pub fn add_deconstruct_error(&mut self, func: syn::Ident) {
        self.push_output(
            func,
            DeconSel::TopLevel,
            DeconTarget::Error,
            Delivery::Callback,
        );
    }

    /// `.deconstruct_error_with(name)` — by named deconstructor.
    pub fn add_deconstruct_error_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.push_output(
            func,
            DeconSel::Explicit(name.into()),
            DeconTarget::Error,
            Delivery::Callback,
        );
    }

    /// `.convert_error()` — convert the fn's domain error to a single value
    /// (one ze leaf after `je`).
    pub fn add_convert_error(&mut self, func: syn::Ident) {
        self.push_output(
            func,
            DeconSel::TopLevel,
            DeconTarget::Error,
            Delivery::Return,
        );
    }

    /// `.convert_error_with(name)` — by named deconstructor.
    pub fn add_convert_error_with(&mut self, func: syn::Ident, name: impl Into<String>) {
        self.push_output(
            func,
            DeconSel::Explicit(name.into()),
            DeconTarget::Error,
            Delivery::Return,
        );
    }

    /// True iff no output expansion was declared (lets `write_rust` skip
    /// [`apply`]). A `.default()` deconstructor counts (it synthesizes decls).
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty() && !self.deconstructors.iter().any(|d| d.default.is_some())
    }
}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Resolve every output-expansion declaration (explicit + `.default()`
/// auto-applied) into an [`UnfoldPlan`], register each leaf's `out_ty` as a
/// required output, and store the plans on the registry (`unfold_plans` for
/// `Output`, `error_plans` for `Error`).
///
/// `declared_fns` is the adapter's claimed `#[prebindgen]` fn set — the domain
/// over which `.default()` deconstructors are auto-applied. `accessor_fns` is
/// the `.fun_accessor` subset — the only functions a decomposer record may
/// reference.
///
/// Runs inside `write_rust` after `expand::apply` and before `resolve`, so leaf
/// converters resolve through the normal rank machinery.
pub fn apply<M>(
    registry: &mut Registry<M>,
    acc: &Deconstructors,
    declared_fns: &std::collections::HashSet<syn::Ident>,
    accessor_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    // Gate: every accessor-function record of every declared deconstructor must
    // be a `.fun_accessor` (the single source of truth for "accessor").
    for d in &acc.deconstructors {
        for rec in &d.records {
            let (func, name) = match rec {
                DeconRecord::Acc { func, name } | DeconRecord::Nested { func, name } => {
                    (func, name)
                }
                DeconRecord::Identity => continue,
            };
            // `"__"` is the reserved nesting/chain separator — author leaf names
            // must not contain it.
            if name.contains("__") {
                return Err(UnfoldError::ReservedSeparator { name: name.clone() });
            }
            if !accessor_fns.contains(func) {
                return Err(UnfoldError::RecordNotAccessor { func: func.clone() });
            }
        }
    }

    // Explicit decls first; they take precedence over (and suppress) a default
    // for the same `(fn, target)`.
    let mut done: std::collections::HashSet<(syn::Ident, DeconTarget)> = Default::default();
    for ed in &acc.outputs {
        process_decl(registry, acc, ed)?;
        done.insert((ed.func.clone(), ed.target));
    }

    // Canonical auto-apply: a type's deconstructor (`.ptr_class_output*`) is
    // applied to every declared fn that returns it (Output) or has it as a
    // `Result<_, E>` error (Error), unless the fn is `fun_accessor` or has a
    // per-fn override. `Delivery` is recomputed from leaf count inside
    // `process_decl` for Output (1 ⇒ Return, N ⇒ Callback).
    for d in &acc.deconstructors {
        if d.default.is_none() {
            continue;
        }
        let dkey = TypeKey::from_type(&d.target);
        // Select THIS declaration explicitly: a defaulted *named* declaration
        // must not be re-resolved through the bare top-level lookup (which
        // would be ambiguous or pick the type's unnamed declaration).
        let sel = match &d.name {
            Some(n) => DeconSel::Explicit(n.clone()),
            None => DeconSel::TopLevel,
        };
        for func in declared_fns {
            // Read accessors are never output-decomposed (they ARE the records).
            if accessor_fns.contains(func) {
                continue;
            }
            let Some((item_fn, _)) = registry.functions.get(func).cloned() else {
                continue;
            };
            let ret = fn_return(&item_fn);
            // Error position: fn returns `Result<_, E>` and `E == d.target`.
            if let Some(err_ty) = result_err_type(&ret) {
                if TypeKey::from_type(&err_ty) == dkey
                    && !acc.skip_error.contains(func)
                    && done.insert((func.clone(), DeconTarget::Error))
                {
                    process_decl(
                        registry,
                        acc,
                        &OutputDecl {
                            func: func.clone(),
                            sel: sel.clone(),
                            target: DeconTarget::Error,
                            delivery: Delivery::Callback,
                        },
                    )?;
                }
            }
            // Output position: fn returns `T` / `&T` / `Option<T|&T>` / `Vec<T>`
            // with `T == d.target` (Result returns keep a handle — factories).
            if returns_type(&ret, &dkey)
                && !acc.skip_output.contains(func)
                && done.insert((func.clone(), DeconTarget::Output))
            {
                process_decl(
                    registry,
                    acc,
                    &OutputDecl {
                        func: func.clone(),
                        sel: sel.clone(),
                        target: DeconTarget::Output,
                        delivery: Delivery::Callback,
                    },
                )?;
            }
        }
    }

    // Callback-argument decomposition: each `T` of a declared fn's
    // `impl Fn(T, …)` parameter is delivered per `T`'s default deconstructor —
    // the same canonical output a *return* of `T` would use — so the foreign
    // callback receives the flattened leaves in one crossing instead of a
    // whole value. Plans are type-level (keyed by `T`, fn-independent) with
    // `by_ref = false` (the trampoline owns the value, so a root identity
    // record moves it). Delivery is always `Callback` regardless of leaf count
    // (there is no return-value lane in a callback invocation). A type without
    // a default deconstructor gets no plan and is delivered whole.
    for func in declared_fns {
        let Some((item_fn, loc)) = registry.functions.get(func).cloned() else {
            continue;
        };
        for input in &item_fn.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let Some(args) = crate::api::core::registry::extract_fn_trait_args(&pt.ty) else {
                continue;
            };
            for arg_ty in args {
                // Only a bare path type can match a deconstructor target
                // (`Option<T>` / `Vec<T>` / `&T` args are delivered whole).
                if !matches!(&arg_ty, syn::Type::Path(_)) {
                    continue;
                }
                let key = TypeKey::from_type(&arg_ty);
                if registry.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                let Some(d) = acc
                    .deconstructors
                    .iter()
                    .find(|d| d.default.is_some() && TypeKey::from_type(&d.target) == key)
                else {
                    continue;
                };
                let ed = OutputDecl {
                    func: func.clone(),
                    sel: DeconSel::TopLevel,
                    target: DeconTarget::Output,
                    delivery: Delivery::Callback,
                };
                let decon = decl_id(&key, d);
                let records = d.records.clone();
                register_decon_spec(registry, acc, func, &decon, &records, &arg_ty)?;
                let plan = build_plan(
                    acc,
                    registry,
                    &ed,
                    false,
                    &arg_ty,
                    UnfoldShape::Base,
                    &records,
                    decon,
                )?;
                if plan.leaves.is_empty() {
                    continue;
                }
                for leaf in &plan.leaves {
                    registry.require_output(&leaf.out_ty, &loc);
                }
                registry.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// The function's return type (or `()` for a unit return).
fn fn_return(item_fn: &syn::ItemFn) -> syn::Type {
    match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => (**t).clone(),
    }
}

/// True when `ret` is `T` / `&T` / `Option<T|&T>` / `Vec<T|&T>` with
/// `T == key` — the canonical-output match. `Result<_, _>` is NOT peeled, so a
/// fallible factory (`-> Result<T, E>`) keeps its handle return; the error
/// position is matched separately on `E`.
fn returns_type(ret: &syn::Type, key: &TypeKey) -> bool {
    // Peel Vec / Option (one layer each, in either order) then a leading `&`.
    let mut core = ret.clone();
    if let Some(inner) = vec_inner_type(&core) {
        core = inner;
    } else if let Some(inner) = option_inner_type(&core) {
        core = inner;
    }
    let bare = match &core {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    TypeKey::from_type(&bare) == *key
}

/// Build one output/error plan for `ed` and store it in the right registry map.
fn process_decl<M>(
    registry: &mut Registry<M>,
    acc: &Deconstructors,
    ed: &OutputDecl,
) -> Result<(), UnfoldError> {
    {
        let (item_fn, loc) = registry
            .functions
            .get(&ed.func)
            .cloned()
            .ok_or_else(|| UnfoldError::UnknownFunction(ed.func.clone()))?;

        // The value to decompose: the success return (`Output`) or the
        // `Result<_, E>` domain error `E` (`Error`).
        let ret_ty: syn::Type = match ed.target {
            DeconTarget::Output => fn_return(&item_fn),
            DeconTarget::Error => {
                result_err_type(&fn_return(&item_fn)).ok_or_else(|| UnfoldError::Unsupported {
                    func: ed.func.clone(),
                    reason: "convert_error/deconstruct_error on a non-Result return",
                })?
            }
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
            let shape = UnfoldShape::Iterable(Box::new(UnfoldShape::Base));
            // Element type peeled of a leading `&` (accessors take `&Element`).
            let (by_ref, element) = match &inner {
                syn::Type::Reference(r) => (true, (*r.elem).clone()),
                other => (false, other.clone()),
            };
            let ekey = TypeKey::from_type(&element);
            if let Ok(d) = find_deconstructor_by_type(acc, &ekey) {
                // Decomposed: reuse the shared flatten (M3 nesting composes).
                let records = d.records.clone();
                let decon = decl_id(&ekey, d);
                register_decon_spec(registry, acc, &ed.func, &decon, &records, &element)?;
                let plan = build_plan(acc, registry, ed, by_ref, &element, shape, &records, decon)?;
                for leaf in &plan.leaves {
                    registry.require_output(&leaf.out_ty, &loc);
                }
                plan
            } else {
                // Whole element: keep the type exactly as written so the
                // element's own output converter matches `into_iter()`'s yield.
                // No declaration is involved (`decon: None`) — the element
                // crosses whole through its own converter.
                let by_ref = matches!(&inner, syn::Type::Reference(_));
                registry.require_output(&inner, &loc);
                UnfoldPlan {
                    source: inner.clone(),
                    decon: None,
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
                UnfoldShape::Optional((), Box::new(UnfoldShape::Base))
            } else {
                UnfoldShape::Base
            };
            let (records, decon) = resolve_deconstructor(acc, &source_key, ed)?;
            register_decon_spec(registry, acc, &ed.func, &decon, &records, &source)?;
            let plan = build_plan(acc, registry, ed, by_ref, &source, shape, &records, decon)?;
            for leaf in &plan.leaves {
                registry.require_output(&leaf.out_ty, &loc);
            }
            plan
        };
        // Delivery is by **leaf count**, not a per-decl flag:
        //   * Output, single leaf, non-Iterable ⇒ Return (wrapper returns the
        //     value via its ordinary output converter — `convert_out_ty`).
        //   * Output, multiple leaves or Iterable ⇒ Callback (builder / fold).
        //   * Error ⇒ always Callback-shaped: every leaf is a `ze` arg after the
        //     fixed `je` (no return-value path; `convert_out_ty` stays None).
        let single_return = ed.target == DeconTarget::Output
            && !matches!(plan.shape, UnfoldShape::Iterable(_))
            && plan.leaves.len() == 1;
        let plan = if single_return {
            let leaf_ty = plan.leaves[0].out_ty.clone();
            let cv_ty: syn::Type = if matches!(plan.shape, UnfoldShape::Optional((), _)) {
                syn::parse_quote!(Option<#leaf_ty>)
            } else {
                leaf_ty
            };
            registry.require_output(&cv_ty, &loc);
            UnfoldPlan {
                delivery: Delivery::Return,
                convert_out_ty: Some(cv_ty),
                ..plan
            }
        } else {
            UnfoldPlan {
                delivery: Delivery::Callback,
                ..plan
            }
        };
        match ed.target {
            DeconTarget::Output => registry.unfold_plans.insert(ed.func.clone(), plan),
            DeconTarget::Error => registry.error_plans.insert(ed.func.clone(), plan),
        };
    }
    Ok(())
}

/// The identity of a found declaration: `Named` when it carries a name,
/// `Canonical` otherwise.
fn decl_id(type_key: &TypeKey, decl: &DeconstructorDecl) -> DeconId {
    match &decl.name {
        Some(n) => DeconId::Named(type_key.to_string(), n.clone()),
        None => DeconId::Canonical(type_key.to_string()),
    }
}

/// Register the declaration-canonical [`DeconSpec`] for `decon` (no-op when
/// already present): re-flatten the records with normalized inputs —
/// borrowed identity, no outer shape — so the stored spec is independent of
/// the using function's return shape and of processing order.
fn register_decon_spec<M>(
    registry: &mut Registry<M>,
    acc: &Deconstructors,
    top_func: &syn::Ident,
    decon: &DeconId,
    records: &[DeconRecord],
    source: &syn::Type,
) -> Result<(), UnfoldError> {
    if registry.decon_plans.contains_key(decon) {
        return Ok(());
    }
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    let mut visited: HashSet<TypeKey> = HashSet::new();
    visited.insert(TypeKey::from_type(source));
    flatten(
        acc,
        registry,
        top_func,
        records,
        source,
        &[],
        &[],
        true,
        false,
        &mut visited,
        &mut leaves,
    )?;
    require_unique_leaf_names(source, &leaves)?;
    registry.decon_plans.insert(
        decon.clone(),
        DeconSpec {
            source: source.clone(),
            leaves,
        },
    );
    Ok(())
}

/// Pick the deconstructor (its records + declaration identity) for one
/// output expansion.
fn resolve_deconstructor(
    acc: &Deconstructors,
    source_key: &TypeKey,
    ed: &OutputDecl,
) -> Result<(Vec<DeconRecord>, DeconId), UnfoldError> {
    match &ed.sel {
        DeconSel::Inline(records) => Ok((
            records.clone(),
            DeconId::PerFn(source_key.to_string(), ed.func.to_string()),
        )),
        DeconSel::Explicit(name) => acc
            .deconstructors
            .iter()
            .find(|c| c.name.as_deref() == Some(name.as_str()))
            .map(|c| (c.records.clone(), decl_id(source_key, c)))
            .ok_or_else(|| UnfoldError::UnknownDeconstructor {
                func: ed.func.clone(),
                name: name.clone(),
            }),
        DeconSel::TopLevel => find_deconstructor_by_type(acc, source_key)
            .map(|d| (d.records.clone(), decl_id(source_key, d)))
            .map_err(|candidates| match candidates {
                None => UnfoldError::NoDeconstructor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                },
                Some(candidates) => UnfoldError::AmbiguousDeconstructor {
                    func: ed.func.clone(),
                    target: source_key.to_string(),
                    candidates,
                },
            }),
    }
}

/// Find the unique deconstructor whose target is `type_key`. `Err(None)` =
/// none registered; `Err(Some(candidates))` = ambiguous (>1). Used for both the
/// top-level output expansion and nested-record resolution.
fn find_deconstructor_by_type<'a>(
    acc: &'a Deconstructors,
    type_key: &TypeKey,
) -> Result<&'a DeconstructorDecl, Option<Vec<String>>> {
    // Only the UNNAMED declaration is the type's canonical decomposition —
    // named alternatives are reachable solely via the `_with(name)` selectors
    // and never shadow the canonical one (so `.default()` auto-apply and
    // nested-child splicing stay unambiguous when alternatives exist).
    let matches: Vec<&DeconstructorDecl> = acc
        .deconstructors
        .iter()
        .filter(|c| c.name.is_none() && TypeKey::from_type(&c.target) == *type_key)
        .collect();
    match matches.len() {
        1 => Ok(matches[0]),
        0 => Err(None),
        _ => Err(Some(
            matches
                .iter()
                .map(|c| {
                    c.name
                        .clone()
                        .unwrap_or_else(|| "<deconstructor>".to_string())
                })
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
    decon: DeconId,
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
        &[],
        by_ref,
        false,
        &mut visited,
        &mut leaves,
    )?;
    require_unique_leaf_names(source, &leaves)?;

    Ok(UnfoldPlan {
        source: source.clone(),
        decon: Some(decon),
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
///   empty && `!by_ref`) — a `Copy` value delivers itself by copy and an
///   opaque handle moves; everywhere else it is **borrowed** (`&source`,
///   cloned).
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
    name_prefix: &[String],
    by_ref: bool,
    nullable: bool,
    visited: &mut HashSet<TypeKey>,
    leaves: &mut Vec<UnfoldLeaf>,
) -> Result<(), UnfoldError> {
    let source_key = TypeKey::from_type(source);
    // The author-supplied (literal) leaf-name segment at this level, appended
    // to the inherited chain prefix. Segments are joined with `"__"`.
    let seg_name = |name: &str| -> Vec<String> {
        let mut v = name_prefix.to_vec();
        v.push(name.to_string());
        v
    };
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
                // Owned at the root of an owned value (a `Copy` blob copies /
                // an opaque handle moves); borrowed (clone) otherwise. The
                // adapter-side type + projection come from this `out_ty`'s
                // output converter.
                let out_ty: syn::Type = if path_prefix.is_empty() && !by_ref {
                    source.clone()
                } else {
                    syn::parse_quote!(&#source)
                };
                leaves.push(UnfoldLeaf {
                    name: if path_prefix.is_empty() {
                        "handle".to_string()
                    } else {
                        name_prefix.join("__")
                    },
                    path: path_prefix.to_vec(),
                    out_ty,
                    identity: true,
                    nullable,
                });
            }
            DeconRecord::Acc { func, name } => {
                let (takes, ret) = accessor_signature(registry, func)?;
                check_takes(func, &takes, source)?;
                // Canonical unwrap: if the return type has its own deconstructor,
                // splice it (recurse); otherwise the return is one leaf. Peel an
                // `Option` (value may be absent) + leading `&` to reach the child.
                let (opt, core) = match option_inner_type(&ret) {
                    Some(inner) => (true, inner),
                    None => (false, ret.clone()),
                };
                let child_ty = match &core {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    other => other.clone(),
                };
                let child_key = TypeKey::from_type(&child_ty);
                let has_canonical = find_deconstructor_by_type(acc, &child_key).is_ok();
                if has_canonical {
                    if !visited.insert(child_key.clone()) {
                        return Err(UnfoldError::Cycle {
                            target: child_key.to_string(),
                        });
                    }
                    let child_records = find_deconstructor_by_type(acc, &child_key)
                        .expect("checked is_ok")
                        .records
                        .clone();
                    let mut child_path = path_prefix.to_vec();
                    child_path.push(func.clone());
                    flatten(
                        acc,
                        registry,
                        top_func,
                        &child_records,
                        &child_ty,
                        &child_path,
                        &seg_name(name),
                        by_ref,
                        nullable || opt,
                        visited,
                        leaves,
                    )?;
                    visited.remove(&child_key);
                } else {
                    // Leaf: the return value as written (`&str`, enum, `i64`, …).
                    let mut path = path_prefix.to_vec();
                    path.push(func.clone());
                    leaves.push(UnfoldLeaf {
                        name: seg_name(name).join("__"),
                        path,
                        out_ty: ret,
                        identity: false,
                        nullable,
                    });
                }
            }
            DeconRecord::Nested { func, name } => {
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
                let child_records = find_deconstructor_by_type(acc, &child_key)
                    .map_err(|_| UnfoldError::NoDeconstructor {
                        func: top_func.clone(),
                        target: child_key.to_string(),
                    })?
                    .records
                    .clone();
                let mut child_path = path_prefix.to_vec();
                child_path.push(func.clone());
                flatten(
                    acc,
                    registry,
                    top_func,
                    &child_records,
                    &child_ty,
                    &child_path,
                    &seg_name(name),
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

/// Error if two leaves of one flattened deconstructor share a name. Author leaf
/// names are explicit and emitted literally, so a collision is a declaration
/// bug — never auto-resolved.
fn require_unique_leaf_names(
    source: &syn::Type,
    leaves: &[UnfoldLeaf],
) -> Result<(), UnfoldError> {
    let mut seen: HashSet<&str> = HashSet::new();
    for l in leaves {
        if !seen.insert(l.name.as_str()) {
            return Err(UnfoldError::DuplicateLeafName {
                target: TypeKey::from_type(source).to_string(),
                name: l.name.clone(),
            });
        }
    }
    Ok(())
}

/// Make a signature's name list unique: a duplicate gets a numeric suffix
/// (`name2`, `name3`, …). Adapters run this over the final per-signature
/// list (after their own casing), since one signature may concatenate the
/// leaves of several plans.
pub fn dedup_names(names: &mut [String]) {
    let mut seen: HashSet<String> = HashSet::new();
    for n in names.iter_mut() {
        if !seen.insert(n.clone()) {
            let mut k = 2;
            while !seen.insert(format!("{n}{k}")) {
                k += 1;
            }
            *n = format!("{n}{k}");
        }
    }
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

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;
    use crate::api::core::{registry::Registry, types_util::ident};

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

    /// A generous `.fun_accessor` set covering every function used as a
    /// deconstructor record across these tests (a superset is fine — `apply`
    /// only checks records are members). The `nested_record_*` tests that
    /// exercise the gate's *rejection* pass an explicit smaller set instead.
    fn acc_set() -> std::collections::HashSet<syn::Ident> {
        [
            "a_to_b",
            "b_to_a",
            "wrong",
            "z_error_message",
            "z_keyexpr_as_str",
            "z_reply_replier_zid",
            "z_reply_is_ok",
            "z_reply_sample",
            "z_reply_err",
            "z_reply_error_payload",
            "z_sample_key_expr",
            "z_sample_payload",
            "z_sample_encoding",
            "z_sample_kind",
            "z_sample_timestamp",
            "z_sample_express",
            "z_sample_priority",
            "z_sample_congestion_control",
            "z_sample_attachment",
            "z_timestamp_ntp64",
            "z_zbytes_to_bytes",
            "z_zenoh_id_to_string",
            "z_encoding_to_string",
        ]
        .iter()
        .map(|s| ident(s))
        .collect()
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
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        acc.add_deconstruct_output(ident("z_sample_timestamp"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");

        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_timestamp"))
            .expect("plan");
        assert!(plan.by_ref, "inner was &ZTimestamp");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZTimestamp");
        assert!(
            matches!(&plan.shape, UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)),
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
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstruct_output(ident("z_sample_key_expr"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");

        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_key_expr"))
            .expect("plan");
        assert!(plan.by_ref, "return was &ZKeyExpr");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZKeyExpr");
        assert!(matches!(plan.shape, UnfoldShape::Base));
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
        assert_eq!(plan.leaves[1].out_ty.to_token_stream().to_string(), "& str");

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
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
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
        acc.add_deconstructor_record(ident("wrong"), "wrong");
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
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
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
        assert!(matches!(err, UnfoldError::MultipleIdentity { .. }));
    }

    #[test]
    fn record_must_be_fun_accessor() {
        // A deconstructor record referencing a non-`.fun_accessor` fn errors.
        let mut reg = reg_with(&[
            "fn z_foo(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstruct_output(ident("z_foo"));
        // Empty accessor set ⇒ z_keyexpr_as_str is not a fun_accessor ⇒ error.
        let err = apply(&mut reg, &acc, &Default::default(), &Default::default()).unwrap_err();
        assert!(matches!(err, UnfoldError::RecordNotAccessor { .. }));
        // With it declared as an accessor, the gate passes.
        let accset: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_as_str"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &Default::default(), &accset).expect("gate passes");
    }

    #[test]
    fn duplicate_leaf_name_errors() {
        // Two records of one deconstructor given the same literal name ⇒ hard
        // error (names are emitted verbatim; never auto-disambiguated).
        let mut reg = reg_with(&[
            "fn z_foo() -> ZSample { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
            "fn z_sample_payload(s: &ZSample) -> Vec<u8> { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "field");
        acc.add_deconstructor_record(ident("z_sample_payload"), "field");
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
        assert!(matches!(err, UnfoldError::DuplicateLeafName { .. }), "{err:?}");
    }

    #[test]
    fn reserved_separator_in_name_errors() {
        // A record name containing the reserved `"__"` chain separator ⇒ error.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZSample { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "key__expr");
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
        assert!(matches!(err, UnfoldError::ReservedSeparator { .. }), "{err:?}");
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
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstructor(syn::parse_quote!(ZZBytes));
        acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
        acc.add_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        // Parent accessor with nested + direct records.
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record_nested(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record_nested(ident("z_sample_payload"), "z_sample_payload");
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        acc.add_deconstructor_record_nested(ident("z_sample_timestamp"), "z_sample_timestamp");
        acc.add_deconstruct_output(ident("z_reply_sample"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_reply_sample"))
            .expect("plan");
        assert!(plan.by_ref);
        assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
        assert!(matches!(&plan.shape, UnfoldShape::Optional((), _)));

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
        assert_eq!(
            plan.leaves[3].out_ty.to_token_stream().to_string(),
            "SampleKind"
        );
        assert_eq!(
            path(&plan.leaves[4]),
            "z_sample_timestamp.z_timestamp_ntp64"
        );
        // Only the timestamp leaf (Option nesting accessor) is nullable.
        assert!(!plan.leaves[1].nullable && !plan.leaves[2].nullable);
        assert!(plan.leaves[4].nullable);
    }

    #[test]
    fn reply_product_double_option_flatten() {
        // ZReply-shaped product (Result<Sample, ReplyError> decomposed in the
        // current product model): the root's records include two
        // `Option<&Child>` nesting accessors (`z_reply_sample`, `z_reply_err`)
        // whose children themselves contain `Option` nesting steps and a
        // nested identity — the double-unwrap case — plus an
        // `Option<ZZenohId>` Acc record with NO canonical child, which keeps
        // the full `Option<…>` as its leaf `out_ty` (its own `Option` is the
        // converter's business, not a nesting step ⇒ NOT nullable).
        let mut reg = reg_with(&[
            "fn z_recv_reply(q: &ZQuery) -> ZReply { todo!() }",
            "fn z_reply_replier_zid(r: &ZReply) -> Option<ZZenohId> { todo!() }",
            "fn z_reply_is_ok(r: &ZReply) -> bool { todo!() }",
            "fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { todo!() }",
            "fn z_reply_err(r: &ZReply) -> Option<&ZReplyError> { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
            "fn z_timestamp_ntp64(t: &ZTimestamp) -> i64 { todo!() }",
            "fn z_reply_error_payload(e: &ZReplyError) -> &ZZBytes { todo!() }",
            "fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        acc.add_deconstructor(syn::parse_quote!(ZZBytes));
        acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record_nested(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record_nested(ident("z_sample_timestamp"), "z_sample_timestamp");
        acc.add_deconstructor(syn::parse_quote!(ZReplyError));
        acc.add_deconstructor_record_nested(ident("z_reply_error_payload"), "z_reply_error_payload");
        acc.add_deconstructor(syn::parse_quote!(ZReply));
        acc.add_deconstructor_record(ident("z_reply_replier_zid"), "z_reply_replier_zid");
        acc.add_deconstructor_record(ident("z_reply_is_ok"), "z_reply_is_ok");
        acc.add_deconstructor_record_nested(ident("z_reply_sample"), "z_reply_sample");
        acc.add_deconstructor_record_nested(ident("z_reply_err"), "z_reply_err");
        acc.add_deconstruct_output(ident("z_recv_reply"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg.unfold_plans.get(&ident("z_recv_reply")).expect("plan");
        assert!(!plan.by_ref, "owned ZReply return");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZReply");
        assert!(matches!(&plan.shape, UnfoldShape::Base));
        assert!(matches!(plan.delivery, Delivery::Callback));

        let path = |l: &UnfoldLeaf| {
            l.path
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(".")
        };
        assert_eq!(plan.leaves.len(), 6);
        // Acc leaf keeping its full `Option<…>` return — not a nesting step.
        assert_eq!(path(&plan.leaves[0]), "z_reply_replier_zid");
        assert_eq!(
            plan.leaves[0].out_ty.to_token_stream().to_string(),
            "Option < ZZenohId >"
        );
        assert!(!plan.leaves[0].nullable && !plan.leaves[0].identity);
        assert_eq!(path(&plan.leaves[1]), "z_reply_is_ok");
        assert!(!plan.leaves[1].nullable);
        // Ok-arm leaves: spliced through the `Option`-returning
        // `z_reply_sample` ⇒ all nullable, incl. the nested keyexpr identity
        // and the doubly-`Option` timestamp path.
        assert!(plan.leaves[2].identity);
        assert_eq!(path(&plan.leaves[2]), "z_reply_sample.z_sample_key_expr");
        assert!(plan.leaves[2].nullable);
        assert_eq!(
            path(&plan.leaves[3]),
            "z_reply_sample.z_sample_key_expr.z_keyexpr_as_str"
        );
        assert!(plan.leaves[3].nullable);
        assert_eq!(
            path(&plan.leaves[4]),
            "z_reply_sample.z_sample_timestamp.z_timestamp_ntp64"
        );
        assert!(plan.leaves[4].nullable);
        // Err-arm leaf: spliced through `z_reply_err`.
        assert_eq!(
            path(&plan.leaves[5]),
            "z_reply_err.z_reply_error_payload.z_zbytes_to_bytes"
        );
        assert!(plan.leaves[5].nullable);
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
        acc.add_deconstructor_record_nested(ident("a_to_b"), "a_to_b");
        acc.add_deconstructor(syn::parse_quote!(ZB));
        acc.add_deconstructor_record_nested(ident("b_to_a"), "b_to_a");
        acc.add_deconstruct_output(ident("z_foo"));
        let err = apply(&mut reg, &acc, &Default::default(), &acc_set()).unwrap_err();
        assert!(matches!(err, UnfoldError::Cycle { .. }));
    }

    #[test]
    fn iterable_whole_element_plan() {
        // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
        // each element delivered WHOLE (no accessor, no leaves).
        let mut reg =
            reg_with(&["fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }"]);
        let mut acc = Deconstructors::default();
        acc.add_deconstruct_output(ident("z_session_peers_zid"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(
            matches!(&plan.shape, UnfoldShape::Iterable(inner) if matches!(**inner, UnfoldShape::Base)),
            "outer shape is Iterable(Decompose)"
        );
        assert!(!plan.by_ref, "Vec<ZZenohId> owns its elements");
        assert!(
            plan.leaves.is_empty(),
            "whole-element: no decomposed leaves"
        );
        assert_eq!(
            plan.element
                .as_ref()
                .map(|t| t.to_token_stream().to_string()),
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
        acc.add_deconstructor_record(ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
        acc.add_deconstructor_record_id();
        acc.add_deconstruct_output(ident("z_session_peers_zid"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
        assert!(plan.element.is_none(), "decomposed: element not used");
        assert_eq!(plan.leaves.len(), 2);
        assert_eq!(plan.leaves[0].path[0].to_string(), "z_zenoh_id_to_string");
        assert_eq!(
            plan.leaves[0].out_ty.to_token_stream().to_string(),
            "String"
        );
        // Identity leaf: owned value (`ZZenohId`, not `&ZZenohId`) since the Vec
        // owns its elements (by_ref = false).
        assert!(plan.leaves[1].identity);
        assert!(plan.leaves[1].path.is_empty());
        assert_eq!(
            plan.leaves[1].out_ty.to_token_stream().to_string(),
            "ZZenohId"
        );
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
        acc.add_converter(syn::parse_quote!(ZTimestamp), ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        acc.add_convert_output(ident("z_sample_timestamp"));

        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_timestamp"))
            .expect("plan");
        assert_eq!(plan.delivery, Delivery::Return);
        assert!(matches!(&plan.shape, UnfoldShape::Optional((), _)));
        assert_eq!(plan.leaves.len(), 1);
        assert_eq!(
            plan.convert_out_ty
                .as_ref()
                .map(|t| t.to_token_stream().to_string()),
            Some("Option < i64 >".to_string())
        );
        // The shaped convert type is registered as a required output.
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(Option<i64>))));
    }

    #[test]
    fn multi_leaf_output_is_callback() {
        // A two-record deconstructor (handle + string) ⇒ Callback delivery (>1 leaf).
        let mut reg = reg_with(&[
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstruct_output(ident("z_sample_key_expr"));
        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_sample_key_expr"))
            .expect("plan");
        assert_eq!(plan.delivery, Delivery::Callback);
        assert_eq!(plan.leaves.len(), 2);
        assert!(plan.convert_out_ty.is_none());
    }

    #[test]
    fn vec_output_is_iterable_callback() {
        // A `Vec` return ⇒ Iterable + Callback (a fold), never a single Return.
        let mut reg = reg_with(&[
            "fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }",
            "fn z_zenoh_id_to_string(z: &ZZenohId) -> String { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_converter(syn::parse_quote!(ZZenohId), ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
        acc.add_deconstruct_output(ident("z_session_peers_zid"));
        apply(&mut reg, &acc, &Default::default(), &acc_set()).expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
        assert_eq!(plan.delivery, Delivery::Callback);
    }

    #[test]
    fn convert_error_decomposes_result_e() {
        // The ZError deconstructor (`z_error_message`) auto-applies to every fn
        // returning `Result<_, ZError>`, storing the plan in `error_plans`. Error
        // delivery is always Callback (its leaves are the `ze` callback args).
        let mut reg = reg_with(&[
            "fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, ZError> { todo!() }",
            "fn z_error_message(e: &ZError) -> String { todo!() }",
            "fn z_infallible(s: &ZSample) -> bool { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_converter(syn::parse_quote!(ZError), ident("z_error_message"), "z_error_message");
        acc.set_default();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_try_from", "z_infallible"]
                .iter()
                .map(|s| ident(s))
                .collect();
        let accset: std::collections::HashSet<syn::Ident> =
            ["z_error_message"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &accset).expect("apply");

        let plan = reg
            .error_plans
            .get(&ident("z_keyexpr_try_from"))
            .expect("error plan for the fallible fn");
        assert_eq!(plan.delivery, Delivery::Callback);
        assert_eq!(plan.leaves.len(), 1);
        assert_eq!(
            plan.leaves[0].out_ty.to_token_stream().to_string(),
            "String"
        );
        assert_eq!(plan.source.to_token_stream().to_string(), "ZError");
        // The infallible fn gets no error plan.
        assert!(!reg.error_plans.contains_key(&ident("z_infallible")));
        // No output plans created (no ZKeyExpr return among the declared fns; the
        // ZError deconstructor only matches the Result error position).
        assert!(reg.unfold_plans.is_empty());
    }

    #[test]
    fn canonical_output_applies_to_owned_and_borrow_returns() {
        // Canonical-everywhere: the ZKeyExpr deconstructor auto-applies to BOTH a
        // `&ZKeyExpr` (borrow) and an owned `ZKeyExpr` return. (`Result<…>` returns
        // are excluded — they keep a handle — and `fun_accessor`s are skipped.)
        let mut reg = reg_with(&[
            "fn z_borrow_keyexpr(s: &ZSession) -> &ZKeyExpr { todo!() }",
            "fn z_make_keyexpr(s: &ZSession) -> ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.set_default();
        // Only the record fn is an accessor; the two return fns are plain.
        let accset: std::collections::HashSet<syn::Ident> =
            ["z_keyexpr_as_str"].iter().map(|s| ident(s)).collect();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_borrow_keyexpr", "z_make_keyexpr"]
                .iter()
                .map(|s| ident(s))
                .collect();
        apply(&mut reg, &acc, &declared, &accset).expect("apply");

        assert!(
            reg.unfold_plans.contains_key(&ident("z_borrow_keyexpr")),
            "borrow return"
        );
        assert!(
            reg.unfold_plans.contains_key(&ident("z_make_keyexpr")),
            "owned return"
        );
    }

    #[test]
    fn skip_default_error_opts_out() {
        let mut reg = reg_with(&[
            "fn z_fallible(s: String) -> Result<ZKeyExpr, ZError> { todo!() }",
            "fn z_error_message(e: &ZError) -> String { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_converter(syn::parse_quote!(ZError), ident("z_error_message"), "z_error_message");
        acc.set_default();
        acc.add_skip_default_error(ident("z_fallible"));
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_fallible"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
        assert!(reg.error_plans.is_empty(), "skipped fn gets no error plan");
    }

    #[test]
    fn callback_arg_plan_derived() {
        // An `impl Fn(ZSample)` parameter of a declared fn gets a type-level
        // plan from ZSample's default deconstructor — same leaves a return of
        // ZSample would produce, but owned (`by_ref = false`).
        let mut reg = reg_with(&[
            "fn z_declare_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record_nested(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        acc.set_default();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_declare_sub"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

        let plan = reg
            .callback_arg_plans
            .get(&TypeKey::from_type(&syn::parse_quote!(ZSample)))
            .expect("callback-arg plan for ZSample");
        assert!(!plan.by_ref, "the trampoline owns the callback arg");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
        assert!(matches!(plan.shape, UnfoldShape::Base));
        assert_eq!(plan.delivery, Delivery::Callback);
        assert_eq!(plan.leaves.len(), 3);
        // Nested keyexpr identity (borrowed: non-root) + string + direct enum.
        assert!(plan.leaves[0].identity);
        assert_eq!(plan.leaves[0].path[0].to_string(), "z_sample_key_expr");
        assert_eq!(
            plan.leaves[0].out_ty.to_token_stream().to_string(),
            "& ZKeyExpr"
        );
        assert_eq!(
            plan.leaves[1].path.last().unwrap().to_string(),
            "z_keyexpr_as_str"
        );
        assert_eq!(
            plan.leaves[2].out_ty.to_token_stream().to_string(),
            "SampleKind"
        );
        // Leaf out_tys registered so the resolver builds their converters.
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(&str))));
        assert!(reg
            .required_outputs_scan
            .contains(&TypeKey::from_type(&syn::parse_quote!(SampleKind))));
        // No return-position plan was created for the declaring fn.
        assert!(reg.unfold_plans.is_empty());
    }

    #[test]
    fn callback_arg_identity_fallback() {
        // No deconstructor for ZQuery ⇒ no plan: the arg is delivered whole.
        let mut reg = reg_with(&[
            "fn z_declare_queryable(cb: impl Fn(ZQuery) + Send + Sync + 'static) { todo!() }",
        ]);
        let acc = Deconstructors::default();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_declare_queryable"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
        assert!(reg.callback_arg_plans.is_empty());
    }

    #[test]
    fn callback_zero_arg_no_plan() {
        let mut reg =
            reg_with(&["fn z_with_close(on_close: impl Fn() + Send + Sync + 'static) { todo!() }"]);
        let acc = Deconstructors::default();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_with_close"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
        assert!(reg.callback_arg_plans.is_empty());
    }

    #[test]
    fn callback_arg_nonbare_skipped() {
        // `impl Fn(Vec<ZSample>)`: the arg type key (`Vec<ZSample>`) matches no
        // deconstructor target ⇒ whole-value fallback, no plan.
        let mut reg = reg_with(&[
            "fn z_batched(cb: impl Fn(Vec<ZSample>) + Send + Sync + 'static) { todo!() }",
            "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.add_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        acc.set_default();
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_batched"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
        assert!(reg.callback_arg_plans.is_empty());
    }
}
