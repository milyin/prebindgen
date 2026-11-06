//! Checked access to boxed fields of array element rows.
use super::*;
use jni::{
    objects::{GlobalRef, JMethodID},
    signature::{ReturnType, TypeSignature},
};
use std::sync::OnceLock;

// These are JVM built-in classes, independent of application class loaders.
// Keep global class references alongside method IDs, as in the output-row cache.
// Only initialization performs name lookup, signature parsing or allocation.
struct RowClass {
    name: &'static str,
    unbox: Option<(&'static str, &'static str)>,
    cached: OnceLock<CachedRowClass>,
}
struct CachedRowClass {
    class: GlobalRef,
    unbox: Option<(JMethodID, ReturnType)>,
}
impl RowClass {
    const fn new(name: &'static str, unbox: Option<(&'static str, &'static str)>) -> Self {
        Self {
            name,
            unbox,
            cached: OnceLock::new(),
        }
    }
    fn get(&self, env: &mut JNIEnv<'_>) -> Result<&CachedRowClass> {
        if let Some(cached) = self.cached.get() {
            return Ok(cached);
        }
        let class = env.find_class(self.name).map_err(|e| e.to_string())?;
        let class = env.auto_local(class);
        let unbox = if let Some((method, signature)) = self.unbox {
            let parsed = TypeSignature::from_str(signature).map_err(|e| e.to_string())?;
            if !parsed.args.is_empty() {
                return Err("row unboxing method must take no arguments".into());
            }
            Some((
                env.get_method_id(&class, method, signature)
                    .map_err(|e| e.to_string())?,
                parsed.ret,
            ))
        } else {
            None
        };
        let class = env.new_global_ref(&class).map_err(|e| e.to_string())?;
        // jni-rs 0.21 uses unchecked NewGlobalRef: a null result or pending
        // exception can arrive inside Ok. Never publish incomplete metadata.
        if class.as_obj().is_null() || env.exception_check().map_err(|e| e.to_string())? {
            return Err("could not retain JNI row class".into());
        }
        // Racing initializers may each resolve the same bootstrap class. Publish
        // only complete metadata; failed initialization is retryable. A losing
        // candidate releases its global reference on this attached thread.
        let _ = self.cached.set(CachedRowClass { class, unbox });
        Ok(self.cached.get().expect("row class initialized"))
    }
}
impl CachedRowClass {
    fn accepts(&self, env: &mut JNIEnv<'_>, value: &JObject<'_>) -> Result<bool> {
        env.is_instance_of(value, &self.class)
            .map_err(|e| e.to_string())
    }
}
static ROW_ARRAY_CLASS: RowClass = RowClass::new("[Ljava/lang/Object;", None);
static STRING_CLASS: RowClass = RowClass::new("java/lang/String", None);
static BYTE_ARRAY_CLASS: RowClass = RowClass::new("[B", None);

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
    if row.is_null() || !ROW_ARRAY_CLASS.get(env)?.accepts(env, &row)? {
        return Err("collection element must be a nonnull Object[] row".into());
    }
    let row = JObjectArray::from(row);
    if env.get_array_length(&row).map_err(|e| e.to_string())? as usize != count {
        return Err("collection row field count differs".into());
    }
    Ok(row)
}
fn field<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
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
    Ok(value)
}
fn object<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
    class: &RowClass,
) -> Result<JObject<'local>> {
    let value = field(env, row, index, true)?;
    if !value.is_null() && !class.get(env)?.accepts(env, &value)? {
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
            static CLASS: RowClass = RowClass::new($class, Some(($method, $signature)));
            let value = field(env, row, index, false)?;
            let cached = CLASS.get(env)?;
            if !cached.accepts(env, &value)? {
                return Err("incorrect collection field type".into());
            }
            let (method, result) = cached
                .unbox
                .as_ref()
                .expect("primitive row unboxing method");
            // The cache resolved this public no-argument method with its exact
            // descriptor. The receiver's class was checked above, and the
            // retained global class reference keeps the method ID valid.
            unsafe { env.call_method_unchecked(&value, *method, result.clone(), &[]) }
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
    object(env, row, index, &STRING_CLASS).map(Into::into)
}
/// Read a byte-array reference, preserving null for optional/inactive payloads.
pub fn row_bytes<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
) -> Result<JByteArray<'local>> {
    object(env, row, index, &BYTE_ARRAY_CLASS).map(Into::into)
}
/// Read a nested collection array reference, preserving null.
pub fn row_rows<'local>(
    env: &mut JNIEnv<'local>,
    row: &JObjectArray<'local>,
    index: usize,
) -> Result<JObjectArray<'local>> {
    object(env, row, index, &ROW_ARRAY_CLASS).map(Into::into)
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
    #[inline]
    pub unsafe fn new(env: &JNIEnv<'local>, capacity: i32) -> Result<Self> {
        env.push_local_frame(capacity).map_err(|e| e.to_string())?;
        Ok(Self {
            env: env.unsafe_clone(),
        })
    }
}
impl Drop for LocalFrame<'_> {
    #[inline]
    fn drop(&mut self) {
        // PopLocalFrame is permitted while a JVM exception is pending.
        let _ = unsafe { self.env.pop_local_frame(&JObject::null()) };
    }
}

/// Reclaim generated output storage while preserving an already pending exception.
///
/// This runs only on rollback. Destructors must catch their own panics so that
/// sibling fields can still be reclaimed. Failure to read trusted output storage
/// cannot be recovered without leaking native owners, so it aborts the process.
pub fn reclaim_output<'local>(
    env: &mut JNIEnv<'local>,
    reclaim: impl FnOnce(&mut JNIEnv<'local>) -> Result<()>,
) {
    let pending = env
        .exception_occurred()
        .unwrap_or_else(|_| std::process::abort());
    if !pending.is_null() {
        env.exception_clear()
            .unwrap_or_else(|_| std::process::abort());
    }
    let result = prebindgen_runtime::boundary(|| reclaim(env));
    // A user destructor can itself interact with the JVM. Its exception must
    // not interfere with siblings or replace the original delivery failure.
    if env
        .exception_check()
        .unwrap_or_else(|_| std::process::abort())
    {
        env.exception_clear()
            .unwrap_or_else(|_| std::process::abort());
    }
    if result.is_err() {
        std::process::abort();
    }
    if !pending.is_null() {
        env.throw(&pending)
            .unwrap_or_else(|_| std::process::abort());
        let _ = env.delete_local_ref(pending);
    }
}

static OUTPUT_LONG: std::sync::OnceLock<jni::objects::JFieldID> = std::sync::OnceLock::new();

/// Read an owned handle from a trusted generated output row during rollback.
///
/// # Safety
/// `OutputRows::new` must have succeeded, and the field must be a boxed Long
/// produced by generated code. No arbitrary foreign row may be read this way.
pub unsafe fn output_handle(
    env: &mut JNIEnv<'_>,
    row: &JObjectArray<'_>,
    index: usize,
) -> Result<i64> {
    let value = env
        .get_object_array_element(row, index as i32)
        .map_err(|e| e.to_string())?;
    let value = env.auto_local(value);
    if value.is_null() {
        return Ok(0);
    }
    env.get_field_unchecked(
        &value,
        *OUTPUT_LONG.get().expect("output cleanup initialized"),
        jni::signature::ReturnType::Primitive(jni::signature::Primitive::Long),
    )
    .and_then(|value| value.j())
    .map_err(|e| e.to_string())
}

/// Reclaim a completed prefix of generated object-array output rows.
///
/// # Safety
/// Every row in the prefix must have the schema expected by `release`, which
/// must reclaim each of its native owners exactly once. The caller must own
/// those resources, and all array references must remain valid during cleanup.
pub unsafe fn reclaim_rows<'local>(
    env: &mut JNIEnv<'local>,
    rows: jni::sys::jobjectArray,
    completed: usize,
    release: for<'a> fn(&mut JNIEnv<'a>, &JObjectArray<'a>),
) {
    if completed == 0 {
        return;
    }
    reclaim_output(env, |env| {
        for index in 0..completed {
            reclaim_output(env, |env| {
                // Read the existing array reference without manufacturing a
                // second Rust owner for that same JNI local-reference slot.
                let table = env.get_native_interface();
                let raw = ((**table).GetObjectArrayElement.expect("JNI array access"))(
                    table,
                    rows,
                    index as i32,
                );
                if env.exception_check().map_err(|e| e.to_string())? || raw.is_null() {
                    return Err("cannot read completed JNI output row".into());
                }
                let row = env.auto_local(JObjectArray::from_raw(raw));
                release(env, &row);
                Ok(())
            });
        }
        Ok(())
    });
}

/// Reclaim a completed prefix of generated typed output objects.
///
/// # Safety
/// Every completed object must match `release` and own its native resources.
/// References must remain live until cleanup finishes.
pub unsafe fn reclaim_objects<'local>(
    env: &mut JNIEnv<'local>,
    rows: jni::sys::jobjectArray,
    completed: usize,
    release: for<'a> fn(&mut JNIEnv<'a>, &JObject<'a>),
) {
    reclaim_output(env, |env| {
        for index in 0..completed {
            reclaim_output(env, |env| {
                let table = env.get_native_interface();
                let raw = ((**table).GetObjectArrayElement.expect("JNI array access"))(
                    table,
                    rows,
                    index as i32,
                );
                if env.exception_check().map_err(|e| e.to_string())? || raw.is_null() {
                    return Err("cannot read completed typed output row".into());
                }
                let row = env.auto_local(JObject::from_raw(raw));
                release(env, &row);
                Ok(())
            });
        }
        Ok(())
    });
}

enum RowRelease {
    Arrays(for<'a> fn(&mut JNIEnv<'a>, &JObjectArray<'a>)),
    Objects(for<'a> fn(&mut JNIEnv<'a>, &JObject<'a>)),
}

/// Owns the completed prefix of a generated JNI collection until final transfer.
///
/// The final JVM array is also the ownership journal: successful row insertion
/// requires only incrementing a stack counter, with no per-row native storage.
pub struct OutputRows<'local> {
    env: JNIEnv<'local>,
    rows: jni::sys::jobjectArray,
    completed: usize,
    release: RowRelease,
}
impl<'local> OutputRows<'local> {
    /// Start owning a newly allocated collection's completed rows.
    ///
    /// # Safety
    /// The array's local reference must outlive the guard. Each completed row
    /// must match `release` and transfer all its native owners to this guard.
    /// Disarm nested guards before their enclosing local-reference frame ends.
    #[inline]
    pub unsafe fn new(
        env: &mut JNIEnv<'local>,
        rows: &JObjectArray<'local>,
        release: for<'a> fn(&mut JNIEnv<'a>, &JObjectArray<'a>),
    ) -> Result<Self> {
        // java.lang.Long is a bootstrap class and cannot unload. The field
        // ID is shared by all generated row schemas and initialized before
        // any row transfers ownership; rollback does no class/method lookup.
        if OUTPUT_LONG.get().is_none() {
            let field = env
                .get_field_id("java/lang/Long", "value", "J")
                .map_err(|e| e.to_string())?;
            let _ = OUTPUT_LONG.set(field);
        }
        Ok(Self {
            env: env.unsafe_clone(),
            rows: rows.as_raw(),
            completed: 0,
            release: RowRelease::Arrays(release),
        })
    }
    /// Own completed typed objects instead of legacy boxed array rows.
    ///
    /// # Safety
    /// The array reference must outlive the guard; each completed object must
    /// match `release` and transfer all of its native owners exactly once.
    #[inline]
    pub unsafe fn new_objects(
        env: &mut JNIEnv<'local>,
        rows: &JObjectArray<'local>,
        release: for<'a> fn(&mut JNIEnv<'a>, &JObject<'a>),
    ) -> Self {
        Self {
            env: env.unsafe_clone(),
            rows: rows.as_raw(),
            completed: 0,
            release: RowRelease::Objects(release),
        }
    }
    /// Record a row after insertion and infallible transfer of its native owners.
    ///
    /// # Safety
    /// The next row must be fully inserted and all its native owners transferred
    /// exactly once. No row may be modified after it is recorded.
    #[inline]
    pub unsafe fn complete_row(&mut self) {
        self.completed += 1;
    }
    /// Transfer all completed rows to their enclosing output or Kotlin recipient.
    #[inline]
    pub fn commit(mut self) {
        self.completed = 0;
    }
}
impl Drop for OutputRows<'_> {
    #[inline]
    fn drop(&mut self) {
        if self.completed == 0 {
            return;
        }
        unsafe {
            match self.release {
                RowRelease::Arrays(release) => {
                    reclaim_rows(&mut self.env, self.rows, self.completed, release)
                }
                RowRelease::Objects(release) => {
                    reclaim_objects(&mut self.env, self.rows, self.completed, release)
                }
            }
        }
    }
}
