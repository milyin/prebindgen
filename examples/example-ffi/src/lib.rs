use std::mem;

use prebindgen_proc_macro::{prebindgen, prebindgen_out_dir};

pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();

#[prebindgen]
#[allow(non_camel_case_types)]
pub type example_result = i8;

#[prebindgen]
pub const EXAMPLE_RESULT_OK: example_result = 0;
#[prebindgen]
pub const EXAMPLE_RESULT_ERROR: example_result = -1;

// Simulate the situation when part of the ffi code is generated
// on build.rs stage. This may cause the problem with cross-compilation,
// so we need to take additional measures to ensure the code generated by
// `#[prebindgen]` macro and available by `prebindgen_path!()` macro
// is correct for the target architecture.
// See build.rs and comment below for details.
include!(concat!(env!("OUT_DIR"), "/bar.rs"));

// This structure is added to test if the Bar generated correctly in build.rs
//
// The problem is that the crates dependent on the example-ffi adds example-ffi to their build.rs dependencies
// and by this extracts and processes the `#[prebindgen]`-marked code.
// So the generation of Bar is performed on the host architecture.
// The example-ffi/build.rs should ensure that both variants of Bar are generated (correctly marked with `#[cfg(target_arch=...`)] tags):
// - one for the host architecture (std::env::var("TARGET") in build.rs) - to ensure that the code compiles in example-cbindgen/build.rs
// - one for the target architecture (std::env::var("CROSS_TARGET") in build.rs) - to allow #[prebindgen] macro to generate correct code for the target architecture
//
// If generation is wrong (Bar is generated to wrong architecture), the example-cbindgen should fail to compile due to From<Bar> for LocalBar implementation
#[prebindgen("structs")]
pub struct LocalBar {
    #[cfg(target_arch = "x86_64")]
    pub x86_64_field: u64,
    #[cfg(target_arch = "aarch64")]
    pub aarch64_field: u64,
}

impl From<Bar> for LocalBar {
    fn from(bar: Bar) -> Self {
        #[cfg(target_arch = "x86_64")]
        return LocalBar {
            x86_64_field: bar.x86_64_field,
        };
        #[cfg(target_arch = "aarch64")]
        return LocalBar {
            aarch64_field: bar.aarch64_field,
        };
    }
}

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
pub fn copy_bar(dst: &mut mem::MaybeUninit<Bar>, src: &Bar) -> example_result {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
    EXAMPLE_RESULT_OK
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
