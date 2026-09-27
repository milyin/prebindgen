//! # prebindgen-c
//!
//! The C language adapter: turns the `#[prebindgen]` items of a flat Rust
//! crate into a Rust file of `extern "C"` wrappers, which
//! [cbindgen](https://github.com/mozilla/cbindgen) then reads to produce the C
//! header.
//!
//! Items are **opt-in**: a build script declares every type and function
//! that crosses, and nothing else is generated.
//!
//! ```rust,ignore
//! use syn::parse_quote as pq;
//!
//! let rust_file = prebindgen_c::Cbindgen::builder()
//!     .source(example_flat::PREBINDGEN_OUT_DIR)
//!     .free_memory_function("example_free")
//!     .mangle_type_name(|base| format!("{base}_t"))
//!     .opaque_ptr(pq!(Calculator))
//!     .function(pq!(calculator_new))
//!     .function(pq!(calculator_get_value)).panic()
//!     .build()?
//!     .write_rust("example_flat.rs")?;
//! ```
//!
//! ## How types cross
//!
//! * **Opaque handle** ([`CbindgenBuilder::opaque_ptr`]): C holds `T *`, from
//!   `Box::into_raw`, and frees it with the generated `<base>_drop`. A handle
//!   taken by value is consumed; `&T` / `&mut T` arrive as `const T *` / `T *`.
//! * **Enum** ([`CbindgenBuilder::enum_type`]): a `#[repr(C)]` mirror. A C
//!   `enum` is an `int` at the ABI, so C can pass a value no variant has, and
//!   Rust must never hold such a value as an enum. An inbound enum therefore
//!   arrives as `MaybeUninit<mirror>`, and its discriminant is checked before
//!   the Rust value is created.
//! * **Tagged union** ([`CbindgenBuilder::tagged_union`]): a `#[repr(C)]` enum
//!   with payloads, which cbindgen renders as a tag and a `union`. The tag is
//!   checked on the way in; `<base>_drop` releases the active arm's memory.
//! * **Data struct** ([`CbindgenBuilder::data_struct`]): a mirror converted
//!   field by field. `String` fields are `malloc`'d `char *`, `bool` fields
//!   are normalised.
//! * **`repr(C)` struct** ([`CbindgenBuilder::repr_c_struct`]): the source
//!   struct's own memory, reinterpreted; passed by pointer, consumed by
//!   moving out and nulling the owned pointer fields.
//! * **Conversion** ([`CbindgenBuilder::convert`]): a type crossing as its
//!   representation, through declared functions or trait impls.
//! * **Callback** ([`CbindgenBuilder::callback`]): an `impl Fn(..)` parameter
//!   arrives as a closure struct `{ context, call, drop }`.
//!
//! Results: a `String` is a `malloc`'d `char *`; `Vec<T>` is a `malloc`'d
//! array plus a `size_t *len` out-parameter; `Option<T>` of a value is a
//! `bool` plus an out-parameter (a nullable pointer for a handle);
//! `Result<T, E>` adds an `char **e` error slot (`E` declared with
//! [`CbindgenBuilder::opaque_error`]) and returns the pointer (null on error)
//! or a `bool` success flag with `T` written to an out-parameter. Memory the
//! layer hands out is released with [`CbindgenBuilder::free_memory_function`].
//!
//! An input that can fail to convert — a null pointer, a bad discriminant —
//! is reported through the error slot of a `Result` function. A function
//! without one must be declared [`CbindgenBuilder::panic`].

mod builder;
mod gen;

use std::path::{Path, PathBuf};

pub use builder::CbindgenBuilder;
use prebindgen_tools::RustFile;
pub use prebindgen_tools::{
    convert, expr, from, fun, into, path, sig, try_from, try_into, ty, Conversion, FnRef, Via,
};

/// A generation failure: a declaration the model cannot satisfy.
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// A generated C binding.
pub struct Cbindgen {
    file: RustFile,
}

impl Cbindgen {
    pub fn builder() -> CbindgenBuilder {
        CbindgenBuilder::new()
    }

    /// The generated Rust source.
    pub fn render(&self) -> String {
        self.file.render()
    }

    /// Write the generated Rust file (a relative path lands in `OUT_DIR`) and
    /// return where it went.
    pub fn write_rust(&self, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        self.file.write(path)
    }
}

#[cfg(test)]
mod tests;
