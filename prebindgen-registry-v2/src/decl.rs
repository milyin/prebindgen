//! What a binding declared, in terms neither language owns.
//!
//! A frontend turns its own declaration storage into these — one
//! [`Declaration`] per thing the user asked for, inside the
//! [`BindingRequests`](crate::BindingRequests) it hands the engine. The
//! declaration is what the report accounts for; the request carries the target's
//! configuration for it.

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
/// Three things depend on the category:
///
/// * **What the engine plans.** [`generate`](crate::generate) dispatches on it,
///   one lowering per kind: a `Function` is planned as an exported wrapper, a
///   `Type` as a record.
/// * **Identity.** A [`DeclarationId`] is `<kind>:<rust origin>`, and the kind
///   is what keeps the origins' several naming spaces apart: an origin may be a
///   captured item's name, a type key, a callback's signature or a name the
///   binding coined, so `type:Foo` and `conversion:Foo` are two declarations
///   about one Rust type. [`SurfaceSpec::requires`](crate::SurfaceSpec) says
///   by id which of them a wrapper needs emitted.
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

/// A declaration's stable identity: `<kind>:<rust origin>`.
///
/// Stable across runs and across pipelines, so a report, a build script and a
/// capability-selected test section can all name the same declaration. Derived from
/// what the *source* calls the thing, not from what the target does — a rename
/// on the foreign side must not silently retire a test's requirement.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DeclarationId(String);

impl DeclarationId {
    /// The id of `origin` declared as `kind`.
    pub fn new(kind: DeclarationKind, origin: impl AsRef<str>) -> Self {
        DeclarationId(format!("{}:{}", kind.as_str(), origin.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeclarationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One declaration, as the report accounts for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Declaration {
    /// Stable identity — see [`DeclarationId`].
    pub id: DeclarationId,
    /// Which kind of declaration it is.
    pub kind: DeclarationKind,
    /// The source item's name, as the Rust source spells it (`Calculator`,
    /// `calculator_new`), or the signature for a callback that has no name of
    /// its own.
    pub rust_origin: String,
    /// Where the export lands in the target language, spelled the way that
    /// language spells it: `calculator_t`, `io.zenoh.jni.Session`. For a
    /// function, this names what the foreign side calls; the source function
    /// keeps its own name and is only ever called by the wrapper.
    pub placement: String,
    /// The declarator that produced it (`opaque_ptr`, `data_class`, `fun`, …).
    /// The adapter's word, printed back verbatim.
    pub representation: String,
    /// What [`Self::rust_origin`] must name in the captured source.
    ///
    /// Not the same question as [`Self::kind`], which says what the *target*
    /// gets: a Kotlin `val` declared with `constant!(X).fun(fun!(f))` is a
    /// [`DeclarationKind::Const`] backed by a captured **function**. Stated by the
    /// adapter, because only the adapter knows which declarator produced the
    /// declaration.
    pub source: SourceKind,
}

/// What a declaration's Rust origin must name in the captured source.
///
/// Captured items live in one flat namespace, but the namespace holds three
/// kinds and a declaration means one of them: `.fun(fun!(x))` naming a captured
/// `const x` is a mistake, not a shape v2 has yet to implement.
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
    /// Declare `rust_origin` as a `kind` the target places at `placement`.
    pub fn new(
        kind: DeclarationKind,
        rust_origin: impl Into<String>,
        placement: impl Into<String>,
        representation: impl Into<String>,
    ) -> Self {
        let rust_origin = rust_origin.into();
        Declaration {
            id: DeclarationId::new(kind, &rust_origin),
            kind,
            rust_origin,
            placement: placement.into(),
            representation: representation.into(),
            // The usual case: the declaration is named after the item it is built
            // from. `sourced_as` states the exceptions.
            source: match kind {
                DeclarationKind::Function => SourceKind::Function,
                DeclarationKind::Type => SourceKind::Type,
                DeclarationKind::Const => SourceKind::Const,
                DeclarationKind::Callback | DeclarationKind::Conversion => SourceKind::BindingLocal,
            },
        }
    }

    /// The same, for something the binding defines itself rather than something
    /// it selects out of the captured source — see [`SourceKind::BindingLocal`].
    pub fn local(self) -> Self {
        self.sourced_as(SourceKind::BindingLocal)
    }

    /// The same, for a declaration whose target kind and source kind differ — see
    /// [`Self::source`].
    pub fn sourced_as(mut self, source: SourceKind) -> Self {
        self.source = source;
        self
    }
}
