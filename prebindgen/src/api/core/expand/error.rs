//! Errors produced while resolving constructor-expansion declarations.

/// Errors surfaced while resolving [`Expansions`](super::Expansions) in
/// [`apply`](super::apply).
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
    },
    /// A recursive-input shape that is declared-but-not-yet-supported (recursion
    /// under a selector-dispatched variant, or on an `Option<…>` parameter).
    UnsupportedRecursive {
        func: syn::Ident,
        reason: &'static str,
    },
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
            ExpandError::InputCycle { ty } => write!(
                f,
                "expand: recursive input forms a cycle through `{}` — a constructor \
                 parameter's type transitively constructs itself",
                ty
            ),
            ExpandError::UnsupportedRecursive { func, reason } => write!(
                f,
                "expand: `{}`: recursive input not supported here: {}",
                func, reason
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
            } => write!(
                f,
                "expand: optional parameter `{}` of `{}` is not supported: {}",
                param, func, reason
            ),
        }
    }
}

impl std::error::Error for ExpandError {}
