//! Build script for generating C bindings using prebindgen and cbindgen.
//!
//! This example demonstrates how to use prebindgen in a language-specific binding crate
//! to generate C headers from a common FFI library.

use std::path::PathBuf;

fn main() {
    // Create a prebindgen builder using the output directory from the common FFI crate
    // The PREBINDGEN_OUT_DIR constant is exported by example_ffi and contains the path
    // to the prebindgen data files generated during example_ffi's build
    let pb = prebindgen::Builder::new(example_ffi::PREBINDGEN_OUT_DIR)
        .edition("2024") // Use Rust 2024 edition features like #[unsafe(no_mangle)]
        .build();

    // Generate FFI bindings for all groups (structs, functions, etc.)
    // This creates extern "C" wrapper functions that call back to the original crate
    let bindings_file = pb.all().write_to_file("example_ffi.rs");

    println!(
        "cargo:warning=Generated bindings at: {}",
        bindings_file.display()
    );

    // Generate C headers using cbindgen directly from the generated bindings
    // This demonstrates the separation: prebindgen generates the FFI code,
    // cbindgen generates the C headers from that code
    generate_c_headers(&bindings_file);
}

/// Generate C header files using cbindgen from the prebindgen-generated Rust code.
///
/// This function demonstrates how to integrate prebindgen with cbindgen:
/// 1. prebindgen generates extern "C" wrapper functions in Rust
/// 2. cbindgen processes those functions to create C header files
/// 3. The result is a clean separation between common FFI logic and C-specific bindings
fn generate_c_headers(cleaned_bindings_file: &PathBuf) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let header_path = PathBuf::from(&crate_dir).join("include/example_ffi.h");

    // Use cbindgen to generate C headers from the prebindgen-generated Rust file
    // The key insight: we pass the generated Rust file as a source to cbindgen
    // This allows cbindgen to see the extern "C" functions without them being
    // directly defined in this crate (which would require cdylib/staticlib)
    match cbindgen::Builder::new()
        .with_config(config)
        .with_crate(&crate_dir)
        .with_src(cleaned_bindings_file) // Use the prebindgen-generated file as source
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(&header_path);
            println!(
                "cargo:warning=Generated C headers at: {}",
                header_path.display()
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to generate C headers: {:?}", e);
        }
    }
}
