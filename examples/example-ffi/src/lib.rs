use std::mem;

use prebindgen_proc_macro::prebindgen;

#[prebindgen]
#[allow(non_camel_case_types)]
pub type example_result = i8;

#[prebindgen]
pub const EXAMPLE_RESULT_OK: example_result = 0;
#[prebindgen]
pub const EXAMPLE_RESULT_ERROR: example_result = -1;

// Separate mod to demonstrate "prefixed_exported_type" feature
pub mod foo {
    use prebindgen_proc_macro::prebindgen;

    #[prebindgen("structs")]
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Default)]
    pub enum InsideFoo {
        #[default]
        DouddleDee = 42,
        DouddleDum = 24,
    }

    #[prebindgen("structs")]
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Default)]
    pub struct Foo {
        // Demonstrate that for #cfg macro all works transparently
        #[cfg(target_arch = "x86_64")]
        pub x86_64_field: u64,
        #[cfg(target_arch = "aarch64")]
        pub aarch64_field: u64,
        #[cfg(feature = "unstable")]
        pub unstable_field: u64,
        #[cfg(not(feature = "unstable"))]
        pub stable_field: u64,
        #[cfg(any(feature = "unstable", feature = "internal"))]
        pub unstable_or_internal_field: u64,
        #[cfg(all(feature = "unstable", feature = "internal"))]
        pub unstable_and_internal_field: u64,
        pub inside: InsideFoo,
    }
}

#[prebindgen("functions")]
pub fn copy_foo(dst: &mut mem::MaybeUninit<foo::Foo>, src: &foo::Foo) -> example_result {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
    EXAMPLE_RESULT_OK
}

#[prebindgen("functions")]
#[cfg(feature = "unstable")]
pub fn get_unstable_field(input: &foo::Foo) -> u64 {
    // Return the unstable field if it exists
    input.unstable_field
}

#[prebindgen("functions")]
#[cfg(not(feature = "unstable"))]
pub fn get_unstable_field(input: &foo::Foo) -> u64 {
    // Return the unstable field if it exists
    input.stable_field
}

#[prebindgen("functions")]
pub fn test_function(a: i32, b: f64) -> i32 {
    // This implementation will be ignored, only the signature is stored
    a + b as i32
}

#[prebindgen("functions")]
pub fn another_test_function() -> example_result {
    EXAMPLE_RESULT_OK
}

// demonstrate auto remove of mut from the function signature in proxy function
#[prebindgen("functions")]
pub fn show_square(mut x: i32) {
    x *= x;
    println!("Square of the number is: {x}");
}

#[prebindgen("functions")]
pub fn get_foo_field(input: &foo::Foo) -> &u64 {
    // Return reference to the architecture-specific field
    #[cfg(target_arch = "x86_64")]
    return &input.x86_64_field;
    #[cfg(target_arch = "aarch64")]
    return &input.aarch64_field;
}

#[prebindgen("functions")]
pub fn get_foo_reference(input: &foo::Foo) -> &foo::Foo {
    // Return reference to the Foo struct
    input
}

#[prebindgen("functions")]
pub fn reference_to_array(input: &mut [u8; 4]) -> &[u8; 4] {
    input
}

#[prebindgen("functions")]
pub fn array_of_references<'a, 'b>(input: &'a [&'b u8; 4]) -> &'a [&'b u8; 4] {
    input
}

#[prebindgen("functions")]
pub fn array_of_arrays(input: &'static [[u8; 4]; 2]) -> &'static [[u8; 4]; 2] {
    input
}

#[prebindgen("functions")]
pub fn option_to_reference(input: Option<&u8>) -> Option<&u8> {
    input
}

#[prebindgen("functions")]
pub fn function_parameter(pfoo: &foo::Foo, f: Option<extern "C" fn(Option<&foo::Foo>) -> i32>) {
    if let Some(func) = f {
        // Call the function with the Foo reference
        let result = func(Some(pfoo));
        println!("Function returned: {result}");
    } else {
        println!("No function provided");
    }
}
