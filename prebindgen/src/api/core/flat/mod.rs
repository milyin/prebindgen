//! The prebindgen **source language**: one parser from captured records to
//! [`Element`]s.
//!
//! > Naming: `core::flat` is the *source* side — the flat API a
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
//! | `type X = ..`, `struct X(..)` | [`Extern`] | named here; contents not modelled |
//! | `&mut MaybeUninit<T>` | [`RefMode::Out`] | an out-param slot the caller supplies |
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
//! **Parsing diagnoses; ingestion raises.** Those are two different points, and
//! the split is what lets one model serve both.
//!
//! Parsing never fails on a single item: an item the language cannot express
//! becomes [`Element::Unsupported`] carrying its diagnosis, and
//! [`FlatBuilder::build`] returns the model with it in place. Only whole-stream
//! rules — a duplicate name in the flat namespace — are [`ParseError`]s, because
//! no declaration can make two items with one name unambiguous. So a consumer
//! that wants to *inspect* what a source crate marked, refusals included, gets
//! exactly that from [`Flat::unsupported`].
//!
//! [`Registry`](crate::core::Registry) ingestion is where the diagnoses are raised.
//! Building a registry from this model **fails if any element is
//! `Unsupported`** — all of them at once, so a source crate that needs migrating
//! sees one list rather than one rebuild per item — and it fails before any
//! adapter declaration is examined. A binding is built against a model the
//! frontend could read in full, or it is not built.
//!
//! No **marked** item passes through verbatim, because a `#[prebindgen]` crate
//! marks the items that cross the boundary and leaves the supporting code to the
//! consumer. The proc-macro already enforces that — a `use`, `mod`, `impl` or
//! `macro_rules!` cannot be marked at all — so the only item kind left that this
//! module does not model is a `union`, and it is diagnosed like anything else the
//! language cannot express.
//!
//! The one item that *is* re-emitted verbatim is a [`Guard`] — an anonymous
//! const, which has no address and so cannot be part of an API addressed by name.
//! Today these are the feature checks [`Source`](crate::Source) injects on its own
//! behalf. Modelled rather than dropped because this module must be total over
//! what it is handed, and a separate element so nothing that consumes the API has
//! to remember to skip it.
//!
//! # Declaring a handle
//!
//! `#[prebindgen] pub type X = path::To<Thing>;` declares an [`Extern`]: it gives
//! a foreign or crate-private type a **name in the flat API** without claiming
//! anything about its contents. That is what makes the API closable — a handle
//! enters it deliberately rather than by being mentioned — and it is why a
//! reference can be required to resolve. A marked tuple struct declares the same
//! thing, since no adapter has ever crossed its fields.
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
//! All three are [`ItemError`]s, carried like any other refusal and raised at
//! registry ingestion. A
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
        Alternative, Constant, Element, Enum, EnumValue, Extern, Field, Function, Guard, Param,
        Struct, Type, Unsupported, Variant,
    },
    origin::Origin,
    ty::{RefMode, ScalarKind, TypeId, TypeKind, TypeRef, UnsupportedType, UnsupportedTypeReason},
};
use crate::SourceLocation;

/// Collects what to parse, then hands over the model.
///
/// Carries no configuration about what the language *accepts* — that is a
/// property of the language, not of the call site. What it carries is **what to
/// parse**: feed the inputs, then [`build`](Self::build) once.
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
/// let flat = Flat::builder().source("source_ffi").build()?;
/// assert!(flat.function("test_function").is_some());
/// assert!(flat.declared_type("TestStruct").is_some());
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
/// let flat = Flat::builder()
///     .items(helpers.items_in_groups(&["functions"]))
///     .build()?;
/// assert_eq!(flat.functions().count(), 1);
/// # Ok::<_, prebindgen::core::flat::ParseError>(())
/// ```
///
/// # Why accumulate, rather than parse each input
///
/// The rules that make a parse fail are **whole-stream** rules: one flat
/// namespace across every ingested crate, one const index an array length may
/// reach into, one set of source modules to normalize paths against, and every
/// type reference resolving against every declaration. None can be
/// decided per input, so every input is in hand before any of it is classified.
#[derive(Debug, Default)]
pub struct FlatBuilder {
    items: Vec<(syn::Item, SourceLocation)>,
}

impl FlatBuilder {
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
    /// let flat = Flat::builder().source("source_ffi").build()?;
    /// assert!(flat.function("test_function").is_some());
    /// assert!(flat.declared_type("TestStruct").is_some());
    /// # Ok::<_, prebindgen::core::flat::ParseError>(())
    /// ```
    pub fn source<P: AsRef<std::path::Path>>(self, dir: P) -> Self {
        let source = crate::Source::new(dir);
        self.items(source.items_all())
    }

    /// The same, for a dependency this crate **renames** in `Cargo.toml`.
    ///
    /// The origin recorded at capture time is the dependency's real package name,
    /// which will not resolve from a crate that refers to it by another name.
    /// `crate_name` is the name *this* crate uses.
    ///
    /// Per directory, deliberately: an override on the whole parse could only fix
    /// one module, and a flat API may layer several sources.
    pub fn source_named<P: AsRef<std::path::Path>>(
        self,
        dir: P,
        crate_name: impl Into<String>,
    ) -> Self {
        let source = crate::Source::builder(dir).crate_name(crate_name).build();
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
    /// let flat = Flat::builder()
    ///     .items(source.items_in_groups(&["structs"]))
    ///     .build()?;
    /// assert_eq!(flat.types().count(), 1);
    /// # Ok::<_, prebindgen::core::flat::ParseError>(())
    /// ```
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        self.items.extend(items);
        self
    }

    /// Parse everything collected so far into the model.
    ///
    /// **Transactional**: an `Err` yields no model at all, so a refused stream
    /// cannot leave a half-built one behind.
    ///
    /// Order-independent: source modules are gathered, consts indexed, and every
    /// item lowered before any reference is resolved — so a type reference, an
    /// array length and a cross-source mention may each name something declared
    /// later, in this input or another.
    pub fn build(self) -> Result<Flat, ParseError> {
        let mut items = self.items;

        // Pass 0: normalize every item's types to the canonical flat spelling
        // before a single one is classified. `std::option::Option<T>` is an
        // `Option`, `source_a::TypeA` is `TypeA`, and `zenoh::Session` is whatever
        // an alias named it — decisions this module owns, so it must be the one to
        // see the reduced form. Gathering EVERY module and alias first is what
        // makes a reference in an earlier item normalize the same as in a later
        // one.
        //
        // The consequence is deliberate and stated on `Origin`: a slice
        // is the spelling generation must EMIT, which is the normalized one —
        // the flat namespace is what the generated crate can actually name.
        let normalization = crate::api::core::types_util::Normalization::from_items(&items);
        for (item, _) in &mut items {
            crate::api::core::types_util::normalize_item_types(item, &normalization);
        }

        // Pass 1: the consts an array length may name. Unnamed items are
        // excluded because no length can name one — the same fact that makes
        // them `Guard`s. This is the one place that tests the spelling rather
        // than the classification, and it has to: it runs before Pass 2, so no
        // classification exists yet.
        let consts = ConstIndex::new(items.iter().filter_map(|(item, loc)| match item {
            syn::Item::Const(c) if c.ident != "_" => Some((
                c.ident.to_string(),
                (*c.expr).clone(),
                loc.crate_name.clone(),
            )),
            _ => None,
        }));

        // Pass 2: lower, checking the flat namespace as we go.
        let mut elements: Vec<Element> = Vec::with_capacity(items.len());
        let mut seen: Vec<(syn::Ident, SourceLocation)> = Vec::new();
        for (item, loc) in items {
            let element = lower_item(item, loc, &consts);
            if let Some(name) = element.name() {
                if let Some((first_name, first)) = seen.iter().find(|(n, _)| n == name) {
                    return Err(ParseError::DuplicateName(Box::new(DuplicateName {
                        name: first_name.clone(),
                        first: first.clone(),
                        second: element.location().clone(),
                        first_crate: first.crate_name.clone(),
                        second_crate: element.location().crate_name.clone(),
                    })));
                }
                seen.push((name.clone(), element.location().clone()));
            }
            elements.push(element);
        }

        // Pass 3: resolve references, now that every declaration is in hand.
        resolve_references(&mut elements);

        // Indexed after resolution, because refusing an item can change its kind
        // (a `Type` becomes `Unsupported`) though never its name.
        let by_name = elements
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.name().map(|n| (n.to_string(), i)))
            .collect();
        // Frozen here, from the captured stream alone. See the field's docs.
        let mut source_modules: Vec<String> = Vec::new();
        for element in &elements {
            if let Some(crate_name) = element.location().crate_name.as_ref() {
                let module = crate_name.replace('-', "_");
                if !source_modules.contains(&module) {
                    source_modules.push(module);
                }
            }
        }
        Ok(Flat {
            elements,
            by_name,
            source_modules,
        })
    }
}

/// The flat API: every `#[prebindgen]` item from every ingested source, parsed,
/// indexed by name, and with every type reference resolved.
///
/// # Direct access, not a stream
///
/// Names are unique across the whole model — a duplicate is a
/// [`ParseError::DuplicateName`] — so a name is a complete address, and the
/// model answers by it. That is what every later stage needs: an adapter asks
/// what a declared name *is*, rather than scanning a list for it.
///
/// # References are already resolved
///
/// Every [`TypeKind::Named`] in a surviving element denotes a [`Type`] this model
/// holds, and [`Self::resolve`] hands it over. An item that named something the
/// flat API does not declare is [`Element::Unsupported`] with
/// [`ItemError::UnresolvedType`], exactly like every other refusal — carried
/// here, raised by [`Registry`](crate::core::Registry) ingestion.
///
/// Resolving here rather than in the adapters is the point of #211: a dangling
/// name used to surface much later as an unresolved-converter error, from
/// whichever adapter happened to look first.
#[derive(Debug, Default)]
pub struct Flat {
    /// Source order, so iteration reports items as the sources were fed.
    elements: Vec<Element>,
    /// Module name of every **captured** source, in first-seen order (crate
    /// names, dashes normalized to underscores). The first doubles as the
    /// default module for a reference with no recorded origin.
    ///
    /// Computed once in [`FlatBuilder::build`] and frozen: it is a property of
    /// the ingested stream, so a binding-local function added later must not
    /// extend it — that would change which module an unqualified reference
    /// resolves against.
    source_modules: Vec<String>,
    /// Name → position in [`Self::elements`].
    ///
    /// A map rather than a scan because every typed accessor and every
    /// [`Self::resolve`] routes through it, and later stages resolve references
    /// in a loop — a linear scan would make that quadratic in the size of the
    /// API. Positions rather than clones, so there is one copy of each element
    /// and source order stays available.
    by_name: std::collections::HashMap<String, usize>,
}

/// A name a lookup can be performed with.
///
/// Exists because callers hold different spellings of the same fact: an adapter
/// walking captured items has a `syn::Ident`, a resolved reference has the
/// `String` inside a [`TypeId`], and a test has a literal. One accessor takes all
/// three rather than each call site converting.
///
/// **The conversion is moved, not removed.** `proc_macro2::Ident` hashes by
/// `to_string()` and offers no borrow as `str`, so an `Ident` lookup allocates
/// wherever it happens; doing it here keeps `&str` and `&String` callers — among
/// them the per-edge and per-reference lookups in the scan and the resolver —
/// allocation-free.
///
/// Sealed: what may name an element is the language's business, not a caller's.
///
/// ```
/// # prebindgen::Source::init_doctest_simulate();
/// use prebindgen::core::flat::Flat;
///
/// let flat = Flat::builder().source("source_ffi").build()?;
/// let ident = quote::format_ident!("test_function");
///
/// // The same element, whichever spelling the caller happens to hold.
/// assert!(flat.function("test_function").is_some());
/// assert!(flat.function(&ident).is_some());
/// # Ok::<_, prebindgen::core::flat::ParseError>(())
/// ```
pub trait Name: sealed::Sealed {
    /// The name as a string, borrowed when the caller already holds one.
    fn as_name(&self) -> std::borrow::Cow<'_, str>;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for str {}
    impl Sealed for String {}
    impl Sealed for syn::Ident {}
    impl<T: ?Sized + Sealed> Sealed for &T {}
}

impl Name for str {
    fn as_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(self)
    }
}

impl Name for String {
    fn as_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(self)
    }
}

impl Name for syn::Ident {
    fn as_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(self.to_string())
    }
}

/// So a caller already holding a reference does not have to reborrow.
impl<T: ?Sized + Name> Name for &T {
    fn as_name(&self) -> std::borrow::Cow<'_, str> {
        T::as_name(self)
    }
}

impl Flat {
    /// Start collecting what to parse.
    pub fn builder() -> FlatBuilder {
        FlatBuilder { items: Vec::new() }
    }

    /// Every element, in the order the sources were fed.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    /// The element with this name, whatever kind it is — including an
    /// [`Element::Unsupported`], which still holds its name against the
    /// namespace.
    pub fn element<N: Name + ?Sized>(&self, name: &N) -> Option<&Element> {
        self.elements
            .get(*self.by_name.get(name.as_name().as_ref())?)
    }

    pub fn function<N: Name + ?Sized>(&self, name: &N) -> Option<&Function> {
        match self.element(name)? {
            Element::Function(f) => Some(f),
            _ => None,
        }
    }

    /// The type declared under this name.
    ///
    /// Named `declared_type` because `type` is a keyword; it is the accessor a
    /// resolved [`TypeKind::Named`] reference leads to, and [`Self::resolve`] is
    /// the same lookup taking a [`TypeId`].
    pub fn declared_type<N: Name + ?Sized>(&self, name: &N) -> Option<&Type> {
        match self.element(name)? {
            Element::Type(t) => Some(t),
            _ => None,
        }
    }

    pub fn constant<N: Name + ?Sized>(&self, name: &N) -> Option<&Constant> {
        match self.element(name)? {
            Element::Constant(c) => Some(c),
            _ => None,
        }
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.elements.iter().filter_map(|e| match e {
            Element::Function(f) => Some(f),
            _ => None,
        })
    }

    pub fn types(&self) -> impl Iterator<Item = &Type> {
        self.elements.iter().filter_map(|e| match e {
            Element::Type(t) => Some(t),
            _ => None,
        })
    }

    pub fn constants(&self) -> impl Iterator<Item = &Constant> {
        self.elements.iter().filter_map(|e| match e {
            Element::Constant(c) => Some(c),
            _ => None,
        })
    }

    /// Every type the API **mentions**, at every nesting depth — as distinct from
    /// [`Self::types`], which is every type it **declares**.
    ///
    /// A parameter, a return, a field, a constant's type, and everything reachable
    /// inside those. The same type mentioned in several places yields one
    /// [`TypeRef`] per mention, each with its own spelling and origin; a consumer
    /// that wants one per type indexes them and picks, and element order makes
    /// that pick deterministic.
    ///
    /// This is how a later stage gets the frontend's reading of a type it holds
    /// only as syntax, without lowering it a second time.
    pub fn type_refs(&self) -> impl Iterator<Item = &TypeRef> {
        self.elements
            .iter()
            .flat_map(element_type_refs)
            .flat_map(TypeRef::walk)
    }

    /// The `struct` declared under this name, or `None` for any other shape.
    ///
    /// A tuple struct is an [`Extern`] rather than a `Struct`, so this answers
    /// only for a product of fields that cross the boundary.
    pub fn struct_type<N: Name + ?Sized>(&self, name: &N) -> Option<&Struct> {
        match self.declared_type(name)? {
            Type::Struct(s) => Some(s),
            _ => None,
        }
    }

    /// The `syn::ItemEnum` behind **either** enum shape.
    ///
    /// A sum and a C-style enum are different elements — numbered differently
    /// and consumed as different constructs — but both were spelled `enum` in
    /// Rust and both keep that item. A consumer re-emitting the source wants the
    /// item without caring which shape it is; one that acts on the distinction
    /// reaches for [`Self::declared_type`].
    pub fn enum_item<N: Name + ?Sized>(&self, name: &N) -> Option<&syn::ItemEnum> {
        match self.declared_type(name)? {
            Type::Variant(v) => Some(&v.origin.syntax),
            Type::Enum(e) => Some(&e.origin.syntax),
            _ => None,
        }
    }

    /// Module name of every captured source, in first-seen order.
    ///
    /// The first entry is the default module for a reference with no recorded
    /// origin. Empty for a hand-built stream that carried no crate stamps.
    pub fn source_modules(&self) -> &[String] {
        &self.source_modules
    }

    /// Every anonymous const, in stream order — **zero or more**.
    ///
    /// Not part of the flat API — see [`Guard`] — but ingested with it, and a
    /// consumer that re-emits the source must re-emit these too.
    pub fn guards(&self) -> impl Iterator<Item = &Guard> {
        self.elements.iter().filter_map(|e| match e {
            Element::Guard(g) => Some(g),
            _ => None,
        })
    }

    /// Every item the language could not express, with its diagnosis.
    ///
    /// Present in the model so a consumer can inspect what a source crate marked
    /// — building a [`Registry`](crate::core::Registry) from a model holding any of
    /// these fails, and reports all of them. See the [module docs](self) on where
    /// acceptance is enforced.
    pub fn unsupported(&self) -> impl Iterator<Item = &Unsupported> {
        self.elements.iter().filter_map(|e| match e {
            Element::Unsupported(u) => Some(u),
            _ => None,
        })
    }

    /// Lower a function signature written outside the captured stream.
    ///
    /// For the **one input that does not come through this module**: a binding's
    /// `local_functions`, whose signatures are written by hand in a build script
    /// and inserted straight into the registry. Everything else was already
    /// lowered here, so this exists to keep the grammar decided in one place
    /// rather than re-checked at the far end.
    ///
    /// Grammar only, and it **validates by lowering**: an `Err` is a shape the
    /// language cannot express, an `Ok` is the element to admit. Whether the types
    /// it names are *declared* is a whole-model question ([`resolve_references`]),
    /// and a binding-local fn may legitimately name types the source crate never
    /// did.
    pub fn lower_signature(&self, f: &syn::ItemFn) -> Result<Function, ItemError> {
        // Rebuilt from the model rather than kept: this runs once per local fn,
        // and a stored index would be a second copy of what `constants()` says.
        let consts = ConstIndex::new(self.constants().map(|c| {
            (
                c.name.to_string(),
                (*c.origin.syntax.expr).clone(),
                c.origin.crate_name().map(str::to_owned),
            )
        }));
        // A synthesized fn has no captured location, but it does have an origin
        // crate — the caller supplies it, and `add_local_function` records it.
        let at = Rc::new(SourceLocation::default());
        lower_fn(f, &at, &consts)
    }

    /// Admit a binding-local function: one a build script wrote via `sig!(..)`
    /// rather than one a source crate marked.
    ///
    /// The model is the pipeline's only index, so a function nothing captured
    /// still has to live here or nothing downstream can find it. `crate_name` is
    /// the module its generated call qualifies against, stamped onto the
    /// element's location where [`Element::location`] already looks for it.
    ///
    /// Deliberately does **not** extend [`Self::source_modules`]: see that
    /// field's docs.
    pub(crate) fn add_local_function(&mut self, mut f: Function, crate_name: String) {
        f.origin.location = Rc::new(SourceLocation {
            crate_name: Some(crate_name),
            ..SourceLocation::default()
        });
        self.by_name.insert(f.name.to_string(), self.elements.len());
        self.elements.push(Element::Function(f));
    }

    /// The declaration a reference denotes.
    ///
    /// Infallible in practice for any reference reached from a surviving element:
    /// [`FlatBuilder::build`] made unresolvable references into
    /// [`ItemError::UnresolvedType`], so what is left resolves.
    pub fn resolve(&self, id: &TypeId) -> Option<&Type> {
        self.declared_type(&id.name)
    }
}

/// Turn every element that names an undeclared type into an
/// [`Element::Unsupported`], **transitively**.
///
/// Runs once every declaration is in hand, so the order sources were fed in does
/// not matter and a reference may point forward or across crates.
///
/// # Why this iterates
///
/// Refusing a type *removes a declaration*, which can strand its dependents:
///
/// ```ignore
/// pub struct Broken { pub field: Missing }   // refused: `Missing` undeclared
/// pub fn use_broken(value: Broken) {}        // `Broken` is now gone too
/// ```
///
/// A single pass against a snapshot of the initial declarations would keep
/// `use_broken`, and [`Flat::resolve`] would then return `None` for its parameter
/// — breaking the invariant that a surviving element's references all resolve.
/// So this runs to a fixed point: each round drops the declarations it refused,
/// and stops when a round refuses nothing. Chains of any length collapse, in
/// either declaration order, because the set only ever shrinks.
fn resolve_references(elements: &mut [Element]) {
    let mut declared: std::collections::HashSet<String> = elements
        .iter()
        .filter_map(|e| match e {
            Element::Type(t) => Some(t.name().to_string()),
            _ => None,
        })
        .collect();

    loop {
        let mut refused = Vec::new();
        for (i, element) in elements.iter().enumerate() {
            if let Some(unresolved) = first_unresolved(element, &declared) {
                refused.push((i, unresolved));
            }
        }
        if refused.is_empty() {
            return;
        }
        for (i, unresolved) in refused {
            // A refused type stops being a declaration, which is what lets the
            // next round see its dependents as unresolved.
            if let Element::Type(t) = &elements[i] {
                declared.remove(&t.name().to_string());
            }
            let element = &mut elements[i];
            let name = element.name().cloned();
            let origin = Origin::new(
                element.syntax(),
                Rc::clone(match element {
                    Element::Function(f) => &f.origin.location,
                    Element::Type(t) => t.location_rc(),
                    Element::Constant(c) => &c.origin.location,
                    Element::Guard(g) => &g.origin.location,
                    Element::Unsupported(u) => &u.origin.location,
                }),
            );
            *element = Element::Unsupported(Unsupported {
                name,
                error: Box::new(ItemError::UnresolvedType { name: unresolved }),
                origin,
            });
        }
    }
}

/// Every type slot this element writes, outermost only — a parameter, a return,
/// a field, a constant's type.
///
/// The one place the slots are enumerated, so a new element shape is taught to
/// every consumer at once instead of drifting between them.
fn element_type_refs(element: &Element) -> Vec<&TypeRef> {
    let mut refs: Vec<&TypeRef> = Vec::new();
    match element {
        Element::Function(f) => {
            refs.extend(f.params.iter().map(|p| &p.ty));
            refs.push(&f.ret);
        }
        Element::Constant(c) => refs.push(&c.ty),
        Element::Type(Type::Struct(s)) => refs.extend(s.fields.iter().map(|f| &f.ty)),
        Element::Type(Type::Variant(v)) => refs.extend(
            v.alternatives
                .iter()
                .flat_map(|a| a.fields.iter().map(|f| &f.ty)),
        ),
        // An enum names nothing, an extern hides what it names, a guard is
        // emitted verbatim so its types are the consumer's business, and an
        // unsupported item already has a diagnosis worth keeping.
        Element::Type(Type::Enum(_) | Type::Extern(_))
        | Element::Guard(_)
        | Element::Unsupported(_) => {}
    }
    refs
}

/// The first type this element names that the flat API does not declare.
fn first_unresolved(
    element: &Element,
    declared: &std::collections::HashSet<String>,
) -> Option<String> {
    element_type_refs(element)
        .into_iter()
        .find_map(|r| r.first_unresolved(declared))
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
    /// The crate each was marked in. A captured file path is crate-relative
    /// (both are `src/lib.rs`), so these are the only unambiguous coordinates
    /// when two sources collide.
    pub first_crate: Option<String>,
    pub second_crate: Option<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::DuplicateName(d) => {
                let at = |loc: &SourceLocation, krate: &Option<String>| match krate {
                    Some(k) => format!("{loc} (crate `{k}`)"),
                    None => loc.to_string(),
                };
                write!(
                    f,
                    "duplicate `#[prebindgen]` name `{}`: first at {}, again at {} — marked items \
                     share one flat namespace across all source crates",
                    d.name,
                    at(&d.first, &d.first_crate),
                    at(&d.second, &d.second_crate)
                )
            }
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
    /// The item names a type the flat API does not declare.
    ///
    /// The flat API is closed over its own names: a handle enters it through
    /// `#[prebindgen] pub type X = ..`, so a name with no declaration is either a
    /// missing marker or a typo. Reporting it here replaces discovering it much
    /// later as an unresolved converter, from whichever adapter looked first.
    UnresolvedType { name: String },
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
            ItemError::UnresolvedType { name } if name.contains("::") => write!(
                f,
                "names the type `{name}`, which the flat API does not declare \u{2014} and being \
                 path-qualified it never could, because marked items live in one flat namespace \
                 of bare names. Give the type a name here with `#[prebindgen] pub type <Name> = \
                 {name};` and refer to that"
            ),
            ItemError::UnresolvedType { name } => write!(
                f,
                "names the type `{name}`, which the flat API does not declare \u{2014} mark its \
                 declaration `#[prebindgen]`, or, for a foreign or crate-private type used as a \
                 handle, give it a name here with `#[prebindgen] pub type {name} = ..;`"
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
            Ok(ty) => Element::Type(ty),
            Err(error) => unsupported(s.ident.clone(), syn::Item::Struct(s), &at, error),
        },
        syn::Item::Enum(e) => match lower_enum(&e, &at, consts) {
            Ok(ty) => Element::Type(ty),
            Err(error) => unsupported(e.ident.clone(), syn::Item::Enum(e), &at, error),
        },
        // `#[prebindgen] pub type X = path;` DECLARES an opaque type: it gives a
        // foreign or crate-private type a name in the flat API, without claiming
        // anything about its contents. That is the only way a handle enters the
        // API deliberately, and the reason references can be required to resolve.
        syn::Item::Type(t) => match reject_generic_params(&t.generics) {
            // `Extern` has no binder and no arity, so a generic alias would be
            // accepted as one declaration that `Handle<u8>` then resolves against
            // — losing exactly the scoped-parameter distinction every other item
            // kind refuses. It is also why `MaybeUninit` needed grammar support
            // rather than an alias.
            Err(error) => unsupported(t.ident.clone(), syn::Item::Type(t), &at, error),
            Ok(()) => {
                let target = Some(t.ty.to_token_stream().to_string());
                Element::Type(Type::Extern(Extern {
                    name: t.ident.clone(),
                    target,
                    origin: Origin::new(syn::Item::Type(t), at),
                }))
            }
        },
        // An unnamed const is a `Guard`, not a constant: nothing can name it, so
        // it is not part of the API, and several sources' guards coexist because
        // none of them has an address to collide on.
        syn::Item::Const(c) if c.ident == "_" => Element::Guard(Guard {
            origin: Origin::new(c, at),
        }),
        syn::Item::Const(c) => match lower_type(&c.ty, consts, &at) {
            Ok(ty) => Element::Constant(Constant {
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
        // An item kind the language does not model. The proc-macro accepts only
        // six kinds and the five above cover the rest, so in practice this is a
        // `union` — never written by any source crate. It is diagnosed rather
        // than carried: a `#[prebindgen]` crate marks what crosses the boundary,
        // and the code around that belongs to the consumer.
        other => {
            let (name, kind) = match &other {
                syn::Item::Union(u) => (Some(u.ident.clone()), "a union"),
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
        syn::ReturnType::Default => TypeRef {
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

/// Lower a `struct` item to whichever of the two shapes it is.
///
/// A **tuple struct** is an [`Extern`]: no adapter has ever crossed its fields,
/// so they are deliberately not lowered and a field type outside the grammar is
/// not an error. Anything else is a product of fields that do cross.
fn lower_struct(
    s: &syn::ItemStruct,
    at: &Rc<SourceLocation>,
    consts: &ConstIndex,
) -> Result<Type, ItemError> {
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
            out
        }
        // Its contents are not a boundary surface, so nothing is lowered.
        syn::Fields::Unnamed(_) => {
            return Ok(Type::Extern(Extern {
                name: s.ident.clone(),
                // A tuple struct IS the definition; it points at nothing.
                target: None,
                origin: Origin::new(syn::Item::Struct(s.clone()), Rc::clone(at)),
            }));
        }
        syn::Fields::Unit => Vec::new(),
    };
    Ok(Type::Struct(Struct {
        name: s.ident.clone(),
        fields,
        origin: Origin::new(s.clone(), Rc::clone(at)),
    }))
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
) -> Result<Type, ItemError> {
    reject_generic_params(&e.generics)?;

    if e.variants.iter().any(|v| !v.fields.is_empty()) {
        return Ok(Type::Variant(lower_variant(e, at, consts)?));
    }
    Ok(Type::Enum(lower_c_enum(e, at)))
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
