use prebindgen_tests::TEST_PATH;
use prebindgen::{prebindgen};
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
    
    // Verify struct definitions are present
    assert!(
        content.contains("TestStruct"),
        "TestStruct should be copied to prebindgen.rs"
    );
    
    assert!(
        content.contains("name : String") || content.contains("name: String"),
        "TestStruct fields should be preserved"
    );
    
    assert!(
        content.contains("value : i32") || content.contains("value: i32"),
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
    
    // Verify enum definitions are present
    assert!(
        content.contains("TestEnum"),
        "TestEnum should be copied to prebindgen.rs"
    );
    
    assert!(
        content.contains("Simple"),
        "Simple enum variant should be preserved"
    );
    
    assert!(
        content.contains("WithData"),
        "Tuple variant should be preserved"
    );
    
    assert!(
        content.contains("WithFields"),
        "Struct variant should be preserved"
    );
    
    assert!(
        content.contains("Complex"),
        "Complex variant should be preserved"
    );
    
    // Check nested enum
    assert!(
        content.contains("TestNestedEnum"),
        "Nested enum should be copied"
    );
    
    println!("✅ Enum copying test passed");
}

#[test]
fn test_generic_struct_copying() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Verify generic struct is copied
    assert!(
        content.contains("GenericStruct"),
        "Generic struct should be copied to prebindgen.rs"
    );
    
    // Check that generics are preserved
    assert!(
        content.contains("<T>") || content.contains("< T >"),
        "Generic parameters should be preserved"
    );
    
    println!("✅ Generic struct copying test passed");
}

#[test]
fn test_derive_attributes_preserved() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Verify derive attributes are preserved
    assert!(
        content.contains("Debug") && content.contains("Clone") && content.contains("PartialEq"),
        "Derive attributes should be preserved in copied definitions"
    );
    
    println!("✅ Derive attributes preservation test passed");
}

#[test]
fn test_no_duplicate_definitions() {
    let prebindgen_path = format!("{}/prebindgen.rs", TEST_PATH);
    
    let content = fs::read_to_string(&prebindgen_path)
        .expect("Should be able to read prebindgen.rs");
    
    // Count occurrences of each definition
    let test_struct_count = content.matches("struct TestStruct").count();
    let test_enum_count = content.matches("enum TestEnum").count();
    
    // Each should appear only once (no duplicates)
    assert_eq!(
        test_struct_count, 1,
        "TestStruct should appear exactly once, found: {}",
        test_struct_count
    );
    
    assert_eq!(
        test_enum_count, 1,
        "TestEnum should appear exactly once, found: {}",
        test_enum_count
    );
    
    println!("✅ No duplicate definitions test passed");
}
