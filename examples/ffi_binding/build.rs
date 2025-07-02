use prebindgen::Prebindgen;

fn main() {
    let src_dir = ffi_common::PREBINDGEN_OUT_DIR;
    let mut pb = Prebindgen::new(src_dir, "ffi_common".to_string());
    pb.read("structs");
    pb.read("functions");
    pb.write("structs", "ffi_common_structs.rs");
    pb.write("functions", "ffi_common_functions.rs");
}
