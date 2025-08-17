/// Returns the absolute path to the cargo workspace root that built this crate.
///
/// This value is determined at build time in `build.rs` and injected via the
/// `PROJECT_ROOT` environment variable using `cargo:rustc-env`.
pub fn get_project_root() -> &'static str {
    env!("PROJECT_ROOT")
}
