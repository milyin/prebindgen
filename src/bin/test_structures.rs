use prebindgen_proc_macro::prebindgen;

// Define these structs to ensure they get processed during compilation
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

fn main() {
    // This binary ensures that the structs above are compiled and processed by prebindgen
    println!("Test structures compiled successfully");
}
