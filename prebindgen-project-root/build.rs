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
        panic!(
            "\nThe crate `prebindgen-project-root` located at\n\
             {}\n\
             is being used as a Cargo dependency from CARGO_HOME.\n\
             Since it is not located inside your workspace, it cannot determine the path to the workspace root.\n\
           Please add `prebindgen-project-root` as a member of your workspace and patch dependencies to use it locally.\n\n\
           This can be done using the helper tool: <instructions to be added>\n",
            manifest_dir.display()
        );
    }

    let workspace_root = project_root::get_project_root()
        .expect("Failed to detect workspace root using project-root");

    println!("cargo:rustc-env=PROJECT_ROOT={}", workspace_root.display());
}
