//! The declaration vocabulary a build script writes.
//!
//! Every declaration is plain data: a macro builds it, builder methods
//! refine it, and [`JniGenBuilder`](crate::JniGenBuilder) collects it. None of
//! it is resolved until [`JniGenBuilder::build`](crate::JniGenBuilder::build).

use std::{ops::RangeInclusive, sync::Arc};

use prebindgen_tools::{Conversion, FnRef, Via};

// ── functions ───────────────────────────────────────────────────────────

/// A function reference plus how it is exported: `fun!(storage_new)`,
/// `fun!(crate::local).sig(sig!(..))`.
#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub(crate) fun: FnRef,
    pub(crate) name: Option<String>,
    pub(crate) splits: Vec<String>,
    pub(crate) params: Vec<(String, ExpandParamDecl)>,
    pub(crate) ret: Option<ExpandReturnDecl>,
}

impl FunctionDecl {
    pub fn new(path: syn::Path) -> Self {
        Self {
            fun: FnRef::new(path),
            name: None,
            splits: Vec::new(),
            params: Vec::new(),
            ret: None,
        }
    }

    /// State the signature of a binding-local function.
    pub fn sig(mut self, sig: syn::Signature) -> Self {
        self.fun = self.fun.sig(sig);
        self
    }

    /// The Kotlin name, verbatim (no mangling).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Emit typed overloads for each variant of this parameter's input
    /// expansion, beside the selector form.
    pub fn split_on_param(mut self, param: impl Into<String>) -> Self {
        self.splits.push(param.into());
        self
    }

    /// Replace the type-level input expansion for one parameter.
    pub fn expand_param(mut self, param: impl Into<String>, decl: ExpandParamDecl) -> Self {
        self.params.push((param.into(), decl));
        self
    }

    /// Replace the type-level output expansion for the result.
    pub fn expand_return(mut self, decl: ExpandReturnDecl) -> Self {
        self.ret = Some(decl);
        self
    }

    pub(crate) fn rust_name(&self) -> String {
        prebindgen_tools::names::bare(self.fun.name())
    }
}

impl From<FunctionDecl> for Via {
    fn from(f: FunctionDecl) -> Self {
        Via::Fn(f.fun)
    }
}

// ── conversions ─────────────────────────────────────────────────────────

/// A type crossing as another type: `convert!(Millis).input(..).output(..)`.
#[derive(Clone, Debug)]
pub struct ConvertDecl {
    pub(crate) conversion: Conversion,
    pub(crate) range: Option<(u128, u128)>,
}

impl ConvertDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            conversion: Conversion::new(ty),
            range: None,
        }
    }

    pub fn input(mut self, via: impl Into<Via>) -> Self {
        self.conversion = self.conversion.input(via);
        self
    }

    pub fn output(mut self, via: impl Into<Via>) -> Self {
        self.conversion = self.conversion.output(via);
        self
    }

    /// The representation's valid domain: a value outside it is a binding
    /// error in either direction.
    pub fn valid_range<T: Into<u128> + Copy>(mut self, range: RangeInclusive<T>) -> Self {
        self.range = Some(((*range.start()).into(), (*range.end()).into()));
        self
    }
}

// ── expansions ──────────────────────────────────────────────────────────

/// How a parameter of a type may be supplied: built from a constructor's
/// arguments, or passed as an existing handle — selected at run time.
#[derive(Clone, Debug)]
pub struct ExpandParamDecl {
    pub(crate) ty: syn::Type,
    pub(crate) variants: Vec<ParamVariant>,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)] // build-script data, a handful per binding
pub(crate) enum ParamVariant {
    Build(FunctionDecl),
    Handle,
}

impl ExpandParamDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            variants: Vec::new(),
        }
    }

    /// A variant built by calling `ctor` with its own arguments.
    pub fn variant(mut self, ctor: FunctionDecl) -> Self {
        self.variants.push(ParamVariant::Build(ctor));
        self
    }

    /// A variant passing an existing handle.
    pub fn variant_self(mut self) -> Self {
        self.variants.push(ParamVariant::Handle);
        self
    }
}

/// How a result of a type is delivered: as a list of fields handed to a
/// builder (or callback, or error handler) instead of the value itself.
#[derive(Clone, Debug)]
pub struct ExpandReturnDecl {
    pub(crate) ty: syn::Type,
    pub(crate) fields: Vec<ReturnField>,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)] // build-script data, a handful per binding
pub(crate) enum ReturnField {
    /// A field computed by a getter.
    Getter(FunctionDecl),
    /// The value itself, as a handle.
    Handle,
    /// Every field of a value form, by a function returning a struct;
    /// `consume` when the function takes the value by value.
    Form { fun: syn::Ident, consume: bool },
}

impl ExpandReturnDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            fields: Vec::new(),
        }
    }

    /// A field computed by `getter` (named after it, or its `.name()`).
    pub fn field(mut self, getter: FunctionDecl) -> Self {
        self.fields.push(ReturnField::Getter(getter));
        self
    }

    /// The value itself, handed over as a handle.
    pub fn field_self(mut self) -> Self {
        self.fields.push(ReturnField::Handle);
        self
    }

    /// Every field of the struct `form` returns from a borrow of the value.
    pub fn fields(mut self, form: FieldsDecl) -> Self {
        self.fields.push(ReturnField::Form {
            fun: form.0,
            consume: false,
        });
        self
    }

    /// Every field of the struct `form` returns by consuming the value.
    pub fn fields_self_into(mut self, form: FieldsDecl) -> Self {
        self.fields.push(ReturnField::Form {
            fun: form.0,
            consume: true,
        });
        self
    }
}

/// A value-form accessor, for [`ExpandReturnDecl::fields`]: `fields!(f)`.
#[derive(Clone, Debug)]
pub struct FieldsDecl(pub(crate) syn::Ident);

impl FieldsDecl {
    pub fn new(f: syn::Ident) -> Self {
        Self(f)
    }
}

/// An input or output expansion, for [`JniGenBuilder::expand`](crate::JniGenBuilder::expand).
#[derive(Clone, Debug)]
pub enum ExpandDecl {
    Param(ExpandParamDecl),
    Return(ExpandReturnDecl),
}

impl From<ExpandParamDecl> for ExpandDecl {
    fn from(d: ExpandParamDecl) -> Self {
        ExpandDecl::Param(d)
    }
}

impl From<ExpandReturnDecl> for ExpandDecl {
    fn from(d: ExpandReturnDecl) -> Self {
        ExpandDecl::Return(d)
    }
}

// ── classes ─────────────────────────────────────────────────────────────

/// Interface options every class carries.
#[derive(Clone, Debug, Default)]
pub(crate) struct Iface {
    pub(crate) enabled: bool,
    pub(crate) name: Option<String>,
    pub(crate) implements: Vec<String>,
}

macro_rules! iface_methods {
    () => {
        /// Emit a `<Name>Api` interface for the class's public surface, and
        /// implement it.
        pub fn interface(mut self) -> Self {
            self.iface.enabled = true;
            self
        }

        /// Emit the interface under this name.
        pub fn interface_name(mut self, name: impl Into<String>) -> Self {
            self.iface.enabled = true;
            self.iface.name = Some(name.into());
            self
        }

        /// Make the class implement a hand-written interface (a Kotlin FQN).
        pub fn implements(mut self, fqn: impl Into<String>) -> Self {
            self.iface.implements.push(fqn.into());
            self
        }
    };
}

/// An opaque handle class: `ptr_class!(Storage)`.
#[derive(Clone, Debug)]
pub struct PtrClassDecl {
    pub(crate) ty: syn::Type,
    pub(crate) name: Option<String>,
    pub(crate) methods: Vec<FunctionDecl>,
    pub(crate) constructors: Vec<FunctionDecl>,
    pub(crate) gc: bool,
    pub(crate) iface: Iface,
}

impl PtrClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            methods: Vec::new(),
            constructors: Vec::new(),
            gc: false,
            iface: Iface::default(),
        }
    }

    /// The Kotlin class name, verbatim.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Free unreachable handles through a `Cleaner` as well as `close()`.
    pub fn gc_managed(mut self) -> Self {
        self.gc = true;
        self
    }

    /// An instance method: the function's first parameter is the receiver.
    pub fn method(mut self, f: FunctionDecl) -> Self {
        self.methods.push(f);
        self
    }

    /// A companion factory.
    pub fn constructor(mut self, f: FunctionDecl) -> Self {
        self.constructors.push(f);
        self
    }

    iface_methods!();
}

/// A Kotlin `data class`: `data_class!(Payload)`.
#[derive(Clone, Debug)]
pub struct DataClassDecl {
    pub(crate) ty: syn::Type,
    pub(crate) name: Option<String>,
    pub(crate) methods: Vec<FunctionDecl>,
    pub(crate) constructors: Vec<FunctionDecl>,
    pub(crate) packed: bool,
    pub(crate) iface: Iface,
}

impl DataClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            methods: Vec::new(),
            constructors: Vec::new(),
            packed: false,
            iface: Iface::default(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// An instance method: the receiver crosses as the value itself.
    pub fn method(mut self, f: FunctionDecl) -> Self {
        self.methods.push(f);
        self
    }

    pub fn constructor(mut self, f: FunctionDecl) -> Self {
        self.constructors.push(f);
        self
    }

    /// Cross an input of this class as one packed argument per JNI type
    /// rather than one argument per field — for values whose fields exceed
    /// the JVM's method-argument limit.
    pub fn jobject_input(mut self) -> Self {
        self.packed = true;
        self
    }

    iface_methods!();
}

/// A Kotlin `enum class`: `enum_class!(Priority)`.
#[derive(Clone, Debug)]
pub struct EnumClassDecl {
    pub(crate) ty: syn::Type,
    pub(crate) name: Option<String>,
    pub(crate) iface: Iface,
}

impl EnumClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            iface: Iface::default(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    iface_methods!();
}

/// A Kotlin `sealed interface` for a data-carrying enum:
/// `sealed_class!(Reading)`.
#[derive(Clone, Debug)]
pub struct SealedClassDecl {
    pub(crate) ty: syn::Type,
    pub(crate) name: Option<String>,
    pub(crate) variants: Vec<VariantDecl>,
    pub(crate) iface: Iface,
}

impl SealedClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            variants: Vec::new(),
            iface: Iface::default(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Per-variant options.
    pub fn variant(mut self, v: VariantDecl) -> Self {
        self.variants.push(v);
        self
    }

    iface_methods!();
}

/// Options for one alternative of a sealed class: `variant!(Labeled)`.
#[derive(Clone, Debug)]
pub struct VariantDecl {
    pub(crate) rust: String,
    pub(crate) name: Option<String>,
}

impl VariantDecl {
    pub fn new(rust: impl Into<String>) -> Self {
        Self {
            rust: rust.into(),
            name: None,
        }
    }

    /// The Kotlin class name of this alternative.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Any class declaration.
#[derive(Clone, Debug)]
pub enum ClassDecl {
    Ptr(PtrClassDecl),
    Data(DataClassDecl),
    Enum(EnumClassDecl),
    Sealed(SealedClassDecl),
}

impl From<PtrClassDecl> for ClassDecl {
    fn from(d: PtrClassDecl) -> Self {
        ClassDecl::Ptr(d)
    }
}
impl From<DataClassDecl> for ClassDecl {
    fn from(d: DataClassDecl) -> Self {
        ClassDecl::Data(d)
    }
}
impl From<EnumClassDecl> for ClassDecl {
    fn from(d: EnumClassDecl) -> Self {
        ClassDecl::Enum(d)
    }
}
impl From<SealedClassDecl> for ClassDecl {
    fn from(d: SealedClassDecl) -> Self {
        ClassDecl::Sealed(d)
    }
}

// ── constants ───────────────────────────────────────────────────────────

/// A top-level Kotlin `val`: `constant!(MAX)`.
#[derive(Clone, Debug)]
pub struct ConstDecl {
    pub(crate) name: syn::Ident,
    pub(crate) source: ConstSource,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)] // build-script data, a handful per binding
pub(crate) enum ConstSource {
    /// The `#[prebindgen]` const of the same name.
    Const,
    /// A nullary function's result.
    Fun(FunctionDecl),
    /// A binding-local nullary function by path, with its type.
    With(syn::Type, syn::Path),
    /// A binding-local expression, with its type.
    Expr(syn::Type, syn::Expr),
}

impl ConstDecl {
    pub fn new(name: syn::Ident) -> Self {
        Self {
            name,
            source: ConstSource::Const,
        }
    }

    /// The value of a nullary `#[prebindgen]` function.
    pub fn fun(mut self, f: FunctionDecl) -> Self {
        self.source = ConstSource::Fun(f);
        self
    }

    /// The value of a binding-local nullary function.
    pub fn with(mut self, ty: syn::Type, path: syn::Path) -> Self {
        self.source = ConstSource::With(ty, path);
        self
    }

    /// The value of an expression evaluated in the generated crate, with
    /// the source crate's items in scope.
    pub fn expr(mut self, ty: syn::Type, expr: syn::Expr) -> Self {
        self.source = ConstSource::Expr(ty, expr);
        self
    }
}

// ── packages ────────────────────────────────────────────────────────────

/// Classes, functions and constants under one Kotlin subpackage:
/// `package!()` (the base) or `package!("model")`.
#[derive(Clone, Debug, Default)]
pub struct PackageDecl {
    pub(crate) name: String,
    pub(crate) classes: Vec<ClassDecl>,
    pub(crate) funs: Vec<FunctionDecl>,
    pub(crate) consts: Vec<ConstDecl>,
}

impl PackageDecl {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn class(mut self, c: impl Into<ClassDecl>) -> Self {
        self.classes.push(c.into());
        self
    }

    pub fn fun(mut self, f: FunctionDecl) -> Self {
        self.funs.push(f);
        self
    }

    pub fn constant(mut self, c: ConstDecl) -> Self {
        self.consts.push(c);
        self
    }
}

// ── ignores ─────────────────────────────────────────────────────────────

/// Items acknowledged as deliberately unbound.
#[derive(Clone)]
pub struct IgnoreDecl(pub(crate) Arc<dyn Fn(&str) -> bool + Send + Sync>);

impl std::fmt::Debug for IgnoreDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IgnoreDecl(..)")
    }
}

impl From<FunctionDecl> for IgnoreDecl {
    fn from(f: FunctionDecl) -> Self {
        let name = f.rust_name();
        IgnoreDecl(Arc::new(move |n| n == name))
    }
}

/// Ignore every item whose name satisfies `f`.
pub fn matching<F>(f: F) -> IgnoreDecl
where
    F: Fn(&str) -> bool + Send + Sync + 'static,
{
    IgnoreDecl(Arc::new(f))
}

// ── macros ──────────────────────────────────────────────────────────────

/// `fun!(name)` / `fun!(crate::path)` — a [`FunctionDecl`].
#[macro_export]
macro_rules! fun {
    ($($path:tt)+) => {
        $crate::FunctionDecl::new($crate::__syn::parse_quote!($($path)+))
    };
}

/// `convert!(T)` — a [`ConvertDecl`].
#[macro_export]
macro_rules! convert {
    ($($ty:tt)+) => {
        $crate::ConvertDecl::new($crate::__syn::parse_quote!($($ty)+))
    };
}

/// `ptr_class!(T)` — a [`PtrClassDecl`].
#[macro_export]
macro_rules! ptr_class {
    ($t:ty) => {
        $crate::PtrClassDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `data_class!(T)` — a [`DataClassDecl`].
#[macro_export]
macro_rules! data_class {
    ($t:ty) => {
        $crate::DataClassDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `enum_class!(T)` — an [`EnumClassDecl`].
#[macro_export]
macro_rules! enum_class {
    ($t:ty) => {
        $crate::EnumClassDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `sealed_class!(T)` — a [`SealedClassDecl`].
#[macro_export]
macro_rules! sealed_class {
    ($t:ty) => {
        $crate::SealedClassDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `variant!(V)` — a [`VariantDecl`].
#[macro_export]
macro_rules! variant {
    ($name:ident) => {
        $crate::VariantDecl::new(stringify!($name))
    };
}

/// `constant!(NAME)` — a [`ConstDecl`].
#[macro_export]
macro_rules! constant {
    ($name:ident) => {
        $crate::ConstDecl::new($crate::__syn::parse_quote!($name))
    };
}

/// `package!()` / `package!("sub")` — a [`PackageDecl`].
#[macro_export]
macro_rules! package {
    () => {
        $crate::PackageDecl::new("")
    };
    ($name:expr) => {
        $crate::PackageDecl::new($name)
    };
}

/// `expand_param!(T)` — an [`ExpandParamDecl`].
#[macro_export]
macro_rules! expand_param {
    ($t:ty) => {
        $crate::ExpandParamDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `expand_return!(T)` — an [`ExpandReturnDecl`].
#[macro_export]
macro_rules! expand_return {
    ($t:ty) => {
        $crate::ExpandReturnDecl::new($crate::__syn::parse_quote!($t))
    };
}

/// `fields!(value_form_fn)` — a [`FieldsDecl`].
#[macro_export]
macro_rules! fields {
    ($name:ident) => {
        $crate::FieldsDecl::new($crate::__syn::parse_quote!($name))
    };
}
