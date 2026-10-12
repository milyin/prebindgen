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
