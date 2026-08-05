//! [`Emit`] — the capability to render captured Rust syntax.
//!
//! # Why this exists
//!
//! The model pairs every element with the syntax it was built from, and
//! generated Rust has to **spell** that syntax: a converter's signature says
//! what the source said. But *reading* the same syntax to decide what a type
//! means is the thing [#211](https://github.com/milyin/prebindgen/issues/211)
//! removed — a decision belongs to `kind`, which cannot disagree with itself
//! the way a spelling can.
//!
//! Those two are the same capability if the model simply hands syntax out, so
//! for a long time the difference was kept by *counting*: a token census froze
//! how many places named a door, and a moving count failed the build. That
//! worked and had three holes. It could be walked around without moving
//! (`spell()` → `parse_quote!` recovers a node while naming no counted door).
//! It could not see an out-of-crate adapter at all, because it scanned this
//! crate's `src/`. And it cost about two thousand lines to maintain.
//!
//! So the difference is a **capability** instead. Syntax is reachable only
//! through this type, this type cannot be constructed outside `api::core`, and
//! core hands one out only to the callbacks whose job is producing Rust.
//! Adapter code that classifies, plans, names or validates never receives one,
//! and a call to a door from there does not compile — in this crate and in
//! anyone else's.
//!
//! # Where one comes from
//!
//! [`Prebindgen::on_function`](super::prebindgen::Prebindgen::on_function) and
//! its four peers, [`prerequisites`](super::prebindgen::Prebindgen::prerequisites),
//! and [`post_process_item`](super::prebindgen::Prebindgen::post_process_item).
//! Nothing else *yet*: the converter path — `RegistryBuilder::convert_with`'s
//! closure, and the selector chains under it — still receives only
//! `(&Crossing, &Building)`, and threading `&Emit` there is C3's job, together
//! with moving [`TypeRef::spell`](super::flat::TypeRef::spell) behind this type.
//!
//! If a helper needs an `&Emit`, that is the helper saying it emits; if
//! threading one to it feels wrong, it is probably deciding something and wants
//! the model instead.
//!
//! # What is closed, and what is not
//!
//! **Closed as of this stage: every route to a captured *item*.**
//! `Element::as_syn`, `Type::as_syn` and `Origin::as_syn` are
//! `pub(in crate::api::core)`, and so is `Origin::spell` — whose tokens
//! re-parse to the item, which is the same door under another name. The
//! `compile_fail` examples on [`Emit`] check each of those from outside the
//! crate, which is where a doctest runs and where the census could never look.
//!
//! **Closed as of C5: type spellings too.** `TypeRef::spell`,
//! `TypeRef::stripped_syntax` and `TypeKind::to_syn` are
//! `pub(in crate::api::core)`. There is no route from the model to captured
//! syntax outside this type, and the `compile_fail` examples check every one
//! from outside the crate.
//!
//! `Origin::declared_spelling` stays public: an adapter declaration's
//! `Origin<syn::Type>` holds a type the **build script** wrote, which was never
//! captured and which #280 leaves the model no reading for.
//!
//! # What the census did that this does not
//!
//! `escape_surface_is_closed` read the model's own surface and failed if a NEW
//! public method handed out a `syn` node under a name its list did not count —
//! four of the five doors were found that way. Visibility does not give that:
//! someone can add `pub fn as_syn2` to `flat` tomorrow.
//!
//! It is a smaller risk than it was. Such a method has to be added inside
//! `flat` **and** surfaced here to be reachable from an adapter, which is a
//! two-file diff in the one module a reviewer of this subsystem reads — where
//! before, a new door could be added anywhere and only a count would notice.
//! Recorded here rather than dropped silently, because a check that is retired
//! deserves to say what it was for.
//!
//! # The residual
//!
//! [`Emit::spell`] yields a `TokenStream`, so emission code can re-parse it and
//! take the node apart. That is deliberate — emission is where syntax belongs —
//! and closing it would mean an emission IR for Rust, mirroring
//! [`api::gen::kotlin`](crate::api::gen), which is a much larger piece of work.
//! It replaces the census's four blind spots, three of which the capability
//! closes outright: ident-name classification, helper delegation and the
//! unwatched syn enums were all reachable only *through* a door.

use proc_macro2::TokenStream;

use super::flat::{Element, EnumValue, Field, Struct, Type, TypeRef};

/// The capability to render captured Rust syntax.
///
/// Unforgeable outside `api::core`: the field is private and there is no public
/// constructor, so the only way to hold one is to have been handed one. See the
/// [module docs](self) for where that happens and why.
///
/// Every method here is a *rendering* — it answers "what did the source write",
/// never "what does this mean". The second question is the model's, and its
/// answers ([`TypeRef::kind`], [`TypeRef::key`], the layer readings) need no
/// capability precisely because they cannot be misused into re-deriving a
/// classification.
///
/// # The seal, as compiled assertions
///
/// A doctest builds as its **own crate** against the published API, so these
/// check the property the token census structurally could not: what an
/// out-of-crate adapter can reach. Each names a route that used to be open.
///
/// An element's item (`E0624` — the method is private):
///
/// ```compile_fail
/// # use prebindgen::core::{Element, flat};
/// fn leak(e: &Element) -> syn::Item { e.as_syn() }
/// ```
///
/// A declared type's item:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(t: &flat::Type) -> syn::Item { t.as_syn() }
/// ```
///
/// A captured function's own node, through its `Origin`:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(f: &flat::Function) -> &syn::ItemFn { f.origin.as_syn() }
/// ```
///
/// …and its tokens, which re-parse to the same item — the door under another
/// name, and the one a reviewer found still open when this type was introduced:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(f: &flat::Function) -> proc_macro2::TokenStream { f.origin.spell() }
/// ```
///
/// A type's spelling, sealed as of C5 — the last route, and the one the census
/// could only ever *count*:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(t: &flat::TypeRef) -> proc_macro2::TokenStream { t.spell() }
/// ```
///
/// …its stripped form, and the kind's reconstruction:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(t: &flat::TypeRef) -> syn::Type { t.stripped_syntax() }
/// ```
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(k: &flat::TypeKind) -> syn::Type { k.to_syn() }
/// ```
///
/// Minting one is not available either — the field is private and `new` is
/// `pub(in crate::api::core)`:
///
/// ```compile_fail
/// # use prebindgen::core::Emit;
/// let forged = Emit { _seal: () };
/// ```
#[derive(Debug)]
pub struct Emit {
    _seal: (),
}

impl Emit {
    /// Mint one. `pub(in crate::api::core)` is the whole enforcement mechanism:
    /// the hand-out sites are exactly the callers of this.
    pub(in crate::api::core) fn new() -> Self {
        Self { _seal: () }
    }

    /// A capability for a test that drives an emission helper directly.
    ///
    /// `#[cfg(test)]`, so it does not exist in a built crate — production code
    /// still cannot mint one, and the `compile_fail` examples above still prove
    /// the out-of-crate seal, since a doctest compiles against the built crate
    /// where this is absent.
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self { _seal: () }
    }

    /// The type as the **source spelled it** — what generated Rust must say.
    ///
    /// Not [`TypeKind::to_syn`](super::flat::TypeKind::to_syn), which reconstructs a
    /// canonical form to check the lowering against: this is the crate's own
    /// tokens, so a generated signature names the type the way the source crate
    /// does and compiles in its scope.
    pub fn spell(&self, ty: &TypeRef) -> TokenStream {
        ty.spell()
    }

    /// [`Self::spell`] as a node, for an emitter that builds a `syn::Type`
    /// around it (`*mut #ty`, `&[#elem]`).
    ///
    /// A convenience over `parse_quote!(#spelled)`, which is what the call
    /// sites wrote before — and which the census could not see, being the one
    /// route to a node that named no door.
    pub fn spell_ty(&self, ty: &TypeRef) -> syn::Type {
        let toks = ty.spell();
        syn::parse_quote!(#toks)
    }

    /// The type under every transparent wrapper, spelled — `Box<Payload>` →
    /// `Payload`.
    ///
    /// The spelling peer of [`TypeRef::stripped_key`](super::flat::TypeRef::stripped_key),
    /// for an emitter that must name what a declaration is *about* rather than
    /// what the use site wrote.
    pub fn spell_stripped(&self, ty: &TypeRef) -> syn::Type {
        ty.stripped_syntax()
    }

    /// A captured item, verbatim — attributes, visibility and body included.
    ///
    /// The legitimate reason to reach for an item at all: an emitter re-stating
    /// one as written. Reading a *fact* off an item is a missing accessor, and
    /// the model is where it belongs.
    pub fn item(&self, e: &Element) -> syn::Item {
        e.as_syn()
    }

    /// A declared type's item, verbatim. The [`Type`] peer of [`Self::item`].
    pub fn type_item(&self, t: &Type) -> syn::Item {
        t.as_syn()
    }

    /// A captured function's tokens, as written.
    ///
    /// One of four per-shape peers of [`Self::item`], for the callback that
    /// already holds the specific element rather than an [`Element`]. An
    /// adapter that re-emits its input unchanged is the whole use — both
    /// in-tree adapters build wrappers instead, so this is what a
    /// pass-through generator would call.
    pub fn verbatim_fn(&self, f: &super::flat::Function) -> TokenStream {
        f.origin.spell()
    }

    /// A captured struct's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_struct(&self, s: &Struct) -> TokenStream {
        s.origin.spell()
    }

    /// A captured sum's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_variant(&self, v: &super::flat::Variant) -> TokenStream {
        v.origin.spell()
    }

    /// A captured fieldless enum's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_enum(&self, e: &super::flat::Enum) -> TokenStream {
        e.origin.spell()
    }

    /// A constant re-emitted as an alias into `source_module`, so the
    /// initializer is never copied and a const referencing source-crate
    /// internals stays valid in the generated file.
    ///
    /// Takes the element rather than its item because the alias needs four
    /// facts off it and nothing else; handing over the whole `syn::ItemConst`
    /// to read four fields is what an accessor is for.
    pub fn const_alias(&self, c: &super::flat::Constant, source_module: &syn::Path) -> TokenStream {
        super::prebindgen::const_path_alias(c.origin.as_syn(), source_module)
    }

    /// A constant re-emitted verbatim, for an adapter with no source module.
    pub fn const_verbatim(&self, c: &super::flat::Constant) -> TokenStream {
        c.origin.spell()
    }

    /// A [`Guard`](super::flat::Guard)'s anonymous `const _`, as written.
    pub fn guard(&self, g: &super::flat::Guard) -> syn::ItemConst {
        g.origin.as_syn().clone()
    }

    /// An enum value's discriminant **as written** — `= 0x07` stays `0x07`.
    ///
    /// `None` when the source wrote none. Distinct from
    /// [`EnumValue::discriminant`], which is the *evaluated* number and this
    /// shape's identity: a C mirror re-states the spelling, a destination
    /// language that transmits a value wants the number.
    pub fn discriminant(&self, v: &EnumValue) -> Option<TokenStream> {
        v.origin
            .as_syn()
            .discriminant
            .as_ref()
            .map(|(_, expr)| quote::quote!(#expr))
    }

    /// A struct spelled with the delimiters the source wrote — `S { a: x }`,
    /// `S(x)`, `S` — for a pattern or a constructor alike.
    pub fn struct_shape(
        &self,
        s: &Struct,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        s.spell(head, parts)
    }

    /// An alternative spelled with its own delimiters. The [`Alternative`](super::flat::Alternative)
    /// peer of [`Self::struct_shape`].
    pub fn alternative_shape(
        &self,
        a: &super::flat::Alternative,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        a.spell(head, parts)
    }

    /// How a field is addressed in a pattern or an initializer — by name when
    /// it has one, else by position.
    pub fn member(&self, f: &Field) -> syn::Member {
        f.member()
    }

    /// A field bound to `bind`, shaped for whichever address it uses:
    /// `id: __f0` for a named field, `__f0` for a positional one.
    pub fn bind(&self, f: &Field, bind: &impl quote::ToTokens) -> TokenStream {
        f.bind(bind)
    }
}
