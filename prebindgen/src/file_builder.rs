use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use roxygen::roxygen;

use crate::{Prebindgen, trace};

/// Builder for writing groups to files with append capability.
///
/// This builder is returned by `Prebindgen::group()` and `Prebindgen::all()` methods
/// and allows you to select multiple groups and write them to a single output file.
///
/// # Example
///
/// ```rust,ignore
/// // Write multiple groups to one file
/// let combined = prebindgen
///     .group("structs")
///     .group("enums")
///     .group("functions")
///     .write_to_file("combined_ffi.rs");
/// ```
pub struct FileBuilder<'a> {
    pub(crate) prebindgen: &'a Prebindgen,
    pub(crate) groups: Vec<String>,
}

impl<'a> FileBuilder<'a> {
    /// Add another group to the selection
    ///
    /// This allows you to combine multiple groups into a single output file.
    ///
    /// # Returns
    ///
    /// The same `FileBuilder` with the additional group added.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let combined = prebindgen
    ///     .group("structs")
    ///     .group("enums")
    ///     .group("functions")
    ///     .write_to_file("combined.rs");
    /// ```
    #[roxygen]
    pub fn group<S: Into<String>>(
        mut self,
        /// Name of the additional group to include
        group_name: S,
    ) -> Self {
        self.groups.push(group_name.into());
        self
    }

    /// Write the selected groups to a file
    ///
    /// Generates the Rust source code for all selected groups and writes it
    /// to the specified file. For functions, this generates `#[no_mangle] extern "C"`
    /// wrapper functions that call the original functions from the source crate.
    /// For types (structs, enums, unions), this copies the original definitions.
    ///
    /// If the file path is relative, it will be created relative to `OUT_DIR`.
    ///
    /// # Returns
    ///
    /// The absolute path to the generated file.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `OUT_DIR` environment variable is not set
    /// - File creation fails
    /// - Writing to the file fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let output_file = prebindgen.all().write_to_file("ffi_bindings.rs");
    /// println!("Generated FFI bindings at: {}", output_file.display());
    /// ```
    #[roxygen]
    pub fn write_to_file<P: AsRef<Path>>(
        self,
        /// Path where the generated code should be written
        file_name: P,
    ) -> std::path::PathBuf {
        // Prepend with OUT_DIR if file_name is relative
        let file_name = if file_name.as_ref().is_relative() {
            let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
            PathBuf::from(out_dir).join(file_name)
        } else {
            file_name.as_ref().to_path_buf()
        };
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
        let dest_path = PathBuf::from(&out_dir).join(&file_name);
        let mut dest = fs::File::create(&dest_path).unwrap_or_else(|e| {
            panic!("Failed to create {}: {}", dest_path.display(), e);
        });

        // Collect type replacements and write records in a single pass
        let mut all_type_replacements = HashSet::new();
        for group in &self.groups {
            // Collect type replacements from this group
            self.prebindgen
                .collect_type_replacements(group, &mut all_type_replacements);

            // Write the records for this group (without assertions)
            self.prebindgen
                .write_internal(&mut dest, group)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to write records for group {} to {}: {}",
                        group,
                        dest_path.display(),
                        e
                    )
                });
        }

        // Generate and append type equivalence assertions once at the end
        let assertions_file = crate::codegen::generate_type_assertions(&all_type_replacements);
        writeln!(dest, "{}", prettyplease::unparse(&assertions_file)).unwrap_or_else(|e| {
            panic!(
                "Failed to write type assertions to {}: {}",
                dest_path.display(),
                e
            );
        });
        dest.flush().unwrap_or_else(|e| {
            panic!("Failed to flush file {}: {}", dest_path.display(), e);
        });

        trace!(
            "Generated bindings for groups [{}] written to: {}",
            self.groups.join(", "),
            dest_path.display()
        );
        dest_path
    }
}
