use std::path::{Path, PathBuf};

use prebindgen_flat::{flat::Guard, Emit, Flat};
use proc_macro2::TokenStream;

/// The generated Rust file: items in the order they were pushed.
#[derive(Default)]
pub struct RustFile {
    items: Vec<TokenStream>,
}

impl RustFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an item.
    pub fn push(&mut self, item: TokenStream) {
        self.items.push(item);
    }

    /// Re-emit the source crates' feature guards (the `const _` items
    /// [`prebindgen::Source`] injects), so the generated crate fails to build
    /// against a source compiled with different features.
    pub fn guards(&mut self, flat: &Flat) {
        let emit = Emit::new();
        for g in flat.guards() {
            let g: &Guard = g;
            let item = emit.guard(g);
            self.items.push(quote::quote!(#item));
        }
    }

    /// The file's text, formatted.
    pub fn render(&self) -> String {
        let tokens: TokenStream = self.items.iter().cloned().collect();
        match syn::parse2::<syn::File>(tokens.clone()) {
            Ok(file) => prettyplease::unparse(&file),
            // Unformatted output still compiles (or reports the real error
            // at the right place), which beats hiding it behind a panic here.
            Err(_) => tokens.to_string(),
        }
    }

    /// Write the file to `path` (relative paths land in `OUT_DIR`), touching
    /// it only when the content changed so an `include!` does not trigger a
    /// rebuild loop. Returns the absolute path written.
    pub fn write(&self, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        let path = resolve_out_path(path.as_ref());
        write_if_changed(&path, &self.render())?;
        Ok(path)
    }
}

/// `path` as is when absolute, else under `OUT_DIR` (or the current
/// directory outside a build script).
pub fn resolve_out_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match std::env::var_os("OUT_DIR") {
        Some(dir) => Path::new(&dir).join(path),
        None => path.to_path_buf(),
    }
}

/// Write `contents` to `path` unless it already holds exactly that, creating
/// parent directories as needed.
pub fn write_if_changed(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    std::fs::write(path, contents)
}
