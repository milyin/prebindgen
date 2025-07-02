fn main() {
    prebindgen::prebindgen_json_to_rs(ffi_common::OUT_DIR, "ffi_common.rs", "ffi_common");
}
