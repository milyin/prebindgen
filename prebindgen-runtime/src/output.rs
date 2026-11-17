//! Ownership of initialized rows before a foreign collection result is delivered.
use core::{marker::PhantomData, mem::MaybeUninit};

/// Final row storage that reclaims owned wire fields if output conversion fails.
///
/// Rows are `Copy` foreign representations; their owned fields need a generated
/// destructor rather than Rust's ordinary field destruction. Storage is allocated
/// once at its final length. Completed rows need no separate ownership queue.
pub struct OutputRows<T: Copy> {
    data: *mut MaybeUninit<T>,
    length: usize,
    initialized: usize,
    release: unsafe fn(&T),
    marker: PhantomData<T>,
}
impl<T: Copy> OutputRows<T> {
    /// Allocate final storage and register destruction of completed rows.
    ///
    /// # Safety
    /// `release` must reclaim exactly the additional resources owned by a
    /// completed row, including nested row allocations, and skip borrowed fields.
    /// It must contain destructor panics per field so other fields are reclaimed.
    /// The generated initializer must transfer each row's resources exactly once
    /// before calling [`Self::commit_row`].
    #[inline]
    pub unsafe fn new(length: usize, release: unsafe fn(&T)) -> Self {
        // Retain a raw pointer, not a Box: moving the guard must not retag storage
        // after its C pointer has been exposed.
        let data = Box::into_raw(Box::<[T]>::new_uninit_slice(length)).cast();
        Self {
            data,
            length,
            initialized: 0,
            release,
            marker: PhantomData,
        }
    }
    /// Store the next row before transferring its resources to this guard.
    ///
    /// # Safety
    /// Fewer than `len()` rows must have completed. The row's resources must
    /// remain owned by the initializer until [`Self::commit_row`]. Do not change
    /// completed rows or retain references to this storage across a write.
    #[inline]
    pub unsafe fn write_next(&mut self, row: T) {
        debug_assert!(self.initialized < self.length);
        self.data.add(self.initialized).write(MaybeUninit::new(row));
    }
    /// Take cleanup responsibility for the row most recently written.
    ///
    /// # Safety
    /// The next row must have been written, and all its owned resources must
    /// have transferred to this guard. No fallible or unwinding operation may
    /// separate the resource transfer from this call. Commit each row once.
    #[inline]
    pub unsafe fn commit_row(&mut self) {
        debug_assert!(self.initialized < self.length);
        self.initialized += 1;
    }
    /// Address of final row storage. Only completed rows may be read.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.cast()
    }
    /// Allocated element count, including rows not yet initialized.
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }
    /// Whether this output has no rows.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    /// Relinquish completed storage and its fields to the foreign recipient.
    /// Its generated destructor must free the row allocation as `Box<[T]>`.
    #[inline]
    pub fn commit(self) {
        debug_assert_eq!(self.initialized, self.length);
        core::mem::forget(self);
    }
}
impl<T: Copy> Drop for OutputRows<T> {
    #[inline]
    fn drop(&mut self) {
        // Recover ownership first; derive row references from this Box after
        // recovery, rather than from the earlier raw pointer's provenance.
        let rows =
            unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(self.data, self.length)) };
        for row in &rows[..self.initialized] {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                (self.release)(row.assume_init_ref());
            }));
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

    struct Resource {
        id: usize,
        drops: Arc<[AtomicUsize; 8]>,
        panic: bool,
    }
    impl Drop for Resource {
        fn drop(&mut self) {
            self.drops[self.id].fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic, "injected resource destructor panic");
        }
    }
    #[derive(Clone, Copy)]
    struct Row {
        first: *mut Resource,
        second: *mut Resource,
    }
    unsafe fn release(row: &Row) {
        for pointer in [row.first, row.second] {
            let _ = crate::boundary(|| {
                if !pointer.is_null() {
                    drop(Box::from_raw(pointer));
                }
                Ok(())
            });
        }
    }
    fn counts() -> Arc<[AtomicUsize; 8]> {
        Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)))
    }
    fn resource(drops: &Arc<[AtomicUsize; 8]>, id: usize, panic: bool) -> Box<Resource> {
        Box::new(Resource {
            id,
            drops: drops.clone(),
            panic,
        })
    }
    #[test]
    fn incomplete_rows_remain_with_the_initializer() {
        let drops = counts();
        let mut rows = unsafe { OutputRows::new(3, release) };
        unsafe {
            rows.write_next(Row {
                first: Box::into_raw(resource(&drops, 0, false)),
                second: Box::into_raw(resource(&drops, 1, false)),
            });
            rows.commit_row();
        }
        let mut pending = resource(&drops, 2, false);
        unsafe {
            rows.write_next(Row {
                first: &mut *pending,
                second: core::ptr::null_mut(),
            });
        }
        drop(rows);
        assert_eq!(drops[0].load(Ordering::SeqCst), 1);
        assert_eq!(drops[1].load(Ordering::SeqCst), 1);
        assert_eq!(drops[2].load(Ordering::SeqCst), 0);
        drop(pending);
        assert_eq!(drops[2].load(Ordering::SeqCst), 1);
    }
    #[test]
    fn successful_delivery_and_guard_moves_preserve_the_exported_pointer() {
        let drops = counts();
        let mut rows = unsafe { OutputRows::new(1, release) };
        let pointer = rows.as_ptr();
        fn moved(rows: OutputRows<Row>) -> OutputRows<Row> {
            rows
        }
        rows = moved(rows);
        unsafe {
            rows.write_next(Row {
                first: Box::into_raw(resource(&drops, 0, false)),
                second: core::ptr::null_mut(),
            });
            rows.commit_row();
        }
        assert_eq!(rows.len(), 1);
        assert!(!rows.is_empty());
        rows.commit();
        assert_eq!(drops[0].load(Ordering::SeqCst), 0);
        unsafe {
            let delivered =
                Box::from_raw(core::ptr::slice_from_raw_parts_mut(pointer.cast_mut(), 1));
            release(&delivered[0]);
        }
        assert_eq!(drops[0].load(Ordering::SeqCst), 1);
    }
    #[test]
    fn empty_and_overaligned_buffers_have_the_correct_layout() {
        #[repr(align(128))]
        #[derive(Clone, Copy)]
        struct Aligned(u8);
        unsafe fn no_release(row: &Aligned) {
            assert_eq!(row.0, 42);
        }
        let empty = unsafe { OutputRows::<Aligned>::new(0, no_release) };
        assert!(empty.is_empty());
        drop(empty);
        let mut rows = unsafe { OutputRows::new(1, no_release) };
        assert!(rows.as_ptr().is_aligned());
        unsafe {
            rows.write_next(Aligned(42));
            rows.commit_row();
        }
        drop(rows);
    }
    #[test]
    fn cleanup_continues_when_a_destructor_panics_during_conversion_unwinding() {
        let drops = counts();
        let result = std::panic::catch_unwind({
            let drops = drops.clone();
            move || {
                let mut rows = unsafe { OutputRows::new(2, release) };
                unsafe {
                    rows.write_next(Row {
                        first: Box::into_raw(resource(&drops, 0, true)),
                        second: Box::into_raw(resource(&drops, 1, false)),
                    });
                    rows.commit_row();
                    rows.write_next(Row {
                        first: Box::into_raw(resource(&drops, 2, false)),
                        second: core::ptr::null_mut(),
                    });
                    rows.commit_row();
                }
                panic!("injected conversion panic");
            }
        });
        assert!(result.is_err());
        assert!(drops[..3].iter().all(|n| n.load(Ordering::SeqCst) == 1));
    }
    #[test]
    fn nested_completed_rows_and_a_later_partial_child_are_reclaimed_once() {
        #[derive(Clone, Copy)]
        struct Parent {
            data: *const Row,
            len: usize,
        }
        unsafe fn release_parent(parent: &Parent) {
            if parent.len != 0 {
                let rows = Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                    parent.data.cast_mut(),
                    parent.len,
                ));
                for row in &rows {
                    release(row);
                }
            }
        }
        let drops = counts();
        let mut parents = unsafe { OutputRows::new(2, release_parent) };
        let mut first = unsafe { OutputRows::new(1, release) };
        unsafe {
            first.write_next(Row {
                first: Box::into_raw(resource(&drops, 0, false)),
                second: core::ptr::null_mut(),
            });
            first.commit_row();
            parents.write_next(Parent {
                data: first.as_ptr(),
                len: first.len(),
            });
            first.commit();
            parents.commit_row();
        }
        let mut second = unsafe { OutputRows::new(2, release) };
        unsafe {
            second.write_next(Row {
                first: Box::into_raw(resource(&drops, 1, false)),
                second: core::ptr::null_mut(),
            });
            second.commit_row();
        }
        let mut pending = resource(&drops, 2, false);
        unsafe {
            second.write_next(Row {
                first: &mut *pending,
                second: core::ptr::null_mut(),
            });
        }
        drop(second);
        drop(parents);
        drop(pending);
        assert!(drops[..3].iter().all(|n| n.load(Ordering::SeqCst) == 1));
    }
}
