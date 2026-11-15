//! The resolved Flat is the complete input to plan compilation.

/// A resolved operation tree, or a converter supplied explicitly by the user.
/// Existing custom converters remain authoritative unless a declaration overrides them.
#[derive(Clone, Debug)]
pub enum ResolvedSlot<T, P> {
    /// Resolved conversion composition.
    Tree(T),
    /// An explicitly supplied converter requiring no declaration compilation.
    Prepared(P),
}
