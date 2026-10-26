//! Retained JVM callbacks and the ownership handoff at callback entry.
use crate::{drop_pointer, LocalFrame};
use jni::{
    objects::{GlobalRef, JBooleanArray, JMethodID, JObject},
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
            if env.exception_check().map_err(|e| e.to_string())? {
                return Err("callback skipped while an outer JVM exception is pending".into());
            }
            let frame = match unsafe { LocalFrame::new(&env, capacity) } {
                Ok(frame) => frame,
                Err(error) => {
                    // PushLocalFrame may leave its own OutOfMemoryError pending.
                    // An outer exception was already excluded above.
                    let _ = env.exception_describe();
                    let _ = env.exception_clear();
                    return Err(error);
                }
            };
            let result = boundary(|| call(&mut env));
            if env.exception_check().unwrap_or(true) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            drop(frame);
            result
        });
        if let Err(error) = result {
            eprintln!("prebindgen callback error: {error}");
        }
    }

    /// Invoke the cached method and settle output ownership even if Kotlin throws.
    ///
    /// The generated bridge sets `accepted[0]` only after it can reclaim every
    /// handle. Until then `outputs` reclaims resources if invocation fails.
    ///
    /// # Safety
    /// `arguments` must match the exact descriptor passed to [`Self::new`], with
    /// live references in `env`. With handle outputs, the descriptor starts with
    /// a boolean-array argument and `accepted` must be that same, fresh one-element
    /// array. Without handle outputs, pass `None` and omit that argument.
    pub unsafe fn invoke(
        &self,
        env: &mut JNIEnv<'_>,
        accepted: Option<&JBooleanArray<'_>>,
        arguments: &[jni::sys::jvalue],
        outputs: &mut CallbackOutputs,
    ) -> Result<()> {
        let result = env
            .call_method_unchecked(
                &self.object,
                self.method,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                arguments,
            )
            .map(|_| ())
            .map_err(|e| e.to_string());
        if env.exception_check().unwrap_or(true) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
        }
        let Some(accepted) = accepted else {
            outputs.accept();
            return result;
        };
        let mut ownership = [0];
        match env.get_boolean_array_region(accepted, 0, &mut ownership) {
            Ok(()) if ownership[0] != 0 => outputs.accept(),
            Ok(()) => {}
            // An unreadable acknowledgement cannot prove that Kotlin did not
            // accept the pointers. Preserve memory safety instead of freeing twice.
            Err(error) => {
                outputs.accept();
                return Err(format!("callback ownership acknowledgement: {error}"));
            }
        }
        result
    }
}

type DropAllocation = unsafe fn(i64) -> Result<()>;

/// Reclaims callback output allocations if JVM invocation never accepts them.
#[derive(Default)]
pub struct CallbackOutputs {
    allocations: Vec<(i64, DropAllocation)>,
    armed: bool,
}
impl CallbackOutputs {
    /// Prepare a guard before output ownership commits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Record a pointer which will be committed before [`Self::arm`].
    ///
    /// # Safety
    /// A nonzero address must identify a unique `Allocation<T>` that remains
    /// Rust-owned until commit. The guard must be armed immediately after commit.
    pub unsafe fn push<T: Send>(&mut self, pointer: i64) {
        if pointer != 0 {
            self.allocations.push((pointer, drop_pointer::<T>));
        }
    }
    /// Take responsibility for all recorded, now committed allocations.
    pub fn arm(&mut self) {
        self.armed = true;
    }
    /// Kotlin accepted responsibility for every allocation.
    pub fn accept(&mut self) {
        self.armed = false;
    }
}
impl Drop for CallbackOutputs {
    fn drop(&mut self) {
        if self.armed {
            for &(pointer, drop) in &self.allocations {
                if let Err(error) = boundary(|| unsafe { drop(pointer) }) {
                    eprintln!("prebindgen callback cleanup error: {error}");
                }
            }
        }
    }
}
