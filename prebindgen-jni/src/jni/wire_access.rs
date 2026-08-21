//! Map a JNI wire type to its `(jvm_signature_chunk, JValue accessor,
//! is_object)` triple — shared between the struct-strategy decoder/encoder
//! and the callback strategy. The primitive recipes are per-aspect views over
//! [`JniPrim`](super::JniPrim); only the object wires are local.

use quote::format_ident;

use super::JniPrim;

/// Map a JNI wire type to `(jvm_field_descriptor, JValue_accessor_ident, is_object)`.
///
/// Primitive types (`jlong`, `jint`, …) set `is_object = false` and the
/// accessor names the `.j()` / `.i()` / … `JValue` variant.
///
/// Object types (`JString`, `JByteArray`, …) set `is_object = true`; the
/// caller uses `.l()` to get a `JObject` and then `.into()` to cast to the
/// wire type.
/// The Kotlin primitive arrays and their JVM descriptors.
///
/// ONE table, read forwards (Kotlin type → descriptor, for interface signatures
/// and field slots) and backwards (descriptor → empty literal, for inert
/// flattened slots). Four separate copies of this knowledge existed before
/// fixed-size arrays needed all of them, and only some got updated.
pub(crate) const KOTLIN_PRIM_ARRAYS: [(&str, &str); 8] = [
    ("BooleanArray", "[Z"),
    ("ByteArray", "[B"),
    ("CharArray", "[C"),
    ("ShortArray", "[S"),
    ("IntArray", "[I"),
    ("LongArray", "[J"),
    ("FloatArray", "[F"),
    ("DoubleArray", "[D"),
];

/// JVM descriptor of a Kotlin primitive array (`ByteArray` → `[B`).
pub(crate) fn kotlin_array_descriptor(name: &str) -> Option<&'static str> {
    KOTLIN_PRIM_ARRAYS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, d)| *d)
}

/// The Kotlin primitive array a JVM array descriptor names (`[B` → `ByteArray`).
pub(crate) fn kotlin_array_of_descriptor(descr: &str) -> Option<&'static str> {
    KOTLIN_PRIM_ARRAYS
        .iter()
        .find(|(_, d)| *d == descr)
        .map(|(n, _)| *n)
}

/// The JNI **reference** (object-shaped) wire types: the `J*` handles plus the
/// eight primitive arrays.
///
/// ONE list. It drives the JVM field descriptor, the wire→`JObject` cast, and
/// the lifetime annotation on a generated converter's signature — three sites
/// that each kept their own copy until fixed-size arrays added a wire to all of
/// them and only two got updated.
pub(crate) fn is_jni_reference_wire(ty: &syn::Type) -> bool {
    let syn::Type::Path(tp) = ty else {
        return false;
    };
    let Some(last) = tp.path.segments.last() else {
        return false;
    };
    matches!(
        last.ident.to_string().as_str(),
        "JObject"
            | "JString"
            | "JClass"
            | "JBooleanArray"
            | "JByteArray"
            | "JCharArray"
            | "JShortArray"
            | "JIntArray"
            | "JLongArray"
            | "JFloatArray"
            | "JDoubleArray"
    )
}

pub(crate) fn jni_field_access(jni_type: &syn::Type) -> Option<(&'static str, syn::Ident, bool)> {
    if let Some(p) = JniPrim::from_wire(jni_type) {
        return Some((p.descriptor(), format_ident!("{}", p.unbox_getter()), false));
    }
    let syn::Type::Path(tp) = jni_type else {
        return None;
    };
    let sig = match tp.path.segments.last()?.ident.to_string().as_str() {
        "JString" => "Ljava/lang/String;",
        // The eight primitive-array wires, one per `JniPrim` scalar — a
        // fixed-size Rust array crosses as the matching Kotlin primitive array
        // (see `prim_array`).
        "JBooleanArray" => "[Z",
        "JByteArray" => "[B",
        "JCharArray" => "[C",
        "JShortArray" => "[S",
        "JIntArray" => "[I",
        "JLongArray" => "[J",
        "JFloatArray" => "[F",
        "JDoubleArray" => "[D",
        _ => return None,
    };
    Some((sig, format_ident!("l"), true))
}

/// Map a JNI **primitive** field descriptor to the descriptor of its
/// `java.lang.*` box class — the JVM slot an `Option`-boxed primitive leaf
/// occupies ([`box_helper_for_wire`] produces the boxed value; this names its
/// type in a method signature).
pub(crate) fn box_descriptor_for_primitive(sig: &str) -> Option<&'static str> {
    JniPrim::from_descriptor(sig).map(JniPrim::box_descriptor)
}

/// Map a JNI **primitive** wire type to the `prebindgen::lang` cached-boxing
/// runtime helper that boxes it into its `java.lang.*` wrapper. Used when a
/// primitive leaf must be delivered through an *erased* `Object`-typed call
/// (e.g. a Kotlin function type's `invoke`). The helpers resolve the box class
/// and its static `valueOf` once per process — boxing inline with
/// `env.new_object("java/lang/Integer", …)` would re-run `FindClass` +
/// `GetMethodID` on every delivery, which dominates the callback hot path.
/// Returns `None` for object wires (`JString`/`JByteArray`/`JObject` are
/// already objects).
pub(crate) fn box_helper_for_wire(wire: &syn::Type) -> Option<syn::Ident> {
    JniPrim::from_wire(wire).map(|p| format_ident!("box_{}", p.wire_name()))
}
