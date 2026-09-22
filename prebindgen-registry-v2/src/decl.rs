//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one [`Declaration`]
//! per thing the user asked for, inside the
//! [`BindingRequests`](crate::BindingRequests) it hands the engine. The
//! declaration is what the report accounts for, and what the target looks its
//! own configuration up by — the request carries none: the foreign name and
//! the declarator word the report prints come from that lookup.

use prebindgen_flat::flat::{Element, Flat, TypeKey};
use serde::Serialize;

use crate::entity::{Entity, EntityKind};

/// An entity by name: which of the three kinds, and what it is called.
///
/// What an ignore names, and what a declaration resolves to when it names an
/// entity at all. Less than a [`Declaration`]: it says nothing about how the
/// target would get the item, so it cannot be a callback or a constant the
/// binding computes on the foreign side — those name no entity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntityName {
    Type(TypeKey),
    Function(syn::Ident),
    Constant(syn::Ident),
}

impl EntityName {
    pub fn kind(&self) -> EntityKind {
        match self {
            EntityName::Type(_) => EntityKind::Type,
            EntityName::Function(_) => EntityKind::Function,
            EntityName::Constant(_) => EntityKind::Constant,
        }
    }

    /// The name as the model indexes it.
    pub fn name(&self) -> String {
        match self {
            EntityName::Type(key) => key.as_str().to_string(),
            EntityName::Function(ident) | EntityName::Constant(ident) => ident.to_string(),
        }
    }
}

impl From<EntityName> for Declaration {
    /// The declaration that exposes the entity as what it is — what a report
    /// row and a duplicate check compare an ignore by.
    fn from(name: EntityName) -> Self {
        match name {
            EntityName::Type(key) => Declaration::Type(key),
            EntityName::Function(ident) => Declaration::Function(ident),
            EntityName::Constant(ident) => Declaration::Const(ident),
        }
    }
}

/// One thing a binding asked for: what the target gets, named by what the
/// Rust source calls it, and what the engine plans it from.
///
/// A name alone says neither: `f` may be a function exported as itself or a
/// Kotlin `val` read through it, and `Foo` a type given a representation or a
/// wire mapping declared for it. The variant says which, and it says it once
/// — what the target gets and which entity, if any, it is planned from both
/// follow from the variant instead of being stated beside it, so they cannot
/// disagree and a pair that means nothing (a callback backed by a captured
/// constant) cannot be written down.
///
/// Where an entity came from is not part of this. A `#[prebindgen]` function
/// and one the binding defines itself are both a [`Declaration::Function`];
/// the model holds both, and only its [`Origin`](prebindgen_flat::flat::Origin)
/// tells them apart. Two declarations name no entity at all, because nothing
/// in Rust backs them: a callback signature, and a constant the binding
/// computes on the foreign side.
///
/// [`generate`](crate::generate) routes on this: each planner is reached by its
/// own variants and is handed the entity they name, rather than a kind to
/// decode and a name to look up.
///
/// It is also the declaration's identity — what an outcome is keyed by, what a
/// [`Position`](crate::target::Position) is rooted at, what a target's
/// [`Requirement`](crate::target::Requirement) resolves to. Stable across runs
/// and across pipelines, so a report, a build script and a capability-selected
/// test section can all name the same declaration: it is what the *source*
/// calls the thing, not what the target does, and a rename on the foreign side
/// must not silently retire a test's requirement. `<kind>:<name>` —
/// `type:Stamp`, `fn:stamp_sum` — is how it prints and how the report writes
/// it, and that spelling is a rendering: nothing reads a declaration back out
/// of it. Declarations order as they print, so a report sorted by declaration
/// reads in id order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// A function, exported as a foreign function.
    Function(syn::Ident),
    /// A foreign constant read by calling a nullary function —
    /// `constant!(X).fun(fun!(f))`. The target renders a constant; the engine
    /// plans the function behind it.
    ConstFromFunction(syn::Ident),
    /// A constant, exposed as a foreign constant.
    Const(syn::Ident),
    /// A type, given a foreign representation.
    Type(TypeKey),
    /// A declared conversion between a type and its wire form, defined by the
    /// binding.
    Conversion(TypeKey),
    /// A callback signature the binding exports as a foreign callable. No
    /// entity backs one, so the signature is the name it goes by.
    Callback(String),
    /// A foreign constant the binding computes rather than reads —
    /// `constant!(X).expr(..)`. No entity holds its value.
    ComputedConst(String),
}

impl Declaration {
    /// Whether this declares a type.
    pub fn is_type(&self) -> bool {
        matches!(self, Declaration::Type(_))
    }

    /// The entity this declaration names, if it names one.
    ///
    /// Not always the declaration's own kind: a constant read through a
    /// function names a function, which is what separates
    /// [`Self::ConstFromFunction`] from [`Self::Const`]. `None` for a callback
    /// and a computed constant, which nothing in Rust backs.
    pub fn entity(&self) -> Option<EntityName> {
        match self {
            Declaration::Function(ident) | Declaration::ConstFromFunction(ident) => {
                Some(EntityName::Function(ident.clone()))
            }
            Declaration::Const(ident) => Some(EntityName::Constant(ident.clone())),
            Declaration::Type(key) | Declaration::Conversion(key) => {
                Some(EntityName::Type(key.clone()))
            }
            Declaration::Callback(_) | Declaration::ComputedConst(_) => None,
        }
    }

    /// The element this declaration names, if the model holds it.
    ///
    /// Entities live in one flat namespace holding functions, types and
    /// constants, so the name alone finds any element; the kind is what says
    /// whether the element found is the one the declaration meant. A
    /// declaration naming no entity finds none — which is not the same as a
    /// missing one, and [`Self::missing_from`] is the question to ask about
    /// presence.
    pub(crate) fn captured<'f>(&self, flat: &'f Flat) -> Option<&'f Element> {
        let wanted = self.entity()?;
        let element = flat.element(&wanted.name())?;
        (Entity::of(element)?.kind() == wanted.kind()).then_some(element)
    }

    /// Whether the model lacks what this declaration must name.
    ///
    /// Naming the wrong kind — `.fun(fun!(x))` where the source captured
    /// `const x` — counts as missing: it is an error in the binding, and the
    /// engine fails the run over it instead of reporting a skip. False for a
    /// declaration that names no entity.
    pub(crate) fn missing_from(&self, flat: &Flat) -> bool {
        self.entity().is_some() && self.captured(flat).is_none()
    }

    /// The word a refusal uses for what this declaration must name.
    pub(crate) fn describe_captured(&self) -> &'static str {
        match self.entity().map(|entity| entity.kind()) {
            Some(EntityKind::Function) => "function",
            Some(EntityKind::Constant) => "constant",
            Some(EntityKind::Type) => "type",
            None => "item",
        }
    }

    /// The name it goes by — the part of the printed form after the prefix,
    /// and what an entity is looked up by.
    pub(crate) fn name(&self) -> String {
        match self {
            Declaration::Function(ident)
            | Declaration::ConstFromFunction(ident)
            | Declaration::Const(ident) => ident.to_string(),
            Declaration::Type(key) | Declaration::Conversion(key) => key.as_str().to_string(),
            Declaration::Callback(name) | Declaration::ComputedConst(name) => name.clone(),
        }
    }
}

impl Ord for Declaration {
    /// As they print. Two declarations of one name print alike and are still
    /// distinct — `const:f` read through `f` and a captured `const f` — so the
    /// variant breaks that tie, and `Ord` agrees with `Eq`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn variant(declaration: &Declaration) -> u8 {
            match declaration {
                Declaration::Function(_) => 0,
                Declaration::ConstFromFunction(_) => 1,
                Declaration::Const(_) => 2,
                Declaration::Type(_) => 3,
                Declaration::Conversion(_) => 4,
                Declaration::Callback(_) => 5,
                Declaration::ComputedConst(_) => 6,
            }
        }
        self.to_string()
            .cmp(&other.to_string())
            .then_with(|| variant(self).cmp(&variant(other)))
    }
}

impl PartialOrd for Declaration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Declaration {
    /// `<what the target gets>:<name>`. The prefix is the foreign side's word,
    /// and it is what keeps the declarations' several naming spaces apart:
    /// `type:Foo` and `conversion:Foo` are two declarations about one Rust
    /// type, and `const:f` and `fn:f` are the `val` read through `f` and `f`
    /// itself.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self {
            Declaration::Function(_) => "fn",
            Declaration::ConstFromFunction(_)
            | Declaration::Const(_)
            | Declaration::ComputedConst(_) => "const",
            Declaration::Type(_) => "type",
            Declaration::Conversion(_) => "conversion",
            Declaration::Callback(_) => "callback",
        };
        write!(f, "{prefix}:{}", self.name())
    }
}

impl Serialize for Declaration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
