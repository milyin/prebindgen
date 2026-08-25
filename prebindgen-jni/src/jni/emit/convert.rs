//! Scalar and `Option` converter bodies and their wire probes.

use super::*;

/// Sentinel value to return through the wrapper signature when the inner
/// closure errors. Must compile against any wire type we emit.
pub(crate) fn sentinel_for_wire(wire: &syn::Type) -> TokenStream {
    // Unit wire (void-returning wrappers): the value *is* the sentinel.
    if let syn::Type::Tuple(t) = wire {
        if t.elems.is_empty() {
            return quote!(());
        }
    }
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let name = last.ident.to_string();
            return match name.as_str() {
                "jboolean" | "jbyte" | "jchar" | "jshort" | "jint" | "jlong" => quote!(0 as #wire),
                "jfloat" | "jdouble" => quote!(0.0 as #wire),
                "JObject" | "JString" | "JByteArray" | "JClass" => {
                    quote!(jni::objects::JObject::null().into())
                }
                _ => quote!(unsafe { std::mem::zeroed::<#wire>() }),
            };
        }
    }
    if matches!(wire, syn::Type::Ptr(_)) {
        return quote!(std::ptr::null());
    }
    quote!(unsafe { std::mem::zeroed::<#wire>() })
}

// ──────────────────────────────────────────────────────────────────────
// Primitive bodies
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn primitive_input(key: &TypeKey) -> Option<(syn::Type, syn::Expr)> {
    let key = key.as_str().to_string();
    // Bodies receive `v: &<wire>`; primitives are Copy so `*v` works.
    Some(match key.as_str() {
        "bool" => (
            syn::parse_quote!(jni::sys::jboolean),
            syn::parse_quote!(*v != 0),
        ),
        "i32" => (syn::parse_quote!(jni::sys::jint), syn::parse_quote!(*v)),
        "i64" => (syn::parse_quote!(jni::sys::jlong), syn::parse_quote!(*v)),
        "u8" => (
            syn::parse_quote!(jni::sys::jint),
            syn::parse_quote!(::core::primitive::u8::try_from(*v).map_err(|_| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "u8 input out of range: {}",
                    *v
                ))
            })?),
        ),
        "u16" => (
            syn::parse_quote!(jni::sys::jint),
            syn::parse_quote!(::core::primitive::u16::try_from(*v).map_err(|_| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "u16 input out of range: {}",
                    *v
                ))
            })?),
        ),
        "u32" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(::core::primitive::u32::try_from(*v).map_err(|_| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "u32 input out of range: {}",
                    *v
                ))
            })?),
        ),
        // Kotlin's public surface is `ULong`, but the JNI tier receives its
        // underlying `Long` bit pattern. Rust's `as u64` is the inverse of
        // Kotlin's `ULong.toLong()` for all 64 bits.
        "u64" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(*v as ::core::primitive::u64),
        ),
        "f64" => (syn::parse_quote!(jni::sys::jdouble), syn::parse_quote!(*v)),
        "String" => (
            syn::parse_quote!(jni::objects::JString),
            syn::parse_quote!({
                let s = env.get_string(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_string: {}",
                        e
                    ))
                })?;
                s.into()
            }),
        ),
        "Vec < u8 >" => (
            syn::parse_quote!(jni::objects::JByteArray),
            syn::parse_quote!({
                env.convert_byte_array(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_byte_array: {}",
                        e
                    ))
                })?
            }),
        ),
        _ => return None,
    })
}

pub(crate) fn primitive_output(key: &TypeKey) -> Option<(syn::Type, syn::Expr)> {
    let key = key.as_str().to_string();
    // Output wrappers take v by value (move). Primitives are Copy, so
    // `v as wire` works. String/Vec consume v.
    Some(match key.as_str() {
        "bool" => (
            syn::parse_quote!(jni::sys::jboolean),
            syn::parse_quote!(v as jni::sys::jboolean),
        ),
        "i32" => (
            syn::parse_quote!(jni::sys::jint),
            syn::parse_quote!(v as jni::sys::jint),
        ),
        "i64" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(v as jni::sys::jlong),
        ),
        "u8" | "u16" => (
            syn::parse_quote!(jni::sys::jint),
            syn::parse_quote!(v as jni::sys::jint),
        ),
        "u32" | "u64" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(v as jni::sys::jlong),
        ),
        "f64" => (
            syn::parse_quote!(jni::sys::jdouble),
            syn::parse_quote!(v as jni::sys::jdouble),
        ),
        "String" => (
            syn::parse_quote!(jni::objects::JString),
            syn::parse_quote!({
                env.new_string(v.as_str()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "encode_string: {}",
                        e
                    ))
                })?
            }),
        ),
        "Vec < u8 >" => (
            syn::parse_quote!(jni::objects::JByteArray),
            syn::parse_quote!({
                env.byte_array_from_slice(v.as_slice()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "encode_byte_array: {}",
                        e
                    ))
                })?
            }),
        ),
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Option<_> wrappers
// ──────────────────────────────────────────────────────────────────────

// ──────────────────────────────────────────────────────────────────────
// Callback wrappers — impl Fn(args) -> JObject (erased Kotlin lambda)
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn is_jobject_wire(wire: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return last.ident == "JObject";
        }
    }
    false
}

/// True if `wire` is a JNI handle (`JObject`, `JString`, `JByteArray`,
/// `JClass`) that natively supports a `null` discriminator. These types
/// all impl `is_null()` and accept `JObject::null().into()` for
/// construction.
pub(crate) fn is_jobject_shaped_wire(wire: &syn::Type) -> bool {
    crate::jni::wire_access::is_jni_reference_wire(wire)
}

/// Default niche set for a JNI wrapper wire: every `J*` handle has a
/// genuine `null` value that no live conversion ever produces, so wrap
/// it as a single niche; everything else (`jlong`, `jint`, `()`, …) has
/// no implicit niche.
///
/// Plugins are free to declare *additional* niches on top of this for
/// pointer-shape primitives like `Box::into_raw`-as-`jlong`.
pub(crate) fn default_niches_for_wire(wire: &syn::Type) -> Niches {
    if is_jobject_shaped_wire(wire) {
        Niches::one(
            syn::parse_quote!(jni::objects::JObject::null().into()),
            syn::parse_quote!(v.is_null()),
        )
    } else {
        Niches::empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// JNI-internal naming convention. Hand-written code in zenoh-jni
// (e.g. liveliness.rs, advanced_subscriber.rs) calls auto-generated
// converters by these computed names — so the convention is part of the
// JNI plugin's public contract, not a private implementation detail.
// ──────────────────────────────────────────────────────────────────────

/// `OwnedObject<T>` definition emitted into the destination Rust file.
///
/// A non-owning borrow wrapper around a `*const T` whose backing
/// `Box<T>` lives on the Java side. The Java side hands Rust the
/// pointer under its `NativeHandle.withPtr` read lock; for the
/// duration of the JNI call the heap allocation is guaranteed live,
/// so `Deref<Target = T>` exposing `&*ptr` is sound. The wrapper has
/// no `Drop`: nothing is freed here, the Box stays with Java.
///
/// By-value `T` extraction is intentionally NOT through this wrapper.
/// Consume call sites use `*Box::from_raw(ptr)` inline, taking
/// ownership of Java's slot; `NativeHandle.consume` (write-lock +
/// atomic null) sequences that against any concurrent borrow.
///
/// Co-locating the definition with the converters keeps the generated
/// file self-contained — no `use` statement or runtime-support module
/// is required from the host crate.
pub(crate) fn owned_object_prerequisite_items() -> Vec<syn::Item> {
    vec![
        syn::parse_quote!(
            /// See module-level docs at [`owned_object_prerequisite_items`].
            #[allow(dead_code)]
            pub(crate) struct OwnedObject<T: ?Sized> {
                ptr: *const T,
            }
        ),
        syn::parse_quote!(
            impl<T: ?Sized> std::ops::Deref for OwnedObject<T> {
                type Target = T;
                #[inline]
                fn deref(&self) -> &Self::Target {
                    unsafe { &*self.ptr }
                }
            }
        ),
        syn::parse_quote!(
            // `&mut OwnedObject<T>` coerces to `&mut T` via this impl,
            // letting source fns that take `&mut T` opaque-handle params
            // be called from generated wrappers. The pointer originated
            // from `Box::into_raw` (which produces `*mut T`); the
            // `*const T → *mut T` cast just restores the original
            // mutability. Sequencing against concurrent borrow / consume
            // is upheld by `NativeHandle.withPtr` on the JVM side, same
            // as `Deref`.
            impl<T: ?Sized> std::ops::DerefMut for OwnedObject<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *(self.ptr as *mut T) }
                }
            }
        ),
        syn::parse_quote!(
            impl<T: ?Sized> OwnedObject<T> {
                /// Borrow a `T` whose backing `Box<T>` lives on the
                /// Java side. Stores only the pointer; the wrapper
                /// does not own the heap allocation and never frees
                /// it on drop.
                ///
                /// # Safety
                ///
                /// `ptr` must be the result of an earlier
                /// `Box::into_raw(Box::new(v))` and the allocation
                /// must still be live (Java still owns it). The Java
                /// side is responsible for sequencing this call
                /// against any concurrent free or consume (via
                /// `NativeHandle.withPtr` read-lock vs `consume` /
                /// `close` write-lock) so the borrow cannot race a
                /// deallocation on the same pointer.
                #[allow(dead_code)]
                pub(crate) unsafe fn from_raw(ptr: *const T) -> Self {
                    Self { ptr }
                }
            }
        ),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────
//
// These tests exercise the niche cascade by hand-building registry
// entries with deliberate niche shapes, then driving `option_input` /
// `option_output` directly. They mirror the documented `Niches`
// semantics: each `Option<_>` layer carves one slot and re-exports the
// rest; once the rest is exhausted, the next layer falls back to the
// boxed-Java-primitive scheme.
