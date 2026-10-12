package io.prebindgen.perftest

/**
 * The single owner of native-library loading. The generated `JNINative` object
 * calls [ensureLoaded] from its static initializer (wired via
 * `.jni_native_init(...)` in build.rs), so the cdylib is loaded before the first
 * `external fun` resolves.
 *
 * The Gradle `run` task puts `target/release` on `java.library.path`, so
 * `System.loadLibrary("perftest_kotlin")` finds `libperftest_kotlin.{dylib,so}`.
 */
internal object NativeLibrary {
    init {
        System.loadLibrary("perftest_kotlin")
    }

    /** No-op trigger: forces this object's `<clinit>` (which loads the library). */
    fun ensureLoaded() {}
}
