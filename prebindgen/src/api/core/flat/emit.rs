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
//! the difference has to be enforced somewhere. Enforcing it by *measurement* —
//! counting how many places name a door, and failing the build when the count
//! moves — was tried and retired: a count can be walked around without moving
//! (`spell()` → `parse_quote!` recovers a node while naming no door), and it
//! cannot see an out-of-crate adapter at all.
//!
//! So the difference is a **capability**. Syntax is reachable only
//! through this type, this type cannot be constructed outside `api::core`, and
//! core hands one out only to the callbacks whose job is producing Rust.
//! Adapter code that classifies, plans, names or validates never receives one,
//! and a call to a door from there does not compile — in this crate and in
//! anyone else's.
//!
//! # Where one comes from
//!
//! [`Prebindgen::on_function`](super::super::prebindgen::Prebindgen::on_function) and
//! its four peers, [`prerequisites`](super::super::prebindgen::Prebindgen::prerequisites),
//! [`post_process_item`](super::super::prebindgen::Prebindgen::post_process_item), and
//! the closure [`RegistryBuilder::convert_with`](super::super::registry::RegistryBuilder::convert_with)
//! calls — a converter is generated Rust, since `ConverterImpl::function` is a
//! complete `syn::ItemFn` the adapter writes. Nothing else.
//!
//! If a helper needs an `&Emit`, that is the helper saying it emits; if
//! threading one to it feels wrong, it is probably deciding something and wants
//! the model instead.
//!
//! # What is closed
//!
//! **Every route from the model to captured syntax.** `TypeRef::{as_syn, spell,
//! stripped_syntax}`, `TypeKind::to_syn`, `Element::as_syn`, `Type::as_syn`,
//! `Origin::{as_syn, spell}`, `Flat::enum_item` and the three `spell(head,
//! parts)` shape methods are all `pub(in crate::api::core)`. The
//! `compile_fail` examples on [`Emit`] check each one from outside the crate,
//! which is the way an adapter author meets them.
//!
//! Two things stay public because they are not that:
//!
//! * [`Origin::declared_spelling`](super::Origin::declared_spelling) — an
//!   adapter declaration's `Origin<syn::Type>` holds a type the **build script**
//!   wrote, never captured, which #280 leaves the model no reading for.
//! * [`Field::member`](super::Field::member) and
//!   [`Field::bind`](super::Field::bind) — they read the field's `name`
//!   and `index`, model facts, no syntax.
//!
//! `Display for TypeRef` renders the **identity**, not the spelling: a message
//! is decision code explaining itself and must not need this capability, and
//! delegating to `spell()` would have handed the captured tokens back out
//! through `format!`.
//!
//! # The residual
//!
//! Two things visibility does not do, both accepted.
//!
//! [`Emit::spell`] yields a `TokenStream`, so emission code can re-parse it and
//! take the node apart. That is deliberate — emission is where syntax belongs —
//! and closing it would mean an emission IR for Rust, mirroring the
//! [`kotlin_codegen`] crate, which is a much larger piece of work.
//!
//! And nothing stops a *new* door being added: someone can write `pub fn
//! as_syn2` in `flat` tomorrow. The reason that is tolerable is that such a
//! method has to be added inside `flat` **and** surfaced here before an adapter
//! can reach it — a two-file diff in the one module a reviewer of this
//! subsystem already reads.

use proc_macro2::TokenStream;

use super::{Element, EnumValue, Field, Struct, Type, TypeRef};

/// Re-emit a captured `#[prebindgen]` const as a **path-alias** to its
/// source-of-truth: same attributes (doc comments), visibility, name and
/// type, with the initializer replaced by `<source_module>::<ident>`. Used
/// by `Prebindgen::on_const` implementations so consts whose initializers
/// reference source-crate internals (private helpers, upstream constants)
/// still compile in the generated file.
///
/// Lives here rather than beside the `Prebindgen` trait because it is pure
/// syntax rendering with no pipeline dependency, and [`Emit::const_alias`] is
/// its only caller.
fn const_path_alias(c: &syn::ItemConst, source_module: &syn::Path) -> TokenStream {
    let attrs = &c.attrs;
    let vis = &c.vis;
    let ident = &c.ident;
    let ty = &c.ty;
    quote::quote! {
        #(#attrs)*
        #vis const #ident: #ty = #source_module::#ident;
    }
}

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
/// check the property that matters: what an out-of-crate adapter can reach.
/// Each names a route that used to be open.
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
/// A type's **node** — the door C5 claimed to have closed and did not:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(t: &flat::TypeRef) -> &syn::Type { t.as_syn() }
/// ```
///
/// A declared enum's item, by name:
///
/// ```compile_fail
/// # use prebindgen::core::Flat;
/// fn leak(f: &Flat) -> Option<&syn::ItemEnum> { f.enum_item("E") }
/// ```
///
/// The delimiters a shape was written with — `S { a }` vs `S(a)` vs `S`:
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(s: &flat::Struct) -> proc_macro2::TokenStream {
///     s.spell(Default::default(), &[])
/// }
/// ```
///
/// ```compile_fail
/// # use prebindgen::core::flat;
/// fn leak(v: &flat::EnumValue) -> proc_macro2::TokenStream {
///     v.spell(Default::default(), &[])
/// }
/// ```
///
/// A type's spelling:
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
    /// Not [`TypeKind::to_syn`](super::TypeKind::to_syn), which reconstructs a
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
    /// sites wrote before.
    pub fn spell_ty(&self, ty: &TypeRef) -> syn::Type {
        let toks = ty.spell();
        syn::parse_quote!(#toks)
    }

    /// The type under every transparent wrapper, spelled — `Box<Payload>` →
    /// `Payload`.
    ///
    /// The spelling peer of [`TypeRef::stripped_key`](super::TypeRef::stripped_key),
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
    pub fn verbatim_fn(&self, f: &super::Function) -> TokenStream {
        f.origin.spell()
    }

    /// A captured struct's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_struct(&self, s: &Struct) -> TokenStream {
        s.origin.spell()
    }

    /// A captured sum's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_variant(&self, v: &super::Variant) -> TokenStream {
        v.origin.spell()
    }

    /// A captured fieldless enum's tokens, as written. See [`Self::verbatim_fn`].
    pub fn verbatim_enum(&self, e: &super::Enum) -> TokenStream {
        e.origin.spell()
    }

    /// A constant re-emitted as an alias into `source_module`, so the
    /// initializer is never copied and a const referencing source-crate
    /// internals stays valid in the generated file.
    ///
    /// Takes the element rather than its item because the alias needs four
    /// facts off it and nothing else; handing over the whole `syn::ItemConst`
    /// to read four fields is what an accessor is for.
    pub fn const_alias(&self, c: &super::Constant, source_module: &syn::Path) -> TokenStream {
        const_path_alias(c.origin.as_syn(), source_module)
    }

    /// A constant re-emitted verbatim, for an adapter with no source module.
    pub fn const_verbatim(&self, c: &super::Constant) -> TokenStream {
        c.origin.spell()
    }

    /// A [`Guard`](super::Guard)'s anonymous `const _`, as written.
    pub fn guard(&self, g: &super::Guard) -> syn::ItemConst {
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

    /// A struct, alternative or enum value spelled with **the delimiters the
    /// source wrote** — `S { a: x }`, `S(x)`, `S` — for a pattern or a
    /// constructor alike.
    ///
    /// `B` and `B()` are both payload-free and still spelled differently, which
    /// is why this is a rendering rather than something `kind` could answer.
    ///
    /// Note what is *not* here: [`Field::member`](super::Field::member)
    /// and [`Field::bind`](super::Field::bind) stay ungated, because they
    /// read the field's `name` and `index` — model facts, no captured syntax.
    // `Shaped` is deliberately more private than this method: that is the
    // sealed-trait pattern, and it is what stops the trait itself becoming a
    // door. An out-of-crate consumer can call `shape` on the three elements
    // and cannot implement it for anything else, or name it to route around.
    #[allow(private_bounds)]
    pub fn shape<S: super::spell::Shaped>(
        &self,
        s: &S,
        head: TokenStream,
        parts: &[TokenStream],
    ) -> TokenStream {
        super::spell::fields(s.shape(), head, parts)
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
