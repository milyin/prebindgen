//! The fixed-size-array length subgrammar: one closed representation and one
//! fallible walk that produces it.
//!
//! See the module docs of [`super`] for why acceptance and rewriting are the
//! same operation here.

use std::{collections::HashMap, fmt};

use quote::ToTokens;

/// A fixed-size array's length, after the frontend has decided it.
///
/// This is the ONLY representation of a length downstream — qualification,
/// planning, and both language adapters read this rather than the captured
/// `syn::Expr`. The set of variants IS the accepted subgrammar: a length that
/// cannot be lowered to one of them is refused before any adapter runs.
///
/// The grammar is deliberately flat and non-recursive. Const arithmetic
/// (`[u8; A + 1]`), casts (`[u8; A as usize]`), and `const fn` calls
/// (`[u8; array_len()]`) are NOT part of the prebindgen language — hoist the
/// value into a named `const` and use that as the length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayLen {
    /// An integer literal — `[u8; 4]`.
    Literal(usize),
    /// A path naming a `#[prebindgen]` item, stored **absolute**: the item's
    /// origin module replaces whatever module prefix the source wrote, so the
    /// length resolves in the generated crate.
    ///
    /// Replacing rather than preserving the prefix is sound because prebindgen
    /// items live in one flat, uniquely-named namespace and are reachable as
    /// `<origin crate>::<bare name>` — the same invariant `normalize_type`
    /// already relies on for types. So every spelling of one item collapses to
    /// one value:
    ///
    /// * `MAX`, `crate::MAX`, `myflat::MAX`, `crate::limits::MAX` →
    ///   `myflat::MAX`;
    /// * `Holder::N`, `crate::limits::Holder::N` → `myflat::Holder::N`, where
    ///   the segments AFTER the item are relative to it and are kept.
    SourceConst { path: syn::Path },
    /// A path that does not name anything in a source crate — `usize::MAX`,
    /// `u8::BITS`, a foreign crate's path. Emitted **verbatim**, exactly as
    /// written: there is no origin module to prefix, and rewriting it would be
    /// a guess.
    ///
    /// A `#[prebindgen]`-less const referred to by its bare name lands here and
    /// will not resolve in the generated crate. Mark it `#[prebindgen]` so the
    /// registry indexes it. (Referred to *source-relatively* —
    /// `crate::limits::UNMARKED` — it is a hard error instead; see
    /// [`ArrayLenReason::UnresolvedSourcePath`].)
    ExternalConst { path: syn::Path },
}

impl ArrayLen {
    /// The length as it must be spelled in generated code.
    pub fn to_expr(&self) -> syn::Expr {
        match self {
            ArrayLen::Literal(n) => {
                let lit = syn::LitInt::new(&n.to_string(), proc_macro2::Span::call_site());
                syn::Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Int(lit),
                })
            }
            ArrayLen::SourceConst { path } | ArrayLen::ExternalConst { path } => {
                syn::Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path.clone(),
                })
            }
        }
    }
}

/// A length the prebindgen source language does not accept.
///
/// Names the offending sub-expression, not just the array type: the whole point
/// of the single walk is that it knows exactly which part it could not lower.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrayLenReason {
    /// Not a literal or a plain path: arithmetic, a cast, a call, a block, a
    /// `match`, a closure — anything with structure the grammar does not have.
    NotLiteralOrPath,
    /// A literal that is not a non-negative integer.
    NotAnIntegerLiteral,
    /// An integer literal too large for `usize`.
    IntegerOutOfRange,
    /// A path with a qualified self — `<Holder>::N`, `<Holder as Trait>::N`.
    /// The owner is a type, not a path segment, so there is no leading segment
    /// to resolve against the source namespace.
    QualifiedSelf,
    /// A path rooted at the crate root — `::MAX`. In the generated crate that
    /// names a dependency of the CONSUMER, which the frontend cannot see.
    CrateRootPath,
    /// A **source-relative** path — one headed by `crate` / `self` / a source
    /// crate's module — none of whose segments names a `#[prebindgen]` item.
    ///
    /// It claims to name something in the source crate, so it cannot be
    /// emitted verbatim: the generated crate is a different crate, where the
    /// same spelling resolves to something else or to nothing. Marking the
    /// intended item `#[prebindgen]` is the fix.
    UnresolvedSourcePath,
}

impl fmt::Display for UnsupportedArrayLen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.reason {
            ArrayLenReason::NotLiteralOrPath => {
                "is neither an integer literal nor a plain path to a const"
            }
            ArrayLenReason::NotAnIntegerLiteral => "is not a non-negative integer literal",
            ArrayLenReason::IntegerOutOfRange => "does not fit in a `usize`",
            ArrayLenReason::QualifiedSelf => {
                "uses a qualified self (`<T>::N` / `<T as Trait>::N`), whose owner is a type \
                 rather than a resolvable path segment"
            }
            ArrayLenReason::CrateRootPath => {
                "is rooted at `::`, which names a dependency of the generated crate rather than \
                 a `#[prebindgen]` item"
            }
            ArrayLenReason::UnresolvedSourcePath => {
                "is a source-relative path that names no `#[prebindgen]` item, so there is no \
                 module it can be qualified with — mark the intended item `#[prebindgen]`"
            }
        };
        write!(
            f,
            "fixed-size array `{}`: the length `{}` {what}. A length must be an integer literal \
             or a path to a `#[prebindgen]` const (`MAX`, `Holder::N`) — hoist anything else into \
             a named `const` and use that as the length.",
            self.array, self.offending
        )
    }
}

impl std::error::Error for UnsupportedArrayLen {}

/// What an indexed item may be within a length path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemRole {
    /// A struct or enum: may own the segments that follow it (`Holder::N`), or
    /// stand alone.
    Owner,
    /// A const or function: can only be the FINAL segment. Nothing is reachable
    /// *through* it, so a const named `limits` can never be the `limits` of
    /// `crate::limits::MAX` — that one is a module.
    Leaf,
}

/// The source namespace a length is resolved against.
///
/// Built once per registry, from every named item the registry indexes — a
/// length may name a const or the type that owns an associated const, and
/// enumerating item kinds at the use site is exactly the drift this module
/// exists to remove.
pub struct NameIndex {
    /// Indexed item name → the module it is reachable under (its origin crate,
    /// or the registry's default module) and what it may be in a path.
    names: HashMap<String, (syn::Path, ItemRole)>,
    /// Path heads that make a path SOURCE-RELATIVE: `crate`, `self`, and every
    /// ingested source crate's module name.
    module_heads: Vec<String>,
}

impl NameIndex {
    /// `names` maps each indexed item's ident to the module it is reachable
    /// under and its role; `source_modules` is the registry's ingested-source
    /// module list.
    pub fn new(names: HashMap<String, (syn::Path, ItemRole)>, source_modules: &[String]) -> Self {
        let mut module_heads = vec!["crate".to_string(), "self".to_string()];
        module_heads.extend(source_modules.iter().cloned());
        Self {
            names,
            module_heads,
        }
    }

    /// Does this path claim to name something in a `#[prebindgen]` source
    /// crate?
    ///
    /// True when the head is `crate` / `self` / a source crate's module, or is
    /// itself an indexed item. Everything else — `usize::MAX`, `u8::BITS`, a
    /// foreign crate's path — is external and is left alone.
    ///
    /// This is the FIRST decision, before anything is rewritten, so that
    /// [`ArrayLen::ExternalConst`]'s verbatim guarantee actually holds. It also
    /// mirrors `normalize_type`'s rule for types: an unknown crate path is
    /// never touched, because the registry has no index of a foreign namespace
    /// and `a::Holder` and `b::Holder` may be genuinely different things.
    fn is_source_relative(&self, path: &syn::Path) -> bool {
        let head = path.segments[0].ident.to_string();
        self.module_heads.contains(&head) || self.names.contains_key(&head)
    }

    /// Index of the segment that names the `#[prebindgen]` item this path
    /// denotes, or `None` if no segment does.
    ///
    /// Everything BEFORE the anchor is a module path within the source crate;
    /// everything after is relative to the item. Because prebindgen items live
    /// in one flat, uniquely-named namespace and are reachable as
    /// `<origin crate>::<bare name>`, the module prefix carries no information
    /// and is replaced wholesale — which is what lets `crate::limits::MAX`,
    /// `myflat::limits::MAX` and a bare `MAX` all mean the same length.
    ///
    /// Scanning is **leftmost-first** because a later segment may be an
    /// associated item rather than the anchor: in `Holder::N` with a free const
    /// `N` also indexed, `Holder` is the anchor and `N` is its associated
    /// const. A `Leaf` item is skipped when segments follow it, since nothing
    /// is reachable through a const or a function — that is the only case where
    /// a module and an indexed item can share a name, Rust putting modules and
    /// types in one namespace but consts in another.
    fn anchor(&self, path: &syn::Path) -> Option<usize> {
        let last = path.segments.len() - 1;
        path.segments.iter().enumerate().find_map(|(i, seg)| {
            let (_, role) = self.names.get(&seg.ident.to_string())?;
            (i == last || *role == ItemRole::Owner).then_some(i)
        })
    }

    /// The module an indexed item is reachable under.
    fn module_of(&self, ident: &str) -> Option<&syn::Path> {
        self.names.get(ident).map(|(module, _)| module)
    }
}

/// Lower one array length to its closed representation.
///
/// **The contract**: `Ok` means the length was fully understood AND fully
/// resolved. There is no separate acceptance check to drift from this — a form
/// this function does not lower is, by construction, a form the language does
/// not accept. That is the fix for the validator/rewriter pair this replaces
/// (issue #210): eight defects there were the two walks disagreeing about one
/// input.
///
/// `array` is the array type's rendered form, used only for the diagnostic.
pub fn lower_array_len(
    len: &syn::Expr,
    array: &str,
    names: &NameIndex,
) -> Result<ArrayLen, UnsupportedArrayLen> {
    let fail = |reason| UnsupportedArrayLen {
        array: array.to_string(),
        offending: len.to_token_stream().to_string(),
        reason,
    };
    match len {
        syn::Expr::Lit(lit) => {
            let syn::Lit::Int(int) = &lit.lit else {
                return Err(fail(ArrayLenReason::NotAnIntegerLiteral));
            };
            let n = int
                .base10_parse::<usize>()
                .map_err(|_| fail(ArrayLenReason::IntegerOutOfRange))?;
            Ok(ArrayLen::Literal(n))
        }
        syn::Expr::Path(ep) => {
            if ep.qself.is_some() {
                return Err(fail(ArrayLenReason::QualifiedSelf));
            }
            if ep.path.leading_colon.is_some() {
                return Err(fail(ArrayLenReason::CrateRootPath));
            }
            // Classify BEFORE rewriting anything: an external path is returned
            // byte-for-byte as written, which is the whole of its contract.
            if !names.is_source_relative(&ep.path) {
                return Ok(ArrayLen::ExternalConst {
                    path: ep.path.clone(),
                });
            }
            // Source-relative: find the segment that names the item, drop the
            // module prefix, keep everything after it.
            let Some(at) = names.anchor(&ep.path) else {
                return Err(fail(ArrayLenReason::UnresolvedSourcePath));
            };
            let anchor = ep.path.segments[at].ident.to_string();
            let Some(module) = names.module_of(&anchor) else {
                return Err(fail(ArrayLenReason::UnresolvedSourcePath));
            };
            let mut qualified = module.clone();
            qualified
                .segments
                .extend(ep.path.segments.iter().skip(at).cloned());
            Ok(ArrayLen::SourceConst { path: qualified })
        }
        _ => Err(fail(ArrayLenReason::NotLiteralOrPath)),
    }
}

/// The [`syn::visit_mut::VisitMut`] pass that lowers array lengths.
///
/// Prefer [`resolve_array_lengths`], which drives this and gives the
/// transactional guarantee. Use the pass directly only when the node must be
/// walked in place.
pub struct ArrayLenResolver<'a> {
    names: &'a NameIndex,
    found: Vec<(syn::Type, ArrayLen)>,
    error: Option<UnsupportedArrayLen>,
}

impl<'a> ArrayLenResolver<'a> {
    pub fn new(names: &'a NameIndex) -> Self {
        Self {
            names,
            found: Vec::new(),
            error: None,
        }
    }

    /// The `(array type, length)` pairs found in walk order — array types in
    /// their rewritten spelling — or the first length that could not be
    /// lowered.
    pub fn finish(self) -> Result<Vec<(syn::Type, ArrayLen)>, UnsupportedArrayLen> {
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
        match lower_array_len(&arr.len, &rendered, self.names) {
            Ok(len) => {
                arr.len = len.to_expr();
                self.found.push((syn::Type::Array(arr.clone()), len));
            }
            Err(e) => self.error = Some(e),
        }
    }
}

/// Lower every fixed-size array length reachable from `node` and rewrite each
/// to its canonical spelling.
///
/// **Transactional**: the walk runs on a clone and is committed only if every
/// length lowered, so a refused node leaves no partially rewritten model.
///
/// `visit` selects the `syn` entry point for the node kind — e.g.
/// `|r, f| r.visit_item_fn_mut(f)`. Taking it from the caller keeps this
/// generic over the item kinds the registry indexes without a `syn::Item`
/// round trip that could only be unwrapped with an `unreachable!`.
///
/// Returns the `(array type, length)` pairs found, in walk order.
pub fn resolve_array_lengths<T: Clone>(
    node: &mut T,
    names: &NameIndex,
    visit: impl FnOnce(&mut ArrayLenResolver<'_>, &mut T),
) -> Result<Vec<(syn::Type, ArrayLen)>, UnsupportedArrayLen> {
    let mut candidate = node.clone();
    let mut resolver = ArrayLenResolver::new(names);
    visit(&mut resolver, &mut candidate);
    let found = resolver.finish()?;
    *node = candidate;
    Ok(found)
}
