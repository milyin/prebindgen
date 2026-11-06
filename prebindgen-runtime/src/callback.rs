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
    /// Track an inline owner stored in callback-result memory until acquisition.
    /// This does not free the foreign storage itself.
    ///
    /// # Safety
    /// Unconsumed entries must remain readable and hold a live `T` until this
    /// guard is committed or dropped. In tracking mode, consumed entries and
    /// their aliases may already have been moved or destroyed. Scan order must
    /// match the decoder's consumption order; unconsumed values have no other owner.
    pub unsafe fn receive_inline<T>(&mut self, pointer: *const T) {
        unsafe fn release<T>(value: *mut ()) {
            drop(core::ptr::read(value as *const T));
        }
        self.values
            .push((pointer.cast_mut().cast::<()>(), release::<T>, false));
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
/// Generated C adapters retain stable foreign rows until this guard is dropped.
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

/// Stack storage for the bounded set of owners in one callback result.
///
/// The generator computes `N` from the result's fields and active alternatives.
/// Handle-bearing collections use [`CallbackReturnAllocations`]. Like its
/// tracking mode, this guard leaves alias validation to the input decoder.
pub struct FixedCallbackReturnAllocations<'a, const N: usize> {
    consumed: &'a core::cell::Cell<usize>,
    values: [core::mem::MaybeUninit<ReceivedAllocation>; N],
    len: usize,
    failure: Option<String>,
}
impl<'a, const N: usize> FixedCallbackReturnAllocations<'a, N> {
    /// Reclaim result owners not acquired by the decoder's consumed-prefix count.
    pub fn tracking(consumed: &'a core::cell::Cell<usize>) -> Self {
        Self {
            consumed,
            values: [core::mem::MaybeUninit::uninit(); N],
            len: 0,
            failure: None,
        }
    }
    /// Receive a callback-owned pointer; null values do not occupy storage.
    ///
    /// # Safety
    /// Each non-null, aligned pointer must transfer a live `Box<T>`. Repeated
    /// addresses must have the same type.
    ///
    /// # Panics
    /// Panics when a valid, non-null owner would exceed the capacity `N`.
    #[inline]
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
            self.push((pointer.cast::<()>(), release::<T>, false));
        }
    }
    /// Receive an inline owner without freeing its foreign storage.
    ///
    /// # Safety
    /// `pointer` must remain readable and contain a live `T` until acquisition or
    /// cleanup. Repeated addresses must have the same type.
    ///
    /// # Panics
    /// Panics if pointer and inline owners together would exceed capacity `N`.
    #[inline]
    pub unsafe fn receive_inline<T>(&mut self, pointer: *const T) {
        unsafe fn release<T>(value: *mut ()) {
            drop(core::ptr::read(value as *const T));
        }
        self.push((pointer.cast_mut().cast::<()>(), release::<T>, false));
    }
    #[inline]
    fn push(&mut self, value: ReceivedAllocation) {
        self.values[self.len].write(value);
        self.len += 1;
    }
    /// Report invalid pointer encodings before the decoder acquires any owner.
    pub fn check(&self) -> Result<()> {
        self.failure
            .as_ref()
            .map_or(Ok(()), |message| Err(message.clone()))
    }
}
impl<const N: usize> Drop for FixedCallbackReturnAllocations<'_, N> {
    fn drop(&mut self) {
        if self.consumed.get() != self.len {
            // The initialized prefix contains only pointer/function/bool tuples.
            // Moving the guard never moves the referenced foreign allocations.
            let values = unsafe {
                core::slice::from_raw_parts_mut(
                    self.values.as_mut_ptr().cast::<ReceivedAllocation>(),
                    self.len,
                )
            };
            release_unconsumed(values, self.consumed.get());
        }
    }
}

// Both storage modes preserve the scan order until decoding has finished. Only
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

/// Heap-free callback owner guard whose ownership is taken by an ordered decoder.
pub struct TransferredOwner<'a, T> {
    value: core::mem::ManuallyDrop<T>,
    consumed: &'a core::cell::Cell<usize>,
    ordinal: usize,
}
impl<'a, T> TransferredOwner<'a, T> {
    /// Guard one owner until its corresponding input slot has been acquired.
    pub fn new(value: T, consumed: &'a core::cell::Cell<usize>, ordinal: usize) -> Self {
        Self {
            value: core::mem::ManuallyDrop::new(value),
            consumed,
            ordinal,
        }
    }
}
impl<T> Drop for TransferredOwner<'_, T> {
    fn drop(&mut self) {
        if self.consumed.get() <= self.ordinal {
            unsafe { core::mem::ManuallyDrop::drop(&mut self.value) }
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
    fn fixed_result_cleanup_deduplicates_and_continues_after_drop_failure() {
        let drops = Arc::new(AtomicUsize::new(0));
        let first = Box::into_raw(Box::new(Owner(drops.clone(), true)));
        let second = Box::into_raw(Box::new(Owner(drops.clone(), false)));
        let consumed = core::cell::Cell::new(0);
        let mut owners = FixedCallbackReturnAllocations::<3>::tracking(&consumed);
        unsafe {
            owners.receive(first);
            owners.receive(first);
            owners.receive(second);
        }
        // Tracking guards delegate alias rejection to the generated decoder.
        owners.check().unwrap();
        drop(owners);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn fixed_result_rejects_invalid_encoding_and_reclaims_later_owners() {
        let drops = Arc::new(AtomicUsize::new(0));
        let valid = Box::into_raw(Box::new(Owner(drops.clone(), false)));
        let consumed = core::cell::Cell::new(0);
        let mut owners = FixedCallbackReturnAllocations::<2>::tracking(&consumed);
        unsafe {
            owners.receive(valid.cast::<u8>().wrapping_add(1).cast::<Owner>());
            owners.receive(valid);
        }
        assert!(owners.check().is_err());
        drop(owners);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
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
    fn fixed_cleanup_skips_null_and_consumed_aliases() {
        let drops = Rc::new(Cell::new(0));
        let consumed = Cell::new(0);
        let a = Box::into_raw(Box::new(Owner(drops.clone())));
        let b = Box::into_raw(Box::new(Owner(drops.clone())));
        let mut guard = FixedCallbackReturnAllocations::<3>::tracking(&consumed);
        unsafe {
            guard.receive(core::ptr::null_mut::<Owner>());
            guard.receive(a);
            guard.receive(a);
            guard.receive(b);
        }
        guard.check().unwrap();
        consumed.set(1);
        unsafe {
            drop(Box::from_raw(a));
        }
        drop(guard);
        assert_eq!(drops.get(), 2);
    }
    #[test]
    fn fixed_cleanup_handles_mixed_inline_and_pointer_owners() {
        for taken in 0..=2 {
            let drops = Rc::new(Cell::new(0));
            let consumed = Cell::new(0);
            let mut inline = core::mem::ManuallyDrop::new(Owner(drops.clone()));
            let pointer = Box::into_raw(Box::new(Owner(drops.clone())));
            let mut guard = FixedCallbackReturnAllocations::<2>::tracking(&consumed);
            unsafe {
                guard.receive_inline(&*inline);
                guard.receive(pointer);
            }
            guard.check().unwrap();
            if taken >= 1 {
                unsafe {
                    core::mem::ManuallyDrop::drop(&mut inline);
                }
            }
            if taken == 2 {
                unsafe {
                    drop(Box::from_raw(pointer));
                }
            }
            consumed.set(taken);
            drop(guard);
            assert_eq!(drops.get(), 2);
        }
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
    #[test]
    fn deferred_rescan_skips_consumed_inline_storage() {
        let drops = Rc::new(Cell::new(0));
        let consumed = Cell::new(0);
        let mut first = core::mem::ManuallyDrop::new(Owner(drops.clone()));
        let second = core::mem::ManuallyDrop::new(Owner(drops.clone()));
        let a = core::ptr::addr_of!(first).cast::<Owner>();
        let b = core::ptr::addr_of!(second).cast::<Owner>();
        let cleanup = CallbackReturnCleanup::new(|| {
            let mut owners = CallbackReturnAllocations::tracking(&consumed);
            unsafe {
                owners.receive_inline(a);
                owners.receive_inline(b);
            }
            owners.check()
        });
        consumed.set(1);
        unsafe {
            core::mem::ManuallyDrop::drop(&mut first);
        }
        drop(cleanup);
        assert_eq!(drops.get(), 2);
    }
    #[test]
    fn stack_guards_release_only_unread_slots() {
        let drops = Rc::new(Cell::new(0));
        let consumed = Cell::new(0);
        let first = TransferredOwner::new(Owner(drops.clone()), &consumed, 0);
        let second = TransferredOwner::new(Owner(drops.clone()), &consumed, 1);
        // Simulate the decoder moving this owner out of its guarded storage.
        let acquired = unsafe { core::ptr::read(&*first.value) };
        consumed.set(1);
        drop(first);
        drop(second);
        assert_eq!(drops.get(), 1);
        drop(acquired);
        assert_eq!(drops.get(), 2);
    }
}
