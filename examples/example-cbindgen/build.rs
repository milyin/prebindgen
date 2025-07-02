use prebindgen::Prebindgen;

fn main() {
    let mut pb = Prebindgen::new(example_ffi::PREBINDGEN_OUT_DIR, "example_ffi");
    
    // Read all available groups
    pb.read_all();

    // Create a file and append all groups to it
    let bindings_file = pb
        .create("example_ffi.rs")
        .append_all()
        .into_path();

    println!(
        "cargo:warning=Generated bindings at: {}",
        bindings_file.display()
    );
}
