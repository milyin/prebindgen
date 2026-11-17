@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.isolatederror

public fun durationResult(milliseconds: Long, onBindingError: io.prebindgen.covertest.isolatederror.JniErrorHandler<Long> = io.prebindgen.covertest.isolatederror.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.isolatederror.LocalErrorHandler_duration_result_c64b5b1058c5c0d0<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.isolatederror.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { milliseconds } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.isolatederror.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.isolatederror.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.isolatederror.JNINative.call_duration_result_1a1b3c1552bf21b6(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.isolatederror.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(java.time.Duration.ofMillis(returnCapture.d0)) }
    nativeResult
    } finally { returnCapture.release() }
}
