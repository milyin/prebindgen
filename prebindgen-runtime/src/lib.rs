//! Panic containment and pointer validation for foreign adapters.
#![warn(missing_docs)]
/// Runtime failure represented as a message for the adapter's error channel.
pub type Result<T> = std::result::Result<T, String>;

/// Run a boundary operation, converting unwinding panics into a generic error.
///
/// `f` is the body of the exported call; its error message passes through to
/// the adapter's error channel. Ordinary errors pass through unchanged. This does not roll back side effects
/// or ownership transfers, and cannot catch a panic configured to abort.
///
/// ```
/// use prebindgen_runtime::boundary;
/// assert_eq!(boundary(|| Ok(42)), Ok(42));
/// assert_eq!(boundary::<()>(|| Err("invalid input".into())),
///            Err("invalid input".into()));
/// ```
pub fn boundary<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("Rust operation panicked".into()))
}

/// Check that `pointer`, an allocation address read from a boundary wire, is
/// non-null and aligned for `T`.
///
/// Check encoding only; the adapter/caller must guarantee type, liveness and
/// synchronization. This helper neither registers objects nor tracks ownership.
/// It never dereferences the pointer. Success does not establish that dereferencing
/// or freeing it is safe; zero-sized pointees are not rejected by this check.
pub fn validate_pointer<T>(pointer: *const T) -> Result<()> {
    if pointer.is_null() || !pointer.is_aligned() {
        return Err("null or misaligned allocation pointer".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panics_do_not_escape() {
        assert!(super::boundary::<()>(|| panic!("test")).is_err());
    }
    #[test]
    fn pointer_encoding_checks() {
        let value = 1u64;
        let pointer = &value as *const u64;
        assert!(validate_pointer(pointer).is_ok());
        assert!(validate_pointer((pointer as usize | 1) as *const u64).is_err());
        assert!(validate_pointer::<u64>(std::ptr::null()).is_err());
    }
}
