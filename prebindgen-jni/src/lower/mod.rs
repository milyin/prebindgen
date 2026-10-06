//! How a value crosses the JNI boundary, in both languages and both
//! directions.
//!
//! Every value crosses as a flat list of [`Leaf`]s — JNI primitives,
//! strings, primitive arrays, object arrays — and generated Rust never reads
//! a Kotlin object's fields. One layer of a type is read with
//! [`Plan::shape`]; everything here recurses on it, so the leaf list and the
//! four conversions cannot drift apart:
//!
//! | | into Rust | out of Rust |
//! |---|---|---|
//! | leaves | [`Plan::leaves`] (`Dir::In`) | [`Plan::leaves`] (`Dir::Out`) |
//! | Kotlin | [`Plan::kt_encode`]: value → leaf expressions | [`Plan::kt_decode`]: leaf expressions → value |
//! | Rust | [`Plan::rs_decode`]: wires → value | [`Plan::rs_encode`]: value → wires |
//!
//! A data class is the concatenation of its fields; an optional value is a
//! presence flag beside its inner leaves (a single object leaf is simply
//! nullable, and a single primitive leaving Rust is boxed); a sum is a tag
//! plus every alternative's fields; a sequence is a count plus one array per
//! element leaf.
//!
//! [`deliver`] hands a value to Kotlin as a list of parameters (a builder's,
//! a callback's, an error handler's); [`select`] builds a parameter from a
//! choice of constructors.

pub(crate) mod deliver;
pub(crate) mod kotlin;
pub(crate) mod leaf;
pub(crate) mod rust;
pub(crate) mod select;

use prebindgen_flat::flat::{Alternative, Field, ScalarKind, TypeKind, TypeRef};
use prebindgen_tools::{names, shape, Access, SequenceKind, Shape, TextKind};

use self::leaf::{join, Leaf, LeafTy, Prim};
use crate::plan::{err, Class, ClassKind, Conv, Plan, Res, Setting};

fn bare_declared_name(ty: &TypeRef) -> Option<&str> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(&id.name),
        TypeKind::String => Some("String"),
        _ => None,
    }
}

fn declared_name(ty: &TypeRef) -> Option<&str> {
    let core = match ty.kind() {
        TypeKind::Ref { inner, .. } => inner.as_ref(),
        _ => ty,
    };
    bare_declared_name(core)
}

/// A parameter's input, plus how the wrapper passes the bound value to the
/// callee when not as is — `&s` for a borrow of a decoded local.
pub(crate) struct Param {
    pub input: prebindgen_tools::Input<Leaf>,
    pub pass: Option<proc_macro2::TokenStream>,
}

/// Which way a value crosses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Dir {
    In,
    Out,
}

pub(crate) fn scalar_prim(k: ScalarKind) -> Prim {
    match k {
        ScalarKind::Bool => Prim::Z,
        ScalarKind::I8 => Prim::B,
        ScalarKind::I16 => Prim::S,
        ScalarKind::I32 | ScalarKind::U8 | ScalarKind::U16 => Prim::I,
        ScalarKind::I64
        | ScalarKind::Isize
        | ScalarKind::U32
        | ScalarKind::U64
        | ScalarKind::Usize => Prim::J,
        ScalarKind::F32 => Prim::F,
        ScalarKind::F64 => Prim::D,
    }
}

fn scalar_kt(k: ScalarKind) -> &'static str {
    match k {
        ScalarKind::Bool => "Boolean",
        ScalarKind::I8 => "Byte",
        ScalarKind::I16 => "Short",
        ScalarKind::I32 | ScalarKind::U8 | ScalarKind::U16 => "Int",
        ScalarKind::I64 | ScalarKind::Isize | ScalarKind::U32 | ScalarKind::Usize => "Long",
        ScalarKind::U64 => "ULong",
        ScalarKind::F32 => "Float",
        ScalarKind::F64 => "Double",
    }
}

/// The primitive an array element crosses as: unsigned elements keep their
/// bits in the same-width signed primitive.
pub(crate) fn array_prim(k: ScalarKind) -> Prim {
    match k {
        ScalarKind::U8 => Prim::B,
        ScalarKind::U16 => Prim::S,
        ScalarKind::U32 => Prim::I,
        ScalarKind::U64 | ScalarKind::Usize => Prim::J,
        other => scalar_prim(other),
    }
}

/// Kotlin's hard keywords, which a generated name must escape.
const KT_KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

/// A Kotlin identifier, escaped when it is a keyword.
pub(crate) fn kt_ident(s: &str) -> String {
    if KT_KEYWORDS.contains(&s) {
        format!("`{s}`")
    } else {
        s.to_string()
    }
}

/// The Kotlin property name of a field: camelCase, or `v<i>` positionally.
pub(crate) fn kt_prop(f: &Field) -> String {
    match &f.name {
        Some(n) => kt_ident(&names::camel(&names::bare(n))),
        None => format!("v{}", f.index),
    }
}

/// The leaf path segment of a field.
pub(crate) fn field_seg(f: &Field) -> String {
    match &f.name {
        Some(n) => names::bare(n),
        None => format!("v{}", f.index),
    }
}

/// The leaf path segment of a sum alternative.
pub(crate) fn alt_seg(alt: &Alternative) -> String {
    names::snake(&names::bare(&alt.name))
}

/// The Kotlin class name of a sum alternative.
pub(crate) fn alt_kt_name(class: &Class, alt: &Alternative) -> String {
    let rust = names::bare(&alt.name);
    if let ClassKind::Sealed { renames } = &class.kind {
        if let Some(n) = renames.get(&rust) {
            return n.clone();
        }
    }
    match rust.as_str() {
        "None" => "None_".to_string(),
        _ => rust,
    }
}

/// A Rust ident for a leaf at `path` under `root`.
pub(crate) fn leaf_ident(root: &str, path: &str) -> syn::Ident {
    names::ident(&join(root, path))
}

impl Plan<'_> {
    /// One layer of `ty`. A borrow of a value the JNI side rebuilds (a data
    /// class, a converted type) is a [`Shape::Ref`] over it: the value is
    /// decoded and lent. A handle keeps its [`Access`].
    pub(crate) fn shape<'t>(&self, ty: &'t TypeRef) -> Res<Shape<'t, &Setting>> {
        let s = shape(ty, |candidate| {
            bare_declared_name(candidate).and_then(|name| self.types.get(name))
        })
        .map_err(|e| crate::Error(format!("`{ty}`: {e}")))?;
        Ok(match s {
            Shape::Declared {
                ty, declaration, ..
            } if Access::of(ty) != Access::Owned
                && !matches!(declaration, Setting::Class(c) if matches!(c.kind, ClassKind::Ptr { .. })) =>
            {
                Shape::Ref {
                    inner: ty.borrow_target().expect("a borrow"),
                    access: Access::of(ty),
                }
            }
            Shape::Str {
                kind: TextKind::Str,
                access: Access::Owned,
            }
            | Shape::Seq {
                kind: SequenceKind::Slice,
                access: Access::Owned,
                ..
            } => {
                return err(format!(
                    "`{ty}`: an unsized value cannot cross by value; borrow it or use Cow"
                ))
            }
            Shape::Str {
                kind: TextKind::Str,
                access: Access::Exclusive,
            }
            | Shape::Seq {
                kind: SequenceKind::Slice,
                access: Access::Exclusive,
                ..
            } => {
                return err(format!(
                    "`{ty}`: mutable unsized values have no JVM representation"
                ))
            }
            Shape::Undeclared(name) => {
                return err(format!(
                    "`{name}` is not declared as a class or a conversion"
                ))
            }
            Shape::Result { .. } => {
                return err(format!("`{ty}`: `Result` crosses only as a result"))
            }
            Shape::Out(_) => return err(format!("`{ty}` has no JVM representation")),
            s => s,
        })
    }

    /// The representation a converted type crosses as.
    pub(crate) fn conv_repr<'c>(&self, c: &'c Conv) -> &'c TypeRef {
        c.resolved.repr()
    }

    /// The fields of a data class's struct.
    pub(crate) fn struct_of(&self, c: &Class) -> Res<&prebindgen_flat::flat::Struct> {
        match self.flat.declared_type(&c.rust) {
            Some(prebindgen_flat::flat::Type::Struct(s)) => Ok(s),
            _ => err(format!("`{}` is not a struct", c.rust)),
        }
    }

    /// The alternatives of a sealed class's enum.
    pub(crate) fn variant_of(&self, c: &Class) -> Res<&prebindgen_flat::flat::Variant> {
        match self.flat.declared_type(&c.rust) {
            Some(prebindgen_flat::flat::Type::Variant(v)) => Ok(v),
            _ => err(format!("`{}` is not a data-carrying enum", c.rust)),
        }
    }

    /// `(name, discriminant literal)` for each value of an enum class.
    pub(crate) fn enum_arms(&self, class: &Class) -> Res<Vec<(syn::Ident, proc_macro2::Literal)>> {
        let Some(prebindgen_flat::flat::Type::Enum(e)) = self.flat.declared_type(&class.rust)
        else {
            return err(format!("`{}` is not a fieldless enum", class.rust));
        };
        let values = e.discriminant_values().map_err(|v| {
            crate::Error(format!(
                "`{}::{v}`: discriminant is not a literal",
                class.rust
            ))
        })?;
        Ok(values
            .into_iter()
            .map(|(n, d)| (n.clone(), proc_macro2::Literal::i32_unsuffixed(d as i32)))
            .collect())
    }

    // ── leaves ──────────────────────────────────────────────────────────

    /// The leaves `ty` crosses on.
    pub(crate) fn leaves(&self, ty: &TypeRef, dir: Dir) -> Res<Vec<Leaf>> {
        Ok(match self.shape(ty)? {
            Shape::Unit => Vec::new(),
            Shape::Scalar(k) => vec![Leaf::new(LeafTy::Prim(scalar_prim(k)))],
            Shape::Str { .. } => vec![Leaf::new(LeafTy::String)],
            Shape::Seq { elem, .. } if is_u8(elem) => vec![Leaf::new(LeafTy::PrimArray(Prim::B))],
            Shape::Array { elem, .. } => match elem.kind() {
                prebindgen_flat::flat::TypeKind::Scalar(k) => {
                    vec![Leaf::new(LeafTy::PrimArray(array_prim(*k)))]
                }
                _ => return err(format!("`{ty}`: only arrays of primitives cross")),
            },
            Shape::Declared { declaration, .. } => match declaration {
                Setting::Converted(c) => self.leaves(self.conv_repr(c), dir)?,
                Setting::Class(c) => match &c.kind {
                    ClassKind::Ptr { .. } => vec![Leaf::new(LeafTy::Prim(Prim::J))],
                    ClassKind::Enum => vec![Leaf::new(LeafTy::Prim(Prim::I))],
                    ClassKind::Data => {
                        let mut out = Vec::new();
                        for f in &self.struct_of(c)?.fields {
                            let seg = field_seg(f);
                            out.extend(self.leaves(&f.ty, dir)?.into_iter().map(|l| l.under(&seg)));
                        }
                        out
                    }
                    ClassKind::Sealed { .. } => {
                        let mut out = vec![Leaf::new(LeafTy::Prim(Prim::I)).under("_tag")];
                        for alt in &self.variant_of(c)?.alternatives {
                            let aseg = alt_seg(alt);
                            for f in &alt.fields {
                                let seg = join(&aseg, &field_seg(f));
                                out.extend(self.leaves(&f.ty, dir)?.into_iter().map(|mut l| {
                                    l.nullable |= l.is_obj();
                                    l.under(&seg)
                                }));
                            }
                        }
                        out
                    }
                },
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, dir)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    let mut l = inner_leaves.into_iter().next().unwrap();
                    l.nullable = true;
                    vec![l]
                } else if dir == Dir::Out && inner_leaves.len() == 1 {
                    let p = inner_leaves[0].prim().expect("a primitive leaf");
                    vec![Leaf::new(LeafTy::Boxed(p))]
                } else {
                    let mut out = vec![Leaf::new(LeafTy::Prim(Prim::Z)).under("_present")];
                    out.extend(inner_leaves.into_iter().map(|mut l| {
                        l.nullable |= l.is_obj();
                        l
                    }));
                    out
                }
            }
            Shape::Seq { elem, .. } => {
                let mut out = vec![Leaf::new(LeafTy::Prim(Prim::I)).under("_n")];
                out.extend(self.leaves(elem, dir)?.iter().map(Leaf::column));
                out
            }
            Shape::Ref { inner, .. } | Shape::Boxed(inner) => self.leaves(inner, dir)?,
            Shape::Cow(inner) => self.leaves(&cow_view(inner), dir)?,
            Shape::Callback(_) => vec![Leaf::new(LeafTy::Callback(self.callback_raw_fqn(ty)?))],
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }

    // ── Kotlin types ────────────────────────────────────────────────────

    /// The Kotlin type of a value of `ty`.
    pub(crate) fn kt_type(&self, ty: &TypeRef) -> Res<String> {
        Ok(match self.shape(ty)? {
            Shape::Unit => "Unit".to_string(),
            Shape::Scalar(k) => scalar_kt(k).to_string(),
            Shape::Str { .. } => "String".to_string(),
            Shape::Seq { elem, .. } if is_u8(elem) => "ByteArray".to_string(),
            Shape::Array { elem, .. } => match elem.kind() {
                prebindgen_flat::flat::TypeKind::Scalar(k) => array_prim(*k).kt_array(),
                _ => return err(format!("`{ty}`: only arrays of primitives cross")),
            },
            Shape::Declared { declaration, .. } => match declaration {
                Setting::Class(c) => c.fqn(),
                Setting::Converted(c) => self.kt_type(self.conv_repr(c))?,
            },
            Shape::Option(inner) => format!("{}?", self.kt_type(inner)?),
            Shape::Seq { elem, .. } => format!("List<{}>", self.kt_type(elem)?),
            Shape::Ref { inner, .. } | Shape::Boxed(inner) => self.kt_type(inner)?,
            Shape::Cow(inner) => self.kt_type(&cow_view(inner))?,
            Shape::Callback(_) => self.callback_fqn(ty)?,
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }

    /// Whether a value of `ty`, held in Kotlin, owns a native handle (and so
    /// must be closed).
    pub(crate) fn owns_handle(&self, ty: &TypeRef) -> Res<bool> {
        Ok(match self.shape(ty)? {
            Shape::Declared {
                declaration: Setting::Class(c),
                ..
            } => match &c.kind {
                ClassKind::Ptr { .. } => Access::of(ty) != Access::Exclusive,
                ClassKind::Data => {
                    let mut any = false;
                    for f in &self.struct_of(c)?.fields {
                        any |= self.owns_handle(&f.ty)?;
                    }
                    any
                }
                ClassKind::Sealed { .. } => {
                    let mut any = false;
                    for a in &self.variant_of(c)?.alternatives {
                        for f in &a.fields {
                            any |= self.owns_handle(&f.ty)?;
                        }
                    }
                    any
                }
                ClassKind::Enum => false,
            },
            Shape::Option(t)
            | Shape::Seq { elem: t, .. }
            | Shape::Ref { inner: t, .. }
            | Shape::Boxed(t) => self.owns_handle(t)?,
            Shape::Cow(t) => self.owns_handle(&cow_view(t))?,
            _ => false,
        })
    }
}

/// Whether `ty` is `u8`: a sequence of it is a byte array.
pub(crate) fn is_u8(ty: &TypeRef) -> bool {
    matches!(
        ty.kind(),
        prebindgen_flat::flat::TypeKind::Scalar(ScalarKind::U8)
    )
}

/// A Cow's unsized target is viewed through a shared borrow when choosing its
/// Kotlin type and leaves. This does not admit bare unsized boundary values.
pub(crate) fn cow_view(inner: &TypeRef) -> std::borrow::Cow<'_, TypeRef> {
    use prebindgen_flat::flat::TypeKind;
    match inner.kind() {
        TypeKind::Str | TypeKind::Slice(_) => std::borrow::Cow::Owned(inner.borrowed()),
        _ => std::borrow::Cow::Borrowed(inner),
    }
}
