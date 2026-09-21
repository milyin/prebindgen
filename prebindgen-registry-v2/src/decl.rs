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
/// * **Identity.** A [`DeclarationId`] pairs the kind with the origin, and the
///   kind is what keeps the origins' several naming spaces apart: an origin may
///   be a captured item's name, a type key, a callback's signature or a name
///   the binding coined, so `type:Foo` and `conversion:Foo` are two
///   declarations about one Rust type, and the engine can tell which of them a
///   report entry or an outcome belongs to.
/// * **Report layout.** [`Report`](crate::Report) groups and sorts by it, so a
///   report reads types first, then conversions, callbacks, constants and
///   functions.
///
/// It says what the *target* language gets, not what the captured Rust source
/// held — a Kotlin constant may be backed by a captured Rust function. That
/// second question is [`Declaration::source`] / [`SourceKind`].
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
    /// The id prefix and report label.
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

/// A declaration's stable identity: its [`DeclarationKind`] and the name its
/// Rust origin goes by.
///
/// Stable across runs and across pipelines, so a report, a build script and a
/// capability-selected test section can all name the same declaration. Derived from
/// what the *source* calls the thing, not from what the target does — a rename
/// on the foreign side must not silently retire a test's requirement.
///
/// The two parts are stored, readable ([`Self::kind`], [`Self::origin`]) and
/// fixed at construction. `<kind>:<origin>` — `type:Stamp`, `fn:stamp_sum` —
/// is how an id prints and how the report writes it, and that spelling is a
/// rendering: nothing reads an id back out of it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId {
    kind: DeclarationKind,
    origin: String,
}

impl DeclarationId {
    /// The id of `origin` declared as `kind`.
    ///
    /// Crate-internal: an id is what a [`Declaration`] already has, and the
    /// engine hands it out. Nothing outside builds one from a name — a target
    /// naming another declaration names the type it needs, with a
    /// [`Requirement`](crate::Requirement).
    pub(crate) fn new(kind: DeclarationKind, origin: impl AsRef<str>) -> Self {
        DeclarationId {
            kind,
            origin: origin.as_ref().to_string(),
        }
    }

    /// Which kind of declaration this names.
    pub fn kind(&self) -> DeclarationKind {
        self.kind
    }

    /// What the Rust source calls it — see [`Declaration::rust_origin`].
    pub fn origin(&self) -> &str {
        &self.origin
    }
}

impl std::fmt::Display for DeclarationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.kind.as_str(), self.origin)
    }
}

impl Serialize for DeclarationId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// What a declaration is named after, and what the engine plans it from.
///
/// A name alone says neither: `Sample` may be a captured type, a type key the
/// binding coined for something the source never exported, or the name of a
/// Kotlin constant built from an expression. The variant says which, and it
/// says it once — a declaration's [`DeclarationKind`] and its [`SourceKind`]
/// both follow from the variant instead of being stated beside it, so the three
/// cannot disagree and a pair that means nothing (a callback backed by a
/// captured constant, a function the binding both defines and selects out of
/// the source) cannot be written down.
///
/// [`generate`](crate::generate) routes on this: each planner is reached by its
/// own variants and is handed the captured item they name, rather than a kind
/// to decode and a name to look up.
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// What this origin must name in the captured source.
    pub fn source_kind(&self) -> SourceKind {
        match self {
            // A constant read through a function is looked up among the
            // functions, which is what separates it from `Const`.
            Origin::Function(_) | Origin::ConstFromFunction(_) => SourceKind::Function,
            Origin::Const(_) => SourceKind::Const,
            Origin::Type(_) => SourceKind::Type,
            Origin::LocalFunction(_)
            | Origin::LocalConst(_)
            | Origin::LocalType(_)
            | Origin::Callback(_)
            | Origin::Conversion(_) => SourceKind::BindingLocal,
        }
    }

    /// The name it goes by — what an id carries and a report prints.
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

/// One declaration, as the report accounts for it.
///
/// Built by the adapter with [`Self::new`] and read back through the accessors.
/// It holds two things about what was declared, for two jobs: the
/// [`DeclarationId`] the run is keyed by, and the [`Origin`] the engine plans
/// from. Everything else about the declaration's identity — its kind, its
/// source kind, the name it goes by — is read back out of those, so nothing is
/// stated twice and nothing can disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    id: DeclarationId,
    origin: Origin,
    placement: String,
    representation: String,
}

impl Serialize for Declaration {
    /// Flat, and with the identity spelled out: the report carries `id`,
    /// `kind` and `rust_origin` as separate columns, and a reader of the JSON
    /// should not have to split the id to get at the last two.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut entry = serializer.serialize_struct("Declaration", 6)?;
        entry.serialize_field("id", &self.id)?;
        entry.serialize_field("kind", &self.id.kind())?;
        entry.serialize_field("rust_origin", self.id.origin())?;
        entry.serialize_field("placement", &self.placement)?;
        entry.serialize_field("representation", &self.representation)?;
        entry.serialize_field("source", &self.source())?;
        entry.end()
    }
}

/// What a declaration's Rust origin must name in the captured source.
///
/// Captured items live in one flat namespace holding functions, types and
/// constants, and a declaration means one of the three. Naming the wrong one —
/// `.fun(fun!(x))` where the source captured `const x` — is an error in the
/// binding, and the engine fails the run over it instead of reporting a skip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// A captured `#[prebindgen]` function.
    Function,
    /// A captured `#[prebindgen]` type.
    Type,
    /// A captured `#[prebindgen]` constant.
    Const,
    /// Nothing: the binding defines this itself. A callback signature, a
    /// binding-local conversion helper, or a type the target represents
    /// although the source never exported it (`String` as an opaque handle).
    BindingLocal,
}

impl SourceKind {
    /// The captured element an origin of this kind names, if the model holds it.
    ///
    /// The namespace is flat, so the name alone finds any element; the kind is
    /// what says whether the element found is the one the declaration meant.
    /// [`Self::BindingLocal`] names no captured item and so finds none — which
    /// is not the same as a missing one, and [`Self::missing_from`] is the
    /// question to ask about presence.
    pub(crate) fn element<'f>(self, flat: &'f Flat, name: &str) -> Option<&'f Element> {
        let element = flat.element(name)?;
        matches!(
            (self, element),
            (SourceKind::Function, Element::Function(_))
                | (SourceKind::Type, Element::Type(_))
                | (SourceKind::Const, Element::Constant(_))
        )
        .then_some(element)
    }

    /// Whether the model lacks what an origin of this kind must name.
    ///
    /// False for a binding-local origin, which requires nothing of the model.
    pub(crate) fn missing_from(self, flat: &Flat, name: &str) -> bool {
        self != SourceKind::BindingLocal && self.element(flat, name).is_none()
    }

    /// The word a refusal uses for this kind.
    pub(crate) fn describe(self) -> &'static str {
        match self {
            SourceKind::Function => "function",
            SourceKind::Type => "type",
            SourceKind::Const => "constant",
            SourceKind::BindingLocal => "binding-local item",
        }
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
            id: DeclarationId::new(origin.kind(), origin.name()),
            origin,
            placement: placement.into(),
            representation: representation.into(),
        }
    }

    /// What this declaration is made of, and what the engine plans it from —
    /// see [`Origin`].
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Stable identity — see [`DeclarationId`].
    pub fn id(&self) -> &DeclarationId {
        &self.id
    }

    /// Which kind of declaration it is.
    pub fn kind(&self) -> DeclarationKind {
        self.id.kind()
    }

    /// What the Rust source calls it (`Calculator`, `calculator_new`), or the
    /// signature for a callback that has no name of its own.
    pub fn rust_origin(&self) -> &str {
        self.id.origin()
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

    /// What [`Self::rust_origin`] must name in the captured source.
    ///
    /// Not the same question as [`Self::kind`], which says what the *target*
    /// gets: a Kotlin `val` declared with `constant!(X).fun(fun!(f))` is a
    /// [`DeclarationKind::Const`] whose origin is
    /// [`Origin::ConstFromFunction`]. It comes from that origin rather than
    /// being stated beside it, so the two cannot disagree.
    pub fn source(&self) -> SourceKind {
        self.origin.source_kind()
    }
}
