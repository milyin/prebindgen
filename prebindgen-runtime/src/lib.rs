//! Panic containment and optional allocation support for foreign adapters.
pub type Result<T> = std::result::Result<T, String>;

/// An exported boundary never unwinds through a foreign caller.
pub fn boundary<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("Rust operation panicked".into()))
}

/// Optional allocation used by adapters that choose boxed-value storage.
/// The anchor gives zero-sized values distinct addresses. Minimum alignment
/// leaves a low address bit available for adapters that tag pointer state.
#[repr(C, align(2))]
pub struct Allocation<T> {
    pub value: T,
    // Alignment alone does not make a zero-sized type occupy storage.
    _anchor: u8,
}
impl<T> Allocation<T> {
    pub fn new(value: T) -> Box<Self> {
        Box::new(Self { value, _anchor: 0 })
    }
}
/// Check encoding only; the adapter/caller must guarantee type, liveness and
/// synchronization. This helper neither registers objects nor tracks ownership.
pub fn validate_allocation<T>(pointer: *const Allocation<T>) -> Result<()> {
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
    fn zero_sized_objects_have_distinct_taggable_addresses() {
        let a = Allocation::new(());
        let b = Allocation::new(());
        let pa = (&*a) as *const Allocation<()>;
        let pb = (&*b) as *const Allocation<()>;
        assert_ne!(pa, pb);
        assert!(validate_allocation(pa).is_ok());
        assert!(validate_allocation((pa as usize | 1) as *const Allocation<()>).is_err());
        assert!(validate_allocation::<()>(std::ptr::null()).is_err());
    }
    #[test]
    fn unpublished_and_transferred_objects_drop_once() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        struct Tracked(Arc<AtomicUsize>);
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let count = Arc::new(AtomicUsize::new(0));
        drop(Allocation::new(Tracked(count.clone())));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        let raw = Box::into_raw(Allocation::new(Tracked(count.clone())));
        unsafe {
            drop(Box::from_raw(raw));
        }
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let raw = Box::into_raw(Allocation::new(Tracked(count.clone())));
        let moved = unsafe { Box::from_raw(raw) }.value;
        assert_eq!(count.load(Ordering::SeqCst), 2);
        drop(moved);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }
}
