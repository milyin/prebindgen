//! Declaration objects: one standalone, independently-constructible value
//! type per kind of thing `JniGen` can be told about (a `ptr_class`, an
//! `enum_class`, a function, a scalar wire mapping, …), plus the `PackageDecl`
//! that aggregates the package-scoped ones. Each type is both its own
//! "builder" and the final value `JniGen`/`PackageDecl` accepts — no separate
//! `Builder`/`Decl` split, no terminal `.build()` call.
//!
//! `JniGen` itself only ever *accepts* fully-built values of these types
//! (`JniGen::package`, `JniGen::scalar_type_wrapper`,
//! `JniGen::generic_type_wrapper`, in `builder.rs`); none of them reach back
//! into any `JniGen` state while being built.

use super::*;

// ──────────────────────────────────────────────────────────────────────
// Shared local accumulators (replayed into `Expansions`/`Deconstructors`
// by the accept logic in `builder.rs` once a decl is handed to `JniGen`)
// ──────────────────────────────────────────────────────────────────────

/// One arm of a `.default_param_expand*`/`.param_expand*` build-from list.
#[derive(Clone)]
pub(crate) enum LocalVariant {
    /// Build via this declared constructor member / constructor fn.
    Ctor(syn::Ident),
    /// Accept an already-built value directly.
    SelfIdentity,
}

/// One arm of a `.default_return_expand*`/`.return_expand*` field list.
#[derive(Clone)]
pub(crate) enum LocalField {
    /// Include the named accessor's value as this leaf/field name.
    Named(syn::Ident, String),
    /// Include the handle itself as a field.
    SelfField,
}

// Class members are stored as the full `(FunctionDecl, MemberKind)` pair —
// not a reduced ident+name record — so the `FunctionDecl`'s per-fn
// `.param_expand*`/`.return_expand*` overrides survive to `builder.rs`'s
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
/// is evaluated inside the generated getter with `use <source_module>::*;`
/// in scope.
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

/// Build a [`ScalarTypeWrapperDecl`] directly from bare Rust types:
/// `scalar_type_wrapper!(Millis, jni::sys::jlong, "Long")` is
/// `ScalarTypeWrapperDecl::new(<Millis as syn::Type>, <jni::sys::jlong as syn::Type>, "Long")`.
#[macro_export]
macro_rules! scalar_type_wrapper {
    ($pattern:ty, $wire:ty, $kotlin_type:expr) => {
        $crate::lang::ScalarTypeWrapperDecl::new(
            $crate::__macro_support::parse_type(stringify!($pattern)),
            $crate::__macro_support::parse_type(stringify!($wire)),
            $kotlin_type,
        )
    };
}

/// Build a [`GenericTypeWrapperDecl`] directly from a bare wildcard type
/// pattern: `generic_type_wrapper!(Result<_, ConcreteErr>)`.
#[macro_export]
macro_rules! generic_type_wrapper {
    ($t:ty) => {
        $crate::lang::GenericTypeWrapperDecl::new($crate::__macro_support::parse_type(stringify!(
            $t
        )))
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
/// Build one with [`ptr_class!`](crate::ptr_class), add it to a
/// [`PackageDecl`], and hand that to [`JniGen::package`].
///
/// A `PtrClassDecl` does two jobs:
///
/// 1. **Defines the Kotlin class** — its name ([`name`](Self::name)), its
///    instance methods ([`fun`](Self::fun)), and its companion-object
///    factories ([`constructor`](Self::constructor)).
/// 2. **Sets the type's default behavior at every FFI boundary** — how a
///    *parameter* of this type is accepted ([`default_param_expand`](Self::default_param_expand))
///    and how a *returned* or callback-delivered value of it is handed to
///    Kotlin ([`default_return_expand`](Self::default_return_expand)), for
///    *every* function that mentions the type. Any single function can
///    override its own copy of these defaults (see [`FunctionDecl`]).
///
/// ```
/// // A KeyExpr handle exposing `str()` as an instance method; by default a
/// // KeyExpr returned to Kotlin is delivered as just the handle.
/// let _ = prebindgen::ptr_class!(KeyExpr)
///     .fun(prebindgen::fun!(keyexpr_get_str).name("str"))
///     .default_return_expand_self();
/// ```
pub struct PtrClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
    pub(crate) input_variants: Option<Vec<LocalVariant>>,
    pub(crate) output_fields: Option<Vec<LocalField>>,
}

impl PtrClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            members: Vec::new(),
            input_variants: None,
            output_fields: None,
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
    /// arguments. A constructor can also serve as a build option in
    /// [`default_param_expand`](Self::default_param_expand).
    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Constructor));
        self
    }

    /// Let callers pass the **ingredients** of this type wherever a parameter
    /// of it is expected, instead of having to build the handle first. Point
    /// this at a `#[prebindgen]` constructor: at every such parameter, the
    /// Kotlin signature gains that constructor's inputs and Rust builds the
    /// value in the same call.
    ///
    /// For example, giving `KeyExpr` a `keyexpr_from_str(&str)` build option
    /// lets every function taking a `KeyExpr` also accept a plain `String`.
    ///
    /// Call repeatedly to offer several ways to build the value (and add
    /// [`default_param_expand_self`](Self::default_param_expand_self) to also
    /// accept a ready-made handle); with more than one option the generated
    /// Kotlin picks at runtime. Overridable per function via
    /// [`FunctionDecl::param_expand`].
    pub fn default_param_expand(mut self, rust_fun: FunctionDecl) -> Self {
        self.input_variants
            .get_or_insert_with(Vec::new)
            .push(LocalVariant::Ctor(rust_fun.rust_ident));
        self
    }

    /// Also accept an **already-built handle** at parameters of this type —
    /// the "…or just pass one you already have" option alongside the
    /// [`default_param_expand`](Self::default_param_expand) build variants.
    /// On its own this is simply the default (a bare handle), so declaring it
    /// alone changes nothing; it earns its place only next to build variants.
    pub fn default_param_expand_self(mut self) -> Self {
        self.input_variants
            .get_or_insert_with(Vec::new)
            .push(LocalVariant::SelfIdentity);
        self
    }

    /// Decompose this type into **named fields delivered in one FFI crossing**
    /// wherever it is returned or handed to a callback — instead of returning
    /// an opaque handle the caller must then query field by field with more
    /// JNI calls.
    ///
    /// Point each call at a `#[prebindgen]` reader (`f(&Self) -> Field`); its
    /// value becomes one field, named via `fun!(reader).name("field")`
    /// (default: the reader's camel-cased name). For example, decomposing a
    /// `Sample` into `keyExpr`, `payload`, `timestamp`, … hands a subscriber
    /// callback the whole sample in a single call with no follow-up accessor
    /// round-trips.
    ///
    /// Call repeatedly to add fields, and add
    /// [`default_return_expand_self`](Self::default_return_expand_self) to
    /// also include the live handle. Overridable per function via
    /// [`FunctionDecl::return_expand`].
    pub fn default_return_expand(mut self, rust_fun: FunctionDecl) -> Self {
        let name = rust_fun
            .kotlin_name_override
            .unwrap_or_else(|| snake_to_camel(&rust_fun.rust_ident.to_string()));
        self.output_fields
            .get_or_insert_with(Vec::new)
            .push(LocalField::Named(rust_fun.rust_ident, name));
        self
    }

    /// Include the **handle itself** among the decomposed fields, so the
    /// consumer gets a live, closeable object in addition to the read-out
    /// values (e.g. a `Query` delivered with its fields *and* the handle it
    /// needs to reply). Declare it **last**, after any field that decomposes
    /// a nested handle, so the generated Rust moves the value only after
    /// those borrows.
    pub fn default_return_expand_self(mut self) -> Self {
        self.output_fields
            .get_or_insert_with(Vec::new)
            .push(LocalField::SelfField);
        self
    }
}

impl From<syn::Type> for PtrClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
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
/// [`name`](Self::name) to set its Kotlin name. The `param_expand*` /
/// `return_expand*` methods **override, for this one function**, the
/// boundary defaults that its parameter/return types set on their
/// `ptr_class!` — most often to opt back out (a lone `_self`) so this
/// function sees the raw handle rather than the class's default expansion.
pub struct FunctionDecl {
    pub(crate) rust_ident: syn::Ident,
    pub(crate) kotlin_name_override: Option<String>,
    pub(crate) input_overrides: Vec<(syn::Ident, Vec<LocalVariant>)>,
    pub(crate) output_override: Option<Vec<LocalField>>,
}

impl FunctionDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            input_overrides: Vec::new(),
            output_override: None,
        }
    }

    /// Set the Kotlin-side name. Default: the Rust name camel-cased
    /// (`session_declare_publisher` → `sessionDeclarePublisher`).
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        self.kotlin_name_override = Some(kotlin_name.into());
        self
    }

    /// Override, for `param` of this function only, how that parameter is
    /// built — the same idea as [`PtrClassDecl::default_param_expand`], but
    /// scoped here and keyed by which parameter (a function may have several
    /// handle parameters, each overridden independently). Point at a
    /// constructor to offer it as a build option; call again (same or a
    /// different `param`) to add more.
    pub fn param_expand(mut self, param: syn::Ident, ctor: FunctionDecl) -> Self {
        self.input_override_entry(param)
            .push(LocalVariant::Ctor(ctor.rust_ident));
        self
    }

    /// Make `param` of this function accept **only a ready-made handle**,
    /// ignoring the build variants its type would otherwise apply here. Use
    /// it when one function needs the real object rather than something built
    /// from ingredients — e.g. *un*-declaring a key expression needs the
    /// handle, not a string.
    pub fn param_expand_self(mut self, param: syn::Ident) -> Self {
        self.input_override_entry(param)
            .push(LocalVariant::SelfIdentity);
        self
    }

    /// The variant list of `param`'s override, creating it on first use.
    fn input_override_entry(&mut self, param: syn::Ident) -> &mut Vec<LocalVariant> {
        let idx = match self.input_overrides.iter().position(|(p, _)| *p == param) {
            Some(i) => i,
            None => {
                self.input_overrides.push((param, Vec::new()));
                self.input_overrides.len() - 1
            }
        };
        &mut self.input_overrides[idx].1
    }

    /// Override this function's return decomposition — the same idea as
    /// [`PtrClassDecl::default_return_expand`], but for this function alone.
    /// Add one field per call.
    pub fn return_expand(mut self, field: FunctionDecl) -> Self {
        let name = field
            .kotlin_name_override
            .unwrap_or_else(|| snake_to_camel(&field.rust_ident.to_string()));
        self.output_override
            .get_or_insert_with(Vec::new)
            .push(LocalField::Named(field.rust_ident, name));
        self
    }

    /// Return this function's result as the **raw handle**, overriding the
    /// decomposition its return type would otherwise apply. Also the right
    /// choice for a borrowed return (`&T` / `Option<&T>`), which crosses by
    /// cloning into a fresh owned handle.
    pub fn return_expand_self(mut self) -> Self {
        self.output_override
            .get_or_insert_with(Vec::new)
            .push(LocalField::SelfField);
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
/// runs with `use <source_module>::*;` in scope, so it composes the source
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
    /// and flatten overrides are meaningless here — both are hard errors.
    pub fn constant_fun(mut self, decl: FunctionDecl) -> Self {
        assert!(
            decl.input_overrides.is_empty() && decl.output_override.is_none(),
            "constant_fun `{}`: flatten overrides don't apply to a constant — \
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
// Wrapper decls — split by rank (see the module doc for why)
// ──────────────────────────────────────────────────────────────────────

/// Rejects a wrapper registration on a Rust **builtin** type: the generated
/// converter qualifies the pattern with the `source_module`
/// (`myflat::usize`), which does not compile. Wrap the builtin in a
/// source-crate newtype (like `Millis(u64)`) instead. Only ever relevant for
/// rank-0 (a builtin type has no wildcards), so this is called from
/// [`ScalarTypeWrapperDecl::new`] alone.
fn reject_builtin_wrapper_pattern(key: &TypeKey) {
    const BUILTINS: &[&str] = &[
        "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
        "f32", "f64", "bool", "char", "str",
    ];
    assert!(
        !BUILTINS.contains(&key.as_str()),
        "ScalarTypeWrapperDecl on builtin `{}`: the generated converter qualifies the pattern \
         with the source module, which is invalid for builtins — wrap the builtin in a newtype \
         instead",
        key.as_str()
    );
}

/// The fixed identifier every wrapper body sees in scope for "the value being
/// converted" (`env: &mut JNIEnv` is NOT provided — confirmed zero real
/// wrapper bodies, built-in or user, ever need it; see the module doc).
pub(crate) fn wrapper_value_ident() -> syn::Ident {
    syn::Ident::new("v", Span::call_site())
}

/// A [`ScalarTypeWrapperDecl`] conversion body: given the in-scope value
/// ident, produce the conversion expression.
pub(crate) type ScalarConvFn = Arc<dyn Fn(&syn::Ident) -> syn::Expr + Send + Sync>;

/// Teaches the generator to carry one Rust type across the boundary as a
/// **plain scalar** — e.g. a `Millis(u64)` newtype that should surface in
/// Kotlin as a `Long`, converted with your own expressions each way, with no
/// generated class. The scalar peer of [`ValueClassDecl`] (which carries
/// `Copy` types as `ByteArray`). Register it with
/// [`JniGen::scalar_type_wrapper`]; it applies wherever the type appears, in
/// any package.
///
/// Build one with [`scalar_type_wrapper!`](crate::scalar_type_wrapper), then
/// give it [`on_param`](Self::on_param) / [`on_return`](Self::on_return)
/// conversions.
pub struct ScalarTypeWrapperDecl {
    pub(crate) pattern: syn::Type,
    // Stored as tokenized source text, not `syn::Type`: this workspace's
    // proc-macro2 feature resolution makes `syn::Type`/`syn::Expr` `!Send`/
    // `!Sync` (it unconditionally wraps the compiler's non-Send
    // `proc_macro::TokenStream` variant even outside a real proc-macro
    // expansion), so an owned one can't be captured into the `Send + Sync`
    // `WrapperFn` closure `JniGen::scalar_type_wrapper` builds from this —
    // re-parsed fresh at lookup time instead.
    pub(crate) wire: String,
    pub(crate) kotlin_type: String,
    pub(crate) input: Option<ScalarConvFn>,
    pub(crate) output: Option<ScalarConvFn>,
}

impl ScalarTypeWrapperDecl {
    /// `pattern` is the Rust type being mapped, `wire` is the primitive it
    /// travels as (e.g. `jni::sys::jlong`), and `kotlin_type` is how it shows
    /// up in Kotlin (e.g. `"Long"`). See
    /// [`scalar_type_wrapper!`](crate::scalar_type_wrapper) for the macro
    /// shorthand.
    pub fn new(pattern: syn::Type, wire: syn::Type, kotlin_type: impl Into<String>) -> Self {
        reject_builtin_wrapper_pattern(&TypeKey::from_type(&pattern));
        Self {
            pattern,
            wire: quote!(#wire).to_string(),
            kotlin_type: kotlin_type.into(),
            input: None,
            output: None,
        }
    }

    /// How to turn the incoming **wire value into the Rust value** (used when
    /// the type is a parameter). `body` gets the wire value's ident and
    /// returns the Rust expression, e.g.
    /// `|v| pq!(perftest_flat::Millis(*#v as u64))`.
    pub fn on_param(
        mut self,
        body: impl Fn(&syn::Ident) -> syn::Expr + Send + Sync + 'static,
    ) -> Self {
        self.input = Some(Arc::new(body));
        self
    }

    /// How to turn the **Rust value into the wire value** (used when the type
    /// is returned). `body` gets the Rust value's ident and returns the wire
    /// expression, e.g. `|v| pq!(#v.0 as jni::sys::jlong)`.
    pub fn on_return(
        mut self,
        body: impl Fn(&syn::Ident) -> syn::Expr + Send + Sync + 'static,
    ) -> Self {
        self.output = Some(Arc::new(body));
        self
    }
}

/// What a [`GenericTypeWrapperDecl`] conversion produces: the wire type plus
/// the expression, and whether the conversion can fail with a **domain
/// error** the caller should see. Use [`infallible`](Self::infallible) when
/// it always succeeds, or [`fallible`](Self::fallible) to route an `Err` to
/// the caller's error handler (as the built-in `Result` unwrap does).
// large_enum_variant: a transient codegen-time value immediately destructured
// by `into_tuple`; boxing the `syn` payloads would only complicate the public
// variant shape.
#[allow(clippy::large_enum_variant)]
pub enum WireBody {
    Infallible(syn::Type, syn::Expr),
    Fallible(syn::Type, syn::Type, syn::Expr),
}

impl WireBody {
    /// The conversion always succeeds. `wire` is the wire type, `expr` the
    /// conversion expression.
    pub fn infallible(wire: syn::Type, expr: syn::Expr) -> Self {
        Self::Infallible(wire, expr)
    }

    /// The conversion may fail: `expr` evaluates to `Result<wire, error>`,
    /// and an `Err` is delivered to the caller's error handler.
    pub fn fallible(wire: syn::Type, error: syn::Type, expr: syn::Expr) -> Self {
        Self::Fallible(wire, error, expr)
    }

    pub(crate) fn into_tuple(self) -> (syn::Type, Option<syn::Type>, syn::Expr) {
        match self {
            Self::Infallible(wire, expr) => (wire, None, expr),
            Self::Fallible(wire, err, expr) => (wire, Some(err), expr),
        }
    }
}

/// Trait selecting the arity-appropriate impl of
/// [`GenericTypeWrapperDecl::input`] / [`GenericTypeWrapperDecl::output`].
/// The phantom type parameter discriminates closures of arity 1..3 so a
/// single public method name accepts any of them. Closures take the wildcard
/// substitutions plus the in-scope value ident, and return a [`WireBody`].
pub trait WrapperBuilder<Arity>: Send + Sync + 'static {
    fn into_wrapper_fn(self) -> WrapperFn;
    fn rank() -> usize;
}

/// Arity-discriminating marker types. `Arity1`/`2`/`3` carry that many `_`
/// slots in the registered pattern (e.g. `Result<_, _>` is `Arity2`).
pub(crate) struct Arity1;
pub(crate) struct Arity2;
pub(crate) struct Arity3;

impl<F> WrapperBuilder<Arity1> for F
where
    F: Fn(&syn::Type, &syn::Ident) -> WireBody + Send + Sync + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(
            move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                Some(self(&args[0], &wrapper_value_ident()).into_tuple())
            },
        )
    }
    fn rank() -> usize {
        1
    }
}

impl<F> WrapperBuilder<Arity2> for F
where
    F: Fn(&syn::Type, &syn::Type, &syn::Ident) -> WireBody + Send + Sync + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(
            move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                Some(self(&args[0], &args[1], &wrapper_value_ident()).into_tuple())
            },
        )
    }
    fn rank() -> usize {
        2
    }
}

impl<F> WrapperBuilder<Arity3> for F
where
    F: Fn(&syn::Type, &syn::Type, &syn::Type, &syn::Ident) -> WireBody + Send + Sync + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(
            move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
                Some(self(&args[0], &args[1], &args[2], &wrapper_value_ident()).into_tuple())
            },
        )
    }
    fn rank() -> usize {
        3
    }
}

/// An **advanced** override for how a generic wrapper (`Option`/`Result`/
/// `Vec`/…) is unwrapped for one specific inner type — e.g. handle
/// `Result<_, MyError>` your own way instead of through the built-in
/// `Result` support. The `pattern` carries `_` wildcards for the parts that
/// stay generic. Register it with [`JniGen::generic_type_wrapper`]; it names
/// no Kotlin type of its own and belongs to no package.
///
/// Build one with [`generic_type_wrapper!`](crate::generic_type_wrapper),
/// then supply [`input`](Self::input) / [`output`](Self::output).
pub struct GenericTypeWrapperDecl {
    pub(crate) pattern: syn::Type,
    pub(crate) input: Option<(usize, WrapperFn)>,
    pub(crate) output: Option<(usize, WrapperFn)>,
}

impl GenericTypeWrapperDecl {
    /// `pattern` contains 1–3 `_` wildcard placeholders (e.g.
    /// `Result<_, ConcreteErr>`). See [`crate::generic_type_wrapper!`] for the
    /// equivalent macro form.
    pub fn new(pattern: syn::Type) -> Self {
        Self {
            pattern,
            input: None,
            output: None,
        }
    }

    /// How to convert **into Rust** (used when the type is a parameter). The
    /// closure receives one `&syn::Type` per `_` in `pattern` (so its arity
    /// tells the generator how many wildcards there are), plus the value's
    /// ident, and returns a [`WireBody`].
    pub fn input<A, B: WrapperBuilder<A>>(mut self, builder: B) -> Self {
        self.input = Some((B::rank(), builder.into_wrapper_fn()));
        self
    }

    /// How to convert **out of Rust** (used when the type is returned) — the
    /// counterpart of [`input`](Self::input).
    pub fn output<A, B: WrapperBuilder<A>>(mut self, builder: B) -> Self {
        self.output = Some((B::rank(), builder.into_wrapper_fn()));
        self
    }
}
