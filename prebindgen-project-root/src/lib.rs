use std::path::PathBuf;

/// Error message shown when the crate is built as a regular (unpatched) dependency
/// and therefore cannot determine the destination workspace root.
const NOT_IN_WORKSPACE: &str =
    "The crate `prebindgen-project-root` is being used as a regular Cargo dependency.\n\
    Because it is not located within your workspace, it cannot determine the path to the workspace root.\n\
    Please add `prebindgen-project-root` as a member of your workspace and patch your dependencies to use the local path.\n\n\
    You can do this with the helper tool:\n\n\
    cargo install prebindgen-project-root\n\
    cargo prebindgen-project-root install <path>\n\n\
    where `<path>` is the path to your workspace root.\n\n\
    If the patch is already applied and the error persists, verify the version of the patched crate.";

/// Returns the absolute path to the cargo workspace root that built this crate.
///
/// This **only** works when the crate is a member of the destination workspace —
/// i.e. patched in via `cargo prebindgen-project-root install`. As a regular
/// (unpatched) dependency it cannot see the destination workspace, so it
/// **panics** rather than returning a wrong or empty path that a caller might
/// silently accept.
pub fn get_project_root() -> PathBuf {
    let project_root = env!("PROJECT_ROOT");
    if project_root.is_empty() {
        panic!("{NOT_IN_WORKSPACE}");
    }
    project_root.into()
}
