//! Checked access to boxed fields of array element rows.
use super::*;

/// Read one array row and verify its exact field count.
pub fn row<'local>(
    env: &mut JNIEnv<'local>,
    rows: &JObjectArray<'local>,
    index: usize,
    count: usize,
) -> Result<JObjectArray<'local>> {
    let row = env
        .get_object_array_element(rows, index as i32)
        .map_err(|e| e.to_string())?;
    if row.is_null()
        || !env
            .is_instance_of(&row, "[Ljava/lang/Object;")
            .map_err(|e| e.to_string())?
    {
        return Err("collection element must be a nonnull Object[] row".into());
    }
    let row = JObjectArray::from(row);
    if env.get_array_length(&row).map_err(|e| e.to_string())? as usize != count {
        return Err("collection row field count differs".into());
    }
    Ok(row)
}
fn object<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
    class: &str,
    nullable: bool,
) -> Result<JObject<'local>> {
    let value = env
        .get_object_array_element(row, index as i32)
        .map_err(|e| e.to_string())?;
    if value.is_null() {
        return if nullable {
            Ok(value)
        } else {
            Err("null collection scalar field".into())
        };
    }
    if !env
        .is_instance_of(&value, class)
        .map_err(|e| e.to_string())?
    {
        return Err("incorrect collection field type".into());
    }
    Ok(value)
}
macro_rules! scalar_reader {
    ($name:ident,$ty:ty,$class:literal,$method:literal,$signature:literal,$read:ident) => {
        #[doc=concat!("Read a boxed ",$class," collection field, rejecting other classes.")]
        pub fn $name<'local>(
            env: &mut JNIEnv<'local>,
            row: &JObjectArray<'local>,
            index: usize,
        ) -> Result<$ty> {
            let value = object(env, row, index, $class, false)?;
            env.call_method(&value, $method, $signature, &[])
                .and_then(|v| v.$read())
                .map_err(|e| e.to_string())
        }
    };
}
scalar_reader!(row_i8, i8, "java/lang/Byte", "byteValue", "()B", b);
scalar_reader!(row_i16, i16, "java/lang/Short", "shortValue", "()S", s);
scalar_reader!(row_i32, i32, "java/lang/Integer", "intValue", "()I", i);
scalar_reader!(row_i64, i64, "java/lang/Long", "longValue", "()J", j);
scalar_reader!(row_f32, f32, "java/lang/Float", "floatValue", "()F", f);
scalar_reader!(row_f64, f64, "java/lang/Double", "doubleValue", "()D", d);
scalar_reader!(
    row_bool,
    bool,
    "java/lang/Boolean",
    "booleanValue",
    "()Z",
    z
);
/// Read a string reference; a null field remains null for optional/inactive payloads.
pub fn row_text<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
) -> Result<JString<'local>> {
    object(env, row, index, "java/lang/String", true).map(Into::into)
}
/// Read a byte-array reference, preserving null for optional/inactive payloads.
pub fn row_bytes<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
) -> Result<JByteArray<'local>> {
    object(env, row, index, "[B", true).map(Into::into)
}
/// Read a nested collection array reference, preserving null.
pub fn row_rows<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
) -> Result<JObjectArray<'local>> {
    object(env, row, index, "[Ljava/lang/Object;", true).map(Into::into)
}

/// Owns a JNI local-reference frame, popping it on success, error or unwinding.
/// No reference created inside this frame may be used after this guard drops.
pub struct LocalFrame<'local> {
    env: JNIEnv<'local>,
}
impl<'local> LocalFrame<'local> {
    /// Push a frame; nested guards must be destroyed in reverse creation order.
    ///
    /// # Safety
    /// The caller must stop using every reference created in this frame before
    /// dropping the guard. The cloned environment is used only for frame cleanup.
    pub unsafe fn new(env: &JNIEnv<'local>, capacity: i32) -> Result<Self> {
        env.push_local_frame(capacity).map_err(|e| e.to_string())?;
        Ok(Self {
            env: env.unsafe_clone(),
        })
    }
}
impl Drop for LocalFrame<'_> {
    fn drop(&mut self) {
        // PopLocalFrame is permitted while a JVM exception is pending.
        let _ = unsafe { self.env.pop_local_frame(&JObject::null()) };
    }
}
