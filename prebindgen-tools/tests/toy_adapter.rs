//! Integration checks through the public API, including compiled generated Rust.

use std::{
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use prebindgen::SourceLocation;
use prebindgen_flat::{
    flat::{ScalarKind, Type, TypeRef},
    Flat,
};
use prebindgen_tools::{
    convert, fun, ConversionPlan, ConversionPolicy, Direction, FormKind, Input, Output, Overrides,
    Place, Qualifier, Resolver, Scope, Seg, Wire, WireType,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug, PartialEq)]
enum Boundary {
    Long(&'static str),
    Flag,
    NoPlaceholder,
}

impl WireType for Boundary {
    fn rust_type(&self) -> syn::Type {
        match self {
            Self::Long(_) | Self::NoPlaceholder => syn::parse_quote!(i64),
            Self::Flag => syn::parse_quote!(u8),
        }
    }
    fn placeholder(&self) -> Option<TokenStream> {
        match self {
            Self::NoPlaceholder => None,
            _ => Some(quote!(0)),
        }
    }
}

#[derive(Clone, Debug)]
enum Rule {
    Number,
    Duration,
    Reject,
    Recurse,
    WrongType,
    WrongDirection,
}

struct Policy;
impl ConversionPolicy for Policy {
    type Wire = Boundary;
    type Rule = Rule;
    type Metadata = &'static str;

    fn resolve(
        &self,
        scope: &Scope<'_, Self>,
        ty: &TypeRef,
        dir: Direction,
        at: &Place,
        rule: Option<&Rule>,
    ) -> Result<ConversionPlan<Boundary, &'static str>, String> {
        match rule {
            Some(Rule::Reject) => return Err("explicit refusal".into()),
            Some(Rule::Recurse) => return scope.resolve(ty, dir, &at.at(Seg::Repr)),
            Some(Rule::WrongType) => {
                return self.resolve(scope, &TypeRef::scalar(ScalarKind::I64), dir, at, None)
            }
            Some(Rule::WrongDirection) => {
                return self.resolve(scope, ty, Direction::OutOfRust, at, None)
            }
            _ => {}
        }
        let meaning = match rule {
            Some(Rule::Duration) => "Duration",
            _ => "Number",
        };
        let name = at.ident("");
        let wire = Wire::new(name.clone(), Boundary::Long(meaning));
        match dir {
            Direction::IntoRust => {
                ConversionPlan::input(Input::wire(ty, wire, quote!(#name as u64)), meaning)
            }
            Direction::OutOfRust => {
                let src = at.ident("source");
                ConversionPlan::output(
                    src.clone(),
                    Output::wire(ty, wire, quote!(#src as i64)),
                    meaning,
                )
            }
        }
    }
}

fn u64_ty() -> TypeRef {
    TypeRef::scalar(ScalarKind::U64)
}
fn part(name: &str) -> Place {
    Place::new("generated").at(Seg::Custom(name.into()))
}

#[test]
fn selection_is_directional_and_the_destination_reads_the_selected_plan() {
    let ty = u64_ty();
    let mut resolver = Resolver::new(Policy);
    resolver
        .default(&ty, Direction::IntoRust, Rule::Duration)
        .unwrap();
    resolver
        .default(&ty, Direction::OutOfRust, Rule::Number)
        .unwrap();
    assert!(resolver
        .default(&ty, Direction::IntoRust, Rule::Reject)
        .is_err());
    let mut overrides = Overrides::new();
    overrides
        .at(part("value"), &ty, Direction::IntoRust, Rule::Number)
        .unwrap();
    assert!(overrides
        .at(part("value"), &ty, Direction::IntoRust, Rule::Reject)
        .is_err());
    let scope = resolver.scope("generated", overrides).unwrap();
    let input = scope
        .resolve(&ty, Direction::IntoRust, &part("value"))
        .unwrap();
    let sibling = scope
        .resolve(&ty, Direction::IntoRust, &part("sibling"))
        .unwrap();
    let output = scope
        .resolve(&ty, Direction::OutOfRust, &part("value"))
        .unwrap();
    assert_eq!(input.metadata(), &"Number");
    assert_eq!(sibling.metadata(), &"Duration");
    assert_eq!(output.metadata(), &"Number");
    assert_eq!(
        input.wires()[0].ty.rust_type(),
        sibling.wires()[0].ty.rust_type()
    );
    assert_ne!(input.wires()[0].ty, sibling.wires()[0].ty);
    assert!(input.emit_output(quote!(value)).is_err());
    assert!(output
        .with_input(&syn::parse_quote!(local), |_| quote!())
        .is_err());
    scope.finish().unwrap();
}

#[test]
fn bad_overrides_do_not_fall_back_and_failed_resolution_does_not_poison_the_scope() {
    for (rule, message) in [
        (Rule::Reject, "explicit refusal"),
        (Rule::Recurse, "cycle"),
        (Rule::WrongType, "policy returned"),
        (Rule::WrongDirection, "policy returned"),
    ] {
        let resolver = Resolver::new(Policy);
        let mut overrides = Overrides::new();
        overrides
            .at(part("bad"), &u64_ty(), Direction::IntoRust, rule)
            .unwrap();
        let scope = resolver.scope("generated", overrides).unwrap();
        assert!(scope
            .resolve(&u64_ty(), Direction::IntoRust, &part("bad"))
            .unwrap_err()
            .contains(message));
        scope
            .resolve(&u64_ty(), Direction::IntoRust, &part("good"))
            .unwrap();
        assert!(scope.finish().unwrap_err().contains("unused"));
    }
    let resolver = Resolver::new(Policy);
    let mut overrides = Overrides::new();
    overrides
        .at(
            part("value"),
            &TypeRef::scalar(ScalarKind::I64),
            Direction::IntoRust,
            Rule::Number,
        )
        .unwrap();
    let scope = resolver.scope("generated", overrides).unwrap();
    assert!(scope
        .resolve(&u64_ty(), Direction::IntoRust, &part("value"))
        .unwrap_err()
        .contains("expects"));
    assert!(scope
        .resolve(&u64_ty(), Direction::IntoRust, &Place::new("other"))
        .is_err());
    assert!(scope.finish().is_err());
    let mut overrides = Overrides::new();
    overrides
        .at(
            Place::new("other"),
            &u64_ty(),
            Direction::IntoRust,
            Rule::Number,
        )
        .unwrap();
    assert!(resolver.scope("generated", overrides).is_err());
}

fn model(src: &str) -> Flat {
    let source = syn::parse_file(src).unwrap();
    let loc = SourceLocation {
        crate_name: Some("source".into()),
        ..Default::default()
    };
    Flat::builder()
        .items(source.items.into_iter().map(|i| (i, loc.clone())))
        .build()
        .unwrap()
}

/// A metadata tree belongs to the language generator. Child selection is
/// recorded here for the destination writer, including occurrence overrides.
#[derive(Debug, PartialEq)]
enum Metadata {
    Scalar(&'static str),
    Point(Vec<Metadata>),
}

struct RecordPolicy<'f> {
    flat: &'f Flat,
    q: Qualifier<'f>,
}

impl ConversionPolicy for RecordPolicy<'_> {
    type Wire = Boundary;
    type Rule = &'static str;
    type Metadata = Metadata;

    fn resolve(
        &self,
        scope: &Scope<'_, Self>,
        ty: &TypeRef,
        dir: Direction,
        at: &Place,
        rule: Option<&&'static str>,
    ) -> Result<ConversionPlan<Boundary, Metadata>, String> {
        if let prebindgen_flat::flat::TypeKind::Named { id, .. } = ty.kind() {
            let Some(Type::Struct(s)) = self.flat.declared_type(id.name.as_str()) else {
                return Err("record expected".into());
            };
            let mut metadata = Vec::new();
            return match dir {
                Direction::IntoRust => {
                    let mut fields = Vec::new();
                    for field in &s.fields {
                        let child = scope.resolve(&field.ty, dir, &at.at(Seg::field(field)))?;
                        let (input, meaning) = child.into_input()?;
                        fields.push(input);
                        metadata.push(meaning);
                    }
                    ConversionPlan::input(
                        Input::record(&self.q, s, fields),
                        Metadata::Point(metadata),
                    )
                }
                Direction::OutOfRust => {
                    let source = at.ident("source");
                    let out = Output::record(&self.q, s, &quote!(#source), |field, value| {
                        let child = scope.resolve(&field.ty, dir, &at.at(Seg::field(field)))?;
                        let (out, meaning) = child.into_output(value)?;
                        metadata.push(meaning);
                        Ok::<_, String>(out)
                    })?;
                    ConversionPlan::output(source, out, Metadata::Point(metadata))
                }
            };
        }
        let meaning = rule.copied().unwrap_or("Number");
        let name = at.ident("");
        let wire = Wire::new(name.clone(), Boundary::Long(meaning));
        let rust_type = self.q.ty(ty);
        match dir {
            Direction::IntoRust => ConversionPlan::input(
                Input::wire(ty, wire, quote!(#name as #rust_type)),
                Metadata::Scalar(meaning),
            ),
            Direction::OutOfRust => {
                let source = at.ident("source");
                ConversionPlan::output(
                    source.clone(),
                    Output::wire(ty, wire, quote!(#source as i64)),
                    Metadata::Scalar(meaning),
                )
            }
        }
    }
}

#[test]
fn composition_checks_names_and_missing_placeholders() {
    let ty = u64_ty();
    let inputs = (0..2)
        .map(|_| {
            (
                Seg::Custom("part".into()),
                Input::wire(
                    &ty,
                    Wire::new(syn::parse_quote!(duplicate), Boundary::Long("Number")),
                    quote!(duplicate),
                ),
            )
        })
        .collect();
    assert!(
        ConversionPlan::input(Input::parts(&ty, inputs, |_| quote!(0)), ())
            .unwrap_err()
            .contains("duplicate wire")
    );
    let flat = model("pub fn optional(x: Option<u64>) {}");
    let ty = &flat.function("optional").unwrap().params[0].ty;
    let err = Output::optional(ty, None, &quote!(value), |src| {
        Ok(Output::wire(
            &u64_ty(),
            Wire::new(syn::parse_quote!(wire), Boundary::NoPlaceholder),
            src,
        ))
    })
    .unwrap_err();
    assert!(err.contains("no placeholder"));
}

#[test]
fn compiled_conversions_preserve_records_intermediates_borrows_and_evaluation_order() {
    let source = quote! {
        pub mod source {
            pub struct Point { pub x: i64, pub y: u64 }
            pub struct Millis(pub u64);
            pub fn millis_from(v: u64) -> Millis { Millis(v) }
            pub fn millis_to(v: &Millis) -> u64 { v.0 }
            pub fn point_echo(p: Point) -> Point { p }
            pub fn length(s: &str) -> Result<u64, String> {
                if s.len() == 3 { Err("length three".into()) } else { Ok(s.len() as u64) }
            }
        }
    };
    // The flat namespace reads the module's items, just as a source crate does.
    let flat = model(
        r#"
        pub struct Point { pub x: i64, pub y: u64 }
        pub struct Millis(pub u64);
        pub fn millis_from(v: u64) -> Millis { Millis(v) }
        pub fn millis_to(v: &Millis) -> u64 { v.0 }
        pub fn point_echo(p: Point) -> Point { p }
        pub fn length(s: &str) -> Result<u64, String> { todo!() }
        pub fn optional(p: Option<Point>) {}
    "#,
    );
    let q = Qualifier::new(&flat);
    let Some(Type::Struct(point)) = flat.declared_type("Point") else {
        panic!()
    };
    let resolver = Resolver::new(RecordPolicy {
        flat: &flat,
        q: Qualifier::new(&flat),
    });
    let input_place = part("point");
    let mut overrides = Overrides::new();
    overrides
        .at(
            input_place.at(Seg::Field("y".into())),
            &u64_ty(),
            Direction::IntoRust,
            "Duration",
        )
        .unwrap();
    let scope = resolver.scope("generated", overrides).unwrap();
    let input = scope
        .resolve(point.type_ref(), Direction::IntoRust, &input_place)
        .unwrap();
    let output = scope
        .resolve(point.type_ref(), Direction::OutOfRust, &part("result"))
        .unwrap();
    assert_eq!(
        input.metadata(),
        &Metadata::Point(vec![
            Metadata::Scalar("Number"),
            Metadata::Scalar("Duration")
        ])
    );
    assert_eq!(
        output.metadata(),
        &Metadata::Point(vec![Metadata::Scalar("Number"), Metadata::Scalar("Number")])
    );
    scope.finish().unwrap();
    let encoded = output
        .emit_output(quote!({
            CALLS.fetch_add(1, Ordering::SeqCst);
            source::point_echo(__arg)
        }))
        .unwrap();
    let body = input
        .with_input(&syn::parse_quote!(__arg), |_| quote!(Ok(#encoded)))
        .unwrap();
    let params: Vec<_> = input.wires().iter().map(|w| w.decl()).collect();

    let stage = convert!(Millis)
        .input(fun!(millis_from))
        .output(fun!(millis_to))
        .resolve(&flat)
        .unwrap();
    let in_stage = stage.input.unwrap();
    let value = Input::wire(
        &u64_ty(),
        Wire::new(syn::parse_quote!(raw), Boundary::Long("Number")),
        quote!(raw as u64),
    );
    let millis = in_stage.decode(&q, value);
    assert!(matches!(millis.form.kind, FormKind::Via(_)));
    let in_plan = ConversionPlan::input(millis, "Millis").unwrap();
    let out_stage = stage.output.unwrap();
    let millis_out = out_stage
        .encode(&q, &quote!(__millis), |repr| {
            Ok::<_, String>(Output::wire(
                &u64_ty(),
                Wire::new(syn::parse_quote!(raw_out), Boundary::Long("Number")),
                quote!(#repr as i64),
            ))
        })
        .unwrap();
    let out_plan =
        ConversionPlan::output(syn::parse_quote!(__millis), millis_out, "Millis").unwrap();
    let emit = out_plan.emit_output(quote!(__arg)).unwrap();
    let millis_body = in_plan
        .with_input(&syn::parse_quote!(__arg), |_| quote!(Ok(#emit)))
        .unwrap();

    // The storage and drop guard must outlive the borrowed call and drop on error.
    let borrowed_ty = &flat.function("length").unwrap().params[0].ty;
    let borrowed = ConversionPlan::input_with_setup(
        Input::wire(
            borrowed_ty,
            Wire::new(syn::parse_quote!(n), Boundary::Long("Number")),
            quote!(__storage.as_str()),
        ),
        quote! {
            let __guard = Guard;
            let __storage = { SETUPS.fetch_add(1, Ordering::SeqCst); "x".repeat(n as usize) };
        },
        "String",
    )
    .unwrap();
    let borrow_body = borrowed
        .with_input(
            &syn::parse_quote!(__text),
            |text| quote!(Ok(source::length(#text)? as i64)),
        )
        .unwrap();

    // Optional decomposition must evaluate the source once and skip inactive conversions.
    let opt_ty = &flat.function("optional").unwrap().params[0].ty;
    let opt = Output::optional(
        opt_ty,
        Some((
            Wire::new(syn::parse_quote!(present), Boundary::Flag),
            quote!(1u8),
        )),
        &quote!(__optional),
        |value| {
            Output::record(&q, point, &value, |f, v| {
                let name = if f.index == 0 {
                    syn::parse_quote!(ox)
                } else {
                    syn::parse_quote!(oy)
                };
                Ok::<_, String>(Output::wire(
                    &f.ty,
                    Wire::new(name, Boundary::Long("Number")),
                    quote!({
                        FIELDS.fetch_add(1, Ordering::SeqCst); #v as i64
                    }),
                ))
            })
        },
    )
    .unwrap();
    let opt_plan =
        ConversionPlan::output(syn::parse_quote!(__optional), opt, "OptionalPoint").unwrap();
    let opt_expr = opt_plan
        .emit_output(quote!({
            CALLS.fetch_add(1, Ordering::SeqCst);
            if some {
                Some(source::Point { x: 7, y: 9 })
            } else {
                None
            }
        }))
        .unwrap();
    let program = quote! {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        static SETUPS: AtomicUsize = AtomicUsize::new(0);
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        static FIELDS: AtomicUsize = AtomicUsize::new(0);
        struct Guard;
        impl Drop for Guard { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
        #source
        fn roundtrip(#(#params),*) -> Result<(i64, i64), String> { #body }
        fn millis(raw: i64) -> Result<i64, String> { #millis_body }
        fn borrowed(n: i64) -> Result<i64, String> { #borrow_body }
        fn optional(some: bool) -> (u8, i64, i64) { #opt_expr }
        fn main() {
            assert_eq!(roundtrip(5, -1).unwrap(), (5, -1));
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(millis(-1).unwrap(), -1);
            assert_eq!(borrowed(4).unwrap(), 4);
            assert!(borrowed(3).is_err());
            assert_eq!(SETUPS.load(Ordering::SeqCst), 2);
            assert_eq!(DROPS.load(Ordering::SeqCst), 2);
            assert_eq!(optional(false), (0, 0, 0));
            assert_eq!(FIELDS.load(Ordering::SeqCst), 0);
            assert_eq!(optional(true), (1, 7, 9));
            assert_eq!(FIELDS.load(Ordering::SeqCst), 2);
            assert_eq!(CALLS.load(Ordering::SeqCst), 3);
        }
    };
    compile_and_run(program);
}

fn compile_and_run(program: TokenStream) {
    static ID: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "prebindgen-tools-{}-{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    struct Remove(std::path::PathBuf);
    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _remove = Remove(dir.clone());
    let source = dir.join("generated.rs");
    let exe = dir.join(format!("generated{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&source, program.to_string()).unwrap();
    let built = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--edition=2021")
        .arg("-Dwarnings")
        .arg(&source)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "generated Rust failed:\n{}\n{}",
        String::from_utf8_lossy(&built.stderr),
        program
    );
    let ran = Command::new(exe).output().unwrap();
    assert!(
        ran.status.success(),
        "generated Rust runtime checks failed:\n{}",
        String::from_utf8_lossy(&ran.stderr)
    );
}
