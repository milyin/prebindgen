//! Leaf JNI operations used by the new converter pipeline.
pub use jni;
use jni::{
    objects::{JByteArray, JObject, JObjectArray, JString, JValue},
    JNIEnv,
};
use prebindgen_runtime::Result;

pub fn text(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Result<String> {
    env.get_string(value)
        .map(|s| s.into())
        .map_err(|e| e.to_string())
}
pub fn bytes(env: &mut JNIEnv<'_>, value: &JByteArray<'_>) -> Result<Vec<u8>> {
    env.convert_byte_array(value).map_err(|e| e.to_string())
}
pub fn handle_object<'a>(env: &mut JNIEnv<'a>, value: u64) -> Result<JObject<'a>> {
    env.call_static_method(
        "java/lang/Long",
        "valueOf",
        "(J)Ljava/lang/Long;",
        &[JValue::Long(value as i64)],
    )
    .and_then(|v| v.l())
    .map_err(|e| e.to_string())
}
pub fn text_object<'a>(env: &mut JNIEnv<'a>, value: &str) -> Result<JObject<'a>> {
    env.new_string(value)
        .map(Into::into)
        .map_err(|e| e.to_string())
}
pub fn bytes_object<'a>(env: &mut JNIEnv<'a>, value: &[u8]) -> Result<JObject<'a>> {
    env.byte_array_from_slice(value)
        .map(Into::into)
        .map_err(|e| e.to_string())
}
pub fn array<'a>(env: &mut JNIEnv<'a>, len: usize) -> Result<JObjectArray<'a>> {
    let len = i32::try_from(len).map_err(|_| "too many results")?;
    env.new_object_array(len, "java/lang/Object", JObject::null())
        .map_err(|e| e.to_string())
}
pub fn fail(env: &mut JNIEnv<'_>, message: &str) {
    // Preserve an existing JVM exception (including allocation failures).
    if matches!(env.exception_check(), Ok(false)) {
        let _ = env.throw_new("java/lang/IllegalArgumentException", message);
    }
}

/// Decode BigInteger's big-endian two's-complement bytes without narrowing.
pub fn signed128(env: &mut JNIEnv<'_>, value: &JByteArray<'_>) -> Result<i128> {
    parse_signed128(&bytes(env, value)?)
}
fn parse_signed128(mut bytes: &[u8]) -> Result<i128> {
    if bytes.is_empty() {
        return Err("empty integer representation".into());
    }
    while bytes.len() > 16
        && ((bytes[0] == 0 && bytes[1] < 128) || (bytes[0] == 255 && bytes[1] >= 128))
    {
        bytes = &bytes[1..];
    }
    if bytes.len() > 16 {
        return Err("i128 input out of range".into());
    }
    let mut result = [if bytes[0] >= 128 { 255 } else { 0 }; 16];
    result[16 - bytes.len()..].copy_from_slice(bytes);
    Ok(i128::from_be_bytes(result))
}
/// Unsigned input still uses BigInteger's signed encoding; negatives are rejected.
pub fn unsigned128(env: &mut JNIEnv<'_>, value: &JByteArray<'_>) -> Result<u128> {
    parse_unsigned128(&bytes(env, value)?)
}
fn parse_unsigned128(mut bytes: &[u8]) -> Result<u128> {
    if bytes.is_empty() || bytes[0] >= 128 {
        return Err("invalid unsigned integer".into());
    }
    while bytes.len() > 16 && bytes[0] == 0 {
        bytes = &bytes[1..];
    }
    if bytes.len() > 16 {
        return Err("u128 input out of range".into());
    }
    let mut result = [0; 16];
    result[16 - bytes.len()..].copy_from_slice(bytes);
    Ok(u128::from_be_bytes(result))
}

/// JNI storage alignment reserves the low pointer bit for Kotlin's consumed
/// state. The flat model requires `T` to have nonzero size.
#[repr(C, align(2))]
pub struct Allocation<T> {
    pub value: T,
}

/// # Safety
/// `raw` must be a live allocation of `T` from this JNI library, exclusively held.
pub unsafe fn drop_pointer<T: Send>(raw: i64) -> Result<()> {
    let pointer = raw as *mut Allocation<T>;
    prebindgen_runtime::validate_pointer(pointer)?;
    drop(Box::from_raw(pointer));
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_objects_leave_the_tag_bit_free_without_an_anchor() {
        assert_eq!(
            std::mem::size_of::<Allocation<u64>>(),
            std::mem::size_of::<u64>()
        );
        assert!(std::mem::align_of::<Allocation<u8>>() >= 2);
        let allocation = Box::new(Allocation { value: 42u8 });
        let pointer = Box::into_raw(allocation);
        assert_eq!(pointer.addr() & 1, 0);
        unsafe {
            assert!(drop_pointer::<u8>(pointer as i64 | 1).is_err());
            assert!(drop_pointer::<u8>(pointer as i64).is_ok());
        }
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
        drop(Box::new(Allocation {
            value: Tracked(count.clone()),
        }));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        let raw = Box::into_raw(Box::new(Allocation {
            value: Tracked(count.clone()),
        }));
        unsafe {
            drop_pointer::<Tracked>(raw as i64).unwrap();
        }
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let raw = Box::into_raw(Box::new(Allocation {
            value: Tracked(count.clone()),
        }));
        let moved = unsafe { Box::from_raw(raw) }.value;
        assert_eq!(count.load(Ordering::SeqCst), 2);
        drop(moved);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }
    #[test]
    fn wide_integer_sign_extension_and_range_checks() {
        for value in [i128::MIN, i128::MAX, -1, 0, 1, 128] {
            assert_eq!(parse_signed128(&value.to_be_bytes()).unwrap(), value);
        }
        assert_eq!(parse_signed128(&[255]).unwrap(), -1);
        assert_eq!(parse_signed128(&[0, 128]).unwrap(), 128);
        let mut max_unsigned = vec![0];
        max_unsigned.extend([255; 16]);
        assert_eq!(parse_unsigned128(&max_unsigned).unwrap(), u128::MAX);
        assert!(parse_signed128(&max_unsigned).is_err());
        assert!(parse_unsigned128(&[255]).is_err());
        assert!(parse_signed128(&[0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(parse_unsigned128(&[1; 17]).is_err());
        assert!(parse_signed128(&[]).is_err());
        assert!(parse_unsigned128(&[]).is_err());
    }
}
