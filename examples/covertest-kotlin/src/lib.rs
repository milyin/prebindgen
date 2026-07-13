// This crate is (almost) entirely machine-generated code; clippy findings in it
// belong to the generator, not to this file.
#![allow(clippy::all)]

// Binding-local conversion fns for `Label` — referenced by build.rs as
// `.convert(convert!(Label).input_with(ty!(String), path!(crate::label_in))…)`.
// NOT `#[prebindgen]`-marked: the generated file compiles inside this crate,
// so plain `crate::` paths resolve; no helper crate needed for infallible
// by-value conversions.
pub fn label_in(s: String) -> perftest_flat::Label {
    perftest_flat::Label(s)
}
pub fn label_out(l: perftest_flat::Label) -> String {
    l.0
}

// The generated JNI bindings, written by build.rs from perftest-flat's
// #[prebindgen] surface (the perf surface plus the `ext` coverage surface). The
// generated code refers to source types fully qualified by each item's origin
// crate (e.g. `perftest_flat::Payload`), so no extra `use` is needed.
include!("generated_bindings.rs");
