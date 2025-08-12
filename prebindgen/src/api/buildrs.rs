use std::{env, fs};

use crate::CRATE_NAME_FILE;

/// Initialize the prebindgen output directory for the current crate
///
/// This function must be called in the `build.rs` file of any crate that uses
/// the `#[prebindgen]` attribute macro. It performs the following operations:
///
/// 1. Creates the prebindgen output directory in `OUT_DIR` and initializes it
/// 2. Prints line "cargo:prebindgen=<path>" which provides path to prebindgen output directory
///    to build.rs of dependent crates via variable DEP_<crate_name>_PREBINDGEN
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
    println!("cargo:prebindgen={}", prebindgen_dir.display());
}

/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";

/// Get the full path to the prebindgen output directory in OUT_DIR.
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect(
        "OUT_DIR environment variable not set. Check if build.rs for the crate exitsts",
    );
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
