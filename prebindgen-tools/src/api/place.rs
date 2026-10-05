use std::collections::HashMap;

use prebindgen_flat::{
    flat::{Field, TypeRef},
    TypeKey,
};

use crate::names;

/// One step from a value to a part of it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Seg {
    /// A function parameter, by name.
    Param(String),
    /// A function's result.
    Return,
    /// A callback argument, by position.
    Arg(usize),
    /// A record field: its name, or its position for a tuple field.
    Field(String),
    /// A sum alternative, by name.
    Alt(String),
    /// The value inside an `Option`.
    Some,
    /// One element of a sequence.
    Elem,
    /// The representation a converted type crosses as.
    Repr,
}

impl Seg {
    /// The segment addressing `f`.
    pub fn field(f: &Field) -> Self {
        Seg::Field(match &f.name {
            Some(n) => names::bare(n),
            None => f.index.to_string(),
        })
    }

    /// This segment in a wire name; `None` when it adds nothing — the inside
    /// of an `Option`, a conversion's representation.
    fn word(&self) -> Option<String> {
        match self {
            Seg::Param(n) => Some(n.clone()),
            Seg::Return => Some("ret".into()),
            Seg::Arg(i) => Some(format!("arg{i}")),
            Seg::Field(n) if n.starts_with(|c: char| c.is_ascii_digit()) => Some(format!("v{n}")),
            Seg::Field(n) => Some(n.clone()),
            Seg::Alt(n) => Some(names::snake(n)),
            Seg::Elem => Some("elem".into()),
            Seg::Some | Seg::Repr => None,
        }
    }
}

/// Where a type occurs: the generated element and the path to the part.
///
/// `Place::new("send").at(Seg::Param("p".into())).at(Seg::Field("x".into()))`
/// is field `x` of parameter `p` of `send`. Places key overrides
/// ([`Overrides`]) and name wires ([`Place::ident`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    pub element: String,
    pub path: Vec<Seg>,
}

impl Place {
    /// The element itself.
    pub fn new(element: impl Into<String>) -> Self {
        Self {
            element: element.into(),
            path: Vec::new(),
        }
    }

    /// A part of this place.
    pub fn at(&self, seg: Seg) -> Self {
        let mut p = self.clone();
        p.path.push(seg);
        p
    }

    /// A wire name for this place: the path's segments joined by `_`, then
    /// `suffix` (`p_x`, `p_x_present`).
    pub fn ident(&self, suffix: &str) -> syn::Ident {
        let words = self.path.iter().filter_map(Seg::word);
        let name = words
            .chain((!suffix.is_empty()).then(|| suffix.to_string()))
            .collect::<Vec<_>>()
            .join("_");
        names::ident(if name.is_empty() { "value" } else { &name })
    }
}

/// The adapter's decisions: one per type, replaced at chosen places.
///
/// `R` is the adapter's own declaration — "a handle", "a record", "converted
/// through these functions". An element's writer asks with the place of the
/// part it is building, so a per-parameter override wins over the type's
/// default without the writer knowing which one applied.
pub struct Overrides<R> {
    types: HashMap<TypeKey, R>,
    places: HashMap<Place, R>,
}

impl<R> Default for Overrides<R> {
    fn default() -> Self {
        Self {
            types: HashMap::new(),
            places: HashMap::new(),
        }
    }
}

impl<R> Overrides<R> {
    pub fn new() -> Self {
        Self::default()
    }

    /// The default for every occurrence of `ty`.
    pub fn ty(&mut self, ty: &TypeRef, decl: R) {
        self.types.insert(ty.key(), decl);
    }

    /// The decision for whatever type sits at `place`, replacing its default.
    pub fn at(&mut self, place: Place, decl: R) {
        self.places.insert(place, decl);
    }

    /// The decision for `ty` at `place`: the place's override, else the
    /// type's default. Shaped to be [`shape()`](crate::shape())'s callback.
    pub fn get(&self, place: &Place, ty: &TypeRef) -> Option<&R> {
        self.places.get(place).or_else(|| self.types.get(&ty.key()))
    }
}
