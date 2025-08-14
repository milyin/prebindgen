# prebindgen

A tool for separating the implementation of FFI interfaces from language-specific binding generation, allowing each to reside in different crates.

## Problem

When creating Rust libraries that need to expose FFI interfaces to multiple languages, it may be preferable to create separate `cdylib` or `staticlib` crates for each language-specific binding. This allows you to tailor each crate to the requirements and quirks of its binding generator and to the specifics of the destination language.

However, `#[no_mangle] extern "C"` functions can only be defined in `cdylib` or `staticlib` crates and cannot be exported from `lib` crates. As a result, these functions must be duplicated in each language-specific binding crate. This duplication becomes cumbersome for large projects with many FFI functions and types.

There is a discussion about this problem in issue [2771](https://github.com/rust-lang/rfcs/issues/2771).

## Solution

`prebindgen` solves this by generating `#[no_mangle] extern "C"` functions for each language binding, proxying a common Rust library crate.

It also allows you to convert and analyze the source to adapt the result for the specific binding generator and/or for collecting data necessary for postprocessing the generated language binding.

The `prebindgen` tool consists of two crates: `prebindgen-proc-macro`, which provides macros for copying code fragments from the source crate, and `prebindgen`, which converts these fragments into an FFI source file.

### Architecture

The dependent crate's `build.rs` accesses the prebindgen output directory path through a constant exported from the source crate, rather than using Cargo's `DEP_` environment variables. This approach avoids race conditions that can occur because data collection happens during package compilation (after `build.rs`), while the dependent package's `build.rs` may start before the source package compilation completes. In cross-compilation scenarios, this race condition persists even if soource crate is added to build-dependencies, as the `DEP_` variable contains the path related to the target platform while build-dependencies is for the host platform.

## Usage

### 1. In the Common FFI Library Crate (e.g., `example-ffi`)

Mark structures and functions that are part of the FFI interface with the `prebindgen` macro and export the prebindgen output directory path:

```rust
// example-ffi/src/lib.rs
use prebindgen_proc_macro::prebindgen;

// Export path to prebindgen output directory
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_proc_macro::prebindgen_out_dir!();

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

Call `init_prebindgen_out_dir()` in the source crate's `build.rs`:

```rust
// example-ffi/build.rs
fn main() {
    prebindgen::init_prebindgen_out_dir();
}
```

### 2. In the Language-Specific FFI Binding Crate (e.g., `example-cbindgen`)

Add the source FFI library to both dependencies and build-dependencies:

```toml
# example-cbindgen/Cargo.toml
[dependencies]
example_ffi = { path = "../example_ffi" }

[build-dependencies]
example_ffi = { path = "../example_ffi" }
prebindgen = "0.4"
cbindgen = "0.29"
itertools = "0.14"
```

Convert `#prebindgen`-marked pieces to an FFI-compatible API (`repr(C)` structures, `extern "C"` functions, constants). Items not valid for FFI will be rejected by `FfiConverter`.

Generate target language bindings based on this source.

If necessary, custom filters can be applied.

```rust
// example-cbindgen/build.rs
use itertools::Itertools;

fn main() {
    // Create a source from the common FFI crate's prebindgen data
    let source = prebindgen::Source::new(example_ffi::PREBINDGEN_OUT_DIR);

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

Include the generated Rust file in your project to build the static or dynamic FFI-compatible library:

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
