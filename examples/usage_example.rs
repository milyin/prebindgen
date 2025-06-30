use prebindgen::prebindgen;

#[prebindgen]
pub struct MyStruct {
    pub name: String,
    pub age: u32,
}

#[prebindgen]
pub enum MyEnum {
    Variant1,
    Variant2(String),
    Variant3 { field: i32 },
}

fn main() {
    println!("Example demonstrating the prebindgen attribute macro");
    println!("Check the target/debug/build/*/out/prebindgen.rs file to see the copied definitions");
}
