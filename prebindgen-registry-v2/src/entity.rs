//! Which elements of the model are the API.
//!
//! The model is structural: it holds everything the source wrote and the
//! binding stated, a guard and an unsupported item included, and it does not
//! rank them. Ranking is this crate's judgment — a declaration can name a
//! type, a function or a constant, and nothing else — and it is stated here,
//! once, as the view a declaration resolves to.

use prebindgen_flat::flat::{Constant, Element, Flat, Function, Name, Type};

/// One real item of the API: a type, a function or a constant, with its whole
/// description.
///
/// The three kinds a binding can name, and the only three. Where an entity
/// came from is not a kind: a `#[prebindgen]` item and one the binding
/// defines itself — a helper with a stated signature, a type the source never
/// exported that the binding represents as a handle — are the same entity,
/// and differ in their origin alone. The location says which crate, and for
/// the binding's own, the module generated code reaches the item through.
#[derive(Clone, Copy, Debug)]
pub enum Entity<'a> {
    Type(&'a Type),
    Function(&'a Function),
    Constant(&'a Constant),
}

/// Which of the three kinds, on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Type,
    Function,
    Constant,
}

impl<'a> Entity<'a> {
    /// The element as an entity — `None` for a guard or an unsupported item,
    /// which are in the model and not in the API.
    pub fn of(element: &'a Element) -> Option<Self> {
        match element {
            Element::Type(t) => Some(Entity::Type(t)),
            Element::Function(f) => Some(Entity::Function(f)),
            Element::Constant(c) => Some(Entity::Constant(c)),
            Element::Guard(_) | Element::Unsupported(_) => None,
        }
    }

    /// The entity with this name, if the model holds one: captured, or the
    /// binding's own.
    pub fn named<N: Name + ?Sized>(flat: &'a Flat, name: &N) -> Option<Self> {
        Entity::of(flat.element(name)?)
    }

    pub fn kind(&self) -> EntityKind {
        match self {
            Entity::Type(_) => EntityKind::Type,
            Entity::Function(_) => EntityKind::Function,
            Entity::Constant(_) => EntityKind::Constant,
        }
    }

    pub fn name(&self) -> &'a syn::Ident {
        match self {
            Entity::Type(t) => t.name(),
            Entity::Function(f) => &f.name,
            Entity::Constant(c) => &c.name,
        }
    }

    /// The crate stamp on the entity's location: the crate a captured item
    /// came from, and for the binding's own, the module generated code
    /// reaches it through.
    pub fn crate_name(&self) -> Option<&'a str> {
        let location = match self {
            Entity::Type(t) => t.location(),
            Entity::Function(f) => &f.origin.location,
            Entity::Constant(c) => &c.origin.location,
        };
        location.crate_name.as_deref()
    }
}
