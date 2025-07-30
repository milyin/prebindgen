use std::{env, fs};

use crate::CRATE_NAME_FILE;

/// Initialize the prebindgen output directory for the current crate
///
/// This function must be called in the `build.rs` file of any crate that uses
/// the `#[prebindgen]` attribute macro. It performs the following operations:
///
/// 1. Creates the prebindgen output directory in `OUT_DIR`
/// 2. Clears any existing files from the directory
/// 3. Stores the current crate's name for later reference
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
    env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    init_prebindgen_out_dir_internal();
}

/// Internal implementation of prebindgen output directory initialization
///
/// This function performs the actual initialization work and can be called
/// even when OUT_DIR is not set (e.g., during doctests).
#[doc(hidden)]
pub fn init_prebindgen_out_dir_internal() {
    // Get the crate name from CARGO_PKG_NAME or use fallback
    // For doctests, use "source_ffi" even if CARGO_PKG_NAME is set to "prebindgen"
    let crate_name = env::var("CARGO_PKG_NAME")
        .map(|name| if name == "prebindgen" { "source_ffi".to_string() } else { name })
        .unwrap_or_else(|_| "source_ffi".to_string());

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
}

/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";

/// Get the full path to the prebindgen output directory in OUT_DIR.
///
/// **Internal API**: This function is public only for interaction with the proc-macro crate.
/// User code should use the `prebindgen_out_dir!()` macro instead.
#[doc(hidden)]
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().join("prebindgen_fallback").to_string_lossy().to_string());
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
