// Exists only so this crate's own doctests can run `#[prebindgen]`.
//
// The macro hard-requires `OUT_DIR` (and a `prebindgen` subdirectory to write
// its JSONL into), and Cargo sets `OUT_DIR` for doctests only when the package
// has a build script. Without this, the crate-level and `Source` examples had to
// be ```ignore — never compiled, free to rot.
//
// It duplicates the three lines of `init_prebindgen_out_dir` rather than calling
// it, because a crate cannot take itself as a build-dependency.
fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("prebindgen");
    std::fs::create_dir_all(&out).unwrap();
    println!("cargo:rustc-env=PREBINDGEN_FEATURES=");
}
