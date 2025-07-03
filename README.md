# prebindgen

A Rust utility crate for separating common FFI interface implementation and language-specific binding generation into deifferent crates.

## Problem

In the Rust `#[no_mangle] extern "C"` functions must be defined in `cdylib`/`staticlib` crate. The reexport of them from simple `lib` crate doesn't work. The issue [2771](https://github.com/rust-lang/rfcs/issues/2771) mentioning this problem is still open.

So when creating Rust libraries that need to expose FFI interfaces to multiple languages, you face a dilemma:

- Generate all bindings from the same crate (tight coupling)
- Manually duplicate FFI functions in each language-specific crate (code duplication)

## Solution

`prebindgen` solves this by generating `#[no_mangle] extern "C"` source code from a common Rust library crate. Language-specific binding crates can then include this generated code and pass it to their respective binding generators (cbindgen, csbindgen, etc.).

## How to Use

### 1. In the Common FFI Library Crate

Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:

```rust
use prebindgen_proc_macro::{prebindgen, prebindgen_out_dir};

// Declare a public constant with the path to prebindgen data
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();

#[prebindgen]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[prebindgen]
pub fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}
```

Call `init_prebindgen_out_dir()` in the crate's `build.rs`:

```rust
// build.rs
fn main() {
    prebindgen::init_prebindgen_out_dir();
}
```

### 2. In Language-Specific FFI Binding Crates

Add dependencies to `Cargo.toml`:

```toml
[build-dependencies]
my_common_ffi = { path = "../my_common_ffi" }
prebindgen = "0.1"
cbindgen = "0.27" # for C bindings
```

Generate bindings in `build.rs`:

```rust
fn main() {
    let pb = prebindgen::Builder::new(my_common_ffi::PREBINDGEN_OUT_DIR)
        .edition("2024")
        .build();

    let bindings_file = pb.all().write_to_file("ffi_bindings.rs");
    
    // Generate C headers with cbindgen
    cbindgen::Builder::new()
        .with_crate(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .with_src(&bindings_file)
        .generate()
        .unwrap()
        .write_to_file("include/bindings.h");
}
```

Include the generated FFI code:

```rust
// In your lib.rs
include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
```

## Examples

See the [examples](examples/) directory for complete working examples:

- **example-ffi**: Common FFI library demonstrating prebindgen usage
- **example-cbindgen**: Language-specific binding using cbindgen for C headers

## Documentation

- **API Reference**: See the [docs.rs documentation](https://docs.rs/prebindgen) for complete API details