//! Unit tests over a small flat model.

use prebindgen::SourceLocation;
use prebindgen_flat::Flat;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{api::qualify::render, FormKind, Input, Output, Wire, WireType};

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
fn items_are_named_by_their_crate() {
    let flat = model(SRC);
    let f = flat.function("payloads").unwrap();
    assert_eq!(norm(f.name.to_token_stream()), "src_crate::payloads");
    assert_eq!(f.name.to_string(), "payloads");
    assert_eq!(
        norm(render(&flat, &f.ret)),
        "::core::option::Option<::std::vec::Vec<src_crate::Payload>>"
    );
    // A type the model does not declare is spelled as written.
    let foreign = flat
        .classify(&syn::parse_str("foreign::Thing").unwrap())
        .unwrap();
    assert_eq!(norm(render(&flat, &foreign)), "foreign::Thing");
}

#[test]
fn conversions_resolve_both_directions() {
    let flat = model(SRC);
    let conv = crate::convert!(Millis)
        .input(crate::fun!(millis_from))
        .output(crate::fun!(millis_to))
        .resolve(&flat)
        .unwrap();
    let ty = |s: &str| flat.classify(&syn::parse_str(s).unwrap()).unwrap();
    let wire = || Wire::new(format_ident!("w"), Raw);
    let (millis, raw) = (ty("Millis"), ty("u64"));
    assert_eq!(conv.repr().key(), raw.key());

    let input = Input::via(&conv, |r| {
        Ok::<_, String>(Input::wire(r, wire(), quote!(w)))
    })
    .unwrap();
    assert!(!input.fallible);
    assert!(matches!(input.form.kind, FormKind::Via(_)));
    assert_eq!(
        norm(input.expr),
        "{let__repr=w;src_crate::millis_from(__repr)}"
    );

    let output = Output::via(&conv, |r| {
        Ok::<_, String>(Output::wire(r, wire(), |v| quote!(#v)))
    })
    .unwrap();
    assert!(output.fallible, "a Result-returning function is fallible");
    assert_eq!(output.form.ty.key(), millis.key());
    let applied = norm(output.apply(quote!(v)));
    assert!(
        applied.starts_with("{let__v=v;{let__v=src_crate::millis_to(&__v)"),
        "{applied}"
    );
    assert_eq!(
        conv.functions()
            .map(|f| f.name().to_string())
            .collect::<Vec<_>>(),
        ["millis_from", "millis_to"]
    );

    // A direction the declaration leaves out is an error at use.
    let one_way = crate::convert!(Millis)
        .input(crate::fun!(millis_from))
        .resolve(&flat)
        .unwrap();
    let e = Output::via(&one_way, |r| {
        Ok::<_, String>(Output::wire(r, wire(), |v| quote!(#v)))
    })
    .expect_err("no output declared");
    assert!(e.contains("declares no output"), "{e}");
}

/// A wire type for tests that only look at expressions.
#[derive(Clone, Debug)]
struct Raw;

impl WireType for Raw {
    fn rust(&self) -> TokenStream {
        quote!(u64)
    }

    fn placeholder(&self) -> TokenStream {
        quote!(0)
    }
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
    // The two directions must agree on the representation.
    let e = err(crate::convert!(Millis)
        .input(crate::fun!(millis_from))
        .output(crate::into!(i32)));
    assert!(e.contains("one representation type"), "{e}");
    // A conversion with no direction converts nothing.
    let e = err(crate::convert!(Millis));
    assert!(e.contains("declares neither"), "{e}");
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
    let at = |i: usize| shape(&f.params[i].ty, shape_declared);
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
        shape(bytes, declared),
        Shape::Declared { ty, declaration: "bytes" } if std::ptr::eq(ty, bytes)
    ));
    assert!(matches!(
        shape(numbers, declared),
        Shape::Seq {
            kind: SequenceKind::Vec,
            access: Access::Owned,
            ..
        }
    ));
    assert!(matches!(
        shape(borrowed, declared),
        Shape::Declared { ty, declaration: "borrowed bytes" }
            if std::ptr::eq(ty, borrowed) && Access::of(ty) == Access::Shared
    ));
    assert!(matches!(
        shape(borrowed, |candidate| (candidate.key() == bytes_key).then_some("bytes")),
        Shape::Declared { ty, declaration: "bytes" }
            if std::ptr::eq(ty, borrowed) && Access::of(ty) == Access::Shared
    ));

    let text = &f.params[3].ty;
    let borrowed_text = &f.params[4].ty;
    let declared_text =
        |candidate: &TypeRef| matches!(candidate.kind(), TypeKind::String).then_some("text");
    assert!(matches!(
        shape(text, declared_text),
        Shape::Declared { ty, declaration: "text" } if std::ptr::eq(ty, text)
    ));
    assert!(matches!(
        shape(borrowed_text, declared_text),
        Shape::Declared { ty, declaration: "text" } if std::ptr::eq(ty, borrowed_text)
    ));
    assert!(matches!(
        shape::<()>(text, |_| None),
        Shape::Str {
            kind: TextKind::String,
            access: Access::Owned
        }
    ));
    assert!(matches!(
        shape::<()>(borrowed_text, |_| None),
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
            let shape = shape::<()>(&ty, |_| None);
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
        let Shape::Cow(inner) = shape::<()>(&ty, |_| None) else {
            panic!("expected Cow")
        };
        assert!(matches!(inner.kind(), TypeKind::Str | TypeKind::Slice(_)));
    }
    for (syntax, expected) in [
        (syn::parse_quote!(String), Access::Owned),
        (syn::parse_quote!(&String), Access::Shared),
        (syn::parse_quote!(&mut String), Access::Exclusive),
    ] {
        let ty = flat.classify(&syntax).unwrap();
        assert!(
            matches!(shape(&ty, |candidate| matches!(candidate.kind(), TypeKind::String).then_some(7)),
            Shape::Declared { ty: declared_ty, declaration: 7 } if Access::of(declared_ty) == expected)
        );
    }
}

#[test]
fn item_paths_follow_the_recorded_crate_or_the_default_module() {
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
    let path = |flat: &Flat, name: &str| {
        norm(
            flat.element(name)
                .unwrap()
                .name()
                .unwrap()
                .to_token_stream(),
        )
    };
    let flat = Flat::builder()
        .items(items.clone())
        .default_module(syn::parse_quote!(fallback))
        .build()
        .unwrap();
    assert_eq!(path(&flat, "make"), "source_crate::make");
    let array = &flat.function("make").unwrap().ret;
    assert_eq!(
        norm(render(&flat, array)),
        "[source_crate::Payload;source_crate::LEN]"
    );
    assert_eq!(path(&flat, "Other"), "other::Other");
    // The default module serves only items whose capture records no crate.
    assert_eq!(path(&flat, "Local"), "fallback::Local");
    // The path does not change the recorded provenance.
    assert_eq!(
        flat.function("make")
            .unwrap()
            .origin
            .location
            .crate_name
            .as_deref(),
        Some("source-crate")
    );
    let flat = Flat::builder().items(items).build().unwrap();
    assert_eq!(path(&flat, "Local"), "Local");
}

#[test]
fn rendering_elides_lifetimes_in_every_nested_shape() {
    let flat = model("pub struct Payload;");
    for (input, expected) in [
        (
            "Option<Result<&'a Payload, &'b str>>",
            quote!(::core::option::Option<::core::result::Result<&src_crate::Payload, &str>>),
        ),
        ("[&'a Payload; 2]", quote!([&src_crate::Payload; 2])),
        (
            "foreign::Wrapper<'a, &'b Payload>",
            quote!(foreign::Wrapper<&src_crate::Payload>),
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
        let elided = render(&flat, &ty);
        syn::parse2::<syn::Type>(elided.clone()).expect("valid Rust type syntax");
        assert_eq!(norm(elided), norm(expected), "{input}");
    }
}
