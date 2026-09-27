//! # prebindgen-jni
//!
//! The JNI / Kotlin language adapter: turns the `#[prebindgen]` items of a
//! flat Rust crate into a Rust file of JNI exports plus the Kotlin sources
//! that call them — typed handle classes, data classes, enum classes, sealed
//! interfaces, callbacks and wrapper functions.
//!
//! ```rust,ignore
//! use prebindgen_jni::{data_class, fun, package, ptr_class, JniGen};
//!
//! let generation = JniGen::builder()
//!     .source(my_flat::PREBINDGEN_OUT_DIR)
//!     .set_package_prefix("io.example")
//!     .set_jni_native_init("io.example.NativeLibrary.ensureLoaded()")
//!     .package(
//!         package!()
//!             .class(ptr_class!(Storage).method(fun!(storage_len)))
//!             .class(data_class!(Payload))
//!             .fun(fun!(storage_new)),
//!     )
//!     .build()?;
//! generation.write_rust("src/generated_bindings.rs")?;
//! generation.write_kotlin("kotlin/generated")?;
//! ```
//!
//! ## How values cross
//!
//! Generated Rust never touches a Kotlin object's fields. Every value crosses
//! as a flat list of *leaves* — JNI primitives, strings, primitive arrays,
//! object arrays — and the generated Kotlin takes objects apart and puts them
//! back together:
//!
//! * a handle (`ptr_class!`) is its pointer, a `Long`;
//! * an enum (`enum_class!`) is its value, an `Int`;
//! * a data class (`data_class!`) is its fields' leaves side by side;
//! * a sealed class (`sealed_class!`) is a tag plus every alternative's
//!   fields;
//! * `Option<T>` adds a presence flag (a single object leaf is just nullable);
//! * `Vec<T>` is a count plus one array per leaf of `T`.
//!
//! A result with several leaves reaches Kotlin through one upcall to a
//! generated sink that assembles it. `expand_return!` turns a result into
//! its fields handed to a caller-supplied builder (a folder for a sequence);
//! `expand_param!` lets a parameter be built from a constructor's arguments
//! or passed as a handle, chosen by a selector.
//!
//! ## Where to start
//!
//! [`JniGenBuilder`] is the entry point: [`JniGenBuilder::package`] declares
//! what Kotlin sees, [`JniGenBuilder::convert`] and
//! [`JniGenBuilder::expand`] shape how types cross. The
//! `examples/perftest-kotlin` build script is a small binding;
//! `examples/covertest-kotlin/build.rs` uses every declaration, and its
//! `Test.kt` shows the resulting Kotlin API in use. `docs/architecture.md` in
//! the repository describes how the adapter is built.
//!
//! Every wrapper takes a `JniErrorHandler` last: a native call that cannot
//! complete — a closed handle, an out-of-range value, a failed conversion, an
//! `Err` with no typed handler — reports there instead of throwing.

mod builder;
mod decl;
mod gen;

use std::path::{Path, PathBuf};

pub use builder::{JniGen, JniGenBuilder};
pub use decl::{
    matching, ClassDecl, ConstDecl, ConvertDecl, DataClassDecl, EnumClassDecl, ExpandDecl,
    ExpandParamDecl, ExpandReturnDecl, FieldsDecl, FunctionDecl, IgnoreDecl, PackageDecl,
    PtrClassDecl, SealedClassDecl, VariantDecl,
};
#[doc(hidden)]
pub use prebindgen_tools::__syn;
pub use prebindgen_tools::{expr, from, ident, into, path, sig, try_from, try_into, ty};
use prebindgen_tools::{file::write_if_changed, RustFile};

/// A generation failure.
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// The generated binding: the Rust file, the Kotlin sources and a report.
pub struct Generation {
    rust: RustFile,
    kotlin: Vec<(String, String)>,
    report: String,
}

/// Marks a directory whose `.kt` files the generator owns.
const MARKER: &str = ".kotlin-codegen-output";

impl Generation {
    /// The generated Rust source.
    pub fn render_rust(&self) -> String {
        self.rust.render()
    }

    /// Write the Rust file (a relative path lands in `OUT_DIR`).
    pub fn write_rust(&self, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        self.rust.write(path)
    }

    /// The Kotlin sources: `(package, text)`, one per package.
    pub fn kotlin(&self) -> &[(String, String)] {
        &self.kotlin
    }

    /// Write the Kotlin sources under `root`, one file per package
    /// (`io.example.model` → `io/example/model.kt`). The directory is the
    /// generator's: stale `.kt` files from an earlier run are removed.
    pub fn write_kotlin(&self, root: impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
        let root = root.as_ref();
        let mut written = Vec::new();
        for (pkg, text) in &self.kotlin {
            let mut path = root.to_path_buf();
            let parts: Vec<&str> = pkg.split('.').collect();
            for p in &parts[..parts.len() - 1] {
                path.push(p);
            }
            path.push(format!("{}.kt", parts[parts.len() - 1]));
            write_if_changed(&path, text)?;
            written.push(path);
        }
        if root.join(MARKER).exists() {
            remove_stale(root, &written)?;
        }
        write_if_changed(&root.join(MARKER), "kotlin-codegen output v1\n")?;
        Ok(written)
    }

    /// A Markdown summary of the bound surface.
    pub fn report(&self) -> String {
        self.report.clone()
    }
}

fn remove_stale(dir: &Path, keep: &[PathBuf]) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            remove_stale(&path, keep)?;
        } else if path.extension().is_some_and(|e| e == "kt") && !keep.contains(&path) {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
