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
// large_enum_variant: a handful of fields exist per binding, held while
// declarations replay — boxing the syn payloads would only complicate the
// arms (same trade-off as `ConvertSourceKind`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub(crate) enum LocalField {
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

/// Build a [`FunctionDecl`] from a bare function ident or a path:
///
/// * `fun!(foo)` — a `#[prebindgen]` fn; its signature is read from the
///   registry.
/// * `fun!(crate::foo)` — a **binding-local** fn: any fn the binding crate
///   defines, exported through the same machinery as a `#[prebindgen]` one.
///   A path carries no signature to read, so chain
///   [`.sig(sig!(…))`](crate::lang::FunctionDecl::sig). The generated file
///   calls it by the declared path (it compiles inside the binding crate,
///   so `crate::`-rooted paths resolve).
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::lang::FunctionDecl::new($crate::ident!($name))
    };
    ($path:path) => {
        $crate::lang::FunctionDecl::new_local($crate::__macro_support::parse_path(stringify!(
            $path
        )))
    };
}

/// State a binding-local fn's exact Rust signature, with **named parameters**
/// (they become the foreign-side parameter names): `sig!((s: &Summary,
/// verbose: bool) -> String)`; the `-> Ret` tail is optional (unit). The
/// signature argument of [`FunctionDecl::sig`](crate::lang::FunctionDecl::sig)
/// for a path-built [`fun!`](crate::fun).
#[macro_export]
macro_rules! sig {
    (($($params:tt)*) $(-> $ret:ty)?) => {
        $crate::__macro_support::parse_signature(stringify!(($($params)*) $(-> $ret)?))
    };
}

/// Build a [`ConstDecl`] from a bare ident: `constant!(MAX_LEN)` is
/// `ConstDecl::new(prebindgen::ident!(MAX_LEN))`.
///
/// What the ident names depends on where the decl lands:
/// * in `.constant(...)` it is always the Kotlin **`val` name**, and in the
///   **bare** form (no source modifier) it is *additionally* the lookup key
///   of the same-named `#[prebindgen]` const — `.fun(…)` / `.with(…)` /
///   `.expr(…)` replace that lookup with the stated value source (see the
///   four-source example on [`ConstDecl`](crate::lang::ConstDecl));
/// * in `.ignore(constant!(X))` it is *only* the `#[prebindgen]` const
///   lookup key — nothing is emitted, so sources and `.name()` are rejected
///   there.
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
/// argument of decl methods like [`ConvertSourceDecl::error`] — always
/// yields the concrete `syn::Type`, so no inference context is needed (see
/// [`ident!`](crate::ident) for the E0283 background).
#[macro_export]
macro_rules! ty {
    ($t:ty) => {
        $crate::__macro_support::parse_type(stringify!($t))
    };
}

/// Build a `syn::Path` from a bare path token: `path!(crate::conv::f)`. The
/// callable argument of [`ConvertSourceDecl::with`] and [`ConstDecl::with`].
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

/// Build a [`ConvertSourceDecl`](crate::lang::ConvertSourceDecl) for an
/// **input** conversion via `core::convert`: `.input(from!(i32))` requires
/// `i32: Into<T>`. Chain [`with`](crate::lang::ConvertSourceDecl::with) to
/// use a binding-local callable instead of the trait.
#[macro_export]
macro_rules! from {
    ($t:ty) => {
        $crate::lang::ConvertSourceDecl::from_type($crate::__macro_support::parse_type(stringify!(
            $t
        )))
    };
}

/// Fallible twin of [`from!`]: `.input(try_from!(i32))` requires
/// `i32: TryInto<T>`; an `Err` routes to the caller's error handler. With
/// [`with`](crate::lang::ConvertSourceDecl::with), the callable returns
/// `Result` and must state its error type via
/// [`error`](crate::lang::ConvertSourceDecl::error).
#[macro_export]
macro_rules! try_from {
    ($t:ty) => {
        $crate::lang::ConvertSourceDecl::try_from_type($crate::__macro_support::parse_type(
            stringify!($t),
        ))
    };
}

/// Build a [`ConvertSourceDecl`](crate::lang::ConvertSourceDecl) for an
/// **output** conversion via `core::convert`: `.output(into!(i32))` requires
/// `T: Into<i32>`. Chain [`with`](crate::lang::ConvertSourceDecl::with) to
/// use a binding-local callable instead of the trait.
#[macro_export]
macro_rules! into {
    ($t:ty) => {
        $crate::lang::ConvertSourceDecl::into_type($crate::__macro_support::parse_type(stringify!(
            $t
        )))
    };
}

/// Fallible twin of [`into!`]: `.output(try_into!(i32))` requires
/// `T: TryInto<i32>`; an `Err` routes to the caller's error handler. With
/// [`with`](crate::lang::ConvertSourceDecl::with), the callable returns
/// `Result` and must state its error type via
/// [`error`](crate::lang::ConvertSourceDecl::error).
#[macro_export]
macro_rules! try_into {
    ($t:ty) => {
        $crate::lang::ConvertSourceDecl::try_into_type($crate::__macro_support::parse_type(
            stringify!($t),
        ))
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
/// ([`name`](Self::name)), its instance methods ([`method`](Self::method)), and its
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
///     .method(prebindgen::fun!(keyexpr_get_str).name("str"));
/// ```
/// Deliberately has no verbatim type mapping: the generated typed-handle
/// class OWNS a lifecycle contract — the `NativeHandle` base, the `ptr`
/// slot, `close()`, the lock protocol, and the paired `freePtr` extern —
/// that an arbitrary existing Kotlin type cannot be assumed to honor.
/// Customize it from above instead: [`interface`](Self::interface) /
/// [`implements`](Self::implements).
pub struct PtrClassDecl {
    pub(crate) key: TypeKey,
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
        /// [`JniGen::set_interface_name_mangle`] hook over the final class
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
        /// bypassing the [`JniGen::set_interface_name_mangle`] hook.
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
            name_override: None,
            members: Vec::new(),
            iface: IfaceOpts::default(),
            gc_managed: false,
        }
    }

    /// Make instances of this handle class **GC-managed**: an unreachable
    /// handle whose native box was not otherwise released is freed by a
    /// shared [`java.lang.ref.Cleaner`].
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
    /// Rust type (via the [`JniGen::set_ptr_class_name_mangle`] hook); `.name("Foo")`
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
/// it to [`JniGen::expand`].
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
    /// `.no_split()` — suppress the proactive splittability check for this
    /// variant set (it will only ever be used as the selector form). See
    /// [`Self::no_split`].
    pub(crate) no_split: bool,
}

impl ExpandParamDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            variants: Vec::new(),
            no_split: false,
        }
    }

    /// Add a **build-from** arm: parameters of this type also carry the
    /// named `#[prebindgen]` constructor's inputs on the wire, and Rust
    /// builds the value in the same call. E.g. `keyexpr_new_try_from(&str)`
    /// gives every function taking a `KeyExpr` a String-carrying arm —
    /// as a selector + nullable slot at the wire tier (see the type-level
    /// docs for the exact generated shape), not as a Kotlin overload.
    ///
    /// A variant arm only *names* the constructor: no Kotlin surface of its
    /// own, so a decorated `fun!` (`.name()` / expand overrides) is a hard
    /// error rather than a silent discard.
    pub fn variant(mut self, ctor: FunctionDecl) -> Self {
        assert!(
            ctor.kotlin_name_override.is_none()
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
    /// The Kotlin field name is uniform for both: an explicit `.name(...)`
    /// on the `fun!`; else the Kotlin name of the class member if the same
    /// fn is declared via [`PtrClassDecl::method`] on this type (so a getter
    /// that is both a method and a field is named once); else the
    /// camel-cased fn ident (a path's LAST segment).
    ///
    /// Only the accessor's name is used here: expand overrides on the `fun!`
    /// are a hard error rather than a silent discard (the field's own
    /// decomposition comes from ITS type's boundary decl, not from the
    /// accessor).
    pub fn field(mut self, accessor: FunctionDecl) -> Self {
        assert!(
            accessor.param_expands.is_empty() && accessor.return_expand.is_none(),
            "expand_return!({}).field(fun!({})): expand overrides don't apply to a \
             field accessor — only .name() is honored",
            self.key.as_str(),
            accessor.rust_ident
        );
        self.fields.push(match accessor.local {
            None => LocalField::Named(accessor.rust_ident, accessor.kotlin_name_override),
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
                    name_override: accessor.kotlin_name_override,
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
/// Has no `.method`/`.constructor` by rule, not omission: members belong to
/// class kinds whose instances can re-enter Rust as an object (handle /
/// blob / field leaves). An enum value is a bare scalar with no object
/// identity — a "method" on it is just a free function taking the enum.
pub struct EnumClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) iface: IfaceOpts,
}

impl EnumClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
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

/// Declares a Rust struct as a Kotlin `data class`. Its fields cross the
/// boundary individually and Kotlin reassembles the object with a generated
/// `fromParts(...)` — no Rust-side heap object, no handle to close. Use this
/// for plain immutable data you copy across, as opposed to
/// [`ptr_class!`](crate::ptr_class) handles or
/// [`value_class!`](crate::value_class) blobs.
///
/// Members work like every class kind whose instance can re-enter Rust —
/// here the receiver re-enters as its **field leaves** (the same call-site
/// destructuring a data-class parameter gets), just rebased to `this`.
pub struct DataClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) iface: IfaceOpts,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
}

impl DataClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            iface: IfaceOpts::default(),
            members: Vec::new(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
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

/// Declares a small **`Copy`** Rust type that crosses **by value** — as its
/// raw bytes in a `ByteArray` — rather than as a heap handle. The
/// lightweight peer of [`PtrClassDecl`] for things like ids and timestamps
/// that have no lifecycle to manage. The type must be `Copy` (the generator
/// asserts it at compile time). Readers added with [`method`](Self::method) become
/// instance methods on the Kotlin value class.
pub struct ValueClassDecl {
    pub(crate) key: TypeKey,
    pub(crate) name_override: Option<String>,
    pub(crate) iface: IfaceOpts,
    pub(crate) members: Vec<(FunctionDecl, MemberKind)>,
}

impl ValueClassDecl {
    pub fn new(rust_type: syn::Type) -> Self {
        Self {
            key: TypeKey::from_type(&rust_type),
            name_override: None,
            iface: IfaceOpts::default(),
            members: Vec::new(),
        }
    }

    /// Override the Kotlin **class name** (relative, no dots).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_override = Some(name.into());
        self
    }

    class_interface_methods!("value_class");

    /// Expose a `#[prebindgen]` reader (`f(&Self) -> R`) as an instance
    /// method on the Kotlin value class (see [`PtrClassDecl::method`]).
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
/// [`PtrClassDecl::method`] / [`PtrClassDecl::constructor`].
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
    pub(crate) split_on_params: Vec<String>,
    /// `fun!(crate::f)` — a **binding-local** fn: the declared path plus the
    /// stated signature ([`sig`](Self::sig), required by acceptance time).
    /// `None` = an ordinary `#[prebindgen]` registry fn.
    pub(crate) local: Option<(syn::Path, Option<syn::Signature>)>,
}

impl FunctionDecl {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            param_expands: Vec::new(),
            return_expand: None,
            split_on_params: Vec::new(),
            local: None,
        }
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
    /// [`ConvertSourceDecl::with`]. The fn lives in the binding crate
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
    /// [`JniGen::ignore`] (+ [`matching`](crate::lang::matching)).
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
/// ([`JniGen::ignore`]), the kind carried by what you built:
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
    Matching(crate::api::core::prebindgen::NamePredicate),
}

impl From<FunctionDecl> for IgnoreDecl {
    fn from(decl: FunctionDecl) -> Self {
        assert!(
            decl.kotlin_name_override.is_none()
                && decl.param_expands.is_empty()
                && decl.return_expand.is_none(),
            "ignore(fun!({})): an ignored fn is never surfaced — \
             .name()/expand overrides don't apply",
            decl.rust_ident
        );
        IgnoreDecl(IgnoreKind::Fun(decl.rust_ident))
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
/// success position, as a `data_class` field. Each direction takes one
/// [`ConvertSourceDecl`]:
///
/// ```rust,ignore
/// .convert(convert!(Millis)
///     .input(fun!(millis_from_long))   // fn(u64) -> Millis    (wire → rust)
///     .output(fun!(millis_value)))     // fn(&Millis) -> u64   (rust → wire)
/// .convert(convert!(Celsius).input(from!(i32)).output(into!(i32)))
/// .convert(convert!(Label)
///     .input(try_from!(String).with(path!(crate::label_in)).error(ty!(String)))
///     .output(into!(String).with(path!(crate::label_out))))
/// ```
///
/// The Kotlin surface derives from the conversion's other-side type
/// (`u64` ⇒ `Long`) — nothing is stated verbatim. A `try_` source's `Err`
/// routes to the caller's error handler. Conversion fns may live in the flat
/// crate or in a **helper crate** whose item stream is chained into the same
/// [`crate::core::Registry::from_items`] call; generated calls qualify each
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
pub(crate) enum ConvertSpec {
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
    pub(crate) fn describe(&self) -> String {
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
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ConvertDirection {
    Input,
    Output,
}

impl ConvertDirection {
    fn macros(self) -> &'static str {
        match self {
            ConvertDirection::Input => "from!/try_from!",
            ConvertDirection::Output => "into!/try_into!",
        }
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
    pub(crate) kind: ConvertSourceKind,
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
        direction: ConvertDirection,
        fallible: bool,
        ty: syn::Type,
    },
}

impl ConvertSourceDecl {
    fn repr(direction: ConvertDirection, fallible: bool, ty: syn::Type) -> Self {
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
        Self::repr(ConvertDirection::Input, false, ty)
    }
    /// `try_from!(T)` — input via `T: TryInto<Self>`.
    pub fn try_from_type(ty: syn::Type) -> Self {
        Self::repr(ConvertDirection::Input, true, ty)
    }
    /// `into!(T)` — output via `Self: Into<T>`.
    pub fn into_type(ty: syn::Type) -> Self {
        Self::repr(ConvertDirection::Output, false, ty)
    }
    /// `try_into!(T)` — output via `Self: TryInto<T>`.
    pub fn try_into_type(ty: syn::Type) -> Self {
        Self::repr(ConvertDirection::Output, true, ty)
    }
}

impl From<FunctionDecl> for ConvertSourceDecl {
    fn from(decl: FunctionDecl) -> Self {
        assert!(
            decl.kotlin_name_override.is_none()
                && decl.param_expands.is_empty()
                && decl.return_expand.is_none(),
            "fun!({}) as a conversion source: a conversion fn is never surfaced in \
             Kotlin — .name()/expand overrides don't apply",
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
    pub(crate) key: TypeKey,
    pub(crate) input: Option<ConvertSpec>,
    pub(crate) output: Option<ConvertSpec>,
    /// Binding-local fn sources declared on this convert (`fun!(crate::f)
    /// .sig(…)`): drained into [`JniGen::local_fns`] at acceptance so the
    /// synthesis pre-pass covers them.
    pub(crate) locals: Vec<(syn::Ident, syn::Path, syn::Signature)>,
}

impl ConvertDecl {
    /// `: input …, output …` suffix for the report's conversions section.
    pub(crate) fn describe_sources(&self) -> String {
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
            input: None,
            output: None,
            locals: Vec::new(),
        }
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
        direction: ConvertDirection,
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
                    got = stated.macros(),
                    want = direction.macros(),
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
        let spec = self.spec_of(ConvertDirection::Input, "input", src.into());
        self.set_input(spec)
    }

    /// The **out-of-Rust** conversion (returns, callback arguments): how a
    /// value of this type is turned into its representation. Accepts
    /// [`fun!`](crate::fun) (a `#[prebindgen]` `fn(&T) -> U` / `fn(T) -> U`)
    /// or [`into!`](crate::into) / [`try_into!`](crate::try_into)
    /// (`T: Into<Repr>` / `TryInto`, or a binding-local callable via
    /// `.with(...)`).
    pub fn output(mut self, src: impl Into<ConvertSourceDecl>) -> Self {
        let spec = self.spec_of(ConvertDirection::Output, "output", src.into());
        self.set_output(spec)
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
