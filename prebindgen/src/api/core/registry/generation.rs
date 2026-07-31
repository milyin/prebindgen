//! A resolved binding, bound to the generator that resolved it.

use std::fmt;

use super::*;

/// A **resolved** binding generation: the [`Registry`] after
/// `RegistryBuilder::build` ran the scan, the plans, and the
/// resolution, bound together with the adapter that produced it. Both
/// halves of a generation run are methods here — [`Self::write_rust`] and
/// any adapter-specific artifact (e.g. `write_kotlin` for the JNI
/// adapter) — so the resolve-before-write ordering is enforced by
/// construction, and the writes themselves are pure reads that may run in
/// any order.
pub struct Generation<E: Prebindgen> {
    pub(super) registry: Registry<E::Metadata>,
    pub(super) adapter: E,
}

// Opaque — exists so `Result<Generation, _>::expect_err` works in tests.
impl<E: Prebindgen> fmt::Debug for Generation<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Generation(..)")
    }
}

impl<E: Prebindgen> Generation<E> {
    /// Write the generated Rust bindings file. `out_path` may be relative
    /// (resolved against `OUT_DIR`) or absolute; returns the path actually
    /// written. Pure emission — the registry was fully resolved by
    /// `RegistryBuilder::build`.
    pub fn write_rust(
        &self,
        out_path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, WriteRustError> {
        Ok(crate::api::core::write::write_rust(
            &self.registry,
            &self.adapter,
            out_path,
        )?)
    }

    /// The resolved registry (converter tables, plans, item maps).
    pub fn registry(&self) -> &Registry<E::Metadata> {
        &self.registry
    }

    /// The adapter this generation was resolved with.
    pub fn adapter(&self) -> &E {
        &self.adapter
    }
}
