//! `CbindgenBuilder` — the C / cbindgen language adapter.
//!
//! # Experimental API
//!
//! This module is a proof of concept. Enable it with the non-default
//! `prebindgen/unstable-cbindgen` feature. Its Rust builder API may change in a
//! minor release; do not rely on it as part of the stable 0.5 API.
//!
//! A [`Prebindgen`] back-end that turns a "flat" `#[prebindgen]` library into a
//! Rust file suitable for [`cbindgen`](https://github.com/mozilla/cbindgen) to
//! parse into a C header plus a static / dynamic library.
//!
//! Items are **opt-in**: nothing is converted unless it is explicitly declared
//! with [`CbindgenBuilder::function`] / [`CbindgenBuilder::opaque_ptr`] /
//! [`CbindgenBuilder::data_struct`] / [`CbindgenBuilder::enum_type`] /
//! [`CbindgenBuilder::tagged_union`]. The C name of a declared
//! type's generated destructor can be pinned by chaining [`CbindgenBuilder::name`].
//!
//! ## C ABI conventions
//!
//! * **Pointer struct** (declared with [`CbindgenBuilder::opaque_ptr`]): a `Box`-owned
//!   Rust value whose lifecycle is owned by the C side. The C type `T` is
//!   **opaque/incomplete** and the handle is a bare `T *` = `Box::into_raw`. A
//!   typed `<name>_drop(T *)` destructor (running the Rust `Drop`) is generated
//!   per handle.
//! * **Data struct** (declared with [`CbindgenBuilder::data_struct`]): a by-value
//!   `#[repr(C)]` struct whose fields are mapped to C-ABI wire types
//!   (`String` → `*mut c_char`). No per-struct destructor — each `char*` field
//!   is released individually via the [`CbindgenBuilder::free_memory_function`].
//! * **Enum type** (declared with [`CbindgenBuilder::enum_type`]): a fieldless enum,
//!   mirrored as a `#[repr(C)]` enum that cbindgen renders as the C enum.
//!   Rust → C hands over the mirror directly (Rust only ever builds declared
//!   variants). C → Rust must **not** do the reverse: a C `enum` is an `int` at
//!   the ABI, so materialising a caller-supplied discriminant as a Rust enum is
//!   undefined behaviour when it matches no variant — before any `match` could
//!   check it. An enum parameter is therefore taken as
//!   `MaybeUninit<mirror>` — the same ABI and the same C spelling (cbindgen
//!   renders `MaybeUninit<T>` as `T`), but legal to hold any bit pattern — and
//!   its raw `c_int` is validated against the mirror's variants before the Rust
//!   value is built. An unmatched value is a fallible-input error (see below),
//!   so a function taking an enum by value needs either a `Result` return or
//!   [`CbindgenBuilder::panic`]. This relies on cbindgen's C rendering; the `C++`
//!   language mode is not supported.
//! * **Tagged union** (declared with [`CbindgenBuilder::tagged_union`]): a
//!   data-carrying enum crossing by value as a `#[repr(C)]` enum with payload
//!   variants, which cbindgen renders as a tag enum plus a `union` of the
//!   variant bodies. When any variant's payload wire owns memory, a typed
//!   `<name>_drop` frees the **active arm**. Inbound it obeys the same rule as
//!   a plain enum, one level up: the mirror arrives as `MaybeUninit<mirror>`,
//!   its leading `c_int` tag is range-checked against the variants, and only
//!   then is the value `assume_init`ed and matched. Every payload wire is
//!   bit-pattern-agnostic (a declared `enum_type` payload rides as
//!   `MaybeUninit` too, and is validated by its own converter), so the tag is
//!   the sole obligation. The typed drop checks it as well and treats an
//!   out-of-range one as nothing to release.
//! * **Direct `String` output**: a bare `char *` — a `malloc`'d, null-terminated
//!   raw block (no wrapper struct), freed via the `free_memory_function`.
//! * **[`CbindgenBuilder::free_memory_function`]**: the single, type-agnostic raw memory
//!   freer (C `free`) for every `char*` the layer hands out (string returns and
//!   data-struct `String` fields). It runs no destructor and needs no length.
//!   Required whenever such string memory is produced.
//! * **`Result<T, E>` return** lowers by the success wire kind:
//!   - **pointer wire** (opaque handle, `char*`) → `T f(<inputs>, E *e)`, where a
//!     **NULL return signals error** (details written to `*e`);
//!   - **unit** → `bool f(<inputs>, E *e)`;
//!   - **value wire** (data struct, scalar, enum) → `bool f(T *out, <inputs>, E *e)`
//!     filling a caller-allocated `*out`.
//!
//!   `e` may be `NULL`, in which case the error value is dropped. Infallible
//!   producers return the value/pointer directly (no out-param).
//!
//! ## Error handling (multiple error types)
//!
//! Any type used as the `E` of a `Result<T, E>` return **must be declared** as an
//! error type via [`CbindgenBuilder::data_struct`] + [`CbindgenBuilder::error`] — otherwise the
//! build fails. Error types are ordinary data structs (marshalled by value) and
//! must additionally implement `From<String>`.
//!
//! Built-in input converters that can fail (a `String` arg, an opaque handle
//! passed by value, a declared enum whose discriminant the caller chose) are
//! **error-type-agnostic**: they return `Result<_, String>`
//! where the `Err` is just a message. The generated wrapper for a `Result<T, E>`
//! function converts such a message into *that function's* `E` via
//! `<E as From<String>>::from(msg)`; the function's own `Err(E)` is marshalled
//! directly through `E`'s output converter.
//!
//! If a function can produce such an internal message but does **not** return
//! `Result`, that is a build error — suppress it by chaining [`CbindgenBuilder::panic`]
//! after the function declaration, which makes the wrapper `panic!` on the
//! internal error instead.
//!
//! References to the original Rust types in generated bodies are written
//! fully-qualified against [`CbindgenBuilder::source_module`] so the generated file can
//! define its own identically-named `#[repr(C)]` wrapper structs without
//! colliding with the source crate's types.

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

// Shared `syn::Type` shape predicates live in `core::types_util`; re-exported
// here under this back-end's historical names so the submodules (`use super::*`)
// keep their call sites. `pub(crate) use` so the glob re-export reaches them.
pub(crate) use crate::api::core::types_util::{
    first_type_arg, is_option_type as is_option, is_result_type as is_result, is_unit,
    is_vec_type as is_vec, path_tail_ident as type_path_tail, result_parts,
};
use crate::api::{
    core::{
        niches::{NicheSlot, Niches},
        prebindgen::{ConverterImpl, Prebindgen},
        registry::{extract_fn_trait_args, Conversions, Direction, Registry, TypeKey},
    },
    lang::jnigen::{ConvertDecl, ConvertSpec},
};

/// Identity of a declared callback signature: its argument-type list (the
/// dedup key, since two `impl Fn` params with the same args share one closure
/// struct). The return is always unit for the supported callbacks.
type CallbackKey = Vec<TypeKey>;

/// Per-opaque-handle / per-data-struct / per-enum configuration.
#[derive(Clone, Default)]
struct TypeCfg {
    /// Per-declaration **base** token override, fed to the name manglers
    /// (`mangle_type_name` / `mangle_destructor` / `mangle_take`) in place of the
    /// `mangle_rust_type`-derived base. Set by [`CbindgenBuilder::base_name`]. `None` ⇒
    /// the base comes from `mangle_rust_type(short)` (or the short name).
    base: Option<String>,
}

/// What an inline-opaque by-value type holds, which decides whether its consume
/// path needs a gravestone write-back (and thus a [`crate::core::Gravestone`]
/// impl). See [`CbindgenBuilder::opaque_data_struct`] / [`CbindgenBuilder::opaque_owned_struct`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum OpaqueKind {
    /// **Plain data** — holds no external resource (typically `Copy`, e.g. a
    /// timestamp). Drop is a no-op, so consuming (moving out) leaves the source's
    /// bitwise duplicate harmlessly droppable: **no gravestone write-back, no
    /// `Gravestone` impl required** (only the autogenerated `Transmute`).
    Data,
    /// **Owns external data** — refcounts / heap (e.g. a byte buffer, a sample).
    /// Consuming must write a [`crate::core::Gravestone`] back over the moved-from
    /// source so a later drop is a no-op (double-free safe). Requires the consumer
    /// to implement `Gravestone` for the opaque counterpart (its *logic* only).
    Owned,
}

/// Per-inline-opaque configuration: the opaque `#[repr(C, align(_))]` counterpart
/// type the Rust value is transmuted to/from, whether it owns external data, plus
/// the usual name config.
#[derive(Clone)]
struct ValueOpaqueCfg {
    /// The opaque counterpart type (defined elsewhere — e.g. by a size/align
    /// probe generator). Used verbatim as the by-value wire type. Must have
    /// identical size+align to the Rust type (a `const _` assert is emitted to
    /// enforce that, fail-closed) and — for [`OpaqueKind::Owned`] — implement
    /// [`crate::core::Gravestone`].
    opaque: syn::Type,
    /// Plain-data vs owns-external-data (gravestone write-back on consume).
    kind: OpaqueKind,
    /// When `true`, the `opaque` counterpart is **not** supplied externally but is
    /// an auto-generated **visible-field** `#[repr(C)]` mirror of the source struct,
    /// emitted by [`CbindgenBuilder::prereq_value_opaque`]. Set by
    /// [`CbindgenBuilder::repr_c_struct`]; `false` for `opaque_data_struct` /
    /// `opaque_owned_struct` (counterpart defined elsewhere).
    generate_mirror: bool,
    /// Opt-out of the restricted-validity field audit (#170 instance 3, #158
    /// instance 3). Set by [`CbindgenBuilder::assume_c_field_validity`]. See
    /// [`CbindgenBuilder::restricted_validity_field`] for what the audit rejects and
    /// why the escape hatch exists.
    assume_c_field_validity: bool,
    /// Name config (`.base_name()` override; default naming via the manglers).
    cfg: TypeCfg,
}

/// Per-declared-callback configuration.
#[derive(Clone, Default)]
struct CbCfg {
    /// Per-declaration **base** token override fed to `mangle_callback` (as the
    /// sole base, replacing the args' derived bases). Set by
    /// [`CbindgenBuilder::base_name`]. `None` ⇒ bases come from the arguments.
    base: Option<String>,
    /// Argument indices delivered to the C `call` as a **takeable owned pointer**
    /// (`*mut z_x_t`) instead of by value: the callee may take the value (move it
    /// out via `z_x_take`, leaving a gravestone) or just read it, and the
    /// trampoline drops it after the call (no-op if taken). Set by
    /// [`CbindgenBuilder::takeable_param`]; each such arg type must be an inline-opaque
    /// type ([`CbindgenBuilder::opaque_owned_struct`] / [`CbindgenBuilder::opaque_data_struct`]).
    takeable: std::collections::BTreeSet<usize>,
}

/// Per-declared-function configuration.
#[derive(Clone, Default)]
struct FnCfg {
    /// Per-declaration **base** token override fed to `mangle_function` in place of
    /// the Rust fn ident. Set by [`CbindgenBuilder::base_name`]. `None` ⇒ the fn ident.
    base: Option<String>,
    /// Allow the generated wrapper to `panic!` on an internal error message
    /// (set by [`CbindgenBuilder::panic`]). Only meaningful for non-`Result` functions
    /// that have a fallible input.
    panic: bool,
}

/// The declaration a chained modifier ([`CbindgenBuilder::name`] / [`CbindgenBuilder::error`]
/// / [`CbindgenBuilder::panic`]) applies to. Set by each declaration method, reset to
/// `None` by root-level modifiers (e.g. [`CbindgenBuilder::source_module`]).
#[derive(Clone)]
enum CurrentDecl {
    Ptr(TypeKey),
    Data(TypeKey),
    ValueOpaque(TypeKey),
    Enum(TypeKey),
    TaggedUnion(TypeKey),
    Callback(CallbackKey),
    Function(syn::Ident),
    Convert(TypeKey),
}

/// Where a fallible input-decode failure is routed in a generated wrapper.
#[allow(clippy::large_enum_variant)]
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

/// Emit the statements that report `__msg` — a `String` already in scope — per
/// `route`, and leave the wrapper.
///
/// Shared by the per-input decode failure and by the alias preflight, so the
/// two cannot drift on how a binding error reaches the caller.
fn route_message(route: &ErrRoute<'_>) -> TokenStream {
    match route {
        ErrRoute::Result {
            e_conv,
            e_ty_src,
            fail_return,
        } => quote!(
            if !e.is_null() {
                *e = #e_conv(
                    <#e_ty_src as ::core::convert::From<::std::string::String>>::from(__msg),
                );
            }
            return #fail_return;
        ),
        ErrRoute::Panic => quote!(panic!("{}", __msg);),
    }
}

/// How a parameter uses the resource it names — the axis
/// [`CbindgenBuilder::alias_preflight`] states its rule on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AliasAccess {
    /// Taken by value: the callee owns it afterwards, and the C-side handle is
    /// dead.
    Consume,
    /// `&mut T`: exclusive for the duration of the call.
    Exclusive,
    /// `&T`: shared, and the only access that may legally coexist with another
    /// of its own kind.
    Shared,
}

impl AliasAccess {
    /// The word used for this access in a preflight rejection message.
    fn describe(self) -> &'static str {
        match self {
            AliasAccess::Consume => "consumed",
            AliasAccess::Exclusive => "exclusively borrowed",
            AliasAccess::Shared => "borrowed",
        }
    }
}

/// C / cbindgen language adapter. Build it with [`CbindgenBuilder::new`], declare the
/// items to convert with the fluent methods, then drive it through
/// [`CbindgenBuilder::build`] → [`Cbindgen::write_rust`].
///
/// A resolved C binding: every crossing has a conversion, and the header-facing
/// Rust file can be written.
///
/// Built by [`CbindgenBuilder::build`]. Read-only, so `write_rust` is a pure
/// emission over a complete registry.
pub struct Cbindgen {
    pub(crate) gen: CbindgenBuilder,
    pub(crate) registry: crate::core::Registry<()>,
}

// Opaque — exists so `Result<Cbindgen, _>::expect_err` works in tests.
impl std::fmt::Debug for Cbindgen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cbindgen(..)")
    }
}

impl Cbindgen {
    /// Describe a C binding.
    pub fn builder() -> CbindgenBuilder {
        CbindgenBuilder::new()
    }

    /// Write the generated Rust file — the `extern "C"` wrappers and their
    /// converters, which `cbindgen` then reads to emit the header.
    pub fn write_rust(
        &self,
        out_path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, crate::core::WriteRustError> {
        Ok(crate::api::core::write::write_rust(
            &self.registry,
            &self.gen,
            out_path,
        )?)
    }

    /// The resolved registry — conversions, decompositions, and the model.
    pub fn registry(&self) -> &crate::core::Registry<()> {
        &self.registry
    }

    /// What the binding declared.
    pub fn declarations(&self) -> &CbindgenBuilder {
        &self.gen
    }
}

#[derive(Default)]
pub struct CbindgenBuilder {
    /// Module path the original `#[prebindgen]` items live under. Used to
    /// fully-qualify bare references to source types in generated bodies.
    source_module: Option<syn::Path>,
    /// `#[prebindgen]` functions explicitly declared for conversion.
    functions: HashMap<syn::Ident, FnCfg>,
    /// Canonical scalar conversions shared with JniGenBuilder.
    convert_decls: Vec<ConvertDecl>,
    /// Per-conversion C naming base used for generated niche constants.
    convert_bases: HashMap<TypeKey, String>,
    /// `#[prebindgen]` functions intentionally not exported by this adapter.
    ignored_functions: HashSet<syn::Ident>,
    /// Opaque-handle types (`Box` + `void*` lifecycle, auto `_drop`).
    opaque: HashMap<TypeKey, TypeCfg>,
    /// By-value `#[repr(C)]` data structs.
    data: HashMap<TypeKey, TypeCfg>,
    /// Inline-opaque by-value types: the Rust value is transmuted to/from an
    /// opaque `#[repr(C, align(_))]` counterpart of identical size+align (no
    /// `Box`). Keyed by the Rust type; the value carries the opaque counterpart.
    value_opaque: HashMap<TypeKey, ValueOpaqueCfg>,
    /// Enum types (unit-variant only — a C `enum` is a bare discriminant).
    enums: HashMap<TypeKey, TypeCfg>,
    /// Data-carrying enum types crossing by value as a `#[repr(C)]` enum with
    /// payload variants, which cbindgen renders as the idiomatic C tag +
    /// `union`. Declared with [`CbindgenBuilder::tagged_union`].
    tagged_unions: HashMap<TypeKey, TypeCfg>,
    /// Declared callback signatures (`impl Fn(...) + Send + Sync + 'static`),
    /// keyed by their argument-type list. Each emits one `#[repr(C)]` closure
    /// struct.
    callbacks: HashMap<CallbackKey, CbCfg>,
    /// Types intentionally not exported by this adapter.
    ignored_types: HashSet<TypeKey>,
    /// Data structs additionally marked as error types (allowlist for the
    /// "Result error type must be declared" rule).
    error: HashSet<TypeKey>,
    /// Opaque error types (e.g. `ZError = Box<dyn Error>`) that are NOT by-value
    /// data structs: they appear as the `E` of a `Result<_, E>` but are
    /// marshalled to C as a `char*` message obtained by calling the recorded
    /// accessor `fn(&E) -> String`. Keyed by the error type; the value is the
    /// message-accessor function ident. Also inserted into [`Self::error`].
    opaque_errors: HashMap<TypeKey, syn::Ident>,
    /// Name of the universal raw-memory freer (C `free`) for `char*` data the
    /// generated code hands out. Set by [`Self::free_memory_function`]. Required
    /// (build error otherwise) whenever string memory is produced.
    free_fn: Option<String>,
    /// The declaration that chained modifiers apply to. Set by declaration
    /// methods; reset to `None` by root-level modifiers.
    current: Option<CurrentDecl>,
    /// Optional name-mangling rules (all `None` ⇒ the built-in defaults below,
    /// which carry no target-language convention). A per-declaration
    /// [`.base_name()`](Self::base_name) replaces the *base* token fed to these
    /// manglers. See [[the builder methods]](Self::mangle_rust_type).
    ///
    /// Base: Rust short name → canonical token, feeding the three type manglers.
    mangle_rust_type: Option<Mangle1>,
    /// Base → C type name (struct / enum / data).
    mangle_type_name: Option<Mangle1>,
    /// Base → opaque-handle destructor symbol.
    mangle_destructor: Option<Mangle1>,
    /// Base → value_opaque "take" (move) symbol, for takeable callback params.
    mangle_take: Option<Mangle1>,
    /// Callback arg bases → closure-struct name.
    mangle_callback: Option<MangleN>,
    /// Rust function ident → exported `#[no_mangle]` symbol.
    mangle_function: Option<Mangle1>,
    /// Where the `#[prebindgen]` items come from — see
    /// `JniGenBuilder::source`.
    pub(crate) sources: crate::api::core::flat::FlatBuilder,
}

/// A mangler over a single name component (Rust short name, base, or fn ident).
type Mangle1 = Box<dyn Fn(&str) -> String>;
/// A mangler over a callback's argument bases.
type MangleN = Box<dyn Fn(&[String]) -> String>;

mod builder;
mod convert;
mod emit;
mod selector;
#[cfg(test)]
mod tests;
mod trait_impl;

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
fn enum_item<'r>(registry: &'r impl Conversions<()>, ty: &syn::Type) -> Option<&'r syn::ItemEnum> {
    let ident = type_path_tail(ty)?;
    registry.flat().enum_item(&ident)
}

/// Hard error when a `.tagged_union()`-declared enum is unit-only. The
/// counterpart of [`assert_unit_enum`]: a fieldless enum is exactly a
/// discriminant, so it belongs to `.enum_type()` — declaring it here would
/// emit a `union` with no bodies and a needlessly indirect C surface. Neither
/// declarator silently accepts the other's shape.
fn assert_payload_enum(e: &syn::ItemEnum) {
    use crate::api::core::types_util::{enum_shape, EnumShape};
    if enum_shape(e) == EnumShape::Unit {
        panic!(
            "Cbindgen: `{}` has no payload variants: declare it with `.enum_type()`, \
             not `.tagged_union()` — a fieldless enum crosses as a plain C `enum`",
            e.ident
        );
    }
}

/// If `fty` is an opaque-pointer payload — `Box<T>` or `Option<Box<T>>` with
/// `T` a path type — return `T`. The shape check only; whether `T` is a
/// declared `opaque_ptr` is [`CbindgenBuilder::mirror_field_wire`]'s call, and this
/// is only reached for a field that already passed it.
fn opaque_ptr_payload_inner(fty: &syn::Type) -> Option<syn::Type> {
    if is_option(fty) {
        first_type_arg(fty).and_then(|inner| box_inner(&inner))
    } else {
        box_inner(fty)
    }
}

/// The `Type` form of a generated C type ident, for the shared
/// [`variant_ctor`] helper (which takes the enum's path either as a source
/// type or as a mirror ident).
fn cname_ty(cname: &syn::Ident) -> syn::Type {
    syn::parse_quote!(#cname)
}

/// `Enum::Variant { a: __f0, .. }` / `Enum::Variant(__f0, ..)` / `Enum::Variant`
/// — the match pattern binding every field of one variant, shaped like the
/// variant itself. Used for both the source enum and its C mirror, and for
/// both directions, so the two sides always destructure the same way.
fn variant_pattern(
    enum_path: &impl ToTokens,
    variant: &syn::Ident,
    fields: &syn::Fields,
    binds: &[syn::Ident],
) -> TokenStream {
    match fields {
        syn::Fields::Unit => quote!(#enum_path::#variant),
        syn::Fields::Named(named) => {
            let pairs = named.named.iter().zip(binds).map(|(f, b)| {
                let n = f.ident.as_ref().expect("named field");
                quote!(#n: #b)
            });
            quote!(#enum_path::#variant { #(#pairs),* })
        }
        syn::Fields::Unnamed(_) => quote!(#enum_path::#variant(#(#binds),*)),
    }
}

/// The constructor counterpart of [`variant_pattern`]: rebuild one variant
/// from already-converted field expressions.
fn variant_ctor(
    enum_path: &impl ToTokens,
    variant: &syn::Ident,
    fields: &syn::Fields,
    exprs: &[TokenStream],
) -> TokenStream {
    match fields {
        syn::Fields::Unit => quote!(#enum_path::#variant),
        syn::Fields::Named(named) => {
            let pairs = named.named.iter().zip(exprs).map(|(f, e)| {
                let n = f.ident.as_ref().expect("named field");
                quote!(#n: #e)
            });
            quote!(#enum_path::#variant { #(#pairs),* })
        }
        syn::Fields::Unnamed(_) => quote!(#enum_path::#variant(#(#exprs),*)),
    }
}

/// Hard error when an `.enum_type()`-declared enum is not the shape that
/// declarator describes. A plain C `enum` is exactly a discriminant, which
/// is [`EnumShape::Unit`]; a data-carrying enum crosses as a tag plus a
/// `union` and is reached through a different declarator, so this names
/// that declarator rather than asserting on `syn::Fields`.
fn assert_unit_enum(e: &syn::ItemEnum) {
    use crate::api::core::types_util::{enum_shape, first_payload_variant, EnumShape};
    if enum_shape(e) == EnumShape::Sum {
        let offender = first_payload_variant(e)
            .map(|v| v.ident.to_string())
            .unwrap_or_default();
        panic!(
            "Cbindgen: `{}` is a data-carrying enum (variant `{}` has fields): \
             declare it with `.tagged_union()`, not `.enum_type()` — a C `enum` \
             is a bare discriminant and has no room for a payload",
            e.ident, offender
        );
    }
}

/// PascalCase → snake_case (`ZKeyExpr` → `z_key_expr`).
/// Convert a `PascalCase` / `camelCase` identifier to `snake_case` (a
/// convention-free helper, re-exported as `prebindgen::lang::snake_case` for
/// consumers composing their own [`CbindgenBuilder::mangle_rust_type`] rules).
/// Thin alias for the core spelling, which sum-variant leaf naming shares.
pub fn snake_case(s: &str) -> String {
    crate::api::core::types_util::pascal_to_snake(s)
}

fn is_string(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "String").unwrap_or(false)
}

fn is_str(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "str").unwrap_or(false)
}

/// If `ty` is `Box<T>`, return `T` (used to peel an opaque-pointer struct field
/// such as `Box<String>` / the inner of `Option<Box<String>>`).
fn box_inner(ty: &syn::Type) -> Option<syn::Type> {
    if type_path_tail(ty).map(|i| i == "Box").unwrap_or(false) {
        return first_type_arg(ty);
    }
    None
}

/// If `ty` is `MaybeUninit<T>` (any path form: `MaybeUninit` / `std::mem::…` /
/// `core::mem::…`), return `T` — the inner of an uninitialized out-param slot.
fn maybe_uninit_inner(ty: &syn::Type) -> Option<syn::Type> {
    if type_path_tail(ty)
        .map(|i| i == "MaybeUninit")
        .unwrap_or(false)
    {
        return first_type_arg(ty);
    }
    None
}

/// Whether `ty` is `bool` — the one scalar whose domain is restricted (`0`/`1`),
/// so C-supplied bytes may not be held in it without being normalised first.
fn is_bool(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "bool").unwrap_or(false)
}

/// The C wire for a `bool` in any position C can write: `MaybeUninit<bool>`.
///
/// `bool` is the one FFI-safe scalar with a restricted domain — only `0` and
/// `1` are valid — so a byte a C caller supplies may not be **held** in a Rust
/// `bool` at all, let alone read from one. `MaybeUninit<bool>` holds any byte
/// legally, has `bool`'s size and alignment (so a layout-preserving mirror
/// still transmutes), and is invisible in the header: cbindgen simplifies
/// `MaybeUninit<T>` to `T`, so the C prototype keeps saying `bool`.
///
/// The counterpart read is [`bool_in_expr`]. Together they are the single
/// policy for #170; every position that lets C hand over a `bool` — a
/// parameter, a `data_struct` field, a tagged-union payload — uses this pair
/// and nothing else.
fn bool_wire() -> syn::Type {
    syn::parse_quote!(::core::mem::MaybeUninit<bool>)
}

/// Read a [`bool_wire`] slot the way C converts to `_Bool`: nonzero is true.
///
/// `access` must evaluate to a `MaybeUninit<bool>` place. The byte is read out
/// as a `u8` — legal for any bit pattern — so no invalid `bool` ever exists.
/// Unlike an enum discriminant there is nothing to reject: every byte has an
/// unambiguous C meaning.
///
/// The read is `unsafe`; every caller emits it inside an `unsafe fn` body.
fn bool_in_expr(access: TokenStream) -> TokenStream {
    quote!(::core::ptr::read(#access.as_ptr() as *const u8) != 0)
}

/// Wrap a Rust `bool` for a [`bool_wire`] slot. Rust only ever writes `0`/`1`,
/// so the outbound direction is a pure wrap.
fn bool_out_expr(value: TokenStream) -> TokenStream {
    quote!(::core::mem::MaybeUninit::new(#value))
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

/// Whether an array-producing output appears anywhere in `ty` (including nested
/// under `Result`/`Option`/references).
fn type_contains_vec(ty: &syn::Type) -> bool {
    is_vec(ty)
        || cow_slice_elem(ty).is_some()
        || crate::api::core::registry::immediate_subtype_positions(ty)
            .iter()
            .any(type_contains_vec)
}

/// If `ty` is `Cow<'_, [E]>` with scalar `E`, return `E`.
fn cow_slice_elem(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Cow" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let elem = args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(syn::Type::Slice(slice)) => Some((*slice.elem).clone()),
        _ => None,
    })?;
    is_scalar(&elem).then_some(elem)
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

/// How a *present / ok* value of a return type is carried over the C ABI: an
/// ordered list of wire components plus the representation niches still free
/// for enclosing `Option`/`Result` layers.
struct ValueShape {
    fields: Vec<WireField>,
    niches: Niches,
}

/// Whether a converter function's return type is `Result<_, _>` (⇒ fallible).
fn returns_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => is_result(ty),
        syn::ReturnType::Default => false,
    }
}

fn route_result(call: TokenStream, route: &ErrRoute<'_>) -> TokenStream {
    match route {
        ErrRoute::Result {
            e_conv,
            e_ty_src,
            fail_return,
        } => quote! {
            match #call {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(message) => {
                    if !e.is_null() {
                        *e = #e_conv(
                            <#e_ty_src as ::core::convert::From<
                                ::std::string::String
                            >>::from(message)
                        );
                    }
                    return #fail_return;
                }
            }
        },
        ErrRoute::Panic => quote! {
            match #call {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(message) => panic!("{}", message),
            }
        },
    }
}

/// C-ABI wire type for a struct field. `String` → `*mut c_char`; `bool` →
/// [`bool_wire`]; the remaining FFI-safe scalars pass through. `None` for
/// anything else (unsupported this increment).
fn c_field_wire(ty: &syn::Type) -> Option<syn::Type> {
    if is_string(ty) {
        return Some(syn::parse_quote!(*mut ::core::ffi::c_char));
    }
    // #170 instance 2: the field arrives from C by value, so it may not be a
    // Rust `bool` until the byte has been normalised.
    if is_bool(ty) {
        return Some(bool_wire());
    }
    if is_scalar(ty) {
        return Some(ty.clone());
    }
    None
}
