//! Does the generated code *behave*?
//!
//! Every stage so far asked whether something could be produced: a plan, Rust
//! that compiles, a header that declares. None of them can answer the questions
//! the call axis raised — whether the alias guard **fires** on a call naming one
//! resource twice, whether it **spares** a call that does not, and whether it
//! runs *before* ownership moves. Those are claims about running code, and the
//! only way to make one is to run the code.
//!
//! # Why this needs no toolchain
//!
//! The C target's `extern "C"` wrappers are ordinary Rust functions. A Rust test
//! can call one with the pointers a C caller would pass, so the whole C runtime
//! stage is a `cargo test` — no C compiler, no linker, no CI change. The JNI
//! side has no such shortcut: its wrappers are entered from a JVM, and reaching
//! them needs a JVM.
//!
//! # The receipt
//!
//! A case passes only when the generated test **executed and passed**, read from
//! the test harness's own per-test result line. A case that fails to compile,
//! never runs, or is filtered out is not a pass, and there is no table anywhere
//! mapping a case to "presumed fine".

use std::{collections::BTreeMap, path::Path, process::Command};

use crate::{
    check,
    corpus::CALLS,
    run::{run_call, Target},
};

/// One behavioural claim, and the code that establishes it.
pub struct Case {
    /// Stable id — the test's name, and the receipt key.
    pub id: &'static str,
    /// The call shape whose binding the case exercises.
    pub call: &'static str,
    /// What passing proves, in the report's words.
    pub proves: &'static str,
    /// The body of the generated `#[test]`.
    pub body: &'static str,
}

/// The cases.
///
/// Deliberately a discrimination *set* rather than a list of things that should
/// work: a guard that fires on everything passes the first case and fails the
/// second, and one that fires on nothing does the reverse. Neither can pass both.
pub const CASES: &[Case] = &[
    Case {
        id: "aliased_consume_is_rejected",
        call: "consume_consume_fallible",
        proves: "a call naming one resource twice is rejected, and neither conversion runs",
        body: r#"
        // One allocation, handed to the call twice — `z_combine(x, x)`.
        let owned = Box::into_raw(Box::new(flat::Handle { id: 7 })) as *mut handle;
        let mut error: *mut ::core::ffi::c_char = std::ptr::null_mut();
        let ok = unsafe { probe(owned, owned, &mut error) };

        assert!(!ok, "the aliased call was accepted");
        assert!(!error.is_null(), "the call was rejected without saying why");
        let message = unsafe { std::ffi::CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();
        assert!(
            message.contains("aliasing"),
            "rejected, but not as an alias: {message}"
        );

        // The resource was never taken: reclaiming it here would be a double
        // free if either converter had run. That is what makes this a statement
        // about ordering, and not merely about failing.
        unsafe { drop(Box::from_raw(owned as *mut flat::Handle)) };
    "#,
    },
    Case {
        id: "distinct_resources_are_spared",
        call: "consume_consume_fallible",
        proves: "two distinct resources in the same call are not flagged",
        body: r#"
        // The false-positive check. A guard that rejected this would remove
        // working surface, which is worse than the defect it prevents.
        let first = Box::into_raw(Box::new(flat::Handle { id: 1 })) as *mut handle;
        let second = Box::into_raw(Box::new(flat::Handle { id: 2 })) as *mut handle;
        let mut error: *mut ::core::ffi::c_char = std::ptr::null_mut();
        let ok = unsafe { probe(first, second, &mut error) };
        assert!(ok, "two distinct resources were rejected as aliases");
        assert!(error.is_null(), "an accepted call reported an error");
        // Both were consumed by the call; nothing to reclaim.
    "#,
    },
    Case {
        id: "shared_borrows_of_one_resource_are_legal",
        call: "borrow_borrow_fallible",
        proves: "two shared borrows of one resource are accepted — legal Rust, legal C",
        body: r#"
        // Aliasing is only a defect where something is consumed or exclusively
        // borrowed. Two `&T` to one resource is neither, and a guard here would
        // remove working surface.
        let owned = Box::into_raw(Box::new(flat::Handle { id: 3 }));
        let mut error: *mut ::core::ffi::c_char = std::ptr::null_mut();
        let ok = unsafe {
            probe(owned as *const handle, owned as *const handle, &mut error)
        };
        assert!(ok, "two shared borrows of one resource were rejected");
        assert!(error.is_null(), "an accepted call reported an error");
        unsafe { drop(Box::from_raw(owned)) };
    "#,
    },
];

/// What became of one case.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The test ran and passed.
    Passed,
    /// The test ran and failed — the claim is false.
    Failed,
    /// The test never ran: the binding did not generate, or the crate did not
    /// build. Not a verdict on the claim.
    NotRun,
}

impl Outcome {
    pub fn cell(self) -> &'static str {
        match self {
            Outcome::Passed => "holds",
            Outcome::Failed => "**fails**",
            Outcome::NotRun => "not run",
        }
    }
}

/// Build the cases into a crate and run them.
///
/// `Err` means the stage could not run at all, which is not a verdict about any
/// case — the caller reports every case as `NotRun` rather than inventing one.
pub fn exercise() -> Result<BTreeMap<String, Outcome>, String> {
    let root = check::crate_dir()?.join("runtime");
    // Each case is an **integration test**, which cargo compiles as its own
    // crate. That is not a stylistic choice: every case exports the same
    // `#[no_mangle]` symbols (`probe`, `handle_drop`), and one crate holding
    // two of them does not link. The compile stage never hit this because
    // `cargo check` does not generate code.
    let tests = root.join("tests");
    let src = root.join("src");
    std::fs::create_dir_all(&tests).map_err(|e| format!("creating {}: {e}", tests.display()))?;
    std::fs::create_dir_all(&src).map_err(|e| format!("creating {}: {e}", src.display()))?;

    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().and_then(Path::parent).expect("workspace");
    check::write(
        &root.join("Cargo.toml"),
        &check::manifest_toml("shape-matrix-runtime", &workspace.display().to_string()),
    )?;

    check::write(
        &src.join("lib.rs"),
        "// Generated by shape-matrix. Do not edit.\n\
         //\n\
         // Empty on purpose: every case lives in `tests/`, so each gets its own\n\
         // crate and its own copy of the exported symbols.\n",
    )?;

    let mut outcomes: BTreeMap<String, Outcome> = BTreeMap::new();
    for case in CASES {
        outcomes.insert(case.id.to_string(), Outcome::NotRun);

        let call = CALLS
            .iter()
            .find(|c| c.id == case.call)
            .unwrap_or_else(|| panic!("runtime case names unknown call `{}`", case.call));
        // Only the C target: a JNI wrapper is entered from a JVM, and this
        // stage's whole premise is that C's is not.
        let Some(emitted) = run_call(call, Target::C).emitted else {
            continue;
        };

        check::write(
            &tests.join(format!("{}.rs", case.id)),
            &case_source(case, &crate::run::call_fixture_source(call), &emitted),
        )?;
    }

    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .arg("test")
        // Not `--quiet`: the per-test result lines *are* the receipts, and the
        // quiet formatter prints dots instead.
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(root.join("target"))
        .output()
        .map_err(|e| format!("running cargo test: {e}"))?;

    let report = String::from_utf8_lossy(&output.stdout);
    for line in report.lines() {
        // `test case_0::aliased_consume_is_rejected ... ok`
        let Some((name, verdict)) = line
            .strip_prefix("test ")
            .and_then(|l| l.split_once(" ... "))
        else {
            continue;
        };
        let Some(id) = name.rsplit("::").next() else {
            continue;
        };
        if let Some(outcome) = outcomes.get_mut(id) {
            *outcome = if verdict.trim() == "ok" {
                Outcome::Passed
            } else {
                Outcome::Failed
            };
        }
    }

    if outcomes.values().all(|o| *o == Outcome::NotRun) {
        return Err(format!(
            "no case reported a result:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(outcomes)
}

/// One case as a Rust module: the source crate, the generated binding, and the
/// test that calls it the way a C caller would.
fn case_source(case: &Case, fixture: &str, emitted: &str) -> String {
    let unit = check::Unit {
        id: case.id.to_string(),
        fixture: fixture.to_string(),
        emitted: emitted.to_string(),
    };
    format!(
        "{}\n#[test]\nfn {}() {{{}}}\n",
        check::cell_source(&unit),
        case.id,
        case.body
    )
}
