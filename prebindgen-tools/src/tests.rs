//! Unit tests over a small flat model.

use prebindgen::SourceLocation;
use prebindgen_flat::Flat;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{record_in, record_out, Input, Output, Qualifier, Record, Wire};

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
fn shape_reads_one_layer_with_the_adapters_declaration() {
    use prebindgen_flat::flat::{TypeKind, TypeRef};

    use crate::{shape, Access, SequenceKind, Shape, TextKind};
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
    let shape_declared = |candidate: &TypeRef| match candidate.kind() {
        TypeKind::Named { id, .. } if id.name == "Payload" => Some(7u8),
        _ => None,
    };
    let at = |i: usize| shape(&f.params[i].ty, shape_declared).unwrap();
    assert!(matches!(
        at(0),
        Shape::Declared {
            ty,
            declaration: 7,
        } if std::ptr::eq(ty, &f.params[0].ty) && Access::of(ty) == Access::Shared
    ));
    assert!(matches!(
        at(1),
        Shape::Declared {
            ty,
            ..
        } if Access::of(ty) == Access::Exclusive
    ));
    assert!(matches!(
        at(2),
        Shape::Str {
            kind: TextKind::Str,
            access: Access::Shared
        }
    ));
    assert!(matches!(
        at(3),
        Shape::Seq {
            kind: SequenceKind::Vec,
            access: Access::Shared,
            ..
        }
    ));
    assert!(matches!(at(4), Shape::Option(_)));
    assert!(matches!(at(5), Shape::Undeclared("Other")));
    assert!(matches!(at(6), Shape::Out(_)));
}

#[test]
fn shape_declared_can_override_a_complete_generic_type() {
    use prebindgen_flat::flat::{TypeKind, TypeRef};

    use crate::{shape, Access, SequenceKind, Shape, TextKind};

    let flat = model(
        "pub fn f(bytes: Vec<u8>, numbers: Vec<i64>, borrowed: &Vec<u8>, text: String, borrowed_text: &String) {}",
    );
    let f = flat.function("f").unwrap();
    let bytes = &f.params[0].ty;
    let numbers = &f.params[1].ty;
    let borrowed = &f.params[2].ty;
    let bytes_key = bytes.key();
    let borrowed_key = borrowed.key();
    let declared = |candidate: &TypeRef| {
        if candidate.key() == borrowed_key {
            Some("borrowed bytes")
        } else if candidate.key() == bytes_key {
            Some("bytes")
        } else {
            None
        }
    };
    assert!(matches!(
        shape(bytes, declared).unwrap(),
        Shape::Declared { ty, declaration: "bytes" } if std::ptr::eq(ty, bytes)
    ));
    assert!(matches!(
        shape(numbers, declared).unwrap(),
        Shape::Seq {
            kind: SequenceKind::Vec,
            access: Access::Owned,
            ..
        }
    ));
    assert!(matches!(
        shape(borrowed, declared).unwrap(),
        Shape::Declared { ty, declaration: "borrowed bytes" }
            if std::ptr::eq(ty, borrowed) && Access::of(ty) == Access::Shared
    ));
    assert!(matches!(
        shape(borrowed, |candidate| (candidate.key() == bytes_key).then_some("bytes")).unwrap(),
        Shape::Declared { ty, declaration: "bytes" }
            if std::ptr::eq(ty, borrowed) && Access::of(ty) == Access::Shared
    ));

    let text = &f.params[3].ty;
    let borrowed_text = &f.params[4].ty;
    let declared_text =
        |candidate: &TypeRef| matches!(candidate.kind(), TypeKind::String).then_some("text");
    assert!(matches!(
        shape(text, declared_text).unwrap(),
        Shape::Declared { ty, declaration: "text" } if std::ptr::eq(ty, text)
    ));
    assert!(matches!(
        shape(borrowed_text, declared_text).unwrap(),
        Shape::Declared { ty, declaration: "text" } if std::ptr::eq(ty, borrowed_text)
    ));
    assert!(matches!(
        shape::<()>(text, |_| None).unwrap(),
        Shape::Str {
            kind: TextKind::String,
            access: Access::Owned
        }
    ));
    assert!(matches!(
        shape::<()>(borrowed_text, |_| None).unwrap(),
        Shape::Str {
            kind: TextKind::String,
            access: Access::Shared
        }
    ));
}

#[test]
fn shape_keeps_container_kind_independent_of_access() {
    use crate::{shape, Access, SequenceKind, Shape, TextKind};
    let flat = model("");
    for (container, text, sequence) in [
        ("String", Some(TextKind::String), None),
        ("str", Some(TextKind::Str), None),
        ("Vec<u8>", None, Some(SequenceKind::Vec)),
        ("[u8]", None, Some(SequenceKind::Slice)),
    ] {
        for (prefix, expected_access) in [
            ("", Access::Owned),
            ("&", Access::Shared),
            ("&mut ", Access::Exclusive),
        ] {
            let syntax = syn::parse_str(&format!("{prefix}{container}")).unwrap();
            let ty = flat.classify(&syntax).unwrap();
            let shape = shape::<()>(&ty, |_| None).unwrap();
            match shape {
                Shape::Str { kind, access } => {
                    assert_eq!(Some(kind), text);
                    assert_eq!(access, expected_access);
                }
                Shape::Seq { kind, access, elem } => {
                    assert_eq!(Some(kind), sequence);
                    assert_eq!(access, expected_access);
                    assert_eq!(elem.to_string(), "u8");
                }
                other => panic!("unexpected shape for {ty}: {other:?}"),
            }
        }
    }
}

#[test]
fn shape_keeps_cow_as_a_wrapper_and_string_declaration_precedence() {
    use prebindgen_flat::flat::TypeKind;

    use crate::{shape, Access, Shape};
    let flat = model("");
    for syntax in [
        syn::parse_quote!(Cow<'static, str>),
        syn::parse_quote!(Cow<'static, [u8]>),
    ] {
        let ty = flat.classify(&syntax).unwrap();
        let Shape::Cow(inner) = shape::<()>(&ty, |_| None).unwrap() else {
            panic!("expected Cow")
        };
        assert!(matches!(inner.kind(), TypeKind::Str | TypeKind::Slice(_)));
        assert!(
            shape::<()>(inner, |_| None).is_ok(),
            "unsized Cow child is classifiable"
        );
    }
    for (syntax, expected) in [
        (syn::parse_quote!(String), Access::Owned),
        (syn::parse_quote!(&String), Access::Shared),
        (syn::parse_quote!(&mut String), Access::Exclusive),
    ] {
        let ty = flat.classify(&syntax).unwrap();
        assert!(
            matches!(shape(&ty, |candidate| matches!(candidate.kind(), TypeKind::String).then_some(7)).unwrap(),
            Shape::Declared { ty: declared_ty, declaration: 7 } if Access::of(declared_ty) == expected)
        );
    }
}

#[test]
fn qualifier_overrides_paths_without_changing_provenance() {
    let mut items = Vec::new();
    for (crate_name, source) in [
        (
            Some("source-crate"),
            "pub struct Payload; pub const LEN: usize = 2; pub fn make() -> [Payload; LEN] { todo!() }",
        ),
        (Some("other"), "pub struct Other;"),
        (None, "pub struct Local;"),
    ] {
        let location = SourceLocation {
            crate_name: crate_name.map(str::to_owned),
            ..Default::default()
        };
        items.extend(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|item| (item, location.clone())),
        );
    }
    let flat = Flat::builder().items(items).build().unwrap();
    let q = Qualifier::new(&flat)
        .with_default_module(Some(syn::parse_quote!(fallback)))
        .with_crate_path("source-crate", syn::parse_quote!(first))
        .with_crate_path("source_crate", syn::parse_quote!(crate::renamed));
    assert_eq!(norm(q.path(&format_ident!("make"))), "crate::renamed::make");
    let array = &flat.function("make").unwrap().ret;
    assert_eq!(
        norm(q.ty(array)),
        "[crate::renamed::Payload;crate::renamed::LEN]"
    );
    assert_eq!(norm(q.path(&format_ident!("Other"))), "other::Other");
    assert_eq!(norm(q.path(&format_ident!("Local"))), "fallback::Local");
    assert_eq!(norm(q.path(&format_ident!("unknown"))), "unknown");
    assert_eq!(
        flat.function("make")
            .unwrap()
            .origin
            .location
            .crate_name
            .as_deref(),
        Some("source-crate")
    );
    let q = q.with_default_module(None);
    assert_eq!(norm(q.path(&format_ident!("Local"))), "Local");
}

#[test]
fn qualifier_elides_lifetimes_in_every_nested_shape() {
    let flat = model("pub struct Payload;");
    let q = Qualifier::new(&flat);
    for (input, expected) in [
        (
            "Option<Result<&'a Payload, &'b str>>",
            quote!(::core::option::Option<::core::result::Result<&src_crate::Payload, &str>>),
        ),
        ("[&'a Payload; 2]", quote!([&src_crate::Payload; 2])),
        (
            "foreign::Wrapper<'a, &'b Payload>",
            quote!(foreign::Wrapper<'_, &src_crate::Payload>),
        ),
        (
            "Vec<Box<Cow<'a, [Payload]>>>",
            quote!(
                ::std::vec::Vec<::std::boxed::Box<::std::borrow::Cow<'_, [src_crate::Payload]>>>
            ),
        ),
        (
            "&'a mut MaybeUninit<&'b Payload>",
            quote!(&mut ::core::mem::MaybeUninit<&src_crate::Payload>),
        ),
        (
            "impl Fn(Result<&'a Payload, &'static str>) + Send + Sync + 'static",
            quote!(
                impl Fn(::core::result::Result<&src_crate::Payload, &str>) + Send + Sync + 'static
            ),
        ),
    ] {
        let ty = flat.classify(&syn::parse_str(input).unwrap()).unwrap();
        let retained = q.ty(&ty);
        assert!(retained.to_string().contains("'a"), "{input}: {retained}");
        let elided = q.ty_elided(&ty);
        syn::parse2::<syn::Type>(elided.clone()).expect("valid Rust type syntax");
        assert_eq!(norm(elided), norm(expected), "{input}");
    }
}
