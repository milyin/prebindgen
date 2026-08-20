//! The Kotlin half, compiled.
//!
//! Step 2b made every JNI cell *write* its Kotlin, which raised the floor from
//! "the Rust half exists" to "both halves exist". It did not ask whether the
//! Kotlin is a program. That is the same gap step 2 closed on the Rust side,
//! and the same argument applies: a generator that produced a file and a
//! generator that produced a file the compiler accepts are different claims,
//! and only the second is worth a row in a table.
//!
//! # One pass, and why each cell has its own package
//!
//! Every cell declares the same classes — `JNINative`, `NativeHandle`, the
//! declared surface — so a hundred cells in one package is a hundred
//! redeclarations. Compiling them one at a time instead costs a JVM start each,
//! which is the whole cost: one `kotlinc` over every cell takes about as long as
//! two separate ones.
//!
//! So a cell's package is `io.prebindgen.matrix.<cell id>`, set on the generator
//! rather than patched into the emitted text afterwards, and one invocation
//! covers the corpus. The id is also the directory the files are written under,
//! which is what attributes a diagnostic back to a cell — the same receipt rule
//! as [`crate::check`]: nothing maps a message to a cell by hand.
//!
//! # Not having a compiler is not a verdict
//!
//! `kotlinc` missing is reported as the stage failing to run, exactly like a
//! `cargo` that will not start. Every cell then stays at `rustc`, which is the
//! honest answer — but it is a *different report*, so a regeneration without the
//! compiler shows up as a large diff rather than as a quiet downgrade.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

/// One cell's emitted Kotlin.
pub struct Unit {
    /// The cell id — the receipt key, the package suffix, and the directory.
    pub id: String,
    /// Files as the generator wrote them: path relative to the output root,
    /// and contents.
    pub files: Vec<crate::run::KotlinFile>,
}

/// Which cells the Kotlin compiler accepted, and what it said about the rest.
#[derive(Default)]
pub struct Compiled {
    pub ok: BTreeSet<String>,
    /// Cell id → its diagnostics. Kept off the committed report for the reason
    /// rustc's are: they vary by compiler version, and the report has to be
    /// identical on every machine that builds it.
    pub failed: BTreeMap<String, Vec<String>>,
}

/// Compile every unit in one pass.
///
/// `workspace` names the output directory, so a caller with its own corpus —
/// this crate's own self-test — does not share a tree with the survey. The
/// sources are wiped before each pass and the tests run in parallel, so sharing
/// one would have them delete each other's inputs.
pub fn compile(workspace: &str, units: &[Unit]) -> Result<Compiled, String> {
    if units.is_empty() {
        return Ok(Compiled::default());
    }
    let root = crate::check::crate_dir()?
        .parent()
        .ok_or("locating the target directory")?
        .join("shape-matrix-kotlin")
        .join(workspace);
    let sources = write_sources(&root, units)?;

    let output = Command::new("kotlinc")
        .args(&sources)
        .arg("-nowarn")
        .arg("-d")
        .arg(root.join("classes"))
        .output()
        .map_err(|e| format!("running kotlinc (is it on PATH?): {e}"))?;

    let mut compiled = Compiled::default();
    for unit in units {
        compiled.ok.insert(unit.id.clone());
    }
    // kotlinc reports on stderr, one diagnostic per line.
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        let Some((file, message)) = diagnostic(line) else {
            continue;
        };
        let Some(id) = cell_of(&root, &file) else {
            continue;
        };
        compiled.ok.remove(&id);
        compiled.failed.entry(id).or_default().push(message);
    }

    // A failure with nothing attributable to a cell is this harness's — a
    // compiler that would not start, an unwritable directory. Reporting every
    // cell as compiling would be a lie in the direction that hides defects.
    if !output.status.success() && compiled.failed.is_empty() {
        return Err(format!(
            "kotlinc failed with nothing attributable to a cell:\n{stderr}"
        ));
    }
    Ok(compiled)
}

/// `<path>:<line>:<col>: error: <message>` — the file and the message.
///
/// Warnings are already suppressed with `-nowarn`, and anything else on the
/// stream (progress, the summary line) has no file position and is skipped.
fn diagnostic(line: &str) -> Option<(String, String)> {
    let (file, rest) = line.split_once(".kt:")?;
    let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit() || c == ':');
    let message = rest.strip_prefix(" error: ")?;
    Some((format!("{file}.kt"), message.trim().to_string()))
}

/// The cell a source file belongs to: the path component after the root's own
/// directory name, which is the id the sources were written out by.
///
/// Matched on that name rather than by stripping the root path, because kotlinc
/// reports a file the way it was easiest for kotlinc to print it — relative to
/// the working directory when the argument was relative to it — and a receipt
/// that only works for one of the two spellings would silently attribute
/// nothing.
fn cell_of(root: &Path, file: &str) -> Option<String> {
    let marker = root.file_name()?;
    let mut components = Path::new(file).components();
    components.find(|c| c.as_os_str() == marker)?;
    components
        .next()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
}

/// Write every unit under `<root>/<cell id>/`, and return the file list to
/// compile.
fn write_sources(root: &Path, units: &[Unit]) -> Result<Vec<PathBuf>, String> {
    // A cell removed from the corpus must not leave its Kotlin behind for the
    // next run to compile — a stale file would be attributed to a cell that no
    // longer exists, or worse, compile fine and pad the count.
    if root.exists() {
        std::fs::remove_dir_all(root).map_err(|e| format!("clearing {}: {e}", root.display()))?;
    }
    let mut sources = Vec::new();
    for unit in units {
        for file in &unit.files {
            let path = root.join(&unit.id).join(&file.path);
            let parent = path.parent().ok_or("a source file has no directory")?;
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("creating {}: {e}", parent.display()))?;
            crate::check::write(&path, &file.source)?;
            sources.push(path);
        }
    }
    Ok(sources)
}
