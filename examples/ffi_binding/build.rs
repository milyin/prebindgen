fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = format!("{}/ffi_common.rs", out_dir);
    std::fs::copy(ffi_common::GENERATED_PATH, &dest_path).expect(&format!(
        "Failed to copy ffi_common generated file from '{}' to '{}'",
        ffi_common::GENERATED_PATH,
        dest_path
    ));
}
