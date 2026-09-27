//! Unit tests over a small flat model.

use prebindgen::SourceLocation;
use prebindgen_flat::{flat::TypeRef, Flat};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    record_in, record_out, FunctionCallbacks, FunctionWriter, Input, Output, Qualifier, Record,
    Return, Wire,
};

fn model(src: &str) -> Flat {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(src)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, loc.clone()));
    Flat::builder().items(items).build().unwrap()
}

const SRC: &str = r#"
    pub struct Payload { pub id: i64, pub label: String }
    pub enum Shape { Empty, Circle(f64), Rect { w: f64, h: f64 } }
    pub fn payload_id(p: &Payload) -> i64 { p.id }
    pub fn payload_new(id: i64, label: String) -> Payload { Payload { id, label } }
    pub fn payloads(n: i64) -> Option<Vec<Payload>> { None }
    pub struct Millis(pub u64);
    pub fn millis_from(v: u64) -> Millis { Millis(v) }
    pub fn millis_to(m: &Millis) -> Result<u64, String> { Ok(m.0) }
    pub fn millis_sum(a: u64, b: u64) -> Millis { Millis(a + b) }
    pub fn millis_raw(v: u64) -> u64 { v }
"#;

fn norm(t: TokenStream) -> String {
    t.to_string().replace(' ', "")
}

#[test]
fn qualifier_names_items_by_their_crate() {
    let flat = model(SRC);
    let q = Qualifier::new(&flat);
    let ret = &flat.function("payloads").unwrap().ret;
    assert_eq!(
        norm(q.ty(ret)),
        "::core::option::Option<::std::vec::Vec<src_crate::Payload>>"
    );
    assert_eq!(
        norm(q.path(&format_ident!("payload_id"))),
        "src_crate::payload_id"
    );
    // A name the model does not know is left alone.
    assert_eq!(norm(q.path(&format_ident!("elsewhere"))), "elsewhere");
}

/// An adapter that passes every value as itself.
struct Identity;

impl FunctionCallbacks for Identity {
    type Error = String;

    fn param(&mut self, name: &syn::Ident, ty: &TypeRef) -> Result<Input, String> {
        Ok(Input::identity(Wire::new(name.clone(), ty.spell())))
    }

    fn ret(&mut self, ret: &TypeRef) -> Result<Return, String> {
        let t = ret.spell();
        Ok(Return {
            ty: Some(t),
            wires: Vec::new(),
            body: quote!(__result),
        })
    }

    fn fail(&mut self, _ret: &Return) -> TokenStream {
        quote!(panic!("{}", __err))
    }
}

#[test]
fn function_writer_assembles_a_wrapper() {
    let flat = model(SRC);
    let f = flat.function("payload_new").unwrap();
    let q = Qualifier::new(&flat);
    let item = FunctionWriter::new(f, q.path(&f.name))
        .name(format_ident!("exported_new"))
        .attr(quote!(#[no_mangle]))
        .write(&mut Identity)
        .unwrap();
    let parsed: syn::ItemFn = syn::parse2(item.clone()).expect("a function");
    assert_eq!(parsed.sig.ident, "exported_new");
    assert_eq!(parsed.sig.inputs.len(), 2);
    assert!(norm(item).contains("let__result=src_crate::payload_new(id,label);"));
}

#[test]
fn records_keep_the_source_delimiters() {
    let flat = model(SRC);
    let Some(prebindgen_flat::flat::Type::Struct(payload)) = flat.declared_type("Payload") else {
        panic!("Payload is a struct");
    };
    let parts = payload
        .fields
        .iter()
        .map(|f| Input::identity(Wire::new(format_ident!("w{}", f.index), quote!(u8))))
        .collect();
    let input = record_in(Record::Struct(payload), &quote!(Payload), parts);
    assert_eq!(norm(input.expr), "Payload{id:w0,label:w1}");
    assert_eq!(input.wires.len(), 2);

    let Some(prebindgen_flat::flat::Type::Variant(shape)) = flat.declared_type("Shape") else {
        panic!("Shape is a sum");
    };
    // A positional alternative is rebuilt positionally.
    let circle = Record::Alt(&shape.alternatives[1]);
    let input = record_in(
        circle,
        &quote!(Shape::Circle),
        vec![Input::identity(Wire::new(format_ident!("r"), quote!(f64)))],
    );
    assert_eq!(norm(input.expr), "Shape::Circle(r)");
    let rect = Record::Alt(&shape.alternatives[2]);
    let out = record_out::<()>(rect, &quote!(Shape::Rect), &quote!(v), |f, b| {
        let n = format_ident!("o_{}", f.name.as_ref().unwrap());
        Ok(Output::single(Wire::new(n, quote!(f64)), b.clone()))
    })
    .unwrap();
    assert_eq!(out.wires.len(), 2);
    assert!(norm(out.expr).starts_with("{letShape::Rect{w:__f0,h:__f1}=v;"));
}

#[test]
fn conversions_resolve_both_directions() {
    let flat = model(SRC);
    let conv = crate::convert!(Millis)
        .input(crate::fun!(millis_from))
        .output(crate::fun!(millis_to))
        .resolve(&flat)
        .unwrap();
    let q = Qualifier::new(&flat);
    let input = conv.input.unwrap();
    assert!(!input.fallible);
    assert_eq!(
        norm(input.apply(&q, &conv.target, &quote!(r))),
        "src_crate::millis_from(r)"
    );
    let output = conv.output.unwrap();
    assert!(output.fallible, "a Result-returning stage is fallible");
    let applied = norm(output.apply(&q, &conv.target, &quote!(v)));
    assert!(applied.starts_with("src_crate::millis_to(&v)"), "{applied}");
}

#[test]
fn conversions_refuse_mismatched_functions() {
    let flat = model(SRC);
    let err = |c: crate::Conversion| c.resolve(&flat).expect_err("a refusal");
    // Two arguments where one is called.
    let e = err(crate::convert!(Millis).input(crate::fun!(millis_sum)));
    assert!(e.contains("exactly one argument"), "{e}");
    // An input stage that does not produce the converted type.
    let e = err(crate::convert!(Millis).input(crate::fun!(millis_raw)));
    assert!(e.contains("must return `Millis`"), "{e}");
    // An output stage that does not take it.
    let e = err(crate::convert!(Millis).output(crate::fun!(millis_raw)));
    assert!(e.contains("must take `Millis`"), "{e}");
    // A path outside the flat model needs its signature stated.
    let e = err(crate::convert!(Millis).input(crate::fun!(crate::local::millis_from)));
    assert!(e.contains(".sig("), "{e}");
}

#[test]
fn unsupported_items_refuse_the_model() {
    let flat = model("pub fn bad(x: *const u8) {}");
    assert!(crate::check_supported(&flat).is_err());
    assert!(crate::check_supported(&model(SRC)).is_ok());
}

#[test]
fn shape_reads_one_layer_with_the_adapters_setting() {
    use crate::{shape, Access, Holding, Shape};
    let flat = model(
        r#"
        pub struct Payload { pub id: i64 }
        pub type Other = inner::Other;
        pub fn f(a: &Payload, b: &mut Payload, c: &str, d: &Vec<u8>, e: Option<Payload>, g: &Other, h: &mut MaybeUninit<Payload>) {}
    "#,
    );
    let unsupported: Vec<String> = flat.unsupported().map(|u| u.error.to_string()).collect();
    let f = flat
        .function("f")
        .unwrap_or_else(|| panic!("{unsupported:?}"));
    let lookup = |n: &str| (n == "Payload").then_some(7u8);
    let at = |i: usize| shape(&f.params[i].ty, lookup).unwrap();
    assert!(matches!(
        at(0),
        Shape::Declared {
            name: "Payload",
            setting: 7,
            access: Access::Shared
        }
    ));
    assert!(matches!(
        at(1),
        Shape::Declared {
            access: Access::Exclusive,
            ..
        }
    ));
    assert!(matches!(at(2), Shape::Str(Holding::Borrowed)));
    assert!(matches!(
        at(3),
        Shape::Seq {
            holding: Holding::BorrowedVec,
            ..
        }
    ));
    assert!(matches!(at(4), Shape::Option(_)));
    assert!(matches!(at(5), Shape::Undeclared("Other")));
    assert!(matches!(at(6), Shape::Out(_)));
}
