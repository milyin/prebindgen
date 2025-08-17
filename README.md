# prebindgen

A tool for separating the implementation of FFI interfaces from language-specific binding generation, allowing each to reside in different crates.

## Problem

When creating Rust libraries that need to expose FFI interfaces to multiple languages, it may be preferable to create separate `cdylib` or `staticlib` crates for each language-specific binding. This allows you to tailor each crate to the requirements and quirks of its binding generator and to the specifics of the destination language.

However, `#[no_mangle] extern "C"` functions can only be defined in `cdylib` or `staticlib` crates and cannot be exported from `lib` crates. As a result, these functions must be duplicated in each language-specific binding crate. This duplication becomes cumbersome for large projects with many FFI functions and types.

There is a discussion about this problem in issue [2771](https://github.com/rust-lang/rfcs/issues/2771).

## Solution

`prebindgen` solves this by generating `#[no_mangle] extern "C"` functions for each language-specific binding which act as proxies to a common Rust library crate.

It also allows you to convert and analyze the source to adapt the result for specific binding generators and/or to collect data necessary for post-processing the generated language bindings.

The `prebindgen` tool consists of two crates: `prebindgen-proc-macro`, which provides macros for copying code fragments from the source crate, and `prebindgen`, which converts these fragments into an FFI source file.

### Architecture

Each element to export is marked in the source crate with the `#[prebindgen]` macro. When the source crate is compiled, these elements are written to an output directory. The `build.rs` of the destination crate reads these elements and creates FFI-compatible functions and proxy structures for them. The generated source file is included with the `include!()` macro in the dependent crate and parsed by the language binding generator (e.g., cbindgen).

It's important to keep in mind that `[build-dependencies]` and `[dependencies]` are different. The `#[prebindgen]` macro collects sources when compiling the `[build-dependencies]` instance of the source crate. Later, these sources are used to generate proxy calls to the `[dependencies]` instance, which may be built with a different feature set and for a different architecture. A set of assertions is put into the generated code to catch possible divergences, but it's the developer's job to manually resolve these errors.

## Usage

### 1. In the Common FFI Library Crate (e.g., `example-ffi`)

Mark structures and functions that are part of the FFI interface with the `prebindgen` macro and export the prebindgen output directory path:

```rust
// example-ffi/src/lib.rs
use prebindgen_proc_macro::prebindgen;

// Path to prebindgen output directory. The `build.rs` of the destination crate
// reads the collected code from this path.
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_proc_macro::prebindgen_out_dir!();

// Features which the crate is compiled with. This constant is used
// in the generated code to validate that it's compatible with the actual crate
pub const FEATURES: &str = prebindgen_proc_macro::features!();

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

Convert `#[prebindgen]`-marked items to an FFI-compatible API (`repr(C)` structures, `extern "C"` functions, constants). Items that are not valid for FFI will be rejected by `FfiConverter`.

Generate target language bindings based on this source.

Custom filters can be applied if necessary.

```rust
// example-cbindgen/build.rs
use itertools::Itertools;

fn main() {
    // Create a source from the common FFI crate's prebindgen data
    let source = prebindgen::Source::new(example_ffi::PREBINDGEN_OUT_DIR);

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
