//! Nullable JNI occurrence metadata and emission.
use super::*;
pub(crate) fn absent(w: &JniWire, niche: Option<&JniNiche>, input: bool) -> Result<TokenStream> {
    // Prepared reference outputs are Option<String>/Option<Vec<u8>>. None
    // becomes a Java null at delivery; Some(empty) stays a non-null object.
    if matches!(
        w,
        JniWire::Rows { .. } | JniWire::Text { .. } | JniWire::Bytes { .. }
    ) {
        return Ok(if input {
            quote!(Default::default())
        } else {
            quote!(None)
        });
    }
    if !input && matches!(w, JniWire::Handle { .. }) {
        return Ok(quote!(std::ptr::null_mut()));
    }
    if let Some(n) = niche {
        return Ok(n.integer_value());
    }
    Ok(match w {
        JniWire::Scalar {
            primitive: Primitive::F32 | Primitive::F64,
            ..
        } => quote!(0.0),
        JniWire::Scalar {
            primitive: Primitive::I128 | Primitive::U128,
            ..
        } if !input => quote!([0u8; 16]),
        JniWire::Scalar {
            primitive: Primitive::I128 | Primitive::U128,
            ..
        } => {
            quote!(Default::default())
        }
        _ => quote!(0),
    })
}
pub(crate) fn default_kt(w: &JniWire, niche: Option<&JniNiche>) -> String {
    if matches!(niche, Some(JniNiche::Null))
        && matches!(
            w,
            JniWire::Rows { .. } | JniWire::Text { .. } | JniWire::Bytes { .. }
        )
    {
        return "null".into();
    }
    if let Some(n) = niche {
        let v = match n {
            JniNiche::Null => 0,
            JniNiche::Integer(n) => *n,
        };
        return w.kotlin_integer(v);
    }
    w.kotlin_default().into()
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
    nodes: &prebindgen_flat::WireNode<
        JniWire,
        crate::JniOptional,
        crate::JniSequence,
        JniGroup,
        (),
    >,
    o: &JniOptional,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let root = &o.presence().to_string();
    let inner = kotlin::leaf_wires(nodes);
    let mut out = KotlinInput::default();
    let mut c = kotlin::input_kotlin(nodes, context)?;
    // A single converted value is idiomatic T?. Multiple views retain an
    // explicit presence argument so Some with nullable components is distinct.
    let presence = if c.parameters.len() == 1 && !kotlin::has_optional(nodes) {
        let (name, ty) = c.parameters[0].split_once(": ").unwrap();
        let name = name.to_string();
        let ty = ty.trim_end_matches('?');
        out.parameters.push(format!("{name}: {ty}?"));
        for a in c
            .arguments
            .iter_mut()
            .chain(c.locks.iter_mut())
            .chain(c.consumed.iter_mut())
            .chain(c.lock_groups.iter_mut())
            .chain(c.setup.iter_mut().map(|b| &mut b.expression))
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
        .chain(c.lock_groups.iter_mut())
        .chain(c.setup.iter_mut().map(|b| &mut b.expression))
    {
        // Every payload expression below runs only in the present branch.
        *a = a.replace(&format!("{} == 1", root), "true");
        *a = replace(a, root, "1");
    }
    if matches!(o, JniOptional::Flag(_)) {
        out.arguments.push(tag);
    }
    for (j, a) in c.arguments.into_iter().enumerate() {
        let niche = o.niche().filter(|_| inner[j].name == *o.discriminator());
        let default = default_kt(&inner[j].kind, niche);
        let a = match niche {
            Some(JniNiche::Integer(_)) => format!("run {{ val value = {a}; require(value != {default}) {{ \"invalid option payload\" }}; value }}"),
            Some(JniNiche::Null) if matches!(inner[j].kind, JniWire::Rows { .. } | JniWire::Text { .. } | JniWire::Bytes { .. }) => {
                // An explicit presence parameter must not silently become None
                // when a nullable raw reference is passed for its discriminator.
                format!("requireNotNull({a}) {{ \"invalid option payload\" }}")
            }
            _ => a,
        };
        out.arguments
            .push(format!("if ({presence}) {a} else {default}"));
    }
    out.setup.extend(c.setup.into_iter().map(|mut b| {
        b.expression = format!("if ({presence}) ({}) else ({})", b.expression, b.absent);
        b
    }));
    out.lock_groups.extend(
        c.lock_groups
            .into_iter()
            .map(|s| format!("if ({presence}) ({s}) else kotlin.collections.emptyList()")),
    );
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
    Ok(out)
}
pub(crate) fn output_kotlin(
    nodes: &prebindgen_flat::WireNode<
        JniWire,
        crate::JniOptional,
        crate::JniSequence,
        JniGroup,
        (),
    >,
    o: &JniOptional,
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let wires = kotlin::leaf_wires(nodes);
    let flag = usize::from(matches!(o, JniOptional::Flag(_)));
    let p = if let Some(n) = o.niche() {
        let i = wires
            .iter()
            .position(|w| w.name == *o.discriminator())
            .ok_or("missing optional discriminator")?;
        let absent = default_kt(&wires[i].kind, Some(n));
        format!("{}[{i}] != {absent}", context.result)
    } else {
        format!("{}[0] == 1", context.result)
    };
    let local = format!("{}_payload", context.result);
    let ctx = KotlinContext {
        result: &local,
        ..*context
    };
    let mut c = kotlin::output_kotlin(nodes, &ctx)?;
    kotlin::remap_output(&mut c, &local, context.result, flag, wires.len());
    let mut out = KotlinOutput {
        output_locals: c.output_locals,
        ..Default::default()
    };
    if c.delivered.is_empty() || kotlin::has_optional(nodes) {
        out.delivered.push(p.clone());
        out.result_types.push("Boolean".into());
    }
    out.materialize = format!("            if ({p}) {{\n{}            }}\n", c.materialize);
    out.cleanup = format!("            if ({p}) {{\n{}            }}\n", c.cleanup);
    for (v, t) in c.delivered.into_iter().zip(c.result_types) {
        out.delivered.push(format!("if ({p}) {v} else null"));
        out.result_types
            .push(format!("{}?", t.trim_end_matches('?')));
    }
    Ok(out)
}

/// A wire value excluded from the representation of a present value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JniNiche {
    /// Null reference or address (including integer-encoded handles).
    Null,
    /// An invalid integer representation, within the wire's integer range.
    Integer(i64),
}
impl JniNiche {
    /// Expression recognizing the present case without decoding the payload.
    pub(crate) fn present(&self, wire: &JniWire, value: TokenStream) -> TokenStream {
        match self {
            Self::Null
                if matches!(
                    wire,
                    JniWire::Rows { .. } | JniWire::Text { .. } | JniWire::Bytes { .. }
                ) =>
            {
                quote!(!#value.is_null())
            }
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

pub(crate) fn remaining(wire: &JniWire, niche: &JniNiche) -> Option<JniNiche> {
    wire.validate_niche(niche).ok()?;
    let JniNiche::Integer(n) = niche else {
        return None;
    };
    if let JniWire::Enum { enum_type, .. } = wire {
        return i32::try_from(*n)
            .ok()?
            .checked_sub(1)
            .and_then(|n| enum_type.mapping.unused_at_or_below(n))
            .map(|n| JniNiche::Integer(n.into()));
    }
    let next = JniNiche::Integer(n.checked_sub(1)?);
    // Stop at the carrier boundary or the next valid payload value. Do not
    // advertise a reserved range's neighbour merely because it fits an integer.
    wire.validate_niche(&next).ok().map(|()| next)
}
