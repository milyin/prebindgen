//! Types: a closed classification paired with the syntax it was read from.
//!
//! [`TypeRef`] is the pattern the whole element model follows — `kind` says what
//! the type *means*, `syntax` is the tokens the source wrote. Consumers
//! **classify off `kind` and spell off `syntax`**; see the [module docs](super)
//! for why that split is the point.
//!
//! [`TypeKind`] is total over the accepted grammar: a form with no variant here
//! is a form the language does not accept, so acceptance is a consequence of
//! lowering rather than a second list that can drift from it. Same contract, and
//! for the same reason, as [`lower_array_len`].

use std::{fmt, rc::Rc};

use quote::ToTokens;

use super::{
    array_len::{lower_array_len, ArrayExtent, ConstIndex, UnsupportedArrayLen},
    origin::Origin,
};
use crate::SourceLocation;

/// A type as the language decided it, plus the exact syntax it came from.
///
/// The [`Origin::syntax`] slice is what removes the pressure to make `kind`
/// lossless: a lifetime, an elided argument, a `Box` that changes nothing
/// outside Rust all survive there at zero modelling cost, so `kind` can stay
/// language-neutral and small.
///
/// # The invariant
///
/// > **Every `TypeRef` was classified by the model.** [`Flat`](super::Flat)
/// > classified it from source syntax, or `api::core` composed it by layering
/// > over something already classified. Nothing above `api::core` can mint one.
///
/// **Scoped to what is enforced, not to what is intended.** The boundary is
/// `api::core`, and it is drawn by visibility at four places, each checked by
/// the compiler on every build:
///
/// | | |
/// |---|---|
/// | the `kind` and `origin` fields | `pub(super)` — a public field **is** a constructor, so restricting only the composers would block nothing |
/// | `borrowed` / `optional` / `scalar` | `pub(in crate::api::core)` |
/// | `named` | `pub(super)` — `flat` alone |
/// | `Flat::classify` | `pub(in crate::api::core)` |
///
/// So a language adapter under `api::lang` cannot mint a reading at all — not
/// by field, not by composer, not by classifying a spelling of its own. Where
/// one needs a type the model already declares, the **declaration** answers:
/// see [`Variant::type_ref`](super::Variant::type_ref), which is what the
/// `SumTag` selector uses instead of composing a reading from an ident.
///
/// The invariant is unconditional — no phase, no lifetime, no direction — so it
/// holds for a **stored** value. That is the point: a `TypeRef` lives in
/// `UnfoldLeaf::out_ty` and `FoldLeaf::ty`, inside plans the registry itself
/// stores, so a borrow-carrying token would make the registry self-referential.
///
/// It deliberately does **not** claim the type's converters exist. That is
/// false by design for stored readings — `unrequire_output` leaves a cell whose
/// converter genuinely cannot resolve, and a `SumTag` leaf never has one — so
/// converter existence stays a lookup that answers `Option`.
#[derive(Clone, Debug)]
pub struct TypeRef {
    /// What the type means — the closed, destination-neutral classification.
    pub(super) kind: TypeKind,
    /// The type as generated Rust must spell it — the source's own tokens,
    /// normalized to the flat namespace the generated crate can name (see
    /// [`Flat::parse`](super::Flat::parse)) — plus the source they came
    /// from.
    ///
    /// The syntax can say strictly more than `kind` does — `Box<String>` is a
    /// `Str` here — which is the point: what Rust needs and no destination
    /// language can see lives in the tokens, not in the classification.
    pub(super) origin: Origin<syn::Type>,
}

impl TypeRef {
    /// What the type means. **Classify off this**, never off the spelling.
    ///
    /// The seal, as a compiled assertion. An out-of-crate consumer cannot
    /// assemble a reading, because the fields it would have to name are private
    /// (`E0451`):
    ///
    /// ```compile_fail
    /// # use prebindgen::core::flat::{TypeKind, TypeRef};
    /// let forged = TypeRef { kind: TypeKind::Unit, origin: todo!() };
    /// ```
    ///
    /// …nor through a composer, which is not visible either (`E0624`):
    ///
    /// ```compile_fail
    /// # use prebindgen::core::flat::{ScalarKind, TypeRef};
    /// let forged = TypeRef::scalar(ScalarKind::Bool);
    /// ```
    ///
    /// These two prove only the **crate** boundary. The stronger claim — that
    /// nothing above `api::core` can mint one either — is enforced by the
    /// visibilities tabulated on [`TypeRef`], and a doctest cannot reach inside
    /// the crate to test it. The compiler checks it on every build instead: an
    /// `api::lang` adapter naming any of the four routes fails with `E0624`.
    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    /// The tokens generated Rust must spell. **Spell off this**, never off
    /// `kind` — the syntax says strictly more, and re-deriving it from the
    /// classification is how `Box<Option<T>>` becomes an `E0308`.
    pub fn syntax(&self) -> &syn::Type {
        &self.origin.syntax
    }

    /// Where the type was written, for diagnostics. A composed type is
    /// **placeless** — [`SourceLocation::has_position`] gates what is printed.
    pub fn location(&self) -> &SourceLocation {
        &self.origin.location
    }

    /// An [`Origin`] for a node that exists **because of** this type, sharing
    /// its location — a synthesized getter built from a return type, say.
    ///
    /// Deliberately narrower than handing out the origin: it lends provenance
    /// without lending the field, so a `TypeRef`'s own `Origin` still cannot be
    /// obtained from outside the model.
    pub(crate) fn origin_with<S>(&self, syntax: S) -> Origin<S> {
        self.origin.with(syntax)
    }
}

impl TypeRef {
    /// The **arity layers** over this type, and what they wrap.
    ///
    /// `Option<Vec<T>>` is `Optional(Iterable(Base))` over `T`. The stack is the
    /// same [`Shape`](crate::core::shape::Shape) the expansion and decomposition plans are built from — so a
    /// consumer that needs a plan shape has it, rather than rebuilding one from
    /// flags that were derived from this type moments earlier.
    ///
    /// **A borrow is not a layer.** `Optional` and `Iterable` change arity — none
    /// or one, none or many — while `&T` is the same single value held
    /// differently. That is ownership, and it stays on the returned core, where
    /// [`borrow_target`](Self::borrow_target) reads it.
    ///
    /// A layer out of position is not a layer: `Vec<Option<T>>` is
    /// `Iterable(Base)` over `Option<T>`, because the optional is inside the run.
    /// The stack is what wraps the payload, in order, and nothing is reordered to
    /// make it fit a shape a caller hoped for.
    ///
    /// Returning the stack rather than a set of flags is what lets a caller
    /// **decline**: a consumer that can only build `Base` and `Optional(Base)`
    /// matches those and falls through on anything else, instead of silently
    /// consuming a layer it cannot honour.
    pub fn layer_stack(&self) -> (crate::api::core::shape::Shape, &TypeRef) {
        use crate::api::core::shape::Shape;
        // Bounded on purpose, and not a recursion: the accepted crossing is
        // `Option<Vec<T>>` — at most one optional, then at most one run, in that
        // order. Recursing would accept `Vec<Option<T>>` as `Iterable(Optional)`,
        // which reads the inner optional as a boundary layer when it is part of
        // the element, and `Option<Option<T>>` as two nullable layers when the
        // boundary has one way to say absent.
        let mut core = self;
        let optional = matches!(core.kind, TypeKind::Optional(_));
        if let TypeKind::Optional(inner) = &core.kind {
            core = inner;
        }
        let iterable = matches!(core.kind, TypeKind::Sequence(_));
        if let TypeKind::Sequence(inner) = &core.kind {
            core = inner;
        }

        let mut shape = Shape::Base;
        if iterable {
            shape = Shape::iterable(shape);
        }
        if optional {
            shape = Shape::optional((), shape);
        }
        (shape, core)
    }

    /// Every type on the way down through the arity layers, outermost first and
    /// ending at the core [`layer_stack`](Self::layer_stack) returns.
    ///
    /// What a **registration** walks, which is a different question from what
    /// crosses: a value delivered layer-by-layer needs each of these un-required,
    /// and none of them has a converter of its own.
    pub fn layer_types(&self) -> Vec<&TypeRef> {
        // Stops exactly where `layer_stack` stops, or the registration view would
        // un-require types the shape says are part of the element.
        let mut out = vec![self];
        let mut cur = self;
        if let TypeKind::Optional(inner) = &cur.kind {
            out.push(inner);
            cur = inner;
        }
        if let TypeKind::Sequence(inner) = &cur.kind {
            out.push(inner);
        }
        out
    }

    /// What an `Option<T>` wraps, else `None`.
    ///
    /// One layer, named. [`layer_stack`](Self::layer_stack) reads the whole
    /// arity stack; these three read exactly the layer a caller asks for, which
    /// is what a consumer wants when it can only *represent* some of them.
    pub fn optional_inner(&self) -> Option<&TypeRef> {
        match &self.kind {
            TypeKind::Optional(inner) => Some(inner),
            _ => None,
        }
    }

    /// The element of a run of values (`Vec<T>`, `[T]`), else `None`.
    pub fn sequence_elem(&self) -> Option<&TypeRef> {
        match &self.kind {
            TypeKind::Sequence(elem) => Some(elem),
            _ => None,
        }
    }

    /// What a borrow points at, else `None`.
    pub fn borrow_target(&self) -> Option<&TypeRef> {
        match &self.kind {
            TypeKind::Ref { inner, .. } => Some(inner),
            _ => None,
        }
    }

    // ── Composition ───────────────────────────────────────────────
    //
    // Building a type the SOURCE did not write, as opposed to reading one it
    // did. The decomposition plans need this: a leaf may be the borrow of a
    // value, a presence flag, or a selector — none of which any source spelled,
    // and all of which have to carry a reading like everything else.
    //
    // Here rather than at the callers, and not via
    // [`Flat::classify`](super::Flat::classify), because the two acts are
    // different. `classify` lowers *source syntax* — it is the frontend reading
    // what a crate wrote, and `classify_has_no_caller_outside_the_registry`
    // keeps it that way. These compose a type from parts already understood,
    // which needs no lowering at all: each builds `kind` **and** the matching
    // `origin.syntax` in one place, so the classification and the spelling
    // cannot disagree — the invariant every consumer of a `TypeRef` relies on.

    /// A borrow of this type — `&T` from `T`.
    ///
    /// Keeps this type's location: the borrow exists *because of* this value,
    /// so a diagnostic about it should point where the value came from.
    pub(in crate::api::core) fn borrowed(&self) -> TypeRef {
        let inner = &self.origin.syntax;
        TypeRef {
            kind: TypeKind::Ref {
                mode: RefMode::Shared,
                inner: Box::new(self.clone()),
            },
            origin: self.origin.with(syn::parse_quote!(&#inner)),
        }
    }

    /// An optional of this type — `Option<T>` from `T`. Location as
    /// [`Self::borrowed`].
    pub(in crate::api::core) fn optional(&self) -> TypeRef {
        let inner = &self.origin.syntax;
        TypeRef {
            kind: TypeKind::Optional(Box::new(self.clone())),
            origin: self.origin.with(syn::parse_quote!(Option<#inner>)),
        }
    }

    /// A scalar the binding invented — a presence flag, a selector.
    ///
    /// **Placeless**, and deliberately: no file wrote it, so claiming a location
    /// would make a fabricated one indistinguishable from a real one.
    /// [`Flat::classify`](super::Flat::classify) does exactly this for a
    /// composed spelling, and `ensure_entry` gives adapter-authored cells the
    /// same treatment — `has_position` already gates what a diagnostic prints.
    pub(in crate::api::core) fn scalar(kind: ScalarKind) -> TypeRef {
        // The spelling comes from the kind, so the two cannot drift.
        let ident = syn::Ident::new(kind.as_str(), proc_macro2::Span::call_site());
        TypeRef {
            kind: TypeKind::Scalar(kind),
            origin: Origin::new(
                syn::parse_quote!(#ident),
                std::rc::Rc::new(crate::SourceLocation::default()),
            ),
        }
    }

    /// A nominal reference to a declared type, by name. Placeless for the same
    /// reason as [`Self::scalar`] — this is the binding naming a type, not a
    /// source mentioning one.
    pub(super) fn named(ident: &syn::Ident) -> TypeRef {
        TypeRef {
            kind: TypeKind::Named {
                id: TypeId {
                    name: ident.to_string(),
                },
            },
            origin: Origin::new(
                syn::parse_quote!(#ident),
                std::rc::Rc::new(crate::SourceLocation::default()),
            ),
        }
    }

    /// This type's identity as a table key.
    ///
    /// The canonical spelling is what a key *is* (#113), and reading it is
    /// legitimate — but it should be the model's answer rather than every caller
    /// reaching into [`syntax`](Self::syntax) for it, since a caller that reaches
    /// into `origin` to *reason* is the thing this model exists to stop.
    pub fn key(&self) -> crate::api::core::registry::TypeKey {
        crate::api::core::registry::TypeKey::from_type(&self.origin.syntax)
    }

    /// The [transparent wrapper](TRANSPARENT_WRAPPERS) this type's **spelling**
    /// adds over its classification, if any — `Box<Option<T>>` → `Some("Box")`,
    /// `Option<T>` → `None`.
    ///
    /// This exists because [`kind`](Self::kind) and [`syntax`](Self::syntax)
    /// answer different questions, and only one of them is about the
    /// destination:
    ///
    /// * `kind` decides what the **destination** sees — the surface type and the
    ///   wire. `Box<Option<String>>` and `Option<String>` are one optional
    ///   string to every destination language, which is why the wrapper is
    ///   erased.
    /// * `syntax` decides how the value is **converted** — and Rust does tell
    ///   them apart. A converter that rebuilds a value must produce the type
    ///   the source actually spelled.
    ///
    /// So a consumer that *classifies* should never consult this; a consumer
    /// that **reconstructs a Rust value** must, because rebuilding from the
    /// classification alone yields the stripped type and handing that to a
    /// parameter spelled `Box<..>` is an `E0308` in the generated crate.
    ///
    /// Only the outermost wrapper is named. That is enough to decide *whether*
    /// a spelling was erased — which is the question a **refusal** asks — but a
    /// consumer that rebuilds a nested `Box<Cow<'_, T>>` needs every layer, and
    /// asks [`erased_wrappers`](Self::erased_wrappers) for the whole list plus
    /// [`stripped_syntax`](Self::stripped_syntax) for what sits under it.
    ///
    /// Erased says nothing about **rebuildable**: `Box` reconstructs as
    /// `Box::new(v)`, while `Cow`'s `Owned`/`Borrowed` choice is not determined
    /// by any fact the model holds. Which wrappers an emitter can rebuild is
    /// that emitter's policy; this only stops the wrapper from being invisible.
    pub fn erased_wrapper(&self) -> Option<&'static str> {
        self.erased_wrappers().into_iter().next()
    }

    /// Every [transparent wrapper](TRANSPARENT_WRAPPERS) this type's **spelling**
    /// adds over its classification, outermost first — `Box<Box<T>>` →
    /// `["Box", "Box"]`, `Box<Cow<'_, T>>` → `["Box", "Cow"]`, an unwrapped
    /// spelling → `[]`.
    ///
    /// The list [`erased_wrapper`](Self::erased_wrapper) names the head of. A
    /// consumer deciding *whether* to refuse needs only that head; one that
    /// **rebuilds** needs all of them, because it has to apply an operation per
    /// layer — and `Box<Cow<'_, T>>` is two different operations, not one
    /// repeated.
    ///
    /// # This answers for one layer's spelling
    ///
    /// **An erasure sits outside the layer it wraps**, so this is a question
    /// that has to be asked on the way *down*, at every layer, and never once at
    /// the top:
    ///
    /// | Spelling | here | on [`borrow_target`](Self::borrow_target) |
    /// |---|---|---|
    /// | `Box<&Vec<T>>` | `["Box"]` | `[]` — `kind` is `Ref`, and peeling it first drops the `Box` |
    /// | `&Box<Vec<T>>` | `[]` — a `syn::Type::Reference` cannot be peeled | `["Box"]` |
    ///
    /// A rebuild therefore collects wrappers **as it descends**: by the time it
    /// reaches the leaf they are gone from `kind`, which is precisely the thing
    /// they are missing from.
    pub fn erased_wrappers(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        let mut ty = std::borrow::Cow::Borrowed(&self.origin.syntax);
        while let Some((name, inner)) = peel_transparent(&ty) {
            names.push(name);
            ty = std::borrow::Cow::Owned(inner);
        }
        names
    }

    /// This type's spelling with every [transparent
    /// wrapper](TRANSPARENT_WRAPPERS) removed — `Box<Box<Option<T>>>` →
    /// `Option<T>`, an unwrapped spelling → itself.
    ///
    /// The spelling a reconstruction builds *before* it puts the wrappers back:
    /// rebuilding from [`kind`](Self::kind) alone yields this, so an emitter
    /// that hands it to a parameter spelled `Box<..>` writes an `E0308`. Paired
    /// with [`erased_wrappers`](Self::erased_wrappers), which says exactly what
    /// has to go back on.
    ///
    /// **The invariant, which is what defines this rather than the loop that
    /// computes it**: it is the spelling whose own lowering yields exactly this
    /// type's `kind`. So the peel runs to a **fixed point** — `Box<Box<T>>`
    /// classifies as `T`, and stripping one layer leaves a `Box<T>` that does
    /// not match.
    ///
    /// Per-layer, for the reason [`erased_wrappers`](Self::erased_wrappers)
    /// tabulates: this strips what stands over *this* node's classification, and
    /// a wrapper under a borrow or inside an `Option` belongs to that inner
    /// node's own spelling.
    pub fn stripped_syntax(&self) -> syn::Type {
        let mut ty = self.origin.syntax.clone();
        while let Some((_, inner)) = peel_transparent(&ty) {
            ty = inner;
        }
        ty
    }

    /// The `Ok` and `Err` sides when this is a `Result`, else `None`.
    pub fn fallible_parts(&self) -> Option<(&TypeRef, &TypeRef)> {
        match &self.kind {
            TypeKind::Fallible { ok, err } => Some((ok, err)),
            _ => None,
        }
    }

    /// The argument types when this is a callback, else `None` — the reading
    /// counterpart of
    /// [`extract_fn_trait_args`](super::extract_fn_trait_args).
    ///
    /// The two are the same question asked of different things, and that is the
    /// whole difference. `extract_fn_trait_args` takes an
    /// `impl Fn(..) + Send + Sync + 'static` **apart**: it walks the bounds,
    /// checks the three markers, and refuses a written return type — it is a
    /// classifier, and the one the model itself runs to build
    /// [`TypeKind::Callback`]. This reads the result of that classification,
    /// already made. A consumer holding a reading has no reason to redo the
    /// walk, and every reason not to: a `Vec<syn::Type>` of *arguments* has lost
    /// which of them the model accepted and how, while each `TypeRef` here
    /// carries its own classification and its own spelling.
    ///
    /// Consequently this answers `None` for a type that merely *looks* like a
    /// callback but was refused (a missing `Send`, an `impl Fn() -> u8`): the
    /// acceptance already happened, and asking again is how the two drift.
    pub fn callback_args(&self) -> Option<&[TypeRef]> {
        match &self.kind {
            TypeKind::Callback { args } => Some(args),
            _ => None,
        }
    }

    /// The extent of this type when it is an array, else `None`.
    pub fn array_extent(&self) -> Option<&ArrayExtent> {
        match &self.kind {
            TypeKind::Array { extent, .. } => Some(extent),
            _ => None,
        }
    }

    /// Every extent reachable from this type, outermost first — so a nested
    /// `[[u8; A]; B]` yields `B` then `A`.
    ///
    /// Used to find which consts an emitted C type may name, and therefore which
    /// must reach the header as a `#define`.
    pub fn extents(&self) -> Vec<&ArrayExtent> {
        let mut out = Vec::new();
        self.collect_extents(&mut out);
        out
    }

    /// The first nominal type reachable from here that `declared` does not hold.
    ///
    /// Recurses the same structure [`Self::collect_extents`] walks: what is
    /// reachable is what a destination language will have to convert, so every
    /// layer's inner reference counts.
    pub(super) fn first_unresolved(
        &self,
        declared: &std::collections::HashSet<String>,
    ) -> Option<String> {
        match &self.kind {
            TypeKind::Named { id } => (!declared.contains(&id.name)).then(|| id.name.clone()),
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.first_unresolved(declared)
            }
            TypeKind::Array { elem, .. } => elem.first_unresolved(declared),
            TypeKind::Fallible { ok, err } => ok
                .first_unresolved(declared)
                .or_else(|| err.first_unresolved(declared)),
            TypeKind::Callback { args } => args.iter().find_map(|a| a.first_unresolved(declared)),
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => None,
        }
    }

    /// This type and every type reachable inside it, outermost first.
    ///
    /// The nested positions are real [`TypeRef`]s carrying their own spelling and
    /// origin, so a consumer that indexes types finds `Foo` from `Vec<Foo>` with
    /// the classification already made rather than a sub-path to re-read.
    ///
    /// A [`Named`](TypeKind::Named)'s generic arguments are **not** among them:
    /// [`TypeId`] keeps a name and nothing else, so `MyBox<Foo>` reaches no `Foo`
    /// here. The full spelling is in [`Self::syntax`] for whoever needs it.
    pub fn walk(&self) -> Vec<&TypeRef> {
        let mut out = Vec::new();
        self.collect_refs(&mut out);
        out
    }

    fn collect_refs<'a>(&'a self, out: &mut Vec<&'a TypeRef>) {
        out.push(self);
        match &self.kind {
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.collect_refs(out)
            }
            TypeKind::Array { elem, .. } => elem.collect_refs(out),
            TypeKind::Fallible { ok, err } => {
                ok.collect_refs(out);
                err.collect_refs(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_refs(out)),
            TypeKind::Named { .. } | TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => {}
        }
    }

    fn collect_extents<'a>(&'a self, out: &mut Vec<&'a ArrayExtent>) {
        match &self.kind {
            TypeKind::Array { elem, extent } => {
                out.push(extent);
                elem.collect_extents(out);
            }
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.collect_extents(out)
            }
            TypeKind::Fallible { ok, err } => {
                ok.collect_extents(out);
                err.collect_extents(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_extents(out)),
            TypeKind::Named { .. } => {}
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => {}
        }
    }
}

/// What a [`TypeRef`] means. The variants are the accepted type grammar.
///
/// One Rust spelling per concept is **not** the rule here — several are. A
/// concept earns a variant when a destination language would act on it; a
/// spelling that changes nothing outside Rust folds into the concept it carries
/// and survives in [`TypeRef::syntax`]:
///
/// | Spelling | Kind | Why |
/// |---|---|---|
/// | `String`, `str` | [`Str`](TypeKind::Str) | one concept, two Rust types |
/// | `Vec<T>`, `[T]` | [`Sequence`](TypeKind::Sequence) | a run of `T`; owned vs borrowed is the [`Ref`](TypeKind::Ref) layer's fact, not a second variant |
/// | `Box<T>` | *whatever `T` is* | an owned `T` either way; nothing outside Rust can tell |
#[derive(Clone, Debug)]
pub enum TypeKind {
    /// A primitive with a fixed C/JVM counterpart.
    Scalar(ScalarKind),
    /// A UTF-8 string — `String` owned, `str` behind a [`Ref`](TypeKind::Ref).
    ///
    /// Both spellings are one concept: `&str` and `&String` are each a borrowed
    /// string and classify identically, which is what every adapter already
    /// does by hand.
    Str,
    /// `Option<T>`.
    Optional(Box<TypeRef>),
    /// A run of `T` — `Vec<T>` owned, `[T]` behind a [`Ref`](TypeKind::Ref).
    ///
    /// One variant, because ownership is already the [`Ref`](TypeKind::Ref)
    /// layer's fact: `&[T]` is `Ref(Sequence)`, `Vec<T>` is `Sequence`. A
    /// second variant would encode ownership twice and let the two copies
    /// disagree. `[T; N]` is *not* this — a fixed extent is a different
    /// concept, see [`Array`](TypeKind::Array).
    Sequence(Box<TypeRef>),
    /// `Result<T, E>`.
    Fallible { ok: Box<TypeRef>, err: Box<TypeRef> },
    /// Any other named type: a `#[prebindgen]` struct or enum, or a foreign
    /// path.
    ///
    /// `id` is the type's **identity** — a name, not syntax, so nothing outside
    /// this module has to take a path apart to learn what a type is. The last
    /// segment's generic arguments live in `args`, and only the *type*
    /// arguments: a lifetime argument says nothing a destination language can
    /// act on. The full spelling is in [`TypeRef::syntax`] for whoever re-emits it.
    Named { id: TypeId },
    /// `[T; N]` — a run of `T` whose length is known at compile time.
    ///
    /// Deliberately not a [`Sequence`](TypeKind::Sequence) with an optional
    /// extent: a fixed array crosses by value as a primitive array, a `Vec`
    /// crosses as a heap collection, and every adapter branches between the two
    /// at every site.
    Array {
        elem: Box<TypeRef>,
        /// Boxed: an extent carries an [`Origin`] over the length expression, which
        /// makes it the size outlier among the kinds, and an array is the rare one.
        /// The same trade-off [`Unsupported::error`](super::Unsupported) makes.
        extent: Box<ArrayExtent>,
    },
    /// A borrow — `&T`, `&mut T`, or `&mut MaybeUninit<T>`. The lifetime is
    /// spelling, so it lives in [`TypeRef::syntax`] rather than here.
    ///
    /// This is the ownership layer for every concept underneath it: `&str` is
    /// `Ref(Str)`, `&[T]` is `Ref(Sequence)`. A shared-ownership handle
    /// (`Arc<T>`, `Rc<T>`) belongs here too when the language accepts one.
    ///
    /// `inner` is always the borrowed *value's* type, so an out-parameter's
    /// `MaybeUninit` is absorbed into [`RefMode::Out`] rather than wrapping it:
    /// uninitialized-ness is a property of the **borrow**, not of the type, and
    /// it is meaningless anywhere else.
    Ref { mode: RefMode, inner: Box<TypeRef> },
    /// `impl Fn(A, B, …) + Send + Sync + 'static` — the callback form.
    Callback { args: Vec<TypeRef> },
    /// `()`.
    Unit,
}

/// What a borrow permits, and what the callee owes.
///
/// One axis with three values rather than a `mutable` flag plus a wrapper, so the
/// combinations that mean nothing at a boundary — a shared borrow of
/// uninitialized storage, an owned `MaybeUninit` — cannot be written down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefMode {
    /// `&T` — the callee may read it and must not write it.
    Shared,
    /// `&mut T` — the callee may read and write an already-valid `T`.
    Exclusive,
    /// `&mut MaybeUninit<T>` — an **out-parameter**: the caller supplies storage
    /// only, and the callee's job is to make it a valid `T`. The callee may not
    /// read it first.
    ///
    /// This is a boundary concept every destination language has (C's `T *out`),
    /// which is why it is modelled here rather than left as a nominal
    /// `MaybeUninit` for each adapter to recognise.
    Out,
}

/// A nominal type's identity: a name, and nothing else.
///
/// `#[prebindgen]` names live in one flat namespace — a duplicate is a
/// [`ParseError`](super::ParseError) — so the name is the whole address. It
/// deliberately carries **no crate**: a reference carries a name, and the
/// declaring crate belongs to the declaration, reachable by looking the name up
/// among the elements. Putting the use site's crate here would make the same
/// type compare unequal to itself across two source crates.
///
/// A name rather than a `syn::Path` on purpose: an identity kept as syntax
/// makes every consumer take a path apart to learn what a type is, which is the
/// re-classification issue #211 exists to stop — and one the boundary ledger
/// would not even see, since it watches `syn::Type` and `syn::Expr`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeId {
    /// The path as written, minus any generic arguments — `Foo`,
    /// `foreign::Option`. Normalized, so a reducible std or source-module path
    /// has already collapsed to its final segment.
    ///
    /// A `String`, so a **raw** identifier is stored the way `Ident` prints
    /// it — `r#type`, hashes and all. Recover it with [`Self::ident`] rather
    /// than `Ident::new`, which rejects that spelling.
    pub name: String,
}

impl TypeId {
    /// This name as an identifier, **raw forms included**.
    ///
    /// `Ident::new("r#type", …)` *panics* — it takes a bare name, not a
    /// spelling — so a consumer rebuilding an ident from [`Self::name`] has to
    /// parse rather than construct. Here so that recovery is written once: the
    /// caller that gets it wrong does not fail until a source happens to use a
    /// keyword, which is exactly the kind of bug that ships.
    ///
    /// `None` for a name that is not a single identifier at all (a
    /// path-qualified `foreign::Option`), which is the same answer
    /// `bare_path_ident` gave for one.
    pub fn ident(&self) -> Option<syn::Ident> {
        syn::parse_str::<syn::Ident>(&self.name).ok()
    }
}

/// The primitives the source language accepts. Mirrors the set every adapter
/// already treats as directly representable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarKind {
    Bool,
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
}

impl ScalarKind {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "bool" => Self::Bool,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "isize" => Self::Isize,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "usize" => Self::Usize,
            "f32" => Self::F32,
            "f64" => Self::F64,
            _ => return None,
        })
    }

    /// The Rust spelling — the identity this was lowered from.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Isize => "isize",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

/// A type the prebindgen source language does not accept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedType {
    /// The offending type as written.
    pub offending: String,
    pub reason: UnsupportedTypeReason,
}

/// Why [`lower_type`] refused a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsupportedTypeReason {
    /// A syntactic form with no place in the language: a raw pointer, a bare
    /// trait object, a closure type, a macro, `Self`, a never type, an inferred
    /// type.
    ///
    /// A `#[prebindgen]` crate is idiomatic Rust — the adapter owns the lowering
    /// to pointers — so `*const T` / `*mut T` are refused here rather than
    /// modelled. No adapter has a selection arm for one, so accepting them would
    /// only defer the failure to a late "unresolved type".
    UnsupportedForm,
    /// `impl Trait` that is not the accepted callback form — anything but
    /// `impl Fn(..) + Send + Sync + 'static` returning `()`.
    DisallowedImplTrait,
    /// A generic that takes a fixed arity and did not get it — `Option` with no
    /// argument, `Result` with one.
    WrongGenericArity { expected: usize },
    /// A non-empty tuple. Only `()` is in the language: no adapter has ever
    /// lowered a tuple, so accepting one would defer the failure to a late
    /// "unresolved type" instead of naming it here.
    UnsupportedTuple,
    /// `MaybeUninit<T>` somewhere other than behind a `&mut`.
    ///
    /// Uninitialized storage is a property of a *borrow*, not of a type — see
    /// [`RefMode::Out`]. Owned, returned or stored in a field it promises nothing
    /// a destination language can use, and reading it would be undefined.
    OwnedUninit,
    /// `&MaybeUninit<T>` — a shared borrow of uninitialized storage.
    ///
    /// A shared borrow promises a readable `T`, and this supplies storage that may
    /// not be one. Only `&mut MaybeUninit<T>` means anything: see
    /// [`RefMode::Out`].
    SharedUninit,
    /// A path with a qualified self — `<T as Trait>::Assoc`.
    ///
    /// The frontend never captures `impl` blocks, so it cannot know what an
    /// associated type resolves to; carrying the spelling would only move the
    /// failure downstream.
    AssociatedType,
    /// A generic argument that is neither a type nor a lifetime — a const
    /// generic, an associated-type binding.
    UnsupportedGenericArgument,
    /// The array's extent — see [`ArrayLenReason`](super::ArrayLenReason).
    BadArrayExtent(Box<UnsupportedArrayLen>),
}

impl fmt::Display for UnsupportedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            UnsupportedTypeReason::BadArrayExtent(e) => return write!(f, "{e}"),
            UnsupportedTypeReason::UnsupportedForm => write!(
                f,
                "type `{}` is a form the prebindgen source language does not accept",
                self.offending
            ),
            UnsupportedTypeReason::DisallowedImplTrait => write!(
                f,
                "type `{}` is not an accepted callback — the only `impl Trait` in the language is \
                 `impl Fn(..) + Send + Sync + 'static` returning `()`",
                self.offending
            ),
            UnsupportedTypeReason::WrongGenericArity { expected } => write!(
                f,
                "type `{}` needs exactly {expected} type argument(s)",
                self.offending
            ),
            UnsupportedTypeReason::UnsupportedTuple => write!(
                f,
                "type `{}` is a tuple; only the unit `()` is supported — return the \
                 components separately, or wrap them in a `#[prebindgen]` struct",
                self.offending
            ),
            UnsupportedTypeReason::OwnedUninit => write!(
                f,
                "type `{}` is uninitialized storage outside an out-parameter. Only `&mut \
                 MaybeUninit<T>` means anything at a boundary \u{2014} it says the caller supplies \
                 the slot and the callee fills it; owned or in a field it promises nothing, and \
                 reading it would be undefined",
                self.offending
            ),
            UnsupportedTypeReason::SharedUninit => write!(
                f,
                "type `{}` is a shared borrow of uninitialized storage: `&T` promises a readable \
                 `T`, which this may not be. Use `&mut MaybeUninit<T>` for an out-parameter",
                self.offending
            ),
            UnsupportedTypeReason::AssociatedType => write!(
                f,
                "type `{}` is an associated type; `#[prebindgen]` never captures `impl` \
                 blocks, so its resolution is unknowable here — name the concrete type",
                self.offending
            ),
            UnsupportedTypeReason::UnsupportedGenericArgument => write!(
                f,
                "type `{}` has a generic argument that is neither a type nor a lifetime",
                self.offending
            ),
        }?;
        write!(f, " — see docs/source-language.md for the accepted grammar")
    }
}

impl std::error::Error for UnsupportedType {}

/// Lower one captured type.
///
/// **Total over the accepted grammar**: `Ok` means every part of the type was
/// understood, so a form this function does not lower is a form the language
/// does not accept.
///
/// `at` is the origin of the item this type was written in — the location every
/// node lowered from that item shares, and the crate an array extent's const
/// must come from.
/// The wrappers this language **erases**: `W<T>` classifies as whatever `T`
/// classifies as, because no destination language can tell them apart.
///
/// The single source of truth for that set. [`lower_type`] erases exactly these,
/// and an adapter that has to *undo* one in generated Rust reads the same list —
/// so the question "which wrappers are transparent?" has one answer instead of a
/// copy per consumer that can drift out of step.
///
/// Adding one is adding a row here. What it means for a given destination is
/// that adapter's business: erasing a wrapper says nothing about whether Rust
/// can move a value out of it, which is why `Cow` is on this list and is still
/// refused where a converter would have to move its payload.
pub const TRANSPARENT_WRAPPERS: &[&str] = &["Box", "Cow"];

/// Strip one [transparent wrapper](TRANSPARENT_WRAPPERS) from a **spelling**,
/// naming the one removed — `Box<Option<T>>` → `("Box", Option<T>)`.
///
/// Spelling in, spelling out: this is the inverse of the erasure, for a consumer
/// that must reconstruct in Rust what the classification dropped.
pub fn peel_transparent(ty: &syn::Type) -> Option<(&'static str, syn::Type)> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    let name = TRANSPARENT_WRAPPERS.iter().find(|w| seg.ident == **w)?;
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(inner) => Some((*name, inner.clone())),
        _ => None,
    })
}

pub(crate) fn lower_type(
    ty: &syn::Type,
    consts: &ConstIndex,
    at: &Rc<SourceLocation>,
) -> Result<TypeRef, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    // Every arm builds `kind` only; the origin is attached once, here, so no arm
    // can forget it or attach a rebuilt approximation.
    let kind = match ty {
        // A group or paren wraps the same type. Its inner node keeps the inner
        // spelling, which is the one a consumer wants to emit.
        syn::Type::Group(g) => return lower_type(&g.elem, consts, at),
        syn::Type::Paren(p) => return lower_type(&p.elem, consts, at),
        // The mode is read off the borrow AND its target together, because
        // `&mut MaybeUninit<T>` is one concept — an out-parameter — rather than a
        // mutable borrow of a distinct `MaybeUninit` type.
        syn::Type::Reference(r) => {
            let (mode, target) = match maybe_uninit_inner(&r.elem) {
                Some(inner) if r.mutability.is_some() => (RefMode::Out, inner),
                // `&MaybeUninit<T>` promises a readable `T` and supplies storage
                // that may not be one. Nothing at a boundary can use it.
                Some(_) => return Err(fail(UnsupportedTypeReason::SharedUninit)),
                None if r.mutability.is_some() => (RefMode::Exclusive, (*r.elem).clone()),
                None => (RefMode::Shared, (*r.elem).clone()),
            };
            TypeKind::Ref {
                mode,
                inner: Box::new(lower_type(&target, consts, at)?),
            }
        }
        // `[T]` is the borrowed spelling of the same concept `Vec<T>` owns.
        syn::Type::Slice(s) => TypeKind::Sequence(Box::new(lower_type(&s.elem, consts, at)?)),
        _ if is_unit_type(ty) => TypeKind::Unit,
        // Only the unit is in the language. Refusing here names the type;
        // accepting would defer the failure to an "unresolved type" much later.
        syn::Type::Tuple(_) => return Err(fail(UnsupportedTypeReason::UnsupportedTuple)),
        syn::Type::Array(a) => {
            let rendered = a.to_token_stream().to_string();
            let extent = lower_array_len(&a.len, &rendered, at, consts)
                .map_err(|e| fail(UnsupportedTypeReason::BadArrayExtent(Box::new(e))))?;
            TypeKind::Array {
                elem: Box::new(lower_type(&a.elem, consts, at)?),
                extent: Box::new(extent),
            }
        }
        // The callback shape is decided by `extract_fn_trait_args`, this
        // module's own — and the pipeline's only — authority for the form.
        syn::Type::ImplTrait(_) => match super::extract_fn_trait_args(ty) {
            Some(args) => TypeKind::Callback {
                args: args
                    .iter()
                    .map(|a| lower_type(a, consts, at))
                    .collect::<Result<_, _>>()?,
            },
            None => return Err(fail(UnsupportedTypeReason::DisallowedImplTrait)),
        },
        syn::Type::Path(tp) => lower_path(ty, tp, consts, at)?,
        _ => return Err(fail(UnsupportedTypeReason::UnsupportedForm)),
    };
    Ok(TypeRef {
        kind,
        origin: Origin::new(ty.clone(), Rc::clone(at)),
    })
}

fn lower_path(
    ty: &syn::Type,
    tp: &syn::TypePath,
    consts: &ConstIndex,
    at: &Rc<SourceLocation>,
) -> Result<TypeKind, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    // An associated type is refused rather than carried: the frontend never
    // captures `impl` blocks, so what `<T as Trait>::Assoc` resolves to is
    // unknowable here, and keeping the spelling would only move the failure
    // downstream.
    if tp.qself.is_some() {
        return Err(fail(UnsupportedTypeReason::AssociatedType));
    }
    let Some(last) = tp.path.segments.last() else {
        return Err(fail(UnsupportedTypeReason::UnsupportedForm));
    };
    let name = last.ident.to_string();

    // Type arguments only. A lifetime argument is accepted and dropped: it is
    // part of the spelling (`Foo<'a>` is not `Foo`), and the spelling is in
    // `TypeRef::origin`, so modelling it would be a second copy of one fact.
    let mut has_lifetime_arg = false;
    let args: Vec<TypeRef> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => {
                        out.push(lower_type(t, consts, at)?);
                    }
                    syn::GenericArgument::Lifetime(_) => has_lifetime_arg = true,
                    _ => return Err(fail(UnsupportedTypeReason::UnsupportedGenericArgument)),
                }
            }
            out
        }
        syn::PathArguments::Parenthesized(_) => {
            return Err(fail(UnsupportedTypeReason::UnsupportedForm))
        }
    };

    // A builtin must be spelled BARE. `normalize_type` has already reduced the
    // real std paths (`std::option::Option` → `Option`) at ingest and
    // deliberately leaves unknown crate paths alone, so anything still carrying
    // a prefix is a foreign type that merely shares a name — `foreign::Option`
    // is not `Option`, and collapsing it would silently retype the field.
    let is_bare = tp.path.leading_colon.is_none() && tp.path.segments.len() == 1;
    if is_bare {
        if args.is_empty() && !has_lifetime_arg {
            if let Some(kind) = ScalarKind::from_name(&name) {
                return Ok(TypeKind::Scalar(kind));
            }
            // `String` and `str` are one concept. `str` is unsized and so only
            // ever appears behind a `&`, which the `Ref` layer already records
            // — classifying it as a nominal type instead would send every
            // adapter looking for an item named `str` to resolve.
            if name == "String" || name == "str" {
                return Ok(TypeKind::Str);
            }
        }
        // A builtin generic takes TYPE arguments only — a lifetime on one is not a
        // shape this language has — with one exception: `Cow`'s own signature HAS a
        // lifetime, so it is the one builtin where a lifetime argument is expected
        // rather than refused.
        if !has_lifetime_arg || name == "Cow" {
            let mut args = args;
            let arity = |n: usize| {
                if args.len() == n {
                    Ok(())
                } else {
                    Err(fail(UnsupportedTypeReason::WrongGenericArity {
                        expected: n,
                    }))
                }
            };
            match name.as_str() {
                "Option" => {
                    arity(1)?;
                    return Ok(TypeKind::Optional(Box::new(args.remove(0))));
                }
                "Vec" => {
                    arity(1)?;
                    return Ok(TypeKind::Sequence(Box::new(args.remove(0))));
                }
                // Reached here, it is not behind a `&mut`, so it is not an
                // out-parameter — see `RefMode::Out`, which is the only place
                // uninitialized storage means anything.
                "MaybeUninit" => return Err(fail(UnsupportedTypeReason::OwnedUninit)),
                // `Box<T>` **is** `T`: an owned value either way, and no
                // destination language can tell the two apart. So it carries no
                // kind of its own and classifies as whatever it wraps — the
                // `Box` survives in `TypeRef::origin`, which is what generated
                // Rust spells. (A shared-ownership handle would classify as a
                // `Ref` for the same reason, when the language accepts one.)
                // `Box` and `Cow` are erased — see `TRANSPARENT_WRAPPERS`,
                // which is the list this arm consults so the set cannot drift
                // from the one adapters undo.
                w if TRANSPARENT_WRAPPERS.contains(&w) => {
                    arity(1)?;
                    return Ok(args.remove(0).kind);
                }
                "Result" => {
                    arity(2)?;
                    let err = Box::new(args.remove(1));
                    let ok = Box::new(args.remove(0));
                    return Ok(TypeKind::Fallible { ok, err });
                }
                _ => return Ok(named(tp)),
            }
        }
    }
    Ok(named(tp))
}

/// If `ty` is a bare `MaybeUninit<T>`, the `T` it holds storage for.
///
/// Bare, for the reason every builtin generic is: `normalize_type` has already
/// reduced the real std paths at ingest, so anything still carrying a prefix is a
/// foreign type that merely shares the name.
fn maybe_uninit_inner(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() || tp.path.segments.len() != 1 {
        return None;
    }
    let seg = &tp.path.segments[0];
    if seg.ident != "MaybeUninit" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    match ab.args.first() {
        Some(syn::GenericArgument::Type(t)) if ab.args.len() == 1 => Some(t.clone()),
        _ => None,
    }
}

/// True when `ty` is the unit type `()`.
///
/// The language's one answer to that question: [`lower_type`] classifies it as
/// [`TypeKind::Unit`], and the callback grammar uses it to insist a callback
/// returns nothing.
pub(crate) fn is_unit_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Tuple(t) => t.elems.is_empty(),
        // A parenthesized or grouped `()` is still `()`.
        syn::Type::Paren(p) => is_unit_type(&p.elem),
        syn::Type::Group(g) => is_unit_type(&g.elem),
        _ => false,
    }
}

/// `Named` with the identity read off the path: every segment joined, minus the
/// generic arguments, which are already in `args`.
/// `Named` with the identity read off the path: every segment joined, minus the
/// generic arguments.
///
/// The arguments are **lowered but not retained** — a bad type inside one is still
/// diagnosed, it just leaves no trace. Nothing could read them: a surviving
/// reference resolves to a declared type, and no declaration takes type parameters,
/// so `Foo<u8>` against a declared `Foo` would not compile in the source crate.
/// Accepting *instantiated* generics — `Wrapper<u8>` as its own declared type — is
/// what would bring the field back.
fn named(tp: &syn::TypePath) -> TypeKind {
    let name = tp
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    TypeKind::Named {
        id: TypeId { name },
    }
}
