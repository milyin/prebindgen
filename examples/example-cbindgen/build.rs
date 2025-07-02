use prebindgen::Prebindgen;

fn main() {
    let src_dir = example_ffi::PREBINDGEN_OUT_DIR;
    let mut pb = Prebindgen::new(src_dir, "example_ffi".to_string());
    pb.read("structs");
    pb.read("functions");
    pb.write("structs", "example_ffi_structs.rs");
    pb.write("functions", "example_ffi_functions.rs");
}
