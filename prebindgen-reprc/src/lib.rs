//! Shared C-representation records and target-measured layouts for binding generators.
//!
//! Schemas describe fields independently of a target. Measurements come from
//! the target compiler, either in a loaded native library or an external layout
//! artifact. This crate generates no language-specific transport or ownership policy.
#![warn(missing_docs)]

/// Binding-generation failure with a diagnostic message.
pub type Result<T> = std::result::Result<T, String>;
mod layout;
pub use layout::{LayoutArtifact, MeasuredLayout, SizeAlign, SourceFeatures};
mod native;
pub use native::NativeRecord;
mod record;
pub use record::Record;

/// Stable, target-independent identifier for a canonical record schema.
///
/// This is a compatibility identifier, not a cryptographic authentication hash.
pub fn schema_id(schema: &str) -> u64 {
    schema
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}
