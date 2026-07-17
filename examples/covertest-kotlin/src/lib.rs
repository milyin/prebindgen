// This crate is (almost) entirely machine-generated code; clippy findings in it
// belong to the generator, not to this file.
#![allow(clippy::all)]

// Binding-local conversion fns for `Label` — declared in build.rs via the
// one binding-local vocabulary, `.convert(convert!(Label).input(fun!(crate::
// label_in).sig(…))…)`. NOT `#[prebindgen]`-marked: the generated file
// compiles inside this crate, so plain `crate::` paths resolve; no helper
// crate needed. The input is FALLIBLE — the sig's `Result<Label, String>`
// return is the error channel.
pub fn label_in(s: String) -> Result<perftest_flat::Label, String> {
    if s.is_empty() {
        Err("label must not be empty".to_string())
    } else {
        Ok(perftest_flat::Label(s))
    }
}
pub fn label_out(l: perftest_flat::Label) -> String {
    l.0
}

// Binding-local FUNCTIONS (`fun!(crate::…).sig(sig!(…))`): full fns defined
// in THIS crate and exported through the ordinary FunctionDecl surface —
// free package fn, instance method, companion constructor. No source-crate
// item exists for any of them. `summary_from_mean` is FALLIBLE: the sig's
// `Result<Summary, String>` return routes the Err to onError.
pub(crate) fn summary_describe(s: &perftest_flat::Summary, verbose: bool) -> String {
    let count = perftest_flat::summary_count(s);
    let total = perftest_flat::summary_total(s);
    if verbose {
        format!("summary of {count} payloads totalling {total}")
    } else {
        format!("{count}/{total}")
    }
}

pub(crate) fn summary_mean(s: &perftest_flat::Summary) -> f64 {
    let count = perftest_flat::summary_count(s);
    if count == 0 {
        0.0
    } else {
        perftest_flat::summary_total(s) / count as f64
    }
}

pub(crate) fn summary_from_mean(count: i64, mean: f64) -> Result<perftest_flat::Summary, String> {
    if count < 0 {
        Err(format!("summary count must be non-negative, got {count}"))
    } else {
        Ok(perftest_flat::summary_new(count, mean * count as f64))
    }
}

// Binding-local CONDITIONAL output field (`expand_return!(Summary)` per-fn
// set of `storage_summary_probe`: `.field(field!("handle").with(ty!(Option<&
// Summary>), path!(crate::summary_if_nonempty)))`): deliver the handle leaf
// only when the summary is non-empty — binding policy with no place in the
// source crate (the zenoh "Encoding handle only when schema-carrying" idiom).
pub(crate) fn summary_if_nonempty(s: &perftest_flat::Summary) -> Option<&perftest_flat::Summary> {
    (perftest_flat::summary_count(s) > 0).then_some(s)
}

// Binding-local nullary fn backing the `.with`-sourced constant
// `COVER_VERSION` (build.rs: `constant!(COVER_VERSION).with(ty!(String),
// path!(crate::cover_version))`).
pub fn cover_version() -> String {
    format!("cover-{}", env!("CARGO_PKG_VERSION"))
}

// The generated JNI bindings, written by build.rs from perftest-flat's
// #[prebindgen] surface (the perf surface plus the `ext` coverage surface). The
// generated code refers to source types fully qualified by each item's origin
// crate (e.g. `perftest_flat::Payload`), so no extra `use` is needed.
include!("generated_bindings.rs");
