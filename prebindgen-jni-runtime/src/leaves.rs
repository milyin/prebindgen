//! Leaf-level helpers: the only JNI operations generated code performs on
//! values — primitive arrays, object arrays, strings — and the upcall that
//! drives a Kotlin callback from any thread.
//!
//! Generated code never reads a field of a Kotlin object: everything crosses
//! as JNI primitives, strings and arrays, and Kotlin assembles the objects.

use std::ops::Deref;

use jni::{
    objects::{
        GlobalRef, JBooleanArray, JByteArray, JCharArray, JDoubleArray, JFloatArray, JIntArray,
        JLongArray, JMethodID, JObject, JObjectArray, JShortArray, JString,
    },
    signature::{Primitive, ReturnType},
    sys::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort, jsize, jvalue},
    JNIEnv, JavaVM,
};

fn jerr(what: &'static str) -> impl Fn(jni::errors::Error) -> String {
    move |e| format!("{what}: {e}")
}

macro_rules! prim_array {
    ($read:ident, $write:ident, $t:ty, $arr:ident, $get:ident, $new:ident, $set:ident) => {
        #[doc = concat!("Read a whole `", stringify!($arr), "` into a `Vec`.")]
        pub fn $read(env: &mut JNIEnv, obj: &JObject) -> Result<Vec<$t>, String> {
            if obj.is_null() {
                return Err(concat!("null ", stringify!($arr)).to_string());
            }
            let arr: &$arr = obj.into();
            let n = env
                .get_array_length(arr)
                .map_err(jerr(concat!("length of ", stringify!($arr))))?;
            let mut v: Vec<$t> = vec![<$t>::default(); n as usize];
            env.$get(arr, 0, &mut v)
                .map_err(jerr(concat!("read ", stringify!($arr))))?;
            Ok(v)
        }

        #[doc = concat!("A new `", stringify!($arr), "` holding `v`.")]
        pub fn $write<'l>(env: &mut JNIEnv<'l>, v: &[$t]) -> Result<JObject<'l>, String> {
            let arr = env
                .$new(v.len() as jsize)
                .map_err(jerr(concat!("new ", stringify!($arr))))?;
            env.$set(&arr, 0, v)
                .map_err(jerr(concat!("write ", stringify!($arr))))?;
            Ok(arr.into())
        }
    };
}

prim_array!(read_booleans, write_booleans, jboolean, JBooleanArray, get_boolean_array_region, new_boolean_array, set_boolean_array_region);
prim_array!(read_bytes, write_bytes, jbyte, JByteArray, get_byte_array_region, new_byte_array, set_byte_array_region);
prim_array!(read_chars, write_chars, jchar, JCharArray, get_char_array_region, new_char_array, set_char_array_region);
prim_array!(read_shorts, write_shorts, jshort, JShortArray, get_short_array_region, new_short_array, set_short_array_region);
prim_array!(read_ints, write_ints, jint, JIntArray, get_int_array_region, new_int_array, set_int_array_region);
prim_array!(read_longs, write_longs, jlong, JLongArray, get_long_array_region, new_long_array, set_long_array_region);
prim_array!(read_floats, write_floats, jfloat, JFloatArray, get_float_array_region, new_float_array, set_float_array_region);
prim_array!(read_doubles, write_doubles, jdouble, JDoubleArray, get_double_array_region, new_double_array, set_double_array_region);

/// A `ByteArray` as bytes.
pub fn read_u8s(env: &mut JNIEnv, obj: &JObject) -> Result<Vec<u8>, String> {
    if obj.is_null() {
        return Err("null ByteArray".to_string());
    }
    let arr: &JByteArray = obj.into();
    env.convert_byte_array(arr).map_err(jerr("read ByteArray"))
}

/// A new `ByteArray` holding `v`.
pub fn write_u8s<'l>(env: &mut JNIEnv<'l>, v: &[u8]) -> Result<JObject<'l>, String> {
    env.byte_array_from_slice(v)
        .map(Into::into)
        .map_err(jerr("new ByteArray"))
}

/// Exactly `N` elements, or the fixed-size-array decode error.
pub fn fixed<T, const N: usize>(v: Vec<T>) -> Result<[T; N], String> {
    let n = v.len();
    v.try_into()
        .map_err(|_| format!("fixed-size array decode: expected {N} elements, got {n}"))
}

/// The length of an `Array<Any?>`.
pub fn object_array_len(env: &mut JNIEnv, obj: &JObject) -> Result<usize, String> {
    if obj.is_null() {
        return Err("null object array".to_string());
    }
    let arr: &JObjectArray = obj.into();
    env.get_array_length(arr)
        .map(|n| n as usize)
        .map_err(jerr("length of object array"))
}

/// Element `i` of an `Array<Any?>`, as a new local reference.
pub fn object_array_get<'l>(
    env: &mut JNIEnv<'l>,
    obj: &JObject,
    i: usize,
) -> Result<JObject<'l>, String> {
    let arr: &JObjectArray = obj.into();
    env.get_object_array_element(arr, i as jsize)
        .map_err(jerr("read object array"))
}

/// A new `Array<Any?>` of `n` nulls.
pub fn new_object_array<'l>(env: &mut JNIEnv<'l>, n: usize) -> Result<JObject<'l>, String> {
    env.new_object_array(n as jsize, "java/lang/Object", JObject::null())
        .map(Into::into)
        .map_err(jerr("new object array"))
}

/// Store `value` at `i` and release its local reference.
pub fn object_array_set(
    env: &mut JNIEnv,
    obj: &JObject,
    i: usize,
    value: JObject,
) -> Result<(), String> {
    let arr: &JObjectArray = obj.into();
    env.set_object_array_element(arr, i as jsize, &value)
        .map_err(jerr("write object array"))?;
    if !value.is_null() {
        env.delete_local_ref(value).map_err(jerr("release local"))?;
    }
    Ok(())
}

/// Release a local reference no longer needed.
pub fn drop_local(env: &mut JNIEnv, obj: JObject) {
    if !obj.is_null() {
        let _ = env.delete_local_ref(obj);
    }
}

/// A `String` as a Rust string; null is an error.
pub fn read_string(env: &mut JNIEnv, obj: &JObject) -> Result<String, String> {
    if obj.is_null() {
        return Err("null String".to_string());
    }
    let s: &JString = obj.into();
    env.get_string(s)
        .map(Into::into)
        .map_err(jerr("read String"))
}

/// A new Java `String`.
pub fn new_string<'l>(env: &mut JNIEnv<'l>, s: &str) -> Result<JObject<'l>, String> {
    env.new_string(s).map(Into::into).map_err(jerr("new String"))
}

/// A value an argument either owns or borrows — what a parameter that may
/// be built on the spot *or* passed as an existing handle holds.
pub enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<T> Deref for MaybeOwned<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            MaybeOwned::Owned(v) => v,
            MaybeOwned::Borrowed(v) => v,
        }
    }
}

/// A Kotlin callback object, callable from any thread: a global reference,
/// the resolved `run` method and the VM to attach to.
pub struct Upcall {
    vm: JavaVM,
    obj: GlobalRef,
    method: JMethodID,
    frame: i32,
}

impl Upcall {
    /// Resolve `method` with JVM descriptor `descr` on `obj`'s class. Runs on
    /// the calling (Java) thread, so class lookup sees the application's
    /// classes.
    pub fn new(
        env: &mut JNIEnv,
        obj: &JObject,
        method: &str,
        descr: &str,
        frame: i32,
    ) -> Result<Self, String> {
        if obj.is_null() {
            return Err("null callback".to_string());
        }
        let vm = env.get_java_vm().map_err(jerr("get JavaVM"))?;
        let class = env.get_object_class(obj).map_err(jerr("callback class"))?;
        let method = env
            .get_method_id(&class, method, descr)
            .map_err(|e| format!("resolve callback {method}{descr}: {e}"))?;
        let obj = env.new_global_ref(obj).map_err(jerr("global-ref callback"))?;
        Ok(Self {
            vm,
            obj,
            method,
            frame,
        })
    }

    /// Attach the current thread, build the arguments with `args` inside a
    /// local frame, and invoke the callback. A Kotlin exception is described
    /// and cleared: a callback may not unwind into native code.
    pub fn call_void(
        &self,
        args: impl FnOnce(&mut JNIEnv) -> Result<Vec<jvalue>, String>,
    ) -> Result<(), String> {
        let mut guard = self
            .vm
            .attach_current_thread_as_daemon()
            .map_err(jerr("attach thread"))?;
        let env: &mut JNIEnv = &mut guard;
        env.push_local_frame(self.frame)
            .map_err(jerr("push local frame"))?;
        let result = (|| {
            let args = args(env)?;
            // SAFETY: the method was resolved against this object's class
            // with the descriptor the arguments were built for.
            let r = unsafe {
                env.call_method_unchecked(
                    &self.obj,
                    self.method,
                    ReturnType::Primitive(Primitive::Void),
                    &args,
                )
            };
            if r.is_err() || env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
                return Err("callback threw".to_string());
            }
            Ok(())
        })();
        // SAFETY: nothing from the frame escapes.
        let _ = unsafe { env.pop_local_frame(&JObject::null()) };
        result
    }
}

/// Report a callback delivery failure: there is no caller to hand it to.
pub fn report_callback_error(what: &str, err: &str) {
    if err != "callback threw" {
        eprintln!("{what}: {err}");
    }
}

/// Move `v` into a new handle: a boxed pointer whose bit 0 is free for
/// Kotlin's closed tag.
pub fn new_handle<T>(v: T) -> jlong {
    Box::into_raw(Box::new(v)) as jlong
}

fn check_handle(p: jlong) -> Result<(), String> {
    if p == 0 || (p & 1) == 1 {
        Err("Operation on a closed native handle.".to_string())
    } else {
        Ok(())
    }
}

/// Take ownership of the value behind a handle.
///
/// # Safety
/// `p` must come from [`new_handle`] for a `T`, and not be used again.
pub unsafe fn take_handle<T>(p: jlong) -> Result<T, String> {
    check_handle(p)?;
    Ok(*Box::from_raw(p as *mut T))
}

/// Borrow the value behind a handle.
///
/// # Safety
/// `p` must come from [`new_handle`] for a `T` that outlives the borrow.
pub unsafe fn borrow_handle<'a, T>(p: jlong) -> Result<&'a T, String> {
    check_handle(p)?;
    Ok(&*(p as *const T))
}

/// Borrow the value behind a handle mutably.
///
/// # Safety
/// As [`borrow_handle`], and nothing else may use the value meanwhile.
pub unsafe fn borrow_handle_mut<'a, T>(p: jlong) -> Result<&'a mut T, String> {
    check_handle(p)?;
    Ok(&mut *(p as *mut T))
}

/// Free a handle; closed or null handles are ignored.
///
/// # Safety
/// `p` must come from [`new_handle`] for a `T`, and not be used again.
pub unsafe fn free_handle<T>(p: jlong) {
    if p != 0 && (p & 1) == 0 {
        drop(Box::from_raw(p as *mut T));
    }
}

/// `v` when it lies in `lo..=hi`, else the out-of-domain error for `ty`.
pub fn check_domain<T>(v: T, lo: u128, hi: u128, ty: &str) -> Result<T, String>
where
    T: Copy + TryInto<u128> + std::fmt::Display,
{
    match v.try_into() {
        Ok(x) if (lo..=hi).contains(&x) => Ok(v),
        _ => Err(format!(
            "`{ty}` value {v} is outside its declared domain {lo}..={hi}"
        )),
    }
}
