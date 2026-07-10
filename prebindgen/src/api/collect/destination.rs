use std::{env, fs, path::Path};

use crate::SourceLocation;

/// Internal collector that accumulates `syn::Item` objects (Rust AST items)
/// and writes them as `prettyplease`-formatted Rust source code to a file.
///
/// This is the final step of `Registry::write_rust`, where the generated FFI
/// items are materialized into the bindings file included by the consumer crate.
///
/// The [`write`](Self::write) method handles path resolution automatically:
/// - Relative paths are resolved relative to the `OUT_DIR` environment variable
/// - Absolute paths are used as-is
pub struct Destination {
    file: syn::File,
}

impl FromIterator<syn::Item> for Destination {
    /// Creates a `Destination` from an iterator of `syn::Item` objects.
    fn from_iter<T: IntoIterator<Item = syn::Item>>(iter: T) -> Self {
        Self {
            file: syn::File {
                shebang: None,
                attrs: vec![],
                items: iter.into_iter().collect(),
            },
        }
    }
}

impl FromIterator<(syn::Item, SourceLocation)> for Destination {
    /// Creates a `Destination` from an iterator of `(syn::Item, SourceLocation)` tuples.
    ///
    /// The source location information is discarded during collection, keeping only
    /// the `syn::Item` objects for code generation.
    fn from_iter<T: IntoIterator<Item = (syn::Item, SourceLocation)>>(iter: T) -> Self {
        Self {
            file: syn::File {
                shebang: None,
                attrs: vec![],
                items: iter.into_iter().map(|(item, _)| item).collect(),
            },
        }
    }
}

impl Destination {
    /// Writes the collected Rust items to a file and returns the absolute path.
    ///
    /// This method formats the collected `syn::Item` objects into valid Rust source code
    /// using `prettyplease` and writes it to the specified file path.
    ///
    /// # Path Resolution
    ///
    /// - **Relative paths**: Resolved relative to the `OUT_DIR` environment variable
    /// - **Absolute paths**: Used as-is without modification
    ///
    /// # Arguments
    ///
    /// * `filename` - The target file path (relative or absolute)
    ///
    /// # Returns
    ///
    /// The absolute path where the file was written.
    ///
    /// # Panics
    ///
    /// - If the `OUT_DIR` environment variable is not set (when using relative paths)
    /// - If the file cannot be written (e.g., permission denied, disk full)
    pub fn write<P: AsRef<Path>>(self, filename: P) -> std::path::PathBuf {
        let file_path = if filename.as_ref().is_relative() {
            let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
            std::path::PathBuf::from(out_dir).join(filename)
        } else {
            filename.as_ref().to_path_buf()
        };

        let content = prettyplease::unparse(&self.file);
        fs::write(&file_path, content).unwrap_or_else(|e| {
            panic!("Failed to write file {}: {}", file_path.display(), e);
        });

        file_path
    }
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", prettyplease::unparse(&self.file))
    }
}
