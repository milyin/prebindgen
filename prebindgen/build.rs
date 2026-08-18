// Exists only so this crate's own doctests can run `#[prebindgen]`.
//
// The macro hard-requires `OUT_DIR`, and Cargo sets it for doctests only when
// the package has a build script. Without this, the crate-level and `Source`
// examples had to be ```ignore — never compiled, free to rot.
//
// This is *not* `init_prebindgen_out_dir` in miniature. That function also
// cleans the output directory, writes `prebindgen_output.toml`, and
// exports the crate's real feature list — all of which exist so a *downstream*
// crate can later read this one's captured surface through `Source`. Nothing
// reads prebindgen's own output, so this supplies only the two things macro
// expansion itself touches:
//
//   - the `prebindgen` subdirectory, which `#[prebindgen]` opens with
//     `create_new` to write its JSONL into;
//   - `PREBINDGEN_FEATURES`, which `features!()` reads. Empty is correct here:
//     the doctests assert nothing about its contents.
//
// It cannot delegate to `init_prebindgen_out_dir` in any case, because a crate
// cannot take itself as a build-dependency.
fn main() {
    let out_dir =
        std::env::var("OUT_DIR").expect("OUT_DIR is not set; this build script requires Cargo");
    let prebindgen_dir = std::path::PathBuf::from(out_dir).join("prebindgen");
    std::fs::create_dir_all(&prebindgen_dir).unwrap_or_else(|e| {
        panic!(
            "failed to create the prebindgen output directory {}: {e}",
            prebindgen_dir.display()
        )
    });
    println!("cargo:rustc-env=PREBINDGEN_FEATURES=");
}
