package io.prebindgen.covertest

/**
 * The single owner of native-library loading. The generated `CovertestNative`
 * object calls [ensureLoaded] from its static initializer (wired via
 * `.jni_native_init(...)` in build.rs), so the cdylib is loaded before the first
 * `external fun` resolves.
 *
 * The Gradle `run` task puts `target/release` on `java.library.path`, so
 * `System.loadLibrary("covertest_kotlin")` finds `libcovertest_kotlin.{dylib,so}`.
 */
internal object NativeLibrary {
    init {
        System.loadLibrary("covertest_kotlin")
    }

    /** No-op trigger: forces this object's `<clinit>` (which loads the library). */
    fun ensureLoaded() {}
}
