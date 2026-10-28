@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.resourceViews

public fun <R> `resourceBundleViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ResourceBundle, String) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_resource_bundle_views_98686c0f53809c0c() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_g1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg0_g1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            outputg0_g1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(result[3] as Long)
            outputg0_g2_0 = io.prebindgen.covertest.resources.OwnedResource(result[4] as Long)
            build(io.prebindgen.covertest.resources.ResourceBundle(result[1] as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg0_g1_g0_g0_0), requireNotNull(outputg0_g1_g0_g1_0)), requireNotNull(outputg0_g2_0), io.prebindgen.covertest.values.DataFloats(result[5] as ByteArray, result[6] as Float, result[7] as Double)), result[8] as String)
        } catch (failure: Throwable) {
            try { val owner = outputg0_g1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(result[4] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

