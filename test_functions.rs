use prebindgen_proc_macro::prebindgen;

// Test simple function
#[prebindgen]
pub fn simple_function() {
    println!("This implementation will be ignored");
}

// Test function with parameters and return value
#[prebindgen]
pub fn function_with_params(x: i32, y: f64, name: &str) -> bool {
    x > 0 && y > 0.0 && !name.is_empty()
}

// Test function with complex types
#[prebindgen]
pub fn complex_function(data: *const u8, len: usize) -> *mut i32 {
    std::ptr::null_mut()
}

// Test function with references (this might be challenging for FFI)
#[prebindgen]
pub fn function_with_ref(value: &i32) -> i32 {
    *value
}

// Test struct to make sure structs still work
#[prebindgen]
#[repr(C)]
pub struct TestStruct {
    pub field1: i32,
    pub field2: f64,
}
