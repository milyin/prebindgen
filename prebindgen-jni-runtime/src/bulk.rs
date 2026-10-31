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
}
impl Writer {
    /// Reserve the root record without copying a Rust structure's padding.
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
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
