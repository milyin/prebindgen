@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.resourceViews

public fun <R> `resourceBundleViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ResourceBundle, String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_resource_bundle_views_98686c0f53809c0c(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_g1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = outputg0_g1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_g1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg0_g1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            outputg0_g1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s2)
            outputg0_g2_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s3)
            build(io.prebindgen.covertest.resources.ResourceBundle(returnCapture.s0 as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg0_g1_g0_g0_0), requireNotNull(outputg0_g1_g0_g1_0)), requireNotNull(outputg0_g2_0), io.prebindgen.covertest.values.DataFloats(returnCapture.s4 as ByteArray, returnCapture.s5, returnCapture.s6)), returnCapture.s7 as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_g1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}
