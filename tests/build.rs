fn main() {
    let generated_path = prebindgen::get_prebindgen_json_path();
    // Use println! to show as info message
    eprintln!("Generated path: {:?}", generated_path);

    prebindgen::init_prebindgen_json();

    // This build script just ensures that OUT_DIR is available for tests
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
}
