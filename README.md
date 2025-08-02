# prebindgen

A tool for separating the implementation of FFI interfaces from language-specific binding generation, allowing each to reside in different crates.

## Problem

When creating Rust libraries that need to expose FFI interfaces to multiple languages, it may be preferable to create separate `cdylib` or `staticlib` crates for each language-specific binding. This allows you to tailor each crate to the requirements and quirks of its binding generator and to the specifics of the destination language.

There is a discussion about this problem in issue [2771](https://github.com/rust-lang/rfcs/issues/2771).

However, `#[no_mangle] extern "C"` functions can only be defined in a `cdylib` or `staticlib` crate, and cannot be exported from a `lib` crate. As a result, these functions must be duplicated in each language-specific binding crate. This duplication is inconvenient for large projects with many FFI functions and types.

## Solution

`prebindgen` solves this by generating `#[no_mangle] extern "C"` Rust proxy source code from a common Rust library crate. Language-specific binding crates can then compile this generated code and pass it to their respective binding generators (such as cbindgen, csbindgen, etc.).

The `prebindgen` tool consists of two crates: `prebindgen-proc-macro`, which provides macros for copying fragments of code from the source crate, and `prebindgen`, which converts these fragments into a source file that can be included in the destination crate and processed by binding generators.

## Usage

### 1. In the Common FFI Library Crate (e.g., `example_ffi`)

Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:

```rust
// example-ffi/src/lib.rs
use prebindgen_proc_macro::{prebindgen, prebindgen_out_dir};

// Declare a public constant with the path to prebindgen data:
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();

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

Call `init_prebindgen_out_dir()` in the crate's `build.rs`:

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
prebindgen = "0.2"
cbindgen = "0.24"
```

```rust
// example-cbindgen/build.rs
use prebindgen::{Source, batching::ffi_converter, filter_map::feature_filter, collect::Destination};
use itertools::Itertools;

fn main() {
    // Create a source from the common FFI crate's prebindgen data
    let source = Source::new(my_common_ffi::PREBINDGEN_OUT_DIR);

    // Process items with filtering and conversion
    let destination = source
        .items_all()
        .filter_map(feature_filter::Builder::new()
            .disable_feature("experimental")
            .enable_feature("std")
            .build()
            .into_closure())
        .batching(ffi_converter::Builder::new(source.crate_name())
            .edition("2024")
            .strip_transparent_wrapper("std::mem::MaybeUninit")
            .build()
            .into_closure())
        .collect::<Destination>();

    // Write generated FFI code to file
    let bindings_file = destination.write("ffi_bindings.rs");

    // Pass the generated file to cbindgen for C header generation
    generate_c_headers(&bindings_file);
}
```

Include the generated Rust files in your project to build the static or dynamic library itself:

```rust
// lib.rs
include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
```

## Examples

See example projects at https://github.com/milyin/prebindgen/tree/main/examples

- **example-ffi**: Common FFI library demonstrating prebindgen usage
- **example-cbindgen**: Language-specific binding using cbindgen for C headers

## Documentation

- **prebindgen API Reference**: [docs.rs/prebindgen](https://docs.rs/prebindgen)
- **prebindgen-proc-macro API Reference**: [docs.rs/prebindgen-proc-macro](https://docs.rs/prebindgen-proc-macro)
