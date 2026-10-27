@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.perftest.storage

public fun `storageNew`(onError: io.prebindgen.perftest.JniErrorHandler<io.prebindgen.perftest.Storage> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.perftest.Storage = run {
    val result = try { run { ; io.prebindgen.perftest.JNINative.call0() } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.perftest.Storage? = null
        try {
            output0 = io.prebindgen.perftest.Storage(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.perftest.JNINative.drop0(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `storageGet`(p0_w1: io.prebindgen.perftest.Storage?, onError: io.prebindgen.perftest.JniErrorHandler<R> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long?, Int?, Double?, Boolean?) -> R): R = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.perftest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call1(nativeArgument0) }
    } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            build(if (result[1] == 1) result[2] as Long else null, if (result[1] == 1) result[3] as Int else null, if (result[1] == 1) result[4] as Double else null, if (result[1] == 1) result[5] as Boolean else null)
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

