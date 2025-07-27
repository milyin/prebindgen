use std::path::Path;
use std::{env, fs};

use crate::SourceLocation;

/// Wrapper over syn::File for generating Rust source code
pub struct Destination {
    file: syn::File,
}

impl FromIterator<syn::Item> for Destination {
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
    /// Write the Rust file to the specified path
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
