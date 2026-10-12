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

/// One boxed JNI object. Alignment reserves the low pointer bit for Kotlin's
/// closed/consumed tag and gives zero-sized source types distinct allocations.
/// This is private to the JNI ABI, not a cross-language handle representation.
#[repr(C, align(2))]
pub struct Allocation<T> {
    pub value: T,
    // Alignment alone does not make a zero-sized type occupy storage.
    _anchor: u8,
}
impl<T: Send> Allocation<T> {
    pub fn new(value: T) -> Box<Self> {
        Box::new(Self { value, _anchor: 0 })
    }
}
/// Checks the pointer encoding only. Provenance and liveness are guaranteed by
/// generated Kotlin wrappers holding the object's monitor across each JNI call.
pub fn validate_pointer<T: Send>(raw: i64) -> Result<()> {
    if raw == 0 || (raw as usize) & (std::mem::align_of::<Allocation<T>>() - 1) != 0 {
        return Err("null, closed or misaligned handle".into());
    }
    Ok(())
}
/// # Safety
/// `raw` must be a live allocation of `T` from this JNI library, exclusively held.
pub unsafe fn drop_pointer<T: Send>(raw: i64) -> Result<()> {
    validate_pointer::<T>(raw)?;
    drop(Box::from_raw(raw as *mut Allocation<T>));
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
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
    #[test]
    fn zero_sized_objects_have_distinct_taggable_addresses() {
        let a = Allocation::new(());
        let b = Allocation::new(());
        let pa = (&*a) as *const _ as i64;
        let pb = (&*b) as *const _ as i64;
        assert_ne!(pa, pb);
        assert!(validate_pointer::<()>(pa).is_ok());
        assert!(validate_pointer::<()>(pa | 1).is_err());
        assert!(validate_pointer::<()>(0).is_err());
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
        let raw = Box::into_raw(Allocation::new(Tracked(count.clone()))) as i64;
        unsafe {
            drop_pointer::<Tracked>(raw).unwrap();
        }
        assert_eq!(count.load(Ordering::SeqCst), 2);
        let raw = Box::into_raw(Allocation::new(Tracked(count.clone())));
        let moved = unsafe { Box::from_raw(raw) }.value;
        assert_eq!(count.load(Ordering::SeqCst), 2);
        drop(moved);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }
}
