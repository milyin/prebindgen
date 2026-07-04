//! Output (data) expansion — the dual of constructor expansion
//! (`api/core/expand.rs`). A function returning a rich type is *decomposed* by a
//! **deconstructor** into a set of leaf values.
//!
//! A **deconstructor** (a type's `.flatten_output()` + `.field(name)` /
//! `.field_self()`, or the per-fn `.flatten_output_with()` override) is a
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
    /// `Some` for `.flatten_output()`-created declarations.
    default: Option<(DeconTarget, Delivery)>,
}

/// How an output expansion chooses the deconstructor for a function's return
/// type: the type's default flatten (`.flatten_output()`) or a per-fn
/// inline record list (`.flatten_output_with()`).
#[derive(Clone)]
enum DeconSel {
    /// Use the return type's unique deconstructor (error if ambiguous).
    TopLevel,
    /// Per-fn override (`.flatten_output_with()`): use exactly these
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
    /// Cursor for the type-level flatten builder (`.flatten_output()` →
    /// `.field`/`.field_self`).
    cur_deconstructor: Option<usize>,
    /// Cursor for an in-progress per-fn inline output flatten
    /// (`.flatten_output_with()` → `.field`/`.field_self`): index into
    /// [`Self::outputs`].
    cur_output: Option<usize>,
    /// `.flatten_output_suppress()` opt-outs: fns excluded from the
    /// default auto-apply.
    skip_output: std::collections::HashSet<syn::Ident>,
}

impl Deconstructors {
    /// Find-or-create the default (always-`default`) deconstructor for `target`
    /// and set the cursor to it. Idempotent across a `.flatten_output()` chain.
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

    /// `.flatten_output_suppress()` — exclude `func` from output-position
    /// default auto-apply.
    pub fn add_skip_default_output(&mut self, func: syn::Ident) {
        self.skip_output.insert(func);
    }

    /// `.field(name)` inside `.flatten_output()` — add an accessor-function
    /// record with the author-supplied (literal) leaf `name`.
    pub fn add_deconstructor_record(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_deconstructor
            .expect(".field called without a current .flatten_output");
        self.deconstructors[i].records.push(DeconRecord::Acc {
            func,
            name: name.into(),
        });
    }

    /// `.field_self()` inside `.flatten_output()` — add the identity record
    /// (the value itself).
    pub fn add_deconstructor_record_id(&mut self) {
        let i = self
            .cur_deconstructor
            .expect(".field_self called without a current .flatten_output");
        self.deconstructors[i].records.push(DeconRecord::Identity);
    }

    /// Begin a per-fn inline output flatten (`.flatten_output_with()`):
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

    /// `.field(fn, name)` inside `.flatten_output_with()` — append an
    /// accessor-function field (named `name`) to the current per-fn inline output.
    pub fn push_inline_field(&mut self, func: syn::Ident, name: impl Into<String>) {
        let i = self
            .cur_output
            .expect(".field called without a current .flatten_output_with");
        if let DeconSel::Inline(records) = &mut self.outputs[i].sel {
            records.push(DeconRecord::Acc {
                func,
                name: name.into(),
            });
        }
    }

    /// `.field_self()` inside `.flatten_output_with()` — append the identity
    /// (the handle itself) field to the current per-fn inline output.
    pub fn push_inline_field_self(&mut self) {
        let i = self
            .cur_output
            .expect(".field_self called without a current .flatten_output_with");
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

    // Default auto-apply: a type's deconstructor (`.flatten_output()`) is
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

    /// [`acc_set`] minus the decomposed fn: the default auto-apply skips
    /// accessor fns, and some tests decompose a fn that doubles as a record
    /// accessor elsewhere in the shared set.
    fn acc_set_without(f: &str) -> std::collections::HashSet<syn::Ident> {
        let mut s = acc_set();
        s.remove(&ident(f));
        s
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");

        apply(
            &mut reg,
            &acc,
            &[ident("z_sample_timestamp")].into_iter().collect(),
            &acc_set_without("z_sample_timestamp"),
        )
        .expect("apply");

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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");

        apply(
            &mut reg,
            &acc,
            &[ident("z_sample_key_expr")].into_iter().collect(),
            &acc_set_without("z_sample_key_expr"),
        )
        .expect("apply");

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
    fn root_identity_before_nested_identity_errors() {
        // Owned return: the root `.field_self()` MOVES the value, a nested
        // identity (spliced ZKeyExpr handle) borrows it — id-first is the
        // order that would generate non-compiling Rust, caught at apply time.
        let mut reg = reg_with(&[
            "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
            "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
        ]);
        let accessors: std::collections::HashSet<syn::Ident> =
            ["z_query_key_expr"].iter().map(|s| ident(s)).collect();
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZQuery));
        acc.add_deconstructor_record_id(); // root identity FIRST — wrong
        acc.add_deconstructor_record(ident("z_query_key_expr"), "key_expr");
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_take_query")].into_iter().collect(),
            &accessors,
        )
        .unwrap_err();
        assert!(matches!(err, UnfoldError::RootIdentityBeforeNested { .. }));

        // Root identity LAST (the zenoh `Query` shape) is accepted.
        let mut reg2 = reg_with(&[
            "fn z_take_query(q: &ZQuery) -> ZQuery { todo!() }",
            "fn z_query_key_expr(q: &ZQuery) -> &ZKeyExpr { todo!() }",
        ]);
        let mut acc2 = Deconstructors::default();
        acc2.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc2.add_deconstructor_record_id();
        acc2.ensure_default_deconstructor(syn::parse_quote!(ZQuery));
        acc2.add_deconstructor_record(ident("z_query_key_expr"), "key_expr");
        acc2.add_deconstructor_record_id(); // root identity last — ok
        apply(
            &mut reg2,
            &acc2,
            &[ident("z_take_query")].into_iter().collect(),
            &accessors,
        )
        .expect("root identity last is the supported order");
    }

    #[test]
    fn accessor_target_mismatch_errors() {
        // Accessor takes a different type than the accessor's target.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZKeyExpr { todo!() }",
            "fn wrong(x: &ZSample) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record(ident("wrong"), "wrong");
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &acc_set(),
        )
        .unwrap_err();
        assert!(matches!(err, UnfoldError::AccessorTargetMismatch { .. }));
    }

    #[test]
    fn multiple_identity_errors() {
        let mut reg = reg_with(&["fn z_foo() -> ZKeyExpr { todo!() }"]);
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record_id();
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &acc_set(),
        )
        .unwrap_err();
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        // Empty accessor set ⇒ z_keyexpr_as_str is not a fun_accessor ⇒ error.
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &Default::default(),
        )
        .unwrap_err();
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "field");
        acc.add_deconstructor_record(ident("z_sample_payload"), "field");
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &acc_set(),
        )
        .unwrap_err();
        assert!(
            matches!(err, UnfoldError::DuplicateLeafName { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn reserved_separator_in_name_errors() {
        // A record name containing the reserved `"__"` chain separator ⇒ error.
        let mut reg = reg_with(&[
            "fn z_foo() -> ZSample { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "key__expr");
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &acc_set(),
        )
        .unwrap_err();
        assert!(
            matches!(err, UnfoldError::ReservedSeparator { .. }),
            "{err:?}"
        );
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZZBytes));
        acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        // Parent accessor with nested + direct records.
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record(ident("z_sample_payload"), "z_sample_payload");
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        acc.add_deconstructor_record(ident("z_sample_timestamp"), "z_sample_timestamp");

        apply(
            &mut reg,
            &acc,
            &[ident("z_reply_sample")].into_iter().collect(),
            &acc_set_without("z_reply_sample"),
        )
        .expect("apply");
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
        // `Option<ZZenohId>` Acc record with NO default child, which keeps
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZZBytes));
        acc.add_deconstructor_record(ident("z_zbytes_to_bytes"), "z_zbytes_to_bytes");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record(ident("z_sample_timestamp"), "z_sample_timestamp");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZReplyError));
        acc.add_deconstructor_record(ident("z_reply_error_payload"), "z_reply_error_payload");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZReply));
        acc.add_deconstructor_record(ident("z_reply_replier_zid"), "z_reply_replier_zid");
        acc.add_deconstructor_record(ident("z_reply_is_ok"), "z_reply_is_ok");
        acc.add_deconstructor_record(ident("z_reply_sample"), "z_reply_sample");
        acc.add_deconstructor_record(ident("z_reply_err"), "z_reply_err");

        apply(
            &mut reg,
            &acc,
            &[ident("z_recv_reply")].into_iter().collect(),
            &acc_set(),
        )
        .expect("apply");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZA));
        acc.add_deconstructor_record(ident("a_to_b"), "a_to_b");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZB));
        acc.add_deconstructor_record(ident("b_to_a"), "b_to_a");
        let err = apply(
            &mut reg,
            &acc,
            &[ident("z_foo")].into_iter().collect(),
            &acc_set(),
        )
        .unwrap_err();
        assert!(matches!(err, UnfoldError::Cycle { .. }));
    }

    #[test]
    fn iterable_whole_element_plan() {
        // M4: `z_session_peers_zid(&ZSession) -> Vec<ZZenohId>` → Iterable;
        // each element delivered WHOLE (no accessor, no leaves): a per-fn
        // flatten with an empty record list on an element type that has no
        // deconstructor of its own.
        let mut reg =
            reg_with(&["fn z_session_peers_zid(s: &ZSession) -> Vec<ZZenohId> { todo!() }"]);
        let mut acc = Deconstructors::default();
        acc.begin_inline_output(ident("z_session_peers_zid"));

        apply(
            &mut reg,
            &acc,
            &[ident("z_session_peers_zid")].into_iter().collect(),
            &acc_set(),
        )
        .expect("apply");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZZenohId));
        acc.add_deconstructor_record(ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
        acc.add_deconstructor_record_id();

        apply(
            &mut reg,
            &acc,
            &[ident("z_session_peers_zid")].into_iter().collect(),
            &acc_set(),
        )
        .expect("apply");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZTimestamp));
        acc.add_deconstructor_record(ident("z_timestamp_ntp64"), "z_timestamp_ntp64");

        apply(
            &mut reg,
            &acc,
            &[ident("z_sample_timestamp")].into_iter().collect(),
            &acc_set_without("z_sample_timestamp"),
        )
        .expect("apply");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        apply(
            &mut reg,
            &acc,
            &[ident("z_sample_key_expr")].into_iter().collect(),
            &acc_set_without("z_sample_key_expr"),
        )
        .expect("apply");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZZenohId));
        acc.add_deconstructor_record(ident("z_zenoh_id_to_string"), "z_zenoh_id_to_string");
        apply(
            &mut reg,
            &acc,
            &[ident("z_session_peers_zid")].into_iter().collect(),
            &acc_set(),
        )
        .expect("apply");
        let plan = reg
            .unfold_plans
            .get(&ident("z_session_peers_zid"))
            .expect("plan");
        assert!(matches!(&plan.shape, UnfoldShape::Iterable(_)));
        assert_eq!(plan.delivery, Delivery::Callback);
    }

    #[test]
    fn value_struct_vec_is_fixed_iterable_fold() {
        // A by-value `data_class` returned as `Option<Vec<T>>` (perftest's
        // `storage_get_vec` contract) synthesizes a FIXED-BUILDER fold wrapped in
        // an Optional layer: the field leaves cross raw per element and the
        // foreign folder rebuilds + appends them (no Java object is built on the
        // Rust side); `None` ⇒ a null list. Closes the data_class→Vec milestone.
        let mut reg =
            reg_with(&["fn storage_get_vec(s: &Storage) -> Option<Vec<Payload>> { todo!() }"]);
        let leaf = |name: &str, ty: syn::Type| UnfoldLeaf {
            name: name.to_string(),
            path: vec![ident(name)],
            out_ty: ty,
            identity: false,
            nullable: false,
            source: LeafSource::Field,
        };
        let vd = ValueDecon {
            key: TypeKey::from_type(&syn::parse_quote!(Payload)),
            source: syn::parse_quote!(Payload),
            leaves: vec![
                leaf("id", syn::parse_quote!(i64)),
                leaf("seq", syn::parse_quote!(i32)),
            ],
        };
        let declared: std::collections::HashSet<syn::Ident> =
            ["storage_get_vec"].iter().map(|s| ident(s)).collect();
        apply_value_structs(&mut reg, vec![vd], &declared).expect("apply_value_structs");

        let plan = reg
            .unfold_plans
            .get(&ident("storage_get_vec"))
            .expect("fixed-builder fold plan");
        assert!(plan.fixed_builder, "Vec<data_class> ⇒ fixed builder");
        assert!(
            matches!(&plan.shape,
                UnfoldShape::Optional((), inner)
                    if matches!(&**inner, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base))),
            "Option<Vec<T>> ⇒ Optional(Iterable(Base))"
        );
        assert_eq!(plan.delivery, Delivery::Callback);
        assert!(plan.decon.is_some(), "carries the field decon");
        assert!(
            plan.element.is_none(),
            "decomposed-leaf fold, not whole-element"
        );
        assert_eq!(plan.leaves.len(), 2, "field leaves cross raw per element");
        assert!(plan.leaves.iter().all(|l| l.source == LeafSource::Field));
        assert!(!plan.by_ref, "owned Vec<Payload> elements");
    }

    #[test]
    fn value_struct_slice_callback_is_fixed_iterable_fold() {
        // An `impl Fn(&[data_class])` callback arg (perftest's
        // `storage_callback_vec`) synthesizes an Iterable fixed-folder
        // `callback_arg_plans` entry keyed by the `&[Payload]` arg: the
        // trampoline folds each element's field leaves into a foreign list, the
        // user callback still sees the whole `List<Payload>`.
        let mut reg = reg_with(&[
            "fn storage_callback_vec(f: impl Fn(&[Payload]) + Send + Sync + 'static) { todo!() }",
        ]);
        let leaf = |name: &str, ty: syn::Type| UnfoldLeaf {
            name: name.to_string(),
            path: vec![ident(name)],
            out_ty: ty,
            identity: false,
            nullable: false,
            source: LeafSource::Field,
        };
        let vd = ValueDecon {
            key: TypeKey::from_type(&syn::parse_quote!(Payload)),
            source: syn::parse_quote!(Payload),
            leaves: vec![
                leaf("id", syn::parse_quote!(i64)),
                leaf("seq", syn::parse_quote!(i32)),
            ],
        };
        let declared: std::collections::HashSet<syn::Ident> =
            ["storage_callback_vec"].iter().map(|s| ident(s)).collect();
        apply_value_structs(&mut reg, vec![vd], &declared).expect("apply_value_structs");

        let key = TypeKey::from_type(&syn::parse_quote!(&[Payload]));
        let plan = reg
            .callback_arg_plans
            .get(&key)
            .expect("slice callback-arg fold plan");
        assert!(plan.fixed_builder, "&[data_class] ⇒ fixed folder");
        assert!(
            matches!(&plan.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)),
            "&[T] ⇒ Iterable(Base)"
        );
        assert_eq!(plan.delivery, Delivery::Callback);
        assert!(plan.decon.is_some(), "carries the field decon");
        assert!(plan.element.is_none(), "decomposed-leaf fold");
        assert_eq!(plan.leaves.len(), 2);
        assert!(plan.leaves.iter().all(|l| l.source == LeafSource::Field));
        // A scalar `&Payload` callback arg must stay a Base fixed builder.
        let mut reg2 = reg_with(&[
            "fn storage_callback(f: impl Fn(&Payload) + Send + Sync + 'static) { todo!() }",
        ]);
        let vd2 = ValueDecon {
            key: TypeKey::from_type(&syn::parse_quote!(Payload)),
            source: syn::parse_quote!(Payload),
            leaves: vec![leaf("id", syn::parse_quote!(i64))],
        };
        let declared2: std::collections::HashSet<syn::Ident> =
            ["storage_callback"].iter().map(|s| ident(s)).collect();
        apply_value_structs(&mut reg2, vec![vd2], &declared2).expect("apply_value_structs");
        let scalar = reg2
            .callback_arg_plans
            .get(&TypeKey::from_type(&syn::parse_quote!(&Payload)))
            .expect("scalar callback-arg plan");
        assert!(matches!(scalar.shape, UnfoldShape::Base), "&T ⇒ Base");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZError));
        acc.add_deconstructor_record(ident("z_error_message"), "z_error_message");
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
    fn default_output_applies_to_owned_and_borrow_returns() {
        // Default-everywhere: the ZKeyExpr deconstructor auto-applies to BOTH a
        // `&ZKeyExpr` (borrow) and an owned `ZKeyExpr` return. (`Result<…>` returns
        // are excluded — they keep a handle — and `fun_accessor`s are skipped.)
        let mut reg = reg_with(&[
            "fn z_borrow_keyexpr(s: &ZSession) -> &ZKeyExpr { todo!() }",
            "fn z_make_keyexpr(s: &ZSession) -> ZKeyExpr { todo!() }",
            "fn z_keyexpr_as_str(k: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
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
    fn callback_arg_borrowed_decomposed() {
        // A BORROWED `impl Fn(&ZSample)` decomposes through the same default
        // deconstructor as the by-value case, but with `by_ref = true` (leaves
        // read through the reference) and keyed under the actual `&ZSample` arg
        // type — so `callback_input`/`callback_iface_spec` find it.
        let mut reg = reg_with(&[
            "fn z_declare_sub(cb: impl Fn(&ZSample) + Send + Sync + 'static) { todo!() }",
            "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
            "fn z_sample_kind(s: &ZSample) -> SampleKind { todo!() }",
            "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        ]);
        let mut acc = Deconstructors::default();
        acc.ensure_default_deconstructor(syn::parse_quote!(ZKeyExpr));
        acc.add_deconstructor_record_id();
        acc.add_deconstructor_record(ident("z_keyexpr_as_str"), "z_keyexpr_as_str");
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_key_expr"), "z_sample_key_expr");
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_declare_sub"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");

        // No plan under the bare `ZSample` key — only under the borrowed arg type.
        assert!(reg
            .callback_arg_plans
            .get(&TypeKey::from_type(&syn::parse_quote!(ZSample)))
            .is_none());
        let plan = reg
            .callback_arg_plans
            .get(&TypeKey::from_type(&syn::parse_quote!(&ZSample)))
            .expect("callback-arg plan for &ZSample");
        assert!(plan.by_ref, "the callback only borrows the delivered value");
        assert_eq!(plan.source.to_token_stream().to_string(), "ZSample");
        assert!(matches!(plan.shape, UnfoldShape::Base));
        assert_eq!(plan.delivery, Delivery::Callback);
        assert_eq!(plan.leaves.len(), 3);
        assert!(plan.leaves[0].identity);
        assert_eq!(plan.leaves[0].path[0].to_string(), "z_sample_key_expr");
        assert_eq!(
            plan.leaves[2].out_ty.to_token_stream().to_string(),
            "SampleKind"
        );
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
        acc.ensure_default_deconstructor(syn::parse_quote!(ZSample));
        acc.add_deconstructor_record(ident("z_sample_kind"), "z_sample_kind");
        let declared: std::collections::HashSet<syn::Ident> =
            ["z_batched"].iter().map(|s| ident(s)).collect();
        apply(&mut reg, &acc, &declared, &acc_set()).expect("apply");
        assert!(reg.callback_arg_plans.is_empty());
    }

    #[test]
    fn leaf_vec_fold_synthesizes_whole_element_plans() {
        // `Vec<String>` / `Option<Vec<ZenohId>>` returns and an `impl Fn(&[String])`
        // callback arg synthesize FIXED **whole-element** folds (no decon, element
        // set, no leaves) — the single-leaf dual of the `data_class` Vec fold.
        let mut reg = reg_with(&[
            "fn hello_get_locators(h: &Hello) -> Vec<String> { todo!() }",
            "fn session_peers(s: &Session) -> Option<Vec<ZenohId>> { todo!() }",
            "fn on_strings(f: impl Fn(&[String]) + Send + Sync + 'static) { todo!() }",
        ]);
        let declared: std::collections::HashSet<syn::Ident> =
            ["hello_get_locators", "session_peers", "on_strings"]
                .iter()
                .map(|s| ident(s))
                .collect();
        let elements = vec![syn::parse_quote!(String), syn::parse_quote!(ZenohId)];
        apply_leaf_vec_folds(&mut reg, elements, &declared).expect("apply_leaf_vec_folds");

        // `Vec<String>` return ⇒ Iterable(Base), whole element.
        let p = reg
            .unfold_plans
            .get(&ident("hello_get_locators"))
            .expect("Vec<String> plan");
        assert!(p.fixed_builder, "synthesized leaf fold is fixed");
        assert!(matches!(&p.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)));
        assert_eq!(p.delivery, Delivery::Callback);
        assert!(p.decon.is_none(), "whole-element fold carries no decon");
        assert!(p.leaves.is_empty(), "no decomposed leaves");
        assert_eq!(
            p.element.as_ref().map(|t| t.to_token_stream().to_string()),
            Some("String".to_string())
        );

        // `Option<Vec<ZenohId>>` ⇒ Optional(Iterable(Base)).
        let p2 = reg
            .unfold_plans
            .get(&ident("session_peers"))
            .expect("Option<Vec<ZenohId>> plan");
        assert!(p2.fixed_builder);
        assert!(matches!(&p2.shape,
            UnfoldShape::Optional((), inner)
                if matches!(&**inner, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base))));
        assert_eq!(
            p2.element.as_ref().map(|t| t.to_token_stream().to_string()),
            Some("ZenohId".to_string())
        );

        // `impl Fn(&[String])` callback arg ⇒ Iterable fold keyed by `&[String]`.
        let key = TypeKey::from_type(&syn::parse_quote!(&[String]));
        let cb = reg
            .callback_arg_plans
            .get(&key)
            .expect("slice callback fold plan");
        assert!(cb.fixed_builder);
        assert!(matches!(&cb.shape, UnfoldShape::Iterable(i) if matches!(**i, UnfoldShape::Base)));
        assert!(cb.element.is_some());
        assert!(cb.decon.is_none());
    }

    #[test]
    fn leaf_vec_fold_skips_unnominated_and_preexisting() {
        // An un-nominated element is left on the ArrayList path (no plan); a fn
        // that already has a plan is never overwritten.
        let mut reg = reg_with(&[
            "fn other(x: &X) -> Vec<NotNominated> { todo!() }",
            "fn strings() -> Vec<String> { todo!() }",
        ]);
        let declared: std::collections::HashSet<syn::Ident> =
            ["other", "strings"].iter().map(|s| ident(s)).collect();
        // Pre-seed `strings` with a sentinel plan to prove it is preserved.
        let sentinel = UnfoldPlan {
            source: syn::parse_quote!(String),
            decon: None,
            by_ref: false,
            shape: UnfoldShape::Base,
            leaves: vec![],
            element: None,
            delivery: Delivery::Return,
            convert_out_ty: None,
            fixed_builder: false,
        };
        reg.unfold_plans.insert(ident("strings"), sentinel);
        apply_leaf_vec_folds(&mut reg, vec![syn::parse_quote!(String)], &declared)
            .expect("apply_leaf_vec_folds");
        assert!(
            reg.unfold_plans.get(&ident("other")).is_none(),
            "un-nominated `NotNominated` element ⇒ no fold plan"
        );
        assert_eq!(
            reg.unfold_plans.get(&ident("strings")).map(|p| p.delivery),
            Some(Delivery::Return),
            "pre-existing plan preserved (not overwritten)"
        );
    }
}
