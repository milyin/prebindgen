//! Declaration objects: one standalone, independently-constructible value
//! type per kind of thing `Declarations` can be told about (a `ptr_class`, an
//! `enum_class`, a function, a scalar wire mapping, …), plus the `PackageDecl`
//! that aggregates the package-scoped ones. Each type is both its own
//! "builder" and the final value `Declarations`/`PackageDecl` accepts — no separate
//! `Builder`/`Decl` split, no terminal `.build()` call.
//!
//! `Declarations` itself only ever *accepts* fully-built values of these types
//! (`JniGenBuilder::package`, `JniGenBuilder::expand`, `JniGenBuilder::convert`, in
//! `builder.rs`); none of them reach back
//! into any `Declarations` state while being built.

// Language-neutral declaration vocabulary moved to `api::core::decl` (it
// references only `TypeKey`/`Origin<syn::Type>`/plain `syn` types, nothing
// Kotlin/JNI-specific) — re-exported here so `jni`'s `pub use decl::*;` (and
// therefore `prebindgen::lang::*` / `prebindgen_registry::*`) is unaffected.
pub(crate) use prebindgen_registry::decl::{
    declared_origin, local_path_prefix, ConvertSpec, LocalField, LocalVariant,
};
pub use prebindgen_registry::decl::{
    ConvertDecl, ConvertSourceDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl,
    FunctionDecl,
};

use super::*;

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
        $crate::PtrClassDecl::new(prebindgen_registry::__macro_support::parse_type(
            stringify!($t),
        ))
    };
}

/// Build an [`EnumClassDecl`] directly from a bare Rust type. See [`ptr_class!`].
#[macro_export]
macro_rules! enum_class {
    ($t:ty) => {
        $crate::EnumClassDecl::new(prebindgen_registry::__macro_support::parse_type(
            stringify!($t),
        ))
    };
}

/// Build a [`SealedClassDecl`] directly from a bare Rust type. See
/// [`ptr_class!`].
#[macro_export]
macro_rules! sealed_class {
    ($t:ty) => {
        $crate::SealedClassDecl::new(prebindgen_registry::__macro_support::parse_type(
            stringify!($t),
        ))
    };
}

/// Build a [`VariantDecl`] from a bare variant ident, for
/// [`SealedClassDecl::variant`]: `variant!(PeriodicQueries).name("Periodic")`.
#[macro_export]
macro_rules! variant {
    ($name:ident) => {
        $crate::VariantDecl::new(stringify!($name))
    };
}

/// Build a [`DataClassDecl`] directly from a bare Rust type. See [`ptr_class!`].
#[macro_export]
macro_rules! data_class {
    ($t:ty) => {
        $crate::DataClassDecl::new(prebindgen_registry::__macro_support::parse_type(
            stringify!($t),
        ))
    };
}

/// Build a [`ConstDecl`] from a bare ident: `constant!(MAX_LEN)` is
/// `ConstDecl::new(prebindgen_registry::ident!(MAX_LEN))`.
///
/// What the ident names depends on where the decl lands:
/// * in `.constant(...)` it is always the Kotlin **`val` name**, and in the
///   **bare** form (no source modifier) it is *additionally* the lookup key
///   of the same-named `#[prebindgen]` const — `.fun(…)` / `.with(…)` /
///   `.expr(…)` replace that lookup with the stated value source (see the
///   four-source example on [`ConstDecl`](crate::ConstDecl));
/// * in `.ignore(constant!(X))` it is *only* the `#[prebindgen]` const
///   lookup key — nothing is emitted, so sources and `.name()` are rejected
///   there.
#[macro_export]
macro_rules! constant {
    ($name:ident) => {
        $crate::ConstDecl::new(prebindgen_registry::ident!($name))
    };
}

/// Build a [`PackageDecl`] directly: `package!("model")` is
/// `PackageDecl::new("model")`; `package!()` (no args) is the base package
/// (`PackageDecl::new("")`).
#[macro_export]
macro_rules! package {
    () => {
        $crate::PackageDecl::new("")
    };
    ($name:expr) => {
        $crate::PackageDecl::new($name)
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
/// as opposed to plain data you copy across ([`data_class!`](crate::data_class)).
///
/// A type that never materializes in Kotlin needs **no class declaration at
/// all**: give it boundary decls only ([`expand_param!`](prebindgen_registry::expand_param)
/// / [`expand_return!`](prebindgen_registry::expand_return)) and it stays rust-side-only —
/// built from ingredients on the way in, decomposed into fields on the way
/// out.
///
/// Build one with [`ptr_class!`](crate::ptr_class), add it to a
/// [`PackageDecl`], and hand that to [`JniGenBuilder::package`].
///
/// A `PtrClassDecl` defines the **Kotlin class only** — its name
/// ([`name`](Self::name)), its instance methods ([`method`](Self::method)), and its
/// companion-object factories ([`constructor`](Self::constructor)). How the
/// type crosses the FFI boundary by default — accepted as which parameter
/// variants, returned as which field set — is declared separately with
/// [`expand_param!`](prebindgen_registry::expand_param) / [`expand_return!`](prebindgen_registry::expand_return)
/// handed to [`JniGenBuilder::expand`]; any single
/// function can override those defaults locally (see [`FunctionDecl`]).
///
/// ```
/// // A KeyExpr handle exposing `str()` as an instance method.
/// let _ = prebindgen_jni::ptr_class!(KeyExpr)
///     .method(prebindgen_registry::fun!(keyexpr_get_str).name("str"));
/// ```
/// Deliberately has no verbatim type mapping: the generated typed-handle
/// class OWNS a lifecycle contract — the `NativeHandle` base, the `ptr`
/// slot, `close()`, the lock protocol, and the paired `freePtr` extern —
/// that an arbitrary existing Kotlin type cannot be assumed to honor.
/// Customize it from above instead: [`interface`](Self::interface) /
/// [`implements`](Self::implements).
pub struct PtrClassDecl {
    pub(crate) key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    pub(crate) rust_type: Origin<syn::Type>,
    pub(crate) name_override: Option<String>,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
    pub(crate) iface: IfaceOpts,
    pub(crate) gc_managed: bool,
}

/// The interface-related options every class decl carries (see
/// [`class_interface_methods!`]): the generated-interface switch + name
/// override, and the `.implements(...)` list. The two features are
/// orthogonal; used together, a user interface extends the generated one.
#[derive(Clone, Default)]
pub(crate) struct IfaceOpts {
    pub(crate) enabled: bool,
    pub(crate) name_override: Option<String>,
    pub(crate) implements: Vec<String>,
}

/// The three interface methods shared verbatim by all four class decls —
/// generated per decl so the panic messages name the right decl macro.
macro_rules! class_interface_methods {
    ($decl_macro:literal) => {
        /// Emit a generated Kotlin **interface** mirroring this class's
        /// public instance surface, and make the class implement it (every
        /// class-body member gains the `override` modifier). The interface
        /// is named by [`interface_name`](Self::interface_name), else the
        /// [`JniGenBuilder::set_interface_name_mangle`] hook over the final class
        /// name (default: append `"Api"`).
        ///
        /// This is the compiler-checked half of the integration hatch: a
        /// hand-written interface that *extends* the generated one can build
        /// default members over the class's real signatures — no
        /// hand-replication. Pair it with [`implements`](Self::implements)
        /// to attach that hand-written interface to the class. (For
        /// behavior-only injection, a Kotlin extension function needs no
        /// declaration at all.)
        pub fn interface(mut self) -> Self {
            self.iface.enabled = true;
            self
        }

        /// Name the generated interface literally (relative, no dots),
        /// bypassing the [`JniGenBuilder::set_interface_name_mangle`] hook.
        /// Implies [`interface`](Self::interface).
        pub fn interface_name(mut self, name: impl Into<String>) -> Self {
            let name = name.into();
            assert!(
                !name.trim().is_empty(),
                concat!($decl_macro, "!({}).interface_name(...): the name is empty"),
                self.key.as_str()
            );
            self.iface.enabled = true;
            self.iface.name_override = Some(name);
            self
        }

        /// Add a Kotlin **interface** to the generated class's supertype
        /// list — the class implements it *nominally*: its abstract members
        /// must be satisfied by the generated surface or carry default
        /// implementations. `iface` is an FQN (dotted names are imported and
        /// shortened) or a same-package name; call again to add several.
        ///
        /// Orthogonal to [`interface`](Self::interface) — but to abstract
        /// over the class's own members from your interface, enable the
        /// generated interface and make yours extend it (that is what turns
        /// mismatches into compile errors in YOUR file).
        pub fn implements(mut self, iface: impl Into<String>) -> Self {
            let iface = iface.into();
            assert!(
                !iface.trim().is_empty(),
                concat!(
                    $decl_macro,
                    "!({}).implements(...): the interface name is empty"
                ),
                self.key.as_str()
            );
            assert!(
                !self.iface.implements.contains(&iface),
                concat!(
                    $decl_macro,
                    "!({}).implements(\"{}\"): the interface is already declared"
                ),
                self.key.as_str(),
                iface
            );
            self.iface.implements.push(iface);
            self
        }
    };
}

impl PtrClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            name_override: None,
            members: Vec::new(),
            iface: IfaceOpts::default(),
            gc_managed: false,
        }
    }

    /// Make instances of this handle class **GC-managed**: an unreachable
    /// handle whose native box was not otherwise released is freed by a
    /// shared `java.lang.ref.Cleaner`.
    ///
    /// The pointer of a GC-managed handle lives in a separate atomic cell
    /// (tag bit and all) so the cleaner action can settle the release after
    /// the handle object itself is gone; the untagged→tagged transition is a
    /// CAS and doubles as the once-only free ticket — explicit `close()`
    /// frees eagerly, `take()`/by-value consumption void the ticket, the GC
    /// action frees only if it wins. Address bits still never change, so the
    /// lock-ordering key stays immutable, and `isClosed()`/Rust-side
    /// tagged-pointer guards are unchanged.
    ///
    /// Opt in for handles whose owner may never close them — value-like
    /// types (an `Encoding`) and long-lived resources where GC is the leak
    /// backstop behind an explicit `close()`. Leave hot-path per-message
    /// handles opted out: registration costs a few small allocations per
    /// instance.
    pub fn gc_managed(mut self) -> Self {
        self.gc_managed = true;
        self
    }

    /// Rename the generated Kotlin class. By default it is named after the
    /// Rust type (via the [`JniGenBuilder::set_ptr_class_name_mangle`] hook); `.name("Foo")`
    /// sets it literally instead. Relative name, no dots — the package comes
    /// from the enclosing [`PackageDecl`].
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    class_interface_methods!("ptr_class");

    /// Expose a `#[prebindgen]` method as a Kotlin **instance method** of this
    /// class. `rust_fun` must take `&Self` first — that receiver becomes
    /// Kotlin's `this` and drops out of the signature; any further parameters
    /// become the method's arguments. Name it with
    /// `fun!(rust_name).name("kotlinName")` (default: the Rust name
    /// camel-cased).
    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Method));
        self
    }

    /// Expose a `#[prebindgen]` factory as a Kotlin **companion-object
    /// factory** — callers write `Class.name(...)`. `rust_fun` returns `Self`
    /// (or `Result<Self, E>`) and its parameters become the factory's
    /// arguments. A constructor can also serve as a build option in a
    /// [`expand_param!`](prebindgen_registry::expand_param) variant list.
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

/// Declares a Rust C-like `enum` as a Kotlin `enum class`. The variants
/// cross the boundary as their `i32` discriminants and Kotlin gets a real
/// `enum class` with a `fromInt(...)` companion. The enum must be
/// unit-variant only and `#[repr(i32)]`-style with explicit discriminants,
/// so both sides agree on the numbers.
///
/// A **data-carrying** enum is a different Kotlin surface — a `sealed
/// interface` whose variants carry their payload — and is declared with
/// `sealed_class!` instead. Handing one to `enum_class!` is a hard error,
/// not a silent upgrade: the value would have to cross as a bare
/// discriminant, dropping the payload.
///
/// Has no `.method`/`.constructor` by rule, not omission: members belong to
/// class kinds whose instances can re-enter Rust as an object (handle /
/// blob / field leaves). An enum value is a bare scalar with no object
/// identity — a "method" on it is just a free function taking the enum.
pub struct EnumClassDecl {
    pub(crate) key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    pub(crate) rust_type: Origin<syn::Type>,
    pub(crate) name_override: Option<String>,
    pub(crate) iface: IfaceOpts,
}

impl EnumClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            name_override: None,
            iface: IfaceOpts::default(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    class_interface_methods!("enum_class");
}

impl From<syn::Type> for EnumClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// Declares a Rust **data-carrying** enum as a Kotlin `sealed interface`
/// whose variant classes are nested inside it — the surface a sum type gets
/// where the target language has sums natively.
///
/// ```ignore
/// .class(sealed_class!(RecoveryMode)
///     .variant(variant!(PeriodicQueries).name("Periodic")))
/// ```
///
/// ```kotlin
/// public sealed interface RecoveryMode {
///     public data class Periodic(val v0: Long) : RecoveryMode
///     public data object Heartbeat : RecoveryMode
///     public companion object { @JvmStatic public fun fromParts(…): RecoveryMode }
/// }
/// ```
///
/// A payload-less alternative becomes a `data object`; the variant classes
/// are **nested** so their names cannot collide package-wide. Tuple payload
/// fields surface as `v0`, `v1`, …; named fields keep their (camelCased)
/// names.
///
/// The counterpart of [`enum_class!`](crate::enum_class), which is for the
/// unit-variant-only case that crosses as a bare discriminant. Handing a
/// payload enum to `enum_class!` — or a fieldless one here — is a hard
/// error naming the other, never a silent upgrade.
///
/// Like `enum_class!` it has no `.method` / `.constructor`: a sum value has
/// no object identity Rust-side, so a "method" on it is a free function
/// taking it.
pub struct SealedClassDecl {
    pub(crate) key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    pub(crate) rust_type: Origin<syn::Type>,
    pub(crate) name_override: Option<String>,
    pub(crate) variants: Vec<VariantDecl>,
    pub(crate) iface: IfaceOpts,
}

impl SealedClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            name_override: None,
            variants: Vec::new(),
            iface: IfaceOpts::default(),
        }
    }

    /// Override the Kotlin **interface name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Configure one variant — currently its Kotlin class name. Undeclared
    /// variants keep their Rust ident; declaring a variant that the enum
    /// does not have is a hard error.
    pub fn variant(mut self, decl: VariantDecl) -> Self {
        self.variants.push(decl);
        self
    }

    class_interface_methods!("sealed_class");
}

impl From<syn::Type> for SealedClassDecl {
    fn from(rust_type: syn::Type) -> Self {
        Self::new(rust_type)
    }
}

/// One variant of a [`SealedClassDecl`]. Build it with
/// [`variant!`](crate::variant).
pub struct VariantDecl {
    pub(crate) rust_ident: String,
    pub(crate) name_override: Option<String>,
}

impl VariantDecl {
    pub fn new(rust_ident: impl Into<String>) -> Self {
        Self {
            rust_ident: rust_ident.into(),
            name_override: None,
        }
    }

    /// Override this variant's Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }
}

/// Declares a Rust struct as a Kotlin `data class`. Its fields cross the
/// boundary individually and Kotlin reassembles the object with a generated
/// `fromParts(...)` — no Rust-side heap object, no handle to close. Use this
/// for plain immutable data you copy across, as opposed to
/// [`ptr_class!`](crate::ptr_class) handles.
///
/// Members work like every class kind whose instance can re-enter Rust —
/// here the receiver re-enters as its **field leaves** (the same call-site
/// destructuring a data-class parameter gets), just rebased to `this`.
pub struct DataClassDecl {
    pub(crate) key: TypeKey,
    /// The type this declaration was **written with** — the `X` the macro
    /// received. Kept because the declaration is where it came from: recovering
    /// it later *from* the key was reasoning backwards from an identity (#291).
    pub(crate) rust_type: Origin<syn::Type>,
    pub(crate) name_override: Option<String>,
    pub(crate) jobject_input: bool,
    pub(crate) iface: IfaceOpts,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
}

impl DataClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            rust_type: declared_origin(rust_type),
            name_override: None,
            jobject_input: false,
            iface: IfaceOpts::default(),
            members: Vec::new(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    /// Explicitly keep this data class object-shaped on the Kotlin → Rust
    /// boundary. By default a data class must flatten completely into its
    /// transitive field leaves; generation fails rather than silently falling
    /// back to Rust-side `JObject` field reads. This escape hatch is intended
    /// for recursive/identity-bearing graphs, legacy ABI compatibility, or a
    /// deliberately chosen object boundary.
    ///
    /// When the marked type is nested inside an unmarked data class, only the
    /// marked branch crosses as a `JObject`; its siblings remain flattened.
    /// Rust → Kotlin output construction is unaffected.
    pub fn jobject_input(mut self) -> Self {
        self.jobject_input = true;
        self
    }

    class_interface_methods!("data_class");

    /// Expose a `#[prebindgen]` reader (`f(&Self) -> R`) as an instance
    /// method on the generated data class (see [`PtrClassDecl::method`]) — the
    /// receiver crosses as `this`'s field leaves, exactly like a data-class
    /// parameter.
    pub fn method(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Method));
        self
    }

    /// Expose a `#[prebindgen]` factory as a companion-object factory
    /// (see [`PtrClassDecl::constructor`]).
    pub fn constructor(mut self, rust_fun: FunctionDecl) -> Self {
        self.members.push((rust_fun, MemberKind::Constructor));
        self
    }
}

impl From<syn::Type> for DataClassDecl {
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
    Sealed(SealedClassDecl),
    Data(DataClassDecl),
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
impl From<SealedClassDecl> for ClassDecl {
    fn from(d: SealedClassDecl) -> Self {
        Self::Sealed(d)
    }
}
impl From<DataClassDecl> for ClassDecl {
    fn from(d: DataClassDecl) -> Self {
        Self::Data(d)
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

/// Declares one **constant** for emission: a lazily-initialized top-level
/// Kotlin `val` (`by lazy`) in its package's `.kt` file, initialized on
/// first use through a generated nullary JNI getter (the value type goes
/// through the ordinary output-converter machinery, exactly like a function
/// return; zero JNI calls at class-load).
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
/// Note the ident's role split: only the **bare** form also looks up the
/// same-named `#[prebindgen]` const (`MAX_LEN` above); under a stated
/// source the ident is purely the `val` name (`TAG_RUNTIME`, `VERSION`,
/// `BANNER` name no Rust item). In `.ignore(constant!(X))` the ident is
/// only the const lookup key.
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
            decl.param_expands().is_empty() && decl.return_expand().is_none(),
            "constant `{}`: expand overrides don't apply to a constant source fn `{}`",
            self.rust_ident,
            decl.rust_ident()
        );
        assert!(
            decl.name_override().is_none(),
            "constant `{}`: the val name belongs on `constant!(…)` (or its `.name(…)`), \
             not on the source fn `{}`",
            self.rust_ident,
            decl.rust_ident()
        );
        self.set_source(ConstSource::Fun(decl.rust_ident().clone()))
    }

    /// Value source: a **binding-local nullary fn** named by path —
    /// `(stated value type, path)`, the const analog of
    /// [`FunctionDecl::new_local`](prebindgen_registry::FunctionDecl::new_local).
    /// The fn lives in the binding crate (callable because the generated file
    /// compiles inside it):
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
    /// [`JniGenBuilder::ignore`] (+ [`matching`](crate::matching)).
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
// IgnoreDecl — one acceptor for acknowledged-unbound items
// ──────────────────────────────────────────────────────────────────────

/// Declares a `#[prebindgen]` item this binding deliberately does NOT
/// bind: nothing is emitted for it and the registry's per-item "skipping
/// undeclared" warning is suppressed. One acceptor
/// ([`JniGenBuilder::ignore`]), the kind carried by what you built:
///
/// ```rust,ignore
/// .ignore(fun!(string_len))                                // a fn
/// .ignore(ty!(InternalThing))                              // a struct/enum
/// .ignore(constant!(INTERNAL_MAGIC))                       // a const
/// .ignore(matching(|n| n.starts_with("encoding_const_")))  // a naming family
/// ```
pub struct IgnoreDecl(pub(crate) IgnoreKind);

pub(crate) enum IgnoreKind {
    Fun(syn::Ident),
    Type(TypeKey),
    Const(syn::Ident),
    Matching(prebindgen_registry::NamePredicate),
}

impl From<FunctionDecl> for IgnoreDecl {
    fn from(decl: FunctionDecl) -> Self {
        assert!(
            decl.name_override().is_none()
                && decl.param_expands().is_empty()
                && decl.return_expand().is_none(),
            "ignore(fun!({})): an ignored fn is never surfaced — \
             .name()/expand overrides don't apply",
            decl.rust_ident()
        );
        IgnoreDecl(IgnoreKind::Fun(decl.rust_ident().clone()))
    }
}

impl From<syn::Type> for IgnoreDecl {
    fn from(ty: syn::Type) -> Self {
        IgnoreDecl(IgnoreKind::Type(TypeKey::from_type(&ty)))
    }
}

impl From<ConstDecl> for IgnoreDecl {
    fn from(decl: ConstDecl) -> Self {
        assert!(
            matches!(decl.source, ConstSource::Item) && decl.kotlin_name_override.is_none(),
            "ignore(constant!({})): an ignore names a `#[prebindgen]` const — \
             value sources/.name() don't apply",
            decl.rust_ident
        );
        IgnoreDecl(IgnoreKind::Const(decl.rust_ident))
    }
}

/// Bulk [`IgnoreDecl`]: acknowledge every `#[prebindgen]` item whose NAME
/// matches the predicate — kind-agnostic (fn, struct/enum, const), since
/// prebindgen items live in one flat namespace. E.g.
/// `.ignore(matching(|n| n.starts_with("encoding_const_")))` instead of one
/// line per member of a naming family. A *declared* item matching the
/// predicate is unaffected (declaration wins), and unlike an exact-name
/// ignore, a predicate matching nothing is silent — it is a filter, not a
/// claim about a specific item (match counts vary across feature configs).
pub fn matching<F>(f: F) -> IgnoreDecl
where
    F: Fn(&str) -> bool + Send + Sync + 'static,
{
    IgnoreDecl(IgnoreKind::Matching(std::sync::Arc::new(f)))
}

// ──────────────────────────────────────────────────────────────────────
// PackageDecl — aggregates the package-scoped decls
// ──────────────────────────────────────────────────────────────────────

/// A batch of class, function and const declarations that land under one
/// Kotlin subpackage. Build it with [`package!`](crate::package)
/// (`package!("session")`, or `package!()` for the base package), fill it
/// with [`class`](Self::class) / [`fun`](Self::fun) /
/// [`constant`](Self::constant), and hand it to
/// [`JniGenBuilder::package`]. Reopening the same subpackage across several
/// `PackageDecl`s is fine — they merge.
pub struct PackageDecl {
    pub(crate) name: String,
    pub(crate) classes: Vec<ClassDecl>,
    pub(crate) functions: Vec<FunctionDecl>,
    pub(crate) constants: Vec<ConstDecl>,
}

impl PackageDecl {
    /// `name` is dot-separated, relative to the base package set by
    /// [`JniGenBuilder::set_package_prefix`]; the empty string is the base
    /// package itself. See [`crate::package!`] for the equivalent macro form
    /// (`package!("model")` / `package!()`).
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let trimmed = name.trim_matches('.').trim_matches('/').to_string();
        // Sanitize each subpackage segment to a valid Kotlin identifier
        // (issue #89); a no-op for already-legal names.
        let name = crate::jni::mangle_kotlin_package(&trimmed);
        if name != trimmed {
            println!(
                "cargo:warning=prebindgen: subpackage `{trimmed}` sanitized to `{name}` \
                 (invalid Kotlin package identifier)"
            );
        }
        Self {
            name,
            classes: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Add a class to this package — any of [`ptr_class!`](crate::ptr_class) /
    /// [`enum_class!`](crate::enum_class) / [`data_class!`](crate::data_class).
    pub fn class(mut self, decl: impl Into<ClassDecl>) -> Self {
        self.classes.push(decl.into());
        self
    }

    /// Add a free function to this package. Take a bare name via
    /// [`fun!`](prebindgen_registry::fun), or a customized [`FunctionDecl`] when you need
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
