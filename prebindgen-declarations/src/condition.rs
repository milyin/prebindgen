//! Selector conditions used only while compiling declarations.
use proc_macro2::TokenStream;
use quote::quote;
/// A leaf is active only on its selected path. Adapters use the same condition
/// for foreign-side resource locking and native conversion code.
#[derive(Clone, Debug, Default)]
pub struct Condition(
    /// Conjunction of `(selector identifier, required arm index)` terms.
    /// An empty vector is always active.
    pub Vec<(syn::Ident, i32)>,
);
impl Condition {
    /// Return a new condition additionally requiring one selector arm.
    ///
    /// Does not modify the original condition; no additional runtime flag is generated.
    ///
    /// # Arguments
    ///
    /// * `tag` — identifier of the selector's index wire.
    /// * `index` — zero-based index of the required arm.
    pub fn arm(&self, tag: syn::Ident, index: i32) -> Self {
        let mut result = self.clone();
        result.0.push((tag, index));
        result
    }
    /// Emit this condition AND a Rust predicate, short-circuiting the predicate
    /// when the selector path is inactive. `predicate` is a Rust `bool` expression.
    pub fn and(&self, predicate: TokenStream) -> TokenStream {
        if self.0.is_empty() {
            predicate
        } else {
            let condition = self.rust();
            quote!((#condition) && (#predicate))
        }
    }
    /// Emit the conjunction as a Rust boolean expression; an empty condition emits `true`.
    pub fn rust(&self) -> TokenStream {
        self.0
            .iter()
            .map(|(tag, i)| quote!(#tag == #i))
            .reduce(|a, b| quote!((#a) && (#b)))
            .unwrap_or_else(|| quote!(true))
    }
}
