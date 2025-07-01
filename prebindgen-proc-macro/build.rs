fn main() {
    // This build script just ensures that OUT_DIR is available for tests
    println!("cargo:rerun-if-changed=build.rs");
}
