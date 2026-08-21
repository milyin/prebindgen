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
///     .convert_with(|crossing, built, emit| my_gen.convert(crossing, built, emit))?
///     .build()?;
/// ```
///
/// The result is read-only. Nothing can add a crossing to a `Registry`, which
/// is what makes "every crossing has a conversion" a fact about the type rather
/// than a phase you have to be careful about.
pub struct RegistryBuilder {
    registry: Registry,
    /// Conversions handed over so far, applied at [`Self::build`].
    built: HashMap<Crossing, Answer>,
    /// The scan runs once, on demand: it needs every declaration, and
    /// [`Self::convert_with`] and [`Self::build`] each need
    /// it to have run. `Some` holds the derived demand, in order.
    order: Option<Vec<Crossing>>,
}

impl Registry {
    /// Start describing a binding over this model.
    ///
    /// A `Flat` is what a registry projects, and reading captured prebindgen
    /// output into one is [`FlatBuilder`](prebindgen_flat::flat::FlatBuilder)'s job
    /// — so a build script says where items come from at the layer that owns
    /// the question, and there is one such layer rather than two:
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen_registry::{Flat, Registry};
    ///
    /// let flat = Flat::builder().source("source_ffi").build()?;
    /// // Annotated only because nothing here resolves: in a build script `M` is
    /// // fixed by the adapter passed to `resolve`, so no call site names it.
    /// let registry: Registry = Registry::builder(flat)?.build()?;
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
    pub fn builder(flat: prebindgen_flat::flat::Flat) -> Result<RegistryBuilder, ScanError> {
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

impl RegistryBuilder {
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
    ///
    /// Takes the **type the declaration was written with**, like its sibling
    /// [`Self::cross`], and derives the key here. It used to take the key alone,
    /// which meant the scan had to recover tokens *from* the key to intern the
    /// type and to diagnose its spelling — reasoning backwards from an identity
    /// to a thing that already existed. A build script wrote `ptr_class!(Foo)`;
    /// this is that `Foo` (#291).
    ///
    /// Declaring the same type twice keeps the **first** spelling. That is what
    /// the `HashSet` this replaced did with the identity, and what
    /// `register_class` does with a reopened declarator: the two spellings agree
    /// on identity by construction, so the tie-break only decides which
    /// equivalent rendering the scan reads, and it should not depend on
    /// declaration order.
    pub fn export_type(mut self, ty: Origin<syn::Type>) -> Self {
        self.registry.declared.types.entry(ty.key()).or_insert(ty);
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
    /// [`Self::convert_with`] derives its order from the type structure, which
    /// covers almost everything: an `Option<T>` visibly contains a `T`. It
    /// cannot see a dependency the *declaration* creates — a `convert!` whose
    /// body chains through a helper function's parameter type, say, where
    /// nothing about the target type mentions the other side.
    ///
    /// State those here, and the order accounts for them. Getting it wrong is
    /// not silent: the conversion that needed the missing one simply cannot be
    /// built, and [`Self::build`] names it.
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
    /// Stated once, before [`Self::build`]. Replaces five separate callbacks
    /// the registry used to make into the generator; see [`Decompositions`].
    pub fn decompose(mut self, d: Decompositions) -> Self {
        self.registry.declared.decompositions = d;
        self
    }
}

impl RegistryBuilder {
    /// The model being described. Complete from the first call: everything that
    /// adds to it ([`Self::local_function`]) is a declaration, not a derivation.
    pub fn flat(&self) -> &prebindgen_flat::flat::Flat {
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
    /// model and the full crossing population.
    fn view(&self) -> Building<'_> {
        Building::new(&self.registry, self.order.as_deref().unwrap_or_default())
    }

    /// Check this binding against a generator's own invariants, now that the
    /// scan has read every declared signature.
    ///
    /// Earliest it can run: a missing declaration has already hard-errored, so
    /// a check here sees only items that exist.
    pub fn validate_with<E: Prebindgen>(mut self, adapter: &E) -> Result<Self, WriteRustError> {
        self.derive()?;
        adapter
            .validate(&self.view())
            .map_err(|message| ScanError::AdapterInvariant { message })?;
        Ok(self)
    }

    /// Build a conversion for each crossing, in dependency order.
    ///
    /// `f` is called once per crossing, and answers with an [`Answer`] rather
    /// than the conversion itself: the conversion belongs to the adapter, which
    /// emits from it and looks it up, and what the registry needs back is which
    /// other crossings this one is built out of. Returning `None` records a gap
    /// — whether that gap matters is decided by [`Self::build`], not here.
    ///
    /// The walk is inner types first, so by the time `f` sees `Option<Handle>`
    /// it has already answered `Handle`.
    ///
    /// The one way to supply them. Nothing about it lets the registry choose
    /// when to call back — the closure is yours, and the walk is finished
    /// before this returns.
    pub fn convert_with<F>(mut self, mut f: F) -> Result<Self, WriteRustError>
    where
        F: FnMut(&Crossing, &Building<'_>, &crate::Emit) -> Option<Answer>,
    {
        // A converter IS generated Rust — `ConverterImpl::function` is a
        // complete `syn::ItemFn` the adapter writes — so this closure is an
        // emission callback and is handed the capability, exactly as the
        // `on_*` ones are. See `prebindgen_flat::flat::emit`.
        let emit = crate::Emit::new();
        let order = self.derive()?.to_vec();
        for crossing in &order {
            if let Some(answer) = f(crossing, &self.view(), &emit) {
                self.built.insert(crossing.clone(), answer);
            }
        }
        Ok(self)
    }

    /// The scanned registry, with no conversions applied and no completeness
    /// check.
    ///
    /// Test-only, and deliberately so: it is the state between "described" and
    /// "answerable", which is exactly what the split exists to keep out of
    /// everyone else's hands.
    #[cfg(test)]
    pub(crate) fn scanned(mut self) -> Result<Registry, ScanError> {
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
    pub fn build(mut self) -> Result<Registry, WriteRustError> {
        self.derive()?;
        for ((dir, key), entry) in self.built {
            if let Some(cell) = self.registry.type_table_mut(dir).get_mut(&key) {
                cell.entry = Some(entry);
            }
        }
        crate::resolve::check_complete(&self.registry)?;
        Ok(self.registry)
    }
}

/// A builder answers the same questions a finished registry does. It used to
/// answer one fewer — it lent out conversions handed over *so far*, so a
/// generator writing one could see its inners and nothing else. An adapter
/// keeps its own conversions now, so that read is gone and the rest — the
/// model, the decompositions — is complete from the moment it is declared.
impl Conversions for RegistryBuilder {
    fn reading(&self, key: &TypeKey) -> Option<prebindgen_flat::flat::TypeRef> {
        self.registry.reading(key)
    }
    fn flat(&self) -> &prebindgen_flat::flat::Flat {
        &self.registry.flat
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
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&crate::unfold::UnfoldPlan> {
        self.registry.callback_arg_plans.get(key)
    }
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, crate::unfold::UnfoldPlan> {
        &self.registry.callback_arg_plans
    }
    fn unfold_plans(&self) -> &HashMap<syn::Ident, crate::unfold::UnfoldPlan> {
        &self.registry.unfold_plans
    }
    fn error_plans(&self) -> &HashMap<syn::Ident, crate::unfold::UnfoldPlan> {
        &self.registry.error_plans
    }
    fn decon_plans(&self) -> &HashMap<crate::unfold::DeconId, crate::unfold::DeconSpec> {
        &self.registry.decon_plans
    }
}
