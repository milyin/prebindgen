use std::fmt;

use prebindgen_flat::flat::Field;

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
/// ([`Choices`](crate::Choices)) and name wires ([`Place::ident`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    pub element: String,
    pub path: Vec<Seg>,
}

impl fmt::Display for Place {
    /// `send.p.x`: the element, then each step.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.element)?;
        for seg in &self.path {
            match seg {
                Seg::Param(n) | Seg::Field(n) | Seg::Alt(n) => write!(f, ".{n}")?,
                Seg::Return => write!(f, ".return")?,
                Seg::Arg(i) => write!(f, ".arg{i}")?,
                Seg::Some => write!(f, ".some")?,
                Seg::Elem => write!(f, ".elem")?,
                Seg::Repr => write!(f, ".repr")?,
            }
        }
        Ok(())
    }
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

    /// The name of the wire that carries the value at this place: the
    /// path's segments joined by `_` (`p_x` for field `x` of parameter `p`).
    pub fn ident(&self) -> syn::Ident {
        self.ident_with_suffix("")
    }

    /// The name of an extra wire at this place that does not carry the value
    /// itself, such as an option's presence flag or a sum's tag: the
    /// [`ident`](Self::ident) name with `_` and `suffix` appended
    /// (`p_x_present`).
    pub fn ident_with_suffix(&self, suffix: &str) -> syn::Ident {
        let words = self.path.iter().filter_map(Seg::word);
        let name = words
            .chain((!suffix.is_empty()).then(|| suffix.to_string()))
            .collect::<Vec<_>>()
            .join("_");
        names::ident(if name.is_empty() { "value" } else { &name })
    }
}
