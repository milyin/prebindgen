@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.perftest.storage

public fun storageNew(onError: io.prebindgen.perftest.JniErrorHandler<io.prebindgen.perftest.Storage> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.perftest.Storage = run {
    val returnCapture = io.prebindgen.perftest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_storage_new_a15b46920214a065(nativeArgument0) } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun <R> storageGet(s: io.prebindgen.perftest.Storage, onError: io.prebindgen.perftest.JniErrorHandler<R> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long?, Int?, Double?, Boolean?) -> R): R = run {
    val returnCapture = io.prebindgen.perftest.Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.perftest.NativeHandle? = try { s } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.perftest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_storage_get_b38090762950a015(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

build(if (returnCapture.s0 == 1) returnCapture.s1 else null, if (returnCapture.s0 == 1) returnCapture.s2 else null, if (returnCapture.s0 == 1) returnCapture.s3 else null, if (returnCapture.s0 == 1) returnCapture.s4 else null)
    } finally { returnCapture.release() }
}

public fun payloadHandlerNew(f: io.prebindgen.perftest.PayloadCallback, onError: io.prebindgen.perftest.JniErrorHandler<io.prebindgen.perftest.PayloadHandler> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.perftest.PayloadHandler = run {
    val returnCapture = io.prebindgen.perftest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Int, raw2: Double, raw3: Boolean) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument0_1: Int = 0
        var callbackArgument0_2: Double = 0.0
        var callbackArgument0_3: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument0_1 = raw1 as Int
            callbackArgument0_2 = raw2 as Double
            callbackArgument0_3 = raw3 as Boolean
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        f.run(callbackArgument0_0, callbackArgument0_1, callbackArgument0_2, callbackArgument0_3)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_payload_handler_new_8f0187f8ce08f8e1(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.perftest.PayloadHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.perftest.JNINative.drop_PayloadHandler_8f188783a3c63cb5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.perftest.PayloadHandler? = null
        try {
            output0 = io.prebindgen.perftest.PayloadHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.perftest.JNINative.drop_PayloadHandler_8f188783a3c63cb5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun storageCallback(s: io.prebindgen.perftest.Storage, handler: io.prebindgen.perftest.PayloadHandler, onError: io.prebindgen.perftest.JniErrorHandler<Unit> = io.prebindgen.perftest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.perftest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.perftest.NativeHandle? = try { s } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.perftest.NativeHandle? = try { handler } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.perftest.withHandleLocks(__pb_handle0, __pb_handle1) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.perftest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.perftest.JNINative.call_storage_callback_43051aaafb4e1514(nativeArgument0, nativeArgument1, nativeArgument2) } } } } catch (failure: io.prebindgen.perftest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}
