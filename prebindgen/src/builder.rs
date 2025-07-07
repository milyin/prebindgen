use roxygen::roxygen;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::record::ParseConfig;
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
    crate_name: String, // Empty string by default, read from file if empty
    edition: String,
    selected_groups: HashSet<String>,
    allowed_prefixes: Vec<syn::Path>,
    pub(crate) disabled_features: HashSet<String>,
    pub(crate) enabled_features: HashSet<String>,
    pub(crate) feature_mappings: HashMap<String, String>,
    pub(crate) transparent_wrappers: Vec<syn::Path>,
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
        // Generate comprehensive allowed prefixes including standard prelude
        let allowed_prefixes = crate::codegen::generate_standard_allowed_prefixes();

        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name: String::new(), // Empty string by default, read from file if empty
            edition: "2024".to_string(), // Default edition
            selected_groups: HashSet::new(),
            allowed_prefixes,
            disabled_features: HashSet::new(),
            enabled_features: HashSet::new(),
            feature_mappings: HashMap::new(),
            transparent_wrappers: Vec::new(),
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
        self.crate_name = crate_name.into();
        self
    }

    /// Set the Rust edition to use for generated code
    ///
    /// This affects how the `#[no_mangle]` attribute is generated:
    /// - For edition "2024": `#[unsafe(no_mangle)]`
    /// - For other editions: `#[no_mangle]`
    ///
    /// Default is "2024" if not specified.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .edition("2021");
    /// ```
    #[roxygen]
    pub fn edition<E: Into<String>>(
        mut self,
        /// The Rust edition ("2021", "2024", etc.)
        edition: E,
    ) -> Self {
        self.edition = edition.into();
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
    pub fn select_group<S: Into<String>>(
        mut self,
        /// Name of the group to include
        group_name: S,
    ) -> Self {
        self.selected_groups.insert(group_name.into());
        self
    }

    /// Add an allowed type prefix for FFI validation
    ///
    /// This method allows you to specify additional type prefixes that should be
    /// considered valid for FFI functions, beyond the comprehensive set of default
    /// allowed prefixes that includes the standard prelude, core library types,
    /// primitive types, and common FFI types.
    ///
    /// # Default Allowed Prefixes
    ///
    /// The builder automatically includes prefixes for:
    /// - Standard library modules (`std`, `core`, `alloc`)
    /// - Standard prelude types (`Option`, `Result`, `Vec`, `String`, etc.)
    /// - Core library modules (`core::mem`, `core::ptr`, etc.)
    /// - Primitive types (`bool`, `i32`, `u64`, etc.)
    /// - Common FFI types (`libc`, `c_char`, `c_int`, etc.)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .allowed_prefix("libc")
    ///     .allowed_prefix("core");
    /// ```
    #[roxygen]
    pub fn allowed_prefix<S: Into<String>>(
        mut self,
        /// The additional type prefix to allow
        prefix: S,
    ) -> Self {
        let prefix_str = prefix.into();
        if let Ok(path) = syn::parse_str::<syn::Path>(&prefix_str) {
            self.allowed_prefixes.push(path);
        } else {
            panic!("Invalid path prefix: '{prefix_str}'");
        }
        self
    }

    /// Add a transparent wrapper type to be stripped from FFI function parameters
    ///
    /// Transparent wrappers are types that wrap other types but have the same
    /// memory layout (like `std::mem::MaybeUninit<T>`). When generating FFI stubs,
    /// these wrappers will be stripped from parameter types to create simpler
    /// C-compatible function signatures.
    ///
    /// For example, if you add `std::mem::MaybeUninit` as a transparent wrapper:
    /// - `&mut std::mem::MaybeUninit<Foo>` becomes `*mut Foo` in the FFI signature
    /// - `&std::mem::MaybeUninit<Bar>` becomes `*const Bar` in the FFI signature
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .strip_transparent_wrapper("std::mem::MaybeUninit")
    ///     .strip_transparent_wrapper("std::mem::ManuallyDrop");
    /// ```
    #[roxygen]
    pub fn strip_transparent_wrapper<S: Into<String>>(
        mut self,
        /// The transparent wrapper type to strip (e.g., "std::mem::MaybeUninit")
        wrapper_type: S,
    ) -> Self {
        let wrapper_str = wrapper_type.into();
        if let Ok(path) = syn::parse_str::<syn::Path>(&wrapper_str) {
            self.transparent_wrappers.push(path);
        } else {
            panic!("Invalid transparent wrapper type: '{wrapper_str}'");
        }
        self
    }

    /// Disable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by disabled features will be completely skipped in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .disable_feature("experimental")
    ///     .disable_feature("deprecated");
    /// ```
    #[roxygen]
    pub fn disable_feature<S: Into<String>>(
        mut self,
        /// The name of the feature to disable
        feature_name: S,
    ) -> Self {
        self.disabled_features.insert(feature_name.into());
        self
    }

    /// Enable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by enabled features will be included in the output with the
    /// `#[cfg(...)]` attribute removed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .enable_feature("experimental");
    /// ```
    #[roxygen]
    pub fn enable_feature<S: Into<String>>(
        mut self,
        /// The name of the feature to enable
        feature_name: S,
    ) -> Self {
        self.enabled_features.insert(feature_name.into());
        self
    }

    /// Map a feature name to a different name in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, features
    /// that match the mapping will have their names replaced with the target
    /// feature name in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .match_feature("unstable", "unstable")
    ///     .match_feature("internal", "unstable");
    /// ```
    #[roxygen]
    pub fn match_feature<S1: Into<String>, S2: Into<String>>(
        mut self,
        /// The original feature name to match
        from_feature: S1,
        /// The new feature name to use in the output
        to_feature: S2,
    ) -> Self {
        self.feature_mappings
            .insert(from_feature.into(), to_feature.into());
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
    ///     .edition("2021")
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
        let crate_name = if self.crate_name.is_empty() {
            original_crate_name
        } else {
            self.crate_name.clone()
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

        let mut exported_types = HashSet::new();
        // Update exported_types with type names from all groups
        for records in raw_records_map.values() {
            for record in records {
                if record.kind.is_type() {
                    exported_types.insert(record.name.clone());
                }
            }
        }

        // Process all raw records
        let mut records = HashMap::new();
        for (group_name, raw_records) in raw_records_map {
            let config = ParseConfig {
                crate_name: &crate_name,
                exported_types: &exported_types,
                disabled_features: &self.disabled_features,
                enabled_features: &self.enabled_features,
                feature_mappings: &self.feature_mappings,
                allowed_prefixes: &self.allowed_prefixes,
                transparent_wrappers: &self.transparent_wrappers,
                edition: &self.edition,
            };

            let processed_records: Result<Vec<RecordSyn>, String> = raw_records
                .into_iter()
                .map(|record| RecordSyn::from_record(record, &config))
                .collect();

            let processed_records = processed_records.unwrap_or_else(|e| {
                panic!("Failed to parse records for group {group_name}: {e}");
            });

            // Store the processed records for this group
            records.insert(group_name, processed_records);
        }

        crate::Prebindgen { records }
    }
}
