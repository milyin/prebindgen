use std::path::PathBuf;

fn main() {
    let pb = prebindgen::Builder::new(example_ffi::PREBINDGEN_OUT_DIR)
        .edition("2021")
        .build();

    // Create a file and append all groups to it
    let bindings_file = pb.all().write_to_file("example_ffi.rs");

    println!(
        "cargo:warning=Generated bindings at: {}",
        bindings_file.display()
    );

    // Generate C headers using cbindgen directly from the generated bindings
    generate_c_headers(&bindings_file);
}

fn generate_c_headers(cleaned_bindings_file: &PathBuf) {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let header_path = PathBuf::from(&crate_dir).join("include/example_ffi.h");

    match cbindgen::Builder::new()
        .with_config(config)
        .with_crate(&crate_dir)
        .with_src(cleaned_bindings_file)
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
