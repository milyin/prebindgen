//! Leaves: the JNI-level slots a value crosses on.
//!
//! Every value crosses as a flat list of leaves — JNI primitives, strings,
//! primitive arrays, object arrays, boxed primitives, callback objects. A
//! leaf knows its Kotlin raw type, its JVM descriptor and its Rust wire type,
//! which is all three sides of the boundary need to agree on.

use proc_macro2::TokenStream;
use quote::quote;

/// A JVM primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Prim {
    Z,
    B,
    C,
    S,
    I,
    J,
    F,
    D,
}

impl Prim {
    pub(crate) fn desc(self) -> &'static str {
        match self {
            Prim::Z => "Z",
            Prim::B => "B",
            Prim::C => "C",
            Prim::S => "S",
            Prim::I => "I",
            Prim::J => "J",
            Prim::F => "F",
            Prim::D => "D",
        }
    }

    pub(crate) fn kt(self) -> &'static str {
        match self {
            Prim::Z => "Boolean",
            Prim::B => "Byte",
            Prim::C => "Char",
            Prim::S => "Short",
            Prim::I => "Int",
            Prim::J => "Long",
            Prim::F => "Float",
            Prim::D => "Double",
        }
    }

    pub(crate) fn kt_default(self) -> &'static str {
        match self {
            Prim::Z => "false",
            Prim::B => "0.toByte()",
            Prim::C => "'\\u0000'",
            Prim::S => "0.toShort()",
            Prim::I => "0",
            Prim::J => "0L",
            Prim::F => "0f",
            Prim::D => "0.0",
        }
    }

    pub(crate) fn rs(self) -> TokenStream {
        match self {
            Prim::Z => quote!(::prebindgen_jni_runtime::jni::sys::jboolean),
            Prim::B => quote!(::prebindgen_jni_runtime::jni::sys::jbyte),
            Prim::C => quote!(::prebindgen_jni_runtime::jni::sys::jchar),
            Prim::S => quote!(::prebindgen_jni_runtime::jni::sys::jshort),
            Prim::I => quote!(::prebindgen_jni_runtime::jni::sys::jint),
            Prim::J => quote!(::prebindgen_jni_runtime::jni::sys::jlong),
            Prim::F => quote!(::prebindgen_jni_runtime::jni::sys::jfloat),
            Prim::D => quote!(::prebindgen_jni_runtime::jni::sys::jdouble),
        }
    }

    pub(crate) fn rs_default(self) -> TokenStream {
        match self {
            Prim::F => quote!(0.0f32),
            Prim::D => quote!(0.0f64),
            _ => quote!(0),
        }
    }

    /// The `jvalue` union field.
    pub(crate) fn jvalue_field(self) -> syn::Ident {
        let s = match self {
            Prim::Z => "z",
            Prim::B => "b",
            Prim::C => "c",
            Prim::S => "s",
            Prim::I => "i",
            Prim::J => "j",
            Prim::F => "f",
            Prim::D => "d",
        };
        syn::Ident::new(s, proc_macro2::Span::call_site())
    }

    /// Kotlin primitive array type.
    pub(crate) fn kt_array(self) -> String {
        format!("{}Array", self.kt())
    }

    /// Runtime helper names reading / writing an array of this primitive.
    pub(crate) fn array_helpers(self) -> (&'static str, &'static str) {
        match self {
            Prim::Z => ("read_booleans", "write_booleans"),
            Prim::B => ("read_bytes", "write_bytes"),
            Prim::C => ("read_chars", "write_chars"),
            Prim::S => ("read_shorts", "write_shorts"),
            Prim::I => ("read_ints", "write_ints"),
            Prim::J => ("read_longs", "write_longs"),
            Prim::F => ("read_floats", "write_floats"),
            Prim::D => ("read_doubles", "write_doubles"),
        }
    }

    /// The runtime helper boxing this primitive (`java.lang.Long` …).
    pub(crate) fn box_helper(self) -> &'static str {
        match self {
            Prim::Z => "box_jboolean",
            Prim::B => "box_jbyte",
            Prim::C => "box_jchar",
            Prim::S => "box_jshort",
            Prim::I => "box_jint",
            Prim::J => "box_jlong",
            Prim::F => "box_jfloat",
            Prim::D => "box_jdouble",
        }
    }

    pub(crate) fn boxed_class(self) -> &'static str {
        match self {
            Prim::Z => "java/lang/Boolean",
            Prim::B => "java/lang/Byte",
            Prim::C => "java/lang/Character",
            Prim::S => "java/lang/Short",
            Prim::I => "java/lang/Integer",
            Prim::J => "java/lang/Long",
            Prim::F => "java/lang/Float",
            Prim::D => "java/lang/Double",
        }
    }

    /// Slots this primitive takes in a JVM method's argument list.
    pub(crate) fn slots(self) -> usize {
        match self {
            Prim::J | Prim::D => 2,
            _ => 1,
        }
    }
}

/// What a leaf is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LeafTy {
    Prim(Prim),
    String,
    PrimArray(Prim),
    ObjArray,
    /// A boxed primitive, always nullable.
    Boxed(Prim),
    /// A generated callback interface, by Kotlin FQN.
    Callback(String),
}

/// One leaf, named by its path under the value it belongs to.
#[derive(Clone, Debug)]
pub(crate) struct Leaf {
    /// `_`-joined path; empty for the value itself.
    pub name: String,
    pub ty: LeafTy,
    /// For an object leaf: whether Kotlin may hand over `null`.
    pub nullable: bool,
}

impl Leaf {
    pub(crate) fn new(ty: LeafTy) -> Self {
        let nullable = matches!(ty, LeafTy::Boxed(_));
        Self {
            name: String::new(),
            ty,
            nullable,
        }
    }

    pub(crate) fn is_obj(&self) -> bool {
        !matches!(self.ty, LeafTy::Prim(_))
    }

    pub(crate) fn prim(&self) -> Option<Prim> {
        match self.ty {
            LeafTy::Prim(p) => Some(p),
            _ => None,
        }
    }

    /// Prefix the path.
    pub(crate) fn under(mut self, prefix: &str) -> Self {
        self.name = join(prefix, &self.name);
        self
    }

    pub(crate) fn kt_raw(&self) -> String {
        let base = match &self.ty {
            LeafTy::Prim(p) => return p.kt().to_string(),
            LeafTy::String => "String".to_string(),
            LeafTy::PrimArray(p) => p.kt_array(),
            LeafTy::ObjArray => "Array<Any?>".to_string(),
            LeafTy::Boxed(p) => p.kt().to_string(),
            LeafTy::Callback(fqn) => fqn.clone(),
        };
        if self.nullable {
            format!("{base}?")
        } else {
            base
        }
    }

    pub(crate) fn desc(&self) -> String {
        match &self.ty {
            LeafTy::Prim(p) => p.desc().to_string(),
            LeafTy::String => "Ljava/lang/String;".to_string(),
            LeafTy::PrimArray(p) => format!("[{}", p.desc()),
            LeafTy::ObjArray => "[Ljava/lang/Object;".to_string(),
            LeafTy::Boxed(p) => format!("L{};", p.boxed_class()),
            LeafTy::Callback(fqn) => format!("L{};", fqn.replace('.', "/")),
        }
    }

    pub(crate) fn rs(&self) -> TokenStream {
        match &self.ty {
            LeafTy::Prim(p) => p.rs(),
            _ => quote!(::prebindgen_jni_runtime::jni::objects::JObject<'a>),
        }
    }

    pub(crate) fn rs_default(&self) -> TokenStream {
        match &self.ty {
            LeafTy::Prim(p) => p.rs_default(),
            _ => quote!(::prebindgen_jni_runtime::jni::objects::JObject::null()),
        }
    }

    /// The `jvalue` carrying `v`.
    pub(crate) fn jvalue(&self, v: &TokenStream) -> TokenStream {
        match &self.ty {
            LeafTy::Prim(p) => {
                let f = p.jvalue_field();
                quote!(::prebindgen_jni_runtime::jni::sys::jvalue { #f: #v })
            }
            _ => quote!(::prebindgen_jni_runtime::jni::sys::jvalue { l: #v.as_raw() }),
        }
    }

    pub(crate) fn slots(&self) -> usize {
        match &self.ty {
            LeafTy::Prim(p) => p.slots(),
            _ => 1,
        }
    }

    /// This leaf as a column of a sequence: a primitive becomes a primitive
    /// array, anything else an object array.
    pub(crate) fn column(&self) -> Leaf {
        Leaf {
            name: self.name.clone(),
            ty: match self.ty {
                LeafTy::Prim(p) => LeafTy::PrimArray(p),
                _ => LeafTy::ObjArray,
            },
            nullable: false,
        }
    }
}

/// A leaf is the JNI adapter's wire type: what the Rust slot is, and what
/// Kotlin makes of it.
impl prebindgen_tools::WireType for Leaf {
    fn rust(&self) -> TokenStream {
        self.rs()
    }

    fn placeholder(&self) -> TokenStream {
        self.rs_default()
    }
}

/// `a_b`, or whichever part is non-empty.
pub(crate) fn join(a: &str, b: &str) -> String {
    match (a.is_empty(), b.is_empty()) {
        (true, _) => b.to_string(),
        (_, true) => a.to_string(),
        _ => format!("{a}_{b}"),
    }
}

/// The JVM method descriptor for these argument leaves and a return.
pub(crate) fn method_desc(leaves: &[Leaf], ret: &str) -> String {
    let args: String = leaves.iter().map(Leaf::desc).collect();
    format!("({args}){ret}")
}
