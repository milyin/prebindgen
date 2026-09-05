//! The emitted-surface manifest: what v2 generated, what it skipped, and why.
//!
//! Two renderings of one value — JSON for tooling (test-section selection reads
//! it) and Markdown for a person. Both are deterministic: elements sort by kind
//! then id, and skip causes are grouped by code so a single missing capability
//! is stated once with the list of roots it took down, rather than repeated
//! forty times.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{
    decl::{DeclaredElement, ElementKind},
    outcome::{EngineError, Outcome},
};

/// The manifest's own version. A consumer that reads the JSON checks this
/// before trusting the shape; it changes whenever a field's meaning does.
pub const SCHEMA_VERSION: u32 = 1;

/// One accounted-for element.
#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    #[serde(flatten)]
    pub element: DeclaredElement,
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// What a run generated, and what it did not.
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    /// Which engine produced this.
    pub pipeline: &'static str,
    /// Which target it generated for — `"c"`, `"jni"`.
    pub target: &'static str,
    /// See [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Enough of the run's input to tell a stale report from a fresh one: the
    /// declaring crate and the source directories it read.
    pub source_identity: SourceIdentity,
    /// Every declared element, sorted by kind then id.
    pub elements: Vec<Entry>,
}

/// What this report was generated from.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SourceIdentity {
    /// The crate whose build script declared the binding.
    pub declaring_crate: String,
    /// The captured-item directories the model was read from.
    pub sources: Vec<String>,
    /// How many `#[prebindgen]` items those directories held.
    pub captured_items: usize,
}

impl Report {
    /// How many elements ended each way.
    pub fn counts(&self) -> Counts {
        let mut counts = Counts::default();
        for entry in &self.elements {
            match entry.outcome {
                Outcome::Emitted => counts.emitted += 1,
                Outcome::Skipped(_) => counts.skipped += 1,
                Outcome::Ignored => counts.ignored += 1,
            }
        }
        counts
    }

    /// The skipped elements grouped by capability code, each group's roots
    /// sorted. One cause, all of its casualties — which is what a reader needs
    /// to decide what to implement next.
    pub fn skips_by_capability(&self) -> BTreeMap<&str, Vec<&Entry>> {
        let mut groups: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
        for entry in &self.elements {
            if let Some(skip) = entry.outcome.skip() {
                groups
                    .entry(skip.capability.as_str())
                    .or_default()
                    .push(entry);
            }
        }
        groups
    }

    /// The one-line summary a build script prints.
    pub fn summary(&self) -> String {
        let counts = self.counts();
        format!(
            "{}: {} emitted, {} skipped, {} ignored ({} target)",
            self.pipeline, counts.emitted, counts.skipped, counts.ignored, self.target
        )
    }

    /// The manifest as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a report is plain data and always serializes")
    }

    /// The manifest as Markdown.
    pub fn to_markdown(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        let counts = self.counts();
        let _ = writeln!(
            out,
            "# prebindgen {} report — {}\n",
            self.pipeline, self.target
        );
        let _ = writeln!(
            out,
            "Declared by `{}` over {} captured item(s) from: {}\n",
            self.source_identity.declaring_crate,
            self.source_identity.captured_items,
            if self.source_identity.sources.is_empty() {
                "(no source directory)".to_string()
            } else {
                self.source_identity.sources.join(", ")
            }
        );
        let _ = writeln!(
            out,
            "| emitted | skipped | ignored |\n| ---: | ---: | ---: |\n| {} | {} | {} |\n",
            counts.emitted, counts.skipped, counts.ignored
        );

        let _ = writeln!(out, "## Skipped, by cause\n");
        let groups = self.skips_by_capability();
        if groups.is_empty() {
            let _ = writeln!(out, "Nothing skipped.\n");
        }
        for (capability, entries) in &groups {
            let explanation = entries
                .first()
                .and_then(|entry| entry.outcome.skip())
                .map(|skip| skip.explanation.as_str())
                .unwrap_or_default();
            let _ = writeln!(out, "### `{capability}` — {explanation}\n");
            for entry in entries {
                let path = entry
                    .outcome
                    .skip()
                    .map(|skip| skip.path())
                    .unwrap_or_default();
                let _ = writeln!(out, "- `{}` ({})", entry.element.id, path);
            }
            let _ = writeln!(out);
        }

        let _ = writeln!(out, "## Every declared element\n");
        let _ = writeln!(out, "| element | representation | placement | outcome |");
        let _ = writeln!(out, "| --- | --- | --- | --- |");
        for entry in &self.elements {
            let outcome = match &entry.outcome {
                Outcome::Emitted => "emitted".to_string(),
                Outcome::Ignored => "ignored".to_string(),
                Outcome::Skipped(skip) => format!("skipped: `{}`", skip.capability),
            };
            let _ = writeln!(
                out,
                "| `{}` | {} | `{}` | {} |",
                entry.element.id, entry.element.representation, entry.element.placement, outcome
            );
        }
        out
    }

    /// Write both renderings into `dir`, which is created if missing.
    ///
    /// Returns the paths written, in the order they were written.
    pub fn write(&self, dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, EngineError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let json = dir.join(format!("{}-report.json", self.target));
        let markdown = dir.join(format!("{}-report.md", self.target));
        std::fs::write(&json, self.to_json())?;
        std::fs::write(&markdown, self.to_markdown())?;
        Ok(vec![json, markdown])
    }

    /// Print the summary and one actionable line per skip cause as cargo
    /// warnings. The full detail stays in the written report — a build log is
    /// not the manifest.
    pub fn warn(&self) {
        println!("cargo:warning={}", self.summary());
        for (capability, entries) in self.skips_by_capability() {
            let roots = entries
                .iter()
                .map(|entry| entry.element.id.as_str())
                .collect::<Vec<_>>();
            let shown = roots.len().min(5);
            let more = match roots.len() - shown {
                0 => String::new(),
                rest => format!(" (+{rest} more)"),
            };
            println!(
                "cargo:warning=SKIP {capability}: {}{more}",
                roots[..shown].join(", ")
            );
        }
    }
}

/// How a run's elements were accounted for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub emitted: usize,
    pub skipped: usize,
    pub ignored: usize,
}

/// Sort key: kind first (so a report reads types, then functions), then id.
pub(crate) fn sort_entries(entries: &mut [Entry]) {
    entries.sort_by(|a, b| {
        kind_order(a.element.kind)
            .cmp(&kind_order(b.element.kind))
            .then_with(|| a.element.id.cmp(&b.element.id))
    });
}

fn kind_order(kind: ElementKind) -> u8 {
    match kind {
        ElementKind::Type => 0,
        ElementKind::Conversion => 1,
        ElementKind::Callback => 2,
        ElementKind::Const => 3,
        ElementKind::Function => 4,
    }
}
