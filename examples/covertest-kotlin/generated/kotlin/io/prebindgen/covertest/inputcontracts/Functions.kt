@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.inputcontracts

public fun durationMillis(milliseconds: java.time.Duration, onError: io.prebindgen.covertest.inputcontracts.JniErrorHandler<Long> = io.prebindgen.covertest.inputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.inputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { milliseconds.toMillis() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.inputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.inputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.inputcontracts.JNINative.call_duration_millis_8c33b85e7452f8b5(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.inputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
