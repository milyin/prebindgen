use std::collections::HashMap;

use prebindgen_flat::{
    flat::{Field, TypeRef},
    TypeKey,
};

use crate::{names, Direction};

/// One step from a generated element to a typed occurrence within it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Seg {
    /// Distinguish source elements when one generated element groups several.
    Source(String),
    /// A function parameter, by name.
    Param(String),
    /// A function's result.
    Return,
    /// A callback argument, by position.
    Arg(usize),
    /// A named field, or a tuple field's decimal position.
    Field(String),
    /// A sum alternative, by name.
    Alt(String),
    /// The value inside an option.
    Some,
    /// One sequence element.
    Elem,
    /// An intermediate representation type.
    Repr,
    /// A generator-specific part, such as a constructor or an error channel.
    Custom(String),
}

impl Seg {
    /// Address a field in the flat model.
    pub fn field(f: &Field) -> Self {
        Self::Field(
            f.name
                .as_ref()
                .map(names::bare)
                .unwrap_or_else(|| f.index.to_string()),
        )
    }

    fn word(&self) -> Option<String> {
        match self {
            Self::Source(n) | Self::Param(n) | Self::Custom(n) => Some(n.clone()),
            Self::Return => Some("ret".into()),
            Self::Arg(i) => Some(format!("arg{i}")),
            Self::Field(n) if n.starts_with(|c: char| c.is_ascii_digit()) => Some(format!("v{n}")),
            Self::Field(n) => Some(n.clone()),
            Self::Alt(n) => Some(names::snake(n)),
            Self::Elem => Some("elem".into()),
            Self::Some | Self::Repr => None,
        }
    }
}

/// The generated element's identity and a path to one of its parts.
///
/// The identity belongs to the generator. It need not name a flat element:
/// a helper can reference none, and a grouped wrapper can reference several
/// through [`Seg::Source`]. The path describes occurrences, not Rust item kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    /// Generator-assigned element identity.
    pub element: String,
    /// Path relative to that element.
    pub path: Vec<Seg>,
}

impl Place {
    /// The generated element itself.
    pub fn new(element: impl Into<String>) -> Self {
        Self {
            element: element.into(),
            path: Vec::new(),
        }
    }

    /// Extend the occurrence path without modifying its parent.
    pub fn at(&self, seg: Seg) -> Self {
        let mut p = self.clone();
        p.path.push(seg);
        p
    }

    /// A readable candidate name for a wire or local.
    ///
    /// This is not a uniqueness allocator: underscore-joined paths can collide,
    /// and `Some`/`Repr` add no word. A [`ConversionPlan`](crate::ConversionPlan)
    /// rejects duplicate wires within a plan. The element writer must also avoid
    /// collisions between plans and with its own locals.
    pub fn ident(&self, suffix: &str) -> syn::Ident {
        let name = self
            .path
            .iter()
            .filter_map(Seg::word)
            .chain((!suffix.is_empty()).then(|| suffix.to_string()))
            .collect::<Vec<_>>()
            .join("_");
        names::ident(if name.is_empty() { "value" } else { &name })
    }
}

/// Explicit conversion rules for occurrences in one generated element.
///
/// A rule replaces the default for the exact occurrence, type, and direction.
/// It is not inherited by descendants. Register child overrides at their own
/// paths. [`Scope`](crate::Scope) checks expected types when resolving and can
/// reject unused overrides with [`Scope::finish`](crate::Scope::finish).
#[derive(Debug)]
pub struct Overrides<R> {
    pub(super) places: HashMap<(Place, Direction), (TypeKey, R)>,
}

impl<R> Default for Overrides<R> {
    fn default() -> Self {
        Self {
            places: HashMap::new(),
        }
    }
}

impl<R> Overrides<R> {
    /// No occurrence overrides.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override an occurrence of `ty` in one direction.
    ///
    /// Duplicate declarations are refused; the original declaration is kept.
    pub fn at(
        &mut self,
        place: Place,
        ty: &TypeRef,
        direction: Direction,
        rule: R,
    ) -> Result<(), String> {
        let key = (place, direction);
        if self.places.contains_key(&key) {
            return Err(format!("duplicate override at {:?} ({direction:?})", key.0));
        }
        self.places.insert(key, (ty.key(), rule));
        Ok(())
    }
}
