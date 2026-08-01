//! Derive the crossing set: walk what was declared, and register every type
//! position it reaches.
//!
//! Deliberately over-approximating — every nested position, every declared
//! struct in both directions. What must actually convert is reachability from
//! the roots, which `order` decides once the graph is complete.

use std::collections::{HashMap, HashSet};

use quote::ToTokens;

use super::*;

impl<M> Registry<M> {
    pub(super) fn scan_declared_items(&mut self, declared: &Declared) -> Result<(), ScanError> {
        // Source-qualified declared types are a hard error (issue #95). The
        // key's own normalization already reduced `crate::`/`self::` and std
        // prelude spellings, so a remaining multi-segment declared path
        // either qualifies a SOURCE item with its crate name (can never
        // match — the flat namespace keys are bare) or names a genuinely
        // foreign type (supported verbatim; warned about below only when it
        // shadows a captured item's name — the likely-mistake heuristic).
        //
        // The two syntax matches below **stay** as this file's boundary-ledger
        // entries, and the reason is what they look at: `declared.types` are keys a
        // *build script author* wrote, and this is a diagnostic about the spelling
        // they wrote — is it path-qualified, and does its tail shadow a captured
        // item? No source type is being classified, so there is no element to read
        // instead; asking the model would answer about a type rather than about the
        // declaration. This is the "legitimately the adapter's business" case the
        // integration map (L2, #229) predicts, not a migration still owed.
        let mut qualified: Vec<(String, String)> = Vec::new();
        let mut probed: HashSet<&TypeKey> = HashSet::new();
        for key in declared
            .types
            .iter()
            .chain(declared.decompositions.replaces.iter())
        {
            if !probed.insert(key) {
                continue;
            }
            let ty = key.to_type();
            // Peel one reference level; the qualified head only appears on
            // path types.
            let inner = match &ty {
                syn::Type::Reference(r) => &*r.elem,
                other => other,
            };
            let syn::Type::Path(tp) = inner else { continue };
            if tp.qself.is_some() || tp.path.segments.len() < 2 {
                continue;
            }
            let head = tp
                .path
                .segments
                .first()
                .expect("len checked")
                .ident
                .to_string();
            let last = tp.path.segments.last().expect("len checked");
            if self.flat.source_modules().contains(&head) {
                qualified.push((key.to_string(), last.to_token_stream().to_string()));
            } else if self.declares_type(&last.ident) {
                println!(
                    "cargo:warning=prebindgen: declared type `{}` is path-qualified, but a \
                     captured #[prebindgen] item `{}` exists — if you meant the source item, \
                     declare it by its bare name",
                    key, last.ident
                );
            }
        }
        if !qualified.is_empty() {
            qualified.sort();
            return Err(ScanError::QualifiedDeclaredTypes { entries: qualified });
        }

        // Declared-but-missing items are collected across all three loops and
        // reported together as one hard error (see
        // [`ScanError::DeclaredNotFound`]).
        let mut missing: Vec<(&'static str, String)> = Vec::new();

        // Scan declared functions.
        for ident in &declared.functions {
            if let Some(item_fn) = self.flat.function(&ident).map(|f| f.origin.syntax.clone()) {
                self.scan_fn_signature(&item_fn)?;
            } else {
                missing.push(("function", ident.to_string()));
            }
        }

        // Helper functions: never emitted, no blanket signature scan (the
        // adapter registers the specific requirements via
        // `extra_required_types`) — but they are referenced by name from
        // adapter declarations, so a missing one is a hard error.
        for ident in &declared.helper_functions {
            if self.flat.function(&ident).is_none() {
                missing.push(("helper function", ident.to_string()));
            }
        }

        // Scan declared consts: a const is a nullary source of its type, so
        // the type is required in the output direction only.
        for ident in declared.consts.iter().flatten() {
            if let Some(item_const) = self.flat.constant(&ident).map(|c| c.origin.syntax.clone()) {
                self.ensure_entry(Direction::Output, &item_const.ty, true)?;
            } else {
                missing.push(("constant", ident.to_string()));
            }
        }

        if !missing.is_empty() {
            missing.sort();
            return Err(ScanError::DeclaredNotFound { entries: missing });
        }

        // Declared crossings with no element behind them (a foreign class type,
        // a synthesized constant's value type), each in its own direction.
        for (dir, ty) in &declared.crossings {
            self.ensure_entry(*dir, ty, true)?;
        }

        // Scan declared types.
        for key in &declared.types {
            let ty = key.to_type();
            let mut matched = false;
            if let Some(ident) = bare_path_ident(&ty) {
                if let Some(s) = self
                    .flat
                    .struct_type(&ident)
                    .map(|s| s.origin.syntax.clone())
                {
                    self.scan_struct(&s)?;
                    self.ensure_entry(Direction::Input, &ty, true)?;
                    self.ensure_entry(Direction::Output, &ty, true)?;
                    matched = true;
                } else if let Some(e) = self.flat.enum_item(&ident).cloned() {
                    self.scan_enum(&e)?;
                    self.ensure_entry(Direction::Input, &ty, true)?;
                    self.ensure_entry(Direction::Output, &ty, true)?;
                    matched = true;
                }
            }
            if !matched {
                // Declared type without an indexed body (e.g.
                // `ptr_class(ZKeyExpr<'static>)` on a re-exported
                // foreign type). Still mark required so the resolver
                // tries to produce a converter for it.
                self.ensure_entry(Direction::Input, &ty, true)?;
                self.ensure_entry(Direction::Output, &ty, true)?;
            }
        }

        Ok(())
    }

    pub(super) fn scan_fn_signature(&mut self, f: &syn::ItemFn) -> Result<(), ScanError> {
        // Mechanical: register every fn-signature type as the user wrote it.
        // No semantic transformations (no &T→T strip, no ZResult<T>→T strip,
        // no skip for () / ZResult<()>). The adapter handles structural
        // wrappers; propagation through `subs` then marks transitive deps
        // (e.g. &Foo's `&_` converter returns subs=[Foo], so Foo becomes
        // required).
        // No receiver or non-ident pattern can reach here: a captured item was
        // refused by the frontend and `from_flat` failed before indexing it, and
        // a binding-local fn was checked against the same grammar
        // (`Flat::lower_signature`) when `resolve` synthesized it.
        for input in &f.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => continue,
                syn::FnArg::Typed(pt) => {
                    self.register_type_recursive(Direction::Input, &pt.ty, true)?;
                }
            }
        }
        let ret_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        self.register_type_recursive(Direction::Output, &ret_ty, true)?;
        Ok(())
    }

    pub(super) fn scan_struct(&mut self, s: &syn::ItemStruct) -> Result<(), ScanError> {
        // The struct itself can appear in either direction.
        let ty: syn::Type = crate::api::core::flat::type_from_ident(&s.ident);
        self.ensure_entry(Direction::Input, &ty, false)?;
        self.ensure_entry(Direction::Output, &ty, false)?;

        if let syn::Fields::Named(named) = &s.fields {
            for field in &named.named {
                self.register_type_recursive(Direction::Input, &field.ty, false)?;
                self.register_type_recursive(Direction::Output, &field.ty, false)?;
            }
        }
        Ok(())
    }

    pub(super) fn scan_enum(&mut self, e: &syn::ItemEnum) -> Result<(), ScanError> {
        let ty: syn::Type = crate::api::core::flat::type_from_ident(&e.ident);
        self.ensure_entry(Direction::Input, &ty, false)?;
        self.ensure_entry(Direction::Output, &ty, false)?;

        for variant in &e.variants {
            for field in &variant.fields {
                self.register_type_recursive(Direction::Input, &field.ty, false)?;
                self.register_type_recursive(Direction::Output, &field.ty, false)?;
            }
        }
        Ok(())
    }

    /// Register `ty` as a cell in the given direction, then recurse into every
    /// nested position. `root` applies only to `ty` itself — a nested position is
    /// never something the binding asked for directly.
    pub(super) fn register_type_recursive(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
    ) -> Result<(), ScanError> {
        let mut visited: HashSet<TypeKey> = HashSet::new();
        self.register_type_inner(dir, ty, root, &mut visited)
    }

    pub(super) fn register_type_inner(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        is_top: bool,
        visited: &mut HashSet<TypeKey>,
    ) -> Result<(), ScanError> {
        // A disallowed `impl Trait` cannot reach here: every fn whose signature
        // reaches this point passed the frontend's grammar — captured items at
        // ingestion, binding-local ones at synthesis — and it names the
        // parameter the bad type sits on.

        let key = TypeKey::from_type(ty);
        if !visited.insert(key.clone()) {
            return Ok(()); // cycle guard
        }

        self.ensure_entry(dir, ty, is_top)?;

        for (child_dir, sub) in self.immediate_edges(dir, ty) {
            self.register_type_inner(child_dir, &sub, false, visited)?;
        }
        Ok(())
    }

    /// Create the cell for `ty` in `dir` if it has none, and mark it a root when
    /// the binding asked for it directly.
    ///
    /// The one place a cell is born, and therefore the one place a type **enters
    /// the pipeline** — so it is where a type the source never wrote is admitted to
    /// the model. Expansion composes such spellings (an `Option<T>` around a `T` it
    /// found) and hands them straight here via `require_input` / `require_output`.
    ///
    /// Admitting rather than classifying on the fly is the rule
    /// [`Flat::add_local_function`](crate::api::core::flat::Flat::add_local_function)
    /// already set for a binding-local `sig!(..)`: lower through the one grammar,
    /// then record it, so the model keeps owning the only index of what a type
    /// means. Every later lookup — this scan, the resolver, an adapter — then gets
    /// the same answer from the same place.
    ///
    /// A spelling the grammar refuses is reported by name, rather than becoming a
    /// cell that quietly means less than its neighbours. Only an *entry point* can
    /// reach that: a type the walk found came from an existing reading's
    /// `origin.syntax`, so it lowered once already.
    pub(super) fn ensure_entry(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
    ) -> Result<(), ScanError> {
        let key = TypeKey::from_type(ty);
        let reading = self
            .flat
            .admit_type(ty)
            .map_err(|source| ScanError::NotExpressible {
                entries: vec![NotExpressibleEntry {
                    name: None,
                    reason: source.to_string(),
                    location: SourceLocation::default(),
                }],
            })?;
        let subject = Box::new(reading.clone());
        let cell = self
            .type_table_mut(dir)
            .entry(key)
            .or_insert_with(|| TypeCell {
                subject,
                root: false,
                entry: None,
            });
        cell.root |= root;
        Ok(())
    }

    /// Enumerate the immediate type-graph edges out of `(dir, ty)`: the model's
    /// own children of this type, plus — if `ty` names a declared struct or sum —
    /// the field types of that item.
    ///
    /// A callback's argument types flow with `dir.flip()`, because an argument the
    /// binding *hands to* a callback crosses the other way; everything else
    /// inherits `dir`. Used by both `register_type_inner` (during scan) and the
    /// unresolved-descendants BFS in `resolve` (for diagnostics).
    ///
    /// The children come from [`TypeKind`], not from taking the syntax apart, and
    /// the difference is load-bearing rather than cosmetic. `&mut MaybeUninit<T>`
    /// is `Ref { mode: Out, inner: T }` — the model absorbed the `MaybeUninit`, so
    /// the edge lands on `T` directly instead of on an intermediate
    /// `MaybeUninit<T>` that no source ever wrote and no adapter can convert.
    /// Each edge is still *spelled* from the child's own `origin.syntax`, which is
    /// what the caller keys the table by.
    ///
    /// A plain index read: `ensure_entry` admitted this type to the model before
    /// the walk reached it, so the reading is already there — including for a
    /// spelling the binding composed. No reading means the grammar refused the
    /// type, and a refused type has no structure to walk.
    pub(crate) fn immediate_edges(
        &self,
        dir: Direction,
        ty: &syn::Type,
    ) -> Vec<(Direction, syn::Type)> {
        use crate::api::core::flat::TypeKind;

        let mut out: Vec<(Direction, syn::Type)> = Vec::new();
        if let Some(reading) = self.flat.type_ref(ty) {
            let (children, child_dir): (Vec<&crate::api::core::flat::TypeRef>, Direction) =
                match &reading.kind {
                    TypeKind::Optional(t)
                    | TypeKind::Sequence(t)
                    | TypeKind::Ref { inner: t, .. } => (vec![t], dir),
                    TypeKind::Array { elem, .. } => (vec![elem], dir),
                    TypeKind::Fallible { ok, err } => (vec![ok, err], dir),
                    TypeKind::Callback { args } => (args.iter().collect(), dir.flip()),
                    // A name is a leaf in the type graph: its generic arguments are
                    // lowered but not retained, because no declaration takes type
                    // parameters. Its *fields* are the edges, and they come off the
                    // element below.
                    TypeKind::Named { .. }
                    | TypeKind::Scalar(_)
                    | TypeKind::Str
                    | TypeKind::Unit => (Vec::new(), dir),
                };
            for child in children {
                out.push((child_dir, child.origin.syntax.clone()));
            }
        }
        // A declared type's own fields, read off the element rather than off its
        // `syn::Fields`: a positional field is an ordinary `Field` there, so the
        // named-only asymmetry the syntax walk had does not arise. An `Enum` has
        // no fields and an `Extern` declares none, which is what makes both
        // contribute nothing here.
        //
        // The **name comes from the classification**, not from taking the spelling
        // apart, and that is what makes a transparent wrapper work: `Box<Node>` is
        // `Named { id: Node }` — `Box<T>` **is** `T` in this language — so it
        // reaches `Node`'s fields, where asking the syntax for a bare ident would
        // have answered `None` and dead-ended the walk.
        if let Some(name) = self.flat.type_ref(ty).and_then(|r| match &r.kind {
            TypeKind::Named { id } => Some(id.name.clone()),
            _ => None,
        }) {
            use crate::api::core::flat::{Field, Type};
            let fields: Vec<&Field> = match self.flat.declared_type(name.as_str()) {
                Some(Type::Struct(s)) => s.fields.iter().collect(),
                Some(Type::Variant(v)) => v
                    .alternatives
                    .iter()
                    .flat_map(|a| a.fields.iter())
                    .collect(),
                Some(Type::Enum(_) | Type::Extern(_)) | None => Vec::new(),
            };
            for field in fields {
                out.push((dir, field.ty.origin.syntax.clone()));
            }
        }
        out
    }

    /// Register `ty` (and its nested positions) as a required **input** so
    /// the resolver produces a converter for it. Used by
    /// [`crate::api::core::expand`] to pull in the leaf types a fold needs.
    pub(crate) fn require_input(&mut self, ty: &syn::Type) {
        // Leaf/expansion types are concrete (no disallowed `impl Trait`), so
        // the recursive registration cannot fail here.
        let _ = self.register_type_recursive(Direction::Input, ty, true);
    }

    /// Register `ty` (and its nested positions) as a required **output** so the
    /// resolver produces a converter for it. The output-side peer of
    /// [`Self::require_input`]; used by [`crate::api::core::unfold`] to pull in
    /// the leaf types a decomposition delivers.
    pub(crate) fn require_output(&mut self, ty: &syn::Type) {
        let _ = self.register_type_recursive(Direction::Output, ty, true);
    }

    /// Drop `ty` from the required-output scan set. The type's table entry is
    /// left intact (so [`crate::api::core::resolve`]'s PASS A still resolves it
    /// if it can, and emits it when resolved), but a `None` resolution no longer
    /// counts as an unresolved-required error. Used by
    /// [`crate::api::core::unfold::apply_leaf_vec_folds`]: when a `Vec<T>` /
    /// `Option<Vec<T>>` return is delivered element-by-element through a fold,
    /// the whole-collection converter is genuinely not needed — and for a
    /// `Vec<opaque-handle>` it cannot resolve at all (a `jlong` wire is not
    /// JObject-shaped), so requiring it would wrongly fail resolution.
    pub(crate) fn unrequire_output(&mut self, ty: &syn::Type) {
        self.clear_root(Direction::Output, ty);
    }

    /// Drop `ty` from the required-input scan set — the input-side peer of
    /// [`Self::unrequire_output`]. Used by [`Self::apply_adapter_plans`] for
    /// the adapter's boundary-only types: a fold plan replaces every direct
    /// crossing of the type with its ingredients, so the type's own input
    /// converter is genuinely not needed (and for an undeclared type cannot
    /// resolve at all).
    pub(crate) fn unrequire_input(&mut self, ty: &syn::Type) {
        self.clear_root(Direction::Input, ty);
    }

    /// Stop treating `ty` as a root. The cell stays, so the resolver still fills
    /// it if it can — only the demand that it *must* resolve is dropped.
    pub(super) fn clear_root(&mut self, dir: Direction, ty: &syn::Type) {
        let key = TypeKey::from_type(ty);
        if let Some(cell) = self.type_table_mut(dir).get_mut(&key) {
            cell.root = false;
        }
    }

    /// Direction-indexed read access to the type-resolution tables.
    pub(crate) fn type_table(&self, dir: Direction) -> &HashMap<TypeKey, TypeCell<M>> {
        match dir {
            Direction::Input => &self.input_types,
            Direction::Output => &self.output_types,
        }
    }

    /// Direction-indexed mutable access to the type-resolution tables.
    pub(crate) fn type_table_mut(&mut self, dir: Direction) -> &mut HashMap<TypeKey, TypeCell<M>> {
        match dir {
            Direction::Input => &mut self.input_types,
            Direction::Output => &mut self.output_types,
        }
    }

    /// Look up the resolved input entry for `ty`, returning `None` if it
    /// was never registered or is still unresolved. The returned entry's
    /// `function.sig.ident` is the converter's call name; `destination` is
    /// its wire form.
    pub fn input_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        let key = TypeKey::from_type(ty);
        self.type_table(Direction::Input).get(&key)?.entry.as_ref()
    }

    /// Look up the resolved output entry for `ty`. See [`Self::input_entry`].
    pub fn output_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        let key = TypeKey::from_type(ty);
        self.type_table(Direction::Output).get(&key)?.entry.as_ref()
    }
}
