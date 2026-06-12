//! Process-wide cached interface-method handles for generated upcalls.
//!
//! Return-site callbacks (output-expansion `build` lambdas, `fold` lambdas,
//! `onError` handlers) arrive as fresh objects on every extern call — there
//! is no long-lived creation site to hoist a method lookup to. But each of
//! them implements a *generated* `fun interface` whose FQN and method
//! descriptor are fixed at codegen time, and JNI permits resolving a method
//! ID on a supertype (the interface class) and invoking it virtually on any
//! implementing instance. So generated code declares one
//! `static CACHED: CachedIfaceMethod` per call site; the first call resolves
//! and pins the interface class, every later call is a single
//! `CallObjectMethodA` — no descriptor parsing, no symbol-table lookup
//! (jni-rs's safe `call_method` re-does both on every call).

use std::sync::OnceLock;

use jni::objects::{GlobalRef, JMethodID, JObject};
use jni::signature::ReturnType;
use jni::sys::jvalue;
use jni::JNIEnv;

/// A `(pinned interface class, method ID)` pair resolved once per process.
/// Declare as `static`; the embedded [`OnceLock`] handles the one-time
/// resolution race (both winners produce equivalent values).
pub struct CachedIfaceMethod {
    cell: OnceLock<Resolved>,
}

struct Resolved {
    /// Pins the interface class so the method ID below stays valid.
    _class: GlobalRef,
    method: JMethodID,
}

impl CachedIfaceMethod {
    pub const fn new() -> Self {
        Self {
            cell: OnceLock::new(),
        }
    }

    fn resolve(
        &self,
        env: &mut JNIEnv,
        class_fqn: &str,
        method: &str,
        descr: &str,
    ) -> Result<&Resolved, String> {
        if let Some(r) = self.cell.get() {
            return Ok(r);
        }
        let class = env
            .find_class(class_fqn)
            .map_err(|e| format!("find callback interface {class_fqn}: {e}"))?;
        let id = env
            .get_method_id(&class, method, descr)
            .map_err(|e| format!("resolve {class_fqn}.{method}{descr}: {e}"))?;
        let class = env
            .new_global_ref(&class)
            .map_err(|e| format!("global-ref callback interface {class_fqn}: {e}"))?;
        let _ = self.cell.set(Resolved {
            _class: class,
            method: id,
        });
        Ok(self.cell.get().expect("cell was just set"))
    }

    /// Invoke the interface method on `obj`, returning its `Object` result.
    /// Resolves (and pins) the interface class on first use.
    ///
    /// SAFETY contract carried for the caller: `obj` must implement the
    /// interface named by `class_fqn`, and `descr` must be the method's
    /// exact JVM descriptor — both are generated from the same plan, so
    /// they agree by construction.
    pub fn call_object<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        class_fqn: &str,
        method: &str,
        descr: &str,
        obj: &JObject,
        args: &[jvalue],
    ) -> Result<JObject<'local>, String> {
        let r = self.resolve(env, class_fqn, method, descr)?;
        // SAFETY: see the doc contract above; the GlobalRef pins the class.
        unsafe { env.call_method_unchecked(obj, r.method, ReturnType::Object, args) }
            .and_then(|v| v.l())
            .map_err(|e| format!("invoke {class_fqn}.{method}: {e}"))
    }
}

impl Default for CachedIfaceMethod {
    fn default() -> Self {
        Self::new()
    }
}
