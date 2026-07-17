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
    /// A binding-local field's callable path has a single segment — the
    /// generated file calls it qualified, so at least `crate::` is required.
    LocalAccessorBarePath {
        path: String,
    },
    /// A binding-local field's fn name collides with a `#[prebindgen]` item
    /// (or another binding-local field with a different signature) — the
    /// emitted call is `<prefix>::<name>`, so the name must be unambiguous.
    LocalAccessorCollision {
        name: syn::Ident,
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
            UnfoldError::LocalAccessorBarePath { path } => write!(
                f,
                "field!(...).with(..., path!({path})): a binding-local accessor is called \
                 QUALIFIED from the generated file — give at least a `crate::`-rooted path",
            ),
            UnfoldError::LocalAccessorCollision { name } => write!(
                f,
                "field!(...).with(...): the binding-local fn name `{name}` collides with a \
                 `#[prebindgen]` item (or another binding-local field with a different \
                 signature) — the emitted call is `<prefix>::{name}`, so rename the \
                 binding-local fn",
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
            UnfoldError::RootIdentityBeforeNested { target } => write!(
                f,
                "return-field list of `{}`: `.field_self()` (the root identity) must be \
                 declared AFTER fields that splice a nested identity — the root identity moves the owned \
                 value while nested identities borrow it, so this order would generate \
                 non-compiling Rust. Declare the `_self` field last.",
                target
            ),
        }
    }
}

impl std::error::Error for UnfoldError {}
