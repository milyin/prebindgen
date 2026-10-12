//! Stable Rust scalar types. This classification does not prescribe a wire format.
use syn::Type;

/// Rust scalar classification, independent of any target language carrier.
///
/// Unit is represented separately by an empty field recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    /// Boolean value.
    Bool,
    /// Unicode scalar value.
    Char,
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// Signed 128-bit integer.
    I128,
    /// Signed pointer-width integer.
    Isize,
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// Unsigned 128-bit integer.
    U128,
    /// Unsigned pointer-width integer.
    Usize,
    /// 32-bit floating-point value.
    F32,
    /// 64-bit floating-point value.
    F64,
}
impl Primitive {
    /// Recognize primitive spellings, including `core::primitive` and `std::primitive`.
    ///
    /// Returns `None` for references, aliases and unrelated qualified lookalikes.
    ///
    /// ```
    /// use prebindgen_flat::Primitive;
    /// assert_eq!(Primitive::from_type(&syn::parse_quote!(::core::primitive::u32)), Some(Primitive::U32));
    /// assert_eq!(Primitive::from_type(&syn::parse_quote!(&u32)), None);
    /// ```
    pub fn from_type(ty: &Type) -> Option<Self> {
        let Type::Path(p) = ty else { return None };
        if p.qself.is_some()
            || p.path
                .segments
                .iter()
                .any(|s| !matches!(s.arguments, syn::PathArguments::None))
        {
            return None;
        }
        let names: Vec<_> = p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        if names.len() != 1
            && !(names.len() == 3
                && matches!(names[0].as_str(), "core" | "std")
                && names[1] == "primitive")
        {
            return None;
        }
        Some(match names.last()?.as_str() {
            "bool" => Self::Bool,
            "char" => Self::Char,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "i128" => Self::I128,
            "isize" => Self::Isize,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "u128" => Self::U128,
            "usize" => Self::Usize,
            "f32" => Self::F32,
            "f64" => Self::F64,
            _ => return None,
        })
    }
    /// Return the canonical unqualified Rust spelling, such as `u32`.
    pub fn rust_type(self) -> Type {
        syn::parse_str(match self {
            Self::Bool => "bool",
            Self::Char => "char",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::Isize => "isize",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::Usize => "usize",
            Self::F32 => "f32",
            Self::F64 => "f64",
        })
        .unwrap()
    }
}
