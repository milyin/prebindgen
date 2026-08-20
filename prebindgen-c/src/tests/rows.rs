//! What Cbindgen's per-type declarations become as registry rows.

use prebindgen_registry::recipe::{Assembly, Crossing, Row, Shape};

use super::*;

/// The model every fixture here is declared over.
fn model() -> prebindgen_registry::Flat {
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct Session;",
        "pub struct Sample { pub key: u32, pub payload: u64 }",
        "pub enum Priority { Low, High }",
        "pub enum Reply { Ok(u32), Err(u64) }",
    ]
    .into_iter()
    .map(|src| {
        (
            syn::parse_str(src).expect("parse"),
            SourceLocation::default(),
        )
    })
    .collect();
    prebindgen_registry::Flat::builder()
        .items(declare_referenced(items))
        .build()
        .expect("parse")
}

fn ty(model: &prebindgen_registry::Flat, spelling: &str) -> TypeRef {
    model
        .classify(&syn::parse_str(spelling).expect("parse type"))
        .expect("classify")
}

fn shape(row: &Row) -> String {
    match row {
        Row::Callback(_) => "callback".to_owned(),
        Row::Constructing(s) => format!("{:?}", Named(s)),
        Row::Deconstructing(s) => format!("{:?}", Named(s)),
    }
}

/// A shape rendered by name, so an assertion reads as the policy it checks.
struct Named<'a, OP>(&'a Shape<OP>);

impl<OP> std::fmt::Debug for Named<'_, OP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            Shape::Atomic => "atomic",
            Shape::Optional { .. } => "optional",
            Shape::Sequence { .. } => "sequence",
            Shape::Product(_) => "product",
            Shape::Choice { arms } => return write!(f, "choice({})", arms.len()),
        })
    }
}

fn rows_of(gen: &CbindgenBuilder, model: &prebindgen_registry::Flat, spelling: &str) -> String {
    let recipes = gen.recipes(model).expect("rows");
    [Assembly::Construct, Assembly::Deconstruct]
        .into_iter()
        .map(|assembly| {
            let crossing = Crossing::new(ty(model, spelling), assembly);
            let (id, row) = recipes.row(&crossing);
            format!("{assembly} {id}:{}", shape(&row))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn an_opaque_handle_has_no_parts() {
    let model = model();
    let gen = Cbindgen::builder().opaque_ptr(syn::parse_quote!(Session));
    assert_eq!(
        rows_of(&gen, &model, "Session"),
        "construct whole:atomic, deconstruct whole:atomic"
    );
}

#[test]
fn an_enum_has_no_parts() {
    let model = model();
    let gen = Cbindgen::builder().enum_type(syn::parse_quote!(Priority));
    assert_eq!(
        rows_of(&gen, &model, "Priority"),
        "construct whole:atomic, deconstruct whole:atomic"
    );
}

#[test]
fn a_data_struct_and_a_tagged_union_still_cross_whole() {
    // Both plainly have parts, and both are one row with none today: the field
    // walk lives inside one generated function per direction, which is exactly
    // what `Atomic` says. Stating the parts is what deletes those walks.
    let model = model();
    let gen = Cbindgen::builder()
        .data_struct(syn::parse_quote!(Sample))
        .tagged_union(syn::parse_quote!(Reply));
    assert_eq!(
        rows_of(&gen, &model, "Sample"),
        "construct whole:atomic, deconstruct whole:atomic"
    );
    assert_eq!(
        rows_of(&gen, &model, "Reply"),
        "construct whole:atomic, deconstruct whole:atomic"
    );
}

#[test]
fn a_borrow_and_a_wrapper_find_the_bare_types_row() {
    let model = model();
    let gen = Cbindgen::builder().opaque_ptr(syn::parse_quote!(Session));
    for spelling in ["&Session", "&mut Session", "Box<Session>"] {
        assert_eq!(
            rows_of(&gen, &model, spelling),
            "construct whole:atomic, deconstruct whole:atomic",
            "{spelling}"
        );
    }
}

#[test]
fn an_undeclared_arity_layer_takes_the_row_the_registry_derives() {
    let model = model();
    let gen = Cbindgen::builder().opaque_ptr(syn::parse_quote!(Session));
    assert_eq!(
        rows_of(&gen, &model, "Option<Session>"),
        "construct derived:optional, deconstruct derived:optional"
    );
    assert_eq!(
        rows_of(&gen, &model, "Vec<Session>"),
        "construct derived:sequence, deconstruct derived:sequence"
    );
    // Nothing declares a scalar, and nothing has to.
    assert_eq!(
        rows_of(&gen, &model, "u32"),
        "construct derived:atomic, deconstruct derived:atomic"
    );
}
