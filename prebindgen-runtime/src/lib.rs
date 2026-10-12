//! Panic containment and pointer validation for foreign adapters.
pub type Result<T> = std::result::Result<T, String>;

/// An exported boundary never unwinds through a foreign caller.
pub fn boundary<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("Rust operation panicked".into()))
}

/// Check encoding only; the adapter/caller must guarantee type, liveness and
/// synchronization. This helper neither registers objects nor tracks ownership.
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
