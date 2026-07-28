//! The Rust frontend: the single authority for what captured `#[prebindgen]`
//! source **means**.
//!
//! Everything downstream of this module — the registry's planning phases and
//! every language adapter — consumes the frontend's decisions rather than the
//! captured syntax. An adapter chooses how a value crosses ITS boundary and
//! how it appears in the destination language; it does not get to decide what
//! the source Rust means, or whether its spelling is allowed.
//!
//! The accepted subset is written down in `docs/source-language.md`.
//!
//! ## Why one walk per construct
//!
//! The frontend's rule is that **validation is a consequence of lowering**, not
//! a separate judgement. For each source construct there is one fallible
//! function from captured syntax to a closed representation; a form it cannot
//! lower is, by construction, a form the language does not accept.
//!
//! That is not a stylistic preference. Array lengths previously had two
//! independent walks — a whitelist deciding what was accepted and a rewriter
//! deciding what it could qualify — with nothing tying them together. They
//! disagreed eight times (issue #210), most sharply over `[u8; <Holder>::N]`,
//! which the whitelist accepted and the rewriter silently declined to qualify,
//! emitting a path the generated crate cannot resolve. With one walk that shape
//! is not representable.
//!
//! ## Scope today
//!
//! Only the fixed-size-array length subgrammar has moved here so far
//! ([`array_len`]); issue #211 tracks migrating the rest. Constructs not listed
//! in `docs/source-language.md` are still classified at their use sites.

mod array_len;

#[cfg(test)]
mod tests;

pub use self::array_len::{
    lower_array_len, resolve_array_lengths, ArrayLen, ArrayLenReason, ArrayLenResolver, NameIndex,
    UnsupportedArrayLen,
};
