//! Preparation-only representation declarations and defaults.
//! Resolve choices and copy required metadata into slots before generation.
use crate::ComponentPath;
use prebindgen_flat::{type_key, Conversion, Flat, Result};
use std::collections::BTreeMap;

/// Identity used only while resolving preparation declarations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepresentationId(pub String);

/// A logical component in a representation, not an executable conversion.
/// Every component must have at least one wire when a conversion selects this
/// representation. Partial projections must select their own representation.
#[derive(Clone, Debug)]
pub struct Component {
    /// Stable path used by conversion wires.
    pub path: ComponentPath,
    /// Public presentation name, separate from generated Rust local names.
    pub name: String,
    /// Optional source record field path. Derived views need no field association.
    pub source_fields: Option<Vec<String>>,
    /// Representation of the component, when it has a separately declared shape.
    pub representation: Option<RepresentationId>,
}

/// Shape declaration used during preparation, never referenced by a conversion slot.
#[derive(Clone, Debug)]
pub struct Representation<R> {
    /// Qualified Rust type represented by this shape.
    pub source_type: syn::Type,
    /// Logical components; declaration order is retained.
    pub components: Vec<Component>,
    /// Adapter-specific type declaration and materialization information.
    pub data: R,
}

/// Representation registry and optional canonical defaults by Rust type.
#[derive(Clone, Debug)]
pub struct Representations<R> {
    entries: BTreeMap<RepresentationId, Representation<R>>,
    canonical: BTreeMap<String, RepresentationId>,
}
impl<R> Default for Representations<R> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            canonical: BTreeMap::new(),
        }
    }
}
impl<R> Representations<R> {
    /// Insert or replace a shape. Reject duplicate component identities atomically.
    pub fn insert(&mut self, id: RepresentationId, shape: Representation<R>) -> Result<()> {
        let mut paths = std::collections::BTreeSet::new();
        for component in &shape.components {
            if !paths.insert(&component.path) {
                return Err(format!("duplicate component in representation `{}`", id.0));
            }
        }
        self.entries.insert(id, shape);
        Ok(())
    }
    /// Resolve a representation without interpreting its adapter data.
    pub fn get(&self, id: &RepresentationId) -> Result<&Representation<R>> {
        self.entries
            .get(id)
            .ok_or_else(|| format!("unknown representation `{}`", id.0))
    }
    /// Iterate declarations in identity order during preparation.
    pub fn iter(&self) -> impl Iterator<Item = (&RepresentationId, &Representation<R>)> {
        self.entries.iter()
    }
    /// Designate an existing representation as its source type's default.
    /// This does not populate or change any occurrence conversion.
    pub fn set_canonical(&mut self, id: &RepresentationId) -> Result<()> {
        let key = type_key(&self.get(id)?.source_type);
        self.canonical.insert(key, id.clone());
        Ok(())
    }
    /// Look up a preparation default. Copy the selected data into prepared slots.
    pub fn canonical(&self, ty: &syn::Type) -> Option<&RepresentationId> {
        self.canonical.get(&type_key(ty))
    }
    /// Validate nested references after all declarations have been registered.
    fn validate_references(&self) -> Result<()> {
        for shape in self.entries.values() {
            for component in &shape.components {
                if let Some(id) = &component.representation {
                    self.get(id)?;
                }
            }
        }
        for (ty, id) in &self.canonical {
            if type_key(&self.get(id)?.source_type) != *ty {
                return Err(format!(
                    "canonical representation `{}` changed source type",
                    id.0
                ));
            }
        }
        Ok(())
    }
}

impl<R> Representations<R> {
    /// Validate all registered type shapes, including schemas not used by a function.
    pub fn validate<I: Conversion, O: Conversion>(&self, flat: &Flat<I, O>) -> Result<()> {
        self.validate_references()?;
        for (id, shape) in self.iter() {
            flat.resolve_type(&shape.source_type)?;
            for component in &shape.components {
                if let Some(fields) = &component.source_fields {
                    let mut ty = &shape.source_type;
                    for field in fields {
                        ty = &flat
                            .record(ty)?
                            .fields
                            .iter()
                            .find(|f| f.name == field.as_str())
                            .ok_or_else(|| {
                                format!(
                                    "unknown source field `{field}` in representation `{}`",
                                    id.0
                                )
                            })?
                            .ty;
                    }
                    if let Some(child) = &component.representation {
                        if !prebindgen_flat::same_type(ty, &self.get(child)?.source_type) {
                            return Err(format!(
                                "component `{}` has an incompatible source type",
                                component.name
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Validate an occurrence mapping during preparation, before installing its slot.
    /// Partial projections require a matching declaration; several wires may share a component.
    pub fn validate_mapping<'a>(
        &self,
        ty: &syn::Type,
        id: &RepresentationId,
        mapping: impl IntoIterator<Item = (&'a syn::Ident, &'a ComponentPath)>,
    ) -> Result<()> {
        let wires: Vec<_> = mapping.into_iter().collect();
        let shape = self.get(id)?;
        if !prebindgen_flat::same_type(ty, &shape.source_type) {
            return Err(format!(
                "representation `{}` does not match occurrence type",
                id.0
            ));
        }
        for (name, path) in &wires {
            if !shape.components.iter().any(|c| c.path == **path) {
                return Err(format!(
                    "wire `{}` has no component in representation `{}`",
                    name, id.0
                ));
            }
        }
        for component in &shape.components {
            if !wires.iter().any(|(_, path)| **path == component.path) {
                return Err(format!(
                    "component `{}` has no wire in representation `{}`",
                    component.name, id.0
                ));
            }
        }
        Ok(())
    }
}
