use std::{collections::BTreeSet, env, fs, path::Path};

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

    let features = get_enabled_features();

    // Save features list to features.txt (one per line)
    let features_path = prebindgen_dir.join(FEATURES_FILE);
    let mut feature_contents = String::new();
    for feature in &features {
        feature_contents += feature;
        feature_contents.push('\n');
    }
    fs::write(&features_path, feature_contents).unwrap_or_else(|e| {
        panic!(
            "Failed to write features to {}: {}",
            features_path.display(),
            e
        );
    });

    // Export features list to the main crate as an env variable
    // Accessible via env!("PREBINDGEN_FEATURES") or std::env::var at compile time/runtime
    // Make the list of format "crate_name/f1 crate_name/f2"
    println!(
        "cargo:rustc-env=PREBINDGEN_FEATURES={}",
        features
            .into_iter()
            .map(|f| format!("{}/{}", crate_name, f))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

/// Read all feature names declared in the current crate's `Cargo.toml`.
///
/// Notes:
/// - Excludes the special `default` feature from the returned set.
/// - Panics if called outside of build.rs (requires OUT_DIR and CARGO_MANIFEST_DIR).
pub fn get_all_features() -> BTreeSet<String> {
    // Ensure we are in build.rs context
    env::var("OUT_DIR").expect(
        "OUT_DIR environment variable not set. This function should be called from build.rs.",
    );
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect(
        "CARGO_MANIFEST_DIR environment variable not set. This function should be called from build.rs.",
    );
    let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");

    let manifest_content = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read Cargo.toml at {}: {}",
            manifest_path.display(),
            e
        )
    });

    let doc: toml::Value = toml::from_str(&manifest_content).unwrap_or_else(|e| {
        panic!(
            "Failed to parse Cargo.toml at {} as TOML: {}",
            manifest_path.display(),
            e
        )
    });

    let mut set = BTreeSet::new();
    if let Some(features_tbl) = doc.get("features").and_then(|v| v.as_table()) {
        for key in features_tbl.keys() {
            if key != "default" {
                set.insert(key.to_string());
            }
        }
    }
    set
}

/// Check whether a feature is enabled by looking at the corresponding
/// `CARGO_FEATURE_<NAME>` environment variable provided to build scripts by Cargo.
/// Hyphens in feature names are converted to underscores to match Cargo's env var format.
pub fn is_feature_enabled(feature: &str) -> bool {
    // Ensure we are in build.rs context
    env::var("OUT_DIR").expect(
        "OUT_DIR environment variable not set. This function should be called from build.rs.",
    );
    let env_key = format!(
        "CARGO_FEATURE_{}",
        feature
            .to_ascii_uppercase()
            .chars()
            .map(|c| if c == '-' { '_' } else { c })
            .collect::<String>()
    );
    env::var_os(env_key).is_some()
}

/// Filter the full features list to only those that are currently enabled.
pub fn get_enabled_features() -> BTreeSet<String> {
    get_all_features()
        .into_iter()
        .filter(|f| is_feature_enabled(f))
        .collect()
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
        println!("cargo:warning=[{}:{}] {}",
            file!(),
            line!(),
            format!($($arg)*)
        );
    };
}
