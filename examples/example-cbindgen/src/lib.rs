// This crate is entirely machine-generated code; clippy findings in it belong
// to the generator, not to this file.
#![allow(clippy::all)]

// The generated example-flat FFI bindings, committed under generated/ and
// (re)produced by build.rs from example-flat's #[prebindgen] surface.
//
// One file per VARIANT — target architecture and enabled features — because
// `#[prebindgen]` cfg handling makes the generated code differ by both (`Foo`'s
// fields and `InsideFoo`'s discriminants change with `target_arch`, and
// `calculator_reset` only exists under `unstable`). Build for both x86_64 and
// aarch64 (see CMakeLists.txt), with and without features, to generate them all
// and compare.
//
// The path comes from build.rs (`cargo:rustc-env`) rather than from a `cfg`
// matrix here: the build script already knows which variant it just wrote, and a
// matrix would need re-teaching for every new arch × feature combination.
include!(env!("EXAMPLE_FLAT_BINDINGS"));

// Hand-written tests calling the generated C ABI the way a C caller would —
// including with an enum discriminant no variant has.
#[cfg(test)]
mod boundary_tests;

// Convenient alternative when you DON'T want to commit generated files to git:
// build.rs always also writes the current target's bindings to OUT_DIR under a
// stable name, so this single line works for any target (the file just isn't kept
// in the repo). Replace the `include!` above with:
//
// include!(concat!(env!("OUT_DIR"), "/example_flat.rs"));
