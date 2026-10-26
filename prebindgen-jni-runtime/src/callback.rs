//! Retained JVM callbacks and argument ownership transfer.
use crate::LocalFrame;
use jni::{
    objects::{GlobalRef, JMethodID, JObject},
    JNIEnv, JavaVM,
};
use prebindgen_runtime::{boundary, Result};

/// A JVM callback retained until its owning Rust closure is dropped.
///
/// The object pins its class, so the method resolved at registration remains valid
/// when a native worker thread invokes it later.
pub struct Callback {
    object: GlobalRef,
    vm: JavaVM,
    method: JMethodID,
}
impl Callback {
    /// Retain `object` and resolve its generated void `run` method once.
    pub fn new(env: &mut JNIEnv<'_>, object: &JObject<'_>, descriptor: &str) -> Result<Self> {
        if object.is_null() {
            return Err("null callback".into());
        }
        let class = env.get_object_class(object).map_err(|e| e.to_string())?;
        let method = env
            .get_method_id(&class, "run", descriptor)
            .map_err(|e| e.to_string())?;
        env.delete_local_ref(class).map_err(|e| e.to_string())?;
        let object = env.new_global_ref(object).map_err(|e| e.to_string())?;
        let vm = env.get_java_vm().map_err(|e| e.to_string())?;
        Ok(Self { object, vm, method })
    }

    /// Run an upcall on the current native thread with a bounded local frame.
    ///
    /// Callback failures have no live declaring-call error handler. Report them
    /// and clear JVM exceptions so subsequent invocations remain usable.
    pub fn with_env(&self, capacity: i32, call: impl FnOnce(&mut JNIEnv<'_>) -> Result<()>) {
        let result = boundary(|| {
            let mut env = self
                .vm
                .attach_current_thread_as_daemon()
                .map_err(|e| e.to_string())?;
            let result = boundary(|| {
                let frame = unsafe { LocalFrame::new(&env, capacity) }?;
                let result = call(&mut env);
                drop(frame);
                result
            });
            if result.is_err() {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            result
        });
        if let Err(error) = result {
            eprintln!("prebindgen callback error: {error}");
        }
    }

    /// Invoke the cached method. Committed argument handles are Kotlin-owned.
    ///
    /// Argument ownership transfers without a per-invocation acceptance handshake.
    /// Kotlin cleans up handles after entry; failure to enter the bridge can leak them.
    ///
    /// # Safety
    /// `arguments` must match the descriptor passed to [`Self::new`], with live
    /// references in `env`. The environment must have no pending JVM exception.
    pub unsafe fn invoke(
        &self,
        env: &mut JNIEnv<'_>,
        arguments: &[jni::sys::jvalue],
    ) -> Result<()> {
        env.call_method_unchecked(
            &self.object,
            self.method,
            jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
            arguments,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}
