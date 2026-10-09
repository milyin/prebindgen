//! [`Emit`] — rendering captured Rust syntax.
//!
//! The model pairs every element with the syntax it was built from, and
//! generated Rust has to **spell** that syntax: a wrapper's signature says
//! what the source said. *Reading* the same syntax to decide what a type
//! means is what [#211](https://github.com/milyin/prebindgen/issues/211)
//! removed — a decision belongs to `kind`, which cannot disagree with itself
//! the way a spelling can.
//!
//! So the renderings live here, apart from the model's readings: a
//! declaration's delimiters, an enum value's discriminant as written, a
//! guard, a verbatim item. The accessors that hand out a `syn` node itself —
//! a type's node and its stripped form, an element's item, an origin's node,
//! the syntax rebuilt from a kind — stay crate-internal; the `compile_fail`
//! examples on [`Emit`] check that from outside the crate.
//!
//! `Display for TypeRef` renders the **identity**, not the spelling: a message
//! is decision code explaining itself, and delegating to `spell()` would hand
//! the captured tokens back out through `format!`.

use proc_macro2::TokenStream;

use super::{Element, EnumValue, Field, Struct, Type, TypeRef};

/// Re-emit a captured `#[prebindgen]` const as a **path-alias** to its
/// source-of-truth: same attributes (doc comments), visibility, name and
/// type, with the initializer replaced by `<source_module>::<ident>`, so a
/// const whose initializer references source-crate internals still compiles
/// in the generated file. [`Emit::const_alias`] is its only caller.
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

/// Renders captured Rust syntax for code that generates Rust.
///
/// Every method here is a *rendering* — it answers "what did the source
/// write", never "what does this mean". The second question is the model's,
/// and its answers ([`TypeRef::kind`], [`TypeRef::key`], the layer readings)
/// are on the model itself.
///
/// What stays closed from outside the crate — each of these is a route to a
/// `syn` node:
///
/// ```compile_fail
/// # use prebindgen_flat::{Element, flat};
/// fn leak(e: &Element) -> syn::Item { e.as_syn() }
/// ```
///
/// A declared type's item:
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(t: &flat::Type) -> syn::Item { t.as_syn() }
/// ```
///
/// A captured function's own node, through its `Origin`:
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(f: &flat::Function) -> &syn::ItemFn { f.origin.as_syn() }
/// ```
///
/// A type's **node** — the door C5 claimed to have closed and did not:
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(t: &flat::TypeRef) -> &syn::Type { t.as_syn() }
/// ```
///
/// The delimiters a shape was written with — `S { a }` vs `S(a)` vs `S`:
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(s: &flat::Struct) -> proc_macro2::TokenStream {
///     s.spell(Default::default(), &[])
/// }
/// ```
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(v: &flat::EnumValue) -> proc_macro2::TokenStream {
///     v.spell(Default::default(), &[])
/// }
/// ```
///
/// …its stripped form, and the kind's reconstruction:
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(t: &flat::TypeRef) -> syn::Type { t.stripped_syntax() }
/// ```
///
/// ```compile_fail
/// # use prebindgen_flat::flat;
/// fn leak(k: &flat::TypeKind) -> syn::Type { k.to_syn() }
/// ```
///
/// Minting one by naming the struct literal is not available either — the
/// field is private:
///
/// ```compile_fail
/// # use prebindgen_flat::Emit;
/// let forged = Emit { _seal: () };
/// ```
#[derive(Debug)]
pub struct Emit {
    _seal: (),
}

impl Emit {
    /// A renderer.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _seal: () }
    }

    /// A renderer, for tests.
    #[cfg(any(test, feature = "testing"))]
    pub fn for_test() -> Self {
        Self { _seal: () }
    }

    /// The type as the **source spelled it** — what generated Rust must say.
    ///
    /// Not a canonical form rebuilt from the classification to check the
    /// lowering against: this is the crate's own
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
