//! What a conversion is built against — the partial view during the fill, and
//! the total one after it.

use std::collections::HashMap;

use prebindgen::core::flat::{Flat, TypeRef};

use super::*;
use crate::unfold::{DeconId, DeconSpec, UnfoldPlan};

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
    fn flat(&self) -> &Flat;

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
    ///
    /// **Keyed**, because that is the one thing a caller has before it has a
    /// reading. This is the door FROM identity TO the model's answer, and the
    /// only lookup on this trait that does not already take a `TypeRef` — the
    /// rest take one precisely because this exists to hand them one (#284).
    fn reading(&self, key: &TypeKey) -> Option<TypeRef>;

    /// The conversion for `reading` in `dir`, if there is one.
    ///
    /// Takes the **reading**, not a spelling. A caller that has to ask what a
    /// type converts to has already established what the type *is*; asking with
    /// tokens instead let a spelling nobody classified reach the table, and cost
    /// a `TypeKey::from_type` on every call for an identity the reading already
    /// carries.
    fn conversion(&self, dir: Direction, reading: &TypeRef) -> Option<&TypeEntry<M>>;

    /// The reading for a **spelling** — identify, then look up.
    ///
    /// The door for a caller holding tokens it peeled or composed itself, which
    /// is a real position: an adapter may strip a `&` or name a wire type, and
    /// #280 sealed minting so it cannot make a reading for the result. It asks
    /// instead, and `None` means the registry never saw that type.
    ///
    /// Kept separate from the entry lookups on purpose. Those take a `TypeRef`,
    /// so they cannot be called about a type the registry does not know — which
    /// is the guarantee, and it survives only while getting a reading from
    /// tokens is a visible step with a `None` to handle.
    fn reading_of(&self, ty: &syn::Type) -> Option<TypeRef> {
        self.reading(&TypeKey::from_type(ty))
    }

    /// Wire → rust.
    fn input_entry(&self, reading: &TypeRef) -> Option<&TypeEntry<M>> {
        self.conversion(Direction::Input, reading)
    }

    /// Rust → wire.
    fn output_entry(&self, reading: &TypeRef) -> Option<&TypeEntry<M>> {
        self.conversion(Direction::Output, reading)
    }

    /// The decomposition of a callback argument type, if it has one.
    ///
    /// On the trait because a callback converter needs it while being built,
    /// and the emitter needs it again afterwards. Plans are applied by
    /// `prepare`, so they are complete either side of that line.
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&UnfoldPlan>;

    /// Every callback-argument decomposition, for the emitters that enumerate
    /// them rather than look one up.
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, UnfoldPlan>;

    /// The return decomposition of a function, if it has one.
    fn unfold_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan>;

    /// The error-position decomposition of a fallible function.
    fn error_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan>;

    /// The declaration-default decomposition behind each deconstructor.
    fn decon_plans(&self) -> &HashMap<DeconId, DeconSpec>;

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
    fn flat(&self) -> &Flat {
        &self.registry.flat
    }
    fn reading(&self, key: &TypeKey) -> Option<TypeRef> {
        self.registry.reading(key)
    }
    fn conversion(&self, dir: Direction, reading: &TypeRef) -> Option<&TypeEntry<M>> {
        self.built.get(&(dir, reading.key()))
    }
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&UnfoldPlan> {
        self.registry.callback_arg_plans.get(key)
    }
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, UnfoldPlan> {
        &self.registry.callback_arg_plans
    }
    fn unfold_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan> {
        &self.registry.unfold_plans
    }
    fn error_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan> {
        &self.registry.error_plans
    }
    fn decon_plans(&self) -> &HashMap<DeconId, DeconSpec> {
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
    fn flat(&self) -> &Flat {
        &self.flat
    }
    fn reading(&self, key: &TypeKey) -> Option<TypeRef> {
        Registry::reading(self, key)
    }
    fn conversion(&self, dir: Direction, reading: &TypeRef) -> Option<&TypeEntry<M>> {
        self.type_table(dir).get(&reading.key())?.entry.as_ref()
    }
    fn callback_arg_plan(&self, key: &TypeKey) -> Option<&UnfoldPlan> {
        self.callback_arg_plans.get(key)
    }
    fn callback_arg_plans(&self) -> &HashMap<TypeKey, UnfoldPlan> {
        &self.callback_arg_plans
    }
    fn unfold_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan> {
        &self.unfold_plans
    }
    fn error_plans(&self) -> &HashMap<syn::Ident, UnfoldPlan> {
        &self.error_plans
    }
    fn decon_plans(&self) -> &HashMap<DeconId, DeconSpec> {
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
pub(super) fn origin_module_of(flat: &Flat, ident: &syn::Ident) -> Option<syn::Path> {
    let crate_name = flat.element(ident)?.location().crate_name.as_ref()?;
    syn::parse_str(&crate_name.replace('-', "_")).ok()
}

pub(super) fn default_module_of(flat: &Flat) -> Option<syn::Path> {
    flat.source_modules()
        .first()
        .and_then(|m| syn::parse_str(m).ok())
}
