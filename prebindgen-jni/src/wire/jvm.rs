//! Physical JVM types shared by method arguments, return values and array fields.
use super::*;

// Rust domains stay on JniWire: handles use Long, enums use Int, and 128-bit
// integers use Bytes. This table describes only their common JVM representation.
#[derive(Clone, Copy)]
pub(super) enum JvmType {
    Object,
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Text,
    Bytes,
    Rows,
}
struct JvmTypeInfo {
    descriptor: &'static str,
    kotlin_type: &'static str,
    kotlin_default: &'static str,
    row_reader: Option<&'static str>,
    carrier: Carrier,
}
enum Carrier {
    Primitive {
        rust_type: &'static str,
        member: &'static str,
        value_variant: &'static str,
        boxed_class: &'static str,
        slots: usize,
    },
    Reference {
        wrapper: &'static str,
    },
}
impl JvmType {
    fn info(self) -> &'static JvmTypeInfo {
        match self {
            Self::Boolean => &JvmTypeInfo {
                descriptor: "Z",
                kotlin_type: "Boolean",
                kotlin_default: "false",
                row_reader: Some("row_bool"),
                carrier: Carrier::Primitive {
                    rust_type: "u8",
                    member: "z",
                    value_variant: "Bool",
                    boxed_class: "Boolean",
                    slots: 1,
                },
            },
            Self::Byte => &JvmTypeInfo {
                descriptor: "B",
                kotlin_type: "Byte",
                kotlin_default: "0.toByte()",
                row_reader: Some("row_i8"),
                carrier: Carrier::Primitive {
                    rust_type: "i8",
                    member: "b",
                    value_variant: "Byte",
                    boxed_class: "Byte",
                    slots: 1,
                },
            },
            Self::Short => &JvmTypeInfo {
                descriptor: "S",
                kotlin_type: "Short",
                kotlin_default: "0.toShort()",
                row_reader: Some("row_i16"),
                carrier: Carrier::Primitive {
                    rust_type: "i16",
                    member: "s",
                    value_variant: "Short",
                    boxed_class: "Short",
                    slots: 1,
                },
            },
            Self::Int => &JvmTypeInfo {
                descriptor: "I",
                kotlin_type: "Int",
                kotlin_default: "0",
                row_reader: Some("row_i32"),
                carrier: Carrier::Primitive {
                    rust_type: "i32",
                    member: "i",
                    value_variant: "Int",
                    boxed_class: "Integer",
                    slots: 1,
                },
            },
            Self::Long => &JvmTypeInfo {
                descriptor: "J",
                kotlin_type: "Long",
                kotlin_default: "0L",
                row_reader: Some("row_i64"),
                carrier: Carrier::Primitive {
                    rust_type: "i64",
                    member: "j",
                    value_variant: "Long",
                    boxed_class: "Long",
                    slots: 2,
                },
            },
            Self::Float => &JvmTypeInfo {
                descriptor: "F",
                kotlin_type: "Float",
                kotlin_default: "0.0f",
                row_reader: Some("row_f32"),
                carrier: Carrier::Primitive {
                    rust_type: "f32",
                    member: "f",
                    value_variant: "Float",
                    boxed_class: "Float",
                    slots: 1,
                },
            },
            Self::Double => &JvmTypeInfo {
                descriptor: "D",
                kotlin_type: "Double",
                kotlin_default: "0.0",
                row_reader: Some("row_f64"),
                carrier: Carrier::Primitive {
                    rust_type: "f64",
                    member: "d",
                    value_variant: "Double",
                    boxed_class: "Double",
                    slots: 2,
                },
            },
            Self::Text => &JvmTypeInfo {
                descriptor: "Ljava/lang/String;",
                kotlin_type: "String?",
                kotlin_default: "null",
                row_reader: Some("row_text"),
                carrier: Carrier::Reference { wrapper: "JString" },
            },
            Self::Bytes => &JvmTypeInfo {
                descriptor: "[B",
                kotlin_type: "ByteArray?",
                kotlin_default: "null",
                row_reader: Some("row_bytes"),
                carrier: Carrier::Reference {
                    wrapper: "JByteArray",
                },
            },
            Self::Rows => &JvmTypeInfo {
                descriptor: "[Ljava/lang/Object;",
                kotlin_type: "Array<Any?>?",
                kotlin_default: "null",
                row_reader: Some("row_rows"),
                carrier: Carrier::Reference {
                    wrapper: "JObjectArray",
                },
            },
            Self::Object => &JvmTypeInfo {
                descriptor: "Ljava/lang/Object;",
                kotlin_type: "Any?",
                kotlin_default: "null",
                row_reader: None,
                carrier: Carrier::Reference { wrapper: "JObject" },
            },
        }
    }
    pub(super) fn descriptor(self) -> &'static str {
        self.info().descriptor
    }
    pub(super) fn kotlin_type(self) -> &'static str {
        self.info().kotlin_type
    }
    pub(super) fn kotlin_default(self) -> &'static str {
        self.info().kotlin_default
    }
    pub(super) fn is_reference(self) -> bool {
        matches!(self.info().carrier, Carrier::Reference { .. })
    }
    pub(super) fn parameter_slots(self) -> usize {
        match self.info().carrier {
            Carrier::Primitive { slots, .. } => slots,
            Carrier::Reference { .. } => 1,
        }
    }
    pub(super) fn value_member(self) -> &'static str {
        match self.info().carrier {
            Carrier::Primitive { member, .. } => member,
            Carrier::Reference { .. } => "l",
        }
    }
    pub(super) fn rust_type(self) -> syn::Type {
        match self.info().carrier {
            Carrier::Primitive { rust_type, .. } => {
                syn::parse_str(rust_type).expect("JVM primitive carrier")
            }
            Carrier::Reference { wrapper } => {
                let wrapper = format_ident!("{wrapper}");
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::#wrapper<'local>)
            }
        }
    }
    pub(super) fn return_type(self) -> TokenStream {
        match self.info().carrier {
            Carrier::Primitive { .. } => {
                let variant = format_ident!("{}", self.kotlin_type());
                quote!(::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(::prebindgen_jni_runtime::jni::signature::Primitive::#variant))
            }
            Carrier::Reference { .. } => {
                quote!(::prebindgen_jni_runtime::jni::signature::ReturnType::Object)
            }
        }
    }
    fn normalize_boolean(self, read: TokenStream) -> TokenStream {
        if matches!(self, Self::Boolean) {
            quote!(#read.map(u8::from))
        } else {
            read
        }
    }
    pub(super) fn read_array_element(self, array: &TokenStream, index: usize) -> TokenStream {
        let env = prebindgen_conversions::template_parameter("environment");
        let Some(reader) = self.info().row_reader else {
            return quote!(#env.get_object_array_element(&(#array), #index as i32).map_err(|e|e.to_string()));
        };
        let reader = format_ident!("{reader}");
        self.normalize_boolean(
            quote!(::prebindgen_jni_runtime::#reader(&mut #env, &(#array), #index)),
        )
    }
    pub(super) fn read_return(self, value: &syn::Ident) -> TokenStream {
        let member = format_ident!("{}", self.value_member());
        let read = quote!(#value.#member().map_err(|e| e.to_string()));
        let read = match self.info().carrier {
            Carrier::Primitive { .. } => self.normalize_boolean(read),
            Carrier::Reference { wrapper } => {
                let wrapper = format_ident!("{wrapper}");
                quote!(#read.map(::prebindgen_jni_runtime::jni::objects::#wrapper::from))
            }
        };
        quote!(#read?)
    }
    pub(super) fn boxed_output(self, value: &TokenStream) -> Option<TokenStream> {
        let Carrier::Primitive {
            value_variant,
            boxed_class,
            ..
        } = self.info().carrier
        else {
            return None;
        };
        let env = prebindgen_conversions::template_parameter("environment");
        let signature = format!("({})Ljava/lang/{boxed_class};", self.descriptor());
        let class = format!("java/lang/{boxed_class}");
        let variant = format_ident!("{value_variant}");
        Some(
            quote!(#env.call_static_method(#class,"valueOf",#signature,&[::prebindgen_jni_runtime::jni::objects::JValue::#variant(#value)]).and_then(|v|v.l()).map_err(|e|e.to_string())?),
        )
    }
}

/// Initial values for Kotlin locals whose types use JVM primitive names.
/// Unsigned Kotlin values and Char have separate public-type conversions.
pub(crate) fn primitive_local_default(kotlin_type: &str) -> Option<&'static str> {
    use JvmType::*;
    [Boolean, Byte, Short, Int, Long, Float, Double]
        .into_iter()
        .find(|ty| ty.kotlin_type() == kotlin_type)
        .map(JvmType::kotlin_default)
}
