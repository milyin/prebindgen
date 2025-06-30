// This module contains all the test definitions that should be copied by prebindgen
// All test files will use this module to ensure consistent state

use prebindgen::{prebindgen, prebindgen_path};

// Test structures for copy_structs_enums tests
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

// Test structures for path_access tests
#[prebindgen]
pub struct PathTestStruct {
    pub id: u64,
    pub name: String,
}

#[prebindgen]
pub enum PathTestEnum {
    Alpha,
    Beta(String),
    Gamma { value: i32 },
}

// Test structures for no_out_dir tests
#[prebindgen]
pub struct NoOutDirStruct {
    pub data: String,
    pub timestamp: u64,
}

#[prebindgen]
pub enum NoOutDirEnum {
    State1,
    State2 { info: String },
    State3(i32, String),
}

// Generate path constants that all tests can use
prebindgen_path!(TEST_PATH);
prebindgen_path!(CUSTOM_PATH);
prebindgen_path!(); // This creates PREBINDGEN_PATH
