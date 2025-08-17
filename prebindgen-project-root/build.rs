use std::path::PathBuf;

fn main() {
    // If the crate is located under CARGO_HOME (i.e., used as a dependency outside the workspace),
    // we cannot infer the workspace root reliably. Panic with guidance.
    let cargo_home = if let Ok(ch) = std::env::var("CARGO_HOME") {
        PathBuf::from(ch)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cargo")
    } else {
        panic!("Unable to determine CARGO_HOME and HOME is not set");
    };

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));

    if manifest_dir.starts_with(&cargo_home) {
        let error = format!(
            "The crate `prebindgen-project-root` located at\n\
            {}\n\
            is being used as a Cargo dependency from CARGO_HOME.\n\
            Since it is not located inside your workspace, it cannot determine the path to the workspace root.\n\
            Please add `prebindgen-project-root` as a member of your workspace and patch dependencies to use it locally.\n\n\
            This can be done using the helper tool:\n\n\
            cargo install prebindgen-project-root\n\
            cargo prebindgen-project-root install <path>\n\n\
            where `<path>` is the path to your workspace root.\n",
            manifest_dir.display()
        );
        println!("cargo:rustc-env=PROJECT_ROOT_ERROR={}", quote::quote!(error));
        println!("cargo:rustc-env=PROJECT_ROOT=");
        println!("cargo:warning={}", error);
    } else {
        let workspace_root = project_root::get_project_root()
            .unwrap_or_else(|e| panic!("Failed to determine workspace root: {}", e));
        println!("cargo:rustc-env=PROJECT_ROOT={}", workspace_root.display());
        println!("cargo:rustc-env=PROJECT_ROOT_ERROR=");
    }
}
