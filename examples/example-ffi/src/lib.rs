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
// See build.rs for details.
include!(concat!(env!("OUT_DIR"), "/bar.rs"));

#[prebindgen("structs")]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
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
}

#[prebindgen("functions")]
pub fn copy_foo(dst: &mut std::mem::MaybeUninit<Foo>, src: &Foo) -> example_result {
    unsafe {
        dst.as_mut_ptr().write(*src);
    }
    EXAMPLE_RESULT_OK
}

#[prebindgen("functions")]
#[cfg(feature = "unstable")]
pub fn get_unstable_field(input: &Foo) -> u64 {
    // Return the unstable field if it exists
    input.unstable_field
}

#[prebindgen("functions")]
#[cfg(not(feature = "unstable"))]
pub fn get_unstable_field(input: &Foo) -> u64 {
    // Return the unstable field if it exists
    input.stable_field
}

#[prebindgen("functions")]
pub fn copy_bar(dst: &mut std::mem::MaybeUninit<Bar>, src: &Bar) -> example_result {
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
pub fn another_test_function() -> bool {
    false
}

#[prebindgen("functions")]
pub fn void_function(x: i32) {
    println!("Called void_function with x = {x}");
}

#[prebindgen("functions")]
pub fn get_foo_field(input: &Foo) -> &u64 {
    // Return reference to the architecture-specific field
    #[cfg(target_arch = "x86_64")]
    return &input.x86_64_field;
    #[cfg(target_arch = "aarch64")]
    return &input.aarch64_field;
}

#[prebindgen("functions")]
pub fn get_foo_reference(input: &Foo) -> &Foo {
    // Return reference to the Foo struct
    input
}

#[prebindgen("functions")]
pub fn reference_to_array(input: &mut [u8; 4]) -> &[u8; 4] {
    input
}

#[prebindgen("functions")]
pub fn array_of_references<'a,'b>(input: &'a[&'b u8; 4]) -> &'a[&'b u8; 4] {
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