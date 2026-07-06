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

/// One arm of a `.flatten_input()`/`.flatten_input_with()` build-from list.
#[derive(Clone)]
pub(crate) enum LocalVariant {
    /// Build via this declared constructor member / constructor fn.
    Ctor(syn::Ident),
    /// Accept an already-built value directly.
    SelfIdentity,
}

/// One arm of a `.flatten_output()`/`.flatten_output_with()` field list.
#[derive(Clone)]
pub(crate) enum LocalField {
    /// Include the named accessor's value as this leaf/field name.
    Named(syn::Ident, String),
    /// Include the handle itself as a field.
    SelfField,
}

/// Push `rust_fun` onto `members` as a member of the given `kind`, shared by
/// every class-kind decl's `.accessor()`/`.method()`/`.constructor()`. The
/// Kotlin-visible name comes from `rust_fun.name(...)` if set, else defaults
/// to `snake_to_camel(rust_ident)` — the same default `PackageDecl::fun` uses.
fn push_member(members: &mut Vec<ClassMember>, rust_fun: FunctionDecl, kind: MemberKind) {
    let kotlin_name = rust_fun
        .kotlin_name_override
        .unwrap_or_else(|| snake_to_camel(&rust_fun.rust_ident.to_string()));
    members.push(ClassMember {
        rust_ident: rust_fun.rust_ident,
        kotlin_name,
        kind,
    });
}

/// Resolve a member `name` of the given kind to its Rust ident, or panic with
/// a clear build-script message. Shared by every class-kind decl's
/// `.variant(name)`/`.field(name)` lookups.
fn resolve_member(members: &[ClassMember], name: &str, kind: MemberKind, verb: &str) -> syn::Ident {
    members
        .iter()
        .find(|m| m.kind == kind && m.kotlin_name == name)
        .unwrap_or_else(|| {
            let what = match kind {
                MemberKind::Accessor => ".accessor",
                MemberKind::Constructor => ".constructor",
                MemberKind::Method => ".method",
            };
            panic!(
                "{verb}(\"{name}\"): no `{what}(.., \"{name}\")` declared on this class before \
                 referencing it here"
            )
        })
        .rust_ident
        .clone()
}

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

/// Build an empty [`FlattenInputDecl`] — shorthand for `FlattenInputDecl::new()`.
#[macro_export]
macro_rules! flatten_input {
    () => {
        $crate::lang::FlattenInputDecl::new()
    };
}

/// Build an empty [`FlattenOutputDecl`] — shorthand for `FlattenOutputDecl::new()`.
#[macro_export]
macro_rules! flatten_output {
    () => {
        $crate::lang::FlattenOutputDecl::new()
    };
}

/// Build an empty [`FunctionFlattenInputDecl`] — shorthand for
/// `FunctionFlattenInputDecl::new()`.
#[macro_export]
macro_rules! function_flatten_input {
    () => {
        $crate::lang::FunctionFlattenInputDecl::new()
    };
}

/// Build an empty [`FunctionFlattenOutputDecl`] — shorthand for
/// `FunctionFlattenOutputDecl::new()`.
#[macro_export]
macro_rules! function_flatten_output {
    () => {
        $crate::lang::FunctionFlattenOutputDecl::new()
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
        $crate::lang::GenericTypeWrapperDecl::new($crate::__macro_support::parse_type(stringify!($t)))
    };
}

// ──────────────────────────────────────────────────────────────────────
// Class-kind decls
// ──────────────────────────────────────────────────────────────────────

/// Declares a typed Kotlin handle class backed by an opaque Rust type.
/// Configures: jlong wire for both input and output, `Box::into_raw`/
/// `Box::from_raw` lifecycle, the `instanceof` dispatch class, and the Kotlin
/// typed-handle class FQN. Feed it to a [`PackageDecl`] (via [`ClassDecl`])
/// which in turn is handed to [`JniGen::package`].
///
/// `.flatten_output`/`.flatten_input` take a
/// [`FlattenOutputDecl`]/[`FlattenInputDecl`] spec built independently:
///
/// ```
/// let _ = prebindgen::ptr_class!(ZThing)
///     .accessor(prebindgen::fun!(z_thing_name).name("name"))
///     .flatten_output(prebindgen::flatten_output!().field_self().field("name"));
/// ```
///
/// `.field()`/`.field_self()` only exist on [`FlattenOutputDecl`], not on
/// `PtrClassDecl` itself, so there's no way to call them before a
/// `.flatten_output(...)` exists to resolve them against:
///
/// ```compile_fail
/// let _ = prebindgen::ptr_class!(ZThing)
///     .accessor(prebindgen::fun!(z_thing_name).name("name"))
///     .field("name"); // no such method on `PtrClassDecl`
/// ```
pub struct PtrClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) members: Vec<ClassMember>,
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

    /// Override the Kotlin **class name** (relative, no dots — the FQN is
    /// derived from the [`PackageDecl`] this class is declared in). Used
    /// literally; the `kotlin_ptr_class_name_mangle` hook does not apply.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Declare a `#[prebindgen]` **read accessor** (`f(&Self) -> R`) as an
    /// instance method `name`. Usable as a `.flatten_output()` `.field(name)`.
    pub fn accessor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Accessor);
        self
    }

    /// Declare a `#[prebindgen]` **method** (`f(&Self, …) -> R`) as an
    /// instance method `name`. Not usable as a flatten field.
    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Method);
        self
    }

    /// Declare a `#[prebindgen]` **constructor** (`f(…) -> Self` /
    /// `Result<Self, E>`) as a companion-object factory `name`. Referenceable
    /// from `.flatten_input().variant(name)`.
    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Constructor);
        self
    }

    /// Set this class's default **input flatten**: how a parameter of this
    /// class type is assembled at the boundary. `decl` is built independently
    /// via [`FlattenInputDecl::new`] + `.variant(name)` (build via a declared
    /// `.constructor`) / `.variant_self()` (accept the handle directly), then
    /// resolved against this class's declared members here.
    pub fn flatten_input(mut self, decl: FlattenInputDecl) -> Self {
        self.input_variants = Some(
            decl.variants
                .into_iter()
                .map(|v| match v {
                    NamedVariant::Ctor(name) => {
                        let func =
                            resolve_member(&self.members, &name, MemberKind::Constructor, "variant");
                        LocalVariant::Ctor(func)
                    }
                    NamedVariant::SelfIdentity => LocalVariant::SelfIdentity,
                })
                .collect(),
        );
        self
    }

    /// Set this class's default **output flatten**: how a returned/callback
    /// value of this class is decomposed into fields. `decl` is built
    /// independently via [`FlattenOutputDecl::new`] + `.field(name)` (a
    /// declared `.accessor`'s value) / `.field_self()` (the handle itself),
    /// then resolved against this class's declared members here.
    pub fn flatten_output(mut self, decl: FlattenOutputDecl) -> Self {
        self.output_fields = Some(
            decl.fields
                .into_iter()
                .map(|f| match f {
                    NamedField::Acc(name) => {
                        let func = resolve_member(&self.members, &name, MemberKind::Accessor, "field");
                        LocalField::Named(func, name)
                    }
                    NamedField::SelfField => LocalField::SelfField,
                })
                .collect(),
        );
        self
    }
}

/// Standalone spec for [`PtrClassDecl::flatten_input`], built independently
/// (`FlattenInputDecl::new().variant("of").variant_self()`, or
/// `prebindgen::flatten_input!().variant("of").variant_self()`) and handed in
/// as a value — `.variant()`/`.variant_self()` only exist on this type, so
/// there is no way to call them before a `.flatten_input()` exists to resolve
/// them against.
pub struct FlattenInputDecl {
    variants: Vec<NamedVariant>,
}

pub(crate) enum NamedVariant {
    Ctor(String),
    SelfIdentity,
}

impl FlattenInputDecl {
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
        }
    }

    /// Build via the constructor declared as `name` (see
    /// [`PtrClassDecl::constructor`]).
    pub fn variant(mut self, name: impl Into<String>) -> Self {
        self.variants.push(NamedVariant::Ctor(name.into()));
        self
    }

    /// Accept an already-built handle directly (the identity variant).
    pub fn variant_self(mut self) -> Self {
        self.variants.push(NamedVariant::SelfIdentity);
        self
    }
}

impl Default for FlattenInputDecl {
    fn default() -> Self {
        Self::new()
    }
}

/// Standalone spec for [`PtrClassDecl::flatten_output`] — the output-side
/// dual of [`FlattenInputDecl`].
pub struct FlattenOutputDecl {
    fields: Vec<NamedField>,
}

pub(crate) enum NamedField {
    Acc(String),
    SelfField,
}

impl FlattenOutputDecl {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Include the value of the accessor declared as `name` (see
    /// [`PtrClassDecl::accessor`]).
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.fields.push(NamedField::Acc(name.into()));
        self
    }

    /// Include the handle itself as a field.
    pub fn field_self(mut self) -> Self {
        self.fields.push(NamedField::SelfField);
        self
    }
}

impl Default for FlattenOutputDecl {
    fn default() -> Self {
        Self::new()
    }
}

impl From<syn::Type> for PtrClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a `#[prebindgen]`-marked `enum` as a Kotlin `enum class`. The
/// enum must be C-like (unit variants only) and `#[repr(i32)]`-alike with
/// explicit discriminants — the Kotlin emitter and the generated
/// `TryFrom<i32>` decode rely on the discriminant values matching the jint
/// wire.
pub struct EnumClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) members: Vec<ClassMember>,
}

impl EnumClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            members: Vec::new(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    pub fn accessor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Accessor);
        self
    }

    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Method);
        self
    }

    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Constructor);
        self
    }
}

impl From<syn::Type> for EnumClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a Rust struct that should appear in Kotlin as a `data class`.
/// Only affects Kotlin emission — no Rust-side converter override.
pub struct DataClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) kotlin_type: Option<String>,
    pub(crate) members: Vec<ClassMember>,
}

impl DataClassDecl {
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

    /// Stamp a verbatim Kotlin type expression (e.g. `"List<ByteArray>"`)
    /// instead of a class FQN — for generics/primitives/container types.
    pub fn kotlin_type(mut self, expr: impl Into<String>) -> Self {
        self.kotlin_type = Some(expr.into());
        self
    }

    pub fn accessor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Accessor);
        self
    }

    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Method);
        self
    }

    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Constructor);
        self
    }
}

impl From<syn::Type> for DataClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a **`Copy` value class** type: a Rust type passed across the JNI
/// boundary **by value as its raw memory bytes** in a `ByteArray`, rather
/// than as a closeable `jlong` heap handle — the value-level peer of
/// [`PtrClassDecl`]. The type **must be `Copy`** — the generator emits a
/// compile-time assertion to that effect.
pub struct ValueClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) kotlin_type: Option<String>,
    pub(crate) members: Vec<ClassMember>,
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

    /// Stamp a verbatim Kotlin type expression instead of a class FQN.
    pub fn kotlin_type(mut self, expr: impl Into<String>) -> Self {
        self.kotlin_type = Some(expr.into());
        self
    }

    pub fn accessor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Accessor);
        self
    }

    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Method);
        self
    }

    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        push_member(&mut self.members, rust_fun, MemberKind::Constructor);
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

/// Declares a `#[prebindgen]` function as a free-standing package wrapper.
/// Feed it to a [`PackageDecl`] via [`PackageDecl::fun`].
pub struct FunctionDecl {
    pub(crate) rust_ident: syn::Ident,
    pub(crate) kotlin_name_override: Option<String>,
    pub(crate) input_suppressed: Vec<syn::Ident>,
    pub(crate) input_overrides: Vec<(syn::Ident, Vec<LocalVariant>)>,
    pub(crate) output_suppressed: bool,
    pub(crate) output_override: Option<Vec<LocalField>>,
}

impl FunctionDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            input_suppressed: Vec::new(),
            input_overrides: Vec::new(),
            output_suppressed: false,
            output_override: None,
        }
    }

    /// Override the Kotlin-side function name. Default (without `.name(...)`)
    /// is `snake_to_camel(rust_ident)`.
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        self.kotlin_name_override = Some(kotlin_name.into());
        self
    }

    /// `param` skips input-flattening and takes the raw handle.
    pub fn flatten_input_suppress(mut self, param: syn::Ident) -> Self {
        self.input_suppressed.push(param);
        self
    }

    /// Replace the default input flatten of `param` with an explicit variant
    /// list. `decl` is built independently via
    /// [`FunctionFlattenInputDecl::new`] + `.variant(fn)` (build-from
    /// constructor fns) / `.variant_self()` (accept the handle directly). May
    /// be called more than once, for different params.
    pub fn flatten_input_with(mut self, param: syn::Ident, decl: FunctionFlattenInputDecl) -> Self {
        self.input_overrides.push((param, decl.variants));
        self
    }

    /// The return value skips output-flattening and stays a raw handle.
    pub fn flatten_output_suppress(mut self) -> Self {
        self.output_suppressed = true;
        self
    }

    /// Replace the default output flatten with an explicit field list.
    /// `decl` is built independently via [`FunctionFlattenOutputDecl::new`] +
    /// `.field(fn)` (accessor fns, named via `fn.name(...)`) /
    /// `.field_self()` (the handle itself).
    pub fn flatten_output_with(mut self, decl: FunctionFlattenOutputDecl) -> Self {
        self.output_override = Some(decl.fields);
        self
    }
}

/// Standalone spec for [`FunctionDecl::flatten_input_with`], built
/// independently and handed in as a value — the per-param dual of
/// [`FlattenInputDecl`], except `.variant(func)` names the `#[prebindgen]`
/// constructor's Rust ident **directly** (a free function has no declared
/// member list to resolve a name against, unlike a class).
pub struct FunctionFlattenInputDecl {
    variants: Vec<LocalVariant>,
}

impl FunctionFlattenInputDecl {
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
        }
    }

    /// Build via this `#[prebindgen]` constructor function directly.
    pub fn variant(mut self, func: FunctionDecl) -> Self {
        self.variants.push(LocalVariant::Ctor(func.rust_ident));
        self
    }

    /// Accept an already-built handle directly (the identity variant).
    pub fn variant_self(mut self) -> Self {
        self.variants.push(LocalVariant::SelfIdentity);
        self
    }
}

impl Default for FunctionFlattenInputDecl {
    fn default() -> Self {
        Self::new()
    }
}

/// Standalone spec for [`FunctionDecl::flatten_output_with`] — the
/// per-function output-side dual of
/// [`FunctionFlattenInputDecl`]/[`FlattenOutputDecl`].
pub struct FunctionFlattenOutputDecl {
    fields: Vec<LocalField>,
}

impl FunctionFlattenOutputDecl {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Include the value of the accessor fn `func` (the rust accessor fn
    /// directly) as a field named via `func.name(...)` (default:
    /// `snake_to_camel(rust_ident)`, same as everywhere else a `FunctionDecl`
    /// supplies its own Kotlin-visible name).
    pub fn field(mut self, func: FunctionDecl) -> Self {
        let name = func
            .kotlin_name_override
            .unwrap_or_else(|| snake_to_camel(&func.rust_ident.to_string()));
        self.fields.push(LocalField::Named(func.rust_ident, name));
        self
    }

    /// Include the handle itself as a field.
    pub fn field_self(mut self) -> Self {
        self.fields.push(LocalField::SelfField);
        self
    }
}

impl Default for FunctionFlattenOutputDecl {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────────────
// PackageDecl — aggregates the package-scoped decls
// ──────────────────────────────────────────────────────────────────────

/// A batch of class and function declarations under one Kotlin subpackage
/// (or the base package, for `PackageDecl::new("")`). Built independently of
/// `JniGen`, then handed to [`JniGen::package`], which **merges** it into
/// whatever that package already holds — so the same subpackage name may be
/// reopened across several `PackageDecl` values / `JniGen::package` calls.
pub struct PackageDecl {
    pub(crate) name: String,
    pub(crate) classes: Vec<ClassDecl>,
    pub(crate) functions: Vec<FunctionDecl>,
}

impl PackageDecl {
    /// `name` is dot-separated, relative to the base package set by
    /// [`JniGenConfig::package_prefix`]; the empty string is the base
    /// package itself. See [`crate::package!`] for the equivalent macro form
    /// (`package!("model")` / `package!()`).
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let name = name.trim_matches('.').trim_matches('/').to_string();
        Self {
            name,
            classes: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// Add a class declaration (any of [`PtrClassDecl`]/[`EnumClassDecl`]/
    /// [`DataClassDecl`]/[`ValueClassDecl`], via [`ClassDecl`]'s `From` impls).
    pub fn class(mut self, decl: impl Into<ClassDecl>) -> Self {
        self.classes.push(decl.into());
        self
    }

    /// Add a free-function declaration — a bare function ident via
    /// [`crate::fun!`] or a fully customized
    /// `FunctionDecl::new(prebindgen::ident!(ident)).name(...)....`.
    pub fn fun(mut self, decl: FunctionDecl) -> Self {
        self.functions.push(decl);
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

/// Declares how one concrete Rust type (`pattern`) crosses the JNI boundary
/// **as a custom scalar wire value** — the primitive-wire peer of
/// [`ValueClassDecl`] (which does the same job for `ByteArray`-backed
/// types). Global — not part of any [`PackageDecl`] (see the module doc for
/// why `kotlin_type` needs no package placement).
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
    pub(crate) input: Option<Arc<dyn Fn(&syn::Ident) -> syn::Expr + Send + Sync>>,
    pub(crate) output: Option<Arc<dyn Fn(&syn::Ident) -> syn::Expr + Send + Sync>>,
}

impl ScalarTypeWrapperDecl {
    /// `wire` is the one wire type shared by both directions; `kotlin_type`
    /// is the Kotlin-visible type this pattern surfaces as (e.g. `"Long"`) —
    /// required, since a scalar mapping has no sensible auto-derived name.
    /// See [`crate::scalar_type_wrapper!`] for the equivalent macro form.
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

    /// Build the wire → rust conversion body. `body` receives the ident of
    /// the wire-typed value in scope (`&wire`) and returns the Rust-typed
    /// expression to splice via `quote!`'s `#value` interpolation.
    pub fn input(mut self, body: impl Fn(&syn::Ident) -> syn::Expr + Send + Sync + 'static) -> Self {
        self.input = Some(Arc::new(body));
        self
    }

    /// Build the rust → wire conversion body. `body` receives the ident of
    /// the rust-typed value in scope and returns the wire-typed expression.
    pub fn output(mut self, body: impl Fn(&syn::Ident) -> syn::Expr + Send + Sync + 'static) -> Self {
        self.output = Some(Arc::new(body));
        self
    }
}

/// The result of one [`GenericTypeWrapperDecl`] `.input()`/`.output()`
/// builder: either a binding-fallible bare value (the framework wraps it
/// `Ok(...)` and any `?` inside routes to the framework's own error type), or
/// a domain-fallible `Result` whose `Err` routes to the per-call error sink
/// verbatim (the `Result<_, _>` peel is the one real user of this arm).
pub enum WireBody {
    Infallible(syn::Type, syn::Expr),
    Fallible(syn::Type, syn::Type, syn::Expr),
}

impl WireBody {
    pub fn infallible(wire: syn::Type, expr: syn::Expr) -> Self {
        Self::Infallible(wire, expr)
    }

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
        Arc::new(move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
            Some(self(&args[0], &wrapper_value_ident()).into_tuple())
        })
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
        Arc::new(move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
            Some(self(&args[0], &args[1], &wrapper_value_ident()).into_tuple())
        })
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
        Arc::new(move |args: &[syn::Type], _registry: &Registry<KotlinMeta>| {
            Some(self(&args[0], &args[1], &args[2], &wrapper_value_ident()).into_tuple())
        })
    }
    fn rank() -> usize {
        3
    }
}

/// Declares how an existing structural wrapper (`Option`/`Result`/`Vec`/…) is
/// peeled for one specific wildcard substitution — e.g. a per-error
/// `Result<_, ConcreteErr>` override of the framework's built-in
/// `Result<_, _>` peel. Declares nothing Kotlin-visible on its own (no
/// `.kotlin_type()` — a structural override never names a type). Global —
/// not part of any [`PackageDecl`].
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

    /// Register the input-direction (wire → rust) peel. The closure's arity
    /// (1–3 leading `&syn::Type` params, one per `_` in `pattern`, plus a
    /// trailing `&syn::Ident` for the value in scope) selects the rank.
    pub fn input<A, B: WrapperBuilder<A>>(mut self, builder: B) -> Self {
        self.input = Some((B::rank(), builder.into_wrapper_fn()));
        self
    }

    /// Output-direction (rust → wire) counterpart of [`Self::input`].
    pub fn output<A, B: WrapperBuilder<A>>(mut self, builder: B) -> Self {
        self.output = Some((B::rank(), builder.into_wrapper_fn()));
        self
    }
}
