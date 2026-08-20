//! Errors produced while resolving constructor-expansion declarations.

/// Errors surfaced while resolving [`Expansions`](super::Expansions).
#[derive(Debug)]
pub enum ExpandError {
    UnknownFunction(syn::Ident),
    UnknownParam(syn::Ident, syn::Ident),
    UnknownConstructor(syn::Ident),
    NoConstructor {
        func: syn::Ident,
        param: syn::Ident,
        target: String,
    },
    TargetMismatch {
        ctor: String,
        produces: String,
        expected: String,
    },
    UnsupportedOptional {
        func: syn::Ident,
        param: syn::Ident,
        reason: &'static str,
        /// Where in the construction — see
        /// [`ExpandError::UnsupportedRecursive::at`].
        at: String,
    },
    /// An explicit per-fn input flatten targeted a read accessor — accessors
    /// are never parameter-composed.
    ConstructOnAccessor {
        func: syn::Ident,
    },
    /// A per-fn `.expand_param(name, expand_param!(T))` decl whose `T` does
    /// not match the named parameter's peeled type.
    ParamTypeMismatch {
        func: syn::Ident,
        param: syn::Ident,
        declared: String,
        actual: String,
    },
    /// Recursive input reached a type already on the build chain (`A → … → A`).
    InputCycle {
        ty: String,
        /// Where in the construction — see
        /// [`ExpandError::UnsupportedRecursive::at`].
        at: String,
    },
    /// A recursive-input shape that is declared-but-not-yet-supported (recursion
    /// under a selector-dispatched variant, or on an `Option<…>` parameter).
    UnsupportedRecursive {
        func: syn::Ident,
        reason: &'static str,
        /// Where in the construction the transformation failed: the chain of
        /// parameter names from the expanded parameter down to the argument
        /// that could not be built, which is also the prefix its wire slots are
        /// named with. A constructor argument may itself be constructed, so
        /// naming only the function leaves the reader to work out which nesting
        /// of it is the one that failed.
        at: String,
    },
    /// Structurally invalid declaration records — empty variant lists or
    /// duplicate targets. All offenders are collected before failing
    /// (mirrors `ScanError::DeclaredNotFound`).
    InvalidDeclarations {
        entries: Vec<ExpandDeclError>,
    },
}

/// One structurally invalid expansion declaration (see
/// [`ExpandError::InvalidDeclarations`]).
#[derive(Debug)]
pub enum ExpandDeclError {
    /// A constructor declaration with no variants.
    EmptyConstructor { target: String },
    /// A per-fn expand with an empty variant subset.
    EmptySubset { func: syn::Ident, param: syn::Ident },
    /// Two constructor declarations for the same target type.
    DuplicateConstructor { target: String },
    /// Two per-fn expands for the same `(fn, param)`.
    DuplicateExpand { func: syn::Ident, param: syn::Ident },
}

impl std::fmt::Display for ExpandDeclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandDeclError::EmptyConstructor { target } => {
                write!(f, "constructor for `{target}` declares no variants")
            }
            ExpandDeclError::EmptySubset { func, param } => write!(
                f,
                "expand for parameter `{param}` of `{func}` declares no variants"
            ),
            ExpandDeclError::DuplicateConstructor { target } => {
                write!(f, "duplicate constructor declaration for `{target}`")
            }
            ExpandDeclError::DuplicateExpand { func, param } => write!(
                f,
                "duplicate expand declaration for parameter `{param}` of `{func}`"
            ),
        }
    }
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::UnknownFunction(name) => {
                write!(f, "expand: function `{}` is not a #[prebindgen] item", name)
            }
            ExpandError::ConstructOnAccessor { func } => write!(
                f,
                "expand: param-variant override on accessor fn `{}` — an accessor is never \
                 parameter-composed (remove the override, or declare it as `.fun`)",
                func
            ),
            ExpandError::InputCycle { ty, at } => write!(
                f,
                "expand at `{}`: recursive input forms a cycle through `{}` — a constructor \
                 parameter's type transitively constructs itself",
                at, ty
            ),
            ExpandError::UnsupportedRecursive { func, reason, at } => write!(
                f,
                "expand: `{}` at `{}`: recursive input not supported here: {}",
                func, at, reason
            ),
            ExpandError::UnknownParam(func, param) => write!(
                f,
                "expand: function `{}` has no parameter named `{}`",
                func, param
            ),
            ExpandError::ParamTypeMismatch {
                func,
                param,
                declared,
                actual,
            } => write!(
                f,
                "expand: `{}`.expand_param(\"{}\", expand_param!({declared})): the parameter's \
                 type is `{actual}`, not `{declared}` — declare the decl for the parameter's \
                 actual type",
                func, param
            ),
            ExpandError::UnknownConstructor(name) => write!(
                f,
                "expand: constructor `{}` is not a #[prebindgen] item",
                name
            ),
            ExpandError::NoConstructor {
                func,
                param,
                target,
            } => write!(
                f,
                "expand: no constructor registered for `{}` (parameter `{}` of `{}`)",
                target, param, func
            ),
            ExpandError::TargetMismatch {
                ctor,
                produces,
                expected,
            } => write!(
                f,
                "expand: constructor `{}` produces `{}` but the parameter expects `{}`",
                ctor, produces, expected
            ),
            ExpandError::UnsupportedOptional {
                func,
                param,
                reason,
                at,
            } => write!(
                f,
                "expand: optional parameter `{}` of `{}` is not supported at `{}`: {}",
                param, func, at, reason
            ),
            ExpandError::InvalidDeclarations { entries } => {
                writeln!(f, "expand: invalid declarations:")?;
                for e in entries {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ExpandError {}
