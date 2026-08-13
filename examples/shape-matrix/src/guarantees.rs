//! The ratchet: cells may improve, and may not quietly get worse.
//!
//! The committed report already makes every changed answer a reviewed diff. That
//! catches a regression only if somebody reads the diff and knows which
//! direction is bad — and a diff shows `header` becoming `rejected` in exactly
//! the same shade as the reverse.
//!
//! So each cell that produces something carries a **floor**: the level it has
//! been seen to reach. Rising above it is free and silent. Falling below it
//! fails a test that names the cell and both levels. The two gates answer
//! different questions — *"did anything move?"* and *"did anything move
//! **down**?"* — and only the second can be enforced without a reviewer.
//!
//! # Raising is automatic, lowering is a hand edit
//!
//! `cargo run -p shape-matrix -- --update-guarantees` raises floors to what the
//! run just achieved and **never lowers one**. Removing a guarantee means
//! editing this file by hand, which is the point: giving up on a shape that used
//! to work should cost a line in a diff that a reviewer can see and argue with,
//! not a silently regenerated artifact.

use std::{collections::BTreeMap, fmt::Write as _};

use crate::report::Cell;

/// How far a cell got, as an ordered ladder.
///
/// `Ord` is the whole mechanism: a floor is a `>=` comparison, so the ladder's
/// order is a load-bearing property rather than a presentation detail.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    /// The generator refused the shape, panicked, or the placement is not Rust.
    /// No floor is recorded for these — a cell that never worked is not a
    /// regression waiting to happen.
    Nothing,
    /// Something was generated, and rustc has not accepted it.
    Generates,
    /// The emitted Rust compiles.
    Compiles,
    /// C only: cbindgen declares the wrapper. The top of C's ladder; JNI's ends
    /// at [`Level::Compiles`] until the Kotlin compiler runs.
    Header,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Nothing => "nothing",
            Level::Generates => "generates",
            Level::Compiles => "compiles",
            Level::Header => "header",
        }
    }

    fn parse(text: &str) -> Option<Level> {
        [
            Level::Nothing,
            Level::Generates,
            Level::Compiles,
            Level::Header,
        ]
        .into_iter()
        .find(|level| level.as_str() == text)
    }
}

/// Where the committed floors live, relative to this crate.
pub const PATH: &str = "GUARANTEES.md";

/// One cell falling below its floor.
pub struct Regression {
    pub id: String,
    pub floor: Level,
    pub now: Level,
}

impl std::fmt::Display for Regression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: was guaranteed to reach `{}`, now reaches only `{}`",
            self.id,
            self.floor.as_str(),
            self.now.as_str()
        )
    }
}

/// Every floor the committed file records.
pub fn committed() -> BTreeMap<String, Level> {
    parse(&read())
}

fn read() -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(PATH))
        .unwrap_or_default()
}

fn parse(text: &str) -> BTreeMap<String, Level> {
    let mut floors = BTreeMap::new();
    for line in text.lines() {
        // Table rows only: `| cell | level |`. Everything else in the file is
        // prose, and prose is not data.
        let mut columns = line.split('|').map(str::trim).filter(|c| !c.is_empty());
        let (Some(id), Some(level)) = (columns.next(), columns.next()) else {
            continue;
        };
        let id = id.trim_matches('`');
        if let Some(level) = Level::parse(level.trim_matches('`')) {
            floors.insert(id.to_string(), level);
        }
    }
    floors
}

/// What a survey achieved, per cell.
pub fn observed(cells: &[Cell]) -> BTreeMap<String, Level> {
    cells.iter().map(|cell| (cell.id(), cell.level())).collect()
}

/// The cells that have fallen below their floor.
///
/// Split from [`committed`] so the rule can be tested against floors this crate
/// states rather than only against the ones on disk — a gate whose behaviour can
/// only be observed by breaking the repository is a gate nobody checks.
pub fn regressions_of(
    floors: &BTreeMap<String, Level>,
    observed: &BTreeMap<String, Level>,
) -> Vec<Regression> {
    floors
        .iter()
        .filter_map(|(id, floor)| {
            let now = *observed.get(id)?;
            (now < *floor).then_some(Regression {
                id: id.clone(),
                floor: *floor,
                now,
            })
        })
        .collect()
}

/// The cells that have fallen below their committed floor.
pub fn regressions(cells: &[Cell]) -> Vec<Regression> {
    regressions_of(&committed(), &observed(cells))
}

/// Floors raised to what a run achieved. **Never lowered** — that is the
/// ratchet, and the reason this is not simply "write down what happened".
pub fn raised(
    mut floors: BTreeMap<String, Level>,
    observed: &BTreeMap<String, Level>,
) -> BTreeMap<String, Level> {
    for (id, level) in observed {
        let entry = floors.entry(id.clone()).or_insert(Level::Nothing);
        *entry = (*entry).max(*level);
    }
    floors.retain(|_, level| *level > Level::Nothing);
    floors
}

/// The file's contents after this run.
pub fn updated(cells: &[Cell]) -> String {
    let floors = raised(committed(), &observed(cells));

    let mut out = String::from(HEADER);
    out.push_str("| Cell | Floor |\n|---|---|\n");
    for (id, level) in &floors {
        let _ = writeln!(out, "| `{id}` | `{}` |", level.as_str());
    }
    out
}

const HEADER: &str = "\
<!-- Floors are raised by `cargo run -p shape-matrix -- --update-guarantees`.
     Lowering one is a hand edit, on purpose. -->
# Guaranteed levels

The level each cell has been seen to reach. A cell may rise above its floor
freely; falling below it fails `no_cell_falls_below_its_guarantee`, which names
the cell and both levels.

This is the gate the committed `REPORT.md` cannot be: a byte-identity diff shows
a cell getting worse in exactly the same shade as one getting better, so it
catches a regression only if a reviewer reads the diff and knows which direction
is which. A floor does not need a reviewer.

`header` is the top of C's ladder and `compiles` the top of Kotlin/JNI's, since
this crate does not run the Kotlin compiler yet.

";
