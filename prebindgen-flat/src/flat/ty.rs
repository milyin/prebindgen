//! Types: the accepted syntax, paired with the tokens it was read from.
//!
//! [`TypeKind`] is the subset of [`syn::Type`] a `#[prebindgen]` crate may
//! write — one variant per accepted **form**, nothing folded together, nothing
//! interpreted. What `&str` and `String` have in common is a *destination*
//! language's business, and the adapters are where that decision belongs.
//!
//! [`TypeRef`] pairs that kind with the tokens the source wrote. The pairing
//! survives the pivot because the two answer different questions — the kind is
//! the grammar an adapter may rely on, the syntax is what generated Rust must
//! spell — but the syntax is no longer *load-bearing*: nothing is recoverable
//! only from it. [`TypeKind::to_syn`] is what checks that, and
//! `syntax_is_recoverable_from_kind` is what runs it over the whole acceptance
//! corpus.
//!
//! [`TypeKind`] is total over the accepted grammar: a form with no variant here
//! is a form the language does not accept, so acceptance is mostly a
//! consequence of lowering rather than a second list that can drift from it.
//! Mostly: [`Uninit`](TypeKind::Uninit) is accepted in one **position** only,
//! which no variant set can express — see
//! [`OwnedUninit`](UnsupportedTypeReason::OwnedUninit). Same contract otherwise,
//! and for the same reason, as [`lower_array_len`].

use std::{fmt, rc::Rc};

use prebindgen::SourceLocation;
use quote::ToTokens;

use super::{
    array_len::{lower_array_len, ArrayExtent, ConstIndex, UnsupportedArrayLen},
    key::TypeKey,
    origin::Origin,
};

/// A type as the language accepted it, plus the exact syntax it came from.
///
/// The retained slice is what generated Rust spells, through
/// [`spell`](Self::spell). It is **not**
/// where facts go to survive a lossy classification any more — `kind` keeps the
/// lifetime, the wrapper and the argument it used to drop, and
/// [`TypeKind::to_syn`] proves it. Keeping the slice anyway is cheap, exact
/// (nothing has to reconstruct token for token what the source already wrote),
/// and it is what makes the proof possible at all.
///
/// # The invariant
///
/// > **Every `TypeRef` was classified by the model.** [`Flat`](super::Flat)
/// > classified it from source syntax, or the registry pipeline composed it by
/// > layering over something already classified.
///
/// **Historically enforced by visibility, now by convention.** Before the
/// registry pipeline moved to the separate `prebindgen-registry` crate, the
/// boundary was `api::core` and was drawn by visibility at four places, each
/// checked by the compiler on every build:
///
/// | | |
/// |---|---|
/// | the `kind` and `origin` fields | `pub(super)` — a public field **is** a constructor, so restricting only the composers would block nothing |
/// | `borrowed` / `optional` / `scalar` | `pub(crate)` |
/// | `named` | `pub(super)` — `flat` alone |
/// | `Flat::classify` | `pub(crate)` |
///
/// A module-path seal can no longer express "the registry pipeline, and
/// nothing else" once that pipeline is a different crate — there is no path
/// inside this crate to name it — so `borrowed` / `optional` / `scalar` and
/// `Flat::classify` are now plain `pub`, and the fields stay `pub(super)`
/// (nothing outside `flat` ever needed them). The intent is unchanged and
/// documented here, but no longer compiler-enforced against a destination
/// adapter (`prebindgen-c`, `prebindgen-jni`): restoring that would need a real
/// API, e.g. a sealed capability token minted only by `prebindgen-registry`.
/// Where one needs a type the model already declares, the **declaration**
/// answers: see [`Variant::type_ref`](super::Variant::type_ref), which is what
/// the `SumTag` selector uses instead of composing a reading from an ident.
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
///
/// It does **not** claim a registry cell either, and the two are separate
/// questions. Holding a `TypeRef` means the model classified the type; whether
/// it is in a type table is the registry's business, and the registry states it
/// in three parts — a **cell** (the type entered the pipeline), a **root** (the
/// binding asked for it directly), an **entry** (a converter resolved). A
/// `SumTag` leaf's type makes the first and not the second, deliberately
/// (#282); see `Registry::reference_output` in the registry layer above.
#[derive(Clone, Debug)]
pub struct TypeRef {
    /// The accepted syntax this type is — the closed grammar, not an
    /// interpretation of it.
    pub(super) kind: TypeKind,
    /// The type as generated Rust must spell it — the source's own tokens,
    /// normalized to the flat namespace the generated crate can name (see
    /// [`Flat::parse`](super::Flat::parse)) — plus the source they came
    /// from.
    ///
    /// It says exactly what `kind` says — that is the invariant
    /// [`TypeKind::to_syn`] checks — and it says it in the source's own tokens,
    /// which is why generated Rust re-emits this rather than a reconstruction.
    pub(super) origin: Origin<syn::Type>,
}

impl fmt::Display for TypeRef {
    /// The type as the source wrote it, **for a message**.
    ///
    /// Diagnostics are not emission: a panic naming an unsupported type is
    /// decision code reporting why it decided, and it must not need the
    /// [`Emit`](crate::flat::emit::Emit) capability to say so. So this is
    /// ungated where [`spell`](Self::spell) is not.
    ///
    /// **The identity, not the spelling** — `TypeKey`, which is
    /// `canonical_type` rendered. Delegating to `spell()` would have handed the
    /// captured spelling back out through `format!("{ty}")`, so
    /// `syn::parse_str(&ty.to_string())` reconstructed it exactly and the
    /// capability was a suggestion. Rendering the canonical form keeps
    /// diagnostics readable while making the round trip land on a *normalized*
    /// type rather than the source's own tokens.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key().as_str())
    }
}

impl TypeRef {
    /// What the type means. **Classify off this**, never off the spelling.
    ///
    /// The seal, as a compiled assertion. An out-of-crate consumer cannot
    /// assemble a reading, because the fields it would have to name are private
    /// (`E0451`):
    ///
    /// ```compile_fail
    /// # use prebindgen_flat::flat::{TypeKind, TypeRef};
    /// let forged = TypeRef { kind: TypeKind::Unit, origin: todo!() };
    /// ```
    ///
    /// …nor, historically, through a composer (`E0624`) — **no longer true**:
    /// `borrowed` / `optional` / `scalar` are `pub` now that the registry
    /// pipeline that composes with them is the separate `prebindgen-registry`
    /// crate rather than code inside this one:
    ///
    /// ```
    /// # use prebindgen_flat::flat::{ScalarKind, TypeRef};
    /// let composed = TypeRef::scalar(ScalarKind::Bool);
    /// ```
    ///
    /// The struct-literal case above still proves the **crate** boundary. The
    /// stronger claim this crate used to enforce by visibility — that nothing
    /// above `api::core` can mint one either — no longer has a module path to
    /// be checked against once the registry pipeline is the separate
    /// `prebindgen-registry` crate; see the type-level doc's "The invariant"
    /// section for what replaced it.
    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    /// The tokens generated Rust must spell. **Spell off this**, never off
    /// `kind` — re-deriving a spelling from the classification is how
    /// `Box<Option<T>>` becomes an `E0308`.
    ///
    /// Tokens, not a `syn::Type`: a spelling is for spelling. What the type
    /// *is* has an answer in [`kind`](Self::kind) and in the readings beside it,
    /// and a consumer that still has to take the node apart says so with
    /// [`as_syn`](Self::as_syn).
    pub fn spell(&self) -> proc_macro2::TokenStream {
        self.origin.spell()
    }

    /// The type as `syn` — **the escape**. See [`Origin::as_syn`].
    // Test-only as of C7: `Emit` hands out a spelling, never the node, so the
    // round-trip checks (`syntax_is_recoverable_from_kind`) are the last
    // callers. That is the correct end state — the check that a kind can
    // reproduce its own syntax needs both halves.
    #[allow(dead_code)]
    pub(crate) fn as_syn(&self) -> &syn::Type {
        self.origin.as_syn()
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
    /// same [`Shape`](crate::shape::Shape) the expansion and decomposition plans are built from — so a
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
    pub fn layer_stack(&self) -> (crate::shape::Shape, &TypeRef) {
        use crate::shape::Shape;
        // Bounded on purpose, and not a recursion: the accepted crossing is
        // `Option<Vec<T>>` — at most one optional, then at most one run, in that
        // order. Recursing would accept `Vec<Option<T>>` as `Iterable(Optional)`,
        // which reads the inner optional as a boundary layer when it is part of
        // the element, and `Option<Option<T>>` as two nullable layers when the
        // boundary has one way to say absent.
        // Through the transparent wrappers, never past the node: a layer is read
        // off `unwrapped`, while the type this returns is the one the source
        // spelled — `Box<Foo>` is a `Base` whose core still spells the `Box`.
        let mut core = self;
        let optional = matches!(core.unwrapped().kind, TypeKind::Optional(_));
        if let TypeKind::Optional(inner) = &core.unwrapped().kind {
            core = inner;
        }
        let iterable = matches!(core.unwrapped().kind, TypeKind::Vec(_) | TypeKind::Slice(_));
        if let TypeKind::Vec(inner) | TypeKind::Slice(inner) = &core.unwrapped().kind {
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
        if let TypeKind::Optional(inner) = &cur.unwrapped().kind {
            out.push(inner);
            cur = inner;
        }
        if let TypeKind::Vec(inner) | TypeKind::Slice(inner) = &cur.unwrapped().kind {
            out.push(inner);
        }
        out
    }

    /// This type with every [transparent wrapper](TRANSPARENT_WRAPPERS) peeled
    /// off — `Box<Cow<'_, [T]>>` → the `[T]` node, an unwrapped type → itself.
    ///
    /// **The fold, made explicit.** [`kind`](Self::kind) is the syntax the source
    /// wrote, wrappers and all; a consumer that does not care which of them stand
    /// over a type says so here, at its own call site, and the ones that must put
    /// them back in generated Rust ask [`erased_wrappers`](Self::erased_wrappers)
    /// instead. That split is why the wrapper is no longer erased during
    /// lowering: the model reports, the consumer decides.
    ///
    /// Per layer, and only this one: a wrapper under a borrow or inside an
    /// `Option` belongs to that inner node, which answers for itself.
    pub fn unwrapped(&self) -> &TypeRef {
        match &self.kind {
            TypeKind::Boxed(inner) | TypeKind::Cow { inner, .. } => inner.unwrapped(),
            _ => self,
        }
    }

    /// What an `Option<T>` wraps, else `None`.
    ///
    /// One layer, named. [`layer_stack`](Self::layer_stack) reads the whole
    /// arity stack; these three read exactly the layer a caller asks for, which
    /// is what a consumer wants when it can only *represent* some of them.
    ///
    /// Read through [`unwrapped`](Self::unwrapped), like every layer accessor
    /// here: `Box<Option<T>>` is an optional to a destination language, and the
    /// `Box` is still on the node for whoever has to spell it.
    pub fn optional_inner(&self) -> Option<&TypeRef> {
        match &self.unwrapped().kind {
            TypeKind::Optional(inner) => Some(inner),
            _ => None,
        }
    }

    /// The element of a run of values (`Vec<T>`, `[T]`), else `None`.
    pub fn sequence_elem(&self) -> Option<&TypeRef> {
        match &self.unwrapped().kind {
            TypeKind::Vec(elem) | TypeKind::Slice(elem) => Some(elem),
            _ => None,
        }
    }

    /// What a borrow points at, else `None`.
    ///
    /// Through an out-parameter's [`Uninit`](TypeKind::Uninit): `&mut
    /// MaybeUninit<T>` points at a `T`'s storage, and the slot is not a type
    /// anything converts, registers or crosses with. A consumer that needs to
    /// tell the two borrows apart reads the [`kind`](Self::kind), where the
    /// `MaybeUninit` the source wrote is still standing.
    pub fn borrow_target(&self) -> Option<&TypeRef> {
        let inner = match &self.unwrapped().kind {
            TypeKind::Ref { inner, .. } => inner,
            _ => return None,
        };
        Some(match &inner.kind {
            TypeKind::Uninit(slot) => slot,
            _ => inner,
        })
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
    // `spell()` in one place, so the classification and the spelling
    // cannot disagree — the invariant every consumer of a `TypeRef` relies on.

    /// A borrow of this type — `&T` from `T`.
    ///
    /// Keeps this type's location: the borrow exists *because of* this value,
    /// so a diagnostic about it should point where the value came from.
    pub fn borrowed(&self) -> TypeRef {
        let inner = self.origin.spell();
        TypeRef {
            kind: TypeKind::Ref {
                lifetime: None,
                mutable: false,
                inner: Box::new(self.clone()),
            },
            origin: self.origin.with(syn::parse_quote!(&#inner)),
        }
    }

    /// An optional of this type — `Option<T>` from `T`. Location as
    /// [`Self::borrowed`].
    pub fn optional(&self) -> TypeRef {
        let inner = self.origin.spell();
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
    pub fn scalar(kind: ScalarKind) -> TypeRef {
        // The spelling comes from the kind, so the two cannot drift.
        let ident = syn::Ident::new(kind.as_str(), proc_macro2::Span::call_site());
        TypeRef {
            kind: TypeKind::Scalar(kind),
            origin: Origin::new(
                syn::parse_quote!(#ident),
                std::rc::Rc::new(prebindgen::SourceLocation::default()),
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
                args: Vec::new(),
            },
            origin: Origin::new(
                syn::parse_quote!(#ident),
                std::rc::Rc::new(prebindgen::SourceLocation::default()),
            ),
        }
    }

    /// This type's identity as a table key.
    ///
    /// The canonical spelling is what a key *is* (#113), and reading it is
    /// legitimate — but it should be the model's answer rather than every caller
    /// reaching for the spelling itself, since a caller that reaches
    /// into `origin` to *reason* is the thing this model exists to stop.
    pub fn key(&self) -> TypeKey {
        TypeKey::from_type(self.origin.as_syn())
    }

    /// The [transparent wrapper](TRANSPARENT_WRAPPERS) this type's **spelling**
    /// adds over its classification, if any — `Box<Option<T>>` → `Some("Box")`,
    /// `Option<T>` → `None`.
    ///
    /// This exists because [`kind`](Self::kind) and [`spell`](Self::spell)
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
        let mut ty = self;
        loop {
            let name = match &ty.kind {
                TypeKind::Boxed(inner) => {
                    ty = inner;
                    "Box"
                }
                TypeKind::Cow { inner, .. } => {
                    ty = inner;
                    "Cow"
                }
                _ => return names,
            };
            names.push(name);
        }
    }

    /// This type's identity as a table key with every transparent wrapper
    /// removed — the key of [`stripped_syntax`](Self::stripped_syntax).
    ///
    /// What [`key`](Self::key) answers for a **spelling**, this answers for the
    /// **type**. The two are different questions and both are legitimate:
    ///
    /// * a *conversion* is keyed by `key`, because `Box<Option<T>>` and
    ///   `Option<T>` genuinely need different converter bodies — one has to put
    ///   a `Box` back and the other must not;
    /// * a **declaration** is keyed by this, because a declaration says what a
    ///   type *is* to the destination language, and a wrapper the model erases
    ///   cannot change that. A `Box<Payload>` parameter is a `Payload` to
    ///   Kotlin, so it must find `Payload`'s data-class declaration — keying it
    ///   by spelling finds nothing and silently costs the parameter its
    ///   lowering.
    ///
    /// Use this wherever the lookup is against declarations the binding author
    /// wrote, and `key` wherever it is against something derived per spelling.
    pub fn stripped_key(&self) -> TypeKey {
        TypeKey::from_type(&self.stripped_syntax())
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
    pub(crate) fn stripped_syntax(&self) -> syn::Type {
        self.unwrapped().origin.as_syn().clone()
    }

    /// True when this is `&mut T` over a **value** — not `&mut MaybeUninit<T>`.
    ///
    /// The one distinction an out-parameter's form makes to a converter: an
    /// exclusive borrow may be read before it is written and an out-parameter
    /// may not, so the two cannot share a conversion. Everything else about the
    /// slot — that it points at a `T`, that the `T` is what crosses — is
    /// [`borrow_target`](Self::borrow_target)'s answer.
    pub fn is_exclusive_borrow(&self) -> bool {
        matches!(
            &self.unwrapped().kind,
            TypeKind::Ref { mutable: true, inner, .. } if !matches!(inner.kind, TypeKind::Uninit(_))
        )
    }

    /// The `Ok` and `Err` sides when this is a `Result`, else `None`.
    pub fn fallible_parts(&self) -> Option<(&TypeRef, &TypeRef)> {
        match &self.unwrapped().kind {
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
        match &self.unwrapped().kind {
            TypeKind::Callback { args } => Some(args),
            _ => None,
        }
    }

    /// The extent of this type when it is an array, else `None`.
    pub fn array_extent(&self) -> Option<&ArrayExtent> {
        match &self.unwrapped().kind {
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
            // The name resolves; the arguments do not. No declaration takes type
            // parameters, so `Foo<Bar>` is one reference to `Foo` — requiring
            // `Bar` to be declared as well would refuse a reference the source
            // crate compiles.
            TypeKind::Named { id, .. } => (!declared.contains(&id.name)).then(|| id.name.clone()),
            TypeKind::Optional(t)
            | TypeKind::Vec(t)
            | TypeKind::Slice(t)
            | TypeKind::Boxed(t)
            | TypeKind::Uninit(t)
            | TypeKind::Cow { inner: t, .. }
            | TypeKind::Ref { inner: t, .. } => t.first_unresolved(declared),
            TypeKind::Array { elem, .. } => elem.first_unresolved(declared),
            TypeKind::Fallible { ok, err } => ok
                .first_unresolved(declared)
                .or_else(|| err.first_unresolved(declared)),
            TypeKind::Callback { args } => args.iter().find_map(|a| a.first_unresolved(declared)),
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::String | TypeKind::Unit => None,
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
    /// here. The full spelling is [`Self::spell`]'s answer for whoever needs it.
    pub fn walk(&self) -> Vec<&TypeRef> {
        let mut out = Vec::new();
        self.collect_refs(&mut out);
        out
    }

    // Both walks descend through [`unwrapped`](Self::unwrapped): a transparent
    // wrapper is not a type of its own to a consumer that indexes or converts,
    // so `Box<Vec<Foo>>` reaches `Foo` and yields no node in between.
    fn collect_refs<'a>(&'a self, out: &mut Vec<&'a TypeRef>) {
        out.push(self);
        match &self.unwrapped().kind {
            // Through [`borrow_target`](Self::borrow_target), so an
            // out-parameter reaches the value and not its slot.
            TypeKind::Ref { .. } => {
                if let Some(t) = self.borrow_target() {
                    t.collect_refs(out)
                }
            }
            TypeKind::Optional(t) | TypeKind::Vec(t) | TypeKind::Slice(t) | TypeKind::Uninit(t) => {
                t.collect_refs(out)
            }
            TypeKind::Array { elem, .. } => elem.collect_refs(out),
            TypeKind::Fallible { ok, err } => {
                ok.collect_refs(out);
                err.collect_refs(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_refs(out)),
            TypeKind::Named { .. }
            | TypeKind::Scalar(_)
            | TypeKind::Str
            | TypeKind::String
            | TypeKind::Unit => {}
            // `unwrapped` peeled these off, so reaching one is impossible.
            TypeKind::Boxed(_) | TypeKind::Cow { .. } => unreachable!(),
        }
    }

    fn collect_extents<'a>(&'a self, out: &mut Vec<&'a ArrayExtent>) {
        match &self.unwrapped().kind {
            TypeKind::Array { elem, extent } => {
                out.push(extent);
                elem.collect_extents(out);
            }
            TypeKind::Optional(t)
            | TypeKind::Vec(t)
            | TypeKind::Slice(t)
            | TypeKind::Uninit(t)
            | TypeKind::Ref { inner: t, .. } => t.collect_extents(out),
            TypeKind::Fallible { ok, err } => {
                ok.collect_extents(out);
                err.collect_extents(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_extents(out)),
            TypeKind::Named { .. }
            | TypeKind::Scalar(_)
            | TypeKind::Str
            | TypeKind::String
            | TypeKind::Unit => {}
            TypeKind::Boxed(_) | TypeKind::Cow { .. } => unreachable!(),
        }
    }
}

/// The **accepted syntax** of a [`TypeRef`]: the subset of [`syn::Type`] a
/// `#[prebindgen]` crate may write, and nothing more.
///
/// One variant per accepted Rust **form**, not per destination concept. `str`
/// and `String` are two forms and get two variants; `Box<T>` is a form of its
/// own and does not disappear into `T`. Nothing here folds two spellings
/// together, which is what makes [`TypeRef::syntax`] recoverable from this —
/// see [`TypeKind::to_syn`], the round-trip that checks it.
///
/// # Why it is only syntax
///
/// It was a *destination-neutral classification* once, and that leaked: `&T`
/// earned a layer while `Box<T>` was declared transparent, on no principle
/// either adapter shared, and `Cbindgen` went on picking its C type from the
/// Rust spelling anyway. Deciding that `&str` and `String` are both "a string"
/// is a **destination** decision, so it belongs to the destination — the model
/// hands over what the source wrote and stays out of it.
///
/// Where two adapters want the same fold, it is a *reading*, not a variant:
/// [`TypeRef::unwrapped`] peels `Box`/`Cow` for the consumers that want them
/// gone, and the ones that must rebuild the Rust value ask
/// [`TypeRef::erased_wrappers`] instead. One helper, visible at the call site,
/// rather than a fold baked into every classification.
#[derive(Clone, Debug)]
pub enum TypeKind {
    /// A primitive with a fixed C/JVM counterpart — `u8`, `bool`, `f64`.
    ///
    /// A closed set of bare idents, so recognising one is reading the syntax
    /// rather than interpreting it — and it keeps every adapter off a name
    /// table of its own.
    Scalar(ScalarKind),
    /// `str` — unsized, so it is only ever reached through a
    /// [`Ref`](TypeKind::Ref) or a wrapper.
    Str,
    /// `String`.
    String,
    /// `Option<T>`.
    Optional(Box<TypeRef>),
    /// `Vec<T>`.
    Vec(Box<TypeRef>),
    /// `[T]` — the unsized run, reached through a [`Ref`](TypeKind::Ref) or a
    /// wrapper. Not the same form as [`Vec`](TypeKind::Vec), so not the same
    /// variant.
    Slice(Box<TypeRef>),
    /// `Result<T, E>`.
    Fallible { ok: Box<TypeRef>, err: Box<TypeRef> },
    /// Any other named type: a `#[prebindgen]` struct or enum, or a foreign
    /// path.
    ///
    /// `id` is the type's **identity** — a name, not a `syn::Path`, so nothing
    /// downstream has to take a path apart to learn what a type is. `args` is
    /// the last segment's generic arguments, in the order they were written and
    /// including lifetimes, because dropping either would make the spelling
    /// unrecoverable.
    Named { id: TypeId, args: Vec<GenericArg> },
    /// `[T; N]` — a run of `T` whose length is known at compile time.
    Array {
        elem: Box<TypeRef>,
        /// Boxed: an extent carries an [`Origin`] over the length expression, which
        /// makes it the size outlier among the kinds, and an array is the rare one.
        /// The same trade-off [`Unsupported::error`](super::Unsupported) makes.
        extent: Box<ArrayExtent>,
    },
    /// A borrow — `&T` or `&mut T`, with the lifetime the source wrote.
    ///
    /// An out-parameter is `&mut` over [`Uninit`](TypeKind::Uninit), which is
    /// what the source spells. What that *means* at a boundary — the caller
    /// supplies the slot, the callee fills it — is the adapter's reading of the
    /// form, not a third value of a mode enum.
    Ref {
        lifetime: Option<syn::Lifetime>,
        mutable: bool,
        inner: Box<TypeRef>,
    },
    /// `Box<T>`.
    ///
    /// A form of its own. It was erased once, on the grounds that no
    /// destination language can tell `Box<T>` from `T` — true, and still the
    /// adapter's call to make: [`TypeRef::unwrapped`] makes it, on demand.
    Boxed(Box<TypeRef>),
    /// `Cow<'a, T>`.
    ///
    /// The lifetime is **not** optional: `Cow` has one in its own signature, so
    /// a `Cow<T>` is not Rust and no source crate can compile it. Lowering
    /// refuses the shape ([`WrongGenericArguments`](UnsupportedTypeReason::WrongGenericArguments))
    /// rather than modelling an absence that would then have to be spelled back
    /// as something the source did not write.
    Cow {
        lifetime: syn::Lifetime,
        inner: Box<TypeRef>,
    },
    /// `MaybeUninit<T>`.
    ///
    /// Accepted **only** directly under a `&mut` — see
    /// [`UnsupportedTypeReason::OwnedUninit`]. It has a variant because the
    /// source writes it; that it is refused elsewhere is an acceptance rule,
    /// which is a separate question from how the form is represented.
    Uninit(Box<TypeRef>),
    /// `impl Fn(A, B, …) + Send + Sync + 'static` — the callback form.
    Callback { args: Vec<TypeRef> },
    /// `()`.
    Unit,
}

/// One generic argument of a [`Named`](TypeKind::Named) type, as written.
///
/// A lifetime is kept rather than dropped: no destination language acts on it,
/// but `Foo<'a>` is not `Foo`, and a model that cannot say which one the source
/// wrote cannot claim to have lost nothing.
#[derive(Clone, Debug)]
pub enum GenericArg {
    Lifetime(syn::Lifetime),
    /// Boxed so a lifetime argument — the common one, and a fraction of the
    /// size — does not pay for a type it is not. The same trade-off
    /// [`Array`](TypeKind::Array)'s extent makes.
    Type(Box<TypeRef>),
}

impl TypeKind {
    /// This kind spelled back as Rust — the inverse of the lowering.
    ///
    /// # What it is for
    ///
    /// **Not** for generating code: generated Rust spells
    /// [`TypeRef::syntax`], the source's own tokens, and always will. This
    /// exists so that claim can be *checked* — a kind that cannot reproduce the
    /// syntax it was lowered from has dropped something, and the round-trip test
    /// is what says so before a consumer has to discover it.
    ///
    /// Two forms reconstruct up to their own freedom rather than token for
    /// token, because the model keeps what was written and not how it was
    /// written:
    ///
    /// * a `Group` or `Paren` around a type, which the lowering sees through;
    /// * a [`Callback`](TypeKind::Callback)'s bound *order* — `Send + Sync` and
    ///   `Sync + Send` are one accepted form, and nothing reads the order.
    // Its whole job is the round-trip check (`syntax_is_recoverable_from_kind`),
    // and with the spelling sealed nothing in a built crate calls it — which is
    // the correct end state, not dead code: a kind that cannot reproduce its
    // own syntax has lost something, and this is what says so.
    #[allow(dead_code)]
    pub(crate) fn to_syn(&self) -> syn::Type {
        let opt_lifetime =
            |l: &Option<syn::Lifetime>| l.as_ref().map(|l| quote::quote!(#l)).unwrap_or_default();
        match self {
            Self::Scalar(k) => {
                let ident = syn::Ident::new(k.as_str(), proc_macro2::Span::call_site());
                syn::parse_quote!(#ident)
            }
            Self::Str => syn::parse_quote!(str),
            Self::String => syn::parse_quote!(String),
            Self::Optional(t) => {
                let inner = t.kind.to_syn();
                syn::parse_quote!(Option<#inner>)
            }
            Self::Vec(t) => {
                let inner = t.kind.to_syn();
                syn::parse_quote!(Vec<#inner>)
            }
            Self::Slice(t) => {
                let inner = t.kind.to_syn();
                syn::parse_quote!([#inner])
            }
            Self::Boxed(t) => {
                let inner = t.kind.to_syn();
                syn::parse_quote!(Box<#inner>)
            }
            Self::Uninit(t) => {
                let inner = t.kind.to_syn();
                syn::parse_quote!(MaybeUninit<#inner>)
            }
            Self::Cow { lifetime, inner } => {
                let inner = inner.kind.to_syn();
                syn::parse_quote!(Cow<#lifetime, #inner>)
            }
            Self::Fallible { ok, err } => {
                let (ok, err) = (ok.kind.to_syn(), err.kind.to_syn());
                syn::parse_quote!(Result<#ok, #err>)
            }
            Self::Ref {
                lifetime,
                mutable,
                inner,
            } => {
                let lt = opt_lifetime(lifetime);
                let mutability = mutable.then(|| quote::quote!(mut)).unwrap_or_default();
                let inner = inner.kind.to_syn();
                syn::parse_quote!(& #lt #mutability #inner)
            }
            Self::Array { elem, extent } => {
                let elem = elem.kind.to_syn();
                let len = extent.origin.spell();
                syn::parse_quote!([#elem; #len])
            }
            Self::Named { id, args } => {
                // The name is a spelling, so it parses back as one — including
                // the leading `::` and any path segments before the last.
                let mut path: syn::Path =
                    syn::parse_str(&id.name).expect("a name this model built from a path");
                if !args.is_empty() {
                    let args = args.iter().map(|a| match a {
                        GenericArg::Lifetime(l) => quote::quote!(#l),
                        GenericArg::Type(t) => {
                            let t = t.kind.to_syn();
                            quote::quote!(#t)
                        }
                    });
                    let last = path.segments.last_mut().expect("a non-empty path");
                    last.arguments =
                        syn::PathArguments::AngleBracketed(syn::parse_quote!(<#(#args),*>));
                }
                syn::parse_quote!(#path)
            }
            Self::Callback { args } => {
                let args = args.iter().map(|a| a.kind.to_syn());
                syn::parse_quote!(impl Fn(#(#args),*) + Send + Sync + 'static)
            }
            Self::Unit => syn::parse_quote!(()),
        }
    }
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
/// re-classification issue #211 exists to stop.
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
    ///
    /// Counts **type** arguments, which is the whole question for every builtin
    /// but one: a lifetime on `Option`, `Vec`, `Box` or `Result` is not a shape
    /// this language has, and such a spelling is a nominal type nobody declared
    /// rather than a builtin with a bad argument. `Cow` is the exception and has
    /// its own reason, [`WrongGenericArguments`](Self::WrongGenericArguments).
    WrongGenericArity { expected: usize },
    /// A builtin whose whole argument list is not the shape it takes —
    /// `Cow<u8>` (no lifetime), `Cow<u8, 'a>` (wrong order), `Cow<'a, 'b, u8>`
    /// (two lifetimes).
    ///
    /// Separate from [`WrongGenericArity`](Self::WrongGenericArity) because it
    /// is about the list and not its type-argument count: each of those three
    /// has exactly one type argument, and refusing them is what keeps
    /// [`TypeKind::to_syn`] able to spell every accepted form back.
    WrongGenericArguments { expected: &'static str },
    /// A non-empty tuple. Only `()` is in the language: no adapter has ever
    /// lowered a tuple, so accepting one would defer the failure to a late
    /// "unresolved type" instead of naming it here.
    UnsupportedTuple,
    /// `MaybeUninit<T>` somewhere other than directly under a `&mut`.
    ///
    /// The one acceptance rule about a **position** rather than a form:
    /// [`Uninit`](TypeKind::Uninit) exists, and only an out-parameter can hold
    /// one. Owned, returned or stored in a field it promises nothing a
    /// destination language can use, and reading it would be undefined.
    OwnedUninit,
    /// `&MaybeUninit<T>` — a shared borrow of uninitialized storage.
    ///
    /// A shared borrow promises a readable `T`, and this supplies storage that may
    /// not be one. Only `&mut MaybeUninit<T>` means anything — see
    /// [`Uninit`](TypeKind::Uninit).
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
            UnsupportedTypeReason::WrongGenericArguments { expected } => write!(
                f,
                "type `{}` is not the shape `{expected}` \u{2014} its arguments are the ones \
                 that type takes, in the order it takes them",
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
/// The wrappers a destination language **cannot see**: `W<T>` crosses as
/// whatever `T` crosses as, because nothing outside Rust can tell them apart.
///
/// The single source of truth for that set. [`TypeRef::unwrapped`] peels exactly
/// these, and an adapter that has to *put one back* in generated Rust reads the
/// same list — so the question "which wrappers are transparent?" has one answer
/// instead of a copy per consumer that can drift out of step.
///
/// It is a **reading**, not a classification: [`TypeKind`] keeps every wrapper
/// the source wrote, and a consumer says here, at its own call site, that it
/// does not care. What that means for a given destination is still that
/// adapter's business — erasing a wrapper says nothing about whether Rust can
/// move a value out of it, which is why `Cow` is on this list and is still
/// refused where a converter would have to move its payload.
pub const TRANSPARENT_WRAPPERS: &[&str] = &["Box", "Cow"];

/// Strip one [transparent wrapper](TRANSPARENT_WRAPPERS) from a **spelling**,
/// naming the one removed — `Box<Option<T>>` → `("Box", Option<T>)`.
///
/// Spelling in, spelling out — the syntax-side peer of
/// [`TypeRef::unwrapped`], for an adapter comparing a spelling it composed
/// against one it has a converter for. Here rather than in the adapter because
/// taking a `syn::Type` apart is this module's job, and doing it next door would
/// put a classifier back outside the model.
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
        // The borrow and its target are read together for one reason only: a
        // `MaybeUninit` is accepted **here** and refused everywhere else, so the
        // position is what decides, and only this arm knows it.
        syn::Type::Reference(r) => {
            let inner = match maybe_uninit_inner(&r.elem) {
                Some(uninit) if r.mutability.is_some() => TypeRef {
                    kind: TypeKind::Uninit(Box::new(lower_type(&uninit, consts, at)?)),
                    origin: Origin::new((*r.elem).clone(), Rc::clone(at)),
                },
                // `&MaybeUninit<T>` promises a readable `T` and supplies storage
                // that may not be one. Nothing at a boundary can use it.
                Some(_) => return Err(fail(UnsupportedTypeReason::SharedUninit)),
                None => lower_type(&r.elem, consts, at)?,
            };
            TypeKind::Ref {
                lifetime: r.lifetime.clone(),
                mutable: r.mutability.is_some(),
                inner: Box::new(inner),
            }
        }
        syn::Type::Slice(s) => TypeKind::Slice(Box::new(lower_type(&s.elem, consts, at)?)),
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

    // Every argument is kept, in the order it was written — a lifetime among
    // them. `Foo<'a>` is not `Foo`, and a model that drops the difference cannot
    // spell the type back.
    let mut has_lifetime_arg = false;
    let args: Vec<GenericArg> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => {
                        out.push(GenericArg::Type(Box::new(lower_type(t, consts, at)?)));
                    }
                    syn::GenericArgument::Lifetime(l) => {
                        has_lifetime_arg = true;
                        out.push(GenericArg::Lifetime(l.clone()));
                    }
                    _ => return Err(fail(UnsupportedTypeReason::UnsupportedGenericArgument)),
                }
            }
            out
        }
        syn::PathArguments::Parenthesized(_) => {
            return Err(fail(UnsupportedTypeReason::UnsupportedForm))
        }
    };
    // `Named` holds the last segment's arguments, so a generic anywhere else is
    // a spelling this model cannot give back. Refused rather than dropped: no
    // flat API writes `a::B<T>::C`.
    if tp
        .path
        .segments
        .iter()
        .rev()
        .skip(1)
        .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return Err(fail(UnsupportedTypeReason::UnsupportedForm));
    }

    // A builtin must be spelled BARE. `normalize_type` has already reduced the
    // real std paths (`std::option::Option` → `Option`) at ingest and
    // deliberately leaves unknown crate paths alone, so anything still carrying
    // a prefix is a foreign type that merely shares a name — `foreign::Option`
    // is not `Option`, and collapsing it would silently retype the field.
    let is_bare = tp.path.leading_colon.is_none() && tp.path.segments.len() == 1;
    if is_bare {
        if args.is_empty() {
            if let Some(kind) = ScalarKind::from_name(&name) {
                return Ok(TypeKind::Scalar(kind));
            }
            match name.as_str() {
                "String" => return Ok(TypeKind::String),
                "str" => return Ok(TypeKind::Str),
                _ => {}
            }
        }
        // A builtin generic takes TYPE arguments only — a lifetime on one is not a
        // shape this language has — with one exception: `Cow`'s own signature HAS a
        // lifetime, so it is the one builtin where a lifetime argument is expected
        // rather than refused.
        if !has_lifetime_arg || name == "Cow" {
            let mut types: Vec<TypeRef> = args
                .iter()
                .filter_map(|a| match a {
                    GenericArg::Type(t) => Some((**t).clone()),
                    GenericArg::Lifetime(_) => None,
                })
                .collect();
            let arity = |n: usize| {
                if types.len() == n {
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
                    return Ok(TypeKind::Optional(Box::new(types.remove(0))));
                }
                "Vec" => {
                    arity(1)?;
                    return Ok(TypeKind::Vec(Box::new(types.remove(0))));
                }
                "Box" => {
                    arity(1)?;
                    return Ok(TypeKind::Boxed(Box::new(types.remove(0))));
                }
                // The one builtin whose signature has a lifetime, so it is the
                // one whose WHOLE argument list has to be checked: counting type
                // arguments alone accepts `Cow<u8, 'a>` and `Cow<'a, 'b, u8>`,
                // which are not `Cow`s at all, and a model that then kept only
                // the first lifetime could not spell either one back.
                "Cow" => {
                    let [GenericArg::Lifetime(lifetime), GenericArg::Type(inner)] = &args[..]
                    else {
                        return Err(fail(UnsupportedTypeReason::WrongGenericArguments {
                            expected: "Cow<'a, T>",
                        }));
                    };
                    return Ok(TypeKind::Cow {
                        lifetime: lifetime.clone(),
                        inner: inner.clone(),
                    });
                }
                // Reached here it is not directly under a `&mut`, and that is the
                // one position where uninitialized storage means anything —
                // `TypeKind::Uninit` is built by the reference arm alone.
                "MaybeUninit" => return Err(fail(UnsupportedTypeReason::OwnedUninit)),
                "Result" => {
                    arity(2)?;
                    let err = Box::new(types.remove(1));
                    let ok = Box::new(types.remove(0));
                    return Ok(TypeKind::Fallible { ok, err });
                }
                _ => return Ok(named(tp, args)),
            }
        }
    }
    Ok(named(tp, args))
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
///
/// The leading `::` rides along in the name when the source wrote one. It is
/// nothing a destination language acts on — but the name is what
/// [`TypeKind::to_syn`] spells the path back from, and `::a::B` is not `a::B`.
fn named(tp: &syn::TypePath, args: Vec<GenericArg>) -> TypeKind {
    let mut name = String::new();
    if tp.path.leading_colon.is_some() {
        name.push_str("::");
    }
    name.push_str(
        &tp.path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
    );
    TypeKind::Named {
        id: TypeId { name },
        args,
    }
}
