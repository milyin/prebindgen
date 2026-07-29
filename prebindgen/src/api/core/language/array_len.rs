//! The fixed-size-array length subgrammar: one closed representation and one
//! fallible walk that produces it.
//!
//! A length must reduce to a **known number** — a generator runs in `build.rs`,
//! where it cannot evaluate arbitrary Rust. Two spellings reach one, and nothing
//! else does: an integer literal, or the bare name of a `#[prebindgen]` const
//! whose own initializer is an integer literal.
//!
//! Both the number and the const identity travel, as an [`ArrayExtent`]: the
//! value is the semantic length that makes `[u8; A]` and `[u8; 4]` one type, and
//! the identity is what lets a C header emit `uint8_t x[NAME]`. The *spelling*
//! travels separately and always — it is in the [`Type::syntax`](super::Type)
//! slice of the array type itself — so this carries only what a destination
//! language cannot read off the source.
//!
//! Ported from #212, which introduced it for issue #210.

use std::{collections::HashMap, fmt};

use quote::ToTokens;

/// A length the prebindgen source language does not accept.
///
/// Names the offending sub-expression, not just the array: the point of the
/// single walk is that it knows exactly which part it could not lower.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedArrayLen {
    /// The array type as written, for context — `[u8 ; A + 1]`.
    pub array: String,
    /// The sub-expression that could not be lowered — `A + 1`.
    pub offending: String,
    /// Why it could not be lowered.
    pub reason: ArrayLenReason,
}

/// Why [`lower_array_len`] refused a length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayLenReason {
    /// Not a literal and not a plain name: arithmetic, a cast, a call, a block,
    /// a `match`, a closure — anything with structure the grammar lacks.
    NotLiteralOrName,
    /// A literal that is not a non-negative integer.
    NotAnIntegerLiteral,
    /// An integer literal too large for `usize`.
    IntegerOutOfRange,
    /// A path with more than one segment, a qualified self, or a leading `::` —
    /// `crate::limits::MAX`, `usize::MAX`, `<Holder>::N`, `::MAX`.
    ///
    /// A length names a `#[prebindgen]` const, and those live in one flat,
    /// uniquely-named namespace, so the bare name is the complete address. Any
    /// longer path either restates that (`crate::limits::MAX`) or reaches
    /// somewhere the frontend cannot follow — a module it does not index, an
    /// associated const it never captured, a foreign crate. Neither can be
    /// reduced to a number, and guessing between them is how a length silently
    /// becomes the wrong one.
    NotABareName,
    /// A bare name that is not a `#[prebindgen]` const.
    ///
    /// The generated crate sees **only** what the macro exposed, so an unmarked
    /// const is not merely unqualifiable — it does not exist downstream.
    NotAMarkedConst,
    /// A `#[prebindgen]` const whose own initializer is not an integer literal.
    ///
    /// `build.rs` cannot evaluate it, and a destination language that needs the
    /// count cannot either. Hoist the arithmetic into the value the const is
    /// computed FROM, or write the number.
    ConstIsNotALiteral,
    /// A `#[prebindgen]` const from a different source crate than the item
    /// using it.
    ///
    /// Uniqueness holds across the *marked* namespace only, so a bare name in
    /// one source crate can collide with an unmarked name of its own — the
    /// frontend would silently bind to the other crate's value. Requiring the
    /// length's const to come from the item's own crate makes that
    /// unrepresentable.
    ForeignSourceConst {
        /// Crate the const was marked in.
        const_crate: String,
        /// Crate the item using it came from.
        item_crate: String,
    },
}

impl fmt::Display for UnsupportedArrayLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match &self.reason {
            ArrayLenReason::NotLiteralOrName => {
                "is neither an integer literal nor the name of a const".to_string()
            }
            ArrayLenReason::NotAnIntegerLiteral => {
                "is not a non-negative integer literal".to_string()
            }
            ArrayLenReason::IntegerOutOfRange => "does not fit in a `usize`".to_string(),
            ArrayLenReason::NotABareName => {
                "is a path rather than a bare name; `#[prebindgen]` items live in one flat \
                 namespace, so the bare name is the whole address"
                    .to_string()
            }
            ArrayLenReason::NotAMarkedConst => {
                "names no `#[prebindgen]` const — the generated crate sees only what the macro \
                 exposed, so mark it `#[prebindgen]`"
                    .to_string()
            }
            ArrayLenReason::ConstIsNotALiteral => {
                "names a const whose value is not an integer literal, so `build.rs` cannot \
                 evaluate it"
                    .to_string()
            }
            ArrayLenReason::ForeignSourceConst {
                const_crate,
                item_crate,
            } => format!(
                "names a const marked in `{const_crate}`, but the item using it comes from \
                 `{item_crate}` — a length must name a const from its own source crate"
            ),
        };
        write!(
            f,
            "fixed-size array `{}`: the length `{}` {what}. A length must be an integer literal, \
             or the bare name of a `#[prebindgen]` const that is itself an integer literal \
             (`pub const N: usize = 4;`) — a generator runs in `build.rs` and cannot evaluate \
             anything else, and some destination languages need the count as a number.",
            self.array, self.offending
        )
    }
}

impl std::error::Error for UnsupportedArrayLen {}

/// A fixed-size array's extent: the number, and the const identity when the
/// source named one.
///
/// Both halves belong to **different consumers**:
///
/// * `value` is the semantic length. It is what makes `[u8; A]` and `[u8; 4]`
///   one Rust type and one converter, and what a destination language needs when
///   it has no way to reference a Rust const.
/// * `source` is the identity a C header re-states as `uint8_t tag[TAG_LEN]`,
///   so changing the size stays one edit rather than a hunt through literals.
///
/// This lives on the **use site** — a field's or parameter's type — and never on
/// anything keyed by type. Three fields whose extents are equal are one type, so
/// a type-keyed table could only report whichever was stored last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayExtent {
    pub value: usize,
    pub source: ExtentSource,
}

/// How an [`ArrayExtent`] was addressed at its use site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtentSource {
    /// Written as an integer literal — `[u8; 4]`.
    Literal,
    /// Written as the name of a `#[prebindgen]` const — `[u8; TAG_LEN]`.
    Const(ConstId),
}

/// A `#[prebindgen]` const, identified the way the flat namespace identifies
/// everything: by name, plus the crate it was marked in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstId {
    pub name: String,
    /// Crate the const was marked in; `None` for an origin-less stream.
    pub origin: Option<String>,
}

impl ArrayExtent {
    /// The const this extent named, if it named one.
    pub fn const_id(&self) -> Option<&ConstId> {
        match &self.source {
            ExtentSource::Literal => None,
            ExtentSource::Const(id) => Some(id),
        }
    }
}

/// One `#[prebindgen]` const, as a length sees it.
struct ConstEntry {
    /// The literal value, or `None` when the initializer is not one. Present
    /// either way, so "not a const" and "not a usable const" stay distinct
    /// diagnostics.
    value: Option<usize>,
    /// Crate the const was marked in; `None` for an origin-less stream.
    origin: Option<String>,
}

/// The `#[prebindgen]` consts a length may name.
///
/// Built once per parse, before any type is lowered, so a const may be declared
/// after the item that uses it. Deliberately holds **only consts**: nothing else
/// can be a length now that the grammar is a bare name, so there is no item-kind
/// enumeration here to drift.
pub(crate) struct ConstIndex {
    consts: HashMap<String, ConstEntry>,
}

impl ConstIndex {
    /// `consts` maps each `#[prebindgen]` const's name to its initializer and
    /// the crate it was marked in.
    pub(crate) fn new<I>(consts: I) -> Self
    where
        I: IntoIterator<Item = (String, syn::Expr, Option<String>)>,
    {
        Self {
            consts: consts
                .into_iter()
                .map(|(name, expr, origin)| {
                    let entry = ConstEntry {
                        value: int_literal(&expr),
                        origin,
                    };
                    (name, entry)
                })
                .collect(),
        }
    }
}

/// The `usize` an expression denotes, if it is plainly an integer literal.
fn int_literal(expr: &syn::Expr) -> Option<usize> {
    let syn::Expr::Lit(lit) = expr else {
        return None;
    };
    let syn::Lit::Int(int) = &lit.lit else {
        return None;
    };
    int.base10_parse::<usize>().ok()
}

/// Lower one array length to its closed representation.
///
/// **The contract**: `Ok` means the length was fully understood AND reduced to a
/// number. There is no separate acceptance check to drift from this — a form
/// this function does not lower is, by construction, a form the language does
/// not accept. That is the fix for the validator/rewriter pair this replaces
/// (issue #210), where eight defects in a row were two walks disagreeing about
/// one input.
///
/// `array` is the array type's rendered form and `item_crate` the origin of the
/// item the length was written in; both are used for diagnostics, and
/// `item_crate` additionally pins provenance.
pub(crate) fn lower_array_len(
    len: &syn::Expr,
    array: &str,
    item_crate: Option<&str>,
    consts: &ConstIndex,
) -> Result<ArrayExtent, UnsupportedArrayLen> {
    let fail = |reason| UnsupportedArrayLen {
        array: array.to_string(),
        offending: len.to_token_stream().to_string(),
        reason,
    };
    match len {
        syn::Expr::Lit(_) => match int_literal(len) {
            Some(value) => Ok(ArrayExtent {
                value,
                source: ExtentSource::Literal,
            }),
            None => Err(fail(match len {
                // Told apart so an out-of-range integer does not report as
                // "not an integer".
                syn::Expr::Lit(l) if matches!(l.lit, syn::Lit::Int(_)) => {
                    ArrayLenReason::IntegerOutOfRange
                }
                _ => ArrayLenReason::NotAnIntegerLiteral,
            })),
        },
        syn::Expr::Path(ep) => {
            // A bare name, and nothing longer. See `NotABareName` for why the
            // flat namespace makes every longer path either redundant or
            // unfollowable.
            if ep.qself.is_some() || ep.path.leading_colon.is_some() || ep.path.segments.len() != 1
            {
                return Err(fail(ArrayLenReason::NotABareName));
            }
            let name = ep.path.segments[0].ident.to_string();
            let Some(entry) = consts.consts.get(&name) else {
                return Err(fail(ArrayLenReason::NotAMarkedConst));
            };
            // Provenance before value: a same-named const from another source
            // may well be a literal, and using it would be the silent wrong
            // answer rather than an error.
            if entry.origin.as_deref() != item_crate {
                return Err(fail(ArrayLenReason::ForeignSourceConst {
                    const_crate: entry.origin.clone().unwrap_or_else(|| "<unstamped>".into()),
                    item_crate: item_crate.unwrap_or("<unstamped>").to_string(),
                }));
            }
            let Some(value) = entry.value else {
                return Err(fail(ArrayLenReason::ConstIsNotALiteral));
            };
            Ok(ArrayExtent {
                value,
                source: ExtentSource::Const(ConstId {
                    name,
                    origin: entry.origin.clone(),
                }),
            })
        }
        _ => Err(fail(ArrayLenReason::NotLiteralOrName)),
    }
}
