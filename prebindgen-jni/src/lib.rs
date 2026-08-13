//! JNI / Kotlin language adapter — the [`JniGenBuilder`] back-end.
//!
//! Sibling of the C adapter (now the separate `prebindgen-c` crate): it implements the
//! language-agnostic [`prebindgen_registry::Prebindgen`] trait to
//! turn a flat `#[prebindgen]` library into a Rust file of JNI `extern "C"`
//! wrappers plus a fan-out of generated Kotlin sources.
//!
//! Pipeline:
//!   1. [`prebindgen_registry::Registry::builder`] describes a binding over a model built from
//!      `(syn::Item, SourceLocation)` (typically `source.items_all()`).
//!   2. `Registry::write_rust` resolves every
//!      required type via a configured [`JniGenBuilder`] and writes the generated
//!      Rust bindings file.
//!   3. `JniGenBuilder::write_kotlin` walks the resolved registry to emit the
//!      secondary Kotlin artifacts (typed-handle classes, data/enum classes,
//!      exception classes, the centralized `JNINative` holder).
//!
//! # The raw-pointer surface is not part of the consumer API
//!
//! Generated Kotlin passes native pointers around as `Long`s, and generated
//! native code dereferences them. A `Long` a consumer made up therefore has to
//! be unable to reach that code (prebindgen#37):
//!
//! * **Handle constructors are `internal`.** `KeyExpr(0xdeadbeef)` does not
//!   compile outside the generated module — including through a subclass of
//!   `NativeHandle`, whose constructor is `internal` too. Construction is
//!   entirely Kotlin-side, so nothing on the Rust side depends on this.
//! * **`object JNINative`, holding every `external fun`, is `internal`.**
//! * **`NativeHandle.peek()` and the `fromParts` factories are marked
//!   `@UnsafeNativeApi`** — a generated `@RequiresOptIn` annotation class in
//!   the base package. Both are looked up from Rust by JNI reflection
//!   (`call_method` / `call_static_method`), so `internal` would rename them
//!   out from under the lookup; an opt-in marker constrains the Kotlin source
//!   without touching the bytecode signature.
//!
//! Every generated file carries `@file:OptIn(<pkg>.UnsafeNativeApi::class)`:
//! generated code is the trusted producer of these pointers. Consumer code
//! gets no such blanket and must opt in per declaration, which is the point.
//!
//! A base package is required for this: with none configured the marker would
//! land in the root package, which Kotlin cannot import from a subpackage, so
//! it is not emitted at all (the `internal` constructors still are).
//!
//! # Fixed-width unsigned integers
//!
//! JniGenBuilder exposes Rust's fixed-width unsigned scalars without narrowing their
//! domain at the Kotlin boundary:
//!
//! | Rust | Kotlin surface | JNI wire |
//! |------|----------------|----------|
//! | `u8` | `Int` | `jint` |
//! | `u16` | `Int` | `jint` |
//! | `u32` | `Long` | `jlong` |
//! | `u64` | `ULong` | `jlong` / `Long` bit pattern |
//!
//! Inputs for `u8`, `u16`, and `u32` are range-checked and report a
//! [`JniBindingError`] through the generated binding-error handler. `u64`
//! uses Kotlin's bit-preserving `ULong.toLong()` / `Long.toULong()` bridge.
//! These mappings compose through nullable/result outputs, generated data
//! classes, callbacks, const getters, and supported output collections.

pub mod jni;
#[cfg(test)]
mod test_util;
pub(crate) mod util;

pub use jni::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
    decode_byte_array, decode_string, encode_byte_array, encode_string, matching, null_byte_array,
    null_string, CachedIfaceMethod, ClassDecl, ConstDecl, ConvertDecl, ConvertSourceDecl,
    DataClassDecl, Declarations, EnumClassDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl, IgnoreDecl, JniBindingError, JniGen, JniGenBuilder, PackageDecl,
    PtrClassDecl, SealedClassDecl, VariantDecl,
};
// Kotlin emission types now live in the standalone `kotlin-codegen` crate;
// re-exported here so the public `lang::` surface is unchanged (`KotlinFile`
// aliases the model's `KtFile`).
pub use kotlin_codegen::KtFile as KotlinFile;
pub use kotlin_codegen::WriteKotlinError;
