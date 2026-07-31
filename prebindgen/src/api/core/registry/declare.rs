//! Configure: what this binding builds.
//!
//! Every method here records; none derives. The registry is a passive recorder
//! precisely so it never has to call back into a generator to find out what it
//! is meant to produce.

use std::collections::HashSet;

use super::*;

impl<M> Registry<M> {
    /// A registry over this model.
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
    /// let registry: Registry<()> = Registry::new(flat)?;
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
    pub fn new(flat: crate::api::core::flat::Flat) -> Result<Self, ScanError> {
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
        Ok(registry)
    }

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
    pub fn export(&mut self, name: &syn::Ident) {
        self.declared.functions.insert(name.clone());
    }

    /// A const this binding exports.
    ///
    /// Separate from [`Self::export`] only because *having a const mechanism at
    /// all* is itself a fact: a binding that never calls this re-emits every
    /// captured const verbatim, while one that calls it emits exactly what it
    /// names. See [`Self::declares_consts`].
    pub fn export_const(&mut self, name: &syn::Ident) {
        self.declared
            .consts
            .get_or_insert_with(HashSet::new)
            .insert(name.clone());
    }

    /// Declare that this binding has a const mechanism, even if it exports no
    /// consts. Without it every captured const is re-emitted verbatim.
    pub fn declares_consts(&mut self) {
        self.declared.consts.get_or_insert_with(HashSet::new);
    }

    /// A type this binding **exports**: it crosses in both directions, and its
    /// body — a struct's fields, an enum's payloads — is scanned too.
    pub fn export_type(&mut self, key: TypeKey) {
        self.declared.types.insert(key);
    }

    /// A type that **crosses** in one direction without being exported.
    ///
    /// The escape hatch for a crossing no signature can yield: a re-exported
    /// foreign type named by a class declaration, or the value type of a
    /// constant the binding synthesizes. Direction is explicit because these
    /// are genuinely one-sided — which is what stops an output-only crossing
    /// from silently lacking its input twin, the asymmetry the old
    /// `required_output_types` had.
    pub fn cross(&mut self, dir: Direction, ty: &syn::Type) {
        self.declared.crossings.push((dir, ty.clone()));
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
    pub fn depends(&mut self, from: Crossing, on: Crossing) {
        self.declared.edges.push((from, on));
    }

    /// A function this binding **references but never emits** — a helper whose
    /// name appears in a declaration. Its absence is an error; its presence
    /// emits nothing.
    pub fn reference(&mut self, name: &syn::Ident) {
        self.declared.helper_functions.insert(name.clone());
    }

    /// A function the **binding crate itself** defines, with the module path
    /// generated calls should qualify it by.
    ///
    /// There is no `#[prebindgen]` item behind it, so this is the one input
    /// that adds to the model rather than selecting from it: only the
    /// signature is read, never the body. A name colliding with a captured
    /// item is an error — the generated call would resolve the wrong function.
    pub fn local_function(
        &mut self,
        item_fn: syn::ItemFn,
        origin: String,
    ) -> Result<(), ScanError> {
        let ident = item_fn.sig.ident.clone();
        // Written by hand in a build script, so the grammar is checked here or
        // nowhere: a dropped `self` receiver would surface as an arity mismatch
        // out of rustc on generated code, which is the wrong end of the pipeline
        // to learn about a build.rs typo.
        let lowered =
            self.flat
                .lower_signature(&item_fn)
                .map_err(|error| ScanError::AdapterInvariant {
                    message: format!("binding-local fn `{ident}`: {error}"),
                })?;
        if self.flat.element(&ident).is_some() {
            return Err(ScanError::AdapterInvariant {
                message: format!(
                    "binding-local fn `{ident}` collides with a `#[prebindgen]` item — \
                     the generated call would resolve the wrong fn; rename the \
                     binding-local fn"
                ),
            });
        }
        self.flat.add_local_function(lowered, origin);
        Ok(())
    }

    /// A function a decomposition reaches through rather than emits — excluded
    /// from constructor composition, and the only functions a decomposer record
    /// may name.
    ///
    /// Rides here until decompositions carry their own shape (step 2 of #251);
    /// it is a property of the decomposition, not of the binding.
    pub fn accessor(&mut self, name: &syn::Ident) {
        self.declared.accessors.insert(name.clone());
    }

    /// The receiver type of a function emitted as a method. Same temporary
    /// home as [`Self::accessor`].
    pub fn method_receiver(&mut self, name: &syn::Ident, receiver: TypeKey) {
        self.declared
            .method_receivers
            .insert(name.clone(), receiver);
    }

    /// How this binding's composites cross **in pieces** instead of whole.
    ///
    /// Stated once, before [`Self::resolve`]. Replaces five separate callbacks
    /// the registry used to make into the generator; see [`Decompositions`].
    pub fn decompose(&mut self, d: Decompositions) {
        self.declared.decompositions = d;
    }
}
