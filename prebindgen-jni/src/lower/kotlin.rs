//! The Kotlin side of the codec: a value taken apart into leaf expressions,
//! and put back together from them.

use std::rc::Rc;

use prebindgen_flat::flat::{ScalarKind, TypeRef};
use prebindgen_tools::{Access, Shape};

use super::{alt_kt_name, is_u8, kt_prop, scalar_prim, Dir};
use crate::plan::{err, Class, ClassKind, Plan, Res, Setting};

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
}

impl Plan<'_> {
    /// Kotlin expressions for the input leaves of `ty`, from `expr` (which is
    /// `null`-able when `nullable`). A handle is `consumed` when the call
    /// takes it by value.
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
        Ok(match self.shape(ty)? {
            Shape::Unit => Vec::new(),
            Shape::Scalar(k) => {
                let e = if k == ScalarKind::U64 {
                    access(expr, "toLong()")
                } else {
                    expr.to_string()
                };
                vec![or_default(e, scalar_prim(k).kt_default())]
            }
            Shape::Str(_) | Shape::Array { .. } => vec![expr.to_string()],
            Shape::Seq { elem, .. } if is_u8(elem) => vec![expr.to_string()],
            Shape::Declared {
                setting, access: a, ..
            } => match setting {
                Setting::Converted(c) => {
                    self.kt_encode(self.conv_repr(c, Dir::In)?, expr, nullable, cx, consumed)?
                }
                Setting::Class(c) => match &c.kind {
                    ClassKind::Ptr { .. } => {
                        cx.handles.push(HandleSite {
                            expr: expr.to_string(),
                            nullable,
                            consumed: consumed && a == Access::Owned,
                            class: c.clone(),
                        });
                        vec![or_default(access(expr, "ptr"), "0L")]
                    }
                    ClassKind::Enum => vec![or_default(access(expr, "value"), "0")],
                    ClassKind::Data { .. } => {
                        let mut out = Vec::new();
                        for f in &self.struct_of(c)?.fields {
                            let e = access(expr, &kt_prop(f));
                            out.extend(self.kt_encode(&f.ty, &e, nullable, cx, consumed)?);
                        }
                        out
                    }
                    ClassKind::Sealed { .. } => {
                        let v = self.variant_of(c)?;
                        let mut arms: Vec<String> = Vec::new();
                        if nullable {
                            arms.push("null -> 0".to_string());
                        }
                        for (i, alt) in v.alternatives.iter().enumerate() {
                            arms.push(format!("is {}.{} -> {i}", c.fqn(), alt_kt_name(c, alt)));
                        }
                        let mut out = vec![format!("when ({expr}) {{ {} }}", arms.join("; "))];
                        for alt in &v.alternatives {
                            let cast = format!("({expr} as? {}.{})", c.fqn(), alt_kt_name(c, alt));
                            for f in &alt.fields {
                                let e = format!("{cast}?.{}", kt_prop(f));
                                out.extend(self.kt_encode(&f.ty, &e, true, cx, consumed)?);
                            }
                        }
                        out
                    }
                },
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::In)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    self.kt_encode(inner, expr, true, cx, consumed)?
                } else {
                    let mut out = vec![format!("({expr} != null)")];
                    out.extend(self.kt_encode(inner, expr, true, cx, consumed)?);
                    out
                }
            }
            Shape::Seq { elem, .. } => {
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
                    return err(format!(
                        "`{ty}`: a sequence of handles cannot cross into Rust"
                    ));
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
                let (src, guard) = if nullable {
                    (format!("{list}!!"), format!("if ({list} != null) "))
                } else {
                    (list.clone(), String::new())
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
            Shape::Ref { inner, .. } | Shape::Boxed(inner) | Shape::Cow(inner) => {
                self.kt_encode(inner, expr, nullable, cx, consumed)?
            }
            Shape::Callback(_) => vec![access(expr, "asRaw()")],
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }

    /// A Kotlin expression building a value of `ty` from its output leaves'
    /// expressions. `gated` marks leaves whose object values Kotlin sees as
    /// nullable although the value itself is present (an alternative's
    /// group, an optional's inner value).
    pub(crate) fn kt_decode(
        &self,
        ty: &TypeRef,
        leaves: &[String],
        gated: bool,
        depth: usize,
    ) -> Res<String> {
        let bang = |e: &str| {
            if gated {
                format!("{e}!!")
            } else {
                e.to_string()
            }
        };
        Ok(match self.shape(ty)? {
            Shape::Unit => "Unit".to_string(),
            Shape::Scalar(ScalarKind::U64) => format!("{}.toULong()", leaves[0]),
            Shape::Scalar(_) => leaves[0].clone(),
            Shape::Str(_) | Shape::Array { .. } => bang(&leaves[0]),
            Shape::Seq { elem, .. } if is_u8(elem) => bang(&leaves[0]),
            Shape::Declared { setting, .. } => match setting {
                Setting::Converted(c) => {
                    self.kt_decode(self.conv_repr(c, Dir::Out)?, leaves, gated, depth)?
                }
                Setting::Class(c) => match &c.kind {
                    ClassKind::Ptr { .. } => format!("{}({})", c.fqn(), leaves[0]),
                    ClassKind::Enum => format!("{}.fromInt({})", c.fqn(), leaves[0]),
                    ClassKind::Data { .. } => {
                        let mut args = Vec::new();
                        let mut at = 0;
                        for f in &self.struct_of(c)?.fields {
                            let n = self.leaves(&f.ty, Dir::Out)?.len();
                            args.push(self.kt_decode(&f.ty, &leaves[at..at + n], gated, depth)?);
                            at += n;
                        }
                        format!("{}({})", c.fqn(), args.join(", "))
                    }
                    ClassKind::Sealed { .. } => {
                        let tag = &leaves[0];
                        let mut at = 1;
                        let mut arms = Vec::new();
                        for (i, alt) in self.variant_of(c)?.alternatives.iter().enumerate() {
                            let cname = format!("{}.{}", c.fqn(), alt_kt_name(c, alt));
                            if alt.fields.is_empty() {
                                arms.push(format!("{i} -> {cname}"));
                                continue;
                            }
                            let mut args = Vec::new();
                            for f in &alt.fields {
                                let n = self.leaves(&f.ty, Dir::Out)?.len();
                                args.push(self.kt_decode(
                                    &f.ty,
                                    &leaves[at..at + n],
                                    true,
                                    depth,
                                )?);
                                at += n;
                            }
                            arms.push(format!("{i} -> {cname}({})", args.join(", ")));
                        }
                        arms.push(format!(
                            "else -> throw IllegalArgumentException(\"{}: invalid tag ${tag}\")",
                            c.name
                        ));
                        format!("when ({tag}) {{ {} }}", arms.join("; "))
                    }
                },
            },
            Shape::Option(inner) => {
                if self.leaves(inner, Dir::Out)?.len() == 1 {
                    let it = format!("__o{depth}");
                    let d = self.kt_decode(inner, std::slice::from_ref(&it), false, depth + 1)?;
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
            Shape::Seq { elem, .. } => {
                let n = &leaves[0];
                let i = format!("__i{depth}");
                let bang = if gated { "!!" } else { "" };
                let items: Vec<String> = self
                    .leaves(elem, Dir::Out)?
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
            Shape::Ref { inner, .. } | Shape::Boxed(inner) | Shape::Cow(inner) => {
                self.kt_decode(inner, leaves, gated, depth)?
            }
            Shape::Callback(_) => return err(format!("`{ty}`: a callback cannot leave Rust")),
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }

    /// The Kotlin statement closing `expr` (of `ty`), when it owns handles.
    pub(crate) fn kt_close(&self, ty: &TypeRef, expr: &str) -> Res<Option<String>> {
        if !self.owns_handle(ty)? {
            return Ok(None);
        }
        Ok(Some(match self.shape(ty)? {
            Shape::Option(_) => format!("{expr}?.close()"),
            Shape::Seq { .. } => format!("{expr}.forEach {{ it.close() }}"),
            _ => format!("{expr}.close()"),
        }))
    }
}
