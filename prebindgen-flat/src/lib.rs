//! # prebindgen-flat
//!
//! The independent flat parser and model that collectors build on.
//!
//! [`flat::Flat::builder`] parses `(syn::Item, [`SourceLocation`](prebindgen::SourceLocation))`
//! records — read through [`prebindgen::Source`] — into one flat
//! namespace — the language-agnostic index of everything a `#[prebindgen]`
//! source crate declared. The Registry-based pipeline that resolves a binding
//! over this model — type conversion, boundary expansion, Rust emission —
//! ships in the separate
//! [`prebindgen-registry`](https://docs.rs/prebindgen-registry) crate, which
//! re-exports this crate's model modules at its own
//! root so a language adapter names one crate for the whole pipeline.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved; see the separate
//! `prebindgen-c` and `prebindgen-jni` crates.

/// The rendering protocol for a collector-owned callback key.
///
/// The flat layer supplies the operations because it owns captured syntax; each
/// collector decides which concrete key implements them and where that key is
/// handed out. `prebindgen-registry` supplies its own private receiver behind
/// the unconstructable `RustWriter` it hands to final callbacks.
pub use crate::flat::emit::{Conditioned, RustEmitter};
pub mod flat;
pub mod pipeline;
pub mod shape;
pub mod types_util;

pub use self::flat::{Element, Flat, TypeKey, TypeKeyParseError};

/// A spelling with the spaces a reader does not want: `Option < Grade >` becomes
/// `Option<Grade>`, `# [cfg (unix)]` becomes `#[cfg(unix)]`, and the space in
/// `dyn Error` — the only kind that separates two words — stays.
///
/// Here because both the model's own output and the keys derived from it arrive
/// spaced, and everything that shows either to a person has the same problem: a
/// canonical type key spells a generic with spaces around its brackets, and a
/// `TokenStream` prints a space between every pair of tokens. Neither is wrong,
/// and neither is what a report, a build warning or a doc comment should show.
pub fn close_up(spaced: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let characters: Vec<char> = spaced.chars().collect();
    let mut out = String::with_capacity(spaced.len());
    for (index, &character) in characters.iter().enumerate() {
        if character == ' ' {
            let before = index.checked_sub(1).map(|i| characters[i]);
            let after = characters.get(index + 1).copied();
            let separates_words = before.is_some_and(word) && after.is_some_and(word);
            if !separates_words {
                continue;
            }
        }
        out.push(character);
    }
    out
}

#[cfg(test)]
mod close_up_tests {
    #[test]
    fn punctuation_closes_up_and_words_stay_apart() {
        for (key, expected) in [
            ("Option < Grade >", "Option<Grade>"),
            ("& [Payload]", "&[Payload]"),
            ("dyn Error", "dyn Error"),
            (
                "Result < Box < dyn Error > , u8 >",
                "Result<Box<dyn Error>,u8>",
            ),
            ("# [cfg (unix)]", "#[cfg(unix)]"),
            (
                "# [cfg (target_os = \"linux\")]",
                "#[cfg(target_os=\"linux\")]",
            ),
        ] {
            assert_eq!(super::close_up(key), expected, "{key}");
        }
    }
}
