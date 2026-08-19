//! Errors produced while resolving output-deconstruction declarations.

/// Errors surfaced while resolving
/// [`Deconstructors`](super::Deconstructors).
#[derive(Debug)]
pub enum UnfoldError {
    UnknownFunction(syn::Ident),
    UnknownAccessor(syn::Ident),
    NoDeconstructor {
        func: syn::Ident,
        target: String,
    },
    AccessorTargetMismatch {
        accessor: String,
        takes: String,
        expected: String,
    },
    MultipleIdentity {
        target: String,
        /// Where in the decomposition — see [`UnfoldError::Unsupported::at`].
        at: String,
    },
    /// A nested deconstructor recurses back into a type already on the nesting
    /// chain (`A → … → A`).
    Cycle {
        target: String,
        /// Where in the decomposition — see [`UnfoldError::Unsupported::at`].
        at: String,
    },
    /// A single-value (`Return`) delivery on a decomposition that does not
    /// flatten to exactly one leaf, or whose shape is `Iterable`.
    ConvertNotSingle {
        func: syn::Ident,
        reason: &'static str,
    },
    /// A decomposer record references a function that was not declared via
    /// `.accessor`.
    RecordNotAccessor {
        func: syn::Ident,
    },
    /// A variant arm of a decomposed sum carries something other than a plain
    /// payload leaf. The flat projection binds one member per payload and gives
    /// every leaf under the arm the arm's own tag, so a subtree there loses
    /// both its member binding and — for a nested sum — its own tags.
    UnsupportedVariantPayload {
        /// The sum being decomposed.
        target: String,
        /// The arm whose payload could not be projected.
        variant: String,
        /// What the payload is instead of a leaf.
        found: &'static str,
    },
    /// A shape / record kind not yet implemented.
    Unsupported {
        func: syn::Ident,
        reason: &'static str,
        /// Where in the decomposition the transformation failed: the access
        /// path from the returned value to the node that could not be built,
        /// `value` at the root. A deconstructor splices nested ones, so naming
        /// only the accessor leaves the reader to work out which nesting of it
        /// is the one that failed.
        at: String,
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
    /// An owned decomposition declared `.field_self()` (the root identity,
    /// which MOVES the value) before a field that splices a nested identity
    /// (which borrows it) — the generated Rust would not compile.
    RootIdentityBeforeNested {
        target: String,
    },
    /// A per-fn `.expand_return(expand_return!(T)…)` decl whose `T` does not
    /// match the function's peeled return type.
    ReturnTypeMismatch {
        func: syn::Ident,
        declared: String,
        actual: String,
    },
    /// Structurally invalid declaration records — empty record lists or
    /// duplicate targets. All offenders are collected before failing
    /// (mirrors `ScanError::DeclaredNotFound`).
    InvalidDeclarations {
        entries: Vec<UnfoldDeclError>,
    },
}

/// One structurally invalid output-expansion declaration (see
/// [`UnfoldError::InvalidDeclarations`]). Note that EMPTY record lists are
/// deliberately not diagnosed here — an empty inline list is the valid
/// whole-element (`Vec<T>` per-element) delivery form.
#[derive(Debug)]
pub enum UnfoldDeclError {
    /// Two deconstructor declarations for the same target type.
    DuplicateDeconstructor { target: String },
    /// Two per-fn output expansions for the same fn and position.
    DuplicateOutput {
        func: syn::Ident,
        target: super::DeconTarget,
    },
}

impl std::fmt::Display for UnfoldDeclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnfoldDeclError::DuplicateDeconstructor { target } => {
                write!(f, "duplicate deconstructor declaration for `{target}`")
            }
            UnfoldDeclError::DuplicateOutput { func, target } => write!(
                f,
                "duplicate output expansion for `{func}` ({target:?} position)"
            ),
        }
    }
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
            UnfoldError::ReturnTypeMismatch {
                func,
                declared,
                actual,
            } => write!(
                f,
                "output expansion: `{}`.expand_return(expand_return!({declared})): the \
                 function's return type is `{actual}`, not `{declared}` — declare the decl \
                 for the actual return type",
                func
            ),
            UnfoldError::NoDeconstructor { func, target } => write!(
                f,
                "output expansion: no deconstructor registered for `{}` (return of `{}`)",
                target, func
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
            UnfoldError::MultipleIdentity { target, at } => write!(
                f,
                "output expansion at `{}`: deconstructor for `{}` has more than one identity record",
                at, target
            ),
            UnfoldError::Cycle { target, at } => write!(
                f,
                "output expansion at `{}`: nested deconstructors form a cycle through `{}`",
                at, target
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
            UnfoldError::UnsupportedVariantPayload {
                target,
                variant,
                found,
            } => write!(
                f,
                "output expansion of `{}` at variant `{}`: a payload that is {} has no flat \
                 reading — one member is bound per payload leaf, and every leaf under an arm \
                 takes that arm's tag, so a subtree there loses its member binding and any tags \
                 of its own",
                target, variant, found
            ),
            UnfoldError::Unsupported { func, reason, at } => write!(
                f,
                "output expansion of `{}` at `{}` not yet supported: {}",
                func, at, reason
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
            UnfoldError::RootIdentityBeforeNested { target } => write!(
                f,
                "return-field list of `{}`: `.field_self()` (the root identity) must be \
                 declared AFTER fields that splice a nested identity — the root identity moves the owned \
                 value while nested identities borrow it, so this order would generate \
                 non-compiling Rust. Declare the `_self` field last.",
                target
            ),
            UnfoldError::InvalidDeclarations { entries } => {
                writeln!(f, "output expansion: invalid declarations:")?;
                for e in entries {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for UnfoldError {}
