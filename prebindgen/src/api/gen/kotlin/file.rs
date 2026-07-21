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

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

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
            // within a package's single merged file. The "valid overloads"
            // assumption is now CHECKED upstream: jnigen's `validate_symbols`
            // (issue #89) rejects two functions with the same erased JVM
            // signature before any file is written, so a same-named function
            // reaching here is a genuine overload.
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

const OWNERSHIP_MARKER: &str = ".prebindgen-kotlin-output";
const OWNERSHIP_MARKER_CONTENT: &str = "prebindgen Kotlin output v1\n";

/// Render and write every merged file; returns the written paths.
///
/// A non-empty `kotlin_root` must contain the prebindgen ownership marker.
/// The initial write accepts a missing or empty directory and creates that
/// marker. Subsequent writes stage the complete output beside the root before
/// replacing the marked tree, so stale generated files are removed without
/// deleting caller-owned files or leaving an old tree half-deleted on failure.
///
/// The marker's content is matched ignoring surrounding whitespace and line
/// endings, so a committed marker checked out with CRLF (git `autocrlf` on
/// Windows) is still recognized.
pub fn write_files(files: &[KtFile], kotlin_root: &Path) -> Result<Vec<PathBuf>, WriteKotlinError> {
    let root_state = inspect_root(kotlin_root)?;
    let parent = kotlin_root.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = unique_sibling_path(kotlin_root, "staging");
    fs::create_dir(&staging)?;

    let result = write_staging(files, &staging).and_then(|relative_paths| {
        replace_root(kotlin_root, root_state, &staging)?;
        Ok(relative_paths
            .into_iter()
            .map(|path| kotlin_root.join(path))
            .collect())
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Clone, Copy)]
enum RootState {
    Missing,
    Empty,
    Owned,
}

fn inspect_root(kotlin_root: &Path) -> Result<RootState, WriteKotlinError> {
    let metadata = match fs::symlink_metadata(kotlin_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RootState::Missing)
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(WriteKotlinError::Other(format!(
            "Kotlin output root `{}` must be a directory",
            kotlin_root.display()
        )));
    }
    if fs::read_dir(kotlin_root)?.next().is_none() {
        return Ok(RootState::Empty);
    }

    let marker = kotlin_root.join(OWNERSHIP_MARKER);
    let marker_metadata = match fs::symlink_metadata(&marker) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(WriteKotlinError::Other(format!(
                "refusing to replace non-empty Kotlin output root `{}` without a prebindgen ownership marker",
                kotlin_root.display()
            )));
        }
        Err(error) => return Err(error.into()),
    };
    if marker_metadata.file_type().is_symlink() || !marker_metadata.is_file() {
        return Err(WriteKotlinError::Other(format!(
            "refusing to replace non-empty Kotlin output root `{}` without a prebindgen ownership marker",
            kotlin_root.display()
        )));
    }
    // Compare ignoring surrounding whitespace / line endings: the marker is a
    // sentinel, and git's `autocrlf` rewrites the committed LF marker to CRLF on
    // a Windows checkout — an exact-byte compare would then reject the (present,
    // valid) marker. Trimming still rejects a wrong/foreign/empty marker.
    if fs::read_to_string(&marker)?.trim() != OWNERSHIP_MARKER_CONTENT.trim() {
        return Err(WriteKotlinError::Other(format!(
            "refusing to replace non-empty Kotlin output root `{}` without a prebindgen ownership marker",
            kotlin_root.display()
        )));
    }
    Ok(RootState::Owned)
}

fn write_staging(files: &[KtFile], staging: &Path) -> Result<Vec<PathBuf>, WriteKotlinError> {
    fs::write(staging.join(OWNERSHIP_MARKER), OWNERSHIP_MARKER_CONTENT)?;
    let mut written = Vec::new();
    for file in files {
        let fallback = file
            .decls
            .first()
            .map(|decl| decl.name().to_string())
            .unwrap_or_else(|| "Generated".to_string());
        let relative_path = merged_file_path(Path::new(""), file, &fallback);
        ensure_relative_output_path(&relative_path)?;
        let path = staging.join(&relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, file.render())?;
        written.push(relative_path);
    }
    Ok(written)
}

fn ensure_relative_output_path(path: &Path) -> Result<(), WriteKotlinError> {
    if path.components().any(|component| {
        matches!(
            component,
            Component::RootDir | Component::Prefix(_) | Component::ParentDir
        )
    }) {
        return Err(WriteKotlinError::Other(format!(
            "Kotlin output path `{}` escapes the output root",
            path.display()
        )));
    }
    Ok(())
}

fn replace_root(
    kotlin_root: &Path,
    root_state: RootState,
    staging: &Path,
) -> Result<(), WriteKotlinError> {
    match root_state {
        RootState::Missing => fs::rename(staging, kotlin_root)?,
        RootState::Empty => {
            fs::remove_dir(kotlin_root)?;
            fs::rename(staging, kotlin_root)?;
        }
        RootState::Owned => {
            let backup = unique_sibling_path(kotlin_root, "previous");
            fs::rename(kotlin_root, &backup)?;
            if let Err(error) = fs::rename(staging, kotlin_root) {
                let _ = fs::rename(&backup, kotlin_root);
                return Err(error.into());
            }
            fs::remove_dir_all(backup)?;
        }
    }
    Ok(())
}

fn unique_sibling_path(kotlin_root: &Path, purpose: &str) -> PathBuf {
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = kotlin_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("kotlin");
    kotlin_root.with_file_name(format!(
        ".{name}.prebindgen-{purpose}-{}_{}",
        std::process::id(),
        sequence
    ))
}
