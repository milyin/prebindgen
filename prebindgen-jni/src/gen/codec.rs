//! The value codec: how a type crosses, in both languages, both directions.
//!
//! [`Gen::kind`] classifies a type one level deep; every other function here
//! recurses on it, so the leaf lists and the four conversions cannot drift
//! apart:
//!
//! | | into Rust | out of Rust |
//! |---|---|---|
//! | leaves | [`Gen::leaves`] (`Dir::In`) | [`Gen::leaves`] (`Dir::Out`) |
//! | Kotlin | [`Gen::kt_encode`]: value → leaf expressions | [`Gen::kt_decode`]: leaf expressions → value |
//! | Rust | [`Gen::rs_decode`]: wires → value ([`Input`]) | [`Gen::rs_encode`]: value → wires ([`Output`]) |
//!
//! A data class is the concatenation of its fields; an optional value is a
//! presence flag beside its inner leaves (a single object leaf is simply
//! nullable, and a single primitive leaving Rust is boxed); a sum is a tag
//! plus every alternative's fields; a sequence is a count plus one array per
//! element leaf.

use std::rc::Rc;

use prebindgen_tools::{
    flat::flat::{Alternative, Field, ScalarKind, Struct, Type as FlatType, TypeKind, TypeRef, Variant},
    names, record_in, record_out, Input, Output, Record, Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    err,
    leaf::{join, Leaf, LeafTy, Prim},
    Class, ClassKind, Conv, Gen, Res,
};

/// Which way a value crosses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Dir {
    In,
    Out,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Borrow {
    Own,
    Shared,
    Mut,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Form {
    Owned,
    /// `&str`, `&[T]`: passed as a borrow of the decoded value.
    Slice,
    /// `&Vec<T>`: passed as a borrow of the decoded `Vec`.
    RefVec,
    Cow,
}

/// A type, classified one level deep.
pub(crate) enum Kind<'t> {
    Unit,
    Scalar(ScalarKind),
    Str(Form),
    Bytes(Form),
    Array { elem: ScalarKind, len: usize },
    Handle { class: Rc<Class>, borrow: Borrow },
    Enum(Rc<Class>),
    Record(Rc<Class>, &'t Struct),
    Sum(Rc<Class>, &'t Variant),
    Option(&'t TypeRef),
    Seq { elem: &'t TypeRef, form: Form },
    Ref { inner: &'t TypeRef, mutable: bool },
    Boxed(&'t TypeRef),
    Cow(&'t TypeRef),
    Converted(Rc<Conv>),
    Callback(&'t [TypeRef]),
}

/// Kotlin-side encoding state: statements to run before the call and the
/// handles the call touches.
#[derive(Default)]
pub(crate) struct KtEnc {
    pub prelude: Vec<String>,
    pub handles: Vec<HandleSite>,
    pub counter: usize,
}

/// A handle a call passes: it is checked, locked and — when consumed —
/// marked as such.
#[derive(Clone)]
pub(crate) struct HandleSite {
    pub expr: String,
    pub nullable: bool,
    pub consumed: bool,
    pub class: Rc<Class>,
    /// A label for diagnostics (the parameter path).
    pub label: String,
}

fn scalar_prim(k: ScalarKind) -> Prim {
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
fn array_prim(k: ScalarKind) -> Prim {
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
    "as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in",
    "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true",
    "try", "typealias", "typeof", "val", "var", "when", "while",
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

fn rt() -> TokenStream {
    quote!(::prebindgen_jni_runtime)
}

impl<'a> Gen<'a> {
    /// Classify `ty` one level deep.
    pub(crate) fn kind<'t>(&self, ty: &'t TypeRef) -> Res<Kind<'t>>
    where
        'a: 't,
    {
        let named = |g: &Self, id: &str, borrow: Borrow| -> Res<Kind<'t>> {
            if let Some(c) = g.converts.get(id) {
                if borrow != Borrow::Own {
                    return err(format!("`{ty}`: a converted type crosses by value"));
                }
                return Ok(Kind::Converted(c.clone()));
            }
            let Some(class) = g.classes.get(id).cloned() else {
                return err(format!("`{id}` is not declared as a class or a conversion"));
            };
            Ok(match &class.kind {
                ClassKind::Ptr { .. } => Kind::Handle { class, borrow },
                _ if borrow != Borrow::Own => unreachable!("handled by the caller"),
                ClassKind::Enum => Kind::Enum(class),
                ClassKind::Data { .. } => {
                    let Some(FlatType::Struct(s)) = g.flat.declared_type(id) else {
                        return err(format!("`{id}` is not a struct"));
                    };
                    Kind::Record(class, s)
                }
                ClassKind::Sealed { .. } => {
                    let Some(FlatType::Variant(v)) = g.flat.declared_type(id) else {
                        return err(format!("`{id}` is not a data-carrying enum"));
                    };
                    Kind::Sum(class, v)
                }
            })
        };
        let is_u8 = |t: &TypeRef| matches!(t.kind(), TypeKind::Scalar(ScalarKind::U8));
        Ok(match ty.kind() {
            TypeKind::Unit => Kind::Unit,
            TypeKind::Scalar(k) => Kind::Scalar(*k),
            TypeKind::String => Kind::Str(Form::Owned),
            TypeKind::Str => return err("a bare `str` cannot cross; use `&str` or `String`"),
            TypeKind::Optional(t) => Kind::Option(t),
            TypeKind::Vec(t) if is_u8(t) => Kind::Bytes(Form::Owned),
            TypeKind::Vec(t) => Kind::Seq {
                elem: t,
                form: Form::Owned,
            },
            TypeKind::Slice(_) => return err("a bare slice cannot cross; use `&[T]` or `Vec<T>`"),
            TypeKind::Array { elem, extent } => match elem.kind() {
                TypeKind::Scalar(k) => Kind::Array {
                    elem: *k,
                    len: extent.value,
                },
                _ => return err(format!("`{ty}`: only arrays of primitives cross")),
            },
            TypeKind::Boxed(t) => Kind::Boxed(t),
            TypeKind::Cow { inner, .. } => match inner.kind() {
                TypeKind::Str => Kind::Str(Form::Cow),
                TypeKind::Slice(e) if is_u8(e) => Kind::Bytes(Form::Cow),
                TypeKind::Slice(e) => Kind::Seq {
                    elem: e,
                    form: Form::Cow,
                },
                _ => Kind::Cow(inner),
            },
            TypeKind::Ref { mutable, inner, .. } => match inner.kind() {
                TypeKind::Str => Kind::Str(Form::Slice),
                TypeKind::Slice(e) if is_u8(e) => Kind::Bytes(Form::Slice),
                TypeKind::Slice(e) => Kind::Seq {
                    elem: e,
                    form: Form::Slice,
                },
                TypeKind::Vec(e) if is_u8(e) => Kind::Bytes(Form::RefVec),
                TypeKind::Vec(e) => Kind::Seq {
                    elem: e,
                    form: Form::RefVec,
                },
                TypeKind::Named { id, .. }
                    if self
                        .classes
                        .get(&id.name)
                        .is_some_and(|c| matches!(c.kind, ClassKind::Ptr { .. })) =>
                {
                    named(
                        self,
                        &id.name,
                        if *mutable { Borrow::Mut } else { Borrow::Shared },
                    )?
                }
                _ => Kind::Ref {
                    inner,
                    mutable: *mutable,
                },
            },
            TypeKind::Named { id, .. } => named(self, &id.name, Borrow::Own)?,
            TypeKind::Callback { args } => Kind::Callback(args),
            TypeKind::Fallible { .. } => return err(format!("`{ty}`: `Result` crosses only as a result")),
            TypeKind::Uninit(_) => return err(format!("`{ty}` has no JVM representation")),
        })
    }

    /// The representation a converted type crosses as, in one direction.
    fn conv_repr<'c>(&self, c: &'c Conv, dir: Dir) -> Res<&'c TypeRef> {
        let stage = match dir {
            Dir::In => c.resolved.input.as_ref().or(c.resolved.output.as_ref()),
            Dir::Out => c.resolved.output.as_ref().or(c.resolved.input.as_ref()),
        };
        stage
            .map(|s| &s.repr)
            .ok_or_else(|| crate::Error(format!("convert!({}) declares no direction", c.name)))
    }

    // ── leaves ──────────────────────────────────────────────────────────

    /// The leaves `ty` crosses on.
    pub(crate) fn leaves(&self, ty: &TypeRef, dir: Dir) -> Res<Vec<Leaf>> {
        Ok(match self.kind(ty)? {
            Kind::Unit => Vec::new(),
            Kind::Scalar(k) => vec![Leaf::new(LeafTy::Prim(scalar_prim(k)))],
            Kind::Str(_) => vec![Leaf::new(LeafTy::String)],
            Kind::Bytes(_) => vec![Leaf::new(LeafTy::PrimArray(Prim::B))],
            Kind::Array { elem, .. } => vec![Leaf::new(LeafTy::PrimArray(array_prim(elem)))],
            Kind::Handle { .. } => vec![Leaf::new(LeafTy::Prim(Prim::J))],
            Kind::Enum(_) => vec![Leaf::new(LeafTy::Prim(Prim::I))],
            Kind::Record(_, s) => {
                let mut out = Vec::new();
                for f in &s.fields {
                    let seg = field_seg(f);
                    out.extend(self.leaves(&f.ty, dir)?.into_iter().map(|l| l.under(&seg)));
                }
                out
            }
            Kind::Sum(_, v) => {
                let mut out = vec![Leaf::new(LeafTy::Prim(Prim::I)).under("_tag")];
                for alt in &v.alternatives {
                    let aseg = names::snake(&names::bare(&alt.name));
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
            Kind::Option(inner) => {
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
            Kind::Seq { elem, .. } => {
                let mut out = vec![Leaf::new(LeafTy::Prim(Prim::I)).under("_n")];
                out.extend(self.leaves(elem, dir)?.iter().map(Leaf::column));
                out
            }
            Kind::Ref { inner, .. } | Kind::Boxed(inner) | Kind::Cow(inner) => {
                self.leaves(inner, dir)?
            }
            Kind::Converted(c) => self.leaves(self.conv_repr(&c, dir)?, dir)?,
            Kind::Callback(_) => vec![Leaf::new(LeafTy::Callback(self.callback_raw_fqn(ty)?))],
        })
    }

    // ── Kotlin types ────────────────────────────────────────────────────

    /// The Kotlin type of a value of `ty`.
    pub(crate) fn kt_type(&self, ty: &TypeRef) -> Res<String> {
        Ok(match self.kind(ty)? {
            Kind::Unit => "Unit".to_string(),
            Kind::Scalar(k) => scalar_kt(k).to_string(),
            Kind::Str(_) => "String".to_string(),
            Kind::Bytes(_) => "ByteArray".to_string(),
            Kind::Array { elem, .. } => array_prim(elem).kt_array(),
            Kind::Handle { class, .. } | Kind::Enum(class) | Kind::Record(class, _) | Kind::Sum(class, _) => {
                class.fqn()
            }
            Kind::Option(inner) => format!("{}?", self.kt_type(inner)?),
            Kind::Seq { elem, .. } => format!("List<{}>", self.kt_type(elem)?),
            Kind::Ref { inner, .. } | Kind::Boxed(inner) | Kind::Cow(inner) => self.kt_type(inner)?,
            Kind::Converted(c) => self.kt_type(self.conv_repr(&c, Dir::In)?)?,
            Kind::Callback(_) => self.callback_fqn(ty)?,
        })
    }

    // ── Kotlin encode ───────────────────────────────────────────────────

    /// Kotlin expressions for the input leaves of `ty`, from `expr` (which is
    /// `null`-able when `nullable`).
    pub(crate) fn kt_encode(
        &self,
        ty: &TypeRef,
        expr: &str,
        nullable: bool,
        cx: &mut KtEnc,
        consumed: bool,
    ) -> Res<Vec<String>> {
        let access = |e: &str, prop: &str| {
            if nullable {
                format!("{e}?.{prop}")
            } else {
                format!("{e}.{prop}")
            }
        };
        let or_default = |e: String, d: &str| {
            if nullable {
                format!("({e} ?: {d})")
            } else {
                e
            }
        };
        Ok(match self.kind(ty)? {
            Kind::Unit => Vec::new(),
            Kind::Scalar(k) => {
                let p = scalar_prim(k);
                let e = if k == ScalarKind::U64 {
                    if nullable {
                        format!("{expr}?.toLong()")
                    } else {
                        format!("{expr}.toLong()")
                    }
                } else {
                    expr.to_string()
                };
                vec![or_default(e, p.kt_default())]
            }
            Kind::Str(_) | Kind::Bytes(_) | Kind::Array { .. } => vec![expr.to_string()],
            Kind::Handle { class, borrow } => {
                cx.handles.push(HandleSite {
                    expr: expr.to_string(),
                    nullable,
                    consumed: consumed && borrow == Borrow::Own,
                    class,
                    label: expr.to_string(),
                });
                vec![or_default(access(expr, "ptr"), "0L")]
            }
            Kind::Enum(_) => vec![or_default(access(expr, "value"), "0")],
            Kind::Record(_, s) => {
                let mut out = Vec::new();
                for f in &s.fields {
                    let e = access(expr, &kt_prop(f));
                    out.extend(self.kt_encode(&f.ty, &e, nullable, cx, consumed)?);
                }
                out
            }
            Kind::Sum(class, v) => {
                let mut arms: Vec<String> = Vec::new();
                if nullable {
                    arms.push("null -> 0".to_string());
                }
                for (i, alt) in v.alternatives.iter().enumerate() {
                    arms.push(format!("is {}.{} -> {i}", class.fqn(), alt_kt_name(&class, alt)));
                }
                let mut out = vec![format!("when ({expr}) {{ {} }}", arms.join("; "))];
                for alt in &v.alternatives {
                    let cast = format!("({expr} as? {}.{})", class.fqn(), alt_kt_name(&class, alt));
                    for f in &alt.fields {
                        let e = format!("{cast}?.{}", kt_prop(f));
                        out.extend(self.kt_encode(&f.ty, &e, true, cx, consumed)?);
                    }
                }
                out
            }
            Kind::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::In)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    self.kt_encode(inner, expr, true, cx, consumed)?
                } else {
                    let mut out = vec![format!("({expr} != null)")];
                    out.extend(self.kt_encode(inner, expr, true, cx, consumed)?);
                    out
                }
            }
            Kind::Seq { elem, .. } => {
                let id = cx.counter;
                cx.counter += 1;
                let list = format!("__s{id}");
                let n = format!("__s{id}_n");
                let e = format!("__s{id}_e");
                let i = format!("__s{id}_i");
                let leaves = self.leaves(elem, Dir::In)?;
                let mut elem_cx = KtEnc {
                    counter: cx.counter,
                    ..KtEnc::default()
                };
                let elem_exprs = self.kt_encode(elem, &e, false, &mut elem_cx, consumed)?;
                cx.counter = elem_cx.counter;
                if !elem_cx.handles.is_empty() {
                    return err(format!("`{ty}`: a sequence of handles cannot cross into Rust"));
                }
                let mut pre = vec![
                    format!("val {list} = {expr}"),
                    if nullable {
                        format!("val {n} = {list}?.size ?: 0")
                    } else {
                        format!("val {n} = {list}.size")
                    },
                ];
                let mut cols = Vec::new();
                let mut fills = Vec::new();
                for (k, (l, v)) in leaves.iter().zip(&elem_exprs).enumerate() {
                    let col = format!("__s{id}_c{k}");
                    match l.prim() {
                        Some(p) => pre.push(format!("val {col} = {}({n})", p.kt_array())),
                        None => pre.push(format!("val {col} = arrayOfNulls<Any?>({n})")),
                    }
                    fills.push(format!("{col}[{i}] = {v}"));
                    cols.push(col);
                }
                let mut body = elem_cx.prelude;
                body.extend(fills);
                let src = if nullable { format!("{list}!!") } else { list.clone() };
                let guard = if nullable {
                    format!("if ({list} != null) ")
                } else {
                    String::new()
                };
                pre.push(format!(
                    "{guard}for ({i} in 0 until {n}) {{ val {e} = {src}[{i}]; {} }}",
                    body.join("; ")
                ));
                cx.prelude.extend(pre);
                let mut out = vec![n];
                out.extend(cols);
                out
            }
            Kind::Ref { inner, .. } | Kind::Boxed(inner) | Kind::Cow(inner) => {
                self.kt_encode(inner, expr, nullable, cx, consumed)?
            }
            Kind::Converted(c) => self.kt_encode(self.conv_repr(&c, Dir::In)?, expr, nullable, cx, consumed)?,
            Kind::Callback(_) => vec![if nullable {
                format!("{expr}?.asRaw()")
            } else {
                format!("{expr}.asRaw()")
            }],
        })
    }

    // ── Kotlin decode ───────────────────────────────────────────────────

    /// A Kotlin expression building a value of `ty` from its output leaves'
    /// expressions. `gated` marks leaves whose object values Kotlin sees as
    /// nullable although the value itself is present (an alternative's
    /// group, an optional's inner value).
    pub(crate) fn kt_decode(&self, ty: &TypeRef, leaves: &[String], gated: bool, depth: usize) -> Res<String> {
        let bang = |e: &str| if gated { format!("{e}!!") } else { e.to_string() };
        Ok(match self.kind(ty)? {
            Kind::Unit => "Unit".to_string(),
            Kind::Scalar(ScalarKind::U64) => format!("{}.toULong()", leaves[0]),
            Kind::Scalar(_) => leaves[0].clone(),
            Kind::Str(_) | Kind::Bytes(_) | Kind::Array { .. } => bang(&leaves[0]),
            Kind::Handle { class, .. } => format!("{}({})", class.fqn(), leaves[0]),
            Kind::Enum(class) => format!("{}.fromInt({})", class.fqn(), leaves[0]),
            Kind::Record(class, s) => {
                let mut args = Vec::new();
                let mut at = 0;
                for f in &s.fields {
                    let n = self.leaves(&f.ty, Dir::Out)?.len();
                    args.push(self.kt_decode(&f.ty, &leaves[at..at + n], gated, depth)?);
                    at += n;
                }
                format!("{}({})", class.fqn(), args.join(", "))
            }
            Kind::Sum(class, v) => {
                let tag = &leaves[0];
                let mut at = 1;
                let mut arms = Vec::new();
                for (i, alt) in v.alternatives.iter().enumerate() {
                    let cname = format!("{}.{}", class.fqn(), alt_kt_name(&class, alt));
                    if alt.fields.is_empty() {
                        arms.push(format!("{i} -> {cname}"));
                        continue;
                    }
                    let mut args = Vec::new();
                    for f in &alt.fields {
                        let n = self.leaves(&f.ty, Dir::Out)?.len();
                        args.push(self.kt_decode(&f.ty, &leaves[at..at + n], true, depth)?);
                        at += n;
                    }
                    arms.push(format!("{i} -> {cname}({})", args.join(", ")));
                }
                arms.push(format!(
                    "else -> throw IllegalArgumentException(\"{}: invalid tag ${tag}\")",
                    class.name
                ));
                format!("when ({tag}) {{ {} }}", arms.join("; "))
            }
            Kind::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::Out)?;
                if inner_leaves.len() == 1 {
                    let it = format!("__o{depth}");
                    let d = self.kt_decode(inner, &[it.clone()], false, depth + 1)?;
                    if d == it {
                        leaves[0].clone()
                    } else {
                        format!("{}?.let {{ {it} -> {d} }}", leaves[0])
                    }
                } else {
                    let d = self.kt_decode(inner, &leaves[1..], true, depth)?;
                    format!("(if ({}) {d} else null)", leaves[0])
                }
            }
            Kind::Seq { elem, .. } => {
                let n = &leaves[0];
                let i = format!("__i{depth}");
                let el = self.leaves(elem, Dir::Out)?;
                let bang = if gated { "!!" } else { "" };
                let items: Vec<String> = el
                    .iter()
                    .zip(&leaves[1..])
                    .map(|(l, col)| match l.prim() {
                        Some(_) => format!("{col}{bang}[{i}]"),
                        None => format!("({col}{bang}[{i}] as {})", l.kt_raw()),
                    })
                    .collect();
                let d = self.kt_decode(elem, &items, false, depth + 1)?;
                format!("List({n}) {{ {i} -> {d} }}")
            }
            Kind::Ref { inner, .. } | Kind::Boxed(inner) | Kind::Cow(inner) => {
                self.kt_decode(inner, leaves, gated, depth)?
            }
            Kind::Converted(c) => self.kt_decode(self.conv_repr(&c, Dir::Out)?, leaves, gated, depth)?,
            Kind::Callback(_) => return err(format!("`{ty}`: a callback cannot leave Rust")),
        })
    }

    // ── Rust decode ─────────────────────────────────────────────────────

    /// Wires → a value of `ty`. The wires are the input leaves of `ty`, named
    /// under `root`.
    pub(crate) fn rs_decode(&self, ty: &TypeRef, root: &str, depth: usize) -> Res<Input> {
        let rt = rt();
        let wires = |g: &Self| -> Res<Vec<Wire>> {
            Ok(g.leaves(ty, Dir::In)?
                .iter()
                .map(|l| Wire::new(leaf_ident(root, &l.name), l.rs()))
                .collect())
        };
        let w = leaf_ident(root, "");
        Ok(match self.kind(ty)? {
            Kind::Unit => Input::new(Vec::new(), quote!(())),
            Kind::Scalar(k) => {
                let wires = wires(self)?;
                let range = |t: TokenStream, name: &str| {
                    let msg = format!("{name} input out of range: {{}}");
                    quote!(<#t as ::core::convert::TryFrom<_>>::try_from(#w).map_err(|_| ::std::format!(#msg, #w))?)
                };
                match k {
                    ScalarKind::Bool => Input::new(wires, quote!((#w != 0))),
                    ScalarKind::U8 => Input::fallible(wires, range(quote!(u8), "u8")),
                    ScalarKind::U16 => Input::fallible(wires, range(quote!(u16), "u16")),
                    ScalarKind::U32 => Input::fallible(wires, range(quote!(u32), "u32")),
                    ScalarKind::Usize => Input::fallible(wires, range(quote!(usize), "usize")),
                    ScalarKind::U64 => Input::new(wires, quote!((#w as u64))),
                    ScalarKind::Isize => Input::new(wires, quote!((#w as isize))),
                    _ => Input::new(wires, quote!(#w)),
                }
            }
            Kind::Str(form) => {
                let base = Input::fallible(wires(self)?, quote!(#rt::read_string(env, &#w)?));
                match form {
                    Form::Cow => base.map(|e| quote!(::std::borrow::Cow::Owned(#e))),
                    _ => base,
                }
            }
            Kind::Bytes(form) => {
                let base = Input::fallible(wires(self)?, quote!(#rt::read_u8s(env, &#w)?));
                match form {
                    Form::Cow => base.map(|e| quote!(::std::borrow::Cow::Owned(#e))),
                    _ => base,
                }
            }
            Kind::Array { elem, len } => {
                let p = array_prim(elem);
                let (read, _) = p.array_helpers();
                let read = format_ident!("{}", read);
                let t = names::ident(elem.as_str());
                let conv = if elem == ScalarKind::Bool {
                    quote!(__x != 0)
                } else {
                    quote!(__x as #t)
                };
                Input::fallible(
                    wires(self)?,
                    quote!(#rt::fixed::<#t, #len>(#rt::#read(env, &#w)?.into_iter().map(|__x| #conv).collect())?),
                )
            }
            Kind::Handle { class, borrow } => {
                let t = self.q.path(&names::ident(&class.rust));
                let f = match borrow {
                    Borrow::Own => quote!(take_handle),
                    Borrow::Shared => quote!(borrow_handle),
                    Borrow::Mut => quote!(borrow_handle_mut),
                };
                Input::fallible(wires(self)?, quote!(#rt::#f::<#t>(#w)?))
            }
            Kind::Enum(class) => {
                let t = self.q.path(&names::ident(&class.rust));
                let arms = self.enum_arms(&class)?;
                let pats = arms.iter().map(|(n, d)| quote!(#d => ::core::result::Result::Ok(#t::#n)));
                let msg = format!("invalid value {{}} for enum `{}`", class.rust);
                Input::fallible(
                    wires(self)?,
                    quote!((match #w { #(#pats,)* __v => ::core::result::Result::Err(::std::format!(#msg, __v)) })?),
                )
            }
            Kind::Record(class, s) => {
                let head = self.q.path(&names::ident(&class.rust));
                let mut parts = Vec::new();
                for f in &s.fields {
                    parts.push(self.rs_decode(&f.ty, &join(root, &field_seg(f)), depth)?);
                }
                record_in(Record::Struct(s), &head, parts)
            }
            Kind::Sum(class, v) => {
                let head = self.q.path(&names::ident(&class.rust));
                let tag = leaf_ident(root, "_tag");
                let mut wires = vec![Wire::new(tag.clone(), Prim::I.rs())];
                let mut arms = Vec::new();
                let mut fallible = false;
                for (i, alt) in v.alternatives.iter().enumerate() {
                    let aseg = names::snake(&names::bare(&alt.name));
                    let mut parts = Vec::new();
                    for f in &alt.fields {
                        parts.push(self.rs_decode(&f.ty, &join(root, &join(&aseg, &field_seg(f))), depth)?);
                    }
                    let an = &alt.name;
                    let built = record_in(Record::Alt(alt), &quote!(#head::#an), parts);
                    fallible |= built.fallible;
                    wires.extend(built.wires);
                    let e = built.expr;
                    let i = i as i32;
                    arms.push(quote!(#i => ::core::result::Result::Ok(#e)));
                }
                let msg = format!("{}: invalid tag {{}}", class.name);
                let _ = fallible;
                Input::fallible(
                    wires,
                    quote!((match #tag { #(#arms,)* __t => ::core::result::Result::Err(::std::format!(#msg, __t)) })?),
                )
            }
            Kind::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::In)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    let i = self.rs_decode(inner, root, depth)?;
                    let n = &i.wires[0].name;
                    Input::optional(None, quote!(!#n.is_null()), i.clone())
                } else {
                    let present = leaf_ident(root, "_present");
                    let i = self.rs_decode(inner, root, depth)?;
                    Input::optional(Some(Wire::new(present.clone(), Prim::Z.rs())), quote!((#present != 0)), i)
                }
            }
            Kind::Seq { elem, form } => {
                let n = leaf_ident(root, "_n");
                let el = self.leaves(elem, Dir::In)?;
                let er = format!("__e{depth}");
                let elem_in = self.rs_decode(elem, &er, depth + 1)?;
                let mut wires = vec![Wire::new(n.clone(), Prim::I.rs())];
                let mut reads = Vec::new();
                let mut binds = Vec::new();
                let mut drops = Vec::new();
                for (k, l) in el.iter().enumerate() {
                    let col = leaf_ident(root, &l.name);
                    wires.push(Wire::new(col.clone(), l.column().rs()));
                    let local = leaf_ident(&er, &l.name);
                    match l.prim() {
                        Some(p) => {
                            let (read, _) = p.array_helpers();
                            let read = format_ident!("{}", read);
                            let buf = format_ident!("__c{}_{}", depth, k);
                            reads.push(quote!(let #buf = #rt::#read(env, &#col)?;));
                            binds.push(quote!(let #local = #buf[__i];));
                        }
                        None => {
                            binds.push(quote!(let #local = #rt::object_array_get(env, &#col, __i)?;));
                            drops.push(quote!(#rt::drop_local(env, #local);));
                        }
                    }
                }
                let e = elem_in.result();
                let collected = quote!({
                    let __n = #n as usize;
                    #(#reads)*
                    let mut __v = ::std::vec::Vec::with_capacity(__n);
                    for __i in 0..__n {
                        #(#binds)*
                        let __x = #e;
                        #(#drops)*
                        __v.push(__x?);
                    }
                    __v
                });
                let input = Input::fallible(wires, collected);
                match form {
                    Form::Cow => input.map(|e| quote!(::std::borrow::Cow::Owned(#e))),
                    _ => input,
                }
            }
            Kind::Ref { inner, .. } => self.rs_decode(inner, root, depth)?,
            Kind::Boxed(inner) => self
                .rs_decode(inner, root, depth)?
                .map(|e| quote!(::std::boxed::Box::new(#e))),
            Kind::Cow(inner) => self
                .rs_decode(inner, root, depth)?
                .map(|e| quote!(::std::borrow::Cow::Owned(#e))),
            Kind::Converted(c) => {
                let stage = c
                    .resolved
                    .input
                    .as_ref()
                    .ok_or_else(|| crate::Error(format!("convert!({}) has no input", c.name)))?;
                let repr = self.rs_decode(&stage.repr, root, depth)?;
                let r = format_ident!("__r{}", depth);
                let check = self.domain_check(&c, &r);
                let applied = stage.apply(&self.q, &c.resolved.target, &r.to_token_stream());
                let fallible = repr.fallible || stage.fallible || c.range.is_some();
                let e = repr.expr;
                Input {
                    wires: repr.wires,
                    expr: quote!({ let #r = #e; #check #applied }),
                    fallible,
                    pass: None,
                }
            }
            Kind::Callback(args) => {
                let decode = self.callback_closure(ty, args, &w)?;
                Input::fallible(wires(self)?, decode)
            }
        })
    }

    /// `(name, discriminant literal)` for each value of an enum class.
    pub(crate) fn enum_arms(&self, class: &Class) -> Res<Vec<(syn::Ident, proc_macro2::Literal)>> {
        let Some(FlatType::Enum(e)) = self.flat.declared_type(&class.rust) else {
            return err(format!("`{}` is not a fieldless enum", class.rust));
        };
        let values = e
            .discriminant_values()
            .map_err(|v| crate::Error(format!("`{}::{v}`: discriminant is not a literal", class.rust)))?;
        Ok(values
            .into_iter()
            .map(|(n, d)| (n.clone(), proc_macro2::Literal::i32_unsuffixed(d as i32)))
            .collect())
    }

    fn domain_check(&self, c: &Conv, r: &syn::Ident) -> TokenStream {
        match c.range {
            Some((lo, hi)) => {
                let name = &c.name;
                quote!(let #r = ::prebindgen_jni_runtime::check_domain(#r, #lo, #hi, #name)?;)
            }
            None => TokenStream::new(),
        }
    }

    // ── Rust encode ─────────────────────────────────────────────────────

    /// A value of `ty` (the expression `value`, owned) → its output wires,
    /// named under `root`.
    pub(crate) fn rs_encode(&self, ty: &TypeRef, value: TokenStream, root: &str, depth: usize) -> Res<Output> {
        let rt = rt();
        let single = |g: &Self, e: TokenStream, fallible: bool| -> Res<Output> {
            let leaves = g.leaves(ty, Dir::Out)?;
            let w = Wire::new(leaf_ident(root, &leaves[0].name), leaves[0].rs());
            Ok(if fallible {
                Output::fallible(vec![w], e)
            } else {
                Output::single(w, e)
            })
        };
        Ok(match self.kind(ty)? {
            Kind::Unit => Output::none(value),
            Kind::Scalar(k) => {
                let p = scalar_prim(k).rs();
                let e = match k {
                    ScalarKind::Bool => quote!((#value as u8)),
                    _ => quote!((#value as #p)),
                };
                single(self, e, false)?
            }
            Kind::Str(_) => single(
                self,
                quote!(#rt::new_string(env, ::core::convert::AsRef::<str>::as_ref(&#value))?),
                true,
            )?,
            Kind::Bytes(_) => single(
                self,
                quote!(#rt::write_u8s(env, ::core::convert::AsRef::<[u8]>::as_ref(&#value))?),
                true,
            )?,
            Kind::Array { elem, .. } => {
                let p = array_prim(elem);
                let (_, write) = p.array_helpers();
                let write = format_ident!("{}", write);
                let pt = p.rs();
                let conv = if elem == ScalarKind::Bool {
                    quote!(u8::from(*__x))
                } else {
                    quote!(*__x as #pt)
                };
                single(
                    self,
                    quote!(#rt::#write(env, &#value.iter().map(|__x| #conv).collect::<::std::vec::Vec<_>>())?),
                    true,
                )?
            }
            Kind::Handle { borrow, .. } => {
                let e = match borrow {
                    Borrow::Own => quote!(#rt::new_handle(#value)),
                    _ => quote!(#rt::new_handle(::core::clone::Clone::clone(#value))),
                };
                single(self, e, false)?
            }
            Kind::Enum(class) => {
                let t = self.q.path(&names::ident(&class.rust));
                let arms = self.enum_arms(&class)?;
                let pats = arms.iter().map(|(n, d)| quote!(#t::#n => #d));
                single(self, quote!((match #value { #(#pats),* } as i32)), false)?
            }
            Kind::Record(class, s) => {
                let head = self.q.path(&names::ident(&class.rust));
                record_out(Record::Struct(s), &head, &value, |f, b| {
                    self.rs_encode(&f.ty, b.clone(), &join(root, &field_seg(f)), depth + 1)
                })?
            }
            Kind::Sum(class, v) => {
                let head = self.q.path(&names::ident(&class.rust));
                let tag = leaf_ident(root, "_tag");
                // Every alternative's wires, in order; an arm fills its own
                // and defaults the rest.
                let mut groups: Vec<Vec<Leaf>> = Vec::new();
                for alt in &v.alternatives {
                    let aseg = names::snake(&names::bare(&alt.name));
                    let mut g = Vec::new();
                    for f in &alt.fields {
                        let seg = join(root, &join(&aseg, &field_seg(f)));
                        g.extend(self.leaves(&f.ty, Dir::Out)?.into_iter().map(|l| l.under(&seg)));
                    }
                    groups.push(g);
                }
                let mut wires = vec![Wire::new(tag.clone(), Prim::I.rs())];
                for g in &groups {
                    wires.extend(g.iter().map(|l| Wire::new(names::ident(&l.name), l.rs())));
                }
                let mut arms = Vec::new();
                let mut fallible = false;
                for (i, alt) in v.alternatives.iter().enumerate() {
                    let aseg = names::snake(&names::bare(&alt.name));
                    let record = Record::Alt(alt);
                    let binds = record.binds();
                    let an = &alt.name;
                    let pat = record.pattern(&quote!(#head::#an), &binds);
                    let mut outs = Vec::new();
                    for (f, b) in alt.fields.iter().zip(&binds) {
                        let seg = join(root, &join(&aseg, &field_seg(f)));
                        outs.push(self.rs_encode(&f.ty, b.to_token_stream(), &seg, depth + 1)?);
                    }
                    fallible |= outs.iter().any(|o| o.fallible);
                    let out_binds: Vec<TokenStream> = outs.iter().map(Output::bind).collect();
                    let i32lit = i as i32;
                    let mut values = vec![quote!(#i32lit)];
                    for (j, g) in groups.iter().enumerate() {
                        for l in g {
                            if j == i {
                                let n = names::ident(&l.name);
                                values.push(quote!(#n));
                            } else {
                                values.push(l.rs_default());
                            }
                        }
                    }
                    arms.push(quote!(#pat => { #(#out_binds)* (#(#values),*) }));
                }
                let out = Output::new(wires, quote!(match #value { #(#arms),* }));
                Output { fallible, ..out }
            }
            Kind::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::Out)?;
                let x = format_ident!("__x{}", depth);
                let inner_out = self.rs_encode(inner, x.to_token_stream(), root, depth + 1)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    let e = &inner_out.expr;
                    let out = Output::new(
                        inner_out.wires.clone(),
                        quote!(match #value {
                            ::core::option::Option::Some(#x) => #e,
                            ::core::option::Option::None => #rt::jni::objects::JObject::null(),
                        }),
                    );
                    Output {
                        fallible: inner_out.fallible,
                        ..out
                    }
                } else if inner_leaves.len() == 1 {
                    let p = inner_leaves[0].prim().expect("a primitive leaf");
                    let boxer = format_ident!("{}", p.box_helper());
                    let e = &inner_out.expr;
                    let w = Wire::new(leaf_ident(root, ""), quote!(#rt::jni::objects::JObject<'a>));
                    Output::fallible(
                        vec![w],
                        quote!(match #value {
                            ::core::option::Option::Some(#x) => #rt::#boxer(env, #e)?,
                            ::core::option::Option::None => #rt::jni::objects::JObject::null(),
                        }),
                    )
                } else {
                    let present = leaf_ident(root, "_present");
                    let mut wires = vec![Wire::new(present, Prim::Z.rs())];
                    wires.extend(inner_out.wires.clone());
                    let defaults: Vec<TokenStream> = inner_leaves.iter().map(Leaf::rs_default).collect();
                    let bind = inner_out.bind();
                    let vals = inner_out.values();
                    let names: Vec<&syn::Ident> = inner_out.wires.iter().map(|w| &w.name).collect();
                    let _ = vals;
                    let out = Output::new(
                        wires,
                        quote!(match #value {
                            ::core::option::Option::Some(#x) => { #bind (1u8, #(#names),*) }
                            ::core::option::Option::None => (0u8, #(#defaults),*),
                        }),
                    );
                    Output {
                        fallible: inner_out.fallible,
                        ..out
                    }
                }
            }
            Kind::Seq { elem, form } => {
                let n = leaf_ident(root, "_n");
                let el = self.leaves(elem, Dir::Out)?;
                let er = format!("__e{depth}");
                let x = format_ident!("__x{}", depth);
                let elem_out = self.rs_encode(elem, x.to_token_stream(), &er, depth + 1)?;
                let mut wires = vec![Wire::new(n.clone(), Prim::I.rs())];
                let mut setup = Vec::new();
                let mut pushes = Vec::new();
                let mut finals = vec![quote!(__n as i32)];
                for (k, l) in el.iter().enumerate() {
                    wires.push(Wire::new(leaf_ident(root, &l.name), l.column().rs()));
                    let local = leaf_ident(&er, &l.name);
                    let col = format_ident!("__c{}_{}", depth, k);
                    match l.prim() {
                        Some(p) => {
                            let (_, write) = p.array_helpers();
                            let write = format_ident!("{}", write);
                            setup.push(quote!(let mut #col = ::std::vec::Vec::with_capacity(__n);));
                            pushes.push(quote!(#col.push(#local);));
                            finals.push(quote!(#rt::#write(env, &#col)?));
                        }
                        None => {
                            setup.push(quote!(let #col = #rt::new_object_array(env, __n)?;));
                            pushes.push(quote!(#rt::object_array_set(env, &#col, __i, #local)?;));
                            finals.push(quote!(#col));
                        }
                    }
                }
                let iter = match form {
                    Form::Owned => quote!(::core::iter::IntoIterator::into_iter(#value)),
                    Form::Cow => quote!(::core::iter::IntoIterator::into_iter(#value.into_owned())),
                    Form::Slice | Form::RefVec => quote!(#value.iter().cloned()),
                };
                let bind = elem_out.bind();
                Output::fallible(
                    wires,
                    quote!({
                        let __it = #iter;
                        let __items: ::std::vec::Vec<_> = __it.collect();
                        let __n = __items.len();
                        #(#setup)*
                        for (__i, #x) in __items.into_iter().enumerate() {
                            #bind
                            #(#pushes)*
                        }
                        (#(#finals),*)
                    }),
                )
            }
            Kind::Ref { inner, .. } => {
                self.rs_encode(inner, quote!(::core::clone::Clone::clone(#value)), root, depth)?
            }
            Kind::Boxed(inner) => self.rs_encode(inner, quote!((*#value)), root, depth)?,
            Kind::Cow(inner) => self.rs_encode(inner, quote!(#value.into_owned()), root, depth)?,
            Kind::Converted(c) => {
                let stage = c
                    .resolved
                    .output
                    .as_ref()
                    .ok_or_else(|| crate::Error(format!("convert!({}) has no output", c.name)))?;
                let r = format_ident!("__r{}", depth);
                let applied = stage.apply(&self.q, &c.resolved.target, &value);
                let check = self.domain_check(&c, &r);
                let repr = self.rs_encode(&stage.repr, r.to_token_stream(), root, depth + 1)?;
                let e = &repr.expr;
                let fallible = repr.fallible || stage.fallible || c.range.is_some();
                Output {
                    wires: repr.wires.clone(),
                    expr: quote!({ let #r = #applied; #check #e }),
                    fallible,
                }
            }
            Kind::Callback(_) => return err(format!("`{ty}`: a callback cannot leave Rust")),
        })
    }

    /// Whether a value of `ty`, held in Kotlin, owns a native handle (and so
    /// must be closed).
    pub(crate) fn owns_handle(&self, ty: &TypeRef) -> Res<bool> {
        Ok(match self.kind(ty)? {
            Kind::Handle { borrow, .. } => borrow == Borrow::Own || borrow == Borrow::Shared,
            Kind::Record(_, s) => {
                let mut any = false;
                for f in &s.fields {
                    any |= self.owns_handle(&f.ty)?;
                }
                any
            }
            Kind::Sum(_, v) => {
                let mut any = false;
                for a in &v.alternatives {
                    for f in &a.fields {
                        any |= self.owns_handle(&f.ty)?;
                    }
                }
                any
            }
            Kind::Option(t) | Kind::Seq { elem: t, .. } | Kind::Ref { inner: t, .. } | Kind::Boxed(t) | Kind::Cow(t) => {
                self.owns_handle(t)?
            }
            _ => false,
        })
    }

    /// The Kotlin statement closing `expr` (of `ty`), when it owns handles.
    pub(crate) fn kt_close(&self, ty: &TypeRef, expr: &str) -> Res<Option<String>> {
        if !self.owns_handle(ty)? {
            return Ok(None);
        }
        Ok(Some(match self.kind(ty)? {
            Kind::Option(_) => format!("{expr}?.close()"),
            Kind::Seq { .. } => format!("{expr}.forEach {{ it.close() }}"),
            _ => format!("{expr}.close()"),
        }))
    }
}
