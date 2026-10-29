//! Synchronous, borrowed return callbacks; no retained callback object per call.
use crate::{jni, Result};
use jni::{
    objects::{GlobalRef, JMethodID, JObject},
    JNIEnv,
};
use std::sync::OnceLock;

/// Cached method for one generated return-capture class and method signature.
/// The class reference keeps its method ID valid; the per-call capture is borrowed.
#[derive(Default)]
pub struct ReturnCallback(OnceLock<(GlobalRef, JMethodID)>);
impl ReturnCallback {
    /// Create an empty cache. First delivery resolves the capture's actual class.
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
    /// Deliver values to a generated capture, preserving pending JVM exceptions.
    ///
    /// # Safety
    /// Every use of this cache must have the same capture class, method name and
    /// descriptor. Arguments must match that descriptor, which must return void.
    /// Reference arguments must remain live through this synchronous invocation.
    pub unsafe fn invoke(
        &self,
        env: &mut JNIEnv<'_>,
        capture: &JObject<'_>,
        method: &str,
        descriptor: &str,
        arguments: &[jni::sys::jvalue],
    ) -> Result<()> {
        if self.0.get().is_none() {
            let class = env.get_object_class(capture).map_err(|e| e.to_string())?;
            let id = env
                .get_method_id(&class, method, descriptor)
                .map_err(|e| e.to_string())?;
            let retained = env.new_global_ref(&class).map_err(|e| e.to_string())?;
            env.delete_local_ref(class).map_err(|e| e.to_string())?;
            let _ = self.0.set((retained, id));
        }
        let (_, method) = self.0.get().expect("return callback initialized");
        env.call_method_unchecked(
            capture,
            *method,
            jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
            arguments,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}
/// Invoke a cold error callback using its receiver's class, without a static
/// application-class lookup or assumptions about another function's capture.
///
/// # Safety
/// `arguments` must match `descriptor`, which must return void. Reference values
/// must remain live through this synchronous invocation.
pub unsafe fn invoke_error_callback(
    env: &mut JNIEnv<'_>,
    capture: &JObject<'_>,
    method: &str,
    descriptor: &str,
    arguments: &[jni::sys::jvalue],
) -> Result<()> {
    let class = env.get_object_class(capture).map_err(|e| e.to_string())?;
    let id = env
        .get_method_id(&class, method, descriptor)
        .map_err(|e| e.to_string())?;
    env.delete_local_ref(class).map_err(|e| e.to_string())?;
    env.call_method_unchecked(
        capture,
        id,
        jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
        arguments,
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
