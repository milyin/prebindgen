//! Stable Rust scalar types. This classification does not prescribe a wire format.
use syn::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
}
impl Primitive {
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
