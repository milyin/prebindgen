//! Nullable JNI occurrence metadata and emission.
use super::*;
use prebindgen_flat::Wire;
pub(crate) fn niche(w: &JniKind) -> Option<JniNiche> {
    match w {
        JniKind::Handle(..) => Some(JniNiche::Null),
        JniKind::Scalar(Primitive::Char | Primitive::U8 | Primitive::U16 | Primitive::U32) => {
            Some(JniNiche::Integer(-1))
        }
        _ => None,
    }
}
pub(crate) fn absent(w: &JniKind, niche: Option<&JniNiche>, input: bool) -> Result<TokenStream> {
    if !input && matches!(w, JniKind::Handle(..)) {
        return Ok(quote!(std::ptr::null_mut()));
    }
    if let Some(n) = niche {
        return Ok(n.integer_value());
    }
    Ok(match w {
        JniKind::Scalar(Primitive::F32 | Primitive::F64) => quote!(0.0),
        JniKind::Text if !input => quote!(String::new()),
        JniKind::Bytes if !input => quote!(Vec::<u8>::new()),
        JniKind::Scalar(Primitive::I128 | Primitive::U128) if !input => quote!([0u8; 16]),
        JniKind::Text | JniKind::Bytes | JniKind::Scalar(Primitive::I128 | Primitive::U128) => {
            quote!(Default::default())
        }
        _ => quote!(0),
    })
}
pub(crate) fn default_kt(w: &JniKind, niche: Option<&JniNiche>) -> String {
    if let Some(n) = niche {
        let v = match n {
            JniNiche::Null => 0,
            JniNiche::Integer(n) => *n,
        };
        return if matches!(
            w,
            JniKind::Handle(..)
                | JniKind::Scalar(
                    Primitive::U32
                        | Primitive::I64
                        | Primitive::U64
                        | Primitive::Isize
                        | Primitive::Usize
                )
        ) {
            format!("{v}L")
        } else {
            v.to_string()
        };
    }
    match w {
        JniKind::Scalar(Primitive::Bool) => "false",
        JniKind::Scalar(Primitive::I8) => "0.toByte()",
        JniKind::Scalar(Primitive::I16) => "0.toShort()",
        JniKind::Handle(..)
        | JniKind::Scalar(
            Primitive::U32 | Primitive::I64 | Primitive::U64 | Primitive::Isize | Primitive::Usize,
        ) => "0L",
        JniKind::Scalar(Primitive::F32) => "0.0f",
        JniKind::Scalar(Primitive::F64) => "0.0",
        JniKind::Text | JniKind::Bytes | JniKind::Scalar(Primitive::I128 | Primitive::U128) => {
            "null"
        }
        _ => "0",
    }
    .into()
}
// Replace generated identifiers only, not substrings of other generated names.
pub(crate) fn replace(s: &str, from: &str, to: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for c in s.chars().chain(std::iter::once(' ')) {
        if c.is_ascii_alphanumeric() || c == '_' {
            word.push(c)
        } else {
            if word == from {
                out.push_str(to)
            } else {
                out.push_str(&word)
            }
            word.clear();
            out.push(c);
        }
    }
    out.pop();
    out
}
pub(crate) fn input_kotlin(
    wires: &[Wire<JniKind>],
    metadata: &JniComposition,
    roots: &[String],
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let mut out = KotlinInput::default();
    let mut i = 0;
    while i < wires.len() {
        let Some(group) = metadata.starting_at(roots, &wires[i].name) else {
            let end = (i + 1..wires.len())
                .find(|j| metadata.starting_at(roots, &wires[*j].name).is_some())
                .unwrap_or(wires.len());
            let c = kotlin::input_kotlin(&wires[i..end], metadata, &[], context)?;
            out.parameters.extend(c.parameters);
            out.arguments.extend(c.arguments);
            out.locks.extend(c.locks);
            out.consumed.extend(c.consumed);
            i = end;
            continue;
        };
        let prebindgen_declarations::GroupKind::Optional(o) = &group.kind else {
            return Err("expected optional group".into());
        };
        let root = &group.name;
        let flag = usize::from(o.niche.is_none());
        let inner = &wires[i + flag..i + group.members.len()];
        let mut c = kotlin::input_kotlin(inner, metadata, &group.children, context)?;
        // A single converted value is idiomatic T?. Multiple views retain an
        // explicit presence argument so Some with nullable components is distinct.
        let presence = if c.parameters.len() == 1
            && !group.children.iter().any(|id| {
                matches!(
                    metadata.groups[id].kind,
                    prebindgen_declarations::GroupKind::Optional(_)
                )
            }) {
            let (name, ty) = c.parameters[0].split_once(": ").unwrap();
            let name = name.to_string();
            let ty = ty.trim_end_matches('?');
            out.parameters.push(format!("{name}: {ty}?"));
            for a in c
                .arguments
                .iter_mut()
                .chain(c.locks.iter_mut())
                .chain(c.consumed.iter_mut())
            {
                *a = a
                    .replace(&format!("{name}?.raw ?: 0L"), &format!("{name}.raw"))
                    .replace(&format!("{name}?."), &format!("{name}."))
                    .replace(&format!("{name}!!"), &name);
                *a = replace(a, &name, &format!("requireNotNull({name})"));
            }
            format!("{name} != null")
        } else {
            out.parameters.push(format!("{}: Boolean", root));
            out.parameters.extend(c.parameters.clone());
            root.to_string()
        };
        let tag = format!("(if ({presence}) 1 else 0)");
        for a in c
            .arguments
            .iter_mut()
            .chain(c.locks.iter_mut())
            .chain(c.consumed.iter_mut())
        {
            // Every payload expression below runs only in the present branch.
            *a = a.replace(&format!("{} == 1", root), "true");
            *a = replace(a, root, "1");
        }
        if o.niche.is_none() {
            out.arguments.push(tag);
        }
        for (j, a) in c.arguments.into_iter().enumerate() {
            let niche = o
                .niche
                .as_ref()
                .filter(|_| inner[j].name == o.discriminator);
            let default = default_kt(&inner[j].kind, niche);
            let a = if matches!(niche, Some(JniNiche::Integer(_))) {
                format!("run {{ val value = {a}; require(value != {default}) {{ \"invalid option payload\" }}; value }}")
            } else {
                a
            };
            out.arguments
                .push(format!("if ({presence}) {a} else {default}"));
        }
        out.locks.extend(
            c.locks
                .into_iter()
                .map(|s| format!("if ({presence}) {s} else null")),
        );
        out.consumed.extend(
            c.consumed
                .into_iter()
                .map(|s| format!("if ({presence}) {{ {s} }}")),
        );
        i += inner.len() + usize::from(o.niche.is_none());
    }
    Ok(out)
}
pub(crate) fn output_kotlin(
    wires: &[Wire<JniKind>],
    metadata: &JniComposition,
    roots: &[String],
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let mut out = KotlinOutput {
        delivered: vec![],
        result_types: vec![],
        output_locals: String::new(),
        materialize: String::new(),
        cleanup: String::new(),
    };
    let mut i = 0;
    while i < wires.len() {
        let (inner, count, offset, present, scope) =
            if let Some(group) = metadata.starting_at(roots, &wires[i].name) {
                let prebindgen_declarations::GroupKind::Optional(o) = &group.kind else {
                    return Err("expected optional group".into());
                };
                let flag = usize::from(o.niche.is_none());
                let present = if let Some(n) = &o.niche {
                    let j = wires[i..i + group.members.len()]
                        .iter()
                        .position(|w| w.name == o.discriminator)
                        .unwrap();
                    let v = default_kt(&wires[i + j].kind, Some(n));
                    format!("{}[{}] != {v}", context.result, i + j)
                } else {
                    format!("{}[{i}] == 1", context.result)
                };
                (
                    &wires[i + flag..i + group.members.len()],
                    group.members.len(),
                    i + flag,
                    Some(present),
                    group.children.as_slice(),
                )
            } else {
                let end = (i + 1..wires.len())
                    .find(|j| metadata.starting_at(roots, &wires[*j].name).is_some())
                    .unwrap_or(wires.len());
                (&wires[i..end], end - i, i, None, &[][..])
            };
        let local = format!("{}_segment{i}", context.result);
        let prefix = format!("{}s{i}_", context.output_prefix);
        let ctx = KotlinContext {
            result: &local,
            output_prefix: &prefix,
            ..*context
        };
        let mut c = kotlin::output_kotlin(inner, metadata, scope, &ctx)?;
        for text in
            c.delivered
                .iter_mut()
                .chain([&mut c.output_locals, &mut c.materialize, &mut c.cleanup])
        {
            for j in 0..inner.len() {
                *text = text.replace(
                    &format!("{local}[{j}]"),
                    &format!("{}[{}]", context.result, offset + j),
                );
            }
        }
        out.output_locals.push_str(&c.output_locals);
        if let Some(p) = present {
            if c.delivered.is_empty()
                || scope.iter().any(|id| {
                    matches!(
                        metadata.groups[id].kind,
                        prebindgen_declarations::GroupKind::Optional(_)
                    )
                })
            {
                out.delivered.push(p.clone());
                out.result_types.push("Boolean".into());
            }
            out.materialize.push_str(&format!(
                "            if ({p}) {{\n{}            }}\n",
                c.materialize
            ));
            out.cleanup.push_str(&format!(
                "            if ({p}) {{\n{}            }}\n",
                c.cleanup
            ));
            for (v, t) in c.delivered.into_iter().zip(c.result_types) {
                out.delivered.push(format!("if ({p}) {v} else null"));
                out.result_types
                    .push(format!("{}?", t.trim_end_matches('?')));
            }
        } else {
            out.materialize.push_str(&c.materialize);
            out.cleanup.push_str(&c.cleanup);
            out.delivered.extend(c.delivered);
            out.result_types.extend(c.result_types);
        }
        i += count;
    }
    Ok(out)
}

/// A wire value excluded from the representation of a present value.
#[derive(Clone, Debug)]
pub enum JniNiche {
    /// Null address (pointer or integer-encoded handle).
    Null,
    /// An invalid integer representation, within the wire's integer range.
    Integer(i64),
}
impl JniNiche {
    /// Expression recognizing the present case without decoding the payload.
    pub(crate) fn present(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Null => quote!((#value as usize) != 0),
            Self::Integer(n) => {
                let n = syn::parse_str::<syn::Expr>(&n.to_string()).unwrap();
                quote!(#value != #n)
            }
        }
    }
    /// Absence expression for a numeric wire. Pointer codecs supply a null pointer.
    pub(crate) fn integer_value(&self) -> TokenStream {
        match self {
            Self::Null => quote!(0),
            Self::Integer(n) => syn::parse_str::<TokenStream>(&n.to_string()).unwrap(),
        }
    }
}

pub(crate) fn remaining(wire: &JniKind, niche: &JniNiche) -> Option<JniNiche> {
    let min = if matches!(wire, JniKind::Scalar(Primitive::U32)) {
        i64::MIN
    } else {
        i32::MIN as i64
    };
    match niche {
        JniNiche::Integer(n) if *n > min => Some(JniNiche::Integer(n - 1)),
        _ => None,
    }
}
