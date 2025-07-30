use std::path::Path;
use std::{env, fs};

use crate::SourceLocation;

/// A destination for collecting and writing generated Rust FFI bindings.
///
/// `Destination` serves as a collector that accumulates `syn::Item` objects (Rust AST items)
/// and provides functionality to write them as formatted Rust source code to a file.
/// It's the final step in the prebindgen pipeline where processed FFI items are materialized
/// into actual Rust code.
///
/// # Usage
///
/// `Destination` is typically used as the target of a `collect()` operation on an iterator
/// of processed FFI items:
///
/// ```rust
/// use prebindgen::collect::Destination;
///
/// // Collect processed items into a destination
/// let destination: Destination = source
///     .items_all()
///     .map(strip_derives.into_closure())
///     .filter_map(feature_filter.into_closure())
///     .batching(converter.into_closure())
///     .collect();
///
/// // Write the collected items to a file
/// let bindings_file = destination.write("ffi_bindings.rs");
/// ```
///
/// # File Writing
///
/// The [`write`](Self::write) method handles path resolution automatically:
/// - Relative paths are resolved relative to the `OUT_DIR` environment variable
/// - Absolute paths are used as-is
/// - The generated code is formatted using `prettyplease` for readability
///
/// # Integration with cbindgen
///
/// `Destination` is commonly used in conjunction with cbindgen for C header generation:
///
/// ```rust
/// // Generate Rust FFI bindings
/// let bindings_file = destination.write("ffi_bindings.rs");
///
/// // Use the generated file with cbindgen
/// cbindgen::Builder::new()
///     .with_crate(&crate_dir)
///     .with_src(&bindings_file)  // Pass the generated file to cbindgen
///     .generate()
///     .unwrap()
///     .write_to_file("bindings.h");
/// ```
pub struct Destination {
    file: syn::File,
}

impl FromIterator<syn::Item> for Destination {
    /// Creates a `Destination` from an iterator of `syn::Item` objects.
    ///
    /// This implementation allows `Destination` to be used as the target of
    /// `collect()` operations on iterators of Rust AST items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prebindgen::collect::Destination;
    ///
    /// let destination: Destination = items_iterator.collect();
    /// ```
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
    /// This implementation handles items that come with source location metadata.
    /// The source location information is discarded during collection, keeping only
    /// the `syn::Item` objects for code generation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prebindgen::collect::Destination;
    ///
    /// let destination: Destination = items_with_locations.collect();
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prebindgen::collect::Destination;
    ///
    /// // Write to a relative path (resolved to OUT_DIR/ffi_bindings.rs)
    /// let bindings_file = destination.write("ffi_bindings.rs");
    ///
    /// // Write to an absolute path
    /// let bindings_file = destination.write("/tmp/my_bindings.rs");
    ///
    /// // The returned path can be used with other tools
    /// cbindgen::Builder::new()
    ///     .with_src(&bindings_file)
    ///     .generate()
    ///     .unwrap();
    /// ```
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
