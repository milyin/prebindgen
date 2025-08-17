use std::path::PathBuf;

/// Returns the absolute path to the cargo workspace root that built this crate.
///
/// This works only if this crate is included in a Cargo workspace. In the opposite case,
/// it will return an error, explaining how to correctly configure the crate.
pub fn get_project_root() -> Result<PathBuf, &'static str> {
    let project_root = env!("PROJECT_ROOT");
    if project_root.is_empty() {
        Err(env!("PROJECT_ROOT_ERROR"))
    } else {
        Ok(project_root.into())
    }
}
