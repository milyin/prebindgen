//! JNI primitive (un)boxing lookup tables.
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.
//!
//! The eight JNI scalar wire types ([`JniPrim`]) and their boxed-`java.lang.*`
//! counterparts are described by one enum + per-aspect accessors, replacing
//! the six parallel `match jni_prim_name(..)` tables that used to repeat the
//! same 8-way classification. The free `jni_box_*` / `jni_unbox_*` /
//! `is_jni_primitive` functions are kept as thin shims so call sites are
//! unchanged.

use super::*;

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

    /// Boxed `java.lang.*` class (JVM internal name).
    pub(crate) fn box_class(self) -> &'static str {
        match self {
            Self::Boolean => "java/lang/Boolean",
            Self::Byte => "java/lang/Byte",
            Self::Char => "java/lang/Character",
            Self::Short => "java/lang/Short",
            Self::Int => "java/lang/Integer",
            Self::Long => "java/lang/Long",
            Self::Float => "java/lang/Float",
            Self::Double => "java/lang/Double",
        }
    }

    /// `valueOf(<prim>)` JVM signature for the boxed class.
    pub(crate) fn box_sig(self) -> &'static str {
        match self {
            Self::Boolean => "(Z)Ljava/lang/Boolean;",
            Self::Byte => "(B)Ljava/lang/Byte;",
            Self::Char => "(C)Ljava/lang/Character;",
            Self::Short => "(S)Ljava/lang/Short;",
            Self::Int => "(I)Ljava/lang/Integer;",
            Self::Long => "(J)Ljava/lang/Long;",
            Self::Float => "(F)Ljava/lang/Float;",
            Self::Double => "(D)Ljava/lang/Double;",
        }
    }

    /// `jni::objects::JValue` variant name for the primitive.
    pub(crate) fn box_variant(self) -> &'static str {
        match self {
            Self::Boolean => "Bool",
            Self::Byte => "Byte",
            Self::Char => "Char",
            Self::Short => "Short",
            Self::Int => "Int",
            Self::Long => "Long",
            Self::Float => "Float",
            Self::Double => "Double",
        }
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

pub(crate) fn jni_box_class(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire)
        .unwrap_or_else(|| panic!("not a JNI primitive: {}", wire.to_token_stream()))
        .box_class()
}

pub(crate) fn jni_box_sig(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().box_sig()
}

pub(crate) fn jni_box_variant(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().box_variant()
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
