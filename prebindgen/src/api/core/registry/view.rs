//! What a conversion is built against — the partial view during the fill, and
//! the total one after it.

use std::collections::HashMap;

use super::*;

/// One `(direction, type)` pair that crosses the boundary.
///
/// Direction is part of the identity, not a separate axis: `&str` inbound
/// decodes a `jstring` and outbound allocates one, and one may be convertible
/// while the other is not.
pub type Crossing = (Direction, TypeKey);

/// What a conversion is built against: the model, and the conversions already
/// available.
///
/// Two implementors, and the reason there are two is the fill phase.
/// [`Building`] is the partial view a generator sees while it is still
/// producing conversions; [`Registry`] is the total one everything else sees.
/// A helper that serves both — reading a signature off the model, say — takes
/// `&impl Conversions<M>` and works either side of the boundary.
pub trait Conversions<M> {
    /// The model.
    fn flat(&self) -> &crate::api::core::flat::Flat;

    /// The reading for `ty` — what the frontend made of it.
    ///
    /// On the trait because it is needed on **both** sides of the fill: a
    /// converter is chosen for a type while the registry is still being built, and
    /// an emitter asks about the same type afterwards. Both views answer from the
    /// cell the scan filled, so the answer does not change across that line.
    ///
    /// This is what lets a generator take a crossing and reason about it without
    /// rebuilding a spelling from the key and classifying that — the round trip
    /// `api/core` removed from itself in #263, which is the same defect one layer
    /// out.
    fn reading(&self, ty: &syn::Type) -> Option<crate::api::core::flat::TypeRef>;

    /// The conversion for `ty` in `dir`, if there is one.
    fn conversion(&self, dir: Direction, ty: &syn::Type) -> Option<&TypeEntry<M>>;

    /// Wire → rust.
    fn input_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        self.conversion(Direction::Input, ty)
    }

    /// Rust → wire.
    fn output_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        self.conversion(Direction::Output, ty)
    }

    /// The decomposition of a callback argument type, if it has one.
    ///
    /// On the trait because a callback converter needs it while being built,
    /// and the emitter needs it again afterwards. Plans are applied by
    /// `prepare`, so they are complete either side of that line.
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&crate::api::core::unfold::UnfoldPlan>;

    /// Every callback-argument decomposition, for the emitters that enumerate
    /// them rather than look one up.
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, crate::api::core::unfold::UnfoldPlan>;

    /// The return decomposition of a function, if it has one.
    fn unfold_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>;

    /// The error-position decomposition of a fallible function.
    fn error_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>;

    /// The declaration-default decomposition behind each deconstructor.
    fn decon_plans(
        &self,
    ) -> &HashMap<crate::api::core::unfold::DeconId, crate::api::core::unfold::DeconSpec>;

    /// Every type key that crosses in `dir`.
    ///
    /// The niche allocator needs the whole population, not one lookup: it picks
    /// sentinel values no sibling conversion can produce.
    fn crossing_keys(&self, dir: Direction) -> Vec<TypeKey>;

    /// The origin crate's module path for an item, or `None` when unknown.
    fn origin_module(&self, ident: &syn::Ident) -> Option<syn::Path> {
        origin_module_of(self.flat(), ident)
    }

    /// The default module for references with no recorded origin.
    fn default_module(&self) -> Option<syn::Path> {
        default_module_of(self.flat())
    }
}

impl<M> Conversions<M> for Building<'_, M> {
    fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.registry.flat
    }
    fn reading(&self, ty: &syn::Type) -> Option<crate::api::core::flat::TypeRef> {
        self.registry.reading(ty)
    }
    fn conversion(&self, dir: Direction, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        self.built.get(&(dir, TypeKey::from_type(ty)))
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
    fn crossing_keys(&self, dir: Direction) -> Vec<TypeKey> {
        self.all_keys
            .iter()
            .filter(|(d, _)| *d == dir)
            .map(|(_, k)| k.clone())
            .collect()
    }
}

impl<M> Conversions<M> for Registry<M> {
    fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.flat
    }
    fn reading(&self, ty: &syn::Type) -> Option<crate::api::core::flat::TypeRef> {
        Registry::reading(self, ty)
    }
    fn conversion(&self, dir: Direction, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        self.type_table(dir)
            .get(&TypeKey::from_type(ty))?
            .entry
            .as_ref()
    }
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&crate::api::core::unfold::UnfoldPlan> {
        self.callback_arg_plans.get(key)
    }
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, crate::api::core::unfold::UnfoldPlan> {
        &self.callback_arg_plans
    }
    fn unfold_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan> {
        &self.unfold_plans
    }
    fn error_plans(&self) -> &HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan> {
        &self.error_plans
    }
    fn decon_plans(
        &self,
    ) -> &HashMap<crate::api::core::unfold::DeconId, crate::api::core::unfold::DeconSpec> {
        &self.decon_plans
    }
    fn crossing_keys(&self, dir: Direction) -> Vec<TypeKey> {
        self.type_table(dir).keys().cloned().collect()
    }
}

/// The registry mid-fill: the model, plus the conversions supplied so far.
///
/// What a generator builds a conversion *against*. It sees every crossing it
/// can compose from — `RegistryBuilder::crossings` hands them out inner-first, so by
/// the time `Option<Handle>` is asked for, `Handle` is already in here.
///
/// It exposes exactly the reads a conversion needs, which is what keeps the
/// half-filled state from leaking anywhere else: the resolved [`Registry`] is
/// what the emitters get, and it is total.
pub struct Building<'a, M> {
    /// The prepared registry: model, decompositions and the full crossing
    /// population. Its conversion cells are still empty — [`Self::conversion`]
    /// deliberately reads [`Self::built`] instead, so a generator can only see
    /// what it has actually produced.
    registry: &'a Registry<M>,
    built: &'a HashMap<Crossing, TypeEntry<M>>,
    /// Every crossing in the binding, resolved or not — the niche allocator
    /// reads the population, not just what is built so far.
    all_keys: &'a [Crossing],
}

impl<'a, M> Building<'a, M> {
    pub(crate) fn new(
        registry: &'a Registry<M>,
        built: &'a HashMap<Crossing, TypeEntry<M>>,
        all_keys: &'a [Crossing],
    ) -> Self {
        Self {
            registry,
            built,
            all_keys,
        }
    }
}

/// Shared by [`Registry::origin_module`] and [`Building::origin_module`], so the
/// two cannot answer differently.
pub(super) fn origin_module_of(
    flat: &crate::api::core::flat::Flat,
    ident: &syn::Ident,
) -> Option<syn::Path> {
    let crate_name = flat.element(ident)?.location().crate_name.as_ref()?;
    syn::parse_str(&crate_name.replace('-', "_")).ok()
}

pub(super) fn default_module_of(flat: &crate::api::core::flat::Flat) -> Option<syn::Path> {
    flat.source_modules()
        .first()
        .and_then(|m| syn::parse_str(m).ok())
}
