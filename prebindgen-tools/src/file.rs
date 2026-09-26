//! The generated Rust file.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use prebindgen_flat::{flat::Guard, Emit, Flat};
use proc_macro2::TokenStream;

/// An ordered list of generated items, with keyed helpers emitted once.
///
/// Adapters push their exported items in declaration order and ask for
/// shared helpers — one converter per type, one mirror per struct — through
/// [`Self::once`], which is safe to call recursively: the key is claimed
/// before the helper is built, so a helper that (directly or through another)
/// needs itself just refers to it by name.
#[derive(Default)]
pub struct RustFile {
    items: Vec<TokenStream>,
    claimed: HashSet<String>,
}

impl RustFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an item.
    pub fn push(&mut self, item: TokenStream) {
        self.items.push(item);
    }

    /// Whether a helper under `key` has been claimed.
    pub fn has(&self, key: &str) -> bool {
        self.claimed.contains(key)
    }

    /// Claim `key`: `true` the first time, `false` after. For an adapter
    /// whose helper builder needs more than the file — build the item after
    /// a successful claim and [`push`](Self::push) it.
    pub fn claim(&mut self, key: impl Into<String>) -> bool {
        self.claimed.insert(key.into())
    }

    /// Emit the helper under `key` unless it was already claimed. `build`
    /// runs at most once per key and may itself call `once` for the helpers
    /// it depends on; those land in the file before this one.
    pub fn once<E>(
        &mut self,
        key: impl Into<String>,
        build: impl FnOnce(&mut Self) -> Result<TokenStream, E>,
    ) -> Result<(), E> {
        let key = key.into();
        if !self.claimed.insert(key) {
            return Ok(());
        }
        let item = build(self)?;
        self.items.push(item);
        Ok(())
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
