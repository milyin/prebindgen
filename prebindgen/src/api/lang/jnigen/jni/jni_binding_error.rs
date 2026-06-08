//! Framework error type for the JNI binding's error channel.
//!
//! A fallible `#[prebindgen] fn f(...) -> Result<T, E>` is delivered to the
//! foreign side as `Result<T, JniBindingError<E>>`:
//!
//! * [`JniBindingError::JniError`] — a **binding** failure (UTF-8 decode of a
//!   `JString`, `instanceof`/null check, struct field read, handle wrap, closed
//!   handle, …). Framework-built converters compose their `?` failures here via
//!   `From<String>`; they are E-agnostic and use `JniBindingError<()>`.
//! * [`JniBindingError::UserError`] — the function's **domain** error `E` (e.g.
//!   `zenoh::Error`). The `Result<T, E>` peel surfaces it on its own arm.
//!
//! The generated wrapper's error callback receives a fixed first `je: String?`
//! (the binding message, set only on `JniError`) plus the domain error
//! converted/deconstructed into one or more leaves (set only on `UserError`).
//! `JniGen::new()` pre-registers this type so `__JniErr` (= `JniBindingError<()>`)
//! is always available to framework converter bodies.

/// Framework error type for the JNI binding's error channel. `T` is the
/// function's domain error (`()` for the E-agnostic framework converters, whose
/// failures are always [`Self::JniError`]).
#[derive(Clone)]
pub enum JniBindingError<T> {
    /// A binding-layer failure, carrying a context message.
    JniError(String),
    /// The wrapped function's domain error.
    UserError(T),
}

impl<T> From<String> for JniBindingError<T> {
    fn from(s: String) -> Self {
        JniBindingError::JniError(s)
    }
}

// `Display`/`Debug` are unconditional in `T` (no `T: Display` bound) so the
// framework's `__JniErr = JniBindingError<()>` (binding errors only) satisfies
// them. The `UserError` arm is never displayed in practice — the domain error is
// decomposed via its `convert_error`/`deconstruct_error` plan, not stringified.
impl<T> core::fmt::Display for JniBindingError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JniBindingError::JniError(s) => f.write_str(s),
            JniBindingError::UserError(_) => f.write_str("<user error>"),
        }
    }
}

impl<T> core::fmt::Debug for JniBindingError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JniBindingError::JniError(s) => write!(f, "JniError({s:?})"),
            JniBindingError::UserError(_) => f.write_str("UserError(..)"),
        }
    }
}
