//! What a C binding declares.
//!
//! One [`Decls`] holds every declaration the binding makes: the types that
//! cross, the functions it exports, the callback signatures it accepts, and the
//! conversions it supplies. The set is **flat**, because an exported C function
//! is not a member of anything: `calculator_apply(calculator_t *, …)` is a free
//! function that happens to take a handle, and the header declares it beside
//! the type rather than inside it. Nothing here groups a function under a type,
//! because C has nothing for that to mean.
//!
//! What each declaration does carry is its own options — a handle's name base,
//! a function's permission to panic, a callback's takeable arguments — rather
//! than the builder carrying them for whatever was declared last. That is the
//! difference from calling the declarators one after another, where
//! `.base_name("value")` modified whichever declaration came before it, so
//! moving a line changed what it meant and an option could attach to the wrong
//! declaration with nothing to notice.
//!
//! ```
//! use prebindgen_c::{data_type, decls, enum_type, fun, ptr_type};
//!
//! let _ = decls!()
//!     .ptr_type(ptr_type!(Calculator))
//!     .data_type(data_type!(Drawing))
//!     .enum_type(enum_type!(Operation))
//!     .fun(fun!(calculator_apply))
//!     .fun(fun!(stamp_sum).abort_on_conversion_error());
//! ```

use prebindgen_registry::ConvertDecl;

/// One exported function.
///
/// `abort_on_conversion_error` is the permission a wrapper needs when a
/// conversion it performs can fail and it has no `Result` to report that
/// through — an input the binding must reject, or a result it cannot encode.
/// Without the permission such a function is a declaration error rather than a
/// wrapper that terminates the process, which is what a C caller would observe.
#[derive(Clone)]
pub struct FunDecl {
    pub(crate) ident: syn::Ident,
    pub(crate) base: Option<String>,
    pub(crate) abort_on_conversion_error: bool,
}

impl FunDecl {
    /// Declare the `#[prebindgen]` function with this Rust name.
    pub fn new(ident: syn::Ident) -> Self {
        FunDecl {
            ident,
            base: None,
            abort_on_conversion_error: false,
        }
    }

    /// Override the token the function-name mangler receives, in place of the
    /// Rust name.
    pub fn base_name(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    /// Allow this wrapper to terminate the process when a conversion fails and
    /// it has no error channel to report through.
    ///
    /// C observes termination, not a catchable Rust panic, which is why the
    /// permission is named for what the caller sees.
    pub fn abort_on_conversion_error(mut self) -> Self {
        self.abort_on_conversion_error = true;
        self
    }
}

/// A type crossing as an opaque handle: the value stays in Rust, C holds a
/// pointer, and a generated destructor frees it.
#[derive(Clone)]
pub struct PtrTypeDecl {
    pub(crate) ty: syn::Type,
    pub(crate) base: Option<String>,
}

/// A type crossing by value as a `#[repr(C)]` aggregate whose members are the
/// converted fields.
#[derive(Clone)]
pub struct DataTypeDecl {
    pub(crate) ty: syn::Type,
    pub(crate) base: Option<String>,
    pub(crate) error: bool,
}

/// A fieldless enum crossing as a C `enum`.
#[derive(Clone)]
pub struct EnumTypeDecl {
    pub(crate) ty: syn::Type,
    pub(crate) base: Option<String>,
}

/// A payload-carrying enum crossing as a `#[repr(C)]` tag plus union.
///
/// # Who frees a payload
///
/// The union crosses **by value**, like a [`data_type!`](crate::data_type). If
/// any variant's payload wire owns memory — a `char *`, an opaque pointer — the
/// declaration also produces a typed `<base>_drop` that frees the **active
/// arm**. An owning payload with no drop is a generation error rather than a
/// leak.
///
/// Ownership follows the wire, so a payload that is itself a declared data type
/// is owning when its own mirror has owning fields, even though that payload
/// crosses as a struct by value rather than as a pointer. The drop reaches
/// through it and releases each of them, nulling the slot so a second drop is a
/// no-op. The reach goes one level further: a struct payload's field may itself
/// be a declared tagged union with an owning arm, and the outer drop delegates
/// to that union's typed drop, which nulls what it frees — so idempotence
/// composes.
///
/// Nothing else can reach those bytes. At top level the data-type contract is
/// that C releases each owning field itself, but a union arm is not a top-level
/// field. One predicate decides both whether the drop is emitted and whether a
/// containing struct calls it, so a nested union cannot be freed through a
/// symbol that was never emitted.
///
/// The drop is a second C entry point into the same bytes, so it range-checks
/// the tag exactly as an inbound conversion does, and treats an out-of-range
/// one as nothing to release.
#[derive(Clone)]
pub struct TaggedUnionDecl {
    pub(crate) ty: syn::Type,
    pub(crate) base: Option<String>,
}

/// A type crossing by value as an opaque byte-struct of identical size and
/// alignment: no `Box`, the Rust value's bytes live inside the C struct.
#[derive(Clone)]
pub struct ValueTypeDecl {
    pub(crate) rust: syn::Type,
    pub(crate) opaque: syn::Type,
    pub(crate) owned: bool,
    pub(crate) base: Option<String>,
}

/// A type that is already `#[repr(C)]` in the source and crosses unchanged.
///
/// Its generated mirror takes the name the manglers give when the declarations
/// are applied, so the naming hooks have to be set on the builder before
/// [`CbindgenBuilder::declare`](crate::CbindgenBuilder::declare).
#[derive(Clone)]
pub struct ReprCTypeDecl {
    pub(crate) ty: syn::Type,
    pub(crate) assume_field_validity: bool,
    pub(crate) base: Option<String>,
}

/// An error type that is not a data struct: it reaches C as the message the
/// recorded accessor produces.
#[derive(Clone)]
pub struct ErrorTypeDecl {
    pub(crate) ty: syn::Type,
    pub(crate) message_fn: syn::Ident,
}

/// A callback signature the binding accepts, emitted as one `#[repr(C)]`
/// closure struct.
#[derive(Clone)]
pub struct CallbackDecl {
    pub(crate) ty: syn::Type,
    pub(crate) base: Option<String>,
    pub(crate) takeable: Vec<usize>,
}

macro_rules! named_decl {
    ($decl:ty) => {
        impl $decl {
            /// Override the token the name manglers receive for this
            /// declaration, in place of the one derived from the Rust name.
            pub fn base_name(mut self, base: impl Into<String>) -> Self {
                self.base = Some(base.into());
                self
            }
        }
    };
}
named_decl!(PtrTypeDecl);
named_decl!(DataTypeDecl);
named_decl!(EnumTypeDecl);
named_decl!(TaggedUnionDecl);
named_decl!(CallbackDecl);
named_decl!(ValueTypeDecl);
named_decl!(ReprCTypeDecl);

impl PtrTypeDecl {
    /// Declare a handle for this Rust type.
    pub fn new(ty: syn::Type) -> Self {
        PtrTypeDecl { ty, base: None }
    }
}

impl DataTypeDecl {
    /// Declare a by-value aggregate for this Rust type.
    pub fn new(ty: syn::Type) -> Self {
        DataTypeDecl {
            ty,
            base: None,
            error: false,
        }
    }

    /// Mark this type as one a `Result` may carry as its error.
    pub fn error(mut self) -> Self {
        self.error = true;
        self
    }
}

impl EnumTypeDecl {
    /// Declare a C `enum` for this fieldless Rust enum.
    pub fn new(ty: syn::Type) -> Self {
        EnumTypeDecl { ty, base: None }
    }
}

impl TaggedUnionDecl {
    /// Declare a tag-plus-union representation for this Rust enum.
    pub fn new(ty: syn::Type) -> Self {
        TaggedUnionDecl { ty, base: None }
    }
}

impl ValueTypeDecl {
    /// Declare that this Rust type crosses inside the given opaque
    /// counterpart, which must match it in size and alignment.
    pub fn new(rust: syn::Type, opaque: syn::Type) -> Self {
        ValueTypeDecl {
            rust,
            opaque,
            owned: true,
            base: None,
        }
    }

    /// Cross as data rather than as an owned value: C may copy it, and no
    /// gravestone is written back when it is moved out.
    pub fn data(mut self) -> Self {
        self.owned = false;
        self
    }
}

impl ReprCTypeDecl {
    /// Declare that this source type is already C-compatible and crosses as
    /// written.
    pub fn new(ty: syn::Type) -> Self {
        ReprCTypeDecl {
            ty,
            assume_field_validity: false,
            base: None,
        }
    }

    /// Accept that C may write this type's fields directly, so a field whose
    /// Rust type has invalid bit patterns — a `bool`, an enum — is trusted
    /// rather than normalized. The whole-struct reinterpret gives no hook to
    /// normalize one, so this is an acknowledgement, not a fix.
    pub fn assume_field_validity(mut self) -> Self {
        self.assume_field_validity = true;
        self
    }
}

impl ErrorTypeDecl {
    /// Declare an opaque error and the accessor that turns one into a message.
    pub fn new(ty: syn::Type, message_fn: syn::Ident) -> Self {
        ErrorTypeDecl { ty, message_fn }
    }
}

impl CallbackDecl {
    /// Declare a callback signature, written as `impl Fn(..) + Send + Sync + 'static`.
    pub fn new(ty: syn::Type) -> Self {
        CallbackDecl {
            ty,
            base: None,
            takeable: Vec::new(),
        }
    }

    /// Deliver this argument as a takeable owned pointer rather than by value.
    pub fn takeable_param(mut self, index: usize) -> Self {
        self.takeable.push(index);
        self
    }
}

/// A declared conversion between a source type and its wire form, with the
/// naming base its generated constants take.
#[derive(Clone)]
pub struct ConvertTypeDecl {
    pub(crate) decl: ConvertDecl,
    pub(crate) base: Option<String>,
}

named_decl!(ConvertTypeDecl);

impl ConvertTypeDecl {
    /// Declare the conversion.
    pub fn new(decl: ConvertDecl) -> Self {
        ConvertTypeDecl { decl, base: None }
    }
}

impl From<ConvertDecl> for ConvertTypeDecl {
    fn from(decl: ConvertDecl) -> Self {
        ConvertTypeDecl::new(decl)
    }
}

/// Everything one C binding declares, as one flat set.
///
/// Hand it to [`CbindgenBuilder::declare`](crate::CbindgenBuilder::declare).
#[derive(Clone, Default)]
pub struct Decls {
    pub(crate) ptr_types: Vec<PtrTypeDecl>,
    pub(crate) data_types: Vec<DataTypeDecl>,
    pub(crate) enum_types: Vec<EnumTypeDecl>,
    pub(crate) tagged_unions: Vec<TaggedUnionDecl>,
    pub(crate) value_types: Vec<ValueTypeDecl>,
    pub(crate) repr_c_types: Vec<ReprCTypeDecl>,
    pub(crate) error_types: Vec<ErrorTypeDecl>,
    pub(crate) callbacks: Vec<CallbackDecl>,
    pub(crate) converts: Vec<ConvertTypeDecl>,
    pub(crate) funs: Vec<FunDecl>,
    pub(crate) ignored_funs: Vec<syn::Ident>,
    pub(crate) ignored_types: Vec<syn::Type>,
}

impl Decls {
    /// An empty declaration set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an opaque-handle type.
    pub fn ptr_type(mut self, decl: PtrTypeDecl) -> Self {
        self.ptr_types.push(decl);
        self
    }

    /// Add a by-value aggregate type.
    pub fn data_type(mut self, decl: DataTypeDecl) -> Self {
        self.data_types.push(decl);
        self
    }

    /// Add a fieldless enum.
    pub fn enum_type(mut self, decl: EnumTypeDecl) -> Self {
        self.enum_types.push(decl);
        self
    }

    /// Add a payload-carrying enum.
    pub fn tagged_union(mut self, decl: TaggedUnionDecl) -> Self {
        self.tagged_unions.push(decl);
        self
    }

    /// Add a by-value opaque type and its counterpart.
    pub fn value_type(mut self, decl: ValueTypeDecl) -> Self {
        self.value_types.push(decl);
        self
    }

    /// Add a source type that is already `#[repr(C)]`.
    pub fn repr_c_type(mut self, decl: ReprCTypeDecl) -> Self {
        self.repr_c_types.push(decl);
        self
    }

    /// Add an opaque error type and its message accessor.
    pub fn error_type(mut self, decl: ErrorTypeDecl) -> Self {
        self.error_types.push(decl);
        self
    }

    /// Add a callback signature.
    pub fn callback(mut self, decl: CallbackDecl) -> Self {
        self.callbacks.push(decl);
        self
    }

    /// Add a declared conversion between a source type and its wire form.
    ///
    /// Takes the conversion itself, or one wrapped in a [`ConvertTypeDecl`]
    /// when its generated constants need a naming base of their own.
    pub fn convert(mut self, decl: impl Into<ConvertTypeDecl>) -> Self {
        self.converts.push(decl.into());
        self
    }

    /// Export a free function.
    pub fn fun(mut self, decl: FunDecl) -> Self {
        self.funs.push(decl);
        self
    }

    /// Record that a captured function is deliberately not exported.
    pub fn ignore_fun(mut self, ident: syn::Ident) -> Self {
        self.ignored_funs.push(ident);
        self
    }

    /// Record that a captured type is deliberately not exported.
    pub fn ignore_type(mut self, ty: syn::Type) -> Self {
        self.ignored_types.push(ty);
        self
    }
}

/// Build an empty [`Decls`]: `decls!()`.
#[macro_export]
macro_rules! decls {
    () => {
        $crate::Decls::new()
    };
}

/// Declare an exported function: `fun!(stamp_sum)`.
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::FunDecl::new($crate::ident!($name))
    };
}

/// Declare an opaque handle: `ptr_type!(Calculator)`.
#[macro_export]
macro_rules! ptr_type {
    ($ty:ty) => {
        $crate::PtrTypeDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare a by-value aggregate: `data_type!(Drawing)`.
#[macro_export]
macro_rules! data_type {
    ($ty:ty) => {
        $crate::DataTypeDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare a C enum: `enum_type!(Operation)`.
#[macro_export]
macro_rules! enum_type {
    ($ty:ty) => {
        $crate::EnumTypeDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare a tag-plus-union enum: `tagged_union!(Shape)`.
#[macro_export]
macro_rules! tagged_union {
    ($ty:ty) => {
        $crate::TaggedUnionDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare an already-`#[repr(C)]` source type: `repr_c_type!(Payload)`.
#[macro_export]
macro_rules! repr_c_type {
    ($ty:ty) => {
        $crate::ReprCTypeDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare a callback signature:
/// `callback!(impl Fn(f64) + Send + Sync + 'static)`.
#[macro_export]
macro_rules! callback {
    ($ty:ty) => {
        $crate::CallbackDecl::new($crate::syn::parse_quote!($ty))
    };
}

/// Declare a by-value opaque type and its counterpart:
/// `value_type!(Payload, PayloadOpaque)`.
#[macro_export]
macro_rules! value_type {
    ($rust:ty, $opaque:ty) => {
        $crate::ValueTypeDecl::new(
            $crate::syn::parse_quote!($rust),
            $crate::syn::parse_quote!($opaque),
        )
    };
}

/// Declare an opaque error and its message accessor:
/// `error_type!(Error, error_get_message)`.
#[macro_export]
macro_rules! error_type {
    ($ty:ty, $message_fn:ident) => {
        $crate::ErrorTypeDecl::new($crate::syn::parse_quote!($ty), $crate::ident!($message_fn))
    };
}
