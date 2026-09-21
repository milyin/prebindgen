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
    /// A captured `#[prebindgen]` function, exported as a foreign function.
    Function(syn::Ident),
    /// A foreign function the binding defines itself — `fun!(crate::x).sig(..)`
    /// in the JNI frontend. Nothing in the captured source names it.
    LocalFunction(String),
    /// A captured `#[prebindgen]` constant, exposed as a foreign constant.
    Const(syn::Ident),
    /// A foreign constant read by calling a captured nullary function —
    /// `constant!(X).fun(fun!(f))`. The target renders a constant; the engine
    /// plans the function behind it.
    ConstFromFunction(syn::Ident),
    /// A foreign constant the binding computes rather than reads —
    /// `constant!(X).expr(..)`. No captured item holds its value.
    LocalConst(String),
    /// A captured `#[prebindgen]` type, given a foreign representation.
    Type(TypeKey),
    /// A type the binding gives a representation although the source never
    /// exported it — `String` crossing as an opaque handle.
    LocalType(TypeKey),
    /// A callback signature the binding exports as a foreign callable. No
    /// captured item names one, so the signature is the name it goes by.
    Callback(String),
    /// A declared conversion between a Rust type and its wire form, defined by
    /// the binding.
    Conversion(TypeKey),
}

impl Declaration {
    /// Whether this declares a type — captured or the binding's own.
    pub fn is_type(&self) -> bool {
        matches!(self, Declaration::Type(_) | Declaration::LocalType(_))
    }

    /// The captured element this declaration names, if the model holds it.
    ///
    /// Captured items live in one flat namespace holding functions, types and
    /// constants, so the name alone finds any element; the variant is what says
    /// whether the element found is the one the declaration meant. A constant
    /// read through a function is looked up among the functions, which is what
    /// separates [`Self::ConstFromFunction`] from [`Self::Const`]. A
    /// binding-local declaration names no captured item and so finds none — which
    /// is not the same as a missing one, and [`Self::missing_from`] is the
    /// question to ask about presence.
    pub(crate) fn captured<'f>(&self, flat: &'f Flat) -> Option<&'f Element> {
        let element = flat.element(&self.name())?;
        matches!(
            (self, element),
            (
                Declaration::Function(_) | Declaration::ConstFromFunction(_),
                Element::Function(_)
            ) | (Declaration::Type(_), Element::Type(_))
                | (Declaration::Const(_), Element::Constant(_))
        )
        .then_some(element)
    }

    /// Whether the binding defines this itself: a callback signature, a
    /// binding-local conversion helper, a function or constant of its own, or a
    /// type the target represents although the source never exported it
    /// (`String` as an opaque handle). Such a declaration requires nothing of the
    /// model.
    pub fn is_binding_local(&self) -> bool {
        matches!(
            self,
            Declaration::LocalFunction(_)
                | Declaration::LocalConst(_)
                | Declaration::LocalType(_)
                | Declaration::Callback(_)
                | Declaration::Conversion(_)
        )
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
        match self {
            Declaration::Function(_) | Declaration::ConstFromFunction(_) => "function",
            Declaration::Const(_) => "constant",
            Declaration::Type(_) => "type",
            _ => "binding-local item",
        }
    }

    /// The name it goes by — the part of the printed form after the prefix,
    /// and what a captured item is looked up by.
    pub(crate) fn name(&self) -> String {
        match self {
            Declaration::Function(ident)
            | Declaration::Const(ident)
            | Declaration::ConstFromFunction(ident) => ident.to_string(),
            Declaration::Type(key) | Declaration::LocalType(key) | Declaration::Conversion(key) => {
                key.as_str().to_string()
            }
            Declaration::LocalFunction(name)
            | Declaration::LocalConst(name)
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
                Declaration::Function(_) => 0,
                Declaration::LocalFunction(_) => 1,
                Declaration::Const(_) => 2,
                Declaration::ConstFromFunction(_) => 3,
                Declaration::LocalConst(_) => 4,
                Declaration::Type(_) => 5,
                Declaration::LocalType(_) => 6,
                Declaration::Callback(_) => 7,
                Declaration::Conversion(_) => 8,
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
        let name = self.name();
        match self {
            Declaration::Function(_) | Declaration::LocalFunction(_) => write!(f, "fn:{name}"),
            Declaration::Const(_)
            | Declaration::ConstFromFunction(_)
            | Declaration::LocalConst(_) => {
                write!(f, "const:{name}")
            }
            Declaration::Type(_) | Declaration::LocalType(_) => write!(f, "type:{name}"),
            Declaration::Callback(_) => write!(f, "callback:{name}"),
            Declaration::Conversion(_) => write!(f, "conversion:{name}"),
        }
    }
}

impl Serialize for Declaration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
