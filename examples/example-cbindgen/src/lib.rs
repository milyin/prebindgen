// Include the combined example_ffi bindings file from OUT_DIR
include!(concat!(env!("OUT_DIR"), "/example_ffi.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    #[test]
    fn test_all_exported_functions() {
        unsafe {
            // Test simple functions without parameters
            let bool_result = another_test_function();
            println!("another_test_function() returned: {}", bool_result);

            // Test function with basic parameters
            let int_result = test_function(42, 42.42);
            println!("test_function(42, 42.42) returned: {}", int_result);

            // Test void function
            void_function(123);
            println!("void_function(123) called successfully");

            // Test functions with struct parameters and return values
            
            // Create test Foo struct
            let test_foo = Foo {
                #[cfg(target_arch = "x86_64")]
                x86_64_field: 12345,
                #[cfg(target_arch = "aarch64")]
                aarch64_field: 12345,
                stable_field: 54321,
            };

            // Test get_foo_field function
            let foo_field_ptr = get_foo_field(&test_foo as *const Foo);
            assert!(!foo_field_ptr.is_null());
            let foo_field_value = *foo_field_ptr;
            println!("get_foo_field returned field value: {foo_field_value}");
            assert_eq!(foo_field_value, 12345);

            // Test copy_foo function
            let mut dst_foo = MaybeUninit::<Foo>::uninit();
            let result = copy_foo(dst_foo.as_mut_ptr(), &test_foo as *const Foo);
            assert_eq!(result, EXAMPLE_RESULT_OK);
            let copied_foo = dst_foo.assume_init();
            
            #[cfg(target_arch = "x86_64")]
            assert_eq!(copied_foo.x86_64_field, test_foo.x86_64_field);
            #[cfg(target_arch = "aarch64")]
            assert_eq!(copied_foo.aarch64_field, test_foo.aarch64_field);
            
            println!("copy_foo completed successfully, result: {result}");

            // Create test Bar struct
            let test_bar = Bar {
                #[cfg(target_arch = "x86_64")]
                x86_64_field: 67890,
                #[cfg(target_arch = "aarch64")]
                aarch64_field: 67890,
            };

            // Test copy_bar function
            let mut dst_bar = MaybeUninit::<Bar>::uninit();
            let result = copy_bar(dst_bar.as_mut_ptr(), &test_bar as *const Bar);
            assert_eq!(result, EXAMPLE_RESULT_OK);
            let copied_bar = dst_bar.assume_init();
            
            #[cfg(target_arch = "x86_64")]
            assert_eq!(copied_bar.x86_64_field, test_bar.x86_64_field);
            #[cfg(target_arch = "aarch64")]
            assert_eq!(copied_bar.aarch64_field, test_bar.aarch64_field);
            
            println!("copy_bar completed successfully, result: {result}");

            // Test constants
            assert_eq!(EXAMPLE_RESULT_OK, 0);
            assert_eq!(EXAMPLE_RESULT_ERROR, -1);
            println!("Constants verified: EXAMPLE_RESULT_OK = {EXAMPLE_RESULT_OK}, EXAMPLE_RESULT_ERROR = {EXAMPLE_RESULT_ERROR}"); 
        }
    }

    #[test]
    fn test_struct_sizes_and_alignments() {
        // Verify that the generated structs have reasonable sizes
        println!("Foo size: {}, alignment: {}", std::mem::size_of::<Foo>(), std::mem::align_of::<Foo>());
        println!("Bar size: {}, alignment: {}", std::mem::size_of::<Bar>(), std::mem::align_of::<Bar>());
        
        // Basic sanity checks
        assert!(std::mem::size_of::<Foo>() >= 8); // Should be at least size of u64
        assert!(std::mem::size_of::<Bar>() >= 8); // Should be at least size of u64
        assert!(std::mem::align_of::<Foo>() >= std::mem::align_of::<u64>());
        assert!(std::mem::align_of::<Bar>() >= std::mem::align_of::<u64>());
    }

    #[test]
    fn test_error_handling() {
        unsafe {
            // Test functions that should return error codes
            // Note: These functions may not actually return errors in this simple example,
            // but we can test that they don't crash with valid inputs
            
            let test_foo = Foo {
                #[cfg(target_arch = "x86_64")]
                x86_64_field: 999,
                #[cfg(target_arch = "aarch64")]
                aarch64_field: 999,
                stable_field: 777,
            };

            let mut dst_foo = MaybeUninit::<Foo>::uninit();
            let result = copy_foo(dst_foo.as_mut_ptr(), &test_foo as *const Foo);
            
            // Should return OK for valid operations
            assert_eq!(result, EXAMPLE_RESULT_OK);
        }
    }
}