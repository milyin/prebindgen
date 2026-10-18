//! Named representation schemas shared by type declarations and conversions.
use crate::{type_key, Result};
use std::collections::BTreeMap;

/// Stable identity of a resolved representation; independent of generated names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepresentationId(pub String);

/// Typed step identifying a logical component independently of generated names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentSegment {
    /// A source record field.
    Field(String),
    /// A constructor argument.
    Argument(String),
    /// An input selector arm.
    Alternative(usize),
    /// One simultaneous output view.
    View(usize),
    /// A source getter's result.
    Getter(String),
    /// An input selector's discriminator.
    Selector,
    /// An adapter-defined logical component with no implied source projection.
    Named(String),
}
impl ComponentSegment {
    /// Default presentation name; applications may assign a different name in the schema.
    pub fn name(&self) -> String {
        match self {
            Self::Field(name) | Self::Argument(name) | Self::Getter(name) | Self::Named(name) => {
                name.clone()
            }
            Self::Alternative(index) | Self::View(index) => index.to_string(),
            Self::Selector => "selector".into(),
        }
    }
}

/// Logical component identity, including nesting. Several wires may implement it.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentPath(pub Vec<ComponentSegment>);
impl ComponentPath {
    /// Prefix a component with a containing field, view, or alternative.
    pub fn prepend(&mut self, segment: ComponentSegment) {
        self.0.insert(0, segment);
    }
}

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

/// Resolved shape used to generate a destination type or describe an occurrence.
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
    /// Iterate in identity order, for generation of selected type declarations.
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
    /// Look up a preparation default; generation uses explicit occurrence references.
    pub fn canonical(&self, ty: &syn::Type) -> Option<&RepresentationId> {
        self.canonical.get(&type_key(ty))
    }
    /// Validate nested references after all declarations have been registered.
    pub fn validate(&self) -> Result<()> {
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
