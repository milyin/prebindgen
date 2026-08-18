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
//! Three participants meet on this directory and nowhere else, and each is
//! named by a different manifest entry:
//!
//! - the source crate's `prebindgen` **build-dependency** prepares the
//!   directory and writes this file;
//! - the source crate's `prebindgen-proc-macro` **dependency** writes the
//!   captures into it;
//! - the binding crate's `prebindgen` **build-dependency** reads them.
//!
//! Nothing makes Cargo compare those. Two copies of prebindgen at different
//! versions, or at one version from different sources, is a legal graph — and
//! the first two are not even the same package, so a manifest can pair a
//! current `prebindgen` with a published `prebindgen-proc-macro` without a
//! word from anyone.
//!
//! [`FORMAT`] is what closes that. It describes the on-disk contract — the
//! directory layout, the names of the files in it, and the schema of a captured
//! record — and changes when any of those change, independently of the crate
//! version. It is checked twice, because two different packages write here:
//!
//! - [`Output::check_writer`] runs in the proc-macro before it captures, so a
//!   macro that writes some other layout fails while compiling the source
//!   crate, naming the two dependencies that disagree;
//! - [`Output::read`] runs in the reader before it reads a capture, and refuses
//!   anything but [`FORMAT`].
//!
//! A macro too old to run the first check cannot be made to announce itself, so
//! the reader recognises what it leaves behind instead: captures outside the
//! layout [`FORMAT`] describes are an error, not files to skip
//! ([`Source::discover_groups`](crate::Source)).
//!
//! **A reader accepts exactly [`FORMAT`].** There is no window of older formats
//! kept alive: every participant comes from one library, and the fix for a
//! mismatch is to say so in the manifest that named the odd one out.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

/// Name of the description file inside the capture directory.
pub(crate) const OUTPUT_FILE: &str = "prebindgen_output.toml";

/// What prebindgen <= 0.5.0 wrote instead, kept only to recognise its output.
const LEGACY_CRATE_NAME_FILE: &str = "crate_name.txt";

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
    ///
    /// Required, not defaulted: this list drives `cfg` filtering, so a
    /// description that lost it would quietly filter feature-gated captures
    /// away rather than report a damaged directory.
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
    /// read or parsed, or was written in a format other than [`FORMAT`]. Each
    /// is a build that would otherwise produce bindings for a surface nobody
    /// captured.
    pub(crate) fn read(dir: &Path) -> Self {
        let body = read_description(dir).unwrap_or_else(|e| panic!("{e}"));
        check_format(dir, &body).unwrap_or_else(|e| panic!("{e}"));
        toml::from_str(&body).unwrap_or_else(|e| {
            panic!(
                "Failed to read {} as prebindgen output: {e}. The capture directory is \
                 damaged — rebuild the source crate.",
                dir.join(OUTPUT_FILE).display()
            )
        })
    }

    /// Check, from the `#[prebindgen]` macro, that the directory it is about to
    /// capture into is described in the format this macro writes.
    ///
    /// The macro and the build script that prepared the directory are two
    /// packages — `prebindgen-proc-macro` and `prebindgen` — so a manifest can
    /// pair versions that do not agree on the layout. Returns the reason as a
    /// string; the macro turns it into a compile error on the captured item.
    pub(crate) fn check_writer(dir: &Path) -> Result<(), String> {
        let body = read_description(dir)?;
        check_format(dir, &body)
    }
}

/// The description text, or why there is none to read.
fn read_description(dir: &Path) -> Result<String, String> {
    let path = dir.join(OUTPUT_FILE);
    match fs::read_to_string(&path) {
        Ok(body) => Ok(body),
        // Only "there is no such file" is a directory nobody described. Every
        // other failure — a permission, a directory in its place, bytes that
        // are not UTF-8 — is a directory that cannot be read, and saying
        // "initialize it" about one of those sends the reader after the wrong
        // thing.
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!(
            "Failed to read {}: {e}. The capture directory is damaged — rebuild \
             the source crate.",
            path.display()
        )),
        // `crate_name.txt` is prebindgen <= 0.5.0's own marker for "this
        // directory was initialized", so its presence tells the two cases
        // apart: an older writer, or no writer at all.
        Err(_) if dir.join(LEGACY_CRATE_NAME_FILE).exists() => Err(format!(
            "The directory {} was written by prebindgen <= 0.5.0, which this prebindgen \
             cannot read: the capture layout changed and carries a format number now \
             ({OUTPUT_FILE}). Every prebindgen that touches one capture directory has to \
             agree on the format — they are separate dependencies, so Cargo does not \
             check it.",
            dir.display()
        )),
        Err(_) => Err(format!(
            "The directory {} was not initialized with init_prebindgen_out_dir(). \
             Please ensure that init_prebindgen_out_dir() is called in the build.rs \
             of the source crate.",
            dir.display()
        )),
    }
}

/// Whether the description in `body` is written in the format this prebindgen
/// understands.
///
/// The number is read on its own first, so that a description whose *schema*
/// this prebindgen does not know is still reported as the format mismatch it
/// is, rather than as a parse failure.
fn check_format(dir: &Path, body: &str) -> Result<(), String> {
    /// Just the format number, out of a description whose remaining shape may
    /// belong to another format.
    #[derive(Deserialize)]
    struct FormatOnly {
        format: u32,
    }

    let found = toml::from_str::<FormatOnly>(body)
        .map_err(|e| {
            format!(
                "Failed to read the format of {}: {e}. The capture directory is damaged — \
                 rebuild the source crate.",
                dir.join(OUTPUT_FILE).display()
            )
        })?
        .format;
    if found == FORMAT {
        return Ok(());
    }
    Err(format!(
        "The captures in {} are in prebindgen output format {found}, and this prebindgen \
         reads format {FORMAT}. The packages that write a capture directory and the one \
         that reads it are separate dependencies — `prebindgen-proc-macro` and \
         `prebindgen`, in the source crate and in the crate reading it — and these are \
         not the same prebindgen. Name one prebindgen in all of them.",
        dir.display(),
    ))
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
    #[should_panic(expected = "not the same prebindgen")]
    fn a_format_this_prebindgen_does_not_know_is_not_read() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(OUTPUT_FILE),
            format!(
                "format = {}\n\n[package]\nname = \"example-flat\"\nfeatures = []\n",
                FORMAT + 1
            ),
        )
        .unwrap();
        Output::read(dir.path());
    }

    #[test]
    #[should_panic(expected = "not the same prebindgen")]
    fn a_format_this_prebindgen_does_not_know_is_reported_by_its_number_alone() {
        // A future format may hold anything at all besides the number. Reading
        // the number on its own is what keeps this a version mismatch rather
        // than a parse failure the reader cannot act on.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(OUTPUT_FILE),
            format!("format = {}\n\n[crate]\nwho = \"knows\"\n", FORMAT + 1),
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

    #[test]
    #[should_panic(expected = "capture directory is damaged")]
    fn a_description_missing_its_feature_list_is_damaged_not_featureless() {
        // Defaulting the list to empty would filter every feature-gated capture
        // away and call the result a build.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(OUTPUT_FILE),
            "format = 1\n\n[package]\nname = \"example-flat\"\n",
        )
        .unwrap();
        Output::read(dir.path());
    }

    #[test]
    #[should_panic(expected = "capture directory is damaged")]
    fn a_description_that_cannot_be_read_is_not_a_missing_one() {
        // A directory where the file belongs: readable metadata, unreadable
        // contents, and nothing to do with whether a build script ever ran.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(OUTPUT_FILE)).unwrap();
        Output::read(dir.path());
    }

    #[test]
    fn the_macro_checks_the_format_it_is_about_to_write_into() {
        let dir = tempfile::tempdir().unwrap();
        Output::new("example-flat".to_string(), []).write(dir.path());
        assert!(Output::check_writer(dir.path()).is_ok());

        let stale = tempfile::tempdir().unwrap();
        fs::write(
            stale.path().join(OUTPUT_FILE),
            format!(
                "format = {}\n\n[package]\nname = \"x\"\nfeatures = []\n",
                FORMAT + 1
            ),
        )
        .unwrap();
        let err = Output::check_writer(stale.path()).unwrap_err();
        assert!(err.contains("not the same prebindgen"), "{err}");
    }
}
