//! The fixed-size-array length subgrammar: one closed representation and one
//! fallible walk that produces it.
//!
//! A length must reduce to a **known number** — a generator runs in `build.rs`,
//! where it cannot evaluate arbitrary Rust. Two spellings reach one, and nothing
//! else does: an integer literal, or the bare name of a `#[prebindgen]` const
//! whose own initializer is an integer literal.
//!
//! Both the number and the spelling travel, as an
//! [`ArrayExtent`](super::model::ArrayExtent): the value is the semantic length
//! that makes `[u8; A]` and `[u8; 4]` one type, and the const identity is what
//! lets a C header emit `uint8_t x[NAME]`. See that type for which consumer
//! needs which half.
//!
//! See the module docs of [`super`] for why acceptance and lowering are the same
//! operation here.

use std::{collections::HashMap, fmt};

use quote::ToTokens;

use super::model::{ArrayExtent, ConstId, ExtentSource};

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
/// Built once per registry. Deliberately holds **only consts**: nothing else can
/// be a length now that the grammar is a bare name, so there is no item-kind
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
            // Both halves travel: the value is the semantic length, the const
            // identity is what lets a C header spell `uint8_t x[NAME]`.
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

/// The [`syn::visit_mut::VisitMut`] pass that lowers array lengths.
///
/// Prefer [`resolve_array_lengths`], which drives this and gives the
/// transactional guarantee.
pub(crate) struct ArrayLenResolver<'a> {
    consts: &'a ConstIndex,
    item_crate: Option<&'a str>,
    found: Vec<(syn::Type, ArrayExtent)>,
    error: Option<UnsupportedArrayLen>,
}

impl<'a> ArrayLenResolver<'a> {
    pub(crate) fn new(consts: &'a ConstIndex, item_crate: Option<&'a str>) -> Self {
        Self {
            consts,
            item_crate,
            found: Vec::new(),
            error: None,
        }
    }

    /// The `(array type, length)` pairs found in walk order — array types in
    /// their rewritten spelling — or the first length that could not be
    /// lowered.
    pub(crate) fn finish(self) -> Result<Vec<(syn::Type, ArrayExtent)>, UnsupportedArrayLen> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(self.found),
        }
    }
}

impl syn::visit_mut::VisitMut for ArrayLenResolver<'_> {
    fn visit_type_array_mut(&mut self, arr: &mut syn::TypeArray) {
        if self.error.is_some() {
            return;
        }
        // Element first: a nested array's length is rewritten before this one's
        // type is recorded, so `found` holds canonical spellings throughout.
        syn::visit_mut::visit_type_mut(self, &mut arr.elem);
        if self.error.is_some() {
            return;
        }
        let rendered = arr.to_token_stream().to_string();
        match lower_array_len(&arr.len, &rendered, self.item_crate, self.consts) {
            Ok(extent) => {
                arr.len = extent.to_expr();
                self.found.push((syn::Type::Array(arr.clone()), extent));
            }
            Err(e) => self.error = Some(e),
        }
    }
}

/// Lower every fixed-size array length reachable from `node` and rewrite each to
/// its numeric form.
///
/// **Transactional**: the walk runs on a clone and is committed only if every
/// length lowered, so a refused node leaves no partially rewritten model.
///
/// `visit` selects the `syn` entry point for the node kind — e.g.
/// `|r, f| r.visit_item_fn_mut(f)`. Taking it from the caller keeps this generic
/// over the item kinds the registry indexes without a `syn::Item` round trip
/// that could only be unwrapped with an `unreachable!`.
///
/// Returns the `(array type, length)` pairs found, in walk order.
pub(crate) fn resolve_array_lengths<T: Clone>(
    node: &mut T,
    consts: &ConstIndex,
    item_crate: Option<&str>,
    visit: impl FnOnce(&mut ArrayLenResolver<'_>, &mut T),
) -> Result<Vec<(syn::Type, ArrayExtent)>, UnsupportedArrayLen> {
    let mut candidate = node.clone();
    let mut resolver = ArrayLenResolver::new(consts, item_crate);
    visit(&mut resolver, &mut candidate);
    let found = resolver.finish()?;
    *node = candidate;
    Ok(found)
}
