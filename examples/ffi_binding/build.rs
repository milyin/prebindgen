use prebindgen::Prebindgen;

fn main() {
    // Initialize Prebindgen and process JSON groups
    let mut pb = Prebindgen::new();
    // Read JSON output directory from ffi_common buildscript via Cargo metadata
    let src_dir = ffi_common::PREBINDGEN_OUT_DIR;
    pb.read_json(src_dir, "structs");
    pb.read_json(src_dir, "functions");
    pb.make_rs(src_dir, "structs", "ffi_common");
    pb.make_rs(src_dir, "functions", "ffi_common");
}
