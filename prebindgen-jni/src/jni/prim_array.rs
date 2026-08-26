//! Fixed-size arrays (`[T; N]`) of JNI-primitive elements, crossing as the
//! matching Kotlin primitive array.
//!
//! `[u8; 16]` ⇄ `ByteArray`, `[i64; 4]` ⇄ `LongArray`, and so on for every
//! [`JniPrim`](super::prim::JniPrim) scalar. Primitive arrays bulk-copy through
//! `set_*_array_region` / `get_*_array_region` and box nothing, which is why a
//! fixed-size array does NOT go through the `Vec<T>` → `List<T>` path.
//!
//! Wider unsigned elements (`[u16; N]`, `[u32; N]`, `[u64; N]`) carry the **raw
//! bit pattern** in the signed array. That matches the existing scalar rule — a
//! `u64` already crosses as a raw `jlong` — and `Vec<u8>` → `ByteArray`, where
//! Kotlin's `Byte` is signed. Kotlin's own `UByteArray`/`ULongArray` are
//! `@ExperimentalUnsignedTypes`, so using them would push an opt-in onto every
//! consumer of a shared binding tier.
//!
//! **`N` is never needed at generation time.** It is often a const *path* rather
//! than a literal (`ZenohId` is `[u8; ZENOH_ID_MAX_SIZE]`), so the decode leans
//! on `TryFrom<&[T]> for [T; N]` and lets `rustc` infer the length; a JVM array
//! of the wrong length becomes a binding error rather than a panic.
//!
//! Element conversion is by `as`-cast (or `!= 0` for `bool`), never a transmute:
//! `jboolean` is a `u8`, and reinterpreting a byte of `2` as a Rust `bool` would
//! be UB — the very hazard that retired the raw-memory value blob this module
//! replaces.

use kotlin_codegen::KtType;

use super::*;

/// The JNI/Kotlin array pair for one primitive element type.
#[derive(Clone)]
pub(crate) struct PrimArray {
    /// Wire type: `jni::objects::JLongArray`.
    pub wire: syn::Type,
    /// JNI element type: `jni::sys::jlong`.
    pub elem_wire: syn::Type,
    /// Kotlin surface: `LongArray`.
    pub kotlin: KtType,
    /// `JNIEnv::new_long_array`.
    pub new_fn: syn::Ident,
    /// `JNIEnv::set_long_array_region`.
    pub set_region: syn::Ident,
    /// `JNIEnv::get_long_array_region`.
    pub get_region: syn::Ident,
    /// True for `[bool; N]` — its element needs normalizing rather than casting.
    pub is_bool: bool,
    /// True for `[u8; N]` — the one case with a dedicated bulk helper.
    pub is_u8: bool,
}

/// Classify `ty` as a fixed-size array of JNI-primitive elements.
///
/// `None` for everything else, including `[T; N]` of a declared class or enum —
/// those keep resolving as unsupported, so an unhandled shape is a clear
/// resolve error rather than silently wrong code.
pub(crate) fn prim_array_of(ty: &prebindgen_registry::flat::TypeRef) -> Option<PrimArray> {
    use prebindgen_registry::flat::{ScalarKind, TypeKind};
    let TypeKind::Array { elem, .. } = ty.kind() else {
        return None;
    };
    let &TypeKind::Scalar(elem) = elem.kind() else {
        return None;
    };
    // `usize`/`isize` are deliberately absent: their width is platform
    // dependent, so there is no stable JNI element type to pick. Reaching them
    // by name is the ScalarKind's own spelling, so the set below is still the
    // set Rust writes — a kind that is not on it falls through as before.
    let (letter, jni_elem) = match elem.as_str() {
        "u8" | "i8" => ("byte", "jbyte"),
        "u16" | "i16" => ("short", "jshort"),
        "u32" | "i32" => ("int", "jint"),
        "u64" | "i64" => ("long", "jlong"),
        "f32" => ("float", "jfloat"),
        "f64" => ("double", "jdouble"),
        "bool" => ("boolean", "jboolean"),
        _ => return None,
    };
    let cap = format!("{}{}", letter[..1].to_uppercase(), &letter[1..]);
    let wire_ident = format_ident!("J{}Array", cap);
    let elem_wire_ident = format_ident!("{}", jni_elem);
    Some(PrimArray {
        wire: syn::parse_quote!(jni::objects::#wire_ident),
        elem_wire: syn::parse_quote!(jni::sys::#elem_wire_ident),
        kotlin: KtType::cls(format!("{cap}Array")),
        new_fn: format_ident!("new_{}_array", letter),
        set_region: format_ident!("set_{}_array_region", letter),
        get_region: format_ident!("get_{}_array_region", letter),
        is_bool: elem == ScalarKind::Bool,
        is_u8: elem == ScalarKind::U8,
    })
}

/// `[T; N]` → the Kotlin primitive array (Rust → wire).
pub(crate) fn output_body(spec: &PrimArray) -> syn::Expr {
    if spec.is_u8 {
        // `&[u8; N]` derefs to `&[u8]`, so the dedicated helper applies with no
        // intermediate buffer — the common case (`ZenohId`).
        return syn::parse_quote!({
            env.byte_array_from_slice(&v).map_err(|e| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "fixed-size array encode: {}",
                    e
                ))
            })?
        });
    }
    let elem_wire = &spec.elem_wire;
    let new_fn = &spec.new_fn;
    let set_region = &spec.set_region;
    // One form for every element: `bool as u8` yields 0/1, and the rest are
    // same-width numeric casts. Only the DECODE needs a special case, because
    // `u8 as bool` is not a cast at all.
    let to_wire: syn::Expr = syn::parse_quote!(*__x as #elem_wire);
    syn::parse_quote!({
        let __buf: ::std::vec::Vec<#elem_wire> = v.iter().map(|__x| #to_wire).collect();
        let __arr = env.#new_fn(__buf.len() as jni::sys::jsize).map_err(|e| {
            <__JniErr as ::core::convert::From<String>>::from(format!(
                "fixed-size array encode: {}",
                e
            ))
        })?;
        env.#set_region(&__arr, 0, &__buf).map_err(|e| {
            <__JniErr as ::core::convert::From<String>>::from(format!(
                "fixed-size array encode: {}",
                e
            ))
        })?;
        __arr
    })
}

/// The Kotlin primitive array → `[T; N]` (wire → Rust).
///
/// The length check is the `try_into`: a JVM array of the wrong size becomes a
/// binding error naming the type, never a panic or a partially-filled array.
pub(crate) fn input_body(
    ty: &prebindgen_registry::flat::TypeRef,
    spec: &PrimArray,
    emit: &prebindgen_registry::RustWriter,
) -> syn::Expr {
    let key = ty.key();
    // The element spelled from the model's own `Array`, so the local's type
    // ascription cannot disagree with what `prim_array_of` matched.
    let elem_ty = match ty.kind() {
        prebindgen_registry::flat::TypeKind::Array { elem, .. } => emit.emit_source_type(elem),
        _ => unreachable!("prim_array_of matched a non-array"),
    };
    // The ascription the decoded array is checked against — spelled from the
    // reading, as generated Rust always spells.
    let ty = emit.emit_source_type(ty);
    let len_err = format!("fixed-size array decode: `{key}` expects a different length");
    if spec.is_u8 {
        return syn::parse_quote!({
            let __buf = env.convert_byte_array(v).map_err(|e| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "fixed-size array decode: {}",
                    e
                ))
            })?;
            let __arr: #ty = __buf.as_slice().try_into().map_err(|_| {
                <__JniErr as ::core::convert::From<String>>::from(#len_err.to_string())
            })?;
            __arr
        });
    }
    let elem_wire = &spec.elem_wire;
    let get_region = &spec.get_region;
    // A `jboolean` is a `u8`: normalize it, never reinterpret it — an out-of-
    // range byte read back as a Rust `bool` would be undefined behavior.
    let from_wire: syn::Expr = if spec.is_bool {
        syn::parse_quote!(*__x != 0)
    } else {
        syn::parse_quote!(*__x as #elem_ty)
    };
    syn::parse_quote!({
        let __len = env.get_array_length(v).map_err(|e| {
            <__JniErr as ::core::convert::From<String>>::from(format!(
                "fixed-size array decode: {}",
                e
            ))
        })? as usize;
        let mut __buf: ::std::vec::Vec<#elem_wire> = ::std::vec![0 as #elem_wire; __len];
        env.#get_region(v, 0, &mut __buf).map_err(|e| {
            <__JniErr as ::core::convert::From<String>>::from(format!(
                "fixed-size array decode: {}",
                e
            ))
        })?;
        let __vals: ::std::vec::Vec<#elem_ty> = __buf.iter().map(|__x| #from_wire).collect();
        let __arr: #ty = __vals.as_slice().try_into().map_err(|_| {
            <__JniErr as ::core::convert::From<String>>::from(#len_err.to_string())
        })?;
        __arr
    })
}
