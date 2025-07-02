use prebindgen::Prebindgen;

fn main() {
    // Initialize Prebindgen and process JSON groups
    // Read JSON output directory from ffi_common buildscript via Cargo metadata
    let src_dir = ffi_common::PREBINDGEN_OUT_DIR;
    let mut pb = Prebindgen::new(src_dir, "ffi_common".to_string());
    pb.read("structs");
    pb.read("functions");
    pb.make_rs("structs", "ffi_common_structs.rs");
    pb.make_rs("functions", "ffi_common_functions.rs");
}
