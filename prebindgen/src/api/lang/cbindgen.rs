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
use crate::api::core::registry::{Registry, TypeKey};

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
}

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
            !self.opaque.contains_key(&key) && !self.data.contains_key(&key) && !self.enums.contains_key(&key),
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
            Some(CurrentDecl::Function(ident)) => {
                self.functions
                    .get_mut(&ident)
                    .expect("entry vanished")
                    .c_name = Some(name);
            }
            None => panic!(
                "Cbindgen::name must be chained directly after a declaration \
                 (`ptr_struct` / `data_struct` / `enum_type` / `function`)"
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

    /// Emitted C type name of a declared type: pinned `c_name`, else the Rust
    /// short name.
    fn c_type_name(&self, ty: &syn::Type) -> String {
        self.type_cfg(ty)
            .and_then(|c| c.c_name.clone())
            .unwrap_or_else(|| type_short(ty))
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
}

/// Human-readable description of the current declaration, for panic messages.
fn describe_current(current: &Option<CurrentDecl>) -> String {
    match current {
        None => "no declaration".to_string(),
        Some(CurrentDecl::Ptr(k)) => format!("ptr_struct `{}`", k.as_str()),
        Some(CurrentDecl::Data(k)) => format!("data_struct `{}`", k.as_str()),
        Some(CurrentDecl::Enum(k)) => format!("enum_type `{}`", k.as_str()),
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
        if self.needs_free(registry) {
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
                pub(crate) fn __cbg_alloc_cstr(s: ::std::string::String) -> *mut ::core::ffi::c_char {
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
            let drop_ident = match self.opaque.get(key).and_then(|c| c.drop_name.clone()) {
                Some(full) => format_ident!("{}", full),
                None => format_ident!("{}_drop", self.c_drop_base(&ty)),
            };
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
        // `&str`: borrow a UTF-8 C string directly from the caller.
        let syn::Type::Reference(r) = pat else {
            return None;
        };
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
        // `&'static T` → `*const <C struct>`: a const (non-owning) pointer.
        // Signals to C callers that the value must NOT be freed.
        let syn::Type::Reference(r) = pat else { return None };
        let is_static = matches!(&r.lifetime, Some(lt) if lt.ident == "static");
        if !is_static || r.mutability.is_some() {
            return None;
        }
        let key = TypeKey::from_type(t1);
        self.opaque.get(&key)?;

        let c_struct = self.c_type_ident(t1);
        let src = self.src_ty(t1);
        let name = Self::out_name(pat);
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) unsafe fn #name(v: &'static #src) -> *const #c_struct {
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
        let name = format_ident!("__cbg_result_{}", sanitize(&TypeKey::from_type(pat)));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        // Destination is the success wire (only used if some outer wrapper reads
        // it; `on_function` does not).
        let t_short = self.c_type_ident(t1);
        Some(ConverterImpl {
            destination: syn::parse_quote!(#t_short),
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
    /// else the Rust ident.
    fn fn_symbol(&self, orig: &syn::Ident) -> syn::Ident {
        self.functions
            .get(orig)
            .and_then(|c| c.c_name.clone())
            .map(|n| format_ident!("{}", n))
            .unwrap_or_else(|| orig.clone())
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

        if let Some((ok_ty, err_ty)) = result_parts(&return_ty) {
            // Rule 1: the Result error type must be declared via `.error()`.
            assert!(
                self.error.contains(&TypeKey::from_type(&err_ty)),
                "Cbindgen: function `{}` returns `Result<_, {}>` but `{}` is not a \
                 declared error type — add `.data_struct({}).error()`",
                orig,
                TypeKey::from_type(&err_ty),
                TypeKey::from_type(&err_ty),
                TypeKey::from_type(&err_ty),
            );
            self.emit_fallible_wrapper(orig, &sym, &call_path, f, &ok_ty, &err_ty, registry)
        } else {
            // Rule 3: a fallible input with no Result channel needs `.panic()`.
            let allows_panic = self.functions.get(orig).map(|c| c.panic).unwrap_or(false);
            assert!(
                !has_fallible_input || allows_panic,
                "Cbindgen: function `{}` has a fallible input (e.g. a `String` or \
                 opaque-by-value argument) but does not return `Result`; add \
                 `.panic()` after its `.function(...)` declaration to allow aborting \
                 on the internal error, or change its signature",
                orig,
            );
            self.emit_infallible_wrapper(orig, &sym, &call_path, f, &return_ty, registry)
        }
    }

    /// `Result<T, E>` → one of three C shapes by the success wire kind:
    /// * pointer wire (opaque handle, `char*`) → `T *f(<inputs>, E *e)`, NULL = err;
    /// * unit → `bool f(<inputs>, E *e)`;
    /// * value wire (data struct, scalar, enum) → `bool f(T *out, <inputs>, E *e)`.
    fn emit_fallible_wrapper(
        &self,
        orig: &syn::Ident,
        sym: &syn::Ident,
        call_path: &syn::Path,
        f: &syn::ItemFn,
        ok_ty: &syn::Type,
        err_ty: &syn::Type,
        registry: &Registry<()>,
    ) -> TokenStream {
        let err_entry = registry.output_entry(err_ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: error type `{}` of `{}` has no output converter",
                TypeKey::from_type(err_ty),
                orig
            )
        });
        let err_wire = err_entry.destination.clone();
        let err_conv = err_entry.function.sig.ident.clone();

        // Choose the success shape. Each arm sets: the wrapper return type, the
        // (optional) leading out-param, the `Ok` match arm, the `Err` tail, and
        // the value an input-decode failure returns.
        let (ret_ty, out_param, ok_arm, err_tail, fail_return): (
            TokenStream,
            TokenStream,
            TokenStream,
            TokenStream,
            TokenStream,
        ) = if is_unit(ok_ty) {
            (
                quote!(bool),
                quote!(),
                quote!(::core::result::Result::Ok(_) => true,),
                quote!(false),
                quote!(false),
            )
        } else {
            let ok_entry = registry.output_entry(ok_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: success type `{}` of `{}` has no output converter",
                    TypeKey::from_type(ok_ty),
                    orig
                )
            });
            let ok_wire = ok_entry.destination.clone();
            let ok_conv = ok_entry.function.sig.ident.clone();
            if let syn::Type::Ptr(ok_ptr) = &ok_wire {
                // Pointer-wire success: return the pointer, NULL on error.
                // Use null() for *const (borrowed/static) and null_mut() for *mut (owned).
                let null = if ok_ptr.mutability.is_some() {
                    quote!(::core::ptr::null_mut())
                } else {
                    quote!(::core::ptr::null())
                };
                (
                    quote!(#ok_wire),
                    quote!(),
                    quote!(::core::result::Result::Ok(__v) => #ok_conv(__v),),
                    null.clone(),
                    null,
                )
            } else {
                // Value-wire success: caller-allocated `*out`, `bool` return.
                (
                    quote!(bool),
                    quote!(out: *mut #ok_wire,),
                    quote!(::core::result::Result::Ok(__v) => {
                        *out = #ok_conv(__v);
                        true
                    }),
                    quote!(false),
                    quote!(false),
                )
            }
        };

        let route = ErrRoute::Result {
            e_conv: &err_conv,
            e_ty_src: self.src_ty(err_ty),
            fail_return,
        };
        let (params, decodes, call_args) = self.emit_inputs(orig, f, registry, &route);

        quote! {
            #[no_mangle]
            #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
            pub unsafe extern "C" fn #sym(
                #out_param
                #(#params,)*
                e: *mut #err_wire,
            ) -> #ret_ty {
                #(#decodes)*
                match #call_path(#(#call_args),*) {
                    #ok_arm
                    ::core::result::Result::Err(__err) => {
                        if !e.is_null() {
                            *e = #err_conv(__err);
                        }
                        #err_tail
                    }
                }
            }
        }
    }

    /// Non-`Result` return: natural shape (`void` for unit, wire by value
    /// otherwise). A fallible input here only reaches this point when the fn is
    /// declared `.panic()`, so decode failures `panic!`.
    fn emit_infallible_wrapper(
        &self,
        orig: &syn::Ident,
        sym: &syn::Ident,
        call_path: &syn::Path,
        f: &syn::ItemFn,
        return_ty: &syn::Type,
        registry: &Registry<()>,
    ) -> TokenStream {
        let (params, decodes, call_args) = self.emit_inputs(orig, f, registry, &ErrRoute::Panic);
        let call = quote!(#call_path(#(#call_args),*));

        if is_unit(return_ty) {
            quote! {
                #[no_mangle]
                #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
                pub unsafe extern "C" fn #sym(#(#params),*) {
                    #(#decodes)*
                    #call;
                }
            }
        } else {
            let out_entry = registry.output_entry(return_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: return type `{}` of `{}` has no output converter",
                    TypeKey::from_type(return_ty),
                    orig
                )
            });
            let wire = out_entry.destination.clone();
            let conv = out_entry.function.sig.ident.clone();
            quote! {
                #[no_mangle]
                #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
                pub unsafe extern "C" fn #sym(#(#params),*) -> #wire {
                    #(#decodes)*
                    #conv(#call)
                }
            }
        }
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
                    ErrRoute::Result { e_conv, e_ty_src, fail_return } => quote!(
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
fn snake_case(s: &str) -> String {
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
        assert!(compact.contains("fnz_keyexpr_free(this_:*mutz_keyexpr"), "{src}");
        assert!(
            compact.contains("Box::from_raw(this_as*mutzenoh_flat::ZKeyExpr)"),
            "{src}"
        );
        // String memory ⇒ malloc/free decls + a single `z_free`; no per-type
        // string/error destructors.
        assert!(compact.contains("fnmalloc(size:usize)"), "{src}");
        assert!(compact.contains("fnz_free(p:*mut::core::ffi::c_void)"), "{src}");
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
        // Ok arm is a bare `true`, with no write through `out`.
        assert!(compact.contains("Result::Ok(_)=>true"), "{src}");
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
        assert!(compact.contains("fnz_free(p:*mut::core::ffi::c_void)"), "{src}");
        // Ok arm returns the pointer directly; error → NULL.
        assert!(compact.contains("=>__cbg_out_String(__v),"), "{src}");
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
}
