//! The registry-owned key for captured Rust syntax.

use std::ops::Deref;

/// The capability handed to Rust-emission callbacks.
///
/// The flat crate owns the rendering protocol because it owns the syntax. The
/// registry owns this concrete key and its private constructor, so ordinary
/// registry users can receive an `Emit` only in `Prebindgen` and
/// `RegistryBuilder::convert_with` callbacks.
///
/// ```compile_fail
/// use prebindgen_registry::Emit;
/// let emit = Emit::new();
/// ```
///
/// ```compile_fail
/// use prebindgen_registry::Emit;
/// let emit = Emit(());
/// ```
///
/// A registry-only adapter cannot name the flat rendering protocol through
/// the registry's model re-export:
///
/// ```compile_fail
/// use prebindgen_registry::flat::emit::RustEmitter;
/// ```
#[derive(Debug)]
pub struct Emit(());

impl Emit {
    pub(crate) fn new() -> Self {
        Self(())
    }

    /// Construct an emission key for an out-of-crate adapter test.
    ///
    /// This is absent from normal builds. Production code receives the key
    /// only in an emission callback.
    #[cfg(any(test, feature = "testing"))]
    pub fn for_test() -> Self {
        Self::new()
    }
}

impl prebindgen_flat::RustEmitter for Emit {}

// Dereferencing to the protocol object keeps call sites ergonomic
// (`emit.spell(ty)`) without copying its method surface into this crate.
impl Deref for Emit {
    type Target = dyn prebindgen_flat::RustEmitter;

    fn deref(&self) -> &Self::Target {
        self
    }
}
