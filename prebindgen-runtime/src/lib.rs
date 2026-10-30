//! Panic containment and pointer validation for foreign adapters.
#![warn(missing_docs)]
mod input;
pub use input::{DynamicInputs, FixedInputs, SharedInputs};
mod callback;
#[cfg(test)]
mod diagnostics_tests;
pub use callback::{CallbackReturnAllocations, TransferredBox, TransferredOwner};

/// Runtime failure represented as a message for the adapter's error channel.
pub type Result<T> = std::result::Result<T, String>;

/// Report a contained callback failure through the application's tracing subscriber.
///
/// Called only on failure. Successful callbacks need no diagnostic work.
#[cold]
#[inline(never)]
pub fn report_callback_error(callback: &str, signature: &str, error: &str) {
    let _ = boundary(|| {
        if no_subscriber() {
            eprintln!(
                "prebindgen callback error [callback={callback}, signature={signature}]: {error}"
            );
        } else {
            tracing::error!(target: "prebindgen::callback", callback, signature, error,
                "foreign callback failed");
        }
        Ok(())
    });
}

/// Report failure to reclaim a returned callback resource, after containing its panic.
#[cold]
#[inline(never)]
pub fn report_callback_cleanup_error(error: &str) {
    let _ = boundary(|| {
        if no_subscriber() {
            eprintln!("prebindgen callback result cleanup error: {error}");
        } else {
            tracing::error!(target: "prebindgen::callback", error, "callback result cleanup failed");
        }
        Ok(())
    });
}

fn no_subscriber() -> bool {
    tracing::dispatcher::get_default(|dispatcher| {
        dispatcher.is::<tracing::subscriber::NoSubscriber>()
    })
}

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

/// Validate array pointer encoding and addressable byte length without reading memory.
/// Empty arrays accept null. Nonempty arrays require nonzero-sized elements,
/// nonnull aligned storage and a byte range fitting isize and the address space.
/// The caller must still guarantee the entire range is readable and has valid provenance.
pub fn validate_slice<T>(pointer: *const T, length: usize) -> Result<()> {
    if length == 0 {
        return Ok(());
    }
    validate_pointer(pointer)?;
    let size = std::mem::size_of::<T>();
    if size == 0 {
        return Err("zero-sized array element carrier".into());
    }
    let bytes = length
        .checked_mul(size)
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or("array byte length exceeds addressable range")?;
    pointer
        .addr()
        .checked_add(bytes)
        .ok_or("array address range overflow")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slice_encoding_checks() {
        let values = [1u64, 2];
        assert!(validate_slice(values.as_ptr(), 2).is_ok());
        assert!(validate_slice::<u64>(std::ptr::null(), 0).is_ok());
        assert!(validate_slice::<u64>(std::ptr::null(), 1).is_err());
        assert!(validate_slice(
            std::ptr::NonNull::<u8>::dangling().as_ptr().cast::<u64>(),
            1
        )
        .is_err());
        assert!(validate_slice(values.as_ptr(), usize::MAX).is_err());
        assert!(validate_slice((usize::MAX - 7) as *const u64, 1).is_err());
        assert!(validate_slice(std::ptr::NonNull::<()>::dangling().as_ptr(), 1).is_err());
    }
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
