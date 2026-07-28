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

#[cfg(test)]
mod tests;
