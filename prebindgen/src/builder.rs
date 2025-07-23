use roxygen::roxygen;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::{JSONL_EXTENSION, record::*};
use crate::{jsonl, trace};

/// Builder for configuring Prebindgen with optional parameters.
///
/// This builder allows you to configure how prebindgen reads and processes
/// the exported FFI definitions before building the final `Prebindgen` instance.
///
/// # Example
///
/// ```rust,ignore
/// let prebindgen = prebindgen::Builder::new("/path/to/prebindgen/data")
///     .crate_name("my_custom_crate")
///     .edition("2024")
///     .allowed_prefix("libc")
///     .allowed_prefix("core")
///     .strip_transparent_wrapper("std::mem::MaybeUninit")
///     .select_group("structs")
///     .select_group("functions")
///     .disable_feature("experimental")
///     .enable_feature("std")
///     .match_feature("unstable", "unstable")
///     .build();
/// ```
pub struct Builder {
    input_dir: std::path::PathBuf,
    source_crate_name: String, // Empty string by default, read from file if empty
    selected_groups: HashSet<String>,
}

impl Builder {
    /// Create a new builder with the specified input directory
    ///
    /// The input directory should contain the prebindgen data files generated
    /// by the common FFI crate. This is typically obtained from the
    /// `PREBINDGEN_OUT_DIR` constant exported by the common FFI crate.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(common_ffi::PREBINDGEN_OUT_DIR);
    /// ```
    #[roxygen]
    pub fn new<P: AsRef<Path>>(
        /// Path to the directory containing prebindgen data files
        input_dir: P,
    ) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            source_crate_name: String::new(), // Empty string by default, read from file if empty
            selected_groups: HashSet::new(),
        }
    }

    /// Internal method to read all exported files matching the group name pattern `<group>_*`
    fn read_group_internal(&self, group: &str) -> Vec<Record> {
        let pattern = format!("{group}_");
        let mut record_map = HashMap::new();

        // Read the directory and find all matching files
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&pattern) && file_name.ends_with(JSONL_EXTENSION) {
                        trace!("Reading exported file: {}", path.display());
                        let path_clone = path.clone();

                        match jsonl::read_jsonl_file(&path) {
                            Ok(records) => {
                                for record in records {
                                    // Use HashMap to deduplicate records by name
                                    record_map.insert(record.name.clone(), record);
                                }
                            }
                            Err(e) => {
                                panic!("Failed to read {}: {}", path_clone.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Return deduplicated records for this group
        record_map.into_values().collect::<Vec<_>>()
    }

    /// Internal method to discover all available groups from the directory
    fn discover_generated_groups(&self) -> HashSet<String> {
        let mut groups = HashSet::new();

        // Discover all available groups
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(JSONL_EXTENSION) {
                        // Extract group name from filename (everything before the first underscore)
                        if let Some(underscore_pos) = file_name.find('_') {
                            let group_name = &file_name[..underscore_pos];
                            groups.insert(group_name.to_string());
                        }
                    }
                }
            }
        }

        groups
    }

    /// Override the source crate name used in generated extern "C" functions
    ///
    /// By default, the crate name is read from the prebindgen data files.
    /// This method allows you to override it, which can be useful when
    /// the crate has been renamed or when you want to use a different
    /// module path in the generated calls.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .crate_name("my_renamed_crate");
    /// ```
    #[roxygen]
    pub fn crate_name<S: Into<String>>(
        mut self,
        /// The crate name to use in generated function calls
        crate_name: S,
    ) -> Self {
        self.source_crate_name = crate_name.into();
        self
    }

    /// Select a specific group to include in the final Prebindgen instance
    ///
    /// This method can be called multiple times to select multiple groups.
    /// If no groups are selected, all available groups will be included.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .select_group("structs")
    ///     .select_group("core_functions");
    /// ```
    #[roxygen]
    pub fn group<S: Into<String>>(
        mut self,
        /// Name of the group to include
        group_name: S,
    ) -> Self {
        self.selected_groups.insert(group_name.into());
        self
    }

    /// Build the configured Prebindgen instance.
    ///
    /// This method reads the prebindgen data files from the input directory
    /// and creates a `Prebindgen` instance ready for generating FFI bindings.
    ///
    /// # Panics
    ///
    /// Panics if the input directory was not properly initialized with
    /// `init_prebindgen_out_dir()` in the source crate's build.rs.
    ///
    /// # Returns
    ///
    /// A configured `Prebindgen` instance ready for use.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let prebindgen = prebindgen::Builder::new(path)
    ///     .build();
    /// ```
    pub fn build(self) -> crate::Prebindgen {
        // Determine the crate name: use provided one, or read from stored file, or panic if not initialized
        let original_crate_name = crate::read_stored_crate_name(&self.input_dir).unwrap_or_else(|| {
            panic!(
                "The directory {} was not initialized with init_prebindgen_out_dir(). \
                Please ensure that init_prebindgen_out_dir() is called in the build.rs of the source crate.",
                self.input_dir.display()
            )
        });
        let source_crate_name = if self.source_crate_name.is_empty() {
            original_crate_name
        } else {
            self.source_crate_name.clone()
        };

        // Read the groups based on selection
        let groups = if self.selected_groups.is_empty() {
            self.discover_generated_groups()
        } else {
            self.selected_groups.clone()
        };

        let raw_records_map: HashMap<String, Vec<Record>> = groups
            .into_iter()
            .map(|group| {
                let records = self.read_group_internal(&group);
                (group, records)
            })
            .collect();

        // Parse raw records
        let mut records = HashMap::new();
        for (group_name, raw_records) in raw_records_map {
            let processed_records = raw_records
                .into_iter()
                .map(|record| {
                    RecordSyn::try_from(record).unwrap_or_else(|e| {
                        panic!("Failed to parse record for group {group_name}: {e}")
                    })
                })
                .collect::<Vec<_>>();

            // Store the processed records for this group
            records.insert(group_name, processed_records);
        }

        crate::Prebindgen {
            source_crate_name,
            records,
        }
    }
}
