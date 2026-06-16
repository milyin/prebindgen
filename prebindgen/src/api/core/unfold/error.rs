//! Errors produced while resolving output-deconstruction declarations.

/// Errors surfaced while resolving [`Deconstructors`](super::Deconstructors) in
/// [`apply`](super::apply).
#[derive(Debug)]
pub enum UnfoldError {
    UnknownFunction(syn::Ident),
    UnknownAccessor(syn::Ident),
    NoDeconstructor {
        func: syn::Ident,
        target: String,
    },
    AmbiguousDeconstructor {
        func: syn::Ident,
        target: String,
        candidates: Vec<String>,
    },
    UnknownDeconstructor {
        func: syn::Ident,
        name: String,
    },
    AccessorTargetMismatch {
        accessor: String,
        takes: String,
        expected: String,
    },
    MultipleIdentity {
        target: String,
    },
    /// A nested deconstructor recurses back into a type already on the nesting
    /// chain (`A → … → A`).
    Cycle {
        target: String,
    },
    /// `.convert_output()` on a deconstructor that does not flatten to exactly
    /// one leaf, or whose shape is `Iterable` (use `.deconstruct_output()`).
    ConvertNotSingle {
        func: syn::Ident,
        reason: &'static str,
    },
    /// A decomposer record references a function that was not declared via
    /// `.fun_accessor`.
    RecordNotAccessor {
        func: syn::Ident,
    },
    /// A shape / record kind not yet implemented.
    Unsupported {
        func: syn::Ident,
        reason: &'static str,
    },
    /// Two leaves of one deconstructor resolved to the same (literal) name.
    /// Author leaf names are explicit and emitted verbatim, so a collision is a
    /// declaration bug — never auto-resolved.
    DuplicateLeafName {
        target: String,
        name: String,
    },
    /// An author-supplied leaf name contains the reserved `"__"` chain
    /// separator (used internally to join nested deconstructor segments).
    ReservedSeparator {
        name: String,
    },
}

impl std::fmt::Display for UnfoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnfoldError::UnknownFunction(name) => write!(
                f,
                "output expansion: function `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::UnknownAccessor(name) => write!(
                f,
                "output expansion: accessor `{}` is not a #[prebindgen] item",
                name
            ),
            UnfoldError::NoDeconstructor { func, target } => write!(
                f,
                "output expansion: no deconstructor registered for `{}` (return of `{}`)",
                target, func
            ),
            UnfoldError::AmbiguousDeconstructor {
                func,
                target,
                candidates,
            } => write!(
                f,
                "output expansion: multiple deconstructors for `{}` (return of `{}`): {} — disambiguate with `.deconstruct_output_with` / `.convert_output_with`",
                target,
                func,
                candidates.join(", ")
            ),
            UnfoldError::UnknownDeconstructor { func, name } => write!(
                f,
                "output expansion: no deconstructor named `{}` (for `{}`)",
                name, func
            ),
            UnfoldError::AccessorTargetMismatch {
                accessor,
                takes,
                expected,
            } => write!(
                f,
                "output expansion: accessor `{}` takes `{}` but the deconstructor decomposes `{}`",
                accessor, takes, expected
            ),
            UnfoldError::MultipleIdentity { target } => write!(
                f,
                "output expansion: deconstructor for `{}` has more than one identity record",
                target
            ),
            UnfoldError::Cycle { target } => write!(
                f,
                "output expansion: nested deconstructors form a cycle through `{}`",
                target
            ),
            UnfoldError::ConvertNotSingle { func, reason } => write!(
                f,
                "convert_output: `{}` is not a single-value deconstructor: {}",
                func, reason
            ),
            UnfoldError::RecordNotAccessor { func } => write!(
                f,
                "deconstructor record `{}` is not a `.fun_accessor` — decomposer records may only \
                 reference functions declared via `.fun_accessor(...)`",
                func
            ),
            UnfoldError::Unsupported { func, reason } => write!(
                f,
                "output expansion: `{}` not yet supported: {}",
                func, reason
            ),
            UnfoldError::DuplicateLeafName { target, name } => write!(
                f,
                "deconstructor for `{}` has two output records named `{}` — leaf names must be \
                 unique (they are emitted literally)",
                target, name
            ),
            UnfoldError::ReservedSeparator { name } => write!(
                f,
                "output record name `{}` contains the reserved `__` separator (used to join \
                 nested deconstructor segments)",
                name
            ),
        }
    }
}

impl std::error::Error for UnfoldError {}
