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
    let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(shape_matrix::REPORT_PATH);
    let report = shape_matrix::report::render();
    std::fs::write(&dest, &report).expect("write REPORT.md");
    eprintln!("shape-matrix: wrote {}", dest.display());
}
