//! Test structures binary for integration testing
//! This generates the prebindgen.json file that tests can examine

use prebindgen_proc_macro::{prebindgen, prebindgen_path};

// Define test structures that will be processed by prebindgen
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

// Generate path constant for tests
pub const TEST_PATH: &str = prebindgen_path!();

pub fn main() {
    println!("Test structures binary executed");
    println!("Generated prebindgen file at: {}", TEST_PATH);
}
