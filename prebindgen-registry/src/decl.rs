//! Language-neutral declaration vocabulary: the decl objects and constructor
//! macros a build script uses to describe boundary expansion (`expand_param!`
//! / `expand_return!`), free functions (`fun!`), and canonical single-value
//! conversions (`convert!`). Moved down from the JNI adapter (formerly
//! `api/lang/jnigen`, now the separate `prebindgen-jni` crate) because none of
//! it is Kotlin/JNI-specific — it only references [`TypeKey`],
//! [`Origin<syn::Type>`], and plain `syn` types. `prebindgen-jni`'s own
//! `jni/decl.rs` keeps the genuinely Kotlin-specific declarations
//! (`ptr_class!`, `enum_class!`, `sealed_class!`, `data_class!`, `constant!`,
//! `package!`) and re-exports these types from here, so the public
//! `prebindgen_flat::*` surface is unaffected by the split.

use prebindgen_flat::flat::{Origin, TypeKey};
use quote::ToTokens;

use crate::recipe::Direction;

/// The origin of a type a **build script** wrote.
///
/// Real tokens, and deliberately no source position: `SourceLocation::default()`
/// is the sanctioned placeless location for exactly this — a signature or type a
/// build script authored was never in a captured file, and `has_position` already
/// gates what a diagnostic prints for one.
pub fn declared_origin(ty: syn::Type) -> Origin<syn::Type> {
    Origin::new(ty, std::rc::Rc::new(prebindgen::SourceLocation::default()))
}

// ──────────────────────────────────────────────────────────────────────
// Shared local accumulators (replayed into an adapter's rows/`Deconstructors`
// by the accept logic in `builder.rs` once a decl is handed to `Declarations`)
// ──────────────────────────────────────────────────────────────────────

/// One arm of an `expand_param!` `.variant*` list (type-level or per-fn).
#[derive(Clone)]
pub enum LocalVariant {
    /// Build via this declared constructor member / constructor fn.
    Ctor(syn::Ident),
    /// Accept an already-built value directly.
    SelfIdentity,
}

/// One arm of an `expand_return!` `.field*` list (type-level or per-fn). The name is stored raw (`None` = derive at replay time: for a
/// class-level field, the class member's target-language name if the accessor is a
/// declared member, else `snake_to_camel`; for a per-fn field,
/// `snake_to_camel`).
// large_enum_variant: a handful of fields exist per binding, held while
// declarations replay — boxing the syn payloads would only complicate the
// arms (same trade-off as `ConvertSourceKind`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum LocalField {
    /// Include the named accessor's value as a leaf/field, with an optional
    /// explicit name override.
    Named(syn::Ident, Option<String>),
    /// Include the handle itself as a field.
    SelfField,
    /// Include a **custom, locally-defined** accessor's value: any fn the
    /// binding crate defines, declared with the one binding-local vocabulary
    /// (`fun!(crate::f).sig(sig!((v: &Self) -> Ret))`) — no `#[prebindgen]`
    /// item behind it, so the full signature (receiver explicit) is stated.
    /// `name_override` follows the uniform field-name precedence.
    Local {
        path: syn::Path,
        sig: syn::Signature,
        name_override: Option<String>,
    },
    /// Include **every field of the type's value form** — the struct returned
    /// by the named accessor — each as its own field. Expands to the same
    /// records the fields would produce if named one by one; see
    /// [`ExpandReturnDecl::fields`].
    Fields(FieldsDecl),
}

/// Build a [`FunctionDecl`] from a bare function ident or a path:
///
/// * `fun!(foo)` — a `#[prebindgen]` fn; its signature is read from the
///   registry.
/// * `fun!(crate::foo)` — a **binding-local** fn: any fn the binding crate
///   defines, exported through the same machinery as a `#[prebindgen]` one.
///   A path carries no signature to read, so chain
///   [`.sig(sig!(…))`](crate::FunctionDecl::sig). The generated file
///   calls it by the declared path (it compiles inside the binding crate,
///   so `crate::`-rooted paths resolve).
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new($crate::ident!($name))
    };
    ($path:path) => {
        $crate::FunctionDecl::new_local($crate::__macro_support::parse_path(stringify!($path)))
    };
}

/// State a binding-local fn's exact Rust signature, with **named parameters**
/// (they become the foreign-side parameter names): `sig!((s: &Summary,
/// verbose: bool) -> String)`; the `-> Ret` tail is optional (unit). The
/// signature argument of [`FunctionDecl::sig`](crate::FunctionDecl::sig)
/// for a path-built [`fun!`](crate::fun).
#[macro_export]
macro_rules! sig {
    (($($params:tt)*) $(-> $ret:ty)?) => {
        $crate::__macro_support::parse_signature(stringify!(($($params)*) $(-> $ret)?))
    };
}

/// Build a `syn::Type` from a bare Rust type token: `ty!(i32)`. The type
/// argument of decl methods like [`ConvertSourceDecl::from_type`] — always
/// yields the concrete `syn::Type`, so no inference context is needed (see
/// [`ident!`](crate::ident) for the E0283 background).
#[macro_export]
macro_rules! ty {
    ($t:ty) => {
        $crate::__macro_support::parse_type(stringify!($t))
    };
}

/// Build a `syn::Path` from a bare path token: `path!(crate::conv::f)`. The
/// callable argument of [`FunctionDecl::new_local`] and of an adapter's
/// `ConstDecl::with`.
#[macro_export]
macro_rules! path {
    ($p:path) => {
        $crate::__macro_support::parse_path(stringify!($p))
    };
}

/// Build a `syn::Expr` from an expression token: `expr!(format!("{A}:{B}"))`.
/// The initializer argument of an adapter's `ConstDecl::expr` — allowed only
/// for constants, where the expression binds no arguments.
#[macro_export]
macro_rules! expr {
    ($e:expr) => {
        $crate::__macro_support::parse_expr(stringify!($e))
    };
}

/// Build a [`ConvertDecl`] directly from a bare Rust type:
/// `convert!(Millis)` is `ConvertDecl::new(<Millis as syn::Type>)`.
/// See `ptr_class!` for the parsing mechanics.
#[macro_export]
macro_rules! convert {
    ($t:ty) => {
        $crate::ConvertDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`ExpandParamDecl`] directly from a bare Rust type:
/// `expand_param!(KeyExpr)` is `ExpandParamDecl::new(<KeyExpr as syn::Type>)`.
/// See `ptr_class!` for the parsing mechanics.
#[macro_export]
macro_rules! expand_param {
    ($t:ty) => {
        $crate::ExpandParamDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`ConvertSourceDecl`](crate::ConvertSourceDecl) for an
/// **input** conversion via `core::convert`: `.input(from!(i32))` requires
/// `i32: Into<T>`. Chain `with` to
/// use a binding-local callable instead of the trait.
#[macro_export]
macro_rules! from {
    ($t:ty) => {
        $crate::ConvertSourceDecl::from_type($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Fallible twin of [`from!`]: `.input(try_from!(i32))` requires
/// `i32: TryInto<T>`; an `Err` routes to the caller's error handler. With
/// `with`, the callable returns
/// `Result` and must state its error type via
/// `error`.
#[macro_export]
macro_rules! try_from {
    ($t:ty) => {
        $crate::ConvertSourceDecl::try_from_type($crate::__macro_support::parse_type(stringify!(
            $t
        )))
    };
}

/// Build a [`ConvertSourceDecl`](crate::ConvertSourceDecl) for an
/// **output** conversion via `core::convert`: `.output(into!(i32))` requires
/// `T: Into<i32>`. Chain `with` to
/// use a binding-local callable instead of the trait.
#[macro_export]
macro_rules! into {
    ($t:ty) => {
        $crate::ConvertSourceDecl::into_type($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Fallible twin of [`into!`]: `.output(try_into!(i32))` requires
/// `T: TryInto<i32>`; an `Err` routes to the caller's error handler. With
/// `with`, the callable returns
/// `Result` and must state its error type via
/// `error`.
#[macro_export]
macro_rules! try_into {
    ($t:ty) => {
        $crate::ConvertSourceDecl::try_into_type($crate::__macro_support::parse_type(stringify!(
            $t
        )))
    };
}

/// Build a [`ExpandReturnDecl`] directly from a bare Rust type:
/// `expand_return!(Sample)` is `ExpandReturnDecl::new(<Sample as syn::Type>)`.
/// See `ptr_class!` for the parsing mechanics.
#[macro_export]
macro_rules! expand_return {
    ($t:ty) => {
        $crate::ExpandReturnDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`FieldsDecl`] from the ident of a **value-form accessor** —
/// `fields!(sample_to_struct)` is `FieldsDecl::new(prebindgen_registry::ident!(sample_to_struct))`.
/// The argument of [`ExpandReturnDecl::fields`](crate::ExpandReturnDecl::fields).
#[macro_export]
macro_rules! fields {
    ($name:ident) => {
        $crate::FieldsDecl::new($crate::ident!($name))
    };
}

// ──────────────────────────────────────────────────────────────────────
// Boundary decls — how a declared type crosses the FFI boundary by default
// ──────────────────────────────────────────────────────────────────────

/// Declares a type's **default input boundary**: how a parameter of this type
/// may be supplied, as a list of *variants* — "built from this constructor's
/// ingredients, OR that one's, OR passed as an existing handle". Applies to
/// every function with a parameter of the type; a single function opts out or
/// narrows via [`FunctionDecl::expand_param`].
///
/// Build one with [`expand_param!`](crate::expand_param), add arms with
/// [`variant`](Self::variant) / [`variant_self`](Self::variant_self), and hand
/// it to the adapter's `expand` declaration.
///
/// **Generated shape** — at the wire tier this is a selector dispatch: with
/// more than one arm the parameter crosses as a selector `Int` plus one
/// nullable slot per arm (`keyExprSel: Int, keyExpr0: String?,
/// keyExpr1: KeyExpr?`), and the raw call site passes `(0, "key", null)`-style
/// tuples. That selector form is always emitted; the wrapper's generated KDoc
/// shape-notes document the exact slots per function.
///
/// **Splittability (checked)** — a multi-variant declaration must be
/// *splittable*: its arms must surface as **distinct JVM signatures**, so a
/// function can request idiomatic typed **overloads** on top of the selector
/// form (`f(key: String, …)` / `f(key: KeyExpr, …)`) via
/// [`FunctionDecl::split_on_param`](crate::fun). This is verified up front —
/// two arms with the same erased parameter types are a hard build error.
/// [`.no_split()`](Self::no_split) suppresses that check for a variant set that
/// will only ever be used as the selector form. The type-level declaration
/// itself emits no overloads; emission is per-function via `.split_on_param`.
/// The selector form always stays public, so consumers can also add their own
/// same-named overloads by hand.
///
/// The type does **not** have to be declared in any package. A boundary decl
/// on an undeclared type makes it **rust-side-only**: the value is always
/// built from its ingredients at the boundary and never materializes in
/// the target language — no class, no handle, nothing to close. The one
/// restriction is structural: [`variant_self`](Self::variant_self) hard-errors
/// for such a type, since there is no target-language object to pass.
///
/// ```
/// // A KeyExpr param accepts EITHER a String (built via keyexpr_new_try_from)
/// // OR an existing KeyExpr handle:
/// let _ = prebindgen_registry::expand_param!(KeyExpr)
///     .variant(prebindgen_registry::fun!(keyexpr_new_try_from))
///     .variant_self();
/// ```
#[derive(Clone)]
pub struct ExpandParamDecl {
    key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    rust_type: Origin<syn::Type>,
    variants: Vec<LocalVariant>,
    /// `.no_split()` — suppress the proactive splittability check for this
    /// variant set (it will only ever be used as the selector form). See
    /// [`Self::no_split`].
    no_split: bool,
}

impl ExpandParamDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            variants: Vec::new(),
            no_split: false,
        }
    }

    /// The type identity this declaration is registered under.
    pub fn key(&self) -> &TypeKey {
        &self.key
    }

    /// The type this declaration was written with, as originally parsed.
    pub fn rust_type(&self) -> &Origin<syn::Type> {
        &self.rust_type
    }

    /// The declared build-from / existing-handle arms, in declaration order.
    /// `pub(crate)`, not `pub` — [`LocalVariant`] itself is `pub(crate)`
    /// (a public fn cannot return a private type).
    pub fn variants(&self) -> &[LocalVariant] {
        &self.variants
    }

    /// Whether `.no_split()` was declared (suppresses the splittability
    /// check). Named `is_no_split` rather than `no_split` — that name is
    /// already the builder method that sets the flag ([`Self::no_split`]).
    pub fn is_no_split(&self) -> bool {
        self.no_split
    }

    /// Add a **build-from** arm: parameters of this type also carry the
    /// named `#[prebindgen]` constructor's inputs on the wire, and Rust
    /// builds the value in the same call. E.g. `keyexpr_new_try_from(&str)`
    /// gives every function taking a `KeyExpr` a String-carrying arm —
    /// as a selector + nullable slot at the wire tier (see the type-level
    /// docs for the exact generated shape), not as a target-language overload.
    ///
    /// A variant arm only *names* the constructor: no target-language surface of its
    /// own, so a decorated `fun!` (`.name()` / expand overrides) is a hard
    /// error rather than a silent discard.
    pub fn variant(mut self, ctor: FunctionDecl) -> Self {
        assert!(
            ctor.name_override.is_none()
                && ctor.param_expands.is_empty()
                && ctor.return_expand.is_none(),
            "expand_param!({}).variant(fun!({})): a variant arm only names the \
             `#[prebindgen]` constructor — .name()/expand overrides don't apply",
            self.key.as_str(),
            ctor.rust_ident
        );
        assert!(
            ctor.local.is_none(),
            "expand_param!({}).variant(fun!(…::{f})): a variant arm only NAMES a fn — \
             declare the binding-local fn via .fun/.method/.constructor/convert! first, \
             then reference it here by ident: fun!({f})",
            self.key.as_str(),
            f = ctor.rust_ident
        );
        self.variants.push(LocalVariant::Ctor(ctor.rust_ident));
        self
    }

    /// Add the **existing-handle** arm: also accept an already-built value.
    /// On its own this is simply the default (a bare handle), so declaring it
    /// alone changes nothing; it earns its place next to build variants.
    pub fn variant_self(mut self) -> Self {
        self.variants.push(LocalVariant::SelfIdentity);
        self
    }

    /// **Suppress the splittability check.** A multi-variant expansion is
    /// verified up front to be *splittable* (its arms surface as distinct JVM
    /// signatures) so that [`FunctionDecl::split_on_param`](crate::fun) can emit
    /// idiomatic typed overloads. `.no_split()` opts this variant set out of
    /// that check — declare it when two arms genuinely share a JVM signature and
    /// you only ever want the selector form (a function that then tries to
    /// `.split_on_param` such a parameter gets the concrete ambiguity error).
    ///
    /// A no-op on a single-variant declaration (nothing to check).
    pub fn no_split(mut self) -> Self {
        self.no_split = true;
        self
    }
}

/// Declares a type's **default output boundary**: wherever the type is
/// returned or handed to a callback, it is decomposed into this set of
/// *fields*, all delivered in one FFI crossing — instead of an opaque handle
/// the caller must then query field by field with more JNI calls. Applies to
/// every function returning the type; a single function opts out or replaces
/// the set via [`FunctionDecl::expand_return`].
///
/// Build one with [`expand_return!`](crate::expand_return), add fields with
/// [`field`](Self::field) / [`field_self`](Self::field_self), and hand it to
/// the adapter's `expand` declaration.
///
/// The type does **not** have to be declared in any package. A boundary decl
/// on an undeclared type makes it **rust-side-only**: every returned /
/// callback-delivered / `Result`-error value of it is decomposed into these
/// fields and the value itself never reaches the target language. This is the
/// natural shape for an error type consumed by an error channel — no dead
/// class is emitted. Restrictions for such a type:
/// [`field_self`](Self::field_self) hard-errors (there is no target-language object to
/// deliver), and field names cannot inherit from class members (there are
/// none) — use `.name(...)` on each field or accept the camel-cased default.
///
/// ```
/// // A returned Sample crosses as { payload, kind } in one call:
/// let _ = prebindgen_registry::expand_return!(Sample)
///     .field(prebindgen_registry::fun!(sample_get_payload))
///     .field(prebindgen_registry::fun!(sample_get_kind));
/// ```

#[derive(Clone)]
pub struct ExpandReturnDecl {
    key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    rust_type: Origin<syn::Type>,
    fields: Vec<LocalField>,
}

impl ExpandReturnDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            fields: Vec::new(),
        }
    }

    /// The type identity this declaration is registered under.
    pub fn key(&self) -> &TypeKey {
        &self.key
    }

    /// The type this declaration was written with, as originally parsed.
    pub fn rust_type(&self) -> &Origin<syn::Type> {
        &self.rust_type
    }

    /// The declared field records, in declaration order. Named `field_list`
    /// rather than `fields` — that name is already the builder method that
    /// appends a value-form ([`Self::fields`]). `pub(crate)`, not `pub` —
    /// [`LocalField`] itself is `pub(crate)` (a public fn cannot return a
    /// private type).
    pub fn field_list(&self) -> &[LocalField] {
        &self.fields
    }

    /// Add one field — a reader whose value crosses as this leaf:
    ///
    /// * `fun!(f)` — a `#[prebindgen]` reader (`f(&Self) -> Field`), its
    ///   signature read from the registry.
    /// * `fun!(crate::f).sig(sig!((v: &Self) -> Field))` — a **custom,
    ///   locally-defined** reader: any fn the binding crate defines, its
    ///   signature stated (the receiver explicit — it must take `&Self`).
    ///   One use among many: conditional delivery, an `Option<&Self>` return
    ///   becoming a nullable handle leaf that is null when the binding-side
    ///   predicate declines.
    ///
    /// The target-language field name is uniform for both: an explicit
    /// `.name(...)` on the `fun!`; else the name of the class member if the same
    /// fn is also declared as a method on this type's class (so a getter
    /// that is both a method and a field is named once); else the
    /// camel-cased fn ident (a path's LAST segment).
    ///
    /// Only the accessor's name is used here: expand overrides on the `fun!`
    /// are a hard error rather than a silent discard (the field's own
    /// decomposition comes from ITS type's boundary decl, not from the
    /// accessor).
    pub fn field(mut self, accessor: FunctionDecl) -> Self {
        self.reject_beside_consuming("field(..)");
        assert!(
            accessor.param_expands.is_empty() && accessor.return_expand.is_none(),
            "expand_return!({}).field(fun!({})): expand overrides don't apply to a \
             field accessor — only .name() is honored",
            self.key.as_str(),
            accessor.rust_ident
        );
        self.fields.push(match accessor.local {
            None => LocalField::Named(accessor.rust_ident, accessor.name_override),
            Some((path, sig)) => {
                let Some(sig) = sig else {
                    panic!(
                        "expand_return!({}).field(fun!({p})): a binding-local field states \
                         its accessor's signature — chain .sig(sig!((v: &{k}) -> Ret))",
                        self.key.as_str(),
                        p = quote::quote!(#path),
                        k = self.key.as_str()
                    );
                };
                LocalField::Local {
                    path,
                    sig,
                    name_override: accessor.name_override,
                }
            }
        });
        self
    }

    /// Include the **handle itself** among the fields, so the consumer gets a
    /// live, closeable object in addition to the read-out values (e.g. a
    /// `Query` delivered with its fields *and* the handle it needs to reply).
    /// Declare it **last**, after any field that decomposes a nested handle,
    /// so the generated Rust moves the value only after those borrows.
    pub fn field_self(mut self) -> Self {
        self.reject_beside_consuming("field_self()");
        self.fields.push(LocalField::SelfField);
        self
    }

    /// The one rule [`Self::fields_self_into`] adds: it hands the value **itself**
    /// over, so nothing else in the decl can still read it.
    fn reject_beside_consuming(&self, what: &str) {
        if let Some(f) = self.fields.iter().find_map(|f| match f {
            LocalField::Fields(d) if d.consuming => Some(&d.func),
            _ => None,
        }) {
            panic!(
                "expand_return!({k}).fields_self_into(fields!({f})).{what}: `.fields_self_into(..)` hands \
                 the value ITSELF over as its fields, so nothing else can read it afterwards — \
                 it must be the decl's only record. Use `.fields(fields!(..))` with the \
                 borrowing form of the accessor if you need both.",
                k = self.key.as_str(),
                f = f,
            );
        }
    }

    /// Take the fields from the type's **value form** — a `#[prebindgen]`
    /// accessor returning "this type's own accessors gathered into one struct"
    /// — instead of restating them.
    ///
    /// `.fields(fields!(f))` is exactly `.field(...)` applied to each field of
    /// that struct, so it has the same configurability (per-field overrides and
    /// renames live on the [`FieldsDecl`]) and, crucially, the same
    /// decomposition rule: **each field crosses by its own type's default
    /// output boundary**. A field whose type has its own `expand_return!` is
    /// decomposed by it (a `KeyExpr` field still crosses as its string, not as
    /// a handle); a declared `data_class!` field expands into its fields; a
    /// field behind `Option` / `Vec` stays one leaf. So swapping a hand-written
    /// field list for `.fields(...)` keeps the boundary shape it already had —
    /// what changes is that the list can no longer drift from the struct.
    ///
    /// ```
    /// // Instead of restating SampleStruct's fields one by one:
    /// let _ = prebindgen_registry::expand_return!(Sample)
    ///     .fields(prebindgen_registry::fields!(sample_to_struct));
    /// ```
    ///
    /// The accessor **borrows** its receiver (`f(v: &Self) -> SelfStruct`):
    /// the struct is built from a borrow, so each field is cloned into it and
    /// the leaves clone again out of it, and the value survives. It therefore
    /// mixes freely — `.fields(...).field_self()` delivers the value form's
    /// fields *and* the live handle. At most one value form per decl.
    ///
    /// Where the value is delivered **owned** — a callback argument
    /// (`impl Fn(Sample)`), an owned return — and nothing else needs it, use
    /// [`fields_self_into`](Self::fields_self_into) instead: those clones are being paid
    /// on a value that is about to be dropped.
    pub fn fields(mut self, decl: FieldsDecl) -> Self {
        self.reject_beside_consuming("fields(..)");
        self.reject_second_value_form(&decl);
        self.fields.push(LocalField::Fields(decl));
        self
    }

    /// Like [`fields`](Self::fields), but the accessor **consumes** its
    /// receiver (`f(v: Self) -> SelfStruct`): the value is moved in and each
    /// field is moved *out* into its leaf. No clones at all.
    ///
    /// This is the same decision [`field_self`](Self::field_self) makes, one
    /// step further: `.field_self()` hands the value over whole,
    /// `.fields_self_into(...)` hands *the value itself* over as its parts, and
    /// `.fields(...)` hands over a copy of its parts. Use it wherever the value
    /// arrives owned and is not needed afterwards — the hot receive path this
    /// whole declarator exists to make cheap.
    ///
    /// ```
    /// let _ = prebindgen_registry::expand_return!(Sample)
    ///     .fields_self_into(prebindgen_registry::fields!(sample_into_struct));
    /// ```
    ///
    /// Because it gives the value away it must be the decl's **only** record —
    /// a `.field_self()` or a sibling `.field(...)` would read a value that is
    /// gone — which is a declaration-time panic either way round. It may still
    /// be reached through *another* value form: the parent's field is handed to
    /// it by move, since a hoisted value form is an owned struct and its fields
    /// are disjoint.
    ///
    /// The declarator and the accessor's signature must agree; naming a
    /// `&Self` accessor here (or a by-value one on [`fields`](Self::fields)) is
    /// an error, so the declared intent cannot drift from the function it
    /// names. At a **borrowed** delivery position there is no value to give up,
    /// so the emitter clones once up front and consumes the clone — the same
    /// cost the borrowing form would have paid, which keeps one declaration
    /// usable by both owned and `&T` returns of the type.
    pub fn fields_self_into(mut self, decl: FieldsDecl) -> Self {
        self.reject_second_value_form(&decl);
        assert!(
            self.fields.is_empty(),
            "expand_return!({k}).fields_self_into(fields!({f})): `.fields_self_into(..)` hands the value \
             ITSELF over as its fields, so it must be the decl's only record — the records \
             already declared would read a value that is gone. Use `.fields(fields!(..))` with \
             the borrowing form of the accessor if you need both.",
            k = self.key.as_str(),
            f = decl.func,
        );
        self.fields.push(LocalField::Fields(decl.consuming()));
        self
    }

    fn reject_second_value_form(&self, decl: &FieldsDecl) {
        assert!(
            !self
                .fields
                .iter()
                .any(|f| matches!(f, LocalField::Fields(_))),
            "expand_return!({}): the decl already expands a value form (fields!({})) — \
             one value form states the whole field set",
            self.key.as_str(),
            decl.func
        );
    }
}

/// A **value-form expansion**: the accessor whose returned struct supplies the
/// fields, plus the per-field adjustments. Built with
/// [`fields!`](crate::fields) and handed to
/// [`ExpandReturnDecl::fields`].
///
/// Both adjusters key on the **Rust struct field name**, mirroring
/// [`FunctionDecl::expand_param`]'s Rust-parameter-name key: an unknown field
/// name or a repeated one is a hard error, so a field renamed upstream is
/// caught rather than silently ignored.
#[derive(Clone)]
pub struct FieldsDecl {
    func: syn::Ident,
    overrides: Vec<(String, ExpandReturnDecl)>,
    names: Vec<(String, String)>,
    /// Set by [`ExpandReturnDecl::fields_self_into`] — the accessor consumes its
    /// receiver. Declared rather than read off the signature, because giving
    /// the value away is a boundary decision; the two are cross-checked when
    /// the records are resolved.
    consuming: bool,
}

impl FieldsDecl {
    pub fn new(func: syn::Ident) -> Self {
        Self {
            func,
            overrides: Vec::new(),
            names: Vec::new(),
            consuming: false,
        }
    }

    pub(crate) fn consuming(mut self) -> Self {
        self.consuming = true;
        self
    }

    /// The value-form accessor's ident, as declared with [`fields!`](crate::fields).
    pub fn func(&self) -> &syn::Ident {
        &self.func
    }

    /// The per-field decomposition overrides, in declaration order.
    pub fn overrides(&self) -> &[(String, ExpandReturnDecl)] {
        &self.overrides
    }

    /// The per-field name overrides, in declaration order.
    pub fn names(&self) -> &[(String, String)] {
        &self.names
    }

    /// Whether the accessor consumes its receiver (set by
    /// [`ExpandReturnDecl::fields_self_into`]). Named `is_consuming` rather
    /// than `consuming` — that name is already the crate-internal builder
    /// method that sets the flag.
    pub fn is_consuming(&self) -> bool {
        self.consuming
    }

    /// Replace **one** field's decomposition, with the same
    /// [`ExpandReturnDecl`] a type-level default uses — so the complete-set
    /// rule applies here too: the decl states that field's entire leaf set.
    /// Use it where the field's type default is not what this boundary wants
    /// (a lone `.field_self()` keeps the raw handle instead of decomposing it).
    ///
    /// An override declaring **no** records states an empty leaf set: the field
    /// **does not cross**. That is the one way to drop a field a value form
    /// carries, and it follows from the complete-set rule rather than adding a
    /// rule — a boundary that wants none of a field's leaves says so the same
    /// way it says it wants some of them. (A *generator-level*
    /// `expand_return!(T)` with no records is still an error: a type has to
    /// cross somehow.) Use it for a field the binding has no surface for —
    /// diagnostics a consumer never reads, a type whose accessors would drag in
    /// a subtree nothing asks for:
    ///
    /// ```ignore
    /// prebindgen_registry::expand_return!(Sample).fields_self_into(
    ///     prebindgen_registry::fields!(sample_into_struct)
    ///         .field("timestamp_stack", prebindgen_registry::expand_return!(TimestampStack)),
    /// );
    /// ```
    pub fn field(mut self, field: impl AsRef<str>, decl: ExpandReturnDecl) -> Self {
        let field = field.as_ref().to_string();
        assert!(
            !self.overrides.iter().any(|(f, _)| *f == field),
            "fields!({}).field(\"{}\", ...): field already has an override — declare its \
             complete field set in ONE decl",
            self.func,
            field
        );
        self.overrides.push((field, decl));
        self
    }

    /// Rename **one** field's leaf, overriding the name derived from the struct
    /// field ident. The literal target-language name, like `fun!(f).name(...)`.
    pub fn name(mut self, field: impl AsRef<str>, target_name: impl Into<String>) -> Self {
        let field = field.as_ref().to_string();
        let target_name = target_name.into();
        assert!(
            !self.names.iter().any(|(f, _)| *f == field),
            "fields!({}).name(\"{}\", ...): field is already renamed",
            self.func,
            field
        );
        // The derived names of inlined nested fields are joined with `"__"`, so
        // an author name carrying one would forge a nesting that isn't there.
        // (Core rejects it for a `.field()` name; here the name never reaches
        // that check, so it is made at the point of declaration.)
        assert!(
            !target_name.contains("__"),
            "fields!({}).name(\"{}\", \"{}\"): `__` is the reserved chain separator \
             and cannot appear in a leaf name",
            self.func,
            field,
            target_name,
        );
        self.names.push((field, target_name));
        self
    }
}

/// Unifies the two boundary decls into one type so an adapter's `expand` can
/// expose a single entry point — the boundary-decl peer of its class
/// declarator.
/// Deliberately **no** `impl From<syn::Type> for ExpandDecl` — a bare
/// `syn::Type` alone doesn't say which direction it describes, so every
/// declaration names its direction via the matching constructor macro:
/// `.expand(prebindgen_registry::expand_param!(Summary)...)`,
/// `.expand(prebindgen_registry::expand_return!(Sample)...)`.
pub enum ExpandDecl {
    Param(ExpandParamDecl),
    Return(ExpandReturnDecl),
}

impl From<ExpandParamDecl> for ExpandDecl {
    fn from(d: ExpandParamDecl) -> Self {
        Self::Param(d)
    }
}
impl From<ExpandReturnDecl> for ExpandDecl {
    fn from(d: ExpandReturnDecl) -> Self {
        Self::Return(d)
    }
}

// ──────────────────────────────────────────────────────────────────────
// Function decl
// ──────────────────────────────────────────────────────────────────────

/// Declares one `#[prebindgen]` function to export. The adapter either adds it
/// to a package or attaches it to a class as a method or a factory.
///
/// Build it from a bare Rust name with [`fun!`](crate::fun) and chain
/// [`name`](Self::name) to set its target-language name.
/// [`expand_param`](Self::expand_param) / [`expand_return`](Self::expand_return)
/// **override, for this one function**, the boundary defaults its
/// parameter/return types declare at the generator level
/// — using the very same decl objects, so the complete-set rule is identical
/// at both scopes.
pub struct FunctionDecl {
    rust_ident: syn::Ident,
    name_override: Option<String>,
    param_expands: Vec<(String, ExpandParamDecl)>,
    return_expand: Option<ExpandReturnDecl>,
    split_on_params: Vec<String>,
    /// `fun!(crate::f)` — a **binding-local** fn: the declared path plus the
    /// stated signature ([`sig`](Self::sig), required by acceptance time).
    /// `None` = an ordinary `#[prebindgen]` registry fn.
    local: Option<(syn::Path, Option<syn::Signature>)>,
}

impl FunctionDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            name_override: None,
            param_expands: Vec::new(),
            return_expand: None,
            split_on_params: Vec::new(),
            local: None,
        }
    }

    /// The Rust-side fn ident (the path's last segment for a binding-local fn).
    pub fn rust_ident(&self) -> &syn::Ident {
        &self.rust_ident
    }

    /// The explicit target-language name, if `.name(...)` was declared.
    pub fn name_override(&self) -> &Option<String> {
        &self.name_override
    }

    /// The per-parameter expand overrides declared with `.expand_param(...)`.
    pub fn param_expands(&self) -> &[(String, ExpandParamDecl)] {
        &self.param_expands
    }

    /// The return expand override declared with `.expand_return(...)`, if any.
    pub fn return_expand(&self) -> &Option<ExpandReturnDecl> {
        &self.return_expand
    }

    /// The parameters named with `.split_on_param(...)`, in declaration order.
    pub fn split_on_params(&self) -> &[String] {
        &self.split_on_params
    }

    /// The binding-local fn's declared path and stated signature, if this is
    /// a `fun!(crate::f)` rather than an ordinary `#[prebindgen]` registry fn.
    pub fn local(&self) -> &Option<(syn::Path, Option<syn::Signature>)> {
        &self.local
    }

    /// Consume `self` into its raw fields, by value. Not a plain accessor —
    /// the one call site (`accept_fn_expands` in the JNI builder) destructures
    /// a whole `FunctionDecl` it owns to move each field (`Vec`s, the `local`
    /// signature) onward without cloning, so a reference-returning accessor
    /// won't do.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        syn::Ident,
        Option<String>,
        Vec<(String, ExpandParamDecl)>,
        Option<ExpandReturnDecl>,
        Vec<String>,
        Option<(syn::Path, Option<syn::Signature>)>,
    ) {
        (
            self.rust_ident,
            self.name_override,
            self.param_expands,
            self.return_expand,
            self.split_on_params,
            self.local,
        )
    }

    /// `fun!(crate::f)` — declare a **binding-local** fn by path. The fn
    /// ident (the path's last segment) names it everywhere a registry fn's
    /// ident would; chain [`sig`](Self::sig) to state its signature.
    pub fn new_local(path: syn::Path) -> Self {
        assert!(
            path.segments.len() >= 2,
            "fun!({}): a binding-local fn is called QUALIFIED from the generated file — \
             give at least a `crate::`-rooted path (a bare ident declares a `#[prebindgen]` fn)",
            quote::quote!(#path)
        );
        let ident = path.segments.last().expect("non-empty path").ident.clone();
        Self {
            local: Some((path, None)),
            ..Self::new(ident)
        }
    }

    /// State a binding-local fn's exact Rust signature (build it with
    /// [`sig!`](crate::sig)) — a path carries no signature to read. The
    /// parameter names become the foreign-side parameter names. Required for
    /// a path-built [`fun!`](crate::fun); a hard error on a registry fn
    /// (its signature is read from the registry).
    pub fn sig(mut self, signature: syn::Signature) -> Self {
        let Some((_, slot)) = &mut self.local else {
            panic!(
                "fun!({}).sig(...): a `#[prebindgen]` fn's signature is read from the \
                 registry — .sig() applies to path-built binding-local fns (fun!(crate::f))",
                self.rust_ident
            );
        };
        assert!(
            slot.is_none(),
            "fun!({}).sig(...): the signature is already stated",
            self.rust_ident
        );
        *slot = Some(signature);
        self
    }

    /// Set the name this function takes in the target language. Left unset,
    /// the adapter derives one from the Rust name — the JNI adapter, for
    /// example, camel-cases it (`session_declare_publisher` →
    /// `sessionDeclarePublisher`).
    pub fn name(mut self, target_name: impl Into<String>) -> Self {
        self.name_override = Some(target_name.into());
        self
    }

    /// Override, for the named parameter of this function only, how that
    /// parameter is supplied — with the same [`ExpandParamDecl`] a type-level
    /// default uses, so the **complete-set rule** applies here too: the decl
    /// states the entire variant set for this param (a lone `.variant_self()`
    /// = "only a ready-made handle", replacing the type's build variants —
    /// e.g. *un*-declaring a key expression needs the handle, not a string).
    ///
    /// `param` is the Rust parameter name; the decl's type is cross-checked
    /// against that parameter's (peeled) type at generation time — an unknown
    /// parameter or a type mismatch is a hard error. Call again with a
    /// different `param` to override several parameters independently;
    /// declaring the same parameter twice is a hard error.
    pub fn expand_param(mut self, param: impl AsRef<str>, decl: ExpandParamDecl) -> Self {
        let param = param.as_ref().to_string();
        assert!(
            !self.param_expands.iter().any(|(p, _)| *p == param),
            "fun!({}).expand_param(\"{}\", ...): parameter already has an expand override — \
             declare each parameter's complete variant set in ONE decl",
            self.rust_ident,
            param
        );
        self.param_expands.push((param, decl));
        self
    }

    /// **Emit idiomatic typed Kotlin overloads for this parameter.** By default
    /// a multi-variant expanded parameter crosses only as the selector tuple
    /// (`expectedSel: Int, expected0: …, expected1: …`). `.split_on_param("p")`
    /// additionally emits, alongside the selector wrapper, one typed overload
    /// per variant of `p` — `f(count: Long, total: Double, …)` for a
    /// `summary_new(count, total)` arm, `f(expected: Summary, …)` for
    /// `variant_self()` — each delegating to the selector form.
    ///
    /// The parameter's variant set must be *splittable* (its arms surface as
    /// distinct JVM signatures) — enforced up front on the
    /// [`expand_param!`](crate::expand_param) declaration unless it opted out
    /// with [`.no_split()`](ExpandParamDecl::no_split).
    ///
    /// Call again for **several** parameters: the generated overloads are then
    /// the **cartesian product** of the named parameters' arms. That concrete
    /// product must have no two combinations sharing a JVM signature — a hard
    /// build error if it does.
    ///
    /// An `Option<…>` parameter splits through its **single-leaf** arms only
    /// (nullable-arm rule): the overload keeps the arm's nullable type and
    /// `null` selects absence — `f(encoding: Encoding?, …)` for a
    /// `variant_self()` arm of an `Option<&Encoding>` parameter. Multi-leaf
    /// arms stay selector-only; an optional parameter with no single-leaf arm
    /// is a hard error.
    ///
    /// `param` is the Rust parameter name; it must be an expanded,
    /// multi-variant parameter of this function (unknown / single-variant /
    /// recursively-built ⇒ a hard error). Declaring the same parameter twice
    /// is a hard error.
    pub fn split_on_param(mut self, param: impl AsRef<str>) -> Self {
        let param = param.as_ref().to_string();
        assert!(
            !self.split_on_params.contains(&param),
            "fun!({}).split_on_param(\"{}\"): parameter is already split",
            self.rust_ident,
            param
        );
        self.split_on_params.push(param);
        self
    }

    /// Override this function's return decomposition — with the same
    /// [`ExpandReturnDecl`] a type-level default uses, stating the complete
    /// field set (a lone `.field_self()` = the raw whole value, which for a
    /// borrowed `&T` / `Option<&T>` return crosses by cloning into a fresh
    /// owned handle). The decl's type is cross-checked against the function's
    /// (peeled) return type at generation time — a mismatch is a hard error.
    /// At most one per function.
    pub fn expand_return(mut self, decl: ExpandReturnDecl) -> Self {
        assert!(
            self.return_expand.is_none(),
            "fun!({}).expand_return(...): the function already has a return expand override — \
             declare the complete field set in ONE decl",
            self.rust_ident
        );
        self.return_expand = Some(decl);
        self
    }
}

// ──────────────────────────────────────────────────────────────────────
// Convert decl — the canonical single-value conversion for a type
// ──────────────────────────────────────────────────────────────────────

/// Declares a type's **canonical single-value conversion**: how one value of
/// the type crosses the boundary wherever a single value is needed — as a
/// parameter or return, inside `Option<_>` / `Vec<_>` / the `Result<T, E>`
/// success position, as a `data_class` field. Each direction takes one
/// [`ConvertSourceDecl`]:
///
/// ```rust,ignore
/// .convert(convert!(Millis)
///     .input(fun!(millis_from_long))   // fn(u64) -> Millis    (wire → rust)
///     .output(fun!(millis_value))      // fn(&Millis) -> u64   (rust → wire)
///     .valid_range(0u64..=86_400_000)) // rejects invalid values; Option uses a niche
/// .convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
/// .convert(convert!(Label)
///     .input(try_from!(String).with(path!(crate::label_in)).error(ty!(String)))
///     .output(into!(String).with(path!(crate::label_out))))
/// ```
///
/// The foreign surface derives from the conversion's other-side type
/// (`u64` ⇒ Kotlin `ULong` / C `uint64_t`) — nothing is stated verbatim.
/// [`valid_range`](ConvertDecl::valid_range) and
/// [`valid_values`](ConvertDecl::valid_values) declare the legal subset of a
/// scalar representation. Generated converters validate the subset, and
/// adapters may reuse values outside it as allocation-free `Option`/`Result`
/// markers; [`exclude_values`](ConvertDecl::exclude_values) reserves holes in
/// an otherwise legal domain. A `try_` source's `Err`
/// routes to the caller's error handler. Conversion fns may live in the flat
/// crate or in a **helper crate** whose item stream is chained into the same
/// [`prebindgen_flat::Flat::builder`] parse; generated calls qualify each
/// function with its origin crate.
///
/// Distinct from the [`expand_param!`](crate::expand_param) /
/// [`expand_return!`](crate::expand_return) boundary decls: those reshape a
/// **function boundary** into multiple leaves (variants in / fields out),
/// while `convert!` defines the type's one-value form used everywhere else.
/// A type may declare both — expansion wins at the fn boundaries where it is
/// declared; the conversion serves every other position. The method names
/// differ deliberately: converters are direction-things ([`input`](ConvertDecl::input)
/// also serves callback returns, [`output`](ConvertDecl::output) also serves
/// callback arguments), while expansion decls are position-things.
/// One direction's conversion **source** — where the conversion code comes
/// from, the lowered form of a [`ConvertSourceDecl`].
// large_enum_variant: a handful of these exist per binding, held once in the
// builder — boxing the syn payloads would only complicate the decl arms.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ConvertSpec {
    /// A `#[prebindgen]` fn (flat or helper crate): the representable type
    /// and fallibility are read from its registry signature at lookup time.
    PrebindgenFn(syn::Ident),
    /// A `core::convert` trait impl; the representable type is stated
    /// explicitly (there is no signature to read). `fallible` selects
    /// `TryInto` (the associated `Error` routes to the caller's error
    /// handler) vs `Into`.
    Trait { repr: syn::Type, fallible: bool },
}

impl ConvertSpec {
    /// One-line human description of the source kind (report use).
    pub fn describe(&self) -> String {
        match self {
            ConvertSpec::PrebindgenFn(f) => format!("`#[prebindgen]` fn `{f}`"),
            ConvertSpec::Trait {
                repr,
                fallible: false,
            } => format!("`Into` ⇄ `{}`", repr.to_token_stream()),
            ConvertSpec::Trait {
                repr,
                fallible: true,
            } => format!("`TryInto` ⇄ `{}`", repr.to_token_stream()),
        }
    }
}

/// Which direction a [`ConvertSourceDecl`] was built for. The constructor
/// macro states it (`from!`/`try_from!` = into-Rust, `into!`/`try_into!` =
/// out-of-Rust) and the acceptor cross-checks it, so a chain like
/// `.output(from!(i32))` is a hard error instead of a silent misread.
///
/// Stated as an [`Direction`], the one type naming this axis: a `from!` builds
/// the Rust value out of what crossed, an `into!` takes one apart.
fn convert_macros(direction: Direction) -> &'static str {
    match direction {
        Direction::Construct => "from!/try_from!",
        Direction::Deconstruct => "into!/try_into!",
    }
}

/// One conversion source, accepted by [`ConvertDecl::input`] /
/// [`ConvertDecl::output`]. Built by [`fun!`](crate::fun) — a
/// `#[prebindgen]` conversion fn (bare ident, signature read from the
/// registry) or a **binding-local** one (`fun!(crate::f)` +
/// [`.sig(sig!(…))`](FunctionDecl::sig), the one vocabulary for locally
/// defined callables; a `Result<_, E>` return states the error channel) —
/// or by the direction-stating macros [`from!`](crate::from) /
/// [`try_from!`](crate::try_from) / [`into!`](crate::into) /
/// [`try_into!`](crate::try_into) (a `core::convert` **trait** conversion
/// with a stated representation type).
#[derive(Clone)]
pub struct ConvertSourceDecl {
    kind: ConvertSourceKind,
}

// large_enum_variant: a handful of these exist per binding, held transiently
// while a decl is built — boxing the syn payloads would only complicate the
// arms (same trade-off as `ConvertSpec`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub(crate) enum ConvertSourceKind {
    /// `fun!(f)` / `fun!(crate::f).sig(…)` — a conversion fn; representable
    /// type and fallibility are read from its signature (registry, or the
    /// stated one carried in `local` and synthesized before scanning).
    Fun {
        ident: syn::Ident,
        local: Option<(syn::Path, syn::Signature)>,
    },
    /// `from!`/`try_from!`/`into!`/`try_into!` — a stated representation
    /// type, converted via the `core::convert` trait.
    Repr {
        direction: Direction,
        fallible: bool,
        ty: syn::Type,
    },
}

impl ConvertSourceDecl {
    fn repr(direction: Direction, fallible: bool, ty: syn::Type) -> Self {
        Self {
            kind: ConvertSourceKind::Repr {
                direction,
                fallible,
                ty,
            },
        }
    }
    /// `from!(T)` — input via `T: Into<Self>`.
    pub fn from_type(ty: syn::Type) -> Self {
        Self::repr(Direction::Construct, false, ty)
    }
    /// `try_from!(T)` — input via `T: TryInto<Self>`.
    pub fn try_from_type(ty: syn::Type) -> Self {
        Self::repr(Direction::Construct, true, ty)
    }
    /// `into!(T)` — output via `Self: Into<T>`.
    pub fn into_type(ty: syn::Type) -> Self {
        Self::repr(Direction::Deconstruct, false, ty)
    }
    /// `try_into!(T)` — output via `Self: TryInto<T>`.
    pub fn try_into_type(ty: syn::Type) -> Self {
        Self::repr(Direction::Deconstruct, true, ty)
    }
}

impl From<FunctionDecl> for ConvertSourceDecl {
    fn from(decl: FunctionDecl) -> Self {
        assert!(
            decl.name_override.is_none()
                && decl.param_expands.is_empty()
                && decl.return_expand.is_none(),
            "fun!({}) as a conversion source: a conversion fn is never surfaced in \
             the target language — .name()/expand overrides don't apply",
            decl.rust_ident
        );
        let local = decl.local.map(|(path, sig)| {
            let Some(sig) = sig else {
                panic!(
                    "fun!({p}) as a conversion source: a binding-local fn states its \
                     signature — chain .sig(sig!((params) -> Ret))",
                    p = quote::quote!(#path)
                );
            };
            (path, sig)
        });
        Self {
            kind: ConvertSourceKind::Fun {
                ident: decl.rust_ident,
                local,
            },
        }
    }
}

#[derive(Clone)]
pub struct ConvertDecl {
    key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    rust_type: Origin<syn::Type>,
    input: Option<ConvertSpec>,
    output: Option<ConvertSpec>,
    domain: Option<crate::RepresentationDomain>,
    /// Binding-local fn sources declared on this convert (`fun!(crate::f)
    /// .sig(…)`): drained into [`Declarations::local_fns`] at acceptance so the
    /// synthesis pre-pass covers them.
    locals: Vec<(syn::Ident, syn::Path, syn::Signature)>,
}

impl ConvertDecl {
    /// `: input …, output …` suffix for the report's conversions section.
    pub fn describe_sources(&self) -> String {
        let mut parts = Vec::new();
        if let Some(i) = &self.input {
            parts.push(format!("input {}", i.describe()));
        }
        if let Some(o) = &self.output {
            parts.push(format!("output {}", o.describe()));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(": {}", parts.join(", "))
        }
    }

    pub fn new(rust_type: syn::Type) -> Self {
        reject_builtin_convert_type(&TypeKey::from_type(&rust_type));
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            input: None,
            output: None,
            domain: None,
            locals: Vec::new(),
        }
    }

    /// The type identity this declaration is registered under.
    pub fn key(&self) -> &TypeKey {
        &self.key
    }

    /// The type this declaration was written with, as originally parsed.
    pub fn rust_type(&self) -> &Origin<syn::Type> {
        &self.rust_type
    }

    /// The declared **into-Rust** conversion source, if any. Named
    /// `input_spec` rather than `input` — that name is already the builder
    /// method that declares it ([`Self::input`]).
    pub fn input_spec(&self) -> &Option<ConvertSpec> {
        &self.input
    }

    /// The declared **out-of-Rust** conversion source, if any. Named
    /// `output_spec` rather than `output` — that name is already the builder
    /// method that declares it ([`Self::output`]).
    pub fn output_spec(&self) -> &Option<ConvertSpec> {
        &self.output
    }

    /// The declared representation-domain restriction, if any.
    pub fn domain(&self) -> &Option<crate::RepresentationDomain> {
        &self.domain
    }

    /// Binding-local fn sources declared on this convert, drained into the
    /// synthesis pre-pass at acceptance.
    pub fn locals(&self) -> &[(syn::Ident, syn::Path, syn::Signature)] {
        &self.locals
    }

    /// Mutable access for draining binding-local fn sources into the
    /// synthesis pre-pass at acceptance (`Vec::append`).
    pub fn locals_mut(&mut self) -> &mut Vec<(syn::Ident, syn::Path, syn::Signature)> {
        &mut self.locals
    }

    fn set_input(mut self, spec: ConvertSpec) -> Self {
        assert!(
            self.input.is_none(),
            "convert!({}): the input conversion is already declared — \
             declare each direction's conversion in ONE .input()/.output() call",
            self.key.as_str()
        );
        self.input = Some(spec);
        self
    }

    fn set_output(mut self, spec: ConvertSpec) -> Self {
        assert!(
            self.output.is_none(),
            "convert!({}): the output conversion is already declared — \
             declare each direction's conversion in ONE .input()/.output() call",
            self.key.as_str()
        );
        self.output = Some(spec);
        self
    }

    fn check_repr(&self, method: &str, repr: &syn::Type) {
        assert!(
            TypeKey::from_type(repr) != self.key,
            "convert!({k}).{method}: the representable type must differ from `{k}` itself",
            k = self.key.as_str()
        );
    }

    /// Lower an accepted [`ConvertSourceDecl`] to the internal spec,
    /// cross-checking the source's stated direction against the acceptor. A
    /// binding-local fn source records its `(ident, path, sig)` in
    /// [`Self::locals`] for the synthesis pre-pass — after which it lowers
    /// exactly like a `#[prebindgen]` fn source.
    fn spec_of(
        &mut self,
        direction: Direction,
        method: &str,
        src: ConvertSourceDecl,
    ) -> ConvertSpec {
        match src.kind {
            ConvertSourceKind::Fun { ident, local } => {
                if let Some((path, sig)) = local {
                    self.locals.push((ident.clone(), path, sig));
                }
                ConvertSpec::PrebindgenFn(ident)
            }
            ConvertSourceKind::Repr {
                direction: stated,
                fallible,
                ty,
            } => {
                assert!(
                    stated == direction,
                    "convert!({k}).{method}(...): the source was built with {got} — \
                     an {method} conversion is built with {want}",
                    k = self.key.as_str(),
                    got = convert_macros(stated),
                    want = convert_macros(direction),
                );
                self.check_repr(method, &ty);
                ConvertSpec::Trait { repr: ty, fallible }
            }
        }
    }

    /// The **into-Rust** conversion (parameters, callback returns): how a
    /// value of this type is built from its representation. Accepts
    /// [`fun!`](crate::fun) (a `#[prebindgen]` `fn(U) -> T` /
    /// `fn(U) -> Result<T, E>`) or [`from!`](crate::from) /
    /// [`try_from!`](crate::try_from) (`Repr: Into<T>` / `TryInto`, or a
    /// binding-local callable via `.with(...)`).
    pub fn input(mut self, src: impl Into<ConvertSourceDecl>) -> Self {
        let spec = self.spec_of(Direction::Construct, "input", src.into());
        self.set_input(spec)
    }

    /// The **out-of-Rust** conversion (returns, callback arguments): how a
    /// value of this type is turned into its representation. Accepts
    /// [`fun!`](crate::fun) (a `#[prebindgen]` `fn(&T) -> U` / `fn(T) -> U`)
    /// or [`into!`](crate::into) / [`try_into!`](crate::try_into)
    /// (`T: Into<Repr>` / `TryInto`, or a binding-local callable via
    /// `.with(...)`).
    pub fn output(mut self, src: impl Into<ConvertSourceDecl>) -> Self {
        let spec = self.spec_of(Direction::Deconstruct, "output", src.into());
        self.set_output(spec)
    }

    /// Restrict the scalar representation to a numeric range. Values outside
    /// the range are rejected and may be reused by wrappers such as `Option`.
    /// Floating-point ranges reject every NaN; use [`Self::valid_values`] when
    /// exact raw IEEE values (including a specific NaN payload) are intended.
    pub fn valid_range<T, R>(mut self, range: R) -> Self
    where
        T: crate::DomainScalar,
        R: ::core::ops::RangeBounds<T>,
    {
        assert!(
            self.domain.is_none(),
            "convert!({}): the representation domain is already declared",
            self.key.as_str()
        );
        self.domain = Some(crate::RepresentationDomain::range(range));
        self
    }

    /// Restrict the scalar representation to a finite valid-value set. Float
    /// membership uses raw IEEE bits, so `0.0`, `-0.0`, and NaN payloads remain
    /// distinct.
    pub fn valid_values<T>(mut self, values: impl IntoIterator<Item = T>) -> Self
    where
        T: crate::DomainScalar,
    {
        assert!(
            self.domain.is_none(),
            "convert!({}): the representation domain is already declared",
            self.key.as_str()
        );
        self.domain = Some(crate::RepresentationDomain::values(values));
        self
    }

    /// Remove finite values from the previously declared base domain. Float
    /// exclusions use raw IEEE bits.
    pub fn exclude_values<T>(mut self, values: impl IntoIterator<Item = T>) -> Self
    where
        T: crate::DomainScalar,
    {
        self.domain
            .as_mut()
            .unwrap_or_else(|| {
                panic!(
                    "convert!({}): .exclude_values(...) requires .valid_range(...) \
                     or .valid_values(...) first",
                    self.key.as_str()
                )
            })
            .exclude(values);
        self
    }
}

/// Rejects a `convert!` declaration on a Rust **builtin** type: builtins
/// already have their own converters, and the generated calls would try to
/// qualify the builtin with a crate path. Wrap the builtin in a source-crate
/// newtype (like `Millis(u64)`) instead.
fn reject_builtin_convert_type(key: &TypeKey) {
    const BUILTINS: &[&str] = &[
        "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
        "f32", "f64", "bool", "char", "str", "String",
    ];
    assert!(
        !BUILTINS.contains(&key.as_str()),
        "convert!({}): builtins already have converters — wrap the builtin in a newtype instead",
        key.as_str()
    );
}

/// Bare-ident type `__JniErr` — the generated file's alias for the
/// `prebindgen-jni` crate's `JniBindingError` framework type. Built-in
/// converters use this as their `Result<…, _>` error type so their bodies'
/// `<__JniErr as From<String>>::from(...)` calls keep compiling. A
/// `Result<T, E>` return instead binds its own raw `E` (see
/// `JniGenBuilder::lookup_output`); the extern's `Err` arm funnels both to the
/// per-call `signal_error` sink via `E: Display`.
/// The origin-module prefix of a binding-local fn's declared path
/// (`crate::sub::f` → `"crate::sub"`). Paths are validated ≥2 segments at
/// decl time (`fun!` path arm / `FieldDecl::with`), so the prefix is
/// always non-empty.
pub fn local_path_prefix(path: &syn::Path) -> String {
    path.segments
        .iter()
        .take(path.segments.len() - 1)
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
