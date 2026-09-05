//! What a binding declared, in terms neither language owns.
//!
//! The v2 engine is handed the frontend's **own** declaration storage, borrowed
//! live behind [`BindingDeclarations`] — no JSON round trip, no replay of
//! builder calls, and nothing copied out that could go stale or lose a closure.
//! What the trait exposes is what v2 can currently *ask*; growing a capability
//! grows the questions, never the encoding.

use serde::Serialize;

/// The kind of thing a binding declared.
///
/// Not the declarator: `opaque_ptr`, `data_struct` and `ptr_class` are all
/// [`ElementKind::Type`], and which declarator produced one is the adapter's
/// [`DeclaredElement::representation`]. This is the axis a report groups and
/// sorts by, and the axis an id is unique within.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementKind {
    /// A `#[prebindgen]` function the binding exports.
    Function,
    /// A `#[prebindgen]` type the binding gives a foreign representation.
    Type,
    /// A `#[prebindgen]` constant the binding exports.
    Const,
    /// A callback signature the binding exports as a foreign callable.
    Callback,
    /// A declared conversion between a Rust type and its wire form.
    Conversion,
}

impl ElementKind {
    /// The id prefix and report label.
    pub fn as_str(self) -> &'static str {
        match self {
            ElementKind::Function => "fn",
            ElementKind::Type => "type",
            ElementKind::Const => "const",
            ElementKind::Callback => "callback",
            ElementKind::Conversion => "conversion",
        }
    }
}

/// A declared element's stable identity: `<kind>:<rust origin>`.
///
/// Stable across runs and across pipelines, so a report, a build script and a
/// capability-selected test section can all name the same element. Derived from
/// what the *source* calls the thing, not from what the target does — a rename
/// on the foreign side must not silently retire a test's requirement.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ElementId(String);

impl ElementId {
    /// The id of `origin` declared as `kind`.
    pub fn new(kind: ElementKind, origin: impl AsRef<str>) -> Self {
        ElementId(format!("{}:{}", kind.as_str(), origin.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One declared output element, as the report accounts for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeclaredElement {
    /// Stable identity — see [`ElementId`].
    pub id: ElementId,
    /// Which kind of element it is.
    pub kind: ElementKind,
    /// What the Rust source calls it (`Calculator`, `calculator_new`), or the
    /// signature for a callback that has no name of its own.
    pub rust_origin: String,
    /// Where it is meant to land in the target language, spelled the way that
    /// language spells it: `calculator_t`, `io.zenoh.jni.Session`.
    pub placement: String,
    /// The declarator that produced it (`opaque_ptr`, `data_class`, `fun`, …).
    /// The adapter's word, printed back verbatim.
    pub representation: String,
    /// Whether [`Self::rust_origin`] is **required** to name a captured
    /// `#[prebindgen]` item.
    ///
    /// `false` for the things a binding may define itself — a callback
    /// signature, a binding-local conversion helper, a type the target
    /// represents but the source never exported (`String` as an opaque handle).
    /// A `true` element that resolves to nothing is an error, so this is stated
    /// by the adapter rather than guessed from the kind.
    pub must_resolve: bool,
}

impl DeclaredElement {
    /// Declare `rust_origin` as a `kind` the target places at `placement`.
    pub fn new(
        kind: ElementKind,
        rust_origin: impl Into<String>,
        placement: impl Into<String>,
        representation: impl Into<String>,
    ) -> Self {
        let rust_origin = rust_origin.into();
        DeclaredElement {
            id: ElementId::new(kind, &rust_origin),
            kind,
            rust_origin,
            placement: placement.into(),
            representation: representation.into(),
            must_resolve: true,
        }
    }

    /// The same, for something the binding may define itself rather than
    /// something it must select out of the captured source — see
    /// [`Self::must_resolve`].
    pub fn local(mut self) -> Self {
        self.must_resolve = false;
        self
    }
}

/// A binding's declarations, borrowed live from the frontend that accumulated
/// them.
///
/// Implemented by the v1 facades over their existing builder storage, which is
/// what lets an example submit its unchanged declarations to either engine.
pub trait BindingDeclarations {
    /// The target this binding generates for — `"c"`, `"jni"`. Names the
    /// report and the output directory.
    fn target(&self) -> &'static str;

    /// Every element the binding asked for, in any order: the report sorts.
    fn declared_elements(&self) -> Vec<DeclaredElement>;

    /// Every element the binding explicitly told the generator to leave alone.
    /// Accounted for separately — an ignore is a decision, not a gap.
    fn ignored_elements(&self) -> Vec<DeclaredElement> {
        Vec::new()
    }
}
