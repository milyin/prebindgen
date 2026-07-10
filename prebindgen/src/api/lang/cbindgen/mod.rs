//! `Cbindgen` — the C / cbindgen language adapter.
//!
//! A [`Prebindgen`] back-end that turns a "flat" `#[prebindgen]` library into a
//! Rust file suitable for [`cbindgen`](https://github.com/mozilla/cbindgen) to
//! parse into a C header plus a static / dynamic library.
//!
//! Items are **opt-in**: nothing is converted unless it is explicitly declared
//! with [`Cbindgen::function`] / [`Cbindgen::opaque_ptr`] /
//! [`Cbindgen::data_struct`] / [`Cbindgen::enum_type`]. The C name of a declared
//! type's generated destructor can be pinned by chaining [`Cbindgen::name`].
//!
//! ## C ABI conventions
//!
//! * **Pointer struct** (declared with [`Cbindgen::opaque_ptr`]): a `Box`-owned
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
//!
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

// Shared `syn::Type` shape predicates live in `core::types_util`; re-exported
// here under this back-end's historical names so the submodules (`use super::*`)
// keep their call sites. `pub(crate) use` so the glob re-export reaches them.
pub(crate) use crate::api::core::types_util::{
    first_type_arg, is_option_type as is_option, is_result_type as is_result, is_unit,
    is_vec_type as is_vec, path_tail_ident as type_path_tail, result_parts,
};
use crate::api::core::{
    niches::Niches,
    prebindgen::{ConverterImpl, Prebindgen},
    registry::{extract_fn_trait_args, Registry, TypeKey},
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
    /// `mangle_rust_type`-derived base. Set by [`Cbindgen::base_name`]. `None` ⇒
    /// the base comes from `mangle_rust_type(short)` (or the short name).
    base: Option<String>,
}

/// What an inline-opaque by-value type holds, which decides whether its consume
/// path needs a gravestone write-back (and thus a [`crate::core::Gravestone`]
/// impl). See [`Cbindgen::opaque_data_struct`] / [`Cbindgen::opaque_owned_struct`].
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
    /// emitted by [`Cbindgen::prereq_value_opaque`]. Set by
    /// [`Cbindgen::repr_c_struct`]; `false` for `opaque_data_struct` /
    /// `opaque_owned_struct` (counterpart defined elsewhere).
    generate_mirror: bool,
    /// Name config (`.base_name()` override; default naming via the manglers).
    cfg: TypeCfg,
}

/// Per-declared-callback configuration.
#[derive(Clone, Default)]
struct CbCfg {
    /// Per-declaration **base** token override fed to `mangle_callback` (as the
    /// sole base, replacing the args' derived bases). Set by
    /// [`Cbindgen::base_name`]. `None` ⇒ bases come from the arguments.
    base: Option<String>,
    /// Argument indices delivered to the C `call` as a **takeable owned pointer**
    /// (`*mut z_x_t`) instead of by value: the callee may take the value (move it
    /// out via `z_x_take`, leaving a gravestone) or just read it, and the
    /// trampoline drops it after the call (no-op if taken). Set by
    /// [`Cbindgen::takeable_param`]; each such arg type must be an inline-opaque
    /// type ([`Cbindgen::opaque_owned_struct`] / [`Cbindgen::opaque_data_struct`]).
    takeable: std::collections::BTreeSet<usize>,
}

/// Per-declared-function configuration.
#[derive(Clone, Default)]
struct FnCfg {
    /// Per-declaration **base** token override fed to `mangle_function` in place of
    /// the Rust fn ident. Set by [`Cbindgen::base_name`]. `None` ⇒ the fn ident.
    base: Option<String>,
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
    ValueOpaque(TypeKey),
    Enum(TypeKey),
    Callback(CallbackKey),
    Function(syn::Ident),
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
    /// Inline-opaque by-value types: the Rust value is transmuted to/from an
    /// opaque `#[repr(C, align(_))]` counterpart of identical size+align (no
    /// `Box`). Keyed by the Rust type; the value carries the opaque counterpart.
    value_opaque: HashMap<TypeKey, ValueOpaqueCfg>,
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
}

/// A mangler over a single name component (Rust short name, base, or fn ident).
type Mangle1 = Box<dyn Fn(&str) -> String>;
/// A mangler over a callback's argument bases.
type MangleN = Box<dyn Fn(&[String]) -> String>;

mod builder;
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
