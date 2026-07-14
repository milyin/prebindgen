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

/// Build a [`ConstDecl`] from a bare ident: `constant!(MAX_LEN)` is
/// `ConstDecl::new(prebindgen::ident!(MAX_LEN))` — with no source modifier
/// it declares the same-named `#[prebindgen]` const; `.fun(…)` / `.with(…)`
/// / `.expr(…)` switch the value source (the ident stays as the `val` name).
#[macro_export]
macro_rules! constant {
    ($name:ident) => {
        $crate::lang::ConstDecl::new($crate::ident!($name))
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

/// Build a `syn::Type` from a bare Rust type token: `ty!(i32)`. The type
/// argument of decl methods like [`ConvertDecl::input_from`] — always yields
/// the concrete `syn::Type`, so no inference context is needed (see
/// [`ident!`](crate::ident) for the E0283 background).
#[macro_export]
macro_rules! ty {
    ($t:ty) => {
        $crate::__macro_support::parse_type(stringify!($t))
    };
}

/// Build a `syn::Path` from a bare path token: `path!(crate::conv::f)`. The
/// callable argument of [`ConvertDecl::input_with`] /
/// [`ConvertDecl::output_with`] and [`ConstDecl::with`].
#[macro_export]
macro_rules! path {
    ($p:path) => {
        $crate::__macro_support::parse_path(stringify!($p))
    };
}

/// Build a `syn::Expr` from an expression token: `expr!(format!("{A}:{B}"))`.
/// The initializer argument of [`ConstDecl::expr`] — allowed only for
/// constants, where the expression binds no arguments.
#[macro_export]
macro_rules! expr {
    ($e:expr) => {
        $crate::__macro_support::parse_expr(stringify!($e))
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

/// A [`ConstDecl`]'s **value source** — where the constant's value comes
/// from. Mirrors `convert!`'s source vocabulary at the nullary edge:
/// prebindgen item (bare) / prebindgen fn (`.fun`) / binding-local named fn
/// (`.with`) / expression (`.expr` — const-only: an expression binds no
/// arguments only when there is no value flowing in).
// Build-time declaration object, a handful per binding — the Expr variant's
// size is irrelevant, same trade-off as `ConvertSpec`.
#[allow(clippy::large_enum_variant)]
pub(crate) enum ConstSource {
    /// The same-named `#[prebindgen]` const (the bare `constant!(X)` form).
    Item,
    /// A **nullary** `#[prebindgen]` fn; the value type is read from its
    /// registry signature and the result flows through the ordinary
    /// generated wrapper, consumed as an eager `val`.
    Fun(syn::Ident),
    /// A binding-defined initializer expression with a **stated** value
    /// type, evaluated once inside a generated nullary JNI getter (with a
    /// glob import of every source module in scope). `.with(ty, path)`
    /// lowers here as `path()`.
    Expr { ty: syn::Type, expr: syn::Expr },
}

/// Declares one **constant** for emission: an eagerly-initialized top-level
/// Kotlin `val` in its package's `.kt` file, initialized through a generated
/// nullary JNI getter (the value type goes through the ordinary
/// output-converter machinery, exactly like a function return).
///
/// Build one with [`constant!`](crate::constant) — the ident is the `val`
/// name — and pick the value source:
///
/// ```rust,ignore
/// .constant(constant!(MAX_LEN))                          // #[prebindgen] const MAX_LEN
/// .constant(constant!(TAG_RUNTIME).fun(fun!(tag_runtime)))  // nullary #[prebindgen] fn
/// .constant(constant!(VERSION).with(ty!(String), path!(crate::version)))  // binding-local fn
/// .constant(constant!(BANNER).expr(ty!(String), expr!(format!("{A}:{B}"))))  // expression
/// ```
///
/// For declaration loops build the subject at runtime with
/// [`ConstDecl::named`]. Opaque-handle-typed (and `Result`-typed) constants
/// are rejected for every source — expose a factory function instead.
pub struct ConstDecl {
    /// Subject ident: the default `val` name; for the [`ConstSource::Item`]
    /// source also the `#[prebindgen]` const to look up.
    pub(crate) rust_ident: syn::Ident,
    pub(crate) kotlin_name_override: Option<String>,
    pub(crate) source: ConstSource,
}

impl ConstDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            source: ConstSource::Item,
        }
    }

    /// Runtime form of [`constant!`](crate::constant) for declaration
    /// loops: `ConstDecl::named(format!("ENCODING_{n}")).expr(ty, expr)`.
    /// The name must be a valid identifier (it seeds the extern symbol).
    pub fn named(name: impl AsRef<str>) -> Self {
        let name = name.as_ref();
        let ident: syn::Ident = syn::parse_str(name)
            .unwrap_or_else(|e| panic!("constant name `{name}` is not a valid identifier: {e}"));
        Self::new(ident)
    }

    /// Set the Kotlin-side `val` name. Default: the subject ident verbatim
    /// (`MAX_LEN` → `val MAX_LEN` — SCREAMING_SNAKE is the Kotlin constant
    /// convention too).
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        self.kotlin_name_override = Some(kotlin_name.into());
        self
    }

    /// The declared `val` name (override, else the subject ident).
    pub(crate) fn val_name(&self) -> String {
        self.kotlin_name_override
            .clone()
            .unwrap_or_else(|| self.rust_ident.to_string())
    }

    fn set_source(mut self, source: ConstSource) -> Self {
        assert!(
            matches!(self.source, ConstSource::Item),
            "constant `{}`: value source already set — a constant has exactly one source \
             (.fun / .with / .expr)",
            self.rust_ident
        );
        self.source = source;
        self
    }

    /// Value source: a **nullary** `#[prebindgen]` fn (e.g. a value a Rust
    /// `const` cannot express — a string only obtainable through a runtime
    /// `Display`). The value type is read from the fn's signature; the fn
    /// must take no parameters and must not return `Result`.
    pub fn fun(self, decl: FunctionDecl) -> Self {
        assert!(
            decl.param_expands.is_empty() && decl.return_expand.is_none(),
            "constant `{}`: expand overrides don't apply to a constant source fn `{}`",
            self.rust_ident,
            decl.rust_ident
        );
        assert!(
            decl.kotlin_name_override.is_none(),
            "constant `{}`: the val name belongs on `constant!(…)` (or its `.name(…)`), \
             not on the source fn `{}`",
            self.rust_ident,
            decl.rust_ident
        );
        self.set_source(ConstSource::Fun(decl.rust_ident))
    }

    /// Value source: a **binding-local nullary fn** named by path —
    /// `(stated value type, path)`, the const analog of
    /// [`ConvertDecl::input_with`]. The fn lives in the binding crate
    /// (callable because the generated file compiles inside it):
    /// `fn() -> T`.
    pub fn with(self, ty: syn::Type, path: syn::Path) -> Self {
        let expr: syn::Expr = syn::parse_quote!(#path());
        self.set_source(ConstSource::Expr { ty, expr })
    }

    /// Value source: a binding-defined **expression** with a stated value
    /// type, evaluated once inside the generated getter with a glob import
    /// of every source module in scope — so it composes source-crate
    /// `#[prebindgen]` items freely, e.g.
    /// `expr!(encoding_to_string(encoding_const_text_plain()))`. This
    /// source exists only for constants: an expression binds no arguments
    /// exactly when nothing flows in (a unary conversion source must be a
    /// named callable — see [`ConvertDecl`]). Fns referenced only inside
    /// expressions are undeclared to the registry — acknowledge them via
    /// [`JniGen::ignore_fun`] / [`JniGen::ignore_funs_where`].
    pub fn expr(self, ty: syn::Type, expr: syn::Expr) -> Self {
        self.set_source(ConstSource::Expr { ty, expr })
    }
}

/// Internal storage form of an expression-backed constant (the lowered
/// `.with` / `.expr` sources of [`ConstDecl`]).
#[derive(Clone)]
pub(crate) struct ConstExprDecl {
    pub(crate) kotlin_name: String,
    pub(crate) ty: syn::Type,
    pub(crate) expr: syn::Expr,
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

    /// Add a **constant** to this package: a top-level Kotlin `val` in the
    /// package file, initialized through a generated nullary JNI getter.
    /// Build the decl with [`constant!`](crate::constant) and pick its
    /// value source (`#[prebindgen]` const by default, `.fun` / `.with` /
    /// `.expr` otherwise) — see [`ConstDecl`].
    pub fn constant(mut self, decl: ConstDecl) -> Self {
        self.constants.push(decl);
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
///     .input_fun(fun!(millis_from_long))   // fn(u64) -> Millis    (wire → rust)
///     .output_fun(fun!(millis_value)))     // fn(&Millis) -> u64   (rust → wire)
/// ```
///
/// The Kotlin surface derives from the conversion functions' other-side type
/// (`u64` ⇒ `Long`) — nothing is stated verbatim. An input function may be
/// fallible (`fn(U) -> Result<T, E>`): an `Err` routes to the caller's error
/// handler. The functions may live in the flat crate or in a **helper
/// crate** whose item stream is chained into the same
/// [`crate::core::Registry::from_items`] call; generated calls qualify each
/// function with its origin crate.
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
/// One direction's conversion **source** — where the conversion code comes
/// from. Four kinds, one per [`ConvertDecl`] method family.
// large_enum_variant: a handful of these exist per binding, held once in the
// builder — boxing the syn payloads would only complicate the decl arms.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub(crate) enum ConvertSpec {
    /// A `#[prebindgen]` fn (flat or helper crate): the representable type
    /// and fallibility are read from its registry signature at lookup time.
    PrebindgenFn(syn::Ident),
    /// A `core::convert` trait impl; the representable type is stated
    /// explicitly (there is no signature to read). `fallible` selects
    /// `TryInto` (the associated `Error` routes to the caller's error
    /// handler) vs `Into`.
    Trait { repr: syn::Type, fallible: bool },
    /// An arbitrary callable path — typically a plain fn in the **binding
    /// crate itself** (the generated file compiles inside it, so
    /// `crate::…` paths resolve). Representable type stated explicitly;
    /// by-value both ways. `error: None` = infallible (`fn(Repr) -> T` /
    /// `fn(T) -> Repr`); `Some(E)` = the fn returns `Result<…, E>` and an
    /// `Err` routes to the caller's error handler (`E: Display`).
    LocalFn {
        repr: syn::Type,
        path: syn::Path,
        error: Option<syn::Type>,
    },
}

#[derive(Clone)]
pub struct ConvertDecl {
    pub(crate) key: TypeKey,
    pub(crate) input: Option<ConvertSpec>,
    pub(crate) output: Option<ConvertSpec>,
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

    fn set_input(mut self, spec: ConvertSpec) -> Self {
        assert!(
            self.input.is_none(),
            "convert!({}): the input conversion is already declared — pick ONE of \
             .input_fun()/.input_from()/.input_try_from()/.input_with()",
            self.key.as_str()
        );
        self.input = Some(spec);
        self
    }

    fn set_output(mut self, spec: ConvertSpec) -> Self {
        assert!(
            self.output.is_none(),
            "convert!({}): the output conversion is already declared — pick ONE of \
             .output_fun()/.output_into()/.output_try_into()/.output_with()",
            self.key.as_str()
        );
        self.output = Some(spec);
        self
    }

    fn plain_fn_ident(&self, dir: &str, rust_fun: FunctionDecl) -> syn::Ident {
        assert!(
            rust_fun.kotlin_name_override.is_none()
                && rust_fun.param_expands.is_empty()
                && rust_fun.return_expand.is_none(),
            "convert!({}).{dir}({}): a conversion function is never surfaced in Kotlin — \
             .name()/expand overrides don't apply",
            self.key.as_str(),
            rust_fun.rust_ident
        );
        rust_fun.rust_ident
    }

    fn check_repr(&self, method: &str, repr: &syn::Type) {
        assert!(
            TypeKey::from_type(repr) != self.key,
            "convert!({k}).{method}: the representable type must differ from `{k}` itself",
            k = self.key.as_str()
        );
    }

    /// The **into-Rust** conversion (parameters, callback returns) as a
    /// `#[prebindgen]` `fn(U) -> T` or `fn(U) -> Result<T, E>` where `T` is
    /// this decl's type. `U` (taken by value or `&U`) determines the wire and
    /// the Kotlin surface through its own converter chain.
    pub fn input_fun(self, rust_fun: FunctionDecl) -> Self {
        let ident = self.plain_fn_ident("input_fun", rust_fun);
        self.set_input(ConvertSpec::PrebindgenFn(ident))
    }

    /// The into-Rust conversion via `core::convert`: requires
    /// `Repr: Into<T>` (satisfied by an `impl From<Repr> for T` through the
    /// blanket). `repr` — stated explicitly, e.g. `ty!(i32)` — determines
    /// the wire and Kotlin surface through its own converter chain.
    pub fn input_from(self, repr: syn::Type) -> Self {
        self.check_repr("input_from", &repr);
        self.set_input(ConvertSpec::Trait {
            repr,
            fallible: false,
        })
    }

    /// The fallible into-Rust conversion via `core::convert`: requires
    /// `Repr: TryInto<T>` (satisfied by an `impl TryFrom<Repr> for T`). The
    /// associated `Error` must implement `Display`; an `Err` routes to the
    /// caller's error handler like any domain error.
    pub fn input_try_from(self, repr: syn::Type) -> Self {
        self.check_repr("input_try_from", &repr);
        self.set_input(ConvertSpec::Trait {
            repr,
            fallible: true,
        })
    }

    /// The into-Rust conversion as an arbitrary callable — typically a plain
    /// fn declared **in the binding crate itself** (no `#[prebindgen]`
    /// needed; the generated file compiles inside the binding crate, so
    /// `path!(crate::…)` resolves). Shape: `fn(Repr) -> T`, by value,
    /// infallible — see [`input_try_with`](Self::input_try_with) for the
    /// fallible form.
    pub fn input_with(self, repr: syn::Type, path: syn::Path) -> Self {
        self.check_repr("input_with", &repr);
        self.set_input(ConvertSpec::LocalFn {
            repr,
            path,
            error: None,
        })
    }

    /// The fallible into-Rust conversion as an arbitrary callable: shape
    /// `fn(Repr) -> Result<T, Error>`, by value. `error` is stated
    /// explicitly (a callable path carries no signature to read); it must
    /// implement `Display`, and an `Err` routes to the caller's error
    /// handler like any domain error.
    pub fn input_try_with(self, repr: syn::Type, error: syn::Type, path: syn::Path) -> Self {
        self.check_repr("input_try_with", &repr);
        self.set_input(ConvertSpec::LocalFn {
            repr,
            path,
            error: Some(error),
        })
    }

    /// The **out-of-Rust** conversion (returns, callback arguments) as a
    /// `#[prebindgen]` `fn(&T) -> U` (or `fn(T) -> U`) where `T` is this
    /// decl's type — the counterpart of [`input_fun`](Self::input_fun).
    pub fn output_fun(self, rust_fun: FunctionDecl) -> Self {
        let ident = self.plain_fn_ident("output_fun", rust_fun);
        self.set_output(ConvertSpec::PrebindgenFn(ident))
    }

    /// The out-of-Rust conversion via `core::convert`: requires
    /// `T: Into<Repr>` (satisfied by an `impl From<T> for Repr`).
    pub fn output_into(self, repr: syn::Type) -> Self {
        self.check_repr("output_into", &repr);
        self.set_output(ConvertSpec::Trait {
            repr,
            fallible: false,
        })
    }

    /// The fallible out-of-Rust conversion via `core::convert`: requires
    /// `T: TryInto<Repr>`; the associated `Error` (must be `Display`) routes
    /// to the caller's error handler.
    pub fn output_try_into(self, repr: syn::Type) -> Self {
        self.check_repr("output_try_into", &repr);
        self.set_output(ConvertSpec::Trait {
            repr,
            fallible: true,
        })
    }

    /// The out-of-Rust conversion as an arbitrary callable (see
    /// [`input_with`](Self::input_with)). Shape: `fn(T) -> Repr`, by value,
    /// infallible.
    pub fn output_with(self, repr: syn::Type, path: syn::Path) -> Self {
        self.check_repr("output_with", &repr);
        self.set_output(ConvertSpec::LocalFn {
            repr,
            path,
            error: None,
        })
    }

    /// The fallible out-of-Rust conversion as an arbitrary callable: shape
    /// `fn(T) -> Result<Repr, Error>`, by value (see
    /// [`input_try_with`](Self::input_try_with) for the error conventions).
    pub fn output_try_with(self, repr: syn::Type, error: syn::Type, path: syn::Path) -> Self {
        self.check_repr("output_try_with", &repr);
        self.set_output(ConvertSpec::LocalFn {
            repr,
            path,
            error: Some(error),
        })
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
