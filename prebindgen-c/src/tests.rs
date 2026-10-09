//! Generation checks over a small source. The end-to-end behavior — the C
//! ABI under ASan, invalid discriminants, aliasing — is covered by
//! `examples/example-cbindgen` (`c/smoke.c`, `boundary_tests.rs`); these pin
//! the adapter's decisions and refusals.

use prebindgen::SourceLocation;
use syn::parse_quote as pq;

use crate::{Cbindgen, CbindgenBuilder};

const SRC: &str = r#"
    pub type Handle = inner::Handle;
    pub type Error = Box<dyn std::error::Error>;
    pub enum Mode { A = 1, B = 2 }
    pub struct Point { pub x: f64, pub label: String }
    pub fn handle_new() -> Handle { todo!() }
    pub fn handle_mode(h: &Handle, m: Mode) -> i32 { todo!() }
    pub fn handle_try(h: &mut Handle, m: Mode) -> Result<f64, Error> { todo!() }
    pub fn handle_merge(a: Handle, b: Handle) -> Result<Handle, Error> { todo!() }
    pub fn handle_name(h: &Handle) -> String { todo!() }
    pub fn point_norm(p: Point) -> f64 { todo!() }
    pub fn error_message(e: &Error) -> String { todo!() }
    pub fn values(h: &Handle) -> Vec<f64> { todo!() }
"#;

fn builder() -> CbindgenBuilder {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(SRC)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, loc.clone()));
    Cbindgen::builder()
        .items(items)
        .free_memory_function("src_free")
        .mangle_type_name(|b| format!("{b}_t"))
        .opaque_ptr(pq!(Handle))
        .opaque_error(pq!(Error), pq!(error_message))
        .enum_type(pq!(Mode))
        .data_struct(pq!(Point))
}

fn flat(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn declared_surface_is_generated() {
    let out = flat(
        builder()
            .function(pq!(handle_new))
            .function(pq!(handle_try))
            .function(pq!(handle_merge))
            .function(pq!(handle_name))
            .panic()
            .function(pq!(point_norm))
            .function(pq!(values))
            .panic()
            .build()
            .unwrap()
            .render(),
    );
    // Opaque handle and its destructor.
    assert!(
        out.contains("pub struct handle_t { _private: [u8; 0], }"),
        "{out}"
    );
    assert!(out.contains("pub unsafe extern \"C\" fn handle_drop(this_: *mut handle_t)"));
    // A fallible value result: bool, an out-param and the error slot.
    assert!(out.contains(
        "fn handle_try( h: *mut handle_t, m: ::core::mem::MaybeUninit<mode_t>, out: *mut f64, e: *mut *mut ::core::ffi::c_char, ) -> bool"
    ), "{out}");
    // A fallible pointer result returns the pointer, null on error.
    assert!(out.contains("-> *mut handle_t"));
    // Two consumed handles of one type are checked for aliasing first.
    assert!(out.contains("aliasing arguments: `a` (consumed) and `b` (consumed)"));
    // A Vec result is an array plus its length.
    assert!(out.contains("fn values(h: *const handle_t, len: *mut usize) -> *mut f64"));
    // Undeclared functions are not generated.
    assert!(!out.contains("fn handle_mode("));
    assert!(out.contains("pub unsafe extern \"C\" fn src_free"));
}

#[test]
fn a_fallible_input_needs_an_error_channel() {
    // `Mode` arrives unchecked from C, and `handle_mode` returns no Result.
    let err = builder().function(pq!(handle_mode)).build().err().unwrap();
    assert!(err.0.contains("declare it `.panic()`"), "{}", err.0);
    // `.panic()` accepts it.
    assert!(builder().function(pq!(handle_mode)).panic().build().is_ok());
}

#[test]
fn undeclared_types_are_refused() {
    let err = Cbindgen::builder()
        .items(syn::parse_file(SRC).unwrap().items.into_iter().map(|i| {
            (
                i,
                SourceLocation {
                    crate_name: Some("src_crate".to_string()),
                    ..Default::default()
                },
            )
        }))
        .function(pq!(handle_new))
        .build()
        .err()
        .unwrap();
    assert!(err.0.contains("not declared to the C adapter"), "{}", err.0);
}

#[test]
fn a_string_result_needs_a_free_function() {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(SRC)
        .unwrap()
        .items
        .into_iter()
        .map(move |i| (i, loc.clone()));
    let err = Cbindgen::builder()
        .items(items)
        .opaque_ptr(pq!(Handle))
        .function(pq!(handle_name))
        .panic()
        .build()
        .err()
        .unwrap();
    assert!(err.0.contains("free_memory_function"), "{}", err.0);
}

#[test]
fn slice_inputs_do_not_accept_vector_references() {
    for (ty, accepted) in [
        ("&[Point]", true),
        ("&Vec<Point>", false),
        ("&mut [Point]", false),
        ("&mut Vec<Point>", false),
        ("[Point]", false),
    ] {
        let source = syn::parse_file(&format!(
            "pub struct Point {{ pub x: i64 }} pub fn inspect(x: {ty}) {{}}"
        ))
        .unwrap();
        let result = Cbindgen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, SourceLocation::default())),
            )
            .repr_c_struct(pq!(Point))
            .function(pq!(inspect))
            .panic()
            .build();
        assert_eq!(result.is_ok(), accepted, "{ty}: {:?}", result.err());
    }
}

#[test]
fn a_mirror_named_by_a_keyword_takes_its_fields_way() {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(SRC)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, loc.clone()));
    // `type` is a keyword: the mirror's identifier is `r#type`.
    let out = Cbindgen::builder()
        .items(items)
        .free_memory_function("src_free")
        .mangle_type_name(|_| "type".into())
        .data_struct(pq!(Point))
        .function(pq!(point_norm))
        .build()
        .unwrap()
        .render();
    assert!(flat(out).contains("pub struct r#type {"));
}

fn small(src: &str) -> CbindgenBuilder {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(src)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, loc.clone()));
    Cbindgen::builder()
        .items(items)
        .free_memory_function("src_free")
}

#[test]
fn a_boxed_parameter_crosses_as_a_value() {
    let out = small(
        "#[repr(C)] pub struct Pos { pub x: f64 } \
         pub fn take(p: Box<Pos>) {}",
    )
    .repr_c_struct(pq!(Pos))
    .function(pq!(take))
    .build()
    .unwrap()
    .render();
    // The boxed `Pos` is a by-value C struct, not the pointer a bare `Pos`
    // parameter is.
    let out = flat(out);
    assert!(out.contains("fn take(p: pos)"), "{out}");
}

#[test]
fn a_one_way_conversion_used_the_other_way_is_refused() {
    let err = small(
        "pub struct Millis(pub u64); \
         pub fn millis_to(m: &Millis) -> u64 { m.0 } \
         pub fn take(m: Millis) {}",
    )
    .convert(prebindgen_tools::convert!(Millis).output(prebindgen_tools::fun!(millis_to)))
    .function(pq!(take))
    .build()
    .err()
    .expect("an output-only conversion cannot be taken");
    assert!(err.0.contains("declares no input"), "{}", err.0);
}

#[test]
fn a_callback_argument_cannot_convert_fallibly() {
    let err = small(
        "pub struct Millis(pub u64); \
         pub fn millis_to(m: &Millis) -> Result<u64, String> { Ok(m.0) } \
         pub fn on(f: impl Fn(Millis) + Send + Sync + 'static) {}",
    )
    .convert(prebindgen_tools::convert!(Millis).output(prebindgen_tools::fun!(millis_to)))
    .function(pq!(on))
    .build()
    .err()
    .expect("a callback cannot report a failed conversion");
    assert!(err.0.contains("fallible output conversion"), "{}", err.0);
}
