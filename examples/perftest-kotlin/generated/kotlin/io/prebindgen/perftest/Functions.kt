@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.perftest

public fun <R> `payloadNew`(p0_w1: Long, p1_w1: Int, p2_w1: Double, p3_w1: Boolean, onError: io.prebindgen.perftest.JniErrorHandler<R> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, Int, Double, Boolean) -> R): R = run {
    val returnCapture = io.prebindgen.perftest.Return_Long_Int_Double4_ErrorEmpty_f89d68e81a93e976.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p3_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_payload_new_4db541865be00283(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1, returnCapture.s2, returnCapture.s3)
    } finally { returnCapture.release() }
}

public fun `storageWithPayload`(p0_w2: Long, p0_w4: Int, p0_w6: Double, p0_w8: Boolean, onError: io.prebindgen.perftest.JniErrorHandler<io.prebindgen.perftest.Storage> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.perftest.Storage = run {
    val returnCapture = io.prebindgen.perftest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w4 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w6 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w8 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_storage_with_payload_b44f150bf309e573(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.perftest.Storage? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.perftest.JNINative.drop_Storage_9651261943550368(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.perftest.Storage? = null
        try {
            output0 = io.prebindgen.perftest.Storage(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.perftest.JNINative.drop_Storage_9651261943550368(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `storagePutOpt`(p0_w1: io.prebindgen.perftest.Storage?, p1_w1: Boolean, p1_w4: Long, p1_w6: Int, p1_w8: Double, p1_w10: Boolean, onError: io.prebindgen.perftest.JniErrorHandler<Boolean> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.perftest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.perftest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p1_w1) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p1_w1) p1_w4 else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (p1_w1) p1_w6 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { if (p1_w1) p1_w8 else 0.0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { if (p1_w1) p1_w10 else false } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_storage_put_opt_38084ee844ee97a7(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6) }
    } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
