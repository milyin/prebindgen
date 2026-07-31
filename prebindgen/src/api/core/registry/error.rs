//! What can go wrong before a binding is written.

use std::fmt;

use crate::SourceLocation;

impl From<crate::api::core::flat::ParseError> for ScanError {
    fn from(e: crate::api::core::flat::ParseError) -> Self {
        match e {
            crate::api::core::flat::ParseError::DuplicateName(d) => {
                ScanError::DuplicateName(Box::new(DuplicateNameError {
                    name: d.name,
                    first: d.first,
                    second: d.second,
                    first_crate: d.first_crate,
                    second_crate: d.second_crate,
                }))
            }
        }
    }
}

/// One item of a [`ScanError::NotExpressible`] report.
#[derive(Debug)]
pub struct NotExpressibleEntry {
    /// The item's name, or `None` for an item kind that has none.
    pub name: Option<syn::Ident>,
    /// Rendered [`ItemError`](crate::core::flat::ItemError) — the frontend's own
    /// message, so one authority produces it.
    pub reason: String,
    pub location: SourceLocation,
}

/// Payload of [`ScanError::DuplicateName`], boxed to keep the error enum
/// small (`clippy::result_large_err`).
#[derive(Debug)]
pub struct DuplicateNameError {
    pub name: syn::Ident,
    pub first: SourceLocation,
    pub second: SourceLocation,
    /// Origin crates of the colliding items, when known (multi-source
    /// ingestion via several `Flat::builder().source(..)` feeders) — the `SourceLocation`
    /// file paths are crate-relative, so with several sources they alone
    /// may not identify the colliding crates.
    pub first_crate: Option<String>,
    pub second_crate: Option<String>,
}

/// Errors surfaced by the scan phase.
#[derive(Debug)]
pub enum ScanError {
    DuplicateName(Box<DuplicateNameError>),
    /// Items the flat language cannot express, all of them at once.
    ///
    /// The message for each comes from
    /// [`ItemError`](crate::core::flat::ItemError), so one authority produces it.
    /// This replaces the per-item guards the registry used to duplicate — a `self`
    /// receiver, a non-ident parameter pattern, a disallowed `impl Trait` — which
    /// the frontend now catches with a richer diagnosis (it names the parameter).
    NotExpressible {
        entries: Vec<NotExpressibleEntry>,
    },
    /// An adapter-invariant check failed — see [`Prebindgen::validate`].
    /// The message is adapter-authored and printed verbatim.
    AdapterInvariant {
        message: String,
    },
    /// Explicitly declared items (functions, helper functions, constants)
    /// that match no indexed `#[prebindgen]` item. A declaration is a
    /// statement of intent — its target being absent is always a bug (a
    /// typo in build.rs, or the item was renamed/removed in the source
    /// crate), so this is a hard error, unlike the soft warnings for stale
    /// *ignore* entries. All missing names are collected before failing.
    DeclaredNotFound {
        entries: Vec<(&'static str, String)>,
    },
    /// Declared type keys that qualify a source item with its crate path
    /// (`ptr_class!(myflat::Foo)` where `myflat` is a chained source crate).
    /// Source items live in one flat namespace and are keyed by their bare
    /// name — the qualified spelling can never match a captured signature,
    /// so it is a hard error with a fix-it instead of a silent miss (issue
    /// #95). All offenders are collected before failing.
    QualifiedDeclaredTypes {
        /// `(qualified spelling, bare fix-it name)` pairs.
        entries: Vec<(String, String)>,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::DuplicateName(e) => {
                let in_crate = |c: &Option<String>| match c {
                    Some(c) => format!(" in crate `{c}`"),
                    None => String::new(),
                };
                write!(
                    f,
                    "duplicate prebindgen name `{}`: first{} at {}, second{} at {} — prebindgen \
                     items live in one flat namespace across all sources; rename one of them",
                    e.name,
                    in_crate(&e.first_crate),
                    e.first,
                    in_crate(&e.second_crate),
                    e.second
                )
            }
            ScanError::NotExpressible { entries } => {
                write!(
                    f,
                    "{} `#[prebindgen]` item(s) the flat language cannot express:",
                    entries.len()
                )?;
                for e in entries {
                    // The crate, because a captured path is crate-relative: with
                    // several sources, two offenders both read `src/lib.rs:..`
                    // and the location alone says nothing about which one to fix.
                    // Same reason the duplicate-name diagnostic carries it.
                    let in_crate = match &e.location.crate_name {
                        Some(c) => format!(" in crate `{c}`"),
                        None => String::new(),
                    };
                    match &e.name {
                        Some(name) => {
                            write!(f, "\n  {}{in_crate}: {name} {}", e.location, e.reason)?
                        }
                        None => write!(f, "\n  {}{in_crate}: {}", e.location, e.reason)?,
                    }
                }
                Ok(())
            }
            ScanError::AdapterInvariant { message } => write!(f, "{}", message),
            ScanError::DeclaredNotFound { entries } => {
                writeln!(
                    f,
                    "{} declared item(s) not found among #[prebindgen] items:",
                    entries.len()
                )?;
                for (kind, name) in entries {
                    writeln!(f, "  - {kind} `{name}`")?;
                }
                write!(
                    f,
                    "a declaration names an item that does not exist — typo in build.rs, \
                     or renamed/removed in the source crate?"
                )
            }
            ScanError::QualifiedDeclaredTypes { entries } => {
                writeln!(
                    f,
                    "{} declared type(s) qualify a source item with its crate path:",
                    entries.len()
                )?;
                for (spelled, bare) in entries {
                    writeln!(f, "  - `{spelled}` — declare it as `{bare}`")?;
                }
                write!(
                    f,
                    "source items live in one flat namespace keyed by their bare name; \
                     a crate-qualified spelling never matches captured signatures"
                )
            }
        }
    }
}

impl std::error::Error for ScanError {}

/// Combined error surfaced by `RegistryBuilder::build` and by a generator's
/// own `build` / `write_rust`.
#[derive(Debug)]
pub enum WriteRustError {
    Scan(ScanError),
    Expand(crate::api::core::expand::ExpandError),
    Unfold(crate::api::core::unfold::UnfoldError),
    Resolve(crate::api::core::resolve::ResolveError),
    Write(crate::api::core::write::WriteError),
}

impl fmt::Display for WriteRustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteRustError::Scan(e) => write!(f, "{}", e),
            WriteRustError::Expand(e) => write!(f, "{}", e),
            WriteRustError::Unfold(e) => write!(f, "{}", e),
            WriteRustError::Resolve(e) => write!(f, "{}", e),
            WriteRustError::Write(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for WriteRustError {}

impl From<ScanError> for WriteRustError {
    fn from(e: ScanError) -> Self {
        WriteRustError::Scan(e)
    }
}

impl From<crate::api::core::expand::ExpandError> for WriteRustError {
    fn from(e: crate::api::core::expand::ExpandError) -> Self {
        WriteRustError::Expand(e)
    }
}

impl From<crate::api::core::unfold::UnfoldError> for WriteRustError {
    fn from(e: crate::api::core::unfold::UnfoldError) -> Self {
        WriteRustError::Unfold(e)
    }
}

impl From<crate::api::core::resolve::ResolveError> for WriteRustError {
    fn from(e: crate::api::core::resolve::ResolveError) -> Self {
        WriteRustError::Resolve(e)
    }
}

impl From<crate::api::core::write::WriteError> for WriteRustError {
    fn from(e: crate::api::core::write::WriteError) -> Self {
        WriteRustError::Write(e)
    }
}
