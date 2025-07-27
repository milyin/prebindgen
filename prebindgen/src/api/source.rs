use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use roxygen::roxygen;

use crate::{api::record::Record, utils::jsonl::read_jsonl_file, SourceLocation, CRATE_NAME_FILE};

/// File extension for data files
const JSONL_EXTENSION: &str = ".jsonl";
pub struct Source {
    crate_name: String,
    items: HashMap<String, Vec<(syn::Item, SourceLocation)>>,
}

impl Source {
    #[roxygen]
    pub fn new<P: AsRef<Path>>(
        /// Path to the directory containing prebindgen data files
        input_dir: P,
    ) -> Self {
        // Determine the crate name or panic if not initialized
        let input_dir = input_dir.as_ref().to_path_buf();
        if !input_dir.is_dir() {
            panic!(
                "Input directory {} does not exist or is not a directory",
                input_dir.display()
            );
        }
        let crate_name = read_stored_crate_name(&input_dir).unwrap_or_else(|| {
            panic!(
                "The directory {} was not initialized with init_prebindgen_out_dir(). \
                Please ensure that init_prebindgen_out_dir() is called in the build.rs of the source crate.",
                input_dir.display()
            )
        });

        let groups = Self::discover_groups(&input_dir);
        let mut items = HashMap::new();
        for group in groups {
            let records = Self::read_group(&input_dir, &group);
            let group_items = records.iter().map(|r| (r.parse())).collect::<Vec<_>>();
            items.insert(group, group_items);
        }

        Self { crate_name, items }
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn items_in_groups(
        &self,
        groups: &[&str],
    ) -> impl Iterator<Item = (syn::Item, SourceLocation)> {
        groups
            .iter()
            .filter_map(|group| self.items.get(*group))
            .flat_map(|records| records.iter()).cloned()
    }

    pub fn items_except_groups(
        &self,
        groups: &[&str],
    ) -> impl Iterator<Item = (syn::Item, SourceLocation)> {
        self.items
            .iter()
            .filter(|(group, _)| !groups.contains(&group.as_str()))
            .flat_map(|(_, records)| records.iter()).cloned()
    }

    pub fn items_all(&self) -> impl Iterator<Item = (syn::Item, SourceLocation)> {
        self.items
            .iter()
            .flat_map(|(_, records)| records.iter()).cloned()
    }

    /// Internal method to read all exported files matching the group name pattern `<group>_*`
    fn read_group<P: AsRef<Path>>(input_dir: P, group: &str) -> Vec<Record> {
        let pattern = format!("{group}_");
        let mut record_map = HashMap::new();

        // Read the directory and find all matching files
        if let Ok(entries) = fs::read_dir(&input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&pattern) && file_name.ends_with(JSONL_EXTENSION) {
                        #[cfg(feature = "debug")]
                        println!("Reading exported file: {}", path.display());
                        let path_clone = path.clone();

                        match read_jsonl_file(&path) {
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
    fn discover_groups<P: AsRef<Path>>(input_dir: P) -> HashSet<String> {
        let mut groups = HashSet::new();

        // Discover all available groups
        if let Ok(entries) = fs::read_dir(input_dir) {
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
}

/// Read the crate name from the stored file
fn read_stored_crate_name(input_dir: &Path) -> Option<String> {
    let crate_name_path = input_dir.join(CRATE_NAME_FILE);
    fs::read_to_string(&crate_name_path)
        .ok()
        .map(|s| s.trim().to_string())
}
