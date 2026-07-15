//! Shared internal utilities used by multiple modules.

/// Convert a `snake_case` Rust identifier name to `camelCase`.
pub(crate) fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else if i == 0 {
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Extract an item's `///` documentation from its captured attributes
/// (`#[doc = " …"]` lines, in order): one leading space stripped per line,
/// joined with `\n`; `None` when the item carries no docs. `*/` is
/// defanged so the text is always safe inside a `/** … */` KDoc block.
pub(crate) fn doc_string(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        else {
            continue;
        };
        let raw = s.value();
        let line = raw.strip_prefix(' ').unwrap_or(&raw);
        lines.push(line.replace("*/", "*\u{200B}/"));
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Convert a `CamelCase` Rust identifier to `SCREAMING_SNAKE_CASE`. Used to
/// project Rust enum variant idents into Kotlin enum constant names.
pub(crate) fn camel_to_screaming_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_uppercase());
    }
    out
}

/// True iff `ty` is the unit type `()`.
pub(crate) fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// Pull a signed integer out of a `syn::Expr` literal (`5`, `-3`,
/// `0x07`). Returns `None` for anything else (constants, paths,
/// arithmetic).
pub(crate) fn extract_int_literal(expr: &syn::Expr) -> Option<i64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.base10_parse::<i64>().ok(),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => extract_int_literal(expr).map(|v| -v),
        _ => None,
    }
}

/// Resolve each enum variant to its discriminant value following Rust's
/// own assignment rule: an explicit `= N` sets the value, an implicit
/// variant takes the previous value plus one (starting at 0). This is
/// the single source of truth for both the Kotlin `value(N)` constants
/// and the generated Rust `jint → variant` decode — keeping the two
/// from drifting and removing the need for a hand-written
/// `TryFrom<i32>` on the flat enum. Non-literal discriminants are
/// rejected because prebindgen-ext cannot reliably evaluate arbitrary
/// expressions at codegen time.
pub(crate) fn enum_discriminant_values(e: &syn::ItemEnum) -> Vec<(syn::Ident, i64)> {
    let mut out = Vec::with_capacity(e.variants.len());
    let mut next: i64 = 0;
    for variant in &e.variants {
        let value = match variant.discriminant.as_ref() {
            Some((_, expr)) => extract_int_literal(expr).unwrap_or_else(|| {
                panic!(
                    "enum `{}` variant `{}` has a non-literal discriminant; use a literal integer value (e.g. `= 1`) or an implicit discriminant",
                    e.ident,
                    variant.ident
                )
            }),
            None => next,
        };
        out.push((variant.ident.clone(), value));
        next = value + 1;
    }
    out
}

#[cfg(test)]
mod tests;
