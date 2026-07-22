//! Scalar / `Option` / enum converter bodies and their wire probes.

use super::*;
use crate::api::core::registry::TypeEntry;

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

pub(crate) fn primitive_input(ty: &syn::Type) -> Option<(syn::Type, syn::Expr)> {
    let key = TypeKey::from_type(ty).as_str().to_string();
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

pub(crate) fn primitive_output(ty: &syn::Type) -> Option<(syn::Type, syn::Expr)> {
    let key = TypeKey::from_type(ty).as_str().to_string();
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

/// Invoke an inner input converter's complete `wire -> Rust` chain.
///
/// Structural wrappers cannot call only [`TypeEntry::function`]: custom
/// conversions may carry semantic steps in `pre_stages` (for example
/// `jlong -> u64 -> Duration`). Keep those steps inside the `Some` arm so a
/// niche discriminator is tested before any conversion runs.
fn composed_inner_input(inner: &TypeEntry<KotlinMeta>, wire: TokenStream) -> syn::Expr {
    let converter = inner.converter_ident();
    if inner.pre_stages.is_empty() {
        return syn::parse2(quote!(#converter(env, #wire)?))
            .expect("single-stage input call is a valid expression");
    }
    let mut body = quote! {
        let __inner_s0 = #converter(env, #wire)?;
    };
    let mut previous = format_ident!("__inner_s0");
    for (order, (_, stage)) in inner.input_stage_order().enumerate() {
        let stage_fn = &stage.function.sig.ident;
        let next = format_ident!("__inner_s{}", order + 1);
        body.extend(quote! {
            let #next = #stage_fn(env, #previous)?;
        });
        previous = next;
    }
    body.extend(quote!(#previous));
    syn::parse2(quote!({ #body })).expect("composed input chain is a valid expression")
}

/// Invoke an inner output converter's complete `Rust -> wire` chain.
/// Mirror of [`composed_inner_input`].
fn composed_inner_output(inner: &TypeEntry<KotlinMeta>, value: TokenStream) -> syn::Expr {
    let converter = inner.converter_ident();
    if inner.pre_stages.is_empty() {
        return syn::parse2(quote!(#converter(env, #value)?))
            .expect("single-stage output call is a valid expression");
    }
    let mut body = TokenStream::new();
    let mut previous = value;
    for (order, (_, stage)) in inner.output_stage_order().enumerate() {
        let stage_fn = &stage.function.sig.ident;
        let next = format_ident!("__inner_s{}", order);
        body.extend(quote! {
            let #next = #stage_fn(env, #previous)?;
        });
        previous = quote!(#next);
    }
    body.extend(quote!(#converter(env, #previous)?));
    syn::parse2(quote!({ #body })).expect("composed output chain is a valid expression")
}

/// Build `Option<T>`'s input converter.
///
/// Two paths, picked in this order:
///
/// 1. **Niche path** (preferred). If `T`'s converter exposes any niche
///    slots, carve the first one and use it as the `None` discriminator.
///    The wrapper keeps `T`'s wire unchanged — no boxing, no extra
///    allocation, ABI-identical to a hand-written `if v == sentinel`.
///    The `rest` of the niche set is re-exported on the wrapper so an
///    enclosing wrapper (e.g. `Option<Option<T>>`) can keep carving.
///
/// 2. **Boxed-primitive fallback**. If `T`'s wire is a JNI primitive
///    (`jlong`, `jint`, …) and there is no niche, the wrapper widens
///    the wire to `JObject` carrying a Java boxed type (`java.lang.Long`,
///    `java.lang.Integer`, …). `null` denotes `None`. The wrapper
///    exposes no further niches — every `JObject` value already carries
///    meaning (null = None, non-null = Some).
///
/// If neither path applies (non-primitive wire, no niche), the wrap
/// fails and the resolver falls through to other rank-1 attempts.
pub(crate) fn option_input(
    t1: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr, Niches)> {
    let inner_entry = registry.input_entry(t1)?;
    let inner_wire = inner_entry.destination.clone();
    let inner_decode = composed_inner_input(inner_entry, quote!(v));

    // 1. Niche path.
    if let Some((slot, rest)) = inner_entry.niches.clone().carve() {
        let pred = &slot.matches;
        let returns_owned_object = inner_entry.metadata.is_direct_handle();
        let body: syn::Expr = if returns_owned_object {
            // Borrow semantics: the Java side still owns the boxed value
            // (its `close()` will free the original Box later via the typed
            // handle's `freePtr`). Cloning the inner T keeps the pointer
            // live across this call — using `Box::from_raw` here would
            // consume the box, leaving the Java slot dangling and causing
            // a double-free the next time the same data-class instance is
            // decoded. Requires `T: Clone`.
            syn::parse_quote!({
                if #pred {
                    None
                } else {
                    Some(unsafe { OwnedObject::from_raw(*v as *const #t1).clone() })
                }
            })
        } else {
            syn::parse_quote!({
                if #pred { None } else { Some(#inner_decode) }
            })
        };
        return Some((inner_wire, body, rest));
    }

    // 2. Boxed-primitive fallback.
    if is_jni_primitive(&inner_wire) {
        let unbox_method = jni_unbox_method(&inner_wire);
        let unbox_sig = jni_unbox_sig(&inner_wire);
        let getter = jni_unbox_getter(&inner_wire);
        let getter_id = format_ident!("{}", getter);
        let inner_decode = composed_inner_input(inner_entry, quote!(&__unboxed));
        let body: syn::Expr = syn::parse_quote!({
            if !v.is_null() {
                let __unboxed: #inner_wire = env
                    .call_method(&v, #unbox_method, #unbox_sig, &[])
                    // `JValue::z()` yields a Rust `bool`, every other accessor
                    // yields its matching `jni::sys` type; the `as #inner_wire`
                    // coerces `bool → jboolean` and is an identity cast for the
                    // numeric accessors.
                    .and_then(|val| val.#getter_id())
                    .map(|__x| __x as #inner_wire)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Option unbox: {}", e)))?;
                Some(#inner_decode)
            } else {
                None
            }
        });
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        return Some((wire, body, Niches::empty()));
    }

    None
}

/// Build `Option<T>`'s output converter — symmetric to [`option_input`].
pub(crate) fn option_output(
    t1: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr, Niches)> {
    let inner_entry = registry.output_entry(t1)?;
    let inner_wire = inner_entry.destination.clone();
    let inner_encode = composed_inner_output(inner_entry, quote!(value));

    // 1. Niche path.
    if let Some((slot, rest)) = inner_entry.niches.clone().carve() {
        let none_value = &slot.value;
        let body: syn::Expr = syn::parse_quote!({
            match v {
                Some(value) => #inner_encode,
                None => #none_value,
            }
        });
        return Some((inner_wire, body, rest));
    }

    // 2. Boxed-primitive fallback (cached box class + `valueOf` method ID).
    if let Some(helper) = box_helper_for_wire(&inner_wire) {
        let inner_encode = composed_inner_output(inner_entry, quote!(value));
        let body: syn::Expr = syn::parse_quote!({
            match v {
                Some(value) => {
                    let __raw: #inner_wire = #inner_encode;
                    ::prebindgen::lang::#helper(env, __raw)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Option box: {}", e)))?
                }
                None => jni::objects::JObject::null(),
            }
        });
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        return Some((wire, body, Niches::empty()));
    }

    None
}

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
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return matches!(
                last.ident.to_string().as_str(),
                "JObject" | "JString" | "JByteArray" | "JClass"
            );
        }
    }
    false
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
// Struct rank-0 bodies
// ──────────────────────────────────────────────────────────────────────

/// `jint → Rust enum` decoder body for a `enum_class`-declared enum.
/// Wire is `jni::sys::jint`. The framework builds the decode `match`
/// directly from the enum's own discriminants — no `TryFrom<i32>` impl
/// is required on the flat enum (the enum declaration is the single
/// source of truth for the int↔variant mapping, shared with the Kotlin
/// `value(N)` constants via [`enum_discriminant_values`]). An unknown
/// discriminant surfaces as the framework `__JniErr`.
///
/// The arms use the bare ident — same shape as the wrapper function's
/// `v: <ident>` signature — so binding crates can pick whichever
/// upstream type a bare `<ident>` resolves to in their include-site
/// `use` statements. Pairs with output body below.
pub(crate) fn enum_input_body(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    e: &syn::ItemEnum,
) -> (syn::Type, syn::Expr) {
    assert_only_unit_variants(e);
    let ident = &e.ident;
    let ident_name = ident.to_string();
    // Qualify the variant constructors with the enum's origin module,
    // exactly as the type-position pass qualifies the enum's return type —
    // otherwise a bare `Enum::Variant` fails to resolve when the enum lives
    // in a source crate (the usual flat-library case).
    let source_module = ext.fn_module(registry, ident);
    let arms = crate::api::lang::jnigen::util::enum_discriminant_values(e)
        .into_iter()
        .map(|(variant, value)| {
            let lit = proc_macro2::Literal::i64_unsuffixed(value);
            quote! { #lit => #source_module::#ident::#variant, }
        });
    let body: syn::Expr = syn::parse_quote!({
        match *v as i64 {
            #(#arms)*
            other => {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(
                        format!("invalid {} discriminant: {}", #ident_name, other)
                    )
                );
            }
        }
    });
    (syn::parse_quote!(jni::sys::jint), body)
}

/// `Rust enum → jint` encoder body for a `enum_class`-declared enum.
/// Wire is `jni::sys::jint`. Relies on the declared enum's repr
/// supporting an `as` cast (i.e. C-like enum, no fields); the
/// [`assert_only_unit_variants`] check below catches violations
/// upstream of the cast. The body works without naming the enum type
/// at all — `v` is already typed via the wrapper signature, so the
/// `as` cast picks up the right type by inference.
pub(crate) fn enum_output_body(_ext: &JniGen, e: &syn::ItemEnum) -> (syn::Type, syn::Expr) {
    assert_only_unit_variants(e);
    let body: syn::Expr = syn::parse_quote!({ v as jni::sys::jint });
    (syn::parse_quote!(jni::sys::jint), body)
}

/// Hard error on any enum that's not C-like (unit variants only).
/// `enum_class`'s discriminant-keyed Kotlin emission and `as jint`
/// encode both depend on unit variants — bail loudly at build time
/// rather than emitting wrong code.
pub(crate) fn assert_only_unit_variants(e: &syn::ItemEnum) {
    for variant in &e.variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            panic!(
                "enum_class only supports C-like enums (unit variants), \
                 but `{}::{}` has fields",
                e.ident, variant.ident
            );
        }
    }
}

/// Decide which [`NullableKind`] to fold for an `Option<_>` wrapper, given
/// the wrapper's destination wire and the registry-resolved inner. The
/// detection mirrors the two paths in [`option_input`] / [`option_output`]:
/// the niche path keeps the inner's wire untouched (e.g. `jlong` stays
/// `jlong`, `JByteArray` stays `JByteArray`), while the boxed-primitive
/// fallback widens the wire to `JObject`. So `outer_wire == inner.destination`
/// uniquely identifies the niche path.
///
/// Symmetric `_input` / `_output` flavors only differ in which registry side
/// they consult — the comparison is identical.
pub(crate) fn nullable_kind_for(
    outer_wire: &syn::Type,
    inner_ty: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> NullableKind {
    let inner_dest = registry
        .input_entry(inner_ty)
        .map(|e| e.destination.clone())
        .expect(
            "nullable_kind_for: Option<_> input handler reached here only after option_input \
             returned Some, so the inner's input entry must exist",
        );
    if outer_wire == &inner_dest {
        NullableKind::Niche
    } else {
        NullableKind::Boxed
    }
}

pub(crate) fn nullable_kind_for_output(
    outer_wire: &syn::Type,
    inner_ty: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> NullableKind {
    let inner_dest = registry
        .output_entry(inner_ty)
        .map(|e| e.destination.clone())
        .expect(
            "nullable_kind_for_output: Option<_> output handler reached here only after \
             option_output returned Some, so the inner's output entry must exist",
        );
    if outer_wire == &inner_dest {
        NullableKind::Niche
    } else {
        NullableKind::Boxed
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
