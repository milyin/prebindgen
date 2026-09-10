//! The declaration list says exactly what the declarators say, in any order,
//! and refuses to say one thing twice.

use super::*;
use crate::{
    callback, data_type, decls, enum_type, error_type, fun, ptr_type, repr_c_type, tagged_union,
    value_type,
};

/// A record crossing by value, a function taking a `String` — a fallible input
/// with no `Result` to report through, so it needs `.abort_on_conversion_error()` — and one that
/// does not.
fn items() -> Vec<(syn::Item, SourceLocation)> {
    let loc = SourceLocation::default();
    let record: syn::ItemStruct = syn::parse_quote!(
        pub struct Point {
            pub x: i64,
        }
    );
    let named: syn::ItemFn = syn::parse_quote!(
        pub fn point_named(label: String) -> i64 {
            unimplemented!()
        }
    );
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn point_make(x: i64) -> Point {
            unimplemented!()
        }
    );
    declare_referenced([
        (syn::Item::Struct(record), loc.clone()),
        (syn::Item::Fn(named), loc.clone()),
        (syn::Item::Fn(make), loc),
    ])
}

fn registry() -> RegistryBuilder {
    crate::test_util::reg_from_items(items()).expect("index items")
}

fn base() -> CbindgenBuilder {
    CbindgenBuilder::new()
        .source_module(syn::parse_quote!(source))
        .free_memory_function("source_free")
}

/// The list lowers to the same binding the declarators build by hand.
///
/// Both modifiers are load-bearing: `pt` is not the base `Point` would default
/// to, so a dropped `.base_name` renames the emitted type, and `point_named`
/// takes a `String` with no `Result`, so a dropped `.panic` fails the build.
#[test]
fn a_list_and_the_declarators_it_lowers_to_agree() {
    let declared = write(
        base()
            .data_struct(syn::parse_quote!(Point))
            .base_name("pt")
            .function(syn::parse_quote!(point_named))
            .panic()
            .function(syn::parse_quote!(point_make)),
        registry(),
        "decl_flat",
    );
    let listed = write(
        base().declare(
            decls!()
                .data_type(data_type!(Point).base_name("pt"))
                .fun(fun!(point_named).abort_on_conversion_error())
                .fun(fun!(point_make)),
        ),
        registry(),
        "decl_list",
    );
    assert_eq!(declared, listed);
    assert!(
        listed.contains("pt"),
        "the declared base reached the output"
    );
}

/// Dropping the modifier the list carried is a build error, which is what makes
/// the comparison above sensitive to it.
#[test]
fn a_function_needing_panic_is_refused_without_it() {
    let message = catch_msg(|| {
        let _ = write(
            base().declare(
                decls!()
                    .data_type(data_type!(Point).base_name("pt"))
                    .fun(fun!(point_named)),
            ),
            registry(),
            "decl_no_panic",
        );
    });
    assert!(
        message.contains(".abort_on_conversion_error()"),
        "refused for the reason the modifier addresses: {message}"
    );
}

/// Declaring the function before the type it belongs to changes nothing, and
/// neither does interleaving two types whose options differ.
#[test]
fn the_order_of_a_list_does_not_change_it() {
    let one = write(
        base().declare(
            decls!()
                .data_type(data_type!(Point).base_name("pt"))
                .enum_type(enum_type!(Mode).base_name("mode"))
                .fun(fun!(point_named).abort_on_conversion_error())
                .fun(fun!(point_make)),
        ),
        registry_with_enum(),
        "decl_order_one",
    );
    let other = write(
        base().declare(
            decls!()
                .fun(fun!(point_make))
                .enum_type(enum_type!(Mode).base_name("mode"))
                .fun(fun!(point_named).abort_on_conversion_error())
                .data_type(data_type!(Point).base_name("pt")),
        ),
        registry_with_enum(),
        "decl_order_other",
    );
    assert_eq!(one, other);
}

/// The fixture above plus a fieldless enum, so the order test has two types
/// carrying different options.
fn registry_with_enum() -> RegistryBuilder {
    let loc = SourceLocation::default();
    let mode: syn::ItemEnum = syn::parse_quote!(
        pub enum Mode {
            Fast,
            Slow,
        }
    );
    let mut items = items();
    items.push((syn::Item::Enum(mode), loc));
    crate::test_util::reg_from_items(items).expect("index items")
}

/// A declaration made twice is refused rather than resolved by lowering order.
#[test]
fn a_repeated_declaration_is_refused() {
    // Two declarations of one function with different options: lowering order
    // would otherwise pick one set and drop the other.
    let message = catch_msg(|| {
        let _ = base().declare(
            decls!()
                .fun(
                    fun!(point_make)
                        .base_name("first")
                        .abort_on_conversion_error(),
                )
                .fun(fun!(point_make).base_name("second")),
        );
    });
    assert!(
        message.contains("point_make") && message.contains("declared twice"),
        "names the clash: {message}"
    );

    // The same function declared twice, however it is spelled.
    let message = catch_msg(|| {
        let _ = base().declare(decls!().fun(fun!(point_make)).fun(fun!(point_make)));
    });
    assert!(message.contains("point_make"), "{message}");

    // A type declared under two representations is a clash too.
    let message = catch_msg(|| {
        let _ = base().declare(
            decls!()
                .data_type(data_type!(Point))
                .ptr_type(ptr_type!(Point)),
        );
    });
    assert!(message.contains("Point"), "{message}");
}

/// Every kind of declaration the list carries reaches the builder, with the
/// option it was given — including the kinds neither C example declares.
#[test]
fn every_declaration_kind_carries_its_options() {
    let built = base().declare(
        decls!()
            .ptr_type(ptr_type!(Handle).base_name("handle"))
            .data_type(data_type!(Failure).base_name("failure").error())
            .enum_type(enum_type!(Mode))
            .tagged_union(tagged_union!(Shape))
            .value_type(value_type!(Owned, OwnedOpaque).base_name("owned"))
            .value_type(value_type!(Plain, PlainOpaque).data())
            .repr_c_type(repr_c_type!(Raw).assume_field_validity().base_name("raw"))
            .error_type(error_type!(Opaque, opaque_message))
            .callback(
                callback!(impl Fn(i64) + Send + Sync + 'static)
                    .base_name("value")
                    .takeable_param(0),
            )
            .convert(
                crate::ConvertTypeDecl::new(
                    prebindgen_registry::convert!(Millis)
                        .input(prebindgen_registry::fun!(millis_in))
                        .output(prebindgen_registry::fun!(millis_out)),
                )
                .base_name("millis"),
            )
            .fun(fun!(point_make))
            .ignore_fun(syn::parse_quote!(unused_fn))
            .ignore_type(syn::parse_quote!(Unused)),
    );

    let key = |ty: syn::Type| prebindgen_registry::TypeKey::from_type(&ty);
    assert_eq!(
        built.opaque[&key(syn::parse_quote!(Handle))]
            .base
            .as_deref(),
        Some("handle")
    );
    let failure = &built.data[&key(syn::parse_quote!(Failure))];
    assert_eq!(failure.base.as_deref(), Some("failure"));
    assert!(built.error.contains(&key(syn::parse_quote!(Failure))));
    assert!(built.enums.contains_key(&key(syn::parse_quote!(Mode))));
    assert!(built
        .tagged_unions
        .contains_key(&key(syn::parse_quote!(Shape))));

    let owned = &built.value_opaque[&key(syn::parse_quote!(Owned))];
    assert_eq!(owned.cfg.base.as_deref(), Some("owned"));
    assert!(
        matches!(owned.kind, crate::OpaqueKind::Owned),
        "value_type! is owned unless `.data()` says so"
    );
    assert!(matches!(
        built.value_opaque[&key(syn::parse_quote!(Plain))].kind,
        crate::OpaqueKind::Data
    ));
    let raw = &built.value_opaque[&key(syn::parse_quote!(Raw))];
    assert!(raw.assume_c_field_validity);
    assert_eq!(raw.cfg.base.as_deref(), Some("raw"));

    assert!(built
        .opaque_errors
        .contains_key(&key(syn::parse_quote!(Opaque))));
    let callback = built.callbacks.values().next().expect("one callback");
    assert_eq!(callback.base.as_deref(), Some("value"));
    assert_eq!(
        callback.takeable.iter().copied().collect::<Vec<_>>(),
        vec![0]
    );
    assert_eq!(
        built.convert_bases[&key(syn::parse_quote!(Millis))],
        "millis"
    );
    assert!(built.functions.contains_key(&syn::parse_quote!(point_make)));
    assert!(built
        .ignored_functions
        .contains(&syn::parse_quote!(unused_fn)));
    assert!(built
        .ignored_types
        .contains(&key(syn::parse_quote!(Unused))));
}

/// A base a declaration carries reaches the emitted names — for a function, an
/// enum, a tagged union and a `repr(C)` mirror.
///
/// Each of these would survive a lowering that quietly dropped its base if the
/// name it produced were the default one, so each asks for a name the default
/// derivation would never give.
#[test]
fn a_declared_base_reaches_the_emitted_names() {
    let loc = SourceLocation::default();
    let mode: syn::ItemEnum = syn::parse_quote!(
        pub enum Mode {
            Fast,
            Slow,
        }
    );
    let shape: syn::ItemEnum = syn::parse_quote!(
        pub enum Shape {
            Dot,
            Line(i64),
        }
    );
    let raw: syn::ItemStruct = syn::parse_quote!(
        #[repr(C)]
        pub struct Raw {
            pub x: i64,
        }
    );
    let make: syn::ItemFn = syn::parse_quote!(
        pub fn raw_make(x: i64) -> Raw {
            unimplemented!()
        }
    );
    let mode_of: syn::ItemFn = syn::parse_quote!(
        pub fn raw_mode(mode: Mode, shape: Shape) -> i64 {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Enum(mode), loc.clone()),
        (syn::Item::Enum(shape), loc.clone()),
        (syn::Item::Struct(raw), loc.clone()),
        (syn::Item::Fn(make), loc.clone()),
        (syn::Item::Fn(mode_of), loc),
    ]))
    .expect("index items");

    let src = write(
        base().mangle_type_name(|base| format!("{base}_t")).declare(
            decls!()
                .enum_type(enum_type!(Mode).base_name("speed"))
                .tagged_union(tagged_union!(Shape).base_name("figure"))
                .repr_c_type(repr_c_type!(Raw).base_name("packet"))
                .fun(fun!(raw_make).base_name("packet_make"))
                .fun(
                    fun!(raw_mode)
                        .base_name("packet_mode")
                        .abort_on_conversion_error(),
                ),
        ),
        registry,
        "decl_bases",
    );
    let compact: String = src.split_whitespace().collect();

    for name in ["speed_t", "figure_t", "packet_t"] {
        assert!(compact.contains(name), "{name} missing from {src}");
    }
    for symbol in ["fnpacket_make(", "fnpacket_mode("] {
        assert!(compact.contains(symbol), "{symbol} missing from {src}");
    }
    // The mirror's name is derived once: everything naming the wire type has to
    // agree with the emitted struct, or the file does not compile.
    assert!(
        !compact.contains("raw_t"),
        "the default mirror name survived beside the declared one: {src}"
    );
}

/// Two spellings of one callback signature are one declaration, and the second
/// would otherwise overwrite the first's options.
#[test]
fn one_callback_signature_spelled_two_ways_is_refused() {
    let message = catch_msg(|| {
        let _ = base().declare(
            decls!()
                .callback(callback!(impl Fn(i64) + Send + Sync + 'static).base_name("first"))
                .callback(callback!(impl Fn(i64) + Sync + Send + 'static).base_name("second")),
        );
    });
    assert!(message.contains("declared twice"), "{message}");
}

/// A second `api()` call cannot quietly reconfigure what the first declared.
#[test]
fn a_declaration_repeated_in_another_api_call_is_refused() {
    let message = catch_msg(|| {
        let _ = base()
            .declare(decls!().fun(fun!(point_make).base_name("first")))
            .declare(decls!().fun(fun!(point_make).base_name("second")));
    });
    assert!(message.contains("point_make"), "{message}");

    let message = catch_msg(|| {
        let _ = base()
            .declare(decls!().data_type(data_type!(Point)))
            .declare(decls!().data_type(data_type!(Point)));
    });
    assert!(message.contains("Point"), "{message}");
}
