//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one
//! [`Declaration`] per thing the user asked for, inside the
//! [`BindingRequests`](crate::BindingRequests) it hands the engine. The
//! declaration is what the report accounts for; the request carries the target's
//! configuration for it.

use prebindgen_flat::flat::{Element, Flat, TypeKey};
use serde::Serialize;

/// What a [`Declaration`] declares: a function, a type, a constant, a callback
/// or a conversion.
///
/// This is the coarse, language-neutral category — the five things any binding
/// can be made of. It is deliberately coarser than an adapter's own vocabulary:
/// `prebindgen-c` declares a type with `opaque_ptr` or `data_struct`, and
/// `prebindgen-jni` with `ptr_class` or `data_class`, but all four produce a
/// `Type`. The word the adapter used is kept separately in
/// [`Declaration::representation`] and printed back verbatim.
///
/// It is bookkeeping, not a plan: what a declaration is planned from comes from
/// its [`Origin`] — the captured item there is to work with — and the kind only
/// picks between planners where one captured item backs two surfaces, as a
/// Kotlin `val` read through a nullary function is planned as that function.
/// Two things depend on the category:
///
/// * **Identity.** An [`Origin`] prints as `<kind>:<name>`, and the kind is
///   what keeps the origins' several naming spaces apart: an origin may be a
///   captured item's name, a type key, a callback's signature or a name the
///   binding coined, so `type:Foo` and `conversion:Foo` are two declarations
///   about one Rust type, and a reader of the report can tell which of them an
///   entry belongs to.
/// * **Report layout.** [`Report`](crate::Report) groups and sorts by it, so a
///   report reads types first, then conversions, callbacks, constants and
///   functions.
///
/// It says what the *target* language gets, not what the captured Rust source
/// held — a Kotlin constant may be backed by a captured Rust function. That
/// second question is the [`Origin`] variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationKind {
    /// A `#[prebindgen]` function the binding exports a wrapper for.
    Function,
    /// A `#[prebindgen]` type the binding gives a foreign representation.
    Type,
    /// A `#[prebindgen]` constant the binding exposes as a foreign constant.
    Const,
    /// A callback signature the binding exports as a foreign callable.
    Callback,
    /// A declared conversion between a Rust type and its wire form.
    Conversion,
}

impl DeclarationKind {
    /// The prefix an [`Origin`] prints with, and the report label.
    pub fn as_str(self) -> &'static str {
        match self {
            DeclarationKind::Function => "fn",
            DeclarationKind::Type => "type",
            DeclarationKind::Const => "const",
            DeclarationKind::Callback => "callback",
            DeclarationKind::Conversion => "conversion",
        }
    }
}

/// What a declaration is named after, and what the engine plans it from.
///
/// A name alone says neither: `Sample` may be a captured type, a type key the
/// binding coined for something the source never exported, or the name of a
/// Kotlin constant built from an expression. The variant says which, and it
/// says it once — a declaration's [`DeclarationKind`] and what its name must
/// find in the captured source both follow from the
/// variant instead of being stated beside it, so they cannot disagree and a
/// pair that means nothing (a callback backed by a captured constant, a
/// function the binding both defines and selects out of the source) cannot be
/// written down.
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
/// it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// What the target gets — see [`DeclarationKind`].
    pub fn kind(&self) -> DeclarationKind {
        match self {
            Origin::Function(_) | Origin::LocalFunction(_) => DeclarationKind::Function,
            Origin::Const(_) | Origin::ConstFromFunction(_) | Origin::LocalConst(_) => {
                DeclarationKind::Const
            }
            Origin::Type(_) | Origin::LocalType(_) => DeclarationKind::Type,
            Origin::Callback(_) => DeclarationKind::Callback,
            Origin::Conversion(_) => DeclarationKind::Conversion,
        }
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

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind().as_str(), self.name())
    }
}

impl Serialize for Origin {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// One declaration, as the report accounts for it.
///
/// Built by the adapter with [`Self::new`] and read back through the accessors.
/// The [`Origin`] is both the run's key for it and what the engine plans it
/// from; everything else about its identity — its kind, what it must find in
/// the captured source, the name it goes by — is read back out of the origin,
/// so nothing is stated twice and nothing can disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    origin: Origin,
    placement: String,
    representation: String,
}

impl Serialize for Declaration {
    /// Flat, and with the identity spelled out: the report carries `id` (the
    /// origin as it prints), `kind` and `rust_origin` as separate columns, and
    /// a reader of the JSON should not have to split the id to get at the last
    /// two.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut entry = serializer.serialize_struct("Declaration", 5)?;
        entry.serialize_field("id", &self.origin)?;
        entry.serialize_field("kind", &self.origin.kind())?;
        entry.serialize_field("rust_origin", &self.origin.name())?;
        entry.serialize_field("placement", &self.placement)?;
        entry.serialize_field("representation", &self.representation)?;
        entry.end()
    }
}

impl Declaration {
    /// Declare `origin`, which the target places at `placement`.
    pub fn new(
        origin: Origin,
        placement: impl Into<String>,
        representation: impl Into<String>,
    ) -> Self {
        Declaration {
            origin,
            placement: placement.into(),
            representation: representation.into(),
        }
    }

    /// What this declaration is, what the engine plans it from, and what the
    /// run keys it by — see [`Origin`].
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Which kind of declaration it is.
    pub fn kind(&self) -> DeclarationKind {
        self.origin.kind()
    }

    /// What the Rust source calls it (`Calculator`, `calculator_new`), or the
    /// signature for a callback that has no name of its own.
    pub fn rust_origin(&self) -> String {
        self.origin.name()
    }

    /// Where it is meant to land in the target language, spelled the way that
    /// language spells it: `calculator_t`, `io.zenoh.jni.Session`.
    pub fn placement(&self) -> &str {
        &self.placement
    }

    /// The declarator that produced it (`opaque_ptr`, `data_class`, `fun`, …) —
    /// the adapter's own word, printed back verbatim.
    pub fn representation(&self) -> &str {
        &self.representation
    }
}
