@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.resources

public fun `projectionPairBorrowed`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_pair_borrowed_bc9032fce77ca56f(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s0)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `projectionPairOwned`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_pair_owned_eda94d58bc123b34(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s0)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `projectionPairCallback`(p0_callback: io.prebindgen.covertest.resources.ProjectionPairCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var callbackOutput0_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.resources.ProjectionPair
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_g0_0 = io.prebindgen.covertest.resources.ProjectionHandle(raw0 as Long)
            callbackOutput0_g1_0 = io.prebindgen.covertest.resources.ProjectionHandle(raw1 as Long)
            callbackArgument0_0 = io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(callbackOutput0_g0_0), requireNotNull(callbackOutput0_g1_0))
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_pair_callback_7a64636be1de6f9f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `projectionPairMixed`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_pair_mixed_833184fb3cea18c0(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s0)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `projectionPairViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ProjectionPair, io.prebindgen.covertest.resources.ProjectionPair) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_pair_views_05197d1569e216c6(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg0_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
             try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg0_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_g0_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s0)
            outputg0_g1_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            outputg1_g0_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s2)
            outputg1_g1_0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s3)
            build(io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_g0_0), requireNotNull(outputg0_g1_0)), io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg1_g0_0), requireNotNull(outputg1_g1_0)))
        } catch (failure: Throwable) {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `projectionClones`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_clones_4cb30df2ddad16fb(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `projectionDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_projection_drops_0b54af22f7d4ea40(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `resourceBundleNew`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ResourceBundle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ResourceBundle = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_resource_bundle_new_4c184c76ecdb9bcd(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            outputg1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s2)
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s3)
            io.prebindgen.covertest.resources.ResourceBundle(returnCapture.s0 as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg1_g0_g0_0), requireNotNull(outputg1_g0_g1_0)), requireNotNull(outputg2_0), io.prebindgen.covertest.values.DataFloats(returnCapture.s4 as ByteArray, returnCapture.s5, returnCapture.s6))
        } catch (failure: Throwable) {
            try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `resourceBundleDefault`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ResourceBundle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ResourceBundle = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_resource_bundle_default_69a6f8678d97e5a4(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            outputg1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s2)
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s3)
            io.prebindgen.covertest.resources.ResourceBundle(returnCapture.s0 as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg1_g0_g0_0), requireNotNull(outputg1_g0_g1_0)), requireNotNull(outputg2_0), io.prebindgen.covertest.values.DataFloats(returnCapture.s4 as ByteArray, returnCapture.s5, returnCapture.s6))
        } catch (failure: Throwable) {
            try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `resourceBundleFields`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, io.prebindgen.covertest.resources.ResourcePair, io.prebindgen.covertest.resources.OwnedResource, io.prebindgen.covertest.values.DataFloats) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_resource_bundle_fields_552abfd813e477ad(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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
            build(returnCapture.s0 as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg0_g1_g0_g0_0), requireNotNull(outputg0_g1_g0_g1_0)), requireNotNull(outputg0_g2_0), io.prebindgen.covertest.values.DataFloats(returnCapture.s4 as ByteArray, returnCapture.s5, returnCapture.s6))
        } catch (failure: Throwable) {
            try { val owner = outputg0_g1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `apiRecordNew`(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_api_record_new_1c66bd6fa63a1f6a(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> `apiRecordViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (`count`: Long, `label`: String) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_String2_ErrorEmpty_1b009765d5d6eed0.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_api_record_views_18103c07542f54c4(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1 as String)
    } finally { returnCapture.release() }
}

public fun `ownedResourceId`(p0_w1: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_owned_resource_id_7691bb5261073433(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `ownedResourceDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_owned_resource_drops_27038bacb19950fb(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun <A> `apiRecordRowsFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, `count`: Long, `label`: String) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_api_record_rows_fold_5bd1116027b7610c { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_api_record_rows_fold_847767faa33a5dce(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long, __pb_fold_row[1] as String)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}
