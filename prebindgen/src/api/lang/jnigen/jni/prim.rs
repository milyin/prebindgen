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

    /// Classify from the Kotlin builtin type name (`"Int"` → `Int`); `None`
    /// for non-primitive Kotlin types (`String`, `Unit`, classes, …).
    pub(crate) fn from_kotlin_name(name: &str) -> Option<Self> {
        Some(match name {
            "Boolean" => Self::Boolean,
            "Byte" => Self::Byte,
            "Char" => Self::Char,
            "Short" => Self::Short,
            "Int" => Self::Int,
            "Long" => Self::Long,
            "Float" => Self::Float,
            "Double" => Self::Double,
            _ => return None,
        })
    }

    /// Reverse lookup from a JVM field descriptor (`"J"` → `Long`).
    pub(crate) fn from_descriptor(sig: &str) -> Option<Self> {
        Some(match sig {
            "Z" => Self::Boolean,
            "B" => Self::Byte,
            "C" => Self::Char,
            "S" => Self::Short,
            "I" => Self::Int,
            "J" => Self::Long,
            "F" => Self::Float,
            "D" => Self::Double,
            _ => return None,
        })
    }

    /// The wire ident (`jboolean` … `jdouble`) as written in generated Rust.
    pub(crate) fn wire_name(self) -> &'static str {
        match self {
            Self::Boolean => "jboolean",
            Self::Byte => "jbyte",
            Self::Char => "jchar",
            Self::Short => "jshort",
            Self::Int => "jint",
            Self::Long => "jlong",
            Self::Float => "jfloat",
            Self::Double => "jdouble",
        }
    }

    /// JVM field-descriptor chunk (`"Z"` … `"D"`).
    pub(crate) fn descriptor(self) -> &'static str {
        match self {
            Self::Boolean => "Z",
            Self::Byte => "B",
            Self::Char => "C",
            Self::Short => "S",
            Self::Int => "I",
            Self::Long => "J",
            Self::Float => "F",
            Self::Double => "D",
        }
    }

    /// JVM descriptor of the `java.lang.*` box class — the slot an
    /// `Option`-boxed primitive occupies in a method signature.
    pub(crate) fn box_descriptor(self) -> &'static str {
        match self {
            Self::Boolean => "Ljava/lang/Boolean;",
            Self::Byte => "Ljava/lang/Byte;",
            Self::Char => "Ljava/lang/Character;",
            Self::Short => "Ljava/lang/Short;",
            Self::Int => "Ljava/lang/Integer;",
            Self::Long => "Ljava/lang/Long;",
            Self::Float => "Ljava/lang/Float;",
            Self::Double => "Ljava/lang/Double;",
        }
    }

    /// The Kotlin zero/default literal for the primitive.
    pub(crate) fn kotlin_zero(self) -> &'static str {
        match self {
            Self::Boolean => "false",
            Self::Byte | Self::Short | Self::Int => "0",
            Self::Char => "'\\u0000'",
            Self::Long => "0L",
            Self::Float => "0.0f",
            Self::Double => "0.0",
        }
    }

    /// The non-null Kotlin type name carrying this primitive wire (the type an
    /// `external fun` value-leaf param declares). Distinct from a Rust enum's
    /// `kotlin_name` (`Priority`) — an `Option<enum>` value leaf crosses as the
    /// raw `Int` discriminant, so the extern must name `Int`, not the enum.
    pub(crate) fn kotlin_type(self) -> &'static str {
        match self {
            Self::Boolean => "Boolean",
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

pub(crate) fn jni_unbox_method(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_method()
}

pub(crate) fn jni_unbox_sig(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_sig()
}

pub(crate) fn jni_unbox_getter(wire: &syn::Type) -> &'static str {
    JniPrim::from_wire(wire).unwrap().unbox_getter()
}
