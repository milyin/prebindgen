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
        // entries, and the reason is what they look at: `declared.types` are types a
        // *build script author* wrote, and this is a diagnostic about the spelling
        // they wrote — is it path-qualified, and does its tail shadow a captured
        // item? No source type is being classified, so there is no element to read
        // instead; asking the model would answer about a type rather than about the
        // declaration. This is the "legitimately the adapter's business" case the
        // integration map (L2, #229) predicts, not a migration still owed.
        //
        // It reads the **declaration's own** spelling, canonicalized here. Both
        // halves of that matter. Normalizing is what the paragraph above relies
        // on — `crate::Foo` must not read as a qualified path — and it used to
        // arrive for free because the tokens came out of the key, which is
        // normalized by construction. Doing it explicitly costs one call and
        // stops the key from being the source of tokens at all (#291).
        let mut qualified: Vec<(String, String)> = Vec::new();
        let mut probed: HashSet<&TypeKey> = HashSet::new();
        for (key, declared_ty) in declared
            .types
            .iter()
            .chain(declared.decompositions.replaces.iter())
        {
            if !probed.insert(key) {
                continue;
            }
            let ty = crate::api::core::flat::canonical_type(declared_ty.as_syn());
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
            if let Some(item_fn) = self
                .flat
                .function(&ident)
                .map(|f| f.origin.as_syn().clone())
            {
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
            if let Some(item_const) = self
                .flat
                .constant(&ident)
                .map(|c| c.origin.as_syn().clone())
            {
                self.intern(Direction::Output, &item_const.ty, true)?;
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
            self.intern(*dir, ty, true)?;
        }

        // Scan declared types. The spelling is the declaration's own — `intern`
        // needs real tokens for a type that is in no table yet, which is exactly
        // the case a key cannot answer once it is only an identity (#291).
        for declared_ty in declared.types.values() {
            // Canonicalized for the same reason the diagnostic above is: this
            // is the form the type used to arrive in, and interning the
            // as-written spelling instead would put a differently-spelled
            // reading in the cell for the same key.
            let ty = crate::api::core::flat::canonical_type(declared_ty.as_syn());
            let mut matched = false;
            if let Some(ident) = bare_path_ident(&ty) {
                if let Some(s) = self.flat.struct_type(&ident).cloned() {
                    self.scan_struct(&s);
                    self.intern(Direction::Input, &ty, true)?;
                    self.intern(Direction::Output, &ty, true)?;
                    matched = true;
                } else if let Some(e) = self
                    .flat
                    .declared_type(&ident)
                    .filter(|t| {
                        matches!(
                            t,
                            crate::api::core::flat::Type::Enum(_)
                                | crate::api::core::flat::Type::Variant(_)
                        )
                    })
                    .cloned()
                {
                    self.scan_enum(&e);
                    self.intern(Direction::Input, &ty, true)?;
                    self.intern(Direction::Output, &ty, true)?;
                    matched = true;
                }
            }
            if !matched {
                // Declared type without an indexed body (e.g.
                // `ptr_class(ZKeyExpr<'static>)` on a re-exported
                // foreign type). Still mark required so the resolver
                // tries to produce a converter for it.
                self.intern(Direction::Input, &ty, true)?;
                self.intern(Direction::Output, &ty, true)?;
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
                    self.intern_recursive(Direction::Input, &pt.ty, true)?;
                }
            }
        }
        let ret_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        self.intern_recursive(Direction::Output, &ret_ty, true)?;
        Ok(())
    }

    /// Register a declared struct and every one of its field types.
    ///
    /// Takes the **element**: the struct's own type is `Struct::type_ref` — the
    /// reading the declaration carries — and each field already holds one, so
    /// nothing here is spelled, keyed or classified on the way in. It used to
    /// take a `syn::ItemStruct`, rebuild the type from the ident, and walk
    /// `syn::Fields::Named` to reach types the element had all along.
    pub(super) fn scan_struct(&mut self, s: &crate::api::core::flat::Struct) {
        // The struct itself can appear in either direction.
        self.intern_reading(Direction::Input, s.type_ref(), false);
        self.intern_reading(Direction::Output, s.type_ref(), false);

        for field in &s.fields {
            self.intern_recursive_reading(Direction::Input, &field.ty, false);
            self.intern_recursive_reading(Direction::Output, &field.ty, false);
        }
    }

    /// Register a declared enum and every payload type its alternatives carry.
    ///
    /// The [`Struct`](crate::api::core::flat::Struct) twin, and it takes the
    /// model's split seriously: a fieldless [`Enum`](crate::api::core::flat::Enum)
    /// has no payload to reach, and a [`Variant`](crate::api::core::flat::Variant)
    /// carries its alternatives' fields as readings. Walking `syn`'s `variants`
    /// could not tell the two apart and had to look at every field to find out.
    pub(super) fn scan_enum(&mut self, e: &crate::api::core::flat::Type) {
        use crate::api::core::flat::Type;
        let reading = match e {
            Type::Enum(en) => en.type_ref(),
            Type::Variant(v) => v.type_ref(),
            _ => return,
        };
        self.intern_reading(Direction::Input, reading, false);
        self.intern_reading(Direction::Output, reading, false);

        if let Type::Variant(v) = e {
            for alt in &v.alternatives {
                for field in &alt.fields {
                    self.intern_recursive_reading(Direction::Input, &field.ty, false);
                    self.intern_recursive_reading(Direction::Output, &field.ty, false);
                }
            }
        }
    }

    /// Register `ty` as a cell in the given direction, then recurse into every
    /// nested position. `root` applies only to `ty` itself — a nested position is
    /// never something the binding asked for directly.
    pub(super) fn register_type_recursive(
        &mut self,
        dir: Direction,
        reading: &crate::api::core::flat::TypeRef,
        root: bool,
    ) {
        let mut visited: HashSet<TypeKey> = HashSet::new();
        self.register_type_inner(dir, reading, root, &mut visited)
    }

    /// Infallible, and structurally so: every type reached here is a reading —
    /// the caller's, or one the model already holds for a child — so there is
    /// nothing left to classify and nothing left to refuse.
    pub(super) fn register_type_inner(
        &mut self,
        dir: Direction,
        reading: &crate::api::core::flat::TypeRef,
        is_top: bool,
        visited: &mut HashSet<TypeKey>,
    ) {
        let key = reading.key();
        if !visited.insert(key.clone()) {
            return; // cycle guard
        }

        self.ensure_entry(dir, reading, is_top);

        for (child_dir, sub) in self.immediate_edges(dir, &key) {
            self.register_type_inner(child_dir, &sub, false, visited);
        }
    }

    /// Create the cell for `reading` in `dir` if it has none, and mark it a root
    /// when the binding asked for it directly.
    ///
    /// The one place a cell is born, and therefore the one place a type **enters
    /// the pipeline** — including a spelling the source never wrote, since
    /// expansion composes those (an `Option<T>` around a `T` it found) and hands
    /// them straight here via `require_input` / `require_output`.
    ///
    /// **The caller's reading is what gets stored.** It is not re-derived from
    /// the spelling, and that is the point (#281): the reading a caller holds and
    /// the one `classify` would produce for its spelling are two answers from two
    /// paths, and nothing was comparing them. Now there is only one answer,
    /// because there is only one classification.
    ///
    /// Which is also why this is **infallible**. It was fallible for exactly one
    /// reason — `classify` refusing a spelling — and a reading has already been
    /// through that. Only [`intern`](Self::intern), the door for a spelling
    /// nobody has classified yet, can still fail.
    ///
    /// The model is consulted, never extended: a composed spelling is an
    /// intermediate in *this binding's* crossing graph, not something the source
    /// API mentions, so `Flat` stays what the source said while every type the
    /// pipeline works with has its reading in the table.
    pub(super) fn ensure_entry(
        &mut self,
        dir: Direction,
        reading: &crate::api::core::flat::TypeRef,
        root: bool,
    ) {
        let key = reading.key();
        // The reading of a given key cannot change, so an existing cell already
        // holds an equal one — only the root flag can still move.
        if let Some(cell) = self.type_table_mut(dir).get_mut(&key) {
            cell.root |= root;
            return;
        }
        self.type_table_mut(dir).insert(
            key,
            TypeCell {
                subject: Box::new(reading.clone()),
                root,
                entry: None,
            },
        );
    }

    /// Classify a **spelling** and register it — the one door for a type that
    /// has no reading yet, and the only fallible way into the table.
    ///
    /// Everything the pipeline composes or walks already holds a
    /// [`TypeRef`](crate::api::core::flat::TypeRef) and goes through
    /// [`ensure_entry`](Self::ensure_entry) instead. What genuinely arrives as
    /// tokens is a spelling *authored outside the model*: a build script's
    /// declared crossing, a constant's declared type, a `syn` type the plan
    /// engines assemble for their own wire shape.
    ///
    /// A spelling the grammar refuses is reported by name here, rather than
    /// becoming a cell that quietly means less than its neighbours.
    ///
    /// **`pub(in crate::api::core)` deliberately**, matching
    /// `Flat::classify` and the `TypeRef` composers. Classifying a spelling
    /// *mints a reading*, and #280 sealed that to `api::core`: an adapter under
    /// `api::lang` must not be able to hand the registry tokens of its own and
    /// receive a `TypeRef` back. A one-door design that widened the door would
    /// have re-opened exactly the capability #280 closed — so this must stay no
    /// wider than the composers it replaces as an entry point.
    pub(in crate::api::core) fn intern(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
    ) -> Result<crate::api::core::flat::TypeRef, ScanError> {
        // The registry's own answer first, in EITHER direction — a reading is
        // direction-free, and a cell that exists already holds the authoritative
        // one. Classifying anyway would derive a second reading for a key that
        // has one, which `ensure_entry` would then discard: the same
        // two-answers-that-never-meet shape this PR removes, surviving as
        // redundant work rather than as a replaced cell.
        let key = TypeKey::from_type(ty);
        if let Some(known) = self
            .input_types
            .get(&key)
            .or_else(|| self.output_types.get(&key))
            .map(|c| (*c.subject).clone())
        {
            self.ensure_entry(dir, &known, root);
            return Ok(known);
        }
        let reading = self
            .flat
            .classify(ty)
            .map_err(|source| ScanError::NotExpressible {
                entries: vec![NotExpressibleEntry {
                    name: None,
                    reason: source.to_string(),
                    location: SourceLocation::default(),
                }],
            })?;
        self.ensure_entry(dir, &reading, root);
        Ok(reading)
    }

    /// [`intern`](Self::intern), then register every nested position — the
    /// recursive door, for a spelling whose children must become crossings too
    /// (a parameter, a return, a declared field).
    ///
    /// Only the top type is classified: the walk below it takes each child's
    /// reading off the parent's, so nothing under here is re-derived.
    pub(super) fn intern_recursive(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
    ) -> Result<(), ScanError> {
        let reading = self.intern(dir, ty, root)?;
        self.register_type_recursive(dir, &reading, root);
        Ok(())
    }

    /// [`Self::intern`] for a caller that **already holds the reading**.
    ///
    /// `intern` exists to turn a spelling into one: it keys the type, looks for
    /// a cell, and classifies when there is none. A caller with a reading in
    /// hand needs none of that — the model already answered, and re-deriving
    /// would be the "two answers that never meet" shape `intern`'s own comment
    /// warns about, arriving from the other side.
    pub(in crate::api::core) fn intern_reading(
        &mut self,
        dir: Direction,
        reading: &crate::api::core::flat::TypeRef,
        root: bool,
    ) {
        let known = self
            .input_types
            .get(&reading.key())
            .or_else(|| self.output_types.get(&reading.key()))
            .map(|c| (*c.subject).clone());
        self.ensure_entry(dir, known.as_ref().unwrap_or(reading), root);
    }

    /// [`Self::intern_recursive`] for a caller that already holds the reading.
    pub(super) fn intern_recursive_reading(
        &mut self,
        dir: Direction,
        reading: &crate::api::core::flat::TypeRef,
        root: bool,
    ) {
        self.intern_reading(dir, reading, root);
        self.register_type_recursive(dir, reading, root);
    }

    /// Enumerate the immediate type-graph edges out of `(dir, key)`: the model's
    /// own children of this type, plus — if `key` names a declared struct or sum —
    /// the field types of that item.
    ///
    /// A callback's argument types flow with `dir.flip()`, because an argument the
    /// binding *hands to* a callback crosses the other way; everything else
    /// inherits `dir`. Used by both `register_type_inner` (during scan) and the
    /// unresolved-descendants BFS in `resolve` (for diagnostics).
    ///
    /// **Takes the key, because a key is all it ever used.** This asked for a
    /// `&syn::Type` and opened by re-keying it, so every caller spelled a key into
    /// tokens purely so this could undo that — a normalize pass and a token render
    /// per call, to arrive back where it started. What the walk needs is a table
    /// lookup, and a table lookup takes an identity (#291).
    ///
    /// The children come from [`TypeKind`], not from taking the syntax apart, and
    /// the difference is load-bearing rather than cosmetic. `&mut MaybeUninit<T>`
    /// yields `T` — [`borrow_target`](crate::api::core::flat::TypeRef::borrow_target)
    /// sees past the slot — instead of an intermediate `MaybeUninit<T>` that no
    /// adapter can convert and no table holds. Each edge is still *spelled* from
    /// the child's own `spell()`, which is what the caller keys the table by.
    ///
    /// The reading comes from **this registry's own table**, where `ensure_entry`
    /// put it before the walk reached this type — so a spelling the binding composed
    /// is answered exactly like one the source wrote, without asking the model about
    /// a type it never saw. No cell means the type was never registered, and an
    /// unregistered type is not part of any crossing to walk.
    pub(crate) fn immediate_edges(
        &self,
        dir: Direction,
        key: &TypeKey,
    ) -> Vec<(Direction, crate::api::core::flat::TypeRef)> {
        use crate::api::core::flat::TypeKind;

        let mut out: Vec<(Direction, crate::api::core::flat::TypeRef)> = Vec::new();
        if let Some(reading) = self.type_table(dir).get(key).map(|c| &c.subject) {
            let (children, child_dir): (Vec<&crate::api::core::flat::TypeRef>, Direction) =
                match reading.unwrapped().kind() {
                    // Through the accessor, not the field: it sees past an
                    // out-parameter's `MaybeUninit` slot, which is storage rather
                    // than a type any converter is keyed by.
                    // `expect`, not a fallible collect: a `Ref` kind always has a
                    // target, so an empty child list here would mean the accessor
                    // and the kind disagree — and it would silently truncate the
                    // graph walk instead of saying so.
                    TypeKind::Ref { .. } => (
                        vec![reading
                            .borrow_target()
                            .expect("a `Ref` kind has a borrow target")],
                        dir,
                    ),
                    TypeKind::Optional(t)
                    | TypeKind::Vec(t)
                    | TypeKind::Slice(t)
                    | TypeKind::Uninit(t) => (vec![t], dir),
                    TypeKind::Array { elem, .. } => (vec![elem], dir),
                    TypeKind::Fallible { ok, err } => (vec![ok, err], dir),
                    TypeKind::Callback { args } => (args.iter().collect(), dir.flip()),
                    // A name is a leaf in the type graph: its generic arguments
                    // belong to the reference, not to a declaration, because no
                    // declaration takes type parameters. Its *fields* are the
                    // edges, and they come off the element below.
                    TypeKind::Named { .. }
                    | TypeKind::Scalar(_)
                    | TypeKind::Str
                    | TypeKind::String
                    | TypeKind::Unit => (Vec::new(), dir),
                    // `unwrapped` peeled these off.
                    TypeKind::Boxed(_) | TypeKind::Cow { .. } => (Vec::new(), dir),
                };
            // The child reading itself, not its spelling: it has already been
            // classified — by the model, or by whoever composed the parent — so
            // handing back tokens for the caller to re-classify is the discard
            // this walk exists to avoid (#281).
            for child in children {
                out.push((child_dir, child.clone()));
            }
        }
        // A spelling the model **erased wrappers from** depends on the stripped
        // spelling: whoever converts `Box<T>` does it by delegating to `T`'s own
        // converter and putting the wrapper back. That is a real edge and the
        // `kind` walk above cannot see it — `Box<T>` classifies as whatever `T`
        // is, so the two share a classification and differ only in spelling.
        //
        // Without it the dependency existed but the ORDER did not: a converter
        // that delegates is built in one pass, so it needs its inner already
        // built, and `subs` says "this is required" rather than "this comes
        // first". `Box<Payload>` resolved only because some other root's fields
        // happened to pull `Payload` in earlier — alphabetical luck, which
        // `Box<ZSample>` did not have.
        if let Some(cell) = self.type_table(dir).get(key) {
            let reading = &cell.subject;
            if !reading.erased_wrappers().is_empty() {
                let stripped = reading.stripped_key();
                if stripped != *key {
                    if let Some(inner) = self.type_table(dir).get(&stripped) {
                        out.push((dir, (*inner.subject).clone()));
                    }
                }
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
        if let Some(name) =
            self.type_table(dir)
                .get(key)
                .and_then(|c| match c.subject.unwrapped().kind() {
                    TypeKind::Named { id, .. } => Some(id.name.clone()),
                    _ => None,
                })
        {
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
                out.push((dir, field.ty.clone()));
            }
        }
        out
    }

    /// Put a crossing in the table with its conversion already decided — the
    /// fixture form of "this type crosses, and here is how".
    ///
    /// Goes through [`Self::ensure_entry`] rather than building a cell beside it,
    /// so a fixture table is reached the same way a real one is and a hand-written
    /// key is held to the same grammar. A test that wants the whole scan builds its
    /// registry from items instead; this is for the ones that need a specific table
    /// shape and nothing else.
    /// Takes the **spelling**, like every other door into the table: interning
    /// needs real tokens, and a fixture has them — it wrote them (#291).
    #[cfg(test)]
    pub(crate) fn insert_crossing(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
        entry: Option<TypeEntry<M>>,
    ) {
        self.intern(dir, ty, root).unwrap_or_else(|e| {
            panic!(
                "fixture type `{}` is not expressible: {e}",
                ty.to_token_stream()
            )
        });
        self.type_table_mut(dir)
            .get_mut(&TypeKey::from_type(ty))
            .expect("just registered")
            .entry = entry;
    }

    /// The reading the scan stored for `ty` — **a lookup, and only a lookup**.
    ///
    /// **The registry is the authority on what a type means**, because it is the
    /// thing that stores readings: `ensure_entry` asks the grammar once when a cell
    /// is born, and this hands that answer back. `Flat::classify` is its private
    /// tool, and `ensure_entry` is its only caller.
    ///
    /// This used to classify on a miss, which meant there were two sources of
    /// readings and no way to tell them apart. The fallback fired constantly, and
    /// on **scalars** — `i64`, `String`, `bool` — which are certainly registered by
    /// the time a binding is built. That was the tell: the misses were not unknown
    /// types but an inverted order, [`unfold`](crate::api::core::unfold) asking
    /// about the leaves its caller registers one loop later. Because `classify`
    /// answered correctly, nothing downstream was wrong and nothing showed it
    /// (#266). The declarations now carry their own readings, so there is no such
    /// caller left.
    ///
    /// `None` therefore means the type never entered the pipeline — a caller
    /// asking out of order, not a cache miss to paper over.
    pub(crate) fn reading(&self, key: &TypeKey) -> Option<crate::api::core::flat::TypeRef> {
        self.input_types
            .get(key)
            .or_else(|| self.output_types.get(key))
            .map(|cell| (*cell.subject).clone())
    }

    /// Register `reading` (and its nested positions) as a required **input** so
    /// the resolver produces a converter for it. Used by
    /// [`crate::api::core::expand`] to pull in the leaf types a fold needs.
    ///
    /// Takes the **reading**, not its spelling. Every caller already holds one —
    /// a plan leaf's `ty` — and used to call `.syntax()` on it here, which is the
    /// discard #281 is about: the registry would then re-classify the tokens and
    /// store its own answer beside the caller's.
    pub(crate) fn require_input(&mut self, reading: &crate::api::core::flat::TypeRef) {
        self.register_type_recursive(Direction::Input, reading, true);
    }

    /// Register `ty` (and its nested positions) as a required **output** so the
    /// resolver produces a converter for it. The output-side peer of
    /// [`Self::require_input`]; used by [`crate::api::core::unfold`] to pull in
    /// the leaf types a decomposition delivers.
    pub(crate) fn require_output(&mut self, reading: &crate::api::core::flat::TypeRef) {
        self.register_type_recursive(Direction::Output, reading, true);
    }

    /// Register `reading` (and its nested positions) as an **output cell without
    /// demanding a converter** — a type some plan *names* rather than one that
    /// crosses.
    ///
    /// The third thing a table cell can mean, now said out loud. A cell records
    /// that a type **entered the pipeline**; `root` records that the binding
    /// asked for it *directly*; `entry` records that a converter resolved. This
    /// makes the first without the second, which is exactly what a
    /// [`SumTag`](crate::api::core::unfold::LeafSource::SumTag) selector needs:
    /// it names *which* sum it chooses between, and that sum has no whole-value
    /// output converter at all, so requiring one would fail resolution (#282).
    ///
    /// **Not [`require_output`](Self::require_output) with a flag.** That one is
    /// `root = true` by definition — its whole job is to say a converter must
    /// exist. Registration and demand are separable facts and this is the door
    /// for the first alone; `ensure_entry`'s `root |= root` means calling it for
    /// a type the binding did declare cannot weaken anything.
    pub(crate) fn reference_output(&mut self, reading: &crate::api::core::flat::TypeRef) {
        self.register_type_recursive(Direction::Output, reading, false);
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
    pub(crate) fn unrequire_output(&mut self, reading: &crate::api::core::flat::TypeRef) {
        self.clear_root(Direction::Output, &reading.key());
    }

    /// Stop treating `key` as a root. The cell stays, so the resolver still
    /// fills it if it can — only the demand that it *must* resolve is dropped.
    ///
    /// Keyed, because that is genuinely all this needs: un-requiring creates no
    /// cell and classifies nothing, so it is the one registration-adjacent
    /// operation with no reading to carry. `unrequire_*` take a `TypeRef` anyway
    /// — they pair with `require_*`, and a caller holding one should not have to
    /// know which of the two wants a key.
    pub(super) fn clear_root(&mut self, dir: Direction, key: &TypeKey) {
        if let Some(cell) = self.type_table_mut(dir).get_mut(key) {
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

    /// Look up the resolved input entry for `reading`, returning `None` if it
    /// was never registered or is still unresolved. The returned entry's
    /// `function.sig.ident` is the converter's call name; `destination` is
    /// its wire form.
    ///
    /// Takes a `TypeRef` for the reason the trait methods do (#284) — and this
    /// pair matters more than they do, because an **inherent** method wins over
    /// a trait method on a concrete `Registry`. While these took a spelling they
    /// were a second door into the table that the trait's signature could not
    /// close, and every caller with a `Registry` in hand silently used it. The
    /// same "second door inside the room" that hid `classify` behind
    /// `Registry::reading` until #267.
    pub fn input_entry(&self, reading: &crate::api::core::flat::TypeRef) -> Option<&TypeEntry<M>> {
        self.type_table(Direction::Input)
            .get(&reading.key())?
            .entry
            .as_ref()
    }

    /// Look up the resolved output entry for `reading`. See
    /// [`Self::input_entry`].
    pub fn output_entry(&self, reading: &crate::api::core::flat::TypeRef) -> Option<&TypeEntry<M>> {
        self.type_table(Direction::Output)
            .get(&reading.key())?
            .entry
            .as_ref()
    }
}
