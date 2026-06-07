//! Kotlin emission support for back-ends that produce Kotlin output.
//!
//! `file` exposes the shared `KotlinFile` / `WriteKotlinError` types
//! consumed by the JNI back-end's Kotlin emitters. `pub(crate)` — Kotlin
//! emission is JniGen-inherent and not exposed at the crate boundary.

pub(crate) mod file;
