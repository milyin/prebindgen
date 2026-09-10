//! The declaration tree says exactly what the declarators say, in any order.

use super::*;
use crate::{
    callback, data_type, enum_type, error_type, fun, module, ptr_type, repr_c_type, tagged_union,
    value_type,
};

/// A record crossing by value and a function reading one of its fields.
fn items() -> Vec<(syn::Item, SourceLocation)> {
    let loc = SourceLocation::default();
    let record: syn::ItemStruct = syn::parse_quote!(
        pub struct Point {
            pub x: i64,
        }
    );
    let function: syn::ItemFn = syn::parse_quote!(
        pub fn point_x(point: Point) -> i64 {
            unimplemented!()
        }
    );
    declare_referenced([
        (syn::Item::Struct(record), loc.clone()),
        (syn::Item::Fn(function), loc),
    ])
}

fn registry() -> RegistryBuilder {
    crate::test_util::reg_from_items(items()).expect("index items")
}

fn base() -> CbindgenBuilder {
    CbindgenBuilder::new().source_module(syn::parse_quote!(source))
}

/// The tree lowers to the same binding the declarators build by hand.
#[test]
fn a_tree_and_the_declarators_it_lowers_to_agree() {
    let declared = write(
        base()
            .data_struct(syn::parse_quote!(Point))
            .base_name("point")
            .function(syn::parse_quote!(point_x))
            .panic(),
        registry(),
        "decl_flat",
    );
    let tree = write(
        base().module(
            module!().data_type(
                data_type!(Point)
                    .base_name("point")
                    .method(fun!(point_x).panic()),
            ),
        ),
        registry(),
        "decl_tree",
    );
    assert_eq!(declared, tree);
}

/// Declaring the function before the type it belongs to changes nothing: a
/// modifier belongs to its own declaration, not to whatever preceded it.
#[test]
fn the_order_of_a_tree_does_not_change_it() {
    let types_first = write(
        base().module(
            module!()
                .data_type(data_type!(Point).base_name("point"))
                .fun(fun!(point_x).panic()),
        ),
        registry(),
        "decl_types_first",
    );
    let functions_first = write(
        base().module(
            module!()
                .fun(fun!(point_x).panic())
                .data_type(data_type!(Point).base_name("point")),
        ),
        registry(),
        "decl_functions_first",
    );
    assert_eq!(types_first, functions_first);
}

/// Every kind of declaration the tree carries reaches the builder, including
/// the ones neither example declares.
#[test]
fn every_declaration_kind_reaches_the_builder() {
    let built = base().module(
        module!()
            .ptr_type(ptr_type!(Handle))
            .data_type(data_type!(Point))
            .enum_type(enum_type!(Mode))
            .tagged_union(tagged_union!(Shape))
            .value_type(value_type!(Owned, OwnedOpaque))
            .value_type(value_type!(Plain, PlainOpaque).data())
            .repr_c_type(repr_c_type!(Raw))
            .error_type(error_type!(Failure, failure_message))
            .callback(callback!(impl Fn(i64) + Send + Sync + 'static).base_name("value"))
            .fun(fun!(point_x))
            .ignore_fun(syn::parse_quote!(unused_fn))
            .ignore_type(syn::parse_quote!(Unused)),
    );

    let key = |ty: syn::Type| prebindgen_registry::TypeKey::from_type(&ty);
    assert!(built.opaque.contains_key(&key(syn::parse_quote!(Handle))));
    assert!(built.data.contains_key(&key(syn::parse_quote!(Point))));
    assert!(built.enums.contains_key(&key(syn::parse_quote!(Mode))));
    assert!(built
        .tagged_unions
        .contains_key(&key(syn::parse_quote!(Shape))));
    assert!(built
        .value_opaque
        .contains_key(&key(syn::parse_quote!(Owned))));
    assert!(built
        .value_opaque
        .contains_key(&key(syn::parse_quote!(Plain))));
    assert!(built
        .value_opaque
        .contains_key(&key(syn::parse_quote!(Raw))));
    assert!(built
        .opaque_errors
        .contains_key(&key(syn::parse_quote!(Failure))));
    assert_eq!(built.callbacks.len(), 1);
    assert!(built.functions.contains_key(&syn::parse_quote!(point_x)));
    assert!(built
        .ignored_functions
        .contains(&syn::parse_quote!(unused_fn)));
    assert!(built
        .ignored_types
        .contains(&key(syn::parse_quote!(Unused))));
}
