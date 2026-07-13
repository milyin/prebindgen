//! Declaration objects: one standalone, independently-constructible value
//! type per kind of thing `JniGen` can be told about (a `ptr_class`, an
//! `enum_class`, a function, a scalar wire mapping, …), plus the `PackageDecl`
//! that aggregates the package-scoped ones. Each type is both its own
//! "builder" and the final value `JniGen`/`PackageDecl` accepts — no separate
//! `Builder`/`Decl` split, no terminal `.build()` call.
//!
//! `JniGen` itself only ever *accepts* fully-built values of these types
//! (`JniGen::package`, `JniGen::expand`, `JniGen::convert`, in
//! `builder.rs`); none of them reach back
//! into any `JniGen` state while being built.

use super::*;

// ──────────────────────────────────────────────────────────────────────
// Shared local accumulators (replayed into `Expansions`/`Deconstructors`
// by the accept logic in `builder.rs` once a decl is handed to `JniGen`)
// ──────────────────────────────────────────────────────────────────────

/// One arm of an `expand_param!` `.variant*` list (type-level or per-fn).
#[derive(Clone)]
pub(crate) enum LocalVariant {
    /// Build via this declared constructor member / constructor fn.
    Ctor(syn::Ident),
    /// Accept an already-built value directly.
    SelfIdentity,
}

/// One arm of an `expand_return!` `.field*` list (type-level or per-fn). The name is stored raw (`None` = derive at replay time: for a
/// class-level field, the class member's Kotlin name if the accessor is a
/// declared member, else `snake_to_camel`; for a per-fn field,
/// `snake_to_camel`).
#[derive(Clone)]
pub(crate) enum LocalField {
    /// Include the named accessor's value as a leaf/field, with an optional
    /// explicit name override.
    Named(syn::Ident, Option<String>),
    /// Include the handle itself as a field.
    SelfField,
}

// Class members are stored as the full `(FunctionDecl, MemberKind)` pair —
// not a reduced ident+name record — so the `FunctionDecl`'s per-fn
// `.expand_param`/`.expand_return` overrides survive to `builder.rs`'s
// `accept_members`, which applies them exactly like `accept_function` does
// for free package functions.

// ──────────────────────────────────────────────────────────────────────
// Decl constructor macros — one per decl type built from bare Rust syntax
// or with no arguments at all. Each is restricted at the `macro_rules!`
// fragment level (`:ty` / `:ident`) and expands to a call with a hard-coded
// concrete return type, so `syn::parse_quote!`/`syn::parse_str` never has to
// infer its output type against a generic bound — there is no `E0283` risk
// to route around here, unlike a bare `syn::parse_quote!(...)` would have if
// fed into a generic `impl Into<T>` parameter.
// ──────────────────────────────────────────────────────────────────────

/// Build a [`PtrClassDecl`] directly from a bare Rust type: `ptr_class!(Foo)`
/// is `PtrClassDecl::new(<Foo as a parsed syn::Type>)`.
#[macro_export]
macro_rules! ptr_class {
    ($t:ty) => {
        $crate::lang::PtrClassDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build an [`EnumClassDecl`] directly from a bare Rust type. See [`ptr_class!`].
#[macro_export]
macro_rules! enum_class {
    ($t:ty) => {
        $crate::lang::EnumClassDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`DataClassDecl`] directly from a bare Rust type. See [`ptr_class!`].
#[macro_export]
macro_rules! data_class {
    ($t:ty) => {
        $crate::lang::DataClassDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`ValueClassDecl`] directly from a bare Rust type. See [`ptr_class!`].
#[macro_export]
macro_rules! value_class {
    ($t:ty) => {
        $crate::lang::ValueClassDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`FunctionDecl`] directly from a bare function ident: `fun!(foo)`
/// is `FunctionDecl::new(prebindgen::ident!(foo))`.
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::lang::FunctionDecl::new($crate::ident!($name))
    };
}

/// Build a [`ConstDecl`] directly from a bare const ident:
/// `constant!(MAX_LEN)` is `ConstDecl::new(prebindgen::ident!(MAX_LEN))`.
#[macro_export]
macro_rules! constant {
    ($name:ident) => {
        $crate::lang::ConstDecl::new($crate::ident!($name))
    };
}

/// Build a [`ConstExprDecl`] in val-declaration syntax:
/// `constant_expr!(BANNER: String = format!("{COVER_TAG}:{COVER_MAGIC}"))` is
/// `ConstExprDecl::new("BANNER", <String>, <the expression>)`. The expression
/// is evaluated inside the generated getter with a glob import of every
/// source module in scope.
#[macro_export]
macro_rules! constant_expr {
    ($name:ident : $ty:ty = $expr:expr) => {
        $crate::lang::ConstExprDecl::new(
            stringify!($name),
            ::syn::parse_quote!($ty),
            ::syn::parse_quote!($expr),
        )
    };
}

/// Build a [`PackageDecl`] directly: `package!("model")` is
/// `PackageDecl::new("model")`; `package!()` (no args) is the base package
/// (`PackageDecl::new("")`).
#[macro_export]
macro_rules! package {
    () => {
        $crate::lang::PackageDecl::new("")
    };
    ($name:expr) => {
        $crate::lang::PackageDecl::new($name)
    };
}

/// Build a [`ConvertDecl`] directly from a bare Rust type:
/// `convert!(Millis)` is `ConvertDecl::new(<Millis as syn::Type>)`.
/// See [`ptr_class!`] for the parsing mechanics.
#[macro_export]
macro_rules! convert {
    ($t:ty) => {
        $crate::lang::ConvertDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`ExpandParamDecl`] directly from a bare Rust type:
/// `expand_param!(KeyExpr)` is `ExpandParamDecl::new(<KeyExpr as syn::Type>)`.
/// See [`ptr_class!`] for the parsing mechanics.
#[macro_export]
macro_rules! expand_param {
    ($t:ty) => {
        $crate::lang::ExpandParamDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

/// Build a [`ExpandReturnDecl`] directly from a bare Rust type:
/// `expand_return!(Sample)` is `ExpandReturnDecl::new(<Sample as syn::Type>)`.
/// See [`ptr_class!`] for the parsing mechanics.
#[macro_export]
macro_rules! expand_return {
    ($t:ty) => {
        $crate::lang::ExpandReturnDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

// ──────────────────────────────────────────────────────────────────────
// Class-kind decls
// ──────────────────────────────────────────────────────────────────────

/// Declares a Rust type as an **opaque handle**. In Kotlin it becomes a
/// closeable class holding a pointer to the real object, which keeps living
/// in Rust; the object crosses the boundary as that pointer, never copied.
/// Use this for types with identity and a lifecycle — sessions, subscribers,
/// configs, key expressions — that you pass around and eventually `close()`,
/// as opposed to plain data you copy across ([`data_class!`](crate::data_class))
/// or small `Copy` values ([`value_class!`](crate::value_class)).
///
/// A type that never materializes in Kotlin needs **no class declaration at
/// all**: give it boundary decls only ([`expand_param!`](crate::expand_param)
/// / [`expand_return!`](crate::expand_return)) and it stays rust-side-only —
/// built from ingredients on the way in, decomposed into fields on the way
/// out.
///
/// Build one with [`ptr_class!`](crate::ptr_class), add it to a
/// [`PackageDecl`], and hand that to [`JniGen::package`].
///
/// A `PtrClassDecl` defines the **Kotlin class only** — its name
/// ([`name`](Self::name)), its instance methods ([`fun`](Self::fun)), and its
/// companion-object factories ([`constructor`](Self::constructor)). How the
/// type crosses the FFI boundary by default — accepted as which parameter
/// variants, returned as which field set — is declared separately with
/// [`expand_param!`](crate::expand_param) / [`expand_return!`](crate::expand_return)
/// handed to [`JniGen::expand`]; any single
/// function can override those defaults locally (see [`FunctionDecl`]).
///
/// ```
/// // A KeyExpr handle exposing `str()` as an instance method.
/// let _ = prebindgen::ptr_class!(KeyExpr)
///     .fun(prebindgen::fun!(keyexpr_get_str).name("str"));
/// ```
pub struct PtrClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
}

impl PtrClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            members: Vec::new(),
        }
    }

    /// Rename the generated Kotlin class. By default it is named after the
    /// Rust type (via the [`JniGen::set_ptr_class_name_mangle`] hook); `.name("Foo")`
    /// sets it literally instead. Relative name, no dots — the package comes
    /// from the enclosing [`PackageDecl`].
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Expose a `#[prebindgen]` method as a Kotlin **instance method** of this
    /// class. `rust_fun` must take `&Self` first — that receiver becomes
    /// Kotlin's `this` and drops out of the signature; any further parameters
    /// become the method's arguments. Name it with
    /// `fun!(rust_name).name("kotlinName")` (default: the Rust name
    /// camel-cased).
    pub fn fun(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Fun));
        self
    }

    /// Expose a `#[prebindgen]` factory as a Kotlin **companion-object
    /// factory** — callers write `Class.name(...)`. `rust_fun` returns `Self`
    /// (or `Result<Self, E>`) and its parameters become the factory's
    /// arguments. A constructor can also serve as a build option in a
    /// [`expand_param!`](crate::expand_param) variant list.
    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Constructor));
        self
    }
}

impl From<syn::Type> for PtrClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
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
/// it to [`JniGen::expand`]. With more than one arm the generated Kotlin
/// selects the variant at runtime.
///
/// The type does **not** have to be declared in any package. A boundary decl
/// on an undeclared type makes it **rust-side-only**: the value is always
/// built from its ingredients at the boundary and never materializes in
/// Kotlin — no class, no handle, nothing to `close()`. The one restriction is
/// structural: [`variant_self`](Self::variant_self) hard-errors for such a
/// type, since there is no Kotlin object to pass.
///
/// ```
/// // A KeyExpr param accepts EITHER a String (built via keyexpr_new_try_from)
/// // OR an existing KeyExpr handle:
/// let _ = prebindgen::expand_param!(KeyExpr)
///     .variant(prebindgen::fun!(keyexpr_new_try_from))
///     .variant_self();
/// ```
#[derive(Clone)]
pub struct ExpandParamDecl {
    pub(crate) key: TypeKey,
    pub(crate) variants: Vec<LocalVariant>,
}

impl ExpandParamDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            variants: Vec::new(),
        }
    }

    /// Add a **build-from** arm: parameters of this type also accept the
    /// named `#[prebindgen]` constructor's inputs, and Rust builds the value
    /// in the same call. E.g. `keyexpr_new_try_from(&str)` lets every
    /// function taking a `KeyExpr` also accept a plain `String`.
    pub fn variant(mut self, ctor: FunctionDecl) -> Self {
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
/// [`JniGen::expand`].
///
/// The type does **not** have to be declared in any package. A boundary decl
/// on an undeclared type makes it **rust-side-only**: every returned /
/// callback-delivered / `Result`-error value of it is decomposed into these
/// fields and the value itself never reaches Kotlin. This is the natural
/// shape for an error type consumed by the `onError` channel — no dead
/// Kotlin class is emitted. Restrictions for such a type:
/// [`field_self`](Self::field_self) hard-errors (there is no Kotlin object to
/// deliver), and field names cannot inherit from class members (there are
/// none) — use `.name(...)` on each field or accept the camel-cased default.
///
/// ```
/// // A returned Sample crosses as { payload, kind } in one call:
/// let _ = prebindgen::expand_return!(Sample)
///     .field(prebindgen::fun!(sample_get_payload))
///     .field(prebindgen::fun!(sample_get_kind));
/// ```
#[derive(Clone)]
pub struct ExpandReturnDecl {
    pub(crate) key: TypeKey,
    pub(crate) fields: Vec<LocalField>,
}

impl ExpandReturnDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            fields: Vec::new(),
        }
    }

    /// Add one field: the named `#[prebindgen]` reader's (`f(&Self) -> Field`)
    /// value. The Kotlin field name is, in order of precedence: an explicit
    /// `.name(...)` on `accessor`; the Kotlin name of the class member if the
    /// same function is declared via [`PtrClassDecl::fun`] on this type (so a
    /// getter that is both a method and a field is named once); else the
    /// camel-cased Rust name.
    pub fn field(mut self, accessor: FunctionDecl) -> Self {
        self.fields.push(LocalField::Named(
            accessor.rust_ident,
            accessor.kotlin_name_override,
        ));
        self
    }

    /// Include the **handle itself** among the fields, so the consumer gets a
    /// live, closeable object in addition to the read-out values (e.g. a
    /// `Query` delivered with its fields *and* the handle it needs to reply).
    /// Declare it **last**, after any field that decomposes a nested handle,
    /// so the generated Rust moves the value only after those borrows.
    pub fn field_self(mut self) -> Self {
        self.fields.push(LocalField::SelfField);
        self
    }
}

/// Unifies the two boundary decls into one type so [`JniGen::expand`] can
/// expose a single entry point — the boundary-decl peer of [`ClassDecl`].
/// Deliberately **no** `impl From<syn::Type> for ExpandDecl` — a bare
/// `syn::Type` alone doesn't say which direction it describes, so every
/// declaration names its direction via the matching constructor macro:
/// `.expand(prebindgen::expand_param!(Summary)...)`,
/// `.expand(prebindgen::expand_return!(Sample)...)`.
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

/// Declares a Rust C-like `enum` as a Kotlin `enum class`. The variants
/// cross the boundary as their `i32` discriminants and Kotlin gets a real
/// `enum class` with a `fromInt(...)` companion. The enum must be
/// unit-variant only and `#[repr(i32)]`-style with explicit discriminants,
/// so both sides agree on the numbers.
///
/// Has no `.fun`/`.constructor` — instance members are only meaningful on
/// handle ([`PtrClassDecl`]) and value ([`ValueClassDecl`]) classes.
pub struct EnumClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
}

impl EnumClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }
}

impl From<syn::Type> for EnumClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a Rust struct as a Kotlin `data class`. Its fields cross the
/// boundary individually and Kotlin reassembles the object with a generated
/// `fromParts(...)` — no Rust-side heap object, no handle to close. Use this
/// for plain immutable data you copy across, as opposed to
/// [`ptr_class!`](crate::ptr_class) handles or
/// [`value_class!`](crate::value_class) blobs.
///
/// Has no `.fun`/`.constructor` — a data class has no handle to hang an
/// instance method on.
pub struct DataClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) kotlin_type: Option<String>,
}

impl DataClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            kotlin_type: None,
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Surface this type as a verbatim Kotlin type instead of a generated
    /// class — for when it should map onto an existing or container type,
    /// e.g. `"List<ByteArray>"`.
    pub fn kotlin_type(mut self, expr: impl Into<String>) -> Self {
        self.kotlin_type = Some(expr.into());
        self
    }
}

impl From<syn::Type> for DataClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a small **`Copy`** Rust type that crosses **by value** — as its
/// raw bytes in a `ByteArray` — rather than as a heap handle. The
/// lightweight peer of [`PtrClassDecl`] for things like ids and timestamps
/// that have no lifecycle to manage. The type must be `Copy` (the generator
/// asserts it at compile time). Readers added with [`fun`](Self::fun) become
/// instance methods on the Kotlin value class.
pub struct ValueClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) kotlin_type: Option<String>,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
}

impl ValueClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            kotlin_type: None,
            members: Vec::new(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Surface this type as a verbatim Kotlin type instead of a generated
    /// value class (see [`DataClassDecl::kotlin_type`]).
    pub fn kotlin_type(mut self, expr: impl Into<String>) -> Self {
        self.kotlin_type = Some(expr.into());
        self
    }

    /// Expose a `#[prebindgen]` reader (`f(&Self) -> R`) as an instance
    /// method on the Kotlin value class (see [`PtrClassDecl::fun`]).
    pub fn fun(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Fun));
        self
    }

    /// Expose a `#[prebindgen]` factory as a companion-object factory
    /// (see [`PtrClassDecl::constructor`]).
    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Constructor));
        self
    }
}

impl From<syn::Type> for ValueClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Unifies the four class-kind decls into one type so [`PackageDecl::class`]
/// can expose a single entry point. Deliberately **no**
/// `impl From<syn::Type> for ClassDecl` — a bare `syn::Type` alone doesn't
/// say which of the four kinds it should become, so every declaration names
/// its kind explicitly via the matching constructor macro:
/// `.class(prebindgen::ptr_class!(Storage))`,
/// `.class(prebindgen::enum_class!(Priority))`, etc.
pub enum ClassDecl {
    Ptr(PtrClassDecl),
    Enum(EnumClassDecl),
    Data(DataClassDecl),
    Value(ValueClassDecl),
}

impl From<PtrClassDecl> for ClassDecl {
    fn from(d: PtrClassDecl) -> Self {
        Self::Ptr(d)
    }
}
impl From<EnumClassDecl> for ClassDecl {
    fn from(d: EnumClassDecl) -> Self {
        Self::Enum(d)
    }
}
impl From<DataClassDecl> for ClassDecl {
    fn from(d: DataClassDecl) -> Self {
        Self::Data(d)
    }
}
impl From<ValueClassDecl> for ClassDecl {
    fn from(d: ValueClassDecl) -> Self {
        Self::Value(d)
    }
}

// ──────────────────────────────────────────────────────────────────────
// Function decl
// ──────────────────────────────────────────────────────────────────────

/// Declares one `#[prebindgen]` function to export. Add it to a package with
/// [`PackageDecl::fun`], or attach it to a class as a method/factory with
/// [`PtrClassDecl::fun`] / [`PtrClassDecl::constructor`].
///
/// Build it from a bare Rust name with [`fun!`](crate::fun) and chain
/// [`name`](Self::name) to set its Kotlin name.
/// [`expand_param`](Self::expand_param) / [`expand_return`](Self::expand_return)
/// **override, for this one function**, the boundary defaults its
/// parameter/return types declare at the generator level ([`JniGen::expand`])
/// — using the very same decl objects, so the complete-set rule is identical
/// at both scopes.
pub struct FunctionDecl {
    pub(crate) rust_ident: syn::Ident,
    pub(crate) kotlin_name_override: Option<String>,
    pub(crate) param_expands: Vec<(String, ExpandParamDecl)>,
    pub(crate) return_expand: Option<ExpandReturnDecl>,
}

impl FunctionDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            param_expands: Vec::new(),
            return_expand: None,
        }
    }

    /// Set the Kotlin-side name. Default: the Rust name camel-cased
    /// (`session_declare_publisher` → `sessionDeclarePublisher`).
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        self.kotlin_name_override = Some(kotlin_name.into());
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

/// Declares one `#[prebindgen]` **const** for emission: on the Rust side a
/// nullary JNI getter extern is generated (the const's type goes through the
/// ordinary output-converter machinery, exactly like a function return); on
/// the Kotlin side the const surfaces as an eagerly-initialized top-level
/// `val` in its package's `.kt` file.
///
/// Build one with [`constant!`](crate::constant) and add it to a
/// [`PackageDecl`] via [`PackageDecl::constant`]. Opaque-handle-typed consts
/// are rejected (a shared closeable `val` is semantically wrong) — expose a
/// factory function instead.
pub struct ConstDecl {
    pub(crate) rust_ident: syn::Ident,
    pub(crate) kotlin_name_override: Option<String>,
}

impl ConstDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
        }
    }

    /// Set the Kotlin-side name. Default: the Rust const ident verbatim
    /// (`MAX_LEN` → `val MAX_LEN` — SCREAMING_SNAKE is the Kotlin constant
    /// convention too).
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        self.kotlin_name_override = Some(kotlin_name.into());
        self
    }
}

/// Declares one **expression-backed constant**: an arbitrary binding-defined
/// Rust expression, evaluated once inside a generated nullary JNI getter and
/// surfaced as an eagerly-initialized top-level Kotlin `val`. The expression
/// runs with a glob import of every source module in scope, so it composes the source
/// crate's `#[prebindgen]` items freely without the source crate having to
/// export a dedicated accessor per constant — e.g.
/// `encoding_to_string(encoding_const_text_plain())`.
///
/// Build one with [`constant_expr!`](crate::constant_expr) (literal form) or
/// [`ConstExprDecl::new`] (runtime form, for declaration loops) and add it to
/// a [`PackageDecl`] via [`PackageDecl::constant_expr`]. The value type is
/// declared explicitly and flows through the ordinary output-converter
/// machinery; opaque-handle and `Result` types are rejected like every other
/// constant kind.
#[derive(Clone)]
pub struct ConstExprDecl {
    pub(crate) kotlin_name: String,
    pub(crate) ty: syn::Type,
    pub(crate) expr: syn::Expr,
}

impl ConstExprDecl {
    /// `kotlin_name` is the top-level `val` name (also the seed of the
    /// extern symbol, so it must be unique among the binding's constants);
    /// `ty` is the Rust value type the expression yields; `expr` is the
    /// initializer expression, resolved against the source module.
    pub fn new(kotlin_name: impl Into<String>, ty: syn::Type, expr: syn::Expr) -> Self {
        Self {
            kotlin_name: kotlin_name.into(),
            ty,
            expr,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// PackageDecl — aggregates the package-scoped decls
// ──────────────────────────────────────────────────────────────────────

/// A batch of class, function and const declarations that land under one
/// Kotlin subpackage. Build it with [`package!`](crate::package)
/// (`package!("session")`, or `package!()` for the base package), fill it
/// with [`class`](Self::class) / [`fun`](Self::fun) /
/// [`constant`](Self::constant), and hand it to
/// [`JniGen::package`]. Reopening the same subpackage across several
/// `PackageDecl`s is fine — they merge.
pub struct PackageDecl {
    pub(crate) name: String,
    pub(crate) classes: Vec<ClassDecl>,
    pub(crate) functions: Vec<FunctionDecl>,
    pub(crate) constants: Vec<ConstDecl>,
    pub(crate) constant_functions: Vec<FunctionDecl>,
    pub(crate) constant_exprs: Vec<ConstExprDecl>,
}

impl PackageDecl {
    /// `name` is dot-separated, relative to the base package set by
    /// [`JniGen::set_package_prefix`]; the empty string is the base
    /// package itself. See [`crate::package!`] for the equivalent macro form
    /// (`package!("model")` / `package!()`).
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let name = name.trim_matches('.').trim_matches('/').to_string();
        Self {
            name,
            classes: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            constant_functions: Vec::new(),
            constant_exprs: Vec::new(),
        }
    }

    /// Add a class to this package — any of [`ptr_class!`](crate::ptr_class) /
    /// [`enum_class!`](crate::enum_class) / [`data_class!`](crate::data_class) /
    /// [`value_class!`](crate::value_class).
    pub fn class(mut self, decl: impl Into<ClassDecl>) -> Self {
        self.classes.push(decl.into());
        self
    }

    /// Add a free function to this package. Take a bare name via
    /// [`fun!`](crate::fun), or a customized [`FunctionDecl`] when you need
    /// `.name(...)` or per-function overrides.
    pub fn fun(mut self, decl: FunctionDecl) -> Self {
        self.functions.push(decl);
        self
    }

    /// Add a `#[prebindgen]` const to this package: a top-level Kotlin `val`
    /// in the package file, initialized through a generated nullary JNI
    /// getter. Take a bare name via [`constant!`](crate::constant), or a
    /// customized [`ConstDecl`] when you need `.name(...)`.
    pub fn constant(mut self, decl: ConstDecl) -> Self {
        self.constants.push(decl);
        self
    }

    /// Add a **function-backed constant** to this package: a **nullary**
    /// `#[prebindgen]` fn whose result surfaces as an eagerly-initialized
    /// top-level Kotlin `val` (computed once, at package-file class-load,
    /// through the ordinary generated wrapper) instead of a callable `fun`.
    /// Use it for constant values a Rust `const` cannot express — e.g. a
    /// string only obtainable through a runtime `Display`.
    ///
    /// `.name(...)` sets the val name; the default is the fn ident verbatim
    /// (you almost always want an explicit SCREAMING_SNAKE name). The same
    /// restrictions as [`Self::constant`] apply to the return type
    /// (opaque-handle results are rejected), the fn must take no parameters,
    /// and expand overrides are meaningless here — both are hard errors.
    pub fn constant_fun(mut self, decl: FunctionDecl) -> Self {
        assert!(
            decl.param_expands.is_empty() && decl.return_expand.is_none(),
            "constant_fun `{}`: expand overrides don't apply to a constant — \
             declare a plain `FunctionDecl` (optionally with `.name(...)`)",
            decl.rust_ident
        );
        self.constant_functions.push(decl);
        self
    }

    /// Add an **expression-backed constant** to this package: an arbitrary
    /// binding-defined Rust expression evaluated once (at package-file
    /// class-load) inside a generated nullary JNI getter, surfacing as an
    /// eagerly-initialized top-level Kotlin `val`. See [`ConstExprDecl`];
    /// build one with [`constant_expr!`](crate::constant_expr) or
    /// [`ConstExprDecl::new`].
    pub fn constant_expr(mut self, decl: ConstExprDecl) -> Self {
        self.constant_exprs.push(decl);
        self
    }
}

// ──────────────────────────────────────────────────────────────────────
// Convert decl — the canonical single-value conversion for a type
// ──────────────────────────────────────────────────────────────────────

/// Declares a type's **canonical single-value conversion**: how one value of
/// the type crosses the boundary wherever a single value is needed — as a
/// parameter or return, inside `Option<_>` / `Vec<_>` / the `Result<T, E>`
/// success position, as a `data_class` field. The conversion is a pair of
/// ordinary `#[prebindgen]` functions (no injected Rust expressions):
///
/// ```rust,ignore
/// .convert(convert!(Millis)
///     .input(fun!(millis_from_long))   // fn(u64) -> Millis    (wire → rust)
///     .output(fun!(millis_value)))     // fn(&Millis) -> u64   (rust → wire)
/// ```
///
/// The Kotlin surface derives from the conversion functions' other-side type
/// (`u64` ⇒ `Long`) — nothing is stated verbatim. An input function may be
/// fallible (`fn(U) -> Result<T, E>`): an `Err` routes to the caller's error
/// handler. The functions may live in the flat crate or in a **helper
/// crate** ingested as an extra source ([`crate::core::Registry::from_sources`]);
/// generated calls qualify each function with its origin crate.
///
/// Distinct from the [`expand_param!`](crate::expand_param) /
/// [`expand_return!`](crate::expand_return) boundary decls: those reshape a
/// **function boundary** into multiple leaves (variants in / fields out),
/// while `convert!` defines the type's one-value form used everywhere else.
/// A type may declare both — expansion wins at the fn boundaries where it is
/// declared; the conversion serves every other position. The method names
/// differ deliberately: converters are direction-things ([`input`](Self::input)
/// also serves callback returns, [`output`](Self::output) also serves
/// callback arguments), while expansion decls are position-things.
#[derive(Clone)]
pub struct ConvertDecl {
    pub(crate) key: TypeKey,
    pub(crate) input: Option<syn::Ident>,
    pub(crate) output: Option<syn::Ident>,
}

impl ConvertDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        reject_builtin_convert_type(&TypeKey::from_type(&rust_type));
        Self {
            key: TypeKey::from_type(&rust_type),
            input: None,
            output: None,
        }
    }

    /// The **into-Rust** conversion (parameters, callback returns): a
    /// `#[prebindgen]` `fn(U) -> T` or `fn(U) -> Result<T, E>` where `T` is
    /// this decl's type. `U` (taken by value or `&U`) determines the wire and
    /// the Kotlin surface through its own converter chain.
    pub fn input(mut self, rust_fun: FunctionDecl) -> Self {
        assert!(
            rust_fun.kotlin_name_override.is_none()
                && rust_fun.param_expands.is_empty()
                && rust_fun.return_expand.is_none(),
            "convert!({}).input({}): a conversion function is never surfaced in Kotlin — \
             .name()/expand overrides don't apply",
            self.key.as_str(),
            rust_fun.rust_ident
        );
        self.input = Some(rust_fun.rust_ident);
        self
    }

    /// The **out-of-Rust** conversion (returns, callback arguments): a
    /// `#[prebindgen]` `fn(&T) -> U` (or `fn(T) -> U`) where `T` is this
    /// decl's type — the counterpart of [`input`](Self::input).
    pub fn output(mut self, rust_fun: FunctionDecl) -> Self {
        assert!(
            rust_fun.kotlin_name_override.is_none()
                && rust_fun.param_expands.is_empty()
                && rust_fun.return_expand.is_none(),
            "convert!({}).output({}): a conversion function is never surfaced in Kotlin — \
             .name()/expand overrides don't apply",
            self.key.as_str(),
            rust_fun.rust_ident
        );
        self.output = Some(rust_fun.rust_ident);
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
