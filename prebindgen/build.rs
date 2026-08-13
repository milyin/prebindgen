// Exists only so this crate's own doctests can run `#[prebindgen]`.
//
// The macro hard-requires `OUT_DIR`, and Cargo sets it for doctests only when
// the package has a build script. Without this, the crate-level and `Source`
// examples had to be ```ignore — never compiled, free to rot.
//
// This cannot delegate to `init_prebindgen_out_dir` because a crate cannot take
// itself as a build-dependency. Keep this bootstrap deliberately small, but
// initialize the same files macro expansion requires:
//
//   - `prebindgen/{crate_name,features}.txt` for capture recovery;
//   - both versioned state slots tracked by rustc;
//   - `PREBINDGEN_FEATURES`, empty because these doctests do not assert it.
fn main() {
    let out_dir = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").expect("OUT_DIR is not set; this build script requires Cargo"),
    );
    let prebindgen_dir = out_dir.join("prebindgen");
    std::fs::create_dir_all(&prebindgen_dir).unwrap_or_else(|error| {
        panic!(
            "failed to create the prebindgen output directory {}: {error}",
            prebindgen_dir.display()
        )
    });

    let crate_name = std::env::var("CARGO_PKG_NAME")
        .expect("CARGO_PKG_NAME is not set; this build script requires Cargo");
    std::fs::write(prebindgen_dir.join("crate_name.txt"), &crate_name)
        .expect("failed to write doctest crate_name.txt");
    std::fs::write(prebindgen_dir.join("features.txt"), [])
        .expect("failed to write doctest features.txt");

    let state = format!(
        "{{\"protocol\":\"prebindgen-capture-v1\",\"generation\":0,\"crate_name\":\"{crate_name}\",\"features\":[]}}\n"
    );
    for slot in [
        ".prebindgen-capture-state-v1-a.json",
        ".prebindgen-capture-state-v1-b.json",
    ] {
        std::fs::write(out_dir.join(slot), &state)
            .expect("failed to write doctest prebindgen capture state");
    }
    println!("cargo:rustc-env=PREBINDGEN_FEATURES=");
}
