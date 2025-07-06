// Unit tests for the codegen module
//
// This module contains all the unit tests for the code generation functionality,
// including tests for function transformation, type validation, feature processing,
// and FFI stub generation.

use super::*;
use std::collections::{HashMap, HashSet};

// Helper function to create a default source location for tests
fn default_test_source_location() -> crate::SourceLocation {
    crate::SourceLocation {
        file: "test.rs".to_string(),
        line: 1,
        column: 1,
    }
}

// Helper function to create a default context for tests
fn default_test_context<'a>(
    crate_name: &'a str,
    exported_types: &'a HashSet<String>,
    allowed_prefixes: &'a [syn::Path],
    transparent_wrappers: &'a [syn::Path],
    edition: &'a str,
) -> Context<'a> {
    Context::new(crate_name, exported_types, allowed_prefixes, transparent_wrappers, edition)
}

#[test]
fn test_process_features_disable() {
    let content = r#"
#[cfg(feature = "experimental")]
pub struct ExperimentalStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

    let mut disabled_features = HashSet::new();
    disabled_features.insert("experimental".to_string());
    let enabled_features = HashSet::new();
    let feature_mappings = HashMap::new();

    let file = syn::parse_file(content).unwrap();
    let result = process_features(
        file,
        &disabled_features,
        &enabled_features,
        &feature_mappings,
    );
    let result_str = prettyplease::unparse(&result);

    // Should not contain the experimental struct
    assert!(!result_str.contains("ExperimentalStruct"));
    // Should still contain the regular struct
    assert!(result_str.contains("RegularStruct"));
}

#[test]
fn test_process_features_enable() {
    let content = r#"
#[cfg(feature = "std")]
pub struct StdStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

    let disabled_features = HashSet::new();
    let mut enabled_features = HashSet::new();
    enabled_features.insert("std".to_string());
    let feature_mappings = HashMap::new();

    let file = syn::parse_file(content).unwrap();
    let result = process_features(
        file,
        &disabled_features,
        &enabled_features,
        &feature_mappings,
    );
    let result_str = prettyplease::unparse(&result);

    // Should contain the std struct without cfg attribute
    assert!(result_str.contains("StdStruct"));
    assert!(!result_str.contains(r#"cfg(feature = "std")"#));
    // Should still contain the regular struct
    assert!(result_str.contains("RegularStruct"));
}

#[test]
fn test_process_features_mapping() {
    let content = r#"
#[cfg(feature = "unstable")]
pub struct UnstableStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

    let disabled_features = HashSet::new();
    let enabled_features = HashSet::new();
    let mut feature_mappings = HashMap::new();
    feature_mappings.insert("unstable".to_string(), "stable".to_string());

    let file = syn::parse_file(content).unwrap();
    let result = process_features(
        file,
        &disabled_features,
        &enabled_features,
        &feature_mappings,
    );
    let result_str = prettyplease::unparse(&result);

    // Should contain the struct with mapped feature name
    assert!(result_str.contains("UnstableStruct"));
    assert!(result_str.contains(r#"cfg(feature = "stable")"#));
    assert!(!result_str.contains(r#"cfg(feature = "unstable")"#));
    // Should still contain the regular struct
    assert!(result_str.contains("RegularStruct"));
}

#[test]
fn test_process_features_complex_syn_parsing() {
    let content = r#"
#[cfg(feature = "async")]
pub struct AsyncStruct {
    pub field: i32,
}

#[cfg(feature = "sync")]
impl AsyncStruct {
    pub fn new() -> Self {
        Self { field: 0 }
    }
}

#[cfg(feature = "deprecated")]
pub fn old_function() {
    // deprecated function
}

pub enum RegularEnum {
    A,
    B,
}
"#;

    let mut disabled_features = HashSet::new();
    disabled_features.insert("deprecated".to_string());

    let mut enabled_features = HashSet::new();
    enabled_features.insert("async".to_string());

    let mut feature_mappings = HashMap::new();
    feature_mappings.insert("sync".to_string(), "synchronous".to_string());

    let file = syn::parse_file(content).unwrap();
    let result = process_features(
        file,
        &disabled_features,
        &enabled_features,
        &feature_mappings,
    );
    let result_str = prettyplease::unparse(&result);

    // Should not contain the deprecated function
    assert!(!result_str.contains("old_function"));

    // Should contain AsyncStruct without cfg attribute
    assert!(result_str.contains("AsyncStruct"));
    assert!(!result_str.contains(r#"cfg(feature = "async")"#));

    // Should contain the impl block with mapped feature name
    assert!(result_str.contains("impl AsyncStruct"));
    assert!(result_str.contains(r#"cfg(feature = "synchronous")"#));
    assert!(!result_str.contains(r#"cfg(feature = "sync")"#));

    // Should still contain the regular enum
    assert!(result_str.contains("RegularEnum"));
}

#[test]
fn test_transform_function_to_stub() {
    let function_content = r#"
pub fn example_function(x: i32, y: &str) -> i32 {
    42
}
"#;

    let exported_types = HashSet::new();
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let _context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");
    let source_location = default_test_source_location();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &source_location,
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the no_mangle attribute
    assert!(result_str.contains("no_mangle"));
    // Should be an unsafe extern "C" function
    assert!(result_str.contains("unsafe extern \"C\""));
    // Should have original parameter names
    assert!(result_str.contains("x"));
    assert!(result_str.contains("y"));
    // Should convert &str to *const str in signature
    assert!(result_str.contains("*const str"));
    // Should convert pointer back to reference in function call
    assert!(result_str.contains("&*y"));
    // Should call the original function from the source crate
    assert!(result_str.contains("my_crate::example_function"));
}

#[test]
fn test_transform_function_to_stub_edition_2024() {
    let function_content = r#"
pub fn example_function() -> i32 {
    42
}
"#;

    let exported_types = HashSet::new();
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2024");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    ).unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the unsafe no_mangle attribute for 2024 edition
    assert!(result_str.contains("#[unsafe(no_mangle)]"));
}

#[test]
fn test_transform_function_to_stub_wrong_item_count() {
    // Test with empty file
    let empty_file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![],
    };

    let exported_types = HashSet::new();
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let result = transform_function_to_stub(
        empty_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    );

    match result {
        Err(error_msg) => assert!(error_msg.contains("Expected exactly one item")),
        Ok(_) => panic!("Expected error but got success"),
    }

    // Test with multiple items
    let function_content = r#"
pub fn first_function() -> i32 { 42 }
pub fn second_function() -> i32 { 24 }
"#;

    let multi_item_file = syn::parse_file(function_content).unwrap();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");
    let result = transform_function_to_stub(
        multi_item_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    );

    match result {
        Err(error_msg) => assert!(error_msg.contains("Expected exactly one item")),
        Ok(_) => panic!("Expected error but got success"),
    }
}

#[test]
fn test_transform_function_to_stub_wrong_item_type() {
    let struct_content = r#"
pub struct MyStruct {
    field: i32,
}
"#;

    let exported_types = HashSet::new();
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let struct_file = syn::parse_file(struct_content).unwrap();
    let result = transform_function_to_stub(
        struct_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    );

    match result {
        Err(error_msg) => assert!(error_msg.contains("Expected function item")),
        Ok(_) => panic!("Expected error but got success"),
    }
}

#[test]
fn test_transform_function_with_references() {
    let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> i32 {
    42
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("Bar".to_string());
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the no_mangle attribute
    assert!(result_str.contains("no_mangle"));
    // Should be an unsafe extern "C" function
    assert!(result_str.contains("unsafe extern \"C\""));
    // Should convert &mut T to *mut T
    assert!(result_str.contains("*mut"));
    // Should convert &T to *const T
    assert!(result_str.contains("*const"));
    // Should convert pointers back to references in function call
    assert!(result_str.contains("&mut *dst"));
    assert!(result_str.contains("&*src"));
    // Should call the original function from the source crate
    assert!(result_str.contains("my_crate::copy_bar"));
}

#[test]
fn test_transform_function_with_transparent_wrapper_assertions() {
    let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> std::mem::MaybeUninit<i32> {
    std::mem::MaybeUninit::new(42)
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("Bar".to_string());
    let allowed_prefixes = generate_standard_allowed_prefixes();

    let mut transparent_wrappers = Vec::new();
    let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
    transparent_wrappers.push(maybe_uninit_path);
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    // Generate assertions from collected pairs and append to result
    let assertions = generate_type_assertions(&assertion_type_pairs);
    let mut complete_result = result;
    complete_result.items.extend(assertions);

    let result_str = prettyplease::unparse(&complete_result);

    // Should contain the extern function
    assert!(result_str.contains("no_mangle"));
    assert!(result_str.contains("unsafe extern \"C\""));

    // Should contain compile-time assertions for size and alignment
    assert!(result_str.contains("std::mem::size_of"));
    assert!(result_str.contains("std::mem::align_of"));
    assert!(
        result_str.contains("Size mismatch between stub parameter type and source crate type")
    );
    assert!(
        result_str
            .contains("Alignment mismatch between stub parameter type and source crate type")
    );

    // Should have assertions for the stripped types (MaybeUninit only in this test)
    assert!(result_str.contains("MaybeUninit"));
}

#[test]
fn test_convert_reference_to_pointer() {
    // Test mutable reference conversion
    let mut_ref: syn::Type = syn::parse_quote! { &mut i32 };
    let converted = convert_reference_to_pointer(&mut_ref);
    let converted_str = quote::quote! { #converted }.to_string();
    assert!(converted_str.contains("* mut i32"));

    // Test immutable reference conversion
    let ref_type: syn::Type = syn::parse_quote! { &str };
    let converted = convert_reference_to_pointer(&ref_type);
    let converted_str = quote::quote! { #converted }.to_string();
    assert!(converted_str.contains("* const str"));

    // Test non-reference type (should remain unchanged)
    let regular_type: syn::Type = syn::parse_quote! { i32 };
    let converted = convert_reference_to_pointer(&regular_type);
    let converted_str = quote::quote! { #converted }.to_string();
    assert_eq!(converted_str, "i32");
}

#[test]
fn test_strip_transparent_wrapper() {
    let mut transparent_wrappers = Vec::new();
    let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
    transparent_wrappers.push(maybe_uninit_path);

    let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> i32 {
    42
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("Bar".to_string());
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the no_mangle attribute
    assert!(result_str.contains("no_mangle"));
    // Should be an unsafe extern "C" function
    assert!(result_str.contains("unsafe extern \"C\""));
    // Should strip MaybeUninit wrapper and convert &mut MaybeUninit<Bar> to *mut Bar
    assert!(result_str.contains("*mut Bar"));
    // Should convert &Bar to *const Bar
    assert!(result_str.contains("*const Bar"));
    // The function signature should NOT contain MaybeUninit (but assertions might)
    // Let's split the check - function signature vs assertions
    let lines: Vec<&str> = result_str.lines().collect();
    let function_lines: Vec<&str> = lines
        .iter()
        .take_while(|line| !line.contains("const _"))
        .cloned()
        .collect();
    let function_code = function_lines.join("\n");
    assert!(!function_code.contains("MaybeUninit"));
    // Should call the original function from the source crate
    assert!(result_str.contains("my_crate::copy_bar"));
}

#[test]
fn test_strip_transparent_wrappers_nested() {
    let transparent_wrappers = vec![
        syn::parse_quote! { std::mem::MaybeUninit },
        syn::parse_quote! { std::mem::ManuallyDrop },
    ];

    // Test nested transparent wrappers: MaybeUninit<ManuallyDrop<T>>
    let nested_type: syn::Type = syn::parse_quote! {
        std::mem::MaybeUninit<std::mem::ManuallyDrop<i32>>
    };

    let mut has_wrapper = false;
    let stripped =
        strip_transparent_wrappers(&nested_type, &transparent_wrappers, &mut has_wrapper);
    let stripped_str = quote::quote! { #stripped }.to_string();

    // Should strip both wrappers and leave just i32
    assert_eq!(stripped_str, "i32");

    // Should have detected wrappers
    assert!(has_wrapper);
}

#[test]
fn test_type_assertions_generation() {
    // Test the assertion generation function directly
    let mut assertion_type_pairs = HashSet::new();
    assertion_type_pairs.insert((
        "std::mem::MaybeUninit<i32>".to_string(),
        "my_crate::i32".to_string(),
    ));
    assertion_type_pairs.insert(("String".to_string(), "my_crate::String".to_string()));

    let assertions = generate_type_assertions(&assertion_type_pairs);
    assert_eq!(assertions.len(), 4); // 2 types × 2 assertions each (size + alignment)

    let assertions_str = assertions
        .iter()
        .map(|item| {
            let file = syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![item.clone()],
            };
            prettyplease::unparse(&file)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Should contain size and alignment checks
    assert!(assertions_str.contains("std::mem::size_of"));
    assert!(assertions_str.contains("std::mem::align_of"));
    assert!(
        assertions_str
            .contains("Size mismatch between stub parameter type and source crate type")
    );
    assert!(
        assertions_str
            .contains("Alignment mismatch between stub parameter type and source crate type")
    );
}

#[test]
fn test_exported_type_assertions() {
    let function_content = r#"
pub fn process_data(
    data: &MyExportedStruct,
    output: &mut AnotherExportedType,
) -> ExportedEnum {
    ExportedEnum::Success
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("MyExportedStruct".to_string());
    exported_types.insert("AnotherExportedType".to_string());
    exported_types.insert("ExportedEnum".to_string());

    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-source-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    // Generate assertions from collected pairs and append to result
    let assertions = generate_type_assertions(&assertion_type_pairs);
    let mut complete_result = result;
    complete_result.items.extend(assertions);

    let result_str = prettyplease::unparse(&complete_result);

    // Should contain the extern function
    assert!(result_str.contains("no_mangle"));
    assert!(result_str.contains("unsafe extern \"C\""));

    // Should contain compile-time assertions for exported types
    assert!(result_str.contains("std::mem::size_of"));
    assert!(result_str.contains("std::mem::align_of"));

    // Should have assertions comparing local types vs source crate types
    assert!(result_str.contains("my_source_crate::"));
    assert!(
        result_str.contains("Size mismatch between stub parameter type and source crate type")
    );
    assert!(
        result_str
            .contains("Alignment mismatch between stub parameter type and source crate type")
    );
}

#[test]
fn test_corrected_assertion_logic() {
    // Test case: function with transparent wrapper and exported type
    let function_content = r#"
pub fn test_func(wrapper: &std::mem::MaybeUninit<ExportedType>) -> ExportedType {
    ExportedType::default()
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("ExportedType".to_string());

    let allowed_prefixes = generate_standard_allowed_prefixes();

    let mut transparent_wrappers = Vec::new();
    let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
    transparent_wrappers.push(maybe_uninit_path);
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("source-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    // Generate assertions from collected pairs and append to result
    let assertions = generate_type_assertions(&assertion_type_pairs);
    let mut complete_result = result;
    complete_result.items.extend(assertions);

    let result_str = prettyplease::unparse(&complete_result);

    println!("Generated code:\n{result_str}");

    // Should contain the extern function
    assert!(result_str.contains("no_mangle"));
    assert!(result_str.contains("unsafe extern \"C\""));

    // Should contain compile-time assertions
    assert!(result_str.contains("const _:"));
    assert!(result_str.contains("std::mem::size_of"));
    assert!(result_str.contains("std::mem::align_of"));

    // Should have the correct assertion message
    assert!(
        result_str.contains("Size mismatch between stub parameter type and source crate type")
    );
    assert!(
        result_str
            .contains("Alignment mismatch between stub parameter type and source crate type")
    );

    // Should have assertions for:
    // 1. Parameter: Stripped type (ExportedType) vs original type (std::mem::MaybeUninit<source_crate::ExportedType>)
    // 2. Return type: ExportedType vs source_crate::ExportedType
    assert!(result_str.contains("source_crate::ExportedType"));
    assert!(result_str.contains("MaybeUninit < source_crate::ExportedType"));

    // Should NOT generate duplicate assertions - count occurrences
    let size_assert_count = result_str.matches("std::mem::size_of").count();
    let align_assert_count = result_str.matches("std::mem::align_of").count();

    // We expect exactly 2 assertions: one for parameter, one for return type
    // Each assertion has both size and alignment checks, so 4 total checks
    assert_eq!(
        size_assert_count, 4,
        "Expected exactly 4 size assertions (2 pairs)"
    );
    assert_eq!(
        align_assert_count, 4,
        "Expected exactly 4 alignment assertions (2 pairs)"
    );

    // Should have stripped the wrapper in the FFI signature (parameter should be *const ExportedType, not *const MaybeUninit<ExportedType>)
    assert!(result_str.contains("*const ExportedType"));
    assert!(!result_str.contains("*const std :: mem :: MaybeUninit"));
}

#[test]
fn test_convert_to_local_type_function() {
    let mut exported_types = HashSet::new();
    exported_types.insert("ExportedType".to_string());

    let transparent_wrappers = vec![syn::parse_quote! { std::mem::MaybeUninit }];
    let source_crate_ident = syn::Ident::new("test_crate", proc_macro2::Span::call_site());

    // Test with transparent wrapper + exported type
    let wrapped_type: syn::Type = syn::parse_quote! { std::mem::MaybeUninit<ExportedType> };
    let mut assertion_pairs = HashSet::new();
    let mut was_converted = false;
    let result = convert_to_local_type(
        &wrapped_type,
        &exported_types,
        &transparent_wrappers,
        &source_crate_ident,
        &mut assertion_pairs,
        &mut was_converted,
    );

    let local_str = quote::quote! { #result }.to_string();
    assert_eq!(local_str, "ExportedType"); // Stripped of wrapper
    assert!(was_converted); // Should be converted

    // Should have collected an assertion pair
    assert_eq!(assertion_pairs.len(), 1);
    let (local_type_str, original_type_str) = assertion_pairs.iter().next().unwrap();
    assert_eq!(local_type_str, "ExportedType");
    assert!(
        original_type_str.contains("std :: mem :: MaybeUninit < test_crate :: ExportedType >")
    );

    // Test with regular type that doesn't need conversion
    let regular_type: syn::Type = syn::parse_quote! { i32 };
    let mut assertion_pairs = HashSet::new();
    let mut was_converted = false;
    let result = convert_to_local_type(
        &regular_type,
        &exported_types,
        &transparent_wrappers,
        &source_crate_ident,
        &mut assertion_pairs,
        &mut was_converted,
    );

    let result_str = quote::quote! { #result }.to_string();
    assert_eq!(result_str, "i32"); // No change
    assert!(!was_converted); // Should not be converted
    assert_eq!(assertion_pairs.len(), 0); // No assertion pairs collected

    // Test with exported type but no wrapper
    let exported_only: syn::Type = syn::parse_quote! { ExportedType };
    let mut assertion_pairs = HashSet::new();
    let mut was_converted = false;
    let result = convert_to_local_type(
        &exported_only,
        &exported_types,
        &transparent_wrappers,
        &source_crate_ident,
        &mut assertion_pairs,
        &mut was_converted,
    );

    let local_str = quote::quote! { #result }.to_string();
    assert_eq!(local_str, "ExportedType"); // No change since no wrapper
    assert!(was_converted); // Should be converted due to exported type

    // Should have collected an assertion pair
    assert_eq!(assertion_pairs.len(), 1);
    let (local_type_str, original_type_str) = assertion_pairs.iter().next().unwrap();
    assert_eq!(local_type_str, "ExportedType");
    assert_eq!(original_type_str, "test_crate :: ExportedType");
}

#[test]
#[should_panic(expected = "FFI functions cannot have receiver arguments")]
fn test_transform_function_with_receiver_argument() {
    let method_content = r#"
impl MyStruct {
    pub fn method(&self, x: i32) -> i32 {
        42
    }
}
"#;

    // Parse the impl block to extract the method
    let file = syn::parse_file(method_content).unwrap();
    if let syn::Item::Impl(impl_block) = &file.items[0] {
        if let syn::ImplItem::Fn(method) = &impl_block.items[0] {
            // Create a file with just the method function
            let method_file = syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![syn::Item::Fn(syn::ItemFn {
                    attrs: method.attrs.clone(),
                    vis: method.vis.clone(),
                    sig: method.sig.clone(),
                    block: Box::new(method.block.clone()),
                })],
            };

            let exported_types = HashSet::new();
            let allowed_prefixes = generate_standard_allowed_prefixes();
            let transparent_wrappers = Vec::new();
            let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

            // This should panic because the method has a receiver argument (&self)
            let _result = transform_function_to_stub(
        method_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    );
        }
    }
}

#[test]
fn test_reference_return_type_no_unnecessary_transmute() {
    // Test case: function that returns reference to primitive type should not transmute return value
    let function_content = r#"
pub fn get_primitive_field(input: &ExportedStruct) -> &u64 {
    &input.field
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("ExportedStruct".to_string());

    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("test-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the extern function
    assert!(result_str.contains("no_mangle"));
    assert!(result_str.contains("unsafe extern \"C\""));

    // Parameter should convert &ExportedStruct to *const ExportedStruct with transmute
    assert!(result_str.contains("*const ExportedStruct"));
    assert!(result_str.contains("std::mem::transmute(&*input)"));

    // Return type should convert &u64 to *const u64 but WITHOUT transmute of return value
    assert!(result_str.contains("*const u64"));

    // Should NOT have transmute for the return value since u64 is primitive
    assert!(!result_str.contains("let result ="));
    assert!(!result_str.contains("unsafe { std::mem::transmute(result) }"));

    // Should directly return the function call
    assert!(result_str.contains("test_crate::get_primitive_field"));

    // Function body should be a simple call without transmute wrapping
    let lines: Vec<&str> = result_str.lines().collect();
    let function_lines: Vec<&str> = lines
        .iter()
        .take_while(|line| !line.contains("const _"))
        .cloned()
        .collect();
    let function_code = function_lines.join("\n");
    assert!(!function_code.contains("MaybeUninit"));
}

#[test]
fn test_reference_return_type_with_exported_type() {
    // Test case: function that returns reference to exported type should transmute return value
    let function_content = r#"
pub fn get_exported_field(input: &Wrapper) -> &ExportedType {
    &input.exported_field
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("Wrapper".to_string());
    exported_types.insert("ExportedType".to_string());

    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the extern function
    assert!(result_str.contains("no_mangle"));
    assert!(result_str.contains("unsafe extern \"C\""));

    // Parameter should convert &Wrapper to *const Wrapper with transmute
    assert!(result_str.contains("*const Wrapper"));
    assert!(result_str.contains("std::mem::transmute(&*input)"));

    // Return type should convert &ExportedType to *const ExportedType with transmute
    assert!(result_str.contains("*const ExportedType"));

    // Should have transmute for the return value since ExportedType is exported
    assert!(result_str.contains("let result ="));
    assert!(result_str.contains("unsafe { std::mem::transmute(result) }"));
}

#[test]
fn test_reference_to_primitive_no_transmute() {
    let function_content = r#"
pub fn get_field(input: &ExportedStruct) -> &u64 {
    &input.field
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("ExportedStruct".to_string());
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the no_mangle attribute
    assert!(result_str.contains("no_mangle"));
    // Should be an unsafe extern "C" function
    assert!(result_str.contains("unsafe extern \"C\""));
    // Parameter should be transmuted (it's a reference to exported type)
    assert!(result_str.contains("transmute(&*input)"));
    // Return value should NOT be transmuted (it's a reference to primitive)
    assert!(!result_str.contains("let result = my_crate::get_field"));
    assert!(!result_str.contains("transmute(result)"));
    // Should call the function directly without storing result
    assert!(result_str.contains("my_crate::get_field("));
}

#[test]
fn test_reference_to_exported_type_with_transmute() {
    let function_content = r#"
pub fn get_exported_field(input: &ExportedStruct) -> &ExportedType {
    &input.exported_field
}
"#;

    let mut exported_types = HashSet::new();
    exported_types.insert("ExportedStruct".to_string());
    exported_types.insert("ExportedType".to_string());
    let allowed_prefixes = generate_standard_allowed_prefixes();
    let transparent_wrappers = Vec::new();
    let mut assertion_type_pairs = HashSet::new();
    let context = default_test_context("my-crate", &exported_types, &allowed_prefixes, &transparent_wrappers, "2021");

    let input_file = syn::parse_file(function_content).unwrap();
    let result = transform_function_to_stub(
        input_file,
        &context,
        &mut assertion_type_pairs,
        &default_test_source_location(),
    )
    .unwrap();

    let result_str = prettyplease::unparse(&result);

    // Should contain the no_mangle attribute
    assert!(result_str.contains("no_mangle"));
    // Should be an unsafe extern "C" function
    assert!(result_str.contains("unsafe extern \"C\""));
    // Parameter should be transmuted (it's a reference to exported type)
    assert!(result_str.contains("transmute(&*input)"));
    // Return value SHOULD be transmuted (it's a reference to exported type)
    assert!(result_str.contains("let result = my_crate::get_exported_field"));
    assert!(result_str.contains("transmute(result)"));
}
