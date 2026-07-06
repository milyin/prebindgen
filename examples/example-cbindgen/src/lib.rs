// This crate is entirely machine-generated code; clippy findings in it belong
// to the generator, not to this file.
#![allow(clippy::all)]

// The generated example-flat FFI bindings, committed under generated/ and
// (re)produced by build.rs from example-flat's #[prebindgen] surface.
//
// The file name is per target architecture: `#[prebindgen]` cfg handling makes the
// generated code differ per target (`Foo`'s fields and `InsideFoo`'s discriminants
// change with `target_arch`), so each target includes its own file. Build for both
// x86_64 and aarch64 (see CMakeLists.txt) to generate both and compare.
#[cfg(target_arch = "x86_64")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/example_flat_x86_64.rs"
));
#[cfg(target_arch = "aarch64")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/example_flat_aarch64.rs"
));

// Convenient alternative when you DON'T want to commit generated files to git:
// build.rs always also writes the current target's bindings to OUT_DIR under a
// stable name, so this single line works for any target (the file just isn't kept
// in the repo). Replace the per-target `include!`s above with:
//
// include!(concat!(env!("OUT_DIR"), "/example_flat.rs"));
