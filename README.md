# prebindgen

A tool for separating the implementation of FFI interfaces from language-specific binding generation, allowing each to reside in different crates.

## Problem

When creating Rust libraries that need to expose FFI interfaces to multiple languages, it may be preferable to create separate `cdylib` or `staticlib` crates for each language-specific binding. This allows you to tailor each crate to the requirements and quirks of its binding generator and to the specifics of the destination language.

However, `#[no_mangle] extern "C"` functions can only be defined in `cdylib` or `staticlib` crates and cannot be exported from `lib` crates. As a result, these functions must be duplicated in each language-specific binding crate. This duplication becomes cumbersome for large projects with many FFI functions and types.

There is a discussion about this problem in issue [2771](https://github.com/rust-lang/rfcs/issues/2771).

## Solution

`prebindgen` solves this by generating `#[no_mangle] extern "C"` Rust proxy source code from a common Rust library crate. Language-specific binding crates can then compile this generated code and pass it to their respective binding generators (such as cbindgen, csbindgen, etc.).

The `prebindgen` tool consists of two crates: `prebindgen-proc-macro`, which provides macros for copying code fragments from the source crate, and `prebindgen`, which converts these fragments into a source file that can be included in the destination crate and processed by binding generators.

## Usage

### 1. In the Common FFI Library Crate (e.g., `example-ffi`)

Add `links` field to Cargo.topl to enable passing data to downstream crates via environment variables

```toml
# example-ffi/Cargo.toml
[package]
name = "example-ffi"
build = "build.rs"
links = "example_ffi" # Required for DEP_<crate_name>_PREBINDGEN variables to work
```

Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:

```rust
// example-ffi/src/lib.rs
use prebindgen_proc_macro::prebindgen;

// Group structures and functions for selective handling
#[prebindgen]
#[repr(C)]
pub struct MyStruct {
    pub field: i32,
}

#[prebindgen]
pub fn my_function(arg: i32) -> i32 {
    arg * 2
}
```

Call `init_prebindgen_out_dir()` in the source crate's `build.rs` to make `#prebindgen`-marked items available to the `prebindgen::Source` object in dependent crates' `build.rs`. This function calls `println!("cargo:prebindgen=<path>")`, which provides the path to the prebindgen output directory to the `build.rs` of dependent crates via the `DEP_<crate_name>_PREBINDGEN` variable.

```rust
// example-ffi/build.rs
fn main() {
    prebindgen::init_prebindgen_out_dir();
}
```

### 2. In the Language-Specific FFI Binding Crate (e.g., `example-cbindgen`)

Add the common FFI library to build dependencies:

```toml
# example-cbindgen/Cargo.toml
[build-dependencies]
example_ffi = { path = "../example_ffi" }
prebindgen = "0.4"
cbindgen = "0.29"
itertools = "0.14"
```

```rust
// example-cbindgen/build.rs
use itertools::Itertools;

fn main() {
    // Create a source from the common FFI crate's prebindgen data
    let source = prebindgen::Source::new("example_ffi");

    // Create feature filter
    let feature_filter = prebindgen::filter_map::FeatureFilter::builder()
        .disable_feature("unstable")
        .disable_feature("internal")
        .build();

    // Create converter with transparent wrapper stripping
    let converter = prebindgen::batching::FfiConverter::builder(source.crate_name())
        .edition(prebindgen::RustEdition::Edition2024)
        .strip_transparent_wrapper("std::mem::MaybeUninit")
        .strip_transparent_wrapper("std::option::Option")
        .prefixed_exported_type("foo::Foo")
        .build();

    // Process items with filtering and conversion
    let bindings_file = source
        .items_all()
        .filter_map(feature_filter.into_closure())
        .batching(converter.into_closure())
        .collect::<prebindgen::collect::Destination>()
        .write("ffi_bindings.rs");

    // Pass the generated file to cbindgen for C header generation
    generate_c_headers(&bindings_file);
}
```

Include the generated Rust file in your project to build the static or dynamic library:

```rust
// lib.rs
include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
```

## Examples

See example projects in the [examples directory](https://github.com/milyin/prebindgen/tree/main/examples):

- **example-ffi**: Common FFI library demonstrating prebindgen usage
- **example-cbindgen**: Language-specific binding using cbindgen for C headers

## Documentation

- **prebindgen API Reference**: [docs.rs/prebindgen](https://docs.rs/prebindgen)
- **prebindgen-proc-macro API Reference**: [docs.rs/prebindgen-proc-macro](https://docs.rs/prebindgen-proc-macro)
