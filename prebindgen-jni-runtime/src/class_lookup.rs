//! Class lookup that works from any thread, including ones the JVM did not start.
//!
//! `JNIEnv::FindClass` resolves against the class loader of the Java method at
//! the top of the calling thread's stack. On a thread that native code attached
//! itself (`AttachCurrentThread`) there is no such method, and the JVM falls back
//! to the *system* class loader. On a desktop JVM that loader sees the classpath,
//! so the fallback is harmless. On Android it sees only the boot classpath, so
//! every application class — every generated binding class included — fails
//! with `ClassNotFoundException`. Generated struct encoders and callback
//! upcalls run on exactly those threads (the binding library's own worker
//! threads), so a plain `FindClass` there aborts the process on Android.
//!
//! The standard remedy: record the application's `ClassLoader` once, from a
//! thread where `FindClass` still works (`JNI_OnLoad`, or any JNI entry point
//! called from Java), and resolve classes through `ClassLoader.loadClass` from
//! then on. Resolved classes are pinned as global references, so each name
//! costs one lookup per process.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use jni::{
    objects::{GlobalRef, JClass, JObject, JValue},
    JNIEnv,
};

static APP_CLASS_LOADER: OnceLock<GlobalRef> = OnceLock::new();
static CLASSES: OnceLock<Mutex<HashMap<String, GlobalRef>>> = OnceLock::new();

fn classes() -> &'static Mutex<HashMap<String, GlobalRef>> {
    CLASSES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Record the class loader that loaded `anchor_class_fqn` (slash form, e.g.
/// `"io/zenoh/jni/NativeLibrary"`), for [`find_class`] to use on threads
/// where `FindClass` cannot see application classes.
///
/// Call it from `JNI_OnLoad`: there, `FindClass` resolves against the loader
/// of the class that called `System.loadLibrary`, which is the application's.
/// Calling it more than once is harmless; the first loader recorded wins.
pub fn init_class_loader(env: &mut JNIEnv, anchor_class_fqn: &str) -> Result<(), String> {
    if APP_CLASS_LOADER.get().is_some() {
        return Ok(());
    }
    let anchor = env.find_class(anchor_class_fqn).map_err(|e| {
        let _ = env.exception_clear();
        format!("find anchor class {anchor_class_fqn}: {e}")
    })?;
    let loader = env
        .call_method(&anchor, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(|v| v.l())
        .map_err(|e| {
            let _ = env.exception_clear();
            format!("getClassLoader of {anchor_class_fqn}: {e}")
        })?;
    let loader = env
        .new_global_ref(&loader)
        .map_err(|e| format!("global-ref class loader: {e}"))?;
    let _ = APP_CLASS_LOADER.set(loader);
    // Pin the anchor too: it is certainly wanted later, and it proves the path works.
    if let Ok(g) = env.new_global_ref(&anchor) {
        classes()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(anchor_class_fqn.to_owned())
            .or_insert(g);
    }
    Ok(())
}

/// Resolve `class_fqn` (slash form, `"io/zenoh/jni/time/Timestamp"`) to a
/// local class reference, from any thread.
///
/// Order: the process-wide cache; then the recorded application class loader,
/// if [`init_class_loader`] ran; then plain `FindClass`, which is the old
/// behaviour and still correct on a desktop JVM or on a thread with Java
/// frames. Whatever resolves is pinned, so later calls never search again.
///
/// Never leaves a Java exception pending on failure: an exception left behind
/// here makes the *next* JNI call abort the process under CheckJNI, which is
/// how a missing class used to become SIGABRT rather than an error.
pub fn find_class<'local>(
    env: &mut JNIEnv<'local>,
    class_fqn: &str,
) -> Result<JClass<'local>, String> {
    let cached = classes()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(class_fqn)
        .cloned();
    if let Some(g) = cached {
        return local_class(env, &g, class_fqn);
    }

    let resolved: JObject<'local> = match APP_CLASS_LOADER.get() {
        Some(loader) => {
            let binary_name = env
                .new_string(class_fqn.replace('/', "."))
                .map_err(|e| format!("class name string for {class_fqn}: {e}"))?;
            env.call_method(
                loader.as_obj(),
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(&binary_name)],
            )
            .and_then(|v| v.l())
            .map_err(|e| {
                let _ = env.exception_clear();
                format!("load class {class_fqn} via application class loader: {e}")
            })?
        }
        None => env
            .find_class(class_fqn)
            .map_err(|e| {
                let _ = env.exception_clear();
                format!("find class {class_fqn}: {e}")
            })?
            .into(),
    };

    let global = env
        .new_global_ref(&resolved)
        .map_err(|e| format!("global-ref class {class_fqn}: {e}"))?;
    classes()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .entry(class_fqn.to_owned())
        .or_insert(global);
    Ok(JClass::from(resolved))
}

fn local_class<'local>(
    env: &mut JNIEnv<'local>,
    global: &GlobalRef,
    class_fqn: &str,
) -> Result<JClass<'local>, String> {
    env.new_local_ref(global.as_obj())
        .map(JClass::from)
        .map_err(|e| format!("local-ref cached class {class_fqn}: {e}"))
}
