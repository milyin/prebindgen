//! Builders for source expansion choices; adapters supply their own node constructors.
use crate::DeclFunction;

/// Alternatives used to construct a source input, optionally as a borrowed temporary.
#[derive(Clone, Debug)]
pub struct DeclInputAlternatives<T> {
    alternatives: Vec<T>,
    reconstruct: bool,
}
impl<T> Default for DeclInputAlternatives<T> {
    fn default() -> Self {
        Self {
            alternatives: Vec::new(),
            reconstruct: false,
        }
    }
}
impl<T> DeclInputAlternatives<T> {
    /// Append an input alternative.
    pub fn push(&mut self, input: T) {
        self.alternatives.push(input);
    }
    /// Use an owned temporary to supply a shared-reference occurrence.
    pub fn reconstruct(&mut self) {
        self.reconstruct = true;
    }
    /// Build the alternatives using only the adapter operations needed here.
    /// An empty reconstructed expansion uses the owned type's default input.
    pub fn build(
        self,
        selector: impl FnOnce(Vec<T>) -> T,
        default: impl FnOnce() -> T,
        temporary: impl FnOnce(T) -> T,
    ) -> T {
        let input = if self.reconstruct && self.alternatives.is_empty() {
            default()
        } else {
            selector(self.alternatives)
        };
        if self.reconstruct {
            temporary(input)
        } else {
            input
        }
    }
}

/// Build one getter observation, preserving an explicit foreign name and rejecting
/// function settings that do not apply to a source accessor.
pub fn getter_view<I, O>(
    function: DeclFunction<I, O>,
    getter: impl FnOnce(String) -> O,
    named: impl FnOnce(String, O) -> O,
) -> O {
    let name = function.foreign.clone();
    let output = getter(function.reference_name(true));
    match name {
        Some(name) => named(name, output),
        None => output,
    }
}

/// A type-wide source-error mapping using the adapter's function declaration.
#[derive(Clone, Debug)]
pub struct DeclError<I, O> {
    /// Source error type.
    pub ty: syn::Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: DeclFunction<I, O>,
}
impl<I, O> DeclError<I, O> {
    /// Select the source helper for this error type.
    pub fn new(ty: syn::Type, mapper: DeclFunction<I, O>) -> Self {
        Self { ty, mapper }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Neither cloning nor an operation vocabulary is required to build alternatives.
    struct Input(String);
    fn alternatives(reconstruct: bool, values: &[&str]) -> String {
        let mut builder = DeclInputAlternatives::default();
        for value in values {
            builder.push(Input((*value).into()));
        }
        if reconstruct {
            builder.reconstruct();
        }
        builder
            .build(
                |inputs| {
                    Input(
                        inputs
                            .into_iter()
                            .map(|i| i.0)
                            .collect::<Vec<_>>()
                            .join(" | "),
                    )
                },
                || Input("default".into()),
                |input| Input(format!("borrow({})", input.0)),
            )
            .0
    }
    #[test]
    fn reconstructed_alternatives_use_defaults_only_when_empty() {
        assert_eq!(alternatives(false, &[]), "");
        assert_eq!(alternatives(true, &[]), "borrow(default)");
        assert_eq!(
            alternatives(true, &["constructor", "direct"]),
            "borrow(constructor | direct)"
        );
    }
    #[test]
    fn getter_views_preserve_explicit_names_and_plain_accessors() {
        let getter = |name| format!("{name}()");
        let named = |name, value| format!("{name}={value}");
        assert_eq!(
            getter_view(
                DeclFunction::<(), String>::new(syn::parse_quote!(accessor)).name("field"),
                getter,
                named
            ),
            "field=accessor()"
        );
        assert_eq!(
            getter_view(
                DeclFunction::<(), String>::new(syn::parse_quote!(second)),
                getter,
                named
            ),
            "second()"
        );
    }
}
