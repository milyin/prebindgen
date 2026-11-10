@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.outputcontracts

public fun durationEcho(milliseconds: Long, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<java.time.Duration> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.time.Duration = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { milliseconds } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_echo_9eff5d3e7eedf8fc(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    java.time.Duration.ofMillis(nativeResult)
    } finally { returnCapture.release() }
}

public fun durationPacket(milliseconds: Long, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<java.time.Duration> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.time.Duration = run {
    val result = try { run { run { val nativeArgument0 = try { milliseconds } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_packet_f309e8fdef6a1a5c(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { java.time.Duration.ofMillis(result[1] as Long) }

        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> durationPair(onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<R> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (java.time.Duration, java.time.Duration) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_pair_7f1ff9c108dc7c4e(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(java.time.Duration.ofMillis(returnCapture.s0), java.time.Duration.ofMillis(returnCapture.s1))
    } finally { returnCapture.release() }
}

public fun durationSeries(onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<kotlin.collections.List<java.time.Duration>> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.time.Duration> = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_series_0147306c7d8df97a(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.outputcontracts.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.outputcontracts.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.outputcontracts.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.outputcontracts.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.outputcontracts.BulkRange?), io.prebindgen.covertest.outputcontracts.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.outputcontracts.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.outputcontracts.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

(requireNotNull(outputbulkList)).map { java.time.Duration.ofMillis(it) }
    } finally { returnCapture.release() }
}

public fun durationOptional(present: Boolean, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<java.time.Duration?> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.time.Duration? = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_optional_cfe965c40fd3f602(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

(if (returnCapture.s0 == 1) returnCapture.s1 else null)?.let { java.time.Duration.ofMillis(it) }
    } finally { returnCapture.release() }
}

public fun durationResult(milliseconds: Long, onBindingError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<java.time.Duration> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.outputcontracts.LocalErrorHandler_duration_result_c64b5b1058c5c0d0<java.time.Duration>): java.time.Duration = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { milliseconds } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_result_1a1b3c1552bf21b6(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(java.time.Duration.ofMillis(returnCapture.d0)) }
    java.time.Duration.ofMillis(nativeResult)
    } finally { returnCapture.release() }
}

public fun plainResult(milliseconds: Long, onBindingError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<Long> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.outputcontracts.i64Handler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { milliseconds } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_plain_result_3f31a43f28b72fff(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    nativeResult
    } finally { returnCapture.release() }
}

public fun durationCallback(callback: io.prebindgen.covertest.outputcontracts.DurationCallback, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<Long> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: java.time.Duration
        lateinit var callbackReturned: java.time.Duration
        try {
            callbackArgument0_0 = java.time.Duration.ofMillis(raw0 as Long)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_duration_wire = callbackReturned
  run { val callbackReturnWire = (callback_duration_wire).toMillis()
val __pb_consume: (io.prebindgen.covertest.outputcontracts.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_callback_90cbff1e8af6c9e8(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun plainCallback(callback: io.prebindgen.covertest.outputcontracts.i64Toi64Callback, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<Long> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.outputcontracts.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_plain_callback_1c7d8c18a52f8e01(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <A> durationSeriesFold(initial: A, onError: io.prebindgen.covertest.outputcontracts.JniErrorHandler<A> = io.prebindgen.covertest.outputcontracts.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, java.time.Duration) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.outputcontracts.JNINative.SequenceCollector_duration_series_fold_e15a6be57122d068()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
val returnCapture = io.prebindgen.covertest.outputcontracts.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.outputcontracts.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.outputcontracts.JNINative.call_duration_series_fold_c232f7a7643f8f2b(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.outputcontracts.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]


current = fold(current, java.time.Duration.ofMillis(__pb_fold_row.c0))
}
__pb_accumulator = current
}

__pb_accumulator as A
    } finally { returnCapture.release() }
}
