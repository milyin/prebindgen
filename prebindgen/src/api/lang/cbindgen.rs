//! `Cbindgen` — the C / cbindgen language adapter.
//!
//! A [`Prebindgen`] back-end that turns a "flat" `#[prebindgen]` library into a
//! Rust file suitable for [`cbindgen`](https://github.com/mozilla/cbindgen) to
//! parse into a C header plus a static / dynamic library.
//!
//! Items are **opt-in**: nothing is converted unless it is explicitly declared
//! with [`Cbindgen::function`] / [`Cbindgen::ptr_struct`] /
//! [`Cbindgen::data_struct`] / [`Cbindgen::enum_type`]. The C name of a declared
//! type's generated destructor can be pinned by chaining [`Cbindgen::name`].
//!
//! ## C ABI conventions
//!
//! * **Pointer struct** (declared with [`Cbindgen::ptr_struct`]): a `Box`-owned
//!   Rust value whose lifecycle is owned by the C side. The C type `T` is
//!   **opaque/incomplete** and the handle is a bare `T *` = `Box::into_raw`. A
//!   typed `<name>_drop(T *)` destructor (running the Rust `Drop`) is generated
//!   per handle.
//! * **Data struct** (declared with [`Cbindgen::data_struct`]): a by-value
//!   `#[repr(C)]` struct whose fields are mapped to C-ABI wire types
//!   (`String` → `*mut c_char`). No per-struct destructor — each `char*` field
//!   is released individually via the [`Cbindgen::free_memory_function`].
//! * **Direct `String` output**: a bare `char *` — a `malloc`'d, null-terminated
//!   raw block (no wrapper struct), freed via the `free_memory_function`.
//! * **[`Cbindgen::free_memory_function`]**: the single, type-agnostic raw memory
//!   freer (C `free`) for every `char*` the layer hands out (string returns and
//!   data-struct `String` fields). It runs no destructor and needs no length.
//!   Required whenever such string memory is produced.
//! * **`Result<T, E>` return** lowers by the success wire kind:
//!   - **pointer wire** (opaque handle, `char*`) → `T f(<inputs>, E *e)`, where a
//!     **NULL return signals error** (details written to `*e`);
//!   - **unit** → `bool f(<inputs>, E *e)`;
//!   - **value wire** (data struct, scalar, enum) → `bool f(T *out, <inputs>, E *e)`
//!     filling a caller-allocated `*out`.
//!   `e` may be `NULL`, in which case the error value is dropped. Infallible
//!   producers return the value/pointer directly (no out-param).
//!
//! ## Error handling (multiple error types)
//!
//! Any type used as the `E` of a `Result<T, E>` return **must be declared** as an
//! error type via [`Cbindgen::data_struct`] + [`Cbindgen::error`] — otherwise the
//! build fails. Error types are ordinary data structs (marshalled by value) and
//! must additionally implement `From<String>`.
//!
//! Built-in input converters that can fail (a `String` arg, an opaque handle
//! passed by value) are **error-type-agnostic**: they return `Result<_, String>`
//! where the `Err` is just a message. The generated wrapper for a `Result<T, E>`
//! function converts such a message into *that function's* `E` via
//! `<E as From<String>>::from(msg)`; the function's own `Err(E)` is marshalled
//! directly through `E`'s output converter.
//!
//! If a function can produce such an internal message but does **not** return
//! `Result`, that is a build error — suppress it by chaining [`Cbindgen::panic`]
//! after the function declaration, which makes the wrapper `panic!` on the
//! internal error instead.
//!
//! References to the original Rust types in generated bodies are written
//! fully-qualified against [`Cbindgen::source_module`] so the generated file can
//! define its own identically-named `#[repr(C)]` wrapper structs without
//! colliding with the source crate's types.

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::api::core::niches::Niches;
use crate::api::core::prebindgen::{ConverterImpl, Prebindgen};
use crate::api::core::registry::{extract_fn_trait_args, Registry, TypeKey};

/// Identity of a declared callback signature: its argument-type list (the
/// dedup key, since two `impl Fn` params with the same args share one closure
/// struct). The return is always unit for the supported callbacks.
type CallbackKey = Vec<TypeKey>;

/// Per-opaque-handle / per-data-struct / per-enum configuration.
#[derive(Clone, Default)]
struct TypeCfg {
    /// Pinned C type name (the emitted `#[repr(C)]` struct/enum identifier;
    /// the destructor is `<c_name>_drop`). Set by [`Cbindgen::name`]. Defaults
    /// to the Rust short name when `None`.
    c_name: Option<String>,
    /// Pinned **full** destructor symbol, independent of `c_name`. Set by
    /// [`Cbindgen::destructor_name`] (opaque handles only). When `None` the
    /// destructor defaults to `<c_drop_base>_drop`.
    drop_name: Option<String>,
}

/// Per-declared-callback configuration.
#[derive(Clone, Default)]
struct CbCfg {
    /// Pinned C type name of the emitted closure struct. Set by
    /// [`Cbindgen::name`]. Defaults to the generic [`Cbindgen::callback_c_name`]
    /// composition when `None`.
    c_name: Option<String>,
}

/// Per-declared-function configuration.
#[derive(Clone, Default)]
struct FnCfg {
    /// Pinned C export symbol (the `#[no_mangle]` wrapper name). Set by
    /// [`Cbindgen::name`]. Defaults to the Rust ident when `None`.
    c_name: Option<String>,
    /// Allow the generated wrapper to `panic!` on an internal error message
    /// (set by [`Cbindgen::panic`]). Only meaningful for non-`Result` functions
    /// that have a fallible input.
    panic: bool,
}

/// The declaration a chained modifier ([`Cbindgen::name`] / [`Cbindgen::error`]
/// / [`Cbindgen::panic`]) applies to. Set by each declaration method, reset to
/// `None` by root-level modifiers (e.g. [`Cbindgen::source_module`]).
#[derive(Clone)]
enum CurrentDecl {
    Ptr(TypeKey),
    Data(TypeKey),
    Enum(TypeKey),
    Callback(CallbackKey),
    Function(syn::Ident),
}

/// Where a fallible input-decode failure is routed in a generated wrapper.
enum ErrRoute<'a> {
    /// `Result<T, E>` function: convert the message to `E`, write `*e`, and
    /// return `fail_return` (`false` for a `bool`/out-param wrapper,
    /// `::core::ptr::null_mut()` for a pointer-returning wrapper).
    Result {
        e_conv: &'a syn::Ident,
        e_ty_src: syn::Type,
        fail_return: TokenStream,
    },
    /// Non-`Result` function declared `.panic()`: abort via `panic!`.
    Panic,
}

/// C / cbindgen language adapter. Build it with [`Cbindgen::new`], declare the
/// items to convert with the fluent methods, then drive it through
/// [`Registry::write_rust`](crate::core::Registry::write_rust).
#[derive(Default)]
pub struct Cbindgen {
    /// Module path the original `#[prebindgen]` items live under. Used to
    /// fully-qualify bare references to source types in generated bodies.
    source_module: Option<syn::Path>,
    /// `#[prebindgen]` functions explicitly declared for conversion.
    functions: HashMap<syn::Ident, FnCfg>,
    /// `#[prebindgen]` functions intentionally not exported by this adapter.
    ignored_functions: HashSet<syn::Ident>,
    /// Opaque-handle types (`Box` + `void*` lifecycle, auto `_drop`).
    opaque: HashMap<TypeKey, TypeCfg>,
    /// By-value `#[repr(C)]` data structs.
    data: HashMap<TypeKey, TypeCfg>,
    /// Enum types.
    enums: HashMap<TypeKey, TypeCfg>,
    /// Declared callback signatures (`impl Fn(...) + Send + Sync + 'static`),
    /// keyed by their argument-type list. Each emits one `#[repr(C)]` closure
    /// struct.
    callbacks: HashMap<CallbackKey, CbCfg>,
    /// Types intentionally not exported by this adapter.
    ignored_types: HashSet<TypeKey>,
    /// Data structs additionally marked as error types (allowlist for the
    /// "Result error type must be declared" rule).
    error: HashSet<TypeKey>,
    /// Name of the universal raw-memory freer (C `free`) for `char*` data the
    /// generated code hands out. Set by [`Self::free_memory_function`]. Required
    /// (build error otherwise) whenever string memory is produced.
    free_fn: Option<String>,
    /// The declaration that chained modifiers apply to. Set by declaration
    /// methods; reset to `None` by root-level modifiers.
    current: Option<CurrentDecl>,
    /// Optional name-mangling rules (all `None` ⇒ the built-in defaults below,
    /// which carry no target-language convention). A per-declaration `.name()` /
    /// `.destructor_name()` always wins over a mangler. See [[the builder
    /// methods]](Self::mangle_rust_type).
    ///
    /// Base: Rust short name → canonical token, feeding the three type manglers.
    mangle_rust_type: Option<Mangle1>,
    /// Base → C type name (struct / enum / data).
    mangle_type_name: Option<Mangle1>,
    /// Base → opaque-handle destructor symbol.
    mangle_destructor: Option<Mangle1>,
    /// Callback arg bases → closure-struct name.
    mangle_callback: Option<MangleN>,
    /// Rust function ident → exported `#[no_mangle]` symbol.
    mangle_function: Option<Mangle1>,
}

/// A mangler over a single name component (Rust short name, base, or fn ident).
type Mangle1 = Box<dyn Fn(&str) -> String>;
/// A mangler over a callback's argument bases.
type MangleN = Box<dyn Fn(&[String]) -> String>;

impl Cbindgen {
    /// Create an adapter with no declarations (emits an empty library).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the module path the original `#[prebindgen]` items live under
    /// (e.g. `syn::parse_quote!(zenoh_flat)`). Root-level modifier: resets the
    /// current declaration, so it can't be followed by `.name()`/`.error()`/etc.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = Some(p);
        self.current = None;
        self
    }

    /// Set the name of the universal memory-freeing function (a type-agnostic C
    /// `free`) the generated layer exports for releasing `char*` data it hands to
    /// C — string returns and `String` fields of data structs. Root-level
    /// modifier: resets the current declaration. Required whenever the adapter
    /// produces such string memory; otherwise that's a build error.
    pub fn free_memory_function(mut self, name: impl Into<String>) -> Self {
        self.free_fn = Some(name.into());
        self.current = None;
        self
    }

    /// Set the **base** Rust-type mangler: maps a type's Rust short name (e.g.
    /// `ZKeyExpr`) to a canonical token (e.g. `keyexpr`). Its output feeds
    /// [`Self::mangle_type_name`], [`Self::mangle_destructor`] and
    /// [`Self::mangle_callback`], so a one-off spelling fix (e.g. `KeyExpr` →
    /// `keyexpr`) lives in a single place instead of a per-declaration `.name()`
    /// exception. Root-level modifier (resets the current declaration). The
    /// adapter ships no default — unset, the base is the Rust short name verbatim.
    pub fn mangle_rust_type(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_rust_type = Some(Box::new(f));
        self.current = None;
        self
    }

    /// Set the type-name mangler: base (see [`Self::mangle_rust_type`]) → the C
    /// type name emitted for a `ptr_struct` / `data_struct` / `enum_type` (e.g.
    /// `keyexpr` → `z_keyexpr_t`). Overridden per declaration by `.name(...)`.
    /// Root-level modifier.
    pub fn mangle_type_name(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_type_name = Some(Box::new(f));
        self.current = None;
        self
    }

    /// Set the destructor mangler: base → an opaque handle's `_drop` symbol (e.g.
    /// `keyexpr` → `z_keyexpr_drop`). Overridden per declaration by
    /// `.destructor_name(...)`. Root-level modifier.
    pub fn mangle_destructor(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_destructor = Some(Box::new(f));
        self.current = None;
        self
    }

    /// Set the callback-struct mangler: the bases of a callback's argument types
    /// → the closure struct's C name (e.g. `["sample"]` → `z_closure_sample_t`,
    /// `[]` → `z_closure_drop_t`). Overridden per declaration by `.name(...)`.
    /// Root-level modifier.
    pub fn mangle_callback(mut self, f: impl Fn(&[String]) -> String + 'static) -> Self {
        self.mangle_callback = Some(Box::new(f));
        self.current = None;
        self
    }

    /// Set the function mangler: a `#[prebindgen]` function's Rust ident → its
    /// exported `#[no_mangle]` symbol (e.g. prefix `z_`). Functions are not types,
    /// so this does not go through the base mangler. Overridden per declaration by
    /// `.name(...)`. Root-level modifier.
    pub fn mangle_function(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_function = Some(Box::new(f));
        self.current = None;
        self
    }

    /// Declare a `#[prebindgen]` function to convert into the C layer.
    pub fn function(mut self, ident: syn::Ident) -> Self {
        assert!(
            !self.ignored_functions.contains(&ident),
            "Cbindgen::function cannot declare `{}` because it is already ignored",
            ident
        );
        self.functions.insert(ident.clone(), FnCfg::default());
        self.current = Some(CurrentDecl::Function(ident));
        self
    }

    /// Mark a `#[prebindgen]` function as intentionally ignored by this
    /// adapter. Root-level modifier: suppresses the registry's
    /// "skipping undeclared" warning for that function without scanning or
    /// emitting it.
    pub fn ignore_function(mut self, ident: syn::Ident) -> Self {
        assert!(
            !self.functions.contains_key(&ident),
            "Cbindgen::ignore_function cannot ignore `{}` because it is already declared",
            ident
        );
        self.ignored_functions.insert(ident);
        self.current = None;
        self
    }

    /// Allow the most recently declared [`Self::function`] to `panic!` on an
    /// internal error message. Required when a non-`Result` function has a
    /// fallible input (otherwise that's a build error).
    pub fn panic(mut self) -> Self {
        match &self.current {
            Some(CurrentDecl::Function(ident)) => {
                let ident = ident.clone();
                self.functions
                    .get_mut(&ident)
                    .expect("function entry vanished")
                    .panic = true;
            }
            other => panic!(
                "Cbindgen::panic must be chained after a `function(...)` call, \
                 not after {}",
                describe_current(other)
            ),
        }
        self
    }

    /// Declare a pointer-struct (opaque-handle) type — a `Box`-owned Rust value
    /// the C side holds as `#[repr(C)] struct T { _0: *mut c_void }`. Its C
    /// struct + `<name>_drop` destructor are generated. (Mirrors `JniExt`'s
    /// `ptr_class`.)
    pub fn ptr_struct(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::ptr_struct cannot declare `{}` because it is already ignored",
            key
        );
        self.opaque.insert(key.clone(), TypeCfg::default());
        self.current = Some(CurrentDecl::Ptr(key));
        self
    }

    /// Declare a by-value `#[repr(C)]` data struct (e.g. `Error`).
    pub fn data_struct(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::data_struct cannot declare `{}` because it is already ignored",
            key
        );
        self.data.insert(key.clone(), TypeCfg::default());
        self.current = Some(CurrentDecl::Data(key));
        self
    }

    /// Mark a `#[prebindgen]` type as intentionally ignored by this adapter.
    /// Root-level modifier: suppresses the registry's "skipping undeclared"
    /// warning for that type without scanning or emitting it.
    pub fn ignore_type(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.opaque.contains_key(&key)
                && !self.data.contains_key(&key)
                && !self.enums.contains_key(&key),
            "Cbindgen::ignore_type cannot ignore `{}` because it is already declared",
            key
        );
        self.ignored_types.insert(key);
        self.current = None;
        self
    }

    /// Pin the C-facing name of the **current declaration** (universal modifier):
    /// a type's emitted `#[repr(C)]` struct/enum identifier (its destructor
    /// becomes `<name>_drop`), or a function's exported `#[no_mangle]` symbol.
    /// E.g. `.ptr_struct(syn::parse_quote!(ZKeyExpr)).name("z_keyexpr")` →
    /// `typedef struct {…} z_keyexpr;` + `z_keyexpr_drop`. Defaults: a type's C
    /// name is the Rust short name, a function's is its Rust ident. Panics if not
    /// chained directly after a declaration. (Mirrors `JniExt`'s `name`.)
    pub fn name(mut self, c_name: impl Into<String>) -> Self {
        let name = c_name.into();
        match self.current.clone() {
            Some(CurrentDecl::Ptr(key)) => {
                self.opaque.get_mut(&key).expect("entry vanished").c_name = Some(name);
            }
            Some(CurrentDecl::Data(key)) => {
                self.data.get_mut(&key).expect("entry vanished").c_name = Some(name);
            }
            Some(CurrentDecl::Enum(key)) => {
                self.enums.get_mut(&key).expect("entry vanished").c_name = Some(name);
            }
            Some(CurrentDecl::Callback(key)) => {
                self.callbacks.get_mut(&key).expect("entry vanished").c_name = Some(name);
            }
            Some(CurrentDecl::Function(ident)) => {
                self.functions
                    .get_mut(&ident)
                    .expect("entry vanished")
                    .c_name = Some(name);
            }
            None => panic!(
                "Cbindgen::name must be chained directly after a declaration \
                 (`ptr_struct` / `data_struct` / `enum_type` / `callback` / `function`)"
            ),
        }
        self
    }

    /// Pin the **full** destructor symbol of the current declaration (which must
    /// be a [`Self::ptr_struct`]), independently of its C type name. E.g.
    /// `.ptr_struct(ZKeyExpr).name("z_keyexpr_t").destructor_name("z_keyexpr_drop")`
    /// emits type `z_keyexpr_t` with destructor `z_keyexpr_drop` (no `_drop`
    /// auto-appended). Defaults to `<c_drop_base>_drop`. Panics if not chained
    /// directly after a `ptr_struct(...)` declaration.
    pub fn destructor_name(mut self, c_name: impl Into<String>) -> Self {
        match &self.current {
            Some(CurrentDecl::Ptr(key)) => {
                let key = key.clone();
                self.opaque.get_mut(&key).expect("entry vanished").drop_name = Some(c_name.into());
            }
            other => panic!(
                "Cbindgen::destructor_name must be chained after a `ptr_struct(...)` \
                 call, not after {}",
                describe_current(other)
            ),
        }
        self
    }

    /// Mark the current declaration (which must be a [`Self::data_struct`]) as an
    /// error type: it may appear as the `E` of a `Result<_, E>` return. The type
    /// must implement `From<String>`. Panics if the current declaration is not a
    /// data struct.
    pub fn error(mut self) -> Self {
        match &self.current {
            Some(CurrentDecl::Data(key)) => {
                self.error.insert(key.clone());
            }
            other => panic!(
                "Cbindgen::error must be chained after a `data_struct(...)` call \
                 (error types are marshalled by value), not after {}",
                describe_current(other)
            ),
        }
        self
    }

    /// Declare a C-like (fieldless) enum type to convert. (Mirrors `JniExt`'s
    /// `enum_class`.)
    pub fn enum_type(mut self, ty: syn::Type) -> Self {
        let key = TypeKey::from_type(&ty);
        assert!(
            !self.ignored_types.contains(&key),
            "Cbindgen::enum_type cannot declare `{}` because it is already ignored",
            key
        );
        self.enums.insert(key.clone(), TypeCfg::default());
        self.current = Some(CurrentDecl::Enum(key));
        self
    }

    /// Declare a callback signature so its `impl Fn(...)` parameters resolve and
    /// a `#[repr(C)]` closure struct (`{ void *context; call; drop }`) is
    /// emitted for it. `ty` must be `impl Fn(Args...) + Send + Sync + 'static`.
    /// Identical signatures share one struct. Sets the declaration cursor, so a
    /// following `.name("...")` overrides the generated struct name (otherwise
    /// the generic [`Self::callback_c_name`] default is used).
    pub fn callback(mut self, ty: syn::Type) -> Self {
        let args = extract_fn_trait_args(&ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::callback expects `impl Fn(Args...) + Send + Sync + 'static`, got `{}`",
                ty.to_token_stream()
            )
        });
        let key: CallbackKey = args.iter().map(TypeKey::from_type).collect();
        self.callbacks.insert(key.clone(), CbCfg::default());
        self.current = Some(CurrentDecl::Callback(key));
        self
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Fully-qualify a bare single-segment source type against
    /// [`Self::source_module`] (e.g. `ZKeyExpr` → `zenoh_flat::ZKeyExpr`).
    /// Anything already qualified, or with no `source_module` set, is returned
    /// unchanged.
    fn src_ty(&self, ty: &syn::Type) -> syn::Type {
        if let (Some(m), syn::Type::Path(tp)) = (&self.source_module, ty) {
            if tp.qself.is_none() && tp.path.leading_colon.is_none() && tp.path.segments.len() == 1
            {
                let mut path = m.clone();
                path.segments.push(tp.path.segments[0].clone());
                return syn::Type::Path(syn::TypePath { qself: None, path });
            }
        }
        ty.clone()
    }

    /// Path to a source function (e.g. `zenoh_flat::z_keyexpr_try_from`).
    fn src_fn(&self, ident: &syn::Ident) -> syn::Path {
        match &self.source_module {
            Some(m) => {
                let mut p = m.clone();
                p.segments.push(syn::PathSegment::from(ident.clone()));
                p
            }
            None => syn::Path::from(ident.clone()),
        }
    }

    fn in_name(ty: &syn::Type) -> syn::Ident {
        format_ident!("__cbg_in_{}", sanitize(&TypeKey::from_type(ty)))
    }

    fn out_name(ty: &syn::Type) -> syn::Ident {
        format_ident!("__cbg_out_{}", sanitize(&TypeKey::from_type(ty)))
    }

    /// Config of a declared type (across the opaque/data/enum maps), by key.
    fn type_cfg(&self, ty: &syn::Type) -> Option<&TypeCfg> {
        let key = TypeKey::from_type(ty);
        self.opaque
            .get(&key)
            .or_else(|| self.data.get(&key))
            .or_else(|| self.enums.get(&key))
    }

    /// Base token for a Rust type: [`Self::mangle_rust_type`] applied to the Rust
    /// short name, or the short name verbatim when unset. Feeds the type-name,
    /// destructor and callback manglers.
    fn rust_base(&self, ty: &syn::Type) -> String {
        let short = type_short(ty);
        match &self.mangle_rust_type {
            Some(f) => f(&short),
            None => short,
        }
    }

    /// Emitted C type name of a declared type: pinned `c_name`, else
    /// [`Self::mangle_type_name`] over the base, else the base (which is the Rust
    /// short name when no base mangler is set).
    fn c_type_name(&self, ty: &syn::Type) -> String {
        if let Some(name) = self.type_cfg(ty).and_then(|c| c.c_name.clone()) {
            return name;
        }
        let base = self.rust_base(ty);
        match &self.mangle_type_name {
            Some(f) => f(&base),
            None => base,
        }
    }

    /// C type identifier (the `#[repr(C)]` struct/enum name + the wire type used
    /// across converters and wrappers).
    fn c_type_ident(&self, ty: &syn::Type) -> syn::Ident {
        format_ident!("{}", self.c_type_name(ty))
    }

    /// Base name of a declared type's destructor: pinned `c_name`, else
    /// `snake_case(short)`. The destructor is `<base>_drop`.
    fn c_drop_base(&self, ty: &syn::Type) -> String {
        self.type_cfg(ty)
            .and_then(|c| c.c_name.clone())
            .unwrap_or_else(|| snake_case(&type_short(ty)))
    }

    /// Destructor symbol of an opaque handle: pinned `.destructor_name(...)`,
    /// else [`Self::mangle_destructor`] over the base, else `<c_drop_base>_drop`.
    fn destructor_symbol(&self, ty: &syn::Type) -> syn::Ident {
        let key = TypeKey::from_type(ty);
        if let Some(full) = self.opaque.get(&key).and_then(|c| c.drop_name.clone()) {
            return format_ident!("{}", full);
        }
        if let Some(f) = &self.mangle_destructor {
            return format_ident!("{}", f(&self.rust_base(ty)));
        }
        format_ident!("{}_drop", self.c_drop_base(ty))
    }

    /// Emitted C type name of a callback's closure struct: pinned `.name(...)`
    /// override, else [`Self::mangle_callback`] over the args' bases, else a
    /// generic default composed from the args' C type names (`closure` for zero
    /// args, `closure_<arg0>_<arg1>…` otherwise). The adapter's own default
    /// carries no target-language naming convention.
    fn callback_c_name(&self, args: &[syn::Type]) -> String {
        let key: CallbackKey = args.iter().map(TypeKey::from_type).collect();
        if let Some(name) = self.callbacks.get(&key).and_then(|c| c.c_name.clone()) {
            return name;
        }
        if let Some(f) = &self.mangle_callback {
            let bases: Vec<String> = args.iter().map(|a| self.rust_base(a)).collect();
            return f(&bases);
        }
        if args.is_empty() {
            "closure".to_string()
        } else {
            let parts: Vec<String> = args.iter().map(|a| self.c_type_name(a)).collect();
            format!("closure_{}", parts.join("_"))
        }
    }

    /// C struct identifier for a callback's closure type (see
    /// [`Self::callback_c_name`]).
    fn callback_c_ident(&self, args: &[syn::Type]) -> syn::Ident {
        format_ident!("{}", self.callback_c_name(args))
    }
}

/// Rebuild the canonical `impl Fn(args...) + Send + Sync + 'static` type from an
/// argument list (matching the source spelling so its [`TypeKey`] round-trips —
/// see `core::resolve`'s reconstruction).
fn callback_fn_type(args: &[syn::Type]) -> syn::Type {
    syn::parse_quote!(impl Fn(#(#args),*) + Send + Sync + 'static)
}

/// Human-readable description of the current declaration, for panic messages.
fn describe_current(current: &Option<CurrentDecl>) -> String {
    match current {
        None => "no declaration".to_string(),
        Some(CurrentDecl::Ptr(k)) => format!("ptr_struct `{}`", k.as_str()),
        Some(CurrentDecl::Data(k)) => format!("data_struct `{}`", k.as_str()),
        Some(CurrentDecl::Enum(k)) => format!("enum_type `{}`", k.as_str()),
        Some(CurrentDecl::Callback(k)) => {
            let args: Vec<&str> = k.iter().map(|t| t.as_str()).collect();
            format!("callback `impl Fn({})`", args.join(", "))
        }
        Some(CurrentDecl::Function(i)) => format!("function `{i}`"),
    }
}

impl Prebindgen for Cbindgen {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.keys().cloned().collect()
    }

    fn ignored_functions(&self) -> HashSet<syn::Ident> {
        self.ignored_functions.clone()
    }

    fn declared_types(&self) -> HashSet<TypeKey> {
        self.opaque
            .keys()
            .chain(self.data.keys())
            .chain(self.enums.keys())
            .cloned()
            .collect()
    }

    fn ignored_types(&self) -> HashSet<TypeKey> {
        self.ignored_types.clone()
    }

    fn prerequisites(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();

        // C-string data memory (string returns + `String` fields of data structs)
        // is malloc'd raw and freed by the single universal `free_memory_function`.
        // Array returns (`Vec<T>`) also hand out a malloc'd block freed via the
        // same function (per element through the `z_free_array` macro), so the
        // allocator/freer prelude is needed for them too.
        let produces_array = self.produces_array(registry);
        if self.needs_free(registry) || produces_array {
            let free_ident = match &self.free_fn {
                Some(name) => format_ident!("{}", name),
                None => panic!(
                    "Cbindgen: the generated layer hands `char*` string memory to C \
                     (a `String` return or a `String` data-struct field) but no \
                     memory-freeing function is declared — add \
                     `.free_memory_function(\"z_free\")`"
                ),
            };
            // C allocator (linked from the C runtime; no crate dependency).
            items.push(syn::parse_quote!(
                extern "C" {
                    fn malloc(size: usize) -> *mut ::core::ffi::c_void;
                    fn free(ptr: *mut ::core::ffi::c_void);
                }
            ));
            // Raw, destructor-free C-string block. `CString::new` drops interior
            // NULs so the terminator marks the true end for C consumers.
            items.push(syn::parse_quote!(
                #[allow(non_snake_case, dead_code)]
                pub(crate) fn __cbg_alloc_cstr(
                    s: ::std::string::String,
                ) -> *mut ::core::ffi::c_char {
                    let c = ::std::ffi::CString::new(s).unwrap_or_default();
                    let bytes = c.as_bytes_with_nul();
                    unsafe {
                        let p = malloc(bytes.len()) as *mut u8;
                        if p.is_null() {
                            return ::core::ptr::null_mut();
                        }
                        ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                        p as *mut ::core::ffi::c_char
                    }
                }
            ));
            // Universal raw memory freer: type-agnostic C `free`, no length, no
            // destructor (NULL-safe via C `free`).
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #free_ident(p: *mut ::core::ffi::c_void) {
                    free(p);
                }
            ));
        }

        // Array builder: copy a `Vec<W>` into a C-`malloc`'d block of `W` and
        // return `(ptr, len)` (empty ⇒ `(NULL, 0)`). The block is freed C-side
        // via the `z_free_array` macro (per-element drop + the universal freer).
        if produces_array {
            items.push(syn::parse_quote!(
                #[allow(non_snake_case, dead_code)]
                pub(crate) unsafe fn __cbg_alloc_array<W>(
                    v: ::std::vec::Vec<W>,
                ) -> (*mut W, usize) {
                    let n = v.len();
                    if n == 0 {
                        return (::core::ptr::null_mut(), 0);
                    }
                    let p = malloc(n.wrapping_mul(::core::mem::size_of::<W>())) as *mut W;
                    if p.is_null() {
                        return (::core::ptr::null_mut(), 0);
                    }
                    for (i, e) in v.into_iter().enumerate() {
                        ::core::ptr::write(p.add(i), e);
                    }
                    (p, n)
                }
            ));
        }

        // Opaque handles: bare-pointer C type (`z_*_t*` = `Box::into_raw`) + typed
        // `_drop`. The C type is an opaque/incomplete struct.
        for (key, _cfg) in sorted_by_key(&self.opaque) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let c_struct = self.c_type_ident(&ty);
            // Opaque/incomplete C type: the handle is `#c_struct *`, which IS the
            // `Box::into_raw` pointer to the source value.
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    _private: [u8; 0],
                }
            ));
            let src = self.src_ty(&ty);
            let drop_ident = self.destructor_symbol(&ty);
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #c_struct) {
                    if !this_.is_null() {
                        drop(::std::boxed::Box::from_raw(this_ as *mut #src));
                    }
                }
            ));
        }

        // Data structs: `#[repr(C)]` mirror only. Heap (`String`) fields are
        // `char*` raw blocks the C user releases individually via the
        // `free_memory_function` — no per-struct destructor.
        for (key, _cfg) in sorted_by_key(&self.data) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(fields) = self.struct_fields(registry, &ty) else {
                continue;
            };
            let c_struct = self.c_type_ident(&ty);
            let mut field_defs: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                let wire = c_field_wire(fty).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: field `{}` of data struct `{}` has unsupported type `{}`",
                        fname,
                        type_short(&ty),
                        fty.to_token_stream()
                    )
                });
                field_defs.push(quote!(pub #fname: #wire));
            }
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    #(#field_defs,)*
                }
            ));
        }

        // Enums: `#[repr(C)]` mirror (variant idents + explicit discriminants).
        for (key, _cfg) in sorted_by_key(&self.enums) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(e) = enum_item(registry, &ty) else {
                continue;
            };
            assert_unit_variants(e);
            let cname = self.c_type_ident(&ty);
            let variants = e.variants.iter().map(|v| {
                let id = &v.ident;
                match &v.discriminant {
                    Some((_, expr)) => quote!(#id = #expr),
                    None => quote!(#id),
                }
            });
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub enum #cname {
                    #(#variants),*
                }
            ));
        }

        // Callback closure structs: one `#[repr(C)]` `{ context, call, drop }`
        // per declared signature actually used (its `impl Fn(...)` input
        // resolved). `call` takes each arg's output wire (the owned handle the
        // C callback must drop) plus the `void *context`; `drop` releases the
        // context. Deterministic order by emitted name.
        let mut cb_keys: Vec<&CallbackKey> = self.callbacks.keys().collect();
        cb_keys.sort_by_key(|k| {
            let args: Vec<syn::Type> = k.iter().map(|t| t.to_type()).collect();
            self.callback_c_name(&args)
        });
        for key in cb_keys {
            let args: Vec<syn::Type> = key.iter().map(|t| t.to_type()).collect();
            // Emit only if the callback is required (its input resolved); skip a
            // declared-but-unused signature.
            if registry.input_entry(&callback_fn_type(&args)).is_none() {
                continue;
            }
            let arg_wires: Vec<syn::Type> = args
                .iter()
                .map(|a| {
                    registry
                        .output_entry(a)
                        .unwrap_or_else(|| {
                            panic!(
                                "Cbindgen: callback arg `{}` has no output converter (declare it \
                                 as a ptr_struct/data_struct/enum_type)",
                                a.to_token_stream()
                            )
                        })
                        .destination
                        .clone()
                })
                .collect();
            let c_struct = self.callback_c_ident(&args);
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    pub context: *mut ::core::ffi::c_void,
                    pub call: ::core::option::Option<
                        unsafe extern "C" fn(#(#arg_wires,)* *mut ::core::ffi::c_void),
                    >,
                    pub drop: ::core::option::Option<
                        unsafe extern "C" fn(*mut ::core::ffi::c_void),
                    >,
                }
            ));
        }

        items
    }

    // ── Item emission ──────────────────────────────────────────────────

    fn on_function(&self, f: &syn::ItemFn, registry: &Registry<()>) -> TokenStream {
        self.emit_function_wrapper(f, registry)
    }

    fn on_struct(&self, _s: &syn::ItemStruct, _registry: &Registry<()>) -> TokenStream {
        // The `#[repr(C)]` mirror + converters come from prerequisites /
        // on_output_type_rank_0; the original (non-FFI-safe) struct is dropped.
        TokenStream::new()
    }

    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }

    // ── Input direction (wire → rust) ──────────────────────────────────

    fn on_input_type_rank_0(&self, ty: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);

        // Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible (null
        // handle → message). The wire is the bare handle pointer `*mut #c_struct`.
        if self.opaque.contains_key(&key) {
            let name = Self::in_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let short = type_short(ty);
            let null_msg = format!("null {short} handle passed by value");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: *mut #c_struct,
                ) -> ::core::result::Result<#src, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_msg),
                        );
                    }
                    ::core::result::Result::Ok(*::std::boxed::Box::from_raw(v as *mut #src))
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*mut #c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Data struct: decode each field from its C wire — infallible.
        if self.data.contains_key(&key) {
            let fields = self.struct_fields(_r, ty)?;
            let name = Self::in_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let mut inits: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                if is_string(fty) {
                    inits.push(quote!(#fname: if v.#fname.is_null() {
                        ::std::string::String::new()
                    } else {
                        ::std::ffi::CStr::from_ptr(v.#fname).to_string_lossy().into_owned()
                    }));
                } else {
                    inits.push(quote!(#fname: v.#fname));
                }
            }
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #c_struct) -> #src {
                    #src { #(#inits),* }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(#c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Enum input: `match` the C enum back to the source enum — infallible.
        if self.enums.contains_key(&key) {
            let e = enum_item(_r, ty)?;
            assert_unit_variants(e);
            let name = Self::in_name(ty);
            let cname = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let arms = e.variants.iter().map(|v| {
                let id = &v.ident;
                quote!(#cname::#id => #src::#id,)
            });
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #cname) -> #src {
                    match v { #(#arms)* }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(#cname),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `String` input: `*const c_char` → owned `String` — fallible.
        if is_string(ty) {
            let name = Self::in_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: *const ::core::ffi::c_char,
                ) -> ::core::result::Result<::std::string::String, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from("null pointer passed for String argument"),
                        );
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => {
                            ::core::result::Result::Ok(s.to_owned())
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(
                                ::std::string::String::from("invalid UTF-8 in String argument"),
                            )
                        }
                    }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*const ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Bare `str` never crosses the C ABI directly, but resolving `&str`
        // inputs requires its inner node to have a filled rank-0 cell.
        if is_str(ty) {
            let name = Self::in_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused_variables)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*const ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // FFI-safe scalar (`bool`, integers, floats): identity pass-through.
        if is_scalar(ty) {
            let name = Self::in_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #ty) -> #ty {
                    v
                }
            );
            return Some(ConverterImpl {
                destination: ty.clone(),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        None
    }

    fn on_input_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Option<T>` input: a single nullable C param, NULL = `None` (the input
        // mirror of the output null-niche rule). A pointer-wire inner (opaque
        // handle, `char*`) reuses its wire directly; a value/scalar inner is
        // boxed behind a `*const` pointer. The inner's own converter does the
        // non-null decode, so its fallibility is preserved.
        //
        // Only the `Option<_>` pattern (wildcard directly the argument) is
        // handled here, so `t1` is the full inner type and its own converter
        // (e.g. the `&T` borrow) is reused verbatim. Patterns like `Option<&_>`
        // or `Option<Vec<_>>` are rejected — the resolver also enumerates the
        // `Option<_>` shape with the concrete inner in `t1`, which is the one
        // that resolves correctly (otherwise an `Option<&ZConfig>` would bind to
        // the *owned* `ZConfig` converter, dropping the reference).
        if is_option(pat) && matches!(first_type_arg(pat), Some(syn::Type::Infer(_))) {
            let inner = _r.input_entry(t1)?;
            let inner_wire = inner.destination.clone();
            let inner_conv = inner.function.sig.ident.clone();
            // Inner Ok type + fallibility from its converter's return type.
            let (inner_ok, fallible): (syn::Type, bool) = match &inner.function.sig.output {
                syn::ReturnType::Type(_, ty) if is_result(ty) => {
                    let (ok, _e) = result_parts(ty).expect("is_result ⇒ result_parts");
                    (ok, true)
                }
                syn::ReturnType::Type(_, ty) => ((**ty).clone(), false),
                syn::ReturnType::Default => (syn::parse_quote!(()), false),
            };
            let is_ptr = matches!(inner_wire, syn::Type::Ptr(_));
            let wire: syn::Type = if is_ptr {
                inner_wire.clone()
            } else {
                syn::parse_quote!(*const #inner_wire)
            };
            // Read the inner wire value out of `v` for the non-null branch.
            let read = if is_ptr { quote!(v) } else { quote!(*v) };
            let name = format_ident!("__cbg_in_option_{}", sanitize(&TypeKey::from_type(t1)));
            // A borrow inner (`Option<&T>`) carries the `'a` of its decoded
            // reference into `inner_ok`, so the wrapper must declare it.
            let lt: TokenStream = if matches!(t1, syn::Type::Reference(_)) {
                quote!(<'a>)
            } else {
                quote!()
            };
            let function: syn::ItemFn = if fallible {
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lt(
                        v: #wire,
                    ) -> ::core::result::Result<::core::option::Option<#inner_ok>, ::std::string::String> {
                        if v.is_null() {
                            return ::core::result::Result::Ok(::core::option::Option::None);
                        }
                        match #inner_conv(#read) {
                            ::core::result::Result::Ok(__x) => {
                                ::core::result::Result::Ok(::core::option::Option::Some(__x))
                            }
                            ::core::result::Result::Err(__e) => ::core::result::Result::Err(__e),
                        }
                    }
                )
            } else {
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lt(
                        v: #wire,
                    ) -> ::core::option::Option<#inner_ok> {
                        if v.is_null() {
                            ::core::option::Option::None
                        } else {
                            ::core::option::Option::Some(#inner_conv(#read))
                        }
                    }
                )
            };
            return Some(ConverterImpl {
                destination: wire,
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&str`: borrow a UTF-8 C string directly from the caller.
        let syn::Type::Reference(r) = pat else {
            return None;
        };
        // `&[E]` slice (scalar `E`): marker only — the two-param (`*const E`,
        // `usize`) lowering is done structurally in `emit_inputs`. `pat` is the
        // wildcard `&[_]`; `t1` is the element type.
        if r.mutability.is_none() {
            if let syn::Type::Slice(s) = &*r.elem {
                if matches!(&*s.elem, syn::Type::Infer(_)) && is_scalar(t1) {
                    let name =
                        format_ident!("__cbg_inmark_slice_{}", sanitize(&TypeKey::from_type(t1)));
                    let function: syn::ItemFn = syn::parse_quote!(
                        #[allow(non_snake_case, dead_code, unused)]
                        pub(crate) fn #name() {}
                    );
                    return Some(ConverterImpl {
                        destination: syn::parse_quote!(*const #t1),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    });
                }
            }
        }
        if r.mutability.is_none() && is_str(t1) {
            let name = Self::in_name(pat);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: *const ::core::ffi::c_char,
                ) -> ::core::result::Result<&'a str, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from("null pointer passed for str argument"),
                        );
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => ::core::result::Result::Ok(s),
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(
                                ::std::string::String::from("invalid UTF-8 in str argument"),
                            )
                        }
                    }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*const ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&mut T` (mutable borrow) of an opaque handle: wire `*mut <C struct>`,
        // decode to `&mut <src>` borrowed from the C-owned `Box`. Fallible
        // (null checks).
        if r.mutability.is_some() {
            if !self.opaque.contains_key(&TypeKey::from_type(t1)) {
                return None;
            }
            let ref_ty: syn::Type = syn::parse_quote!(&mut #t1);
            let name = Self::in_name(&ref_ty);
            let c_struct = self.c_type_ident(t1);
            let src = self.src_ty(t1);
            let short = type_short(t1);
            let null_ptr_msg = format!("null {short} pointer");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: *mut #c_struct,
                ) -> ::core::result::Result<&'a mut #src, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_ptr_msg),
                        );
                    }
                    ::core::result::Result::Ok(&mut *(v as *mut #src))
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*mut #c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&T` (shared borrow) of an opaque handle: wire `*const <C struct>`,
        // decode to `&<src>` borrowed from the C-owned `Box`. Fallible (null
        // checks). Non-opaque inners fall through.
        if !self.opaque.contains_key(&TypeKey::from_type(t1)) {
            return None;
        }
        let ref_ty: syn::Type = syn::parse_quote!(&#t1);
        let name = Self::in_name(&ref_ty);
        let c_struct = self.c_type_ident(t1);
        let src = self.src_ty(t1);
        let short = type_short(t1);
        let null_ptr_msg = format!("null {short} pointer");
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name<'a>(
                v: *const #c_struct,
            ) -> ::core::result::Result<&'a #src, ::std::string::String> {
                if v.is_null() {
                    return ::core::result::Result::Err(::std::string::String::from(#null_ptr_msg));
                }
                ::core::result::Result::Ok(&*(v as *const #src))
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(*const #c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_input_type_rank_2(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_input_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    /// `impl Fn(Args...) + Send + Sync + 'static` callback input. The C wire is a
    /// by-value closure struct (`{ void *context; call; drop }`, emitted in
    /// `prerequisites`); the converter rebuilds a Rust closure that, on each
    /// invocation, encodes its args through their **output** converters (the
    /// args travel Rust→C when the callback fires — they're owned handles the C
    /// `call` is responsible for dropping) and invokes the C function pointer.
    /// An `Arc<Ctx>` carries the `void *context` + `drop`, releasing it (once,
    /// `Send + Sync`) when the Rust closure is dropped. Only signatures declared
    /// via [`Cbindgen::callback`] are handled.
    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        let key: CallbackKey = args.iter().map(TypeKey::from_type).collect();
        if !self.callbacks.contains_key(&key) {
            // Undeclared callback signature: leave unresolved so the registry
            // reports it (the consumer must `.callback(...)`-declare it).
            return None;
        }
        let c_struct = self.callback_c_ident(args);

        // Per-arg: closure parameter (`__aN: <src>`) + encode statement
        // (`let __wN = <output_conv>(__aN);`, panicking if the converter is
        // fallible — a firing callback has no error channel) + the `__wN` passed
        // to the C `call`.
        let mut closure_params: Vec<TokenStream> = Vec::new();
        let mut encode_stmts: Vec<TokenStream> = Vec::new();
        let mut wire_idents: Vec<syn::Ident> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let entry = registry.output_entry(arg)?;
            let conv = entry.function.sig.ident.clone();
            let fallible = matches!(
                &entry.function.sig.output,
                syn::ReturnType::Type(_, ty) if is_result(ty)
            );
            let src = self.src_ty(arg);
            let ai = format_ident!("__a{}", i);
            let wi = format_ident!("__w{}", i);
            closure_params.push(quote!(#ai: #src));
            if fallible {
                encode_stmts.push(quote!(
                    let #wi = match #conv(#ai) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__e) => {
                            ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                        }
                    };
                ));
            } else {
                encode_stmts.push(quote!(let #wi = #conv(#ai);));
            }
            wire_idents.push(wi);
        }

        let fn_ty = callback_fn_type(&args.iter().map(|a| self.src_ty(a)).collect::<Vec<_>>());
        let name = format_ident!("__cbg_in_{}", self.callback_c_name(args));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(c: #c_struct) -> #fn_ty {
                struct __Ctx {
                    context: *mut ::core::ffi::c_void,
                    drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
                }
                unsafe impl ::core::marker::Send for __Ctx {}
                unsafe impl ::core::marker::Sync for __Ctx {}
                impl ::core::ops::Drop for __Ctx {
                    fn drop(&mut self) {
                        if let ::core::option::Option::Some(__d) = self.drop {
                            unsafe { __d(self.context) }
                        }
                    }
                }
                let __call = c.call;
                let __ctx = ::std::sync::Arc::new(__Ctx { context: c.context, drop: c.drop });
                move |#(#closure_params),*| {
                    #(#encode_stmts)*
                    if let ::core::option::Option::Some(__f) = __call {
                        unsafe { __f(#(#wire_idents,)* __ctx.context) }
                    }
                }
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(#c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    // ── Output direction (rust → wire) ─────────────────────────────────

    fn on_output_type_rank_0(
        &self,
        ty: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // Unit return: trivial converter so `()` (and `Result<(), _>`) resolves.
        // Never actually called — void-returning wrappers ignore it, and
        // `emit_fallible_wrapper` special-cases `Result<(), E>` to drop the
        // out-param entirely (it exists only to satisfy the resolver).
        if is_unit(ty) {
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused_variables)]
                pub(crate) fn __cbg_out_unit(v: ()) {}
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `String` output: encode into the owning `cbg_string_t` helper so C
        // callers get an explicit destructor instead of a raw `char **`.
        if is_string(ty) {
            let name = Self::out_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: ::std::string::String) -> *mut ::core::ffi::c_char {
                    __cbg_alloc_cstr(v)
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*mut ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // FFI-safe scalar (`bool`, integers, floats): identity pass-through.
        if is_scalar(ty) {
            let name = Self::out_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #ty) -> #ty {
                    v
                }
            );
            return Some(ConverterImpl {
                destination: ty.clone(),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        let key = TypeKey::from_type(ty);

        // Opaque handle output: `Box::into_raw` → the bare `*mut #c_struct` handle.
        if self.opaque.contains_key(&key) {
            let name = Self::out_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> *mut #c_struct {
                    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut #c_struct
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*mut #c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Data struct output: encode each field into its C wire (`String` →
        // malloc'd `char*` raw block, freed by the `free_memory_function`).
        if self.data.contains_key(&key) {
            let fields = self.struct_fields(_r, ty)?;
            let name = Self::out_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let mut inits: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                if is_string(fty) {
                    inits.push(quote!(#fname: __cbg_alloc_cstr(v.#fname)));
                } else {
                    inits.push(quote!(#fname: v.#fname));
                }
            }
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #c_struct {
                    #c_struct { #(#inits),* }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(#c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Enum output: `match` the source enum to the C enum.
        if self.enums.contains_key(&key) {
            let e = enum_item(_r, ty)?;
            assert_unit_variants(e);
            let name = Self::out_name(ty);
            let cname = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let arms = e.variants.iter().map(|v| {
                let id = &v.ident;
                quote!(#src::#id => #cname::#id,)
            });
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #cname {
                    match v { #(#arms)* }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(#cname),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        None
    }

    fn on_output_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // Composite output layers (`Option<T>`, `Vec<T>`): the structural
        // lowering happens in `emit_function_wrapper` via `lower_value`. These
        // markers exist only so the resolver marks the composite type resolved
        // and propagates required-ness to the inner/element type (resolved here
        // first, deepest-first). The marker fn is never called.
        if is_option(pat) || is_vec(pat) {
            _r.output_entry(t1)?;
            let kind = if is_option(pat) { "option" } else { "vec" };
            let name = format_ident!(
                "__cbg_outmark_{}_{}",
                kind,
                sanitize(&TypeKey::from_type(t1))
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&T` (any shared borrow — `&'static`, `&'a`, or elided) of an opaque
        // handle → `*const <C struct>`: a const, **non-owning** pointer that
        // reinterprets the borrow with no allocation. Signals to C callers that
        // the value must NOT be freed (it is loaned from the receiver / a static).
        // Composes under `Option<&T>` (NULL niche) for nullable loaned returns.
        let syn::Type::Reference(r) = pat else {
            return None;
        };
        if r.mutability.is_some() {
            return None;
        }
        let key = TypeKey::from_type(t1);
        self.opaque.get(&key)?;

        let c_struct = self.c_type_ident(t1);
        let src = self.src_ty(t1);
        // Name off the concrete inner `t1` (not the `&_` wildcard pattern), so
        // distinct borrowed-return types don't collide on one converter ident.
        let name = format_ident!("__cbg_out_ref_{}", sanitize(&TypeKey::from_type(t1)));
        // Elided input lifetime: the raw-pointer output carries no lifetime, so
        // this accepts a borrow of any lifetime (a `&'static` coerces in).
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) unsafe fn #name(v: &#src) -> *const #c_struct {
                v as *const #src as *const #c_struct
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(*const #c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_output_type_rank_2(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Result<T, E>` return: the resolver needs *some* converter so the
        // entry resolves and its inner T / E become required. The real lowering
        // (bool + out-param + error-param) happens in `on_function`; this marker
        // function is never called.
        if !is_result(pat) {
            return None;
        }
        let _ = t1;
        let name = format_ident!("__cbg_result_{}", sanitize(&TypeKey::from_type(pat)));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        // Marker only — `emit_function_wrapper` does the real lowering via
        // `lower_value`. Destination is unused (the success type may itself be a
        // composite like `Option<Vec<_>>` with no single C type).
        Some(ConverterImpl {
            destination: syn::parse_quote!(()),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_output_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }
}

impl Cbindgen {
    /// Whether the generated layer hands `char*` data memory to C — a `String`
    /// return value, or a declared data struct that is produced as output and has
    /// a `String` field. When true, a `free_memory_function` must be declared.
    fn needs_free(&self, registry: &Registry<()>) -> bool {
        let string_ty: syn::Type = syn::parse_quote!(String);
        if registry.output_entry(&string_ty).is_some() {
            return true;
        }
        self.data.keys().any(|key| {
            let ty = key.to_type();
            registry.output_entry(&ty).is_some()
                && self
                    .struct_fields(registry, &ty)
                    .map(|fields| fields.iter().any(|(_, fty)| is_string(fty)))
                    .unwrap_or(false)
        })
    }

    /// Whether any declared function returns a `Vec<_>` (possibly nested under
    /// `Result`/`Option`), so the array builder/freer prelude must be emitted.
    fn produces_array(&self, registry: &Registry<()>) -> bool {
        self.functions.keys().any(|orig| {
            registry
                .functions
                .get(orig)
                .map(|(f, _)| match &f.sig.output {
                    syn::ReturnType::Type(_, ty) => type_contains_vec(ty),
                    syn::ReturnType::Default => false,
                })
                .unwrap_or(false)
        })
    }

    /// Fields (`name`, `type`) of a declared data struct, looked up from the
    /// registry's indexed structs. `None` if the type isn't an indexed named
    /// struct.
    fn struct_fields(
        &self,
        registry: &Registry<()>,
        ty: &syn::Type,
    ) -> Option<Vec<(syn::Ident, syn::Type)>> {
        let ident = type_path_tail(ty)?;
        let (item, _) = registry.structs.get(&ident)?;
        if let syn::Fields::Named(named) = &item.fields {
            Some(
                named
                    .named
                    .iter()
                    .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Exported `#[no_mangle]` symbol for a declared function: pinned `c_name`,
    /// else [`Self::mangle_function`] over the ident, else the Rust ident.
    fn fn_symbol(&self, orig: &syn::Ident) -> syn::Ident {
        if let Some(n) = self.functions.get(orig).and_then(|c| c.c_name.clone()) {
            return format_ident!("{}", n);
        }
        if let Some(f) = &self.mangle_function {
            return format_ident!("{}", f(&orig.to_string()));
        }
        orig.clone()
    }

    /// Assemble the `#[no_mangle] extern "C"` wrapper for one declared fn.
    fn emit_function_wrapper(&self, f: &syn::ItemFn, registry: &Registry<()>) -> TokenStream {
        let orig = &f.sig.ident;
        let call_path = self.src_fn(orig);
        let sym = self.fn_symbol(orig);

        let return_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };

        let has_fallible_input = f.sig.inputs.iter().any(|input| {
            let syn::FnArg::Typed(pt) = input else {
                return false;
            };
            registry
                .input_entry(&pt.ty)
                .map(|e| returns_result(&e.function.sig.output))
                .unwrap_or(false)
        });

        // Peel an outer `Result<_, E>`; `value_ty` is the success/return value.
        let (value_ty, err_ty): (syn::Type, Option<syn::Type>) = match result_parts(&return_ty) {
            Some((ok, e)) => (ok, Some(e)),
            None => (return_ty.clone(), None),
        };

        // Error wiring: the error type must be declared via `.error()`.
        let err_bits = err_ty.as_ref().map(|err_ty| {
            assert!(
                self.error.contains(&TypeKey::from_type(err_ty)),
                "Cbindgen: function `{}` returns `Result<_, {}>` but `{}` is not a \
                 declared error type — add `.data_struct({}).error()`",
                orig,
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
                TypeKey::from_type(err_ty),
            );
            let entry = registry.output_entry(err_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: error type `{}` of `{}` has no output converter",
                    TypeKey::from_type(err_ty),
                    orig
                )
            });
            (
                entry.destination.clone(),
                entry.function.sig.ident.clone(),
                self.src_ty(err_ty),
            )
        });

        // No `Result` channel ⇒ a fallible input must be declared `.panic()`.
        if err_ty.is_none() {
            let allows_panic = self.functions.get(orig).map(|c| c.panic).unwrap_or(false);
            assert!(
                !has_fallible_input || allows_panic,
                "Cbindgen: function `{}` has a fallible input (e.g. a `String` or \
                 opaque-by-value argument) but does not return `Result`; add \
                 `.panic()` after its `.function(...)` declaration to allow aborting \
                 on the internal error, or change its signature",
                orig,
            );
        }

        // Structural lowering of the (present/ok) value, then the null-niche rule:
        //   * Result + a free pointer niche  → NULL marks `Err` (value in-band);
        //   * Result without a free niche     → `bool` status, value to out-params;
        //   * no Result                       → field 0 is the C return, rest out.
        let shape = self.lower_shape(&value_ty, registry);
        let result_in_band = err_ty.is_some() && shape.has_niche; // value rides the return
        let field0_is_return = result_in_band || err_ty.is_none();

        // Partition fields into the (optional) C return value + out-parameters,
        // and pick C names for the out-params (see `out_param_name`).
        let mut targets: Vec<TokenStream> = Vec::new();
        let mut out_fields: Vec<&WireField> = Vec::new();
        // `field0_wire` is the wire of the value's primary field when that field
        // is carried by the C return slot (modes A/D); `None` for mode B and unit.
        let field0_wire: Option<syn::Type> = if field0_is_return {
            shape.fields.first().map(|f| f.wire.clone())
        } else {
            None
        };
        if field0_is_return {
            if !shape.fields.is_empty() {
                targets.push(quote!(__ret));
                out_fields.extend(shape.fields[1..].iter());
            }
        } else {
            out_fields.extend(shape.fields.iter());
        }
        let prefixed = out_fields.iter().any(|wf| wf.suffix.is_empty());
        let out_names: Vec<syn::Ident> = out_fields
            .iter()
            .map(|wf| out_param_name(wf.suffix, prefixed))
            .collect();
        for name in &out_names {
            targets.push(quote!(*#name));
        }
        let out_param_decls: Vec<TokenStream> = out_fields
            .iter()
            .zip(&out_names)
            .map(|(wf, name)| {
                let wire = &wf.wire;
                quote!(#name: *mut #wire)
            })
            .collect();

        // C wrapper return type: the payload's field 0 (modes A/D), `bool` status
        // (mode B), or `void` (a unit value with no `Result`).
        let c_return: Option<syn::Type> = if field0_is_return {
            field0_wire.clone()
        } else {
            Some(syn::parse_quote!(bool))
        };

        // Input decode: route a fallible-input failure to the error out-param
        // (with the wrapper's fail value) when there is a `Result`, else panic.
        let fail_return = if result_in_band {
            null_for(field0_wire.as_ref().expect("in-band ⇒ pointer return"))
        } else {
            quote!(false)
        };
        let input_route = match &err_bits {
            Some((_, e_conv, e_ty_src)) => ErrRoute::Result {
                e_conv,
                e_ty_src: e_ty_src.clone(),
                fail_return: fail_return.clone(),
            },
            None => ErrRoute::Panic,
        };
        let (in_params, decodes, call_args) = self.emit_inputs(orig, f, registry, &input_route);
        let call = quote!(#call_path(#(#call_args),*));

        let e_param = err_bits
            .as_ref()
            .map(|(err_wire, _, _)| quote!(e: *mut #err_wire));
        let ret_arrow = c_return.as_ref().map(|w| quote!(-> #w));

        // Assemble the body per the three structural modes.
        let body = match (&err_bits, field0_is_return) {
            // No `Result`: straight-line. `void` when there are no fields.
            (None, _) => {
                if field0_wire.is_none() {
                    quote!( #(#decodes)* #call; )
                } else {
                    let field0_wire = field0_wire.as_ref().unwrap();
                    let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                    quote!(
                        #(#decodes)*
                        let __v = #call;
                        let __ret: #field0_wire;
                        #enc
                        __ret
                    )
                }
            }
            // `Result` with a free niche: value in-band, NULL marks `Err`.
            (Some((_, e_conv, _)), true) => {
                let field0_wire = field0_wire.as_ref().expect("in-band ⇒ pointer return");
                let null = null_for(field0_wire);
                let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { let __ret: #field0_wire; #enc __ret }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            #null
                        }
                    }
                )
            }
            // `Result` without a free niche: `bool` status, value to out-params.
            (Some((_, e_conv, _)), false) => {
                let enc = self.encode_value(&value_ty, quote!(__v), &targets, registry);
                quote!(
                    #(#decodes)*
                    match #call {
                        ::core::result::Result::Ok(__v) => { #enc true }
                        ::core::result::Result::Err(__err) => {
                            if !e.is_null() { *e = #e_conv(__err); }
                            false
                        }
                    }
                )
            }
        };

        quote! {
            #[no_mangle]
            #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
            pub unsafe extern "C" fn #sym(
                #(#in_params,)*
                #(#out_param_decls,)*
                #e_param
            ) #ret_arrow {
                #body
            }
        }
    }

    /// Lower how a *present / ok* value of `ty` is carried over the C ABI: an
    /// ordered list of wire components, plus whether `fields[0]` is a pointer
    /// whose NULL bit-pattern is still free for an enclosing `Option`/`Result`
    /// layer to claim. Mirrors the niche-stacking model in `core::niches`.
    fn lower_shape(&self, ty: &syn::Type, registry: &Registry<()>) -> ValueShape {
        if is_unit(ty) {
            return ValueShape {
                fields: vec![],
                has_niche: false,
            };
        }
        // `Vec<T>` → `T_wire* + size_t`. The element must lower to a single C
        // value (one converter); a composite element is unsupported.
        if is_vec(ty) {
            let elem = first_type_arg(ty).expect("Vec<T> has a type argument");
            assert!(
                !is_option(&elem) && !is_vec(&elem) && !is_result(&elem),
                "Cbindgen: `Vec<{}>` element must be a single-value type \
                 (scalar, data struct, String, or handle), not a composite",
                TypeKey::from_type(&elem),
            );
            let entry = registry.output_entry(&elem).unwrap_or_else(|| {
                panic!(
                    "Cbindgen: `Vec` element `{}` has no output converter",
                    TypeKey::from_type(&elem)
                )
            });
            let elem_wire = entry.destination.clone();
            return ValueShape {
                fields: vec![
                    WireField {
                        suffix: "",
                        wire: syn::parse_quote!(*mut #elem_wire),
                    },
                    WireField {
                        suffix: "_len",
                        wire: syn::parse_quote!(usize),
                    },
                ],
                has_niche: false,
            };
        }
        // `Option<T>` consumes one discriminant. If the inner value still has a
        // free pointer niche, reuse it (NULL = `None`); otherwise prepend an
        // explicit `present: bool`. Either way the result exposes no niche.
        if is_option(ty) {
            let inner_ty = first_type_arg(ty).expect("Option<T> has a type argument");
            let inner = self.lower_shape(&inner_ty, registry);
            if inner.has_niche {
                return ValueShape {
                    fields: inner.fields,
                    has_niche: false,
                };
            }
            let mut fields = vec![WireField {
                suffix: "_present",
                wire: syn::parse_quote!(bool),
            }];
            fields.extend(inner.fields);
            return ValueShape {
                fields,
                has_niche: false,
            };
        }
        // Base value: one wire component from its rank-0/1 converter. A pointer
        // wire (String, opaque handle, `&'static`) carries a free NULL niche.
        let entry = registry.output_entry(ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: type `{}` has no output converter",
                TypeKey::from_type(ty)
            )
        });
        let wire = entry.destination.clone();
        let has_niche = matches!(wire, syn::Type::Ptr(_));
        ValueShape {
            fields: vec![WireField { suffix: "", wire }],
            has_niche,
        }
    }

    /// Emit the statements that write a native value `val` of type `ty` into the
    /// `targets` lvalues (one per field of `lower_shape(ty)`, in order).
    fn encode_value(
        &self,
        ty: &syn::Type,
        val: TokenStream,
        targets: &[TokenStream],
        registry: &Registry<()>,
    ) -> TokenStream {
        if is_unit(ty) {
            return quote!();
        }
        if is_vec(ty) {
            let elem = first_type_arg(ty).expect("Vec<T> has a type argument");
            let entry = registry.output_entry(&elem).expect("Vec element converter");
            let elem_conv = entry.function.sig.ident.clone();
            let elem_wire = entry.destination.clone();
            let t_ptr = &targets[0];
            let t_len = &targets[1];
            return quote!(
                let __arr: ::std::vec::Vec<#elem_wire> =
                    #val.into_iter().map(#elem_conv).collect();
                let (__p, __n) = __cbg_alloc_array(__arr);
                #t_ptr = __p;
                #t_len = __n;
            );
        }
        if is_option(ty) {
            let inner_ty = first_type_arg(ty).expect("Option<T> has a type argument");
            let inner = self.lower_shape(&inner_ty, registry);
            if inner.has_niche {
                // None reuses the inner pointer's NULL; Some encodes inline.
                let inner_enc = self.encode_value(&inner_ty, quote!(__x), targets, registry);
                let null = null_for(&inner.fields[0].wire);
                let t0 = &targets[0];
                return quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => { #inner_enc }
                        ::core::option::Option::None => { #t0 = #null; }
                    }
                );
            }
            // Explicit `present` flag in targets[0]; inner value follows.
            let present = &targets[0];
            let inner_enc = self.encode_value(&inner_ty, quote!(__x), &targets[1..], registry);
            return quote!(
                match #val {
                    ::core::option::Option::Some(__x) => { #present = true; #inner_enc }
                    ::core::option::Option::None => { #present = false; }
                }
            );
        }
        // Base value: run its output converter into the single target.
        let entry = registry.output_entry(ty).expect("base value converter");
        let conv = entry.function.sig.ident.clone();
        let t0 = &targets[0];
        quote!( #t0 = #conv(#val); )
    }

    /// Build the wire param list, per-input decode statements, and call-site
    /// argument expressions. Fallible inputs (converter returns `Result<_,
    /// String>`) route their `Err(msg)` per `route`; infallible inputs decode
    /// directly.
    fn emit_inputs(
        &self,
        orig: &syn::Ident,
        f: &syn::ItemFn,
        registry: &Registry<()>,
        route: &ErrRoute,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let mut params = Vec::new();
        let mut decodes = Vec::new();
        let mut call_args = Vec::new();

        for input in &f.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let syn::Pat::Ident(pat_id) = &*pt.pat else {
                continue;
            };
            let ident = &pat_id.ident;
            let arg_ty = &*pt.ty;

            // `&[E]` slice (scalar `E`): two wire params (`*const E`, `usize`),
            // decoded zero-copy. NULL pointer ⇒ empty slice (not an error).
            if let Some(elem) = scalar_slice_elem(arg_ty) {
                let len_id = format_ident!("{}_len", ident);
                params.push(quote!(#ident: *const #elem));
                params.push(quote!(#len_id: usize));
                decodes.push(quote!(
                    let #ident: &[#elem] = if #ident.is_null() {
                        &[]
                    } else {
                        ::core::slice::from_raw_parts(#ident, #len_id)
                    };
                ));
                call_args.push(quote!(#ident));
                continue;
            }

            let entry = registry.input_entry(arg_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: input type `{}` of `{}` has no input converter",
                    TypeKey::from_type(arg_ty),
                    orig
                )
            });
            let wire = &entry.destination;
            let conv = &entry.function.sig.ident;

            params.push(quote!(#ident: #wire));

            if returns_result(&entry.function.sig.output) {
                let on_err = match route {
                    ErrRoute::Result {
                        e_conv,
                        e_ty_src,
                        fail_return,
                    } => quote!(
                        if !e.is_null() {
                            *e = #e_conv(<#e_ty_src as ::core::convert::From<::std::string::String>>::from(__msg));
                        }
                        return #fail_return;
                    ),
                    ErrRoute::Panic => quote!(panic!("{}", __msg);),
                };
                decodes.push(quote!(
                    let #ident = match #conv(#ident) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__msg) => { #on_err }
                    };
                ));
            } else {
                decodes.push(quote!(let #ident = #conv(#ident);));
            }

            // Each input converter produces exactly the source param type
            // (`String` by value, `&T` for borrows, owned `T` for consume), so
            // the decoded binding is passed straight through.
            call_args.push(quote!(#ident));
        }

        (params, decodes, call_args)
    }
}

// ── Free helpers ───────────────────────────────────────────────────────

/// Iterate a `TypeKey`-keyed map in deterministic (key-string) order.
fn sorted_by_key(map: &HashMap<TypeKey, TypeCfg>) -> Vec<(&TypeKey, &TypeCfg)> {
    let mut entries: Vec<(&TypeKey, &TypeCfg)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    entries
}

/// Turn a `TypeKey` into a valid ident fragment (non-alphanumerics → `_`).
fn sanitize(key: &TypeKey) -> String {
    key.as_str()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Last path-segment ident of a type as a `String` (e.g. `ZKeyExpr`).
fn type_short(ty: &syn::Type) -> String {
    type_path_tail(ty)
        .map(|i| i.to_string())
        .unwrap_or_else(|| sanitize(&TypeKey::from_type(ty)))
}

/// The indexed `syn::ItemEnum` for a declared enum type, by tail ident.
fn enum_item<'r>(registry: &'r Registry<()>, ty: &syn::Type) -> Option<&'r syn::ItemEnum> {
    let ident = type_path_tail(ty)?;
    registry.enums.get(&ident).map(|(e, _)| e)
}

/// Hard error on a non-C-like enum (only fieldless / unit variants supported).
fn assert_unit_variants(e: &syn::ItemEnum) {
    for v in &e.variants {
        assert!(
            matches!(v.fields, syn::Fields::Unit),
            "Cbindgen: enum `{}` variant `{}` has fields; only C-like (fieldless) \
             enums are supported",
            e.ident,
            v.ident
        );
    }
}

/// Last path-segment ident of a path type.
fn type_path_tail(ty: &syn::Type) -> Option<syn::Ident> {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.clone())
    } else {
        None
    }
}

/// PascalCase → snake_case (`ZKeyExpr` → `z_key_expr`).
/// Convert a `PascalCase` / `camelCase` identifier to `snake_case` (a
/// convention-free helper, re-exported as `prebindgen::lang::snake_case` for
/// consumers composing their own [`Cbindgen::mangle_rust_type`] rules).
pub fn snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn is_string(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "String").unwrap_or(false)
}

fn is_str(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "str").unwrap_or(false)
}

/// Whether `ty` is an FFI-safe scalar primitive that passes through unchanged
/// (`bool`, the fixed-width / pointer-width integers, and floats).
fn is_scalar(ty: &syn::Type) -> bool {
    type_path_tail(ty)
        .map(|i| {
            matches!(
                i.to_string().as_str(),
                "bool"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "isize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "usize"
                    | "f32"
                    | "f64"
            )
        })
        .unwrap_or(false)
}

fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

fn is_result(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "Result").unwrap_or(false)
}

fn is_option(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "Option").unwrap_or(false)
}

fn is_vec(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "Vec").unwrap_or(false)
}

/// Whether `Vec<_>` appears anywhere in `ty` (including nested under
/// `Result`/`Option`/references).
fn type_contains_vec(ty: &syn::Type) -> bool {
    is_vec(ty)
        || crate::api::core::registry::immediate_subtype_positions(ty)
            .iter()
            .any(type_contains_vec)
}

/// First angle-bracketed type argument of a path type (e.g. `T` of `Option<T>`
/// or `Vec<T>`). `None` if there is none.
fn first_type_arg(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    })
}

/// If `ty` is `&[E]` (a shared slice borrow) with scalar `E`, return `E`.
fn scalar_slice_elem(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Reference(r) = ty else {
        return None;
    };
    if r.mutability.is_some() {
        return None;
    }
    let syn::Type::Slice(s) = &*r.elem else {
        return None;
    };
    let elem = (*s.elem).clone();
    is_scalar(&elem).then_some(elem)
}

/// C name for an out-parameter field. When the value's primary field (suffix
/// `""`) is itself an out-param the whole group is `out`-prefixed (`out`,
/// `out_len`, `out_present`); otherwise the accompanying fields use bare names
/// (`len`, `present`).
fn out_param_name(suffix: &str, prefixed: bool) -> syn::Ident {
    if prefixed {
        format_ident!("out{}", suffix)
    } else {
        format_ident!("{}", suffix.trim_start_matches('_'))
    }
}

/// NULL literal matching a raw-pointer wire: `null_mut()` for `*mut`, else `null()`.
fn null_for(wire: &syn::Type) -> TokenStream {
    match wire {
        syn::Type::Ptr(p) if p.mutability.is_some() => quote!(::core::ptr::null_mut()),
        _ => quote!(::core::ptr::null()),
    }
}

/// One C-ABI wire component of a lowered return value. `suffix` names it
/// relative to a base (`""` → `out`, `"_len"` → `len`, `"_present"` → `present`).
struct WireField {
    suffix: &'static str,
    wire: syn::Type,
}

/// How a *present / ok* value of a return type is carried over the C ABI:
/// an ordered list of wire components, plus whether `fields[0]` is a pointer
/// whose NULL bit-pattern is still free for an enclosing `Option`/`Result`
/// layer to claim (the niche).
struct ValueShape {
    fields: Vec<WireField>,
    has_niche: bool,
}

/// Whether a converter function's return type is `Result<_, _>` (⇒ fallible).
fn returns_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => is_result(ty),
        syn::ReturnType::Default => false,
    }
}

/// If `ty` is `Result<T, E>`, return `(T, E)`.
fn result_parts(ty: &syn::Type) -> Option<(syn::Type, syn::Type)> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let mut tys = ab.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    });
    let t = tys.next()?;
    let e = tys.next()?;
    Some((t, e))
}

/// C-ABI wire type for a struct field. `String` → `*mut c_char`; FFI-safe
/// scalars pass through. `None` for anything else (unsupported this increment).
fn c_field_wire(ty: &syn::Type) -> Option<syn::Type> {
    if is_string(ty) {
        return Some(syn::parse_quote!(*mut ::core::ffi::c_char));
    }
    if is_scalar(ty) {
        return Some(ty.clone());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;

    fn write(cbindgen: &Cbindgen, registry: &mut Registry<()>, tag: &str) -> String {
        let dir = std::env::temp_dir().join(format!("cbindgen_{}_{}", tag, std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join(format!("{tag}.rs"));
        let path = registry.write_rust(cbindgen, &out).expect("write_rust");
        std::fs::read_to_string(&path).unwrap()
    }

    fn error_struct() -> syn::ItemStruct {
        syn::parse_quote!(
            pub struct Error {
                pub message: String,
            }
        )
    }

    /// An adapter with no declarations writes an empty (whitespace-only) file.
    #[test]
    fn empty_adapter_writes_empty_file() {
        let cbindgen = Cbindgen::new();
        let mut registry: Registry<()> = Registry::default();
        let src = write(&cbindgen, &mut registry, "empty");
        assert!(src.trim().is_empty(), "expected empty output, got:\n{src}");
    }

    /// `z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error>` lowers to a
    /// **pointer-returning** wrapper (opaque handle, NULL on error); decode
    /// failures route through `From<String>` into the declared error type.
    #[test]
    fn keyexpr_try_from_lowering() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
                unimplemented!()
            }
        );

        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .name("z_keyexpr")
            .destructor_name("z_keyexpr_free")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_keyexpr_try_from));

        let src = write(&cbindgen, &mut registry, "keyexpr");
        // Whitespace-insensitive haystack (the file is prettyplease-formatted).
        let compact: String = src.split_whitespace().collect();

        // Pointer-return wrapper: returns the opaque handle, no `out` param.
        assert!(compact.contains("extern\"C\"fnz_keyexpr_try_from"), "{src}");
        assert!(compact.contains("->*mutz_keyexpr"), "{src}");
        assert!(!compact.contains("out:*mut"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
        // Opaque handle marker struct + typed (pinned) destructor on the bare ptr.
        assert!(compact.contains("structz_keyexpr{_private"), "{src}");
        assert!(compact.contains("structz_error"), "{src}");
        assert!(
            compact.contains("fnz_keyexpr_free(this_:*mutz_keyexpr"),
            "{src}"
        );
        assert!(
            compact.contains("Box::from_raw(this_as*mutzenoh_flat::ZKeyExpr)"),
            "{src}"
        );
        // String memory ⇒ malloc/free decls + a single `z_free`; no per-type
        // string/error destructors.
        assert!(compact.contains("fnmalloc(size:usize)"), "{src}");
        assert!(
            compact.contains("fnz_free(p:*mut::core::ffi::c_void)"),
            "{src}"
        );
        assert!(!compact.contains("z_error_drop"), "{src}");
        assert!(!compact.contains("cbg_string_t"), "{src}");
        // Source call fully qualified.
        assert!(compact.contains("zenoh_flat::z_keyexpr_try_from"), "{src}");
        // Error model: decode failure routes via From<String> through the declared
        // error's output converter, and the failing return is NULL.
        assert!(!compact.contains("__CErr"), "{src}");
        assert!(
            compact.contains("as::core::convert::From<::std::string::String"),
            "{src}"
        );
        assert!(compact.contains("__cbg_out_Error"), "{src}");
        assert!(compact.contains("return::core::ptr::null_mut()"), "{src}");
    }

    /// A `Result<(), E>` function lowers to `bool f(<inputs>, E *e)` — no
    /// out-param, just `true` on `Ok`.
    #[test]
    fn result_unit_omits_out_param() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_unit_op(s: String) -> Result<(), Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_unit_op));

        let src = write(&cbindgen, &mut registry, "resultunit");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("extern\"C\"fnz_unit_op"), "{src}");
        assert!(compact.contains("->bool"), "{src}");
        // Out-param dropped; error param kept.
        assert!(!compact.contains("out:*mut"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
        // Ok arm yields `true`, with no write through `out`.
        assert!(compact.contains("Result::Ok(__v)=>true"), "{src}");
        assert!(!compact.contains("*out="), "{src}");
    }

    /// `Result<String, E>` returns a bare `char*` (a `malloc`'d raw block, freed
    /// by `z_free`), NULL on error — no `cbg_string_t` wrapper.
    #[test]
    fn result_string_uses_owned_string_wire() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_config_get_json(key: String) -> Result<String, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_config_get_json));

        let src = write(&cbindgen, &mut registry, "result_string");
        let compact: String = src.split_whitespace().collect();

        assert!(!compact.contains("cbg_string_t"), "{src}");
        assert!(compact.contains("extern\"C\"fnz_config_get_json"), "{src}");
        // Returns char*, no out-param; string built via the raw malloc'd block.
        assert!(compact.contains("->*mut::core::ffi::c_char"), "{src}");
        assert!(!compact.contains("out:*mut"), "{src}");
        assert!(compact.contains("__cbg_alloc_cstr(v)"), "{src}");
        assert!(
            compact.contains("fnz_free(p:*mut::core::ffi::c_void)"),
            "{src}"
        );
        // Ok arm encodes the pointer into the return slot; error → NULL.
        assert!(compact.contains("__ret=__cbg_out_String(__v);"), "{src}");
        assert!(
            compact
                .contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}::core::ptr::null_mut()}"),
            "{src}"
        );
    }

    /// `z_encoding_schema(e: &ZEncoding) -> Option<String>` lowers to a bare
    /// `char*` return where NULL encodes `None` (a value, not an error). The
    /// fallible borrow input forces `.panic()`; there is no `out`/`e` param.
    #[test]
    fn option_string_returns_pointer_null_for_none() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_encoding_schema(e: &ZEncoding) -> Option<String> {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZEncoding))
            .name("z_encoding")
            .function(syn::parse_quote!(z_encoding_schema))
            .panic();

        let src = write(&cbindgen, &mut registry, "option_string");
        let compact: String = src.split_whitespace().collect();

        // Plain-Option wrapper: `char*` return, no out-param, no error param.
        assert!(compact.contains("extern\"C\"fnz_encoding_schema"), "{src}");
        assert!(compact.contains("->*mut::core::ffi::c_char"), "{src}");
        assert!(!compact.contains("out:*mut"), "{src}");
        assert!(!compact.contains("e:*mut"), "{src}");
        // Inline Option encoding into the return slot: Some → inner wire, None → NULL.
        assert!(
            compact.contains("::core::option::Option::Some(__x)=>{__ret=__cbg_out_String(__x);}"),
            "{src}"
        );
        assert!(
            compact.contains("::core::option::Option::None=>{__ret=::core::ptr::null_mut();}"),
            "{src}"
        );
        // Fallible borrow decode aborts (no Result channel).
        assert!(compact.contains("panic!("), "{src}");
    }

    /// `Result<Option<T>, E>` cannot use NULL for both `None` and error, so it
    /// takes the value-wire shape: `bool f(T **out, …, E *e)`. `None` writes a
    /// NULL into `*out` and still returns `true`.
    #[test]
    fn result_option_uses_out_param() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_get_opt(key: String) -> Result<Option<ZThing>, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZThing))
            .name("z_thing")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_get_opt));

        let src = write(&cbindgen, &mut registry, "result_option");
        let compact: String = src.split_whitespace().collect();

        // Value-wire shape: bool return, pointer-to-pointer out-param, error param.
        assert!(compact.contains("extern\"C\"fnz_get_opt"), "{src}");
        assert!(compact.contains("->bool"), "{src}");
        assert!(compact.contains("out:*mut*mutz_thing"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
        // Ok arm writes the Option (pointer-or-NULL) through `out`, returns true.
        assert!(compact.contains("*out=__cbg_out_ZThing(__x);"), "{src}");
        assert!(
            compact.contains("::core::option::Option::None=>{*out=::core::ptr::null_mut();}"),
            "{src}"
        );
        assert!(
            compact.contains("=>{") && compact.contains("true}"),
            "{src}"
        );
    }

    /// `Vec<String>` lowers to `char** f(<inputs>, size_t* len)`: the malloc'd
    /// array pointer is returned, the element count goes to `*len`. Each element
    /// is encoded via the inner `String` converter.
    #[test]
    fn vec_string_returns_ptr_and_len() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_hello_locators(h: &ZHello) -> Vec<String> {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZHello))
            .name("z_hello")
            .function(syn::parse_quote!(z_hello_locators))
            .panic();

        let src = write(&cbindgen, &mut registry, "vec_string");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("extern\"C\"fnz_hello_locators"), "{src}");
        // Returns `char**`, with a trailing `len` out-param; no `out`/`e`.
        assert!(compact.contains("->*mut*mut::core::ffi::c_char"), "{src}");
        assert!(compact.contains("len:*mutusize"), "{src}");
        assert!(!compact.contains("e:*mut"), "{src}");
        // Built from the element converter via the malloc'd array helper.
        assert!(
            compact.contains(".map(__cbg_out_String).collect()"),
            "{src}"
        );
        assert!(
            compact.contains("let(__p,__n)=__cbg_alloc_array(__arr);"),
            "{src}"
        );
        assert!(
            compact.contains("__ret=__p;") && compact.contains("*len=__n;"),
            "{src}"
        );
        // The array builder prelude is emitted.
        assert!(compact.contains("fn__cbg_alloc_array<W>"), "{src}");
        // Fallible borrow decode aborts (no Result channel).
        assert!(compact.contains("panic!("), "{src}");
    }

    /// `Vec<u8>` lowers to a scalar array `uint8_t* f(<inputs>, size_t* len)` —
    /// elements pass through (no per-element pointer).
    #[test]
    fn vec_u8_returns_scalar_array() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZZBytes))
            .name("z_zbytes")
            .function(syn::parse_quote!(z_zbytes_to_bytes))
            .panic();

        let src = write(&cbindgen, &mut registry, "vec_u8");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("->*mutu8"), "{src}");
        assert!(compact.contains("len:*mutusize"), "{src}");
        assert!(compact.contains("__cbg_alloc_array(__arr)"), "{src}");
    }

    /// `Result<Vec<T>, E>` has no free niche (the array NULL means *empty*), so
    /// it takes `bool f(T** out, size_t* out_len, <inputs>, E* e)`.
    #[test]
    fn result_vec_uses_out_params() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_things(key: String) -> Result<Vec<ZThing>, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZThing))
            .name("z_thing")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_things));

        let src = write(&cbindgen, &mut registry, "result_vec");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("->bool"), "{src}");
        assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
        assert!(compact.contains("out_len:*mutusize"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
        // Ok writes both out-params; Err writes `*e` and returns false.
        assert!(
            compact.contains("*out=__p;") && compact.contains("*out_len=__n;"),
            "{src}"
        );
    }

    /// `Option<Vec<T>>` (no `Result`): the inner `Vec` has no niche, so an
    /// explicit `present` flag rides the `bool` return while the array goes to
    /// `out`/`out_len`.
    #[test]
    fn option_vec_uses_present_and_out() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_maybe_things(h: &ZHello) -> Option<Vec<ZThing>> {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZHello))
            .name("z_hello")
            .ptr_struct(syn::parse_quote!(ZThing))
            .name("z_thing")
            .function(syn::parse_quote!(z_maybe_things))
            .panic();

        let src = write(&cbindgen, &mut registry, "option_vec");
        let compact: String = src.split_whitespace().collect();

        // `bool` return is the `present` flag; the array rides `out`/`out_len`.
        assert!(compact.contains("->bool"), "{src}");
        assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
        assert!(compact.contains("out_len:*mutusize"), "{src}");
        assert!(!compact.contains("e:*mut"), "{src}");
        assert!(
            compact.contains("__ret=true;") && compact.contains("__ret=false;"),
            "{src}"
        );
    }

    /// `Result<Option<Vec<T>>, E>`: full stack — `Result` finds no niche (Option
    /// consumed it), so `bool` status; the `present` flag and the array all ride
    /// out-params: `bool f(bool* out_present, T** out, size_t* out_len, …, E* e)`.
    #[test]
    fn result_option_vec_full() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_full(key: String) -> Result<Option<Vec<ZThing>>, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZThing))
            .name("z_thing")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_full));

        let src = write(&cbindgen, &mut registry, "result_option_vec");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("->bool"), "{src}");
        assert!(compact.contains("out_present:*mutbool"), "{src}");
        assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
        assert!(compact.contains("out_len:*mutusize"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
        // present flag set inside the Ok arm; array filled when Some.
        assert!(
            compact.contains("*out_present=true;") && compact.contains("*out_present=false;"),
            "{src}"
        );
    }

    /// A scalar slice input `&[u8]` lowers to two wire params (`*const u8`,
    /// `usize`) decoded zero-copy; a NULL pointer is an empty slice.
    #[test]
    fn slice_u8_input_two_params() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_zbytes_from_bytes(bytes: &[u8]) -> ZZBytes {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .ptr_struct(syn::parse_quote!(ZZBytes))
            .name("z_zbytes")
            .function(syn::parse_quote!(z_zbytes_from_bytes));

        let src = write(&cbindgen, &mut registry, "slice_u8");
        let compact: String = src.split_whitespace().collect();

        // Two params: pointer + length.
        assert!(compact.contains("bytes:*constu8"), "{src}");
        assert!(compact.contains("bytes_len:usize"), "{src}");
        // Zero-copy decode, NULL ⇒ empty slice.
        assert!(
            compact.contains("::core::slice::from_raw_parts(bytes,bytes_len)"),
            "{src}"
        );
        // Returns the opaque handle (Box::into_raw).
        assert!(compact.contains("->*mutz_zbytes"), "{src}");
    }

    /// `Option<ZZBytes>` input (opaque, pointer-wire inner) reuses the handle
    /// wire `z_zbytes_t*`: NULL ⇒ `None`, non-NULL is consumed via the inner
    /// converter. The inner is fallible, so the decode routes through the
    /// `Result<(), Error>` error channel.
    #[test]
    fn option_opaque_input_reuses_pointer() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_op(attachment: Option<ZZBytes>) -> Result<(), Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZZBytes))
            .name("z_zbytes")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_op));

        let src = write(&cbindgen, &mut registry, "option_in_opaque");
        let compact: String = src.split_whitespace().collect();

        // Param reuses the bare handle pointer; NULL ⇒ None.
        assert!(compact.contains("attachment:*mutz_zbytes"), "{src}");
        assert!(
            compact.contains(
                "ifv.is_null(){return::core::result::Result::Ok(::core::option::Option::None);}"
            ),
            "{src}"
        );
        // Non-null path consumes through the inner handle converter.
        assert!(compact.contains("match__cbg_in_ZZBytes(v)"), "{src}");
        // Fallible inner decode routes its error through the Result channel (`*e`).
        assert!(compact.contains("e:*mutz_error"), "{src}");
        assert!(compact.contains("__cbg_out_Error"), "{src}");
    }

    /// `Option<i64>` input (scalar inner, no niche) is boxed behind a `*const`
    /// pointer: NULL ⇒ `None`, else `*v`. Infallible.
    #[test]
    fn option_scalar_input_boxed_pointer() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_op(timestamp_ntp64: Option<i64>) {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .function(syn::parse_quote!(z_op));

        let src = write(&cbindgen, &mut registry, "option_in_scalar");
        let compact: String = src.split_whitespace().collect();

        // Boxed behind a const pointer; NULL ⇒ None, else `Some(*v)`.
        assert!(compact.contains("timestamp_ntp64:*consti64"), "{src}");
        assert!(
            compact.contains("ifv.is_null(){::core::option::Option::None}"),
            "{src}"
        );
        assert!(compact.contains("::core::option::Option::Some"), "{src}");
        // Infallible ⇒ no error param.
        assert!(!compact.contains("e:*mut"), "{src}");
    }

    /// `&str` inputs decode directly from `const char *` and can be used by
    /// non-`Result` wrappers when `.panic()` is enabled.
    #[test]
    fn str_borrow_input_lowering() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_init_logs(filter: &str) {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .function(syn::parse_quote!(z_init_logs))
            .panic();

        let src = write(&cbindgen, &mut registry, "str_borrow");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("extern\"C\"fnz_init_logs"), "{src}");
        assert!(
            compact.contains("filter:*const::core::ffi::c_char"),
            "{src}"
        );
        assert!(compact.contains("CStr::from_ptr(v).to_str()"), "{src}");
        assert!(compact.contains("panic!("), "{src}");
    }

    /// `z_keyexpr_relation_to(a: &ZKeyExpr, b: &ZKeyExpr) -> SetIntersectionLevel`
    /// lowers to a borrow-input + enum-return wrapper; `.panic()` lets the
    /// fallible borrow decode abort.
    #[test]
    fn relation_to_lowering() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_relation_to(a: &ZKeyExpr, b: &ZKeyExpr) -> SetIntersectionLevel {
                unimplemented!()
            }
        );
        let enum_item: syn::ItemEnum = syn::parse_quote!(
            pub enum SetIntersectionLevel {
                Disjoint = 0,
                Intersects = 1,
                Includes = 2,
                Equals = 3,
            }
        );

        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Enum(enum_item), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .name("z_keyexpr")
            .enum_type(syn::parse_quote!(SetIntersectionLevel))
            .name("z_intersection")
            .function(syn::parse_quote!(z_keyexpr_relation_to))
            .panic();

        let src = write(&cbindgen, &mut registry, "relation_to");
        let compact: String = src.split_whitespace().collect();

        // repr(C) enum mirror with discriminants — renamed via `.name()`.
        assert!(compact.contains("#[repr(C)]"), "{src}");
        assert!(compact.contains("pubenumz_intersection"), "{src}");
        assert!(compact.contains("Disjoint=0"), "{src}");
        // Wrapper: borrow params (renamed type) + enum return.
        assert!(
            compact.contains("extern\"C\"fnz_keyexpr_relation_to"),
            "{src}"
        );
        assert!(compact.contains("a:*constz_keyexpr"), "{src}");
        assert!(compact.contains("b:*constz_keyexpr"), "{src}");
        assert!(compact.contains("->z_intersection"), "{src}");
        // Fallible borrow decode aborts (no Result channel).
        assert!(compact.contains("panic!("), "{src}");
        // Enum output converter matches by variant name (src enum → C enum).
        assert!(
            compact
                .contains("zenoh_flat::SetIntersectionLevel::Disjoint=>z_intersection::Disjoint"),
            "{src}"
        );
    }

    /// A mutable borrow of an opaque handle lowers to `*mut <handle>` and
    /// decodes back to `&mut T`.
    #[test]
    fn mutable_opaque_borrow_input_lowering() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_config_insert_json5(
                c: &mut ZConfig,
                key: String,
                value: String,
            ) -> Result<(), Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZConfig))
            .name("z_config")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_config_insert_json5));

        let src = write(&cbindgen, &mut registry, "mut_opaque_borrow");
        let compact: String = src.split_whitespace().collect();

        assert!(
            compact.contains("extern\"C\"fnz_config_insert_json5"),
            "{src}"
        );
        assert!(compact.contains("c:*mutz_config"), "{src}");
        // The handle pointer IS the box — decode directly, no `_0` indirection.
        assert!(!compact.contains("__h._0"), "{src}");
        assert!(
            compact.contains("Result::Ok(&mut*(vas*mutzenoh_flat::ZConfig))"),
            "{src}"
        );
    }

    /// Returning `Result<_, E>` where `E` is not declared via `.error()` is a
    /// build error.
    #[test]
    fn result_error_not_declared_is_build_error() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        // Error declared as data_struct but NOT marked `.error()`.
        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .name("z_keyexpr")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .function(syn::parse_quote!(z_keyexpr_try_from));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = registry.write_rust(&cbindgen, std::env::temp_dir().join("nope.rs"));
        }));
        assert!(
            result.is_err(),
            "expected a build error for undeclared error type"
        );
    }

    /// A non-`Result` fn with a fallible (`String`) input needs `.panic()`;
    /// without it that's a build error, with it the wrapper `panic!`s.
    #[test]
    fn fallible_input_without_result_needs_panic() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_log(s: String) {
                unimplemented!()
            }
        );

        // No `.panic()` → build error.
        let mut reg1 = Registry::<()>::from_items([(syn::Item::Fn(func.clone()), loc.clone())])
            .expect("index items");
        let cb1 = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .function(syn::parse_quote!(z_log));
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = reg1.write_rust(&cb1, std::env::temp_dir().join("nope2.rs"));
        }));
        assert!(err.is_err(), "expected a build error without .panic()");

        // With `.panic()` → wrapper aborts on decode failure.
        let mut reg2 =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
        let cb2 = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .function(syn::parse_quote!(z_log))
            .panic();
        let src = write(&cb2, &mut reg2, "panicfn");
        let compact: String = src.split_whitespace().collect();
        assert!(compact.contains("extern\"C\"fnz_log"), "{src}");
        assert!(compact.contains("panic!("), "{src}");
    }

    /// `.name()` on a function renames the exported `#[no_mangle]` symbol while
    /// still calling the original Rust fn.
    #[test]
    fn function_name_renames_symbol() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn rust_init() {
                unimplemented!()
            }
        );
        let mut reg =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
        let cb = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .function(syn::parse_quote!(rust_init))
            .name("z_init");
        let src = write(&cb, &mut reg, "fnname");
        let compact: String = src.split_whitespace().collect();
        assert!(compact.contains("extern\"C\"fnz_init("), "{src}");
        assert!(compact.contains("zenoh_flat::rust_init("), "{src}");
    }

    // ── Strict modifier rules (misapplied modifiers are build errors) ──────

    fn catch<F: FnOnce()>(f: F) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
    }

    #[test]
    fn error_after_ptr_struct_panics() {
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .ptr_struct(syn::parse_quote!(ZKeyExpr))
                .error();
        }));
    }

    #[test]
    fn panic_after_data_struct_panics() {
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .data_struct(syn::parse_quote!(Error))
                .panic();
        }));
    }

    #[test]
    fn destructor_name_after_data_struct_panics() {
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .data_struct(syn::parse_quote!(Error))
                .destructor_name("x");
        }));
    }

    #[test]
    fn name_with_no_declaration_panics() {
        // `source_module` is a root modifier — it resets the current declaration,
        // so a trailing `.name()` has nothing to apply to.
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .source_module(syn::parse_quote!(zenoh_flat))
                .name("x");
        }));
    }

    #[test]
    fn function_and_ignore_function_conflict_panics() {
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .function(syn::parse_quote!(z_open))
                .ignore_function(syn::parse_quote!(z_open));
        }));
    }

    #[test]
    fn data_struct_and_ignore_type_conflict_panics() {
        assert!(catch(|| {
            let _ = Cbindgen::new()
                .data_struct(syn::parse_quote!(Error))
                .ignore_type(syn::parse_quote!(Error));
        }));
    }

    /// A `Result<ptr, E>` wrapper returns the pointer and signals errors with
    /// NULL — both the `Err(E)` arm and an input-decode failure return null.
    #[test]
    fn result_pointer_returns_null_on_error() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .name("z_keyexpr")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_keyexpr_try_from));

        let src = write(&cbindgen, &mut registry, "ptr_null");
        let compact: String = src.split_whitespace().collect();

        assert!(compact.contains("->*mutz_keyexpr"), "{src}");
        // Err(E) arm: write *e then return null.
        assert!(compact.contains("null_mut()"), "{src}");
        // Decode failure also returns null (not `false`).
        assert!(compact.contains("return::core::ptr::null_mut()"), "{src}");
        assert!(!compact.contains("returnfalse"), "{src}");
    }

    /// Producing `char*` string memory without declaring a
    /// `free_memory_function` is a build error.
    #[test]
    fn free_memory_function_required() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_describe(key: String) -> Result<String, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        // String output (and an Error with a String field) but no free fn declared.
        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_describe));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = registry.write_rust(&cbindgen, std::env::temp_dir().join("nofree.rs"));
        }));
        assert!(
            result.is_err(),
            "expected a build error when string memory is produced without a free fn"
        );
    }

    /// A subscriber-shaped fn with an `impl Fn(ZSample)` callback and a zero-arg
    /// `impl Fn()` on-close: each declared callback emits a by-value `#[repr(C)]`
    /// closure struct (`context`/`call`/`drop`), `call` taking the arg's **owned**
    /// output wire (`z_sample_t *`) plus the `void *context`. The trampoline
    /// rebuilds a Rust closure that encodes args via their output converters and
    /// invokes the C `call` through an `Arc<Ctx>` that runs `drop(context)` on
    /// release.
    #[test]
    fn callback_subscriber_emits_closure_structs() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_sub(
                session: &ZSession,
                callback: impl Fn(ZSample) + Send + Sync + 'static,
                on_close: impl Fn() + Send + Sync + 'static,
            ) -> Result<ZSubscriber, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZSession))
            .name("z_session_t")
            .ptr_struct(syn::parse_quote!(ZSample))
            .name("z_sample_t")
            .ptr_struct(syn::parse_quote!(ZSubscriber))
            .name("z_subscriber_t")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
            .name("z_closure_sample_t")
            .callback(syn::parse_quote!(impl Fn() + Send + Sync + 'static))
            .name("z_closure_drop_t")
            .function(syn::parse_quote!(z_sub));

        let src = write(&cbindgen, &mut registry, "cb_sub");
        let compact: String = src.split_whitespace().collect();

        // Closure structs: sample carries the owned handle wire; drop is zero-arg.
        assert!(compact.contains("structz_closure_sample_t"), "{src}");
        assert!(
            compact.contains(
                "pubcall:::core::option::Option<unsafeextern\"C\"fn(*mutz_sample_t,*mut::core::ffi::c_void),>"
            ),
            "{src}"
        );
        assert!(compact.contains("structz_closure_drop_t"), "{src}");

        // Trampoline: by-value struct in, `impl Fn(<src arg>)` out; Arc-held ctx.
        assert!(
            compact.contains(
                "fn__cbg_in_z_closure_sample_t(c:z_closure_sample_t,)->implFn(zenoh_flat::ZSample)+Send+Sync+'static"
            ),
            "{src}"
        );
        assert!(
            compact.contains("Arc::new(__Ctx{context:c.context,drop:c.drop"),
            "{src}"
        );
        // Arg encoded via its OUTPUT converter, then passed (owned) with context.
        assert!(
            compact.contains("let__w0=__cbg_out_ZSample(__a0);"),
            "{src}"
        );
        assert!(compact.contains("__f(__w0,__ctx.context)"), "{src}");
        assert!(compact.contains("move|__a0:zenoh_flat::ZSample|"), "{src}");
        // Zero-arg trampoline.
        assert!(
            compact.contains(
                "fn__cbg_in_z_closure_drop_t(c:z_closure_drop_t,)->implFn()+Send+Sync+'static"
            ),
            "{src}"
        );
        assert!(compact.contains("move||{"), "{src}");
        assert!(compact.contains("__f(__ctx.context)"), "{src}");
        // Drop runs the C `drop(context)` on release.
        assert!(compact.contains("Some(__d)=self.drop"), "{src}");
        assert!(compact.contains("__d(self.context)"), "{src}");

        // Wrapper takes both closures by value and decodes them.
        assert!(compact.contains("callback:z_closure_sample_t"), "{src}");
        assert!(compact.contains("on_close:z_closure_drop_t"), "{src}");
        assert!(
            compact.contains("letcallback=__cbg_in_z_closure_sample_t(callback);"),
            "{src}"
        );
        assert!(
            compact.contains("leton_close=__cbg_in_z_closure_drop_t(on_close);"),
            "{src}"
        );
        // Result of an opaque handle rides the return (NULL = Err); `e` out-param.
        assert!(compact.contains("->*mutz_subscriber_t"), "{src}");
        assert!(compact.contains("e:*mutz_error"), "{src}");
    }

    /// Without a `.name(...)` override the closure-struct C name is composed
    /// generically from the args' configured C type names (`closure_<argCname>`)
    /// — `lang::Cbindgen` invents no target-language convention of its own.
    #[test]
    fn callback_struct_name_defaults_generically() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_sub2(
                session: &ZSession,
                callback: impl Fn(ZSample) + Send + Sync + 'static,
            ) -> Result<ZSubscriber, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZSession))
            .name("z_session_t")
            .ptr_struct(syn::parse_quote!(ZSample))
            .name("z_sample_t")
            .ptr_struct(syn::parse_quote!(ZSubscriber))
            .name("z_subscriber_t")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            // No `.name(...)` on the callback ⇒ generic default.
            .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
            .function(syn::parse_quote!(z_sub2));

        let src = write(&cbindgen, &mut registry, "cb_default");
        let compact: String = src.split_whitespace().collect();

        // Composed from the arg's configured C name `z_sample_t`.
        assert!(compact.contains("structclosure_z_sample_t"), "{src}");
        assert!(compact.contains("callback:closure_z_sample_t"), "{src}");
    }

    /// The five manglers generate every C-facing name from the Rust types — the
    /// base mangler centralizes per-type spelling (here `ZKeyExpr`→`keyexpr`),
    /// and the type/destructor/callback/function manglers decorate it. No
    /// per-declaration `.name(...)` is used.
    #[test]
    fn manglers_generate_all_names() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_sub(
                key_expr: ZKeyExpr,
                callback: impl Fn(ZSample) + Send + Sync + 'static,
                on_close: impl Fn() + Send + Sync + 'static,
            ) -> Result<ZSubscriber, Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            // One base rule fixes the `KeyExpr`→`keyexpr` irregular in a single
            // place; everything else is `snake_case` of the `Z`-stripped name.
            .mangle_rust_type(|short| {
                let s = short.strip_prefix('Z').unwrap_or(short);
                match s {
                    "KeyExpr" => "keyexpr".to_string(),
                    other => snake_case(other),
                }
            })
            .mangle_type_name(|base| format!("z_{base}_t"))
            .mangle_destructor(|base| format!("z_{base}_drop"))
            .mangle_callback(|bases| {
                if bases.is_empty() {
                    "z_closure_drop_t".to_string()
                } else {
                    format!("z_closure_{}_t", bases.join("_"))
                }
            })
            .mangle_function(|n| {
                if n.starts_with("z_") {
                    n.to_string()
                } else {
                    format!("z_{n}")
                }
            })
            // No `.name(...)` / `.destructor_name(...)` anywhere.
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .ptr_struct(syn::parse_quote!(ZSample))
            .ptr_struct(syn::parse_quote!(ZSubscriber))
            .data_struct(syn::parse_quote!(Error))
            .error()
            .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
            .callback(syn::parse_quote!(impl Fn() + Send + Sync + 'static))
            .function(syn::parse_quote!(z_sub));

        let src = write(&cbindgen, &mut registry, "manglers");
        let compact: String = src.split_whitespace().collect();

        // Type-name mangler over the base (note `keyexpr`, not `key_expr`).
        assert!(compact.contains("structz_keyexpr_t"), "{src}");
        assert!(compact.contains("structz_sample_t"), "{src}");
        assert!(compact.contains("structz_subscriber_t"), "{src}");
        assert!(compact.contains("structz_error_t"), "{src}");
        // Destructor mangler.
        assert!(
            compact.contains("fnz_keyexpr_drop(this_:*mutz_keyexpr_t"),
            "{src}"
        );
        assert!(
            compact.contains("fnz_sample_drop(this_:*mutz_sample_t"),
            "{src}"
        );
        // Callback mangler (arg base + zero-arg).
        assert!(compact.contains("structz_closure_sample_t"), "{src}");
        assert!(compact.contains("structz_closure_drop_t"), "{src}");
        // Callback `call` takes the owned handle wire produced via the manglers.
        assert!(
            compact.contains("fn(*mutz_sample_t,*mut::core::ffi::c_void)"),
            "{src}"
        );
        // Function mangler leaves the already-`z_`-prefixed symbol unchanged.
        assert!(compact.contains("extern\"C\"fnz_sub("), "{src}");
        // Return handle rides the return.
        assert!(compact.contains("->*mutz_subscriber_t"), "{src}");
    }

    /// A borrowed (non-`'static`) `&T` return of an opaque handle lowers to a
    /// const, **non-owning** `*const z_X_t` (no `Box::into_raw`) — a loaned
    /// accessor. The converter reinterprets the borrow.
    #[test]
    fn borrowed_ref_output_is_const_non_owning() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_sample_payload(s: &ZSample) -> &ZBytes {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .ptr_struct(syn::parse_quote!(ZSample))
            .name("z_sample_t")
            .ptr_struct(syn::parse_quote!(ZBytes))
            .name("z_zbytes_t")
            .function(syn::parse_quote!(z_sample_payload))
            .panic();

        let src = write(&cbindgen, &mut registry, "borrow_ret");
        let compact: String = src.split_whitespace().collect();

        // Const, non-owning return; the return path goes through the reinterpret
        // (`&` → `*const`) converter, not an owning `Box::into_raw`.
        assert!(compact.contains("->*constz_zbytes_t"), "{src}");
        assert!(
            compact.contains("vas*constzenoh_flat::ZBytesas*constz_zbytes_t"),
            "{src}"
        );
        assert!(
            compact.contains("__ret=__cbg_out_ref_ZBytes(__v);"),
            "{src}"
        );
    }

    /// `Option<&T>` borrowed return composes: a nullable const loaned pointer
    /// (NULL = `None`), via the Option null-niche path over the borrow wire.
    #[test]
    fn borrowed_option_ref_output_nullable() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> {
                unimplemented!()
            }
        );
        let mut registry =
            Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .ptr_struct(syn::parse_quote!(ZSample))
            .name("z_sample_t")
            .ptr_struct(syn::parse_quote!(ZTimestamp))
            .name("z_timestamp_t")
            .function(syn::parse_quote!(z_sample_timestamp))
            .panic();

        let src = write(&cbindgen, &mut registry, "borrow_opt_ret");
        let compact: String = src.split_whitespace().collect();

        // Nullable const loaned pointer rides the return (no out-param needed:
        // the pointer's NULL niche encodes `None`).
        assert!(compact.contains("->*constz_timestamp_t"), "{src}");
        assert!(compact.contains("__cbg_out_ref_ZTimestamp"), "{src}");
        assert!(!compact.contains("out:*mut*constz_timestamp_t"), "{src}");
    }

    /// Contract: the error out-parameter `e` may be NULL. EVERY `*e =` write in a
    /// generated wrapper is guarded by `if !e.is_null()` — both on the input-decode
    /// failure path and on the `Result::Err` return path, and for both error-routing
    /// modes (pointer-return in-band niche, and `bool`-status). Consumers (e.g. the
    /// zenoh-c compat layer) rely on passing NULL and reading the return value.
    #[test]
    fn error_out_param_is_null_guarded() {
        let loc = SourceLocation::default();
        // (a) pointer-returning Result<Handle, E> + a fallible `String` input
        //     (exercises both the input-decode and the Result::Err error paths).
        let ptr_fn: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
                unimplemented!()
            }
        );
        // (b) Result<(), E> → `bool` status return.
        let unit_fn: syn::ItemFn = syn::parse_quote!(
            pub fn z_unit_op(s: String) -> Result<(), Error> {
                unimplemented!()
            }
        );
        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(ptr_fn), loc.clone()),
            (syn::Item::Fn(unit_fn), loc.clone()),
            (syn::Item::Struct(error_struct()), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .free_memory_function("z_free")
            .ptr_struct(syn::parse_quote!(ZKeyExpr))
            .name("z_keyexpr")
            .data_struct(syn::parse_quote!(Error))
            .name("z_error")
            .error()
            .function(syn::parse_quote!(z_keyexpr_try_from))
            .function(syn::parse_quote!(z_unit_op));

        let src = write(&cbindgen, &mut registry, "err_null_guard");
        let compact: String = src.split_whitespace().collect();

        // Pointer-return Err arm: guarded write, then NULL.
        assert!(
            compact
                .contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}::core::ptr::null_mut()}"),
            "{src}"
        );
        // `Result<(),E>` Err arm: guarded write, then `false`.
        assert!(
            compact.contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}false}"),
            "{src}"
        );
        // The input-decode failure path also guards the write (it routes the
        // message through `From<String>`). Both functions have a fallible `String`
        // input plus a `Result::Err` arm, so the guarded write appears ≥4 times.
        assert!(
            compact.matches("if!e.is_null(){*e=__cbg_out_Error(").count() >= 4,
            "expected ≥4 guarded `*e =` writes (2 input-decode + 2 Err arms):\n{src}"
        );

        // Strongest guarantee: NO unguarded `*e =`. Every occurrence of `*e=` in
        // the compacted source is immediately preceded by `if!e.is_null(){`.
        let mut search = compact.as_str();
        while let Some(pos) = search.find("*e=") {
            let before = &search[..pos];
            assert!(
                before.ends_with("if!e.is_null(){"),
                "unguarded `*e =` found before offset {pos}:\n{src}"
            );
            search = &search[pos + 3..];
        }
    }
}
