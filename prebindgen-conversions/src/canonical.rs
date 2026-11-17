//! Prepared single-value representations, independently of declaration syntax.
use syn::Type;

/// A Rust operation converting one value to or from its foreign representation.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueSource {
    /// Captured helper function, identified in the same Flat function table.
    Function(String),
    /// Fully qualified `Into`, or `TryInto` when fallible, for the stated types.
    Into {
        /// Whether the operation returns `Result` and can fail conversion.
        fallible: bool,
    },
}
/// One prepared direction of a source type's single-value representation.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueConversion {
    /// The Rust type representing the source value on the foreign side.
    pub representation: Type,
    /// Operation constructing the source value or producing its representation.
    pub source: ValueSource,
}
/// Independent input/output representations shared by occurrences of a type.
///
/// These are binding decisions, not extra source declarations or runtime state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValueConversions {
    /// Foreign representation into an owned source value.
    pub input: Option<ValueConversion>,
    /// Owned or borrowed source value into its foreign representation.
    pub output: Option<ValueConversion>,
}
