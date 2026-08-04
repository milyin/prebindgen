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
//! [`post_process_item`](super::prebindgen::Prebindgen::post_process_item), and
//! the closure [`RegistryBuilder::convert_with`](super::registry::RegistryBuilder::convert_with)
//! calls. Nothing else. If a helper needs an `&Emit`, that is the helper saying
//! it emits; if threading one to it feels wrong, it is probably deciding
//! something and wants the model instead.
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

    /// The type as the **source spelled it** — what generated Rust must say.
    ///
    /// Not [`TypeKind::to_syn`](super::flat::TypeKind), which reconstructs a
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
