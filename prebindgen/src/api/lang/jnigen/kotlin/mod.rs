//! Kotlin emission support for back-ends that produce Kotlin output.
//!
//! `file` exposes the shared `KotlinFile` / `WriteKotlinError`
//! types; `type_map` is the internal `KotlinTypeMap` (Rust → Kotlin
//! name lookup) consumed by the JNI back-end's Kotlin emitters. Both modules
//! are `pub(crate)` — Kotlin emission is JniGen-inherent and not
//! exposed at the crate boundary.

pub(crate) mod file;
pub(crate) mod type_map;
