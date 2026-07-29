//! The prebindgen **source language**: one parser from captured records to
//! [`Element`]s.
//!
//! > Naming: `core::language` is the *source* language — the Rust subset a
//! > `#[prebindgen]` crate may write. `api::lang` is the *destination* adapters
//! > (C, JNI). They are opposite ends of the pipeline.
//!
//! ```text
//! Source(s) ──items──> Flat ──Elements──> Registry ──> adapters
//!   raw records          parse +               indexes       classify off `kind`
//!   (syn::Item)          validate              elements      spell off `origin`
//! ```
//!
//! [`Flat::source`] folds the first arrow in for the common case, so a build
//! script names one directory and gets elements; [`Flat::items`] keeps the
//! arrow itself, for a stream that needs shaping first.
//!
//! # What an element is
//!
//! Two things at once, and that pairing is the whole design:
//!
//! * a **closed classification** — [`TypeKind`], the field list, which of the two
//!   enum shapes an item is — that says what the source *means*, in terms every
//!   destination language shares;
//! * one [`Origin`], carrying the **exact syntax** the node was built from and
//!   the source it arrived in.
//!
//! The `Origin` is uniform: every node has one, at every level — item,
//! parameter, field, variant, type, array extent. Some levels know less than
//! others (a field has no line of its own, so it shares its item's), but the
//! shape does not change with the level, and no level copies a piece of
//! provenance down from the one above. That copying is what previously let the
//! same crate name appear under three field names with two meanings.
//!
//! So the rule for every consumer is:
//!
//! > **Classify off `kind`, spell off `origin.syntax`.**
//!
//! Matching a `syn::Type` or `syn::Expr` variant outside this module is a
//! classifier, and issue #211 says classification lives here alone. Passing a
//! node's [`Origin`] into `quote!` is spelling, and spelling the source is
//! exactly what generated Rust must do — see [`spell`] for the helpers that do
//! it.
//!
//! # What earns a variant
//!
//! A concept, not a Rust spelling. The test is whether a *destination* language
//! would act on the distinction; if only Rust can see it, it is spelling, and
//! the slice already carries it:
//!
//! | Rust writes | The model says | Because |
//! |---|---|---|
//! | `String`, `str` | [`TypeKind::Str`] | one concept, two Rust types |
//! | `Vec<T>`, `[T]` | [`TypeKind::Sequence`] | a run of `T`; owned vs borrowed is the [`Ref`](TypeKind::Ref) layer's fact |
//! | `Box<T>` | whatever `T` is | an owned `T` either way |
//! | `struct S;`, `struct S {}` | zero fields | the delimiters are spelling |
//! | `enum E { A(u8) }` | [`Variant`] | a sum, identified by position |
//! | `enum E { A = 7 }` | [`Enum`] | a named integer, identified by its value |
//! | no `->`, `-> ()` | [`TypeKind::Unit`] | the same function |
//! | `*const T` | *rejected* | a source crate is idiomatic Rust; the adapter owns pointers |
//!
//! The two enum shapes are the clearest case of a *concept* splitting where Rust
//! has one spelling. Both are `enum` and both keep a `syn::ItemEnum`, but a sum's
//! alternatives are identified by **position** — the mirror an adapter builds
//! carries no `repr` and numbers its own arms — while a fieldless enum's members
//! are identified by the **value Rust assigns**, which a C header re-states and a
//! Kotlin `enum class` entry carries. Neither numbering means anything for the
//! other shape, so one model covering both would carry a field that is dead in
//! each direction, and worse than dead: Rust does assign a discriminant to a
//! sum's alternatives, and using it would be wrong.
//!
//! The identities follow the same rule: a nominal type is a [`TypeId`] — a
//! name — not a `syn::Path`, so nothing downstream has to take a path apart to
//! learn what a type is. And a name is *all* it is: **a reference carries a
//! name, the declaration carries the origin**, so the same type never compares
//! unequal to itself because two source crates mentioned it. The one place a
//! crate name rides with an identity is [`ConstId`], and that is the const's
//! *declaring* crate, resolved by lookup — which is exactly what lets an array
//! extent refuse a const from another source.
//!
//! # Why the syntax rides along
//!
//! The generated Rust glue is itself a destination artifact, and the only one
//! that needs syntax fidelity: `B()` must not be re-spelled `B`, `= 0x07` must
//! not become `= 7`, `Foo<'a>` is not `Foo`. A model that carries no syntax has
//! to become *lossless* to serve it — which is how a language-neutral IR turns
//! back into a second `syn`. Carrying the original slice costs nothing and lets
//! the classification stay small: a lifetime, a delimiter and a literal's base
//! are simply not modelled facts.
//!
//! # Where acceptance is enforced
//!
//! Lowering is **total over the accepted grammar**: a form with no variant in
//! [`TypeKind`] is a form the language does not accept, so there is no second
//! acceptance list to drift from it.
//!
//! But an item the language cannot express is not automatically a build failure.
//! A source crate may mark items no binding uses, and those have never been
//! required to be expressible — the pipeline scans a signature only once an
//! adapter *declares* it. So parsing diagnoses per item and defers the raising:
//! such an item becomes [`Element::Unsupported`], carrying the diagnosis, inert
//! until something declares it. Only whole-stream rules — a duplicate name in
//! the flat namespace — are [`ParseError`]s, because no declaration can make
//! two items with one name unambiguous.
//!
//! There is **no verbatim passthrough**, because a `#[prebindgen]` crate marks
//! the items that cross the boundary and leaves the supporting code to the
//! consumer. The proc-macro already enforces that — a `use`, `mod`, `impl` or
//! `macro_rules!` cannot be marked at all — so an item kind this module does
//! not model is a `union` or a type alias, and it is diagnosed like any other
//! thing the language cannot express.
//!
//! # Shapes that must be refused rather than approximated
//!
//! An [`Element`] holds what it holds: ordinary parameters, a direct return, no
//! generic binder. A shape with no slot in that structure cannot be *partly*
//! accepted — the missing piece would simply be dropped, and silently:
//!
//! | Shape | Would become | So |
//! |---|---|---|
//! | `async fn` | a function returning `()` | the future is dropped and the export's body never runs |
//! | `fn f(a: u8, ...)` | a function without the tail | the variadic arguments vanish |
//! | `struct S<T>`, `fn f<T>()`, `struct S<const N: usize>` | `T` as a nominal reference | a parameter is indistinguishable from an item named `T` |
//!
//! All three are [`ItemError`]s, inert until declared, like any other refusal. A
//! **lifetime** binder is not among them: lifetimes are spelling, and the
//! spelling already travels. Nor is `impl Trait` in argument position — Rust
//! calls it an anonymous type parameter, but it is not a binder in the syntax,
//! so the callback form is untouched.

use std::{fmt, rc::Rc};

use quote::ToTokens;

mod array_len;
#[cfg(test)]
mod boundary;
mod element;
mod origin;
pub mod spell;
mod ty;

#[cfg(test)]
mod tests;

use self::{array_len::ConstIndex, ty::lower_type};
pub use self::{
    array_len::{ArrayExtent, ArrayLenReason, ConstId, ExtentSource, UnsupportedArrayLen},
    element::{
        Alternative, Const, Element, Enum, EnumValue, Field, Function, Param, Struct, Unsupported,
        Variant,
    },
    origin::Origin,
    ty::{ScalarKind, Type, TypeId, TypeKind, UnsupportedType, UnsupportedTypeReason},
};
use crate::SourceLocation;

/// The parser for the prebindgen source language.
///
/// Carries no configuration about what it *accepts* — that is a property of the
/// language, not of the call site. What it does carry is **what to parse**:
/// collect the inputs, then [`parse`](Self::parse) once.
///
/// # Reading a source directory
///
/// A build script's whole job, in one expression — the
/// [`Source`](crate::Source) step included. Pass
/// `<source_crate>::PREBINDGEN_OUT_DIR`:
///
/// ```
/// # prebindgen::Source::init_doctest_simulate();
/// use prebindgen::core::Flat;
///
/// let elements = Flat::new().source("source_ffi").parse()?;
/// assert_eq!(elements.len(), 2);
/// # Ok::<_, prebindgen::core::flat::ParseError>(())
/// ```
///
/// # Reading a stream
///
/// [`Self::items`] takes any `(syn::Item, SourceLocation)` iterator, so
/// everything a [`Source`](crate::Source) can express still composes — a group
/// selection, a renamed dependency, several sources at once. The feeders
/// accumulate, so mix them freely:
///
/// ```
/// # prebindgen::Source::init_doctest_simulate();
/// use prebindgen::{core::Flat, Source};
///
/// // A dependency renamed in Cargo.toml needs the name THIS crate uses, so it
/// // is configured rather than named by directory.
/// let helpers = Source::builder("source_ffi").crate_name("helpers").build();
/// let elements = Flat::new()
///     .items(helpers.items_in_groups(&["functions"]))
///     .parse()?;
/// assert_eq!(elements.len(), 1);
/// # Ok::<_, prebindgen::core::flat::ParseError>(())
/// ```
///
/// # Why accumulate, rather than parse each input
///
/// The rules that make a parse fail are **whole-stream** rules: one flat
/// namespace across every ingested crate, one const index an array length may
/// reach into, one set of source modules to normalize paths against. None can be
/// decided per input, so every input is in hand before any of it is classified.
#[derive(Debug, Default)]
pub struct Flat {
    items: Vec<(syn::Item, SourceLocation)>,
}

impl Flat {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every `#[prebindgen]` item captured in `dir`.
    ///
    /// Sugar for [`Self::items`] over [`Source::items_all`](crate::Source::items_all),
    /// which is the whole of what a build script normally needs — pass
    /// `<source_crate>::PREBINDGEN_OUT_DIR`. Reach for a
    /// [`Source`](crate::Source) directly, and feed it through [`Self::items`],
    /// only when it needs configuring.
    ///
    /// Panics the way [`Source::new`](crate::Source::new) does if `dir` is not
    /// readable prebindgen output: a build script has nothing to recover with.
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen::core::Flat;
    ///
    /// let elements = Flat::new().source("source_ffi").parse().unwrap();
    /// let mut names: Vec<String> =
    ///     elements.iter().filter_map(|e| e.name()).map(|n| n.to_string()).collect();
    /// names.sort();
    /// assert_eq!(names, ["TestStruct", "test_function"]);
    /// ```
    pub fn source<P: AsRef<std::path::Path>>(self, dir: P) -> Self {
        let source = crate::Source::new(dir);
        self.items(source.items_all())
    }

    /// Add a captured item stream.
    ///
    /// The general feeder: any `(syn::Item, SourceLocation)` iterator, so
    /// item-level selection and multi-source composition stay upstream where
    /// they already are. Call it as often as needed; the streams accumulate.
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen::{core::Flat, Source};
    ///
    /// let source = Source::new("source_ffi");
    /// let elements = Flat::new()
    ///     .items(source.items_in_groups(&["structs"]))
    ///     .parse()
    ///     .unwrap();
    /// assert_eq!(elements.len(), 1);
    /// ```
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        self.items.extend(items);
        self
    }

    /// Parse everything collected so far into elements.
    ///
    /// **Transactional**: an `Err` yields no elements at all, so a refused
    /// stream cannot leave a half-built model behind.
    ///
    /// Order-independent: source modules are gathered, and consts indexed,
    /// before anything is lowered — so a cross-source type reference and an
    /// array length may both name something declared later, in this input or
    /// another.
    pub fn parse(self) -> Result<Vec<Element>, ParseError> {
        let mut items = self.items;

        // Pass 0: normalize every item's types to the canonical flat spelling
        // before a single one is classified. `std::option::Option<T>` is an
        // `Option`, and `source_a::TypeA` is `TypeA` — decisions this module
        // owns, so it must be the one to see the reduced form. Gathering EVERY
        // module name first is what makes a cross-source reference in an
        // earlier item normalize the same as in a later one.
        //
        // The consequence is deliberate and stated on `Origin`: a slice
        // is the spelling generation must EMIT, which is the normalized one —
        // the flat namespace is what the generated crate can actually name.
        let mut modules: Vec<String> = Vec::new();
        for (_, loc) in &items {
            if let Some(crate_name) = &loc.crate_name {
                let module = crate_name.replace('-', "_");
                if !modules.contains(&module) {
                    modules.push(module);
                }
            }
        }
        for (item, _) in &mut items {
            crate::api::core::types_util::normalize_item_types(item, &modules);
        }

        // Pass 1: the consts an array length may name. Unnamed `const _` items
        // are excluded for the same reason `Element::name` skips them — they
        // are not addressable, so no length can name one.
        let consts = ConstIndex::new(items.iter().filter_map(|(item, loc)| match item {
            syn::Item::Const(c) if c.ident != "_" => Some((
                c.ident.to_string(),
                (*c.expr).clone(),
                loc.crate_name.clone(),
            )),
            _ => None,
        }));

        // Pass 2: lower, checking the flat namespace as we go.
        let mut out: Vec<Element> = Vec::with_capacity(items.len());
        let mut seen: Vec<(syn::Ident, SourceLocation)> = Vec::new();
        for (item, loc) in items {
            let element = lower_item(item, loc, &consts);
            if let Some(name) = element.name() {
                if let Some((first_name, first)) = seen.iter().find(|(n, _)| n == name) {
                    return Err(ParseError::DuplicateName(Box::new(DuplicateName {
                        name: first_name.clone(),
                        first: first.clone(),
                        second: element.location().clone(),
                    })));
                }
                seen.push((name.clone(), element.location().clone()));
            }
            out.push(element);
        }
        Ok(out)
    }
}

/// If `ty` is `impl Fn(T1, T2, ...) + Send + Sync + 'static`, return the `Fn`
/// argument types in declaration order. Otherwise `None`.
///
/// A callback **returns nothing**, and that is checked, not assumed: a written
/// `-> ()` is the same thing spelled out, and any other return is refused.
/// [`TypeKind::Callback`] has no slot for one, so accepting `impl Fn() -> u8`
/// would drop a fact a destination language needs — and drop it silently, which
/// is worse than the refusal.
///
/// The callback grammar, and the language's alone: [`TypeKind::Callback`] is
/// exactly what this accepts, so acceptance cannot drift from classification.
/// The registry re-exports it for the consumers that have not migrated yet.
pub fn extract_fn_trait_args(ty: &syn::Type) -> Option<Vec<syn::Type>> {
    let syn::Type::ImplTrait(it) = ty else {
        return None;
    };
    let mut args: Option<Vec<syn::Type>> = None;
    let mut has_send = false;
    let mut has_sync = false;
    let mut has_static = false;
    for bound in &it.bounds {
        match bound {
            syn::TypeParamBound::Trait(tb) => {
                let last = tb.path.segments.last()?;
                let name = last.ident.to_string();
                match name.as_str() {
                    "Fn" => {
                        let syn::PathArguments::Parenthesized(p) = &last.arguments else {
                            return None;
                        };
                        match &p.output {
                            syn::ReturnType::Default => {}
                            syn::ReturnType::Type(_, t) if ty::is_unit_type(t) => {}
                            syn::ReturnType::Type(..) => return None,
                        }
                        args = Some(p.inputs.iter().cloned().collect());
                    }
                    "Send" => has_send = true,
                    "Sync" => has_sync = true,
                    _ => return None,
                }
            }
            syn::TypeParamBound::Lifetime(lt) if lt.ident == "static" => has_static = true,
            _ => return None,
        }
    }
    if has_send && has_sync && has_static {
        args
    } else {
        None
    }
}

/// A rule of the language that no single item can satisfy on its own, and that
/// no adapter declaration can excuse.
#[derive(Clone, Debug)]
pub enum ParseError {
    /// Two `#[prebindgen]` items share a name. Names live in one flat namespace
    /// across every ingested source crate, so this is ambiguous however the
    /// crates are arranged.
    DuplicateName(Box<DuplicateName>),
}

/// The two items of a [`ParseError::DuplicateName`].
#[derive(Clone, Debug)]
pub struct DuplicateName {
    pub name: syn::Ident,
    pub first: SourceLocation,
    pub second: SourceLocation,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::DuplicateName(d) => write!(
                f,
                "duplicate `#[prebindgen]` name `{}`: first at {}, again at {} — marked items \
                 share one flat namespace across all source crates",
                d.name, d.first, d.second
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Why one item could not be expressed in the language.
///
/// Carried by [`Element::Unsupported`] rather than raised at parse time: see
/// the [module docs](self) on where acceptance is enforced.
#[derive(Clone, Debug)]
pub enum ItemError {
    /// A `self` receiver. `#[prebindgen]` captures free functions only.
    UnsupportedReceiver,
    /// A parameter pattern that is not a plain name — `(a, b): (u8, u8)`.
    UnsupportedParamPattern { pattern: String },
    /// A parameter's type is not in the language.
    ParamType {
        param: syn::Ident,
        source: UnsupportedType,
    },
    /// A return type is not in the language.
    ReturnType { source: UnsupportedType },
    /// A named struct field's type is not in the language.
    FieldType {
        field: syn::Ident,
        source: UnsupportedType,
    },
    /// A variant payload's type is not in the language.
    VariantFieldType {
        variant: syn::Ident,
        /// The field's name, or its position for a tuple variant.
        field: String,
        source: UnsupportedType,
    },
    /// A const's type is not in the language.
    ConstType { source: UnsupportedType },
    /// An `async fn`.
    ///
    /// The most dangerous shape to accept quietly: [`Function`] has a direct
    /// return, so an `async fn ping()` lowers as one returning `()`, and a
    /// generated wrapper calls it, drops the future and exports a function whose
    /// body never runs.
    UnsupportedAsync,
    /// A C-variadic tail — `fn f(a: u8, ...)`.
    ///
    /// [`Function`] holds ordinary parameters only, so the tail would simply be
    /// dropped from the signature.
    UnsupportedVariadic,
    /// A type or const generic parameter on the item.
    ///
    /// The elements have no generic binder, so a `T` in a field or parameter
    /// would lower as [`TypeKind::Named`] — an ordinary nominal reference into
    /// the flat namespace, indistinguishable from a real item called `T`. That
    /// loses the scoping every downstream resolver needs, and no destination
    /// language can express an uninstantiated parameter anyway.
    ///
    /// A lifetime parameter is *not* this: lifetimes are spelling and already
    /// travel in the syntax.
    UnsupportedGenericParam {
        param: String,
        /// `a type parameter` / `a const generic parameter`.
        kind: &'static str,
    },
    /// A whole item kind the language does not model — a `union`, a type alias.
    ///
    /// The proc-macro refuses to mark a `use`, `mod`, `impl` or `macro_rules!`
    /// at all, so only the kinds it accepts can reach here.
    UnsupportedItemKind { kind: &'static str },
}

impl fmt::Display for ItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemError::UnsupportedReceiver => write!(
                f,
                "takes a `self` receiver; `#[prebindgen]` captures free functions only"
            ),
            ItemError::UnsupportedParamPattern { pattern } => write!(
                f,
                "parameter pattern `{pattern}` is not a plain name — bind each parameter to one \
                 identifier"
            ),
            ItemError::ParamType { param, source } => {
                write!(f, "parameter `{param}`: {source}")
            }
            ItemError::ReturnType { source } => write!(f, "return type: {source}"),
            ItemError::FieldType { field, source } => write!(f, "field `{field}`: {source}"),
            ItemError::VariantFieldType {
                variant,
                field,
                source,
            } => write!(f, "variant `{variant}` field `{field}`: {source}"),
            ItemError::ConstType { source } => write!(f, "const type: {source}"),
            ItemError::UnsupportedAsync => write!(
                f,
                "is an `async fn`; the boundary has no way to drive a future, and the generated \
                 wrapper would drop it and export a function whose body never runs — expose a \
                 blocking wrapper instead"
            ),
            ItemError::UnsupportedVariadic => write!(
                f,
                "has a C-variadic tail, which the prebindgen source language does not model — \
                 take a slice, or one parameter per value"
            ),
            ItemError::UnsupportedGenericParam { param, kind } => write!(
                f,
                "declares `{param}`, {kind}: the prebindgen source language has no generic \
                 binder, so an uninstantiated parameter is indistinguishable from a nominal type \
                 of the same name and no destination language can express it — write the \
                 concrete types, one marked item per instantiation (a newtype is the usual way)"
            ),
            ItemError::UnsupportedItemKind { kind } => write!(
                f,
                "is {kind}; the prebindgen source language models functions, structs, enums and \
                 consts — everything else belongs in the consumer crate"
            ),
        }
    }
}

impl std::error::Error for ItemError {}

/// Lower one captured item. Total: every item becomes an element, and an item
/// whose contents the language cannot express becomes [`Element::Unsupported`]
/// rather than failing the parse.
fn lower_item(item: syn::Item, loc: SourceLocation, consts: &ConstIndex) -> Element {
    // One captured record is one item, so this is allocated once and shared by
    // the item and every node lowered out of it.
    let at = Rc::new(loc);
    match item {
        syn::Item::Fn(f) => match lower_fn(&f, &at, consts) {
            Ok(func) => Element::Function(func),
            Err(error) => unsupported(f.sig.ident.clone(), syn::Item::Fn(f), &at, error),
        },
        syn::Item::Struct(s) => match lower_struct(&s, &at, consts) {
            Ok(st) => Element::Struct(st),
            Err(error) => unsupported(s.ident.clone(), syn::Item::Struct(s), &at, error),
        },
        syn::Item::Enum(e) => match lower_enum(&e, &at, consts) {
            Ok(element) => element,
            Err(error) => unsupported(e.ident.clone(), syn::Item::Enum(e), &at, error),
        },
        // Including the unnamed `const _` each source injects as its feature
        // guard: it is a const, so it is one here. `Element::name` returns
        // `None` for `_`, which is what keeps several sources' guards from
        // colliding in the flat namespace.
        syn::Item::Const(c) => match lower_type(&c.ty, consts, &at) {
            Ok(ty) => Element::Const(Const {
                name: c.ident.clone(),
                ty,
                origin: Origin::new(c, at),
            }),
            Err(source) => unsupported(
                c.ident.clone(),
                syn::Item::Const(c),
                &at,
                ItemError::ConstType { source },
            ),
        },
        // An item kind the language does not model. The proc-macro accepts
        // only six kinds, so in practice this is a `union` or a type alias —
        // both named, neither ever written by a source crate. It is diagnosed
        // rather than carried: a `#[prebindgen]` crate marks what crosses the
        // boundary, and the code around that belongs to the consumer.
        other => {
            let (name, kind) = match &other {
                syn::Item::Union(u) => (Some(u.ident.clone()), "a union"),
                syn::Item::Type(t) => (Some(t.ident.clone()), "a type alias"),
                _ => (None, "an item kind"),
            };
            unsupported(name, other, &at, ItemError::UnsupportedItemKind { kind })
        }
    }
}

fn unsupported(
    name: impl Into<Option<syn::Ident>>,
    syntax: syn::Item,
    at: &Rc<SourceLocation>,
    error: ItemError,
) -> Element {
    Element::Unsupported(Unsupported {
        name: name.into(),
        error: Box::new(error),
        origin: Origin::new(syntax, Rc::clone(at)),
    })
}

/// Refuse a type or const generic parameter, naming the first one found.
///
/// Lifetimes pass: they say nothing a destination language can act on, and the
/// spelling that needs them is already in the syntax — the same call
/// [`lower_type`] makes for a lifetime *argument*.
fn reject_generic_params(generics: &syn::Generics) -> Result<(), ItemError> {
    for param in &generics.params {
        let (name, kind) = match param {
            syn::GenericParam::Lifetime(_) => continue,
            syn::GenericParam::Type(t) => (t.ident.to_string(), "a type parameter"),
            syn::GenericParam::Const(c) => (c.ident.to_string(), "a const generic parameter"),
        };
        return Err(ItemError::UnsupportedGenericParam { param: name, kind });
    }
    Ok(())
}

fn lower_fn(
    f: &syn::ItemFn,
    at: &Rc<SourceLocation>,
    consts: &ConstIndex,
) -> Result<Function, ItemError> {
    // Shapes `Function` has no slot for, and would therefore drop in silence.
    if f.sig.asyncness.is_some() {
        return Err(ItemError::UnsupportedAsync);
    }
    if f.sig.variadic.is_some() {
        return Err(ItemError::UnsupportedVariadic);
    }
    reject_generic_params(&f.sig.generics)?;
    let mut params = Vec::with_capacity(f.sig.inputs.len());
    for input in &f.sig.inputs {
        let pt = match input {
            syn::FnArg::Receiver(_) => return Err(ItemError::UnsupportedReceiver),
            syn::FnArg::Typed(pt) => pt,
        };
        let syn::Pat::Ident(pat) = &*pt.pat else {
            return Err(ItemError::UnsupportedParamPattern {
                pattern: pt.pat.to_token_stream().to_string(),
            });
        };
        let name = pat.ident.clone();
        let ty = lower_type(&pt.ty, consts, at).map_err(|source| ItemError::ParamType {
            param: name.clone(),
            source,
        })?;
        params.push(Param {
            name,
            ty,
            origin: Origin::new(pt.clone(), Rc::clone(at)),
        });
    }
    // An elided return and a written `-> ()` are the same function. The model
    // says so once, here, instead of leaving every consumer to normalize one to
    // the other — which is what they all do today, in eight separate copies.
    let ret = match &f.sig.output {
        syn::ReturnType::Default => Type {
            kind: TypeKind::Unit,
            origin: Origin::new(syn::parse_quote!(()), Rc::clone(at)),
        },
        syn::ReturnType::Type(_, t) => {
            lower_type(t, consts, at).map_err(|source| ItemError::ReturnType { source })?
        }
    };
    Ok(Function {
        name: f.sig.ident.clone(),
        params,
        ret,
        origin: Origin::new(f.clone(), Rc::clone(at)),
    })
}

fn lower_struct(
    s: &syn::ItemStruct,
    at: &Rc<SourceLocation>,
    consts: &ConstIndex,
) -> Result<Struct, ItemError> {
    reject_generic_params(&s.generics)?;
    let fields = match &s.fields {
        syn::Fields::Named(named) => {
            let mut out = Vec::with_capacity(named.named.len());
            for (index, f) in named.named.iter().enumerate() {
                let name = f.ident.clone().expect("named fields have idents");
                let ty = lower_type(&f.ty, consts, at).map_err(|source| ItemError::FieldType {
                    field: name.clone(),
                    source,
                })?;
                out.push(Field {
                    name: Some(name),
                    index,
                    ty,
                    origin: Origin::new(f.clone(), Rc::clone(at)),
                });
            }
            Some(out)
        }
        // Opaque: a tuple struct's contents are not a boundary surface, so they
        // are not lowered and a field type outside the grammar is not an error.
        syn::Fields::Unnamed(_) => None,
        syn::Fields::Unit => Some(Vec::new()),
    };
    Ok(Struct {
        name: s.ident.clone(),
        fields,
        origin: Origin::new(s.clone(), Rc::clone(at)),
    })
}

/// Lower an `enum` item to whichever of the two shapes it is.
///
/// **The classification**: any alternative with a field makes it a [`Variant`] —
/// a sum, numbered by position. Otherwise it is an [`Enum`] — a named set of
/// integers, identified by the value Rust assigns. Both are spelled `enum` in
/// Rust and both keep the `syn::ItemEnum`; only what a destination language can
/// do with them differs, and that is what the model records.
///
/// `enum E {}` has no alternative carrying anything, so it is the degenerate
/// `Enum`.
fn lower_enum(
    e: &syn::ItemEnum,
    at: &Rc<SourceLocation>,
    consts: &ConstIndex,
) -> Result<Element, ItemError> {
    reject_generic_params(&e.generics)?;

    if e.variants.iter().any(|v| !v.fields.is_empty()) {
        return Ok(Element::Variant(lower_variant(e, at, consts)?));
    }
    Ok(Element::Enum(lower_c_enum(e, at)))
}

/// The payload-carrying shape. Position is the only numbering a sum has, so no
/// discriminant is evaluated: the mirror an adapter builds numbers its own arms.
fn lower_variant(
    e: &syn::ItemEnum,
    at: &Rc<SourceLocation>,
    consts: &ConstIndex,
) -> Result<Variant, ItemError> {
    let mut alternatives = Vec::with_capacity(e.variants.len());
    for (index, v) in e.variants.iter().enumerate() {
        let mut fields = Vec::with_capacity(v.fields.len());
        for (field_index, f) in v.fields.iter().enumerate() {
            let ty =
                lower_type(&f.ty, consts, at).map_err(|source| ItemError::VariantFieldType {
                    variant: v.ident.clone(),
                    field: match &f.ident {
                        Some(id) => id.to_string(),
                        None => field_index.to_string(),
                    },
                    source,
                })?;
            fields.push(Field {
                name: f.ident.clone(),
                index: field_index,
                ty,
                origin: Origin::new(f.clone(), Rc::clone(at)),
            });
        }
        alternatives.push(Alternative {
            name: v.ident.clone(),
            index,
            fields,
            origin: Origin::new(v.clone(), Rc::clone(at)),
        });
    }
    Ok(Variant {
        name: e.ident.clone(),
        alternatives,
        origin: Origin::new(e.clone(), Rc::clone(at)),
    })
}

/// The fieldless shape. Nothing here can fail to lower — there are no field
/// types — so an unevaluable discriminant ends the numeric chain rather than
/// refusing the item.
fn lower_c_enum(e: &syn::ItemEnum, at: &Rc<SourceLocation>) -> Enum {
    let mut values = Vec::with_capacity(e.variants.len());
    // Rust's own numbering rule: an explicit `= N` sets the value, an implicit
    // one takes the previous plus one, starting at 0.
    let mut next: Option<i64> = Some(0);
    for (index, v) in e.variants.iter().enumerate() {
        let discriminant = match v.discriminant.as_ref() {
            Some((_, expr)) => int_literal(expr),
            None => next,
        };
        // `checked_add`: a discriminant at the top of the range is valid Rust
        // (`#[repr(u64)] enum E { A = i64::MAX as u64, B }`), so running out of
        // `i64` ends the numeric chain exactly as an unevaluable spelling does.
        // The spelling is untouched either way — it is in `EnumValue::origin`.
        next = discriminant.and_then(|n| n.checked_add(1));

        values.push(EnumValue {
            name: v.ident.clone(),
            index,
            discriminant,
            origin: Origin::new(v.clone(), Rc::clone(at)),
        });
    }
    Enum {
        name: e.ident.clone(),
        values,
        origin: Origin::new(e.clone(), Rc::clone(at)),
    }
}

/// Pull a signed integer out of a literal expression (`5`, `-3`, `0x07`).
/// `None` for anything else — a `const`, a path, arithmetic.
fn int_literal(expr: &syn::Expr) -> Option<i64> {
    i64::try_from(int_literal_wide(expr)?).ok()
}

/// [`int_literal`] before the range check.
///
/// The magnitude is parsed **wider than the result** so the sign can be applied
/// first: `-9223372036854775808` is `i64::MIN` and a valid Rust discriminant,
/// but its magnitude is one past `i64::MAX`, so parsing the digits as `i64`
/// would reject the whole literal. A magnitude too large for `i128` fails here
/// and is reported as an unevaluable discriminant, which is the existing
/// contract for anything the frontend cannot reduce to a number.
fn int_literal_wide(expr: &syn::Expr) -> Option<i128> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.base10_parse::<i128>().ok(),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => int_literal_wide(expr).map(|v| -v),
        _ => None,
    }
}
