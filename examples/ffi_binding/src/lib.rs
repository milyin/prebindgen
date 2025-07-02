// Include the copied ffi_common.rs file from OUT_DIR
include!(concat!(env!("OUT_DIR"), "/ffi_common.rs"));

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
    // These functions should be available from the generated ffi_common.rs
    let _result = test_function(42, 3.14);
    let _flag = another_test_function();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_function_calls() {
        // Test that the generated extern "C" functions can be called
        // and actually invoke the original source functions
        let result = test_function(10, 2.5);
        // The original function in ffi_common returns a + b as i32
        // So 10 + 2.5 = 12 (rounded down)
        assert_eq!(result, 12);

        let flag = another_test_function();
        // The original function returns false
        assert_eq!(flag, false);

        // Test void function (should not panic or crash)
        void_function(42);
    }
}