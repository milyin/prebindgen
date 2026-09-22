//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one [`Declaration`]
//! per thing the user asked for, inside the
//! [`BindingRequests`](crate::BindingRequests) it hands the engine. The
//! declaration is what the report accounts for, and what the target looks its
//! own configuration up by — the request carries none: the foreign name and
//! the declarator word the report prints come from that lookup.

use prebindgen_flat::flat::{Element, Entity, EntityKind, Flat, TypeKey};
use serde::Serialize;

/// A captured item by name: one of the three kinds the source captures, and
/// what it is called there.
///
/// Less than a [`Declaration`], deliberately. An ignore names such an item and
/// says nothing about how the target would get it, so it cannot be a
/// callback, a conversion or anything the binding coined — those are not
/// captured, and there is nothing in the source to leave alone. A declaration
/// that exposes a captured item as itself *is* one of these, under
/// [`Declaration::Captured`].
pub type CapturedName = Entity<TypeKey, syn::Ident, syn::Ident>;

/// An item the binding defines itself, by name: a function or constant of its
/// own (a path or a Kotlin name, so a string), or a type the target
/// represents although the source never exported it (`String` as an opaque
/// handle, so a type key).
pub type LocalName = Entity<TypeKey, String, String>;

/// One thing a binding asked for: what the target gets, named by what the
/// Rust source calls it, and what the engine plans it from.
///
/// A name alone says neither: `Sample` may be a captured type, a type key the
/// binding coined for something the source never exported, or the name of a
/// Kotlin constant built from an expression. The variant says which, and it
/// says it once — what the target gets and where it comes from (a captured
/// item, looked up in the model, or the binding itself,
/// which [`Self::is_binding_local`] answers) both follow from the variant
/// instead of being stated beside it, so they cannot disagree and a pair that
/// means nothing (a callback backed by a captured constant, a function the
/// binding both defines and selects out of the source) cannot be written
/// down.
///
/// The three kinds the source captures — a type, a function, a constant — are
/// Flat's [`Entity`], and the two variants that name one of them carry it as
/// such: [`Self::Captured`] is a [`CapturedName`], the item exposed as itself,
/// and [`Self::Local`] a [`LocalName`], the binding's own item of that kind.
/// [`Self::ConstFromFunction`] is the one declaration whose kind on the
/// target side differs from its kind in the source; a callback and a
/// conversion have no source kind at all.
///
/// [`generate`](crate::generate) routes on this: each planner is reached by its
/// own variants and is handed the captured item they name, rather than a kind
/// to decode and a name to look up.
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
    /// A captured `#[prebindgen]` item, exposed as what it is: a type given a
    /// foreign representation, a function exported as a foreign function, a
    /// constant exposed as a foreign constant.
    Captured(CapturedName),
    /// A foreign constant read by calling a captured nullary function —
    /// `constant!(X).fun(fun!(f))`. The target renders a constant; the engine
    /// plans the function behind it.
    ConstFromFunction(syn::Ident),
    /// An item the binding defines itself, exposed as what it is: a foreign
    /// function over `fun!(crate::x).sig(..)`, a foreign constant the binding
    /// computes with `constant!(X).expr(..)`, or a type the target represents
    /// although the source never exported it. Nothing in the captured source
    /// names one.
    Local(LocalName),
    /// A callback signature the binding exports as a foreign callable. No
    /// captured item names one, so the signature is the name it goes by.
    Callback(String),
    /// A declared conversion between a Rust type and its wire form, defined by
    /// the binding.
    Conversion(TypeKey),
}

impl From<CapturedName> for Declaration {
    /// The declaration that exposes a captured item as itself — what a report
    /// row and a duplicate check compare an ignore by.
    fn from(name: CapturedName) -> Self {
        Declaration::Captured(name)
    }
}

impl Declaration {
    /// Whether this declares a type — captured or the binding's own.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Declaration::Captured(Entity::Type(_)) | Declaration::Local(Entity::Type(_))
        )
    }

    /// The kind of captured item this declaration must name, if it names one.
    ///
    /// Not always the declaration's own kind: a constant read through a
    /// function names a function, which is what separates
    /// [`Self::ConstFromFunction`] from a captured constant. `None` for what
    /// the binding defines itself.
    pub(crate) fn captured_kind(&self) -> Option<EntityKind> {
        match self {
            Declaration::Captured(name) => Some(name.kind()),
            Declaration::ConstFromFunction(_) => Some(EntityKind::Function),
            Declaration::Local(_) | Declaration::Callback(_) | Declaration::Conversion(_) => None,
        }
    }

    /// The captured element this declaration names, if the model holds it.
    ///
    /// Captured items live in one flat namespace holding functions, types and
    /// constants, so the name alone finds any element; the kind is what says
    /// whether the element found is the one the declaration meant. A
    /// binding-local declaration names no captured item and so finds none —
    /// which is not the same as a missing one, and [`Self::missing_from`] is
    /// the question to ask about presence.
    pub(crate) fn captured<'f>(&self, flat: &'f Flat) -> Option<&'f Element> {
        let wanted = self.captured_kind()?;
        let element = flat.element(&self.name())?;
        (element.entity()?.kind() == wanted).then_some(element)
    }

    /// Whether the binding defines this itself: a callback signature, a
    /// conversion helper, a function or constant of its own, or a type the
    /// target represents although the source never exported it (`String` as
    /// an opaque handle). Such a declaration names no captured item and
    /// requires nothing of the model.
    pub fn is_binding_local(&self) -> bool {
        self.captured_kind().is_none()
    }

    /// Whether the model lacks what this declaration must name.
    ///
    /// Naming the wrong kind — `.fun(fun!(x))` where the source captured
    /// `const x` — counts as missing: it is an error in the binding, and the
    /// engine fails the run over it instead of reporting a skip. False for a
    /// binding-local declaration.
    pub(crate) fn missing_from(&self, flat: &Flat) -> bool {
        !self.is_binding_local() && self.captured(flat).is_none()
    }

    /// The word a refusal uses for what this declaration must name.
    pub(crate) fn describe_captured(&self) -> &'static str {
        match self.captured_kind() {
            Some(EntityKind::Function) => "function",
            Some(EntityKind::Constant) => "constant",
            Some(EntityKind::Type) => "type",
            None => "binding-local item",
        }
    }

    /// The kind the target gets, as the printed prefix spells it — `None` for
    /// a callback and a conversion, which print under words of their own.
    fn target_kind(&self) -> Option<EntityKind> {
        match self {
            Declaration::Captured(name) => Some(name.kind()),
            Declaration::Local(name) => Some(name.kind()),
            Declaration::ConstFromFunction(_) => Some(EntityKind::Constant),
            Declaration::Callback(_) | Declaration::Conversion(_) => None,
        }
    }

    /// The name it goes by — the part of the printed form after the prefix,
    /// and what a captured item is looked up by.
    pub(crate) fn name(&self) -> String {
        match self {
            Declaration::Captured(Entity::Function(ident) | Entity::Constant(ident))
            | Declaration::ConstFromFunction(ident) => ident.to_string(),
            Declaration::Captured(Entity::Type(key))
            | Declaration::Local(Entity::Type(key))
            | Declaration::Conversion(key) => key.as_str().to_string(),
            Declaration::Local(Entity::Function(name) | Entity::Constant(name))
            | Declaration::Callback(name) => name.clone(),
        }
    }
}

impl Ord for Declaration {
    /// As they print. A captured and a binding-local declaration of one kind and
    /// name print alike and are still distinct, so the variant breaks that
    /// tie, and `Ord` agrees with `Eq`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn variant(declaration: &Declaration) -> u8 {
            match declaration {
                Declaration::Captured(_) => 0,
                Declaration::ConstFromFunction(_) => 1,
                Declaration::Local(_) => 2,
                Declaration::Callback(_) => 3,
                Declaration::Conversion(_) => 4,
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
    /// `<what the target gets>:<name>`. The prefix is the foreign side's word —
    /// a function the binding defines and a captured one are both a `fn` —
    /// and it is what keeps the declarations' several naming spaces apart: `type:Foo`
    /// and `conversion:Foo` are two declarations about one Rust type, and
    /// `const:f` and `fn:f` are the `val` read through `f` and `f` itself.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match (self.target_kind(), self) {
            (Some(EntityKind::Function), _) => "fn",
            (Some(EntityKind::Constant), _) => "const",
            (Some(EntityKind::Type), _) => "type",
            (None, Declaration::Callback(_)) => "callback",
            (None, _) => "conversion",
        };
        write!(f, "{prefix}:{}", self.name())
    }
}

impl Serialize for Declaration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
