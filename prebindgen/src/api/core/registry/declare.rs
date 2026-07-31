//! Build a registry: say what the binding contains, then close it.
//!
//! Every declaring method here records; none derives. The builder is a passive
//! recorder precisely so it never has to call back into a generator to find out
//! what it is meant to produce — and it is a *separate type* from [`Registry`]
//! so that "still being described" and "finished, and answerable" cannot be
//! confused for one another.

use std::collections::{HashMap, HashSet};

use super::*;

/// A registry under construction.
///
/// Chain the declarations, hand over the conversions, then [`build`](Self::build):
///
/// ```ignore
/// let registry = Registry::builder(flat)?
///     .export(&name)
///     .decompose(decompositions)
///     .convert_with(|crossing, built| my_gen.convert(crossing, built))?
///     .build()?;
/// ```
///
/// The result is read-only. Nothing can add a crossing to a `Registry`, which
/// is what makes "every crossing has a conversion" a fact about the type rather
/// than a phase you have to be careful about.
pub struct RegistryBuilder<M> {
    registry: Registry<M>,
    /// Conversions handed over so far, applied at [`Self::build`].
    built: HashMap<Crossing, TypeEntry<M>>,
    /// The scan runs once, on demand: it needs every declaration, and
    /// [`Self::crossings`] / [`Self::convert_with`] / [`Self::build`] each need
    /// it to have run. `Some` holds the derived demand, in order.
    order: Option<Vec<Crossing>>,
}

impl<M> Registry<M> {
    /// Start describing a binding over this model.
    ///
    /// A `Flat` is what a registry projects, and reading captured prebindgen
    /// output into one is [`FlatBuilder`](crate::core::flat::FlatBuilder)'s job
    /// — so a build script says where items come from at the layer that owns
    /// the question, and there is one such layer rather than two:
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen::core::{Flat, Registry};
    ///
    /// let flat = Flat::builder().source("source_ffi").build()?;
    /// // Annotated only because nothing here resolves: in a build script `M` is
    /// // fixed by the adapter passed to `resolve`, so no call site names it.
    /// let registry: Registry<()> = Registry::builder(flat)?.build()?;
    /// assert!(registry.flat().function("test_function").is_some());
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Several sources compose there too, including one this crate renames:
    ///
    /// ```ignore
    /// let flat = Flat::builder()
    ///     .source(flat_crate::PREBINDGEN_OUT_DIR)
    ///     .source_named(helpers::PREBINDGEN_OUT_DIR, "helpers")
    ///     .build()?;
    /// ```
    ///
    /// **Fails on anything the language cannot express** — a `self` receiver, an
    /// `async fn`, a generic binder, a type form outside the grammar, or a
    /// reference to a type the flat API does not declare. All of them at once, so
    /// a source crate that needs migrating sees one list instead of one rebuild
    /// per item. This is independent of what any binding declares: an
    /// inexpressible item is a hard error whether or not it is ever named.
    pub fn builder(flat: crate::api::core::flat::Flat) -> Result<RegistryBuilder<M>, ScanError> {
        let entries: Vec<NotExpressibleEntry> = flat
            .unsupported()
            .map(|u| NotExpressibleEntry {
                name: u.name.clone(),
                reason: u.error.to_string(),
                location: (*u.origin.location).clone(),
            })
            .collect();
        if !entries.is_empty() {
            return Err(ScanError::NotExpressible { entries });
        }

        let mut registry = Registry::empty();
        registry.flat = flat;
        Ok(RegistryBuilder {
            registry,
            built: HashMap::new(),
            order: None,
        })
    }
}

impl<M> RegistryBuilder<M> {
    // ── configure: what this binding builds ───────────────────────────
    //
    // Pushed in by the generator before `resolve`. The registry never asks —
    // it records, then derives the crossing set from what it was given.

    /// An element this binding **exports**.
    ///
    /// The model says how to derive its crossings, so the caller does not: a
    /// function's signature gives its parameters (in) and its return (out); a
    /// const gives its value type (out). A name matching no element is an
    /// error, reported with every other missing name at once by `resolve`
    /// rather than here — a build script with three typos should learn all
    /// three in one build.
    pub fn export(mut self, name: &syn::Ident) -> Self {
        self.registry.declared.functions.insert(name.clone());
        self
    }

    /// A const this binding exports.
    ///
    /// Separate from [`Self::export`] only because *having a const mechanism at
    /// all* is itself a fact: a binding that never calls this re-emits every
    /// captured const verbatim, while one that calls it emits exactly what it
    /// names. See [`Self::declares_consts`].
    pub fn export_const(mut self, name: &syn::Ident) -> Self {
        self.registry
            .declared
            .consts
            .get_or_insert_with(HashSet::new)
            .insert(name.clone());
        self
    }

    /// Declare that this binding has a const mechanism, even if it exports no
    /// consts. Without it every captured const is re-emitted verbatim.
    pub fn declares_consts(mut self) -> Self {
        self.registry
            .declared
            .consts
            .get_or_insert_with(HashSet::new);
        self
    }

    /// A type this binding **exports**: it crosses in both directions, and its
    /// body — a struct's fields, an enum's payloads — is scanned too.
    pub fn export_type(mut self, key: TypeKey) -> Self {
        self.registry.declared.types.insert(key);
        self
    }

    /// A type that **crosses** in one direction without being exported.
    ///
    /// The escape hatch for a crossing no signature can yield: a re-exported
    /// foreign type named by a class declaration, or the value type of a
    /// constant the binding synthesizes. Direction is explicit because these
    /// are genuinely one-sided — which is what stops an output-only crossing
    /// from silently lacking its input twin, the asymmetry the old
    /// `required_output_types` had.
    pub fn cross(mut self, dir: Direction, ty: &syn::Type) -> Self {
        self.registry.declared.crossings.push((dir, ty.clone()));
        self
    }

    /// `from`'s conversion needs `on`'s to exist first.
    ///
    /// [`Self::crossings`] derives its order from the type structure, which
    /// covers almost everything: an `Option<T>` visibly contains a `T`. It
    /// cannot see a dependency the *declaration* creates — a `convert!` whose
    /// body chains through a helper function's parameter type, say, where
    /// nothing about the target type mentions the other side.
    ///
    /// State those here, and the order accounts for them. Getting it wrong is
    /// not silent: the conversion that needed the missing one simply cannot be
    /// built, and [`Self::supply`] names it.
    pub fn depends(mut self, from: Crossing, on: Crossing) -> Self {
        self.registry.declared.edges.push((from, on));
        self
    }

    /// A function this binding **references but never emits** — a helper whose
    /// name appears in a declaration. Its absence is an error; its presence
    /// emits nothing.
    pub fn reference(mut self, name: &syn::Ident) -> Self {
        self.registry.declared.helper_functions.insert(name.clone());
        self
    }

    /// A function the **binding crate itself** defines, with the module path
    /// generated calls should qualify it by.
    ///
    /// There is no `#[prebindgen]` item behind it, so this is the one input
    /// that adds to the model rather than selecting from it: only the
    /// signature is read, never the body. A name colliding with a captured
    /// item is an error — the generated call would resolve the wrong function.
    pub fn local_function(
        mut self,
        item_fn: syn::ItemFn,
        origin: String,
    ) -> Result<Self, ScanError> {
        let ident = item_fn.sig.ident.clone();
        // Written by hand in a build script, so the grammar is checked here or
        // nowhere: a dropped `self` receiver would surface as an arity mismatch
        // out of rustc on generated code, which is the wrong end of the pipeline
        // to learn about a build.rs typo.
        let lowered = self
            .registry
            .flat
            .lower_signature(&item_fn)
            .map_err(|error| ScanError::AdapterInvariant {
                message: format!("binding-local fn `{ident}`: {error}"),
            })?;
        if self.registry.flat.element(&ident).is_some() {
            return Err(ScanError::AdapterInvariant {
                message: format!(
                    "binding-local fn `{ident}` collides with a `#[prebindgen]` item — \
                     the generated call would resolve the wrong fn; rename the \
                     binding-local fn"
                ),
            });
        }
        self.registry.flat.add_local_function(lowered, origin);
        Ok(self)
    }

    /// A function a decomposition reaches through rather than emits — excluded
    /// from constructor composition, and the only functions a decomposer record
    /// may name.
    ///
    /// Rides here until decompositions carry their own shape (step 2 of #251);
    /// it is a property of the decomposition, not of the binding.
    pub fn accessor(mut self, name: &syn::Ident) -> Self {
        self.registry.declared.accessors.insert(name.clone());
        self
    }

    /// The receiver type of a function emitted as a method. Same temporary
    /// home as [`Self::accessor`].
    pub fn method_receiver(mut self, name: &syn::Ident, receiver: TypeKey) -> Self {
        self.registry
            .declared
            .method_receivers
            .insert(name.clone(), receiver);
        self
    }

    /// How this binding's composites cross **in pieces** instead of whole.
    ///
    /// Stated once, before [`Self::resolve`]. Replaces five separate callbacks
    /// the registry used to make into the generator; see [`Decompositions`].
    pub fn decompose(mut self, d: Decompositions) -> Self {
        self.registry.declared.decompositions = d;
        self
    }
}

impl<M> RegistryBuilder<M> {
    /// The model being described. Complete from the first call: everything that
    /// adds to it ([`Self::local_function`]) is a declaration, not a derivation.
    pub fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.registry.flat
    }

    /// Module paths of every ingested source, ingestion order.
    ///
    /// A model question, and the model is complete from the first call — so the
    /// builder answers it exactly as the finished registry does.
    pub fn all_source_modules(&self) -> Vec<syn::Path> {
        self.registry.all_source_modules()
    }

    /// The origin crate's module path for an item — see
    /// [`Registry::origin_module`].
    pub fn origin_module(&self, ident: &syn::Ident) -> Option<syn::Path> {
        self.registry.origin_module(ident)
    }

    /// The default module for references with no recorded origin — see
    /// [`Registry::default_module`].
    pub fn default_module(&self) -> Option<syn::Path> {
        self.registry.default_module()
    }

    /// Every **named** item the model holds — see
    /// [`Registry::named_item_idents`].
    pub fn named_item_idents(&self) -> impl Iterator<Item = &syn::Ident> {
        self.registry.named_item_idents()
    }

    /// Whether the source declares a type under this name — see
    /// `Registry::declares_type`.
    #[cfg(test)]
    pub(crate) fn declares_type(&self, ident: &syn::Ident) -> bool {
        self.registry.declares_type(ident)
    }

    /// Run the scan and apply the decompositions, once.
    ///
    /// Private and idempotent: three entry points need it to have happened, and
    /// none of them should care whether it already did.
    fn derive(&mut self) -> Result<&[Crossing], WriteRustError> {
        if self.order.is_none() {
            let mut declared = std::mem::take(&mut self.registry.declared);
            let out = (|| {
                self.registry.scan_declared_items(&declared)?;
                self.registry.apply_adapter_plans(&mut declared)
            })();
            self.registry.declared = declared;
            out?;
            self.order = Some(self.registry.crossings());
        }
        Ok(self.order.as_deref().unwrap_or_default())
    }

    /// What a conversion — or a validation — is written against right now: the
    /// model, the full crossing population, and whatever has been built so far.
    fn view(&self) -> Building<'_, M> {
        Building::new(
            &self.registry,
            &self.built,
            self.order.as_deref().unwrap_or_default(),
        )
    }

    /// Check this binding against a generator's own invariants, now that the
    /// scan has read every declared signature.
    ///
    /// Earliest it can run: a missing declaration has already hard-errored, so
    /// a check here sees only items that exist.
    pub fn validate_with<E>(mut self, adapter: &E) -> Result<Self, WriteRustError>
    where
        E: Prebindgen<Metadata = M>,
    {
        self.derive()?;
        adapter
            .validate(&self.view())
            .map_err(|message| ScanError::AdapterInvariant { message })?;
        Ok(self)
    }

    /// Every crossing this binding needs a conversion for, **inner types
    /// first** — see [`Registry::crossings`] for what the order guarantees.
    ///
    /// Take this when you want to drive the loop yourself and hand the result
    /// back through [`Self::conversions`]. [`Self::convert_with`] is the same
    /// walk with the loop written for you.
    pub fn crossings(&mut self) -> Result<Vec<Crossing>, WriteRustError> {
        Ok(self.derive()?.to_vec())
    }

    /// Build a conversion for each crossing, in dependency order.
    ///
    /// `f` is called once per crossing with the conversions already built, so
    /// by the time it sees `Option<Handle>` it can look up `Handle`. Returning
    /// `None` records a gap — whether that gap matters is decided by
    /// [`Self::build`], not here.
    ///
    /// This is a convenience over [`Self::crossings`] + [`Self::conversions`],
    /// not a second mechanism: it is the same list, walked in the same order.
    /// Nothing about it lets the registry choose when to call back — the
    /// closure is yours, and the walk is finished before this returns.
    pub fn convert_with<F>(mut self, mut f: F) -> Result<Self, WriteRustError>
    where
        F: FnMut(
            &Crossing,
            &Building<'_, M>,
        ) -> Option<crate::api::core::prebindgen::ConverterImpl<M>>,
    {
        let order = self.derive()?.to_vec();
        for crossing in &order {
            let conv = f(crossing, &self.view());
            if let Some(c) = conv {
                self.built
                    .insert(crossing.clone(), TypeEntry::from_converter(c));
            }
        }
        Ok(self)
    }

    /// Hand over conversions built elsewhere — the bulk peer of
    /// [`Self::convert_with`], for a generator that walked
    /// [`Self::crossings`] itself.
    ///
    /// Accumulates, so it composes with `convert_with` and with itself.
    pub fn conversions(mut self, conversions: HashMap<Crossing, TypeEntry<M>>) -> Self {
        self.built.extend(conversions);
        self
    }

    /// The scanned registry, with no conversions applied and no completeness
    /// check.
    ///
    /// Test-only, and deliberately so: it is the state between "described" and
    /// "answerable", which is exactly what the split exists to keep out of
    /// everyone else's hands.
    #[cfg(test)]
    pub(crate) fn scanned(mut self) -> Result<Registry<M>, ScanError> {
        // Narrower than `build`'s error on purpose: the scan is the only phase
        // this runs, so a test matching on `ScanError` says what it means.
        match self.derive() {
            Ok(_) => Ok(self.registry),
            Err(WriteRustError::Scan(e)) => Err(e),
            Err(other) => panic!("scanned(): unexpected non-scan failure: {other}"),
        }
    }

    /// Close the binding: apply every conversion, check the set is complete,
    /// and hand back a registry that can only be read.
    ///
    /// A crossing with no conversion is not itself a failure — the scan
    /// over-approximates on purpose. What fails is a crossing *reachable from
    /// an export* with none, and the error names every one at once.
    pub fn build(mut self) -> Result<Registry<M>, WriteRustError> {
        self.derive()?;
        for ((dir, key), entry) in self.built {
            if let Some(cell) = self.registry.type_table_mut(dir).get_mut(&key) {
                cell.entry = Some(entry);
            }
        }
        crate::api::core::resolve::check_complete(&self.registry)?;
        Ok(self.registry)
    }
}

/// A builder answers the same questions a finished registry does — with one
/// difference that is the whole point of the split: [`conversion`] sees only
/// what has been handed over *so far*.
///
/// That is what a generator writing a conversion needs (its inners, already
/// built) and it is all it should be able to see. Everything else — the model,
/// the decompositions — is complete from the moment it is declared.
///
/// [`conversion`]: Conversions::conversion
impl<M> Conversions<M> for RegistryBuilder<M> {
    fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.registry.flat
    }
    fn conversion(&self, dir: Direction, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        self.built.get(&(dir, TypeKey::from_type(ty)))
    }
    fn crossing_keys(&self, dir: Direction) -> Vec<TypeKey> {
        self.order
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|(d, _)| *d == dir)
            .map(|(_, k)| k.clone())
            .collect()
    }
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&crate::api::core::unfold::UnfoldPlan> {
        self.registry.callback_arg_plans.get(key)
    }
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, crate::api::core::unfold::UnfoldPlan> {
        &self.registry.callback_arg_plans
    }
    fn unfold_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan> {
        &self.registry.unfold_plans
    }
    fn error_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan> {
        &self.registry.error_plans
    }
    fn decon_plans(
        &self,
    ) -> &HashMap<crate::api::core::unfold::DeconId, crate::api::core::unfold::DeconSpec> {
        &self.registry.decon_plans
    }
}
