fn main() {
    prebindgen::prebindgen_json_to_rs(ffi_common::OUT_DIR, Some("structs"), "ffi_common_structs.rs", "ffi_common");
    prebindgen::prebindgen_json_to_rs(ffi_common::OUT_DIR, Some("functions"), "ffi_common_functions.rs", "ffi_common");
}
