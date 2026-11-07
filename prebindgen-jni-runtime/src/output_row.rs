//! Constructors and fields for generated immutable JVM output rows.
use crate::{jni, Result};
use jni::{
    objects::{GlobalRef, JClass, JFieldID, JMethodID, JObject},
    JNIEnv,
};
use std::sync::OnceLock;

/// One generated row schema, registered from Kotlin before native calls begin.
/// Passing the actual class avoids application-class lookups on native workers.
#[derive(Default)]
pub struct OutputRowType(OnceLock<OutputRowConstructor>);
/// A pinned row class with constructor and raw-field IDs resolved at registration.
/// Borrow once per collection; constructing rows performs no metadata lookup.
pub struct OutputRowConstructor {
    class: GlobalRef,
    constructor: JMethodID,
    signature: Box<str>,
    fields: Box<[JFieldID]>,
}
impl OutputRowType {
    /// Create an unregistered generated row type.
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
    /// Register the generated class's raw constructor and `c0`, `c1`, … fields.
    /// JVM descriptors must follow the constructor's field order.
    pub fn initialize(
        &self,
        env: &mut JNIEnv<'_>,
        class: &JClass<'_>,
        descriptors: &[&str],
    ) -> Result<()> {
        let signature = format!("({})V", descriptors.concat());
        if let Some(previous) = self.0.get() {
            return previous.check_registration(env, class, &signature);
        }
        let constructor = env
            .get_method_id(class, "<init>", &signature)
            .map_err(|e| e.to_string())?;
        let fields = descriptors
            .iter()
            .enumerate()
            .map(|(i, descriptor)| {
                env.get_field_id(class, format!("c{i}"), descriptor)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>>>()?
            .into_boxed_slice();
        let class = env.new_global_ref(class).map_err(|e| e.to_string())?;
        if let Err(candidate) = self.0.set(OutputRowConstructor {
            class,
            constructor,
            signature: signature.into_boxed_str(),
            fields,
        }) {
            // Another initializer may have won while IDs were being resolved.
            // Accept only the same schema and class, never silently another loader.
            let class: &JClass<'_> = candidate.class.as_obj().into();
            self.0
                .get()
                .unwrap()
                .check_registration(env, class, &candidate.signature)?;
        }
        Ok(())
    }
    /// Borrow registered metadata before converting a collection.
    pub fn get(&self) -> Result<&OutputRowConstructor> {
        self.0
            .get()
            .ok_or_else(|| "output row type is not initialized".into())
    }
}
impl OutputRowConstructor {
    fn check_registration(
        &self,
        env: &mut JNIEnv<'_>,
        class: &JClass<'_>,
        signature: &str,
    ) -> Result<()> {
        if self.signature.as_ref() != signature
            || !env
                .is_same_object(self.class.as_obj(), class)
                .map_err(|e| e.to_string())?
        {
            return Err("output row type registered with a different class or schema".into());
        }
        Ok(())
    }
    /// Allocate a row using already prepared raw JVM arguments.
    ///
    /// # Safety
    /// Arguments must match the registered constructor descriptors and stay live
    /// through this invocation. Construction does not transfer native ownership.
    #[inline]
    pub unsafe fn construct<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        arguments: &[jni::sys::jvalue],
    ) -> Result<JObject<'local>> {
        let class: &JClass<'_> = self.class.as_obj().into();
        env.new_object_unchecked(class, self.constructor, arguments)
            .map_err(|e| e.to_string())
    }
    /// Field ID for a trusted generated row's raw field, used on rollback.
    pub fn field(&self, index: usize) -> JFieldID {
        self.fields[index]
    }
}
