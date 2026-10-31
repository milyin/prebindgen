@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.runtimecosts

public fun <R> `bufferViewsOptionalText`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String?, ULong) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_optional_text_2e5df9089cef5211(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 != null) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 != null) {
            }
            build(if (returnCapture.s0 != null) returnCapture.s0 as String else null, (returnCapture.s1).toULong())
        } catch (failure: Throwable) {
            if (returnCapture.s0 != null) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `bufferViewsOptionalBytes`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (ByteArray?, ULong) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_ByteArray_Long2_ErrorEmpty_34705c584e211183.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_optional_bytes_f3f56b8a8b1996c2(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 != null) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 != null) {
            }
            build(if (returnCapture.s0 != null) returnCapture.s0 as ByteArray else null, (returnCapture.s1).toULong())
        } catch (failure: Throwable) {
            if (returnCapture.s0 != null) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `bufferViewsText`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, ULong) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_text_d3011d2eb33189c1(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0 as String, (returnCapture.s1).toULong())
    } finally { returnCapture.release() }
}

public fun <R> `bufferViewsBytes`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (ByteArray, ULong) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_ByteArray_Long2_ErrorEmpty_34705c584e211183.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_bytes_491539966ab74896(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0 as ByteArray, (returnCapture.s1).toULong())
    } finally { returnCapture.release() }
}
