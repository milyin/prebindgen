//! JNI primitive (un)boxing lookup tables.
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.
//!
//! The eight JNI scalar wire types ([`JniPrim`]) and their boxed-`java.lang.*`
//! counterparts are described by one enum + per-aspect accessors, replacing
//! the parallel `match jni_prim_name(..)` tables that used to repeat the
//! same 8-way classification. The free `jni_unbox_*` / `is_jni_primitive`
//! functions are kept as thin shims so call sites are unchanged. (The boxing
//! direction lives in the `box_helpers` runtime module — generated code boxes
//! via the cached `prebindgen::lang::box_j*` helpers, not inline tables.)

/// One of the eight JNI primitive wire types (`jboolean` … `jdouble`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum JniPrim {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
}

impl JniPrim {
    /// Classify a wire type by its last path segment ident; `None` for
    /// anything that is not one of the eight JNI scalars.
    pub(crate) fn from_wire(ty: &syn::Type) -> Option<Self> {
        let syn::Type::Path(tp) = ty else {
            return None;
        };
        let last = tp.path.segments.last()?;
        Some(match last.ident.to_string().as_str() {
            "jboolean" => Self::Boolean,
            "jbyte" => Self::Byte,
            "jchar" => Self::Char,
            "jshort" => Self::Short,
            "jint" => Self::Int,
            "jlong" => Self::Long,
            "jfloat" => Self::Float,
            "jdouble" => Self::Double,
            _ => return None,
        })
    }

    /// `<prim>Value()` unboxing accessor method on the boxed class.
    pub(crate) fn unbox_method(self) -> &'static str {
        match self {
            Self::Boolean => "booleanValue",
            Self::Byte => "byteValue",
            Self::Char => "charValue",
            Self::Short => "shortValue",
            Self::Int => "intValue",
            Self::Long => "longValue",
            Self::Float => "floatValue",
            Self::Double => "doubleValue",
        }
    }

    /// JVM signature of the `<prim>Value()` accessor.
    pub(crate) fn unbox_sig(self) -> &'static str {
        match self {
            Self::Boolean => "()Z",
            Self::Byte => "()B",
            Self::Char => "()C",
            Self::Short => "()S",
            Self::Int => "()I",
            Self::Long => "()J",
            Self::Float => "()F",
            Self::Double => "()D",
        }
    }

    /// `JValue` getter ident used to pull the scalar back out.
    pub(crate) fn unbox_getter(self) -> &'static str {
        match self {
            Self::Boolean => "z",
            Self::Byte => "b",
            Self::Char => "c",
            Self::Short => "s",
            Self::Int => "i",
            Self::Long => "j",
            Self::Float => "f",
            Self::Double => "d",
        }
    }
}

pub(crate) fn is_jni_primitive(ty: &syn::Type) -> bool {
    JniPrim::from_wire(ty).is_some()
}

pub(crate) fn jni_unbox_method(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_method()
}

pub(crate) fn jni_unbox_sig(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_sig()
}

pub(crate) fn jni_unbox_getter(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_getter()
}
