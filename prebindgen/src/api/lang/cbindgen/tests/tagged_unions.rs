use super::*;

/// The running example: every variant shape a sum can take, including an
/// owning payload (`String` → `char *`) beside a declared-`enum_type` payload.
fn shape_enum() -> syn::ItemEnum {
    syn::parse_quote!(
        pub enum Shape {
            Empty,
            Circle(f64),
            Rect { width: f64, height: f64 },
            Labeled(String, Operation),
        }
    )
}

fn operation_enum() -> syn::ItemEnum {
    syn::parse_quote!(
        pub enum Operation {
            Add = 0,
            Sub = 1,
        }
    )
}

/// A tagged union crosses by value as a `#[repr(C)]` enum with payload
/// variants — the mirror cbindgen renders as a C tag + `union`. Variant shape
/// is preserved (unit stays unit, tuple stays tuple, named stays named) and
/// each payload takes its own wire.
#[test]
fn tagged_union_mirror_and_converters() {
    let loc = SourceLocation::default();
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn shape_new() -> Shape {
            unimplemented!()
        }
    );
    let take: syn::ItemFn = syn::parse_quote!(
        pub fn shape_area(s: Shape) -> f64 {
            unimplemented!()
        }
    );
    let registry = Registry::<()>::from_items([
        (syn::Item::Enum(shape_enum()), loc.clone()),
        (syn::Item::Enum(operation_enum()), loc.clone()),
        (syn::Item::Fn(make), loc.clone()),
        (syn::Item::Fn(take), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(example_flat))
        .free_memory_function("example_free")
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .enum_type(syn::parse_quote!(Operation))
        .tagged_union(syn::parse_quote!(Shape))
        .function(syn::parse_quote!(shape_new))
        .function(syn::parse_quote!(shape_area));

    let src = write(cbindgen, registry, "tagged_union");
    let compact: String = src.split_whitespace().collect();

    // The mirror: one variant per alternative, each keeping its own shape.
    assert!(compact.contains("pubenumshape_t{"), "{src}");
    assert!(compact.contains("Empty,"), "{src}");
    assert!(compact.contains("Circle(f64),"), "{src}");
    assert!(compact.contains("Rect{width:f64,height:f64},"), "{src}");
    // The owning payload lowers to `char *`; the declared enum to its C enum.
    assert!(
        compact.contains("Labeled(*mut::core::ffi::c_char,operation_t),"),
        "{src}"
    );

    // Output: match the source enum, convert each arm's fields.
    assert!(
        compact.contains("example_flat::Shape::Empty=>shape_t::Empty,"),
        "{src}"
    );
    assert!(
        compact.contains("example_flat::Shape::Circle(__f0)=>shape_t::Circle(__f0),"),
        "{src}"
    );
    assert!(
        compact.contains("shape_t::Labeled(__cbg_alloc_cstr(__f0),__cbg_out_Operation(__f1))"),
        "{src}"
    );

    // Input: the same match in reverse, through the same per-field policy.
    assert!(
        compact.contains("shape_t::Empty=>example_flat::Shape::Empty,"),
        "{src}"
    );
    assert!(compact.contains("__cbg_in_Operation(__f1),"), "{src}");
}

/// An owning payload gets a typed drop that frees the **active arm** and nulls
/// the freed slot, so a second drop is a no-op. Non-owning arms fall to the
/// wildcard and free nothing.
#[test]
fn owning_payload_gets_typed_drop() {
    let loc = SourceLocation::default();
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn shape_new() -> Shape {
            unimplemented!()
        }
    );
    let registry = Registry::<()>::from_items([
        (syn::Item::Enum(shape_enum()), loc.clone()),
        (syn::Item::Enum(operation_enum()), loc.clone()),
        (syn::Item::Fn(make), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(example_flat))
        .free_memory_function("example_free")
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .enum_type(syn::parse_quote!(Operation))
        .tagged_union(syn::parse_quote!(Shape))
        .function(syn::parse_quote!(shape_new));

    let src = write(cbindgen, registry, "tagged_union_drop");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("pubunsafeextern\"C\"fnshape_drop(this_:*mutshape_t)"),
        "{src}"
    );
    assert!(compact.contains("shape_t::Labeled(__f0,__f1)=>{"), "{src}");
    assert!(
        compact.contains("free(*__f0as*mut::core::ffi::c_void);"),
        "{src}"
    );
    assert!(compact.contains("*__f0=::core::ptr::null_mut();"), "{src}");
    // The plain-data arms need nothing freed.
    assert!(compact.contains("_=>{}"), "{src}");
}

/// A union of plain data owns nothing, so no drop is generated — there is
/// nothing for the C caller to release.
#[test]
fn plain_data_union_has_no_drop() {
    let loc = SourceLocation::default();
    let e: syn::ItemEnum = syn::parse_quote!(
        pub enum Value {
            Nothing,
            Int(i64),
        }
    );
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn value_new() -> Value {
            unimplemented!()
        }
    );
    let registry = Registry::<()>::from_items([
        (syn::Item::Enum(e), loc.clone()),
        (syn::Item::Fn(make), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(example_flat))
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .tagged_union(syn::parse_quote!(Value))
        .function(syn::parse_quote!(value_new));

    let src = write(cbindgen, registry, "tagged_union_plain");
    assert!(src.contains("pub enum value_t"), "{src}");
    assert!(!src.contains("value_drop"), "{src}");
}

/// A sum as a `data_struct` **field** crosses by value as its mirror, and the
/// struct's converters route the field through the union's own converter.
#[test]
fn tagged_union_as_data_struct_field() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Drawing {
            pub id: u64,
            pub shape: Shape,
        }
    );
    let f: syn::ItemFn = syn::parse_quote!(
        pub fn drawing_new(id: u64, shape: Shape) -> Drawing {
            unimplemented!()
        }
    );
    let registry = Registry::<()>::from_items([
        (syn::Item::Enum(shape_enum()), loc.clone()),
        (syn::Item::Enum(operation_enum()), loc.clone()),
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(f), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(example_flat))
        .free_memory_function("example_free")
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .enum_type(syn::parse_quote!(Operation))
        .tagged_union(syn::parse_quote!(Shape))
        .data_struct(syn::parse_quote!(Drawing))
        .function(syn::parse_quote!(drawing_new));

    let src = write(cbindgen, registry, "tagged_union_field");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("pubshape:shape_t,"), "{src}");
    assert!(compact.contains("shape:__cbg_in_Shape(v.shape),"), "{src}");
    assert!(compact.contains("shape:__cbg_out_Shape(v.shape),"), "{src}");
}

/// The two enum declarators are shape-exclusive in both directions, and each
/// rejection names the declarator to use instead. Neither silently upgrades.
#[test]
fn declarators_do_not_accept_each_others_shape() {
    let loc = SourceLocation::default();
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn shape_new() -> Shape {
            unimplemented!()
        }
    );

    // Payload enum handed to `.enum_type()`.
    let payload_as_enum = || {
        let registry = Registry::<()>::from_items([
            (syn::Item::Enum(shape_enum()), loc.clone()),
            (syn::Item::Enum(operation_enum()), loc.clone()),
            (syn::Item::Fn(make.clone()), loc.clone()),
        ])
        .expect("index items");
        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(example_flat))
            .free_memory_function("example_free")
            .mangle_type_name(|base| format!("{base}_t"))
            .enum_type(syn::parse_quote!(Shape))
            .function(syn::parse_quote!(shape_new));
        let _ = write(cbindgen, registry, "payload_as_enum");
    };
    assert!(catch(payload_as_enum));

    // Unit enum handed to `.tagged_union()`.
    let unit_fn: syn::ItemFn = syn::parse_quote!(
        pub fn op_new() -> Operation {
            unimplemented!()
        }
    );
    let unit_as_union = || {
        let registry = Registry::<()>::from_items([
            (syn::Item::Enum(operation_enum()), loc.clone()),
            (syn::Item::Fn(unit_fn.clone()), loc.clone()),
        ])
        .expect("index items");
        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(example_flat))
            .mangle_type_name(|base| format!("{base}_t"))
            .tagged_union(syn::parse_quote!(Operation))
            .function(syn::parse_quote!(op_new));
        let _ = write(cbindgen, registry, "unit_as_union");
    };
    assert!(catch(unit_as_union));
}

/// A payload type outside the supported wire set is a generation error naming
/// the offending variant field, not a silently wrong wire.
#[test]
fn unsupported_payload_is_a_generation_error() {
    let loc = SourceLocation::default();
    let e: syn::ItemEnum = syn::parse_quote!(
        pub enum Weird {
            Nothing,
            Odd(Vec<u8>),
        }
    );
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn weird_new() -> Weird {
            unimplemented!()
        }
    );
    let boom = || {
        let registry = Registry::<()>::from_items([
            (syn::Item::Enum(e.clone()), loc.clone()),
            (syn::Item::Fn(make.clone()), loc.clone()),
        ])
        .expect("index items");
        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(example_flat))
            .mangle_type_name(|base| format!("{base}_t"))
            .tagged_union(syn::parse_quote!(Weird))
            .function(syn::parse_quote!(weird_new));
        let _ = write(cbindgen, registry, "weird_payload");
    };
    assert!(catch(boom));
}
