//! Explicit ownership policy for a converted output occurrence.
use std::collections::BTreeMap;

/// How a source value is supplied to its output converter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionMode {
    /// Observe through a shared reference; a handle converter may clone it.
    Borrowed,
    /// Transfer the owned value to its converter.
    Owned,
}
/// Explicit output policy, with optional overrides for named record fields.
///
/// The chosen mode applies recursively unless a field overrides it. Policies
/// never depend on whether the record contains handles. A borrowed parent cannot
/// supply an owned child. Adapters without named-record conversion reject field
/// overrides. Rust checks required `Clone` bounds and whether field moves are legal.
#[derive(Clone, Debug)]
pub struct Projection {
    /// Policy for this occurrence and all descendants without an override.
    pub mode: ProjectionMode,
    /// Overrides keyed by source field name.
    pub fields: BTreeMap<String, Projection>,
}
impl Projection {
    /// Observe the value and its fields through shared references.
    pub fn borrowed() -> Self {
        Self {
            mode: ProjectionMode::Borrowed,
            fields: BTreeMap::new(),
        }
    }
    /// Move the value and its fields into their converters.
    pub fn owned() -> Self {
        Self {
            mode: ProjectionMode::Owned,
            fields: BTreeMap::new(),
        }
    }
    /// Override one named field, including any nested policies.
    ///
    /// # Panics
    /// Panics if the field already has an override. Unknown fields are rejected
    /// by the adapter when preparing the occurrence.
    pub fn field(mut self, name: impl Into<String>, projection: Self) -> Self {
        assert!(
            self.fields.insert(name.into(), projection).is_none(),
            "duplicate projection field"
        );
        self
    }
    /// Policy inherited by an unlisted field, without this record's overrides.
    pub fn inherited(&self) -> Self {
        Self {
            mode: self.mode,
            fields: BTreeMap::new(),
        }
    }
}
