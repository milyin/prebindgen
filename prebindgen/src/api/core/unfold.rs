//! Output (data) expansion — the dual of constructor expansion
//! (`api/core/expand.rs`). A function returning a rich type is *decomposed* by a
//! **deconstructor** into a set of leaf values.
//!
//! A **deconstructor** (a type's `.default_return_field*` list /
//! or the per-fn `.return_field*` override) is a
//! **deterministic product**: every record always runs and contributes its leaf
//! — there is no selector (unlike a *constructor*, whose selector picks one
//! variant). A record's accessor is a `#[prebindgen]` function `f(&T) -> &F` (a
//! reference return where possible, for zero-copy); an accessor whose return
//! type has its own deconstructor splices the child's records with prefixed
//! leaf names.
//!
//! Two **deliveries** (see [`Delivery`]), derived from the resolved leaf count:
//! * `Callback` — replaces the return with a foreign **callback** receiving
//!   all the leaves (any leaf count).
//! * `Return` — **returns** the single leaf value directly (no callback);
//!   requires a single-leaf decomposition.
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
    plan::{DeconId, DeconSpec, LeafSource, UnfoldLeaf, UnfoldPlan, UnfoldShape},
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
    /// An accessor whose return type has its own deconstructor splices that
    /// child's records with the leaf names prefixed `name__<child>`.
    Acc { func: syn::Ident, name: String },
    /// The value itself — the handle/identity leaf (cloned for a `&T` return,
    /// moved for an owned `T`, copied for a `Copy` value_blob). At most one per
    /// deconstructor.
    Identity,
}

#[derive(Clone)]
struct DeconstructorDecl {
    target: syn::Type,
    records: Vec<DeconRecord>,
    /// Auto-apply this deconstructor to every matching declared fn (`Some`
    /// carries the inferred `(target-position, delivery)` to use). Always
    /// `Some` for class-default (`.default_return_field*`) declarations.
    default: Option<(DeconTarget, Delivery)>,
}

/// How an output expansion chooses the deconstructor for a function's return
/// type: the type's default (`.default_return_field*`) or a per-fn
/// inline record list (`.return_field*`).
#[derive(Clone)]
enum DeconSel {
    /// Use the return type's unique deconstructor (error if ambiguous).
    TopLevel,
    /// Per-fn override (`.return_field`): use exactly these
    /// accessor-fn records.
    Inline(Vec<DeconRecord>),
}

/// Which value of a function the deconstructor decomposes: its success return
/// (`Output`) or its `Result<_, E>` domain error (`Error`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeconTarget {
    Output,
    Error,
}

/// How the decomposed value(s) are delivered to the foreign side. Derived
/// from the resolved leaf count (1 ⇒ `Return`, N ⇒ `Callback`); errors are
/// always `Callback`-shaped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    /// Deliver the leaves to a foreign **callback** (builder / fold). Any
    /// leaf count.
    Callback,
    /// **Return**/deliver the single decomposed value (no builder). Requires
    /// exactly one leaf and a non-`Iterable` shape.
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
    /// Cursor for the type-level default builder (`.default_return_field` →
    /// `.field`/`.field_self`).
    cur_deconstructor: Option<usize>,
    /// Cursor for an in-progress per-fn inline output override    /// (`.return_field`/`.return_field_self`): index into
    /// [`Self::outputs`].
    cur_output: Option<usize>,
    /// identity-only `.return_field_self()` opt-outs: fns excluded from the
    /// default auto-apply.
    skip_output: std::collections::HashSet<syn::Ident>,
}

impl Deconstructors {
    /// Find-or-create the default (always-`default`) deconstructor for `target`
    /// and set the cursor to it. Idempotent across a `.default_return_field` chain.
    /// Delivery is derived from leaf count at emit time (1 ⇒ return, N ⇒ callback),
    /// so the stored `Delivery` is just a marker.
    pub fn ensure_default_deconstructor(&mut self, target: syn::Type) {
        let key = TypeKey::from_type(&target);
        // Already building a deconstructor of this type — keep the cursor so
        // records append to it.
        if let Some(i) = self.cur_deconstructor {
            if TypeKey::from_type(&self.deconstructors[i].target) == key {
                return;
            }
        }
        if let Some(i) = self
            .deconstructors
            .iter()
            .position(|d| TypeKey::from_type(&d.target) == key)
        {
            self.cur_deconstructor = Some(i);
        } else {
            self.deconstructors.push(DeconstructorDecl {
                target,
                records: Vec::new(),
                default: Some((DeconTarget::Output, Delivery::Callback)),
            });
            self.cur_deconstructor = Some(self.deconstructors.len() - 1);
        }
    }

    /// identity-only `.return_field_self()` — exclude `func` from output-position
    /// default auto-apply.
    pub fn add_skip_default_output(&mut self, func: syn::Ident) {
        self.skip_output.insert(func);
    }

    /// `.default_return_field(fun)` — add an accessor-function
    /// record with the author-supplied (literal) leaf `name`.
    pub fn add_deconstructor_record(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".field called without a current deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Acc {
            func,
            name: name.into(),
        });
    }

    /// `.default_return_field_self()` — add the identity record
    /// (the value itself).
    pub fn add_deconstructor_record_id(&mut self) {
        let i = self
            .cur_deconstructor
            .expect(".field_self called without a current deconstructor");
        self.deconstructors[i].records.push(DeconRecord::Identity);
    }

    /// Begin a per-fn inline output override (`.return_field`):
    /// decompose `func`'s return via an incrementally-built record list
    /// (accessor fields via [`Self::push_inline_field`] and/or the identity/self
    /// field via [`Self::push_inline_field_self`]). Recorded as an explicit decl
    /// so the auto-`default` skips it, and leaves the output cursor on it.
    pub fn begin_inline_output(&mut self, func: syn::Ident) {
        self.outputs.push(OutputDecl {
            func,
            sel: DeconSel::Inline(Vec::new()),
            target: DeconTarget::Output,
            delivery: Delivery::Callback,
        });
        self.cur_deconstructor = None;
        self.cur_output = Some(self.outputs.len() - 1);
    }

    /// `.return_field(fun)` — append an
    /// accessor-function field (named `name`) to the current per-fn inline output.
    pub fn push_inline_field(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_output
            .expect(".field called without a current deconstructor");
        if let DeconSel::Inline(records) = &mut self.outputs[i].sel {
            records.push(DeconRecord::Acc {
                func,
                name: name.into(),
            });
        }
    }

    /// `.default_return_field_self()` — append the identity
    /// (the handle itself) field to the current per-fn inline output.
    pub fn push_inline_field_self(&mut self) {
        let i = self
            .cur_output
            .expect(".field_self called without a current deconstructor");
        if let DeconSel::Inline(records) = &mut self.outputs[i].sel {
            records.push(DeconRecord::Identity);
        }
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
                DeconRecord::Acc { func, name } => (func, name),
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

    // Default auto-apply: a type's deconstructor (`.default_return_field*`) is
    // applied to every declared fn that returns it (Output) or has it as a
    // `Result<_, E>` error (Error), unless the fn is `fun_accessor` or has a
    // per-fn override. `Delivery` is recomputed from leaf count inside
    // `process_decl` for Output (1 ⇒ Return, N ⇒ Callback).
    for d in &acc.deconstructors {
        if d.default.is_none() {
            continue;
        }
        let dkey = TypeKey::from_type(&d.target);
        let sel = DeconSel::TopLevel;
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
    // the same default output a *return* of `T` would use — so the foreign
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
                // A borrowed arg (`impl Fn(&T)`) decomposes through the same
                // machinery as a `&T` return: strip the leading `&` to reach the
                // deconstructor target and set `by_ref` so the leaves are read
                // (cloned) through the reference instead of by move. The plan is
                // keyed under the ACTUAL arg type (`&T`) — that is what
                // `callback_input`/`callback_iface_spec` look up.
                let (by_ref, core_ty) = match &arg_ty {
                    syn::Type::Reference(r) => (true, (*r.elem).clone()),
                    other => (false, other.clone()),
                };
                // Only a bare path core type can match a deconstructor target
                // (`Option<T>` / `Vec<T>` / tuple args are delivered whole).
                if !matches!(&core_ty, syn::Type::Path(_)) {
                    continue;
                }
                let key = TypeKey::from_type(&arg_ty);
                if registry.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                let core_key = TypeKey::from_type(&core_ty);
                let Some(d) = acc
                    .deconstructors
                    .iter()
                    .find(|d| d.default.is_some() && TypeKey::from_type(&d.target) == core_key)
                else {
                    continue;
                };
                let ed = OutputDecl {
                    func: func.clone(),
                    sel: DeconSel::TopLevel,
                    target: DeconTarget::Output,
                    delivery: Delivery::Callback,
                };
                let decon = decl_id(&core_key, d);
                let records = d.records.clone();
                register_decon_spec(registry, acc, &decon, &records, &core_ty)?;
                let plan = build_plan(
                    acc,
                    registry,
                    &ed,
                    by_ref,
                    &core_ty,
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

/// A synthesized by-value `data_class` decomposition, produced by the language
/// adapter (which knows the per-field encoding — projections, enums, nested
/// classes) and fed to [`apply_value_structs`]. Its [`leaves`](Self::leaves)
/// are [`LeafSource::Field`] leaves: each crosses the boundary as its own field
/// value and the foreign side reassembles the object (no Java object is built
/// on the Rust side).
pub struct ValueDecon {
    /// Canonical key of the value struct (the `DeconId::Default` key).
    pub key: TypeKey,
    /// The struct type (owned) the leaves decompose.
    pub source: syn::Type,
    /// Field-access leaves in foreign-signature / `fromParts` order.
    pub leaves: Vec<UnfoldLeaf>,
}

/// Wire the synthesized by-value `data_class` decompositions into the registry:
/// register each as a `DeconId::Default` [`DeconSpec`], then build a
/// **fixed-builder** [`UnfoldPlan`] for every declared function that returns the
/// struct (`T` / `&T` / `Option<T>` / `Vec<T>`) and a callback-arg plan for
/// every `impl Fn(&T)` / `impl Fn(T)` parameter. Each leaf's `out_ty` is
/// registered as a required output. Mirrors the per-function matching of
/// [`apply`], but the builder/folder is a fixed foreign singleton
/// (`fixed_builder = true`) reconstructing the concrete class, so delivery is
/// always `Callback` (never the single-leaf `Return` shortcut) and the wrapper
/// stays non-generic.
///
/// Runs in `write_rust` right after [`apply`] and before `resolve`.
pub fn apply_value_structs<M>(
    registry: &mut Registry<M>,
    decons: Vec<ValueDecon>,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    for vd in &decons {
        let decon = DeconId::Default(vd.key.to_string());
        require_unique_leaf_names(&vd.source, &vd.leaves)?;
        registry
            .decon_plans
            .entry(decon.clone())
            .or_insert_with(|| DeconSpec {
                source: vd.source.clone(),
                leaves: vd.leaves.clone(),
            });

        // Output position: a declared fn returning the struct
        // (`T` / `&T` / `Option<T|&T>` / `Vec<T|&T>`) decomposes into a
        // fixed-builder plan. (`Result<T, E>` is left to the whole-value
        // converter — the synthesizer covers the infallible returns.)
        for func in declared_fns {
            let Some((item_fn, loc)) = registry.functions.get(func).cloned() else {
                continue;
            };
            let ret = fn_return(&item_fn);
            if !returns_type(&ret, &vd.key) || registry.unfold_plans.contains_key(func) {
                continue;
            }
            // Shape over the field decomposition: peel an outer `Option`, then a
            // `Vec`, then a leading `&`. `Vec<T|&T>` ⇒ Iterable (a **fixed
            // folder**: each element's field leaves cross raw and the foreign
            // folder rebuilds it via `fromParts` + appends, so no Java object is
            // built on the Rust side); `Option<…>` wraps the inner shape in
            // Optional (`None` ⇒ a null result). `element: None` keeps the
            // decomposed-leaf path. The element/inner borrow-ness sets `by_ref`
            // (the field reach clones either way).
            let (optional, after_opt) = match option_inner_type(&ret) {
                Some(inner) => (true, inner),
                None => (false, ret.clone()),
            };
            let (iterable, core) = match vec_inner_type(&after_opt) {
                Some(inner) => (true, inner),
                None => (false, after_opt),
            };
            let by_ref = matches!(&core, syn::Type::Reference(_));
            let inner_shape = if iterable {
                UnfoldShape::Iterable(Box::new(UnfoldShape::Base))
            } else {
                UnfoldShape::Base
            };
            let shape = if optional {
                UnfoldShape::Optional((), Box::new(inner_shape))
            } else {
                inner_shape
            };
            for leaf in &vd.leaves {
                registry.require_output(&leaf.out_ty, &loc);
            }
            let plan = UnfoldPlan {
                source: vd.source.clone(),
                decon: Some(decon.clone()),
                by_ref,
                shape,
                leaves: vd.leaves.clone(),
                element: None,
                delivery: Delivery::Callback,
                convert_out_ty: None,
                fixed_builder: true,
            };
            registry.unfold_plans.insert(func.clone(), plan);
        }

        // Callback-argument position: an `impl Fn(&T)` / `impl Fn(T)` parameter
        // of a declared fn delivers the flattened leaves to the foreign
        // callback, which reassembles the whole value via the data class's
        // `fromParts` before invoking the user's typed callback (the group
        // reassembly lives in the JNI adapter's `asRaw` proxy).
        apply_value_struct_callbacks(registry, vd, &decon, declared_fns)?;
    }
    Ok(())
}

/// Build a fixed-builder callback-arg plan for every `impl Fn(&T)` /
/// `impl Fn(T)` parameter (of a declared fn) whose value is the value struct
/// `vd`. The foreign callback receives the flattened leaves (reassembled there
/// via the data class's `fromParts`) instead of a whole value built on the Rust
/// side. Separate from the output-position wiring so the callback path (which
/// needs the foreign-side group-reassembly adapter) can be enabled on its own.
fn apply_value_struct_callbacks<M>(
    registry: &mut Registry<M>,
    vd: &ValueDecon,
    decon: &DeconId,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
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
                // Peel a leading `&`, then detect a slice element. An
                // `impl Fn(&T)` / `impl Fn(T)` arg of the value struct decomposes
                // into a `Base` fixed builder (foreign side reassembles the whole
                // value via `fromParts`); an `impl Fn(&[T])` / `impl Fn([T])` arg
                // becomes an `Iterable` fixed FOLDER (the trampoline folds each
                // element's leaves into a foreign list — see the callback emitter).
                let (by_ref, after_ref) = match &arg_ty {
                    syn::Type::Reference(r) => (true, (*r.elem).clone()),
                    other => (false, other.clone()),
                };
                let (shape, matches_key) = match &after_ref {
                    syn::Type::Slice(s) => (
                        UnfoldShape::Iterable(Box::new(UnfoldShape::Base)),
                        TypeKey::from_type(&s.elem) == vd.key,
                    ),
                    other => (UnfoldShape::Base, TypeKey::from_type(other) == vd.key),
                };
                if !matches_key {
                    continue;
                }
                let key = TypeKey::from_type(&arg_ty);
                if registry.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                for leaf in &vd.leaves {
                    registry.require_output(&leaf.out_ty, &loc);
                }
                let plan = UnfoldPlan {
                    source: vd.source.clone(),
                    decon: Some(decon.clone()),
                    by_ref,
                    shape,
                    leaves: vd.leaves.clone(),
                    element: None,
                    delivery: Delivery::Callback,
                    convert_out_ty: None,
                    fixed_builder: true,
                };
                registry.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// Wire **whole-element** `Iterable` fold plans for bare `Vec<T>` /
/// `Option<Vec<T>>` returns and `impl Fn(&[T])` callback args whose element `T`
/// is a single leaf (String, value blob, opaque handle) nominated by the adapter
/// via [`crate::api::core::prebindgen::Prebindgen::leaf_vec_fold_elements`]. Each
/// such position crosses as decoupled raw leaves folded into a **foreign-built**
/// list — the single-leaf dual of [`apply_value_structs`] (which handles
/// multi-field `data_class` elements). The fold is a **fixed** foreign singleton
/// (`fixed_builder = true`): the wrapper allocates the list, passes the hoisted
/// appender, and returns the concrete `List<T>` (never a caller `fold` param), so
/// no `java.util.ArrayList` is built on the Rust side.
///
/// Runs right after [`apply_value_structs`]; skips any function/arg that already
/// carries a plan (an explicit `.deconstruct_output`, a `data_class` fold, …) so
/// declared decompositions and value-struct folds win.
pub fn apply_leaf_vec_folds<M>(
    registry: &mut Registry<M>,
    elements: Vec<syn::Type>,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    if elements.is_empty() {
        return Ok(());
    }
    let elem_keys: Vec<TypeKey> = elements.iter().map(TypeKey::from_type).collect();
    // Is the leading-`&`-peeled `bare` one of the nominated single-leaf elements?
    let is_nominated = |bare: &syn::Type| elem_keys.contains(&TypeKey::from_type(bare));
    for func in declared_fns {
        let Some((item_fn, loc)) = registry.functions.get(func).cloned() else {
            continue;
        };
        // Output position: `Vec<T>` / `Option<Vec<T>>` return. Skip if a plan
        // already exists (declared deconstructor / value-struct fold).
        if !registry.unfold_plans.contains_key(func) {
            let ret = fn_return(&item_fn);
            let (optional, after_opt) = match option_inner_type(&ret) {
                Some(inner) => (true, inner),
                None => (false, ret.clone()),
            };
            if let Some(vec_elem) = vec_inner_type(&after_opt) {
                let bare = peel_ref(&vec_elem);
                if is_nominated(&bare) {
                    let inner_shape = UnfoldShape::Iterable(Box::new(UnfoldShape::Base));
                    let shape = if optional {
                        UnfoldShape::Optional((), Box::new(inner_shape))
                    } else {
                        inner_shape
                    };
                    registry.require_output(&vec_elem, &loc);
                    // The fold delivers the return element-by-element, so the
                    // whole `Vec<T>` / `Option<Vec<T>>` converter is not needed.
                    // De-require it: for String / value-blob elements it still
                    // resolves (and is emitted as harmless dead code); for an
                    // opaque-handle element it cannot resolve (`jlong` wire isn't
                    // JObject-shaped), and de-requiring keeps that `None` from
                    // being flagged as an unresolved-required error.
                    registry.unrequire_output(&ret);
                    registry
                        .unfold_plans
                        .insert(func.clone(), whole_leaf_fold_plan(&vec_elem, shape));
                }
            }
        }
        // Callback-arg position: `impl Fn(&[T])` / `impl Fn([T])`.
        for input in &item_fn.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let Some(args) = crate::api::core::registry::extract_fn_trait_args(&pt.ty) else {
                continue;
            };
            for arg_ty in args {
                let after_ref = peel_ref(&arg_ty);
                let syn::Type::Slice(s) = &after_ref else {
                    continue;
                };
                let elem = (*s.elem).clone();
                if !is_nominated(&peel_ref(&elem)) {
                    continue;
                }
                let key = TypeKey::from_type(&arg_ty);
                if registry.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                registry.require_output(&elem, &loc);
                let plan =
                    whole_leaf_fold_plan(&elem, UnfoldShape::Iterable(Box::new(UnfoldShape::Base)));
                registry.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// Build a fixed-builder whole-element fold [`UnfoldPlan`] for a single-leaf
/// element `vec_elem` (the `Vec`/slice element as written, keeping any leading
/// `&` so `into_iter()`'s yield matches the element's own output converter).
fn whole_leaf_fold_plan(vec_elem: &syn::Type, shape: UnfoldShape) -> UnfoldPlan {
    UnfoldPlan {
        source: vec_elem.clone(),
        decon: None,
        by_ref: matches!(vec_elem, syn::Type::Reference(_)),
        shape,
        leaves: vec![],
        element: Some(vec_elem.clone()),
        delivery: Delivery::Callback,
        convert_out_ty: None,
        fixed_builder: true,
    }
}

/// Strip a single leading `&` (one level) from a type.
fn peel_ref(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    }
}

/// The function's return type (or `()` for a unit return).
fn fn_return(item_fn: &syn::ItemFn) -> syn::Type {
    match &item_fn.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => (**t).clone(),
    }
}

/// True when `ret` is `T` / `&T` / `Option<T|&T>` / `Vec<T|&T>` with
/// `T == key` — the default-output match. `Result<_, _>` is NOT peeled, so a
/// fallible factory (`-> Result<T, E>`) keeps its handle return; the error
/// position is matched separately on `E`.
fn returns_type(ret: &syn::Type, key: &TypeKey) -> bool {
    // Peel an outer `Option`, then a `Vec` (so `Option<Vec<T>>` matches too),
    // then a leading `&`.
    let mut core = ret.clone();
    if let Some(inner) = option_inner_type(&core) {
        core = inner;
    }
    if let Some(inner) = vec_inner_type(&core) {
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
            if let Some(d) = find_deconstructor_by_type(acc, &ekey) {
                // Decomposed: reuse the shared flatten (M3 nesting composes).
                let records = d.records.clone();
                let decon = decl_id(&ekey, d);
                register_decon_spec(registry, acc, &decon, &records, &element)?;
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
                    fixed_builder: false,
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
            register_decon_spec(registry, acc, &decon, &records, &source)?;
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

/// The identity of a found declaration — the type's default deconstructor.
fn decl_id(type_key: &TypeKey, _decl: &DeconstructorDecl) -> DeconId {
    DeconId::Default(type_key.to_string())
}

/// Register the declaration-default [`DeconSpec`] for `decon` (no-op when
/// already present): re-flatten the records with normalized inputs —
/// borrowed identity, no outer shape — so the stored spec is independent of
/// the using function's return shape and of processing order.
fn register_decon_spec<M>(
    registry: &mut Registry<M>,
    acc: &Deconstructors,
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
        DeconSel::TopLevel => find_deconstructor_by_type(acc, source_key)
            .map(|d| (d.records.clone(), DeconId::Default(source_key.to_string())))
            .ok_or_else(|| UnfoldError::NoDeconstructor {
                func: ed.func.clone(),
                target: source_key.to_string(),
            }),
    }
}

/// Find the deconstructor whose target is `type_key` (unique per type:
/// `ensure_default_deconstructor` dedups by type key). Used for both the
/// top-level output expansion and nested-record splicing.
fn find_deconstructor_by_type<'a>(
    acc: &'a Deconstructors,
    type_key: &TypeKey,
) -> Option<&'a DeconstructorDecl> {
    acc.deconstructors
        .iter()
        .find(|c| TypeKey::from_type(&c.target) == *type_key)
}

/// Build the [`UnfoldPlan`] for a chosen accessor. `shape` is the outer
/// shape over the core decomposition (`Decompose` for `T`/`&T`,
/// `Optional(Decompose)` for `Option<T>`/`Option<&T>`). The records are
/// recursively flattened ([`flatten`]) — nested accessors contribute
/// their leaves with the access path prefixed.
#[allow(clippy::too_many_arguments)]
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
    require_root_identity_last(by_ref, source, &leaves)?;

    Ok(UnfoldPlan {
        source: source.clone(),
        decon: Some(decon),
        by_ref,
        shape,
        leaves,
        element: None,
        delivery: ed.delivery,
        convert_out_ty: None,
        fixed_builder: false,
    })
}

/// Error when an **owned** decomposition emits the root identity leaf before a
/// nested identity leaf. Leaves are emitted in declaration order, and the root
/// identity MOVES the owned value while a nested identity clones from a borrow
/// of it — the wrong order generates non-compiling Rust ("use of moved value")
/// with a cryptic rustc message. Caught here instead, with the fix in the
/// error: declare `.field_self()` after the nested-identity fields. (Borrowed
/// decompositions clone the root identity, so any order is fine.)
fn require_root_identity_last(
    by_ref: bool,
    source: &syn::Type,
    leaves: &[UnfoldLeaf],
) -> Result<(), UnfoldError> {
    if by_ref {
        return Ok(());
    }
    let root_at = leaves.iter().position(|l| l.identity && l.path.is_empty());
    let last_nested_at = leaves
        .iter()
        .rposition(|l| l.identity && !l.path.is_empty());
    if let (Some(root), Some(nested)) = (root_at, last_nested_at) {
        if root < nested {
            return Err(UnfoldError::RootIdentityBeforeNested {
                target: TypeKey::from_type(source).to_string(),
            });
        }
    }
    Ok(())
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
                    source: LeafSource::Accessor,
                });
            }
            DeconRecord::Acc { func, name } => {
                let (takes, ret) = accessor_signature(registry, func)?;
                check_takes(func, &takes, source)?;
                // Default unwrap: if the return type has its own deconstructor,
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
                if let Some(child_decl) = find_deconstructor_by_type(acc, &child_key) {
                    if !visited.insert(child_key.clone()) {
                        return Err(UnfoldError::Cycle {
                            target: child_key.to_string(),
                        });
                    }
                    let child_records = child_decl.records.clone();
                    let mut child_path = path_prefix.to_vec();
                    child_path.push(func.clone());
                    flatten(
                        acc,
                        registry,
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
                        source: LeafSource::Accessor,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Error if two leaves of one flattened deconstructor share a name. Author leaf
/// names are explicit and emitted literally, so a collision is a declaration
/// bug — never auto-resolved.
fn require_unique_leaf_names(source: &syn::Type, leaves: &[UnfoldLeaf]) -> Result<(), UnfoldError> {
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
mod tests;
