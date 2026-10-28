@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.runtimecosts

public fun <R> `bufferViewsOptionalText`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String?, ULong) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_optional_text_2e5df9089cef5211(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            build(if (result[1] != null) result[1] as String else null, (result[2] as Long).toULong())
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `bufferViewsOptionalBytes`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (ByteArray?, ULong) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_buffer_views_optional_bytes_f3f56b8a8b1996c2(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            build(if (result[1] != null) result[1] as ByteArray else null, (result[2] as Long).toULong())
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `bufferViewsText`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, ULong) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_buffer_views_text_d3011d2eb33189c1() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as String, (result[2] as Long).toULong()) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `bufferViewsBytes`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (ByteArray, ULong) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call_buffer_views_bytes_491539966ab74896() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as ByteArray, (result[2] as Long).toULong()) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

