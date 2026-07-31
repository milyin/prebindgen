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
                self.ensure_entry(Direction::Output, &item_const.ty, true);
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
            self.ensure_entry(*dir, ty, true);
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
                    self.ensure_entry(Direction::Input, &ty, true);
                    self.ensure_entry(Direction::Output, &ty, true);
                    matched = true;
                } else if let Some(e) = self.flat.enum_item(&ident).cloned() {
                    self.scan_enum(&e)?;
                    self.ensure_entry(Direction::Input, &ty, true);
                    self.ensure_entry(Direction::Output, &ty, true);
                    matched = true;
                }
            }
            if !matched {
                // Declared type without an indexed body (e.g.
                // `ptr_class(ZKeyExpr<'static>)` on a re-exported
                // foreign type). Still mark required so the resolver
                // tries to produce a converter for it.
                self.ensure_entry(Direction::Input, &ty, true);
                self.ensure_entry(Direction::Output, &ty, true);
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
        let ty: syn::Type = crate::api::core::types_util::type_from_ident(&s.ident);
        self.ensure_entry(Direction::Input, &ty, false);
        self.ensure_entry(Direction::Output, &ty, false);

        if let syn::Fields::Named(named) = &s.fields {
            for field in &named.named {
                self.register_type_recursive(Direction::Input, &field.ty, false)?;
                self.register_type_recursive(Direction::Output, &field.ty, false)?;
            }
        }
        Ok(())
    }

    pub(super) fn scan_enum(&mut self, e: &syn::ItemEnum) -> Result<(), ScanError> {
        let ty: syn::Type = crate::api::core::types_util::type_from_ident(&e.ident);
        self.ensure_entry(Direction::Input, &ty, false);
        self.ensure_entry(Direction::Output, &ty, false);

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

        self.ensure_entry(dir, ty, is_top);

        for (child_dir, sub) in self.immediate_edges(dir, ty) {
            self.register_type_inner(child_dir, &sub, false, visited)?;
        }
        Ok(())
    }

    /// Create the cell for `ty` in `dir` if it has none, and mark it a root when
    /// the binding asked for it directly.
    ///
    /// The one place a cell is born, which is what lets the subject be decided
    /// once: the model's reading if the flat API mentions this type, an
    /// adapter-authored type otherwise.
    pub(super) fn ensure_entry(&mut self, dir: Direction, ty: &syn::Type, root: bool) {
        let key = TypeKey::from_type(ty);
        let subject = match self.flat.type_ref(ty) {
            Some(t) => TypeSubject::Source(Box::new(t.clone())),
            None => TypeSubject::Adapter,
        };
        let cell = self
            .type_table_mut(dir)
            .entry(key)
            .or_insert_with(|| TypeCell {
                subject,
                root: false,
                entry: None,
            });
        cell.root |= root;
    }

    /// Enumerate the immediate type-graph edges out of `(dir, ty)`:
    /// generic args / Fn args / tuple elements / ref/array/slice/ptr targets,
    /// plus — if `ty` is the bare ident of an indexed struct or enum — the
    /// field types of that struct/enum.
    ///
    /// `impl Fn(args)` arg types flow with `dir.flip()`; everything else
    /// inherits `dir`. Used by both `register_type_inner` (during scan) and
    /// the unresolved-descendants BFS in `resolve` (for diagnostics).
    pub(crate) fn immediate_edges(
        &self,
        dir: Direction,
        ty: &syn::Type,
    ) -> Vec<(Direction, syn::Type)> {
        let mut out: Vec<(Direction, syn::Type)> = Vec::new();
        let (positions, child_dir) = if let Some(args) = extract_fn_trait_args(ty) {
            (args, dir.flip())
        } else {
            (immediate_subtype_positions(ty), dir)
        };
        for sub in positions {
            out.push((child_dir, sub));
        }
        // A declared type's own fields, read off the element rather than off its
        // `syn::Fields`: a positional field is an ordinary `Field` there, so the
        // named-only asymmetry the syntax walk had does not arise. An `Enum` has
        // no fields and an `Extern` declares none, which is what makes both
        // contribute nothing here.
        if let Some(name) = bare_path_ident(ty) {
            use crate::api::core::flat::{Field, Type};
            let fields: Vec<&Field> = match self.flat.declared_type(&name) {
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
