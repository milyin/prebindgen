@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package com.prebindgen.paramnames

public fun parameterExternal(org_1c87d37708ffb3b8: Long, com_b3aadd76cd5757cb: Long, callback: com.prebindgen.paramnames.ParameterPointCallback?, onError: org.prebindgen.paramnames.JniErrorHandler<com.prebindgen.paramnames.ParameterPoint> = org.prebindgen.paramnames.JniErrorHandler { message -> throw IllegalArgumentException(message) }): com.prebindgen.paramnames.ParameterPoint = run {
    val returnCapture = org.prebindgen.paramnames.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { org_1c87d37708ffb3b8 } catch (failure: IllegalArgumentException) { throw org.prebindgen.paramnames.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { com_b3aadd76cd5757cb } catch (failure: IllegalArgumentException) { throw org.prebindgen.paramnames.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long) {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: com.prebindgen.paramnames.ParameterPoint
        try {
            callbackArgument0_0 = com.prebindgen.paramnames.ParameterPoint(raw0 as Long, raw1 as Long)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw org.prebindgen.paramnames.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw org.prebindgen.paramnames.HandleValidationFailure(failure.message ?: "invalid input") }; org.prebindgen.paramnames.JNINative.call_parameter_external_12d641fa6bd87fee(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: org.prebindgen.paramnames.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    com.prebindgen.paramnames.ParameterPoint(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}
