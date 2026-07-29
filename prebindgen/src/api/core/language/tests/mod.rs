//! The language's own test suite, in two halves:
//!
//! * [`roundtrip`] — every element's syntax slices re-emit what the source
//!   wrote. This is the property the whole design rests on: if a slice were
//!   rebuilt rather than kept, the classification would have to be lossless and
//!   would grow back into a second `syn`.
//! * [`acceptance`] — source spelling → element, or a diagnosis naming the item
//!   and the component. The matrix issue #211 asks for.

use quote::ToTokens;

use super::*;

mod acceptance;
mod roundtrip;

/// Parse one item, stamped with an origin crate so array extents can name
/// `#[prebindgen]` consts from "their own" crate.
fn parse_one(item: syn::Item) -> Element {
    let mut out = parse(vec![item]);
    assert_eq!(out.len(), 1);
    out.remove(0)
}

/// Parse a whole stream, all items stamped with the same origin crate.
fn parse(items: Vec<syn::Item>) -> Vec<Element> {
    try_parse(items).expect("stream parses")
}

fn try_parse(items: Vec<syn::Item>) -> Result<Vec<Element>, ParseError> {
    Language::new().parse(items.into_iter().map(|i| (i, loc())))
}

fn loc() -> SourceLocation {
    SourceLocation {
        file: "src/lib.rs".to_string(),
        line: 1,
        column: 1,
        crate_name: Some("myflat".to_string()),
    }
}

/// `pub const TAG_LEN: usize = 4;` — the const an array extent may name.
fn tag_len_const() -> syn::Item {
    syn::parse_quote!(
        pub const TAG_LEN: usize = 4;
    )
}

/// Whitespace-insensitive token comparison, so a test states what the tokens
/// are rather than how they were spaced.
fn tokens(t: &impl ToTokens) -> String {
    t.to_token_stream().to_string()
}

/// The element as a [`Function`], or a panic naming what it actually is.
fn as_fn(e: &Element) -> &Function {
    match e {
        Element::Function(f) => f,
        other => panic!("expected a function, got {}", describe(other)),
    }
}

fn as_struct(e: &Element) -> &Struct {
    match e {
        Element::Struct(s) => s,
        other => panic!("expected a struct, got {}", describe(other)),
    }
}

fn as_enum(e: &Element) -> &Enum {
    match e {
        Element::Enum(en) => en,
        other => panic!("expected an enum, got {}", describe(other)),
    }
}

fn as_const(e: &Element) -> &Const {
    match e {
        Element::Const(c) => c,
        other => panic!("expected a const, got {}", describe(other)),
    }
}

/// The diagnosis of an [`Element::Unsupported`], or a panic naming what the
/// element actually is — so a test that expected a refusal and got an
/// acceptance says so.
fn as_unsupported(e: &Element) -> &ItemError {
    match e {
        Element::Unsupported(u) => &u.error,
        other => panic!("expected an unsupported item, got {}", describe(other)),
    }
}

fn describe(e: &Element) -> String {
    match e {
        Element::Function(f) => format!("function `{}`", f.name),
        Element::Struct(s) => format!("struct `{}`", s.name),
        Element::Enum(en) => format!("enum `{}`", en.name),
        Element::Const(c) => format!("const `{}`", c.name),
        Element::Unsupported(u) => match &u.name {
            Some(name) => format!("unsupported `{name}` ({})", u.error),
            None => format!("unsupported ({})", u.error),
        },
    }
}
