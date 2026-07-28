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
    /// A path naming a `#[prebindgen]` item, stored **absolute**: the leading
    /// segment carries the item's origin module, so the length resolves in the
    /// generated crate.
    ///
    /// Covers a free const (`MAX` → `myflat::MAX`) and an associated const
    /// (`Holder::N` → `myflat::Holder::N`, where only the OWNER is qualified —
    /// `::N` is relative to it).
    SourceConst { path: syn::Path },
    /// A path naming nothing the registry indexes — `usize::MAX`, `u8::BITS`.
    /// Emitted verbatim: it is not a source item, so there is no origin module
    /// to prefix and rewriting it would be a guess.
    ///
    /// A `#[prebindgen]`-less const in the source crate lands here and will not
    /// resolve in the generated crate. Mark it `#[prebindgen]` so the registry
    /// indexes it.
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

/// The source namespace a length is resolved against.
///
/// Built once per registry, from every named item the registry indexes — a
/// length may name a const or the type that owns an associated const, and
/// enumerating item kinds at the use site is exactly the drift this module
/// exists to remove.
pub struct NameIndex {
    /// Indexed item name → the module it is reachable under (its origin crate,
    /// or the registry's default module).
    names: HashMap<String, syn::Path>,
    /// Path heads a captured length may already carry: `crate`, `self`, and
    /// every ingested source crate's module name. Stripped before resolution —
    /// see [`NameIndex::strip_source_head`].
    module_heads: Vec<String>,
}

impl NameIndex {
    /// `names` maps each indexed item's ident to the module it is reachable
    /// under; `source_modules` is the registry's ingested-source module list.
    pub fn new(names: HashMap<String, syn::Path>, source_modules: &[String]) -> Self {
        let mut module_heads = vec!["crate".to_string(), "self".to_string()];
        module_heads.extend(source_modules.iter().cloned());
        Self {
            names,
            module_heads,
        }
    }

    /// Drop a leading `crate` / `self` / source-crate segment, so a captured
    /// `crate::MAX` and a captured `MAX` resolve identically.
    ///
    /// The expression-path analogue of `normalize_type`'s path reduction, and
    /// deliberately NOT the same rule: that one reduces to the FINAL segment,
    /// which is right for a type (the last segment is the type name) and wrong
    /// here (`myflat::Holder::N` must keep its `Holder`, the owner of `N`).
    fn strip_source_head(&self, path: &syn::Path) -> syn::Path {
        if path.segments.len() < 2 {
            return path.clone();
        }
        let head = path.segments[0].ident.to_string();
        if !self.module_heads.contains(&head) {
            return path.clone();
        }
        let mut stripped = path.clone();
        stripped.segments = stripped.segments.into_iter().skip(1).collect();
        stripped
    }

    /// The module an indexed item is reachable under, or `None` when the name
    /// is not a source item.
    fn module_of(&self, ident: &str) -> Option<&syn::Path> {
        self.names.get(ident)
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
            let path = names.strip_source_head(&ep.path);
            // One segment names the const itself; more than one means the
            // leading segment is the type that owns it. Either way it is the
            // leading segment that carries the origin module, and the rest is
            // relative to it.
            let head = path.segments[0].ident.to_string();
            match names.module_of(&head) {
                Some(module) => {
                    let mut qualified = module.clone();
                    qualified.segments.extend(path.segments.iter().cloned());
                    Ok(ArrayLen::SourceConst { path: qualified })
                }
                None => Ok(ArrayLen::ExternalConst { path }),
            }
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
