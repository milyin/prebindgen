fn main() {
    prebindgen::prebindgen_json_to_rs(ffi_common::PREBINDGEN_JSON, "ffi_common.rs");
}
