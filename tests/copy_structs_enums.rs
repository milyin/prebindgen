use prebindgen::{prebindgen, prebindgen_path};
use std::fs;

// Define these structs directly in the test to ensure they get processed
#[prebindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct TestStruct {
    pub name: String,
    pub value: i32,
    pub active: bool,
    pub optional_field: Option<f64>,
}

#[prebindgen]
#[derive(Debug, PartialEq)]
pub enum TestEnum {
    Simple,
    WithData(String),
    WithFields { id: u64, description: String },
    Complex { 
        nested: TestNestedEnum,
        count: usize,
    },
}

#[prebindgen]
#[derive(Debug, PartialEq)]
pub enum TestNestedEnum {
    First,
    Second(i32),
}

#[prebindgen]
pub struct GenericStruct<T> {
    pub data: T,
    pub meta: String,
}

// Generate path constant for tests
prebindgen_path!(TEST_PATH);

#[test]
fn test_struct_copying() {
    // Get the prebindgen file path using our path constant
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    
    // Verify the file exists
    assert!(
        std::path::Path::new(&prebindgen_path).exists(),
        "prebindgen.rs should exist at: {}",
        prebindgen_path
    );
    
    // Read the generated content
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    println!("Generated content:\n{}", content);
    
    // Verify TestStruct was copied
    assert!(
        content.contains("TestStruct"),
        "TestStruct should be present in generated file"
    );
    
    // Verify all fields are preserved
    assert!(
        content.contains("name") && content.contains("String"),
        "TestStruct fields should be preserved"
    );
    
    assert!(
        content.contains("value") && content.contains("i32"),
        "TestStruct fields should be preserved"
    );
    
    assert!(
        content.contains("active : bool") || content.contains("active: bool"),
        "TestStruct fields should be preserved"
    );
    
    assert!(
        content.contains("optional_field") && content.contains("Option"),
        "Optional fields should be preserved"
    );
    
    println!("✅ Struct copying test passed");
}

#[test]
fn test_enum_copying() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Verify TestEnum was copied
    assert!(
        content.contains("TestEnum"),
        "TestEnum should be present in generated file"
    );
    
    // Verify enum variants are preserved
    assert!(
        content.contains("Simple"),
        "Simple enum variant should be preserved"
    );
    
    assert!(
        content.contains("WithData"),
        "WithData enum variant should be preserved"
    );
    
    assert!(
        content.contains("WithFields"),
        "WithFields enum variant should be preserved"
    );
    
    assert!(
        content.contains("Complex"),
        "Complex enum variant should be preserved"
    );
    
    println!("✅ Enum copying test passed");
}

#[test]
fn test_generic_struct_copying() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Verify GenericStruct was copied with generics
    assert!(
        content.contains("GenericStruct"),
        "GenericStruct should be present in generated file"
    );
    
    assert!(
        content.contains("< T >") || content.contains("<T>"),
        "Generic parameter should be preserved"
    );
    
    println!("✅ Generic struct copying test passed");
}

#[test]
fn test_derive_attributes_preserved() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Check for derive attributes (they should be preserved)
    assert!(
        content.contains("derive") || content.contains("Debug") || content.contains("PartialEq"),
        "Derive attributes should be preserved in copied definitions"
    );
    
    println!("✅ Derive attributes preservation test passed");
}

#[test]
fn test_no_duplicate_definitions() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Count occurrences of TestStruct
    let test_struct_count = content.matches("struct TestStruct").count();
    assert_eq!(
        test_struct_count, 1,
        "TestStruct should appear exactly once, found: {}",
        test_struct_count
    );
    
    // Count occurrences of TestEnum  
    let test_enum_count = content.matches("enum TestEnum").count();
    assert_eq!(
        test_enum_count, 1,
        "TestEnum should appear exactly once, found: {}",
        test_enum_count
    );
    
    println!("✅ No duplicate definitions test passed");
}
