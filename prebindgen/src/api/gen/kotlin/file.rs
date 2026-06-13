//! Model-level file merging and on-disk layout.
//!
//! Emitters produce per-declaration [`KtFile`] fragments; [`merge_files`]
//! groups them so every Java/Kotlin package collapses to ONE file, written
//! at the FLATTENED path `<root>/<package as dirs>.kt` (`io.zenoh.jni.bytes`
//! → `io/zenoh/jni/bytes.kt`) — the file is named after the package's last
//! segment and lives in its parent package's directory. Kotlin imposes no
//! file-location/`package` correspondence and a file `bytes.kt` never
//! clashes with a sibling `bytes/` directory, so the layout is
//! collision-free.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::model::KtFile;

/// Errors surfaced by Kotlin emission.
#[derive(Debug)]
pub enum WriteKotlinError {
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for WriteKotlinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteKotlinError::Io(e) => write!(f, "I/O error writing Kotlin file: {}", e),
            WriteKotlinError::Other(s) => write!(f, "Kotlin emission error: {}", s),
        }
    }
}

impl std::error::Error for WriteKotlinError {}

impl From<std::io::Error> for WriteKotlinError {
    fn from(e: std::io::Error) -> Self {
        WriteKotlinError::Io(e)
    }
}

/// Merge fragments into one [`KtFile`] per package (sorted package order;
/// within a package, fragments keep their emission order). Fails on a
/// duplicate top-level declaration name within a package — a single merged
/// file can't hold two declarations of the same identity.
pub fn merge_files(fragments: Vec<KtFile>) -> Result<Vec<KtFile>, WriteKotlinError> {
    let mut groups: BTreeMap<String, KtFile> = BTreeMap::new();
    let mut seen: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for frag in fragments {
        let names = seen.entry(frag.package.clone()).or_default();
        for d in &frag.decls {
            // Functions may overload (same name, different signature) and Raw
            // blocks are opaque; only class-like identities must be unique
            // within a package's single merged file.
            let unique_required = matches!(
                d,
                super::model::KtDecl::Class(_)
                    | super::model::KtDecl::TypeAlias { .. }
                    | super::model::KtDecl::Property(_)
            );
            if unique_required && !d.name().is_empty() && !names.insert(d.name().to_string()) {
                return Err(WriteKotlinError::Other(format!(
                    "duplicate declaration `{}` in package `{}`",
                    d.name(),
                    frag.package
                )));
            }
        }
        let merged = groups
            .entry(frag.package.clone())
            .or_insert_with(|| KtFile::new(frag.package.clone()));
        merged.decls.extend(frag.decls);
        merged.extra_imports.extend(frag.extra_imports);
    }
    // Extra imports carry pre-shortened raw text references, so a simple-name
    // collision between two distinct FQNs cannot be repaired by qualifying a
    // use site — reject it (parity with the previous string-merge behavior).
    // Exception: lowercase simple names are top-level FUNCTION imports, which
    // Kotlin allows to overload across packages (resolution by signature/
    // receiver) — e.g. several `asRaw` extension adapters in one file.
    for (package, file) in &groups {
        let mut by_simple: BTreeMap<&str, &str> = BTreeMap::new();
        for imp in &file.extra_imports {
            let simple = imp.rsplit_once('.').map(|(_, s)| s).unwrap_or(imp.as_str());
            if simple.chars().next().is_some_and(|c| c.is_lowercase()) {
                continue;
            }
            if let Some(prev) = by_simple.insert(simple, imp.as_str()) {
                if prev != imp.as_str() {
                    return Err(WriteKotlinError::Other(format!(
                        "import simple-name collision in package `{}`: `{}` and `{}`",
                        package, prev, imp
                    )));
                }
            }
        }
    }
    Ok(groups.into_values().collect())
}

/// The flattened on-disk path of one merged file under `kotlin_root`.
/// `fallback_name` names the file when the package is empty.
pub fn merged_file_path(kotlin_root: &Path, file: &KtFile, fallback_name: &str) -> PathBuf {
    if file.package.is_empty() {
        kotlin_root.join(format!("{fallback_name}.kt"))
    } else {
        kotlin_root.join(format!("{}.kt", file.package.replace('.', "/")))
    }
}

/// Render and write every merged file; returns the written paths.
pub fn write_files(files: &[KtFile], kotlin_root: &Path) -> Result<Vec<PathBuf>, WriteKotlinError> {
    let mut written = Vec::new();
    for f in files {
        let fallback = f
            .decls
            .first()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|| "Generated".to_string());
        let path = merged_file_path(kotlin_root, f, &fallback);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, f.render())?;
        written.push(path);
    }
    Ok(written)
}
