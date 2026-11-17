//! Native access to C-representation batches without per-field JNI calls.
use crate::Result;
/// Offset and byte/element count; MAX offset represents absence.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Range {
    /// Offset from the beginning of the frame.
    pub offset: u64,
    /// Byte length for buffers; element count for rows.
    pub length: u64,
}
impl Range {
    /// Absent descriptor, distinct from empty present data.
    pub const ABSENT: Self = Self {
        offset: u64::MAX,
        length: 0,
    };
}
const _: () = {
    assert!(std::mem::size_of::<Range>() == 16);
    assert!(std::mem::offset_of!(Range, offset) == 0);
    assert!(std::mem::offset_of!(Range, length) == 8);
};
mod sealed {
    pub trait Sealed {}
}
/// Padding-free scalar accepting every bit pattern. Implementations are sealed.
pub trait Field: sealed::Sealed + Copy {}
macro_rules! fields {($($t:ty),*)=>{$(impl sealed::Sealed for $t {} impl Field for $t {})*};}
fields!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, [u8; 16], Range);
/// Borrowed frame; scalar reads need no alignment guarantee from byte-array storage.
pub struct Reader<'a> {
    bytes: &'a [u8],
}
impl<'a> Reader<'a> {
    /// Retain the complete input storage until conversion and the source call finish.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
    /// Read a padding-free scalar with checked bounds and unaligned access.
    pub fn read<T: Field>(&self, offset: usize) -> Result<T> {
        let end = offset
            .checked_add(std::mem::size_of::<T>())
            .ok_or("bulk field offset overflow")?;
        let bytes = self
            .bytes
            .get(offset..end)
            .ok_or("bulk field outside frame")?;
        // Field is sealed to types without padding or invalid bit patterns.
        Ok(unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<T>()) })
    }
    /// Read and validate a nullable byte range before allocating any source value.
    pub fn buffer(&self, offset: usize) -> Result<Option<&'a [u8]>> {
        self.buffer_range(self.range(offset)?)
    }
    /// Validate a previously read nullable byte descriptor when its field is active.
    pub fn buffer_range(&self, range: Option<Range>) -> Result<Option<&'a [u8]>> {
        let Some(range) = range else {
            return Ok(None);
        };
        let start = usize::try_from(range.offset).map_err(|_| "bulk offset out of range")?;
        let len = usize::try_from(range.length).map_err(|_| "bulk length out of range")?;
        let end = start
            .checked_add(len)
            .ok_or("bulk buffer length overflow")?;
        Ok(Some(
            self.bytes
                .get(start..end)
                .ok_or("bulk buffer outside frame")?,
        ))
    }
    /// Read a nullable row descriptor.
    pub fn range(&self, offset: usize) -> Result<Option<Range>> {
        let r = self.read::<Range>(offset)?;
        Ok((r.offset != u64::MAX).then_some(r))
    }
    /// Validate a whole row array before constructing elements or acquiring handles.
    pub fn rows(&self, range: Option<Range>, stride: usize) -> Result<(usize, usize)> {
        let r = range.ok_or("null bulk collection")?;
        if stride == 0 {
            return Err("zero-sized bulk rows are unsupported".into());
        }
        let start = usize::try_from(r.offset).map_err(|_| "bulk row offset out of range")?;
        let len = usize::try_from(r.length).map_err(|_| "bulk row length out of range")?;
        let end = len
            .checked_mul(stride)
            .and_then(|n| start.checked_add(n))
            .ok_or("bulk row extent overflow")?;
        self.bytes
            .get(start..end)
            .ok_or("bulk rows outside frame")?;
        Ok((start, len))
    }
}
/// Native frame builder; all storage, including padding, is initialized.
pub struct Writer {
    bytes: Vec<u8>,
    first_owner: Option<RowOwners>,
    second_owner: Option<RowOwners>,
    other_owners: Vec<RowOwners>,
    transferred: bool,
}
// Pending collection stack. Completing a parent row subsumes its nested ranges,
// so metadata is bounded by schema nesting/siblings rather than element count.
struct RowOwners {
    range: Range,
    stride: usize,
    initialized: usize,
    release: unsafe fn(&Reader<'_>, usize),
}
impl Writer {
    /// Reserve the root record without copying a Rust structure's padding.
    #[inline]
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
            first_owner: None,
            second_owner: None,
            other_owners: Vec::new(),
            transferred: false,
        }
    }
    /// Register rollback for a collection's rows, including nested owners. Two
    /// pending ranges fit inline, covering flat and singly nested collections.
    ///
    /// # Safety
    /// `range` must be a row allocation made by this writer with this `stride`.
    /// `release` must reclaim every owned field of an initialized row, including
    /// nested rows, skipping nulls and catching destructor panics to visit every
    /// field. Nested collections register after their parent; completed parent
    /// rows must contain their complete descriptors. Registered fields must not
    /// be overwritten after ownership transfer, and no rows may be registered
    /// after [`Self::commit`]. Owners must support destruction on another thread.
    #[inline]
    pub unsafe fn own_rows(
        &mut self,
        range: Range,
        stride: usize,
        release: unsafe fn(&Reader<'_>, usize),
    ) -> usize {
        debug_assert!(!self.transferred);
        debug_assert!(Reader::new(&self.bytes).rows(Some(range), stride).is_ok());
        if range.length == 0 {
            return usize::MAX;
        }
        let owner = RowOwners {
            range,
            stride,
            initialized: 0,
            release,
        };
        if self.first_owner.is_none() {
            self.first_owner = Some(owner);
            0
        } else if self.second_owner.is_none() {
            self.second_owner = Some(owner);
            1
        } else {
            self.other_owners.push(owner);
            self.other_owners.len() + 1
        }
    }
    /// Transfer a completed row prefix to this frame's rollback protection.
    ///
    /// # Safety
    /// `owner` must identify a nonempty registration from this writer, which
    /// must not have committed. Every direct owned field in the newly completed
    /// rows, including complete nested descriptors, must have been initialized
    /// and transferred exactly once. Every later
    /// registration must belong to those newly completed rows: this call removes
    /// the nested registrations, whose owners become reachable through the parent.
    /// Previous rows remain owned by this writer; the prefix must not shrink.
    #[inline]
    pub unsafe fn commit_rows(&mut self, owner: usize, initialized: usize) {
        debug_assert!(!self.transferred);
        let rows = if owner == 0 {
            self.first_owner.as_mut().unwrap_unchecked()
        } else if owner == 1 {
            self.second_owner.as_mut().unwrap_unchecked()
        } else {
            self.other_owners.get_unchecked_mut(owner - 2)
        };
        debug_assert!(initialized >= rows.initialized && initialized <= rows.range.length as usize);
        rows.initialized = initialized;
        // RowOwners contains no user values or destructors. Truncating retained
        // capacity is infallible and does not visit transferred handle fields.
        if owner == 0 {
            self.second_owner = None;
            self.other_owners.clear();
        } else {
            self.other_owners.truncate(owner - 1);
        }
    }
    /// Transfer every registered handle to the output recipient. Until this
    /// point, dropping the writer reclaims completed rows on conversion failure.
    #[inline]
    pub fn commit(mut self) {
        self.transferred = true;
    }
    /// Reserve a contiguous array at an aligned offset in the frame.
    pub fn rows(&mut self, len: usize, size: usize, align: usize) -> Result<Range> {
        if size == 0 || !align.is_power_of_two() {
            return Err("invalid bulk row layout".into());
        }
        let start = self
            .bytes
            .len()
            .checked_add(align - 1)
            .map(|n| n & !(align - 1))
            .ok_or("bulk alignment overflow")?;
        let end = len
            .checked_mul(size)
            .and_then(|n| start.checked_add(n))
            .ok_or("bulk row extent overflow")?;
        if end > i32::MAX as usize {
            return Err("bulk frame exceeds JVM array limit".into());
        }
        self.bytes.resize(end, 0);
        Ok(Range {
            offset: start as u64,
            length: len as u64,
        })
    }
    /// Store one padding-free scalar with unaligned access.
    pub fn write<T: Field>(&mut self, offset: usize, value: T) -> Result<()> {
        let end = offset
            .checked_add(std::mem::size_of::<T>())
            .ok_or("bulk offset overflow")?;
        debug_assert!(
            self.transferred
                || self
                    .first_owner
                    .iter()
                    .chain(&self.second_owner)
                    .chain(&self.other_owners)
                    .all(|rows| {
                        rows.initialized == 0
                            || end <= rows.range.offset as usize
                            || offset >= rows.range.offset as usize + rows.initialized * rows.stride
                    }),
            "bulk write overlaps completed owned rows"
        );
        let bytes = self
            .bytes
            .get_mut(offset..end)
            .ok_or("bulk write outside frame")?;
        // Field has no padding; the byte storage need not be aligned.
        unsafe {
            std::ptr::write_unaligned(bytes.as_mut_ptr().cast::<T>(), value);
        }
        Ok(())
    }
    /// Write a nullable collection descriptor, preserving absent versus empty rows.
    pub fn range(&mut self, offset: usize, value: Option<Range>) -> Result<()> {
        self.write(offset, value.unwrap_or(Range::ABSENT))
    }
    /// Append a payload while its source is alive and return its frame descriptor.
    pub fn append_buffer(&mut self, value: &[u8]) -> Result<Range> {
        let start = self.bytes.len();
        let end = start
            .checked_add(value.len())
            .ok_or("bulk payload length overflow")?;
        if end > i32::MAX as usize {
            return Err("bulk frame exceeds JVM array limit".into());
        }
        self.bytes.extend_from_slice(value);
        Ok(Range {
            offset: start as u64,
            length: value.len() as u64,
        })
    }
    /// Append a byte payload and write its descriptor, preserving null versus empty.
    pub fn buffer(&mut self, offset: usize, value: Option<&[u8]>) -> Result<()> {
        let range = value.map(|bytes| self.append_buffer(bytes)).transpose()?;
        self.range(offset, range)
    }
    /// Encoded bytes without compacting or copying the native vector.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
impl Drop for Writer {
    #[inline]
    fn drop(&mut self) {
        if self.transferred {
            return;
        }
        let reader = Reader::new(&self.bytes);
        for rows in self
            .first_owner
            .iter()
            .chain(&self.second_owner)
            .chain(&self.other_owners)
        {
            for index in 0..rows.initialized {
                let base = rows.range.offset as usize + index * rows.stride;
                // Rollback must continue even if a user destructor panics.
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                    (rows.release)(&reader, base);
                }));
            }
        }
    }
}
/// Copy only the initialized portion of a JVM frame, preserving spare writer capacity.
pub fn input(
    env: &mut jni::JNIEnv<'_>,
    array: &jni::objects::JByteArray<'_>,
    length: i32,
) -> Result<Vec<u8>> {
    if length < 0 || array.is_null() {
        return Err("invalid bulk frame".into());
    }
    let actual = env.get_array_length(array).map_err(|e| e.to_string())?;
    if length > actual {
        return Err("bulk length exceeds byte array".into());
    }
    let mut bytes = vec![0i8; length as usize];
    env.get_byte_array_region(array, 0, &mut bytes)
        .map_err(|e| e.to_string())?;
    // i8/u8 share layout and every bit pattern; preserve the allocation.
    let mut bytes = std::mem::ManuallyDrop::new(bytes);
    Ok(unsafe {
        Vec::from_raw_parts(
            bytes.as_mut_ptr().cast::<u8>(),
            bytes.len(),
            bytes.capacity(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct Tracked {
        drops: Arc<AtomicUsize>,
        panic: bool,
    }
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic, "injected owner destructor panic");
        }
    }
    fn allocation(drops: &Arc<AtomicUsize>, panic: bool) -> Box<crate::Allocation<Tracked>> {
        Box::new(crate::Allocation {
            value: Tracked {
                drops: drops.clone(),
                panic,
            },
        })
    }
    unsafe fn release_one(reader: &Reader<'_>, base: usize) {
        let _ = prebindgen_runtime::boundary(|| {
            let raw = reader.read::<i64>(base)?;
            if raw != 0 {
                crate::drop_pointer::<Tracked>(raw)?;
            }
            Ok(())
        });
    }
    unsafe fn release_nested(reader: &Reader<'_>, base: usize) {
        if let Some(range) = reader.range(base).unwrap() {
            let (start, len) = reader.rows(Some(range), 8).unwrap();
            for i in 0..len {
                release_one(reader, start + i * 8);
            }
        }
    }
    #[test]
    fn rollback_owns_only_completed_rows_and_survives_writer_moves() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut writer = Writer::new(0);
        let range = writer.rows(3, 8, 8).unwrap();
        let owners = unsafe { writer.own_rows(range, 8, release_one) };
        let mut first = allocation(&drops, false);
        writer
            .write(range.offset as usize, (&mut *first as *mut _) as i64)
            .unwrap();
        let _ = Box::into_raw(first).expose_provenance();
        unsafe {
            writer.commit_rows(owners, 1);
        }
        let pending = allocation(&drops, false);
        writer
            .write(range.offset as usize + 8, (&*pending as *const _) as i64)
            .unwrap();
        let moved = writer;
        drop(moved);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(pending);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn successful_transfer_leaves_the_recipient_as_owner() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut writer = Writer::new(0);
        let range = writer.rows(1, 8, 8).unwrap();
        let owners = unsafe { writer.own_rows(range, 8, release_one) };
        let raw = Box::into_raw(allocation(&drops, false)) as i64;
        writer.write(range.offset as usize, raw).unwrap();
        unsafe {
            writer.commit_rows(owners, 1);
        }
        writer.commit();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        unsafe {
            crate::drop_pointer::<Tracked>(raw).unwrap();
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn nested_metadata_is_bounded_by_depth_not_row_count() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut writer = Writer::new(0);
        let parent = writer.rows(512, 16, 8).unwrap();
        let parent_owners = unsafe { writer.own_rows(parent, 16, release_nested) };
        for i in 0..512 {
            let child = writer.rows(1, 8, 8).unwrap();
            let owners = unsafe { writer.own_rows(child, 8, release_one) };
            writer
                .write(
                    child.offset as usize,
                    Box::into_raw(allocation(&drops, false)) as i64,
                )
                .unwrap();
            unsafe {
                writer.commit_rows(owners, 1);
            }
            writer
                .range(parent.offset as usize + i * 16, Some(child))
                .unwrap();
            unsafe {
                writer.commit_rows(parent_owners, i + 1);
            }
            assert!(writer.other_owners.is_empty());
            assert_eq!(writer.other_owners.capacity(), 0);
        }
        drop(writer);
        assert_eq!(drops.load(Ordering::SeqCst), 512);
    }
    #[test]
    fn partial_parent_reclaims_siblings_not_yet_linked_into_its_row() {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut writer = Writer::new(0);
        let parent = writer.rows(1, 16, 8).unwrap();
        let _parent_owners = unsafe { writer.own_rows(parent, 16, release_nested) };
        let a = writer.rows(1, 8, 8).unwrap();
        let a_owners = unsafe { writer.own_rows(a, 8, release_one) };
        writer
            .write(
                a.offset as usize,
                Box::into_raw(allocation(&drops, false)) as i64,
            )
            .unwrap();
        unsafe {
            writer.commit_rows(a_owners, 1);
        }
        let b = writer.rows(2, 8, 8).unwrap();
        let b_owners = unsafe { writer.own_rows(b, 8, release_one) };
        writer
            .write(
                b.offset as usize,
                Box::into_raw(allocation(&drops, false)) as i64,
            )
            .unwrap();
        unsafe {
            writer.commit_rows(b_owners, 1);
        }
        let pending = allocation(&drops, false);
        writer
            .write(b.offset as usize + 8, (&*pending as *const _) as i64)
            .unwrap();
        // Parent descriptor has not been written; pending ranges remain reachable.
        drop(writer);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
        drop(pending);
        assert_eq!(drops.load(Ordering::SeqCst), 3);
    }
    #[test]
    fn empty_ranges_use_no_owner_slot_and_panicking_fields_do_not_stop_cleanup() {
        unsafe fn pair(reader: &Reader<'_>, base: usize) {
            release_one(reader, base);
            release_one(reader, base + 8);
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let mut writer = Writer::new(0);
        let empty = writer.rows(0, 16, 8).unwrap();
        unsafe {
            writer.own_rows(empty, 16, pair);
        }
        assert!(writer.first_owner.is_none());
        let range = writer.rows(1, 16, 8).unwrap();
        let owners = unsafe { writer.own_rows(range, 16, pair) };
        writer
            .write(
                range.offset as usize,
                Box::into_raw(allocation(&drops, true)) as i64,
            )
            .unwrap();
        writer
            .write(
                range.offset as usize + 8,
                Box::into_raw(allocation(&drops, false)) as i64,
            )
            .unwrap();
        unsafe {
            writer.commit_rows(owners, 1);
        }
        drop(writer);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn rollback_catches_destructor_panics_while_conversion_is_already_unwinding() {
        let drops = Arc::new(AtomicUsize::new(0));
        let failed = std::panic::catch_unwind({
            let drops = drops.clone();
            move || {
                let mut writer = Writer::new(0);
                let range = writer.rows(2, 8, 8).unwrap();
                let owners = unsafe { writer.own_rows(range, 8, release_one) };
                for i in 0..2 {
                    writer
                        .write(
                            range.offset as usize + i * 8,
                            Box::into_raw(allocation(&drops, i == 0)) as i64,
                        )
                        .unwrap();
                    unsafe {
                        writer.commit_rows(owners, i + 1);
                    }
                }
                panic!("injected native conversion failure");
            }
        });
        assert!(failed.is_err());
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn completed_parent_and_later_partial_child_drop_each_allocation_once() {
        struct Specific {
            id: usize,
            drops: Arc<[AtomicUsize; 4]>,
        }
        impl Drop for Specific {
            fn drop(&mut self) {
                self.drops[self.id].fetch_add(1, Ordering::SeqCst);
            }
        }
        unsafe fn leaf(reader: &Reader<'_>, base: usize) {
            let raw = reader.read::<i64>(base).unwrap();
            if raw != 0 {
                crate::drop_pointer::<Specific>(raw).unwrap();
            }
        }
        unsafe fn parent(reader: &Reader<'_>, base: usize) {
            let (start, len) = reader.rows(reader.range(base).unwrap(), 8).unwrap();
            for index in 0..len {
                leaf(reader, start + index * 8);
            }
        }
        let drops = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let make = |id| {
            Box::new(crate::Allocation {
                value: Specific {
                    id,
                    drops: drops.clone(),
                },
            })
        };
        let mut writer = Writer::new(0);
        let outer = writer.rows(2, 16, 8).unwrap();
        let root = unsafe { writer.own_rows(outer, 16, parent) };
        let a = writer.rows(2, 8, 8).unwrap();
        let a_owner = unsafe { writer.own_rows(a, 8, leaf) };
        for i in 0..2 {
            writer
                .write(a.offset as usize + i * 8, Box::into_raw(make(i)) as i64)
                .unwrap();
            unsafe {
                writer.commit_rows(a_owner, i + 1);
            }
        }
        writer.range(outer.offset as usize, Some(a)).unwrap();
        unsafe {
            writer.commit_rows(root, 1);
        }
        let b = writer.rows(2, 8, 8).unwrap();
        let b_owner = unsafe { writer.own_rows(b, 8, leaf) };
        writer
            .write(b.offset as usize, Box::into_raw(make(2)) as i64)
            .unwrap();
        unsafe {
            writer.commit_rows(b_owner, 1);
        }
        let mut pending = make(3);
        writer
            .write(b.offset as usize + 8, (&mut *pending as *mut _) as i64)
            .unwrap();
        drop(writer);
        drop(pending);
        assert!(drops.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    }
    #[test]
    fn unaligned_fields_and_padding_are_handled_fieldwise() {
        let mut writer = Writer::new(24);
        writer.write(1, i64::MIN).unwrap();
        writer.write(10, 1.5f64).unwrap();
        assert_eq!(
            Reader::new(writer.bytes()).read::<i64>(1).unwrap(),
            i64::MIN
        );
        assert_eq!(Reader::new(writer.bytes()).read::<f64>(10).unwrap(), 1.5);
        for at in [0, 9, 18, 19, 20, 21, 22, 23] {
            assert_eq!(writer.bytes()[at], 0);
        }
        assert!(writer.write(20, 1i64).is_err());
        assert!(Reader::new(writer.bytes()).read::<i64>(usize::MAX).is_err());
    }
    #[test]
    fn buffers_distinguish_empty_from_absent_and_rows_check_the_whole_extent() {
        let mut writer = Writer::new(48);
        writer.buffer(0, None).unwrap();
        writer.buffer(16, Some(&[])).unwrap();
        writer.buffer(32, Some(&[0, 255])).unwrap();
        let range = writer.rows(2, 16, 8).unwrap();
        assert_eq!(range.offset % 8, 0);
        let reader = Reader::new(writer.bytes());
        assert_eq!(reader.buffer(0).unwrap(), None);
        assert_eq!(reader.buffer(16).unwrap(), Some(&[][..]));
        assert_eq!(reader.buffer(32).unwrap(), Some(&[0, 255][..]));
        assert_eq!(
            reader.rows(Some(range), 16).unwrap(),
            (range.offset as usize, 2)
        );
        for bad in [
            Range {
                offset: range.offset,
                length: u64::MAX,
            },
            Range {
                offset: u64::MAX - 1,
                length: 2,
            },
        ] {
            assert!(reader.rows(Some(bad), 16).is_err());
        }
        assert!(reader.rows(Some(range), 0).is_err());
        assert!(writer.rows(usize::MAX, 16, 8).is_err());
    }
}
