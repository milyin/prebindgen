//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one [`Origin`]
//! per thing the user asked for, inside the
//! [`BindingRequests`](crate::BindingRequests) it hands the engine. The origin
//! is what the report accounts for; the request carries the target's
//! configuration for it, and that policy is also where the foreign name and
//! the declarator word the report prints come from.

use prebindgen_flat::flat::{Element, Flat, TypeKey};
use serde::Serialize;

/// What a declaration is named after, and what the engine plans it from.
///
/// A name alone says neither: `Sample` may be a captured type, a type key the
/// binding coined for something the source never exported, or the name of a
/// Kotlin constant built from an expression. The variant says which, and it
/// says it once — what the target gets ([`Self::kind`]) and what the name must
/// find in the captured source both follow from the variant instead of being
/// stated beside it, so they cannot disagree and a pair that means nothing (a
/// callback backed by a captured constant, a function the binding both defines
/// and selects out of the source) cannot be written down.
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
/// it, and that spelling is a rendering: nothing reads an origin back out of
/// it. Origins order by kind — types first, then conversions, callbacks,
/// constants and functions, which is how a report reads — and then by name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Origin {
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

impl Origin {
    /// What the target gets: `fn`, `type`, `const`, `callback` or
    /// `conversion` — the prefix the origin prints with, the report's `kind`
    /// column, and the middle of an `unsupported.<kind>.…` code.
    ///
    /// The coarse, language-neutral category — the five things any binding can
    /// be made of. Deliberately coarser than an adapter's own vocabulary:
    /// `prebindgen-c` declares a type with `opaque_ptr` or `data_struct`, and
    /// `prebindgen-jni` with `ptr_class` or `data_class`, but all four are a
    /// `type`; the adapter's word is what [`Target::describe`] prints back.
    /// And it says what the *target* gets, not what the captured Rust source
    /// held: a Kotlin constant read through a captured function is a `const`.
    /// It is what keeps the origins' several naming spaces apart — `type:Foo`
    /// and `conversion:Foo` are two declarations about one Rust type.
    ///
    /// [`Target::describe`]: crate::target::Target::describe
    pub fn kind(&self) -> &'static str {
        match self {
            Origin::Function(_) | Origin::LocalFunction(_) => "fn",
            Origin::Const(_) | Origin::ConstFromFunction(_) | Origin::LocalConst(_) => "const",
            Origin::Type(_) | Origin::LocalType(_) => "type",
            Origin::Callback(_) => "callback",
            Origin::Conversion(_) => "conversion",
        }
    }

    /// Whether this declares a type — captured or the binding's own.
    pub fn is_type(&self) -> bool {
        matches!(self, Origin::Type(_) | Origin::LocalType(_))
    }

    /// The captured element this origin names, if the model holds it.
    ///
    /// Captured items live in one flat namespace holding functions, types and
    /// constants, so the name alone finds any element; the variant is what says
    /// whether the element found is the one the declaration meant. A constant
    /// read through a function is looked up among the functions, which is what
    /// separates [`Self::ConstFromFunction`] from [`Self::Const`]. A
    /// binding-local origin names no captured item and so finds none — which
    /// is not the same as a missing one, and [`Self::missing_from`] is the
    /// question to ask about presence.
    pub(crate) fn captured<'f>(&self, flat: &'f Flat) -> Option<&'f Element> {
        let element = flat.element(&self.name())?;
        matches!(
            (self, element),
            (
                Origin::Function(_) | Origin::ConstFromFunction(_),
                Element::Function(_)
            ) | (Origin::Type(_), Element::Type(_))
                | (Origin::Const(_), Element::Constant(_))
        )
        .then_some(element)
    }

    /// Whether the binding defines this itself: a callback signature, a
    /// binding-local conversion helper, a function or constant of its own, or a
    /// type the target represents although the source never exported it
    /// (`String` as an opaque handle). Such an origin requires nothing of the
    /// model.
    pub fn is_binding_local(&self) -> bool {
        matches!(
            self,
            Origin::LocalFunction(_)
                | Origin::LocalConst(_)
                | Origin::LocalType(_)
                | Origin::Callback(_)
                | Origin::Conversion(_)
        )
    }

    /// Whether the model lacks what this origin must name.
    ///
    /// Naming the wrong kind — `.fun(fun!(x))` where the source captured
    /// `const x` — counts as missing: it is an error in the binding, and the
    /// engine fails the run over it instead of reporting a skip. False for a
    /// binding-local origin.
    pub(crate) fn missing_from(&self, flat: &Flat) -> bool {
        !self.is_binding_local() && self.captured(flat).is_none()
    }

    /// The word a refusal uses for what this origin must name.
    pub(crate) fn describe_captured(&self) -> &'static str {
        match self {
            Origin::Function(_) | Origin::ConstFromFunction(_) => "function",
            Origin::Const(_) => "constant",
            Origin::Type(_) => "type",
            _ => "binding-local item",
        }
    }

    /// The name it goes by — the part of the printed form after the kind.
    pub fn name(&self) -> String {
        match self {
            Origin::Function(ident) | Origin::Const(ident) | Origin::ConstFromFunction(ident) => {
                ident.to_string()
            }
            Origin::Type(key) | Origin::LocalType(key) | Origin::Conversion(key) => {
                key.as_str().to_string()
            }
            Origin::LocalFunction(name) | Origin::LocalConst(name) | Origin::Callback(name) => {
                name.clone()
            }
        }
    }
}

impl Ord for Origin {
    /// By kind in report order, then by name. A captured and a binding-local
    /// origin of one kind and name print alike and are still distinct, so the
    /// variant breaks that tie last, and `Ord` agrees with `Eq`.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Kind first, variant second: the variants of one kind are adjacent.
        fn rank(origin: &Origin) -> (u8, u8) {
            match origin {
                Origin::Type(_) => (0, 0),
                Origin::LocalType(_) => (0, 1),
                Origin::Conversion(_) => (1, 0),
                Origin::Callback(_) => (2, 0),
                Origin::Const(_) => (3, 0),
                Origin::ConstFromFunction(_) => (3, 1),
                Origin::LocalConst(_) => (3, 2),
                Origin::Function(_) => (4, 0),
                Origin::LocalFunction(_) => (4, 1),
            }
        }
        let (kind, variant) = rank(self);
        let (other_kind, other_variant) = rank(other);
        kind.cmp(&other_kind)
            .then_with(|| self.name().cmp(&other.name()))
            .then_with(|| variant.cmp(&other_variant))
    }
}

impl PartialOrd for Origin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind(), self.name())
    }
}

impl Serialize for Origin {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
