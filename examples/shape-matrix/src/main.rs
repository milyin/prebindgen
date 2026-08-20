//! Regenerates `REPORT.md`.
//!
//! Committed next to the source, and diffed by `examples/regen-check.sh`, for
//! the same reason every other generated artifact in this repo is: the
//! generators both decide legality and produce this report, so *"the build
//! fails until new cells are classified"* is vacuous on its own — a regression
//! that flips a working cell to `rejected` would be recorded as a successful
//! classification. A committed report makes every change to an answer a
//! reviewed diff.

use std::path::PathBuf;

fn main() {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let dest = here.join(shape_matrix::REPORT_PATH);
    let report = shape_matrix::report::render();
    std::fs::write(&dest, &report).expect("write REPORT.md");
    eprintln!("shape-matrix: wrote {}", dest.display());

    // Raising a floor is a deliberate act, so it takes a flag. Doing it on every
    // run would let a cell's floor follow it downwards — the ratchet would hold
    // nothing.
    if std::env::args().any(|arg| arg == "--update-guarantees") {
        let dest = here.join(shape_matrix::guarantees::PATH);
        let updated = shape_matrix::guarantees::updated(shape_matrix::report::survey());
        std::fs::write(&dest, &updated).expect("write GUARANTEES.md");
        eprintln!("shape-matrix: raised floors in {}", dest.display());
    }
}
