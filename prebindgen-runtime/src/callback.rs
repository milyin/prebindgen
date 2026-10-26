//! Ownership of allocations received together as a callback result.
use crate::{boundary, validate_pointer, Result};

/// Reclaims owned callback-result allocations if another returned value is invalid.
///
/// Generated adapters use this only for composite results containing handles.
/// Unit, scalar and single-handle results do not allocate this collection.
#[derive(Default)]
pub struct CallbackReturnAllocations {
    values: Vec<(usize, unsafe fn(usize))>,
    failure: Option<String>,
}
impl CallbackReturnAllocations {
    /// Take responsibility for a non-null callback-result allocation.
    /// Duplicate addresses and invalid encodings are recorded for [`Self::check`].
    ///
    /// # Safety
    /// Each non-null, aligned address must be a live `Box<T>` allocation whose
    /// ownership is being transferred by this callback. Repeated addresses must
    /// refer to the same allocation and type.
    pub unsafe fn receive<T>(&mut self, pointer: *mut T) {
        if pointer.is_null() {
            return;
        }
        if let Err(error) = validate_pointer(pointer) {
            self.failure.get_or_insert(error);
        } else {
            unsafe fn release<T>(value: usize) {
                drop(Box::from_raw(value as *mut T));
            }
            self.values.push((pointer as usize, release::<T>));
        }
    }
    /// Report an invalid pointer encoding or duplicate ownership transfer.
    pub fn check(&mut self) -> Result<()> {
        self.deduplicate();
        self.failure
            .as_ref()
            .map_or(Ok(()), |message| Err(message.clone()))
    }
    fn deduplicate(&mut self) {
        self.values.sort_unstable_by_key(|(pointer, _)| *pointer);
        let before = self.values.len();
        self.values.dedup_by_key(|(pointer, _)| *pointer);
        if before != self.values.len() {
            self.failure
                .get_or_insert_with(|| "conflicting callback return aliases".into());
        }
    }
    /// Hand responsibility to the prepared input conversion's infallible acquisition.
    pub fn commit(&mut self) {
        self.values.clear();
    }
}
impl Drop for CallbackReturnAllocations {
    fn drop(&mut self) {
        self.deduplicate();
        for (pointer, release) in self.values.drain(..) {
            if let Err(error) = boundary(|| {
                unsafe { release(pointer) };
                Ok(())
            }) {
                eprintln!("prebindgen callback result cleanup error: {error}");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    struct Owner(Arc<AtomicUsize>, bool);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
            assert!(!self.1, "expected drop failure");
        }
    }
    #[test]
    fn result_owners_roll_back_once_and_continue_after_drop_failure() {
        let drops = Arc::new(AtomicUsize::new(0));
        let first = Box::into_raw(Box::new(Owner(drops.clone(), true)));
        let second = Box::into_raw(Box::new(Owner(drops.clone(), false)));
        let mut owners = CallbackReturnAllocations::default();
        unsafe {
            owners.receive(first);
            owners.receive(first);
            owners.receive(second);
        }
        assert!(owners.check().unwrap_err().contains("aliases"));
        drop(owners);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn result_owner_commit_leaves_allocations_for_acquisition() {
        let drops = Arc::new(AtomicUsize::new(0));
        let pointer = Box::into_raw(Box::new(Owner(drops.clone(), false)));
        let mut owners = CallbackReturnAllocations::default();
        unsafe {
            owners.receive(pointer);
        }
        owners.check().unwrap();
        owners.commit();
        drop(owners);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        unsafe {
            drop(Box::from_raw(pointer));
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
