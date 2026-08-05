//! The language's own test suite, in two halves:
//!
//! * [`roundtrip`] — every element's syntax slices re-emit what the source
//!   wrote. This is the property the whole design rests on: if a slice were
//!   rebuilt rather than kept, the classification would have to be lossless and
//!   would grow back into a second `syn`.
//! * [`acceptance`] — source spelling → element, or a diagnosis naming the item
//!   and the component. The matrix issue #211 asks for.

use std::rc::Rc;

use quote::ToTokens;

use super::*;

mod acceptance;
mod roundtrip;

/// Lower one type by putting it in a struct field, and report what the language
/// made of it. The field path is used because a field is the position every
/// consumer already agrees is a boundary surface.
fn lower(ty: proc_macro2::TokenStream) -> Result<TypeRef, UnsupportedType> {
    let item: syn::Item = syn::parse_quote!(
        pub struct S {
            pub f: #ty,
        }
    );
    // The fixture types stand in for a declared type wherever the grammar needs
    // a nominal one, so references resolve and the test is about the grammar.
    let mut items = fixture_types();
    items.push(tag_len_const());
    items.push(opaque("Sample"));
    let n = items.len();
    items.push(item);
    match parse(items).remove(n) {
        Element::Type(Type::Struct(s)) => Ok(s.fields[0].ty.clone()),
        Element::Unsupported(u) => match *u.error {
            ItemError::FieldType { source, .. } => Err(source),
            other => panic!("expected a field-type diagnosis, got {other}"),
        },
        other => panic!("expected a struct, got {}", describe(&other)),
    }
}

/// Parse one item, stamped with an origin crate so array extents can name
/// `#[prebindgen]` consts from "their own" crate.
///
/// The fixture types are declared alongside, so a test naming `Sample` or
/// `KeyExpr` is about whatever it is testing rather than about resolution.
fn parse_one(item: syn::Item) -> Element {
    let mut items = fixture_types();
    let n = items.len();
    items.push(item);
    let mut out = parse(items);
    assert_eq!(out.len(), n + 1);
    out.remove(n)
}

/// Names that stand in for "a type exists" across the tests. Declared as opaque
/// handles, since none of them is the subject of the test that names it.
///
/// Deliberately excludes `Sample`, which several tests *declare* themselves —
/// declaring it here too would be a duplicate name. `lower` adds it, because a
/// type-grammar test only ever references it.
fn fixture_types() -> Vec<syn::Item> {
    [
        "Error",
        "KeyExpr",
        "Foo",
        "Whatever",
        "SomethingUnexpressible",
    ]
    .into_iter()
    .map(opaque)
    .collect()
}

/// Parse a whole stream, all items stamped with the same origin crate.
fn parse(items: Vec<syn::Item>) -> Vec<Element> {
    try_parse(items).expect("stream parses")
}

fn try_parse(items: Vec<syn::Item>) -> Result<Vec<Element>, ParseError> {
    Flat::builder()
        .items(items.into_iter().map(|i| (i, loc())))
        .build()
        .map(|flat| flat.elements().cloned().collect())
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

/// A marked alias declaring `name` as an opaque handle — the fixture for
/// "some type exists under this name", now that references must resolve.
fn opaque(name: &str) -> syn::Item {
    let ident = quote::format_ident!("{name}");
    syn::parse_quote!(
        pub type #ident = other::#ident;
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

fn as_type(e: &Element) -> &Type {
    match e {
        Element::Type(t) => t,
        other => panic!("expected a type, got {}", describe(other)),
    }
}

fn as_struct(e: &Element) -> &Struct {
    match as_type(e) {
        Type::Struct(s) => s,
        other => panic!("expected a struct, got {}", describe_type(other)),
    }
}

fn as_enum(e: &Element) -> &Enum {
    match as_type(e) {
        Type::Enum(en) => en,
        other => panic!("expected a fieldless enum, got {}", describe_type(other)),
    }
}

fn as_variant(e: &Element) -> &Variant {
    match as_type(e) {
        Type::Variant(v) => v,
        other => panic!("expected a sum, got {}", describe_type(other)),
    }
}

fn as_extern(e: &Element) -> &Extern {
    match as_type(e) {
        Type::Extern(e) => e,
        other => panic!("expected an extern, got {}", describe_type(other)),
    }
}

fn as_const(e: &Element) -> &Constant {
    match e {
        Element::Constant(c) => c,
        other => panic!("expected a constant, got {}", describe(other)),
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
        Element::Type(t) => describe_type(t),
        Element::Constant(c) => format!("constant `{}`", c.name),
        Element::Guard(_) => "guard".to_string(),
        Element::Unsupported(u) => match &u.name {
            Some(name) => format!("unsupported `{name}` ({})", u.error),
            None => format!("unsupported ({})", u.error),
        },
    }
}

fn describe_type(t: &Type) -> String {
    let kind = match t {
        Type::Struct(_) => "struct",
        Type::Variant(_) => "sum",
        Type::Enum(_) => "enum",
        Type::Extern(_) => "extern",
    };
    format!("{kind} `{}`", t.name())
}
