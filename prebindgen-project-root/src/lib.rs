/// Returns the absolute path to the cargo workspace root that built this crate.
///
/// This value is determined at build time in `build.rs` and injected via the
/// `PROJECT_ROOT` environment variable using `cargo:rustc-env`.
pub fn get_project_root() -> &'static str {
    env!("PROJECT_ROOT")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn project_root_is_absolute() {
        let p = get_project_root();
        assert!(Path::new(p).is_absolute(), "PROJECT_ROOT should be an absolute path: {}", p);
    }
}
