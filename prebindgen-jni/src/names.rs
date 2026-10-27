//! Stable internal names derived from source identities, never declaration order.
use prebindgen_flat::Result;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Names {
    identities: BTreeMap<String, Vec<String>>,
}
impl Names {
    pub(crate) fn reserve(&mut self, name: &str, role: &str, parts: &[&str]) -> Result<()> {
        let identity = std::iter::once(role)
            .chain(parts.iter().copied())
            .map(str::to_owned)
            .collect();
        self.claim(name, identity)
    }

    fn claim(&mut self, name: &str, identity: Vec<String>) -> Result<()> {
        if let Some(previous) = self.identities.get(name) {
            if previous != &identity {
                return Err(format!(
                    "generated name hash collision for `{name}`: {previous:?} and {identity:?}"
                ));
            }
        } else {
            self.identities.insert(name.to_owned(), identity);
        }
        Ok(())
    }

    pub(crate) fn name(&mut self, role: &str, readable: &str, parts: &[&str]) -> Result<String> {
        let identity = std::iter::once(role)
            .chain(parts.iter().copied())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        // Fixed FNV-1a parameters and length-prefixed UTF-8 components make this
        // independent of Rust's Hash implementation, host width and byte order.
        let mut hash = 0xcbf29ce484222325u64;
        for part in &identity {
            for byte in (part.len() as u64)
                .to_le_bytes()
                .into_iter()
                .chain(part.bytes())
            {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        let label = readable
            .strip_prefix("r#")
            .unwrap_or(readable)
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let label = if label.chars().all(|c| c == '_') {
            "item"
        } else {
            &label
        };
        let name = format!("{role}_{label}_{hash:016x}");
        self.claim(&name, identity)?;
        Ok(name)
    }
}

// Encode tokens explicitly: proc_macro2's display whitespace is not an identity,
// and removing whitespace from printed text would corrupt string/char literals.
fn tokens_identity(tokens: TokenStream) -> String {
    fn atom(output: &mut String, tag: char, value: &str) {
        output.push(tag);
        output.push_str(&value.len().to_string());
        output.push(':');
        output.push_str(value);
    }
    let mut output = String::new();
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => atom(&mut output, 'i', &ident.to_string()),
            TokenTree::Literal(literal) => atom(&mut output, 'l', &literal.to_string()),
            TokenTree::Punct(punct) => {
                output.push('p');
                output.push(punct.as_char());
            }
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::None => {
                        output.push_str(&tokens_identity(group.stream()));
                        continue;
                    }
                };
                output.push(open);
                output.push_str(&tokens_identity(group.stream()));
                output.push(close);
            }
        }
    }
    output
}
pub(crate) fn path_identity(path: &syn::Path) -> String {
    tokens_identity(path.to_token_stream())
}
pub(crate) fn type_identity(ty: &syn::Type) -> String {
    tokens_identity(ty.to_token_stream())
}

pub(crate) fn type_label(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => {
            let segment = path.path.segments.last().unwrap();
            let mut label = segment.ident.to_string();
            if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
                for argument in &arguments.args {
                    if let syn::GenericArgument::Type(ty) = argument {
                        label.push('_');
                        label.push_str(&type_label(ty));
                    }
                }
            }
            label
        }
        syn::Type::Reference(r) => type_label(&r.elem),
        syn::Type::Tuple(t) if t.elems.is_empty() => "Unit".into(),
        syn::Type::Tuple(_) => "Tuple".into(),
        syn::Type::Array(_) => "Array".into(),
        syn::Type::Slice(_) => "Slice".into(),
        _ => "Value".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_do_not_depend_on_registration_order() {
        let mut first = Names::default();
        let original = first
            .name("call", "read", &["example :: read", "read"])
            .unwrap();
        assert_eq!(original, "call_read_2bb3c87ce2943d1c");
        first
            .name("call", "added", &["example :: added", "added"])
            .unwrap();
        let mut second = Names::default();
        second
            .name("call", "added", &["example :: added", "added"])
            .unwrap();
        assert_eq!(
            original,
            second
                .name("call", "read", &["example :: read", "read"])
                .unwrap()
        );
        assert_eq!(
            original,
            first
                .name("call", "read", &["example :: read", "read"])
                .unwrap()
        );
        assert_ne!(
            first.name("call", "test", &["ab", "c"]).unwrap(),
            first.name("call", "test", &["a", "bc"]).unwrap()
        );
        assert_ne!(
            first
                .name("drop", "read", &["example :: read", "read"])
                .unwrap(),
            original
        );
    }

    #[test]
    fn readable_names_disambiguate_equal_short_type_names() {
        let mut names = Names::default();
        let a = names.name("drop", "Handle", &["alpha :: Handle"]).unwrap();
        let b = names.name("drop", "Handle", &["beta :: Handle"]).unwrap();
        assert!(a.starts_with("drop_Handle_") && b.starts_with("drop_Handle_"));
        assert_ne!(a, b);
        assert!(names
            .name("call", "r#type", &["r#type"])
            .unwrap()
            .starts_with("call_type_"));
    }

    #[test]
    fn collisions_are_errors_instead_of_counter_suffixes() {
        let mut names = Names::default();
        let name = names
            .name("drop", "Handle", &["example :: Handle"])
            .unwrap();
        let mut names = Names::default();
        names
            .reserve(&name, "drop", &["different identity"])
            .unwrap();
        assert!(names
            .name("drop", "Handle", &["example :: Handle"])
            .unwrap_err()
            .contains("hash collision"));
    }

    #[test]
    fn token_identities_ignore_layout_and_preserve_literal_contents() {
        let a: syn::Type = syn::parse_str("Vec < [u8; 3] >").unwrap();
        let b: syn::Type = syn::parse_str("Vec<[u8;3]>").unwrap();
        assert_eq!(type_identity(&a), type_identity(&b));
        let a: syn::Type = syn::parse_str("Value<{ length(\"a b\") }>").unwrap();
        let b: syn::Type = syn::parse_str("Value<{ length(\"ab\") }>").unwrap();
        assert_ne!(type_identity(&a), type_identity(&b));
        assert_eq!(
            path_identity(&syn::parse_quote!(example::read)),
            "i7:examplep:p:i4:read"
        );
    }
}
