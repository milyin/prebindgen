// Include the copied example_ffi.rs file from OUT_DIR
include!(concat!(env!("OUT_DIR"), "/example_ffi_structs.rs"));
include!(concat!(env!("OUT_DIR"), "/example_ffi_functions.rs"));

// Demonstrate that we can use the included Foo struct
pub fn create_foo(id: u64) -> Foo {
    Foo {
        #[cfg(target_arch = "x86_64")]
        x86_64_field: id,
        #[cfg(target_arch = "aarch64")]
        aarch64_field: id,
    }
}
// Demonstrate that we can use the included Bar struct
pub fn create_bar(id: u64) -> Bar {
    Bar {
        #[cfg(target_arch = "x86_64")]
        x86_64_field: id,
        #[cfg(target_arch = "aarch64")]
        aarch64_field: id,
    }
}

pub fn test_calling_generated_functions() {
    // These functions should be available from the generated example_ffi.rs
    let _result = unsafe { test_function(5, 1.234) };
    let _flag = unsafe { another_test_function() };
}
