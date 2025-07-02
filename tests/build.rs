fn main() {
    let generated_path = prebindgen::get_prebindgen_json_path();
    // Use println! with cargo:warning to make output visible
    println!("cargo:warning=Generated path: {:?}", generated_path);

    // let _ = std::fs::remove_file(generated_path);

    // This build script just ensures that OUT_DIR is available for tests
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
}
