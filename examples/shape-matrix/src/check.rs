//! Does the emitted Rust actually compile?
//!
//! `plan` only says the generator produced a file. An emission can be
//! well-formed, contain every substring a unit test looks for, and still not
//! type-check — [`examples/emitcheck`](../../emitcheck) exists because that
//! happened once with 41 of 41 tests green over it. This stage asks rustc, per
//! cell.
//!
//! # The answer is a receipt, not a claim
//!
//! A cell is recorded as compiling only if **rustc says so about that cell's own
//! file**. Every cell is written to `<id>.rs`, the whole crate is checked in one
//! pass, and each diagnostic is attributed back by the file path the compiler
//! reports. Nothing maps a cell to a fixture by name — a name-keyed mapping can
//! claim coverage for a fixture that never touched the cell, which is the defect
//! #175 was.
//!
//! # What it does not cover
//!
//! rustc, and rustc only. Neither `cbindgen` nor the Kotlin compiler runs here,
//! so a cell that compiles has not been shown to produce a valid C header or a
//! loadable JVM class. Those are the rest of `ToolchainCompiled`, and they are
//! not collected yet.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

/// One cell's emitted Rust, ready to be checked.
pub struct Unit {
    /// `<shape>__<position>__<target>`, the receipt key.
    pub id: String,
    /// The fixture's own items — what the source crate would contain.
    pub fixture: String,
    /// What the generator emitted for it.
    pub emitted: String,
}

/// Which cells rustc accepted, and what it said about the rest.
#[derive(Default)]
pub struct Checked {
    pub compiled: BTreeSet<String>,
    /// Cell id → the diagnostics rustc reported for it. **Not** rendered into
    /// the committed report: a message is a property of the compiler version,
    /// and the report has to be identical on every toolchain that builds it.
    pub failed: BTreeMap<String, Vec<String>>,
}

/// Check every unit in one crate, and attribute the result per cell.
///
/// `Err` is reserved for the check itself failing to run — a missing cargo, an
/// unwritable directory. That is not a verdict about any cell, and callers must
/// not record it as one.
/// `workspace` names the directory this batch is checked in. Two batches must
/// not share one: the report's run and the self-test run concurrently under
/// `cargo test`, and a shared directory would have them overwriting each
/// other's sources. A *stable* name per batch rather than a unique one per
/// call, so the dependencies stay compiled between runs and the emitted Rust
/// stays on disk to be read after a failure.
pub fn check(workspace: &str, units: &[Unit]) -> Result<Checked, String> {
    if units.is_empty() {
        return Ok(Checked::default());
    }
    let root = crate_dir()?.join(workspace);
    write_crate(&root, units)?;

    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .arg("check")
        .arg("--quiet")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        // Its own target directory: the parent build may still hold the
        // workspace one, and a nested cargo blocking on that lock would look
        // like a hang rather than a queue.
        .arg("--target-dir")
        .arg(root.join("target"))
        .output()
        .map_err(|e| format!("running cargo check: {e}"))?;

    let mut checked = Checked::default();
    for unit in units {
        checked.compiled.insert(unit.id.clone());
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((file, message)) = diagnostic(line) else {
            continue;
        };
        let Some(id) = cell_of(&file) else { continue };
        checked.compiled.remove(&id);
        checked.failed.entry(id).or_default().push(message);
    }

    // A crate that failed to build with no attributable diagnostic means the
    // failure was the harness's, not a cell's — a bad `Cargo.toml`, a missing
    // dependency. Reporting every cell as compiling would be a lie in the
    // direction that hides defects.
    if !output.status.success() && checked.failed.is_empty() {
        return Err(format!(
            "cargo check failed with nothing attributable to a cell:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(checked)
}

/// The file and message of one rustc error, or `None` for anything else on the
/// JSON stream.
///
/// Deliberately hand-parsed rather than pulled in with `serde_json`: this reads
/// two fields of a stable format, and the alternative is a dependency in a crate
/// whose whole point is to have no opinions of its own.
fn diagnostic(line: &str) -> Option<(String, String)> {
    if !line.contains("\"level\":\"error\"") {
        return None;
    }
    let rendered = field(line, "\"rendered\":\"")?;
    let file = rendered
        .split(&['\\', '"'][..])
        .find(|part| part.ends_with(".rs") && part.contains(CELL_SEP))
        .or_else(|| {
            rendered
                .split_whitespace()
                .find(|w| w.contains(".rs") && w.contains(CELL_SEP))
        })?
        .to_string();
    let message = rendered.lines().next().unwrap_or(&rendered).to_string();
    Some((file, message))
}

fn field(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => break,
            },
            other => out.push(other),
        }
    }
    Some(out)
}

/// The separator between a cell id's parts, chosen so a file name cannot be
/// mistaken for anything else on a diagnostic line.
const CELL_SEP: &str = "__";

fn cell_of(file: &str) -> Option<String> {
    let stem = Path::new(file.trim_matches(|c: char| !c.is_ascii_graphic()))
        .file_stem()?
        .to_str()?;
    stem.contains(CELL_SEP).then(|| stem.to_string())
}

/// Where the generated crate lives: under the workspace target directory, so it
/// is already ignored by git and cleaned by `cargo clean`.
pub(crate) fn crate_dir() -> Result<PathBuf, String> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or("locating the workspace root")?;
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace.join("target"));
    Ok(target.join("shape-matrix-check"))
}

fn write_crate(root: &Path, units: &[Unit]) -> Result<(), String> {
    let src = root.join("src");
    std::fs::create_dir_all(&src).map_err(|e| format!("creating {}: {e}", src.display()))?;

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().and_then(Path::parent).expect("workspace");
    write(
        &root.join("Cargo.toml"),
        &manifest_toml("shape-matrix-check", &workspace.display().to_string()),
    )?;

    let mut lib = String::from(LIB_HEADER);
    for (n, unit) in units.iter().enumerate() {
        let file = format!("{}.rs", unit.id);
        write(&src.join(&file), &cell_source(unit))?;
        lib.push_str(&format!("#[path = \"{file}\"]\npub mod cell_{n};\n"));
    }
    write(&src.join("lib.rs"), &lib)
}

pub(crate) fn write(path: &Path, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// One cell as a Rust module: the source crate, then the generated file.
///
/// This mirrors how a binding crate is actually built — `pub mod myflat;` plus
/// `include!("generated_bindings.rs")` in `emitcheck`, the same two lines every
/// consumer writes.
///
/// The imports go **inside** the fixture module and nowhere else. A source crate
/// writing `Cow<'static, str>` has imported `Cow`; the generated file is a
/// separate scope, and if it needs an import nobody gave it, that is a finding
/// about the generator rather than something for this harness to paper over.
pub(crate) fn cell_source(unit: &Unit) -> String {
    format!(
        "// Generated by shape-matrix. Do not edit.\n\
         #![allow(clippy::all, dead_code, unused_imports, unused_variables)]\n\
         \n\
         pub mod {} {{\n\
         use std::borrow::Cow;\n\
         use std::mem::MaybeUninit;\n\
         {}\n\
         }}\n\
         \n\
         {}\n",
        crate::run::SOURCE_CRATE,
        unit.fixture,
        unit.emitted
    )
}

/// The manifest of a generated crate: exactly the dependencies a real binding
/// crate declares, and nothing else.
pub(crate) fn manifest_toml(name: &str, workspace: &str) -> String {
    format!(
        r#"# Generated by shape-matrix. Do not edit.
[package]
name = "{name}"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"

# Exactly what generated code calls into — the dependencies a real binding
# crate declares, and nothing else.
[dependencies]
prebindgen-jni-runtime = {{ path = "{workspace}/prebindgen-jni-runtime" }}
prebindgen-c-runtime = {{ path = "{workspace}/prebindgen-c-runtime" }}
# Pinned exactly, and to what the workspace already resolves: this crate has
# its own lockfile, so a caret range would let a new upstream release change a
# cell's answer with nothing in this repo having changed.
jni = "=0.21.1"
tracing = "=0.1.44"
konst = "=0.3.17"

[workspace]
"#
    )
}

const LIB_HEADER: &str = "\
// Generated by shape-matrix. Do not edit.
//
// One module per cell that produced Rust, each in its own file so a diagnostic
// names the cell it belongs to.
";
