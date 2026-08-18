//! The description file a capture directory carries, and the compatibility
//! rule that goes with it.
//!
//! ```toml
//! # {OUT_DIR}/prebindgen/prebindgen_output.toml
//! format = 1
//!
//! [package]
//! name = "example-flat"
//! features = ["unstable"]
//! ```
//!
//! [`init_prebindgen_out_dir`](crate::init_prebindgen_out_dir) writes it when
//! it prepares the directory, and [`Source`](crate::Source) reads it before it
//! reads a single capture. `[package]` spells the source crate the way
//! `Cargo.toml` does — its `name`, and the `features` that were enabled for the
//! compilation that captured.
//!
//! # Why the format number is here
//!
//! The two halves of prebindgen meet on this directory and nowhere else. The
//! source crate names the half that writes (the `#[prebindgen]` proc-macro,
//! plus this crate in its `build.rs`); the binding crate names the half that
//! reads. They are separate dependency edges in separate manifests, so nothing
//! makes Cargo compare them — two copies of prebindgen at different versions,
//! or at one version from different sources, is a legal graph and a silent one.
//!
//! [`FORMAT`] is what closes that. It describes the on-disk contract — the
//! directory layout, the names of the files in it, and the schema of a captured
//! record — and changes when any of those change, independently of the crate
//! version. A reader that meets a number it does not know says so and stops,
//! rather than reporting a capture directory it cannot read as an empty one.
//!
//! **A reader accepts exactly [`FORMAT`].** There is no window of older formats
//! kept alive: the two halves come from one library, and the fix for a mismatch
//! is to say so in the manifest that named the other one.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

/// Name of the description file inside the capture directory.
pub(crate) const OUTPUT_FILE: &str = "prebindgen_output.toml";

/// The on-disk contract this prebindgen writes and reads.
///
/// 1 — `prebindgen_output.toml`, and captures at
/// `g_{group}/{name}_{digest}.jsonl`. prebindgen ≤ 0.5.0 wrote no description
/// file at all (`crate_name.txt` and `features.txt` instead, and captures at
/// `{group}_{pid}_{thread}.jsonl`); it is recognised only to name itself in the
/// error.
pub(crate) const FORMAT: u32 = 1;

/// The description of a capture directory: what wrote it, and in what format.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Output {
    /// See [`FORMAT`].
    pub(crate) format: u32,
    pub(crate) package: Package,
}

/// The source crate, spelled as `Cargo.toml` spells it.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Package {
    /// `CARGO_PKG_NAME` of the crate that captured.
    pub(crate) name: String,
    /// The features enabled for the compilation that captured, sorted.
    #[serde(default)]
    pub(crate) features: Vec<String>,
}

impl Output {
    /// The description of a crate captured under `features`.
    pub(crate) fn new(crate_name: String, features: impl IntoIterator<Item = String>) -> Self {
        let mut features = features.into_iter().collect::<Vec<_>>();
        features.sort();
        features.dedup();
        Self {
            format: FORMAT,
            package: Package {
                name: crate_name,
                features,
            },
        }
    }

    /// Write the description into a prepared capture directory.
    ///
    /// # Panics
    ///
    /// If the file cannot be serialized or written — either means the build
    /// script cannot produce a directory anyone may read.
    pub(crate) fn write(&self, dir: &Path) {
        let path = dir.join(OUTPUT_FILE);
        let body = toml::to_string(self)
            .unwrap_or_else(|e| panic!("Failed to describe the prebindgen output: {e}"));
        fs::write(&path, body)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", path.display(), e));
    }

    /// Read the description of `dir` and check that this prebindgen can read
    /// what it describes.
    ///
    /// # Panics
    ///
    /// If the directory carries no description, carries one that cannot be
    /// parsed, or was written in a format other than [`FORMAT`]. Each is a
    /// build that would otherwise produce bindings for a surface nobody
    /// captured.
    pub(crate) fn read(dir: &Path) -> Self {
        let path = dir.join(OUTPUT_FILE);
        let body = fs::read_to_string(&path).unwrap_or_else(|_| {
            // `crate_name.txt` is prebindgen <= 0.5.0's own marker for "this
            // directory was initialized", so its presence tells the two cases
            // apart: an older writer, or no writer at all.
            assert!(
                !dir.join("crate_name.txt").exists(),
                "The directory {} was written by prebindgen <= 0.5.0, which this prebindgen \
                 cannot read: the capture layout changed and carries a format number now \
                 ({OUTPUT_FILE}). The source crate's prebindgen and this build script's \
                 prebindgen have to be the same version — they are separate dependencies, \
                 so Cargo does not check it.",
                dir.display()
            );
            panic!(
                "The directory {} was not initialized with init_prebindgen_out_dir(). \
                 Please ensure that init_prebindgen_out_dir() is called in the build.rs \
                 of the source crate.",
                dir.display()
            )
        });
        let output: Self = toml::from_str(&body).unwrap_or_else(|e| {
            panic!(
                "Failed to read {} as prebindgen output: {e}. The capture directory is \
                 damaged — rebuild the source crate.",
                path.display()
            )
        });
        assert!(
            output.format == FORMAT,
            "The captures in {} are in prebindgen output format {}, and this prebindgen \
             reads format {FORMAT}. The source crate and this build script resolve to \
             different prebindgen versions; name the same one in both manifests.",
            dir.display(),
            output.format,
        );
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_description_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        Output::new(
            "example-flat".to_string(),
            ["unstable".to_string(), "extra".to_string()],
        )
        .write(dir.path());

        let read = Output::read(dir.path());
        assert_eq!(read.format, FORMAT);
        assert_eq!(read.package.name, "example-flat");
        assert_eq!(read.package.features, ["extra", "unstable"]);
    }

    #[test]
    fn the_description_spells_the_crate_the_way_cargo_does() {
        let toml = toml::to_string(&Output::new(
            "example-flat".to_string(),
            ["unstable".to_string()],
        ))
        .unwrap();
        assert_eq!(
            toml,
            "format = 1\n\n[package]\nname = \"example-flat\"\nfeatures = [\"unstable\"]\n"
        );
    }

    #[test]
    #[should_panic(expected = "was not initialized with init_prebindgen_out_dir()")]
    fn a_directory_no_build_script_prepared_says_so() {
        let dir = tempfile::tempdir().unwrap();
        Output::read(dir.path());
    }

    #[test]
    #[should_panic(expected = "written by prebindgen <= 0.5.0")]
    fn an_unstamped_directory_names_the_prebindgen_that_wrote_it() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("crate_name.txt"), "example-flat").unwrap();
        Output::read(dir.path());
    }

    #[test]
    #[should_panic(expected = "resolve to different prebindgen versions")]
    fn a_format_this_prebindgen_does_not_know_is_not_read() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(OUTPUT_FILE),
            format!(
                "format = {}\n\n[package]\nname = \"example-flat\"\n",
                FORMAT + 1
            ),
        )
        .unwrap();
        Output::read(dir.path());
    }

    #[test]
    #[should_panic(expected = "capture directory is damaged")]
    fn a_description_that_does_not_parse_is_not_guessed_at() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(OUTPUT_FILE), "format = 1\n[package]\n").unwrap();
        Output::read(dir.path());
    }
}
