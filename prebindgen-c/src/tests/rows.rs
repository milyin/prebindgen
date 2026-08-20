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
fn a_data_struct_is_its_fields() {
    // Many parts, one wire value: each field converts itself and the converted
    // fields are reassembled into one C struct. That pair is what #450 keeps
    // apart, and the struct is where C shows both halves at once.
    let model = model();
    let gen = Cbindgen::builder().data_struct(syn::parse_quote!(Sample));
    assert_eq!(
        rows_of(&gen, &model, "Sample"),
        "construct parts:product, deconstruct parts:product"
    );
}

#[test]
fn a_tagged_union_is_its_arms() {
    // One arm per alternative, each a product of that alternative's payload.
    // Neither the tag nor the selector appears: how the C side is told which
    // arm is live is the adapter's business, so no row mentions it.
    let model = model();
    let gen = Cbindgen::builder().tagged_union(syn::parse_quote!(Reply));
    assert_eq!(
        rows_of(&gen, &model, "Reply"),
        "construct parts:choice(2), deconstruct parts:choice(2)"
    );
}

#[test]
fn a_value_read_two_ways_inside_a_struct_has_two_rows() {
    // `bool` and `String` cross differently inside a `data_struct`'s mirror
    // than they do on their own, which is two rows of one crossing with the
    // site picking — the table's own answer to one crossing with two wires.
    let model = model();
    let gen = Cbindgen::builder().data_struct(syn::parse_quote!(Sample));
    let recipes = gen.recipes(&model).expect("rows");

    for (spelling, assembly, expected) in [
        ("bool", Assembly::Construct, 2),
        ("bool", Assembly::Deconstruct, 2),
        // A `String` reads differently only on the way in.
        ("String", Assembly::Construct, 2),
        ("String", Assembly::Deconstruct, 0),
    ] {
        let key = Crossing::new(ty(&model, spelling), assembly).key();
        assert_eq!(recipes.rows(&key).len(), expected, "{spelling} {assembly}");
    }
    // The default is the whole-value reading in every case; the field reading
    // is reached only by a part that asks for it.
    let key = Crossing::new(ty(&model, "bool"), Assembly::Construct).key();
    assert_eq!(recipes.default_of(&key).unwrap().as_str(), "whole");
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

/// A `String` declared as a handle must not strand the **field** reading.
///
/// `convert!` refuses a builtin outright, so a custom `String` conversion is
/// not the route in — but `opaque_ptr` accepts one, and `out_terminal` has an
/// arm for exactly that shape. Declaring it that way used to suppress the field
/// row while `bindings` still asked every string field for it.
#[test]
fn a_string_declared_as_a_handle_keeps_the_field_reading() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct Caption { pub id: u64, pub text: String }",
        "pub fn caption_id(c: Caption) -> u64 { unimplemented!() }",
    ]
    .into_iter()
    .map(|src| (syn::parse_str(src).expect("parse"), loc.clone()))
    .collect();
    let model = prebindgen_registry::Flat::builder()
        .items(declare_referenced(items))
        .build()
        .expect("parse");

    let gen = Cbindgen::builder()
        .source_module(syn::parse_quote!(myflat))
        .free_memory_function("myflat_free")
        .opaque_ptr(syn::parse_quote!(String))
        .data_struct(syn::parse_quote!(Caption));
    let recipes = gen
        .recipes(&model)
        .expect("a String handle declaration leaves the field row in place");
    gen.bindings(&model, &recipes)
        .expect("every string field still finds the field row");
}
