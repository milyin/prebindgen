//! JNI carriers and Kotlin projections for Rust scalar leaves.
use prebindgen_flat::Primitive;
use proc_macro2::TokenStream;
use quote::quote;
use Primitive::*;

pub fn rust_type(p: Primitive) -> TokenStream {
    match p {
        Bool => quote!(u8),
        I8 => quote!(i8),
        I16 => quote!(i16),
        U8 | U16 | I32 | Char => quote!(i32),
        U32 | U64 | I64 | Isize | Usize => quote!(i64),
        F32 => quote!(f32),
        F64 => quote!(f64),
        I128 | U128 => quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>),
    }
}
pub fn raw_kotlin(p: Primitive) -> &'static str {
    match p {
        Bool => "Boolean",
        I8 => "Byte",
        I16 => "Short",
        U8 | U16 | I32 | Char => "Int",
        U32 | U64 | I64 | Isize | Usize => "Long",
        F32 => "Float",
        F64 => "Double",
        I128 | U128 => "ByteArray?",
    }
}
pub fn kotlin(p: Primitive) -> &'static str {
    match p {
        U64 | Usize => "ULong",
        I128 | U128 => "java.math.BigInteger",
        _ => raw_kotlin(p),
    }
}
pub fn input(p: Primitive, name: &syn::Ident) -> (String, String) {
    match p {
        U64 | Usize => (kotlin(p).into(), format!("{name}.toLong()")),
        I128 | U128 => (format!("{}?", kotlin(p)), format!("{name}?.toByteArray()")),
        _ => (kotlin(p).into(), name.to_string()),
    }
}
pub fn decode(p: Primitive, w: &syn::Ident) -> TokenStream {
    let env = prebindgen_flat::template_parameter("environment");
    let ty = p.rust_type();
    match p {
        Bool => quote!(#w != 0),
        Char => {
            quote!(u32::try_from(#w).ok().and_then(char::from_u32).ok_or("invalid Unicode scalar")?)
        }
        U8 | U16 | U32 | Isize => {
            quote!(#ty::try_from(#w).map_err(|_| "primitive input out of range")?)
        }
        U64 => quote!(#w as u64),
        Usize => quote!(usize::try_from(#w as u64).map_err(|_| "usize input out of range")?),
        I128 => quote!(::prebindgen_jni_runtime::signed128(&mut #env, &#w)?),
        U128 => quote!(::prebindgen_jni_runtime::unsigned128(&mut #env, &#w)?),
        _ => quote!(#w),
    }
}
pub fn encode(p: Primitive, v: TokenStream) -> TokenStream {
    match p {
        Bool => quote!(u8::from(#v)),
        Char | U8 | U16 => quote!((#v) as i32),
        U32 | U64 | Isize | Usize => quote!((#v) as i64),
        I128 | U128 => quote!((#v).to_be_bytes()),
        _ => v,
    }
}
pub fn output_kotlin(p: Primitive, index: usize, result: &str) -> (String, String) {
    let decode = match p {
        U64 | Usize => format!("({result}[{index}] as Long).toULong()"),
        I128 => format!("java.math.BigInteger({result}[{index}] as ByteArray)"),
        U128 => format!("java.math.BigInteger(1, {result}[{index}] as ByteArray)"),
        _ => format!("{result}[{index}] as {}", kotlin(p)),
    };
    (kotlin(p).into(), decode)
}
pub fn output_java(p: Primitive, v: &TokenStream) -> TokenStream {
    let env = prebindgen_flat::template_parameter("environment");
    if matches!(p, I128 | U128) {
        quote!(::prebindgen_jni_runtime::bytes_object(&mut #env, &#v)?)
    } else {
        let (class, signature, value) = match p {
            Bool => ("Boolean", "Z", quote!(Bool(#v))),
            I8 => ("Byte", "B", quote!(Byte(#v))),
            I16 => ("Short", "S", quote!(Short(#v))),
            I32 | U8 | U16 | Char => ("Integer", "I", quote!(Int(#v))),
            I64 | U32 | U64 | Isize | Usize => ("Long", "J", quote!(Long(#v))),
            F32 => ("Float", "F", quote!(Float(#v))),
            F64 => ("Double", "D", quote!(Double(#v))),
            _ => unreachable!(),
        };
        let signature = format!("({signature})Ljava/lang/{class};");
        let class = format!("java/lang/{class}");
        quote!(#env.call_static_method(#class, "valueOf", #signature, &[::prebindgen_jni_runtime::jni::objects::JValue::#value]).and_then(|v| v.l()).map_err(|e| e.to_string())?)
    }
}
pub fn parameter_slots(p: Primitive) -> usize {
    if matches!(p, I64 | U32 | U64 | Isize | Usize | F64) {
        2
    } else {
        1
    }
}
