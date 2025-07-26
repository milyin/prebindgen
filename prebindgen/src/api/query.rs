//! Query utilities for extracting information from syn::Item structures

use quote::ToTokens;

/// Extract alignment value from a struct's repr attribute
pub fn struct_align(item: &syn::Item) -> Option<u32> {
    if let syn::Item::Struct(s) = item {
        s.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("repr") {
                let tokens_str = attr.meta.to_token_stream().to_string();
                if let Some(align_pos) = tokens_str.find("align") {
                    let after_align = &tokens_str[align_pos + 5..];
                    let start = after_align.find('(')?;
                    let end = after_align.find(')')?;
                    after_align[start + 1..end]
                        .trim()
                        .parse().ok()
                } else { None }
            } else { None }
        })
    } else { None }
}