fn main() {
    prebindgen::init_prebindgen_out_dir();

    // In the complex cases involving code generation, the FFI crate may need
    // access to the actual Cargo.lock file. Demonstrate how to obtain it.
    let cargo_lock_path = prebindgen_project_root::get_project_root()
        .unwrap_or_else(|e| panic!("Failed to determine workspace root: {}", e))
        .join("Cargo.lock");
    prebindgen::trace!("project's Cargo.lock is {}", cargo_lock_path.display());
}
