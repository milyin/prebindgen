# prebindgen

A Rust proc-macro crate that provides the `#[prebindgen]` attribute macro for copying struct and enum definitions to a file during compilation.

## Features

- Attribute macro that can be applied to structs and enums
- Copies the complete definition to `prebindgen.rs` in the `OUT_DIR` when available
- Falls back to a unique directory in the system temp directory when `OUT_DIR` is not available
- Avoids duplicate definitions in the output file
- Works in both build-time and development contexts

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
prebindgen = "0.1.0"
```

Then use the macro on your structs and enums:

```rust
use prebindgen::prebindgen;

#[prebindgen]
#[derive(Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

#[prebindgen]
#[derive(Debug, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending { reason: String },
}
```

## Accessing Generated Definitions

### During Build Time (with OUT_DIR)

The generated definitions are written to `prebindgen.rs` in your crate's `OUT_DIR`. You can include them in your code using:

```rust
// Include the generated definitions
include!(concat!(env!("OUT_DIR"), "/prebindgen.rs"));
```

### During Development (without OUT_DIR)

When `OUT_DIR` is not available (e.g., during IDE analysis or development), the macro automatically falls back to creating a unique directory in the system temp directory. This ensures the macro works seamlessly in all contexts.

Or in a build script (`build.rs`):

```rust
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let prebindgen_path = Path::new(&out_dir).join("prebindgen.rs");
    
    if prebindgen_path.exists() {
        println!("cargo:rerun-if-changed={}", prebindgen_path.display());
        // Process the generated file as needed
    }
}
```

## How It Works

1. When you apply `#[prebindgen]` to a struct or enum, the macro captures the complete definition
2. During compilation (when `OUT_DIR` is available), it writes the definition to `prebindgen.rs`
3. The macro ensures no duplicate definitions are written to the file
4. The original code remains unchanged and continues to work normally

## Use Cases

- Code generation workflows
- Creating copies of types for external tools
- Build-time type introspection
- Generating bindings or interfaces

## Requirements

- Rust 2021 edition or later
- Available during build time (requires `OUT_DIR` environment variable)

## License

This project is licensed under the MIT License.
