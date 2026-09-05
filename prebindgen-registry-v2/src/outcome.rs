//! What became of a declared element, and why.
//!
//! Four outcomes, and they are not interchangeable. A capability v2 has not
//! implemented is a [`Skipped`](Outcome::Skipped) element with a code and a
//! path to the site that could not be lowered. Malformed input, a missing
//! declared item, or a bug in the engine are **errors** — never skips — because
//! a build that quietly reports its own bug as an unsupported feature is a
//! build that cannot be trusted to say what it generated.

use serde::Serialize;

use crate::decl::{ElementId, SourceKind};

/// A stable code naming *what* is not implemented, dotted from general to
/// specific: `unsupported.string`, `unsupported.handle.borrowed_input`.
///
/// Stable because reports are diffed and CI gates name codes. The readable
/// explanation beside it is free to change; this is not.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Capability(String);

impl Capability {
    pub fn new(code: impl Into<String>) -> Self {
        Capability(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why one element was not generated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Skip {
    /// The unimplemented capability — see [`Capability`].
    pub capability: Capability,
    /// One readable sentence a build script can print as it stands.
    pub explanation: String,
    /// Root → failing site, one step per line of the printed path: the element,
    /// then how the walk reached the thing that could not be lowered. A direct
    /// skip has just the root.
    pub dependency_path: Vec<String>,
}

impl Skip {
    /// A skip at the element itself, with no dependency to walk through.
    pub fn direct(
        capability: impl Into<String>,
        explanation: impl Into<String>,
        root: impl Into<String>,
    ) -> Self {
        Skip {
            capability: Capability::new(capability),
            explanation: explanation.into(),
            dependency_path: vec![root.into()],
        }
    }

    /// This skip as `a -> b -> c`.
    pub fn path(&self) -> String {
        self.dependency_path.join(" -> ")
    }
}

/// What became of one declared element.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    /// Fully generated, with its dependencies and its declared semantics.
    Emitted,
    /// Not generated: a capability is missing, here or in something this
    /// element needs.
    Skipped(Skip),
    /// The binding asked for it to be left alone.
    Ignored,
}

impl Outcome {
    /// The skip, when this is one.
    pub fn skip(&self) -> Option<&Skip> {
        match self {
            Outcome::Skipped(skip) => Some(skip),
            Outcome::Emitted | Outcome::Ignored => None,
        }
    }
}

/// The engine failed, and no report describes the run.
///
/// Distinct from a skip on purpose: everything here means the *input* or the
/// *engine* is wrong, not that a feature is missing.
#[derive(Debug)]
pub enum EngineError {
    /// The declared sources could not be parsed into the model.
    Source(prebindgen_flat::flat::ParseError),
    /// Declared elements that match no captured `#[prebindgen]` item. A
    /// declaration is a statement of intent; its target being absent is a typo
    /// or a source-crate rename, and never a capability question. All of them
    /// are collected before failing.
    DeclaredNotFound {
        entries: Vec<(ElementId, SourceKind, String)>,
    },
    /// Two declared elements answering to one id. An id identifies an element,
    /// and a manifest that gave one id to two entries could not account for
    /// either — so this is a contradiction in the declarations, not a gap in
    /// what v2 implements.
    DuplicateElement { entries: Vec<ElementId> },
    /// Writing an artifact or a report failed.
    Io(std::io::Error),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Source(error) => write!(f, "v2: {error}"),
            EngineError::DeclaredNotFound { entries } => {
                writeln!(
                    f,
                    "v2: {} declared item(s) match no captured `#[prebindgen]` item:",
                    entries.len()
                )?;
                for (id, source, origin) in entries {
                    writeln!(f, "  {id} — no captured {} `{origin}`", source.describe())?;
                }
                write!(
                    f,
                    "check the spelling in build.rs, and that the source crate still \
                     exports them"
                )
            }
            EngineError::DuplicateElement { entries } => {
                writeln!(
                    f,
                    "v2: {} element id(s) were declared more than once:",
                    entries.len()
                )?;
                for id in entries {
                    writeln!(f, "  {id}")?;
                }
                write!(f, "each declared element answers to one id")
            }
            EngineError::Io(error) => write!(f, "v2: {error}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<std::io::Error> for EngineError {
    fn from(error: std::io::Error) -> Self {
        EngineError::Io(error)
    }
}

impl From<prebindgen_flat::flat::ParseError> for EngineError {
    fn from(error: prebindgen_flat::flat::ParseError) -> Self {
        EngineError::Source(error)
    }
}
