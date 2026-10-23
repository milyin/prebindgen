//! JNI representation and code generation for individual wire values.
use super::*;

/// Code preparing one JNI method argument from a prepared output value.
///
/// Emit `statements` before using `value`, in the same local-reference frame.
/// The caller owns that frame and must keep its references alive until the
/// method invocation finishes. Neither fragment commits an ownership transfer.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct JniArgument {
    /// Fallible preparation, including any Java object allocation.
    pub statements: TokenStream,
    /// A `jni::sys::jvalue` expression referring to the prepared value.
    pub value: TokenStream,
}

// Derived from JniWire, never stored alongside it. This is only the JVM
// representation: enums and selectors are Int, handles are Long, and Rust
// i128/u128 use ByteArray. Rust domains and niche state remain on JniWire.
#[derive(Clone, Copy)]
enum JvmType {
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
impl JvmType {
    fn descriptor(self) -> &'static str {
        match self {
            Self::Boolean => "Z",
            Self::Byte => "B",
            Self::Short => "S",
            Self::Int => "I",
            Self::Long => "J",
            Self::Float => "F",
            Self::Double => "D",
            Self::Text => "Ljava/lang/String;",
            Self::Bytes => "[B",
            Self::Rows => "[Ljava/lang/Object;",
        }
    }
    fn kotlin_type(self) -> &'static str {
        match self {
            Self::Boolean => "Boolean",
            Self::Byte => "Byte",
            Self::Short => "Short",
            Self::Int => "Int",
            Self::Long => "Long",
            Self::Float => "Float",
            Self::Double => "Double",
            Self::Text => "String?",
            Self::Bytes => "ByteArray?",
            Self::Rows => "Array<Any?>?",
        }
    }
    fn is_reference(self) -> bool {
        matches!(self, Self::Text | Self::Bytes | Self::Rows)
    }
    fn reader(self) -> &'static str {
        match self {
            Self::Boolean => "row_bool",
            Self::Byte => "row_i8",
            Self::Short => "row_i16",
            Self::Int => "row_i32",
            Self::Long => "row_i64",
            Self::Float => "row_f32",
            Self::Double => "row_f64",
            Self::Text => "row_text",
            Self::Bytes => "row_bytes",
            Self::Rows => "row_rows",
        }
    }
}
impl JniWire {
    fn jvm_type(&self) -> JvmType {
        match self {
            Self::Scalar { primitive, .. } => match primitive {
                Primitive::Bool => JvmType::Boolean,
                Primitive::I8 => JvmType::Byte,
                Primitive::I16 => JvmType::Short,
                Primitive::U8 | Primitive::U16 | Primitive::I32 | Primitive::Char => JvmType::Int,
                Primitive::U32
                | Primitive::U64
                | Primitive::I64
                | Primitive::Usize
                | Primitive::Isize => JvmType::Long,
                Primitive::F32 => JvmType::Float,
                Primitive::F64 => JvmType::Double,
                Primitive::I128 | Primitive::U128 => JvmType::Bytes,
            },
            Self::Handle { .. } => JvmType::Long,
            Self::Selector { .. } | Self::Enum { .. } => JvmType::Int,
            Self::Text { .. } => JvmType::Text,
            Self::Bytes { .. } => JvmType::Bytes,
            Self::Rows { .. } => JvmType::Rows,
        }
    }
    /// JVM field descriptor used in method signatures, independent of Kotlin spelling.
    pub fn jvm_descriptor(&self) -> &'static str {
        self.jvm_type().descriptor()
    }
    /// Kotlin type used by the native declaration, before typed projections.
    pub fn kotlin_type(&self) -> &'static str {
        self.jvm_type().kotlin_type()
    }
    /// JVM argument slots occupied by this value (`long` and `double` occupy two).
    pub fn parameter_slots(&self) -> usize {
        if matches!(self.jvm_type(), JvmType::Long | JvmType::Double) {
            2
        } else {
            1
        }
    }
    /// Whether the native input is a JNI reference rather than a copyable scalar.
    ///
    /// Handles are integers, so this returns false for them. Rust i128/u128
    /// arrive as byte-array references, so this returns true for those scalars.
    pub fn is_reference(&self) -> bool {
        self.jvm_type().is_reference()
    }
    /// Kotlin placeholder for an inactive wire, without selecting an option niche.
    pub fn kotlin_default(&self) -> &'static str {
        match self.jvm_type() {
            JvmType::Boolean => "false",
            JvmType::Byte => "0.toByte()",
            JvmType::Short => "0.toShort()",
            JvmType::Int => "0",
            JvmType::Long => "0L",
            JvmType::Float => "0.0f",
            JvmType::Double => "0.0",
            JvmType::Text | JvmType::Bytes | JvmType::Rows => "null",
        }
    }
    pub(crate) fn kotlin_integer(&self, value: i64) -> String {
        if matches!(self.jvm_type(), JvmType::Long) {
            format!("{value}L")
        } else {
            value.to_string()
        }
    }
    /// Generate an expression reading a checked, boxed field of a Java `Object[]`.
    ///
    /// The expression returns `Result<rust_type()>`, without applying Rust-domain
    /// validation or acquiring handles. Boolean inputs are normalized to `u8`.
    /// Reference fields preserve Java null for optional/inactive values. The
    /// caller supplies the `environment` template parameter and owns local refs.
    pub fn read_array_element(&self, array: &TokenStream, index: usize) -> TokenStream {
        let env = prebindgen_conversions::template_parameter("environment");
        let reader = format_ident!("{}", self.jvm_type().reader());
        let convert = matches!(self.jvm_type(), JvmType::Boolean).then(|| quote!(.map(u8::from)));
        quote!(::prebindgen_jni_runtime::#reader(&mut #env, &(#array), #index)#convert)
    }
    /// Generate a Java object expression from a prepared output, boxing primitives.
    ///
    /// This accepts the representation in `OutputPlan::values`, not a native
    /// input: text/bytes are optional Rust payloads, i128/u128 are encoded bytes,
    /// handles are allocation pointers, and rows are optional JNI arrays.
    /// The expression can fail through `?`, uses the `environment` template
    /// parameter, and returns a local reference owned by the caller's frame.
    /// It does not commit any ownership transfer.
    pub fn output_object(&self, value: &TokenStream) -> TokenStream {
        let env = prebindgen_conversions::template_parameter("environment");
        let (class, variant) = match self.jvm_type() {
            JvmType::Boolean => ("Boolean", "Bool"),
            JvmType::Byte => ("Byte", "Byte"),
            JvmType::Short => ("Short", "Short"),
            JvmType::Int => ("Integer", "Int"),
            JvmType::Long => ("Long", "Long"),
            JvmType::Float => ("Float", "Float"),
            JvmType::Double => ("Double", "Double"),
            JvmType::Text => {
                return quote!(match (#value).as_ref(){Some(value)=>::prebindgen_jni_runtime::text_object(&mut #env,value)?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()})
            }
            JvmType::Bytes if matches!(self, Self::Scalar { .. }) => {
                return quote!(::prebindgen_jni_runtime::bytes_object(&mut #env, &(#value))?)
            }
            JvmType::Bytes => {
                return quote!(match (#value).as_ref(){Some(value)=>::prebindgen_jni_runtime::bytes_object(&mut #env,value)?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()})
            }
            JvmType::Rows => {
                return quote!(match (#value).as_ref(){Some(value)=>#env.new_local_ref(value).map_err(|e|e.to_string())?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()})
            }
        };
        let signature = format!("({})Ljava/lang/{class};", self.jvm_descriptor());
        let class = format!("java/lang/{class}");
        let variant = format_ident!("{variant}");
        let value = self.output_scalar(value);
        quote!(#env.call_static_method(#class,"valueOf",#signature,&[::prebindgen_jni_runtime::jni::objects::JValue::#variant(#value)]).and_then(|v|v.l()).map_err(|e|e.to_string())?)
    }
    fn output_scalar(&self, value: &TokenStream) -> TokenStream {
        if matches!(self, Self::Handle { .. }) {
            quote!((#value) as i64)
        } else {
            value.clone()
        }
    }
    /// Generate one JNI method argument from the same prepared output as
    /// [`Self::output_object`], leaving JVM primitives unboxed.
    ///
    /// `local` must be a fresh identifier for this argument's Java object.
    /// Emit the returned statements before its value and keep references alive
    /// through the invocation. The caller supplies `environment`, validates the
    /// method signature, and commits ownership only after successful delivery.
    /// A custom generator can render both fragments with its [`JniContext`]:
    ///
    /// ```
    /// use prebindgen_conversions::TemplateContext;
    /// use prebindgen_jni::{JniContext, JniWire};
    /// use quote::quote;
    /// let context = JniContext {
    ///     environment: syn::parse_quote!(env),
    ///     acknowledgement: syn::parse_quote!(accepted),
    ///     domain_output: syn::parse_quote!(domain_output),
    ///     is_domain_error: false,
    ///     output_committed: quote!(),
    /// };
    /// let argument = JniWire::text().output_argument(
    ///     &quote!(prepared_text), &syn::parse_quote!(java_text));
    /// let statements = context.render(&argument.statements)?;
    /// let value = context.render(&argument.value)?;
    /// // Emit statements, invoke with value, then release the local frame.
    /// # assert!(!statements.to_string().contains("__prebindgen_emit"));
    /// # assert!(!value.to_string().contains("__prebindgen_emit"));
    /// # Ok::<(), String>(())
    /// ```
    pub fn output_argument(&self, value: &TokenStream, local: &syn::Ident) -> JniArgument {
        let member = match self.jvm_type() {
            JvmType::Boolean => "z",
            JvmType::Byte => "b",
            JvmType::Short => "s",
            JvmType::Int => "i",
            JvmType::Long => "j",
            JvmType::Float => "f",
            JvmType::Double => "d",
            JvmType::Text | JvmType::Bytes | JvmType::Rows => {
                let object = self.output_object(value);
                return JniArgument {
                    statements: quote!(let #local=#object;),
                    value: quote!(::prebindgen_jni_runtime::jni::sys::jvalue { l:#local.as_raw() }),
                };
            }
        };
        let member = format_ident!("{member}");
        let value = self.output_scalar(value);
        JniArgument {
            statements: TokenStream::new(),
            value: quote!(::prebindgen_jni_runtime::jni::sys::jvalue { #member:#value }),
        }
    }
}
impl WireKind for JniWire {
    fn rust_type(&self) -> syn::Type {
        match self.jvm_type() {
            JvmType::Boolean => syn::parse_quote!(u8),
            JvmType::Byte => syn::parse_quote!(i8),
            JvmType::Short => syn::parse_quote!(i16),
            JvmType::Int => syn::parse_quote!(i32),
            JvmType::Long => syn::parse_quote!(i64),
            JvmType::Float => syn::parse_quote!(f32),
            JvmType::Double => syn::parse_quote!(f64),
            JvmType::Text => {
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::JString<'local>)
            }
            JvmType::Bytes => {
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>)
            }
            JvmType::Rows => {
                syn::parse_quote!(::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>)
            }
        }
    }
}
