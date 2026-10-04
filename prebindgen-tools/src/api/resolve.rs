use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use prebindgen_flat::{flat::TypeRef, TypeKey};

use crate::{ConversionPlan, Direction, Overrides, Place, WireType};

/// A language generator's conversion policy.
///
/// Tools selects an occurrence override or a type default before calling
/// [`Self::resolve`]. With neither, `rule` is `None` and the policy chooses its
/// fallback (often by matching [`shape()`](crate::shape())). An unsupported
/// type or invalid rule must return an error, not silently choose another rule.
/// Children are resolved through the same scope so their overrides apply.
///
/// Policy owns wire encodings and destination metadata. Shared [`Input`](crate::Input),
/// [`Output`](crate::Output), and [`Stage`](crate::Stage) helpers compose the
/// selected children without prescribing a generated element's layout.
pub trait ConversionPolicy: Sized {
    /// The closed vocabulary of supported boundary slots.
    type Wire: WireType;
    /// A declared conversion choice, such as a handle, record, or custom stage.
    type Rule;
    /// Destination-language information for one complete converted value.
    type Metadata;

    /// Build a plan using the selected rule, or the policy's fallback.
    fn resolve(
        &self,
        scope: &Scope<'_, Self>,
        ty: &TypeRef,
        direction: Direction,
        place: &Place,
        rule: Option<&Self::Rule>,
    ) -> Result<ConversionPlan<Self::Wire, Self::Metadata>, String>;
}

/// Type defaults and a generator's conversion policy.
///
/// Build defaults first, then create a [`Scope`] per generated element. Type
/// defaults match exact normalized types and directions; borrowed types do not
/// implicitly inherit owned defaults. A policy can deliberately implement
/// that relation through its fallback. No global conversion graph or shortest
/// path search is involved: the policy selects explicit, directional paths.
pub struct Resolver<P: ConversionPolicy> {
    policy: P,
    defaults: HashMap<(TypeKey, Direction), P::Rule>,
}

impl<P: ConversionPolicy> Resolver<P> {
    /// A policy with no declared defaults.
    pub fn new(policy: P) -> Self {
        Self {
            policy,
            defaults: HashMap::new(),
        }
    }

    /// Register an exact type's default for one direction.
    ///
    /// Duplicate declarations are errors and leave the first declaration intact.
    pub fn default(
        &mut self,
        ty: &TypeRef,
        direction: Direction,
        rule: P::Rule,
    ) -> Result<(), String> {
        let key = (ty.key(), direction);
        if self.defaults.contains_key(&key) {
            return Err(format!("duplicate default for `{ty}` ({direction:?})"));
        }
        self.defaults.insert(key, rule);
        Ok(())
    }

    /// Apply one generated element's overrides before resolving its parts.
    ///
    /// `element` is generator-assigned, with no required flat counterpart.
    /// Overrides for a different element are rejected immediately. Call
    /// [`Scope::finish`] after resolving all parts to reject unused overrides.
    pub fn scope(
        &self,
        element: impl Into<String>,
        overrides: Overrides<P::Rule>,
    ) -> Result<Scope<'_, P>, String> {
        let element = Place::new(element);
        for (place, _) in overrides.places.keys() {
            if place.element != element.element {
                return Err(format!(
                    "override for `{}` belongs outside `{}`",
                    place.element, element.element
                ));
            }
        }
        Ok(Scope {
            resolver: self,
            element,
            overrides,
            active: RefCell::new(Vec::new()),
            used: RefCell::new(HashSet::new()),
        })
    }
}

/// Resolution context for a single generated element.
///
/// Selection order is occurrence override → exact type default → policy
/// fallback. Resolution checks root type/direction and detects recursive
/// conversion cycles before they overflow the stack. This iteration rejects
/// a repeated type/direction even at a deeper place; recursive layouts that
/// need forward declarations are deferred.
pub struct Scope<'r, P: ConversionPolicy> {
    resolver: &'r Resolver<P>,
    element: Place,
    overrides: Overrides<P::Rule>,
    active: RefCell<Vec<(TypeKey, Direction)>>,
    used: RefCell<HashSet<(Place, Direction)>>,
}

impl<P: ConversionPolicy> Scope<'_, P> {
    /// The root place for this generated element.
    pub fn element(&self) -> &Place {
        &self.element
    }

    /// Resolve one occurrence, including nested occurrences requested by policy.
    pub fn resolve(
        &self,
        ty: &TypeRef,
        direction: Direction,
        place: &Place,
    ) -> Result<ConversionPlan<P::Wire, P::Metadata>, String> {
        if place.element != self.element.element {
            return Err(format!(
                "place for `{}` belongs outside `{}`",
                place.element, self.element.element
            ));
        }
        let at = (place.clone(), direction);
        let rule = if let Some((expected, rule)) = self.overrides.places.get(&at) {
            if *expected != ty.key() {
                return Err(format!(
                    "override at {place:?} expects `{expected}`, encountered `{ty}`"
                ));
            }
            Some(rule)
        } else {
            self.resolver.defaults.get(&(ty.key(), direction))
        };
        let key = (ty.key(), direction);
        if self.active.borrow().contains(&key) {
            return Err(format!(
                "conversion cycle for `{ty}` ({direction:?}) at {place:?}"
            ));
        }
        self.active.borrow_mut().push(key);
        // Do not hold a RefCell borrow while calling policy; children use this
        // same scope. The guard removes the active key on every exit path.
        let _active = Active(&self.active);
        let plan = self
            .resolver
            .policy
            .resolve(self, ty, direction, place, rule)?;
        if plan.source_type().key() != ty.key() || plan.direction() != direction {
            return Err(format!(
                "policy returned a plan for `{}` ({:?}), requested `{ty}` ({direction:?})",
                plan.source_type(),
                plan.direction()
            ));
        }
        if self.overrides.places.contains_key(&at) {
            self.used.borrow_mut().insert(at);
        }
        Ok(plan)
    }

    /// Finish this element and reject overrides whose parts were never resolved.
    ///
    /// Consume the scope so subsequent resolutions cannot bypass this check.
    pub fn finish(self) -> Result<(), String> {
        let mut unused: Vec<_> = self
            .overrides
            .places
            .keys()
            .filter(|key| !self.used.borrow().contains(*key))
            .map(|(place, dir)| format!("{place:?} ({dir:?})"))
            .collect();
        unused.sort();
        if unused.is_empty() {
            Ok(())
        } else {
            Err(format!("unused overrides: {}", unused.join(", ")))
        }
    }
}

struct Active<'a>(&'a RefCell<Vec<(TypeKey, Direction)>>);

impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}
