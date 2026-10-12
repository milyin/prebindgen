//! Panic containment for generated foreign boundaries. Storage belongs to adapters.
pub type Result<T> = std::result::Result<T, String>;

/// An exported boundary never unwinds through a foreign caller.
pub fn boundary<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("Rust operation panicked".into()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn panics_do_not_escape() {
        assert!(super::boundary::<()>(|| panic!("test")).is_err());
    }
}
