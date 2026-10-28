//! JNI representation and code generation for individual wire values.
use super::jvm::JvmType;
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
            Self::Callback { .. } => JvmType::Object,
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
        self.jvm_type().parameter_slots()
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
        self.jvm_type().kotlin_default()
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
        self.jvm_type().read_array_element(array, index)
    }
    pub(crate) fn callback_return_type(&self) -> TokenStream {
        self.jvm_type().return_type()
    }
    pub(crate) fn read_callback_return(&self, value: &syn::Ident) -> TokenStream {
        self.jvm_type().read_return(value)
    }

    /// Generate a Java object expression from a prepared output, boxing primitives.
    ///
    /// This accepts the representation in `OutputPlan::values`, not a native
    /// input: text/bytes are prepared nullable JVM objects, i128/u128 are encoded bytes,
    /// handles are allocation pointers, and rows are optional JNI arrays.
    /// The expression can fail through `?`, uses the `environment` template
    /// parameter, and returns a local reference owned by the caller's frame.
    /// Text/byte payloads are consumed; each prepared object may be delivered
    /// once. Other wire kinds retain their declared representation.
    /// It does not commit any ownership transfer.
    pub fn output_object(&self, value: &TokenStream) -> TokenStream {
        let env = prebindgen_conversions::template_parameter("environment");
        if let Some(boxed) = self.jvm_type().boxed_output(&self.output_scalar(value)) {
            return boxed;
        }
        match self.jvm_type() {
            JvmType::Text => quote!(#value),
            JvmType::Bytes if matches!(self, Self::Scalar { .. }) => {
                quote!(::prebindgen_jni_runtime::bytes_object(&mut #env, &(#value))?)
            }
            JvmType::Bytes => quote!(#value),
            JvmType::Object => quote!(#env.new_local_ref(&(#value)).map_err(|e|e.to_string())?),
            JvmType::Rows => {
                quote!(match (#value).as_ref(){Some(value)=>#env.new_local_ref(value).map_err(|e|e.to_string())?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()})
            }
            _ => unreachable!("JVM primitives use their declared boxing conversion"),
        }
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
    /// through the invocation. Text/byte prepared payloads are consumed once.
    /// The caller supplies `environment`, validates the
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
        let member = format_ident!("{}", self.jvm_type().value_member());
        let (statements, value) = if self.is_reference() {
            let object = self.output_object(value);
            (quote!(let #local=#object;), quote!(#local.as_raw()))
        } else {
            (TokenStream::new(), self.output_scalar(value))
        };
        JniArgument {
            statements,
            value: quote!(::prebindgen_jni_runtime::jni::sys::jvalue { #member:#value }),
        }
    }
}
impl WireKind for JniWire {
    fn rust_type(&self) -> syn::Type {
        self.jvm_type().rust_type()
    }
}
