//! Ownership of allocations received together as a callback result.
use crate::{boundary, validate_pointer, Result};

type ReceivedAllocation = (*mut (), unsafe fn(*mut ()), bool);

/// Reclaims owned callback-result allocations if another returned value is invalid.
///
/// Generated adapters use this only for composite results containing handles.
/// Unit, scalar and single-handle results do not allocate this collection.
#[derive(Default)]
pub struct CallbackReturnAllocations<'a> {
    consumed: Option<&'a core::cell::Cell<usize>>,
    values: Vec<ReceivedAllocation>,
    failure: Option<String>,
}
impl<'a> CallbackReturnAllocations<'a> {
    /// Reclaim only callback owners not already acquired by eager decoding.
    pub fn tracking(consumed: &'a core::cell::Cell<usize>) -> Self {
        Self {
            consumed: Some(consumed),
            values: Vec::new(),
            failure: None,
        }
    }
    /// Take responsibility for a non-null callback-result allocation.
    /// Duplicate addresses and invalid encodings are recorded for [`Self::check`].
    ///
    /// # Safety
    /// Each non-null, aligned address must be a live `Box<T>` allocation whose
    /// ownership is being transferred by this callback. Repeated addresses must
    /// refer to the same allocation and type. With [`Self::tracking`], entries in
    /// the consumed prefix and their aliases may already have been acquired or
    /// destroyed; all other entries must still own live allocations. Scan order
    /// must match the decoder's consumption order.
    pub unsafe fn receive<T>(&mut self, pointer: *mut T) {
        if pointer.is_null() {
            return;
        }
        if let Err(error) = validate_pointer(pointer) {
            self.failure.get_or_insert(error);
        } else {
            unsafe fn release<T>(value: *mut ()) {
                drop(Box::from_raw(value as *mut T));
            }
            self.values
                .push((pointer.cast::<()>(), release::<T>, false));
        }
    }
    /// Report an invalid pointer encoding or duplicate ownership transfer.
    pub fn check(&mut self) -> Result<()> {
        if self.consumed.is_none() {
            self.deduplicate();
        }
        self.failure
            .as_ref()
            .map_or(Ok(()), |message| Err(message.clone()))
    }
    fn deduplicate(&mut self) {
        self.values.sort_unstable_by_key(|(pointer, _, _)| *pointer);
        let before = self.values.len();
        self.values.dedup_by(|a, b| {
            if a.0 == b.0 {
                b.2 |= a.2;
                true
            } else {
                false
            }
        });
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
impl Drop for CallbackReturnAllocations<'_> {
    fn drop(&mut self) {
        let consumed = self.consumed.map_or(0, core::cell::Cell::get);
        if consumed != self.values.len() {
            release_unconsumed(&mut self.values, consumed);
        }
    }
}

/// Defers a callback result's rollback scan until decoding fails or unwinds.
///
/// Generated adapters retain stable native addresses or rows until this guard is dropped.
/// The cleanup closure may allocate on failure, but is never run after commit.
pub struct CallbackReturnCleanup<F: FnOnce() -> Result<()>> {
    cleanup: Option<F>,
}
impl<F: FnOnce() -> Result<()>> CallbackReturnCleanup<F> {
    /// Arm cleanup before acquiring any returned owners.
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }
    /// Decoding succeeded; Rust values now own all transferred resources.
    pub fn commit(&mut self) {
        let _ = self.cleanup.take();
    }
}
impl<F: FnOnce() -> Result<()>> Drop for CallbackReturnCleanup<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            failed_callback_cleanup(cleanup);
        }
    }
}
#[cold]
fn failed_callback_cleanup(cleanup: impl FnOnce() -> Result<()>) {
    if let Err(error) = boundary(cleanup) {
        crate::report_callback_cleanup_error(&error);
    }
}

// Received owners preserve the scan order until decoding has finished. Only
// failure cleanup sorts addresses, combining taken flags before releasing a
// duplicate. This keeps a consumed alias from ever being destroyed twice.
#[cold]
fn release_unconsumed(values: &mut [ReceivedAllocation], consumed: usize) {
    debug_assert!(
        consumed <= values.len(),
        "callback owner scan and decoder disagree"
    );
    for value in values.iter_mut().take(consumed) {
        value.2 = true;
    }
    values.sort_unstable_by_key(|(pointer, _, _)| *pointer);
    let mut index = 0;
    while index < values.len() {
        let (pointer, release, mut taken) = values[index];
        index += 1;
        while index < values.len() && values[index].0 == pointer {
            taken |= values[index].2;
            index += 1;
        }
        if !taken {
            if let Err(error) = boundary(|| {
                unsafe { release(pointer) };
                Ok(())
            }) {
                crate::report_callback_cleanup_error(&error);
            }
        }
    }
}

/// Heap-free guard for a callback allocation before eager decoding takes it.
pub struct TransferredBox<'a, T> {
    pointer: *mut T,
    consumed: &'a core::cell::Cell<usize>,
}
impl<'a, T> TransferredBox<'a, T> {
    /// Guard a callback-owned Box allocation, allowing null for inactive slots.
    ///
    /// # Safety
    /// A nonnull pointer must transfer a live `Box<T>` allocation to this guard.
    pub unsafe fn new(pointer: *mut T, consumed: &'a core::cell::Cell<usize>) -> Result<Self> {
        if !pointer.is_null() {
            validate_pointer(pointer)?;
        }
        Ok(Self { pointer, consumed })
    }
}
impl<T> Drop for TransferredBox<'_, T> {
    fn drop(&mut self) {
        if !self.pointer.is_null() && self.consumed.get() == 0 {
            unsafe {
                drop(Box::from_raw(self.pointer));
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
    fn deferred_cleanup_skips_success_and_contains_cleanup_panics() {
        let calls = core::cell::Cell::new(0);
        let mut committed = CallbackReturnCleanup::new(|| {
            calls.set(calls.get() + 1);
            Ok(())
        });
        committed.commit();
        drop(committed);
        assert_eq!(calls.get(), 0);
        let result = boundary::<()>(|| {
            let _cleanup = CallbackReturnCleanup::new(|| -> Result<()> {
                calls.set(calls.get() + 1);
                panic!("cleanup failure")
            });
            panic!("decoding failure")
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 1);
    }
    #[test]
    fn deferred_cleanup_reclaims_unread_owners_after_decoder_unwinds() {
        let drops = Arc::new(AtomicUsize::new(0));
        let rows = [
            Box::into_raw(Box::new(Owner(drops.clone(), true))),
            Box::into_raw(Box::new(Owner(drops.clone(), false))),
        ];
        let consumed = core::cell::Cell::new(0);
        assert!(boundary::<()>(|| {
            let _cleanup = CallbackReturnCleanup::new(|| {
                let mut owners = CallbackReturnAllocations::tracking(&consumed);
                for pointer in rows {
                    unsafe {
                        owners.receive(pointer);
                    }
                }
                owners.check()
            });
            panic!("constructor failure")
        })
        .is_err());
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
#[cfg(test)]
mod eager_tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    struct Owner(Rc<Cell<usize>>);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    #[test]
    fn partial_callback_decode_reclaims_unread_owners_and_skips_taken_aliases() {
        let drops = Rc::new(Cell::new(0));
        let consumed = Cell::new(0);
        let a = Box::into_raw(Box::new(Owner(drops.clone())));
        let b = Box::into_raw(Box::new(Owner(drops.clone())));
        let mut guard = CallbackReturnAllocations::tracking(&consumed);
        unsafe {
            guard.receive(a);
            guard.receive(a);
            guard.receive(b);
        }
        guard.check().unwrap();
        let value = unsafe { Box::from_raw(a) };
        consumed.set(1);
        drop(value);
        drop(guard);
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn deferred_rescan_accepts_consumed_dangling_pointers_and_their_aliases() {
        let drops = Rc::new(Cell::new(0));
        let consumed = Cell::new(0);
        let a = Box::into_raw(Box::new(Owner(drops.clone())));
        let b = Box::into_raw(Box::new(Owner(drops.clone())));
        let rows = [a, a, b];
        let cleanup = CallbackReturnCleanup::new(|| {
            let mut owners = CallbackReturnAllocations::tracking(&consumed);
            for pointer in rows {
                unsafe {
                    owners.receive(pointer);
                }
            }
            owners.check()
        });
        consumed.set(1);
        unsafe {
            drop(Box::from_raw(a));
        }
        drop(cleanup);
        assert_eq!(drops.get(), 2);
    }
}
