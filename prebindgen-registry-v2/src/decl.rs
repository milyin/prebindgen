//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one [`Declaration`]
//! per thing the user asked for, in the list it hands [`generate`](crate::generate).
//! The declaration is what the report accounts for, and what the target looks
//! its own configuration up by — the declaration carries none: the foreign
//! name and the declarator word the report prints come from that lookup.

use prebindgen_flat::flat::{Element, Flat, TypeKey};
use serde::Serialize;

/// One thing a binding asked for: what the target gets, named by what the
/// Rust source calls it, and what the engine plans it from.
///
/// A name alone says neither: `Foo` may be a type given a representation or a
/// wire mapping declared for it. The variant says which, and it says it once
/// — what the target gets and which entity, if any, it is planned from both
/// follow from the variant instead of being stated beside it, so they cannot
/// disagree and a pair that means nothing (a callback backed by a captured
/// constant) cannot be written down. What a declaration does *not* say is how
/// the target shows the entity: a function exposed as a Kotlin `val` read
/// through it is still [`Declaration::Function`], and the `val` is the
/// target's choice, recorded under that declaration.
///
/// An **entity** is one real item of the API — a type, a function or a
/// constant — and the three variants that name one are the registry's whole
/// judgment of which elements those are: the model holds a guard and an
/// unsupported item beside them and ranks none of it. Where an entity came
/// from is not part of this either. A `#[prebindgen]` function and one the
/// binding defines itself are both a [`Declaration::Function`]; the model
/// holds both, and only its [`Origin`](prebindgen_flat::flat::Origin) tells
/// them apart. Three declarations name no entity: a callback signature and a
/// constant the binding computes on the foreign side, because nothing in Rust
/// backs them; and a conversion, which is the binding's own wire mapping
/// *about* a type — `convert!(Option<Payload>)` — and requires no item of that
/// name in the model.
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
///
/// # One entity, several projections
///
/// A binding may expose one entity more than once — `stamp_sum` as a Kotlin
/// `fun` in one package and as a `val` read through it, `Stamp` as a data
/// class and as a handle. Each is its own declaration, with its own choices,
/// outcome and report row, and what tells them apart is the **projection**:
/// a label the frontend supplies, in the target's own terms (a placement's
/// fully qualified name, a symbol), which the engine compares and never
/// reads. It prints after an `@`: `fn:stamp_sum@io.zenoh.jni.Other.stampSum`.
/// A frontend labels an entity's projections only when there is more than
/// one, so the common single projection is `fn:stamp_sum` as it always was,
/// and two declarations with the same entity and label are a duplicate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// A function, exported through a wrapper that calls it — as a foreign
    /// function, or as whatever else the target chooses to show a call as: a
    /// Kotlin `val` over `constant!(X).fun(fun!(f))` is this, with the `val`
    /// the target's business.
    Function {
        name: syn::Ident,
        projection: Option<String>,
    },
    /// A constant, exposed as a foreign constant.
    Const {
        name: syn::Ident,
        projection: Option<String>,
    },
    /// A type, given a foreign representation.
    Type {
        key: TypeKey,
        projection: Option<String>,
    },
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
    /// A function, as the one projection of it.
    pub fn function(name: syn::Ident) -> Self {
        Declaration::Function {
            name,
            projection: None,
        }
    }

    /// A constant, as the one projection of it.
    pub fn constant(name: syn::Ident) -> Self {
        Declaration::Const {
            name,
            projection: None,
        }
    }

    /// A type, as the one projection of it.
    pub fn declared_type(key: TypeKey) -> Self {
        Declaration::Type {
            key,
            projection: None,
        }
    }

    /// This declaration as one of several projections of its entity, told
    /// apart by `label`. A declaration that names no entity has nothing to
    /// project, and is returned as it is.
    pub fn projected(self, label: impl Into<String>) -> Self {
        let projection = Some(label.into());
        match self {
            Declaration::Function { name, .. } => Declaration::Function { name, projection },
            Declaration::Const { name, .. } => Declaration::Const { name, projection },
            Declaration::Type { key, .. } => Declaration::Type { key, projection },
            other => other,
        }
    }

    /// The label telling this projection from its entity's others, when the
    /// entity has more than one.
    pub fn projection(&self) -> Option<&str> {
        match self {
            Declaration::Function { projection, .. }
            | Declaration::Const { projection, .. }
            | Declaration::Type { projection, .. } => projection.as_deref(),
            Declaration::Conversion(_)
            | Declaration::Callback(_)
            | Declaration::ComputedConst(_) => None,
        }
    }

    /// Whether this declares a type.
    pub fn is_type(&self) -> bool {
        matches!(self, Declaration::Type { .. })
    }

    /// The name the model indexes the entity this declaration names under —
    /// `None` for a callback, a computed constant and a conversion.
    ///
    /// A type key may carry arguments the item does not — `ptr_class!(
    /// Publisher<'static>)` means the item `Publisher` — so the item's name is
    /// the key's last segment without them. A key that is not a path at all
    /// names no item, and is looked up as it is spelled, to be found missing.
    pub fn entity_name(&self) -> Option<String> {
        match self {
            Declaration::Function { name, .. } | Declaration::Const { name, .. } => {
                Some(name.to_string())
            }
            Declaration::Type { key, .. } => {
                Some(key.short_name().unwrap_or_else(|| key.as_str().to_string()))
            }
            Declaration::Conversion(_)
            | Declaration::Callback(_)
            | Declaration::ComputedConst(_) => None,
        }
    }

    /// The element this declaration names, if the model holds it.
    ///
    /// Entities live in one flat namespace holding functions, types and
    /// constants, so the name alone finds any element; the variant is what
    /// says whether the element found is the one the declaration meant. A
    /// declaration naming no entity finds none — which is not the same as a
    /// missing one, and [`Self::missing_from`] is the question to ask about
    /// presence.
    pub(crate) fn captured<'f>(&self, flat: &'f Flat) -> Option<&'f Element> {
        let element = flat.element(&self.entity_name()?)?;
        matches!(
            (self, element),
            (Declaration::Function { .. }, Element::Function(_))
                | (Declaration::Const { .. }, Element::Constant(_))
                | (Declaration::Type { .. }, Element::Type(_))
        )
        .then_some(element)
    }

    /// Whether the model lacks what this declaration must name.
    ///
    /// Naming the wrong kind — `.fun(fun!(x))` where the source captured
    /// `const x` — counts as missing: it is an error in the binding, and the
    /// engine fails the run over it instead of reporting a skip. False for a
    /// declaration that names no entity.
    pub(crate) fn missing_from(&self, flat: &Flat) -> bool {
        self.entity_name().is_some() && self.captured(flat).is_none()
    }

    /// The word a refusal uses for what this declaration must name.
    pub(crate) fn describe_captured(&self) -> &'static str {
        match self {
            Declaration::Function { .. } => "function",
            Declaration::Const { .. } => "constant",
            Declaration::Type { .. } => "type",
            Declaration::Conversion(_)
            | Declaration::Callback(_)
            | Declaration::ComputedConst(_) => "item",
        }
    }

    /// The name it goes by — the part of the printed form after the prefix,
    /// and what an entity is looked up by.
    pub(crate) fn name(&self) -> String {
        match self {
            Declaration::Function { name, .. } | Declaration::Const { name, .. } => {
                name.to_string()
            }
            Declaration::Type { key, .. } | Declaration::Conversion(key) => {
                key.as_str().to_string()
            }
            Declaration::Callback(name) | Declaration::ComputedConst(name) => name.clone(),
        }
    }
}

impl Ord for Declaration {
    /// As they print. Two declarations of one name can print alike and still
    /// be distinct — a captured `const X` and a computed `X` are both
    /// `const:X` — so the variant breaks that tie, and `Ord` agrees with `Eq`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn variant(declaration: &Declaration) -> u8 {
            match declaration {
                Declaration::Function { .. } => 0,
                Declaration::Const { .. } => 1,
                Declaration::Type { .. } => 2,
                Declaration::Conversion(_) => 3,
                Declaration::Callback(_) => 4,
                Declaration::ComputedConst(_) => 5,
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
    /// type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self {
            Declaration::Function { .. } => "fn",
            Declaration::Const { .. } | Declaration::ComputedConst(_) => "const",
            Declaration::Type { .. } => "type",
            Declaration::Conversion(_) => "conversion",
            Declaration::Callback(_) => "callback",
        };
        write!(f, "{prefix}:{}", self.name())?;
        if let Some(projection) = self.projection() {
            write!(f, "@{projection}")?;
        }
        Ok(())
    }
}

impl Serialize for Declaration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
