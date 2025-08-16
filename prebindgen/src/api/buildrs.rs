use std::{env, fs};

use crate::{CRATE_NAME_FILE, FEATURES_FILE};

/// Initialize the prebindgen output directory for the current crate
///
/// This function must be called in the `build.rs` file of any crate that uses
/// the `#[prebindgen]` attribute macro.
///
/// # Panics
///
/// Panics if:
/// - `CARGO_PKG_NAME` environment variable is not set
/// - `OUT_DIR` environment variable is not set  
/// - Directory creation or file operations fail
///
/// # Example
///
/// ```rust,ignore
/// // build.rs
/// fn main() {
///     prebindgen::init_prebindgen_out_dir();
/// }
/// ```
pub fn init_prebindgen_out_dir() {
    env::var("OUT_DIR").expect(
        "OUT_DIR environment variable not set. This function should be called from build.rs.",
    );
    // Get the crate name from CARGO_PKG_NAME or use fallback
    // For doctests, use "source_ffi" even if CARGO_PKG_NAME is set to "prebindgen"
    let crate_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME environment variable not set. This function should be called from build.rs.");

    // delete all files in the prebindgen directory
    let prebindgen_dir = get_prebindgen_out_dir();
    if prebindgen_dir.exists() {
        for entry in fs::read_dir(&prebindgen_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path).unwrap_or_else(|e| {
                    panic!("Failed to delete {}: {}", path.display(), e);
                });
            }
        }
    } else {
        fs::create_dir_all(&prebindgen_dir).unwrap_or_else(|e| {
            panic!(
                "Failed to create prebindgen directory {}: {}",
                prebindgen_dir.display(),
                e
            );
        });
    }

    // Store the crate name in a separate file
    let crate_name_path = prebindgen_dir.join(CRATE_NAME_FILE);
    fs::write(&crate_name_path, &crate_name).unwrap_or_else(|e| {
        panic!(
            "Failed to write crate name to {}: {}",
            crate_name_path.display(),
            e
        );
    });

    // Collect enabled Cargo features from environment and store them
    // Cargo exposes enabled features to build.rs as env vars CARGO_FEATURE_<NAME>
    // where <NAME> is uppercased and '-' replaced with '_'. Here we convert back.
    let mut features: Vec<String> = std::env::vars()
        .filter_map(|(k, _)| {
            k.strip_prefix("CARGO_FEATURE_")
                .map(|name| name.to_string())
        })
        .map(|name| name.to_lowercase().replace('_', "-"))
        .collect();
    features.sort();
    features.dedup();

    // Save features list to features.txt (one per line)
    let features_path = prebindgen_dir.join(FEATURES_FILE);
    let features_contents = if features.is_empty() {
        String::new()
    } else {
        let mut s = features.join("\n");
        s.push('\n');
        s
    };
    fs::write(&features_path, features_contents).unwrap_or_else(|e| {
        panic!(
            "Failed to write features to {}: {}",
            features_path.display(),
            e
        );
    });

    // Export features list to the main crate as an env variable
    // Accessible via env!("PREBINDGEN_FEATURES") or std::env::var at compile time/runtime
    println!("cargo:rustc-env=PREBINDGEN_FEATURES={}", features.join(","));
}

/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";

/// Get the full path to the prebindgen output directory in OUT_DIR.
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Check if build.rs for the crate exitsts");
    std::path::Path::new(&out_dir).join(PREBINDGEN_DIR)
}

/// Macro for debug tracing in build.rs. Used by prebindgen-proc-macro to display paths to
/// generated files, but can be also used in other contexts.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("cargo:warning=[{}:{}] {}",
            file!(),
            line!(),
            format!($($arg)*)
        );
    };
}
