use prebindgen::Prebindgen;
use std::env;

fn main() {
    // Initialize Prebindgen and process JSON groups
    let mut pb = Prebindgen::new();
    // Read JSON output directory from ffi_common buildscript via Cargo metadata
    let src_dir = env::var("DEP_FFI_COMMON_JSON_OUT_DIR").expect("DEP_FFI_COMMON_JSON_OUT_DIR not set");
    pb.read_json(&src_dir, "structs");
    pb.read_json(&src_dir, "functions");
    pb.make_rs(&src_dir, "structs", "ffi_common");
    pb.make_rs(&src_dir, "functions", "ffi_common");
}
