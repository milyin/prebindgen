//! Output (data) expansion — the dual of constructor expansion
//! ([`crate::expand`]). A function returning a rich type is *decomposed* by a
//! **deconstructor** into a set of leaf values.
//!
//! A **deconstructor** (a type-level `expand_return!` `.field*` list,
//! or the per-fn `.expand_return` override) is a
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
//! Resolution is language-agnostic: it turns the declarations into
//! [`UnfoldPlan`]s (stored on the registry, keyed by function ident) and
//! registers every leaf's `out_ty` as a required **output** so the resolver
//! produces its converter (and projection). The jnigen adapter reads the
//! plan at the return-emission site.
//!
//! [`Iterable`]: UnfoldShape::Iterable

use std::collections::HashSet;

use crate::{declared_target::check_declared_target, registry::TypeKey};

mod error;
mod walk;
pub use walk::{
    bind_as_option, bind_hoists, compose_step, conditional_arm, fold_steps, project_leading_fields,
    reach_leaf, reached_is_ours, segment, segments, segments_at, DecomposedLeaf, DeliveryBridge,
    Hoisted, LeafAt, LeafPlace, Reach, Slot,
};

mod plan;

pub use self::{
    error::{UnfoldDeclError, UnfoldError},
    plan::{
        steps_are_movable, DeconId, DeconSpec, Hoist, LeafSource, PathStep, UnfoldLeaf, UnfoldPlan,
        UnfoldShape,
    },
};

// ──────────────────────────────────────────────────────────────────────
// Where applying the declarations puts its answers
// ──────────────────────────────────────────────────────────────────────

/// One output-side registration a decomposition asks the registry for.
///
/// Applying the declarations reads the model and produces plans; the only thing
/// it needs the registry itself for is to say which readings must cross on the
/// output side. Those asks are recorded here, in the order they were made, and
/// replayed onto the registry's type tables in one place — so the plans can be
/// built before a registry exists, and nothing but the replay mutates it.
///
/// The order is part of the meaning: [`Self::Unrequire`] drops a demand an
/// earlier [`Self::Output`] made.
#[derive(Clone, Debug)]
pub enum Requirement {
    /// This reading must cross on the output side, and a converter for it must
    /// resolve.
    Output(prebindgen_flat::flat::TypeRef),
    /// This reading enters the output table without demanding a converter — a
    /// type a plan *names* rather than one that crosses.
    Reference(prebindgen_flat::flat::TypeRef),
    /// Drop an earlier demand that this reading's converter must resolve. The
    /// table entry stays, so a converter is still produced if one resolves.
    Unrequire(prebindgen_flat::flat::TypeRef),
}

/// Applying decomposition declarations: the model they are read against, the
/// plans they produce, and the registrations they ask for.
///
/// The adapter that declared the decompositions owns this. It holds no
/// registry: every `apply*` function here reads signatures from the model and
/// writes plans and [`Requirement`]s into this, which is what lets the plans be
/// built at declaration time.
pub struct Unfolding<'f> {
    flat: &'f prebindgen_flat::flat::Flat,
    requirements: Vec<Requirement>,

    /// The plans built so far.
    pub plans: Unfolded,
}

/// The decomposition plans applying a set of declarations produced.
///
/// The adapter keeps this and reads it back at every emission site; nothing in
/// the registry holds one.
#[derive(Default, Clone)]
pub struct Unfolded {
    /// Resolved output-expansion plans, keyed by function ident. Read at the
    /// return-emission site.
    pub unfold_plans: std::collections::HashMap<syn::Ident, UnfoldPlan>,

    /// Resolved **error**-position expansion plans, keyed by function ident: the
    /// decomposition of a fallible fn's `Result<_, E>` domain error `E` (from
    /// `.convert_error` / `.deconstruct_error`). Separate from
    /// [`Self::unfold_plans`] — a fn may have both an output and an error plan.
    pub error_plans: std::collections::HashMap<syn::Ident, UnfoldPlan>,

    /// Default decomposition of a **callback argument** type — the `T` of a
    /// declared fn's `impl Fn(T, …)` parameter — keyed by the bare arg type
    /// (type-level, fn-independent), from the type's default deconstructor
    /// (`by_ref = false`: the trampoline owns the value). A type without a
    /// default deconstructor has no entry and is delivered whole.
    pub callback_arg_plans: std::collections::HashMap<TypeKey, UnfoldPlan>,

    /// The declaration-default decomposition per deconstructor declaration
    /// ([`DeconId`]) — resolved once with normalized inputs, independent of
    /// using functions and processing order. The single source an adapter
    /// derives declaration-keyed signature artifacts (generated callback
    /// interfaces, say) from, so every function selecting the same declaration
    /// sees one signature by construction.
    pub decon_plans: std::collections::HashMap<DeconId, DeconSpec>,
}

impl<'f> Unfolding<'f> {
    /// Start applying declarations against `flat`.
    pub fn new(flat: &'f prebindgen_flat::flat::Flat) -> Self {
        Self {
            flat,
            requirements: Vec::new(),
            plans: Unfolded::default(),
        }
    }

    /// The model the declarations are read against.
    pub fn flat(&self) -> &'f prebindgen_flat::flat::Flat {
        self.flat
    }

    /// The registrations asked for, in the order they were asked. Hand these to
    /// [`Decompositions::requirements`](crate::Decompositions::requirements).
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    /// Per callback-argument type, the readings its decomposition delivers.
    ///
    /// The ordering fact the registry needs and cannot see: a callback argument
    /// delivered as leaves needs each leaf's own conversion before the
    /// callback's can be built, and a leaf is named by a plan rather than by the
    /// argument's syntax. Hand this to
    /// [`Decompositions::callback_arg_leaves`](crate::Decompositions::callback_arg_leaves).
    pub fn callback_arg_leaves(
        &self,
    ) -> std::collections::HashMap<TypeKey, Vec<prebindgen_flat::flat::TypeRef>> {
        self.plans
            .callback_arg_plans
            .iter()
            .map(|(key, plan)| {
                (
                    key.clone(),
                    plan.leaves.iter().map(|leaf| leaf.out_ty.clone()).collect(),
                )
            })
            .collect()
    }

    /// Take the plans, leaving the requirements behind.
    pub fn into_plans(self) -> Unfolded {
        self.plans
    }

    /// Register `reading` (and its nested positions) as a required **output** so
    /// the resolver produces a converter for it.
    fn require_output(&mut self, reading: &prebindgen_flat::flat::TypeRef) {
        self.requirements.push(Requirement::Output(reading.clone()));
    }

    /// Register `reading` as an output cell **without** demanding a converter —
    /// a type some plan names rather than one that crosses. What a
    /// [`SumTag`](LeafSource::SumTag) selector needs: it names *which* sum it
    /// chooses between, and that sum has no whole-value output converter at all,
    /// so requiring one would fail resolution (#282).
    fn reference_output(&mut self, reading: &prebindgen_flat::flat::TypeRef) {
        self.requirements
            .push(Requirement::Reference(reading.clone()));
    }

    /// Drop `reading` from the required-output set. Used by
    /// [`apply_leaf_vec_folds`]: when a `Vec<T>` / `Option<Vec<T>>` return is
    /// delivered element-by-element through a fold, the whole-collection
    /// converter is genuinely not needed — and for a `Vec` of opaque handles it
    /// cannot resolve at all, so requiring it would wrongly fail resolution.
    fn unrequire_output(&mut self, reading: &prebindgen_flat::flat::TypeRef) {
        self.requirements
            .push(Requirement::Unrequire(reading.clone()));
    }
}

// ──────────────────────────────────────────────────────────────────────
// Declarations (populated by the language builder)
// ──────────────────────────────────────────────────────────────────────

/// One record (field) of a deconstructor. A deconstructor is a product: every
/// record contributes a leaf.
// large_enum_variant: a handful of records exist per binding — boxing the
// syn payloads would only complicate the arms (same trade-off as
// `ConvertSourceKind`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum DeconRecord {
    /// Read this field by calling the accessor function `f(&T) -> &F`. `name`
    /// is the author-supplied leaf name, used **literally** (no casing /
    /// stripping); it may not contain the reserved `"__"` chain separator.
    /// An accessor whose return type has its own deconstructor splices that
    /// child's records with the leaf names prefixed `name__<child>`.
    Acc { func: syn::Ident, name: String },
    /// Read this field by calling a **custom, locally-defined** accessor: any
    /// callable in the binding crate (`path`) with a STATED return type
    /// (`ty`) — there is no `#[prebindgen]` item behind it, so the signature
    /// cannot be looked up. The adapter's `local_functions()` pre-pass
    /// synthesizes a registry entry from the stated signature (so call
    /// qualification and `Option`-nesting checks work unchanged); splicing
    /// follows [`Self::Acc`] rules except that a self-referential field (its
    /// type already being decomposed) degrades to a plain converter leaf
    /// instead of a cycle error — that is what lets such a field re-deliver
    /// (part of) the value itself, e.g. under a binding-defined condition.
    LocalAcc { path: syn::Path, name: String },
    /// The value itself — the handle/identity leaf (cloned for a `&T` return,
    /// moved for an owned `T`). At most one per
    /// deconstructor.
    Identity,
    /// Read the fields of the type's **value form**: call `func` once
    /// (`f(&T) -> TStruct`) and contribute one record per [`FieldRecord`],
    /// reached by field access on the returned struct. The language adapter
    /// builds the field list (it knows which structs are declared classes and
    /// therefore inline); this record only says how to get there.
    ///
    /// Each field then decomposes exactly like an [`Acc`](Self::Acc) record's
    /// return does — its own `records` if the
    /// declaration overrode it, else its type's own deconstructor if it has
    /// one, else one leaf — so a value form and a hand-written field list
    /// produce the same leaves.
    Fields {
        func: syn::Ident,
        /// The accessor **consumes** its receiver (`f(T) -> TStruct`): the
        /// value is moved in and each field moved *out* into its leaf, instead
        /// of being cloned out of a borrow. Declared by the adapter rather than
        /// read off the signature — giving the value away is a boundary
        /// decision — and cross-checked against the signature when the records
        /// are flattened, so the two cannot drift.
        consuming: bool,
        fields: Vec<FieldRecord>,
    },
}

/// One field of a value form (see [`DeconRecord::Fields`]).
#[derive(Clone)]
pub struct FieldRecord {
    /// Field-access chain from the value form's returned struct. More than one
    /// element when the adapter inlined a nested declared class.
    pub members: Vec<syn::Ident>,
    /// The leaf name (already `__`-joined across inlined nesting).
    pub name: String,
    /// The field's **reading**, `Option` / `Vec` layers included — and its
    /// syntax with it, which is what a leaf's `out_ty` spells.
    ///
    /// The declaration carries this rather than naming a `syn::Type` for the
    /// walk to look up, because there was nothing to look up: a field record is
    /// built from an element whose every field already has a reading, and the
    /// types it names are the ones the caller registers *after* the walk
    /// returns. Asking the registry here was asking before registration
    /// (#266) — a lookup that could only miss, answered by a second source of
    /// readings that hid the ordering.
    pub ty: prebindgen_flat::flat::TypeRef,
    /// How this field decomposes.
    pub decon: FieldDecon,
}

/// How one [`FieldRecord`] decomposes.
#[derive(Clone)]
pub enum FieldDecon {
    /// By the field type's own deconstructor if it has one, else one leaf —
    /// the same default a [`DeconRecord::Acc`] record's return follows.
    Default,
    /// Explicit records, replacing the type default wholesale (the declaration
    /// stated this field's complete leaf set).
    Records(Vec<DeconRecord>),
    /// Leaves the **adapter** built, appended with this field's path and name
    /// prefixed onto each. For shapes whose leaf structure only the adapter
    /// knows — a decomposed sum, which is a selector plus one group per
    /// alternative rather than a product of records.
    Leaves(Vec<UnfoldLeaf>),
}

impl DeconRecord {
    /// The fn ident of a [`Self::LocalAcc`] — its path's last segment (the
    /// name the emitted call resolves to under the path-prefix origin).
    fn local_ident(path: &syn::Path) -> syn::Ident {
        path.segments
            .last()
            .expect("field!(...).with(...): empty accessor path")
            .ident
            .clone()
    }
}

/// A type-level deconstructor declaration (`expand_return!(T).field*`): the
/// complete, ordered record list decomposing `target`. An immutable record —
/// the leaf order is the declaration order of the `records` vector.
#[derive(Clone)]
pub struct DeconstructorDecl {
    /// The type being decomposed, as an **identity** — see
    /// [`ConstructorDecl::target`](crate::expand::ConstructorDecl::target).
    pub target: TypeKey,
    pub records: Vec<DeconRecord>,
    /// Auto-apply this deconstructor to every matching declared fn (`Some`
    /// carries the inferred `(target-position, delivery)` to use). Always
    /// `Some` for type-level default (`expand_return!`) declarations.
    pub default: Option<(DeconTarget, Delivery)>,
}

/// How an output expansion chooses the deconstructor for a function's return
/// type: the type's default (`expand_return!`-declared) or a per-fn
/// inline record list (`.expand_return`).
#[derive(Clone)]
pub enum DeconSel {
    /// Use the return type's unique deconstructor (error if ambiguous).
    TopLevel,
    /// Per-fn override (`.expand_return`): use exactly these
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

/// A per-fn output expansion (`.expand_return(expand_return!(T)…)`) —
/// decompose `func`'s return (or error position) via the record list.
/// Recorded as an explicit decl so the auto-`default` skips it; an
/// identity-only record list lowers to the raw whole-value return at
/// resolution.
#[derive(Clone)]
pub struct OutputDecl {
    pub func: syn::Ident,
    pub sel: DeconSel,
    pub target: DeconTarget,
    pub delivery: Delivery,
    /// The type the per-fn decl was declared for (`expand_return!(T)`) —
    /// cross-checked against the fn's peeled return type at resolution.
    /// `None` for internally-synthesized decls (the type comes from the
    /// return itself).
    pub declared_source: Option<TypeKey>,
}

/// Deconstructor / output-expansion declarations gathered from a language
/// builder — an immutable record set: complete values, no build protocol.
/// Declaration order is the vector order; leaf order is each record
/// vector's order. Handed to [`apply`]; empty or duplicate declarations are
/// diagnosed there (collected), not at construction.
#[derive(Clone, Default)]
pub struct Deconstructors {
    pub deconstructors: Vec<DeconstructorDecl>,
    pub outputs: Vec<OutputDecl>,
    /// Identity-only per-fn field-set opt-outs: fns excluded from the
    /// default auto-apply (the raw whole-value return).
    pub skip_output: std::collections::HashSet<syn::Ident>,
}

// ──────────────────────────────────────────────────────────────────────
// apply
// ──────────────────────────────────────────────────────────────────────

/// Structural validation of the declaration records — duplicate targets,
/// collected (EVERY offender before failing) so a build surfaces all
/// declaration problems at once. Empty record lists are NOT diagnosed:
/// an empty inline list is the valid whole-element delivery form.
fn validate_declarations(acc: &Deconstructors) -> Result<(), UnfoldError> {
    let mut entries: Vec<UnfoldDeclError> = Vec::new();
    let mut decon_targets: std::collections::HashSet<String> = std::collections::HashSet::new();
    for d in &acc.deconstructors {
        let target = d.target.as_str().to_string();
        if !decon_targets.insert(target.clone()) {
            entries.push(UnfoldDeclError::DuplicateDeconstructor { target });
        }
    }
    let mut output_keys: std::collections::HashSet<(String, DeconTarget)> =
        std::collections::HashSet::new();
    for od in &acc.outputs {
        if !output_keys.insert((od.func.to_string(), od.target)) {
            entries.push(UnfoldDeclError::DuplicateOutput {
                func: od.func.clone(),
                target: od.target,
            });
        }
    }
    if entries.is_empty() {
        Ok(())
    } else {
        Err(UnfoldError::InvalidDeclarations { entries })
    }
}

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
pub fn apply(
    registry: &mut Unfolding<'_>,
    acc: &Deconstructors,
    declared_fns: &std::collections::HashSet<syn::Ident>,
    accessor_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    validate_declarations(acc)?;
    // Binding-local accessors (`LocalAcc` records) resolve through registry
    // entries synthesized by the adapter's `local_functions()` pre-pass in
    // the builder's scan — by this point they read exactly like
    // `#[prebindgen]` accessors.

    // Gate: every accessor-function record of every declared deconstructor must
    // be a `.fun_accessor` (the single source of truth for "accessor").
    // Binding-local records skip the gate — there is no `#[prebindgen]` item
    // behind them — but keep the reserved-separator name check.
    for d in &acc.deconstructors {
        check_records(&d.records, accessor_fns)?;
    }

    // Explicit decls first; they take precedence over (and suppress) a default
    // for the same `(fn, target)`.
    let mut done: std::collections::HashSet<(syn::Ident, DeconTarget)> = Default::default();
    for ed in &acc.outputs {
        // Per-fn decl cross-check: the decl's declared type must match the
        // fn's peeled (`Option`/`Vec`/`&`) return type — the typo guard for
        // `.expand_return(expand_return!(T)…)`.
        if let Some(declared) = &ed.declared_source {
            let ret = registry
                .flat()
                .function(&ed.func)
                .map(|f| f.ret.clone())
                .ok_or_else(|| UnfoldError::UnknownFunction(ed.func.clone()))?;
            if !returns_type(&ret, declared) {
                return Err(UnfoldError::ReturnTypeMismatch {
                    func: ed.func.clone(),
                    declared: declared.as_str().to_string(),
                    actual: ret.to_string(),
                });
            }
        }
        // Identity-only field set = the raw whole-value return (the
        // complete-set rule: "the set is {self}"): no plan — mark done so the
        // type default doesn't re-apply, and let the return cross through the
        // type's ordinary output converter (borrowed-`&T`-capable).
        if let DeconSel::Inline(records) = &ed.sel {
            if matches!(records.as_slice(), [DeconRecord::Identity]) {
                done.insert((ed.func.clone(), ed.target));
                continue;
            }
        }
        process_decl(registry, acc, ed)?;
        done.insert((ed.func.clone(), ed.target));
    }

    // Default auto-apply: a type's deconstructor (`expand_return!`) is
    // applied to every declared fn that returns it (Output) or has it as a
    // `Result<_, E>` error (Error), unless the fn is `fun_accessor` or has a
    // per-fn override. `Delivery` is recomputed from leaf count inside
    // `process_decl` for Output (1 ⇒ Return, N ⇒ Callback).
    for d in &acc.deconstructors {
        if d.default.is_none() {
            continue;
        }
        let dkey = d.target.clone();
        let sel = DeconSel::TopLevel;
        for func in declared_fns {
            // Read accessors are never output-decomposed (they ARE the records).
            if accessor_fns.contains(func) {
                continue;
            }
            let Some(ret) = registry.flat().function(&func).map(|f| f.ret.clone()) else {
                continue;
            };
            // Error position: fn returns `Result<_, E>` and `E == d.target`.
            if let Some(err_ty) = ret.fallible_parts().map(|(_, e)| e) {
                if err_ty.key() == dkey && done.insert((func.clone(), DeconTarget::Error)) {
                    process_decl(
                        registry,
                        acc,
                        &OutputDecl {
                            func: func.clone(),
                            sel: sel.clone(),
                            target: DeconTarget::Error,
                            delivery: Delivery::Callback,
                            declared_source: None,
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
                        declared_source: None,
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
        let Some(params) = registry.flat().function(&func).map(|f| f.params.clone()) else {
            continue;
        };
        for param in &params {
            // The callback's argument types, read off the parameter's
            // classification. `TypeKind::Callback` carries them as `TypeRef`s, so
            // there is nothing to re-extract from the signature's syntax.
            let prebindgen_flat::flat::TypeKind::Callback { args } = param.ty.kind() else {
                continue;
            };
            for arg_ty in args {
                // A borrowed arg (`impl Fn(&T)`) decomposes through the same
                // machinery as a `&T` return: strip the leading `&` to reach the
                // deconstructor target and set `by_ref` so the leaves are read
                // (cloned) through the reference instead of by move. The plan is
                // keyed under the ACTUAL arg type (`&T`) — that is what
                // `callback_input`/`callback_iface_spec` look up.
                let (by_ref, core_ty) = peel_borrow(arg_ty);
                // Only a NAMED core can match a deconstructor target: an
                // `Option<T>` / `Vec<T>` / tuple arg is delivered whole. The model
                // says which, and `unwrapped` is where a wrapper the destination
                // cannot see — `Box<T>` — stops reading as un-nameable.
                if !matches!(
                    core_ty.unwrapped().kind(),
                    prebindgen_flat::flat::TypeKind::Named { .. }
                ) {
                    continue;
                }
                let key = arg_ty.key();
                if registry.plans.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                let core_key = core_ty.key();
                let Some(d) = acc
                    .deconstructors
                    .iter()
                    .find(|d| d.default.is_some() && d.target == core_key)
                else {
                    continue;
                };
                let ed = OutputDecl {
                    func: func.clone(),
                    sel: DeconSel::TopLevel,
                    target: DeconTarget::Output,
                    delivery: Delivery::Callback,
                    declared_source: None,
                };
                let decon = decl_id(&core_key, d);
                let records = d.records.clone();
                register_decon_spec(registry, acc, &decon, &records, core_ty)?;
                let plan = build_plan(
                    acc,
                    registry,
                    &ed,
                    by_ref,
                    core_ty,
                    UnfoldShape::Base,
                    &records,
                    decon,
                )?;
                if plan.leaves.is_empty() {
                    continue;
                }
                for leaf in &plan.leaves {
                    registry.require_output(&leaf.out_ty);
                }
                registry.plans.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// A synthesized by-value `data_class` decomposition, produced by the language
/// adapter (which knows the per-field encoding — projections, enums, nested
/// classes) and handed to [`apply_value_structs`].
/// Its [`leaves`](Self::leaves)
/// are [`LeafSource::Reach`] leaves: each crosses the boundary as its own field
/// value and the foreign side reassembles the object (no Java object is built
/// on the Rust side).
pub struct ValueDecon {
    /// Canonical key of the value struct (the `DeconId::Default` key).
    pub key: TypeKey,
    /// The struct type (owned) the leaves decompose.
    pub source: prebindgen_flat::flat::TypeRef,
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
pub fn apply_value_structs(
    registry: &mut Unfolding<'_>,
    decons: Vec<ValueDecon>,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    for vd in &decons {
        let decon = wire_fixed_decon(registry, &vd.key, &vd.source, &vd.leaves)?;

        // Output position: a declared fn returning the struct
        // (`T` / `&T` / `Option<T|&T>` / `Vec<T|&T>`) decomposes into a
        // fixed-builder plan. (`Result<T, E>` is left to the whole-value
        // converter — the synthesizer covers the infallible returns.)
        wire_fixed_returns(registry, vd, &decon, declared_fns, false);

        // Callback-argument position: an `impl Fn(&T)` / `impl Fn(T)` parameter
        // of a declared fn delivers the flattened leaves to the foreign
        // callback, which reassembles the whole value via the data class's
        // `fromParts` before invoking the user's typed callback (the group
        // reassembly lives in the JNI adapter's `asRaw` proxy).
        wire_fixed_callbacks(registry, vd, &decon, declared_fns)?;
    }
    Ok(())
}

/// A synthesized **sum** decomposition, produced by the language adapter (which
/// knows how each payload encodes) and handed to [`apply_sum_returns`] — the
/// selector-carrying sibling of [`ValueDecon`].
///
/// Its [`leaves`](Self::leaves) are a [`LeafSource::SumTag`] selector followed
/// by one **group** per alternative ([`LeafSource::VariantField`] leaves
/// carrying [`UnfoldLeaf::groups`]). Exactly one group is live per value; the
/// emitter reads the whole list as ONE `match` over the value, filling every
/// inert slot with its wire default. The foreign side picks the live group by
/// the tag and rebuilds the alternative, so no object is built on the Rust
/// side.
pub struct SumDecon {
    /// Canonical key of the sum type (the `DeconId::Default` key).
    pub key: TypeKey,
    /// The enum type (owned) the leaves decompose.
    pub source: prebindgen_flat::flat::TypeRef,
    /// The tag leaf followed by every variant's group, in tag order.
    pub leaves: Vec<UnfoldLeaf>,
}

/// Wire the synthesized **sum** decompositions into the registry — the
/// [`apply_value_structs`] analog for a value whose alternatives are chosen at
/// runtime instead of being a fixed product.
///
/// For every declared function returning the sum (`E` / `&E` / `Option<E>` /
/// `Vec<E>`) and every `impl Fn(E)` / `impl Fn(&E)` callback parameter, builds a
/// **fixed-builder** [`UnfoldPlan`] over the tag + group leaves, registering
/// each leaf's `out_ty` as a required output.
///
/// A sum has no converter of its own (it is boundary-only: a tag plus groups is
/// not a single wire), so the declared return's scan-time output requirement —
/// including the `Option<E>` / `Vec<E>` layers, which the boundary-only pass
/// does not reach — is dropped here as the plan takes over.
///
/// Put every leaf's `out_ty` in the table, and demand a converter for the ones
/// that need one.
///
/// **Every leaf is registered; only a converter-bearing leaf is a root** (#282).
/// The two are separate facts and this is the one place a sum plan states both:
/// a cell says the type entered the pipeline, a root says the binding needs its
/// conversion to resolve. The `SumTag` selector is registered and not required —
/// it names *which* sum it chooses between, and a sum has no whole-value output
/// converter, so requiring one would fail resolution over a type that never
/// crosses whole.
///
/// This used to `filter` the selector out entirely, which left its `out_ty`
/// with a cell only when the adapter happened to declare the sum separately —
/// true for jnigen via `export_type`, and not true at all for a registry
/// assembled without declarations. The invariant holds by construction now
/// rather than by declaration order.
fn register_leaves(registry: &mut Unfolding<'_>, leaves: &[UnfoldLeaf]) {
    for leaf in leaves {
        if leaf.has_converter() {
            registry.require_output(&leaf.out_ty);
        } else {
            registry.reference_output(&leaf.out_ty);
        }
    }
}

/// Runs in `write_rust` right after [`apply_value_structs`] and before `resolve`.
pub fn apply_sum_returns(
    registry: &mut Unfolding<'_>,
    decons: Vec<SumDecon>,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    for sd in &decons {
        let decon = wire_fixed_decon(registry, &sd.key, &sd.source, &sd.leaves)?;
        let vd = ValueDecon {
            key: sd.key.clone(),
            source: sd.source.clone(),
            leaves: sd.leaves.clone(),
        };
        wire_fixed_returns(registry, &vd, &decon, declared_fns, true);
        wire_fixed_callbacks(registry, &vd, &decon, declared_fns)?;
    }
    Ok(())
}

/// Register the declaration-canonical [`DeconSpec`] of a synthesized
/// decomposition (first writer wins) and return its identity.
fn wire_fixed_decon(
    registry: &mut Unfolding<'_>,
    key: &TypeKey,
    source: &prebindgen_flat::flat::TypeRef,
    leaves: &[UnfoldLeaf],
) -> Result<DeconId, UnfoldError> {
    let decon = DeconId::Default(key.to_string());
    require_unique_leaf_names(source, leaves)?;
    registry
        .plans
        .decon_plans
        .entry(decon.clone())
        .or_insert_with(|| DeconSpec {
            source: source.clone(),
            leaves: leaves.to_vec(),
        });
    Ok(decon)
}

/// Build the fixed-builder output plan for every declared fn returning the
/// decomposed type (`T` / `&T` / `Option<T|&T>` / `Vec<T|&T>`). `no_converter`
/// marks a type that has no whole-value converter at all (a sum), so the
/// declared return's scan-time output requirement is dropped as the plan
/// replaces it.
fn wire_fixed_returns(
    registry: &mut Unfolding<'_>,
    vd: &ValueDecon,
    decon: &DeconId,
    declared_fns: &std::collections::HashSet<syn::Ident>,
    no_converter: bool,
) {
    for func in declared_fns {
        let Some(ret) = registry.flat().function(&func).map(|f| f.ret.clone()) else {
            continue;
        };
        if !returns_type(&ret, &vd.key) || registry.plans.unfold_plans.contains_key(func) {
            continue;
        }
        // Shape over the leaf decomposition: peel an outer `Option`, then a
        // `Vec`, then a leading `&`. `Vec<T|&T>` ⇒ Iterable (a **fixed
        // folder**: each element's leaves cross raw and the foreign folder
        // rebuilds it + appends, so no Java object is built on the Rust
        // side); `Option<…>` wraps the inner shape in Optional (`None` ⇒ a
        // null result). `element: None` keeps the decomposed-leaf path. The
        // element/inner borrow-ness sets `by_ref` (the reach clones either
        // way).
        let layers = peel(&ret);
        let by_ref = layers.by_ref;
        // The model's layer stack is the plan's shape — `UnfoldShape` is `Shape`
        // — so there is nothing to rebuild here.
        let shape = layers.shape.clone();
        if no_converter {
            // The plan delivers the return leaf-by-leaf, so no converter is
            // needed for the declared return — and for a sum none can exist.
            // Drop the scan-time registrations of every layer (the boundary-only
            // pass only reaches the bare type), so the missing converters are not
            // flagged as unresolved-required.
            // EVERY layer, the `Vec` element included. The shape fold peels here,
            // so the matching unrequire belongs here; leaving the element out made
            // the invariant depend on the adapter's `boundary_only_types` covering
            // it — true for JniGenBuilder today, and the only reason a
            // `Vec<sum>`-only declaration resolves.
            for layer in &layers.layer_types {
                registry.unrequire_output(layer);
            }
        }
        register_leaves(registry, &vd.leaves);
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
            hoists: Vec::new(),
        };
        registry.plans.unfold_plans.insert(func.clone(), plan);
    }
}

/// Build a fixed-builder callback-arg plan for every `impl Fn(&T)`,
/// `impl Fn(T)`, or `impl Fn(Option<T>)` parameter (of a declared fn) whose
/// value is the decomposed type `vd`. The foreign callback receives the
/// flattened leaves (reassembled there) instead of a whole value built on the
/// Rust side. Separate from the
/// output-position wiring so the callback path (which needs the foreign-side
/// group-reassembly adapter) can be enabled on its own.
fn wire_fixed_callbacks(
    registry: &mut Unfolding<'_>,
    vd: &ValueDecon,
    decon: &DeconId,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    for func in declared_fns {
        let Some(params) = registry.flat().function(&func).map(|f| f.params.clone()) else {
            continue;
        };
        for param in &params {
            // The callback's argument types, read off the parameter's
            // classification. `TypeKind::Callback` carries them as `TypeRef`s, so
            // there is nothing to re-extract from the signature's syntax.
            let prebindgen_flat::flat::TypeKind::Callback { args } = param.ty.kind() else {
                continue;
            };
            for arg_ty in args {
                // Peel a leading `&`, then one by-value `Option`, then detect a
                // slice element. A scalar value-struct arg decomposes into a
                // `Base` fixed builder; `Option<T>` wraps that in `Optional` and
                // adds a presence slot. A run becomes an `Iterable` fixed FOLDER.
                // Optional runs are deliberately left to their own shape step.
                let (by_ref, after_ref) = peel_borrow(arg_ty);
                let (optional, after_optional) = match after_ref.optional_inner() {
                    Some(inner) if !by_ref => (true, inner),
                    _ => (false, after_ref),
                };
                // A run of `T` is an Iterable fold over the element; anything else
                // is a Base fold over the value itself. `Sequence` is the one
                // question, and it covers `[T]` and `Vec<T>` alike.
                let (shape, matches_key) = match after_optional.sequence_elem() {
                    Some(elem) if !optional => (
                        UnfoldShape::Iterable(Box::new(UnfoldShape::Base)),
                        elem.key() == vd.key,
                    ),
                    Some(_) => continue,
                    None => (UnfoldShape::Base, after_optional.key() == vd.key),
                };
                let shape = if optional {
                    UnfoldShape::Optional((), Box::new(shape))
                } else {
                    shape
                };
                if !matches_key {
                    continue;
                }
                let key = arg_ty.key();
                if registry.plans.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                register_leaves(registry, &vd.leaves);
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
                    hoists: Vec::new(),
                };
                registry.plans.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// Wire **whole-element** `Iterable` fold plans for bare `Vec<T>` /
/// `Option<Vec<T>>` returns and `impl Fn(&[T])` callback args whose element `T`
/// is a single leaf (String, scalar, opaque handle) nominated by the adapter
/// via `Prebindgen::leaf_vec_fold_elements`. Each
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
pub fn apply_leaf_vec_folds(
    registry: &mut Unfolding<'_>,
    elements: Vec<TypeKey>,
    declared_fns: &std::collections::HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    if elements.is_empty() {
        return Ok(());
    }
    let elem_keys = elements;
    // Is the leading-`&`-peeled `bare` one of the nominated single-leaf elements?
    let is_nominated = |bare: &prebindgen_flat::flat::TypeRef| elem_keys.contains(&bare.key());
    for func in declared_fns {
        let Some(params) = registry.flat().function(&func).map(|f| f.params.clone()) else {
            continue;
        };
        // Output position: `Vec<T>` / `Option<Vec<T>>` return. Skip if a plan
        // already exists (declared deconstructor / value-struct fold).
        if !registry.plans.unfold_plans.contains_key(func) {
            let Some(ret) = registry.flat().function(&func).map(|f| f.ret.clone()) else {
                continue;
            };
            let (optional, after_opt) = match ret.optional_inner() {
                Some(inner) => (true, inner),
                None => (false, &ret),
            };
            if let Some(vec_elem) = after_opt.sequence_elem() {
                let bare = peel_borrow(vec_elem).1;
                if is_nominated(bare) {
                    let inner_shape = UnfoldShape::Iterable(Box::new(UnfoldShape::Base));
                    let shape = if optional {
                        UnfoldShape::Optional((), Box::new(inner_shape))
                    } else {
                        inner_shape
                    };
                    registry.require_output(vec_elem);
                    // The fold delivers the return element-by-element, so the
                    // whole `Vec<T>` / `Option<Vec<T>>` converter is not needed.
                    // De-require it: for String / scalar elements it still
                    // resolves (and is emitted as harmless dead code); for an
                    // opaque-handle element it cannot resolve (`jlong` wire isn't
                    // JObject-shaped), and de-requiring keeps that `None` from
                    // being flagged as an unresolved-required error.
                    registry.unrequire_output(&ret);
                    registry
                        .plans
                        .unfold_plans
                        .insert(func.clone(), whole_leaf_fold_plan(vec_elem, shape));
                }
            }
        }
        // Callback-arg position: `impl Fn(&[T])` / `impl Fn([T])`.
        for param in &params {
            // The callback's argument types, read off the parameter's
            // classification. `TypeKind::Callback` carries them as `TypeRef`s, so
            // there is nothing to re-extract from the signature's syntax.
            let prebindgen_flat::flat::TypeKind::Callback { args } = param.ty.kind() else {
                continue;
            };
            for arg_ty in args {
                let (_, after_ref) = peel_borrow(arg_ty);
                let Some(elem) = after_ref.sequence_elem() else {
                    continue;
                };
                if !is_nominated(peel_borrow(elem).1) {
                    continue;
                }
                let key = arg_ty.key();
                if registry.plans.callback_arg_plans.contains_key(&key) {
                    continue;
                }
                registry.require_output(elem);
                let plan =
                    whole_leaf_fold_plan(elem, UnfoldShape::Iterable(Box::new(UnfoldShape::Base)));
                registry.plans.callback_arg_plans.insert(key, plan);
            }
        }
    }
    Ok(())
}

/// Build a fixed-builder whole-element fold [`UnfoldPlan`] for a single-leaf
/// element `vec_elem` (the `Vec`/slice element as written, keeping any leading
/// `&` so `into_iter()`'s yield matches the element's own output converter).
fn whole_leaf_fold_plan(
    vec_elem: &prebindgen_flat::flat::TypeRef,
    shape: UnfoldShape,
) -> UnfoldPlan {
    UnfoldPlan {
        source: vec_elem.clone(),
        decon: None,
        by_ref: peel_borrow(vec_elem).0,
        shape,
        leaves: vec![],
        element: Some(vec_elem.clone()),
        delivery: Delivery::Callback,
        convert_out_ty: None,
        fixed_builder: true,
        hoists: Vec::new(),
    }
}

/// The deconstructor gate: every accessor-function record must be a declared
/// `.fun_accessor` (the single source of truth for "accessor"), and no author
/// leaf name may contain the reserved `"__"` chain separator. Binding-local
/// records skip the accessor check — there is no `#[prebindgen]` item behind
/// them — but keep the name check.
///
/// Recurses into a value form's per-field override records, so an override is
/// held to the same rules as the declaration it replaces.
fn check_records(
    records: &[DeconRecord],
    accessor_fns: &HashSet<syn::Ident>,
) -> Result<(), UnfoldError> {
    for rec in records {
        let (func, name) = match rec {
            DeconRecord::Acc { func, name } => (Some(func), name),
            DeconRecord::LocalAcc { name, .. } => (None, name),
            DeconRecord::Identity => continue,
            // A value form's field names come from struct idents, not from the
            // author, so the `"__"` in an inlined nested name is the separator
            // doing its job. An author-supplied rename is checked where it is
            // declared.
            DeconRecord::Fields { func, fields, .. } => {
                if !accessor_fns.contains(func) {
                    return Err(UnfoldError::RecordNotAccessor { func: func.clone() });
                }
                for fr in fields {
                    if let FieldDecon::Records(recs) = &fr.decon {
                        check_records(recs, accessor_fns)?;
                    }
                }
                continue;
            }
        };
        if name.contains("__") {
            return Err(UnfoldError::ReservedSeparator { name: name.clone() });
        }
        if let Some(func) = func {
            if !accessor_fns.contains(func) {
                return Err(UnfoldError::RecordNotAccessor { func: func.clone() });
            }
        }
    }
    Ok(())
}

/// The arity layers over `ty`, the types they wrap, and the value underneath.
///
/// A thin owned view over
/// [`TypeRef::layer_stack`](prebindgen_flat::flat::TypeRef::layer_stack) and
/// [`layer_types`](prebindgen_flat::flat::TypeRef::layer_types): the borrows are
/// resolved to clones because these feed plan fields and registry calls that own
/// their types. The classification is the model's; only the copying is local, and
/// it happens **once**, where a value is stored — not on every question asked.
///
/// The stack **is** the plan's shape — `UnfoldShape` is `Shape` — so a caller
/// stores it rather than rebuilding one from flags.
struct Layered {
    /// The arity layers, outermost first.
    shape: UnfoldShape,
    /// Every type on the way down, outermost first — what a registration walks.
    layer_types: Vec<prebindgen_flat::flat::TypeRef>,
    /// Past the borrow too: what actually crosses — as an **identity**, which
    /// is all its one consumer ever asked of it.
    core: TypeKey,
    /// Whether the core is reached through a borrow.
    by_ref: bool,
}

/// The layers of a type the model has **already read**.
///
/// Takes a `&TypeRef`, not a `&syn::Type`, and that is the whole point: a caller
/// must hold a reading, and the ways to hold one are to take it off an element or
/// to be the scan admitting a type with no element. Re-deriving a reading from
/// `spell()` — the round trip this signature makes impossible — is reasoning
/// from the spelling, which is what `origin` is not for.
fn peel(ty: &prebindgen_flat::flat::TypeRef) -> Layered {
    let (shape, layered) = ty.layer_stack();
    let borrowed = layered.borrow_target();
    Layered {
        shape,
        layer_types: ty.layer_types().into_iter().cloned().collect(),
        core: borrowed.unwrap_or(layered).key(),
        by_ref: borrowed.is_some(),
    }
}

/// Just the borrow: whether `ty` is one, and what it borrows.
///
/// Deliberately **not** [`peel`]: a site that peels only the borrow means it,
/// because the layer underneath is the thing it is about to classify. `Vec<T>`
/// answers `(false, Vec<T>)` here and `(iterable, T)` there, and confusing the two
/// turns "this arg is a collection" into "this arg is a T".
fn peel_borrow(ty: &prebindgen_flat::flat::TypeRef) -> (bool, &prebindgen_flat::flat::TypeRef) {
    match ty.borrow_target() {
        Some(inner) => (true, inner),
        None => (false, ty),
    }
}

/// True when `ret` is `T` / `&T` / `Option<T|&T>` / `Vec<T|&T>` with
/// `T == key` — the default-output match. `Result<_, _>` is NOT peeled, so a
/// fallible factory (`-> Result<T, E>`) keeps its handle return; the error
/// position is matched separately on `E`.
fn returns_type(ret: &prebindgen_flat::flat::TypeRef, key: &TypeKey) -> bool {
    peel(ret).core == *key
}

/// Build one output/error plan for `ed` and store it in the right registry map.
fn process_decl(
    registry: &mut Unfolding<'_>,
    acc: &Deconstructors,
    ed: &OutputDecl,
) -> Result<(), UnfoldError> {
    {
        // The value to decompose: the success return (`Output`) or the
        // `Result<_, E>` domain error `E` (`Error`).
        let fn_ret = registry
            .flat()
            .function(&ed.func)
            .map(|f| f.ret.clone())
            .ok_or_else(|| UnfoldError::UnknownFunction(ed.func.clone()))?;
        let ret_ty = match ed.target {
            DeconTarget::Output => fn_ret,
            DeconTarget::Error => {
                fn_ret
                    .fallible_parts()
                    .map(|(_, e)| e.clone())
                    .ok_or_else(|| UnfoldError::Unsupported {
                        func: ed.func.clone(),
                        reason: "convert_error/deconstruct_error on a non-Result return",
                    })?
            }
        };

        // Peel an outer `Option` off the success return BEFORE probing for a
        // `Vec`, so `Option<Vec<T>>` composes as `Optional(Iterable)` — the
        // fold is skipped and a null result delivered for `None` (issue
        // #105). The scalar arm below reuses this peel. Error targets keep
        // the historical probe order (the `Vec` probe runs on `E` itself), so
        // an `Option<Vec<E>>` error stays whole.
        let (optional, after_opt) = match ed.target {
            DeconTarget::Output => match ret_ty.optional_inner() {
                Some(inner) => (true, inner),
                None => (false, &ret_ty),
            },
            DeconTarget::Error => (false, &ret_ty),
        };
        // `Vec<T>` / `Option<Vec<T>>` return → `Iterable` (± an `Optional`
        // layer). Two element-delivery modes:
        //   * **decomposed** (M5): the element type has an accessor →
        //     flatten it into leaves, fold `(acc, leaf0, …) -> acc`.
        //   * **whole** (M4): no accessor → deliver each element whole
        //     via its own output converter + projection, fold `(acc, T) -> acc`.
        // The other shapes (`Option`/scalar) decompose via an accessor
        // (M1–M3). `Vec<Option<…>>` is not supported.
        let plan = if let Some(inner) = after_opt.sequence_elem() {
            if inner.optional_inner().is_some() {
                return Err(UnfoldError::Unsupported {
                    func: ed.func.clone(),
                    reason: "Vec<Option<…>> returns",
                });
            }
            let iterable = UnfoldShape::Iterable(Box::new(UnfoldShape::Base));
            let shape = if optional {
                UnfoldShape::Optional((), Box::new(iterable))
            } else {
                iterable
            };
            // The fold delivers the return element-by-element, so the
            // whole-collection converter is not needed — and for an
            // opaque-handle element it cannot resolve at all (a `jlong` wire
            // isn't JObject-shaped). De-require the scan-time registrations
            // (the declared return and, under `Option`, the inner `Vec` its
            // recursive registration also required) — same reasoning as
            // [`apply_leaf_vec_folds`] for the fixed folds.
            if ed.target == DeconTarget::Output {
                registry.unrequire_output(&ret_ty);
                if optional {
                    registry.unrequire_output(after_opt);
                }
            }
            // Element type peeled of a leading `&` (accessors take `&Element`).
            let (by_ref, element) = peel_borrow(inner);
            let ekey = element.key();
            if let Some(d) = find_deconstructor_by_type(acc, &ekey) {
                // Decomposed: reuse the shared flatten (M3 nesting composes).
                let records = d.records.clone();
                let decon = decl_id(&ekey, d);
                register_decon_spec(registry, acc, &decon, &records, element)?;
                let plan = build_plan(acc, registry, ed, by_ref, element, shape, &records, decon)?;
                for leaf in &plan.leaves {
                    registry.require_output(&leaf.out_ty);
                }
                plan
            } else {
                // Whole element: keep the type exactly as written so the
                // element's own output converter matches `into_iter()`'s yield.
                // No declaration is involved (`decon: None`) — the element
                // crosses whole through its own converter.
                let by_ref = peel_borrow(inner).0;
                registry.require_output(inner);
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
                    hoists: Vec::new(),
                }
            }
        } else {
            // Scalar/decomposed arm. The `Option` peel already happened above
            // for `Output` (exactly one layer — `Option<Option<…>>` is NOT
            // re-peeled and fails as "no deconstructor" for the inner
            // `Option`); for `Error` it happens here, unchanged.
            let (optional, core_ty) = match ed.target {
                DeconTarget::Output => (optional, after_opt),
                DeconTarget::Error => match after_opt.optional_inner() {
                    Some(inner) => (true, inner),
                    None => (false, after_opt),
                },
            };
            let (by_ref, source) = peel_borrow(core_ty);
            let source_key = source.key();
            let shape = if optional {
                UnfoldShape::Optional((), Box::new(UnfoldShape::Base))
            } else {
                UnfoldShape::Base
            };
            let (records, decon) = resolve_deconstructor(acc, &source_key, ed)?;
            register_decon_spec(registry, acc, &decon, &records, source)?;
            let plan = build_plan(acc, registry, ed, by_ref, source, shape, &records, decon)?;
            for leaf in &plan.leaves {
                registry.require_output(&leaf.out_ty);
            }
            plan
        };
        // Delivery is by **leaf count**, not a per-decl flag:
        //   * Output, single non-nullable leaf, non-Iterable ⇒ Return (wrapper
        //     returns the value via its ordinary output converter —
        //     `convert_out_ty`).
        //   * Output, multiple leaves or Iterable (at any layer — an
        //     `Optional(Iterable)` fold has no single value to return) ⇒
        //     Callback (builder / fold).
        //   * Error ⇒ always Callback-shaped: every leaf is a `ze` arg after the
        //     fixed `je` (no return-value path; `convert_out_ty` stays None).
        //
        // A NULLABLE leaf is one whose path passes through an `Option` that
        // something is decomposed below (`Option<Handle>` reached by
        // `.field_self()`, a nested value form behind an `Option`). Returning it
        // has nowhere to put the absent case: a return value is one expression,
        // so there is no `None` arm, and `convert_out_ty` names the leaf's own
        // type rather than an optional of it. Callback delivery has that arm
        // already — the leaf crosses as a boxed `Long` / JVM null — so the
        // shape goes there instead of being composed into Rust that hands
        // `&Option<T>` to a converter typed for `T`.
        let single_return = ed.target == DeconTarget::Output
            && !plan.shape.has_iterable_layer()
            && plan.leaves.len() == 1
            && !plan.leaves[0].nullable;
        let plan = if single_return {
            // Composed with the model's own layering rather than by spelling
            // `Option<#leaf_ty>` and handing the tokens over: `optional()` pairs
            // the `kind` with its spelling in one place, so the reading that
            // reaches the table is the one this plan carries (#281).
            let cv = if matches!(plan.shape, UnfoldShape::Optional((), _)) {
                plan.leaves[0].out_ty.optional()
            } else {
                plan.leaves[0].out_ty.clone()
            };
            registry.require_output(&cv);
            UnfoldPlan {
                delivery: Delivery::Return,
                convert_out_ty: Some(cv.clone()),
                ..plan
            }
        } else {
            UnfoldPlan {
                delivery: Delivery::Callback,
                ..plan
            }
        };
        match ed.target {
            DeconTarget::Output => registry.plans.unfold_plans.insert(ed.func.clone(), plan),
            DeconTarget::Error => registry.plans.error_plans.insert(ed.func.clone(), plan),
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
fn register_decon_spec(
    registry: &mut Unfolding<'_>,
    acc: &Deconstructors,
    decon: &DeconId,
    records: &[DeconRecord],
    source: &prebindgen_flat::flat::TypeRef,
) -> Result<(), UnfoldError> {
    if registry.plans.decon_plans.contains_key(decon) {
        return Ok(());
    }
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    let mut visited: HashSet<TypeKey> = HashSet::new();
    visited.insert(source.key());
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
        // A `DeconSpec` describes the leaf list only — signature artifacts are
        // derived from it, never emitted code — so its hoists are discarded.
        &mut Vec::new(),
    )?;
    require_unique_leaf_names(source, &leaves)?;
    registry.plans.decon_plans.insert(
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
    acc.deconstructors.iter().find(|c| c.target == *type_key)
}

/// Build the [`UnfoldPlan`] for a chosen accessor. `shape` is the outer
/// shape over the core decomposition (`Decompose` for `T`/`&T`,
/// `Optional(Decompose)` for `Option<T>`/`Option<&T>`). The records are
/// recursively flattened ([`flatten`]) — nested accessors contribute
/// their leaves with the access path prefixed.
#[allow(clippy::too_many_arguments)]
fn build_plan(
    acc: &Deconstructors,
    registry: &Unfolding<'_>,
    ed: &OutputDecl,
    by_ref: bool,
    source: &prebindgen_flat::flat::TypeRef,
    shape: UnfoldShape,
    records: &[DeconRecord],
    decon: DeconId,
) -> Result<UnfoldPlan, UnfoldError> {
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    let mut visited: HashSet<TypeKey> = HashSet::new();
    visited.insert(source.key());
    let mut hoists: Vec<Hoist> = Vec::new();
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
        &mut hoists,
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
        hoists,
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
    source: &prebindgen_flat::flat::TypeRef,
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
                target: source.key().to_string(),
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
fn flatten(
    acc: &Deconstructors,
    registry: &Unfolding<'_>,
    records: &[DeconRecord],
    source: &prebindgen_flat::flat::TypeRef,
    path_prefix: &[PathStep],
    name_prefix: &[String],
    by_ref: bool,
    nullable: bool,
    visited: &mut HashSet<TypeKey>,
    leaves: &mut Vec<UnfoldLeaf>,
    hoists: &mut Vec<Hoist>,
) -> Result<(), UnfoldError> {
    let source_key = source.key();
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
                // Owned where the value is OURS to give: the root of an owned
                // plan (a `Copy` blob copies / an opaque handle moves), or a
                // field of a value form that CONSUMED its value — that form was
                // handed the value, so its fields move out like every other
                // field of it. Borrowed (clone) otherwise. The adapter-side type
                // + projection come from this `out_ty`'s output converter, so
                // this is what decides whether the leaf is boxed by move or
                // cloned through the borrowed-opaque one.
                // A plan field: the drop to spelling happens here, where the value
                // is stored for emission, and the borrowed form is composed rather
                // than looked up because no source wrote it.
                // The borrowed form is COMPOSED — no source wrote it — and the
                // composition pairs the kind with its own spelling, so nothing
                // downstream has to look either up.
                let out_ty = if place_is_owned(hoists, path_prefix, by_ref) {
                    source.clone()
                } else {
                    source.borrowed()
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
                    source: LeafSource::Reach,
                    groups: Vec::new(),
                });
            }
            DeconRecord::Fields {
                func,
                consuming,
                fields,
            } => {
                let consuming = *consuming;
                // The value form is called once; every field hangs off that one
                // call, so the whole record shares a single `Call` step and the
                // emitter can hoist it.
                accessor_signature(registry, func, &source.key())?;
                // The declarator states whether the value is given away; the
                // signature has to agree, or the emitted call would not compile
                // in the consumer's crate. Checked rather than inferred so that
                // declaring `.fields_self_into(..)` on a borrowing accessor is a
                // named error instead of a silently downgraded boundary.
                if consuming != accessor_consumes(registry, func) {
                    return Err(UnfoldError::Unsupported {
                        func: func.clone(),
                        reason: if consuming {
                            "declared as a CONSUMING value form (`.fields_self_into(..)`) but the \
                             accessor borrows its receiver — declare it with `.fields(..)`, or \
                             name the by-value accessor"
                        } else {
                            "declared as a BORROWING value form (`.fields(..)`) but the accessor \
                             takes its receiver by value — declare it with `.fields_self_into(..)`, or \
                             name the `&Self` accessor"
                        },
                    });
                }
                let mut root_path = path_prefix.to_vec();
                root_path.push(PathStep::call(func.clone(), false, false));
                // A hoist below an optional step is CONDITIONAL: it binds an
                // `Option<TStruct>` local (built only in the `Some` arm) and
                // every leaf under it is null when the value is absent — which
                // is the nullability `flatten` already propagates down here.
                // What it cannot do is nest: composing a second hoist off a
                // conditional one would have to reach through the outer
                // `Option`, and the binder has no arm to put that in. One level
                // is the shape real bindings need (`Option<&Sample>` delivering
                // a sample's value form), so implement that and name the rest.
                //
                // A top-level `Option<T>` is represented by
                // `UnfoldShape::Optional`, not by a path step, and is unaffected.
                if root_path.iter().any(PathStep::is_optional)
                    && hoists.iter().any(|h| {
                        h.prefix.len() < root_path.len() && root_path.starts_with(&h.prefix)
                    })
                {
                    return Err(UnfoldError::Unsupported {
                        func: func.clone(),
                        reason: "a value form nested under another one that is reached through \
                                 `Option` — conditional hoists do not nest",
                    });
                }
                // A consuming value form DESTROYS the value into its parts, so
                // a sibling record — `.field_self()` or another `.field()` —
                // would read what it just gave away. jnigen refuses this in the
                // declarator, where the author can see it; this is the backstop
                // for records built directly against core.
                //
                // Being reached through ANOTHER value form is fine: a hoisted
                // value form is an owned struct and its fields are disjoint, so
                // the parent's field is handed over by move.
                if consuming && records.len() > 1 {
                    return Err(UnfoldError::Unsupported {
                        func: func.clone(),
                        reason: "a consuming value form must be the only record of its \
                                 declaration — it moves the value, so `.field_self()` or \
                                 a sibling `.field()` would read a moved value",
                    });
                }
                // Evaluate this value form ONCE. Recorded at the prefix it sits
                // at rather than as a lone accessor, so a nested value form
                // (this record reached through another one's field) gets its own
                // hoist instead of being rebuilt per child leaf. `path_prefix`
                // grows as `flatten` descends, so the list comes out
                // outermost-first.
                hoists.push(Hoist {
                    prefix: root_path.clone(),
                    consuming,
                });

                for fr in fields {
                    // The declaration carries the field's reading, so nothing is
                    // looked up and nothing is re-classified.
                    //
                    // A field's own `Option` makes everything under it nullable,
                    // exactly as an `Option`-returning accessor step does.
                    let (opt, core) = match fr.ty.optional_inner() {
                        Some(inner) => (true, inner),
                        None => (false, &fr.ty),
                    };
                    let child_ty = core.borrow_target().unwrap_or(core);
                    let child_key = child_ty.key();

                    // Same three-way choice a `.field()` record makes: declared
                    // override, else the field type's own deconstructor, else
                    // one leaf — with the adapter able to pre-build the leaves
                    // for a shape only it can describe.
                    let child_records = match &fr.decon {
                        FieldDecon::Records(recs) => Some(recs.clone()),
                        FieldDecon::Leaves(_) => None,
                        FieldDecon::Default => match find_deconstructor_by_type(acc, &child_key) {
                            Some(child_decl) if !visited.contains(&child_key) => {
                                Some(child_decl.records.clone())
                            }
                            Some(_) => {
                                return Err(UnfoldError::Cycle {
                                    target: child_key.to_string(),
                                });
                            }
                            None => None,
                        },
                    };
                    let decomposed =
                        child_records.is_some() || matches!(fr.decon, FieldDecon::Leaves(_));

                    // The field's own `Option` is a nullable NESTING step only
                    // when something is decomposed below it. For a plain leaf
                    // the whole `Option<F>` is what the converter takes — the
                    // same rule that makes a terminal accessor's `Option` ride
                    // its converter instead of being unwrapped.
                    let mut field_path = root_path.clone();
                    let (last, lead) = fr
                        .members
                        .split_last()
                        .expect("a field record addresses at least one member");
                    // Only the LAST member can be optional — an inlined nested
                    // class is reached directly, never through an `Option`.
                    field_path.extend(lead.iter().map(|m| PathStep::field(m.clone(), false)));
                    field_path.push(PathStep::field(last.clone(), opt && decomposed));

                    // Adapter-built leaves: rebase each onto this field's path
                    // and name. Their internal structure (a selector plus its
                    // groups) is opaque here and passes through untouched.
                    if let FieldDecon::Leaves(built) = &fr.decon {
                        for l in built {
                            let mut path = field_path.clone();
                            path.extend(l.path.iter().cloned());
                            let mut name = seg_name(&fr.name);
                            name.push(l.name.clone());
                            leaves.push(UnfoldLeaf {
                                name: name.join("__"),
                                path,
                                nullable: l.nullable || nullable || opt,
                                ..l.clone()
                            });
                        }
                        continue;
                    }

                    if let Some(child_records) = child_records {
                        visited.insert(child_key.clone());
                        flatten(
                            acc,
                            registry,
                            &child_records,
                            // The declaration's reading, peeled — not a new one.
                            child_ty,
                            &field_path,
                            &seg_name(&fr.name),
                            by_ref,
                            nullable || opt,
                            visited,
                            leaves,
                            hoists,
                        )?;
                        visited.remove(&child_key);
                    } else {
                        // A plain field leaf: the value is CLONED out of the
                        // struct, so its converter takes the owned field type as
                        // written — `Option` and all, which is why a terminal
                        // `Option` step is not a nesting step for it.
                        leaves.push(UnfoldLeaf {
                            name: seg_name(&fr.name).join("__"),
                            path: field_path,
                            out_ty: fr.ty.clone(),
                            identity: false,
                            nullable,
                            source: LeafSource::Reach,
                            groups: Vec::new(),
                        });
                    }
                }
            }
            DeconRecord::Acc { name, .. } | DeconRecord::LocalAcc { name, .. } => {
                // A binding-local record resolves through its synthesized
                // registry entry (see `synthesize_local_accessors`), so both
                // kinds read one signature source; only the cycle rule below
                // differs.
                let (func, local) = match rec {
                    DeconRecord::Acc { func, .. } => (func.clone(), false),
                    DeconRecord::LocalAcc { path, .. } => (DeconRecord::local_ident(path), true),
                    DeconRecord::Identity | DeconRecord::Fields { .. } => unreachable!(),
                };
                let ret = accessor_signature(registry, &func, &source.key())?;
                // Default unwrap: if the return type has its own deconstructor,
                // splice it (recurse); otherwise the return is one leaf. Peel an
                // `Option` (value may be absent) + leading `&` to reach the child.
                // This site peels an `Option` only — an accessor returning a run
                // of values is not spliced — so it asks the model for that one
                // layer rather than the whole stack.
                let after_opt = ret.optional_inner();
                let opt = after_opt.is_some();
                let core = after_opt.unwrap_or(&ret);
                let (core_by_ref, child_ty) = peel_borrow(core);
                let child_key = child_ty.key();
                // A child already on the nesting chain: for a `#[prebindgen]`
                // accessor that is an authoring cycle (hard error); a
                // binding-local field re-delivering (part of) its own type
                // under a binding-defined condition is the POINT — degrade to
                // a plain converter leaf instead of splicing.
                let splice = match find_deconstructor_by_type(acc, &child_key) {
                    Some(child_decl) if !visited.contains(&child_key) => Some(child_decl),
                    Some(_) if local => None,
                    Some(_) => {
                        return Err(UnfoldError::Cycle {
                            target: child_key.to_string(),
                        });
                    }
                    None => None,
                };
                if let Some(child_decl) = splice {
                    visited.insert(child_key.clone());
                    let child_records = child_decl.records.clone();
                    let mut child_path = path_prefix.to_vec();
                    child_path.push(PathStep::call(func.clone(), opt, !core_by_ref));
                    flatten(
                        acc,
                        registry,
                        &child_records,
                        child_ty,
                        &child_path,
                        &seg_name(name),
                        by_ref,
                        nullable || opt,
                        visited,
                        leaves,
                        hoists,
                    )?;
                    visited.remove(&child_key);
                } else {
                    // Leaf: the return value as written (`&str`, enum, `i64`, …).
                    // One exception: a binding-local field returning an
                    // OPTIONAL BORROW (`Option<&T>`) is the conditional
                    // HANDLE-delivery idiom — structurally a spliced identity
                    // behind an `Option`-returning step (cf. an
                    // `Option`-returning nesting accessor + the child's
                    // `field_self`), so it contributes a nullable IDENTITY
                    // leaf of the borrowed type: the reach path unwraps the
                    // final `Option` (the synthesized signature keeps the
                    // full return) and the value clones through its handle
                    // projection, `None` delivering null. It shares the
                    // one-identity-per-deconstructor budget with
                    // `.field_self()` — two handle deliveries of one value
                    // make no sense.
                    let cond_handle = local && opt && core_by_ref;
                    if cond_handle {
                        if seen_identity {
                            return Err(UnfoldError::MultipleIdentity {
                                target: source_key.to_string(),
                            });
                        }
                        seen_identity = true;
                    }
                    // A plan field: the spelling is taken here, once, where the
                    // leaf is stored for emission.
                    let (out_ty, nullable, identity) = if cond_handle {
                        (core.clone(), true, true)
                    } else {
                        (ret.clone(), nullable, false)
                    };
                    let mut path = path_prefix.to_vec();
                    path.push(PathStep::call(func.clone(), opt, !core_by_ref));
                    leaves.push(UnfoldLeaf {
                        name: seg_name(name).join("__"),
                        path,
                        out_ty,
                        identity,
                        nullable,
                        source: LeafSource::Reach,
                        groups: Vec::new(),
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
fn require_unique_leaf_names(
    source: &prebindgen_flat::flat::TypeRef,
    leaves: &[UnfoldLeaf],
) -> Result<(), UnfoldError> {
    let mut seen: HashSet<&str> = HashSet::new();
    for l in leaves {
        if !seen.insert(l.name.as_str()) {
            return Err(UnfoldError::DuplicateLeafName {
                target: source.key().to_string(),
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

/// An accessor `f(&T) -> R`: returns its return type `R` as written (a
/// reference where possible).
///
/// `expected` is the type the deconstructor decomposes, and what comes back is
/// an accessor already proven to take it. Taking it as a parameter rather than
/// leaving the caller to check afterwards is the point: a declarator cannot
/// reach an accessor's signature without saying what that accessor is supposed
/// to be about, so the check cannot be the thing a new declarator forgets
/// (#223). The comparison is [`check_declared_target`], shared with the input
/// side's constructor lookup.
fn accessor_signature(
    registry: &Unfolding<'_>,
    func: &syn::Ident,
    expected: &TypeKey,
) -> Result<prebindgen_flat::flat::TypeRef, UnfoldError> {
    let f = registry
        .flat()
        .function(&func)
        .ok_or_else(|| UnfoldError::UnknownAccessor(func.clone()))?;

    // First parameter is the receiver `&T`; peel the borrow to get `T`.
    // `borrow_target` is the model's own answer, so the peel reads a
    // classification instead of re-deciding it from `syn::Type::Reference`.
    let first = f
        .params
        .first()
        .ok_or_else(|| UnfoldError::UnknownAccessor(func.clone()))?;
    // The receiver's identity, keyed so the comparison below cannot fail on a
    // spelling difference that does not change which type this is about.
    let takes = match first.ty.borrow_target() {
        Some(inner) => inner.key(),
        None => first.ty.key(),
    };
    check_declared_target(func, &takes, expected)?;
    Ok(f.ret.clone())
}

/// Whether the value sitting at `path_prefix` is the plan's **to give away**:
/// the root of an owned plan, or a field of a value form that consumed its
/// value and is reached by a movable run of field steps.
///
/// Consulted where a leaf's `out_ty` is chosen, so the ownership decision is
/// made ONCE, in the plan, rather than re-derived by each emitter — a leaf
/// whose `out_ty` is the owned type is boxed by move, one whose `out_ty` is a
/// borrow is cloned through the borrowed-opaque converter.
fn place_is_owned(hoists: &[Hoist], path_prefix: &[PathStep], by_ref: bool) -> bool {
    if path_prefix.is_empty() {
        return !by_ref;
    }
    hoists
        .iter()
        .filter(|h| h.prefix.len() <= path_prefix.len() && path_prefix.starts_with(&h.prefix))
        .max_by_key(|h| h.prefix.len())
        .is_some_and(|h| h.consuming && steps_are_movable(&path_prefix[h.prefix.len()..]))
}

/// Whether an accessor takes its receiver **by value** — a *consuming* value
/// form, which destroys the object into its parts instead of cloning them out
/// of a borrow.
///
/// Asked separately because [`accessor_signature`] peels the `&` in order to
/// compare target types, so `f(v: T)` and `f(v: &T)` are indistinguishable
/// there by design.
fn accessor_consumes(registry: &Unfolding<'_>, func: &syn::Ident) -> bool {
    registry
        .flat()
        .function(&func)
        .and_then(|f| f.params.first())
        .is_some_and(|p| p.ty.borrow_target().is_none())
}

/// The shared mismatch, in this direction's vocabulary: an output accessor is
/// declared to **take** the type the deconstructor decomposes.
impl From<crate::declared_target::TargetMismatch> for UnfoldError {
    fn from(m: crate::declared_target::TargetMismatch) -> Self {
        UnfoldError::AccessorTargetMismatch {
            accessor: m.func,
            takes: m.actual,
            expected: m.expected,
        }
    }
}

#[cfg(test)]
mod tests;
