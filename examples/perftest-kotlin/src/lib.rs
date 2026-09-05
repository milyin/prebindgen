// This crate is entirely machine-generated code; clippy findings in it belong
// to the generator, not to this file.
#![allow(clippy::all)]

// The generated JNI bindings, written by build.rs from perftest-flat's
// #[prebindgen] surface. The generated code refers to source types fully
// qualified through the `source_module` (e.g. `perftest_flat::Payload`), so no
// extra `use` is needed here.
// The path is chosen by build.rs, so the engine that generated the file is
// the one whose file is compiled.
include!(env!("PERFTEST_BINDINGS"));
