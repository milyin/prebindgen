//! Rendering the table.
//!
//! The output is committed and diffed in CI, so it must be a pure function of
//! the corpus and the generators: no timestamps, no paths, no iteration order
//! that depends on a hash map.

use std::fmt::Write as _;

use crate::{
    check::{self, Unit},
    corpus::{Call, Policy, Position, Shape, CALLS, POLICIES, SHAPES},
    header::{self, Header},
    run::{declarations, run, run_call, run_policy, ClassKind, State, Target},
    tag::TypeTag,
};

/// Every cell's outcome, computed once per process.
///
/// Memoized because two consumers want it — the report and the guarantee
/// ratchet — and a survey costs a full pass over both generators plus a cargo
/// check. Running it twice in one test binary would double the suite's time to
/// produce the same answer.
pub fn survey() -> &'static [Cell] {
    static SURVEY: std::sync::OnceLock<Vec<Cell>> = std::sync::OnceLock::new();
    SURVEY.get_or_init(run_all)
}

/// The whole report.
pub fn render() -> String {
    let mut out = String::new();
    out.push_str(HEADER);

    let results = survey();

    render_summary(&mut out, results);
    for position in Position::ALL {
        render_position(&mut out, results, *position);
    }
    render_calls(&mut out, results);
    render_runtime(&mut out);
    render_policies(&mut out, results);
    render_coverage(&mut out);
    render_class_coverage(&mut out);

    out
}

/// What a cell is about: one value in one position, or a whole call.
#[derive(Clone, Copy)]
pub enum Subject {
    Value {
        shape: &'static Shape,
        position: Position,
    },
    /// Several values crossing together — the axis a per-value cell cannot
    /// express, because aliasing is a property of the call.
    Call(&'static Call),
    /// The same value, in the same position, declared as something else.
    Policy(&'static Policy),
}

impl Subject {
    /// The two parts of a cell id that come from the subject.
    fn id_parts(&self) -> (&'static str, String) {
        match self {
            Subject::Value { shape, position } => (shape.id, position.slug().to_string()),
            Subject::Call(call) => (call.id, "call".to_string()),
            Subject::Policy(policy) => (
                policy.shape,
                format!("{}_as_{}", policy.position.slug(), policy.class.slug()),
            ),
        }
    }
}

/// One row of the run: every cell, in corpus order.
pub struct Cell {
    subject: Subject,
    target: Target,
    state: State,
    /// Whether rustc accepted the emitted Rust. `None` when there was nothing
    /// to check — the generator refused the cell, or the check could not run at
    /// all, which is not a verdict about the cell either.
    compiled: Option<bool>,
    /// What cbindgen made of it. `None` for every JNI cell — that target's next
    /// stage is the Kotlin compiler, not a header — and for any cell whose Rust
    /// did not compile, since there would be nothing sound to hand on.
    header: Option<Header>,
    /// Whether the Kotlin compiler accepted this cell's emitted Kotlin. `None`
    /// for every C cell, for a cell that emitted none, and for a run where the
    /// stage could not start at all.
    kotlin: Option<bool>,
}

/// `<shape>__<position>__<target>` — the receipt key, the name of the file
/// rustc reports against, the directory the Kotlin compiler reports against,
/// and the Kotlin package a cell's own binding surface is emitted into.
///
/// A free function because the id is needed **before** a cell exists: the JNI
/// run is given it so the emitted Kotlin says which cell it belongs to, and one
/// formula in one place is what keeps that package and this key the same string.
pub fn id_of(subject: &Subject, target: Target) -> String {
    let (shape, kind) = subject.id_parts();
    format!("{shape}__{kind}__{}", target.slug())
}

impl Cell {
    /// This cell's [`id_of`].
    pub fn id(&self) -> String {
        id_of(&self.subject, self.target)
    }

    /// What the table prints: the furthest stage this cell reached.
    ///
    /// The ladders differ by target and say so, rather than being levelled to
    /// the shorter one: C runs one stage further than JNI does here, and
    /// printing `rustc` for a C cell whose header is fine would throw away the
    /// stronger evidence.
    fn text(&self) -> String {
        match (&self.state, self.compiled) {
            (State::PlanSupported, Some(true)) => match &self.header {
                None => match self.kotlin {
                    Some(true) => "kotlin".to_string(),
                    Some(false) => "**bad kotlin**".to_string(),
                    None => "rustc".to_string(),
                },
                Some(Header::Declared) => "header".to_string(),
                Some(Header::Missing) => "**no decl**".to_string(),
                Some(Header::Failed(_)) => "**bad header**".to_string(),
            },
            (State::PlanSupported, Some(false)) => "**bad rust**".to_string(),
            (state, _) => state.cell(),
        }
    }

    /// How far this cell got, on the ladder the guarantee ratchet compares.
    ///
    /// Derived from the same fields the table prints, so a cell cannot be
    /// guaranteed at a level the report does not show.
    pub fn level(&self) -> crate::guarantees::Level {
        use crate::guarantees::Level;
        match (&self.state, self.compiled) {
            (State::PlanSupported, Some(true)) => match &self.header {
                None => match self.kotlin {
                    Some(true) => Level::Kotlin,
                    // A Kotlin stage that ran and failed is *worse* than not
                    // having run, for the same reason a failed header stage is:
                    // the Rust compiles and the Kotlin caller still has nothing.
                    _ => Level::Compiles,
                },
                Some(h) if h.is_ok() => Level::Header,
                // A header stage that ran and failed is *worse* than not having
                // run: the Rust compiles and the C caller still gets nothing.
                Some(_) => Level::Compiles,
            },
            (State::PlanSupported, _) => Level::Generates,
            _ => Level::Nothing,
        }
    }

    /// The line the diagnostics list carries for this cell, if any.
    fn detail(&self) -> Option<String> {
        self.state
            .detail()
            .or_else(|| self.header.as_ref().and_then(Header::detail))
    }
}

fn run_all() -> Vec<Cell> {
    // A generator that panics is being asked a question it cannot answer; the
    // default hook would print a backtrace per cell and bury the report.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let mut cells = Vec::new();
    let mut units = Vec::new();
    let mut kotlin_units = Vec::new();
    let mut record =
        |subject: Subject, target: Target, outcome: crate::run::Outcome, fixture: String| {
            let cell = Cell {
                subject,
                target,
                state: outcome.state,
                compiled: None,
                header: None,
                kotlin: None,
            };
            if !outcome.kotlin.is_empty() {
                kotlin_units.push(crate::kotlin::Unit {
                    id: cell.id(),
                    files: outcome.kotlin,
                });
            }
            if let Some(emitted) = outcome.emitted {
                units.push(Unit {
                    id: cell.id(),
                    fixture,
                    emitted,
                });
            }
            cells.push(cell);
        };

    for shape in SHAPES {
        for position in Position::ALL {
            for target in Target::ALL {
                let subject = Subject::Value {
                    shape,
                    position: *position,
                };
                let outcome = run(shape, *position, *target, &id_of(&subject, *target));
                record(
                    subject,
                    *target,
                    outcome,
                    crate::run::fixture_source(shape, *position),
                );
            }
        }
    }
    for call in CALLS {
        for target in Target::ALL {
            let subject = Subject::Call(call);
            let outcome = run_call(call, *target, &id_of(&subject, *target));
            record(
                subject,
                *target,
                outcome,
                crate::run::call_fixture_source(call),
            );
        }
    }
    for policy in POLICIES {
        let shape = policy.shape();
        for target in Target::ALL {
            let subject = Subject::Policy(policy);
            let outcome = run_policy(
                shape,
                policy.position,
                policy.class,
                *target,
                &id_of(&subject, *target),
            );
            record(
                subject,
                *target,
                outcome,
                crate::run::fixture_source(shape, policy.position),
            );
        }
    }

    std::panic::set_hook(previous);

    // A check that could not run is reported as such and leaves every cell
    // uncompiled, rather than being folded into the cells as a verdict they did
    // not earn.
    match check::check("cells", &units) {
        Ok(checked) => {
            for cell in &mut cells {
                if matches!(cell.state, State::PlanSupported) {
                    cell.compiled = Some(checked.compiled.contains(&cell.id()));
                }
            }
            // Only what compiles goes on to cbindgen: a header derived from
            // Rust that does not build says nothing about the cell.
            for cell in &mut cells {
                if cell.target == Target::C && cell.compiled == Some(true) {
                    let emitted = units
                        .iter()
                        .find(|u| u.id == cell.id())
                        .map(|u| u.emitted.as_str())
                        .unwrap_or_default();
                    cell.header = Some(header::generate(emitted, crate::run::PROBE_FN));
                }
            }
            for (id, messages) in &checked.failed {
                eprintln!("shape-matrix: {id} emitted Rust that does not compile:");
                for message in messages {
                    eprintln!("    {message}");
                }
            }
        }
        Err(err) => eprintln!("shape-matrix: the compile check did not run: {err}"),
    }

    // The JNI target's own next stage. Only cells whose Rust compiles are asked:
    // the two halves are one binding, and Kotlin that compiles beside Rust that
    // does not is not a working cell.
    kotlin_units.retain(|unit| {
        cells
            .iter()
            .any(|cell| cell.compiled == Some(true) && cell.id() == unit.id)
    });
    match crate::kotlin::compile("cells", &kotlin_units) {
        Ok(compiled) => {
            for cell in &mut cells {
                let id = cell.id();
                if kotlin_units.iter().any(|unit| unit.id == id) {
                    cell.kotlin = Some(compiled.ok.contains(&id));
                }
            }
            for (id, messages) in &compiled.failed {
                eprintln!("shape-matrix: {id} emitted Kotlin that does not compile:");
                for message in messages {
                    eprintln!("    {message}");
                }
            }
        }
        Err(err) => eprintln!("shape-matrix: the Kotlin check did not run: {err}"),
    }

    cells
}

fn render_summary(out: &mut String, results: &[Cell]) {
    out.push_str("## Summary\n\n");
    out.push_str(
        "| Target | header | kotlin | rustc | bad header | bad kotlin | bad rust | rejected | \
         panic | n/a |\n",
    );
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for target in Target::ALL {
        let of = |f: fn(&Cell) -> bool| {
            results
                .iter()
                .filter(|c| c.target == *target && f(c))
                .count()
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            target.as_str(),
            of(|c| c.header.as_ref().is_some_and(Header::is_ok)),
            of(|c| c.kotlin == Some(true)),
            of(|c| c.compiled == Some(true) && c.header.is_none() && c.kotlin.is_none()),
            of(|c| c.header.as_ref().is_some_and(|h| !h.is_ok())),
            of(|c| c.kotlin == Some(false)),
            of(|c| c.compiled == Some(false)),
            of(|c| matches!(c.state, State::Rejected(_))),
            of(|c| matches!(c.state, State::Panicked(_))),
            of(|c| matches!(c.state, State::NotApplicable(_))),
        );
    }
    out.push_str(
        "\nThe two targets end at different stages, and each column belongs to one of \
         them: a C cell goes on to cbindgen and tops out at `header`, a Kotlin/JNI cell \
         goes on to the Kotlin compiler and tops out at `kotlin`. `rustc` is where a cell \
         stopped without reaching either — for JNI that means it emitted no Kotlin to \
         compile, and for C that its Rust did not reach the header stage.\n",
    );
    out.push('\n');
}

fn render_position(out: &mut String, results: &[Cell], position: Position) {
    let _ = writeln!(out, "## Position: {}\n", position.as_str());
    out.push_str("| Shape | Rust | C | Kotlin/JNI |\n|---|---|---|---|\n");

    for shape in SHAPES {
        let state_of = |target: Target| {
            results
                .iter()
                .find(|c| match c.subject {
                    Subject::Value {
                        shape: s,
                        position: p,
                    } => std::ptr::eq(s, shape) && p == position && c.target == target,
                    Subject::Call(_) | Subject::Policy(_) => false,
                })
                .map(|c| c.text())
                .unwrap_or_default()
        };
        let _ = writeln!(
            out,
            "| `{}` | `{}` | {} | {} |",
            shape.id,
            shape.spelling,
            state_of(Target::C),
            state_of(Target::Jni),
        );
    }
    out.push('\n');

    let mut notes: Vec<String> = Vec::new();
    for cell in results
        .iter()
        .filter(|c| matches!(c.subject, Subject::Value { position: p, .. } if p == position))
    {
        if let Some(detail) = cell.detail() {
            notes.push(format!(
                "- `{}` / {}: {}",
                cell.subject.id_parts().0,
                cell.target.as_str(),
                detail
            ));
        }
    }
    if !notes.is_empty() {
        out.push_str("<details><summary>What the generators said</summary>\n\n");
        for note in notes {
            let _ = writeln!(out, "{note}");
        }
        out.push_str("\n</details>\n\n");
    }
}

fn render_calls(out: &mut String, results: &[Cell]) {
    out.push_str("## Calls\n\n");
    out.push_str(
        "Several values crossing together. Aliasing is a property of a **call** — two \
         parameters can name the same resource — so it cannot be stated about either \
         value on its own, and both generators emit a preflight under a rule about the \
         whole parameter set.\n\nWhat this table says is whether such a call can be \
         expressed. Whether the guard *fires* — rejecting the aliased call, sparing the \
         unaliased one, running before ownership moves — is a claim about running code, \
         and no cell here claims it.\n\n",
    );
    out.push_str("| Call | Parameters | C | Kotlin/JNI |\n|---|---|---|---|\n");
    for call in CALLS {
        let state_of = |target: Target| {
            results
                .iter()
                .find(|c| match c.subject {
                    Subject::Call(k) => std::ptr::eq(k, call) && c.target == target,
                    Subject::Value { .. } | Subject::Policy(_) => false,
                })
                .map(|c| c.text())
                .unwrap_or_default()
        };
        let _ = writeln!(
            out,
            "| `{}` | `({})` | {} | {} |",
            call.id,
            call.params.join(", "),
            state_of(Target::C),
            state_of(Target::Jni),
        );
    }
    out.push('\n');

    let mut notes: Vec<String> = Vec::new();
    for cell in results
        .iter()
        .filter(|c| matches!(c.subject, Subject::Call(_)))
    {
        if let Some(detail) = cell.detail() {
            notes.push(format!(
                "- `{}` / {}: {}",
                cell.subject.id_parts().0,
                cell.target.as_str(),
                detail
            ));
        }
    }
    if !notes.is_empty() {
        out.push_str("<details><summary>What the generators said</summary>\n\n");
        for note in notes {
            let _ = writeln!(out, "{note}");
        }
        out.push_str("\n</details>\n\n");
    }
}

fn render_runtime(out: &mut String) {
    out.push_str("## Runtime\n\n");
    out.push_str(
        "The claims the call axis could not make. Every stage above asks whether \
         something could be *produced*; these ask what the produced code **does**, by \
         calling it — the C target's `extern \"C\"` wrappers are ordinary Rust \
         functions, so this needs no C toolchain and no JVM. A case counts as holding \
         only when its generated test executed and passed.\n\nThey are a discrimination \
         set, not a wish list: a guard that fires on everything passes the first and \
         fails the second, and one that fires on nothing does the reverse.\n\n",
    );
    out.push_str("| Case | Proves | Outcome |\n|---|---|---|\n");

    let outcomes = match crate::runtime::exercise() {
        Ok(outcomes) => outcomes,
        Err(err) => {
            eprintln!("shape-matrix: the runtime stage did not run: {err}");
            Default::default()
        }
    };
    for case in crate::runtime::CASES {
        let outcome = outcomes
            .get(case.id)
            .copied()
            .unwrap_or(crate::runtime::Outcome::NotRun);
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            case.id,
            case.proves,
            outcome.cell()
        );
    }
    out.push_str(
        "\nKotlin/JNI has no equivalent here: its wrappers are entered from a JVM, and \
         reaching them needs one.\n\n",
    );
}

fn render_policies(out: &mut String, results: &[Cell]) {
    out.push_str("## Declaration policy\n\n");
    out.push_str(
        "The same Rust, in the same position, with its declared type declared as \
         something else. A binding author chooses this — a struct can cross by value or \
         as an opaque handle — and while every type is declared exactly one way, whether \
         the choice decides the answer is invisible.\n\nEach row shows the canonical \
         answer beside the varied one, because the difference is the point.\n\n",
    );
    out.push_str(
        "| Shape | Position | Declared as | C | C canonical | Kotlin/JNI | JNI canonical |\n|---|---|---|---|---|---|---|\n",
    );

    for policy in POLICIES {
        let varied = |target: Target| {
            results
                .iter()
                .find(|c| match c.subject {
                    Subject::Policy(p) => std::ptr::eq(p, policy) && c.target == target,
                    _ => false,
                })
                .map(|c| c.text())
                .unwrap_or_default()
        };
        let canonical = |target: Target| {
            results
                .iter()
                .find(|c| match c.subject {
                    Subject::Value { shape, position } => {
                        shape.id == policy.shape
                            && position == policy.position
                            && c.target == target
                    }
                    _ => false,
                })
                .map(|c| c.text())
                .unwrap_or_default()
        };
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            policy.shape,
            policy.position.as_str(),
            policy.class.as_str(),
            varied(Target::C),
            canonical(Target::C),
            varied(Target::Jni),
            canonical(Target::Jni),
        );
    }
    out.push('\n');

    let mut notes: Vec<String> = Vec::new();
    for cell in results
        .iter()
        .filter(|c| matches!(c.subject, Subject::Policy(_)))
    {
        if let Some(detail) = cell.detail() {
            notes.push(format!(
                "- `{}` / {}: {}",
                cell.id(),
                cell.target.as_str(),
                detail
            ));
        }
    }
    if !notes.is_empty() {
        out.push_str("<details><summary>What the generators said</summary>\n\n");
        for note in notes {
            let _ = writeln!(out, "{note}");
        }
        out.push_str("\n</details>\n\n");
    }
}

fn render_coverage(out: &mut String) {
    out.push_str("## Type-form coverage\n\n");
    out.push_str(
        "Every form `TypeKind` accepts, and the shapes that write one. Enforced by \
         `every_type_form_is_covered`; a new form cannot be added to the model without \
         breaking `tag_of` first.\n\n",
    );
    out.push_str("| Form | Covered by |\n|---|---|\n");

    let classified = crate::classify_corpus();
    for tag in TypeTag::ALL {
        let mut covering: Vec<&str> = classified
            .iter()
            .filter(|(_, tags)| tags.contains(tag))
            .map(|(shape, _)| shape.id)
            .collect();
        covering.sort_unstable();
        let _ = writeln!(
            out,
            "| `{}` | {} |",
            tag.as_str(),
            if covering.is_empty() {
                "**nothing**".to_string()
            } else {
                covering
                    .iter()
                    .map(|id| format!("`{id}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
    }
    out.push('\n');
}

fn render_class_coverage(out: &mut String) {
    out.push_str("## Declaration-kind coverage\n\n");
    out.push_str(
        "What a declared type can be declared **as**, and the positions that exercise each. \
         The vocabulary is the JNI adapter\'s own `ClassDecl`, matched exhaustively by \
         `kind_of`, so a fifth class kind stops this crate compiling; the C side is a \
         translation of the same axis, since its build-script API has no closed kind \
         vocabulary yet.\n\n",
    );
    out.push_str("| Declared as | Exercised by |\n|---|---|\n");

    for kind in ClassKind::ALL {
        let mut users: Vec<String> = Vec::new();
        for shape in SHAPES {
            for position in Position::ALL {
                let used = declarations(shape, *position)
                    .iter()
                    .any(|d| d.class == *kind);
                if used {
                    users.push(format!("`{}` ({})", shape.id, position.as_str()));
                }
            }
        }
        let summary = if users.is_empty() {
            "**nothing**".to_string()
        } else {
            format!("{} cells, e.g. {}", users.len(), users[0])
        };
        let _ = writeln!(out, "| {} | {} |", kind.as_str(), summary);
    }
    out.push('\n');
}

const HEADER: &str = "\
<!-- Generated by `cargo run -p shape-matrix`. Do not edit by hand. -->
# Shape matrix

Which Rust shapes cross the boundary, in which position, for each target
language — enumerated by running the real generators over synthesized fixtures,
never by a hand-written list of what is supposed to work. See
[#198](https://github.com/milyin/prebindgen/issues/198).

**How to read a cell**

| Cell | Meaning |
|---|---|
| `header` | C only: rustc accepted the Rust **and cbindgen declared the wrapper in a header**. The furthest any cell gets today. |
| `kotlin` | JNI only: rustc accepted the Rust **and the Kotlin compiler accepted the Kotlin**. The top of that target's ladder, and the counterpart of `header` — what a Kotlin caller is given is classes, not a header. |
| `rustc` | the generator produced Rust and rustc accepted it, and the cell stopped there: it reached neither its target's last stage nor a failure in it. |
| **`bad kotlin`** | the generator produced Kotlin the Kotlin compiler refused, beside Rust that compiles. Worse than not having run the stage, exactly as **`bad header`** is on the C side. |
| **`no decl`** | cbindgen produced a header that does not declare the wrapper — nothing a C program can call. |
| **`bad header`** | cbindgen refused the emitted Rust, or panicked on it. |
| **`bad rust`** | the generator produced Rust that does not compile. Green unit tests can coexist with this — that is why the check exists. It also covers a refusal the generator *chose* to make at compile time, through a generated assertion with its own message: same user-visible outcome, and a refusal arriving as a compile error rather than as a named rejection at declaration time is [#191](https://github.com/milyin/prebindgen/issues/191)'s subject. The run's stderr distinguishes them. |
| `plan` | generation succeeded and the compile check did not run (see the run\'s stderr). |
| `rejected` | the generator refused the shape **and said why**. The intended outcome for anything unsupported. |
| **`panic`** | the generator refused it without a diagnosis — the user gets a stack trace instead of a sentence ([#191](https://github.com/milyin/prebindgen/issues/191)). |
| `—` | the placement is not legal Rust, so there is nothing to ask. |

A compiler is evidence, not a guarantee. `rustc` says the emitted Rust
type-checks against the fixture the way a binding crate compiles it, and
`kotlin` says the emitted classes are a program; neither says the two halves
agree at runtime about a signature, which is what a JVM would answer and what no
cell here asks yet.

Compiler messages are deliberately **not** in this file — they vary by
toolchain, and the report has to be identical on every one that builds it. A
failing cell prints its diagnostics on the run\'s stderr.

";
